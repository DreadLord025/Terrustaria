//! The App — the port of `Terraria.Main`'s instance state (the C# statics,
//! one struct) plus the `main` bootstrap that builds it: the world load, the
//! atlas and the starting entities. The remaining `impl App` glue that has
//! no submodule of its own lives here too: the sim delegates, the
//! front-screen openers and the keyboard tails. The winit event loop is
//! `loop.rs`, the frame orchestrator `frame.rs`. (A7: extracted from
//! `main.rs`.)

pub(crate) mod clicks;
pub(crate) mod filters;
pub(crate) mod flow;
pub(crate) mod frame;
mod r#loop;
pub(crate) mod netglue;
pub(crate) mod saveglue;
pub(crate) mod session;
pub(crate) mod settings_ui;

// The RNG helpers live in `frame` with their main consumer, but the world
// simulation and the sky reach them through the app root.
pub(crate) use frame::{next_rand, rand01};

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anyhow::{Context, Result};
use glam::Vec2;
use tr_core::framing;
use tr_core::lighting::{LightMap, LightMode};
use tr_core::liquid::LiquidSystem;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit, save, wld, worldgen};
use tr_render::{Atlas, Gpu, SpriteRenderer};
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::achievements;
use crate::app::flow::{FrontScreen, KeyboardSession, KeyboardTail, tr_fake_count, tr_ui_preset};
use crate::assets;
use crate::audio;
use crate::banners;
use crate::biome::{
    advance_bg_alphas, advance_bg_style, is_desert_biome, is_ocean_biome, is_snow_biome,
};
use crate::chat;
use crate::craft;
use crate::emotes;
use crate::inv;
use crate::item::{EMPTY_SLOT, HOTBAR, INV, ItemDrop, ItemSlot};
use crate::map;
use crate::menu;
use crate::net;
use crate::npc::{Npc, NpcKind};
use crate::persist;
use crate::player::{self, Player};
use crate::projectile::ProjectilePool;
use crate::rain::Rain;
use crate::registry;
use crate::render::debug::{DebugCache, StatsAcc};
use crate::render::entities::{NpcAssets, PlayerAssets, make_npc_assets, make_player_assets};
use crate::render::sky::{
    CloudSprite, SkyAssets, StarSprite, make_clouds, make_sky_assets, make_stars,
};
use crate::render::tiles::{DrawScratch, TileDrawStats};
use crate::settings;
use crate::ui;
use crate::ui::text::font_width;
use crate::world_sim;
use crate::world_sim::step::FixedStepper;

// ---
/// Multiplayer role of this process (`M6`).
pub(crate) enum NetRole {
    Off,
    Host(net::Server),
    Client(net::Client),
}

