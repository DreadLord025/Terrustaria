//! The bestiary screen (`UIBestiaryTest.cs`, opened through
//! `Main.DrawBestiaryIcon`'s inventory button) — the entry grid
//! (`UIBestiaryEntryGrid`/`UIBestiaryEntryButton`), the per-entry info page
//! (`UIBestiaryEntryInfoPage`'s stat rows) and the unlock-tier gating
//! (`CommonEnemyUICollectionInfoProvider` via `tr_core::bestiary`).
//!
//! The geometry constants follow the source: the 72 px cells
//! (`UIBestiaryEntryButton.cs:27-28`), the 72 px grid division
//! (`UIBestiaryEntryGrid.cs:95-100`), the inventory button's
//! `(498, 278) 30x30` hover rect (`Main.cs:42442`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use tr_core::bestiary::{KillTracker, UnlockState};

use super::push_whole;
use super::text;
use crate::assets::{self, Lookups};
use crate::npc::{NpcKind, SlimeVariant};

/// `UIBestiaryEntryButton`'s 72 px cell (`:27-28`).
pub const CELL: f32 = 72.0;

/// The inventory's emote button — `DrawEmoteBubblesButton`'s rect
/// (`Main.cs:42369-42408`: num=534, num2=244+30+4=278, the 30x30 box).
pub const EMOTE_BUTTON_RECT: [f32; 4] = [534.0, 278.0, 30.0, 30.0];

/// `true` while the cursor is over the emote button (`:42397`'s inclusive
/// `rectangle.Contains`).
pub fn emote_button_hit(mouse: Vec2) -> bool {
    mouse.x >= EMOTE_BUTTON_RECT[0]
        && mouse.x <= EMOTE_BUTTON_RECT[0] + EMOTE_BUTTON_RECT[2]
        && mouse.y >= EMOTE_BUTTON_RECT[1]
        && mouse.y <= EMOTE_BUTTON_RECT[1] + EMOTE_BUTTON_RECT[3]
}

/// The inventory button's hover rect — `DrawBestiaryIcon`'s
/// `rectangle = new Rectangle(num, num2, 30, 30)` with the sheet-position
/// constants resolved (`Main.cs:42438-42442`: num=498, num2=244+30+4=278).
pub const BUTTON_RECT: [f32; 4] = [498.0, 278.0, 30.0, 30.0];

/// The dark backdrop behind the panel (`UIBestiaryTest`'s state draw scales
/// the game down behind `Main.DrawBackground`; the port dims instead — the
/// shared overlay treatment of the other full-screen UI).
const BACKDROP: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// `BuildPage`'s outer container: width 0.875 (max 900 px), top 180,
/// height -220 (`UIBestiaryTest.cs:71-79`).
pub fn panel_rect(screen: Vec2) -> [f32; 4] {
    let w = ((screen.x * 0.875).min(900.0)).max(560.0);
    [screen.x * 0.5 - w * 0.5, 180.0, w, screen.y - 220.0 - 180.0]
}

/// The `Back` panel — `MakeExitButton` (`:364-380`): 50 px tall, bottom
/// -25, centred, half the container's width (-10).
pub fn back_rect(panel: &[f32; 4]) -> [f32; 4] {
    let w = (panel[2] - 10.0) * 0.5;
    [
        panel[0] + panel[2] * 0.5 - w * 0.5,
        panel[1] + panel[3] - 25.0 - 50.0,
        w,
        50.0,
    ]
}

/// The grid area — the `-12px wide, -4px tall` bestiary space above the
/// bottom bar (`:139-146`).
fn grid_rect(panel: &[f32; 4]) -> [f32; 4] {
    [
        panel[0] + 6.0,
        panel[1] + 40.0,
        panel[2] - 12.0,
        panel[3] - 40.0 - 4.0,
    ]
}

/// `UIBestiaryEntryGrid.GetEntriesToShow` (`:95-100`): the cells per
/// row/column are the space divided by 72.
fn grid_metrics(rect: &[f32; 4]) -> (usize, usize) {
    let cols = ((rect[2] / CELL) as usize).max(1);
    let rows = ((rect[3] / CELL) as usize).max(1);
    // The rows grow with the entry count — the vanilla's bestiary scrolls
    // (`UIBestiaryGrid`'s scroll bars land with the UI-scroll work); the
    // fixed rows clipped the roster at the town-NPC growth.
    let need = (crate::bestiary::ENTRIES.len() + cols - 1) / cols;
    (cols, rows.max(need))
}

