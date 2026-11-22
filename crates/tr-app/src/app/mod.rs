//! The App — the port of `Terraria.Main`'s instance state (the C# statics,
//! one struct) plus the `main` bootstrap that builds it: the world load, the
//! atlas and the starting entities. The remaining `impl App` glue that has
//! no submodule of its own lives here too: the sim delegates, the
//! front-screen openers and the keyboard tails. The winit event loop is
//! `loop.rs`, the frame orchestrator `frame.rs`. (A7: extracted from
//! `main.rs`.)

pub(crate) mod capture_driver;
pub(crate) mod capture_entities;
pub(crate) mod capture_render;
pub(crate) mod clicks;
pub(crate) mod equip_swap;
pub(crate) mod filters;
pub(crate) mod flow;
pub(crate) mod frame;
pub(crate) mod layer_targets;
mod r#loop;
pub(crate) mod net_npc_sync;
pub(crate) mod netglue;
pub(crate) mod netmodules;
pub(crate) mod net_proj_sync;
pub(crate) mod rng;
pub(crate) mod saveglue;
pub(crate) mod session;
pub(crate) mod settings_ui;
pub(crate) mod worldgen_job;

// The RNG helpers live in `frame` with their main consumer, but the world
// simulation and the sky reach them through the app root.
pub(crate) use frame::{SplitmixRand, next_rand, rand01};
pub(crate) use rng::{NamedRngs, RngKey};

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
use crate::app::flow::{FrontScreen, KeyboardSession, KeyboardTail, tr_ui_preset};
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
    /// S1-G-13: the render-target family (`Main.InitTargets`,
    /// `Main.cs:58160-58216`) — seven world-space `WorldSceneLayerTarget`
    /// caches + skyTarget/screenTarget/screenTargetSwap. Allocated by the
    /// `EnsureRenderTargetContent` call at the draw head
    /// (`Main.cs:61644`/`:58152-58158`); no draw pass consumes the layers
    /// yet (the port's single cached world range is unchanged — ANALYSIS
    /// "The render-target family (S1-G-13)"). The capture mode's
    /// `DrawCapture` chunk render is the consumer this seam waits for.
    pub(crate) targets: Option<tr_render::RenderTargetFamily>,
    /// S1-G-13 consumers: the one `TargetCompositor` the six composite sites
    /// share (`Main.DoDraw`'s `spriteBatch.Draw(target.Texture, ...)` — the
    /// port rebuilds the bind group per draw, the instance buffer persists).
    /// Allocated lazily beside the family.
    pub(crate) target_compositor: Option<tr_render::TargetCompositor>,
    /// S1-G-13 consumers: `Main.renderCount`'s port for the gated layer path
    /// — the lighting phase index `LightingEngine.ProcessArea` publishes
    /// (`LightingEngine.cs:90-103`: MinimapUpdate→3, ExportMetrics→0, Scan→1,
    /// Blur→2, cycling every frame). Starts at 1 (the first frame's Scan).
    pub(crate) layer_render_count: i32,
    /// S1-G-13 consumers: the last draw region's `LightParams` snapshot —
    /// the capture chunk render (`Main.DrawCapture`, `Main.cs:56727-56732`)
    /// re-lights its area with the frame's ambient parameters. `None` until
    /// the first draw; the capture tick falls back to the default params.
    pub(crate) capture_light_params: Option<tr_core::lighting::LightParams>,
    /// `Main.SceneState` — the phase-1 tr-core port; `update_scene` steps its
    /// fades and the filter driver reads its intensities.
    pub(crate) scene_state: tr_core::scene::SceneState,
    /// `Sandstorm` — the event state machine; the storm machine ticks in
    /// `update_scene` and the storm filters read its severity.
    pub(crate) sandstorm: tr_core::scene::Sandstorm,
    /// W4-c40: `SkyManager.Instance["Sandstorm"]` (`SandstormSky.cs`) — the
    /// sky tint quad's fade state; the `SceneState.cs:97` edge flips it in
    /// `update_scene`, `SkyManager.Update` fades it in `ambience_phase`,
    /// the sky pass draws it at the `DrawRemainingDepth` flush.
    pub(crate) sandstorm_sky: crate::render::sandstorm_sky::SandstormSky,
    /// M1.4-BG-18 phase 3: the `ApplyVisuals` driver (`app::filters`) — filter
    /// board, storm shader state and the winner uniform the post branch
    /// applies.
    pub(crate) filter_driver: crate::app::filters::FilterDriver,
    pub(crate) world: World,
    pub(crate) atlas: Option<Atlas>,
    /// The world-independent half of the atlas content, kept decoded so a world
    /// swap (`swap_world`) repacks it instead of re-reading ~7 000 sheets from
    /// disk. `None` until the first build; the content dir is fixed for the
    /// process, so the pixels cannot go stale.
    pub(crate) static_sheets: Option<Vec<tr_render::atlas::AtlasImage>>,
    /// The decoded retail font glyph tables — the other world-independent cost
    /// of a load (~1–2 s). Cached beside [`App::static_sheets`] for the same
    /// reason; the `FontAtlas` page texture rides [`App::font_atlas`] instead of
    /// being rebuilt.
    pub(crate) fonts_cache: Option<assets::FontTables>,
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
    /// `Player.talkNPC` (`Player.cs:27494`'s latch) — the NPC the chat dialog
    /// is open for; `None` is the C#'s `-1`.
    pub(crate) talk_npc: Option<usize>,
    /// `Main.npcChatText` (`Main.cs:1755`) — the chat dialog's line.
    pub(crate) npc_chat_text: String,
    /// `NPCChatPanel._lastHovered` (`NPCChatPanel.cs:32`) — the hovered
    /// interaction index; the hover-change edge plays sound 12 (`:157-160`)
    /// and `Close` resets it to −1 (`:267`).
    pub(crate) npc_chat_last_hovered: i32,
    /// W4-f30 — `NPC.PreventJojaColaDialog` (`NPC.cs:6629`): the Dryad's
    /// Stardew bit latches it after the gift so the cola reminder does not
    /// re-print over the thank-you line. Cleared on the chat close
    /// (`NPC.cs:93772`/`:95303`).
    pub(crate) prevent_joja_cola_dialog: bool,
    /// W4-f30 — `NPC.RerollDryadText` (`NPC.cs:6631`): the Stardew bit's
    /// text-reroll state machine (`NPCChatPanel.cs:199-208`): 2 right after
    /// the gift, 1 once the Dryad enters the animation state 24, 0 once she
    /// leaves it (the chat re-rolls to `GetChat()`).
    pub(crate) reroll_dryad_text: i32,
    /// `Main.combatText[100]` (`CombatText.cs:65`): the floating damage
    /// numbers, indexed and updated like the C# pool.
    pub(crate) combat_texts: Vec<crate::combat_text::CombatText>,
    /// `PopupText.popupText[20]` (`PopupText.cs:11-12`): the item-name
    /// popups — the reforge reveal (`Main.cs:41906`) is the only spawn
    /// site the port has (W4-g57).
    pub(crate) popup_texts: crate::popup_text::Pool,
    pub(crate) npc_assets: NpcAssets,
    pub(crate) npc_timer: f32,
    /// The worm chain id counter (`AI_006`'s chain identity) — the head
    /// takes the next id, the segments share it.
    pub(crate) next_worm_id: u32,
    /// The NPC id counter (`NPC.whoAmI`, `NPC.cs:6145`) — the stable
    /// identity the boss-part links ride (`ai[3]`, `localAI[0..2]`). The
    /// C#'s id is the NPC slot; the port's list compacts every frame, so
    /// the id is handed out once and never reused.
    pub(crate) next_npc_uid: u32,
    /// The Journey powers (`GameContent/Creative/CreativePowers.cs`).
    pub(crate) creative_powers: crate::creative::CreativePowers,
    /// The debug cheat-sheet panel (`debug_panel.rs` — the self-contained
    /// module; delete the field + the module + the call sites to remove).
    /// Compiled only with the `debug` feature (see `tr-app/Cargo.toml`).
    #[cfg(feature = "debug")]
    pub(crate) debug_panel: tr_debug::Panel,
    /// The favorited inventory slots (`Item.favorited`'s port — the slot
    /// indices, not the per-item flag; the v1 tracks the slot positions).
    pub(crate) favorite_slots: Vec<usize>,
    /// The Goblin Army spawn cadence (the 2-second invasion pass).
    pub(crate) invasion_spawn_timer: f32,
    /// The open shop — the NPC's Vec index (`Main.npcShop`; the stock and
    /// the mood captured at the open).
    pub(crate) shop_npc: Option<usize>,
    /// The stock captured at the open (`Chest.item[]`) — the rows carry
    /// their per-row price fields (`Item.shopCustomPrice`/`buyOnce`/
    /// `isAShopItem`), priced by `Item.GetStoreValue`. Sold-in rows append
    /// as buyOnce clones (`Chest.AddItemToShop`); a buyOnce row depletes
    /// and turns to air (`ItemSlot.cs:2090-2093`).
    pub(crate) shop_stock: Vec<crate::shop::ShopEntry>,
    /// `Main.shopSellbackHelper` (`Main.cs:1775`): the bought-goods memo
    /// the sell payout refunds through (`Player.cs:35191-35195`); cleared
    /// whenever the shop index changes (`Main.UpdateOldNPCShop`,
    /// `Main.cs:18204-18211`).
    pub(crate) shop_sellback: crate::shop::SellbackHelper,
    /// The happiness multiplier captured at the open
    /// (`currentShoppingSettings.PriceAdjustment`).
    pub(crate) shop_mood: f32,
    /// `Player.discountAvailable` (`Player.cs:25638`): refreshed from
    /// `discountEquipped` when no shop is open; the Discount Card writer
    /// lands with the equipment pass (S1-C-12) — the price arms
    /// (`Player.cs:35369-35378`) consume it today.
    pub(crate) discount_available: bool,
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
    /// roll (`:41910`), decremented per tick (`:17121-17123`) and cleared
    /// while the button is unhovered (`:41568`).
    pub(crate) reforge_cooldown: i32,
    /// `Main.mouseReforge` (`Main.cs:1947`): the reforge button's hover latch,
    /// set in the draw block (`:41656`/`:41665`) and read by the per-frame
    /// scale/cooldown update (`:41559-41572`).
    pub(crate) mouse_reforge: bool,
    /// `Main.reforgeScale` (`Main.cs:1949`, init 0.8 `:67157`): the button's
    /// hover animation scale — +0.02/frame toward 1 while hovered, −0.02
    /// toward 1 while not (`:41561-41572`).
    pub(crate) reforge_scale: f32,
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
    /// `Main.EquipPageSelected` (`Main.cs:749`): the page the player *asked*
    /// for with the tab icons (`DrawPageIcons`' click arm, `:42846-42858`).
    /// `Main.EquipPage` above is the page actually drawn, recomputed every frame
    /// by `ItemSlot.SelectEquipPage` (`ItemSlot.cs:4622-4652`) from the held item
    /// and falling back to this when nothing overrides it (`Main.cs:42864-42867`).
    pub(crate) equip_page_selected: i32,
    /// `ItemSlot.dyeSwapCounter` (`ItemSlot.cs:3549-3564`): the round-robin
    /// slot `DyeSwap` starts its empty scan from and advances past.
    pub(crate) dye_swap_counter: usize,
    pub(crate) selected: usize,
    /// `Player.selectedItemState` (`Player.cs:414-568`) — the buffered /
    /// overridden halves of the selection machine (S1-P-i54); the live
    /// slot stays `selected` above. The switch applies only when idle
    /// (`CanChangeSelectedItemImmediately`, `:428-438`).
    pub(crate) selection: crate::hotbar::SelectionState,
    /// `Main.blockMouse` (`Main.cs:2445`): a mid-use switch blocks the
    /// held left button until its release (`OnSelectionChanged`,
    /// `Player.cs:563`; the release clears it, `TriggersSet.cs:846`).
    pub(crate) block_mouse: bool,
    /// Vanilla-style per-slot hotbar scale (selected grows to 1.0).
    pub(crate) hotbar_scales: [f32; HOTBAR],
    /// Escape toggles the inventory panel (`Main.OpenInventory`), it no longer quits.
    pub(crate) inventory_open: bool,
    /// `Main.BestiaryTracker` (`BestiaryUnlocksTracker.cs`): the per-world
    /// bestiary state — the kill counts plus the sighted/chatted credit-id
    /// sets. For a `.wld` it rides world-file section 10 (`SaveBestiary:3396`);
    /// for the native `.trwld` the kills ride the sidecar's v5 line, which the
    /// sights/chats have no equivalent for (they stay empty).
    pub(crate) bestiary: tr_core::bestiary::WorldBestiary,
    /// W4-f30 — `CreativeItemSacrificesCatalog`'s per-world sacrificed counts
    /// (`ItemsSacrificedUnlocksTracker`, the Journey research). The live
    /// sacrifice UI is unported, so the tracker never edits in play yet; the
    /// `CheckResearchAchievement` call sites (the per-tick armor tail +
    /// EnterWorld) read it through `check_research_achievement` (the prep
    /// landed with the f30 tail census). The save rides M6-SAVE's world
    /// sections when the UI lands.
    pub(crate) research: crate::research::ResearchTracker,
    /// `AchievementsHelper._lastResearchVersion` (`AchievementsHelper.cs:17`):
    /// the static latch `CheckResearchAchievement` compares the tracker's
    /// `LastEditId` against to skip unchanged frames.
    pub(crate) last_research_version: i32,
    /// `WorldGen.TownManager` (`TownRoomManager.cs`): one room per town NPC,
    /// loaded from world-file section 9 (`SaveTownManager`'s list) and rebuilt
    /// by the load fix-ups.
    pub(crate) town_rooms: crate::town::TownRooms,
    /// `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`): the `bool[NPCID.Count]`
    /// bitmap the NPC section's first list persists (truth indices).
    pub(crate) shimmered_npcs: crate::town::ShimmeredTownNpcs,
    /// `IngameFancyUI.OpenUIState(Main.BestiaryUI)`'s port — the bestiary
    /// screen over the world, opened from the inventory's bestiary button
    /// (`Main.cs:42420-42470`). `Some(index)` = an entry's info page.
    pub(crate) bestiary_open: bool,
    /// BUG-11: `Main.ingameOptionsWindow` (`Main.cs:2025`) — the in-game
    /// options window the inventory's Settings button opens
    /// (`IngameOptions.Open`, `IngameOptions.cs:56`).
    pub(crate) ingame_options: crate::ui::ingame_options::IngameOptions,
    pub(crate) bestiary_selected: Option<usize>,
    /// `Main.chatMonitor` + `Main.drawingPlayerChat`/`Main.chatText` — the
    /// chat state (M6-CHAT-1).
    pub(crate) chat: chat::ChatMonitor,
    /// `EmoteBubble.byID` + the wheel state (M6-EMOTE-1).
    pub(crate) emotes: emotes::EmoteSystem,
    /// `UIScrollbar._isDragging`'s `_dragYOffset` for the emote wheel's bar
    /// (`UIScrollbar.cs:23-27`, `LeftMouseDown` `:131-135`) — W4-g54.
    pub(crate) emote_scroll_drag: Option<f32>,
    /// `BannerSystem`'s kill tally + claimable counts (M2-BUFF-8, W4-e24).
    pub(crate) banners: banners::BannerSystem,
    /// `FakeCursorItem` (`GameContent/FakeCursorItem.cs`): the pending-claim
    /// cursor overlay the BannerClaimingUI shows while a banner claim is in
    /// flight (W4-e24). The single-player loopback adds then removes it
    /// within one claim, so it reads empty between claims.
    pub(crate) fake_cursor: banners::FakeCursorItem,
    /// The homeless town NPCs' look-for-home timer (`lookForHomeTimeout`'s
    /// stand-in; M4-TOWN-1).
    pub(crate) town_look_timer: f32,
    /// `Main.checkForSpawns` + `Main.townNPCCanSpawn` +
    /// `WorldGen.prioritizedTownNPCType` (`Main.cs:65542-66116`) — the town
    /// NPC arrival gates (M4-TOWN-1b).
    pub(crate) town_arrival: crate::town_arrival::ArrivalState,
    /// `Player.reuseDelay` (`Player.cs:1444`): the post-swing reuse lock in
    /// seconds (`Item.reuseDelay` ticks, `Item.cs:317`); M3-ITEM-5.
    pub(crate) reuse_delay: f32,
    /// `Main.Achievements` — the achievement registry (M6-ACH-1, the full
    /// 137-row initializer table with M6-ACH-2).
    pub(crate) achievements: achievements::AchievementManager,
    /// `UIAchievementsMenu`'s port (M6-ACH-2): the overlay state over the
    /// menu/world backdrop, opened from `Lang.menu[131]` (`Main.cs:48931`)
    /// or `TR_ACHIEVEMENTS=1`.
    pub(crate) achievements_open: bool,
    /// The list's scroll, in px (`UIScrollbar._viewPosition` via
    /// `UIList.DrawSelf`'s `_innerList.Top`, `UIList.cs:181`).
    pub(crate) achievements_scroll: f32,
    /// The four category toggles, all on at `OnActivate`
    /// (`UIAchievementsMenu.InitializePage`'s `SetState(value: true)`).
    pub(crate) achievements_categories: [bool; 4],
    /// The scrollbar's drag latch (`UIScrollbar._isDragging`/`_dragYOffset`,
    /// `UIScrollbar.cs:126-149`): `Some(offset)` while the handle is held.
    pub(crate) achievements_scroll_drag: Option<f32>,
    /// The Back panel's hover-sound latch (`FadedMouseOver` fires sound 12
    /// once per hover edge, `UIAchievementsMenu.cs:150-155`).
    pub(crate) achievements_back_sounded: bool,
    /// The scrollbar handle's hover latch (`UIScrollbar._isHoveringOverHandle`,
    /// `UIScrollbar.cs:25`/`:113-120`): sound 12 fires on the false→true edge,
    /// and `DrawSelf`'s handle tint reads `dragging || hovering` (`:122`).
    pub(crate) achievements_handle_hover: bool,
    /// `InGameNotificationsTracker._notifications`
    /// (`UI/InGameNotificationsTracker.cs:13`) — the achievement-unlock
    /// popups (S1-E-26 wave A). The `OnAchievementCompleted` subscriber
    /// (`AddCompleted`, `:75-81`) appends here; `Update` (`Main.cs:17045`)
    /// ticks it and `DrawInterface_13_AchievementCompletePopups`
    /// (`Main.cs:44779-44783`) draws it.
    pub(crate) notifications: ui::notifications::NotificationsTracker,
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
    /// `Main.defaultChestName` (`Main.cs:1753`) — the tile default captured
    /// at the rename keyboard's open (`IngameFancyUI.cs:103-123`); the
    /// submit's empty-if-default read (`ChestUI.cs:849-852`).
    pub(crate) chest_default_name: String,
    /// `Main.instance.textBlinkerCount/State` (`Main.cs:2287-2289`) — the
    /// chest-panel name editor's caret blinker (`ui::chest::TextBlinker`).
    pub(crate) text_blinker: ui::chest::TextBlinker,
    /// The crafting RNG stream — the alchemy discount rolls
    /// (`Recipe.cs:221`'s `Main.rand.Next(3)`; the exact C# `UnifiedRandom`
    /// generator seeded per world like the wiring RNG, S1-L-6c).
    pub(crate) craft_rng: tr_core::unified_random::UnifiedRandom,
    /// The coin cells' display-state machine (`CoinSlot.cs`, S1-H-9): the
    /// entry arrays the vanilla carries as statics, owned by the App.
    pub(crate) coin_slot: crate::ui::coin_slot::CoinSlotState,
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

    /// Bug 17: the front-end hover edge — the framework element id the pointer
    /// sat on last frame (`None` = outside), so `FadedMouseOver`'s menu tick
    /// fires once per enter (`UICharacterSelect.cs:124`,
    /// `UIWorldSelect.cs:126`).
    pub(crate) front_hover_hit: Option<u32>,
    /// `Main.mouseLeft`'s port for the front screens (`Main.cs:51101`'s role):
    /// held between the press and release events while `front` is live — the
    /// creation screen's HSL sliders drag off it, like the menu's value bars.
    pub(crate) front_mouse_down: bool,
    /// The two list screens' `UIScrollbar.ViewPosition` (`UIList.cs:181` lifts
    /// `_innerList.Top` by it every draw): the scrolled px of the live
    /// `UIList`, clamped into `[0, maxViewSize - viewSize]` wherever it is
    /// written (`UIScrollbar.cs:45`/`:75`). Reset on every `SetState`
    /// (`UserInterface.cs:295-299` builds fresh elements, so a re-entered
    /// screen starts at the top).
    pub(crate) front_scroll: f32,
    /// `UIScrollbar._isDragging` + `_dragYOffset` (`UIScrollbar.cs:23`/`:27`):
    /// `Some(offset)` while the handle is dragged, so the pointer's grab point
    /// inside the handle is kept (`:134-135`). `LeftMouseUp` clears it
    /// (`:145-149`).
    pub(crate) front_scroll_drag: Option<f32>,
    /// Which list the `front_scroll_drag` latch belongs to on the two-list
    /// pack-selection screen (`UIResourcePackSelectionMenu`'s
    /// `_availablePacksList`/`_enabledPacksList`): `false` = available (left),
    /// `true` = enabled (right). The single-list screens ignore it.
    pub(crate) front_scroll_drag_enabled: bool,
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
    /// The world the front end selected on Play (`UIWorldListItem.PlayGame`
    /// → `Main.LoadWorld`, BUG-04): the file the world swap loads at the next
    /// frame boundary. `None` while no world is pending.
    pub(crate) play_world: Option<PathBuf>,
    /// S1-E-17: the live world-generation job (`WorldGen.CreateNewWorld`'s
    /// task + the `WorldGenerator.CurrentController`/`CurrentGenerationProgress`
    /// statics, `WorldGenerator.cs:398-400`). `Some` while `FrontScreen::WorldLoad`
    /// is up; the frame polls it and the completion tail hops to the world
    /// select (`worldGenCallback`, `WorldGen.cs:6599-6602`).
    pub(crate) world_gen_job: Option<worldgen_job::WorldGenJob>,
    /// S1-E-17: the capture-mode singleton (`CaptureManager.Instance`,
    /// `CaptureManager.cs:10`, `:110-113`). The interface + camera state
    /// machines; the GPU render half is the S1-G-13 seam. `update_scene`
    /// reads `is_capturing()` for the `Main.UpdateSceneMetrics` gate
    /// (`Main.cs:62518-62521`).
    pub(crate) capture: crate::capture::CaptureManager,
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
    /// The `10 TileBlock` sections buffered while the host streams the world,
    /// applied in order at `49 InitialSpawn` (the C#'s section stream,
    /// `MessageBuffer.cs:812-841`).
    pub(crate) net_pending_sections: Vec<net::TileBlock>,
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
    /// Set when the npc store changed and `self.npcs` must be rebuilt.
    pub(crate) net_npcs_dirty: bool,
    /// S1-I-4d: the per-NPC `netSpam` throttle state keyed by the stable
    /// `Npc.uid` (`NPC.cs:6133-6149`). The C# keeps it on the NPC; the port's
    /// Vec compacts on death, so the state rides the uid, not the index.
    pub(crate) net_npc_states: HashMap<u32, crate::app::net_npc_sync::NpcNetState>,
    /// S1-I-4d: the `NpcSend` outbox the 60 Hz NPC cadence fills
    /// (`net_npc_sync::update_network_code`), tagged with the sender's uid so
    /// the 10 Hz drain resolves it to the current (post-compaction) slot.
    /// Replaces the old whole-set 10 Hz broadcast.
    pub(crate) net_npc_outbox: Vec<(u32, crate::app::net_npc_sync::NpcSend)>,
    /// S1-I-4d: despawn `23` bodies the cadence queues when a uid it knew is no
    /// longer in `npcs` (`NPC.cs:90385-90392`'s `life <= 0` arm, `:77991-77998`'s
    /// timeLeft arm). The host's Vec compacts on death, so the cadence — not the
    /// removed slot — emits the priority despawn.
    pub(crate) net_npc_despawns: Vec<net::NpcInfo>,
    /// S1-I-4d: uid → the wire slot index it last occupied (the despawn body's
    /// `slot`, resolved before compaction shifts the Vec).
    pub(crate) net_npc_uid_slot: HashMap<u32, u32>,
    /// S1-I-4d: uid → the last `23` body the cadence saw (the change-detect
    /// producer's reference + the despawn body's kind/position source).
    pub(crate) net_npc_last_info: HashMap<u32, net::NpcInfo>,
    /// S1-I-4d: the projectile cadence engine (`net_proj_sync::ProjSync`),
    /// replacing `netglue::projectile_sync_plan`'s 10 Hz diff. Its per-slot
    /// `netSpam` budget is the C#'s (`Projectile.cs:17210-17244`).
    pub(crate) proj_sync: crate::app::net_proj_sync::ProjSync,
    /// S1-I-4d: the `ProjSend` outbox the 60 Hz projectile cadence fills;
    /// `net_tick` drains it through the existing `27`/`29` wires.
    pub(crate) net_proj_outbox: Vec<crate::app::net_proj_sync::ProjSend>,
    /// S1-I-4d interim producer: the last `27` body per local key. The C#'s
    /// `netUpdate = true` comes from 337 AI sites (unwired); until they land,
    /// a state change vs this reference arms `ProjSync::mark_dirty`, and a
    /// vanished key emits the `29` kill. The send BUDGET is the faithful
    /// `ProjSync` (`Projectile.cs:17210-17244`), not this diff.
    pub(crate) net_proj_prev: HashMap<i32, net::ProjectileState>,
    /// S1-I-4d: the NPC/projectile cadence's 60 Hz accumulator. The C# runs
    /// `UpdateNetworkCode` per `UpdateNPC` (`NPC.cs:90513`) and the projectile
    /// budget per `Projectile.Update` (`Projectile.cs:17210`); the port steps
    /// both decision layers once per fixed step here, then drains the outboxes
    /// on the 10 Hz `net_tick` below.
    pub(crate) net_sync_cadence_timer: f32,
    /// W4-d100: the remote-entity smoothing's own 60 Hz accumulator — the
    /// C# runs `Player.UpdateNetOffset` once per `Player.Update`
    /// (`Player.cs:28172`, `Main.cs:17824-17857`'s fixed-step walk) and
    /// `NPC.UpdateNetOffset` once per `UpdateNPC`; the port steps both on
    /// this timer (`net_interp_tick`), never per render frame.
    pub(crate) net_interp_timer: f32,
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
    /// W4-d70 DOC-SWEEP: the local player's last-sent armour block
    /// (`PlayerItemSlotID.Armor0..19`) — the `5 PlayerInventory` diff the
    /// paladin peer loops' shield re-derivation reads on the other side.
    pub(crate) net_armor_last: [u16; 20],
    /// The heartbeat counter for the `13` cadence (`net_tick` calls).
    pub(crate) net_beat: u32,
    /// `Main.netPlayCounter` (`Main.cs:64557`): the host's per-tick server
    /// counter, wrapping at 3600 (`Main.cs:64558-64562`).
    pub(crate) net_play_counter: u32,
    /// `Main.lastItemUpdate` (`Main.cs:64572-64576`): the round-robin slot the
    /// 1 Hz `21` heartbeat walks.
    pub(crate) last_item_update: usize,
    /// The host slot id per client-npc index (the `28 NPCStrike` target and
    /// the `23` order), parallel to `npcs` on a joining client.
    pub(crate) net_npc_slots: Vec<u32>,
    /// The `154 Ping` round trip (`Net/Ping.cs`: the stopwatch restart point
    /// and the `_waitingForResponse` flag).
    pub(crate) net_ping_at: Option<Instant>,
    pub(crate) net_ping_waiting: bool,
    pub(crate) net_rtt_ms: u32,
    /// `Main._lastPingMousePosition` / `_lastPingMouseDownTime`
    /// (`Main.cs:377,379`): the double-click detector for the fullscreen
    /// map's ping (`:55486-55495` - two presses within 0.5 s and 2 px).
    pub(crate) last_ping_pos: Vec2,
    pub(crate) last_ping_time: f64,
    pub(crate) join_target: Option<(String, String)>,
    /// Explored-tile minimap and full map (`M`).
    pub(crate) map: Option<map::WorldMap>,
    /// `Terraria.Map.MapUpdateQueue` (`MapUpdateQueue.cs`): the edit-side map
    /// refresh queues, drained per frame (`Main.cs:61862`).
    pub(crate) map_queue: map::MapUpdateQueue,
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
    /// `Main.teamCooldown` (`Main.cs:965`): ticks left before the PVP toggle
    /// and the team cells accept another change (`teamCooldownLen = 300`,
    /// `Main.cs:66781`). Decremented once per update (`Main.cs:17048-17051`).
    pub(crate) team_cooldown: i32,
    /// `Main.hidePVPAndTeamIcons` (`Main.cs:2357`): set by the NPC-heads column
    /// overflow (`Main.cs:42046`), cleared on every non-`EquipPage == 1` draw
    /// (`Main.cs:42656-42659`) and on world load (`Main.cs:42020`/`:67272`).
    pub(crate) hide_pvp_and_team_icons: bool,
    /// The `TR_PVP` pin resolved at boot ([`crate::env::pvp_icons`]) — held in a
    /// field so tests can open the PVP draw/click gates without touching the
    /// process environment (the pin is a `OnceLock`).
    pub(crate) pvp_icons_forced: bool,
    pub(crate) audio: audio::Audio,
    pub(crate) music: audio::MusicSystem,
    /// `Main.saveTime`: seconds since the last autosave (`Main.cs:19751-19765`).
    pub(crate) autosave: f32,
    /// Stack carried on the cursor while the inventory is open (`Main.mouseItem`).
    pub(crate) mouse_item: ItemSlot,
    /// `Player.releaseQuickMana` (`Player.cs:1768`): the quick-mana press
    /// latch (S1-P-i37). `Player.Update` (`:24834-24845`) fires `QuickMana`
    /// only while the latch stands (the key's press edge), then drops it
    /// until the key releases. Starts false — the first held tick counts as
    /// the edge. (The QuickHeal twin `releaseQuickHeal` `:1770` is the i54
    /// hotbar worker's `quick_heal_prev` — not duplicated here.)
    pub(crate) quick_mana_prev: bool,
    /// The B-key (`Main.cBuff`) press latch (W4-a5), the `quick_mana_prev`
    /// precedent: `TriggersSet.CopyInto` (`TriggersSet.cs:801-804`) calls
    /// `Player.QuickBuff` on the QuickBuff trigger under
    /// `PlayerInput.Triggers.Current` — the port fires it on the key's
    /// press EDGE (the two-frame latch), not every held tick. Starts
    /// false — the first held tick counts as the edge.
    pub(crate) quick_buff_prev: bool,
    /// `Main.mouseNPCIndex` (`Main.cs:751`, default −1): the `Main.npc[]`
    /// slot whose head rides the cursor for a housing relocation; −1 with
    /// `mouse_npc_type == 0` is the housing query, −1/−1 is nothing.
    pub(crate) mouse_npc_index: i32,
    /// `Main.mouseNPCType` (`Main.cs:753`, default −1): the carried head's
    /// NPC type (0 = the housing query's "?"). `SetMouseNPC`
    /// (`Main.cs:3176-3185`) writes the pair.
    pub(crate) mouse_npc_type: i32,
    pub(crate) ui_left_press: bool,
    pub(crate) ui_right_press: bool,
    /// The frame's left-press edge snapshotted BEFORE the click cluster
    /// consumes `ui_left_press` (`handle_inventory_clicks` clears it every
    /// frame). The fullscreen map's pylon marker click reads it in the draw
    /// (`TeleportPylonsMapLayer.cs:83`/`:90` — `Main.mouseLeft &&
    /// Main.mouseLeftRelease`), which runs after the cluster.
    pub(crate) map_left_press: bool,
    /// `Main.mouseRightRelease` (`Main.cs:67166`'s analog — the right
    /// button's release edge, consumed per frame).
    pub(crate) ui_right_release: bool,
    /// `Player.controlThrow` (`Main.cThrowItem`, default `T` —
    /// `Main.cs:67168`; the task's "Q key" is the rebind row, the vanilla
    /// seed is T): the held throw key, refreshed once per frame from the
    /// bind table (`world_sim/player.rs`'s input assembly).
    pub(crate) control_throw: bool,
    /// The right-button press edge captured BEFORE the player pass —
    /// `Main.mouseRight && !mouseInterface && Main.mouseRightRelease`'s
    /// `dropItemCheck` arm (`Player.cs:5147`). `update_editing` refreshes
    /// `place_prev_down` AFTER the player pass, so the drop gate reads this
    /// pre-pass snapshot instead (the C# evaluates the gate inside
    /// `Player.Update`, before the item-check pass).
    pub(crate) drop_right_edge: bool,
    /// `Player.noThrow` (`Player.cs:1456`): the NPC-hover throw lockout
    /// (`Main.cs:43030`/`:43067` arm it to 4); `dropItemCheck` zeroes it on
    /// a closed inventory and counts it down (`Player.cs:5109-5116`). The
    /// hover sites are unported, so nothing arms it today — the field keeps
    /// the check's shape.
    pub(crate) no_throw: i32,
    /// W4-g56 — `Player.lastMouseInterface` (`Player.cs:1454`, latched at
    /// `Main.cs:61811-61812`: last frame's `mouseInterface`): the
    /// void-lens interaction gate reads it (`Main.cs:37688`). The port
    /// recomputes it at the interface-draw tail from the same hover
    /// surfaces that set `mouseInterface` (the slot/panel hit tests).
    pub(crate) last_mouse_interface: bool,
    /// W4-g56 — the per-slot `TryInteractingWithVoidLens` answers
    /// (`Main.cs:27924`), computed once per draw at the draw head
    /// (`void_lens::update_selection`) and read by the projectile draw
    /// loop's type-734 arm. Indexed by projectile slot; 0 for non-lenses.
    pub(crate) void_lens_selection: Vec<u8>,
    /// W4-g56 — the inventory/chest slot glow (`ItemSlot.cs`'s four glow
    /// arrays, `ui/glow.rs`): the open-time clear (`SetGlowForChest`), the
    /// Sort-feedback writer (`ItemSorting.Sort:1586`), the per-frame decay
    /// (`UpdateInterface:522-544`) and the draw tint (`ItemSlot.Draw:2305-
    /// 2326`). The chest pair rides the open container (contexts 3/4/32).
    pub(crate) slot_glow: crate::ui::glow::GlowState,
    /// `Main.mouseRightRelease` as the vanilla recomputes it at the END of
    /// every interface draw (`Main.cs:62458-62465`: `!Main.mouseRight`) —
    /// the buff bar's cancel reads `mouseRight && mouseRightRelease`
    /// (`:43333`), which is the press-after-release frame. Starts true
    /// (the button starts released, `Main.cs:1689`'s false static would be
    /// set true by the first menu draw; the port's first in-world frame
    /// with the button already held has no prior release — matching the
    /// vanilla's menu-drawn latch).
    pub(crate) ui_right_release_latch: bool,
    /// `Main.mouseLeftRelease`'s in-world arm — the left button's release
    /// edge the achievement popup's click reads (`InGamePopups.cs:124`'s
    /// `Main.mouseLeft && Main.mouseLeftRelease`), consumed per frame.
    pub(crate) ui_left_release: bool,
    /// `Main.buffAlpha[Player.maxBuffs]` (`Main.cs:668`-ish): the buff bar's
    /// per-slot icon alpha (`Main.DrawBuffIcon`).
    pub(crate) buff_alpha: [f32; crate::buffs::MAX_BUFFS],
    /// The buff bar's right-click removal requests — the C# removes in the
    /// draw pass (`Main.cs:43348`); the port drains them in the tick.
    pub(crate) buff_removals: Vec<usize>,
    /// `Main.focusRecipe` (`Main.cs:1729`): the selected recipe's position
    /// in the RAW `Main.availableRecipe` list (`HandleCraftSlot`
    /// `NewCraftingUI.cs:510`, `UpdateContents` `:610`/`:622`) — the first
    /// click focuses, the next crafts. NOT a recipe-table index and NOT a
    /// shown-grid position (the W4-f23 index-space fix).
    pub(crate) focus_recipe: i32,
    /// The previous frame's RAW available list — `Main.availableRecipe`'s
    /// state across `Recipe.UpdateRecipeList`'s clear+rebuild: the refocus
    /// tail (`Recipe.cs:232-233`) captures `availableRecipe[focusRecipe]`
    /// from THIS list before the rebuild (`TryRefocusingRecipe` `:477-495`,
    /// wired in `frame/draw.rs`). Empty until the panel first draws.
    pub(crate) craft_last_available: Vec<usize>,
    /// The craft grid's `UIScrollbar.ViewPosition` (`UIScrollbar.cs:37-47`):
    /// the wheel (`UIList.ScrollWheel` `:107-114`) and the bar's press/drag
    /// (`:126-143`, `:109-112`) write it; `SetView` clamps it to
    /// `contentHeight - listHeight` every rebuild.
    pub(crate) craft_scroll: f32,
    /// The bar's `_isDragging` grab offset (`UIScrollbar.cs:134-135`) —
    /// `Some` while the handle is held, `None` after `LeftMouseUp` (`:148`).
    pub(crate) craft_scroll_drag: Option<f32>,
    /// `_isHoveringOverHandle` (`UIScrollbar.cs:115-120`): re-derived every
    /// draw frame; the enter edge plays sound 12 (`:119`).
    pub(crate) craft_scroll_hover: bool,
    /// `Main.nearbyCraftingMouseOver` (`Main.cs:1309`): the nearby-chests
    /// toggle's hover latch (`CraftingUI.cs:354`'s
    /// `DoStatefulTickSound`), re-derived every draw frame the panel draws;
    /// the enter edge plays sound 12 (`Main.cs:3244`). Cleared by
    /// `Main.ResetInventoryState` (`:3225`) on the inventory open/close.
    pub(crate) nearby_crafting_mouse_over: bool,
    /// `Main._currentRecipeBeingCrafted` (`Main.cs:2352`): the held-repeat's
    /// recipe lock — a different recipe or a maxStack-1 product resets it.
    pub(crate) current_recipe_being_crafted: i32,
    /// `Main._preventCraftingBecauseClickWasUsedToChangeFocusedRecipe`
    /// (`Main.cs:2353`): the focus click's lock, cleared when the buttons
    /// release (`:42301-42305`).
    pub(crate) prevent_craft_click: bool,
    /// `UserInterface`'s pointer machine (`UserInterface.cs:53-91`, `:230-255`):
    /// the press/release → click/`DoubleClick` caches with the 500 ms window and
    /// the 200 ms state-change lockout. Fed by `app/loop.rs` (the front screens'
    /// event path) and ticked from `app/frame.rs`; the clock is `pointer_ms`,
    /// deterministic under `TR_SHOT`.
    pub(crate) pointer: ui::pointer::PointerState,
    /// The machine's clock in ms — `Main.TotalGameTime.TotalMilliseconds`'s
    /// analogue, derived from `anim_time` (1/60-pinned under `TR_SHOT`).
    pub(crate) pointer_ms: f64,
    /// The last `PointerEvent` the machine produced on the front screens' event
    /// path, read by `handle_front_click` and cleared there — the C#'s event
    /// delegate delivery (`UserInterface.cs:55-70`) without a listener table.
    pub(crate) pointer_event: Option<ui::pointer::PointerEvent>,
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
    /// The current dig's break interval in seconds (the pre-i29 mining
    /// pacing; the crack overlay now reads the `hit_tile` book directly —
    /// W4-g77 — while `sync_mine_crack_state` keeps this pair fresh for
    /// the mining-state witnesses).
    pub(crate) mine_hardness: f32,
    /// S1-P-i33 — the wall-hammer hit counter (`Player.PickWall`'s `hitTile`
    /// damage book, `Player.cs:45944-45945`): the number of `useTime / 2`-tick
    /// hits landed on the current wall target. The break fires at
    /// `ceil(100 / (hammer * 1.5))` hits — an integer count, so the cadence
    /// is exact (the crack overlay reads the `hit_tile` book — W4-g77).
    pub(crate) mine_wall_hits: i32,
    /// The render-side per-slot crack styles for the `hit_tile` book
    /// (`HitTile.cs:23`'s `crackStyle` + `:82`'s static `lastCrack`): the
    /// sim book is RNG-free (i29), so `CrackStyles` rolls on the slot's
    /// new occupancy (the C# rolls on `HitTileObject.Clear`, i.e. slot
    /// reuse — `HitTile.cs:29-46`). W4-g77 replaced the single-style
    /// projection with the book-driven walk.
    pub(crate) mine_crack_styles: crate::render::tile_cracks::CrackStyles,
    /// `HitTile.rand` (`HitTile.cs:78`): the crack-style stream. Deviation:
    /// the C# seeds it `DateTime.Now.Ticks`; the port seeds the world seed
    /// (determinism rule, ANALYSIS "The tile-crack overlay").
    pub(crate) mine_crack_rng: tr_core::UnifiedRandom,
    /// W4-f6 wiring — `TileDrawing._rand` (`TileDrawing.cs:104`/:191): the
    /// special-tile pass's PRIVATE stream (the pylon crystal's dust gate,
    /// `:8669-8670`). Deviation: the C# parameterless ctor is clock-seeded;
    /// the port seeds the world seed (the `mine_crack_rng` determinism
    /// convention). NOT a `SwapRandom` key. g68 collision keep-one
    /// (research/198, renamed 67→198 round-1): g68's drip gates (`EmitLiquidDrops` :7609) read the
    /// SAME C# field on the ONE `Main.TilesRenderer` (`Main.cs:2301/6541`)
    /// — at merge keep ONE field, both consumers, drips-before-pylons.
    pub(crate) tile_draw_rand: tr_core::UnifiedRandom,
    /// W4-f6 wiring — `LiquidRenderer._random` (`LiquidRenderer.cs:132`):
    /// the lava-bubble PRIVATE stream (`InternalPrepareDraw` :558/:562).
    /// Same clock-seed → world-seed deviation; NOT a `SwapRandom` key.
    pub(crate) liquid_rand: tr_core::UnifiedRandom,
    pub(crate) edit_target: Option<(i32, i32)>,
    /// The smart cursor's continuity locks (`SmartCursorHelper`'s two
    /// static `Point?` fields — the M7-SMARTCURSOR-1 app wiring).
    pub(crate) smart_cursor: tr_core::smart_cursor::ContinuityState,
    /// `Main.SmartCursorWanted_Mouse` (`Main.cs:2011`, the mouse's
    /// `SmartCursorIsUsed`, `:2889`): the **runtime** smart-cursor state,
    /// flipped on the left-Shift press edge
    /// (`Player.TryToToggleSmartCursor`, `Player.cs:24867`/`:28814-28820`) and
    /// gating the lookup (`world_sim/smart_cursor::enabled`). Vanilla starts
    /// it off (`Main.cs:67179`), so the port's default is `false`; the
    /// persisted `settings.smart_cursor_toggle` is the toggle-mode flag
    /// (`Main.cSmartCursorModeIsToggleAndNotHold`, `Main.cs:67178`), **not**
    /// this state.
    pub(crate) smart_cursor_used: bool,
    pub(crate) world_path: PathBuf,
    pub(crate) dev_break: Option<(i32, i32)>,
    pub(crate) dev_place: Vec<(i32, i32, u16)>,
    pub(crate) camera: Vec2,
    /// The gfx-quality state (S1-B-6): `Main.gfxQuality`'s adaptive loop and
    /// the per-update budgets it feeds (`Main.cs:953`/`:955`,
    /// `:16960-17101`).
    pub(crate) gfx_quality: world_sim::gfx_quality::GfxQuality,
    /// The world-event schedulers' state (S1-F-14): `LanternNight`,
    /// `BirthdayParty` and `CultistRitual`'s delay machine.
    pub(crate) world_events: crate::world_events::WorldEvents,
    /// The C#'s draw-time camera state (`Main.cs:463`/`:477`/`:1097-1099`):
    /// the teleport lerp pair, the object-tracking pan and the vestigial
    /// `DebugCameraPan` accumulator. S1-B-4 moved the camera update to the
    /// draw path; the update tail runs the pan/lerp walks (`:17814-17815`).
    pub(crate) camera_state: world_sim::camera::CameraState,
    pub(crate) zoom: f32,
    /// `TR_ZOOM` pins the zoom for screenshots, ignoring the settings value.
    pub(crate) zoom_pinned: bool,
    pub(crate) debug: bool,
    /// The debug *window* (the typed always-present overlay, top-left). Its own
    /// toggle (F4) so it can be up while the large `TR_DEBUG` overlay is off.
    pub(crate) debug_overlay: bool,
    /// `UIWorldGenDebug.IsActive` (`UIWorldGenDebug.cs:794`) — the F5 debug
    /// pass-list overlay over the loading screen (`UIWorldLoad.cs:50-53`'s
    /// `DebugOptions.enableDebugCommands && F5` arm, the port's `TR_DEBUG`
    /// stand-in). W4-f23 closeout item 1: the driver lands as an overlay,
    /// not a new FrontScreen variant (the d85 dev-screens co-lane owns the
    /// full screen; this wires the parked element halves to the live job).
    pub(crate) worldgen_debug_open: bool,
    pub(crate) keys: HashSet<KeyCode>,
    /// `Main.cUp`..`Main.cMount` (`Main.cs:67163-67176`): the live gameplay
    /// bind table, mirrored from `settings.keybinds` so a rebind in the
    /// settings screen changes the input pass without a relaunch
    /// ([`App::apply_settings`]).
    pub(crate) keybinds: [String; crate::input_map::BIND_COUNT],
    /// `Main.cMapStyle`..`Main.cMapAlphaDown` (`Main.cs:67190-67195`).
    pub(crate) map_binds: [String; crate::input_map::MAP_BIND_COUNT],
    /// The `controlMount` press edge's previous state (`Player.cs:24888`'s
    /// `releaseMount` latch). An `App` field rather than a `static` so two
    /// headless apps in one test process cannot share it.
    pub(crate) mount_prev: bool,
    /// `Player.releaseQuickHeal` (`Player.cs:1770`): the quick-heal press
    /// edge latch (`:24822-24833`), the per-player twin of `mount_prev`.
    pub(crate) quick_heal_prev: bool,
    /// `Main.gamePaused` (`Main.cs:645`): the world-sim freeze written by the
    /// two pause sources — the focus arm (`Main.cs:17138`) and the in-game
    /// gate's arm pair (`Main.cs:17197-17201`). Cleared on world load
    /// (`Main.cs:19856` → [`App::enter_loaded_world`] / the fresh [`App::new`]
    /// a `swap_world` builds). Read by the Phase-2 HUD overlay.
    pub(crate) game_paused: bool,
    /// `FocusHelper.IsSelectedApplication`'s core (`FocusHelper.cs:135-136` —
    /// `Main.instance.IsActive`): the window's focus, fed by winit's
    /// `WindowEvent::Focused` (`app/loop.rs`). Defaults to `true` so a headless
    /// frame (no window events) plays like a focused one.
    pub(crate) window_focused: bool,
    /// `PlayerInput.MouseInfo.XButton1`'s held state (`WorldUpdateStepper.cs:48`
    /// — the step key's mouse arm). Fed by winit's `MouseButton::Back`
    /// (`app/loop.rs`); consumed by the Phase-2 stepper.
    pub(crate) mouse_back_held: bool,
    /// `Main.GameUpdateCount` (`Main.cs:857`/`:2772`) — one increment per
    /// *executed* world step (`DoUpdateInWorld_Inner`'s head, `Main.cs:17746`),
    /// not per drawn frame (that is [`App::frame_count`]). Initialised 65536 by
    /// `ResetGameCounter` (`Main.cs:6442-6445`), whose `OnWorldLoad` hook
    /// (`Main.cs:6605`) makes a world load reset it — the fresh [`App::new`]
    /// a `swap_world` builds carries the same reset. The Phase-2 stepper's
    /// `StartUpdateCount` pin and the HUD overlay read it.
    pub(crate) game_update_count: u32,
    /// `Terraria.Testing.WorldUpdateStepper` (`WorldUpdateStepper.cs:7-58`)
    /// — the testing step-frame state; see [`crate::world_stepper`]. The
    /// `pauseworld` toggle, the End/NumpadAdd/XButton1 pacing and the
    /// `Updates Since Paused` overlay pin live here; the gate site is
    /// `app/frame.rs`'s `step_allowed` (`Main.cs:17278`).
    pub(crate) stepper: crate::world_stepper::WorldUpdateStepper,
    /// The step keys' this-frame reading
    /// (`WorldUpdateStepper.GetStepKeyState`'s `down`, `:39-43`): pinned at
    /// the frame head where the events have landed, consumed by the
    /// `Main.cs:17278` gate.
    pub(crate) step_input_down: bool,
    /// The step keys' previous-frame reading (the `wasDown` half, `:42`).
    /// Shifted from [`App::step_input_down`] every frame — the
    /// `Main.oldKeyState = Main.keyState` snapshot's analog
    /// (`Main.cs:18381-18382`, the region the port already runs at the frame
    /// head through [`App::update_view_zoom_keys`]).
    pub(crate) step_input_was: bool,
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
    /// S1-E-19: the legacy lighting engine (`LegacyLighting.cs`) — the
    /// White/Retro/Trippy modes' additive path. `None` while Color (the
    /// modern engine) is active; constructed on the first legacy frame and
    /// kept across mode switches (the C# `Lighting.LegacyEngine` is a static
    /// singleton, `Lighting.cs:23`).
    pub(crate) legacy_light: Option<tr_core::lighting::legacy::LegacyLighting>,
    /// `Main.mouseTextColor`: oscillates 190..=255 each frame, driving torch/lava/shimmer
    /// crystal-wall flicker (`DoUpdate_AnimateCursorColors` in Main.cs).
    pub(crate) mouse_text_color: u8,
    pub(crate) mouse_text_color_dir: i8,
    /// `Main.invAlpha` / `Main.invDir` (`Main.cs:1287-1289`): the Interface
    /// Logic 2 pulse (`DrawInterface_24_InterfaceLogic2`, `:45695-45711`) —
    /// the crafting grid's selection-overlay tint `Main.inventoryBack`
    /// (`ItemSlot.cs:2373`). W4-f23 deviation fix: was the midpoint-210
    /// constant.
    pub(crate) inv_pulse: crate::ui::inv_pulse::InvPulse,
    /// `Main.hideUI` (`Main.cs:2085`): the F11 screenshot-mode toggle
    /// (`DoUpdate_F11_ToggleUI`, `:18402-18424`). While true the interface
    /// layer loop stops at layer 8 (`DrawInterface_8_CheckF11UIHideToggle`,
    /// `:46175-46183`) and the world-side chat bubbles / combat text skip
    /// (`:62334`). The port gates the HUD draw + those two passes on it.
    pub(crate) hide_ui: bool,
    /// `Main.releaseUI` (`Main.cs:2087`): the F11 press latch — starts true
    /// (the key starts released), drops while held, re-arms on release.
    pub(crate) release_ui: bool,
    /// `Player.releaseMapStyle` (`Player.cs:1838`): the Tab map-style
    /// cycle's press latch (`:24951-24964`).
    pub(crate) release_map_style: bool,
    /// `Main.demonTorch` / `Main.demonTorchDir` (`DoUpdate_AnimateTileGlows`).
    pub(crate) demon_torch: f32,
    pub(crate) demon_torch_dir: f32,
    /// `Main.masterColor` / `Main.masterColorDir` (`Main.cs:1235-1237`,
    /// the ±0.05 cycle `:18225-18235`, reset 1/+1 at `:66833-66834`): the
    /// master-rarity pulse the reforge popup recolours by
    /// (`PopupText.cs:667-670`, W4-g57).
    pub(crate) master_color: f32,
    pub(crate) master_color_dir: i8,
    /// `Main.martianLight` / `Main.martianLightDir` (`Main.cs:1231-1233`,
    /// the same `DoUpdate_AnimateTileGlows`, `:18251-18261`): the 0.015
    /// bounce that feeds `TileDrawing._martianGlow` (`:498-499`). It ticks
    /// EVERY frame in C# — no event gate (`Main.cs:17145` is unconditional);
    /// the "0 outside the event" reading was wrong (W4-g12 doc-sweep).
    pub(crate) martian_light: f32,
    pub(crate) martian_light_dir: f32,
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
    /// `CraftingUI._lastFilter` (`CraftingUI.cs:130`): the station filter the
    /// player's right-click on a crafting station installed
    /// (`Player.GetCraftingFilterForTile`, `Player.cs:33312-33335`, via
    /// `OpenCloseFilter`); `CraftingUI.RecipeFilterHack` exposes it to
    /// `Recipe.UpdateRecipeList` only while the crafting page is up
    /// (`CraftingUI.cs:112-121`) — the port reads it behind the same gate.
    pub(crate) craft_filter: Option<craft::CraftingFilter>,
    /// `NewCraftingUI._filterer` (`NewCraftingUI.cs:216`, built at `:281-299`):
    /// the 11-tab recipe filter strip's state. `Main.focusRecipe` stays an
    /// `availableRecipe` index while the grid shows only the survivors
    /// (`:596-611`), which is what `craft::available_with_gates` + [`shown`]
    /// reproduce.
    pub(crate) craft_filters: crate::craft_filter::FilterBoard,
    /// `Main.waterfallManager`: scanning, animation counters and draw geometry.
    pub(crate) waterfall: tr_core::waterfall::WaterfallManager,
    pub(crate) waterfall_step: FixedStepper,
    /// `Main.tileFrame`/`tileFrameCounter` + `weatherVaneBobframe`
    /// (`Main.cs:1596-1597`, `:2237`) — the animated-tile animator
    /// (`Main.AnimateTiles`, `Main.cs:18516-19304`), ticked once per client
    /// update beside the waterfall animator (`Main.cs:17148-17150`),
    /// S1-G-1f. `draw_tiles` reads its rows/counters; the G6
    /// `WorldFrameKey::tile_anim` covers every value it feeds the draw.
    pub(crate) tile_anim: tr_core::drawdata_anim::TileAnimator,
    pub(crate) tile_anim_step: FixedStepper,
    /// W4-f14: `Main.critterCage` + the `Main.*CageFrame` tables
    /// (`Main.cs:1445`, `:66902-66972`) — the critter-cage animator
    /// (`Main.AnimateTiles_CritterCages`, `Main.cs:13511-16596`), ticked at
    /// the `AnimateTiles` tail (`Main.cs:19303`) on the SAME accumulator
    /// and `Main.rand` stream as `tile_anim`. PROCESS state in the C# —
    /// never re-zeroed per world (the `:8393-8394` reset covers
    /// `tileFrame` only); `new` replays the 3600-tick startup warm-up
    /// (`Main.cs:6800-6807`). `draw_tiles` clears/sets `critter_cage`
    /// (TileDrawing.cs:474/:5672) and reads the tables through
    /// `drawdata_furniture::cage_draw_data`; the G6
    /// `WorldFrameKey::cage_anim` covers the state.
    pub(crate) cage_anim: tr_core::drawdata_cages::CritterCageAnimator,
    /// `Wiring`'s statics: the mech queue, the wire floods and the teleporter
    /// pads (`Wiring.cs`).
    pub(crate) wiring: tr_core::wiring::Wiring,
    pub(crate) wiring_step: FixedStepper,
    /// The tile-entity update pass (`TileEntity.PerformUpdates` — the
    /// logic sensors' per-tick scan, M7-TE-2).
    pub(crate) te_step: FixedStepper,
    /// The world-growth pass (`WorldGen.UpdateWorld`'s rate loops, M7-GROWTH-1).
    pub(crate) growth_step: FixedStepper,
    /// The saved hardmode ore tiers (`SavedOreTiers`, the altar smashes).
    pub(crate) ore_tiers: tr_core::ores::SavedOreTiers,
    /// Weather and raindrops (`Rain.MakeRain` / `Rain.Update`).
    pub(crate) rain: Rain,
    /// The sandstorm's flying sand (`Sandstorm.EmitDust`).
    pub(crate) sand: crate::sand::Sand,
    /// `Main.dust` (`Main.cs:66985`): the 6001-slot pressure-gated pool —
    /// the wheel sparks and the sandstorm spawn/tick/draw through it
    /// (S1-E-1 phase 2); the `world_sim` drain feeds the cart's queued
    /// requests into it and the frame ticks/draws the client slots.
    pub(crate) dust: tr_core::particles::DustPool,
    /// W4-g11 union-exec (research/116 gap 4) — `Main.waterStyle`
    /// (`Main.cs:1105`): the STORED water style, written by `DrawWaters`
    /// (`:57475`, the port's `frame/draw.rs` liquid-alpha block); the
    /// `TeleportEffect` style-5 arm reads this static, NOT an inline
    /// `CalculateWaterStyle` recompute (a mid-frame biome change would
    /// flip the dust type).
    pub(crate) water_style: i32,
    /// `Main.ambientWaterfallX/Y/Strength` (`Main.cs:2201-2211`): the
    /// waterfall-scan accumulators `Main.Ambience` reads every ¼ s.
    pub(crate) ambient_waterfall: (f32, f32, f32),
    /// `Main.ambientLavafallX/Y/Strength` (`Main.cs:2207-2211`, stored by
    /// `WaterfallManager.cs:963-965`): the lavafall loop's (strength, x, y).
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
    /// The `AmbientSky` manager + `AmbienceServer` scheduler
    /// (`crates/tr-app/src/ambience.rs`). Its scheduler draws ride
    /// [`Self::rngs`]`::global()` — the bare-`Main.rand` stand-in
    /// (`Main.cs:17214-17216`), shared with chat commands and menu rolls.
    pub(crate) ambience: crate::ambience::Ambience,
    /// W4-f40: the `LanternSky : CustomSky` background lantern field
    /// (`crates/tr-app/src/render/lantern_sky.rs`). Its private RNG draws on
    /// no named stream (the C#'s `_random`, `LanternSky.cs:71`); the
    /// activate/deactivate edges ride `world_events::update_time_lanterns`.
    pub(crate) lantern_sky: crate::render::lantern_sky::LanternSky,
    /// W4-f40: the `LanternSky`'s 60 Hz accumulator (`SkyManager.Update`
    /// runs once per world update, `Main.cs:17139`; the ambience precedent).
    pub(crate) lantern_sky_step: FixedStepper,
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
    /// `TileDrawing._vineWindCounter` (`TileDrawing.cs:375`): the vine
    /// strips' own sway counter (`DrawVineStrip` reads it, `:9041`). C#
    /// advances it in the same `TileDrawing.Update` as the sunflower's, so
    /// it rides the same 60 Hz accumulator (S1-G-1).
    pub(crate) vine_wind_counter: f32,
    /// Ambient tree leaves (`TileDrawing.EmitTreeLeaves` / `Gore_UpdateLeaf`).
    pub(crate) leaves: Vec<crate::render::foliage::LeafGore>,
    /// `Main.gore[601]` (`Main.cs:66990`): the gore pool, ticked by
    /// `UpdateWorld_Gores` (`Main.cs:17971-17996`) after the NPCs and before
    /// the projectiles (`:17767-17775`). Its collision probes are injected once
    /// at init through [`crate::gore_seam::AppGoreCollision`].
    pub(crate) gore: tr_core::gore::GorePool,
    /// The per-frame `Lighting.AddLight` list pair (`LightingEngine.cs:36-38`):
    /// `_perFrameLights` / `_oldPerFrameLights`. The light pass reads the
    /// previous frame's list (`LightingEngine.cs:210-214`), so `end_frame` must
    /// run once per frame after every source has contributed.
    pub(crate) entity_lights: crate::entity_lights::EntityLights,
    /// `Main.essScale` / `Main.essDir` (`Main.cs:544-545`): the essence
    /// flicker scale, seeded 1.0 / -1 (`Main.cs:66673-66674`), drifting
    /// 0.7..1.0 once per draw pass (`:62264-62277`,
    /// `item_lights::update_ess_scale`). The step-time light arms read it.
    pub(crate) ess_scale: f32,
    pub(crate) ess_dir: i32,
    /// `Main.liquidAlpha` (`Main.cs:1103`): the per-water-style cross-fade the
    /// `DrawWaters` ramp drives (`Main.cs:57476-57489`); the liquid and
    /// waterfall passes consume it. Zeros, like the original's allocation
    /// (`Main.cs:66805`), so the liquids fade in over the first frames.
    pub(crate) liquid_alpha: [f32; 15],
    /// World item drops (`Terraria.Item` first pass).
    pub(crate) items: Vec<ItemDrop>,
    /// `Main._rngs` (`Main.cs:1294-1295`): the `Main.SwapRandom` named streams,
    /// one `UnifiedRandom` per stage key, all seeded from the world seed
    /// (`:3210-3212`). Stages take their stream from here instead of reading a
    /// shared `Main.rand` (`app/rng.rs`).
    pub(crate) rngs: NamedRngs,
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
    /// The Snake Band's repeat-swing latch: the current whip throw is the
    /// `withAudioVisualFeedback = false` refire (`Player.cs:42729` →
    /// `ItemCheck_Shoot`'s `:48668-48682` variant). Set by the repeat-swing
    /// trigger, consumed by the whip fire arm.
    pub(crate) whip_snake_refire: bool,
    /// Use-button edges for the non-`autoReuse` gates; refreshed every frame
    /// like `Player.releaseUseItem` (`Player.cs:42822`).
    pub(crate) place_prev_down: bool,
    pub(crate) place_released: bool,
    pub(crate) mine_prev_down: bool,
    pub(crate) mine_released: bool,
    /// `Player.itemTime` in seconds: the action accumulator inside a swing
    /// (`Player.cs:4416-4418`); placement re-fires whenever it hits zero.
    pub(crate) item_time: f32,
    /// `Player.toolTime` in seconds (`Player.cs:3022`): the mining-action
    /// cadence. `ItemCheck_StartActualUse` arms 1 tick for the tools
    /// (`Player.cs:51660-51663`); the per-tick block mirrors `itemTime`
    /// while not channeling, or counts down to `useTime - 1` on a channel
    /// drill (`Player.cs:43576-43587`). The pick hit fires at
    /// `toolTime == 0 && itemAnimation > 0 && controlUseItem`
    /// (`Player.cs:45608`/`:45626`).
    pub(crate) tool_time: f32,
    /// `Player.hitTile` (`Player.cs`, the `HitTile` field; `HitTile.cs:9`) —
    /// the mining damage book: per-cell accumulated damage, 100 breaks the
    /// tile (`Player.PickTile` `Player.cs:53769`), 60-tick TTL decay.
    pub(crate) hit_tile: tr_core::hit_tile::HitTile,
    /// `Player.hitReplace` (`Player.cs:1660`) — the block-swap damage book
    /// (`PlaceThing_TryReplacingTiles`, `:39857-39873`): a SECOND `HitTile`,
    /// independent of the mining book; `ClearMiningCacheAt` clears both
    /// (`:53846-53850`).
    pub(crate) hit_replace: tr_core::hit_tile::HitTile,
    /// `Player.poundRelease` (`Player.cs:1502`): the hammer's pound latch.
    /// Set on the use-button release (`Player.cs:45635-45638`), consumed by
    /// `ItemCheck_UseMiningTools_TryPoundingTile` (`:45969-46311`) on the
    /// next press; cleared when the cursor tile is not a hammer target
    /// (`:45608-45615`) or the pound refuses (`:45976-45985`).
    pub(crate) pound_release: bool,
    /// `ItemCheck_UseMiningTools_ActuallyUseMiningTool`'s `out canHitWalls`
    /// (`Player.cs:45650`) — the mining tick's wall-pass gate, read by the
    /// `:45639-45643` wall pass in `update_editing`. True unless a matching
    /// tool hit its tile (`:45656-45659`), the tile is `tileHammer`
    /// (`:45667`) or the pound arm ran (`:45986`).
    pub(crate) mining_can_hit_walls: bool,
    /// Coin purse value in copper coins.
    pub(crate) coins: u64,
    /// The 1000-slot projectile pool (`Main.projectile`, `projectile.rs`).
    pub(crate) projectiles: ProjectilePool,
    /// `Main.LocalGolfState` (`Main.cs:1807`, `GolfState.cs:6-173`): the
    /// per-client golf scoring state — the hit records, the score timer and
    /// the last-hit-ball tracking. `Update()` rides the player stepper's
    /// head (`Main.cs:17219` runs before `UpdatePlayers`); the cup scoring
    /// and the strike's `RecordSwing` run pool-side
    /// (`projectile_ai_golf.rs`, S1-P-i55).
    pub(crate) golf_state: tr_core::golf_state::GolfState,
    /// The falling tombstone projectiles (`Player.DropTombstone`).
    pub(crate) tomb_falls: Vec<crate::death::TombFall>,
    /// `Player.lostCoins`/`lostCoinString` + the last death text — the
    /// death screen's "Dropped coins" line (`Main.cs:45333-45338`).
    pub(crate) lost_coins: u64,
    pub(crate) lost_coin_string: String,
    /// The full death broadcast text (`{name} {text}`).
    pub(crate) death_text: String,
    /// `Main._framesWithRespawnSkipAvailable`: the death screen's
    /// "{InputTrigger_Jump} to skip" prompt's ramp counter
    /// (`Main.cs:45347-45378`) — counts up while the skip is allowed and
    /// zeroes otherwise (the C# counts it inside `DrawDeathText`).
    pub(crate) respawn_skip_frames: i32,
    /// `Player.AllowsRespawnTimerSkip()` for this frame, computed on the world
    /// tick beside `UpdateDead` (`Player.cs:16919`) — the gate the prompt's
    /// counter follows.
    pub(crate) respawn_skip_available: bool,
    /// `Main.invasionProgress*` (`Main.cs:1913-1921`): the invasion bar's
    /// state — the points, the max, the icon (the group + 3), the 160-tick
    /// display timer, the fade alpha and the near-invasion flag.
    pub(crate) invasion_progress: i32,
    pub(crate) invasion_progress_max: i32,
    pub(crate) invasion_progress_icon: i32,
    /// `Main.invasionProgressWave` (`Main.cs:1929`) — the moon-event wave
    /// number; `0` selects `DrawInvasionProgress`'s `Game.WaveCleared` arm
    /// (`:47806`), `> 0` the `Game.WaveMessage` arm (`:47782`).
    pub(crate) invasion_progress_wave: i32,
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
    /// `TR_SUMMON` debug hook: spawn the Hornet + Frost Hydra draw pair.
    pub(crate) summon_shot: bool,
    /// G6: identity of the last world instance range; a hit lets the frame reuse it.
    pub(crate) world_key: Option<WorldFrameKey>,
    /// `Main.hellBlackBoxBottom` (`Main.cs:2463`): world Y of the hell floor line the
    /// previous frame's magma pass produced, clamping the underworld black box. It is
    /// written after the box draws, so the box always consumes the previous frame's
    /// value (0 on the first frame — a negative height draws nothing, like XNA).
    pub(crate) hell_black_box_bottom: f32,
    /// `Main.undergroundBackground`/`oldUndergroundBackground`/`ugBackTransition`
    /// (`Main.cs:52823-52841`): the cavern style latch + cross-fade, carried
    /// across frames like `hellBlackBoxBottom`. The g72e 14-arg
    /// `draw_background` consumes it (`&mut`); the layer callback threads
    /// this field (W4-g72 closeout delta 64d §2 — `bg_styles::UgBackState`
    /// lands with the g72 lane).
    pub(crate) ug_back: crate::render::bg_styles::UgBackState,
}