pub(crate) struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) gpu: Option<Gpu>,
    pub(crate) renderer: Option<SpriteRenderer>,
    /// M1.4-BG-18 phase 2 harness (`TR_POST=1`): the post-pass pipeline routing
    /// the scene through an offscreen target + filter pass. Phase 3's
    /// `ApplyVisuals` driver decides per-frame whether a filter runs.
    pub(crate) post: Option<tr_render::PostPipeline>,
    /// Noise image (`Images/Misc/noise`) uploaded for the post pass bindings.
    pub(crate) noise_tex: Option<wgpu::Texture>,
    /// M1.4-BG-18 phase 4: the water coverage mask target (created with `post`;
    /// its texture only exists while WaterDistortion is wanted and water is in
    /// view).
    pub(crate) water_mask: Option<tr_render::WaterMask>,
    /// `Main.SceneState` — the phase-1 tr-core port; `update_scene` steps its
    /// fades and the filter driver reads its intensities.
    pub(crate) scene_state: tr_core::scene::SceneState,
    /// `Sandstorm` — the event state machine; the storm machine ticks in
    /// `update_scene` and the storm filters read its severity.
    pub(crate) sandstorm: tr_core::scene::Sandstorm,
    /// M1.4-BG-18 phase 3: the `ApplyVisuals` driver (`app::filters`) — filter
    /// board, storm shader state and the winner uniform the post branch
    /// applies.
    pub(crate) filter_driver: crate::app::filters::FilterDriver,
    pub(crate) world: World,
    pub(crate) atlas: Option<Atlas>,
    /// The packed BC2 font pages the renderer uploads (`loop`'s
    /// `SpriteRenderer::new`); `None` without the retail fonts.
    pub(crate) font_atlas: Option<tr_render::FontAtlas>,
    pub(crate) lookups: assets::Lookups,
    pub(crate) clouds: Vec<CloudSprite>,
    pub(crate) sky: SkyAssets,
    /// `Main.bgStyle`: the biome background style currently being drawn (0 forest,
    /// 2 desert, 4 ocean, 7 snow; the full vanilla table also has 1/3/5/6/8/9/10..14).
    pub(crate) bg_style: i32,
    /// `Main.bgDelay`: ticks to wait before starting a new transition (30 normally,
    /// 60 for a forest->forest variant swap); negative while the post-switch
    /// cooldown counts back up.
    pub(crate) bg_delay: i32,
    /// `Main.instantBGTransitionCounter`: forces `bgStyle` on the first frames
    /// after a world load so the background starts on the right biome.
    pub(crate) bg_instant: i32,
    /// `Main.bgAlphaFrontLayer` (16 slots): per-front-layer cross-fade alpha.
    pub(crate) bg_alpha_front: [f32; 16],
    /// `Main.bgAlphaFarBackLayer` (16 slots): per far-back (mountain) layer alpha.
    pub(crate) bg_alpha_far: [f32; 16],
    pub(crate) stars: Vec<StarSprite>,
    pub(crate) player_assets: PlayerAssets,
    pub(crate) player: Option<Player>,
    pub(crate) follow_player: bool,
    pub(crate) player_step: FixedStepper,
    pub(crate) npc_step: FixedStepper,
    pub(crate) npcs: Vec<Npc>,
    pub(crate) npc_assets: NpcAssets,
    pub(crate) npc_timer: f32,
    /// The worm chain id counter (`AI_006`'s chain identity) — the head
    /// takes the next id, the segments share it.
    pub(crate) next_worm_id: u32,
    /// The Journey powers (`GameContent/Creative/CreativePowers.cs`).
    pub(crate) creative_powers: crate::creative::CreativePowers,
    /// The debug cheat-sheet panel (`debug_panel.rs` — the self-contained
    /// module; delete the field + the module + the call sites to remove).
    pub(crate) debug_panel: crate::debug_panel::DebugPanel,
    /// The favorited inventory slots (`Item.favorited`'s port — the slot
    /// indices, not the per-item flag; the v1 tracks the slot positions).
    pub(crate) favorite_slots: Vec<usize>,
    /// The Goblin Army spawn cadence (the 2-second invasion pass).
    pub(crate) invasion_spawn_timer: f32,
    /// The open shop — the NPC's Vec index (`Main.npcShop`; the stock and
    /// the mood captured at the open).
    pub(crate) shop_npc: Option<usize>,
    /// The stock captured at the open (`Chest.item[]`).
    pub(crate) shop_stock: Vec<u16>,
    /// The happiness multiplier captured at the open
    /// (`currentShoppingSettings.PriceAdjustment`).
    pub(crate) shop_mood: f32,
    /// The closed shop — the state cleared (`Main.npcShop = -1`'s
    /// equivalent).
    pub(crate) shop_closed_flag: bool,
    /// `Main.InReforgeMenu` (`Main.cs:40022-40029`): the Tinkerer's chat
    /// option flips it beside the shop; `close_shop` clears it.
    pub(crate) in_reforge_menu: bool,
    /// `Main.reforgeItem` — the slot at the menu's (50, 270); the transfer
    /// keeps it at one item (`ItemSlot.cs:821-836`).
    pub(crate) reforge_slot: ItemSlot,
    /// `Main.reforgeCooldown` (`Main.cs:67053`): 60 ticks after a top-tier
    /// roll (`:41924`), decremented per tick (`:17121-17123`).
    pub(crate) reforge_cooldown: i32,
    pub(crate) npc_rng: u64,
    /// `NPC.noSpawnCycle` (`NPC.cs:6179`): one spawn call is suppressed while
    /// set, then the flag clears (`:80171-80174`).
    pub(crate) spawn_no_cycle: bool,
    pub(crate) attack_cd: f32,
    pub(crate) inventory: [ItemSlot; INV],
    /// The non-grid inventory slots (`EquipSlots`): coins, ammo, equipment,
    /// vanity, dye, misc equips and the trash slot (`Main.DrawInventory`).
    pub(crate) equip: inv::EquipSlots,
    /// `Main.EquipPage`: 0 armour/vanity/dye, 2 misc equips, 3 camera.
    pub(crate) equip_page: i32,
    pub(crate) selected: usize,
    /// Vanilla-style per-slot hotbar scale (selected grows to 1.0).
    pub(crate) hotbar_scales: [f32; HOTBAR],
    /// Escape toggles the inventory panel (`Main.OpenInventory`), it no longer quits.
    pub(crate) inventory_open: bool,
    /// `Main.BestiaryTracker.Kills` (`NPCKillsTracker.cs`): the per-world
    /// bestiary kill counts, persisted in the sidecar's v5 kills line.
    pub(crate) kill_tracker: tr_core::bestiary::KillTracker,
    /// `IngameFancyUI.OpenUIState(Main.BestiaryUI)`'s port — the bestiary
    /// screen over the world, opened from the inventory's bestiary button
    /// (`Main.cs:42420-42470`). `Some(index)` = an entry's info page.
    pub(crate) bestiary_open: bool,
    pub(crate) bestiary_selected: Option<usize>,
    /// `Main.chatMonitor` + `Main.drawingPlayerChat`/`Main.chatText` — the
    /// chat state (M6-CHAT-1).
    pub(crate) chat: chat::ChatMonitor,
    /// `EmoteBubble.byID` + the wheel state (M6-EMOTE-1).
    pub(crate) emotes: emotes::EmoteSystem,
    /// `BannerSystem`'s kill tally (M2-BUFF-8).
    pub(crate) banners: banners::BannerSystem,
    /// The homeless town NPCs' look-for-home timer (`lookForHomeTimeout`'s
    /// stand-in; M4-TOWN-1).
    pub(crate) town_look_timer: f32,
    /// `Player.reuseDelay` (`Player.cs:1444`): the post-swing reuse lock in
    /// seconds (`Item.reuseDelay` ticks, `Item.cs:317`); M3-ITEM-5.
    pub(crate) reuse_delay: f32,
    /// `Main.Achievements` — the achievement registry (M6-ACH-1).
    pub(crate) achievements: achievements::AchievementManager,
    /// The modern crafting panel, open alongside the inventory
    /// (`NewCraftingUI`, `Main.cs:41706-41709`); `TR_NOCRAFT=1` hides it,
    /// `TR_CRAFT=1` forces it on.
    pub(crate) crafting_open: bool,
    /// The world chest item stores (`Main.chest[]`'s `Chest.item`) the
    /// crafting reads; the chest UI/storage fills it (`M3-CHEST-UI`,
    /// `M6-SAVE-2`) — in-memory for now.
    pub(crate) world_chests: Vec<craft::WorldChest>,
    /// `ChestUI.StartingRowForDrawing` (`ChestUI.cs:52`): the chest grid's top
    /// visible row, clamped per frame by the draw and by the wheel scroll.
    pub(crate) chest_scroll_row: i32,
    /// `ChestUI.ButtonScale` (`ChestUI.cs:48`; the static ctor's `new
    /// float[7]` starts at 0.0, `:1306`): the text buttons' scale animation —
    /// the rename pair (5/6) draws in phase 3, the LootAll family (0-4) is
    /// M3-STORE-2's.
    pub(crate) chest_button_scales: [f32; ui::chest::BUTTON_COUNT],
    /// `ChestUI.ButtonHovered` (`ChestUI.cs:50`).
    pub(crate) chest_button_hovered: [bool; ui::chest::BUTTON_COUNT],
    /// `Main.stackSplit` (`Main.cs:1693`): the slot stack-split's repeat
    /// cooldown — 0 idle, >1 gating the next transfer (`ItemSlot.cs:1700`).
    /// Ticked per drawn frame by `app::frame` (the Main.cs:61781-61807
    /// accelerator + the :62466-62473 decay).
    pub(crate) stack_split: i32,
    /// `Main.stackDelay` (`Main.cs:1694`): the repeat delay between
    /// right-click transfers, decaying 7 → 2 by the accelerator ladder.
    pub(crate) stack_delay: i32,
    /// `Main.stackCounter` (`Main.cs:1695`): frames toward the next
    /// `stackDelay` decay step.
    pub(crate) stack_counter: i32,
    /// `Main.superFastStack` (`Main.cs:1696`): extra items per frame once the
    /// delay bottoms out (~2 s of holding).
    pub(crate) super_fast_stack: i32,
    /// `Main.timesTriedToFastStack` (`Main.cs:1697`).
    pub(crate) times_tried_to_fast_stack: i32,
    /// `Main.cursorOverride` (`Main.cs:2047`): the hovered slot's
    /// alternate-click cursor — 8 = transfer from the chest, 9 = transfer to
    /// it (`ItemSlot.cs:213-218`) — or −1. Reset per interface frame
    /// (`Main.cs:44941`), set by the chest hover evaluation (`ItemSlot.cs:
    /// 599-627`); the Cursors_8/9 texture draw is a recorded follow-up (the
    /// phase-4 file scope excludes assets.rs/cursor.rs).
    pub(crate) cursor_override: i32,
    /// The stored `Chest.name` (`Chest.cs:82`) per store index — the panel's
    /// name line and the rename submit's write target (`App.chest_names`
    /// feeds `chest::chest_name`). A side map until M6-SAVE-2 widens the
    /// `WorldChest` entity: the frozen `editing.rs` stocked-chest test
    /// literal pins the struct shape (the phase-5 discovery, Gap #2).
    pub(crate) chest_names: HashMap<usize, String>,
    /// `Main.defaultChestName` (`Main.cs:1753`) — the tile default captured
    /// at the rename keyboard's open (`IngameFancyUI.cs:103-123`); the
    /// submit's empty-if-default read (`ChestUI.cs:849-852`).
    pub(crate) chest_default_name: String,
    /// `Main.instance.textBlinkerCount/State` (`Main.cs:2287-2289`) — the
    /// chest-panel name editor's caret blinker (`ui::chest::TextBlinker`).
    pub(crate) text_blinker: ui::chest::TextBlinker,
    /// The crafting RNG stream (`Main.rand`'s per-consumer stand-in) — the
    /// alchemy discount rolls.
    pub(crate) craft_rng: u64,
    /// `NPC.RevengeManager._markers` (`NPC.cs:6437`, `CoinLossRevengeSystem
    /// .cs:315`): the cached coin carriers. Session-only — cleared on the
    /// world unload (the vanilla never saves them).
    pub(crate) revenge_markers: Vec<crate::revenge::RevengeMarker>,
    /// `CoinLossRevengeSystem._gameTime` (:382-389) — the revenge clock.
    pub(crate) revenge_tick: u64,
    /// Main menu; `None` while playing.
    pub(crate) menu: Option<menu::Menu>,
    /// `Main.menuMode == 888`'s half of the single-mode invariant
    /// (`Main.cs:51059-51069`): exactly one of `front` and `menu` is `Some` —
    /// the draw site picks `front` first, then `menu`, and every navigation
    /// sets/clears both sides through [`app::flow::next_flow_state`]'s table
    /// (Review Focus 1).
    pub(crate) front: Option<FrontScreen>,
    /// `UICharacterCreation.BackupConfirmationState` (`UICharacterCreation.cs:
    /// 152`, parked at `:1215`): the screen menuMode 40's No branch restores.
    /// `None` except across a discard confirm.
    pub(crate) front_backup: Option<FrontScreen>,
    /// `_characterSelectMenu`'s card list — `OnActivate`'s `LoadPlayers` +
    /// `UpdatePlayersList` (`UICharacterSelect.cs:135-157`).
    pub(crate) players: Vec<ui::save_list::PlayerEntry>,
    /// `_worldSelectMenu`'s card list (`UIWorldSelect.cs:137-166`).
    pub(crate) worlds: Vec<ui::save_list::WorldEntry>,
    /// The port's not-ported notice for front-screen clicks whose target
    /// lands with Tasks 6-8 (ours, like `Menu.hint`).
    pub(crate) front_hint: Option<String>,
    /// `Main.mouseLeft`'s port for the front screens (`Main.cs:51101`'s role):
    /// held between the press and release events while `front` is live — the
    /// creation screen's HSL sliders drag off it, like the menu's value bars.
    pub(crate) front_mouse_down: bool,
    /// `Main.PendingPlayer`'s recorded twin — `FinishCreatingCharacter`'s
    /// `PlayerFileData.CreateAndSave` + `Main.LoadPlayers` port: the created
    /// draft feeds the implicit session entry's name/difficulty
    /// (`UICharacterCreation.cs:1704-1711`; no `.plr` writer — documented).
    pub(crate) session_draft: Option<ui::char_creation::NewPlayerDraft>,
    /// `PlayerInput.SettingsForUI.ShowGamepadHints`'s port stand-in: the
    /// keyboard's two-layer gate (`UIVirtualKeyboard.cs:675-678`) — false
    /// without a gamepad, so the desktop `outerLayer2` draws (Review Focus 6);
    /// `TR_VK_GRID=1` overrides it for screenshots (the plan Task 6 hook).
    pub(crate) vk_grid: bool,
    /// `config.json` options edited by the settings screens.
    pub(crate) settings: settings::Settings,
    pub(crate) player_name: String,
    pub(crate) net: NetRole,
    /// Players of other machines streaming from the host/client.
    pub(crate) remote_players: HashMap<u32, net::PlayerNet>,
    /// Our id assigned by the host (0 while hosting).
    pub(crate) local_id: u32,
    pub(crate) net_timer: f32,
    /// The client handshake state — `Netplay.Connection.State`
    /// (`Netplay.cs:469-543`); stays at 0 while hosting.
    pub(crate) net_state: i32,
    /// The world-blob slice buffer while the host streams the world
    /// (`15 WorldChunk`), assembled in order at `WorldComplete`.
    pub(crate) net_chunks: Vec<Option<Vec<u8>>>,
    /// The `7 WorldInfo` header, held until the blob completes the install.
    pub(crate) net_info: Option<net::WorldInfo>,
    /// The `9 Status` load-progress line (`Netplay.cs:526-540`'s StatusText).
    pub(crate) net_status_text: String,
    pub(crate) net_status_max: i32,
    pub(crate) net_status_count: i32,
    /// The last received frame stamp — the client-side `TimeOutTimer`
    /// (`Main.cs:64540-64544`).
    pub(crate) net_last_recv: Instant,
    /// A disconnect ran inside `poll_net`'s dispatch: the swapped-out role
    /// must not be restored afterwards (the `mem::replace` borrow dance has
    /// no other way to say "the handlers changed the role").
    pub(crate) net_dropped: bool,
    /// The client's NPC slots keyed by the host's slot index (the `23`
    /// store — the C#'s `Main.npc[]` on a client). Rebuilt into `npcs` in
    /// slot order for the render.
    pub(crate) net_npcs: HashMap<u32, crate::npc::Npc>,
    /// The slots seen in the current `poll_net` batch: the `23` despawn diff.
    pub(crate) net_npc_seen: Vec<u32>,
    /// When each `23` slot was last heard from — a slot silent past
    /// `NPC_STALE` is despawned (the host sends nothing when an NPC dies).
    pub(crate) net_npcs_seen_at: HashMap<u32, Instant>,
    /// Set when the npc store changed and `self.npcs` must be rebuilt.
    pub(crate) net_npcs_dirty: bool,
    /// The host's per-slot NPC bookkeeping: `(kind, generation)` — the
    /// generation bumps on a kind change so the client recreates the slot
    /// (`MessageBuffer.cs:1602-1613`'s test).
    pub(crate) net_npc_gen: HashMap<u32, (i16, u32)>,
    /// The client's item drops keyed by the host slot index (the `21` store).
    pub(crate) net_items: HashMap<i16, crate::item::ItemDrop>,
    /// Set when the item store changed and `self.items` must be rebuilt.
    pub(crate) net_items_dirty: bool,
    /// The host's last-sent `21` state per slot — the change-detection diff.
    pub(crate) net_item_last: HashMap<i16, net::ItemState>,
    /// The client's `22 ItemOwner` reservations: slot → (reserver, hold ticks).
    pub(crate) net_reserved: HashMap<i16, (u8, u32)>,
    /// The client's last-sent `13 PlayerState` — the change-driven send test.
    pub(crate) net_ps_last: net::PlayerState,
    /// The heartbeat counter for the `13` cadence (`net_tick` calls).
    pub(crate) net_beat: u32,
    pub(crate) join_target: Option<(String, String)>,
    /// Explored-tile minimap and full map (`M`).
    pub(crate) map: Option<map::WorldMap>,
    /// `Main.grabMapX`/`grabMapY`: the cursor anchor of an in-progress
    /// fullscreen-map drag (`Main.cs:55346-55354`).
    pub(crate) map_grab: Option<[f32; 2]>,
    /// `MinimapFrame.Update`'s hovered button (`MinimapFrame.cs:163`),
    /// resolved every frame in `handle_minimap_clicks` and consumed by the
    /// minimap branch's `draw_foreground`; `Some` also mirrors `:164`'s
    /// `Main.LocalPlayer.mouseInterface = true` (the mine/place suppression).
    pub(crate) minimap_hover: Option<ui::minimap::MinimapButton>,
    /// `Main.mapStyle` (`Main.cs:66750`, default 1): 0 = off, 1 = the minimap
    /// frame, 2 = the overlay (out of P4 scope — nothing draws, the whole-UI P4
    /// plan's Uncertainty 7). The map-style buttons land in Task 3, so a debug
    /// env var selects it for screenshots.
    pub(crate) map_style: i32,
    pub(crate) audio: audio::Audio,
    pub(crate) music: audio::MusicSystem,
    pub(crate) last_hp: i32,
    /// `Main.saveTime`: seconds since the last autosave (`Main.cs:19751-19765`).
    pub(crate) autosave: f32,
    /// Stack carried on the cursor while the inventory is open (`Main.mouseItem`).
    pub(crate) mouse_item: ItemSlot,
    pub(crate) ui_left_press: bool,
    pub(crate) ui_right_press: bool,
    pub(crate) mouse_pos: Vec2,
    /// `Main.cursorAlpha`/`cursorColorDirection`/`cursorScale`: the vanilla cursor
    /// pulse, advanced once per frame by `Main.CursorColor` (`Main.cs:52477`).
    pub(crate) cursor: ui::cursor::CursorState,
    /// Last value passed to `Window::set_cursor_visible`, so the OS pointer is only
    /// toggled when the in-world HUD (and thus the vanilla cursor) appears.
    pub(crate) os_cursor_hidden: Option<bool>,
    pub(crate) mine_down: bool,
    pub(crate) place_down: bool,
    pub(crate) mine_target: Option<(i32, i32)>,
    pub(crate) mine_timer: f32,
    pub(crate) edit_target: Option<(i32, i32)>,
    pub(crate) world_path: PathBuf,
    pub(crate) dev_break: Option<(i32, i32)>,
    pub(crate) dev_place: Vec<(i32, i32, u16)>,
    pub(crate) camera: Vec2,
    pub(crate) zoom: f32,
    /// `TR_ZOOM` pins the zoom for screenshots, ignoring the settings value.
    pub(crate) zoom_pinned: bool,
    pub(crate) debug: bool,
    pub(crate) keys: HashSet<KeyCode>,
    pub(crate) mouse_down: bool,
    pub(crate) last_mouse: Vec2,
    pub(crate) last_frame: Instant,
    pub(crate) frame_history: VecDeque<f32>,
    pub(crate) last_stats: tr_render::sprite::RenderStats,
    pub(crate) bench_frames: Option<u32>,
    pub(crate) bench_samples: Vec<f32>,
    pub(crate) frame_count: u64,
    pub(crate) shot_path: Option<PathBuf>,
    pub(crate) shot_at: u64,
    pub(crate) atlas_stats: (u32, u32, usize, usize),
    /// Reused light-map storage (D1) and dark-rect merge scratch (A3/A4).
    pub(crate) light: LightMap,
    pub(crate) scratch: DrawScratch,
    pub(crate) tile_stats: TileDrawStats,
    pub(crate) stats_on: bool,
    pub(crate) stats_acc: StatsAcc,
    pub(crate) debug_cache: DebugCache,
    pub(crate) light_mode: LightMode,
    /// `Main.mouseTextColor`: oscillates 190..=255 each frame, driving torch/lava/shimmer
    /// crystal-wall flicker (`DoUpdate_AnimateCursorColors` in Main.cs).
    pub(crate) mouse_text_color: u8,
    pub(crate) mouse_text_color_dir: i8,
    /// `Main.demonTorch` / `Main.demonTorchDir` (`DoUpdate_AnimateTileGlows`).
    pub(crate) demon_torch: f32,
    pub(crate) demon_torch_dir: f32,
    /// `Main.DiscoR/G/B` and the `Main.DiscoStyle` cycle (`DoUpdate_AnimateDiscoRGB`).
    pub(crate) disco: [u8; 3],
    pub(crate) disco_style: u8,
    /// `TileLightScanner._random` seed; advanced once per frame like `NextSeed()`.
    pub(crate) light_rng_seed: u64,
    pub(crate) anim_time: f32,
    pub(crate) has_liquids: bool,
    /// Cellular liquid flow (`Liquid.UpdateLiquid`), ticked at 30 Hz.
    pub(crate) liquids: LiquidSystem,
    pub(crate) liquid_step: FixedStepper,
    pub(crate) liquid_tick: u32,
    /// `Main.waterfallManager`: scanning, animation counters and draw geometry.
    pub(crate) waterfall: tr_core::waterfall::WaterfallManager,
    pub(crate) waterfall_step: FixedStepper,
    /// `Wiring`'s statics: the mech queue, the wire floods and the teleporter
    /// pads (`Wiring.cs`).
    pub(crate) wiring: tr_core::wiring::Wiring,
    pub(crate) wiring_step: FixedStepper,
    /// The tile-entity update pass (`TileEntity.PerformUpdates` — the
    /// logic sensors' per-tick scan, M7-TE-2).
    pub(crate) te_step: FixedStepper,
    /// The world-growth pass (`WorldGen.UpdateWorld`'s rate loops, M7-GROWTH-1).
    pub(crate) growth_step: FixedStepper,
    /// The growth stream (the `Main.rand` rolls inside `UpdateWorld`).
    pub(crate) growth_rng: tr_core::tile_light::FastRandom,
    /// The saved hardmode ore tiers (`SavedOreTiers`, the altar smashes).
    pub(crate) ore_tiers: tr_core::ores::SavedOreTiers,
    /// Weather and raindrops (`Rain.MakeRain` / `Rain.Update`).
    pub(crate) rain: Rain,
    /// The sandstorm's flying sand (`Sandstorm.EmitDust`).
    pub(crate) sand: crate::sand::Sand,
    /// The minecart wheel sparks (`Minecart.WheelSparks` → the type-213 dust)
    /// — the `Main.dust` stand-in for the cart's pool, fed by the `world_sim`
    /// drain and ticked/drawn with the other world dust.
    pub(crate) minecart_sparks: crate::minecart::Sparks,
    /// `Main.ambientWaterfallX/Y/Strength` (`Main.cs:2201-2211`): the
    /// waterfall-scan accumulators `Main.Ambience` reads every ¼ s.
    pub(crate) ambient_waterfall: (f32, f32, f32),
    /// `Main.ambientLavafallX/Y/Strength` (`WaterfallManager.cs:960-962`).
    pub(crate) ambient_lavafall: (f32, f32, f32),
    /// `Main.ambientCounter` — the ¼ s gate of `Main.Ambience`
    /// (`Main.cs:13469-13476`).
    pub(crate) ambient_counter: f32,
    /// Global wind (`Main.windSpeedCurrent`/`windSpeedTarget`/`windCounter`,
    /// `tr-core/src/wind.rs`) plus the `TileDrawing.Update` sway counters.
    pub(crate) wind: tr_core::wind::Wind,
    /// The shared 60 Hz world tick (M1.4-BG-22): the dayRate cluster — the
    /// `UpdateWeather` thunder arm, the ×dayRate wind walk,
    /// `Sandstorm.UpdateTime` — steps on this accumulator in
    /// `world_sim::weather::update_world_tick` (`Main.cs:17228-17231`).
    pub(crate) world_tick_step: FixedStepper,
    /// `Main.atmo` (`Main.cs:2503`): the atmosphere density
    /// `UpdateAtmosphereTransparencyToSkyColor` derives per draw
    /// (`Main.cs:61983`, formula `:63185-63199`); the ambient-sky creatures
    /// multiply their colour by it (`AmbientSky.cs:63`).
    pub(crate) atmo: f32,
    /// `Main.SceneMetrics` (`Main.cs:1789`): the per-frame sensor state
    /// `update_scene` derives; the ambience gates read its zone flags
    /// (`AmbienceServer.cs:77-84`).
    pub(crate) scene_metrics: tr_core::scene::SceneMetrics,
    /// The single grappling hook (`AI_007_GrapplingHooks` state machine).
    pub(crate) grapple: Option<crate::grapple::GrappleHook>,
    /// The `AmbientSky` manager + `AmbienceServer` scheduler
    /// (`crates/tr-app/src/ambience.rs`).
    pub(crate) ambience: crate::ambience::Ambience,
    /// The scheduler's `Main.rand` stand-in stream (the seed the inlined
    /// `NetAmbienceModule` loopback hands to `FastRandom`).
    pub(crate) ambience_rng: u64,
    pub(crate) tree_wind_counter: f32,
    pub(crate) grass_wind_counter: f32,
    /// `TileDrawing._sunflowerWindCounter` (`TileDrawing.cs:374`): the sunflower's
    /// own sway counter — the flower never rides the grass one (its special pass
    /// reads `_sunflowerWindCounter`, `TileDrawing.cs:8918`). C# advances it per
    /// world-update tick (`TilesRenderer.Update()` in `Main.DoUpdateInWorld_Inner`,
    /// `Main.cs:17812`), so [`Self::sunflower_wind_step`] steps it on the 60 Hz
    /// accumulator instead of the redraw cadence the grass/tree counters inherited.
    pub(crate) sunflower_wind_counter: f32,
    /// The 1/60 stepper that ticks [`Self::sunflower_wind_counter`] — the port
    /// shape of C#'s per-world-update advance (`TileDrawing.Update`).
    pub(crate) sunflower_wind_step: FixedStepper,
    /// Ambient tree leaves (`TileDrawing.EmitTreeLeaves` / `Gore_UpdateLeaf`).
    pub(crate) leaves: Vec<crate::render::foliage::LeafGore>,
    /// `Main.liquidAlpha` (`Main.cs:1103`): the per-water-style cross-fade the
    /// `DrawWaters` ramp drives (`Main.cs:57476-57489`); the liquid and
    /// waterfall passes consume it. Zeros, like the original's allocation
    /// (`Main.cs:66805`), so the liquids fade in over the first frames.
    pub(crate) liquid_alpha: [f32; 15],
    /// World item drops (`Terraria.Item` first pass).
    pub(crate) items: Vec<ItemDrop>,
    pub(crate) item_rng: u64,
    pub(crate) item_step: FixedStepper,
    /// Remaining seconds of the current held-item swing.
    pub(crate) swing: f32,
    /// The swing's `useStyle` — the held item's at the swing start
    /// (`Player.cs:4449`'s `ApplyItemAnimation` carries it).
    pub(crate) swing_style: u8,
    /// The aim angle (the mouse direction, radians) the shooting/throwing
    /// styles read — `Player.itemRotation`'s source for useStyle 5.
    pub(crate) swing_aim: f32,
    /// The shot-fired latch of the current swing (`ItemTimeIsZero` fires
    /// once per use — the refire waits for the next `TryStartUse`).
    pub(crate) shot_fired: bool,
    /// The swing's full duration (`Player.itemAnimationMax` in seconds); the
    /// body-row thirds and the hand key read `swing / swing_max`.
    pub(crate) swing_max: f32,
    /// Use-button edges for the non-`autoReuse` gates; refreshed every frame
    /// like `Player.releaseUseItem` (`Player.cs:42822`).
    pub(crate) place_prev_down: bool,
    pub(crate) place_released: bool,
    pub(crate) mine_prev_down: bool,
    pub(crate) mine_released: bool,
    /// `Player.itemTime` in seconds: the action accumulator inside a swing
    /// (`Player.cs:4416-4418`); placement re-fires whenever it hits zero.
    pub(crate) item_time: f32,
    /// Coin purse value in copper coins.
    pub(crate) coins: u64,
    /// The 1000-slot projectile pool (`Main.projectile`, `projectile.rs`).
    pub(crate) projectiles: ProjectilePool,
    /// The falling tombstone projectiles (`Player.DropTombstone`).
    pub(crate) tomb_falls: Vec<crate::death::TombFall>,
    /// `Player.lostCoins`/`lostCoinString` + the last death text — the
    /// death screen's "Dropped coins" line (`Main.cs:45333-45338`).
    pub(crate) lost_coins: u64,
    pub(crate) lost_coin_string: String,
    /// The full death broadcast text (`{name} {text}`).
    pub(crate) death_text: String,
    /// `Main.invasionProgress*` (`Main.cs:1913-1921`): the invasion bar's
    /// state — the points, the max, the icon (the group + 3), the 160-tick
    /// display timer, the fade alpha and the near-invasion flag.
    pub(crate) invasion_progress: i32,
    pub(crate) invasion_progress_max: i32,
    pub(crate) invasion_progress_icon: i32,
    pub(crate) invasion_progress_display_left: i32,
    pub(crate) invasion_progress_alpha: f32,
    pub(crate) invasion_progress_near: bool,
    /// The placed tombstones' epitaphs keyed by the tile anchor
    /// (vanilla stores the text in the projectile's `miscText` and then the
    /// tile render reads it — `Projectile.miscText`).
    pub(crate) tomb_texts: Vec<(i32, i32, String)>,
    pub(crate) projectile_step: FixedStepper,
    /// `TR_THROW` debug hook: throw one shuriken at startup.
    pub(crate) dev_throw: bool,
    /// G6: identity of the last world instance range; a hit lets the frame reuse it.
    pub(crate) world_key: Option<WorldFrameKey>,
    /// `Main.hellBlackBoxBottom` (`Main.cs:2463`): world Y of the hell floor line the
    /// previous frame's magma pass produced, clamping the underworld black box. It is
    /// written after the box draws, so the box always consumes the previous frame's
    /// value (0 on the first frame — a negative height draws nothing, like XNA).
    pub(crate) hell_black_box_bottom: f32,
}

/// G6: everything the world instance range (sky strips, tiles, walls, dark rects)
/// depends on, compared bit-exactly. A hit keeps the previous frame's instances and
/// only redraws the UI; the light map has its own cache (D2) inside `LightMap`.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct WorldFrameKey {
    /// Camera floored to its tile: sub-tile pans keep the world range (the GPU
    /// transform moves it) while the background range is rebuilt every frame.
    pub(crate) cam_tile: [i32; 2],
    pub(crate) zoom: u32,
    pub(crate) screen: [u32; 2],
    pub(crate) sky_light: [u32; 3],
    pub(crate) hell_glow: u32,
    /// `LightParams::identity()`: every tile-light parameter (mouseTextColor,
    /// demon torch, disco, shimmer...) in one value.
    pub(crate) light_params: u64,
    pub(crate) light_mode: u32,
    /// Non-zero only for modes whose light depends on time ("Trippy").
    pub(crate) anim: u32,
    /// Player position/animation identity; the player is part of the world range.
    pub(crate) player_key: u32,
    /// Hash of all NPC positions/animations.
    pub(crate) npc_key: u32,
    /// Item drops and their ground-bob step.
    pub(crate) items_key: u32,
    /// Held item kind + swing phase.
    pub(crate) hand_key: u32,
    /// Light-map generation; a recompute (D3) changes tile tints.
    pub(crate) light_generation: u64,
    /// Waterfall animation frames + visible-list revision (0 when none visible).
    pub(crate) waterfall: u64,
    /// Remote peers' smoothed positions/animations and the NPC smoothing
    /// accumulators: the networked entities drawn inside the cached world
    /// range. Zero in single-player (the offsets are zero and no peers).
    pub(crate) net_key: u32,
    pub(crate) world_revision: u64,
}