/// The entry index under the cursor (the grid's per-cell centre placement
/// simplifies to a fixed lattice — the cards are `HAlign/VAlign = 0.5` with
/// the fractional `Left/Top` offsets `:104-108` produce).
pub fn card_at(screen: Vec2, mouse: Vec2) -> Option<usize> {
    let panel = panel_rect(screen);
    let rect = grid_rect(&panel);
    let (cols, rows) = grid_metrics(&rect);
    // `GetEntriesToShow`'s raw CELL lattice (the space divided by 72 — no
    // insets): the draw and the hit-test share it, so the overflow rows
    // keep the hits.
    let cx = ((mouse.x - rect[0]) / CELL) as usize;
    let cy = ((mouse.y - rect[1]) / CELL) as usize;
    if cx >= cols || cy >= rows {
        return None;
    }
    let idx = cy * cols + cx;
    if idx < crate::bestiary::ENTRIES.len() {
        Some(idx)
    } else {
        None
    }
}

/// `true` while the cursor is over the Back panel (`MakeExitButton`'s
/// `OnLeftMouseDown` -> `Click_GoBack`).
pub fn back_hit(screen: Vec2, mouse: Vec2) -> bool {
    let panel = panel_rect(screen);
    let r = back_rect(&panel);
    mouse.x >= r[0] && mouse.x <= r[0] + r[2] && mouse.y >= r[1] && mouse.y <= r[1] + r[3]
}

/// `true` while the cursor is over the inventory's bestiary button
/// (`Main.cs:42441`'s `rectangle.Contains(mouse)` — inclusive edges).
pub fn button_hit(mouse: Vec2) -> bool {
    mouse.x >= BUTTON_RECT[0]
        && mouse.x <= BUTTON_RECT[0] + BUTTON_RECT[2]
        && mouse.y >= BUTTON_RECT[1]
        && mouse.y <= BUTTON_RECT[1] + BUTTON_RECT[3]
}

/// `DrawBestiaryIcon` (`Main.cs:42423-42485`): the 2-frame `Images/UI/Bestiary`
/// sheet drawn frame-hover, inset 2 px per side, centred in the 30x30 rect.
pub fn draw_inventory_button(r: &mut SpriteRenderer, lookups: &Lookups, mouse: Vec2) {
    let Some(sheet) = lookups.ui_bestiary_button else {
        return;
    };
    // `value.Frame(2, 1, flag ? 1 : 0)`, `rectangle2.Width -= 2` (`:42464-42467`).
    let frame_w = (sheet.width / 2) as f32;
    let frame_h = sheet.height as f32;
    let cx = BUTTON_RECT[0] + BUTTON_RECT[2] * 0.5;
    let cy = BUTTON_RECT[1] + BUTTON_RECT[3] * 0.5;
    let dst = [
        cx - (frame_w - 2.0) * 0.5,
        cy - (frame_h - 2.0) * 0.5,
        frame_w - 2.0,
        frame_h - 2.0,
    ];
    draw_sheet_frame(
        r,
        lookups,
        &sheet,
        button_hit(mouse) as usize,
        dst,
        [1.0; 4],
    );
}