/// G6: everything the world instance range (sky strips, tiles, walls, dark rects)
/// depends on, compared bit-exactly. A hit keeps the previous frame's instances and
/// only redraws the UI; the light map has its own cache (D2) inside `LightMap`.
#[derive(Clone, Copy, PartialEq, Debug)]
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
    /// `LightStyle::brightness` (`Lighting.cs:108-116`): the Darkness debuff's
    /// `blind` flag drops `GlobalBrightness` 1.2 → 1.0, and `shade` multiplies
    /// every tile tint by it — an HUD-buff input inside the cached world range,
    /// so it must ride the key (S1-H-15).
    pub(crate) brightness: u32,
    /// Waterfall animation frames + visible-list revision (0 when none visible).
    pub(crate) waterfall: u64,
    /// S1-G-1f: `TileAnimator::anim_key()` — the animated-tile rows/counters
    /// and the weather-vane bob (`Main.AnimateTiles`, `Main.cs:18516-19304`)
    /// that `draw_tiles` feeds `draw_data`/`case_draw_data`. Zero unless an
    /// animated kind is active in the view rect, so animator-free worlds
    /// keep the pre-wiring cache behaviour bit-for-bit (the waterfall key's
    /// convention).
    pub(crate) tile_anim: u64,
    /// W4-d58: `chest::chest_frame_key()` — every live chest's anchor + lid
    /// frame (`Main.chest[i].frame`, `Chest.cs:86`), which the chest rewrite
    /// (`CacheSpecialDraws_Part2`, `TileDrawing.cs:4289-4330`) folds into the
    /// source rects inside the cached world range. Zero unless a BasicChest
    /// tile (21/467) is active in the view rect, so chest-free worlds keep
    /// the pre-wiring cache behaviour bit-for-bit (the `tile_anim`
    /// convention). The lid animator ticks on the world step
    /// (`Chest.UpdateChestFrames`, `Main.cs:17810`), so an opening/closing
    /// chest changes this while the camera stands still.
    pub(crate) chest_frames: u64,
    /// W4-f14: `CritterCageAnimator::anim_key()` — the `Main.*CageFrame`
    /// tables the cage arms' `addFrY` rewrites read
    /// (TileDrawing.cs:5629-5907). Zero unless a cage kind is active in
    /// the view rect (the `tile_anim` convention).
    pub(crate) cage_anim: u64,
    /// Remote peers' smoothed positions/animations and the NPC smoothing
    /// accumulators: the networked entities drawn inside the cached world
    /// range. Zero in single-player (the offsets are zero and no peers).
    pub(crate) net_key: u32,
    /// W4-g80: the 83 herb-maturity sheet swap's weather inputs that the
    /// light key does not already carry — `Main.cloudAlpha`'s zero/non-zero
    /// gate and the deathweed moon gate (`blood_moon || moonPhase == 0`),
    /// plus the fireblossom `time > 40500` gate. `day_time`/`raining`/`time`
    /// already ride `light_params`. Zero unless an herb 83 is active in the
    /// view rect, so herb-free worlds keep the pre-wiring cache behaviour
    /// bit-for-bit (the vine-grid key's convention).
    pub(crate) herb_weather: u32,
    /// W4-g12: `DrawGoreBehind`'s drip slots draw inside the cached world
    /// range (the behind-tiles seam), so their state rides the key — a
    /// hash of every `draws_in_back` slot's position/frame/alpha/scale/
    /// rotation/type. Zero while no drip slot draws (the waterfall
    /// convention: gore-free frames keep the pre-wiring cache behaviour).
    pub(crate) back_gore: u64,
    /// W4-d70: `MoonLordScreenShaderData.LatestFilterStrength` — the Moon Lord
    /// parts' `GetMoonLordColoring` tint (`Main.cs:26325`) rides the cached
    /// world range. Zero unless a Moon Lord part is present, so Moon-Lord-free
    /// worlds keep the pre-wiring cache behaviour bit-for-bit (the tile_anim
    /// key's convention).
    pub(crate) moon_lord_strength: u32,
    pub(crate) world_revision: u64,
    /// W4-d127 — the WoF draw inputs inside the cached world range: the
    /// mouth clock (`Main.wofDrawFrameIndex`, `Main.cs:761`), the band
    /// statics (`wofDrawAreaTop/Bottom`, `:757`/`:759`) and the tongue
    /// gate (`Player.tongued`). Non-zero only while a wall lives, so
    /// wall-free worlds keep the pre-wiring cache behaviour bit-for-bit
    /// (the waterfall/tile_anim key convention).
    pub(crate) wof_key: u64,
}