fn parse_light_mode(v: &str) -> Option<LightMode> {
    match v.trim().to_ascii_lowercase().as_str() {
        "color" | "colour" | "0" => Some(LightMode::Color),
        "white" | "1" => Some(LightMode::White),
        "retro" | "2" => Some(LightMode::Retro),
        "trippy" | "glitch" | "3" => Some(LightMode::Trippy),
        _ => None,
    }
}

fn parse_xy(v: &str) -> Option<(i32, i32)> {
    let (a, b) = v.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// `TR_NOHUD=1` hides the HUD, hotbar and minimap so screenshots show only the
/// world. The debug overlay (`TR_DEBUG`) is independent and still drawn.
pub(crate) fn no_hud() -> bool {
    static NO_HUD: OnceLock<bool> = OnceLock::new();
    *NO_HUD.get_or_init(|| std::env::var_os("TR_NOHUD").is_some())
}

/// True when a `TR_SHOT` capture runs without an explicit `TR_MOUSE` pin: the
/// OS pointer is then parked offscreen and `CursorMoved` is ignored, so two runs
/// of the same command produce SHA256-equal screenshots no matter where the
/// desktop mouse happens to be. The pointer is harness input, not scene content;
/// an explicit `TR_MOUSE` still wins (the cursor/tooltip shots).
pub(crate) fn deterministic_pointer_park() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var_os("TR_SHOT").is_some() && mouse_pin().is_none())
}

/// The `TR_DYE=k1,k2,...` pin parse: the first ten comma-separated kinds fill
/// `Player.dye` positionally; garbage and 0 leave the slot empty.
fn parse_tr_dye(list: &str) -> [u16; 10] {
    let mut out = [0u16; 10];
    for (slot, kind) in out.iter_mut().zip(list.split(',')) {
        *slot = kind.trim().parse().unwrap_or(0);
    }
    out
}

/// The `TR_DMOUNT=<kind>` pin parse (`App.equip.misc_dyes[3]`); garbage → 0.
fn parse_tr_dmount(s: &str) -> u16 {
    s.trim().parse().unwrap_or(0)
}

/// `TR_POSE=sit|sleep` — writes the furniture beside the world spawn and
/// snaps the player on through the real interaction decode, so the
/// screenshots/tests exercise the live sitting/sleeping pose (the sim's
/// re-validation holds it; nothing wakes in a headless capture).
fn apply_pose_pin(world: &mut World, mut p: Option<&mut crate::player::Player>, mode: &str) {
    let put = |world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16| {
        let i = world.index(x, y);
        world.tiles[i] = tr_core::tile::Tile {
            kind,
            frame_x: fx,
            frame_y: fy,
            ..Default::default()
        };
        world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    };
    // `Player::spawn` seeds the header's spawn row and the fixed step drops
    // the feet onto the surface below. The pin builds its own stage — a flat
    // dirt ledge eight tiles right of the spawn with three air rows over it
    // — so the furniture/snap scene never fights the terrain (trees, bushes
    // and slopes would otherwise draw over or block the pose).
    let sx = world.spawn_x + 8;
    let mut ground = world.spawn_y;
    for _ in 0..400 {
        if world
            .tile(sx, ground)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        {
            break;
        }
        ground += 1;
    }
    let sy = ground;
    for col in sx - 1..=sx + 4 {
        put(world, col, sy, 1, 0, 0); // the dirt floor
        for y in (sy - 3)..sy {
            let i = world.index(col, y);
            world.tiles[i] = tr_core::tile::Tile::default(); // the clearance
        }
    }
    match mode {
        // The left-facing chair (15): the top cell carries `frameY = 0`, the
        // bottom `frameY = 18` (the 40-px style stride's second row).
        "sit" => {
            put(world, sx, sy - 2, 15, 0, 0);
            put(world, sx, sy - 1, 15, 0, 18);
            if let Some((seat_x, seat_y, dir, off)) =
                edit::sitting_target_info(world, sx, sy - 1, 1)
            {
                if let Some(p) = p.as_deref_mut() {
                    p.sit_down(world, seat_x, seat_y, dir, off);
                }
            }
        }
        // The unflipped style-0 bed (79): columns `frameX 0/18/36/54`, the
        // bottom row `frameY = 18` (the 36-px style stride).
        "sleep" => {
            for (dx, fx) in [0i16, 18, 36, 54].iter().enumerate() {
                put(world, sx + dx as i32, sy - 2, 79, *fx, 0);
                put(world, sx + dx as i32, sy - 1, 79, *fx, 18);
            }
            if let Some((ax, ay, dir, style)) = edit::sleeping_target_info(world, sx, sy - 1) {
                if let Some(p) = p.as_deref_mut() {
                    p.sleep_down(world, ax, ay, dir, style);
                }
            }
        }
        _ => {}
    }
    // The pin writes tiles after the render chunks were built — the
    // revision bump is what `apply_edit` uses to rebuild them.
    world.revision = world.revision.wrapping_add(1);
}

/// `TR_MOUSE=x,y` pins the logical pointer position and ignores real cursor
/// motion, so screenshots can deterministically show the cursor and a hovered
/// item tooltip (like `TR_CAM`/`TR_DAY` do for the world).
pub(crate) fn mouse_pin() -> Option<Vec2> {
    static PIN: OnceLock<Option<Vec2>> = OnceLock::new();
    *PIN.get_or_init(|| {
        let s = std::env::var("TR_MOUSE").ok()?;
        let mut it = s.split(',').map(|v| v.trim().parse::<f32>());
        match (it.next(), it.next()) {
            (Some(Ok(x)), Some(Ok(y))) => Some(Vec2::new(x, y)),
            _ => None,
        }
    })
}

/// `TR_PAN=<tiles/frame>` pans the camera deterministically. Benchmarks use it to
/// measure the moving-camera path, where the static-scene caches (D2/G6) mostly miss.
pub(crate) fn pan_step() -> Option<f32> {
    static PAN: OnceLock<Option<f32>> = OnceLock::new();
    *PAN.get_or_init(|| {
        std::env::var("TR_PAN")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
    })
}

impl App {
    /// The swapchain size in pixels (the deterministic fallback mirrors the
    /// `TR_SHOT`/headless default of 1280x720).
    pub(crate) fn screen_size(&self) -> (f32, f32) {
        world_sim::camera::screen_size(self)
    }

    pub(crate) fn update_player(&mut self, dt: f32) {
        world_sim::player::update_player(self, dt);
    }

    /// `Player.cs:24774-24793` + `24898-24908`: the fullscreen-map controls. The pan
    /// step is `16 / scale` tiles **once per frame, with no `dt` factor** (the
    /// original runs it in the per-frame input pass).
    pub(crate) fn update_map_input(&mut self) {
        let (sw, _) = self.screen_size();
        let Some(m) = self.map.as_mut() else { return };
        let step = 16.0 / m.scale;
        let k = |code| self.keys.contains(&code);
        if k(KeyCode::KeyW) || k(KeyCode::ArrowUp) {
            m.pos[1] -= step;
        }
        if k(KeyCode::KeyS) || k(KeyCode::ArrowDown) {
            m.pos[1] += step;
        }
        if k(KeyCode::KeyA) || k(KeyCode::ArrowLeft) {
            m.pos[0] -= step;
        }
        if k(KeyCode::KeyD) || k(KeyCode::ArrowRight) {
            m.pos[0] += step;
        }
        // `Main.cs:50453-50454`: `cMapZoomIn = "Add"`, `cMapZoomOut = "Subtract"`.
        if k(KeyCode::NumpadAdd) {
            m.scale = map::map_key_scale(m.scale, true);
        }
        if k(KeyCode::NumpadSubtract) {
            m.scale = map::map_key_scale(m.scale, false);
        }
        // `Main.cs:55342-55359`: the left button drags the fullscreen map by the
        // per-frame cursor delta (`Main.grabMapX`/`grabMapY`).
        let cursor = self.mouse_pos;
        if self.mine_down {
            if let Some(anchor) = self.map_grab {
                let d = map::map_drag_delta([cursor.x - anchor[0], cursor.y - anchor[1]], m.scale);
                m.pos[0] += d[0];
                m.pos[1] += d[1];
            }
            self.map_grab = Some([cursor.x, cursor.y]);
        } else {
            self.map_grab = None;
        }
        m.scale = map::map_scale_clamp(sw, self.world.width, m.scale);
        m.pos = map::map_clamp_pos(m.pos, &self.world);
    }

    /// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
    pub(crate) fn advance_disco(&mut self) {
        world_sim::clock::advance_disco(self);
    }

    /// Fixed 60 Hz liquid stepping; `Liquid.UpdateLiquid` itself runs every second
    /// tick, matching the original 30 Hz rate.
    pub(crate) fn update_liquids(&mut self, dt: f32) {
        world_sim::liquids::update_liquids(self, dt);
    }

    /// Fixed 60 Hz `WaterfallManager.UpdateFrame` / `FindWaterfalls`, matching the
    /// original's per-update tick and 30-tick rescan cadence.
    pub(crate) fn update_waterfalls(&mut self, dt: f32, screen: Vec2) {
        world_sim::liquids::update_waterfalls(self, dt, screen);
    }

    pub(crate) fn update_wiring(&mut self, dt: f32) {
        world_sim::wiring::update_wiring(self, dt);
    }

    /// The tile-entity pass (`WorldGen.cs:71507`'s
    /// `TileEntity.PerformUpdates`): the logic sensors scan their state and
    /// fire the wire trips (M7-TE-2).
    pub(crate) fn update_tile_entities(&mut self, dt: f32) {
        world_sim::tile_entities::update_tile_entities(self, dt);
    }

    /// The world-growth pass (`UpdateWorld`'s rate loops, M7-GROWTH-1).
    pub(crate) fn update_growth(&mut self, dt: f32) {
        world_sim::growth::update_growth(self, dt);
    }

    /// `Main.GetPreferredBGStyleForPlayer`: the surface background style wanted by
    /// the biome under the screen centre. Supported styles: 0 forest, 2 desert,
    /// 4 ocean, 7 snow. `SceneMetrics` zones are approximated with the same
    /// camera-column tests the water style uses (beach before desert before snow).
    pub(crate) fn preferred_bg_style(&self) -> i32 {
        if is_ocean_biome(&self.world, self.camera) {
            4
        } else if is_desert_biome(&self.world, self.camera) {
            2
        } else if is_snow_biome(&self.world, self.camera) {
            7
        } else {
            0
        }
    }

    /// `Main.DrawBG` (the cross-fade half): `UpdateBGVisibility_BackLayer` +
    /// `UpdateBGVisibility_FrontLayer`, then the style/delay timers.
    ///
    /// `backgroundLayerTransitionSpeed = 0.05` per drawn frame, so a full
    /// transition takes 20 frames once the delay elapses.
    pub(crate) fn update_bg_visibility(&mut self) {
        let preferred = self.preferred_bg_style();
        let speed = if self.bg_instant > 0 {
            // `instantBGTransitionCounter`: adopt the loaded biome immediately.
            self.bg_instant -= 1;
            self.bg_style = preferred;
            1.0
        } else {
            advance_bg_style(&mut self.bg_style, &mut self.bg_delay, preferred);
            0.05 // `Main.backgroundLayerTransitionSpeed`
        };
        advance_bg_alphas(
            &mut self.bg_alpha_front,
            &mut self.bg_alpha_far,
            self.bg_style,
            speed,
        );
    }

    /// Rain runs on its own 60 Hz accumulator; drops are simulated at the original
    /// per-tick velocity regardless of frame rate.
    pub(crate) fn update_rain(&mut self, dt: f32, screen: Vec2) {
        world_sim::weather::update_rain(self, dt, screen);
    }

    pub(crate) fn update_npcs(&mut self, dt: f32) {
        world_sim::npcs::update_npcs(self, dt);
        // The shop closes when the seller leaves (the NPC's death/retreat
        // invalidates the index).
        if let Some(i) = self.shop_npc {
            if self.npcs.get(i).map(|n| n.hp > 0) != Some(true) {
                self.close_shop();
            }
        } else if self.shop_closed_flag {
            self.shop_closed_flag = false;
        }
    }

    /// `Player.BuyItem`'s shape: the purse covers the price → deduct and
    /// deposit the item through the pickup stack (`Player.PickupItem`'s
    /// slot_for_pickup). Returns false when the purse is short or the slot
    /// is out of range.
    pub(crate) fn buy_shop_item(&mut self, slot: usize) -> bool {
        let Some(item) = self.shop_stock.get(slot).copied() else {
            return false;
        };
        let price = crate::shop::buy_price(item, self.shop_mood) as u64;
        if self.coins < price {
            return false;
        }
        let idx = match crate::item::slot_for_pickup(&mut self.inventory, item, 0) {
            Some(i) => i,
            None => return false,
        };
        // `Player.PickupItem`'s deposit: the stack merge on the same kind,
        // else the fresh slot.
        let sl = &mut self.inventory[idx];
        if sl.kind == item && sl.count > 0 {
            sl.count += 1;
        } else {
            *sl = crate::item::ItemSlot {
                kind: item,
                name: crate::itemdata::ITEMS[item as usize].name,
                count: 1,
                prefix: 0,
            };
        }
        self.coins -= price;
        true
    }

    /// The reforge slot transfer (`ItemSlot.cs:821-836`): swap with the
    /// cursor when the cursor's item is single; a stack splits one item in
    /// (only into the empty slot), mirroring the vanilla's clone-1 rule.
    pub(crate) fn reforge_slot_click(&mut self) {
        if self.mouse_item.count > 1 {
            if self.reforge_slot.count == 0 {
                self.reforge_slot = self.mouse_item;
                self.reforge_slot.count = 1;
                self.mouse_item.count -= 1;
            }
            return;
        }
        std::mem::swap(&mut self.mouse_item, &mut self.reforge_slot);
    }

    /// `Main.ReforgeItemInReforgeSlot` (`Main.cs:41902-41933`): the charge
    /// (`BuyItem`'s purse gate) precedes the roll; then the reset-to-
    /// defaults + `Prefix(-2)` — the port recomputes the stats from the
    /// kind's defaults, so the reset is the prefix-field write — and the
    /// top-tier 60-tick cooldown (`Main.cs:41924`). The vanilla plays
    /// `BestReforge`/`Item37`; the port's cue set has no such key yet.
    pub(crate) fn reforge_click(&mut self) {
        let Some(def) = crate::registry::get(self.reforge_slot.kind) else {
            return;
        };
        let cost = crate::shop::reforge_cost(
            crate::prefix::prefixed_stats(self.reforge_slot.kind, self.reforge_slot.prefix)
                .map(|s| s.value)
                .unwrap_or(def.value),
            self.reforge_slot.count.max(1),
            self.shop_mood,
        );
        if self.reforge_cooldown > 0 || self.coins < cost {
            return;
        }
        self.coins -= cost;
        let mut rng = self.npc_rng;
        let (prefix, top_tier) = crate::prefix::reforge_roll(self.reforge_slot.kind, &mut rng);
        self.npc_rng = rng;
        self.reforge_slot.prefix = prefix;
        if top_tier {
            self.reforge_cooldown = 60;
        }
    }

    /// `StartDayImmediately`/`StartNightImmediately`/`SetNoon`/`SetMidnight`
    /// (`CreativePowers.cs:769-860`): the clock jumps to the target — the
    /// (dayTime, time) pair from `button_target`. The boundary events (the
    /// moon-phase roll, the event rolls) land with their tasks.
    pub(crate) fn set_time_of_day(&mut self, id: crate::creative::PowerId) -> bool {
        let Some((day, time)) = self.creative_powers.button_target(id) else {
            return false;
        };
        self.world.day_time = day;
        self.world.time = time;
        true
    }

    /// `ItemSlot.ToggleFavorited` (`ItemSlot.cs:873-877`): flip the slot's
    /// favorite + `PlaySound(12)`. Returns the legacy sound id (12) so the
    /// caller plays it and the tests pin it (the `chest::toggle_open`
    /// precedent — no audio capture seam), 0 when the guard refuses. The
    /// `idx >= INV` guard keeps the coin/ammo rows out — `canFavoriteAt`
    /// allows contexts 1/2 in the C# (`ItemSlot.cs:386-389`), but the port's
    /// favorites address the 10x5 grid only (the recorded coin-row gap,
    /// Phase 5's ANALYSIS entry).
    pub(crate) fn toggle_favorite(&mut self, idx: usize) -> u32 {
        if idx >= crate::item::INV {
            return 0;
        }
        if let Some(pos) = self.favorite_slots.iter().position(|&i| i == idx) {
            self.favorite_slots.remove(pos);
        } else {
            self.favorite_slots.push(idx);
        }
        12
    }

    pub(crate) fn is_favorite(&self, idx: usize) -> bool {
        self.favorite_slots.contains(&idx)
    }

    /// `ItemSlot.MouseItemSwap` (`ItemSlot.cs:1255-1263`) at context 6 — the
    /// plain held/trash swap. The retain table has no context 6, so the C#
    /// clears both favorites here (`:1258-1261`); the port's slot-positional
    /// favorites have nothing to clear (the trash slot is not favoritable and
    /// a held favorite is unrepresentable — the v1 model, Phase 5's ANALYSIS
    /// entry). **No favorited-held guard exists in the C# swap** — favorited
    /// held items CAN be trashed; the can't-drop rule lives in
    /// `DropSelectedItem` (`Player.cs:5170-5184`), the sell/drop pass's seam.
    pub(crate) fn trash_swap(&mut self) {
        std::mem::swap(&mut self.equip.trash, &mut self.mouse_item);
    }

    /// `Main.SetNPCShopIndex(-1)`'s equivalent. The reforge menu force-
    /// closes with the shop (`Main.cs:41574-41578`'s talk/chest change
    /// `dropItemCheck`); the slot item returns to the cursor if held there
    /// is impossible, so it drops to the inventory's first free slot.
    pub(crate) fn close_shop(&mut self) {
        self.shop_npc = None;
        self.shop_stock.clear();
        self.shop_mood = 1.0;
        self.in_reforge_menu = false;
        if self.reforge_slot.count > 0 {
            if let Some(i) = crate::item::slot_for_pickup(
                &mut self.inventory,
                self.reforge_slot.kind,
                self.reforge_slot.prefix,
            ) {
                self.inventory[i].count =
                    (self.inventory[i].count + self.reforge_slot.count)
                        .min(crate::registry::max_stack(self.reforge_slot.kind));
                self.reforge_slot = EMPTY_SLOT;
            }
        }
    }