/// The whole screen: the dim backdrop, the container panel, the title, the
/// grid of entry cards (or the selected entry's info page), the completion
/// bar and the Back panel.
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    selected: Option<usize>,
    mouse: Vec2,
) {
    // The state's background — the game keeps rendering behind
    // (`IngameFancyUI`), dimmed the way the port's other overlays are.
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let _ = (aw, ah);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);
    draw_panel_chrome(r, lookups, screen);

    match selected {
        Some(idx) => draw_info_page(r, lookups, screen, tracker, idx, mouse),
        None => draw_grid(r, lookups, screen, tracker, mouse),
    }

    // The completion bar — `FillProgressBottomBar` + `GetCompletionPercentText`
    // (`:390-430`): the fully-unlocked share of the entries.
    let panel = panel_rect(screen);
    let total = crate::bestiary::ENTRIES.len() as f32;
    let done = crate::bestiary::ENTRIES
        .iter()
        .filter(|e| {
            crate::bestiary::unlock_state(e, tracker)
                == tr_core::bestiary::UnlockState::CanShowDropsWithDropRates
        })
        .count() as f32;
    let pct = if total > 0.0 {
        done / total * 100.0
    } else {
        0.0
    };
    text::hud_text(
        r,
        lookups,
        panel[0] + 8.0,
        panel[1] + panel[3] - 46.0,
        0.8,
        [1.0, 1.0, 1.0, 1.0],
        &format!("Unlocked: {:.0}%", pct),
    );

    // `MakeExitButton` (`:364-380`): the 0.7-large Back panel.
    let r0 = back_rect(&panel);
    let style = super::cards::fat_button_style();
    let text_w = text::hud_text_width(r, lookups, "Back", style.text_scale);
    super::uistate::draw_text_panel(
        r,
        lookups,
        r0,
        &style,
        "Back",
        text_w,
        back_hit(screen, mouse),
    );
}

fn draw_panel_chrome(r: &mut SpriteRenderer, lookups: &Lookups, screen: Vec2) {
    // The panel body: `UIPanel`'s `BackgroundColor = (33, 43, 79) * 0.8`
    // (`UIBestiaryTest.cs:84-88`), drawn as a flat quad (the port's panel
    // style for the full-screen states).
    let p = panel_rect(screen);
    r.push_ui_sprite(
        [p[0], p[1]],
        [p[2], p[3]],
        r.white_uv(),
        [
            33.0 / 255.0 * 0.8,
            43.0 / 255.0 * 0.8,
            79.0 / 255.0 * 0.8,
            0.8,
        ],
    );
    // The header text — the screen has no title element; the port labels the
    // panel with the inventory button's tooltip name.
    text::hud_text(
        r,
        lookups,
        p[0] + 12.0,
        p[1] + 10.0,
        1.0,
        [1.0; 4],
        "Bestiary",
    );
}

fn draw_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    mouse: Vec2,
) {
    let panel = panel_rect(screen);
    let rect = grid_rect(&panel);
    let (cols, rows) = grid_metrics(&rect);
    for (idx, entry) in crate::bestiary::ENTRIES
        .iter()
        .take(cols * rows)
        .enumerate()
    {
        let col = idx % cols;
        let row = idx / cols;
        let x = rect[0] + col as f32 * CELL;
        let y = rect[1] + row as f32 * CELL;
        let cell = [x, y, CELL, CELL];
        let hover = card_at(screen, mouse) == Some(idx);
        draw_card(r, lookups, cell, entry, tracker, idx, hover);
    }
}

/// One `UIBestiaryEntryButton` (`:20-86`): the inner 68 px box holds
/// `Slot_Back` + the portrait (or the 15 % white `Icon_Locked`), the
/// `Slot_Overlay` at 60 % white and `Slot_Front` stack over it, the
/// `Slot_Selection` frame appears on hover and the display index prints at
/// (10, 10) scale 0.9.
fn draw_card(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    cell: [f32; 4],
    entry: &crate::bestiary::BestiaryEntryDef,
    tracker: &KillTracker,
    idx: usize,
    hover: bool,
) {
    let inner = [cell[0] + 2.0, cell[1] + 2.0, CELL - 4.0, CELL - 4.0];
    let cx = inner[0] + inner[2] * 0.5;
    let cy = inner[1] + inner[3] * 0.5;
    if let Some(sheet) = lookups.ui_bestiary_slot_back {
        push_whole(r, lookups, &sheet, inner, [1.0; 4]);
    }
    let state = crate::bestiary::unlock_state(entry, tracker);
    if state.known() {
        draw_portrait(r, lookups, entry, cx, cy, inner[3]);
    } else if let Some(locked) = lookups.ui_bestiary_locked {
        // `UIBestiaryEntryIcon.DrawSelf`'s locked branch (`:44-56`): the
        // texture at white * 0.15, centred.
        push_whole(r, lookups, &locked, inner, [0.15; 4]);
    }
    if let Some(overlay) = lookups.ui_bestiary_slot_overlay {
        push_whole(r, lookups, &overlay, inner, [0.6; 4]);
    }
    if let Some(front) = lookups.ui_bestiary_slot_front {
        push_whole(r, lookups, &front, inner, [1.0; 4]);
    }
    if hover {
        if let Some(sel) = lookups.ui_bestiary_slot_selection {
            push_whole(r, lookups, &sel, cell, [1.0; 4]);
        }
    }
    // The display index (`:58-66` — `IBestiaryEntryDisplayIndex`'s number).
    text::hud_text(
        r,
        lookups,
        cell[0] + 10.0,
        cell[1] + 10.0,
        0.9,
        [1.0; 4],
        &format!("{}", idx + 1),
    );
}

