#![recursion_limit = "512"]

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
mod void_lens;
mod banners;
mod bestiary;
mod bestiary_data;
mod bestiary_filter;
mod bestiary_sorting_ids;
mod bestiary_tags;
mod biome;
mod boss_bags;
mod fishing_crates;
mod misc_bags;
mod npc_death_heals;
mod presents;
mod buff_effects;
mod buff_effects_frame;
mod buff_effects_misc;
mod buff_effects_summons;
mod buff_effects_orbs;
mod buff_effects_potions_sets;
mod buff_effects_status;
mod buffs;
mod buff_effects_accessory;
mod buff_effects_boosters;
mod buff_effects_tail;
mod buff_frame_wiring;
mod buff_inferno_aura;
mod bug_net;
mod chat;
mod chest;
mod cloud;
mod collision;
mod combat_text;
#[cfg(feature = "debug")]
mod debug_panel;
mod death;
mod drawdata_cases_a;
mod drawdata_cases_b;
mod dust_lights;
mod dust_lights_b;
mod dye;
mod emotes;
mod env;
mod entity_lights;
mod item_lights;
mod npc_lights;
mod npc_lights_c;
mod npc_lights_a;
mod npc_lights_b;
mod player_lights;
mod player_lights_b;
mod proj_ai_lights;
mod proj_lights;
mod proj_lights_a;
mod tile_lights;
mod eoc_fight;
mod input_map;
mod input_profiles;
mod inv;
mod house_banners;
mod loadout;
mod luck;
mod torch_god;
mod projectile_ai_torch_god;
// W4-f33 — the Torch God's Favor (5043) permanent-upgrade use path
// (ItemCheck_UseTorchGodsFavor, Player.cs:45226-45243), wired at the
// ItemCheck_OwnerOnlyCode permanent-item seam (:43600).
mod torch_god_favor_use;
mod wiring_stub;
mod yoyo_string;
// W4-d16 — the kite string + tail geometry (`Main.DrawKite`,
// `Main.cs:37134-37569`): the pure-math half of the aiStyle-160 draw.
mod kite_string;
mod item;
mod item_dust;
mod itemdata;
mod item_sorting;
mod quick_stack;
mod sortdata;
mod lang;
mod lang_tables;
mod loot_db;
mod loot_db_gen;
mod map;
mod map_layers;
mod map_npc_icons;
mod map_player_icons;
mod map_sections;
// W4-g11: the Main.cs `Dust.NewDust*` sites, batch 1 of 3 — `snowing()`
// (Main.cs:13088) + the `TeleportEffect` App-level dispatcher/seams,
// per the claim doc (docs/research/64-dust-main-sites-claims.md) and the
// union-exec (docs/research/116-teleport-reconcile.md).
mod main_dust;
// W4-g11 union-exec (research/116 keep-one): the surviving
// `Main.TeleportEffect` body — g61's widest-11 arms adapted per-hunk +
// the wired `NPC.UpdateNPC_TeleportVisuals` decay.
mod teleport_dust;
// W4-g23: the Main.cs PlaySound census batch 3 (sites 181-271 of 271,
// :49734-:66248) — the DrawMenu inline-cue census (the rows are landed, the
// per-site cue seam is g22's in-flight Menu::pending_cue) + the tail sites'
// pure cue-decision halves (colour/volume hover ticks, star-game hit pair,
// click-pan toggle, unity-map hover, fullscreen-map close, NewText tail,
// SelectPlayer/password/IP arms). Code-only — witnesses written not run.
mod main_sounds_batch3;
mod melee_swing;
mod menu;
mod minecart;
mod main_dust_b;
mod mount;
mod mount_dust;
mod net;
mod npc;
mod npc_scaling;
mod npc_glow;
mod npc_chat;
mod npc_dust;
mod npc_pet_exchange;
mod player_petting;
mod party_hat;
mod event_skies;
mod npc_death_dust;
mod npc_names;
mod npc_spatial;
mod revenge;
mod npc_ai;
mod npc_ai_prime_limbs;
mod npc_ai_style64;
mod npc_ai_style65;
// W4-f18 first-20 port: the Butterfly 356's AI_065 machine (NPC.cs:
// 44896-45119) — the style-axis 65 arm claims it (npc.rs).
// W4-f18 batch 2: the town-CRITTER path of AI_007 (NPC.cs:52965-55561,
// the TownCritter subset) — the Frog 361 / Duck 362 / DuckWhite 364 ride
// the style-axis 7 arm (npc.rs); the full town machine stays unwired.
mod npc_ai_town_critter;
// W4-f18 first-20 port: the stat witnesses for the twenty kinds 355-380
// + 430 (the census doc's first-20 list). WRITTEN NOT RUN (code-only).
mod npc_w4_f18_witnesses;
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
// W4-d22 (NPC aiStyle 3): the shared AI_003_Fighters core — the
// walk/jump/fall/knockback/door-bash phases for the plain-walker family.
mod npc_ai_style3;
mod npc_ai_drippler;
mod npc_ai_dungeon_shooter;
// W4-c9: the gold town-critters' AI_007 wander/hop subset (GoldBunny 443 /
// GoldFrog 445 / GoldMouse 447) - the TownCritter walk core, the housing/
// danger states deferred (see the module doc).
mod npc_ai_critter;
mod npc_ai_ghost;
mod npc_ai_blood_squid;
mod npc_ai_poltergeist;
mod npc_ai_pumpkin_boss;
mod npc_ai_unicorn;
// W4-c40 — the Sand Elemental's aiStyle-102 machine (UNWIRED, the d41
// roster wave owns the kind row; the style73 precedent).
mod npc_ai_sand_elemental;
mod npc_ai_style10;
mod npc_ai_style14;
mod npc_ai_style16;
mod npc_ai_style17;
mod npc_ai_style18;
mod npc_ai_style19;
mod npc_ai_style20;
mod npc_ai_style24;
mod npc_ai_style28;
mod npc_ai_style29;
mod npc_ai_style33;
mod npc_ai_style37;
mod npc_ai_style39;
mod npc_ai_style68;
// W4-e26: the aiStyle-115 ladybug flight (AI_115_LadyBugs, kinds 604/605).
mod npc_ai_style115;
mod npc_ai_unicorn_b;
mod npc_ai_style27;
mod npc_ai_fishron;
// S1-D-3 empress: the Empress of Light machine (aiStyle 120, kind 636).
// WIRED by W4-d125: the NpcKind row + the aiStyle-120 axis arm +
// the request drains (see the module doc + docs/ANALYSIS.md "Empress of
// Light's machine" / "The empress wiring (W4-d125)").
mod npc_ai_empress;
// W4-d125: the Prismatic Lacewing (661) — AI_065's 661 branch, the
// release gate + the death-spawn chain (docs/ANALYSIS.md, same entry).
mod npc_ai_lacewing;
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
// W4-c10 roster tail — the d30 butterfly module (aiStyle 65, the
// GoldButterfly 444 wiring) + the d38 TownCritter wander/hop subset
// (the GoldBunny/GoldFrog/GoldMouse 443/445/447).
mod npc_ai_style72;
mod npc_ai_style73;
mod npc_ai_style74;
mod npc_ai_style75;
// W4-d31 (research/25 row 83): UNWIRED cultist-ritual family (the
// CultistTablet 437 / CultistDevote 438, NPC.cs:36757-36925) - the kind
// rows + the style-axis arm are the roster wave's merge step (the
// npc_ai_style73 precedent; see the module doc).
// W4-d31 (research/25 row 83): the cultist-ritual family (the
// CultistTablet 437 / CultistDevote 438, NPC.cs:36757-36925) - WIRED on
// the style axis (the d10 activation contract: npc.rs's 83 arm + the
// world_sim pooled branch + the ritual link/spawn/526-owner-re-pin
// passes); the 437/438 kinds ride the roster wave (W4-d40 7fec14aa), so
// the dispatch is dormant until they merge (see the module doc).
mod npc_ai_style83;
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
// W4-d32 style 92: UNWIRED Target Dummy machine (NPC.cs:38810-38857) -
// the 488 kind + the axis arm wait on the roster wave (the style86
// precedent; see the module doc).
mod npc_ai_style92;
// W4-d33 (research/25 rows 99-127): the MISSING aiStyle band's unwired
// pure machines, one module per style (the npc_ai_style73 precedent).
mod npc_ai_style99;
mod npc_ai_style100;
mod npc_ai_style101;
mod npc_ai_style102;
mod npc_ai_style103;
mod npc_ai_style104;
mod npc_ai_style111;
mod npc_ai_style112;
mod npc_ai_style113;
mod npc_ai_style114;
mod npc_ai_style116;
mod npc_ai_style117;
mod npc_ai_style118;
mod npc_ai_style119;
mod npc_ai_style121;
mod npc_ai_style122;
mod npc_ai_style125;
mod npc_ai_style126;
mod npc_ai_style127;
mod npc_ai_wyvern;
mod npc_ai_style9;
// W4-d23 (aiStyle-13): UNWIRED man-eater family (43/56/101/175/259/260,
// NPC.cs:22615-22847) - the six kinds + the axis claim are the wiring
// step (the npc_ai_style17/20 precedent).
mod npc_ai_style13;
// W4-d36 E4 item 4: UNWIRED aiStyle-5 tail arms (the Moth wet buoyancy
// NPC.cs:50726-50738 + the Hornet family's stinger volley :50739-50812) -
// the wiring point (ai_005_hover_family's rng thread) is the follow-up
// edit; the projectile-side infrastructure (STINGER 55 row + rider +
// timed-shot list) IS live (see the module doc + docs/research/110).
mod npc_ai_style5;
// W4-f2 closeout (research/94 §10) — the tail census's two UNCLAIMED
// sites' pure staged decisions (the Medusa gaze + the BigMimic reflect).
// The merge fold (research/94 §4/§15): e9's npc_water_collision module
// DROPPED — f2's water_collision_sounds pass is the superset keeper.
mod npc_sounds_tail;
mod npc_ai_style30;
mod npc_ai_style31;
mod npc_ai_style32;
mod npc_ai_style40;
mod npc_ai_style47;
mod npc_ai_bat;
mod npc_ai_eye;
mod npc_ai_golem;
mod npc_ai_slime;
// S1-D-3 cultist / W4-c1: WIRED Lunatic Cultist machine (AI_084, 439) -
// the aiStyle-84 axis arm (npc.rs::ai_084_cultist) claims the dispatch;
// the old update_lunatic_cultist stub is retired. The 440 clone has no
// port kind (solo path); states 4/7/8 deferred, see the module doc.
mod npc_ai_cultist;
// S1-D-3 (B-6a): UNWIRED pillar machine (aiStyle 94) - the dispatch swap
// + the checkDead intercept are the wiring wave; R72 wires the 629
// shield-chip seams (see the module doc).
mod npc_ai_pillar;
mod npc_ai_style71;
mod npc_target;
// W4-c10 sweep — NPC.TransformVisuals (NPC.cs:80963-81033): the
// transform-visuals draw census + the Item8 cue intent; the eight
// waterfowl pairs are VISUALLY SILENT (the census verdict, witnessed).
mod npc_transform_visuals;
// W4-d55 (gore table wave A): UNWIRED NPC.HitEffect per-NPC gore table —
// the first 30 death sites (NPC.cs:81660-81941); the StrikeNPC_Inner
// dispatch claim is the wiring wave (the npc_ai_style39 precedent).
mod npc_hit_effect_a;
// W4-g62 — the NPC HIT dust lane: NPC.HitEffect's LIVE per-kind dust
// arms (NPC.cs:81654-89834) reached from StrikeNPC_Inner's :81488 tail;
// the strike captures a HitDustSnapshot, update_npcs drains it on the
// UpdateNPCs stream through the verbatim DustPool::new_dust (the death
// halves are g59/g60's, the spawn dust g61's, the sounds c13's).
mod npc_hit_dust;
mod npc_ai_style21;
mod npc_ai_style22;
mod npc_ai_style23;
mod npc_ai_style25;
mod npc_ai_style26;
mod npc_buffs;
// W4-g78 DOC-SWEEP lane 4: NPC.GetNPCColorTintedByBuffs (NPC.cs:91024-91141)
// + buffColor (:90986-90993) - the debuff tint chain whose last arm is the
// Hunter-potion floor. PORTED-UNWIRED: the render/entities.rs consumer is
// contested by the npc-kind/style lanes; the friendly predicate stays
// f47's shimmer::appears_friendly_to_hunter_potion (keep-one, research/76).
mod npc_hunter_tint;
mod npc_findframe_a;
mod npc_findframe_b;
mod npc_findframe_mid2;
mod npc_findframe_c;
mod npc_findframe_high2;
// W4-g10 lane 3: the town pets' FindFrame arm (the humanoid-group body
// with the pet overrides, NPC.cs:68622-69655).
mod npc_findframe_pet;
mod npc_findframe_extra2;
mod npc_findframe_extra3;
mod npc_findframe_extra5;
mod npc_findframe_extra6;
mod npc_findframe_extra7;
mod npc_findframe_extra7b;
mod npc_findframe_extra7c;
mod npc_findframe_extra8;
mod npc_findframe_low1;
mod npc_findframe_low2;
mod npc_findframe_mid1;
mod npc_findframe_extra1;
mod npc_findframe_extra4;
mod npc_findframe_extra9;
mod npc_findframe_extra10;
mod npc_findframe_extra11;
mod npc_findframe_extra12;
mod npc_findframe_extra13;
mod npc_findframe_extra7d;
mod npc_findframe_extra7e;
mod npc_findframe_extra7f;
mod npc_findframe_extra7g;
mod npc_findframe_extra7h;
mod npc_findframe_extra7i;
mod npc_findframe_extra7j;
mod npc_findframe_extra7k;
// W4-f4 wave E: the 17 unclaimed out-of-HitEffect gore sites (research/211)
// — UNWIRED standalone machines, the hosts' wiring is the follow-up round.
mod npc_hit_effect_e;
// W4-g10 lane 3: the town pets' FindFrame arm (the humanoid-group body
// with the pet overrides, NPC.cs:68622-69655).
// W4-g6 wiring lane 1 — the FIRST-half unwired findframe waves'
// modules (the C#-slot order: extra2 84-181, mid2 166-180, extra3
// 182-219; the SECOND half 233+ rides W4-g7).
mod persist;
mod grapple;
mod gore_seam;
mod creative;
mod creative_sacrifices;
mod hair_window;
// W4-d122 shop currency: the Defender Medal wallet (CustomCurrencyManager/
// CustomCurrencySingleCoin) — the Tavernkeep's medal rows buy through it.
mod custom_currency;
mod happiness;
mod hotbar;
mod hurt_modifiers;
// S1-P-i10: the melee on-hit debuff infliction (Player.StatusToNPC,
// Player.cs:6502-6617) — wired at world_sim/editing.rs's pre-strike site.
mod melee_on_hit;
mod player;
// W4-d115 lane 19: the dodge family's DRAW maths (LegacyPlayerRenderer
// :433-467 afterimages + PlayerDrawLayers :2954-3024 shield pulses) —
// UNWIRED pure functions; the entities.rs draw claims are the follow-up
// round (the W4-g17 player_gore_c precedent; dust side g17-owned).
mod player_dodge_draw;
// W4-e1: the Player.cs `Dust.NewDust` sites' SECOND batch (the first 26 are
// W4-d51's `player_dust.rs`). The tick queues the box/velocity snapshot at the
// C# read moment; this module replays each site on the `UpdatePlayers` stream.
// Ledger: docs/research/64-dust-sites-player-2.md.
mod player_dust2;
mod player_file;
mod popup_text;
mod potions;
// W4-a6: the permanent power-up consumables — ItemCheck_UseLifeCrystal /
// UseManaCrystal (Player.cs:45274-45320) + the sibling Use* arms
// (:45192-45423). Wired into update_editing beside the boss-spawner block.
mod powerups;
// W4-g88: the Life Fruit permanent upgrade (Player.cs:45290-45304) — wired
// at world_sim/editing.rs's permanent-item seam (the :43596 slot).
mod life_fruit;
// W4-g88: the Life/Mana Crystal permanent upgrades (Player.cs:45306-45320
// / :45274-45288) — the :43595 / :43597 slots around the fruit.
mod crystals;
mod prefix;
pub(crate) mod prefix_id;
pub(crate) mod projectile_id;
pub(crate) mod id_tables;
pub(crate) mod world_events;
mod proj_trail_dust;
mod projectile;
mod debug_options;
// W4-d53: the projectile-tick `Dust.NewDust` drain lanes — the per-tick arrow
// trail (Projectile.cs:17036-17077), the stationary Vortex-bolt trail
// (:17117-17125) and the enchantment-visuals lane (:17290-17467). The tick
// snapshots the box/velocity/owner flags, this module replays on the
// UpdateProjectiles stream (the player_dust / pending_proj_sounds convention).
mod projectile_dust;
mod projectile_ai001_b;
mod projectile_ai_dd2_shots;
mod projectile_ai_rc_drone;
mod projectile_ai_frostmoon;
mod projectile_ai_guns;
mod projectile_gun_splash;
mod projectile_ai_moonlord;
// W4-d9 — the five missing aiStyle 81-90 projectile arms (census: 82/83/84/
// 85/88 were armed on main). One module per style; the rows land in
// projectile/data.rs, the dispatch arms + on-hit/damage seams in
// projectile.rs. Code-only mode — witness tests written not run.
mod projectile_ai_style81;
mod projectile_ai_style86;
mod projectile_ai_style87;
mod projectile_ai_style89;
mod projectile_ai_style90;
// W4-e7 lane 23: the Snapping Stone trio's strike machine
// (Projectile.TryMakingSnappingStoneStrike, Projectile.cs:14283-14395) +
// the ParticleOrchestrator draw census — wired at the melee hit
// (world_sim/editing.rs), the PVE walk (projectile.rs::damage_pve) and the
// Update_SnappingStone zero-crossing (world_sim/player.rs).
mod snapping_stone;