    /// `Main.GetNurseHealCost` (`Main.cs:40054-40100`): the missing life
    /// plus the removable debuffs x100, multiplied by the happiness
    /// adjustment. The boss-progression/expert/discount multipliers sit
    /// constants-off (the downed tracking lands with the boss work).
    pub(crate) fn nurse_heal_cost(&self, nurse_idx: usize) -> i32 {
        let Some(player) = self.player.as_ref() else {
            return 0;
        };
        let missing = player.max_hp - player.hp;
        let debuffs = player
            .buffs
            .active_types()
            .iter()
            .filter(|t| crate::buffs::is_debuff(**t))
            .count() as i32;
        let npc = &self.npcs[nurse_idx];
        let mood =
            crate::happiness::process_mood(&self.world, &self.npcs, npc, player.center(), false);
        nurse_heal_cost_for(missing, debuffs, mood.multiplier)
    }

    /// `Main.NPCChatText_DoNurseHeal` (`Main.cs:40103+`): `BuyItem(healCost)`
    /// → the life to full + the removable debuffs cleared. Returns false
    /// when the purse is short or there is nothing to heal.
    pub(crate) fn nurse_heal(&mut self, nurse_idx: usize) -> bool {
        let cost = self.nurse_heal_cost(nurse_idx);
        if cost <= 0 || self.coins < cost as u64 {
            return false;
        }
        let Some(player) = self.player.as_mut() else {
            return false;
        };
        player.hp = player.max_hp;
        // The removable debuffs clear (`DoNurseHeal`'s removal block).
        let types: Vec<i32> = player
            .buffs
            .active_types()
            .iter()
            .copied()
            .filter(|t| crate::buffs::is_debuff(*t))
            .collect();
        for t in types {
            player.buffs.clear_buff(t);
        }
        self.coins -= cost as u64;
        true
    }

    pub(crate) fn update_editing(&mut self, dt: f32, screen: Vec2) {
        world_sim::editing::update_editing(self, dt, screen);
    }

    pub(crate) fn is_client(&self) -> bool {
        matches!(self.net, NetRole::Client(_))
    }

    /// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
    pub(crate) fn update_items(&mut self, dt: f32) {
        world_sim::items::update_items(self, dt);
    }

    /// Fixed 60 Hz projectile stepping. Damage lands through `Npc::hurt`, so the
    /// hit cooldown and knockback are shared with melee.
    pub(crate) fn update_projectiles(&mut self, dt: f32) {
        world_sim::projectiles::update_projectiles(self, dt);
    }

    /// Enters a front screen the way `OpenCharacterSelectUI`/
    /// `OpenWorldSelectUI` do (`Main.cs:52346-52350`, `:52362-52366`): the
    /// singleton state is set and `OnActivate` re-lists (`LoadPlayers`/
    /// `LoadWorlds` + `UpdatePlayersList`/`UpdateWorldsList`,
    /// `UICharacterSelect.cs:135-157` / `UIWorldSelect.cs:137-166`). The menu
    /// side closes with the settings sync first.
    pub(crate) fn enter_front(&mut self, front: FrontScreen) {
        match front {
            FrontScreen::CharSelect => {
                let fakes = tr_fake_count("TR_FAKE_CHARS");
                // The listing root is `Main.PlayerPath` (`Main.cs:67065-67075`)
                // — the port's `Terrustaria/Players` (`save.rs`'s mirror).
                let dir = save::default_players_dir();
                self.players =
                    ui::save_list::list_players(dir.as_deref(), Some(self.session_entry()), fakes);
                ui::char_select::sort_players(&mut self.players);
            }
            FrontScreen::WorldSelect => {
                let fakes = tr_fake_count("TR_FAKE_WORLDS");
                // The listing root is the port's `Documents/My Games/
                // Terrustaria/Worlds` (`Main.WorldPath`'s directory,
                // `Main.cs:67065-67075`; `save.rs:20-29`).
                let dir = save::default_world_dir().unwrap_or_default();
                self.worlds = ui::save_list::list_worlds(&dir, fakes);
                ui::world_select::sort_worlds(&mut self.worlds, false);
            }
            // The keyboard re-lists nothing (`UIVirtualKeyboard.OnActivate`'s
            // only setup is the gamepad snap point, `:843-850` — out of scope,
            // §J.3). It enters through `open_keyboard`, which parks the menu
            // side the same way.
            FrontScreen::Keyboard(_) => {}
            // The creation screen re-lists nothing — `NewCharacterClick`
            // (`UICharacterSelect.cs:108-114`) just sets `Main.PendingPlayer`
            // and the state. Entering through `enter_front` (the Back/New
            // hops) keeps the single-mode invariant's write pattern.
            FrontScreen::CharCreation(_) => {}
            // The world creator re-lists nothing either — `NewWorldClick`
            // (`UIWorldSelect.cs:110-116`) sets `Main.newWorldName` and the
            // state; the draft rode in from the table row.
            FrontScreen::WorldCreation(_) => {}
        }
        self.park_menu();
        self.front = Some(front);
    }

    /// Opens the virtual keyboard the way `RenameButtonClick` does
    /// (`Main.clrInput()` + `Main.MenuUI.SetState(state)`,
    /// `UICharacterListItem.cs:197-201`): the ctor args become the session's
    /// [`ui::keyboard::Request`], the [`ui::keyboard::KeyboardState`] seeds
    /// from them (`:429-433`), and `back` is the `_cancelAction` target. The
    /// menu side closes with the settings sync first, so exactly one of
    /// `front`/`menu` stays live (Review Focus 1).
    pub(crate) fn open_keyboard(&mut self, request: ui::keyboard::Request, back: FrontScreen) {
        self.open_keyboard_with_tail(request, Some(back), KeyboardTail::Rename);
    }

    /// The [`App::open_keyboard`] twin for the creator's naming flows — the
    /// `tail` selects the submit behaviour (`KeyboardTail`). `back: None`
    /// backs the in-game chest rename (`IngameFancyUI.Close` returns to
    /// gameplay, `:162-188`).
    pub(crate) fn open_keyboard_with_tail(
        &mut self,
        request: ui::keyboard::Request,
        back: Option<FrontScreen>,
        tail: KeyboardTail,
    ) {
        let state = ui::keyboard::KeyboardState::new(&request);
        self.park_menu();
        self.front = Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
            request,
            state,
            back,
            tail,
        })));
    }

    /// The unified open-container index (`Player.chest`, `Player.cs:2876`):
    /// -1 closed, -2..-5 the banks (`bank.rs`'s `bank_for_chest`), >= 0 the
    /// `App.world_chests` store index. The -1 answer when no player is live
    /// (the front screens park the world routes).
    pub(crate) fn open_container(&self) -> i32 {
        self.player
            .as_ref()
            .map_or(crate::bank::CHEST_CLOSED, |p| p.chest)
    }

    /// `Main.editChest` (`Main.cs:1747`) — the port's editor is exactly "the
    /// chest-rename keyboard session is open" (`IngameFancyUI.cs:124` sets
    /// it, the submit/cancel closures clear it).
    pub(crate) fn chest_editing(&self) -> bool {
        matches!(
            self.front.as_ref(),
            Some(FrontScreen::Keyboard(s)) if s.tail == KeyboardTail::ChestRename
        )
    }

    /// `Main.npcChatText`'s editor read (`ChestUI.cs:113`) — the live
    /// keyboard text while the chest editor is open
    /// (`UIVirtualKeyboard.cs:1061-1067`'s sync), "" otherwise.
    pub(crate) fn chest_edit_text(&self) -> &str {
        match self.front.as_ref() {
            Some(FrontScreen::Keyboard(s)) if s.tail == KeyboardTail::ChestRename => {
                s.state.text.as_str()
            }
            _ => "",
        }
    }

    /// The submit tail. The rename tail keeps Task 6's notice (no `.plr`
    /// writer, spec §9.4); the creator's naming tails run
    /// `OnFinishedNaming` (`:1684-1689`: the trimmed name lands in the draft,
    /// `SetState(this)` restores the screen) and `OnFinishedNamingAndCreating`
    /// (`:1696-1702`: the same, then `FinishCreatingCharacter`). The chest
    /// rename tail runs `ChestUI.RenameChestSubmit` + `Close(quiet: true)`
    /// (`IngameFancyUI.cs:148-156`).
    pub(crate) fn keyboard_submit(&mut self) {
        let Some(FrontScreen::Keyboard(session)) = self.front.as_ref() else {
            return;
        };
        let name = session.state.text.trim().to_string();
        // The chest submit reads the live untrimmed text — the C# submit
        // closure ignores its trimmed argument (`UIVirtualKeyboard.cs:363`,
        // `Main.npcChatText`'s sync `:1061-1067`).
        let chest_text = session.state.text.clone();
        let back = session.back.clone();
        // `InternalSubmit`'s cue (`SoundEngine.PlaySound(10)`,
        // `UIVirtualKeyboard.cs:975`) — the callers' `TextIsValidForSubmit`
        // gates already ran (`:358-365`'s click gate, `keyboard_enter`'s
        // `CanSubmit`).
        self.audio.play("Menu_Open");
        match session.tail {
            KeyboardTail::Rename => {
                self.front_hint = Some("Renaming is not ported yet (no save-file writer)".into());
                self.enter_front(back.expect("the rename tail is built with a back screen"));
            }
            KeyboardTail::Name => {
                let mut back = back.expect("the naming tail is built with a back screen");
                if let FrontScreen::CharCreation(state) = &mut back {
                    state.draft.name = name;
                }
                self.enter_front(back);
            }
            KeyboardTail::NameAndCreate => {
                let mut back = back.expect("the naming tail is built with a back screen");
                if let FrontScreen::CharCreation(state) = &mut back {
                    state.draft.name = name;
                }
                self.enter_front(back);
                self.finish_creating_character();
            }
            // `OnFinishedSettingName` (`UIWorldCreation.cs:726-731`): the
            // trimmed name lands (`UpdateInputFields` -> `SetContents`), the
            // creation screen returns through `GoBackHere` (`:730`).
            KeyboardTail::WorldName => {
                let mut back = back.expect("the world-name tail is built with a back screen");
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.name = name;
                }
                self.enter_front(back);
            }
            // `OnFinishedSettingSeed` (`:817-831`): the trimmed seed lands
            // (`TryApplyingCopiedSeed`'s secret-seed paths are the out-of-
            // scope Advanced tab, plan Uncertainty 4).
            KeyboardTail::WorldSeed => {
                let mut back = back.expect("the world-seed tail is built with a back screen");
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.seed = name;
                }
                self.enter_front(back);
            }
            // `OnFinishedNamingAndCreating` (`:838-842`): the name lands,
            // then `FinishCreatingWorld` (`:845-863`).
            KeyboardTail::WorldNameAndCreate => {
                let mut back =
                    back.expect("the world-name-and-create tail is built with a back screen");
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.name = name;
                }
                self.enter_front(back);
                if let Some(FrontScreen::WorldCreation(draft)) = self.front.clone() {
                    self.finish_creating_world(draft);
                }
            }
            // `ChestUI.RenameChestSubmit` + `Close(quiet: true)`
            // (`IngameFancyUI.cs:148-156`) — the name lands, the keyboard
            // closes back into gameplay.
            KeyboardTail::ChestRename => {
                crate::chest::rename_submit(self, &chest_text);
            }
        }
    }

    /// The cancel tail — `_cancelAction` (`UIVirtualKeyboard.cs:86`) is the
    /// card lists' `GoBackHere` (`UICharacterListItem.cs:214-216`,
    /// `UIWorldListItem.cs:428-430`): reopen the screen the keyboard came
    /// from, no rename, no notice. The chest rename tail runs
    /// `ChestUI.RenameChestCancel` + `Close(quiet: true)`
    /// (`IngameFancyUI.cs:152-156`) — back into gameplay, nothing written.
    pub(crate) fn keyboard_cancel(&mut self) {
        let Some(FrontScreen::Keyboard(session)) = self.front.as_ref() else {
            return;
        };
        let is_chest = session.tail == KeyboardTail::ChestRename;
        let back = session.back.clone();
        // `Cancel`'s cue (`SoundEngine.PlaySound(11)`,
        // `UIVirtualKeyboard.cs:984`) — the C# cancels play it before the
        // `_cancelAction` (the buttons `:321`/:387, Escape's
        // `TryEscapingMenu` :806-810).
        self.audio.play("Menu_Close");
        if is_chest {
            crate::chest::rename_cancel(self);
            return;
        }
        self.enter_front(back.expect("the cancel tail is built with a back screen"));
    }

    /// One printable character from the physical keyboard — `Main.GetInputText`'s
    /// result landing in `this.Text` (`UIVirtualKeyboard.cs:778-788`). That
    /// path never touches `KeyState`, so a seeded Shift stays latched (the
    /// on-screen keys' [`ui::keyboard::type_char`] is the only unlatcher).
    pub(crate) fn keyboard_character(&mut self, c: char) {
        if let Some(FrontScreen::Keyboard(session)) = self.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, &c.to_string());
        }
    }

    /// The physical Backspace — `Main.GetInputText` shrinks the string the
    /// same frame (`Main.cs:50793`'s handler the keyboard state feeds through).
    pub(crate) fn keyboard_backspace(&mut self) {
        if let Some(FrontScreen::Keyboard(session)) = self.front.as_mut() {
            session.state.backspace();
        }
    }

    /// The physical Enter — `Main.inputTextEnter && UIVirtualKeyboard.CanSubmit`
    /// fires the submit (`:763-766`); `CanSubmit` is `ValidateText`'s cached
    /// `TextIsValidForSubmit` (`:108-145`, `:585-626`), so an invalid text
    /// keeps the keyboard open, exactly like the dead Submit click. The chest
    /// context is always valid (`:615-626`'s `_editingChest` gate).
    pub(crate) fn keyboard_enter(&mut self) {
        let valid = match self.front.as_ref() {
            Some(FrontScreen::Keyboard(session)) => ui::keyboard::text_valid_for(
                &session.state.text,
                session.request.allow_empty,
                session.request.context,
            ),
            _ => return,
        };
        if valid {
            self.keyboard_submit();
        }
    }

    /// The Death-font width a large `UITextPanel` measures (`UITextPanel.cs:
    /// 126-131`), or the bitmap fallback's — the front screens' `header_w`
    /// convention (the P3 `title_width` rule).
    pub(crate) fn front_header_w(&self, text: &str) -> f32 {
        match (&self.lookups.death_font, self.renderer.as_ref()) {
            (Some(f), _) => font_width(f, text, 1.0),
            (None, Some(r)) => r.text_width(text, 1.0),
            (None, None) => 0.0,
        }
    }
    pub(crate) fn pan_camera(&mut self, dt: f32) {
        world_sim::camera::pan_camera(self, dt);
    }
}