/// `UnlockableNPCEntryIcon.Draw` (`:194-211`) — the NPC drawn through
/// `Main.DrawNPCDirect` at the icon box: non-floaters bottom-anchored
/// `topLeft + size*(0.5, 1) + (0, -8)` (`UpdatePosition`, `:112-123`),
/// floaters centred. The UI draws with full light; the slime variants keep
/// their `NPC.color` tint.
fn draw_portrait(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    entry: &crate::bestiary::BestiaryEntryDef,
    cx: f32,
    cy: f32,
    box_h: f32,
) {
    let (sheet_i, frame_h, w, h, tint) = portrait_data(entry, lookups);
    let Some(sheet) = lookups.bestiary_npc_sheets[sheet_i] else {
        return;
    };
    let floats = entry.net_id == 2 || entry.net_id == 133; // the demon-eye family noGravity
    let (x, y) = if floats {
        (cx - w * 0.5, cy - h * 0.5)
    } else {
        let top = cy + box_h * 0.5;
        (cx - w * 0.5, top - h - 8.0)
    };
    draw_sheet_frame(r, lookups, &sheet, 0, [x, y, w, h], tint);
}

/// The portrait parameters — the `NpcAssets` table's sizes
/// (`render/entities.rs::draw_npcs`) restated per entry; the slime frames
/// are 32x26 scaled by the variant's size scale and tinted with its
/// `NPC.color`.
#[allow(clippy::type_complexity)]
fn portrait_data(
    entry: &crate::bestiary::BestiaryEntryDef,
    lookups: &Lookups,
) -> (usize, f32, f32, f32, [f32; 4]) {
    // The sheet indexes ride `bestiary_npc_sheets`' order (`assets.rs`) —
    // each kind maps to its `NPC_<net_id>` sheet's position. The parallel
    // merges had let the per-agent indexes collide (the frost trio took
    // 48-50, the boss/leech/goblin/golem/pirate rows drifted); the whole
    // table is re-derived from the vec once.
    let sheet_i = match entry.kind {
        NpcKind::Slime => 0,           // NPC_1
        NpcKind::DemonEye => 1,        // NPC_2
        NpcKind::WanderingEye => 2,    // NPC_133
        NpcKind::Zombie => 3,          // NPC_3
        NpcKind::KingSlime => 4,       // NPC_50
        NpcKind::Guide => 5,           // NPC_22
        NpcKind::Skeleton => 6,        // NPC_21
        NpcKind::UndeadMiner => 7,     // NPC_44
        NpcKind::CaveBat => 8,         // NPC_49
        NpcKind::JungleBat => 9,       // NPC_51
        NpcKind::Hellbat => 10,        // NPC_60
        NpcKind::VoodooDemon => 11,    // NPC_66
        NpcKind::EaterOfSouls => 12,   // NPC_6
        NpcKind::Hornet => 13,         // NPC_42
        NpcKind::MeteorHead => 14,     // NPC_23
        NpcKind::GoblinSorcerer => 15, // NPC_29
        NpcKind::Tim => 16,            // NPC_45
        NpcKind::DarkCaster => 17,     // NPC_32
        NpcKind::FireImp => 18,        // NPC_24
        NpcKind::ChaosBall => 19,      // NPC_30
        NpcKind::BurningSphere => 20,  // NPC_25
        NpcKind::WaterSphere => 21,    // NPC_33
        NpcKind::Goldfish => 22,       // NPC_55
        NpcKind::Piranha => 23,        // NPC_58
        NpcKind::Shark => 24,          // NPC_65
        NpcKind::DevourerHead => 25,   // NPC_7
        NpcKind::Merchant => 26,       // NPC_17
        NpcKind::DevourerBody => 27,   // NPC_8
        NpcKind::DevourerTail => 28,   // NPC_9
        NpcKind::GiantWormHead => 29,  // NPC_10
        NpcKind::GiantWormBody => 30,  // NPC_11
        NpcKind::GiantWormTail => 31,  // NPC_12
        NpcKind::EaterOfWorldsHead => 32, // NPC_13
        NpcKind::EaterOfWorldsBody => 33, // NPC_14
        NpcKind::EaterOfWorldsTail => 34, // NPC_15
        NpcKind::Dryad => 35,          // NPC_20
        NpcKind::Nurse => 36,          // NPC_18
        NpcKind::ArmsDealer => 37,     // NPC_19
        NpcKind::Demolitionist => 38,  // NPC_38
        NpcKind::Painter => 39,        // NPC_227
        NpcKind::WitchDoctor => 40,    // NPC_228
        NpcKind::Clothier => 41,       // NPC_54
        NpcKind::EyeOfCthulhu => 42,   // NPC_4
        NpcKind::QueenBee => 43,       // NPC_222
        NpcKind::SkeletronHead => 44,  // NPC_35
        NpcKind::ServantOfCthulhu => 45, // NPC_5
        NpcKind::WallOfFlesh => 46,    // NPC_113
        NpcKind::Hungry => 47,         // NPC_115
        NpcKind::SnowmanGangsta => 48, // NPC_143
        NpcKind::SnowBalla => 49,      // NPC_144
        NpcKind::MysterySnowman => 50, // NPC_145
        NpcKind::MartianOfficer => 51, // NPC_383 (the humanoid martians
        NpcKind::GrayGrunt => 51,      // share the Officer's strip)
        NpcKind::MartianEngineer => 51,
        NpcKind::GigaZapper => 51,
        NpcKind::MartianWalker => 52,  // NPC_520
        NpcKind::PirateDeckhand => 53, // NPC_212 (the humanoid pirates
        NpcKind::PirateCaptainsMate => 53, // share the Deckhand's strip)
        NpcKind::PirateArcher => 53,
        NpcKind::PirateCorsair => 53,
        NpcKind::PirateCaptain => 54,  // NPC_216
        NpcKind::LeechHead => 55,      // NPC_117
        NpcKind::LeechBody => 56,      // NPC_118
        NpcKind::TheDestroyer => 57,   // NPC_134
        NpcKind::SkeletronPrime => 58, // NPC_127
        NpcKind::Retinazer => 59,      // NPC_125
        NpcKind::Spazmatism => 60,     // NPC_126
        NpcKind::Plantera => 61,       // NPC_262
        NpcKind::BrainOfCthulhu => 62, // NPC_266
        NpcKind::Creeper => 63,        // NPC_267
        NpcKind::GoblinThief => 64,    // NPC_26
        NpcKind::GoblinWarrior => 65,  // NPC_27
        NpcKind::GoblinArcher => 66,   // NPC_28
        NpcKind::GoblinPeon => 67,     // NPC_111
        NpcKind::GoblinSummoner => 68, // NPC_471
        NpcKind::BoundGoblin => 69,    // NPC_105
        NpcKind::GoblinTinkerer => 70, // NPC_107
        NpcKind::Golem => 71,          // NPC_245
        NpcKind::GolemHead => 72,      // NPC_246
        NpcKind::GolemFistLeft => 73,  // NPC_247 (the FistRight shares it
        NpcKind::GolemFistRight => 73, // — NPC_248 is not loaded)
        NpcKind::GolemHeadFree => 74,  // NPC_249
        NpcKind::MartianProbe => 76,   // NPC_399
        NpcKind::MartianDrone => 77,   // NPC_388
        NpcKind::Deerclops => 75,      // NPC_668
    };
    let white = [1.0, 1.0, 1.0, 1.0];
    match (entry.kind, entry.variant) {
        (NpcKind::Golem, _) => (sheet_i, 140.0, 140.0, 140.0, white),
        (NpcKind::GolemHead, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::GolemFistLeft, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemFistRight, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemHeadFree, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::Deerclops, _) => (sheet_i, 154.0, 60.0, 154.0, white),
        (NpcKind::Slime, Some(v)) => {
            let s = v.stats();
            (
                sheet_i,
                26.0,
                32.0 * s.scale,
                26.0 * s.scale,
                [
                    s.color[0] as f32 / 255.0,
                    s.color[1] as f32 / 255.0,
                    s.color[2] as f32 / 255.0,
                    s.color[3] as f32 / 255.0,
                ],
            )
        }
        // The hungry (3 frames) / the leech segments (1).
        (NpcKind::Hungry, _) => {
            let h = lookups.bestiary_npc_sheets[47]
                .map(|s| s.height as f32 / 3.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
        (NpcKind::LeechHead, _) | (NpcKind::LeechBody, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        // The eye: the 6-frame sheet's frame height rides the sheet
        // (the 100x110 frames).
        (NpcKind::EyeOfCthulhu, _) => {
            let h = lookups.bestiary_npc_sheets[42]
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(110.0);
            (sheet_i, h, 100.0, h, white)
        }
        (NpcKind::ServantOfCthulhu, _) => (sheet_i, 20.0, 20.0, 20.0, white),
        (NpcKind::Slime, None) => (sheet_i, 26.0, 32.0, 26.0, white),
        (NpcKind::DemonEye, _) => (sheet_i, 24.0, 38.0, 24.0, white),
        (NpcKind::WanderingEye, _) => {
            // The 4-frame sheet's frame height rides the sheet itself
            // (`draw_npcs`'s `t.h / 4`).
            let h = lookups.bestiary_npc_sheets[2]
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(24.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::Zombie, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Guide, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Skeleton | NpcKind::UndeadMiner, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::CaveBat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::JungleBat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::Hellbat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::VoodooDemon, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::EaterOfSouls, _) => (sheet_i, 30.0, 30.0, 30.0, white),
        (NpcKind::Hornet, _) => (sheet_i, 32.0, 34.0, 32.0, white),
        (NpcKind::MeteorHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        // The casters ride the 38x48 humanoid frame box; the spheres the
        // 16x16 sheet.
        (NpcKind::GoblinSorcerer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Tim, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::DarkCaster, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::FireImp, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::ChaosBall, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::BurningSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::WaterSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::Goldfish, _) => (sheet_i, 18.0, 20.0, 18.0, white),
        (NpcKind::Piranha, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        (NpcKind::Shark, _) => (sheet_i, 24.0, 100.0, 24.0, white),
        // The worm segments: the square sheets.
        (NpcKind::DevourerHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerBody, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerTail, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::GiantWormHead, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormBody, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormTail, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::EaterOfWorldsHead, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsBody, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsTail, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::Merchant, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Dryad, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Nurse, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::ArmsDealer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Demolitionist, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Painter, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::WitchDoctor, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Clothier, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::EyeOfCthulhu, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::QueenBee, _) => (sheet_i, 66.0, 66.0, 66.0, white),
        (NpcKind::SkeletronHead, _) => (sheet_i, 102.0, 80.0, 102.0, white),
        (NpcKind::WallOfFlesh, _) => (sheet_i, 120.0, 120.0, 120.0, white),
        (NpcKind::TheDestroyer, _) => (sheet_i, 48.0, 48.0, 48.0, white),
        (NpcKind::SkeletronPrime, _) => (sheet_i, 102.0, 80.0, 102.0, white),
        (NpcKind::Retinazer, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::Spazmatism, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::Plantera, _) => (sheet_i, 86.0, 86.0, 86.0, white),
        (NpcKind::BrainOfCthulhu, _) => (sheet_i, 110.0, 160.0, 110.0, white),
        (NpcKind::Creeper, _) => (sheet_i, 30.0, 30.0, 30.0, white),
        (NpcKind::GoblinPeon, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinThief, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinWarrior, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinArcher, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundGoblin, _) => (sheet_i, 48.0, 34.0, 54.0, white),
        (NpcKind::GoblinTinkerer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        // The frost trio: the 4/5/7-frame strips over the 26x40 frames.
        (NpcKind::SnowmanGangsta, _) => {
            let h = lookups.bestiary_npc_sheets[48]
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::SnowBalla, _) => {
            let h = lookups.bestiary_npc_sheets[49]
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::MysterySnowman, _) => {
            let h = lookups.bestiary_npc_sheets[50]
                .map(|s| s.height as f32 / 7.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        // The martians: the 9-frame strips over the 18x40 frames (the
        // walker's 11-frame 34x110 sheet).
        (NpcKind::MartianOfficer, _)
        | (NpcKind::GrayGrunt, _)
        | (NpcKind::MartianEngineer, _)
        | (NpcKind::GigaZapper, _) => {
            let h = lookups.bestiary_npc_sheets[51]
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::MartianWalker, _) => {
            let h = lookups.bestiary_npc_sheets[52]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(110.0);
            (sheet_i, h, 34.0, h, white)
        }
        // The probe: the 8-frame NPC_399 strip over the 42x72 frames.
        (NpcKind::MartianProbe, _) => {
            let h = lookups.bestiary_npc_sheets[76]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(72.0);
            (sheet_i, h, 42.0, h, white)
        }
        // The drone: the 2-frame NPC_388 strip over the 40x30 frames.
        (NpcKind::MartianDrone, _) => {
            let h = lookups.bestiary_npc_sheets[77]
                .map(|s| s.height as f32 / 2.0)
                .unwrap_or(30.0);
            (sheet_i, h, 40.0, h, white)
        }
        // The pirates: the 16/20-frame strips over the 18x40 frames.
        (NpcKind::PirateDeckhand, _)
        | (NpcKind::PirateCaptainsMate, _)
        | (NpcKind::PirateArcher, _)
        | (NpcKind::PirateCorsair, _) => {
            let h = lookups.bestiary_npc_sheets[53]
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::PirateCaptain, _) => {
            let h = lookups.bestiary_npc_sheets[54]
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::KingSlime, _) => (sheet_i, 120.0, 174.0, 120.0, white),
        // The goblins: the 16/21/20-frame humanoid strips over the 38-wide
        // display boxes (the frame height rides each sheet).
        (NpcKind::GoblinThief, _) => {
            let h = lookups.bestiary_npc_sheets[64]
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinWarrior, _) => {
            let h = lookups.bestiary_npc_sheets[65]
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinArcher, _) => {
            let h = lookups.bestiary_npc_sheets[66]
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinPeon, _) => {
            let h = lookups.bestiary_npc_sheets[67]
                .map(|s| s.height as f32 / 21.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinSummoner, _) => {
            let h = lookups.bestiary_npc_sheets[68]
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
    }
}

/// `UIBestiaryEntryInfoPage`'s content for the selected entry: the
/// `Portrait_Front` backdrop, the big portrait, the name (tier >= 1), the
/// stat rows (tier >= 2 — `Stat_HP`/`Stat_Attack`/`Stat_Defense`/
/// `Stat_Knockback`/`Stat_Copper`), the kill progress and the drop lines
/// (rates only from tier 4).
fn draw_info_page(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    idx: usize,
    mouse: Vec2,
) {
    let _ = mouse;
    let panel = panel_rect(screen);
    let entry = &crate::bestiary::ENTRIES[idx];
    let state = crate::bestiary::unlock_state(entry, tracker);
    if let Some(portrait_bg) = lookups.ui_bestiary_portrait {
        // The portrait card left of the info (the pretty-portrait mode's
        // 72 px card scaled up).
        let size = 216.0;
        let px = panel[0] + 24.0;
        let py = panel[1] + 60.0;
        let inner = [px + 2.0, py + 2.0, size - 4.0, size - 4.0];
        if let Some(back) = lookups.ui_bestiary_slot_back {
            push_whole(r, lookups, &back, inner, [1.0; 4]);
        }
        if state.known() {
            draw_portrait(r, lookups, entry, px + size * 0.5, py + size * 0.5, size);
        } else if let Some(locked) = lookups.ui_bestiary_locked {
            push_whole(r, lookups, &locked, inner, [0.15; 4]);
        }
        if let Some(front) = lookups.ui_bestiary_slot_front {
            push_whole(r, lookups, &front, inner, [1.0; 4]);
        }
    }
    let tx = panel[0] + 264.0;
    let mut ty = panel[1] + 60.0;
    // The name — tier 1+ (`GetHoverText`'s `???` gate,
    // `UnlockableNPCEntryIcon.cs:213-226`).
    let shown_name = if state.known() { entry.name } else { "???" };
    text::hud_text(r, lookups, tx, ty, 1.2, [1.0, 0.9, 0.4, 1.0], shown_name);
    ty += 34.0;

    // The stat rows — tier 2+ (`CanShowStats`).
    if state >= UnlockState::CanShowStats {
        let stats =
            crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_hp,
            &format!("{}", stats.hp),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_attack,
            &format!("{}", stats.damage),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_defense,
            &format!("{}", stats.defense),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_knockback,
            &format!("{:.2}", stats.kb),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_copper,
            &format!("{}", stats.value as i64),
        );
        ty += 10.0;
    }

    // The kill progress — `NPCKillCounterInfoElement`, tier 1+ (the count
    // rides the entry's portrait page, one tier above the `???` name).
    if state >= UnlockState::CanShowPortraitOnly {
        let (kills, needed) = crate::bestiary::kill_progress(entry, tracker);
        let progress = if entry.quick_unlock {
            format!("Kills: {kills}")
        } else {
            format!("Kills: {kills}/{needed}")
        };
        text::hud_text(r, lookups, tx, ty, 0.8, [0.8, 0.85, 1.0, 1.0], &progress);
        ty += 26.0;
    }

    // The drops — tier 3+, the rates only from tier 4.
    if state >= UnlockState::CanShowDropsWithoutDropRates {
        text::hud_text(r, lookups, tx, ty, 0.8, [1.0; 4], "Drops:");
        ty += 22.0;
        for (item, min, max, denom) in crate::loot_db::drop_lines(entry.net_id) {
            let name = crate::registry::name(*item as u16);
            let line = if state >= UnlockState::CanShowDropsWithDropRates {
                if *denom <= 1 {
                    format!("  {name}: {min}-{max} (100%)")
                } else {
                    format!("  {name} (1/{denom})")
                }
            } else {
                format!("  {name}")
            };
            text::hud_text(r, lookups, tx, ty, 0.8, [1.0; 4], &line);
            ty += 22.0;
        }
        if crate::loot_db::drop_lines(entry.net_id).is_empty() {
            text::hud_text(
                r,
                lookups,
                tx,
                ty,
                0.8,
                [0.7; 4],
                "  (nothing recorded yet)",
            );
        }
    }
}

/// One `Stat_*` icon + value row (the info page's stat list style).
fn stat_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    ty: &mut f32,
    tx: f32,
    sheet: Option<tr_render::SheetRect>,
    value: &str,
) {
    if let Some(sheet) = sheet {
        push_whole(r, lookups, &sheet, [tx, *ty, 20.0, 20.0], [1.0; 4]);
    }
    text::hud_text(r, lookups, tx + 28.0, *ty + 2.0, 0.8, [1.0; 4], value);
    *ty += 24.0;
}

/// One frame of a multi-frame sheet drawn into `dst` (the atlas-space source
/// rect is `frame * frame_h` down the sheet).
fn draw_sheet_frame(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sheet: &tr_render::SheetRect,
    frame: usize,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let sh = sheet.height as f32;
    let sw = sheet.width as f32;
    let y0 = sheet.y as f32 + sh * frame as f32;
    let uv = [
        sheet.x as f32 / aw,
        y0 / ah,
        (sheet.x as f32 + sw) / aw,
        (y0 + sh) / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_indexes_match_the_layout() {
        // A 1280x720 screen: the panel is 900 wide (0.875*1280 = 1120 capped)
        // and the grid fills rows of 72 px cells; every entry stays reachable.
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen);
        assert_eq!(panel[2], 900.0);
        let rect = grid_rect(&panel);
        let (cols, rows) = grid_metrics(&rect);
        assert!(cols * rows >= crate::bestiary::ENTRIES.len());
        // The first cell's centre hits entry 0 — the probe rides
        // `card_at`'s own lattice (the raw CELL grid, no insets) so the
        // overflow rows keep the hit.
        let first_x = rect[0] + CELL * 0.5;
        let first_y = rect[1] + CELL * 0.5;
        assert_eq!(card_at(screen, Vec2::new(first_x, first_y)), Some(0));
        // The Back panel sits inside the container above the bottom edge.
        let back = back_rect(&panel);
        assert!(back[1] + back[3] <= panel[1] + panel[3]);
        assert!(back_hit(screen, Vec2::new(back[0] + 10.0, back[1] + 10.0)));
        // The inventory button's hover rect matches the vanilla constants.
        assert!(button_hit(Vec2::new(498.0, 278.0)));
        assert!(!button_hit(Vec2::new(490.0, 278.0)));
    }
}