mod projectile_ai001_a;
mod special_arrows;
mod projectile_frost_shots;
mod projectile_ai001_c;
// W4-f20 slice D — the AI_001 per-type arms for the 7 rowed aiStyle-1
// kinds the S1-D-5 slices never covered (323/357/485/661/710/719/819,
// Projectile.cs:54002-57798); census research/167 F6/F7. Code-only mode —
// witness tests written not run.
mod projectile_ai001_d;
// W4-f20 slice E — the AI_001 per-type arms for the aiStyle-1 kinds the
// round-b row branches rowed but left on the generic ai_arrow (435/576/
// 709, Projectile.cs:54002-57794); census research/26f. ROW-GATED INERT
// until c11/d11/d15 merge (the d7 unwired precedent). Code-only mode —
// witness tests written not run.
mod projectile_ai001_e;
// W4-f20 F7 — the Harpy-charm chain: the ai[2]-bits producer
// (Player.Update_HarpyCharm, Player.cs:28436-28490) + ApplyHarpyCharm
// (Projectile.cs:57796-57824) + the AI_001 charm tail (:57769-57793);
// census research/26f F7. Code-only mode — witness tests written not run.
mod projectile_harpy_charm;
mod projectile_ai067_b;
// W4-d13 — the aiStyle-124 pet family (650/882/888/895/896/898/957): the
// keep-alive pins, the home-spot ladder, the flight/idle frame machines,
// the 957 ground bounce and the 650 ore scan + steering tail
// (`Projectile.cs:34970-35696`). Code-only — witnesses written not run.
mod projectile_ai_pets124;
// W4-d13 — the aiStyle-125 Wire Kite (651): the channel mouse-follow, the
// three kill gates and the owner tail (`Projectile.cs:35697-35833`).
// Code-only — witnesses written not run.
mod projectile_ai_wire_kite;
// W4-d13 — the aiStyle-126 column family (654 geyser / 670 ogre stomp): the
// anchor scan, the column-height growth, the stomp's tick-12 child split and
// the dust RNG census (`Projectile.cs:35834-35997`). Code-only — witnesses
// written not run.
mod projectile_ai_geyser;
// W4-d13 — the aiStyle-127 sandnado column (656 friendly / 657 hostile): the
// life cap, the 656 fade + owner sight test, the vertical-expand box and the
// tail dust census (`Projectile.cs:35998-36085`). Code-only — witnesses
// written not run.
mod projectile_ai_sandnado;
// W4-d13 — the aiStyle-128 sandnado mark (658): the spin/fade/scale machine,
// the tick-60 hostile-sandnado spawn and the dust census + Kill burst
// (`Projectile.cs:36086-36197`, `:72374-72393`). Code-only — witnesses
// written not run.
mod projectile_ai_sandnado_mark;
mod projectile_ai026_a;
mod projectile_ai026_b;
mod projectile_ai026_c;
mod projectile_ai_misc;
// W4-d76/s1-e-21h: the kill-hook batch the merged projectile body calls
// (`crystal_pulse*`, `charged_blaster_orb*`, `ice_block_kill_draws`).
mod projectile_kill_hooks_a;
// W4-d5: UNWIRED projectile aiStyle machines (research/156, was 26d) — the rows +
// dispatch arms are the wiring round (the npc_ai_style25 precedent).
mod projectile_ai_style41;
mod projectile_ai_style42;
mod projectile_ai_style43;
mod projectile_ai_style46;
mod projectile_ai_style49;
// W4-d7: UNWIRED projectile aiStyle machines (research/26e) — the rows +
// dispatch arms are the wiring round (the d5 / npc_ai_style25 precedent).
mod projectile_ai_style63;
mod projectile_ai_style69;
// W4-d11 — the aiStyle 101-110 census slice (the tower orbs, the spore
// sac/gas, the nebula sphere / djinn curse, the ritual markers' 813 arm,
// the mechanic wrench and the nurse syringe). Code-only round — the
// witness tests are WRITTEN NOT RUN.
mod projectile_ai_101_110;
// W4-d12: the aiStyle-117 explosion family (612/624/953/1044 + the 608
// Solar Counter's Kill-path burst). The shared arm's scale/age/frame
// machine + the tick-1 resized-box Damage + the three dust censuses; the
// dispatch claim + the 608 death-hook drain are wired in projectile.rs.
// Code-only mode — witness tests written not run.
mod projectile_ai_explosions;
mod projectile_ai_dd2;
// W4-d19 — the aiStyles 182-186/190 census slice: the Zenith illusions
// (182), the Zoologist strikes (183), the Torch God's fireball (184), the
// Soul Drain's mana latch (185), the Resonance Scepter's blast (186) and
// the seven big-sword slashes (190). The rows land in projectile/data.rs;
// the dispatch arms are wired in projectile.rs::update. Code-only mode —
// witness tests written not run.
mod projectile_ai_sword_styles;
// W4-d19 follow-up — the aiStyle-92 cloud family (the 511-513 toxic
// clouds, the 1007 gas trap, the 1128 Chlorophyte Claymore spore child)
// + the 1126 spore machine's 1128/1129 child drain. The rows land in
// projectile/data.rs; the dispatch arm + the drain are wired in
// projectile.rs::update. The 1129 row/arm are W4-d21's claim (the
// documented cross-branch contract). Code-only — witnesses written not run.
mod projectile_ai_clouds;
// W4-d21 — the projectile aiStyles 201-210 missing machines (201/202/203/
// 204/205/208/209; 206 landed a9 8c4fded8 verify-only, 207 empty/no kinds,
// 210 already ai_yoyo_shot). The rows land in projectile/data.rs; the
// dispatch arms are wired in projectile.rs::update. Code-only mode —
// witness tests written not run.
mod projectile_ai_201_210;
// W4-d21 folds 3+4 — the MultiPointHitbox + LightningGenerator port (the
// 203 storm-lightning bolt paths: tile truncation + the multi-segment
// damage hitbox; the census proved the generator is NOT render-only).
mod lightning_generator;
// W4-d7: UNWIRED projectile aiStyle machines (research/167) — the rows +
// W4-d10 — the aiStyles 91-100 census port: the nine missing styles +
// the aiStyle-99 counterweight sub-arm (f19 F1). The rows land in
// projectile/data.rs; the dispatch arms are wired in projectile.rs::update.
// Code-only mode — witness tests written not run.
mod projectile_ai_91_100;
// S1-W4-g81 residuals - the Wire Kite's aiStyle-125 placement machine
// (`Projectile.cs:35697-35833` + Kill's 651 arm `:72051-72067`).
mod projectile_ai_kite;
// W4-c39 exec (research/114 §1c): the duplicate `projectile_ai166` module
// RETIRED — d17's in-file `ai_dove`/`dove_movement`/`dove_bounce` in
// projectile.rs is canonical (the pool's module convention). c39's live
// entity-light feed stays grafted in app/frame/draw.rs.
// S1-P-i42 — the torch/glowstick/flare light-item projectile family: the
// aiStyle-14 glowsticks (50/53/473/515/870/1089) + the aiStyle-33 flares
// (163/310/1008-1011) + the fairy's aiStyle-170 hover. The rows land in
// projectile/data.rs; the dispatch arms (14-gate + 33) are wired in
// projectile.rs::update. Code-only mode — witness tests written not run.
mod projectile_ai_light_items;
// W4-d2 — the aiStyle-11 light pets (the Shadow Orb 18 + the fairies
// 72/86/87): the fairy head, the affirmative keep-alive pin and the
// owner-center seek (`Projectile.cs:24371-24500`). The rows land in
// projectile/data.rs; the dispatch arm (11) is wired in
// projectile.rs::update. Code-only mode — witnesses written not run.
mod projectile_ai_light_pets;
// W4-d2 — the aiStyle-13 chain weapons (the Harpoon 23, Golem Fist 262,
// Boxing Glove 271, Chain Knife 273, Chain Guillotine 481): the outbound
// flight + per-type distance flips, the return seek + catch, the 262
// hitbox burst, the itemRotation tail and the shared blocked-move return
// (`Projectile.cs:24636-24829` + `:19512-19571`). The on-hit return +
// StatusNPC debuffs ride projectile.rs::damage_pve. Code-only mode.
mod projectile_ai_chain;
// W4-d2 — the aiStyle-17 tombstones (43/201-205/527-531): the grounded
// friction/spin/gravity, the FTW dead-roll disarm, the local-owner
// tile-85 placement (CanPlace/Place + the Sign epitaph + Kill) and the
// ×−0.75/×−0.7 bounce (`Projectile.cs:25316-25366` + `:19310-19320`).
// The death spawn (DropTombstone's launch draws) is wired in death.rs.
// Code-only mode.
mod projectile_ai_tombstone;
// W4-g78 DOC-SWEEP lane 4: the SpelunkerProjectileHelper ore-spot dust
// helper (SpelunkerProjectileHelper.cs:6-67) - PORTED-UNWIRED, the 473
// glowstick head's AddSpotToCheck call + the PreUpdateAllProjectiles
// cadence are the wiring lane's merge step (research/76).
mod spelunker_helper;
// S1-P-i55 golf: the two golf projectile AIs (721/739-752 aiStyle 149,
// 722 aiStyle 150) + the club strike / tee placement / cup scoring,
// reusing tr-core's golf.rs + golf_state.rs (code-only round — the
// witness tests are WRITTEN NOT RUN).
mod projectile_ai_golf;
// W4-f42 sandstorm second half: the Desert Djinn's curse (596, aiStyle
// 107) — the orbit/homing AI + the Kill burst census. The row lands in
// projectile/data.rs; the dispatch arm (107) + the kill hook are wired
// in projectile.rs::update. Code-only mode — witnesses written not run.
mod projectile_ai_djinn;
// W4-f42 sandstorm second half: the desert debris + family AI (the
// Vulture 61 aiStyle 17, the Antlion 69 aiStyle 19, the FlyingAntlion 581
// aiStyle 44; the mummies + WalkingAntlion ride update_zombie, the
// Tumbleweed 546 rides unicorn_step). Code-only — witnesses written not run.
mod npc_ai_desert;
// S1-P-i16: the dart guns' projectile AIs (267/477/478/479/442 aiStyle 1,
// 443 aiStyle 77, 167-170 aiStyle 34, the 480 flame drop's dust census).
mod projectile_ai_darts;
// W4-c10: the BirdDroppings 1080's aiStyle-200 arm (AI_200_BirdDroppings,
// Projectile.cs:39985-40017 + the HandleMovement :18921-18928 splat) —
// the aiStyle-24/68 birds' poop shot.
mod projectile_bird_droppings;
// W4-d1: the aiStyle-6 powder/fertilizer cloud (10/11/463/1019/1051 —
// Projectile.cs:24100-24228): the 0.95 decay + 180-tick kill, the one-shot
// dust burst (the 1051's 50 Next(3) draws stay), the owner-gated Convert
// tile scan (types 11/1/4) + the fertilizer sapling growth (genRand defers
// to the pass). The rows + the dispatch arm are wired in projectile.rs.
mod projectile_ai_powder;
// W4-d1 wave 3: the aiStyle-10 falling-block family (AI_010,
// Projectile.cs:57826-58100 — the sandfall kinds 31/39/40/56/67/71/179/241/
// 812/411-414 + the DirtBall 17 + the player Mud Ball 1081 + the sandgun
// balls 42/65/68/354, closing the S1-D-5 "42-row gap"). The dust-gate
// draws stay, the channel aim-steer + gravity ladder + tile walk + the
// HandleMovement blocked arms + the Kill_ExplodeTiles death placement
// (item drops ride pending_block_drops → UpdateItems). W4-d1 retire: the
// natural-fall spawner rides the data path (the frozen-tail carve-out is
// retired — every served kind runs the arm + the death hook).
mod projectile_ai_falling;
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
// W4-g64 (the d13 machine port, dust-wired) — the aiStyle-124 pet machine
// (`Projectile.cs:34970-35696`): the home-spot ladder, the frame machines,
// the 957 ground-bounce column scan and the 650 ore scan + steering tail;
// the 895/898 flight sparkles + the 650's 204 pair ride proj_trail_dust.
// S1-D-4 empress proj: the Empress of Light's five projectile AIs
// (872/873/874/919/923 — AI_173/171/none/179/180), pure per-kind fns behind
// a kind gate, UNWIRED spawner (code-only mode). R7's npc_ai_empress.rs
// remainder; the dispatch claim + the R7 request drain are the wiring
// follow-up (module doc + docs/ANALYSIS.md "S1-D-4 empress proj").
mod projectile_ai_empress;
// S1-P-i35: the fishing bobber's AI_061 flight/bite machine + the FishingCheck
// orchestration (the pure roll engine is tr_core::fishing).
mod projectile_bobber;
// W4-d3: the aiStyle-25 boulder family (`AI()`'s inline block
// Projectile.cs:26015-26415 + HandleMovement's bounce ladder :18941-19277 +
// the RainbowBoulder machine :20024-20315 + the Shimmer arm :22504-22515 +
// the Kill hooks). The 13 kinds' rows land in projectile/data.rs; the
// dispatch claim, the death-hook drain and the statue wiring are wired in
// projectile.rs / world_sim (module doc + docs/ANALYSIS.md "The boulder
// family (W4-d3)").
mod projectile_ai_boulder;
// W4-d4 × W4-g82 union (research/26e): the rope-coil lane's EXTRAS — the
// W4-d4 × W4-g82 union (research/169): the rope-coil lane's EXTRAS — the
// Kill() refund ladder (:77572-77615, WIRED at the pool's death hook + the
// world-sim drain) and the trailing-chain placement preview
// (Main.cs:28499-28669, the UNWIRED draw half — the g82b lane's contract).
// d4's ai_rope_coil stays the single aiStyle-35 machine.
mod projectile_rope_coil;
// W4-d53 batch 4 — the generic Update wet machine (Projectile.cs:16786-17007):
// the wetCount cooldown, the liquid-flag latching, the entry/exit splash edges
// (the 8 census sites :16851-16987 ride projectile_dust's WetSplash drain).
mod projectile_wet;
// S1-P-i06 — the spear family (aiStyle 19, `AI_019_Spears`): the held-spear
// thrust arm + its extension/retract curve and reach-ahead hitbox, plus the
// small sub-shot arms (the spore/cloud/bolt/snowflake/flake the spears spawn).
mod projectile_ai_spear;
// W4-g63 — the projectile TRAIL dust lane 1 (kinds 1-400): the per-kind
// trailing/ambient dust arms inside Projectile.AI(), re-armed onto the live
// DustPool on the UpdateProjectiles stream (research/66; g64 owns 401+, g65
// the Kill death dust).
mod projectile_trail_dust;
// W4-d16 — the kite family (aiStyle 160, `AI_160_Kites` + `KiteLogic`):
// the 24 DefaultToKite kinds' wind hover + anchor reel + frame machine.
mod projectile_ai_kites;
mod rain;
// W4-g70 lane 8: the live star field (Star.cs) — spawn/update/NightSetup/
// StarFall, replacing the static BG-11/16 stand-in.
mod star_field;
// W4-g82 — the rope-coil lane: the four coils' aiStyle-35 machine
// (171/475/505/506, Projectile.cs:27189-27291), the DrawProjDirect chain
// placement preview (Main.cs:28499-28669), the Kill refund ladder
// (:77572-77615) + CheckRopeUsability (Player.cs:41506-41545). WIRED:
// the 35 dispatch arm + rows (projectile.rs / projectile/data.rs), the
// death-hook refund + ai1==0 cue (the d77 kill_tail_cue pattern), the
// chain draw in draw_with_hand, the usability gate in use_tile_item.
// W4-c40c — the storm-bolt lane (research/78 item 4): the LightningGenerator
// bolt geometry (`LightningGenerator.cs:9-326`, the private-LCG random walk +
// the CanHitTarget tile/liquid gate) and the damaging SpawnStormLightning
// spawn chain (`WorldGen.cs:71864-72018`, the strike roll + attractor scan +
// safe-spot/rain-shelter/nearby-bolt gates). UNWIRED modules — the 1091
// aiStyle-203 AI + the UpdateWorldGen tick drain are the wiring follow-up
// (the npc_ai_sand_elemental precedent). Witnesses WRITTEN NOT RUN.
mod storm_lightning;
mod resource_pack;
// S1-P-i38: the recall / magic-mirror / ice-mirror / potion-of-return use
// path — ItemCheck's spawn-recall arms (Player.cs:42976-42999 / :43107-43140 /
// :43141-43172) + Spawn(RecallFromItem) (:37534-37710) + the spawn-position
// helpers (:37718-37991). Wired into update_editing (the boss-spawner seam).
mod recall_teleport;
// W4-d51: the player-tick `Dust.NewDust` drain lanes — the extra-jump chain
// (`Player.JumpMovement`, Player.cs:20818-21012) first; the tick queues the
// box/velocity snapshot, this module replays on the `UpdatePlayers` stream.
mod player_dust;
// W4-g66: the movement-trail drain lanes — the DoubleJumpVisuals and
// floor-dust chains first. Same snapshot-then-replay convention as
// `player_dust`.
mod player_boot_dust;
mod player_trail_dust;
mod player_wing_dust;
// W4-g89f: the Rod of Discord 1326 / Rod of Harmony 5335 use path —
// ItemCheck_UseTeleportRod (Player.cs:45476-45534) + the chaos-state tax.
// Wired into update_editing (the recall/boss-spawner seam).
mod rod_teleport;
mod regen;
mod research;
mod sand;
// W4-e4: the world dust sites' SECOND batch — ParticleOrchestrator sites
// 41-61 + Mount sites 1-19 (research/66's claim; d54's first-40 orch lane
// reserved, WorldGen/Main stay g14/g11-g13). UNWIRED code-only emitters.
mod orchestrator_dust;
// W4-d130: the ambient snowfall (`Main.snowing()`, Main.cs:13032-13116) —
// the type-76 dust spawn/AI/draw, the rain rig's flake approximation retired.
mod snow;
mod ambience;
mod ambient_sound;
mod capture;
mod chest_gen;
mod craft;
mod craft_filter;
mod registry;
mod variants;
mod render;
// W4-d60: UNWIRED tile-draw particle layer (TileDrawing.DrawTiles_EmitParticles,
// TileDrawing.cs:6688-7547) - grass-sway leaf shed / petals / lava bubbles /
// herb sparkles / ore shine; the DrawSingleTile call-site gate + the inline
// 192-leaf reconciliation are the wiring wave (see the module doc +
// docs/ANALYSIS.md "The tile-draw particle layer (W4-d60)").
mod tile_draw_particles;
// W4-g13: Main.cs dust-site batch 3 (sites 17-23 of the 23-site census,
// docs/research/64-dust-main-sites-claims.md) - the Moon Lord core's dead
// tentacle-tip puff, the Tavernkeep ale sparkle pair, the world-item coin
// sparkle and the retro oldDrawWater surface/lava pair. WIRED: the feeds
// run in app/frame/draw.rs::draw_region outside the G6 cache on
// rngs.global() (see the module doc + docs/ANALYSIS.md "The Main.cs dust
// sites, batch 3 (W4-g13)").
mod spawner;
mod tag_effects;
mod settings;
mod shop;
mod shimmer;
mod slime_sky;
mod signs;
mod sound;
mod soundid;
// W4-g22: the Main.cs PlaySound census batch 2 (sites 91-180 of 271,
// :44118-:49727) — the hair-window scroll machine, the clothes-window
// open/cancel/save cue arms, the builder-accessory toggle cycles and the
// case-20 female-hit slot pick. The menu-half cues ride Menu::pending_cue
// (code-only mode — witnesses written not run).
mod main_sounds;
mod sun_scorch;
mod dryad_stardew;
mod town_shimmer;
mod test_world_helpers;
mod orch_dust;
mod orch_dust_boulder;
mod orch_dust_lightning;
mod orch_dust_town;
mod orch_dispatch;
mod town;
mod town_arrival;
mod town_rehome;
mod town_spawn;
mod town_arrival_pets;
mod tracked_sound;
mod touch_damage;
mod ui;
mod world_dust;
mod world_dust_main;
mod world_dust_tile;
mod world_dust_wall;