impl App {
    /// Assembles the `App` from the loaded world and the built atlas — the
    /// entity/session half of the startup: the render assets, the starting
    /// player and slimes, the inventory and the persisted session, the debug
    /// env hooks, the menu/network/map setup and the `TR_UI` preset.
    pub(crate) fn new(
        world: World,
        world_path: PathBuf,
        atlas: Atlas,
        font_atlas: Option<tr_render::FontAtlas>,
        lookups: assets::Lookups,
        atlas_stats: (u32, u32, usize, usize),
    ) -> App {
        let mut world = world;
        let clouds = make_clouds(&world, &lookups);
        let sky = make_sky_assets(&atlas);
        let stars = make_stars(&world);
        let player_assets = make_player_assets(&atlas);
        let npc_assets = make_npc_assets(&atlas);
        let player_enabled = std::env::var_os("TR_NOPLAYER").is_none();
        let mut player = player_enabled.then(|| Player::spawn(&world));
        let mut npcs = Vec::new();
        // `TR_POSE` keeps the capture scene empty — a contact-damage NPC
        // would knock the pinned player off the furniture.
        let spawn_npcs = std::env::var_os("TR_POSE").is_none();
        if spawn_npcs && player.is_some() {
            let p = player.as_ref().unwrap();
            let xt = (p.x / TILE_PX) as i32;
            for (i, dx) in [-20, 16, 26].iter().enumerate() {
                let spawned = if world.day_time {
                    Npc::spawn_slime(&world, xt + dx)
                } else {
                    match i {
                        0 => Npc::spawn_zombie(&world, xt + dx),
                        1 => Npc::spawn_eye(&world, xt + dx),
                        _ => Npc::spawn_slime(&world, xt + dx),
                    }
                };
                if let Some(n) = spawned {
                    npcs.push(n);
                }
            }
        }
        let follow_player = player.is_some()
            && std::env::var_os("TR_CAM").is_none()
            && std::env::var_os("TR_PAN").is_none();
        let mut coins = 0u64;
        // The bestiary kill pairs, restored from the sidecar below.
        let mut loaded_kills: Vec<(String, i32)> = Vec::new();
        // The achievement condition states, restored from the port's own
        // `achievements.trdat` (the Rijndael+BSON vanilla format is the
        // documented M6-ACH-1 deviation).
        let mut loaded_achievements = achievements::AchievementManager::ported();
        if let Some(path) = tr_core::save::achievements_path() {
            achievements::load(&path, &mut loaded_achievements);
        }
        // `Player.armor[0..3]`'s item kinds, restored from the sidecar below.
        let mut armor_kinds = [0u16; 3];
        // `Player.dye[0..10]`'s item kinds (sidecar v6), same restore path.
        let mut dye_kinds = [0u16; 10];
        let mut inventory: [ItemSlot; INV] = std::array::from_fn(|i| match i {
            0 => ItemSlot {
                kind: 2, // ItemID.DirtBlock
                name: registry::name(2),
                count: 50,
                prefix: 0,
            },
            1 => ItemSlot {
                kind: 3, // ItemID.StoneBlock
                name: registry::name(3),
                count: 50,
                prefix: 0,
            },
            2 => ItemSlot {
                kind: 8, // ItemID.Torch
                name: registry::name(8),
                count: 99,
                prefix: 0,
            },
            3 => ItemSlot {
                kind: 42, // ItemID.Shuriken
                name: registry::name(42),
                count: 99,
                prefix: 0,
            },
            // The vanilla starter tools (`Player.cs:54356-54360`'s copper
            // trio) plus the copper hammer — the port has no shop yet, so
            // the hammer ships in the kit (the deviation keeps the wall
            // mining reachable; the vanilla starter omits it).
            4 => ItemSlot {
                kind: 3509, // ItemID.CopperPickaxe
                name: registry::name(3509),
                count: 1,
                prefix: 0,
            },
            5 => ItemSlot {
                kind: 3506, // ItemID.CopperAxe
                name: registry::name(3506),
                count: 1,
                prefix: 0,
            },
            6 => ItemSlot {
                kind: 3505, // ItemID.CopperHammer
                name: registry::name(3505),
                count: 1,
                prefix: 0,
            },
            7 => ItemSlot {
                kind: 3507, // ItemID.CopperShortsword
                name: registry::name(3507),
                count: 1,
                prefix: 0,
            },
            _ => ItemSlot {
                kind: 0,
                name: "",
                count: 0,
                prefix: 0,
            },
        });
        if world_path.extension().is_some_and(|e| e == "trwld") {
            if let Some(saved) = persist::load(&world_path) {
                if let Some(p) = player.as_mut() {
                    if !player::collides_player(&world, saved.x, saved.y) {
                        p.x = saved.x;
                        p.y = saved.y;
                    }
                    p.hp = saved.hp.clamp(1, p.max_hp);
                    coins = saved.coins;
                    armor_kinds = saved.armor;
                    dye_kinds = saved.dye;
                    for (i, (kind, count)) in saved.inventory.iter().enumerate().take(INV) {
                        inventory[i] = ItemSlot {
                            kind: *kind,
                            name: registry::name(*kind),
                            count: *count,
                            prefix: 0, // the M6-SAVE sidecar carries (kind, count)
                        };
                    }
                }
                // `NPCKillsTracker.Load` — the bestiary kill pairs ride the
                // sidecar's v5 line; they feed the tracker the App literal
                // below builds from.
                loaded_kills = saved.kills;
            }
        }
        if let Ok(v) = std::env::var("TR_COINS") {
            if let Ok(n) = v.parse::<u64>() {
                coins = n;
            }
        }
        // `TR_DASH=<1|2|3|6>` pins `Player.dashType` — the granting items land
        // with M3-EQUIP-3 (Tabi/EoC Shield/Master Ninja Gear).
        if let Some(d) = std::env::var("TR_DASH")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
        {
            if let Some(p) = player.as_mut() {
                p.dash_type = d.clamp(0, 6);
            }
        }
        // `TR_MOUNT=<id|name>` pins a mount (the granting saddle items are
        // real items now — this stands in for the `miscEquips[3]` slot and
        // for tests/screenshots).
        if let Ok(v) = std::env::var("TR_MOUNT") {
            if let Some(p) = player.as_mut() {
                let name = v.trim().to_ascii_lowercase();
                let id = name.parse::<i32>().ok().or_else(|| match name.as_str() {
                    "rudolph" => Some(0),
                    "bunny" => Some(1),
                    "slime" => Some(3),
                    "scutlix" => Some(9),
                    "unicorn" => Some(10),
                    "basilisk" => Some(14),
                    "horse" | "painted_horse" => Some(40),
                    "majestic_horse" => Some(41),
                    "dark_horse" => Some(42),
                    "pogo" | "pogo_stick" => Some(43),
                    "wofgoat" | "goat" => Some(47),
                    "queenslime" => Some(50),
                    _ => None,
                });
                p.pinned_mount = id;
            }
        }
        // `TR_POSE=sit|sleep` pins the furniture pose for screenshots/tests:
        // the furniture tiles are written beside the spawn and the player is
        // snapped on through the real decode — the sim's own re-validation
        // then holds the pose (no input reaches the headless capture). The
        // tile writes run without a player too (`TR_NOPLAYER` + `TR_POSE`
        // verifies the furniture draw itself).
        if let Ok(v) = std::env::var("TR_POSE") {
            apply_pose_pin(&mut world, player.as_mut(), v.trim());
        }
        // `TR_JUMPS=cloud,sandstorm,...` arms the extra-jump options — the
        // granting accessories (CloudinaBottle 53, SandstorminaBottle 857,
        // BlizzardinaBottle 987, FartinaJar 1724, TsunamiInABottle 3201) and
        // the mounts land with M3-EQUIP-3 / M2-MOUNT-1. `balloon` also lifts
        // the jump statics (`jumpBoost`, `Player.cs:19589-19593`).
        if let Ok(list) = std::env::var("TR_JUMPS") {
            if let Some(p) = player.as_mut() {
                for name in list.split(',') {
                    if name.trim().eq_ignore_ascii_case("balloon") {
                        p.pinned_balloon = true;
                        p.jump_boost = true;
                        continue;
                    }
                    if let Some(j) = crate::player::ExtraJump::parse_name(name) {
                        p.pinned_jumps[j.index()] = true;
                        p.has_jump_options[j.index()] = true;
                        p.can_jump_again[j.index()] = true;
                    }
                }
            }
        }
        // `TR_WINGS=<ArmorIDs.Wing>` / `TR_BOOTS=<1..5>` pin the flight gear —
        // the granting accessories land with M3-EQUIP-3. The wings set
        // `hasWings` (the fall-damage suppressor, `Player.cs:15602`).
        if let Some(w) = std::env::var("TR_WINGS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
        {
            if let Some(p) = player.as_mut() {
                p.wings_logic = w.clamp(0, 49);
                let (fly, _, _, _, _, _) = crate::player::Player::wing_stats(w);
                p.wing_time_max = fly;
                p.has_wings = w > 0;
            }
        }
        if let Some(b) = std::env::var("TR_BOOTS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
        {
            if let Some(p) = player.as_mut() {
                p.pinned_rocket_boots = b.clamp(0, 5);
                p.rocket_boots = b.clamp(0, 5);
            }
        }
        // The Guide's world-gen arrival: one homeless Guide at the spawn
        // point. The NPC persistence is M6-SAVE-1's (the world-file NPC
        // sections), so every load respawns the Guide homeless — the
        // look-for-home tick re-houses it within seconds.
        if let Some(guide) = Npc::spawn_guide(&world, world.spawn_x) {
            npcs.push(guide);
        }
        // Debug spawn override: place the player at a tile coordinate (the camera follows).
        if let Some((tx, ty)) = std::env::var("TR_XY").ok().and_then(|v| parse_xy(&v)) {
            if let Some(p) = player.as_mut() {
                p.teleport(
                    tx as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5,
                    ty as f32 * TILE_PX,
                );
            }
        }
        if std::env::var_os("TR_BOSS").is_some() {
            if let Some(p) = &player {
                // `TR_BOSS=golem` places the Golem body for the family's
                // deterministic shots (the parts ride the body's one-shot
                // init); any other value (or a bare flag) keeps the King
                // Slime. The body takes the `SetDefaults` alpha fade.
                let boss = std::env::var("TR_BOSS").unwrap_or_default();
                if boss == "golem" {
                    let (bw, bh) = NpcKind::Golem.size();
                    let mut g = Npc::new(
                        NpcKind::Golem,
                        p.x + 12.0 * TILE_PX,
                        p.y + p.height - bh,
                        1,
                        0.0,
                    );
                    g.hp = g.max_hp;
                    g.alpha = 255;
                    npcs.push(g);
                } else if let Some(n) =
                    Npc::spawn_king_slime(&world, (p.x / TILE_PX) as i32 + 12)
                {
                    npcs.push(n);
                }
            }
        }
        if let Ok(v) = std::env::var("TR_SLIME_AT") {
            for x in v.split(',').filter_map(|s| s.trim().parse::<i32>().ok()) {
                if let Some(n) = Npc::spawn_slime(&world, x) {
                    npcs.push(n);
                }
            }
        }
        let dev_break = std::env::var("TR_BREAK").ok().and_then(|v| parse_xy(&v));
        // `TR_PLACE=x,y,kind[;x,y,kind…]`: place tiles on the first update, in order.
        let dev_place: Vec<(i32, i32, u16)> = std::env::var("TR_PLACE")
            .ok()
            .map(|v| {
                v.split(';')
                    .filter_map(|entry| {
                        let mut it = entry.split(',');
                        let x = it.next()?.trim().parse::<i32>().ok()?;
                        let y = it.next()?.trim().parse::<i32>().ok()?;
                        let k = it.next()?.trim().parse::<u16>().ok()?;
                        Some((x, y, k))
                    })
                    .collect()
            })
            .unwrap_or_default();
        // `TR_SPOUT=x,y`: build the `FindWaterfalls` half-brick case at a fixed tile (a half
        // brick with full liquid on its right and a clear drop below) so the waterfall
        // overlay can be screenshotted deterministically.
        if let Some((x, y)) = std::env::var("TR_SPOUT").ok().and_then(|v| parse_xy(&v)) {
            for j in 1..=14 {
                edit::kill_tile(&mut world, x, y + j, true);
            }
            edit::kill_tile(&mut world, x, y, true);
            if edit::place_tile(&mut world, x, y, 1) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.flags |= tr_core::tile::FLAG_HALF_BRICK;
                }
            }
            for (dx, dy) in [(1, 0), (1, -1)] {
                if let Some(t) = world.tile_mut(x + dx, y + dy) {
                    t.liquid = 255;
                    // `TR_SPOUT_KIND=<0..3>`: the fall's liquid kind (1 = lava) so
                    // the typed waterfall branches and their `AddLight` glow are
                    // screenshotable.
                    let kind_flag = match std::env::var("TR_SPOUT_KIND").as_deref() {
                        Ok("1") => tr_core::tile::FLAG_LIQUID_LAVA,
                        Ok("2") => tr_core::tile::FLAG_LIQUID_HONEY,
                        Ok("3") => tr_core::tile::FLAG_LIQUID_SHIMMER,
                        _ => 0,
                    };
                    t.flags |= kind_flag;
                }
            }
        }
        // `TR_CHEST`: the chest-panel screenshot fixture — a style-0 chest on
        // the spawn plateau's floor, stocked and opened (`OpenChest`'s state
        // arms: `chest`/`chestX`/`chestY` :33507-33522, `playerInventory` and
        // the crafting close ride the same path, `chest.rs:406-412`).
        let mut tr_chest_store: Vec<craft::WorldChest> = Vec::new();
        let tr_chest = if std::env::var_os("TR_CHEST").is_some() {
            let cx = world.spawn_x + 3;
            // The first solid tile below the spawn point; the 2×2 sits on it.
            let mut gy = world.spawn_y;
            while gy < world.height && !world.is_active(cx, gy) && !world.is_active(cx + 1, gy) {
                gy += 1;
            }
            let cy = gy - 2;
            // The 2×2 chest tile: the style-0 frame corners 0/18 (the 18 px
            // sheet pitch — `chest.rs`'s test fixture shape, `Chest.cs`'s
            // frame layout).
            for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
                if let Some(t) = world.tile_mut(cx + dx, cy + dy) {
                    t.kind = 21;
                    t.frame_x = if dx == 1 { 18 } else { 0 };
                    t.frame_y = if dy == 1 { 18 } else { 0 };
                    t.flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
            let idx = crate::chest::create_chest(&mut tr_chest_store, cx, cy);
            // A few stocked slots so the screenshot shows the item-icon path
            // (`draw_slot`'s existing item draw — verified, not reinvented).
            // The store's `ItemSlot`s since the M3-CHEST-INT upgrade — the
            // same kinds/counts, so the screenshot pixels are unchanged.
            tr_chest_store[idx]
                .slots
                .extend_from_slice(&[
                    crate::item::ItemSlot {
                        kind: 9,
                        name: crate::registry::name(9),
                        count: 6,
                        prefix: 0,
                    },
                    crate::item::ItemSlot {
                        kind: 166,
                        name: crate::registry::name(166),
                        count: 42,
                        prefix: 0,
                    },
                    crate::item::ItemSlot {
                        kind: 71,
                        name: crate::registry::name(71),
                        count: 3,
                        prefix: 0,
                    },
                ]);
            Some((idx as i32, cx, cy))
        } else {
            None
        };
        // The open state lives on the player (`Player.chest`, the C#
        // location — M3-CHEST-INT's unify): the fixture stages it on the
        // spawned player. `TR_CHEST` + `TR_NOPLAYER` stages no open panel
        // (no player = no container state, C#-faithful); the fixture's
        // screenshot combos keep the player.
        if let (Some(p), Some((idx, cx, cy))) = (player.as_mut(), tr_chest) {
            p.chest = idx;
            p.chest_x = cx;
            p.chest_y = cy;
        }
        let has_liquids = world.tiles.iter().any(|t| t.liquid > 0);
        let world_seed = world.seed;
        let mut wind = tr_core::wind::Wind::new(world_seed as u32);
        wind.world_surface = world.world_surface;
        wind.sway_enabled = std::env::var_os("TR_NO_WIND").is_none();
        let liquids_sim = LiquidSystem::new(&world);
        let rain = Rain::new(world_seed);
        let ambient_waterfall = (0.0, 0.0, 0.0);
        let ambient_lavafall = (0.0, 0.0, 0.0);
        let ambient_counter = 0.0f32;
        // `Main.atmo` starts at its surface value; recomputed per draw below.
        let atmo = 1.0_f32;

        // Character name and the session menu (`M6`): screenshots and debug hooks skip
        // the menu so the deterministic output stays unchanged.
        let player_name = std::env::var("TR_NAME").unwrap_or_else(|_| menu::random_name());
        let join_target = std::env::var("TR_JOIN")
            .ok()
            .map(|addr| (addr, std::env::var("TR_JOIN_PW").unwrap_or_default()));
        let auto_host = std::env::var("TR_HOST")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());
        let joined_before = std::env::var_os("TR_JOINED").is_some();
        let force_menu = std::env::var_os("TR_MENU").is_some();
        // `TR_UI=<screen>` presets the front-end flow state after construction
        // (the `TR_PAGE` pattern; the plan Task 5 hook table).
        let tr_ui = std::env::var("TR_UI").ok();
        let skip_menu = (!force_menu && std::env::var_os("TR_SHOT").is_some())
            || std::env::var_os("TR_CAM").is_some()
            || std::env::var_os("TR_HOLD").is_some()
            || std::env::var_os("TR_NOMENU").is_some()
            || std::env::var_os("TR_MAP").is_some()
            || joined_before
            || auto_host.is_some()
            || tr_ui.is_some();
        let menu_state = if skip_menu {
            None
        } else {
            let mut m = menu::Menu::new(player_name.clone());
            m.connecting = join_target.is_some();
            Some(m)
        };
        let mut net_role = NetRole::Off;
        if let Some((addr, pw)) = join_target.as_ref() {
            match net::Client::connect(addr) {
                Ok(c) => {
                    log::info!("joining {addr}");
                    net_role = NetRole::Client(c);
                }
                Err(e) => log::warn!("connect to {addr} failed: {e:#}"),
            }
        }
        if let Some(port) = auto_host {
            match net::Server::start(port, String::new()) {
                Ok(server) => {
                    log::info!("hosting on {}", server.addr());
                    net_role = NetRole::Host(server);
                }
                Err(e) => log::warn!("host on {port} failed: {e:#}"),
            }
        }
        let mut world_map = map::WorldMap::new(&world);
        if std::env::var_os("TR_MAP").is_some() {
            world_map.set_open(true);
        }
        // `TR_AMBIENCE=<SkyEntityType>[,seed]`: queue one forced spawn (the
        // `AmbienceServer.ForceEntitySpawn` path, `AmbienceServer.cs:139-142`)
        // with a pinned seed, so the creature skies stay screenshotable
        // without waiting for the 10-120 s scheduler. Test harness only.
        let ambience_pin = std::env::var("TR_AMBIENCE").ok().and_then(|v| {
            let mut it = v.split(',');
            let ty = it
                .next()
                .map(str::trim)
                .and_then(crate::ambience::parse_sky_entity_type);
            let seed = it
                .next()
                .map(str::trim)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            ty.map(|ty| (ty, seed))
        });
        // The chests roll their loot here — before the struct literal moves
        // `world` (`AddBuriedChest`'s stock inline with the placement, the
        // port's load-time re-roll). The `TR_CHEST` fixture's own stocked
        // chest merges in (its anchor is already stocked — the roll skips).
        let world_chests = crate::chest_gen::generate_world_chests(&world);
        let mut app = App {
            window: None,
            gpu: None,
            renderer: None,
            post: None,
            noise_tex: None,
            sand: crate::sand::Sand::new(1),
            minecart_sparks: crate::minecart::Sparks::new(),
            water_mask: None,
            scene_state: tr_core::scene::SceneState::new(),
            sandstorm: tr_core::scene::Sandstorm::new(world_seed),
            filter_driver: crate::app::filters::FilterDriver::new(),
            world,
            atlas: Some(atlas),
            font_atlas,
            lookups,
            clouds,
            sky,
            // `Main` statics initialise `bgAlphaFrontLayer[0] = bgAlphaFarBackLayer[0] = 1`
            // (forest visible from the start).
            bg_style: 0,
            bg_delay: 0,
            // `Main.instantBGTransitionCounter = 2` on world load.
            bg_instant: 2,
            bg_alpha_front: {
                let mut a = [0.0; 16];
                a[0] = 1.0;
                a
            },
            bg_alpha_far: {
                let mut a = [0.0; 16];
                a[0] = 1.0;
                a
            },
            stars,
            player_assets,
            player,
            follow_player,
            player_step: FixedStepper::new(1.0 / 60.0, 8),
            npc_step: FixedStepper::new(1.0 / 60.0, 8),
            npcs,
            npc_assets,
            npc_timer: 6.0,
            next_worm_id: 1,
            shop_npc: None,
            shop_stock: Vec::new(),
            in_reforge_menu: false,
            reforge_slot: EMPTY_SLOT,
            reforge_cooldown: 0,
            shop_mood: 1.0,
            shop_closed_flag: false,
            creative_powers: crate::creative::CreativePowers::new(),
            debug_panel: crate::debug_panel::DebugPanel::new(),
            favorite_slots: Vec::new(),
            invasion_spawn_timer: 0.0,
            npc_rng: world_seed,
            spawn_no_cycle: false,
            attack_cd: 0.0,
            inventory,
            equip: {
                let mut equip = inv::EquipSlots::empty();
                // The saved character's armor first (`Player.LoadPlayer`); the
                // `TR_ARMOR` env below overrides it so the armor layers stay
                // screenshotable without a drag-and-drop flow (vanilla equips
                // through `Player.EquipArmor`).
                for (slot, kind) in equip.armor.iter_mut().zip(armor_kinds) {
                    if kind != 0 {
                        *slot = ItemSlot {
                            kind,
                            name: registry::name(kind),
                            count: 1,
                            prefix: 0,
                        };
                    }
                }
                // The saved character's dye slots (sidecar v6; each restores as
                // a single stack — the vanilla keeps dyes at stack 1 in
                // `Player.dye`, `ItemSlot.cs:1182-1187`).
                for (slot, kind) in equip.dye.iter_mut().zip(dye_kinds) {
                    if kind != 0 {
                        *slot = ItemSlot {
                            kind,
                            name: registry::name(kind),
                            count: 1,
                            // The sidecar v6 persists kinds only — a restored
                            // dye is prefixless.
                            prefix: 0,
                        };
                    }
                }
                // `TR_ARMOR=copper|iron|silver|gold`: equip a full early-metal set
                // at load so the armor layers stay screenshotable without a
                // drag-and-drop flow (vanilla equips through `Player.EquipArmor`).
                if let Ok(set) = std::env::var("TR_ARMOR") {
                    let ids: [u16; 3] = match set.trim() {
                        "copper" => [89, 80, 76],
                        "iron" => [90, 81, 77],
                        "silver" => [91, 82, 78],
                        "gold" => [92, 83, 79],
                        _ => [0; 3],
                    };
                    for (slot, kind) in equip.armor.iter_mut().zip(ids) {
                        if kind != 0 {
                            *slot = ItemSlot {
                                kind,
                                name: registry::name(kind),
                                count: 1,
                                prefix: 0,
                            };
                        }
                    }
                }
                // `TR_DYE=1007,1015,...`: fill the dye slots positionally — the
                // vanilla equips dyes through the `ItemSlot` swap paths
                // (deferred to the inventory-interaction task), so the plumbing
                // stays screenshotable through the pin.
                if let Ok(list) = std::env::var("TR_DYE") {
                    for (slot, kind) in equip.dye.iter_mut().zip(parse_tr_dye(&list)) {
                        if kind != 0 {
                            *slot = ItemSlot {
                                kind,
                                name: registry::name(kind),
                                count: 1,
                                prefix: 0,
                            };
                        }
                    }
                }
                // `TR_DMOUNT=<kind>`: the saddle-dye stand-in — `miscDyes[3]`
                // (`Player.cs:1484`, the vanilla mount dye slot).
                let dmount = std::env::var("TR_DMOUNT").map_or(0, |s| parse_tr_dmount(&s));
                if dmount != 0 {
                    equip.misc_dyes[3] = ItemSlot {
                        kind: dmount,
                        name: registry::name(dmount),
                        count: 1,
                        prefix: 0,
                    };
                }
                equip
            },
            // `Main.EquipPage`; the page tabs are not clickable yet, so a debug env
            // var selects the page for screenshots (0 = equipment, 2 = misc).
            equip_page: std::env::var("TR_EQUIPPAGE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            selected: 0,
            hotbar_scales: {
                let mut s = [0.75; HOTBAR];
                s[0] = 1.0;
                s
            },
            // `TR_CHEST` opens the chest (`OpenChest` sets `playerInventory`,
            // `Player.cs:33517` — chest.rs:409), so the panel + the player
            // inventory draw together.
            inventory_open: std::env::var_os("TR_INV").is_some() || tr_chest.is_some(),
            kill_tracker: {
                let mut tracker = tr_core::bestiary::KillTracker::new();
                tracker.load_pairs(loaded_kills.into_iter());
                tracker
            },
            bestiary_open: std::env::var_os("TR_BESTIARY").is_some(),
            // `TR_BESTIARY=<n>` also selects entry n (1-based) for the
            // info-page screenshots; `0`/absent stays on the grid.
            bestiary_selected: std::env::var("TR_BESTIARY")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .and_then(|n| {
                    if (1..=crate::bestiary::ENTRIES.len()).contains(&n) {
                        Some(n - 1)
                    } else {
                        None
                    }
                }),
            chat: {
                // `TR_CHAT` opens the chat for the screenshot runs
                // (the vanilla state opens on Enter only).
                let mut chat = chat::ChatMonitor::default();
                if std::env::var_os("TR_CHAT").is_some() {
                    chat.open = true;
                    chat.new_text("Belsin was slain...", crate::chat::COLOR_DEATH);
                    chat.new_text("*Belsin rolls a 42", crate::chat::COLOR_SERVER_MESSAGE);
                    chat.new_text("Belsin hello world", [255, 255, 255]);
                }
                chat
            },
            achievements: {
                // The condition states load below (the sidecar block).
                loaded_achievements
            },
            emotes: {
                // `TR_EMOTES` opens the wheel for the screenshot runs.
                let mut sys = emotes::EmoteSystem::default();
                if std::env::var_os("TR_EMOTES").is_some() {
                    sys.menu_open = true;
                    sys.new_bubble(0);
                }
                sys
            },
            banners: banners::BannerSystem::default(),
            town_look_timer: 0.0,
            reuse_delay: 0.0,
            // The panel ships open with the inventory (`Main.cs:41706-41709`);
            // `TR_NOCRAFT=1` turns it off and `TR_CRAFT=1` forces it back on
            // (the `TR_INV` presence pattern). An open chest closes it
            // (`OpenChest` → `NewCraftingUI.Close`, `chest.rs:408`) — and the
            // chest panel hides under it (`ChestUI.cs:86`).
            crafting_open: (std::env::var_os("TR_NOCRAFT").is_none()
                || std::env::var_os("TR_CRAFT").is_some())
                && tr_chest.is_none(),
            // The load-time loot roll (the hoist above); the `TR_CHEST`
            // store's own chests stay (its anchor gets double-stocked in
            // vanilla too — the fixture stock wins the first slots).
            world_chests: crate::chest_gen::merge_store(world_chests, tr_chest_store),
            chest_scroll_row: 0,
            chest_button_scales: [0.0; ui::chest::BUTTON_COUNT],
            chest_button_hovered: [false; ui::chest::BUTTON_COUNT],
            // `Main.stackSplit`'s cluster — the C# statics start at 0
            // (`Main.cs:1693-1697`); the first idle tick installs the
            // `stackDelay` 7 (`Main.cs:61781-61807`).
            stack_split: 0,
            stack_delay: 0,
            stack_counter: 0,
            super_fast_stack: 0,
            times_tried_to_fast_stack: 0,
            cursor_override: -1,
            chest_names: HashMap::new(),
            chest_default_name: String::new(),
            text_blinker: ui::chest::TextBlinker::default(),
            craft_rng: 1,
            revenge_markers: Vec::new(),
            revenge_tick: 0,
            menu: menu_state,
            front: None,
            front_backup: None,
            players: Vec::new(),
            worlds: Vec::new(),
            front_hint: None,
            front_mouse_down: false,
            session_draft: None,
            vk_grid: std::env::var_os("TR_VK_GRID").is_some(),
            settings: settings::Settings::load(),
            player_name,
            net: net_role,
            remote_players: HashMap::new(),
            local_id: 0,
            net_timer: 0.0,
            net_state: net::client_state::NOT_CONNECTED,
            net_chunks: Vec::new(),
            net_info: None,
            net_status_text: String::new(),
            net_status_max: 0,
            net_status_count: 0,
            net_last_recv: Instant::now(),
            net_dropped: false,
            net_npcs: HashMap::new(),
            net_npc_seen: Vec::new(),
            net_npcs_seen_at: HashMap::new(),
            net_npcs_dirty: false,
            net_npc_gen: HashMap::new(),
            net_items: HashMap::new(),
            net_items_dirty: false,
            net_item_last: HashMap::new(),
            net_reserved: HashMap::new(),
            net_ps_last: net::PlayerState::default(),
            net_beat: 0,
            join_target,
            map: Some(world_map),
            map_grab: None,
            minimap_hover: None,
            // `Main.mapStyle = 1` (`Main.cs:66750`); `TR_MAPSTYLE=<0|1|2>` selects
            // it for screenshots (the `TR_EQUIPPAGE` pattern; an invalid or
            // out-of-range value falls back to 1).
            map_style: std::env::var("TR_MAPSTYLE")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                .filter(|v| (0..=2).contains(v))
                .unwrap_or(1),
            audio: audio::Audio::new(assets::content_dir()),
            music: audio::MusicSystem::new(assets::content_dir()),
            last_hp: 100,
            autosave: 0.0,
            mouse_item: EMPTY_SLOT,
            ui_left_press: false,
            ui_right_press: false,
            mouse_pos: if deterministic_pointer_park() {
                Vec2::new(-64.0, -64.0)
            } else {
                mouse_pin().unwrap_or(Vec2::ZERO)
            },
            cursor: ui::cursor::CursorState::default(),
            os_cursor_hidden: None,
            mine_down: false,
            place_down: false,
            mine_target: None,
            mine_timer: 0.0,
            edit_target: None,
            world_path: world_path.clone(),
            dev_break,
            dev_place,
            camera: Vec2::ZERO,
            zoom: 1.0,
            zoom_pinned: std::env::var_os("TR_ZOOM").is_some(),
            debug: std::env::var("TR_DEBUG").map(|v| v != "0").unwrap_or(false),
            keys: HashSet::new(),
            mouse_down: false,
            last_mouse: Vec2::ZERO,
            last_frame: Instant::now(),
            frame_history: VecDeque::with_capacity(256),
            last_stats: tr_render::sprite::RenderStats::default(),
            bench_frames: std::env::var("TR_BENCH").ok().and_then(|v| v.parse().ok()),
            bench_samples: Vec::new(),
            frame_count: 0,
            shot_path: std::env::var("TR_SHOT").ok().map(PathBuf::from),
            shot_at: std::env::var("TR_SHOT_FRAME")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            atlas_stats,
            light: LightMap::new(),
            scratch: DrawScratch::default(),
            tile_stats: TileDrawStats::default(),
            stats_on: std::env::var("TR_STATS").is_ok(),
            stats_acc: StatsAcc::default(),
            debug_cache: DebugCache::default(),
            light_mode: std::env::var("TR_LIGHT_MODE")
                .ok()
                .and_then(|v| parse_light_mode(&v))
                .unwrap_or(LightMode::Color),
            // `Main.mouseTextColor` starts at 225 and drifts toward 255.
            mouse_text_color: 225,
            mouse_text_color_dir: 1,
            // `Main.demonTorch = 1f; Main.demonTorchDir = 1` (Main.Initialize).
            demon_torch: 1.0,
            demon_torch_dir: 1.0,
            // `Main.DiscoR = 255` (Main.Initialize); G/B and the style start at 0.
            disco: [255, 0, 0],
            disco_style: 0,
            light_rng_seed: world_seed | 1,
            anim_time: 0.0,
            has_liquids,
            liquids: liquids_sim,
            liquid_step: FixedStepper::new(1.0 / 60.0, 8),
            liquid_tick: 0,
            waterfall: tr_core::waterfall::WaterfallManager::new(),
            waterfall_step: FixedStepper::new(1.0 / 60.0, 8),
            wiring: tr_core::wiring::Wiring::new(),
            wiring_step: FixedStepper::new(1.0 / 60.0, 8),
            te_step: FixedStepper::new(1.0 / 60.0, 8),
            growth_step: FixedStepper::new(1.0 / 60.0, 8),
            growth_rng: tr_core::tile_light::FastRandom::new(world_seed ^ 0x2545_F491_4F6C_DD1D),
            ore_tiers: tr_core::ores::SavedOreTiers::new(),
            rain,
            ambient_waterfall,
            ambient_lavafall,
            ambient_counter,
            wind,
            world_tick_step: FixedStepper::new(1.0 / 60.0, 8),
            atmo,
            scene_metrics: tr_core::scene::SceneMetrics::new(),
            grapple: None,
            ambience: {
                let mut ambience = crate::ambience::Ambience::new();
                if let Some((ty, seed)) = ambience_pin {
                    ambience.forced_spawn_for_tests(ty, -1, seed);
                }
                ambience
            },
            ambience_rng: world_seed ^ 0x0A5C_E17A_1C1E_5EED,
            tree_wind_counter: 0.0,
            grass_wind_counter: 0.0,
            sunflower_wind_counter: 0.0,
            sunflower_wind_step: FixedStepper::new(1.0 / 60.0, 8),
            leaves: Vec::new(),
            liquid_alpha: [0.0; 15],
            items: Vec::new(),
            item_rng: world_seed ^ 0x5DEE_CE66_D1A4_9B5F,
            item_step: FixedStepper::new(1.0 / 60.0, 8),
            swing: 0.0,
            swing_style: 0,
            swing_aim: 0.0,
            shot_fired: false,
            swing_max: 0.0,
            place_prev_down: false,
            place_released: false,
            mine_prev_down: false,
            mine_released: false,
            item_time: 0.0,
            coins,
            projectiles: ProjectilePool::new(),
            tomb_falls: Vec::new(),
            lost_coins: 0,
            lost_coin_string: String::new(),
            death_text: String::new(),
            invasion_progress: -1,
            invasion_progress_max: 0,
            invasion_progress_icon: 0,
            invasion_progress_display_left: 0,
            invasion_progress_alpha: 0.0,
            invasion_progress_near: false,
            tomb_texts: Vec::new(),
            projectile_step: FixedStepper::new(1.0 / 60.0, 8),
            dev_throw: std::env::var_os("TR_THROW").is_some(),
            world_key: None,
            hell_black_box_bottom: 0.0,
        };
        // The faulty-lamp roll stream seeds from the world (`Main.rand`'s
        // role in `Wiring`'s random branch, `Wiring.cs:812`).
        app.wiring.seed_faulty_rng(world_seed | 1);
        // `TR_UI=<screen>`: preset the front-end flow state for screenshots (the
        // `TR_PAGE` pattern — the plan Task 5 hook table).
        if let Some(preset) = tr_ui.as_deref().and_then(tr_ui_preset) {
            app.preset_ui(preset);
        }
        app
    }
}

/// Loads the world (`save::load` for the port's `.trwld`, `wld::load` for the
/// retail `.wld`), applies the `TR_DAY` clock override and frames the loaded
/// tiles and walls — the load half of the startup, verbatim from `main`.
pub(crate) fn load_world(world_path: &Path) -> Result<World> {
    let t0 = Instant::now();
    let mut world = if world_path.extension().is_some_and(|e| e == "trwld") {
        save::load(&world_path)?
    } else {
        wld::load(&world_path)?
    };
    log::info!(
        "loaded '{}' {}x{} ({} tiles, {:.1} MB) in {:.0} ms",
        world.name,
        world.width,
        world.height,
        world.tiles.len(),
        world.memory_bytes() as f32 / 1e6,
        t0.elapsed().as_secs_f32() * 1000.0
    );
    // `TR_POOL=tx,ty,w,h,kind`: fill a rectangle with liquid at load (0 water,
    // 1 lava, 2 honey, 3 shimmer) so the honey/shimmer draw passes and the
    // waterfall types stay screenshotable without the biomes that produce them.
    if let Ok(v) = std::env::var("TR_POOL") {
        let mut it = v.split(',');
        let vals: Result<Vec<i64>, _> = it.map(|x| x.trim().parse::<i64>()).collect();
        if let Ok(vals) = vals
            && vals.len() == 5
        {
            let (tx, ty, tw, th, kind) = (
                vals[0] as i32,
                vals[1] as i32,
                vals[2] as i32,
                vals[3] as i32,
                vals[4] as i32,
            );
            let kind_flag: u8 = match kind {
                1 => tr_core::tile::FLAG_LIQUID_LAVA,
                2 => tr_core::tile::FLAG_LIQUID_HONEY,
                3 => tr_core::tile::FLAG_LIQUID_SHIMMER,
                _ => 0,
            };
            for x in tx..(tx + tw) {
                for y in ty..(ty + th) {
                    if let Some(t) = world.tile_mut(x, y) {
                        // A pure liquid cell: the tile body stays inactive and
                        // only the liquid byte plus the kind flags are set.
                        t.kind = 0;
                        t.flags &= !(tr_core::tile::FLAG_LIQUID_LAVA
                            | tr_core::tile::FLAG_LIQUID_HONEY
                            | tr_core::tile::FLAG_LIQUID_SHIMMER);
                        t.flags |= kind_flag;
                        t.liquid = 255;
                    }
                }
            }
            world.revision = world.revision.wrapping_add(1);
        }
    }
    // `TR_PAINT=tx,ty,w,h,paint[,wall]`: paint a rectangle of tiles (and walls
    // with the optional 6th value) at load, so the paint recolor and the wall
    // paint path stay screenshotable without a paint item. paint 29 = shadow,
    // 30 = negative, 31 = illuminant.
    if let Ok(v) = std::env::var("TR_PAINT") {
        let mut it = v.split(',');
        let vals: Result<Vec<i64>, _> = it.map(|x| x.trim().parse::<i64>()).collect();
        if let Ok(vals) = vals
            && (vals.len() == 5 || vals.len() == 6)
        {
            let (tx, ty, tw, th, paint) = (
                vals[0] as i32,
                vals[1] as i32,
                vals[2] as i32,
                vals[3] as i32,
                vals[4] as u8,
            );
            let wall_paint = if vals.len() == 6 { vals[5] as u8 } else { 0 };
            for x in tx..(tx + tw) {
                for y in ty..(ty + th) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.set_paint(paint);
                    }
                    if wall_paint != 0 {
                        world.set_wall_paint(x, y, wall_paint);
                    }
                }
            }
            world.revision = world.revision.wrapping_add(1);
        }
    }
    // `TR_FILL=tx,ty,w,h,kind[;...]`: fill rectangles with a tile kind at
    // load (the `TR_POOL` shape for tiles) — builds the floor/ceiling rows
    // the wooden set's anchors need; `TR_PAINT` writes paint colours, not
    // tile kinds.
    if let Ok(v) = std::env::var("TR_FILL") {
        apply_tr_fill(&mut world, &v);
    }
    // `TR_PLACE=x,y,kind[,style[,dir]][;...]`: place tile objects through the
    // Phase-2 route (`edit::place_item_tile`, the player's own RMB call) for
    // the object screenshots.
    if let Ok(v) = std::env::var("TR_PLACE") {
        apply_tr_places(&mut world, &parse_tr_place(&v));
    }
    // `TR_TOGGLE=x,y[,n][;...]`: n presses of the Phase-4 interaction
    // dispatcher (`edit::interact_at`) — the door closed→open→closed scenario.
    if let Ok(v) = std::env::var("TR_TOGGLE") {
        apply_tr_toggles(&mut world, &parse_tr_toggle(&v));
    }
    // `TR_TILES=x,y,kind,fx,fy[;...]`: write tiles with explicit frames at
    // load — the render fixture for the M1.4-TO-4 kinds (the plan stages the
    // scene by direct World writes, not through Phase 1 placement).
    if let Ok(v) = std::env::var("TR_TILES") {
        apply_tr_tiles(&mut world, &parse_tr_tiles(&v));
    }
    // TR_DAY also sets the world clock so NPC day/night logic matches the pinned sky.
    if let Some(d) = std::env::var("TR_DAY")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(0.0, 1.0))
    {
        world.day_time = d >= 0.5;
        world.time = if world.day_time {
            d as f64 * 54000.0
        } else {
            (1.0 - d as f64) * 32400.0
        };
    }
    // `TR_MOON=<0..7>` pins `Main.moonPhase` — the night light floor follows it
    // (`Main.cs:63792-63829`), so the full/new-moon A/B stays screenshotable.
    if let Some(p) = std::env::var("TR_MOON")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
    {
        world.moon_phase = p & 7;
    }
    // `TR_TIME=<ticks>` pins the raw world clock — the dawn/dusk windows the
    // `TR_DAY` day-half mapping cannot reach (the sunrise visibility spans
    // ticks 0..10800, the sunset 43200..54000).
    if let Some(t) = std::env::var("TR_TIME")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        world.time = t;
        world.day_time = t < 54000.0;
    }
    // `TR_SUNDIAL=1`/`TR_MOONDIAL=1`: arm the dial fast-forwards at start
    // (BG-21's screenshot pins). The flags only — `sundialing()` would also
    // start the 8-day cooldown and mute the dial glow the shot pair needs.
    if std::env::var("TR_SUNDIAL").is_ok() {
        world.fast_forward_to_dawn = true;
    }
    if std::env::var("TR_MOONDIAL").is_ok() {
        world.fast_forward_to_dusk = true;
    }
    // `TR_ECHO=tx,ty,w,h`: echo-coat a rectangle — the `invisibleBlock` +
    // `invisibleWall` bits (`WorldGen.paintCoatTile` case 2 / `paintCoatWall`,
    // `WorldGen.cs:44134-44217`) — so the light pass's invisible handling stays
    // screenshotable without a coating item.
    if let Ok(v) = std::env::var("TR_ECHO") {
        let mut it = v.split(',');
        let vals: Result<Vec<i64>, _> = it.map(|x| x.trim().parse::<i64>()).collect();
        if let Ok(vals) = vals
            && vals.len() == 4
        {
            let (tx, ty, tw, th) = (
                vals[0] as i32,
                vals[1] as i32,
                vals[2] as i32,
                vals[3] as i32,
            );
            for x in tx..(tx + tw) {
                for y in ty..(ty + th) {
                    world.set_invisible_block(x, y, true);
                    world.set_invisible_wall(x, y, true);
                }
            }
            world.revision = world.revision.wrapping_add(1);
        }
    }

    let t_frame = Instant::now();
    let framed = framing::frame_world(&mut world);
    let tile_frame_ms = t_frame.elapsed().as_secs_f32() * 1000.0;
    let t_walls = Instant::now();
    let walled = framing::frame_walls(&mut world);
    let wall_frame_ms = t_walls.elapsed().as_secs_f32() * 1000.0;
    world.compute_column_surface();
    world.compute_chunks();
    // Torches are important tiles; generated saves written before the hotbar existed
    // may lack this bit.
    if world.frame_important.len() > 4 {
        world.frame_important[4] = true;
    }
    let slopes = world
        .tiles
        .iter()
        .filter(|t| t.active() && (t.slope() != 0 || t.half_brick()))
        .count();
    log::info!(
        "framed {framed} tiles in {tile_frame_ms:.0} ms, {walled} walls in {wall_frame_ms:.0} ms, {slopes} slopes"
    );
    log::info!(
        "spawn {},{}, surface {:.1}, rock layer {:.1}, time {:.0} ({})",
        world.spawn_x,
        world.spawn_y,
        world.world_surface,
        world.rock_layer,
        world.time,
        if world.day_time { "day" } else { "night" }
    );
    Ok(world)
}

