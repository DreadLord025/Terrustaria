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
use super::uistate::{self, Style};
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

/// `BuildPage`'s outer container (`UIBestiaryTest.cs:71-79`), resolved through
/// the layout pass (`UIElement.cs:306-322`) instead of by hand:
///
/// ```text
/// Width  (0, 0.875)   MaxWidth 900   MinWidth 700   (`:71-73`)
/// Top    180          Height (-220, 1)              HAlign 0.5   (`:74-77`)
/// ```
///
/// `800 + Utils.ToInt(true) * 100` is the C#'s 900/700 pair (`:69`), the
/// non-zero/zero-style modes' widths. The container's **height** is the source's
/// `screenH − 220` — the previous hand-written rect subtracted the 180 top a
/// second time (`screenH − 400`, 180 px short at the bottom); the layout pass is
/// the faithful form, so the panel now ends at `screenH − 40`.
pub fn panel_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y], // the `UIState` root (`UIState.cs:9-11`)
        Style::px(0.0),        // `Left` unset (`StyleDimension.Empty`)
        Style::px(180.0),      // `Top.Set(180f, 0f)` (`:74`)
        Style::new(0.0, 0.875), // `Width.Set(0f, 0.875f)` (`:71`)
        Style::new(-220.0, 1.0), // `Height.Set(-220f, 1f)` (`:76`)
        0.5,                            // `HAlign = 0.5f` (`:77`)
        0.0,                            // `VAlign` unset
        700.0,                          // `MinWidth` (`:73`)
        0.0,                            // `MinHeight` unset
        900.0,                          // `MaxWidth` (`:72`)
        [0.0; 4],                       // the container carries no padding
    )
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
    // The sheet index rides `Lookups::bestiary_sheet_index` — the port's
    // order table `BESTIARY_NPC_SHEET_IDS` paired with the vec, derived
    // from the entry's own vanilla `net_id` (the hand-maintained table
    // that used to stand here had drifted twice: the frost trio's, the
    // boss rows' and finally the martian sheets' inserts shifted it).
    let sheet_i = lookups
        .bestiary_sheet_index(entry.net_id as u16)
        .unwrap_or(0);;    let white = [1.0, 1.0, 1.0, 1.0];
    match (entry.kind, entry.variant) {
        (NpcKind::Golem, _) => (sheet_i, 140.0, 140.0, 140.0, white),
        (NpcKind::GolemHead, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::GolemFistLeft, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemFistRight, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemHeadFree, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::Deerclops, _) => (sheet_i, 154.0, 60.0, 154.0, white),
        (NpcKind::SkeletronHand, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::WallOfFleshEye, _) => (sheet_i, 100.0, 100.0, 100.0, white),
        (NpcKind::PrimeCannon, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeSaw, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeVice, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeLaser, _) => (sheet_i, 52.0, 52.0, 52.0, white),
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
        (NpcKind::TravellingMerchant, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Angler, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::SleepingAngler, _) => (sheet_i, 8.0, 34.0, 8.0, white),
        (NpcKind::Wizard, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Mechanic, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundWizard, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundMechanic, _) => (sheet_i, 40.0, 38.0, 40.0, white),
        (NpcKind::Stylist, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::TaxCollector, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::WebbedStylist, _) => (sheet_i, 40.0, 38.0, 40.0, white),
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
        // The shooters: the 9-frame NPC_381/382 strips over the 18x40
        // frames.
        (NpcKind::BrainScrambler | NpcKind::RayGunner, _) => {
            let h = lookups.bestiary_npc_sheets[78]
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The rider: the 5-frame NPC_390 strip over the 18x40 frames.
        (NpcKind::ScutlixRider, _) => {
            let h = lookups.bestiary_npc_sheets[80]
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The carrier: the 6-frame NPC_391 strip over the 60x40 frames.
        (NpcKind::Scutlix, _) => {
            let h = lookups.bestiary_npc_sheets[81]
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(40.0);
            (sheet_i, h, 60.0, h, white)
        }
        // The saucer family: the shell 4-frame, the turret 18-frame, the
        // cannon 8-frame, the core 11-frame strips.
        (NpcKind::MartianSaucer, _) => {
            let h = lookups.bestiary_npc_sheets[82]
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(80.0);
            (sheet_i, h, 150.0, h, white)
        }
        (NpcKind::MartianSaucerTurret, _) => {
            let h = lookups.bestiary_npc_sheets[83]
                .map(|s| s.height as f32 / 18.0)
                .unwrap_or(16.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::MartianSaucerCannon, _) => {
            let h = lookups.bestiary_npc_sheets[84]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(36.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MartianSaucerCore, _) => {
            let h = lookups.bestiary_npc_sheets[85]
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(90.0);
            (sheet_i, h, 120.0, h, white)
        }
        // The Dutchman: the 1-frame ship strip + the cannons' 9-frame.
        (NpcKind::FlyingDutchman, _) => {
            let h = lookups.bestiary_npc_sheets[86]
                .map(|s| s.height as f32)
                .unwrap_or(120.0);
            (sheet_i, h, 350.0, h, white)
        }
        (NpcKind::FlyingDutchmanCannon, _) => {
            let h = lookups.bestiary_npc_sheets[87]
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
        // The DD2 goblins: the 14-frame NPC_552-554 strips over the 18x40
        // frames.
        (NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3, _) => {
            let h = lookups.bestiary_npc_sheets[88]
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The support family: the bombers'/skeletons' 18x40 and the
        // javelin throwers' 26x52 strips.
        (
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3
            | NpcKind::DD2SkeletonT1
            | NpcKind::DD2SkeletonT3,
            _,
        ) => {
            let h = lookups.bestiary_npc_sheets[91]
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3, _) => {
            let h = lookups.bestiary_npc_sheets[94]
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(52.0);
            (sheet_i, h, 26.0, h, white)
        }
        // The Dark Mage: the 20-frame NPC_564 strip over the 34x62 frames.
        (NpcKind::DD2DarkMage, _) => {
            let h = lookups.bestiary_npc_sheets[99]
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(62.0);
            (sheet_i, h, 34.0, h, white)
        }
        // Betsy: the 11-frame NPC_551 strip over the 190x90 frames.
        (NpcKind::DD2Betsy, _) => {
            let h = lookups.bestiary_npc_sheets[100]
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(90.0);
            (sheet_i, h, 190.0, h, white)
        }
        // The WitherBeast: the 14-frame NPC_568 strip over the 18x40 frames.
        (NpcKind::DD2WitherBeast, _) => {
            let h = lookups.bestiary_npc_sheets[101]
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The tavernkeeper: the 16-frame NPC_550 strip over the 18x40.
        (NpcKind::DD2Bartender, _) => {
            let h = lookups.bestiary_npc_sheets[102]
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The crystal: the NPC_548 strip over the 40x40 frames.
        (NpcKind::DD2EterniaCrystal, _) => {
            let h = lookups.bestiary_npc_sheets[103]
                .map(|s| s.height as f32)
                .unwrap_or(40.0);
            (sheet_i, h, 40.0, h, white)
        }
        // The cultist: the 1-frame NPC_422 strip over the 130x270 frames.
        (NpcKind::AncientCultist, _) => {
            let h = lookups.bestiary_npc_sheets[104]
                .map(|s| s.height as f32)
                .unwrap_or(270.0);
            (sheet_i, h, 130.0, h, white)
        }
        // The towers: the 1-frame NPC_493/507/517 strips.
        // The four pillars: the single 174x364 frames (the sheet's own
        // height — the hand-held 270 read of `bestiary_npc_sheets[111]`
        // was the Stardust row for every kind).
        (
            NpcKind::StardustTower
                | NpcKind::NebulaTower
                | NpcKind::SolarTower
                | NpcKind::VortexTower,
            _
        ) => {
            let h = lookups
                .bestiary_npc_sheets
                .get(sheet_i)
                .and_then(|s| *s)
                .map(|s| s.height as f32)
                .unwrap_or(364.0);
            (sheet_i, h, 174.0, h, white)
        }
        // The Old One's Army's lane gate (M4-EVENT-18): the single 78x130
        // NPC_549 frame — the body's own size (`NPC.cs`'s case 549).
        (NpcKind::DD2LanePortal, _) => {
            let h = lookups
                .bestiary_npc_sheets
                .get(sheet_i)
                .and_then(|s| *s)
                .map(|s| s.height as f32)
                .unwrap_or(130.0);
            (sheet_i, h, 78.0, h, white)
        }
        // The shards' portraits.
        (NpcKind::StardustCellBig, _) => {
            let h = lookups.bestiary_npc_sheets[119]
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(44.0);
            (sheet_i, h, 44.0, h, white)
        }
        (NpcKind::StardustCellSmall, _) => {
            let h = lookups.bestiary_npc_sheets[114]
                .map(|s| s.height as f32 / 2.0)
                .unwrap_or(22.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::StardustJellyfishBig, _) => {
            let h = lookups.bestiary_npc_sheets[115]
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(60.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::NebulaBeast, _) => {
            let h = lookups.bestiary_npc_sheets[116]
                .map(|s| s.height as f32 / 12.0)
                .unwrap_or(44.0);
            (sheet_i, h, 50.0, h, white)
        }
        (NpcKind::VortexRifleman | NpcKind::VortexSoldier, _) => {
            let h = lookups.bestiary_npc_sheets[117]
                .map(|s| s.height as f32 / 13.0)
                .unwrap_or(56.0);
            (sheet_i, h, 30.0, h, white)
        }
        // The pirates: the 16/20-frame strips over the 18x40 frames.
        // The Moon Lord family's portraits over the 396 sheet's rows.
        // The vortex trio's portraits over the 426 sheet's 15 rows.
        // The nebula trio's portraits over the 420 sheet's rows.
        // The tower mobs' portraits over the 409 sheet's rows.
        (NpcKind::StardustSpiderBig, _) => {
            let h = lookups.bestiary_npc_sheets[132]
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(42.0);
            (sheet_i, h, 34.0, h, white)
        }
        (NpcKind::StardustSpiderSmall, _) => {
            let h = lookups.bestiary_npc_sheets[138]
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(22.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::StardustSoldier, _) => {
            let h = lookups.bestiary_npc_sheets[144]
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::SolarDrakomire, _) => {
            let h = lookups.bestiary_npc_sheets[150]
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(40.0);
            (sheet_i, h, 60.0, h, white)
        }
        (NpcKind::SolarDrakomireRider, _) => {
            let h = lookups.bestiary_npc_sheets[156]
                .map(|s| s.height as f32 / 13.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::SolarSroller, _) => {
            let h = lookups.bestiary_npc_sheets[162]
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(52.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::SolarCorite, _) => {
            let h = lookups.bestiary_npc_sheets[168]
                .map(|s| s.height as f32 / 15.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        // The pillar worms' portraits over the 412 sheet's rows.
        (NpcKind::SolarCrawltipedeHead, _) => {
            let h = lookups.bestiary_npc_sheets[139]
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::SolarCrawltipedeBody, _) => {
            let h = lookups.bestiary_npc_sheets[140]
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::SolarCrawltipedeTail, _) => {
            let h = lookups.bestiary_npc_sheets[141]
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::StardustWormHead, _) => {
            let h = lookups.bestiary_npc_sheets[142]
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(32.0);
            (sheet_i, h, 32.0, h, white)
        }
        (NpcKind::SolarSpearman, _) => {
            let h = lookups.bestiary_npc_sheets[143]
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::SolarSelenian, _) => {
            let h = lookups.bestiary_npc_sheets[174]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::NebulaFloater, _) => {
            let h = lookups.bestiary_npc_sheets[128]
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(40.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::BrainSuckler, _) => {
            let h = lookups.bestiary_npc_sheets[129]
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(44.0);
            (sheet_i, h, 44.0, h, white)
        }
        (NpcKind::Predictor, _) => {
            let h = lookups.bestiary_npc_sheets[130]
                .map(|s| s.height as f32 / 12.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::VortexHunter, _) => {
            let h = lookups.bestiary_npc_sheets[125]
                .map(|s| s.height as f32 / 15.0)
                .unwrap_or(62.0);
            (sheet_i, h, 50.0, h, white)
        }
        (NpcKind::VortexAlien, _) => {
            let h = lookups.bestiary_npc_sheets[126]
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(28.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::VortexHeadcrab, _) => {
            let h = lookups.bestiary_npc_sheets[127]
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(26.0);
            (sheet_i, h, 24.0, h, white)
        }
        (NpcKind::MoonLordHead, _) => {
            let h = lookups.bestiary_npc_sheets[120]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(56.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::MoonLordHand, _) => {
            let h = lookups.bestiary_npc_sheets[121]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(66.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MoonLordCore, _) => {
            let h = lookups.bestiary_npc_sheets[122]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(66.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MoonLordFreeEye, _) => {
            let h = lookups.bestiary_npc_sheets[123]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(60.0);
            (sheet_i, h, 60.0, h, white)
        }
        (NpcKind::MoonLordLeechBlob, _) => {
            let h = lookups.bestiary_npc_sheets[124]
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
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
    fn dw_3_1_panel_rect_is_the_c_sharp_container() {
        // `UIBestiaryTest.cs:71-79` at the 1600x900 reference: width 0.875*1600
        // capped to 900, centred; Top 180; Height = screenH - 220 = 680, so the
        // panel ends at 860 (screenH - 40).
        let p = panel_rect(Vec2::new(1600.0, 900.0));
        assert_eq!(p, [350.0, 180.0, 900.0, 680.0]);
        // 1280x720: the same cap, height 720 - 220 = 500.
        let p = panel_rect(Vec2::new(1280.0, 720.0));
        assert_eq!(p, [190.0, 180.0, 900.0, 500.0]);
        // A narrow screen clamps to `MinWidth` 700 (`:73`, 600 + 100) — not the
        // old hand-written 560 floor.
        let p = panel_rect(Vec2::new(640.0, 480.0));
        assert_eq!(p[2], 700.0);
        // The Back panel still sits inside the container.
        let back = back_rect(&p);
        assert!(back[1] + back[3] <= p[1] + p[3]);
    }

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