impl WorldFrameKey {
    /// The `back_gore` input: one hash over the drip slots the behind
    /// pass draws (`Main.cs:21846-21854`'s gate), every field its quad
    /// reads. Order is the pool walk order (slot index ascending).
    pub(crate) fn back_gore_key(pool: &tr_core::gore::GorePool) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        let mut any = false;
        for i in 0..tr_core::gore::GORE_SLOTS {
            let g = pool.slot(i);
            if !crate::render::gore::draws_in_back(g) {
                continue;
            }
            any = true;
            i.hash(&mut h);
            g.gore_type.hash(&mut h);
            g.position.x.to_bits().hash(&mut h);
            g.position.y.to_bits().hash(&mut h);
            g.rotation.to_bits().hash(&mut h);
            g.scale.to_bits().hash(&mut h);
            g.alpha.hash(&mut h);
            g.frame.current_column.hash(&mut h);
            g.frame.current_row.hash(&mut h);
            g.frame.column_count.hash(&mut h);
            g.frame.row_count.hash(&mut h);
        }
        if any { h.finish() } else { 0 }
    }
}

#[cfg(test)]
mod frame_key {
    use super::*;

    /// S1-H-15: the Darkness debuff's `blind` flag drops `LightStyle::brightness`
    /// 1.2 → 1.0 (`Lighting.cs:108-116`) and `LightStyle::shade` multiplies every
    /// tile tint by it, so the brightness rides the G6 `WorldFrameKey` — two
    /// frames that differ only in the blind state must not share a cached world
    /// range. (The buff bar itself redraws in the uncached UI pass; chat and the
    /// emote wheel have no draw path yet — S1-E-26 — so the buff bar's world-input
    /// is the only HUD key this frame needs.)
    #[test]
    fn blind_brightness_changes_the_frame_key() {
        let key = |brightness: f32| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: brightness.to_bits(),
            waterfall: 0,
            tile_anim: 0,
            chest_frames: 0,
            cage_anim: 0,
            net_key: 0,
            herb_weather: 0,
            back_gore: 0,
            moon_lord_strength: 0,
            world_revision: 0,
            wof_key: 0,
        };
        assert_ne!(key(1.2), key(1.0), "blind's GlobalBrightness drop must change the key");
        assert_eq!(key(1.2), key(1.2), "an unchanged brightness must not");
    }

    /// S1-G-1f key-coverage witness: the wired tile animator's state rides
    /// the cached world range (`draw_tiles` feeds `animation_row`/counters
    /// into `draw_data`/`case_draw_data`), so two frames that differ only
    /// in animator state must not share a cached range. The key input is
    /// `TileAnimator::anim_key()` — this pins that a changed animation row,
    /// counter or bobframe changes it.
    #[test]
    fn tile_animator_state_changes_the_frame_key() {
        let key = |tile_anim: u64| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: 0,
            waterfall: 0,
            tile_anim,
            chest_frames: 0,
            cage_anim: 0,
            net_key: 0,
            herb_weather: 0,
            back_gore: 0,
            moon_lord_strength: 0,
            world_revision: 0,
            wof_key: 0,
        };
        let mut a = tr_core::drawdata_anim::TileAnimator::new();
        let k0 = key(a.anim_key());
        let mut rng = tr_core::UnifiedRandom::new(1);
        a.animate_tiles(0.5, &mut rng, false);
        let k1 = key(a.anim_key());
        assert_ne!(k0, k1, "one animator tick must change the key");
        assert_eq!(k1, key(a.anim_key()), "an unchanged animator must not");
        // The bobframe alone (a draw input the rows/counters never show)
        // must ride the key too.
        let before = a.anim_key();
        a.weather_vane_bobframe += 1;
        assert_ne!(before, a.anim_key(), "the weather-vane bob rides the key");
    }

    /// W4-d58 key-coverage witness: the chest lid frames feed the chest
    /// rewrite inside the cached world range (`CacheSpecialDraws_Part2`,
    /// `TileDrawing.cs:4309-4320`), so two frames that differ only in a
    /// chest's lid state must not share a cached range. The key input is
    /// `chest::chest_frame_key()` — this pins that the i43 animator's
    /// open/close walk changes it while everything else stands still.
    #[test]
    fn chest_lid_state_changes_the_frame_key() {
        let key = |chest_frames: u64| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: 0,
            waterfall: 0,
            tile_anim: 0,
            chest_frames,
            net_key: 0,
            world_revision: 0,
        };
        let mut chests = vec![crate::chest::WorldChest {
            tile: (10, 20),
            ..Default::default()
        }];
        let k0 = key(crate::chest::chest_frame_key(&chests));
        // One open tick: counter 1 → frame 1 (the +38 half-open row).
        crate::chest::update_chest_frames(&mut chests, 0);
        let k1 = key(crate::chest::chest_frame_key(&chests));
        assert_ne!(k0, k1, "an opening lid must change the key");
        assert_eq!(k1, key(crate::chest::chest_frame_key(&chests)), "a still lid must not");
        // Nine more ticks reach frame 2 (+76); the close walk returns to 0.
        for _ in 0..9 {
            crate::chest::update_chest_frames(&mut chests, 0);
        }
        let k2 = key(crate::chest::chest_frame_key(&chests));
        assert_ne!(k1, k2, "the fully open lid differs from half-open");
        for _ in 0..10 {
            crate::chest::update_chest_frames(&mut chests, -1);
        }
        assert_eq!(k0, key(crate::chest::chest_frame_key(&chests)), "closed again = k0");
    }

    /// W4-f14 key-coverage witness: the cage animator's frame tables ride
    /// the cached world range (`draw_tiles` feeds `frame_of` into the cage
    /// arms' addFrY rewrites), so two frames that differ only in cage
    /// state must not share a cached range.
    #[test]
    fn cage_animator_state_changes_the_frame_key() {
        let key = |cage_anim: u64| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: 0,
            waterfall: 0,
            tile_anim: 0,
            cage_anim,
            net_key: 0,
            world_revision: 0,
        };
        let mut a = tr_core::drawdata_cages::CritterCageAnimator::zeroed();
        let k0 = key(a.anim_key());
        a.critter_cage = true;
        let mut rng = tr_core::UnifiedRandom::new(1);
        a.animate_critter_cages(&mut rng);
        let k1 = key(a.anim_key());
        assert_ne!(k0, k1, "one open-gate cage tick must change the key");
        assert_eq!(k1, key(a.anim_key()), "an unchanged animator must not");
        // The closed gate draws nothing — the key holds (the pre-wiring
        // cache behaviour for cage-free views).
        a.critter_cage = false;
        a.animate_critter_cages(&mut rng);
        assert_eq!(k1, key(a.anim_key()), "the closed gate keeps the key");
    }

    /// W4-g80 key-coverage witness: the 83 herb-maturity sheet swap reads
    /// weather the light key does not carry (`cloudAlpha > 0`, the deathweed
    /// moon gate, the fireblossom `time > 40500` gate), so two frames that
    /// differ only in those gates must not share a cached world range while
    /// an herb 83 is in view. The key input is the three-gate bitmask.
    #[test]
    fn herb_weather_gates_change_the_frame_key() {
        let key = |herb_weather: u32| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: 0,
            waterfall: 0,
            tile_anim: 0,
            net_key: 0,
            herb_weather,
            world_revision: 0,
        };
        // bit 0 = cloudAlpha > 0 (waterleaf), bit 1 = the deathweed moon
        // gate, bit 2 = time > 40500 (fireblossom).
        for gate in [1u32, 2, 4] {
            assert_ne!(key(0), key(gate), "gate {gate} must change the key");
        }
        assert_eq!(key(5), key(5), "unchanged gates must not");
    }

    /// W4-g12 key-coverage witness: `DrawGoreBehind`'s drip quads draw
    /// inside the cached world range, so a drip's motion/frame/alpha must
    /// change `back_gore`; a gore-free pool keeps the key at 0 (the
    /// waterfall convention — the pre-wiring cache behaviour survives).
    #[test]
    fn w4_g12_back_gore_state_changes_the_frame_key() {
        let mut pool = tr_core::gore::GorePool::new();
        assert_eq!(WorldFrameKey::back_gore_key(&pool), 0, "no gores, no key");
        // A hanging drip (type 706, row 3 — outside the fall frames).
        let g = pool.slot_mut(0);
        g.active = true;
        g.gore_type = 706;
        g.frame = tr_core::gore::SpriteFrame::new(1, 15);
        g.frame.current_row = 3;
        g.position = glam::Vec2::new(100.0, 200.0);
        let k0 = WorldFrameKey::back_gore_key(&pool);
        assert_ne!(k0, 0, "a live drip rides the key");
        // Motion changes it.
        pool.slot_mut(0).position.x += 1.0;
        let k1 = WorldFrameKey::back_gore_key(&pool);
        assert_ne!(k0, k1, "drip motion must invalidate the cached range");
        // The frame advance changes it.
        pool.slot_mut(0).frame.current_row = 4;
        assert_ne!(k1, WorldFrameKey::back_gore_key(&pool));
        // The fall frames (7..=9) leave the behind pass — back to 0.
        pool.slot_mut(0).frame.current_row = 8;
        assert_eq!(WorldFrameKey::back_gore_key(&pool), 0, "fall frames draw in front");
    }

    /// W4-d127 re-verify — the `wof_key` bit-pack (`frame/draw.rs`'s
    /// `WorldFrameKey.wof_key`, `Main.DrawWoF`'s draw inputs) is injective
    /// over the reachable band range. The four fields occupy disjoint bit
    /// ranges: clock [0,8), band top [8,24), band bottom [24,40), tongue
    /// [40,41). While a wall lives the bands are always valid (the first
    /// `style27_step` initialises them before any draw; the clamps hold them
    /// in `[underworld+10, +80]*16`, max `2400*16 = 38400 < 2^16`), so the
    /// sign-extended `as i64 as u64` shift never sees a negative and the
    /// fields never overlap. This pins the pack's layout: two states that
    /// differ in ANY one field produce different keys, and the max band
    /// value still fits its 16-bit field without spilling into the tongue.
    #[test]
    fn wof_key_field_layout_is_injective_over_the_band_range() {
        // The pack, verbatim from frame/draw.rs (the wall-present branch).
        let pack = |clock: i32, top: i32, bottom: i32, tongue: bool| -> u64 {
            (clock as u64)
                | ((top as i64 as u64) << 8)
                | ((bottom as i64 as u64) << 24)
                | ((tongue as u64) << 40)
        };
        // The largest reachable band pixel value (Large world 2400 rows).
        let max_band = 2400 * 16; // 38400 < 65536 = 2^16
        assert!(max_band < (1 << 16), "the band fits its 16-bit field");
        // Each field alone changes the key (disjoint ranges).
        let base = pack(0, 1000, 2000, false);
        assert_ne!(base, pack(1, 1000, 2000, false), "the clock rides bits [0,8)");
        assert_ne!(base, pack(0, 1001, 2000, false), "the top rides bits [8,24)");
        assert_ne!(base, pack(0, 1000, 2001, false), "the bottom rides bits [24,40)");
        assert_ne!(base, pack(0, 1000, 2000, true), "the tongue rides bit 40");
        // The max band value does not spill into the tongue bit.
        let hi = pack(17, max_band, max_band, false);
        assert_eq!(hi >> 40, 0, "no band bit reaches the tongue field");
        // The clock wraps at 18 (the mouth clock's range), never reaching
        // the band top's bit 8.
        assert!(pack(17, 0, 0, false) < (1 << 8), "the clock stays below bit 8");
        // A full-range sweep: distinct (top, bottom) pairs stay distinct.
        let mut seen = std::collections::HashSet::new();
        for top in [0, 160, 1600, 16000, max_band] {
            for bottom in [0, 160, 1600, 16000, max_band] {
                assert!(seen.insert(pack(0, top, bottom, false)), "({top},{bottom}) aliases");
            }
        }
    }

    /// W4-d70 key-coverage witness: the Moon Lord parts' `GetMoonLordColoring`
    /// tint (`Main.cs:26325`, `Color.Lerp(White, (183,255,255), strength)`)
    /// rides the cached world range, so two frames that differ only in
    /// `LatestFilterStrength` must not share a cached range. The key input is
    /// the strength's bits, gated on a Moon Lord part being present (zero
    /// otherwise — Moon-Lord-free worlds keep the pre-wiring cache behaviour).
    #[test]
    fn moon_lord_strength_changes_the_frame_key() {
        let key = |strength: f32| WorldFrameKey {
            cam_tile: [0, 0],
            zoom: 0,
            screen: [0, 0],
            sky_light: [0, 0, 0],
            hell_glow: 0,
            light_params: 0,
            light_mode: 0,
            anim: 0,
            player_key: 0,
            npc_key: 0,
            items_key: 0,
            hand_key: 0,
            light_generation: 0,
            brightness: 0,
            waterfall: 0,
            tile_anim: 0,
            net_key: 0,
            moon_lord_strength: strength.to_bits(),
            world_revision: 0,
        };
        assert_ne!(key(1.0), key(0.5), "a changed strength must change the key");
        assert_eq!(key(1.0), key(1.0), "an unchanged strength must not");
    }
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

impl App {
    /// The swapchain size in pixels (the deterministic fallback mirrors the
    /// `TR_SHOT`/headless default of 1280x720).
    pub(crate) fn screen_size(&self) -> (f32, f32) {
        world_sim::camera::screen_size(self)
    }

    /// The per-call inputs `Dust.NewDust` reads off `Main` (`Dust.cs:77-98`),
    /// S1-E-1 phase 2: `gameMenu` is the front state (`menu.is_some() ||
    /// front.is_some()`, the mod.rs:1054 gate), `gamePaused` is S1-B-3's
    /// field, sync loads never generate (`isGeneratingOrLoadingWorld` only
    /// covers the async loader), `screenPosition` is the view's world-px
    /// top-left and `screenWidth/Height` the UI size the C# cull rect uses
    /// verbatim (`Dust.cs:97-101`).
    pub(crate) fn spawn_ctx(&self) -> tr_core::particles::SpawnCtx {
        let (sw, sh) = self.screen_size();
        spawn_ctx_from(
            self.menu.is_some() || self.front.is_some(),
            self.game_paused,
            self.camera,
            Vec2::new(sw, sh),
            self.zoom,
            self.gfx_quality.max_dust_to_draw,
        )
    }

    /// `Main.mH` (`Main.cs:45771-45783`): 256 while the minimap HUD is live,
    /// clamped by the 600 px equipment push-up — the same value `frame()`
    /// hands the inventory draw, so the HUD hit tests and the draw agree.
    pub(crate) fn main_m_h(&self) -> f32 {
        let minimap_live = self.settings.map_enabled
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        crate::ui::minimap::main_m_h(minimap_live, self.screen_size().1)
    }

    /// `ChildSafety.Disabled = GoreVisualsAllowed` — the settings load/store
    /// pair (`Main.cs:4101`, `:4349`) and the toggle (`:50055`). The blood
    /// toggle ON is the vanilla default (the gate skipped); OFF arms the
    /// spawn-side re-type (`ChildSafety.cs:33`). The menu owns the live copy
    /// while it is open, so `menu_blood` carries that copy's flag and `None`
    /// falls back to the stored settings; run once per frame from `frame()`.
    pub(crate) fn sync_child_safety(&mut self, menu_blood: Option<bool>) {
        let disabled = menu_blood.unwrap_or(self.settings.blood);
        self.dust.child_safety.disabled = disabled;
        // `ChildSafety.Disabled` is one static in the C# (`ChildSafety.cs:15`),
        // so the gore spawn gate reads the same flag.
        self.gore.child_safety.disabled = disabled;
    }

    /// `Main.DrawPageIcons`' tail (`Main.cs:42864-42867`): `EquipPage` is the
    /// page the held item forces (`ItemSlot.SelectEquipPage`,
    /// `ItemSlot.cs:4622-4652`), falling back to `Main.EquipPageSelected`. Run
    /// once per frame from `frame()` so the auto-jump is undone as soon as the
    /// cursor stack empties.
    pub(crate) fn resolve_equip_page(&mut self) {
        let forced = crate::ui::chrome::select_equip_page(
            self.mouse_item.kind,
            self.mouse_item.count,
            self.equip_page_selected,
        );
        self.equip_page = if forced == -1 {
            self.equip_page_selected
        } else {
            forced
        };
    }

    pub(crate) fn update_player(&mut self, dt: f32) {
        world_sim::player::update_player(self, dt);
    }

    /// `Main.mouseNPC`'s per-frame lifecycle
    /// (`DrawInterface_38_MouseCarriedObject`, `Main.cs:45182-45211` +
    /// `DrawInterface_25`'s closed-inventory clear, `:45664-45667`):
    /// a carried item drops the head (`:45185-45187`), a closed inventory
    /// drops it (`:45666`), and a carried NPC that died or changed type
    /// downgrades to the housing query (`:45205-45211`). Runs once per
    /// frame beside `resolve_equip_page`; the click handler re-checks the
    /// item arm after its own pickups so the drop is same-frame.
    pub(crate) fn refresh_mouse_npc(&mut self) {
        if self.mouse_item.count > 0 || !self.inventory_open {
            // `SetMouseNPC(-1, -1)` (`Main.cs:3176-3180`).
            self.mouse_npc_index = -1;
            self.mouse_npc_type = -1;
            return;
        }
        if self.mouse_npc_index >= 0 {
            let live = self
                .npcs
                .get(self.mouse_npc_index as usize)
                .is_some_and(|n| n.kind.net_id() == self.mouse_npc_type);
            if !live {
                // `SetMouseNPC_ToHousingQuery()` (`Main.cs:3182-3185`).
                self.mouse_npc_index = -1;
                self.mouse_npc_type = 0;
            }
        }
    }

    /// `Main.DoUpdate_F11_ToggleUI` (`Main.cs:18402-18424`): the F11
    /// screenshot-mode toggle. `releaseUI` is the press latch — the flip
    /// fires only while it stands, then drops until the key releases. The
    /// flip plays `SoundEngine.PlaySound(12)` (`:18416`). The C# runs the
    /// pass on every update (menus included, `:17156`); the port routes it
    /// through `world_key` (the in-world key arm) — the menu/front screens
    /// own their keys, and `hideUI` only gates the in-world interface.
    pub(crate) fn toggle_hide_ui(&mut self, f11_down: bool) {
        if f11_down {
            if self.release_ui {
                self.hide_ui = !self.hide_ui;
                self.audio.play("Menu_Tick");
            }
            self.release_ui = false;
        } else {
            self.release_ui = true;
        }
    }