// --- Deterministic load hooks for the object screenshots --------------------
//
// `TR_FILL=tx,ty,w,h,kind[;...]` fills rectangles with a tile kind at load
// (the `TR_POOL` shape for tiles): it builds the floor/ceiling rows the
// wooden set's anchors need — `TR_PAINT` writes paint colours, not tile kinds.
//
// `TR_PLACE=x,y,kind[,style[,dir]][;...]` places tile objects through the
// Phase-2 route (`edit::place_item_tile`, the same call the player's RMB
// takes), semicolon-separated so one screenshot can hold several objects.
//
// `TR_TOGGLE=x,y[,n][;...]` presses the Phase-4 interaction dispatcher
// (`edit::interact_at`) n times at one tile — the closed→open→closed door
// scenario.
//
// `TR_TILES=x,y,kind,fx,fy[;...]` writes single cells with explicit frames —
// the render fixture for the M1.4-TO-4 kinds (sunflower 2x4, pots, clay pot,
// herbs, piles, planter box), staged without the placement route.

/// One `TR_PLACE` group: place this tile kind through the registry route.
struct TrPlace {
    x: i32,
    y: i32,
    kind: u16,
    style: i32,
    dir: i32,
}

/// One `TR_TOGGLE` group: `n` interaction presses at one tile.
struct TrToggle {
    x: i32,
    y: i32,
    n: u32,
}