mod npc_hit_effect_d;

// W4-g61: UNWIRED NPC spawn/teleport-in dust (PoofOfSmoke, the
// BigMimicSpawnSmoke, TransformVisuals, Main.TeleportEffect +
// UpdateNPC_TeleportVisuals) - the wiring round drains them at the
// bug-net/statue/transform/teleport seams (see the module doc).
mod npc_spawn_visuals;

// W4-e5 (gore table wave B): UNWIRED NPC.HitEffect tail — the next 40
// death sites (NPC.cs:81942-82240); appended to wave A's dispatch.
mod npc_hit_effect_b;

// W4-f10 (gore table wave C): UNWIRED NPC.HitEffect per-NPC gore table,
// the third batch of death sites (NPC.cs:82206 onward); the StrikeNPC_Inner
// dispatch claim is the wiring wave (the wave-A npc_hit_effect_a precedent).
mod npc_hit_effect_c;

// W4-g60: the NPC death dust lane 2 (HitEffect death halves, types
// 231-697) — the death retain's feed (see the module doc).

mod projectile_dust3;

// W4-g65: the per-kind DEATH dust bursts inside Projectile.Kill() (the trail
// dust is g63/g64; the i17/i18 special-arrow/bullet deaths stay with their
// modules). The death hook queues resolved bodies; the world-sim pass drains
// them onto the DustPool on the UpdateProjectiles stream (gallop-dust shape).
mod projectile_death_dust;