    /// `Player.cs:24774-24793` + `24898-24908`: the fullscreen-map controls. The pan
    /// step is `16 / scale` tiles **once per frame, with no `dt` factor** (the
    /// original runs it in the per-frame input pass).
    pub(crate) fn update_map_input(&mut self) {
        let (sw, _) = self.screen_size();
        let Some(m) = self.map.as_mut() else { return };
        let step = 16.0 / m.scale;
        let binds = self.keybinds.clone();
        let map_binds = self.map_binds.clone();
        let k = |code| self.keys.contains(&code);
        // The pan rows are the movement binds (`Main.cUp`..`Main.cRight`,
        // `Main.cs:67163-67166`); the arrows stay as the port's convenience.
        let held = |action: crate::input_map::Action, arrow: KeyCode| {
            self.keys
                .iter()
                .any(|k| crate::input_map::is_bound(&binds, action, *k))
                || k(arrow)
        };
        if held(crate::input_map::Action::Up, KeyCode::ArrowUp) {
            m.pos[1] -= step;
        }
        if held(crate::input_map::Action::Down, KeyCode::ArrowDown) {
            m.pos[1] += step;
        }
        if held(crate::input_map::Action::Left, KeyCode::ArrowLeft) {
            m.pos[0] -= step;
        }
        if held(crate::input_map::Action::Right, KeyCode::ArrowRight) {
            m.pos[0] += step;
        }
        // `Main.cs:67191-67192`: `cMapZoomIn = "Add"`, `cMapZoomOut =
        // "Subtract"` — the menuMode 272727 rows, not a fixed key.
        if self.keys.iter().any(|k| {
            map_binds[crate::input_map::MapAction::ZoomIn.index()].as_str()
                == crate::input_map::key_name(*k)
        }) {
            m.scale = map::map_key_scale(m.scale, true);
        }
        if self.keys.iter().any(|k| {
            map_binds[crate::input_map::MapAction::ZoomOut.index()].as_str()
                == crate::input_map::key_name(*k)
        }) {
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

    /// `Player.cs:24909-24965` — the NOT-fullscreen map-key block: the
    /// MapZoomIn/Out and MapAlphaUp/Down triggers step the minimap
    /// (`mapStyle == 1`, ×1.025/×0.975, ±0.015) or the overlay
    /// (`mapStyle == 2`, ×1.05/×0.95, ±0.015), and the MapStyle trigger
    /// (Tab) cycles `Main.mapStyle` 0→1→2→0 with sound 12 on the
    /// `releaseMapStyle` press latch. Runs once per player tick while the
    /// map is enabled and NOT fullscreen (the fullscreen arm lives in
    /// [`Self::update_map_input`]); the clamps are the draw-side ones
    /// (`Main.cs:55515-55530` / `:55550-55566`).
    pub(crate) fn update_map_style_keys(&mut self) {
        if !self.settings.map_enabled || self.map.as_ref().is_some_and(|m| m.open) {
            return;
        }
        let (sw, _) = self.screen_size();
        let map_binds = self.map_binds.clone();
        let bound = |action: crate::input_map::MapAction| {
            self.keys.iter().any(|k| {
                map_binds[action.index()].as_str() == crate::input_map::key_name(*k)
            })
        };
        let zoom_in = bound(crate::input_map::MapAction::ZoomIn);
        let zoom_out = bound(crate::input_map::MapAction::ZoomOut);
        let alpha_up = bound(crate::input_map::MapAction::AlphaUp);
        let alpha_down = bound(crate::input_map::MapAction::AlphaDown);
        if let Some(m) = self.map.as_mut() {
            match self.map_style {
                // `Player.cs:24911-24928`.
                1 => {
                    if zoom_in {
                        m.minimap_scale = map::minimap_key_scale(m.minimap_scale, true);
                    }
                    if zoom_out {
                        m.minimap_scale = map::minimap_key_scale(m.minimap_scale, false);
                    }
                    if alpha_up {
                        m.minimap_alpha = map::map_key_alpha(m.minimap_alpha, true);
                    }
                    if alpha_down {
                        m.minimap_alpha = map::map_key_alpha(m.minimap_alpha, false);
                    }
                    // `Main.cs:55515-55530`.
                    m.minimap_scale = ui::minimap::clamp_minimap_scale(m.minimap_scale);
                    m.minimap_alpha = map::map_alpha_clamp(m.minimap_alpha);
                }
                // `Player.cs:24930-24947`.
                2 => {
                    if zoom_in {
                        m.overlay_scale = map::map_key_scale(m.overlay_scale, true);
                    }
                    if zoom_out {
                        m.overlay_scale = map::map_key_scale(m.overlay_scale, false);
                    }
                    if alpha_up {
                        m.overlay_alpha = map::map_key_alpha(m.overlay_alpha, true);
                    }
                    if alpha_down {
                        m.overlay_alpha = map::map_key_alpha(m.overlay_alpha, false);
                    }
                    // `Main.cs:55550-55566`.
                    m.overlay_scale =
                        map::overlay_scale_clamp(sw, self.world.width, self.zoom, m.overlay_scale);
                    m.overlay_alpha = map::map_alpha_clamp(m.overlay_alpha);
                }
                _ => {}
            }
        }
        // `Player.cs:24949-24965` — the style cycle rides `mapStyle != 0`
        // (the `if (this.mapStyle)` trigger, not the Main.mapStyle value):
        // the Tab press cycles 0→1→2→0 with sound 12, on the release latch.
        if bound(crate::input_map::MapAction::Style) {
            if self.release_map_style {
                self.audio.play("Menu_Tick");
                self.map_style += 1;
                if self.map_style > 2 {
                    self.map_style = 0;
                }
            }
            self.release_map_style = false;
        } else {
            self.release_map_style = true;
        }
    }

    /// `Main.UpdateViewZoomKeys` (`Main.cs:16801-16815`): ViewZoomIn/Out adjust
    /// `GameZoomTarget` by 0.02 per update while held, clamped to 1.0..2.0.
    /// Port uses default keys OemPlus/OemMinus (`KeyCode::Equal`/`Minus`);
    /// skipped while in fancy UI / front/menu screens.
    pub(crate) fn update_view_zoom_keys(&mut self) {
        // S1-B-5: TR_ZOOM pins the zoom for screenshots and must not be moved
        // by key input (and must not widen the vanilla 1.0..2.0 clamp).
        if self.zoom_pinned {
            return;
        }
        if self.menu.is_some() || self.front.is_some() {
            return;
        }
        let mut changed = false;
        if self.keys.contains(&KeyCode::Equal) {
            self.zoom = (self.zoom + 0.02).clamp(1.0, 2.0);
            changed = true;
        }
        if self.keys.contains(&KeyCode::Minus) {
            self.zoom = (self.zoom - 0.02).clamp(1.0, 2.0);
            changed = true;
        }
        if changed {
            self.settings.zoom = self.zoom;
        }
    }

    /// `Main.MaxWorldViewSize` (`Main.cs:677-681`): the design view the zoom
    /// floor keeps the world inside.
    pub(crate) const MAX_WORLD_VIEW_SIZE: (f32, f32) = (1920.0, 1200.0);
    /// `Main.SupportWideScreen` (`Main.cs:1819`) — the C# init `false`
    /// (`Main.cs:67046`). The port has no settings row for it (the settings
    /// work owns the row), so the computed arm runs (S1-G-7).
    pub(crate) const SUPPORT_WIDE_SCREEN: bool = false;

    /// `Main.ForcedMinimumZoom` (`Main.cs:61989-61994`, computed per draw):
    /// `max(max(1f, screenWidth / MaxWorldViewSize.X), screenHeight /
    /// MaxWorldViewSize.Y)` — the zoom floor that keeps the visible world
    /// inside the design size. `Main.SupportWideScreen` forces it to 1f
    /// (`:61992-61994`). The two view matrices consume it split:
    /// `BackgroundViewMatrix.Zoom = ForcedMinimumZoom` (`:61998`) and
    /// `GameViewMatrix.Zoom = ForcedMinimumZoom * Clamp(GameZoomTarget, 1f,
    /// 2f)` (`:61999`) — the port's `zoom` is that clamp (S1-B-5).
    pub(crate) fn forced_minimum_zoom(screen: (f32, f32)) -> f32 {
        if Self::SUPPORT_WIDE_SCREEN {
            return 1.0;
        }
        let val = screen.0 / Self::MAX_WORLD_VIEW_SIZE.0;
        let y_arm = screen.1 / Self::MAX_WORLD_VIEW_SIZE.1;
        val.max(1.0).max(y_arm)
    }

    /// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
    pub(crate) fn advance_disco(&mut self) {
        world_sim::clock::advance_disco(self);
    }

    /// The front-end half of `Main.UpdateAudio` (`Main.cs:17816`): the audio
    /// clock runs on every frame, so the title/character/world screens are
    /// not silent (BUG-01). `Main.UpdateAudio_DecideOnNewMusic`'s `gameMenu`
    /// arm (`Main.cs:12455-12532`) picks the title track; the worldgen seed
    /// flags it reads come from the world-creation screen's seed text (the
    /// vanilla's `WorldGen.drunkWorldGen` and friends), and the other front
    /// screens play the plain title track.
    pub(crate) fn update_menu_music(&mut self) {
        let menu_flags = match &self.front {
            Some(FrontScreen::WorldCreation(draft)) => {
                tr_core::seeds::SeedFlags::from_seed_text(&draft.seed)
            }
            _ => tr_core::seeds::SeedFlags::default(),
        };
        self.music.update(audio::MusicEnv {
            music_volume: self.settings.music_volume,
            menu: true,
            menu_flags,
            // `Main.activeMusicBox = -1` — the derived default 0 is the first
            // box's id, which would override the title track.
            music_box: -1,
            ..Default::default()
        });
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

    /// `Main.GetPreferredBGStyleForPlayer` (`Main.cs:64211-64256`): the surface
    /// background style wanted by the biome around the scene centre.
    ///
    /// Two deviations, both because the style they would select is not ported:
    /// the forest sub-styles 10/11/12 need `Main.treeX[]` and the three extra
    /// tree sets (only tree set 1 / `Background_50..52` is packed), and the
    /// desert-variant styles 5/13/14 need `Main.desertBackgroundSet`. The forest
    /// branch therefore stays 0 and the desert branch stays 2.
    pub(crate) fn preferred_bg_style(&self) -> i32 {
        // `Main.SceneMetrics.Zone*`, read in the C#'s order.
        let m = &self.scene_metrics;
        if m.zone_beach {
            if m.zone_hallow {
                6
            } else if m.zone_corrupt {
                if m.blood_tile_count <= m.evil_tile_count {
                    1
                } else {
                    8
                }
            } else if m.zone_crimson {
                8
            } else if m.honey_block_count <= 400 {
                4
            } else {
                3
            }
        } else if m.enough_tiles_for_glowing_mushroom() {
            9
        } else if m.zone_desert {
            // `(corrupt && evil > blood && evil > holy) ? 5 : ((crimson &&
            // blood > holy) ? 14 : (!hallow ? 2 : 13))` — 5/13/14 unported.
            2
        } else if m.zone_hallow {
            6
        } else if m.zone_corrupt {
            if m.blood_tile_count <= m.evil_tile_count {
                1
            } else {
                8
            }
        } else if m.zone_crimson {
            8
        } else if m.zone_jungle {
            3
        } else if m.zone_snow {
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
            self.world.jungle_bg_style as i32,
            self.world.hallow_bg_style as i32,
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

    /// `Main.DrawInventory`'s shop-close rule (`Main.cs:41784-41787`):
    /// `npcShop > 0 && (!playerInventory || talkNPC == -1)` →
    /// `SetNPCShopIndex(0)`. The port's shop opens collapse the talk onto
    /// the open (`try_open_town_shop_at_cursor`), so the talk index rides
    /// `shop_npc` — the inventory close is the operative half. Runs in the
    /// click cluster beside the other interface handlers (the C# runs it
    /// inside `DrawInventory`).
    pub(crate) fn update_shop_close(&mut self) {
        if self.shop_npc.is_some() && !self.inventory_open {
            self.close_shop();
        }
    }

    /// One unit of the shop buy — `ItemSlot.HandleShopSlot`'s loop body
    /// (`UI/ItemSlot.cs:2053-2095`) with the `BuyItem` charge
    /// (`Player.cs:35400-35414`): the purse covers the row's buying value
    /// → deduct, memo the purchase (`shopSellbackHelper.Add`, `:2084-2087`
    /// — plain-currency non-buyOnce rows only), push ONE unit onto the
    /// cursor (`Main.mouseItem`, `:2075-2088` — the vanilla buy lands on
    /// the CURSOR, not the inventory) and deplete a buyOnce row
    /// (`:2090-2093`). The caller gates the cursor room and plays the
    /// first-unit cue. Returns false when the purse is short, the slot is
    /// out of range or the row is air.
    ///
    /// S1-P-i54: the Digit1 shop-buy arm is gone (vanilla `Hotbar1` always
    /// selects, `Player.cs:25095-25097`; the shop buys are mouse-only,
    /// `ItemSlot.cs:1220`/`:1650`). The consumer is the shop-grid click
    /// (`app/clicks.rs`).
    ///
    /// Deviation: the charge rides the port's aggregate `App.coins` purse
    /// (the recorded coin model, ANALYSIS) — the C# `BuyItem` spends the
    /// inventory + bank coin stacks through `TryPurchasing`.
    pub(crate) fn buy_shop_item(&mut self, slot: usize) -> bool {
        // `ItemSlot.HandleShopSlot` (`UI/ItemSlot.cs:2059-2060`): the
        // charge is `GetItemExpectedPrice`'s buying value for the stock
        // ROW (the row's `shopCustomPrice`/`buyOnce` ride
        // `Item.GetStoreValue`, `Item.cs:49901-49908`).
        let Some(entry) = self.shop_stock.get(slot) else {
            return false;
        };
        if entry.item == 0 || entry.stack == 0 {
            return false; // `:2043`'s `inv[slot].type <= 0` / `:2060`'s stack gate
        }
        // `Player.BuyItem`'s custom-currency arm (`Player.cs:35402-35405`)
        // spends the medal wallet (`CustomCurrencySingleCoin.TryPurchasing`
        // over the bank's 3817 stacks) — the wallet is unported, so a
        // medal row refuses here rather than charging the coin purse
        // (the recorded CustomCurrencyManager gap).
        if entry.shop_special_currency != -1 {
            return false;
        }
        let price =
            crate::shop::buy_price_row(entry, self.discount_available, self.shop_mood) as u64;
        if self.coins < price {
            return false;
        }
        // `:2075-2083` — an air cursor takes the row's kind (and prefix);
        // the unit rides the cursor stack (`:2088`).
        if self.mouse_item.count == 0 {
            self.mouse_item = ItemSlot {
                kind: entry.item,
                name: crate::itemdata::ITEMS[entry.item as usize].name,
                count: 0,
                prefix: entry.prefix,
            };
        }
        // `:2084-2087` — the sellback memo (the medal rows refuse above,
        // so the `shopSpecialCurrency == -1` half is always true here).
        if !entry.buy_once {
            self.shop_sellback.add(entry.item, entry.prefix, 1);
        }
        self.mouse_item.count += 1;
        // `:2090-2093` — a buyOnce row depletes; at 0 it turns to air
        // (`SetDefaults(0)`, `Item.cs:48685-48688`).
        let entry = &mut self.shop_stock[slot];
        if entry.buy_once {
            entry.stack = entry.stack.saturating_sub(1);
            if entry.stack == 0 {
                *entry = crate::shop::ShopEntry {
                    item: 0,
                    when: crate::shop::StockWhen::Always,
                    shop_custom_price: None,
                    shop_special_currency: -1,
                    buy_once: false,
                    is_a_shop_item: false,
                    prefix: 0,
                    stack: 0,
                };
            }
        }
        self.coins -= price;
        true
    }

    /// `Player.SellItem`'s transaction half (`Player.cs:35168-35329`) over
    /// the port's purse: the payout (`shop::sell_payout` — the ÷5, the
    /// stack multiply and the sellback refund) lands in `App.coins`;
    /// returns the payout, or 0 on the `calcForSelling <= 0` refusal
    /// (`:35170-35174`). The C# spawns the payout as coin stacks through
    /// the four denomination loops (74/73/72/71, descending slots 53→0,
    /// `DoCoins` carrying at 100) and restores the cloned inventory when
    /// no room exists (`:35321-35328`) — the aggregate purse cannot fill
    /// up, so the refusal half folds out (the recorded coin-model
    /// deviation, ANALYSIS). The item removal + `AddItemToShop` ride the
    /// caller (`ItemSlot.cs:854-867`/`:1225-1238`).
    pub(crate) fn sell_item(&mut self, item: &ItemSlot) -> i64 {
        let value = crate::prefix::prefixed_stats(item.kind, item.prefix)
            .map(|s| s.value)
            .unwrap_or(crate::itemdata::ITEMS[item.kind as usize].value);
        let amount = self.shop_sellback.get_amount(item.kind, item.prefix);
        let payout = crate::shop::sell_payout(
            value,
            false,
            false,
            self.discount_available,
            self.shop_mood,
            item.count,
            amount,
        );
        if payout <= 0 {
            return 0;
        }
        self.coins += payout as u64;
        payout
    }

    /// The reforge slot transfer (`ItemSlot.Handle(ref reforgeItem, 5)`,
    /// `ItemSlot.cs:588-597` → `LeftClick`'s action-1 arms). The ctx-5
    /// movement gate (`PickItemMovementAction`, `ItemSlot.cs:1515-1520`)
    /// accepts only a prefixable cursor (`CanHavePrefixes`) or air — a
    /// non-prefixable cursor refuses the move outright. Then the three
    /// action-1 arms (`ItemSlot.cs:1019-1106`):
    /// - cursor stack 1 + held item + `IsNotTheSameAs` → swap (`:1020-1023`);
    /// - air cursor + held item → take it out (`:1044-1046`);
    /// - cursor stack 1 + empty slot → place it (`:1060-1077`);
    /// - cursor stack >1 + empty slot → clone ONE in (`:1078-1084`).
    /// Every arm that moves an item plays `SoundEngine.PlaySound(7)`.
    pub(crate) fn reforge_slot_click(&mut self) {
        // `ItemSlot.cs:1515-1520` — ctx 5 → action 1 only when the cursor is
        // prefixable or air; otherwise action -1 (the `:928-931` gate refuses).
        let cursor_prefixable =
            self.mouse_item.kind == 0 || crate::prefix::can_have_prefixes(self.mouse_item.kind);
        if !cursor_prefixable {
            return;
        }
        let moved = if self.mouse_item.count == 1
            && self.mouse_item.kind != 0
            && self.reforge_slot.kind != 0
        {
            // `:1020-1023` — swap only when the two are NOT the same
            // (`IsNotTheSameAs`, `Item.cs:49847-49854`: same type+stack but a
            // different prefix, or a different type/stack).
            let same = self.reforge_slot.kind == self.mouse_item.kind
                && self.reforge_slot.count == self.mouse_item.count
                && self.reforge_slot.prefix == self.mouse_item.prefix;
            if !same {
                std::mem::swap(&mut self.mouse_item, &mut self.reforge_slot);
                true
            } else {
                false
            }
        } else if self.mouse_item.kind == 0 && self.reforge_slot.kind != 0 {
            // `:1044-1046` — air cursor takes the held item out.
            std::mem::swap(&mut self.mouse_item, &mut self.reforge_slot);
            true
        } else if self.mouse_item.kind != 0 && self.reforge_slot.kind == 0 {
            if self.mouse_item.count == 1 {
                // `:1060-1077` — a single item places whole.
                std::mem::swap(&mut self.mouse_item, &mut self.reforge_slot);
            } else {
                // `:1078-1084` — a stack clones ONE into the empty slot.
                self.reforge_slot = self.mouse_item;
                self.reforge_slot.count = 1;
                self.mouse_item.count -= 1;
            }
            true
        } else {
            false
        };
        if moved {
            self.audio.play("Grab"); // `SoundEngine.PlaySound(7)`
        }
    }

    /// `Main.ReforgeItemInReforgeSlot` (`Main.cs:41902-41933`): the charge
    /// (`BuyItem`'s purse gate, `:41658`) precedes the roll; then the reset-to-
    /// defaults + `Prefix(-2)` — the port recomputes the stats from the
    /// kind's defaults, so the reset is the prefix-field write — and the
    /// top-tier 60-tick cooldown (`Main.cs:41907-41910`). The presentation
    /// tail (`Main.cs:41906-41932`) — the `BestReforge`/`Item37` sound and the
    /// top-tier particle draws — runs AFTER the roll on the same shared
    /// `Main.rand` (`prefix::reforge_presentation_tail`; the particle visuals
    /// are the documented skip, the draws are consumed verbatim). Both the
    /// roll and the tail consume the bare-`Main.rand` stand-in
    /// (`NamedRngs::global`) — the menu path runs outside every swap
    /// (`S1-A-11` §4).
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
            self.discount_available,
        );
        if self.reforge_cooldown > 0 || self.coins < cost {
            return;
        }
        self.coins -= cost;
        // `Prefix(-2)` draws `Main.rand` — the menu path runs outside every
        // swap, so the port's bare-`Main.rand` stand-in (`NamedRngs::global`,
        // `app/rng.rs`) feeds it.
        let kind = self.reforge_slot.kind;
        let (prefix, top_tier) = crate::prefix::reforge_roll(kind, self.rngs.global());
        self.reforge_slot.prefix = prefix;
        if top_tier {
            self.reforge_cooldown = 60; // `Main.cs:41910`
        }
        // `PopupText.NewText(...)` (`Main.cs:41906`, W4-g57): the new-prefix
        // reveal — the affix name floats up from the player's centre
        // (`Main.LocalPlayer.Center`), `noStack: true`. VISUAL only: the
        // spawn draws no `Main.rand` (`PopupText.cs:194-412`), so the i45
        // tail below keeps its exact draw order. The `showItemText` gate
        // (`PopupText.cs:196-199`) rides `settings.pickup_text`.
        if self.settings.pickup_text {
            let name = crate::lang::prefixed_item_name(kind, prefix);
            let stack = self.reforge_slot.count.max(1) as i32;
            let display = crate::popup_text::display_text(&name, stack);
            let text_size = [
                crate::ui::text::ui_font_width(&self.lookups, &display, 1.0)
                    .unwrap_or(display.chars().count() as f32 * 8.0),
                crate::ui::text::ui_font_height(&self.lookups, 1.0).unwrap_or(8.0),
            ];
            let (cx, cy) = self
                .player
                .as_ref()
                .map(|p| p.center())
                .unwrap_or((0.0, 0.0));
            let rare = crate::prefix::prefixed_stats(kind, prefix)
                .map(|s| s.rare)
                .unwrap_or(0);
            crate::popup_text::spawn_reforge(
                &mut self.popup_texts,
                top_tier,
                &name,
                stack,
                [cx, cy],
                crate::popup_text::popup_rarity_color(rare),
                rare == -13, // `PrepareEffects` (`PopupText.cs:397-400`)
                false,       // the port's item table carries no expert flag
                text_size,
                self.world.height as f32 * 16.0, // `Main.bottomWorld`
            );
        }
        // The presentation tail (`Main.cs:41906-41932`): the sound + the
        // top-tier particle draws, in the C#'s order on the same global
        // stream (the roll's draws are already consumed above).
        crate::prefix::reforge_presentation_tail(top_tier, self.rngs.global(), &mut self.audio);
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
    /// key space is the vanilla inventory numbering — grid `0..50`, coins
    /// `50..54`, ammo `54..58` (`crate::inv::coin_slot_index` /
    /// `ammo_slot_index`), i.e. `canFavoriteAt`'s contexts 0/1/2
    /// (`ItemSlot.cs:386-389`).
    pub(crate) fn toggle_favorite(&mut self, idx: usize) -> u32 {
        if idx >= crate::inv::INV_SLOTS {
            return 0;
        }
        if let Some(pos) = self.favorite_slots.iter().position(|&i| i == idx) {
            self.favorite_slots.remove(pos);
        } else {
            self.favorite_slots.push(idx);
        }
        12
    }

    /// `Main.PendingPlayer`'s favourites on the front end
    /// (`UICharacterSelect.cs:110`): a card's `.plr` bitfield hydrates the
    /// live slots when the character is played (BUG-03). The bits are the
    /// `Item.favorited` flags (`Player.cs:54820-54824`) in slot order.
    pub(crate) fn hydrate_favorites(&mut self, bits: u64) {
        let mut slots = Vec::new();
        for i in 0..crate::inv::INV_SLOTS {
            if bits & (1 << i) != 0 {
                slots.push(i);
            }
        }
        self.favorite_slots = slots;
    }

    /// The save-side mirror of [`Self::hydrate_favorites`]: the live
    /// `favorite_slots` flattened into the `.plr` bitfield
    /// (`Player.cs:54820-54824`).
    pub(crate) fn favorite_bits(&self) -> u64 {
        let mut bits = 0u64;
        for &i in &self.favorite_slots {
            if i < crate::inv::INV_SLOTS {
                bits |= 1 << i;
            }
        }
        bits
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
    /// `dropItemCheck`); the slot item returns to the inventory through
    /// `Player.GetOrDropItem` (`Player.cs:7193-7200`, `GetItemSettings.
    /// ReturnItemFromSlot`): the pickup stack first, and the OVERFLOW spawns
    /// a world drop at the player's centre with the prefix preserved
    /// (`Item.RequestNewItem(..., item2.prefix, ...)`, `:7198`) — the old
    /// port silently destroyed the item when the inventory was full.
    /// `Main.UpdateOldNPCShop` (`Main.cs:18204-18211`) clears the sellback
    /// memo on the index change — the close is one.
    pub(crate) fn close_shop(&mut self) {
        self.shop_npc = None;
        self.shop_stock.clear();
        self.shop_sellback.clear();
        self.shop_mood = 1.0;
        self.in_reforge_menu = false;
        self.return_reforge_slot();
    }

    /// `Player.dropItemCheck`'s reforge arm (`Player.cs:5123-5127`): with the
    /// menu closed, `GetOrDropItem(Main.reforgeItem, ReturnItemFromSlot)`
    /// returns the slot item to the inventory (the pickup stack first) and
    /// clears it; the OVERFLOW spawns a world drop at the player's centre
    /// with the prefix preserved (`Player.cs:7196-7199`, `Item.RequestNewItem`
    /// keeps the rolled prefix — `Item.NewItem`'s `Prefix(exact)` applies it
    /// directly, `Item.cs:49730`, no fresh roll). Shared by `close_shop` and
    /// the inventory-close chain (`Main.cs:45676-45683`).
    pub(crate) fn return_reforge_slot(&mut self) {
        if self.reforge_slot.count > 0 {
            if let Some(i) = crate::item::slot_for_pickup(
                &mut self.inventory,
                self.reforge_slot.kind,
                self.reforge_slot.prefix,
            ) {
                self.inventory[i].count = (self.inventory[i].count + self.reforge_slot.count)
                    .min(crate::registry::max_stack(self.reforge_slot.kind));
                self.reforge_slot = EMPTY_SLOT;
            } else {
                // The velocity/phase triplet rides the port's pinned ItemDrop
                // model on the bare-`Main.rand` stand-in (the close runs from
                // the click/talk handlers outside every swap, S1-A-11 §4 —
                // the recorded spawn-order deviation, `world_sim/items.rs`).
                let (cx, cy) = self
                    .player
                    .as_ref()
                    .map(|p| p.center())
                    .unwrap_or((0.0, 0.0));
                let rng = self.rngs.global();
                let vx = (rng.next_float() * 2.0 - 1.0) * 1.2;
                let vy = -1.5 - rng.next_float();
                let phase = rng.next_float() * std::f32::consts::TAU;
                self.items.push(ItemDrop::new(
                    self.reforge_slot.kind,
                    self.reforge_slot.count,
                    self.reforge_slot.prefix,
                    cx,
                    cy,
                    vx,
                    vy,
                    phase,
                ));
                self.reforge_slot = EMPTY_SLOT;
            }
        }
    }

    /// The reforge menu's per-frame head (`Main.DrawInventory`'s inline block,
    /// `Main.cs:41557-41578`): the button's hover-scale animation and the
    /// unhovered cooldown clear, then the force-close gate. Runs every frame
    /// the menu is open (the C# block sits in `DrawInventory`, which runs
    /// whenever `playerInventory`). The hover latch (`mouse_reforge`) is
    /// resolved here from the button hit box (`:41649-41666`, the C# sets it
    /// in the draw block every frame); the scale ramps +0.02/frame toward 1
    /// while hovered, −0.02 toward 1 while not (`:41561-41572`), and an
    /// unhovered button clears the cooldown (`:41568`). The force-close
    /// (`:41574-41578`) fires when a chest opens, the shop closes (the port's
    /// collapsed `talkNPC == -1`/`npcShop != 0` pair — the menu rides beside
    /// the Tinkerer shop), or the crafting panel opens.
    pub(crate) fn update_reforge_menu(&mut self) {
        if !self.in_reforge_menu {
            return;
        }
        // `:41649-41666` — the button hover latch + the hover-edge sound 12.
        // The C# sets `mouseReforge` in the draw block every frame the button
        // draws (the slot holds an item); the port resolves the same hit box
        // here (the per-frame head) so the scale/cooldown update below reads
        // the live latch.
        let hovered = self.reforge_slot.kind > 0
            && crate::ui::inventory::reforge_button_hit(self.mouse_pos);
        if hovered && !self.mouse_reforge {
            crate::chest::play_menu_sound(self, 12); // `:41652-41655`
        }
        self.mouse_reforge = hovered;
        // `:41559-41572` — the hover-scale animation + the unhovered clear.
        if self.mouse_reforge {
            if self.reforge_scale < 1.0 {
                self.reforge_scale += 0.02;
            }
        } else {
            self.reforge_cooldown = 0; // `:41568`
            if self.reforge_scale > 1.0 {
                self.reforge_scale -= 0.02;
            }
        }
        // `:41574-41578` — the force-close gate. The C# closes on
        // `chest != -1 || npcShop != 0 || talkNPC == -1 || InGuideCraftMenu`.
        // The port collapses the Tinkerer's chat-button entry into the
        // shop-open (the recorded #9 deviation — `world_sim/npcs.rs:971`
        // auto-opens the menu beside the shop with `talk_npc = None`), so the
        // `talkNPC == -1`/`npcShop != 0` pair maps to the port's shop-closed
        // state (`shop_npc.is_none()`): the menu lives while its shop is open
        // and closes when the shop closes. A container opening
        // (`open_container`) or the crafting panel (`crafting_open`) also
        // force-closes, exactly the C#'s `chest != -1`/`InGuideCraftMenu`.
        let force_close = self.open_container() != crate::bank::CHEST_CLOSED
            || self.shop_npc.is_none()
            || self.crafting_open;
        if force_close {
            self.in_reforge_menu = false;
            self.mouse_reforge = false;
            self.return_reforge_slot(); // `dropItemCheck` (`Player.cs:5123-5127`)
        }
    }

    /// `Main.GetNurseHealCost` (`Main.cs:40054-40100`): the missing life
    /// plus the removable debuffs x100 — a slot counts when
    /// `Main.debuff[b] && buffTime > 60 && !NurseCannotRemoveDebuff[b]` —
    /// then the boss-progression ladder, the discount and the happiness
    /// adjustment. The expert x2 stays constants-off (the difficulty
    /// plumbing rides R-03-6).
    pub(crate) fn nurse_heal_cost(&self, nurse_idx: usize) -> i32 {
        let Some(player) = self.player.as_ref() else {
            return 0;
        };
        let missing = player.max_hp - player.hp;
        let debuffs = player.remountable_debuff_count();
        let npc = &self.npcs[nurse_idx];
        let mood =
            crate::happiness::process_mood(&self.world, &self.npcs, npc, player.center(), false);
        let w = &self.world;
        nurse_heal_cost_for(
            missing,
            debuffs,
            mood.multiplier,
            &NurseCostFlags {
                downed_golem: w.downed_golem,
                downed_plant_boss: w.downed_plant_boss,
                downed_mech_any: w.downed_mech_boss1 || w.downed_mech_boss2 || w.downed_mech_boss3,
                hard_mode: w.hard_mode,
                downed_boss3_or_queen_bee: w.downed_skeletron || w.downed_queen_bee,
                downed_boss2: w.downed_eow,
                downed_boss1: w.downed_eoc,
                discount_available: self.discount_available,
            },
        )
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
        // The removable debuffs clear (`DoNurseHeal`'s removal block,
        // `Main.cs:40147`: `Main.debuff[b] && buffTime > 0 &&
        // !NurseCannotRemoveDebuff[b]`).
        player.clear_nurse_removable_debuffs();
        self.coins -= cost as u64;
        // `AchievementsHelper.HandleNurseService(healCost)` (`Main.cs:40111`,
        // `AchievementsHelper.cs:407-410`): the paid amount rides
        // FREQUENT_FLYER's `Pay` float, i.e. `BuyItem` succeeded.
        let _ = self.achievements.notify_custom_float("Pay", cost as f32);
        true
    }

    /// The `OnAchievementCompleted` subscribers (`AchievementManager.cs:39`,
    /// fired from `AchievementCompleted` `:180-187`), drained once per frame
    /// after the update branch:
    ///
    /// - `InGameNotificationsTracker.AddCompleted`
    ///   (`UI/InGameNotificationsTracker.cs:75-81`): the unlock popup, behind
    ///   the `Main.netMode != 2` gate — the port's `NetRole::Host` server
    ///   role (the `audio.rs` S1-E-24 convention).
    /// - `AchievementInitializer.OnAchievementCompleted`
    ///   (`Initializers/AchievementInitializer.cs:728-735`): the
    ///   `Achievements.Completed` chat line with the `[a:NAME]` tag
    ///   (`AchievementTagHandler.GenerateTag`) through `Main.NewText` (white,
    ///   `PlaySound(12)` — the port's `Menu_Tick` cue), then the tracked
    ///   `SoundID.AchievementComplete` when no instance is already playing
    ///   (`SoundEngine.FindActiveSound`'s guard).
    pub(crate) fn drain_achievement_completions(&mut self) {
        let completed = self.achievements.take_completed_events();
        if completed.is_empty() {
            return;
        }
        // `Main.netMode != 2` (`InGameNotificationsTracker.cs:77`).
        let not_server = !matches!(self.net, NetRole::Host(_));
        for name in completed {
            if not_server {
                let icon = achievements::AchievementManager::icon_index(name).unwrap_or(0);
                let title = crate::lang::text(&ui::achievements::name_key(name));
                self.notifications.add_completed(name, icon, title);
            }
            // `Main.NewText(Language.GetTextValue("Achievements.Completed",
            // AchievementTagHandler.GenerateTag(achievement)))` (`:730`).
            let tag = format!("[a:{name}]");
            let line = crate::lang::text_args("Achievements.Completed", &[tag]);
            self.chat.new_text(line, [255, 255, 255]);
            // `Main.NewText`'s `SoundEngine.PlaySound(12)` tail — W4-g23
            // centralized it in `ChatMonitor::new_text`; drain the queue
            // here so the cue still fires inside this subscriber (the
            // per-frame drain in `frame()` covers the other callers).
            self.chat.drain_new_text_cues(&self.audio);
            // `if (SoundEngine.FindActiveSound(SoundID.AchievementComplete)
            // == null) SoundEngine.PlayTrackedSound(...)` (`:731-734`).
            let style = crate::achievements::achievement_complete_style();
            if self.audio.find_active_tracked(&style).is_none() {
                self.audio.play_tracked_global(style);
            }
        }
    }

    /// The edit-to-map-queue hook: the edited tile's 3x3 ring queues for
    /// the map refresh — `WorldGen.TileFrame`'s head queues its tile
    /// (`WorldGen.cs:81334`) and `SquareTileFrame`'s walk (`:80185-80197`)
    /// is the port's `reframe_area` ring. The `Main.mapEnabled` gate rides
    /// (`MapUpdateQueue.cs:24`'s `!Main.mapEnabled` return).
    pub(crate) fn queue_map_ring(&mut self, x: i32, y: i32) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(m) = self.map.as_mut() else {
            return;
        };
        for dx in -1..=1 {
            for dy in -1..=1 {
                self.map_queue.add_tile(m, x + dx, y + dy);
            }
        }
    }

    /// The single-tile map enqueue — `MapUpdateQueue.Add(x, y)` at
    /// `MessageBuffer.cs:1153`'s head of the net-`5` tile-change arm (BEFORE
    /// the KillTile/PlaceTile dispatch). The `Main.dedServ` gate
    /// (`MapUpdateQueue.cs:23/43`) is structural: a dedicated server has no
    /// map — the port's listen-server host DOES keep one (`netMode == 2` but
    /// not `dedServ`), so its arm queues too; the
    /// `generatingWorld`/`!mapEnabled` gates ride the callers as elsewhere.
    pub(crate) fn queue_map_tile(&mut self, x: i32, y: i32) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(m) = self.map.as_mut() else {
            return;
        };
        self.map_queue.add_tile(m, x, y);
    }

    /// `MapUpdateQueue.Add(Rectangle)` behind the map-enabled gate — the
    /// `NetMessage.cs:2456` tail of `DecompressTileBlock` (the net-`10`
    /// section snapshot queues its whole rect).
    pub(crate) fn queue_map_area(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(m) = self.map.as_mut() else {
            return;
        };
        self.map_queue.add_area(m, x, y, w, h);
    }

    /// `WorldGen.RangeFrame`'s map half — the net-`20` tail
    /// (`MessageBuffer.cs:1438`): the (vanilla-inverted) area add plus the
    /// border-gated per-tile walk (`WorldGen.cs:80402-80411`).
    pub(crate) fn queue_map_range_frame(
        &mut self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(m) = self.map.as_mut() else {
            return;
        };
        self.map_queue.add_range_frame(m, start_x, start_y, end_x, end_y);
    }

    pub(crate) fn update_editing(&mut self, dt: f32, screen: Vec2) {
        world_sim::editing::update_editing(self, dt, screen);
    }

    /// The per-frame advance of the pointer clock and the state-change lockout:
    /// `UserInterface.Update`'s countdown, decremented *before* it is read
    /// (`UserInterface.cs:203-204`). `Main.UpdateUIStates`
    /// (`Main.cs:17039`/`:17487-17492`) calls `MenuUI.Update` on **every**
    /// frame, which is why the 200 ms post-`SetState` disable always expires:
    /// gating this on the live side being closed would latch it forever and
    /// silence every button of whichever front screen was open.
    ///
    /// Split out of `frame()` so the front-screen regime (`Main.menuMode ==
    /// 888`) is reachable from a headless test; `dt` is the same frame step
    /// `frame()` uses, pinned to 1/60 under `TR_SHOT`.
    pub(crate) fn tick_pointer_clock(&mut self, dt: f32) {
        self.pointer_ms = self.anim_time as f64 * 1000.0;
        self.pointer.tick(f64::from(dt) * 1000.0);
    }

    /// `Main.CanPauseGame` (`Main.cs:17315-17327`) — the in-game pause gate's
    /// condition. netMode is always 0 in the port (single-player), so the C#'s
    /// `if (netMode == 0)` wrapper is dropped and the body runs as written:
    ///
    /// - unconditional arms: the in-game options window (`ingameOptionsWindow`)
    ///   and the `InGameUI` overlay states — the achievements menu ports to
    ///   [`App::achievements_open`] (the controls-rebind half of the C#'s
    ///   `CurrentState == ManageControlsMenu` has no port UI — deviation). The
    ///   bestiary overlay is an `InGameUI` state but **not** one of the two
    ///   named ones — it never pauses unconditionally (the review-pass
    ///   correction; see the code comment);
    /// - the `autoPause` arm adds `playerInventory` ([`App::inventory_open`])
    ///   and the other visible in-game UIState (`InGameUI.IsVisible` ≈ the
    ///   achievements + bestiary overlays; `LocalPlayer.sign >= 0` has no sign
    ///   UI to gate — deviation, Phase-3 ANALYSIS).
    ///
    /// Nothing here draws or rolls RNG: the pause removes phases, adds none.
    pub(crate) fn can_pause_game(&self) -> bool {
        // `flag |= ingameOptionsWindow | (InGameUI.IsVisible && (…ManageControlsMenu || …AchievementsMenu))`
        // (`Main.cs:17320`). The bestiary is an `InGameUI` state too
        // (`IngameFancyUI.OpenUIState(Main.BestiaryUI)` →
        // `Main.InGameUI.SetState`, `Main.cs:42459`/`IngameFancyUI.cs:78`), but
        // its `CurrentState` is neither of the two named ones — it is caught
        // only by the autoPause arm's bare `InGameUI.IsVisible` below, so the
        // bestiary open never pauses by itself.
        let mut flag = self.ingame_options.open || self.achievements_open;
        // `if (autoPause) flag |= playerInventory | (LocalPlayer.sign >= 0) | InGameUI.IsVisible`
        // (`Main.cs:17322-17324`) — the bare `InGameUI.IsVisible` half is the
        // achievements + bestiary overlays; the inventory rides its own arm.
        if self.settings.auto_pause {
            flag |= self.inventory_open || self.achievements_open || self.bestiary_open;
        }
        flag
    }

    /// `Main.ShouldUpdateEntities` (`Main.cs:17717-17726`): the world-preparation
    /// half of the `DoUpdateInWorld` gate (`Main.cs:17278`; the stepper half is
    /// Phase 2). The port installs worlds synchronously at frame boundaries —
    /// [`App::new`], [`App::swap_world`] and `client_install_world` swap the
    /// world in whole — so `_worldPreparationState == Ready` and
    /// `!WorldGen.generatingWorld` hold on every frame the world branch runs
    /// (the C#'s async load states are not ported). The named seam keeps the
    /// `Main.cs:17278` call site 1:1 and gives Phase 2 its gate.
    pub(crate) fn should_update_entities(&self) -> bool {
        true
    }

    /// `Main.DoUpdate_WhilePaused` (`Main.cs:17329-17407`) — what keeps living
    /// while the in-game gate holds. The C# body, arm by arm, and where each
    /// lives in the port (every portable arm is already event- or draw-driven,
    /// so this shell carries no executable state of its own yet):
    ///
    /// - `WorldGen.generatingWorld` early return (`:17332`): the port has no
    ///   generating state (sync loads, see [`App::should_update_entities`]).
    /// - the `controlInv`/`controlCreativeMenu` edge latches → `ToggleInv`/
    ///   `ToggleCreativeMenu` (`:17335-17362`): the port's toggle fires on the
    ///   key event itself (`loop.rs::world_key`, the `cInv`/Escape route) and
    ///   nothing gates it on the pause — the C# UX (items movable while the
    ///   world is frozen) is already the port's. No creative menu exists
    ///   (deviation).
    /// - `playerInventory` → `RefreshInfoAccs`/`DoScrollingInInventory`/
    ///   `dropItemCheck` (`:17365-17369`): no info-accessory pass (deviation);
    ///   the wheel scroll routes through the pointer machine's wheel event
    ///   (S1-H-4, event-driven); the held-item drop rides the click cluster's
    ///   trash/transfer path (event-driven).
    /// - `head/body/legs` from `GetEffectiveArmor` (`:17370-17385`): draw-driven
    ///   here — `Player::update_frames`/`render_key` recompute the display rows
    ///   from the equip slots at draw time.
    /// - `editSign`/`editChest` arms (`:17386-17397`): the port has no sign
    ///   editor and the chest-rename editor is a keyboard session (event
    ///   path) — no per-frame clear to port (deviation).
    /// - `Player.tileTargetX/Y` + `LookForTileInteractions`/`ChestChangeEvents`/
    ///   `UpdateNearbyCraftingTiles` (`:17398-17406`): the port fuses the tile
    ///   target + interactions into `update_editing` (step-gated, so frozen
    ///   while paused) and the crafting adjacency into the HUD draw's
    ///   `craft::scan_adjacent` (`app/frame.rs`, draw time — live). The
    ///   frozen-target half is a recorded deviation (Phase-3 ANALYSIS).
    pub(crate) fn do_update_while_paused(&mut self) {}

    /// The "the world is live now" state that both world-install paths end on:
    /// the front end closes and the legacy menu hands its settings hint back.
    /// `Main.LoadWorld` and the net install both leave `Main.gameMenu` false
    /// (`client_install_world`'s tail is the twin) — exactly one of `front`/`menu`
    /// is ever live (`Review Focus 1`), and in-game neither is.
    pub(crate) fn enter_loaded_world(&mut self) {
        if self.menu.is_some() {
            self.close_menu();
        }
        self.front = None;
        self.front_backup = None;
        self.front_hint = None;
        self.front_hover_hit = None;
        // `Main.gamePaused = false` in `LoadWorld`'s tail (`Main.cs:19856`): a
        // world never enters paused. (`GameUpdateCount` resets through the
        // `OnWorldLoad` hook — the fresh [`App::new`] both install paths build.)
        self.game_paused = false;
        // W4-g66 gap-fill — the session character's appearance lands on the
        // live player (`Main.ActivePlayerFileData.Player`'s colours; the
        // wings-33/38 dust arms read them).
        self.apply_session_appearance();
        // S1-G-1f: the world-load animator reset (`Main.cs:8393-8394` —
        // `tileFrame[l] = 0; tileFrameCounter[l] = 0`, and
        // `weatherVaneBobframe = 0` at `:67242`).
        self.tile_anim.reset();
        self.tile_anim_step = FixedStepper::new(1.0 / 60.0, 8);
        // `Main.mapDelay = 2` (`WorldGen.cs:6947`, the world-load reset):
        // the legacy engine owns the counter in the port (S1-E-19 wave B,
        // `LegacyLighting.cs:345-348`).
        if let Some(engine) = self.legacy_light.as_mut() {
            engine.reset_map_delay();
        }
        // W4-f30 — `Player.Hooks.EnterWorld`'s forced research check
        // (`Player.cs:325` → `AchievementsHelper.CheckResearchAchievement(
        // forced: true)`): the world-entry re-count fires 45 (TRASH_COMPACTOR)
        // if half the researchable catalogue is already complete. The tracker
        // loads empty until the Journey sacrifice UI lands (M6-SAVE's world
        // section), so this is inert in play but the call site is wired.
        let (fired, version) =
            crate::research::check_research_achievement(&self.research, self.last_research_version, true);
        self.last_research_version = version;
        if fired {
            let _ = self.notify_progression_net(45);
        }
        // W4-f40 DOC-SWEEP: the lantern world-load clear (`WorldGen.cs:6996`
        // `LanternNight.WorldClear()` + the `SkyManager.Reset` family) lives
        // at the clear-then-restore sites — `swap_world` (ahead of the
        // `App::new` restore) and `client_install_world` (ahead of the tile
        // install) — NOT here: this tail runs AFTER the restore, and a
        // clear here would clobber a loaded lantern night (the C#
        // `clearWorld` precedes `SetOngoingToTemps`, `WorldFile.cs:1977`
        // vs `:173-176`).
    }

    pub(crate) fn is_client(&self) -> bool {
        matches!(self.net, NetRole::Client(_))
    }

    /// `Main.DrawPVPIcons`'s state block (`Main.cs:42654-42676`): the two gates
    /// (`ShouldPVPDraw` = `netMode == 1`, `ShouldTeamSelectDraw`), the local
    /// player's `hostile`/`team` and `Main.teamCooldown`.
    ///
    /// `Main.netMode` is the port's [`App::is_client`]; `teamBasedSpawnsSeed`
    /// (`Main.cs:351`) is not ported (no team-based spawn seeds until the
    /// M5-seed rows), so the single-player arm reads `false` — with it off the
    /// vanilla hides the team cells outside a multiplayer client anyway.
    pub(crate) fn pvp_icon_state(&self) -> crate::ui::chrome::PvpIconState {
        // `TR_PVP=1` opens the vanilla netsplit gates for screenshots (see
        // [`crate::env::pvp_icons`]); the game rule stays `Main.netMode == 1`.
        let pvp_draw = self.is_client() || self.pvp_icons_forced;
        let (hostile, team) = self
            .player
            .as_ref()
            .map(|p| (p.hostile, p.team))
            .unwrap_or((false, 0));
        crate::ui::chrome::PvpIconState {
            pvp_draw,
            // `Main.cs:2911-2919`: `ShouldTeamSelectDraw` — always inside the
            // PVP gate, else the team-based-spawns seed (not ported: false).
            team_select_draw: pvp_draw,
            hide: self.hide_pvp_and_team_icons,
            hostile,
            team,
            team_cooldown: self.team_cooldown,
        }
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

    /// `Main.ActivePlayerFileData.Player.difficulty`'s port stand-in
    /// (`UIWorldSelect.cs:179` / `UIWorldCreation.cs:319`): the session
    /// character's difficulty. The port loads nothing per-entry on the
    /// character select (`CharPlay` is routing-only, spec §9.4), so the
    /// seeded session draft (`session_entry`'s source) is the active player —
    /// 0 (`Player.cs:1654`) while no character has been created.
    pub(crate) fn active_player_difficulty(&self) -> u8 {
        self.session_draft.as_ref().map_or(0, |d| d.difficulty)
    }

    /// Re-sorts `App.worlds` through the active player's Journey flag — the
    /// first key of `UpdateWorldsList` (`UIWorldSelect.cs:156`,
    /// `CanWorldBeJoinedByActivePlayer` `:177-180`). The lone sort site so the
    /// list hop and the favourite toggle cannot disagree (`enter_front`'s
    /// re-list, `flow.rs`'s `UpdateOrder`).
    pub(crate) fn resort_worlds(&mut self) {
        let active_is_creative = self.active_player_difficulty() == 3;
        ui::world_select::sort_worlds(&mut self.worlds, active_is_creative);
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
                let fakes = crate::env::fake_count("TR_FAKE_CHARS");
                // The listing root is `Main.PlayerPath` (`Main.cs:67065-67075`)
                // — the port's `Terrustaria/Players` (`save.rs`'s mirror).
                let dir = save::default_players_dir();
                self.players =
                    ui::save_list::list_players(dir.as_deref(), Some(self.session_entry()), fakes);
                ui::char_select::sort_players(&mut self.players);
            }
            FrontScreen::WorldSelect => {
                let fakes = crate::env::fake_count("TR_FAKE_WORLDS");
                // The listing root is the port's `Documents/My Games/
                // Terrustaria/Worlds` (`Main.WorldPath`'s directory,
                // `Main.cs:67065-67075`; `save.rs:20-29`).
                let dir = save::default_world_dir().unwrap_or_default();
                self.worlds = ui::save_list::list_worlds(&dir, fakes);
                // `UpdateWorldsList` (`UIWorldSelect.cs:153-166`): the re-list
                // sorts by the active player's Journey flag first (`:177-180`).
                self.resort_worlds();
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
            // `UIWorldLoad.OnActivate` (`UIWorldLoad.cs:37-44`) is the
            // gamepad snap point only (out of scope, §J.3) — no re-list.
            // The generation job is started by `finish_creating_world`
            // before the hop (`WorldGen.CreateNewWorld`, `:6607-6625`).
            FrontScreen::WorldLoad(_) => {}
            // `UIResourcePackInfoMenu` has no `OnActivate` override — the
            // ctor's `BuildPage` (`:33-38`) is the whole setup; the pack and
            // the back target rode in from `Click_Info`.
            FrontScreen::ResourcePackInfo(_) => {}
            // `Main.OpenResourcePacksMenu` (`Main.cs:52371-52375`): the ctor
            // takes `AssetInitializer.CreateResourcePackList` — the config's
            // saved entries + the directory discovery (`ResourcePackList.
            // FromJson`) — then `PopulatePackList` (`:57`). The back target
            // rode in from the opener (null for the menu row, the hub for
            // `Click_OpenResourcePacksMenu`).
            FrontScreen::ResourcePackSelection(ref mut state) => {
                let dir = crate::resource_pack::resource_packs_dir();
                state.packs = match dir {
                    Some(dir) => crate::resource_pack::ResourcePackList::from_config(
                        &self.settings.resource_packs,
                        &dir,
                    ),
                    None => crate::resource_pack::ResourcePackList::default(),
                };
                state.populate();
                state.available_scroll = 0.0;
                state.enabled_scroll = 0.0;
            }
            // `UIWorkshopSelectResourcePackToPublish.OnActivate` (`:128-135`)
            // → `PopulateEntries` (`:137-151`): the publishable list (no
            // saved state), Branding-None filter + IsCompressed sort. The
            // back target rode in from the hub's opener.
            FrontScreen::WorkshopPublishPacks(ref mut state) => {
                let dir = crate::resource_pack::resource_packs_dir();
                state.packs = match dir {
                    Some(dir) => crate::resource_pack::ResourcePackList::publishable(&dir),
                    None => crate::resource_pack::ResourcePackList::default(),
                };
                state.populate();
                state.scroll = 0.0;
            }
            // `UIWorkshopSelectWorldToPublish.OnActivate` (`:125-132`) →
            // `PopulateEntries` (`:134-149`): `Main.LoadWorlds`, drop
            // `!IsValid`, the favourite/name/file sort — NOT the world
            // select's Journey-first sort. The back target rode in from the
            // hub's opener.
            FrontScreen::WorkshopPublishWorlds(ref mut state) => {
                let fakes = crate::env::fake_count("TR_FAKE_WORLDS");
                let dir = save::default_world_dir().unwrap_or_default();
                let mut worlds = ui::save_list::list_worlds(&dir, fakes);
                worlds.retain(|w| w.valid);
                ui::workshop_publish::sort_publish_worlds(&mut worlds);
                state.worlds = worlds;
                state.scroll = 0.0;
            }
            // `UIWorkshopWorldImport.OnActivate` (`:131-140`):
            // `UpdateWorkshopWorldList` (`:142-153`) returns early without
            // `SocialAPI.Workshop`, so `UpdateWorldsList` (`:155-166`)
            // sorts an empty list — the screen is always empty live. The
            // back target rode in from the hub's opener.
            FrontScreen::WorkshopImportWorlds(ref mut state) => {
                state.worlds.clear();
                ui::workshop_publish::sort_publish_worlds(&mut state.worlds);
                state.scroll = 0.0;
            }
            // `UIWorkshopHub` has no `OnActivate` override — `EnterHub`
            // (`:40-43`) fires the empty `OnWorkshopHubMenuOpened` event
            // (`:368-373`); no re-list. The back target rode in from the
            // opener (null from the menu row).
            FrontScreen::WorkshopHub(_) => {}
        }
        self.park_menu();
        self.front = Some(front);
        // A fresh `UIState` builds a fresh `UIList`, whose scrollbar starts at
        // `_viewPosition = 0` (`UIScrollbar.cs:17`) — re-entering a screen is
        // not a continuation of the last visit's scroll.
        self.front_scroll = 0.0;
        self.front_scroll_drag = None;
        // `UserInterface.SetState`'s `ResetState` (`UserInterface.cs:295-299`):
        // a button held across the switch must not read as a fresh press edge
        // on the new screen, and the next press is locked out for 200 ms.
        self.pointer
            .reset_on_state_change([self.front_mouse_down, self.place_down]);
        self.pointer_ms = self.anim_time as f64 * 1000.0;
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
            // `OnFinishedSettingName` (`UIWorkshopImportWorldListItem.cs:
            // 107-114`): `SocialAPI.Workshop` is null, so the import never
            // runs and no state change happens — the keyboard stays open,
            // exactly like the source. The notice is the port's channel.
            KeyboardTail::WorldImport => {
                self.front_hint = Some(format!(
                    "Importing \"{name}\" is not ported yet (no Steam Workshop)"
                ));
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

    /// The front end's Play tail (`UIWorldListItem.PlayGame` → `Main.LoadWorld`,
    /// BUG-04): the selected `.wld`/`.trwld` is loaded and the whole
    /// world-derived state re-derives from it through [`App::new`]'s own body —
    /// the same construction the boot path runs, so a swapped world is
    /// indistinguishable from a booted one. The window/GPU/audio/music and the
    /// shot-capture state survive the swap (the vanilla's `LoadWorld` walks the
    /// same split: the device and the config stay, the world state reloads).
    ///
    /// The atlas is rebuilt from the new world's tile/wall usage
    /// ([`build_atlas`]) and the renderer is recreated from it — the old one
    /// holds the old atlas texture (the same recreation the boot's `loop` runs).
    /// A failed load leaves the current world untouched and returns the error,
    /// so the menu stays alive with its own world rather than a black screen.
    pub(crate) fn swap_world(&mut self, path: &Path) -> anyhow::Result<()> {
        let (world, sections) = load_world(path)?;
        // W4-f40 DOC-SWEEP: the C# `LoadWorld` runs `WorldGen.clearWorld()`
        // (`WorldFile.cs:1977`) BEFORE `LoadWorldFlags` reads the event temps
        // back (`SetOngoingToTemps`, `:173-176`) — the clear-then-restore
        // order. The port's restore rides `App::new` below (f45's from_wld
        // block), so the clear runs here, ahead of it; a clear in
        // `enter_loaded_world` (AFTER the rebuild) would clobber the
        // restored lantern quad. The fresh `App::new` builds
        // `WorldEvents`/`LanternSky` from defaults anyway — this pins the
        // order for any future restore-onto-self path. The failed-load
        // early return above leaves the current world untouched.
        crate::world_events::world_clear(&mut self.world_events);
        self.lantern_sky.reset();
        let content = assets::content_dir().context("no retail Content directory")?;
        let mut static_cache = self.static_sheets.take();
        let mut fonts_cache = self.fonts_cache.take();
        let built = build_atlas_reusing_static(
            &world,
            &content.join("Images"),
            &mut static_cache,
            &mut fonts_cache,
        );
        // Whatever happened, the decoded halves are carried onto the App (they
        // are world-independent, so a failed swap must not throw them away — the
        // next Play would re-decode for nothing).
        let rebuilt = match built {
            Ok(parts) => parts,
            Err(e) => {
                self.static_sheets = static_cache;
                self.fonts_cache = fonts_cache;
                return Err(e);
            }
        };
        let (atlas, new_font_atlas, lookups, atlas_stats) = rebuilt;
        // A cached font build returns no page texture (nothing was decoded), so
        // the one already uploaded rides the swap — the same keep rule as the
        // window/GPU below. A fresh build replaces it.
        let font_atlas = new_font_atlas.or_else(|| self.font_atlas.take());
        let settings = self.settings.clone();
        let shot_path = self.shot_path.clone();
        // W4-g66 gap-fill — the session draft survives the world swap
        // (`Main.PendingPlayer`/`ActivePlayerFileData` are world-independent;
        // `enter_loaded_world` re-applies its appearance to the fresh player).
        let session_draft = self.session_draft.take();
        let audio = std::mem::replace(&mut self.audio, audio::Audio::new(assets::content_dir()));
        let music = std::mem::replace(
            &mut self.music,
            audio::MusicSystem::new(assets::content_dir()),
        );
        let window = self.window.take();
        let gpu = self.gpu.take();
        let post = self.post.take();
        let noise_tex = self.noise_tex.take();
        let water_mask = self.water_mask.take();
        // The fresh App carries the new world's state (clouds, sky, stars,
        // player, npcs, town rooms, chests, liquids, rain, wind, atmo,
        // bestiary, the loader pins) exactly as App::new built it at boot.
        *self = App::new(
            settings,
            world,
            sections,
            path.to_path_buf(),
            atlas,
            font_atlas,
            lookups,
            atlas_stats,
        );
        // The fresh manager loaded its conditions in `App::new`; re-bind its
        // `_savePath` (`AchievementManager.cs:41-55`) exactly as boot did.
        self.bind_achievements_save_path();
        self.window = window;
        self.gpu = gpu;
        self.audio = audio;
        self.music = music;
        self.post = post;
        self.noise_tex = noise_tex;
        self.water_mask = water_mask;
        self.shot_path = shot_path;
        self.session_draft = session_draft;
        // `App::new` builds fresh `None` caches; the decoded halves ride the swap
        // so the next Play is fast too (both are world-independent).
        self.static_sheets = static_cache;
        self.fonts_cache = fonts_cache;
        // `UIWorldListItem.PlayGame` -> `Main.LoadWorld` leaves the game menu
        // (`Main.gameMenu = false`): the fresh `App::new` boots the legacy menu
        // (the boot default), so the launch state is restored here — the same
        // tail the net client install does (`client_install_world`). Without it
        // the select-world Play loaded the world and then drew the lobby over
        // it instead of entering the world (BUG-21).
        self.enter_loaded_world();
        // `loop`'s boot block (`loop.rs:88-100`): the camera lands on the
        // world's spawn and the zoom on the setting, before the first frame.
        self.camera = Vec2::new(
            self.world.width as f32 * TILE_PX * 0.5,
            self.world.spawn_y as f32 * TILE_PX,
        );
        if let Some((x, y)) = crate::env::camera_pin() {
            self.camera = Vec2::new(x * TILE_PX, y * TILE_PX);
        }
        self.zoom = self.settings.zoom.clamp(1.0, 2.0);
        if let Some(z) = crate::env::zoom_pin() {
            // Main.cs:16801-16815 — vanilla GameZoomTarget clamps to 1.0..2.0;
            // TR_ZOOM must not widen it (S1-B-5).
            self.zoom = z.clamp(1.0, 2.0);
            self.zoom_pinned = true;
        }
        // The renderer rides the new atlas texture (the dye-param table is
        // static vanilla data, re-uploaded like the boot's).
        if let Some(gpu) = self.gpu.as_ref() {
            if let Some(atlas) = self.atlas.as_ref() {
                let mut renderer = SpriteRenderer::new(gpu, atlas, 8, self.font_atlas.as_ref());
                renderer.set_dye_params(
                    gpu,
                    &crate::dye::dye_params()
                        .iter()
                        .map(|p| tr_render::sprite::DyeParam {
                            mode: p.mode,
                            _pad: p._pad,
                            color: p.color,
                            secondary: p.secondary,
                        })
                        .collect::<Vec<_>>(),
                );
                self.renderer = Some(renderer);
            }
        }
        Ok(())
    }
}

impl App {
    /// Assembles the `App` from the loaded world and the built atlas — the
    /// entity/session half of the startup: the render assets, the starting
    /// player and slimes, the inventory and the persisted session, the debug
    /// env hooks, the menu/network/map setup and the `TR_UI` preset.
    ///
    /// `settings` is the master copy, read once by `main` (`Settings::load`,
    /// the read-once rule); the menu receives a clone so its edits park here on
    /// close (`close_menu`/`park_menu`).
    ///
    /// `chests` is the world save's v7 chest record list (`load_world` →
    /// `save::load`), M6-SAVE-1b's load-side twin of [`App::save_world`]: each
    /// record re-stocks a store entry at its anchor, winning over the load-time
    /// loot re-roll (the vanilla stocks the `Chest` objects inline with the
    /// placement, so the re-roll only fills the tiles a save never came from).
    pub(crate) fn new(
        settings: settings::Settings,
        world: World,
        sections: WorldSections,
        world_path: PathBuf,
        atlas: Atlas,
        font_atlas: Option<tr_render::FontAtlas>,
        lookups: assets::Lookups,
        atlas_stats: (u32, u32, usize, usize),
    ) -> App {
        let mut world = world;
        let clouds = make_clouds(&world, &lookups);
        let sky = make_sky_assets(&atlas, &world);
        let stars = make_stars(&world);
        let player_assets = make_player_assets(&atlas);
        let npc_assets = make_npc_assets(&atlas);
        let player_enabled = !crate::env::no_player();
        let mut player = player_enabled.then(|| Player::spawn(&world));
        let mut npcs = Vec::new();
        // The town rooms (`WorldGen.TownManager`, world-file section 9): built
        // from the loaded section so the home re-validation below can consult
        // them, and carried onto the App so the save writes them back. An empty
        // store for the generated worlds — `M4-TOWN-1`'s rooms wait for their
        // own follow-up, the `.wld` store is this phase's.
        let mut town_rooms = crate::town::TownRooms::from_wld_rooms(&sections.data.town_rooms);
        // `TR_POSE` keeps the capture scene empty — a contact-damage NPC
        // would knock the pinned player off the furniture.
        let spawn_npcs = !crate::env::pose_set();
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
        let follow_player = player.is_some() && !crate::env::cam_set() && !crate::env::pan_set();
        let mut coins = 0u64;
        // The bestiary kill pairs, restored from the sidecar below.
        let mut loaded_kills: Vec<(String, i32)> = Vec::new();
        // The achievement condition states, restored from the port's own
        // `achievements.trdat` (the Rijndael+BSON vanilla format is the
        // documented M6-ACH-1 deviation).
        let mut loaded_achievements = achievements::AchievementManager::registry();
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
                    // W4-g89 — the permanent Aegis Fruit unlock rides the
                    // sidecar's v7 line (`Player.usedAegisFruit` load,
                    // `Player.cs:55256`); the booster pass re-adds the +4
                    // every tick (`:12853-12856`). The Gummy Worm rides the
                    // same line's second token (v8, `:55259`; the +3 row
                    // `:12857-12860`).
                    p.used_aegis_fruit = saved.used_aegis_fruit;
                    p.used_gummy_worm = saved.used_gummy_worm;
                    // W4-g89c — the Ambrosia rides the third token (v9,
                    // `:55260`; the speed row `:12861-12866`).
                    p.used_ambrosia = saved.used_ambrosia;
                    // W4-g89i — the Aegis Crystal rides the fourth token
                    // (v10, `:55255`; the +0.2 regen row `:19113-19116`).
                    p.used_aegis_crystal = saved.used_aegis_crystal;
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
            // W4-g56 — the void-vault favorites restore from the `.trvault`
            // sidecar (the `.plr` bank4-favourited rows are f44's lane; the
            // contract is in ANALYSIS). A fresh world (no sidecar) keeps the
            // all-false default.
            if let Some(favs) = persist::load_vault_favorites(&world_path) {
                if let Some(p) = player.as_mut() {
                    p.vault_favorites = favs;
                }
            }
        }
        if let Some(n) = crate::env::coins() {
            coins = n;
        }
        // `TR_DASH=<1|2|3|6>` pins `Player.dashType` through the `pinned_dash`
        // stand-in (the accessory granters - Tabi/EoC Shield/Master Ninja
        // Gear - arm it for real now; the pin survives the per-tick clear).
        if let Some(d) = crate::env::dash_type() {
            if let Some(p) = player.as_mut() {
                p.pinned_dash = d.clamp(0, 6);
            }
        }
        // `TR_MOUNT=<id|name>` pins a mount (the granting saddle items are
        // real items now — this stands in for the `miscEquips[3]` slot and
        // for tests/screenshots).
        if let Some(id) = crate::env::mount_pin() {
            if let Some(p) = player.as_mut() {
                p.pinned_mount = Some(id);
            }
        }
        // `TR_POSE=sit|sleep` pins the furniture pose for screenshots/tests:
        // the furniture tiles are written beside the spawn and the player is
        // snapped on through the real decode — the sim's own re-validation
        // then holds the pose (no input reaches the headless capture). The
        // tile writes run without a player too (`TR_NOPLAYER` + `TR_POSE`
        // verifies the furniture draw itself).
        if let Some(v) = crate::env::pose_mode() {
            apply_pose_pin(&mut world, player.as_mut(), v.trim());
        }
        // `TR_JUMPS=cloud,sandstorm,...` arms the extra-jump options — the
        // granting accessories (CloudinaBottle 53, SandstorminaBottle 857,
        // BlizzardinaBottle 987, FartinaJar 1724, TsunamiInABottle 3201) and
        // the mounts land with M3-EQUIP-3 / M2-MOUNT-1. `balloon` also lifts
        // the jump statics (`jumpBoost`, `Player.cs:19589-19593`).
        if let Some(list) = crate::env::jumps_list() {
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
        if let Some(w) = crate::env::wings() {
            if let Some(p) = player.as_mut() {
                p.wings_logic = w.clamp(0, 49);
                let (fly, _, _, _, _, _) = crate::player::Player::wing_stats(w);
                p.wing_time_max = fly;
                p.has_wings = w > 0;
            }
        }
        if let Some(b) = crate::env::rocket_boots() {
            if let Some(p) = player.as_mut() {
                p.pinned_rocket_boots = b.clamp(0, 5);
                p.rocket_boots = b.clamp(0, 5);
            }
        }
        // The town roster's load (`WorldFile.cs:2884-2954`): a `.trwld`'s v10
        // records restore the block-B town list — the kind, the given name,
        // the position, the homelessness and the home tile. A world whose save
        // carries none (a freshly generated world, or a pre-v10 save) keeps
        // the vanilla-gen arrival: one homeless Guide at the spawn point,
        // which the look-for-home tick re-houses within seconds. A `.wld`'s
        // roster replaces these spawns outright below (`Main.npc[num2++]`).
        if sections.from_wld {
            // The file's roster wins; the Guide spawn is skipped (below).
        } else if sections.native.npcs.is_empty() {
            if let Some(mut guide) = Npc::spawn_guide(&world, world.spawn_x) {
                // `NPC.NewNPC` → `GiveTownUniqueDataToNPCsThatNeedIt`
                // (`NPC.cs:80769`/`:80839-80856`): the starter Guide names
                // himself at spawn — two `genRand` draws through the legacy
                // profile (`npc_names.rs`). The C# `Main.rand` is the
                // worldgen stream here; the legacy profile rolls variation
                // 0 and never draws it, so the seed stand-in is inert.
                let mut main_rand = tr_core::unified_random::UnifiedRandom::new(world.seed as i32);
                crate::npc_names::give_town_unique_data(
                    &mut guide,
                    &mut main_rand,
                    &mut world.frame_rng,
                );
                npcs.push(guide);
            }
        } else {
            for rec in &sections.native.npcs {
                // An unknown kind is skipped, ' ownership read.
                let Some(kind) = NpcKind::from_u16(rec.kind) else {
                    continue;
                };
                let Some(mut npc) = Npc::spawn_town(&world, (rec.x / TILE_PX) as i32, kind) else {
                    continue;
                };
                npc.x = rec.x;
                npc.y = rec.y;
                npc.home = if rec.homeless {
                    None
                } else {
                    Some((rec.home_x, rec.home_y))
                };
                npc.given_name = rec.given_name.clone();
                npcs.push(npc);
            }
        }
        // `LoadNPCs` (`LoadWorld_Version2:1791`): a `.wld`'s town list IS the
        // world's NPCs, so the file replaces the startup spawns outright
        // (`Main.npc[num2++] = ...`, the C# shape — the spawn loop runs during
        // play, not at load). The town rooms ride along because
        // `FixAgainstExploits`' home re-validation consults them.
        if sections.from_wld {
            npcs = crate::npc::from_wld(&sections.data.npcs);
            // `LoadWorld_LastMinuteFixes` → `FixAgainstExploits:1861`: the
            // home re-validation runs after every section is read (the town
            // rooms among them), not during `LoadNPCs`.
            crate::town::fix_town_npc_homes(&world, &mut npcs, &mut town_rooms);
        }
        // Debug spawn override: place the player at a tile coordinate (the camera follows).
        if let Some((tx, ty)) = crate::env::spawn_xy() {
            if let Some(p) = player.as_mut() {
                p.teleport(
                    tx as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5,
                    ty as f32 * TILE_PX,
                );
            }
        }
        if crate::env::boss_set() {
            if let Some(p) = &player {
                // `TR_BOSS=golem` places the Golem body for the family's
                // deterministic shots (the parts ride the body's one-shot
                // init); `TR_BOSS=deerclops` places the Deerclops beside
                // the player on the ground; any other value (or a bare
                // flag) keeps the King Slime. The Golem body takes the
                // `SetDefaults` alpha fade.
                let boss = crate::env::boss_mode();
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
                    // R22 G3: the ctor carries `SetDefaults`' alpha = 255
                    // (`NPC.cs:12175`) — the fade rides the machine's
                    // `alpha -= 10` (`:19535-19543`).
                    npcs.push(g);
                } else if let Some(state) = boss.strip_prefix("deerclops") {
                    // The Deerclops' box is 60x154 (`SetDefaults` 668,
                    // `NPC.cs:17451-17452`) — the shot hook stands it on
                    // the ground beside the player; the optional
                    // `deerclops,<ai0>` suffix pins an attack state so the
                    // spells' sequences are capturable (the fight's
                    // walk sends the boss off the terrain otherwise).
                    let (bw, bh) = NpcKind::Deerclops.size();
                    let mut d = Npc::new(
                        NpcKind::Deerclops,
                        p.x + 8.0 * TILE_PX,
                        p.y + p.height - bh,
                        1,
                        0.0,
                    );
                    d.hp = d.max_hp;
                    if let Ok(s) = state.trim_start_matches(',').parse::<i32>() {
                        d.ai[0] = s as f32;
                        d.ai[1] = 12.0;
                    }
                    npcs.push(d);
                } else if boss == "prime" {
                    // Skeletron Prime (`TR_BOSS=prime`, M4-NPC-3j): the head
                    // above the player; its machine's one-shot
                    // `wants_limbs` init spawns the four limbs on the next
                    // tick, so the shot hook captures the whole boss.
                    let mut b = Npc::new(
                        NpcKind::SkeletronPrime,
                        p.x + 12.0 * TILE_PX,
                        p.y - 14.0 * TILE_PX,
                        1,
                        0.0,
                    );
                    b.hp = b.max_hp;
                    npcs.push(b);
                } else if let Some(n) = Npc::spawn_king_slime(&world, (p.x / TILE_PX) as i32 + 12) {
                    npcs.push(n);
                }
            }
        }
        if let Some(v) = crate::env::slime_at() {
            for x in v.split(',').filter_map(|s| s.trim().parse::<i32>().ok()) {
                if let Some(n) = Npc::spawn_slime(&world, x) {
                    npcs.push(n);
                }
            }
        }
        // `TR_NPC=<kind>[,<kind>…]`: the humanoid-walker verification hook —
        // the kinds stand beside the player so the draw's rows ride each
        // sheet's `Main.npcFrameCount` (the town 21-26, the goblins 16/21,
        // the cave walkers 15, the zombie's single frame).
        if let Some(v) = crate::env::npc_names() {
            if let Some(p) = &player {
                // The lineup spreads one kind every three tiles to the right
                // so a single screenshot shows each sprite on its own — the
                // v1 piled every kind on the same spot and the audit could
                // only see a jumble.
                for (slot, name) in v.split(',').map(|s| s.trim()).enumerate() {
                    let kind = match name {
                        "guide" => NpcKind::Guide,
                        // The Eye of Cthulhu's six-row strip (BUG-22's
                        // shadowed arm) and the Travelling Merchant's shared
                        // walk band.
                        "eoc" => NpcKind::EyeOfCthulhu,
                        "travellingmerchant" => NpcKind::TravellingMerchant,
                        "zombie" => NpcKind::Zombie,
                        "skeleton" => NpcKind::Skeleton,
                        "peon" => NpcKind::GoblinPeon,
                        "thief" => NpcKind::GoblinThief,
                        "warrior" => NpcKind::GoblinWarrior,
                        "archer" => NpcKind::GoblinArcher,
                        "tinkerer" => NpcKind::GoblinTinkerer,
                        "merchant" => NpcKind::Merchant,
                        "goldfish" => NpcKind::Goldfish,
                        "piranha" => NpcKind::Piranha,
                        "shark" => NpcKind::Shark,
                        "cavebat" => NpcKind::CaveBat,
                        "junglebat" => NpcKind::JungleBat,
                        "hellbat" => NpcKind::Hellbat,
                        "voodoodemon" => NpcKind::VoodooDemon,
                        "sorcerer" => NpcKind::GoblinSorcerer,
                        "tim" => NpcKind::Tim,
                        "darkcaster" => NpcKind::DarkCaster,
                        "fireimp" => NpcKind::FireImp,
                        // The martians' lineup (M4-NPC-3i): the ground
                        // group's per-kind sheets, the probe/drone, the
                        // Scutlix pair and the saucer family's four strips.
                        "officer" => NpcKind::MartianOfficer,
                        "grunt" => NpcKind::GrayGrunt,
                        "engineer" => NpcKind::MartianEngineer,
                        "zapper" => NpcKind::GigaZapper,
                        "scrambler" => NpcKind::BrainScrambler,
                        "raygunner" => NpcKind::RayGunner,
                        "walker" => NpcKind::MartianWalker,
                        "probe" => NpcKind::MartianProbe,
                        "drone" => NpcKind::MartianDrone,
                        "rider" => NpcKind::ScutlixRider,
                        "scutlix" => NpcKind::Scutlix,
                        "saucer" => NpcKind::MartianSaucer,
                        "turret" => NpcKind::MartianSaucerTurret,
                        "cannon" => NpcKind::MartianSaucerCannon,
                        "saucercore" => NpcKind::MartianSaucerCore,
                        // The Prime family (M4-NPC-3j): the head plus its
                        // four limbs (the martian "cannon" already holds
                        // the saucer mount, so the Prime's takes the
                        // `primecannon` spelling).
                        "prime" => NpcKind::SkeletronPrime,
                        "primecannon" => NpcKind::PrimeCannon,
                        "primesaw" => NpcKind::PrimeSaw,
                        "primevice" => NpcKind::PrimeVice,
                        "primelaser" => NpcKind::PrimeLaser,
                        // The Twins (M4-NPC-3k): the two eyes ride their
                        // hover machines up beside the player once spawned.
                        "retinazer" => NpcKind::Retinazer,
                        "spazmatism" => NpcKind::Spazmatism,
                        // The Lunar pillars (M4-NPC-3l): the four towers
                        // spawn on the ground beneath the player (their
                        // machines hover them up over the first ticks).
                        "stardust" => NpcKind::StardustTower,
                        "nebula" => NpcKind::NebulaTower,
                        "solar" => NpcKind::SolarTower,
                        "vortex" => NpcKind::VortexTower,
                        "cultist" => NpcKind::AncientCultist,
                        // The pillar families' own-sheet audit (M4-NPC-3t):
                        // each kind must draw its OWN `NPC_<netID>` strip, so
                        // the lineup shows eight different sprites.
                        "stardustcell" => NpcKind::StardustCellBig,
                        "stardustspider" => NpcKind::StardustSpiderBig,
                        "stardustsoldier" => NpcKind::StardustSoldier,
                        "solarselenian" => NpcKind::SolarSelenian,
                        "solarcrawltipede" => NpcKind::SolarCrawltipedeHead,
                        "nebulafloater" => NpcKind::NebulaFloater,
                        "predictor" => NpcKind::Predictor,
                        "vortexhunter" => NpcKind::VortexHunter,
                        "vortexalien" => NpcKind::VortexAlien,
                        "vortexheadcrab" => NpcKind::VortexHeadcrab,
                        "moonlordhead" => NpcKind::MoonLordHead,
                        // The Moon Lord's parts (M4-NPC-3v): the hands draw
                        // from their own `(120, 180)` origin, the core carries
                        // its torso halves and the free eye from `(40, 40)`.
                        "moonlordhand" => NpcKind::MoonLordHand,
                        "moonlordcore" => NpcKind::MoonLordCore,
                        "moonlordfreeeye" => NpcKind::MoonLordFreeEye,
                        "dd2darkmage" => NpcKind::DD2DarkMage,
                        "dd2bartender" => NpcKind::DD2Bartender,
                        // The Eye of Cthulhu (M4-NPC-3m): its 6-row strip
                        // and the shared case-4 machine with the Twins.
                        "eoc" => NpcKind::EyeOfCthulhu,
                        // The Plantera + the Brain (M4-NPC-3o): the shared
                        // case-262/266 machine.
                        "plantera" => NpcKind::Plantera,
                        "brain" => NpcKind::BrainOfCthulhu,
                        "creeper" => NpcKind::Creeper,
                        // The Queen Bee (M4-NPC-3p).
                        "queenbee" => NpcKind::QueenBee,
                        "kingslime" => NpcKind::KingSlime,
                        // The boss parts (M4-NPC-3r).
                        "wof" => NpcKind::WallOfFlesh,
                        "wofeye" => NpcKind::WallOfFleshEye,
                        "hungry" => NpcKind::Hungry,
                        "skeletronhead" => NpcKind::SkeletronHead,
                        // S1-D-3 W-9 fishron wiring — the boss + hazards.
                        "fishron" => NpcKind::DukeFishron,
                        "bubble" => NpcKind::DetonatingBubble,
                        "sharkron" => NpcKind::Sharkron,
                        "sharkron2" => NpcKind::Sharkron2,
                        _ => continue,
                    };
                    let (kw, kh) = kind.size();
                    let mut n = Npc::new(
                        kind,
                        p.x + (4 + 3 * slot) as f32 * TILE_PX,
                        p.y + p.height - kh,
                        1,
                        0.0,
                    );
                    // The Moon Lord's parts declare `lifeMax = 0` (they die
                    // with the core, `NPC.cs`), so the spawn would be an
                    // instant corpse on the shot frame — give the lineup a
                    // survivable pool.
                    n.hp = n.max_hp.max(1000);
                    // The saucer family's `FindFrame` cases read the core's
                    // mirrored `ai[3]` clock; give the spawn a mid-band value
                    // so the lineup shows the attack rows rather than the
                    // stripped idle frame.
                    if n.kind.is_martian() {
                        n.ai[3] = 100.0;
                    }
                    // The Moon Lord's hands split by their `ai[2]` slot (0 =
                    // right, 1 = left, `AI_078`) and the core self-kills
                    // without both hands plus the head (`:41251-41291`) —
                    // alternate the slot so a lineup of two hands keeps the
                    // core alive and shows both mirrors.
                    if n.kind == NpcKind::MoonLordHand {
                        n.ai[2] = (slot % 2) as f32;
                    }
                    npcs.push(n);
                }
            }
        }
        let dev_break = crate::env::dev_break_xy();
        // `TR_PLACE=x,y,kind[;x,y,kind…]`: place tiles on the first update, in order.
        let dev_place: Vec<(i32, i32, u16)> = crate::env::dev_place_list();
        // `TR_SPOUT=x,y`: build the `FindWaterfalls` half-brick case at a fixed tile (a half
        // brick with full liquid on its right and a clear drop below) so the waterfall
        // overlay can be screenshotted deterministically.
        if let Some((x, y)) = crate::env::spout_xy() {
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
                    let kind_flag = crate::env::spout_kind();
                    t.flags |= kind_flag;
                }
            }
        }
        // `TR_CHEST`: the chest-panel screenshot fixture — a style-0 chest on
        // the spawn plateau's floor, stocked and opened (`OpenChest`'s state
        // arms: `chest`/`chestX`/`chestY` :33507-33522, `playerInventory` and
        // the crafting close ride the same path, `chest.rs:406-412`).
        let mut tr_chest_store: Vec<craft::WorldChest> = Vec::new();
        let tr_chest = if crate::env::chest_fixture() {
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
            tr_chest_store[idx].slots.extend_from_slice(&[
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
        wind.sway_enabled = !crate::env::no_wind();
        // M5-SEED-1: `Wind.remix_world` is the wind module's stand-in for
        // `Main.remixWorld` (`WorldGen.cs:11221`) — the rain/wind gates read
        // it (`rain.rs:524-592`, `wind.rs:294-329`). The world's own flag set
        // is the source, so a remix world's rain starts like vanilla.
        wind.remix_world = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
        let liquids_sim = LiquidSystem::new(&world);
        let mut rain = Rain::new(world_seed);
        // `SetOngoingToTemps` (`WorldFile.cs:159-162`): the load applies the
        // save-time snapshot — the rain event comes back, `cloudAlpha` :=
        // `maxRaining`. The triple rode the `.trwld` v14 block (S1-J-7).
        rain.restore_saved(world.raining, world.rain_time, world.max_raining);
        let ambient_waterfall = (0.0, 0.0, 0.0);
        let ambient_lavafall = (0.0, 0.0, 0.0);
        let ambient_counter = 0.0f32;
        // `Main.atmo` starts at its surface value; recomputed per draw below.
        let atmo = 1.0_f32;

        // Character name and the session menu (`M6`): screenshots and debug hooks skip
        // the menu so the deterministic output stays unchanged.
        let player_name = crate::env::player_name().unwrap_or_else(|| menu::random_name());
        let join_target = crate::env::join_target()
            .map(|addr| (addr, crate::env::join_password().unwrap_or_default()));
        let auto_host = crate::env::auto_host();
        let joined_before = crate::env::joined_before();
        let force_menu = crate::env::force_menu();
        // `TR_UI=<screen>` presets the front-end flow state after construction
        // (the `TR_PAGE` pattern; the plan Task 5 hook table).
        let tr_ui = crate::env::ui_preset_name();
        let skip_menu = (!force_menu && crate::env::shot_set())
            || crate::env::cam_set()
            || crate::env::hold_set()
            || crate::env::no_menu()
            || crate::env::map_open()
            || joined_before
            || auto_host.is_some()
            || tr_ui.is_some();
        let menu_state = if skip_menu {
            None
        } else {
            let mut m = menu::Menu::new(settings.clone(), player_name.clone());
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
        if crate::env::map_open() {
            world_map.set_open(true);
        }
        // `TR_AMBIENCE=<SkyEntityType>[,seed]`: queue one forced spawn (the
        // `AmbienceServer.ForceEntitySpawn` path, `AmbienceServer.cs:139-142`)
        // with a pinned seed, so the creature skies stay screenshotable
        // without waiting for the 10-120 s scheduler. Test harness only.
        let ambience_pin = crate::env::ambience_pin();
        // The chests' loot is fixed at GENERATION time (S1-A-4): the
        // placement sites rolled it through the stocking hook and the
        // records ride `World.chests`, so a freshly generated world's store
        // reads them directly — the C#'s `AddBuriedChest` inline shape. A
        // `.wld` load runs no worldgen, so its store is the file's own
        // chests (`LoadChests`, `WorldFile.cs:2723-2805` — `Main.chest` is
        // the file, nothing rolls); a `.trwld` load's store is the file's
        // native records (the gen-time loot the save carried). The
        // `TR_CHEST` fixture merges in as before.
        let ore_tiers_seed = world.ore_tiers;
        let seeded_chests = crate::chest_gen::store_from_records(&world, &sections.native.chests);
        // M6-SAVE-1c: `BannerSystem.Load` (`BannerSystem.cs:188-212`) — the
        // v8 section's recorded counts restore the tally; the vanilla rides
        // them inside the world-flags block.
        let seeded_banners: Vec<(i32, i32)> = sections
            .native
            .banners
            .iter()
            .map(|b| (b.banner, b.kills))
            .collect();
        // W4-e24: `BannerSystem.Load`'s `version >= 289` claimable half
        // (`BannerSystem.cs:203-211`) — the v18 section's recorded claimable
        // counts restore the pool the milestone fills.
        let seeded_claimable: Vec<(i32, u16)> = sections
            .native
            .banner_claimable
            .iter()
            .map(|b| (b.banner, b.count))
            .collect();
        let world_chests = if sections.from_wld {
            let mut loaded = crate::chest::from_wld(&sections.data.chests);
            // `LoadChests:2801-2804`: the pre-115 dresser pass runs after the
            // file's records are in the store (`FixDresserChests:3269`), and
            // only for a file old enough to predate the stored chest list.
            if sections.version < 115 {
                crate::chest::fix_dresser_chests(&mut loaded, &world);
            }
            loaded
        } else {
            // The `.trwld`/fresh-world path: the file's native records (the
            // gen-time loot for a saved generated world) + the `TR_CHEST`
            // fixture. Nothing rolls (S1-A-4).
            crate::chest_gen::merge_store(seeded_chests, tr_chest_store)
        };
        // `LoadBestiary:3402` + `LoadCreativePowers:3418` (the last two
        // sections, applied to their state owners). A `.wld`'s bestiary tracker
        // *replaces* the built one — the world file is the `.wld` write target
        // for the kills (owner decision 2026-11-08) and the sights/chats have
        // no other form, so an empty bag would wipe them. The `.trwld` half:
        // M6-SAVE-1d's native v9 section is the vanilla home
        // (`NPCKillsTracker.Save`); a pre-v9 world falls back to the
        // `.trplayer` sidecar's line, which stays the back-compat restore.
        let mut app_bestiary = {
            let mut tracker = tr_core::bestiary::WorldBestiary::new();
            if sections.from_wld {
                tracker = tr_core::wld::sections::bestiary::from_wld(&sections.data.bestiary);
            } else {
                let world_kills: Vec<(String, i32)> = sections
                    .native
                    .bestiary
                    .iter()
                    .map(|b| (b.credit_id.clone(), b.kills))
                    .collect();
                let kills = if world_kills.is_empty() {
                    loaded_kills
                } else {
                    world_kills
                };
                tracker.kills.load_pairs(kills.into_iter());
            }
            tracker
        };
        let mut creative_powers = crate::creative::CreativePowers::new();
        if sections.from_wld {
            crate::creative::apply_persistent(&mut creative_powers, &sections.data.creative_powers);
        }
        // `Main.mouseColor`'s stored colour seeds the cursor tint
        // (`Main.cs:67199`); `frame.rs`'s per-frame sync replaces it from the
        // live settings, so this is only the pre-first-frame value.
        let cursor_mouse_color = settings.mouse_color;
        let mut app = App {
            window: None,
            talk_npc: None,
            npc_chat_text: String::new(),
            npc_chat_last_hovered: -1,
            prevent_joja_cola_dialog: false,
            reroll_dryad_text: 0,
            gpu: None,
            renderer: None,
            post: None,
            noise_tex: None,
            sand: crate::sand::Sand::new(1),
            dust: {
                // The `Main.dust[6001]` init (`Main.cs:66985`) with the type-34
                // wet probe injected once: `Collision.WetCollision`'s port is
                // `player::wet_collision` (`Dust.cs:248` probe, `S1-E-1`
                // decision 2) — tr-core cannot depend on tr-app.
                let mut dust = tr_core::particles::DustPool::new();
                dust.set_wet_probe(Box::new(crate::player::wet_collision));
                dust
            },
            // `Main.waterStyle` starts 0 (`Main.cs:1105`'s static default);
            // `DrawWaters` (`:57475`) writes it each foreground draw.
            water_style: 0,
            water_mask: None,
            targets: None,
            target_compositor: None,
            layer_render_count: 1,
            capture_light_params: None,
            scene_state: tr_core::scene::SceneState::new(),
            sandstorm: tr_core::scene::Sandstorm::new(world_seed),
            sandstorm_sky: crate::render::sandstorm_sky::SandstormSky::default(),
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
            next_npc_uid: 1,
            shop_npc: None,
            shop_stock: Vec::new(),
            shop_sellback: crate::shop::SellbackHelper::default(),
            in_reforge_menu: false,
            reforge_slot: EMPTY_SLOT,
            reforge_cooldown: 0,
            mouse_reforge: false,
            reforge_scale: 0.8, // `Main.cs:67157`
            shop_mood: 1.0,
            discount_available: false,
            shop_closed_flag: false,
            creative_powers,
            #[cfg(feature = "debug")]
            debug_panel: tr_debug::Panel::new(),
            favorite_slots: Vec::new(),
            play_world: None,
            world_gen_job: None,
            capture: crate::capture::CaptureManager::default(),
            invasion_spawn_timer: 0.0,
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
                if let Some(ids) = crate::env::armor_set() {
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
                for (slot, kind) in equip.dye.iter_mut().zip(crate::env::dye_kinds()) {
                    if kind != 0 {
                        *slot = ItemSlot {
                            kind,
                            name: registry::name(kind),
                            count: 1,
                            prefix: 0,
                        };
                    }
                }
                // `TR_DMOUNT=<kind>`: the saddle-dye stand-in — `miscDyes[3]`
                // (`Player.cs:1484`, the vanilla mount dye slot).
                let dmount = crate::env::misc_dye_mount();
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
            equip_page: crate::env::parsed_raw("TR_EQUIPPAGE").unwrap_or(0),
            // `TR_EQUIPPAGE` seeds the *selected* page too, so the pin survives
            // the per-frame `SelectEquipPage` resolution below.
            equip_page_selected: crate::env::parsed_raw("TR_EQUIPPAGE").unwrap_or(0),
            dye_swap_counter: 0,
            selected: 0,
            selection: crate::hotbar::SelectionState::new(),
            block_mouse: false,
            hotbar_scales: {
                let mut s = [0.75; HOTBAR];
                s[0] = 1.0;
                s
            },
            // `TR_CHEST` opens the chest (`OpenChest` sets `playerInventory`,
            // `Player.cs:33517` — chest.rs:409), so the panel + the player
            // inventory draw together.
            inventory_open: crate::env::inv_open() || tr_chest.is_some(),
            craft_filter: None,
            craft_filters: crate::craft_filter::FilterBoard::default(),
            bestiary: app_bestiary,
            research: crate::research::ResearchTracker::default(),
            last_research_version: 0,
            town_rooms,
            shimmered_npcs: crate::town::ShimmeredTownNpcs::from_wld_indices(
                &sections.data.npcs.shimmered,
            ),
            bestiary_open: crate::env::bestiary_open(),
            // `TR_BESTIARY=<n>` also selects entry n (1-based) for the
            // info-page screenshots; `0`/absent stays on the grid.
            bestiary_selected: crate::env::bestiary_selected().and_then(|n| {
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
                if crate::env::chat_open() {
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
            // `UIAchievementsMenu`'s overlay state (M6-ACH-2):
            // `TR_ACHIEVEMENTS` opens it for the screenshot runs, the
            // categories all start on (`InitializePage`'s `SetState(true)`).
            achievements_open: crate::env::achievements_open(),
            achievements_scroll: 0.0,
            // The filter bits default to all-on (`SetState(value: true)`);
            // a shot run can pin a single category through the bitmask.
            achievements_categories: {
                let mask = crate::env::achievements_categories().unwrap_or(0xF);
                [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0]
            },
            achievements_scroll_drag: None,
            achievements_back_sounded: false,
            achievements_handle_hover: false,
            // `InGameNotificationsTracker`'s static list starts empty
            // (`InGameNotificationsTracker.cs:109-112`).
            notifications: ui::notifications::NotificationsTracker::default(),
            emotes: {
                // `TR_EMOTES` opens the wheel for the screenshot runs —
                // through the same `OnActivate` rebuild the click arm uses
                // (W4-g54: the page needs its groups, not just the flag).
                let mut sys = emotes::EmoteSystem::default();
                if crate::env::emotes_open() {
                    sys.open();
                    sys.new_bubble(0);
                }
                sys
            },
            emote_scroll_drag: None,
            banners: {
                let mut banners = banners::BannerSystem::default();
                banners.load_pairs(seeded_banners);
                banners.load_claimable_pairs(seeded_claimable);
                banners
            },
            fake_cursor: banners::FakeCursorItem::default(),
            town_look_timer: 0.0,
            town_arrival: crate::town_arrival::ArrivalState::new(),
            reuse_delay: 0.0,
            // The panel ships open with the inventory (`Main.cs:41706-41709`);
            // `TR_NOCRAFT=1` turns it off and `TR_CRAFT=1` forces it back on
            // (the `TR_INV` presence pattern). An open chest closes it
            // (`OpenChest` → `NewCraftingUI.Close`, `chest.rs:408`) — and the
            // chest panel hides under it (`ChestUI.cs:86`).
            crafting_open: (!crate::env::no_craft() || crate::env::craft_open())
                && tr_chest.is_none(),
            // The load-time loot roll (the hoist above), merged with the
            // `TR_CHEST` fixture's own chests (they keep their slots; the roll's
            // anchors append) — the `.trwld` path. A `.wld`'s store is built
            // beside this one, in the hoist above. The chest names ride the
            // `WorldChest` entity (M6-SAVE-1 phase 3's fold), so the old
            // `App.chest_names` side map is gone — the records' names land in
            // `store_from_records`' folded entity directly.
            world_chests,
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
            chest_default_name: String::new(),
            text_blinker: ui::chest::TextBlinker::default(),
            // The craft RNG (`Main.rand` in the C#, `Recipe.cs:221`) seeds
            // from the world; vanilla seeds `Main.rand` from the clock
            // (`Main.cs:6461-6463`) — the same deterministic stand-in as the
            // wiring RNG (S1-L-6c).
            craft_rng: tr_core::unified_random::UnifiedRandom::new(world_seed as i32 | 1),
            coin_slot: crate::ui::coin_slot::CoinSlotState::new(),
            revenge_markers: Vec::new(),
            revenge_tick: 0,
            menu: menu_state,
            front: None,
            front_backup: None,
            players: Vec::new(),
            worlds: Vec::new(),
            front_hint: None,
            front_hover_hit: None,
            front_mouse_down: false,
            front_scroll: 0.0,
            front_scroll_drag: None,
            front_scroll_drag_enabled: false,
            session_draft: None,
            vk_grid: crate::env::vk_grid(),
            settings,
            player_name,
            net: net_role,
            remote_players: HashMap::new(),
            local_id: 0,
            net_timer: 0.0,
            net_state: net::client_state::NOT_CONNECTED,
            net_pending_sections: Vec::new(),
            net_info: None,
            net_status_text: String::new(),
            net_status_max: 0,
            net_status_count: 0,
            net_last_recv: Instant::now(),
            net_dropped: false,
            net_npcs: HashMap::new(),
            net_npcs_dirty: false,
            net_npc_states: HashMap::new(),
            net_npc_outbox: Vec::new(),
            net_npc_despawns: Vec::new(),
            net_npc_uid_slot: HashMap::new(),
            net_npc_last_info: HashMap::new(),
            proj_sync: crate::app::net_proj_sync::ProjSync::default(),
            net_proj_outbox: Vec::new(),
            net_proj_prev: HashMap::new(),
            net_sync_cadence_timer: 0.0,
            net_interp_timer: 0.0,
            net_npc_gen: HashMap::new(),
            net_items: HashMap::new(),
            net_items_dirty: false,
            net_item_last: HashMap::new(),
            net_reserved: HashMap::new(),
            net_ps_last: net::PlayerState::default(),
            net_armor_last: [0; 20],
            net_beat: 0,
            net_play_counter: 0,
            last_item_update: 0,
            net_npc_slots: Vec::new(),
            net_ping_at: None,
            net_ping_waiting: false,
            net_rtt_ms: 0,
            last_ping_pos: Vec2::ZERO,
            last_ping_time: 0.0,
            join_target,
            map: Some(world_map),
            map_queue: map::MapUpdateQueue::default(),
            map_grab: None,
            minimap_hover: None,
            // `Main.mapStyle = 1` (`Main.cs:66750`); `TR_MAPSTYLE=<0|1|2>` selects
            // it for screenshots (the `TR_EQUIPPAGE` pattern; an invalid or
            // out-of-range value falls back to 1).
            map_style: crate::env::map_style(),
            team_cooldown: 0,
            hide_pvp_and_team_icons: false,
            pvp_icons_forced: crate::env::pvp_icons(),
            audio: audio::Audio::new(assets::content_dir()),
            music: audio::MusicSystem::new(assets::content_dir()),
            autosave: 0.0,
            mouse_item: EMPTY_SLOT,
            quick_mana_prev: false,
            quick_buff_prev: false,
            mouse_npc_index: -1,
            mouse_npc_type: -1,
            ui_left_press: false,
            ui_right_press: false,
            map_left_press: false,
            ui_right_release: false,
            control_throw: false,
            drop_right_edge: false,
            no_throw: 0,
            last_mouse_interface: false,
            void_lens_selection: Vec::new(),
            slot_glow: crate::ui::glow::GlowState::default(),
            ui_right_release_latch: true,
            ui_left_release: false,
            buff_alpha: [0.4; crate::buffs::MAX_BUFFS],
            buff_removals: Vec::new(),
            focus_recipe: 0,
            craft_last_available: Vec::new(),
            craft_scroll: 0.0,
            craft_scroll_drag: None,
            craft_scroll_hover: false,
            nearby_crafting_mouse_over: false,
            current_recipe_being_crafted: -1,
            prevent_craft_click: false,
            pointer: ui::pointer::PointerState::default(),
            pointer_ms: 0.0,
            pointer_event: None,
            mouse_pos: if crate::env::deterministic_pointer_park() {
                Vec2::new(-64.0, -64.0)
            } else {
                crate::env::mouse_pin().unwrap_or(Vec2::ZERO)
            },
            cursor: ui::cursor::CursorState::with_mouse_color(cursor_mouse_color),
            os_cursor_hidden: None,
            mine_down: false,
            place_down: false,
            mine_target: None,
            mine_timer: 0.0,
            mine_hardness: 0.0,
            mine_wall_hits: 0,
            mine_crack_styles: crate::render::tile_cracks::CrackStyles::new(),
            mine_crack_rng: tr_core::UnifiedRandom::new(world_seed as i32),
            tile_draw_rand: tr_core::UnifiedRandom::new(world_seed as i32),
            liquid_rand: tr_core::UnifiedRandom::new(world_seed as i32),
            edit_target: None,
            smart_cursor: Default::default(),
            smart_cursor_used: false,
            ingame_options: {
                // `TR_OPTIONS` (the M7-UI-4 screenshot fixture): the window opens
                // at boot so `TR_SHOT` captures the panel.
                let mut o = crate::ui::ingame_options::IngameOptions::new();
                o.open = crate::env::ingame_options_open();
                o
            },
            world_path: world_path.clone(),
            dev_break,
            dev_place,
            camera: Vec2::ZERO,
            camera_state: Default::default(),
            gfx_quality: Default::default(),
            world_events: Default::default(),
            zoom: 1.0,
            zoom_pinned: crate::env::zoom_pinned(),
            debug: crate::env::debug(),
            debug_overlay: false,
            worldgen_debug_open: false,
            keys: HashSet::new(),
            keybinds: crate::input_map::default_binds(),
            map_binds: crate::input_map::default_map_binds(),
            mount_prev: false,
            quick_heal_prev: false,
            // `Main.gamePaused` starts false and `ResetGameCounter` pins
            // `GameUpdateCount` at 65536 (`Main.cs:6442-6445`, the `OnWorldLoad`
            // hook at `:6605`) — both ride this constructor, so a `swap_world`
            // reset is the same write the boot path makes.
            game_paused: false,
            window_focused: true,
            mouse_back_held: false,
            game_update_count: 65536,
            stepper: crate::world_stepper::WorldUpdateStepper::new(),
            step_input_down: false,
            step_input_was: false,
            mouse_down: false,
            last_mouse: Vec2::ZERO,
            last_frame: Instant::now(),
            frame_history: VecDeque::with_capacity(256),
            last_stats: tr_render::sprite::RenderStats::default(),
            bench_frames: crate::env::bench_frames(),
            bench_samples: Vec::new(),
            frame_count: 0,
            shot_path: crate::env::shot_path().map(PathBuf::from),
            shot_at: crate::env::shot_at().unwrap_or(20),
            atlas_stats,
            combat_texts: Vec::new(),
            popup_texts: crate::popup_text::Pool::default(),
            static_sheets: None,
            fonts_cache: None,
            light: LightMap::new(),
            scratch: DrawScratch::default(),
            tile_stats: TileDrawStats::default(),
            stats_on: crate::env::stats_on(),
            stats_acc: StatsAcc::default(),
            debug_cache: DebugCache::default(),
            light_mode: crate::env::light_mode().unwrap_or(LightMode::Color),
            legacy_light: None,
            // `Main.mouseTextColor` starts at 225 and drifts toward 255.
            mouse_text_color: 225,
            mouse_text_color_dir: 1,
            // `Main.invAlpha = 1f; Main.invDir = 1f` (`Main.cs:66846-66847`).
            inv_pulse: crate::ui::inv_pulse::InvPulse::default(),
            // `Main.hideUI`/`releaseUI`'s static defaults (`Main.cs:2085-2087`:
            // false / false — the first released-frame pass arms the latch).
            hide_ui: false,
            release_ui: false,
            release_map_style: false,
            // `Main.demonTorch = 1f; Main.demonTorchDir = 1` (Main.Initialize).
            demon_torch: 1.0,
            demon_torch_dir: 1.0,
            // `Main.masterColor = 1f; Main.masterColorDir = 1` (`Main.cs:66833-66834`).
            master_color: 1.0,
            master_color_dir: 1,
            // `Main.martianLight = 1f; Main.martianLightDir = 1`
            // (`Main.cs:66831-66832`).
            martian_light: 1.0,
            martian_light_dir: 1.0,
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
            tile_anim: tr_core::drawdata_anim::TileAnimator::new(),
            tile_anim_step: FixedStepper::new(1.0 / 60.0, 8),
            // W4-f14: the cage animator is PROCESS state — constructed
            // once per App with the world-seed warm-up stand-in (the C#
            // seeds Main.rand from DateTime.Now.Ticks, Main.cs:6463).
            cage_anim: tr_core::drawdata_cages::CritterCageAnimator::new(world_seed as i32),
            wiring: tr_core::wiring::Wiring::new(),
            wiring_step: FixedStepper::new(1.0 / 60.0, 8),
            te_step: FixedStepper::new(1.0 / 60.0, 8),
            growth_step: FixedStepper::new(1.0 / 60.0, 8),
            // M6-SAVE-1f: the tiers ride the World (the worldgen setup roll
            // wrote them there; a loaded save's v11 section restored them),
            // so the runtime copy reads them back instead of the static
            // defaults — the altars' `smash_altar` then advances the same
            // values `save_world` writes out.
            ore_tiers: ore_tiers_seed,
            rain,
            ambient_waterfall,
            ambient_lavafall,
            ambient_counter,
            wind,
            world_tick_step: FixedStepper::new(1.0 / 60.0, 8),
            atmo,
            scene_metrics: tr_core::scene::SceneMetrics::new(),
            ambience: {
                let mut ambience = crate::ambience::Ambience::new();
                if let Some((ty, seed)) = ambience_pin {
                    ambience.forced_spawn_for_tests(ty, -1, seed);
                }
                ambience
            },
            lantern_sky: crate::render::lantern_sky::LanternSky::new(world_seed),
            lantern_sky_step: FixedStepper::new(1.0 / 60.0, 8),

            tree_wind_counter: 0.0,
            grass_wind_counter: 0.0,
            sunflower_wind_counter: 0.0,
            sunflower_wind_step: FixedStepper::new(1.0 / 60.0, 8),
            vine_wind_counter: 0.0,
            leaves: Vec::new(),
            // `Main.gore = new Gore[601]` (`Main.cs:66990`) — the 600 live
            // slots plus the refuse sentinel; `goreTime` seeds its 600
            // (`Gore.cs:1507-1510`).
            gore: tr_core::gore::GorePool::new(),
            entity_lights: crate::entity_lights::EntityLights::default(),
            // `Main.cs:66673-66674` — `essScale = 1f; essDir = -1;`.
            ess_scale: 1.0,
            ess_dir: -1,
            liquid_alpha: [0.0; 15],
            items: Vec::new(),
            // The C# seeds every `SwapRandom` key with the world seed
            // (`Main.cs:3212`); the registry creates them lazily.
            rngs: NamedRngs::new(world_seed as i32),
            item_step: FixedStepper::new(1.0 / 60.0, 8),
            swing: 0.0,
            swing_style: 0,
            swing_aim: 0.0,
            shot_fired: false,
            swing_max: 0.0,
            whip_snake_refire: false,
            place_prev_down: false,
            place_released: false,
            mine_prev_down: false,
            mine_released: false,
            item_time: 0.0,
            tool_time: 0.0,
            hit_tile: tr_core::hit_tile::HitTile::new(),
            hit_replace: tr_core::hit_tile::HitTile::new(),
            pound_release: false,
            mining_can_hit_walls: true,
            coins,
            projectiles: ProjectilePool::new(),
            // `Main.LocalGolfState = new GolfState()` (`Main.cs:6483`).
            golf_state: tr_core::golf_state::GolfState::new(),
            tomb_falls: Vec::new(),
            lost_coins: 0,
            lost_coin_string: String::new(),
            death_text: String::new(),
            respawn_skip_frames: 0,
            respawn_skip_available: false,
            invasion_progress: -1,
            invasion_progress_max: 0,
            invasion_progress_icon: 0,
            invasion_progress_wave: 0,
            invasion_progress_display_left: 0,
            invasion_progress_alpha: 0.0,
            invasion_progress_near: false,
            tomb_texts: Vec::new(),
            projectile_step: FixedStepper::new(1.0 / 60.0, 8),
            dev_throw: crate::env::dev_throw(),
            summon_shot: crate::env::summon_shot(),
            world_key: None,
            hell_black_box_bottom: 0.0,
            ug_back: crate::render::bg_styles::UgBackState {
                style: 0,
                old_style: 0,
                transition: 0.0,
            },
        };
        // The wiring RNG (`Main.rand` in the C#, `Wiring.cs:811-812`) seeds
        // from the world; vanilla seeds it from the clock (`Main.cs:6461-6463`).
        app.wiring.seed_rand(world_seed | 1);
        // `SetOngoingToTemps:169-172`: the sandstorm snapshot the load read out
        // of the flags block (`_tempSandstorm*`, `LoadWorldFlags:2277-2283`)
        // restores the event's live state, exactly as the rain triple above
        // does for `Main.raining`.
        app.sandstorm.happening = app.world.sandstorm_happening;
        app.sandstorm.time_left = app.world.sandstorm_time_left;
        app.sandstorm.severity = app.world.sandstorm_severity;
        app.sandstorm.intended_severity = app.world.sandstorm_intended_severity;
        // `TR_UI=<screen>`: preset the front-end flow state for screenshots (the
        // `TR_PAGE` pattern — the plan Task 5 hook table).
        if let Some(preset) = tr_ui.as_deref().and_then(tr_ui_preset) {
            app.preset_ui(preset);
        }
        app
    }
}

/// Whether `path` is the port's own world format (`.trwld`) — the one whose
/// player/map sidecars and `save::load` apply. A `.wld` is a retail file: its
/// sections, and only its world file, are the state's home.
pub(crate) fn is_native_trwld(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "trwld")
}

/// What [`load_world`] hands [`App::new`] beside the `World`: the loaded
/// `.wld`'s section payloads, the format version their load fix-ups gate on,
/// and the native `.trwld`'s own side records (`save::WorldSide` — its v7-v10
/// sections). Exactly one of the two is populated for a given file.
pub(crate) struct WorldSections {
    /// The native `.trwld`'s v7-v10 side records; empty for a `.wld`.
    pub native: save::WorldSide,
    pub data: wld::sections::SectionData,
    /// `WorldFile._versionNumber`; 0 for a `.trwld` (no version gates apply).
    pub version: i32,
    /// Whether `data` came from a `.wld`'s section table.
    pub from_wld: bool,
}

impl Default for WorldSections {
    fn default() -> Self {
        Self {
            native: save::WorldSide::default(),
            data: wld::sections::SectionData::default(),
            version: 0,
            from_wld: false,
        }
    }
}

impl WorldSections {
    /// A native `.trwld`'s side records as the whole payload.
    pub(crate) fn native(native: save::WorldSide) -> Self {
        Self {
            native,
            ..Self::default()
        }
    }
}

/// Loads the world (`save::load` for the port's `.trwld`,
/// `wld::load_with_sections` for the retail `.wld`), applies the `TR_DAY` clock
/// override and frames the loaded tiles and walls — the load half of the
/// startup, verbatim from `main`.
///
/// The three `.wld` sections that are world state (`World.signs`,
/// `World.pressure_plates`, `World.tile_entities`) are applied here in the C#'s
/// `LoadWorld_Version2` order — signs, tile entities, plates — so the loaded
/// world is complete before the framing passes and the `TR_*` hooks run; the
/// App-owned sections ride [`WorldSections`] to `App::new`. The native
/// `.trwld`'s records ride `sections.native` (`save::WorldSide`).
pub(crate) fn load_world(world_path: &Path) -> Result<(World, WorldSections)> {
    let t0 = Instant::now();
    let (mut world, sections) = if is_native_trwld(world_path) {
        let (world, native) = save::load(&world_path)?;
        (
            world,
            WorldSections {
                native,
                ..WorldSections::default()
            },
        )
    } else {
        let mut loaded = wld::load_with_sections(&world_path)?;
        // `LoadWorld_Version2:1786` (signs) → `:1808` (tile entities) → `:1825`
        // (pressure plates): the world-state halves, in that order.
        tr_core::wld::sections::signs::apply_loaded(&mut loaded.world, &loaded.sections.signs);
        tr_core::wld::sections::tile_entities::apply_loaded(
            &mut loaded.world,
            &loaded.sections.tile_entities,
        );
        tr_core::wld::sections::pressure_plates::apply_loaded(
            &mut loaded.world,
            &loaded.sections.pressure_plates,
        );
        (
            loaded.world,
            WorldSections {
                data: loaded.sections,
                version: loaded.version,
                from_wld: true,
                ..WorldSections::default()
            },
        )
    };
    // The identity fallbacks the C# runs at load (W4-c43):
    // * `LoadHeader:1962-1969`: a pre-181 `.wld` (no GUID field) gets
    //   `UniqueId = Guid.NewGuid()` — the port's clock-seeded stand-in
    //   (`wld::stamp_created_identity`'s GUID arm). A v >= 181 `.wld` keeps
    //   its stored GUID even when it is Guid.Empty; a pre-v18 `.trwld` has
    //   no GUID field at all, so its zero reads as absent.
    // * `WorldFileData.GetFileData:315-318`: a file with no LastPlayed
    //   field falls back to `File.GetLastWriteTimeUtc(file)`. The port's
    //   zero stamp (a pre-v18 `.trwld`, a pre-284 `.wld`, or a never-saved
    //   world) takes the same mtime fallback.
    let guid_absent = if sections.from_wld {
        sections.version < 181
    } else {
        world.unique_id == [0u8; 16]
    };
    if guid_absent {
        let mut rng = tr_core::wld::identity_rng();
        let mut guid = [0u8; 16];
        rng.next_bytes(&mut guid);
        guid[7] = (guid[7] & 0x0F) | 0x40;
        guid[8] = (guid[8] & 0x3F) | 0x80;
        world.unique_id = guid;
    }
    if world.last_played == 0 {
        if let Some(stamp) = tr_core::clock::file_mtime_binary(world_path) {
            world.last_played = stamp;
        }
    }
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
    if let Some(v) = crate::env::pool_rect() {
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
    if let Some(v) = crate::env::paint_rect() {
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
    if let Some(v) = crate::env::fill_rects() {
        apply_tr_fill(&mut world, &v);
    }
    // `TR_PLACE=x,y,kind[,style[,dir]][;...]`: place tile objects through the
    // Phase-2 route (`edit::place_item_tile`, the player's own RMB call) for
    // the object screenshots.
    apply_tr_places(&mut world, &crate::env::place_ops());
    // `TR_TOGGLE=x,y[,n][;...]`: n presses of the Phase-4 interaction
    // dispatcher (`edit::interact_at`) — the door closed→open→closed scenario.
    apply_tr_toggles(&mut world, &crate::env::toggle_ops());
    // `TR_TILES=x,y,kind,fx,fy[;...]`: write tiles with explicit frames at
    // load — the render fixture for the M1.4-TO-4 kinds (the plan stages the
    // scene by direct World writes, not through Phase 1 placement).
    apply_tr_tiles(&mut world, &crate::env::tile_ops());
    // TR_DAY also sets the world clock so NPC day/night logic matches the pinned sky.
    if let Some(d) = crate::env::day_override() {
        world.day_time = d >= 0.5;
        world.time = if world.day_time {
            d as f64 * 54000.0
        } else {
            (1.0 - d as f64) * 32400.0
        };
    }
    // `TR_MOON=<0..7>` pins `Main.moonPhase` — the night light floor follows it
    // (`Main.cs:63792-63829`), so the full/new-moon A/B stays screenshotable.
    if let Some(p) = crate::env::moon_phase() {
        world.moon_phase = p & 7;
    }
    // `TR_TIME=<ticks>` pins the raw world clock — the dawn/dusk windows the
    // `TR_DAY` day-half mapping cannot reach (the sunrise visibility spans
    // ticks 0..10800, the sunset 43200..54000).
    if let Some(t) = crate::env::time_pin() {
        world.time = t;
        world.day_time = t < 54000.0;
    }
    // `TR_SUNDIAL=1`/`TR_MOONDIAL=1`: arm the dial fast-forwards at start
    // (BG-21's screenshot pins). The flags only — `sundialing()` would also
    // start the 8-day cooldown and mute the dial glow the shot pair needs.
    if crate::env::sundial_pin() {
        world.fast_forward_to_dawn = true;
    }
    if crate::env::moondial_pin() {
        world.fast_forward_to_dusk = true;
    }
    // `TR_ECHO=tx,ty,w,h`: echo-coat a rectangle — the `invisibleBlock` +
    // `invisibleWall` bits (`WorldGen.paintCoatTile` case 2 / `paintCoatWall`,
    // `WorldGen.cs:44134-44217`) — so the light pass's invisible handling stays
    // screenshotable without a coating item.
    if let Some(v) = crate::env::echo_rect() {
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
    // The variant reset-roll stream rides the world seed (S1-A-3): vanilla
    // reseeds `Main.rand` from the clock on load (`Main.cs:6461-6463`), the
    // port pins it deterministically.
    world.seed_frame_rng();
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
    if sections.from_wld {
        log::info!(
            "wld sections: {} chests, {} signs, {} npcs, {} tile entities, {} plates, {} rooms, {} bestiary kills, {} powers",
            sections.data.chests.len(),
            sections.data.signs.len(),
            sections.data.npcs.town.len(),
            sections.data.tile_entities.len(),
            sections.data.pressure_plates.len(),
            sections.data.town_rooms.len(),
            sections.data.bestiary.kills.len(),
            sections.data.creative_powers.records.len(),
        );
    }
    Ok((world, sections))
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

fn apply_tr_fill(world: &mut World, v: &str) {
    let mut touched = false;
    for group in v.split(';') {
        let Some(nums) = crate::env::tr_nums(group) else {
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

fn apply_tr_places(world: &mut World, ops: &[crate::env::TrPlace]) {
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

fn apply_tr_tiles(world: &mut World, ops: &[crate::env::TrTile]) {
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

fn apply_tr_toggles(world: &mut World, ops: &[crate::env::TrToggle]) {
    for op in ops {
        for _ in 0..op.n {
            // No player exists at load: `direction` is 1 (right-facing) and
            // no entity can block the door cells.
            let hit = edit::interact_at(world, op.x, op.y, 1, &|_, _| false);
            match hit {
                edit::TileInteraction::None => {
                    log::info!("TR_TOGGLE {},{} -> none", op.x, op.y)
                }
                edit::TileInteraction::StationInteract { kind } => {
                    log::info!("TR_TOGGLE {},{} -> station probe ({kind})", op.x, op.y)
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
                // W4-e28: the station-buff grant needs the player's buff
                // slots (none at load) — the dispatch only reports the row.
                edit::TileInteraction::StationBuff { buff_type, .. } => {
                    log::info!("TR_TOGGLE {},{} -> station buff {buff_type}", op.x, op.y)
                }
                edit::TileInteraction::BankOpen { bank } => {
                    log::info!("TR_TOGGLE {},{} -> bank {bank}", op.x, op.y)
                }
                edit::TileInteraction::HitSwitch { x, y } => {
                    log::info!("TR_TOGGLE {},{} -> switch {x},{y}", op.x, op.y)
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
/// Builds the sprite atlas and `Lookups` with the world-independent halves cached
/// across calls — the world-swap fast path. `static_cache` holds the decoded static sheets (moved
/// in and back out, no pixel copy) and `fonts_cache` the glyph tables; both are
/// world-independent, so a swap only re-decodes the world's own tile/wall
/// sheets. A cached font build returns `None` for the page texture: the caller
/// keeps the `FontAtlas` it already uploaded.
pub(crate) fn build_atlas_reusing_static(
    world: &World,
    assets_dir: &Path,
    static_cache: &mut Option<Vec<tr_render::atlas::AtlasImage>>,
    fonts_cache: &mut Option<assets::FontTables>,
) -> Result<(
    Atlas,
    Option<tr_render::FontAtlas>,
    assets::Lookups,
    (u32, u32, usize, usize),
)> {
    build_atlas_inner(world, assets_dir, static_cache, fonts_cache)
}

fn build_atlas_inner(
    world: &World,
    assets_dir: &Path,
    static_cache: &mut Option<Vec<tr_render::atlas::AtlasImage>>,
    fonts_cache: &mut Option<assets::FontTables>,
) -> Result<(
    Atlas,
    Option<tr_render::FontAtlas>,
    assets::Lookups,
    (u32, u32, usize, usize),
)> {
    let (mut tile_ids, wall_ids) = assets::usage(&world);
    // The hotbar can place these even when the loaded world does not use them yet.
    tile_ids.insert(4); // torch
    tile_ids.insert(7); // copper ore
    // `TR_POSE=sit|sleep` writes the chair/bed beside the spawn after the
    // usage scan — the pose pin's sheets ride along.
    if crate::env::pose_set() {
        tile_ids.insert(15); // chair
        tile_ids.insert(79); // bed
    }
    log::info!(
        "world uses {} tile types and {} wall types",
        tile_ids.len(),
        wall_ids.len()
    );

    let t1 = Instant::now();
    // Only the world's own sheets depend on the world; the static half and the
    // retail fonts are re-decoded at boot and reused by every swap (BUG-21: the
    // Play froze for ~1.4 s re-decoding ~7 000 sheets and ~1–2 s more on the
    // fonts, all of them already in memory). `world ++ static ++ font` is the
    // pre-split list's order, so the atlas layout and every UV are unchanged.
    let mut images = assets::load_world_sheets(&assets_dir, world, &tile_ids, &wall_ids)?;
    let world_len = images.len();
    let cached = static_cache.take();
    let static_reused = cached.is_some();
    let static_sheets = match cached {
        Some(tail) => tail,
        None => assets::load_static_sheets(&assets_dir),
    };
    let static_len = static_sheets.len();
    images.extend(static_sheets);
    let cached_fonts = fonts_cache.take();
    let fonts_reused = cached_fonts.is_some();
    let (tables, font_atlas) = match cached_fonts {
        Some(t) => {
            // Straight back in: the cache is world-independent and the next swap
            // must hit it too.
            *fonts_cache = Some(t.clone());
            (t, None)
        }
        None => {
            let loaded = assets::load_fonts();
            let t = loaded.tables();
            *fonts_cache = Some(t.clone());
            (t, loaded.atlas)
        }
    };
    log::info!(
        "{} static sheets ({}), fonts ({})",
        static_len,
        if static_reused { "reused" } else { "decoded" },
        if fonts_reused { "reused" } else { "decoded" }
    );
    let font = tr_render::atlas::font_sheet();
    images.push(font.0);
    log::info!(
        "decoded {} sheets in {:.0} ms",
        images.len(),
        t1.elapsed().as_secs_f32() * 1000.0
    );

    let t2 = Instant::now();
    // `build_ref` borrows the sheets, so the static half survives the packing:
    // the tail is split back off the packed list and refills the cache (the boot
    // path's too, so the first Play skips the re-decode as well).
    let atlas = Atlas::build_ref(&images, 8192);
    let tail: Vec<_> = images.drain(world_len..world_len + static_len).collect();
    *static_cache = Some(tail);
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
        tables.ui,
        tables.item,
        tables.death,
        tables.combat_text,
        tables.combat_crit,
    );

    let atlas_stats = (
        atlas.width,
        atlas.height,
        atlas.memory_bytes(),
        atlas.sheets.len(),
    );
    Ok((atlas, font_atlas, lookups, atlas_stats))
}

/// The world-dump diagnostics: `TR_DUMP_TILES` prints the raw tile grid and
/// the `TR_SCAN_*` family locates the densest liquid/torch/self-lit 40x40
/// windows, for deterministic screenshots.
pub(crate) fn log_world_diagnostics(world: &World) {
    if crate::env::dump_tiles() {
        let cx = crate::env::parsed_raw("TR_DUMP_X").unwrap_or(world.width / 2);
        let cy = crate::env::parsed_raw("TR_DUMP_Y").unwrap_or(world.spawn_y + 12);
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
    if crate::env::scan_water() {
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
    if crate::env::scan_torch() {
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
    if crate::env::scan_light() {
        const STEP: i32 = 40;
        let bounds = crate::env::scan_y().unwrap_or((0, world.height));
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
    if crate::env::check_frames() {
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

    if let Some(path) = crate::env::atlas_dump() {
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

/// The production boot world (BUG-05): the player's own most recently
/// modified `.trwld` in the saves dir — the world the lobby's `WorldPlay`
/// would open anyway (`UIWorldSelect` lists them, `Main.LoadWorld` boots the
/// selected one). `None` when the player has no saved world yet, and
/// [`terrstaria_world`] takes over as the out-of-the-box stand-in.
///
/// The vanilla has no boot world at all (`Main.ActiveWorldFileData` is null
/// until `PlayGame`); the port's `App::new` needs one, so the newest own
/// world is the closest shape. The pre-baked `Terrustaria` world (seed 1 or
/// `TR_SEED`) therefore stops being the production path and stays an explicit
/// debug/demo one, alongside `TR_WORLD`.
pub(crate) fn latest_own_world() -> Option<PathBuf> {
    let dir = save::default_world_dir()?;
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_trwld = path.extension().is_some_and(|e| e == "trwld") && path.is_file();
        if !is_trwld {
            continue;
        }
        let Ok(when) = entry.metadata().and_then(|m| m.modified()) else {
            continue;
        };
        if newest.as_ref().is_none_or(|(best, _)| when > *best) {
            newest = Some((when, path));
        }
    }
    newest.map(|(_, path)| path)
}

pub(crate) fn terrstaria_world() -> Result<PathBuf> {
    let path = save::default_world_path("Terrustaria")
        .context("cannot resolve Documents/My Games/Terrustaria")?;
    let regen = crate::env::regen();
    if !path.exists() || regen {
        let seed = crate::env::seed().unwrap_or(1);
        let t = Instant::now();
        // `GenOptions::default()` carries no seed flags — no remix table.
        let mut chest_stock = crate::chest_gen::chest_stock_hook(seed, false);
        let world = worldgen::generate_with_chest_stock(
            &worldgen::GenOptions {
                seed,
                ..Default::default()
            },
            Some(&mut chest_stock),
        );
        // The gen-time chest loot rides the save's native v7 section (S1-A-4).
        let side = save::WorldSide {
            chests: world.chests.clone(),
            ..Default::default()
        };
        save::save(&path, &world, &side)?;
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

/// `SpawnCtx` from the pieces a world-sim stage holds at hand (the C# reads
/// the `Main` statics directly; the port's stages keep disjoint field
/// borrows, so the front-state/pause flags, the camera and the quality cap
/// arrive separately — `App::spawn_ctx` is the whole-App convenience).
/// `screen` is the UI size the C# cull rect uses verbatim (`Dust.cs:97-101`)
/// and `camera` the view's world-px centre.
pub(crate) fn spawn_ctx_from(
    game_menu: bool,
    game_paused: bool,
    camera: Vec2,
    screen: Vec2,
    zoom: f32,
    max_dust_to_draw: i32,
) -> tr_core::particles::SpawnCtx {
    tr_core::particles::SpawnCtx {
        game_menu,
        game_paused,
        generating: false,
        screen_position: camera - screen * 0.5 / zoom,
        screen_size: screen,
        max_dust_to_draw,
    }
}

#[cfg(test)]
pub(crate) mod tr_hooks {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// Serializes env-var mutation across the binary's parallel test threads:
    /// `load_world`'s pins are process-global reads, so any test that sets or
    /// removes a `TR_*` variable must hold this lock for the whole call.
    pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Runs `f` with `name` set to `value`, then removes it — the `TR_PLACE`/
    /// `TR_TOGGLE`/`TR_TILES` parsers moved into `crate::env` as env-reading
    /// accessors, so these tests drive the same read the load path does
    /// (`ENV_LOCK` above serializes the mutation).
    fn with_pin<T>(name: &str, value: &str, f: impl FnOnce() -> T) -> T {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Edition 2024: env mutation is `unsafe` (a parallel reader could
        // observe a half-update); the ENV_LOCK above is that serialization.
        unsafe { std::env::set_var(name, value) };
        let out = f();
        unsafe { std::env::remove_var(name) };
        out
    }

    /// BUG-21 end to end: the select-world Play enters the world (the lobby does
    /// not come back) and the second swap reuses the cached atlas/font halves
    /// instead of re-decoding them.
    ///
    /// The real `App::new` + `swap_world` path, headless (no window/GPU): the
    /// first swap pays the boot decode, the second must be markedly cheaper and
    /// leave the same atlas.
    #[test]
    fn bug21_swap_world_enters_the_world_and_reuses_the_caches() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let Some(content) = find_content_dir() else {
            eprintln!("no assets/Content junction: skipping the swap e2e");
            return;
        };
        assets::set_content_dir(content.clone());
        let assets_dir = content.join("Images");

        // Two scratch worlds through the real save path.
        let dir = std::env::temp_dir().join(format!("tr-bug21-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut paths = Vec::new();
        for name in ["Alpha", "Beta"] {
            let path = dir.join(format!("{name}.trwld"));
            let mut w = World::new(name.into(), 64, 64);
            w.compute_column_surface();
            save::save(&path, &w, &save::WorldSide::default()).unwrap();
            paths.push(path);
        }

        // The boot App over the first world, with the boot caches filled exactly
        // like `main` does.
        let (world, sections) = load_world(&paths[0]).unwrap();
        let mut boot_static = None;
        let mut boot_fonts = None;
        let (atlas, font_atlas, lookups, stats) =
            build_atlas_reusing_static(&world, &assets_dir, &mut boot_static, &mut boot_fonts)
                .expect("boot atlas");
        let mut app = App::new(
            settings::Settings::default(),
            world,
            sections,
            paths[0].clone(),
            atlas,
            font_atlas,
            lookups,
            stats,
        );
        app.static_sheets = boot_static;
        app.fonts_cache = boot_fonts;
        // The boot default: the legacy menu is open (the lobby).
        assert!(app.menu.is_some(), "the boot App opens the lobby");

        // The Play: the world swaps in and the lobby must not come back.
        app.swap_world(&paths[1]).expect("first swap");
        assert!(app.menu.is_none(), "the lobby must not come back");
        assert!(app.front.is_none(), "the 888 side clears too");
        assert_eq!(app.world.name, "Beta", "the selected world is live");
        assert!(app.static_sheets.is_some(), "the static half is cached");
        assert!(app.fonts_cache.is_some(), "the font tables are cached");
        assert!(
            app.font_atlas.is_some(),
            "the font page texture rides the swap"
        );

        // The second Play reuses both caches (the decode half is pinned by
        // `assets::tests::test_the_second_atlas_build_reuses_the_static_half`,
        // which measures the atlas build alone; a wall-clock assertion here
        // would flake on the rest of `App::new`).
        app.swap_world(&paths[0]).expect("second swap");
        assert_eq!(app.world.name, "Alpha");
        assert!(app.menu.is_none() && app.front.is_none());
        assert!(app.static_sheets.is_some() && app.fonts_cache.is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The production boot path (BUG-05): the player's own newest `.trwld`,
    /// not the pre-baked `Terrustaria` stand-in. The mtime read is the vanilla
    /// `UIWorldSelect`'s list shape — the lobby opens what `WorldPlay` picks.
    #[test]
    fn the_boot_world_is_the_players_newest_save() {
        let dir = std::env::temp_dir().join(format!("tr-boot-world-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let worlds = dir.join("Documents/My Games/Terrustaria/Worlds");
        std::fs::create_dir_all(&worlds).unwrap();
        // Two saves; the second is written last so its mtime is the newest.
        for name in ["Alpha", "Beta"] {
            let path = worlds.join(format!("{name}.trwld"));
            std::fs::write(&path, &[b'T', b'R', b'W', b'L', b'D', b'0', b'1', 0]).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let picked = with_pin("USERPROFILE", dir.to_str().unwrap(), latest_own_world);
        assert_eq!(
            picked.as_deref(),
            Some(worlds.join("Beta.trwld").as_path()),
            "the newest own world boots"
        );
        // An empty saves dir has no own world: the port's stand-in takes over
        // (the documented debug/demo path).
        let empty = std::env::temp_dir().join(format!("tr-boot-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();
        assert!(with_pin("USERPROFILE", empty.to_str().unwrap(), latest_own_world).is_none());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&empty);
    }

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
        let ops = with_pin("TR_PLACE", "10,19,18;14,19,21,0,-1", || {
            crate::env::place_ops()
        });
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
        assert_eq!(
            with_pin("TR_PLACE", "x,y,18;5,19,4", || crate::env::place_ops()).len(),
            1
        );
        assert!(with_pin("TR_PLACE", "", || crate::env::place_ops()).is_empty());
    }

    #[test]
    fn tr_toggle_parsing_default_count() {
        let ops = with_pin("TR_TOGGLE", "2026,198;7,9,3", || crate::env::toggle_ops());
        assert_eq!(ops.len(), 2);
        assert_eq!((ops[0].x, ops[0].y, ops[0].n), (2026, 198, 1));
        assert_eq!((ops[1].x, ops[1].y, ops[1].n), (7, 9, 3));
        // Wrong arity is dropped; a negative count clamps to 0 presses.
        assert!(with_pin("TR_TOGGLE", "1,2,3,4", || crate::env::toggle_ops()).is_empty());
        assert_eq!(
            with_pin("TR_TOGGLE", "1,2,-5", || crate::env::toggle_ops())[0].n,
            0
        );
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
        apply_tr_places(
            &mut w,
            &with_pin("TR_PLACE", "10,19,10", crate::env::place_ops),
        );
        assert!(w.tile(10, 19).unwrap().active() && w.tile(10, 17).unwrap().active());
        // `place` guarantees the frame_important bit (Decision #4).
        assert!(w.frame_important[10]);
        let closed_frame = w.tile(10, 18).unwrap().frame_y;
        apply_tr_toggles(
            &mut w,
            &with_pin("TR_TOGGLE", "10,18,1", crate::env::toggle_ops),
        );
        assert!(
            w.tile(10, 18).unwrap().active() && w.tile(10, 18).unwrap().kind == 11,
            "one press opens the door"
        );
        assert!(w.frame_important[11], "open_door keeps the tile-11 bit");
        apply_tr_toggles(
            &mut w,
            &with_pin("TR_TOGGLE", "10,18,1", crate::env::toggle_ops),
        );
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
        let ops = with_pin("TR_TILES", "5,6,82,0,18;7,8,82,18,-1", || {
            crate::env::tile_ops()
        });
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
        let clear = with_pin("TR_TILES", "10,10,3,2", crate::env::tile_ops);
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
        let mixed = with_pin("TR_TILES", "5,6,82,0,18;0,0,3,2", crate::env::tile_ops);
        assert_eq!(mixed.len(), 2);
        assert_eq!(mixed[0].kind, 82);
        assert_eq!(mixed[1].kind, -1);
    }

    #[test]
    fn tr_tiles_parsing_skips_malformed_groups() {
        // The fixture is one env value; a malformed group must be dropped whole
        // without aborting the well-formed ones around it.
        assert!(
            with_pin("TR_TILES", "", crate::env::tile_ops).is_empty(),
            "the empty string places nothing"
        );
        assert!(
            with_pin("TR_TILES", "1,2", crate::env::tile_ops).is_empty(),
            "2 tokens is not a form"
        );
        assert!(
            with_pin("TR_TILES", "1,2,3", crate::env::tile_ops).is_empty(),
            "3 tokens is not a form"
        );
        assert!(
            with_pin("TR_TILES", "1,2,3,4,5,6", crate::env::tile_ops).is_empty(),
            "6 tokens is not a form"
        );
        assert!(
            with_pin("TR_TILES", "x,y,82,0,18", crate::env::tile_ops).is_empty(),
            "non-numeric coords"
        );
        assert!(
            with_pin("TR_TILES", "1,2,82,0,z", crate::env::tile_ops).is_empty(),
            "non-numeric frames"
        );
        assert_eq!(
            with_pin("TR_TILES", "1,2,3;5,6,82,0,18", crate::env::tile_ops).len(),
            1
        );
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
        apply_tr_tiles(
            &mut w,
            &with_pin("TR_TILES", "5,6,82,0,18", crate::env::tile_ops),
        );
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
        apply_tr_tiles(
            &mut w,
            &with_pin("TR_TILES", "10,10,3,2", crate::env::tile_ops),
        );
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
        apply_tr_tiles(
            &mut w,
            &with_pin("TR_TILES", "1000,1000,4,4;-5,-5,4,4", crate::env::tile_ops),
        );
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
        let dir = crate::test_world_helpers::test_scratch("dw-1-6-pins");
        let path = dir.join("tr-dw-1-6-pins.trwld");
        save::save(
            &path,
            &World::new("pins".into(), 32, 32),
            &save::WorldSide::default(),
        )
        .unwrap();

        // Edition 2024: env mutation is `unsafe` (a parallel reader could
        // observe a half-update); the ENV_LOCK above is that serialization.
        unsafe { std::env::set_var("TR_SUNDIAL", "1") };
        unsafe { std::env::set_var("TR_MOONDIAL", "1") };
        let (world, sections) = load_world(&path).expect("scratch world loads");
        unsafe { std::env::remove_var("TR_SUNDIAL") };
        unsafe { std::env::remove_var("TR_MOONDIAL") };
        crate::test_world_helpers::remove_scratch(&dir);

        assert!(
            world.fast_forward_to_dawn,
            "TR_SUNDIAL arms the dawn fast-forward"
        );
        assert!(
            world.fast_forward_to_dusk,
            "TR_MOONDIAL arms the dusk fast-forward"
        );
        assert!(
            sections.native.chests.is_empty(),
            "the scratch world has no chest records"
        );
        assert!(
            sections.native.banners.is_empty(),
            "the scratch world has no banner records"
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

        let dir = crate::test_world_helpers::test_scratch("dw-1-6-clear");
        let path = dir.join("tr-dw-1-6-clear.trwld");
        save::save(
            &path,
            &World::new("pins".into(), 32, 32),
            &save::WorldSide::default(),
        )
        .unwrap();
        let (world, sections) = load_world(&path).expect("scratch world loads");
        crate::test_world_helpers::remove_scratch(&dir);

        assert!(!world.fast_forward_to_dawn && !world.fast_forward_to_dusk);
        assert_eq!((world.sundial_cooldown, world.moondial_cooldown), (0, 0));
    }
}

/// `GetNurseHealCost`'s boss-progression ladder inputs (`Main.cs:40069-40088`
/// + the discount `:40094-40096`) — the World's downed flags and the
/// Discount Card's `discountAvailable` (R-03-5's field). The expert x2
/// (`:40091-40093`) sits out with the difficulty plumbing (R-03-6).
pub(crate) struct NurseCostFlags {
    pub downed_golem: bool,
    pub downed_plant_boss: bool,
    pub downed_mech_any: bool,
    pub hard_mode: bool,
    pub downed_boss3_or_queen_bee: bool,
    pub downed_boss2: bool,
    pub downed_boss1: bool,
    pub discount_available: bool,
}

/// `GetNurseHealCost`'s core arithmetic (`Main.cs:40054-40100`): the missing
/// life plus the removable debuffs x100, the if/else-if ladder (the
/// Golem 200 ×, the Plant 150 ×, the mechs 100 ×, hardmode 60 ×, the
/// Skeletron/Queen-Bee pair 25 ×, boss2 10 ×, boss1 3 ×), the Discount
/// Card's `(int)((float)num * 0.8f)` truncation, then the happiness
/// adjustment.
pub(crate) fn nurse_heal_cost_for(
    missing: i32,
    debuff_count: i32,
    mood: f32,
    flags: &NurseCostFlags,
) -> i32 {
    let mut num = missing + debuff_count * 100;
    if flags.downed_golem {
        num *= 200;
    } else if flags.downed_plant_boss {
        num *= 150;
    } else if flags.downed_mech_any {
        num *= 100;
    } else if flags.hard_mode {
        num *= 60;
    } else if flags.downed_boss3_or_queen_bee {
        num *= 25;
    } else if flags.downed_boss2 {
        num *= 10;
    } else if flags.downed_boss1 {
        num *= 3;
    }
    if flags.discount_available {
        num = (num as f32 * 0.8f32) as i32;
    }
    (num as f32 * mood) as i32
}

#[cfg(test)]
mod s1_g_7_tests {
    use super::*;

    #[test]
    fn forced_minimum_zoom_matches_the_c_sharp_computation() {
        // Main.cs:61989-61994: `val = screenWidth / MaxWorldViewSize.X`,
        // `ForcedMinimumZoom = max(max(1f, val), screenHeight /
        // MaxWorldViewSize.Y)` — the floor keeps the visible world inside the
        // 1920x1200 design size (Main.cs:677-681). At or below it the floor
        // is 1; either arm alone lifts it.
        assert_eq!(App::forced_minimum_zoom((1920.0, 1200.0)), 1.0);
        assert_eq!(App::forced_minimum_zoom((960.0, 600.0)), 1.0);
        assert!(
            (App::forced_minimum_zoom((2560.0, 1200.0)) - 2560.0 / 1920.0).abs() < f32::EPSILON
        );
        assert!(
            (App::forced_minimum_zoom((1920.0, 1440.0)) - 1440.0 / 1200.0).abs() < f32::EPSILON
        );
        assert!((App::forced_minimum_zoom((3840.0, 2400.0)) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn support_wide_screen_is_the_c_sharp_init_default() {
        // Main.cs:67046 (`Main.SupportWideScreen = false`): the port carries
        // no settings row for it, so the computed arm runs and the toggle
        // stays unported (S1-G-7). The arm itself: `Main.SupportWideScreen`
        // forces the floor to 1f (`:61992-61994`).
        assert!(!App::SUPPORT_WIDE_SCREEN);
    }
}

#[cfg(test)]
mod s1_e_18_tests {
    use super::*;

    /// `NPCChatText_DoNurseHeal` (`Main.cs:40104-40111`): inside the
    /// `BuyItem(healCost)` success arm, `AchievementsHelper.HandleNurseService`
    /// adds the paid copper to FREQUENT_FLYER's `Pay` float
    /// (`AchievementsHelper.cs:407-410`). A purse too short never reaches it.
    #[test]
    fn the_nurse_heal_credits_the_paid_copper_to_frequent_flyer() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.hp = player.max_hp / 2;
        app.player = Some(player);
        app.npcs.push(crate::npc::Npc::new(
            crate::npc::NpcKind::Nurse,
            32.0,
            100.0,
            1,
            0.0,
        ));
        let cost = app.nurse_heal_cost(0);
        assert!(cost > 0, "half life is a heal");
        // Shy of the price: the C# returns before `HandleNurseService`.
        app.coins = cost as u64 - 1;
        assert!(!app.nurse_heal(0));
        assert_eq!(pay(&app), 0);
        // The paid heal credits the exact copper and tops the life up.
        app.coins = cost as u64;
        assert!(app.nurse_heal(0));
        assert_eq!(pay(&app), cost);
        assert_eq!(
            app.player.as_ref().unwrap().hp,
            app.player.as_ref().unwrap().max_hp
        );
        assert_eq!(app.coins, 0);
    }

    /// FREQUENT_FLYER's `Pay` tracker value (`AchievementsHelper.cs:408`).
    fn pay(app: &App) -> i32 {
        app.achievements
            .list
            .iter()
            .find(|a| a.name == "FREQUENT_FLYER")
            .and_then(|a| a.tracker_values())
            .map_or(0, |(v, _)| v)
    }
}

#[cfg(test)]
mod dye_pin_tests {
    use super::*;

    /// DW-2.3: the `TR_DYE`/`TR_DMOUNT` pin parses — positional fills,
    /// garbage and empty entries stay empty slots.
    #[test]
    fn tr_dye_pin_fills_positionally_and_forgives_garbage() {
        let dyes = crate::env::parse_tr_dye("1007, 1015 ,, 1023");
        assert_eq!(dyes[0], 1007);
        assert_eq!(dyes[1], 1015);
        assert_eq!(dyes[2], 0, "empty entry leaves the slot empty");
        assert_eq!(dyes[3], 1023);
        assert_eq!(dyes[4..], [0; 6]);
        assert_eq!(crate::env::parse_tr_dye(""), [0; 10]);
        assert_eq!(crate::env::parse_tr_dye("nope,x")[..2], [0, 0]);
        // Overlong lists clip to the ten dye slots.
        assert_eq!(
            crate::env::parse_tr_dye("1,2,3,4,5,6,7,8,9,10,11,12")[9],
            10
        );
    }

    #[test]
    fn tr_dmount_pin_parses_one_kind() {
        assert_eq!(crate::env::parse_tr_dmount("1007"), 1007);
        assert_eq!(crate::env::parse_tr_dmount(" 2871 "), 2871);
        assert_eq!(crate::env::parse_tr_dmount("nope"), 0);
        assert_eq!(crate::env::parse_tr_dmount(""), 0);
    }
}
/// S1-P-i44 — the shop TRANSACTION witnesses: the buy lands on the CURSOR
/// (`ItemSlot.HandleShopSlot`, `ItemSlot.cs:2053-2095`), the sell pays the
/// purse and re-stocks the sold units (`Player.SellItem` +
/// `Chest.AddItemToShop`), the sellback memo refunds the bought units at
/// their buy price, and the close clears the memo
/// (`Main.UpdateOldNPCShop`, `Main.cs:18204-18211`). Written, not run.
#[cfg(test)]
mod s1_p_i44_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// An open Merchant shop with the base stock, mood 1 and a funded
    /// purse. The stock resolves through the same seam the open uses.
    fn shop_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.shop_npc = Some(0);
        app.shop_stock =
            crate::shop::resolve_stock_entries(1, &crate::shop::ShopFlags::default(), &[]);
        app.shop_mood = 1.0;
        app.coins = 1_000_000;
        app
    }

    fn held(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count,
            prefix: 0,
        }
    }

    #[test]
    fn the_buy_charges_the_purse_and_lands_on_the_cursor() {
        // `ItemSlot.cs:2059-2088`: the Mining Helmet 88 costs its 40000
        // store value at mood 1; the unit rides `Main.mouseItem`, NOT the
        // inventory grid; the sellback memo records the purchase.
        let mut app = shop_app();
        assert_eq!(app.shop_stock[0].item, 88);
        assert!(app.buy_shop_item(0));
        assert_eq!(app.coins, 1_000_000 - 40000);
        assert_eq!((app.mouse_item.kind, app.mouse_item.count), (88, 1));
        assert!(app.inventory.iter().all(|s| s.count == 0), "no grid deposit");
        assert_eq!(app.shop_sellback.get_amount(88, 0), 1);
        // A second unit stacks on the cursor and the memo.
        assert!(app.buy_shop_item(0));
        assert_eq!(app.mouse_item.count, 2);
        assert_eq!(app.shop_sellback.get_amount(88, 0), 2);
        // The plain stock row never depletes (`:2090` gates on buyOnce).
        assert_eq!(app.shop_stock[0].stack, 1);
    }

    #[test]
    fn the_short_purse_refuses_the_buy() {
        // `Player.BuyItem`'s gate (`Player.cs:35411-35414`): nothing moves.
        let mut app = shop_app();
        app.coins = 39_999;
        assert!(!app.buy_shop_item(0));
        assert_eq!(app.coins, 39_999);
        assert_eq!(app.mouse_item.count, 0);
        assert_eq!(app.shop_sellback.get_amount(88, 0), 0);
    }

    #[test]
    fn the_happiness_and_discount_ride_the_buy_charge() {
        // `Player.cs:35387` — the mood multiplies the buy; `:35373` — the
        // Discount Card truncates ×0.8 first.
        let mut app = shop_app();
        app.shop_mood = 0.75;
        assert!(app.buy_shop_item(0));
        assert_eq!(app.coins, 1_000_000 - 30000);
        let mut app = shop_app();
        app.discount_available = true;
        assert!(app.buy_shop_item(0));
        assert_eq!(app.coins, 1_000_000 - 32000);
    }

    #[test]
    fn the_buy_once_row_depletes_and_turns_to_air() {
        // `ItemSlot.cs:2090-2093`: a buyOnce row loses one unit per buy and
        // `SetDefaults(0)` at zero; the memo never records it (`:2084`).
        let mut app = shop_app();
        app.shop_stock.push(crate::shop::ShopEntry {
            item: 9,
            when: crate::shop::StockWhen::Always,
            shop_custom_price: None,
            shop_special_currency: -1,
            buy_once: true,
            is_a_shop_item: false,
            prefix: 0,
            stack: 2,
        });
        let slot = app.shop_stock.len() - 1;
        assert!(app.buy_shop_item(slot));
        assert_eq!(app.shop_stock[slot].stack, 1);
        assert_eq!(app.shop_sellback.get_amount(9, 0), 0, "no memo on buyOnce");
        assert!(app.buy_shop_item(slot));
        assert_eq!(app.shop_stock[slot].item, 0, "the depleted row is air");
        assert_eq!(app.shop_stock[slot].stack, 0);
        assert!(!app.buy_shop_item(slot), "an air row sells nothing");
    }

    #[test]
    fn the_sell_pays_the_purse_at_a_fifth() {
        // `Player.cs:35184-35190`: the Torch 8 (value 50) sells 50/5 = 10
        // per unit × the stack; a value-0 item refuses.
        let mut app = shop_app();
        let payout = app.sell_item(&held(8, 7));
        assert_eq!(payout, 70);
        assert_eq!(app.coins, 1_000_000 + 70);
        // The value-0 refusal (`:35170-35174`) pays nothing.
        let before = app.coins;
        assert_eq!(app.sell_item(&held(9, 5)), 0);
        assert_eq!(app.coins, before);
    }

    #[test]
    fn the_sellback_refund_pays_the_buy_price_back() {
        // The round trip: buy 3 helmets (40000 each), sell all 3 — the
        // memo refunds every unit at its buy price (`Player.cs:35191-35195`),
        // so the purse returns to its start and the shop gains NO row
        // (`Chest.cs:672-676`).
        let mut app = shop_app();
        for _ in 0..3 {
            assert!(app.buy_shop_item(0));
        }
        assert_eq!(app.coins, 1_000_000 - 120000);
        let stock_len = app.shop_stock.len();
        let item = app.mouse_item;
        let payout = app.sell_item(&item);
        assert_eq!(payout, 120000, "3 × the 40000 buy price");
        assert_eq!(app.coins, 1_000_000);
        crate::shop::add_item_to_shop(&mut app.shop_stock, &mut app.shop_sellback, &item);
        assert_eq!(app.shop_stock.len(), stock_len, "bought goods never re-stock");
        assert_eq!(app.shop_sellback.get_amount(88, 0), 0);
    }

    #[test]
    fn the_sell_prices_the_prefixed_instance() {
        // `Item.cs:614-615`'s ×value²: the Large Mining Helmet (prefix 1)
        // is worth 50176, selling 10035 (`shop.rs`'s seed vector).
        let mut app = shop_app();
        let item = ItemSlot {
            prefix: 1,
            ..held(88, 1)
        };
        assert_eq!(app.sell_item(&item), 10035);
    }

    #[test]
    fn the_close_clears_the_stock_and_the_memo() {
        // `Main.UpdateOldNPCShop` (`Main.cs:18204-18211`): the index change
        // clears the sellback helper; `close_shop` is the port's index-0.
        let mut app = shop_app();
        assert!(app.buy_shop_item(0));
        assert!(app.shop_sellback.get_amount(88, 0) > 0);
        app.close_shop();
        assert!(app.shop_npc.is_none());
        assert!(app.shop_stock.is_empty());
        assert_eq!(app.shop_sellback.get_amount(88, 0), 0);
        assert_eq!(app.shop_mood, 1.0);
    }

    #[test]
    fn the_inventory_close_closes_the_shop() {
        // `Main.cs:41784-41787`: `npcShop > 0 && !playerInventory` →
        // `SetNPCShopIndex(0)`.
        let mut app = shop_app();
        app.update_shop_close();
        assert!(app.shop_npc.is_some(), "the open inventory keeps the shop");
        app.inventory_open = false;
        app.update_shop_close();
        assert!(app.shop_npc.is_none());
        assert!(app.shop_stock.is_empty());
    }
}