/// Comma-separated `i64`s (`None` when any token is not a number) — the
/// shared tail of every `TR_` hook parser.
fn tr_nums(s: &str) -> Option<Vec<i64>> {
    s.split(',').map(|v| v.trim().parse::<i64>().ok()).collect()
}

fn parse_tr_place(v: &str) -> Vec<TrPlace> {
    v.split(';')
        .filter_map(|group| {
            // `[x, y, kind]` with the defaults of `Item.place_style` and the
            // wired direction (Decisions #6/#7); longer groups override them.
            let nums = tr_nums(group)?;
            match nums[..] {
                [x, y, kind] => Some(TrPlace {
                    x: x as i32,
                    y: y as i32,
                    kind: kind as u16,
                    style: 0,
                    dir: 1,
                }),
                [x, y, kind, style] => Some(TrPlace {
                    x: x as i32,
                    y: y as i32,
                    kind: kind as u16,
                    style: style as i32,
                    dir: 1,
                }),
                [x, y, kind, style, dir] => Some(TrPlace {
                    x: x as i32,
                    y: y as i32,
                    kind: kind as u16,
                    style: style as i32,
                    dir: dir as i32,
                }),
                _ => None,
            }
        })
        .collect()
}

fn parse_tr_toggle(v: &str) -> Vec<TrToggle> {
    v.split(';')
        .filter_map(|group| match tr_nums(group)?[..] {
            [x, y] => Some(TrToggle {
                x: x as i32,
                y: y as i32,
                n: 1,
            }),
            [x, y, n] => Some(TrToggle {
                x: x as i32,
                y: y as i32,
                n: n.max(0) as u32,
            }),
            _ => None,
        })
        .collect()
}

fn apply_tr_fill(world: &mut World, v: &str) {
    let mut touched = false;
    for group in v.split(';') {
        let Some(nums) = tr_nums(group) else {
            continue;
        };
        if nums.len() != 5 {
            continue;
        }
        let (tx, ty, tw, th, kind) = (
            nums[0] as i32,
            nums[1] as i32,
            nums[2] as i32,
            nums[3] as i32,
            nums[4] as u16,
        );
        for x in tx..(tx + tw) {
            for y in ty..(ty + th) {
                if let Some(t) = world.tile_mut(x, y) {
                    // A fresh solid cell, the way worldgen writes one.
                    *t = tr_core::tile::Tile {
                        kind,
                        ..Default::default()
                    };
                    t.flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
        touched = true;
    }
    if touched {
        world.revision = world.revision.wrapping_add(1);
    }
}

fn apply_tr_places(world: &mut World, ops: &[TrPlace]) {
    for op in ops {
        let ok = edit::place_item_tile(world, op.x, op.y, op.kind, op.style, op.dir);
        log::info!(
            "TR_PLACE {},{} kind {} (style {}, dir {}) -> {}",
            op.x,
            op.y,
            op.kind,
            op.style,
            op.dir,
            if ok { "placed" } else { "refused" }
        );
    }
    if !ops.is_empty() {
        world.revision = world.revision.wrapping_add(1);
    }
}

/// One `TR_TILES` cell: write this exact tile kind with these exact frames —
/// the render fixture's direct world write (the plan's "tiles and frames are
/// written into the World directly"), bypassing placement and framing. The
/// 4-token `x,y,w,h` form clears the rectangle to air (and wipes its walls) so
/// the fixture can stage a clean stage on lived-in terrain.
struct TrTile {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    kind: i64,
    fx: i16,
    fy: i16,
}

fn parse_tr_tiles(v: &str) -> Vec<TrTile> {
    v.split(';')
        .filter_map(|group| match tr_nums(group)?[..] {
            [x, y, w, h] => Some(TrTile {
                x: x as i32,
                y: y as i32,
                w: w as i32,
                h: h as i32,
                kind: -1,
                fx: 0,
                fy: 0,
            }),
            [x, y, kind, fx, fy] => Some(TrTile {
                x: x as i32,
                y: y as i32,
                w: 1,
                h: 1,
                kind,
                fx: fx as i16,
                fy: fy as i16,
            }),
            _ => None,
        })
        .collect()
}

fn apply_tr_tiles(world: &mut World, ops: &[TrTile]) {
    let mut touched = false;
    for op in ops {
        for dy in 0..op.h {
            for dx in 0..op.w {
                let (x, y) = (op.x + dx, op.y + dy);
                if op.kind >= 0 {
                    let kind = op.kind as u16;
                    // Every fixture kind is `Main.tileFrameImportant` in vanilla
                    // (`Main.cs:7415/:7632/:7663-7685`), so the written frames
                    // survive the post-hook `frame_world` even when the save's
                    // table lacks the bit.
                    if (kind as usize) < world.frame_important.len() {
                        world.frame_important[kind as usize] = true;
                    }
                }
                let Some(t) = world.tile_mut(x, y) else {
                    continue;
                };
                if op.kind < 0 {
                    // The clear: air, no wall — a blank stage cell.
                    *t = tr_core::tile::Tile::default();
                    touched = true;
                    continue;
                }
                let kind = op.kind as u16;
                *t = tr_core::tile::Tile {
                    kind,
                    frame_x: op.fx,
                    frame_y: op.fy,
                    ..Default::default()
                };
                t.flags |= tr_core::tile::FLAG_ACTIVE;
                touched = true;
            }
        }
        if op.kind >= 0 {
            log::info!(
                "TR_TILES {},{} kind {} frame ({},{}) {}x{}",
                op.x,
                op.y,
                op.kind,
                op.fx,
                op.fy,
                op.w,
                op.h
            );
        }
    }
    if touched {
        world.revision = world.revision.wrapping_add(1);
    }
}

fn apply_tr_toggles(world: &mut World, ops: &[TrToggle]) {
    for op in ops {
        for _ in 0..op.n {
            // No player exists at load: `direction` is 1 (right-facing) and
            // no entity can block the door cells.
            let hit = edit::interact_at(world, op.x, op.y, 1, &|_, _| false);
            match hit {
                edit::TileInteraction::None => {
                    log::info!("TR_TOGGLE {},{} -> none", op.x, op.y)
                }
                edit::TileInteraction::DoorOpened { object_drops } => log::info!(
                    "TR_TOGGLE {},{} -> opened ({} drops)",
                    op.x,
                    op.y,
                    object_drops.len()
                ),
                edit::TileInteraction::DoorClosed { object_drops } => log::info!(
                    "TR_TOGGLE {},{} -> closed ({} drops)",
                    op.x,
                    op.y,
                    object_drops.len()
                ),
                edit::TileInteraction::DoorBlocked { object_drops } => log::info!(
                    "TR_TOGGLE {},{} -> blocked ({} drops)",
                    op.x,
                    op.y,
                    object_drops.len()
                ),
                edit::TileInteraction::DialStarted { sundial } => log::info!(
                    "TR_TOGGLE {},{} -> {} started",
                    op.x,
                    op.y,
                    if sundial { "sundial" } else { "moondial" }
                ),
                edit::TileInteraction::DialRefused => {
                    log::info!("TR_TOGGLE {},{} -> dial refused", op.x, op.y)
                }
                edit::TileInteraction::CandleToggled => {
                    log::info!("TR_TOGGLE {},{} -> candle toggled", op.x, op.y)
                }
                edit::TileInteraction::ChairSit { .. } => {
                    log::info!("TR_TOGGLE {},{} -> chair sit", op.x, op.y)
                }
                edit::TileInteraction::BedSleep { .. } => {
                    log::info!("TR_TOGGLE -> bed sleep (no player at load)")
                }
                edit::TileInteraction::BedInteract { set_spawn, .. } => log::info!(
                    "TR_TOGGLE {},{} -> bed ({})",
                    op.x,
                    op.y,
                    if set_spawn {
                        "spawn half"
                    } else {
                        "sleep half"
                    }
                ),
                edit::TileInteraction::ClockRead { text } => {
                    log::info!("TR_TOGGLE {},{} -> clock {text}", op.x, op.y)
                }
                // M7-EXTRACT-1: the extractinator's item-anchored arm has no
                // load-time meaning (no held item) — the dispatch only reports
                // the tile kind.
                edit::TileInteraction::Extractinator => {
                    log::info!("TR_TOGGLE {},{} -> extractinator", op.x, op.y)
                }
                edit::TileInteraction::BankOpen { bank } => {
                    log::info!("TR_TOGGLE {},{} -> bank {bank}", op.x, op.y)
                }
            }
        }
    }
    if !ops.is_empty() {
        world.revision = world.revision.wrapping_add(1);
    }
}

/// Builds the sprite atlas and the sheet lookups from the world's used tile
/// and wall types plus the UI fonts — the atlas+fonts half of the startup.
pub(crate) fn build_atlas(
    world: &World,
    assets_dir: &Path,
) -> Result<(Atlas, Option<tr_render::FontAtlas>, assets::Lookups, (u32, u32, usize, usize))> {
    let (mut tile_ids, wall_ids) = assets::usage(&world);
    // The hotbar can place these even when the loaded world does not use them yet.
    tile_ids.insert(4); // torch
    tile_ids.insert(7); // copper ore
    // `TR_POSE=sit|sleep` writes the chair/bed beside the spawn after the
    // usage scan — the pose pin's sheets ride along.
    if std::env::var_os("TR_POSE").is_some() {
        tile_ids.insert(15); // chair
        tile_ids.insert(79); // bed
    }
    log::info!(
        "world uses {} tile types and {} wall types",
        tile_ids.len(),
        wall_ids.len()
    );

    let t1 = Instant::now();
    let mut images = assets::load_world_images(&assets_dir, &tile_ids, &wall_ids)?;
    let fonts = assets::load_fonts();
    let font = tr_render::atlas::font_sheet();
    images.push(font.0);
    log::info!(
        "decoded {} sheets in {:.0} ms",
        images.len(),
        t1.elapsed().as_secs_f32() * 1000.0
    );

    let t2 = Instant::now();
    let atlas = Atlas::build(images, 8192);
    log::info!(
        "atlas {}x{} ({:.1} MB, {} sheets) in {:.0} ms",
        atlas.width,
        atlas.height,
        atlas.memory_bytes() as f32 / 1e6,
        atlas.sheets.len(),
        t2.elapsed().as_secs_f32() * 1000.0
    );
    if atlas.height > 8192 {
        log::warn!("atlas exceeds 8192 px in height; some sheets may be cut off on strict GPUs");
    }

    let lookups = assets::Lookups::build(
        &atlas,
        tile_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_tile(id))),
        wall_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_wall(id))),
        fonts.ui,
        fonts.item,
        fonts.death,
        fonts.combat_text,
        fonts.combat_crit,
    );

    let atlas_stats = (
        atlas.width,
        atlas.height,
        atlas.memory_bytes(),
        atlas.sheets.len(),
    );
    Ok((atlas, fonts.atlas, lookups, atlas_stats))
}