mod projectile_gore_a;

mod projectile_gore_b;

mod projectile_gore_c;

mod projectile_gore_d;

mod projectile_gore_e;

mod projectile_gore_f;

mod projectile_gore_g;

mod projectile_gore_h;

mod projectile_gore_i;

mod projectile_gore_j;

mod projectile_gore_k;

mod projectile_gore_l;

mod projectile_gore_m;

mod projectile_gore_n;

mod projectile_gore_o;

// W4-g17 player wave A (code-only): the extra-jump gore bursts —
// Fart/Santank/Cloud (Player.cs:20838-21022); the extra-jump chain
// dispatch is the wiring wave.
mod player_gore_a;

// W4-g17 player wave B (code-only): the dash launch + sandstorm hold
// + team-hair-dye gore (Player.cs:21419-24293); the dash/sandstorm/
// hair-dye dispatch claims are the wiring wave.
mod player_gore_b;

// W4-g17 player wave C (code-only): the dodge bursts — Mystic Sash +
// Ninja (Player.cs:38057-38165); the dodge dispatch claims are the
// wiring wave.
mod player_gore_c;

// W4-g17 player wave D (code-only): the death confetti + item-use
// visuals (Player.cs:38869-45027); the KillMe/ItemCheck dispatch
// claims are the wiring wave.
mod player_gore_d;

// W4-g17 player wave E (code-only, FINAL): the melee-hitbox sparkle
// ladder + use-style-5 lantern mote (Player.cs:45048-51125); completes
// the Player.cs census (32 sites). ItemCheck dispatch deferred.
mod player_gore_e;

// W4-g17 wiring wave 2 (code-only): the ItemCheck_EmitUseVisuals dust-only
// item arms (Player.cs:44756-45023) — the 989/2880/44-46/273/723/190/213/
// 121/122/155/676/3063/3823/724/795-802/367 arms + the frostBurn/magmaStone/
// meleeEnchant-1-6/8 tails; the co-lane census confirmed them unclaimed.
mod player_dust_use;

// W4-g17 (code-only): the world-side Gore.NewGore sites, wave A — the
// TreeGrowFX trunk/leaf column + ash shake (WorldGen.cs:29600-29672)
// and the Check3x3 larva burst (:53230-53257); the TreeGrowFXCheck /
// axe-hit / Check3x3 dispatch claims are the wiring wave.
mod world_gore_a;

// W4-g17 wave B (code-only): the CheckPot gore switch, styles 0-2
// (WorldGen.cs:56878-56894); the PotEvent dispatch is the wiring wave.
mod world_gore_b;

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
        // The vanilla launch flags are `key=value` args (`Main.cs:5040+`),
        // never world paths — skip them when picking the positional path.
        match args.iter().skip(1).find(|a| !a.contains('=')) {
            Some(p) => PathBuf::from(p),
            None => app::latest_own_world().unwrap_or(app::terrstaria_world()?),
        }
    };

    log::info!("world: {}", world_path.display());
    // The world plus its side sections — the `.wld`'s section payloads
    // (M6-SAVE-1) or the native `.trwld`'s v7-v10 records (M6-SAVE-1b/c/d) —
    // the loaded stores the `App` merges ahead of the load-time loot roll.
    let (mut world, sections) = app::load_world(&world_path)?;

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
    let mut settings = settings::Settings::load();
    // `Main.cs:5079-5083` — the `slowliquids=1` launch flag raises
    // `Setting_UseReducedMaxLiquids` (no config.json key; the settings
    // screens never write it). Applied ahead of `App::new` so the boot
    // settle's `ReInit` cap (`Liquid.cs:98-101`) sees it.
    settings.apply_launch_args(&args);
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
        app::build_atlas_reusing_static(&mut world, &assets_dir, &mut boot_static, &mut boot_fonts)?;

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
        // The boot has no played card yet — the front end parks one on
        // `CharPlay` (the W4-f44 hydration seam).
        None,
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