/// The world-dump diagnostics: `TR_DUMP_TILES` prints the raw tile grid and
/// the `TR_SCAN_*` family locates the densest liquid/torch/self-lit 40x40
/// windows, for deterministic screenshots.
pub(crate) fn log_world_diagnostics(world: &World) {
    if std::env::var("TR_DUMP_TILES").is_ok() {
        let cx = std::env::var("TR_DUMP_X")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(world.width / 2);
        let cy = std::env::var("TR_DUMP_Y")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(world.spawn_y + 12);
        for y in cy..cy + 8 {
            let mut line = String::new();
            for x in cx..cx + 16 {
                let t = world.tile(x, y).copied().unwrap_or_default();
                line.push_str(&format!(
                    "{}:{}:{}/{}/w{}L{} ",
                    if t.active() { t.kind } else { 999 },
                    t.slope(),
                    t.frame_x,
                    t.frame_y,
                    t.wall,
                    t.liquid
                ));
            }
            log::info!("row {y}: {line}");
        }
    }
    if std::env::var("TR_SCAN_WATER").is_ok() {
        // Finds the densest 40x40 window of liquid cells, for flow/liquid screenshots.
        // The map edges (oceans) are skipped so the inland lake wins.
        const STEP: i32 = 40;
        let mut best = (0usize, 0i32, 0i32);
        let mut y = 300;
        while y + STEP < world.height {
            let mut x = 300;
            while x + STEP < world.width - 300 {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        if world.tile(xx, yy).is_some_and(|t| t.liquid > 0) {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "densest liquid window: {} at {},{} -> TR_CAM={},{}",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }
    if std::env::var("TR_SCAN_TORCH").is_ok() {
        // Finds the densest 40x40 window of torches, for lighting-mode screenshots.
        const STEP: i32 = 40;
        let mut best = (0usize, 0i32, 0i32);
        let mut y = 0;
        while y + STEP < world.height {
            let mut x = 0;
            while x + STEP < world.width {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        if world
                            .tile(xx, yy)
                            .is_some_and(|t| t.active() && t.kind == 4)
                        {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "densest torch window: {} at {},{} -> TR_CAM={},{}",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }
    // `TR_SCAN_LIGHT`: finds the 40x40 tile window with the most self-lit tiles
    // (torches, glowing mushrooms, lava...) so screenshots can target a lit cave.
    if std::env::var("TR_SCAN_LIGHT").is_ok() {
        const STEP: i32 = 40;
        let bounds = std::env::var("TR_SCAN_Y")
            .ok()
            .and_then(|v| {
                let (a, b) = v.split_once(',')?;
                Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<i32>().ok()?))
            })
            .unwrap_or((0, world.height));
        let mut best = (0usize, 0i32, 0i32);
        let mut y = bounds.0.max(0);
        while y + STEP < bounds.1.min(world.height) {
            let mut x = 0;
            while x + STEP < world.width {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        let t = world.tile(xx, yy).copied().unwrap_or_default();
                        if !t.active() {
                            continue;
                        }
                        let e = td::EMISSIVE[t.kind as usize];
                        let emissive =
                            (t.kind as usize) < td::TILE_COUNT && (e[0].max(e[1]).max(e[2]) > 0.2);
                        if emissive || t.liquid_kind() == tr_core::tile::LiquidKind::Lava {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "brightest 40x40 window: {} tiles at {},{} -> TR_CAM={},{} (tiles)",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }
}

/// The atlas diagnostics: `TR_CHECK_FRAMES` warns about near-empty tile and
/// wall frames, `TR_ATLAS_DUMP` writes the atlas PNG and logs key sheet rects.
pub(crate) fn log_atlas_diagnostics(atlas: &Atlas, lookups: &assets::Lookups) {
    if std::env::var("TR_CHECK_FRAMES").is_ok() {
        for (kind, slot) in lookups.tiles.iter().enumerate() {
            let Some(tt) = slot else { continue };
            if let Some(rect) = atlas.sheet(&tt.sheet) {
                let fx = rect.plain_frame.0.max(0) as u32;
                let fy = rect.plain_frame.1.max(0) as u32;
                let mut opaque = 0;
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        let px = rect.x + fx + x;
                        let py = rect.y + fy + y;
                        let i = ((py * atlas.width + px) * 4 + 3) as usize;
                        if atlas.pixels[i] >= 250 {
                            opaque += 1;
                        }
                    }
                }
                if opaque < 200 {
                    log::warn!(
                        "tile {kind} ({}) plain frame is {opaque}/256 opaque",
                        tt.sheet
                    );
                }
            }
        }
        for (wall, slot) in lookups.walls.iter().enumerate() {
            let Some(tt) = slot else { continue };
            if let Some(rect) = atlas.sheet(&tt.sheet) {
                let fx = rect.plain_frame.0.max(0) as u32;
                let fy = rect.plain_frame.1.max(0) as u32;
                let mut opaque = 0;
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        let px = rect.x + fx + x;
                        let py = rect.y + fy + y;
                        let i = ((py * atlas.width + px) * 4 + 3) as usize;
                        if atlas.pixels[i] >= 250 {
                            opaque += 1;
                        }
                    }
                }
                if opaque < 200 {
                    log::warn!(
                        "wall {wall} ({}) plain frame is {opaque}/256 opaque",
                        tt.sheet
                    );
                }
            }
        }
    }

    if let Ok(path) = std::env::var("TR_ATLAS_DUMP") {
        image::save_buffer(
            &path,
            &atlas.pixels,
            atlas.width,
            atlas.height,
            image::ColorType::Rgba8,
        )
        .ok();
        log::info!("atlas dumped to {path}");
        for name in [
            "Tiles_0",
            "Tiles_1",
            "Tiles_2",
            "Wall_1",
            "Background_0",
            "LiquidSlope_0",
            "Liquid_0",
            "Misc/water_0",
        ] {
            if let Some(s) = atlas.sheet(name) {
                log::info!(
                    "{name}: atlas rect {}x{} at {},{} plain {:?}",
                    s.width,
                    s.height,
                    s.x,
                    s.y,
                    s.plain_frame
                );
            }
        }
    }
}

pub(crate) fn terrstaria_world() -> Result<PathBuf> {
    let path = save::default_world_path("Terrustaria")
        .context("cannot resolve Documents/My Games/Terrustaria")?;
    let regen = std::env::var_os("TR_REGEN").is_some();
    if !path.exists() || regen {
        let seed = std::env::var("TR_SEED")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1);
        let t = Instant::now();
        let world = worldgen::generate(&worldgen::GenOptions {
            seed,
            ..Default::default()
        });
        save::save(&path, &world)?;
        log::info!(
            "generated '{}' (seed {seed}) in {:.0} ms -> {}",
            world.name,
            t.elapsed().as_secs_f32() * 1000.0,
            path.display()
        );
    }
    Ok(path)
}

/// Locates the bundled retail `Content` directory: next to the executable first (the
/// release layout), then the project tree.
pub(crate) fn find_content_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let mut candidates = Vec::new();
    if let Some(exe) = exe.as_ref().and_then(|p| p.parent()) {
        candidates.push(exe.join("assets/Content"));
        candidates.push(exe.join("../../assets/Content"));
    }
    candidates.push(PathBuf::from("assets/Content"));
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/Content"));
    candidates.into_iter().find(|p| p.join("Images").is_dir())
}

#[cfg(test)]
mod tr_hooks {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// Serializes env-var mutation across the binary's parallel test threads:
    /// `load_world`'s pins are process-global reads, so any test that sets or
    /// removes a `TR_*` variable must hold this lock for the whole call.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// The core `edit.rs` doorway fixture: stone floor y = 20, ceiling y = 16
    /// (a 3-tall opening at rows 17..=19), deep enough inside the world for
    /// `CanPlace`'s 5-tile edge margin (`TileObject.cs:215`).
    fn doorway_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 0..40 {
            for y in [16, 20] {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn tr_place_parsing_defaults_and_groups() {
        // `[x, y, kind]` groups default to style 0 / dir 1 (Decisions #6/#7);
        // 4- and 5-number groups override style and style+dir.
        let ops = parse_tr_place("10,19,18;14,19,21,0,-1");
        assert_eq!(ops.len(), 2);
        assert_eq!(
            (ops[0].x, ops[0].y, ops[0].kind, ops[0].style, ops[0].dir),
            (10, 19, 18, 0, 1)
        );
        assert_eq!(
            (ops[1].x, ops[1].y, ops[1].kind, ops[1].style, ops[1].dir),
            (14, 19, 21, 0, -1)
        );
        // Malformed groups are dropped; well-formed ones survive.
        assert_eq!(parse_tr_place("x,y,18;5,19,4").len(), 1);
        assert!(parse_tr_place("").is_empty());
    }

    #[test]
    fn tr_toggle_parsing_default_count() {
        let ops = parse_tr_toggle("2026,198;7,9,3");
        assert_eq!(ops.len(), 2);
        assert_eq!((ops[0].x, ops[0].y, ops[0].n), (2026, 198, 1));
        assert_eq!((ops[1].x, ops[1].y, ops[1].n), (7, 9, 3));
        // Wrong arity is dropped; a negative count clamps to 0 presses.
        assert!(parse_tr_toggle("1,2,3,4").is_empty());
        assert_eq!(parse_tr_toggle("1,2,-5")[0].n, 0);
    }

    #[test]
    fn tr_hook_chain_places_and_toggles_a_door() {
        // TR_FILL floor+ceiling -> TR_PLACE door -> TR_TOGGLE x2: the whole
        // hook chain on a synthetic world, ending closed with the closed
        // frames restored — the screenshot scenario in miniature.
        let mut w = World::new("t".into(), 40, 40);
        apply_tr_fill(&mut w, "0,20,40,1,1");
        apply_tr_fill(&mut w, "0,16,40,1,1");
        assert!(w.tile(10, 20).unwrap().active(), "fill wrote the floor");
        apply_tr_places(&mut w, &parse_tr_place("10,19,10"));
        assert!(w.tile(10, 19).unwrap().active() && w.tile(10, 17).unwrap().active());
        // `place` guarantees the frame_important bit (Decision #4).
        assert!(w.frame_important[10]);
        let closed_frame = w.tile(10, 18).unwrap().frame_y;
        apply_tr_toggles(&mut w, &parse_tr_toggle("10,18,1"));
        assert!(
            w.tile(10, 18).unwrap().active() && w.tile(10, 18).unwrap().kind == 11,
            "one press opens the door"
        );
        assert!(w.frame_important[11], "open_door keeps the tile-11 bit");
        apply_tr_toggles(&mut w, &parse_tr_toggle("10,18,1"));
        assert_eq!(
            w.tile(10, 18).unwrap().kind,
            10,
            "the second press closes it again"
        );
        assert_eq!(w.tile(10, 18).unwrap().frame_y, closed_frame);
    }

    #[test]
    fn tr_tiles_parsing_kinds_frames_and_clear_groups() {
        // The TR_TILES dev hook parses fixture writes: 5 tokens `x,y,kind,fx,fy`
        // place one tile with explicit frames (negative frames are meaningful —
        // `-1` is "must be computed"), 4 tokens `x,y,w,h` are the clear
        // rectangle carrying the kind −1 sentinel.
        let ops = parse_tr_tiles("5,6,82,0,18;7,8,82,18,-1");
        assert_eq!(ops.len(), 2, "groups split on ';'");
        assert_eq!(
            (
                ops[0].x,
                ops[0].y,
                ops[0].kind,
                ops[0].fx,
                ops[0].fy,
                ops[0].w,
                ops[0].h
            ),
            (5, 6, 82, 0, 18, 1, 1)
        );
        assert_eq!(
            (ops[1].x, ops[1].y, ops[1].kind, ops[1].fx, ops[1].fy),
            (7, 8, 82, 18, -1)
        );
        let clear = parse_tr_tiles("10,10,3,2");
        assert_eq!(clear.len(), 1);
        assert_eq!(
            (
                clear[0].x,
                clear[0].y,
                clear[0].w,
                clear[0].h,
                clear[0].kind
            ),
            (10, 10, 3, 2, -1)
        );
        assert_eq!((clear[0].fx, clear[0].fy), (0, 0));
        // Both forms mix freely in one fixture string.
        let mixed = parse_tr_tiles("5,6,82,0,18;0,0,3,2");
        assert_eq!(mixed.len(), 2);
        assert_eq!(mixed[0].kind, 82);
        assert_eq!(mixed[1].kind, -1);
    }

    #[test]
    fn tr_tiles_parsing_skips_malformed_groups() {
        // The fixture is one env value; a malformed group must be dropped whole
        // without aborting the well-formed ones around it.
        assert!(
            parse_tr_tiles("").is_empty(),
            "the empty string places nothing"
        );
        assert!(parse_tr_tiles("1,2").is_empty(), "2 tokens is not a form");
        assert!(parse_tr_tiles("1,2,3").is_empty(), "3 tokens is not a form");
        assert!(
            parse_tr_tiles("1,2,3,4,5,6").is_empty(),
            "6 tokens is not a form"
        );
        assert!(
            parse_tr_tiles("x,y,82,0,18").is_empty(),
            "non-numeric coords"
        );
        assert!(
            parse_tr_tiles("1,2,82,0,z").is_empty(),
            "non-numeric frames"
        );
        assert_eq!(parse_tr_tiles("1,2,3;5,6,82,0,18").len(), 1);
    }

    #[test]
    fn tr_tiles_apply_writes_kind_frames_and_marks_frame_important() {
        // The dev harness stages the render scene by direct world writes: the
        // exact kind and frames land at the one cell with FLAG_ACTIVE, the
        // frame_important mark keeps the written frames alive through the
        // post-hook re-frame, and the edit bumps the world revision.
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        // A lived-in cell: the write form must replace it wholesale (wall drops).
        let i = w.index(5, 6);
        w.tiles[i] = Tile {
            kind: 1,
            wall: 5,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let rev_before = w.revision;
        apply_tr_tiles(&mut w, &parse_tr_tiles("5,6,82,0,18"));
        let t = w.tile(5, 6).unwrap();
        assert_eq!(t.kind, 82);
        assert_eq!((t.frame_x, t.frame_y), (0, 18));
        assert!(t.active(), "the fixture cell renders");
        assert_eq!(t.wall, 0, "the write replaces the whole cell");
        assert!(
            w.frame_important[82],
            "the written kind is marked frame-important"
        );
        assert!(!w.frame_important[1], "only the written kind is marked");
        assert!(w.revision > rev_before, "a write bumps the world revision");
        assert!(!w.tile(6, 6).unwrap().active(), "the write stays 1x1");
    }

    #[test]
    fn tr_tiles_apply_clear_resets_cells_to_default() {
        // The clear form wipes the whole rectangle to a blank stage cell —
        // the default Tile: no kind, no wall, nothing active — so the fixture
        // stages a clean scene on lived-in terrain.
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 10..13 {
            for y in 10..12 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    wall: 5,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let rev_before = w.revision;
        apply_tr_tiles(&mut w, &parse_tr_tiles("10,10,3,2"));
        for x in 10..13 {
            for y in 10..12 {
                assert_eq!(
                    *w.tile(x, y).unwrap(),
                    Tile::default(),
                    "the clear resets {x},{y} to the default cell"
                );
            }
        }
        assert!(w.revision > rev_before, "a clear bumps the world revision");
        assert!(
            !w.tile(9, 10).unwrap().active(),
            "the clear stays inside w*h"
        );
    }

    #[test]
    fn tr_tiles_apply_out_of_bounds_writes_nothing() {
        // Groups that fall outside the world are skipped cell by cell; nothing
        // is touched, so the revision stays exactly where it was.
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        apply_tr_tiles(&mut w, &parse_tr_tiles("1000,1000,4,4;-5,-5,4,4"));
        assert_eq!(w.revision, 0, "nothing touched -> no revision bump");
    }

    /// DW-1.6: `TR_SUNDIAL=1`/`TR_MOONDIAL=1` arm the dial fast-forwards at
    /// load (the pin block next to `TR_TIME` in `load_world`), flags only —
    /// the pins must not route through `sundialing()`/`moondialing()`, whose
    /// 8-day cooldown would mute the dial glow the screenshot pairs need.
    #[test]
    fn dw_1_6_load_world_pins_arm_dial_fast_forwards() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // A minimal `.trwld` through the real save path — `load_world` reads
        // a file (the scratch-file pattern of persist.rs's tests).
        let path = std::env::temp_dir().join("tr-dw-1-6-pins.trwld");
        save::save(&path, &World::new("pins".into(), 32, 32)).unwrap();

        // Edition 2024: env mutation is `unsafe` (a parallel reader could
        // observe a half-update); the ENV_LOCK above is that serialization.
        unsafe { std::env::set_var("TR_SUNDIAL", "1") };
        unsafe { std::env::set_var("TR_MOONDIAL", "1") };
        let world = load_world(&path).expect("scratch world loads");
        unsafe { std::env::remove_var("TR_SUNDIAL") };
        unsafe { std::env::remove_var("TR_MOONDIAL") };
        let _ = std::fs::remove_file(&path);

        assert!(
            world.fast_forward_to_dawn,
            "TR_SUNDIAL arms the dawn fast-forward"
        );
        assert!(
            world.fast_forward_to_dusk,
            "TR_MOONDIAL arms the dusk fast-forward"
        );
        assert_eq!(
            (world.sundial_cooldown, world.moondial_cooldown),
            (0, 0),
            "the pins set flags only, no cooldown"
        );
    }

    /// DW-1.6 (control): without the pins, the same scratch world loads with
    /// both fast-forward flags clear and cooldowns 0 — the arm is env-driven,
    /// not a `load_world` default.
    #[test]
    fn dw_1_6_absent_pins_leave_dial_flags_clear() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::remove_var("TR_SUNDIAL") };
        unsafe { std::env::remove_var("TR_MOONDIAL") };

        let path = std::env::temp_dir().join("tr-dw-1-6-clear.trwld");
        save::save(&path, &World::new("pins".into(), 32, 32)).unwrap();
        let world = load_world(&path).expect("scratch world loads");
        let _ = std::fs::remove_file(&path);

        assert!(!world.fast_forward_to_dawn && !world.fast_forward_to_dusk);
        assert_eq!((world.sundial_cooldown, world.moondial_cooldown), (0, 0));
    }
}

/// `GetNurseHealCost`'s core arithmetic (`Main.cs:40054-40100`): the
/// missing life plus the removable debuffs x100, times the happiness
/// adjustment. The boss-progression/expert/discount multipliers sit
/// constants-off.
pub(crate) fn nurse_heal_cost_for(missing: i32, debuff_count: i32, mood: f32) -> i32 {
    ((missing + debuff_count * 100) as f32 * mood) as i32
}

#[cfg(test)]
mod dye_pin_tests {
    use super::*;

    /// DW-2.3: the `TR_DYE`/`TR_DMOUNT` pin parses — positional fills,
    /// garbage and empty entries stay empty slots.
    #[test]
    fn tr_dye_pin_fills_positionally_and_forgives_garbage() {
        let dyes = parse_tr_dye("1007, 1015 ,, 1023");
        assert_eq!(dyes[0], 1007);
        assert_eq!(dyes[1], 1015);
        assert_eq!(dyes[2], 0, "empty entry leaves the slot empty");
        assert_eq!(dyes[3], 1023);
        assert_eq!(dyes[4..], [0; 6]);
        assert_eq!(parse_tr_dye(""), [0; 10]);
        assert_eq!(parse_tr_dye("nope,x")[..2], [0, 0]);
        // Overlong lists clip to the ten dye slots.
        assert_eq!(parse_tr_dye("1,2,3,4,5,6,7,8,9,10,11,12")[9], 10);
    }

    #[test]
    fn tr_dmount_pin_parses_one_kind() {
        assert_eq!(parse_tr_dmount("1007"), 1007);
        assert_eq!(parse_tr_dmount(" 2871 "), 2871);
        assert_eq!(parse_tr_dmount("nope"), 0);
        assert_eq!(parse_tr_dmount(""), 0);
    }
}
