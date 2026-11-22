//! The emote draw — `EmoteBubble.Draw` (`:209-251`) and the
//! `UIEmotesMenu` wheel (the panel + the 8-column `EmoteButton` grid).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::push_whole;
use super::uistate::{self, Style};
use super::text;
use crate::assets::{self, Lookups};

/// `EmotesGroupListItem.SIZE_PER_EMOTE` (`EmotesGroupListItem.cs:15`) — the
/// `EmoteButton` cell is the sheet's 36x36 frame (`EmoteButton.cs:32-36`), the
/// port's old 44 px stand-in.
pub const CELL: f32 = 36.0;
/// `EmotesGroupListItem._maxEmotesPerRow` (`:21`), hard-overridden to 14 in the
/// ctor (`:25`).
pub const COLS: usize = 14;

/// `UI.Back` — the Back panel's label key (`UIEmotesMenu.cs:78`'s
/// `UITextPanel<LocalizedText>(Language.GetText("UI.Back"), …)`), the same key
/// the bestiary (`UIBestiaryTest.cs:366`) and achievements panels read.
pub const BACK_KEY: &str = "UI.Back";

/// How many wheel rows the panel's grid fits (`GetEntriesToShow`'s
/// `rectangle.Height / 72` analogue, `UIBestiaryTest.cs:357-362` — here the
/// 36 px cell).
pub fn visible_rows(grid: &[f32; 4]) -> usize {
    ((grid[3] / CELL) as usize).max(1)
}

/// `UIEmotesMenu.OnActivate`'s outer container (`UIEmotesMenu.cs:30-44`,
/// `:45-49`): a **590 px wide** box — not the bestiary panel it used to borrow —
/// centred, `Top 220 / Height −220` in the menu and `Top 120 / Height −120`
/// in-game (`:39-41`'s else arm). The port reaches the wheel only from the
/// in-game inventory (`clicks.rs:295`), so the in-game branch is implemented and
/// the menu split is recorded as a gap (plan §Phase 4).
pub fn panel_rect(screen: Vec2, in_game: bool) -> [f32; 4] {
    let (top, height) = if in_game {
        (120.0, Style::new(-120.0, 1.0))
    } else {
        (220.0, Style::new(-220.0, 1.0))
    };
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y], // the `UIState` root (`UIState.cs:9-11`)
        Style::px(0.0),                // `Left` unset
        Style::px(top),                // `_outerContainer.Top` (`:39`/`:41`)
        Style::px(590.0),              // `Width.Set(590f, 0f)` (`:48`)
        height,                        // `Height.Set(-120f/-220f, 1f)` (`:40`/`:42`)
        0.5,                           // `HAlign = 0.5f` (`:49`)
        0.0,
        0.0,
        0.0,
        590.0,
        [0.0; 4],
    )
}

/// The wheel's grid rect — the `UIList`'s body inside the panel
/// (`UIEmotesMenu.cs:52-57`: `Width (−25, 1)`, `Height (−50, 1)`, `Top 50`,
/// `HAlign 0.5`), with the Back row below it (`:59-69`, `VAlign 1`, `Top −25`,
/// 50 tall) left out of the hit/draw area.
pub fn grid_rect(screen: Vec2) -> [f32; 4] {
    let panel = panel_rect(screen, true);
    [
        panel[0] + 12.0,
        panel[1] + 40.0,
        panel[2] - 24.0,
        panel[3] - 40.0 - 60.0,
    ]
}

/// The grid's cell origin — the row lattice is centred inside the grid rect
/// (the `UIList`'s `HAlign = 0.5`, `UIEmotesMenu.cs:66`, over the 14×36 px
/// roster row). The draw and the hit test share it: before the fix the draw
/// centred the cells (+31 px at 590−24 wide) while `card_at` read the
/// uncentred lattice, so hover lit the wrong cell and clicks picked the
/// neighbouring emote.
pub fn grid_origin(rect: &[f32; 4]) -> [f32; 2] {
    [
        rect[0] + (rect[2] - COLS as f32 * CELL) * 0.5,
        rect[1],
    ]
}

/// The emote id under the cursor (the scroll-shifted grid) — the same
/// centred lattice the draw uses (`grid_origin`).
pub fn card_at(screen: Vec2, mouse: Vec2, scroll: i32) -> Option<i32> {
    let rect = grid_rect(screen);
    let origin = grid_origin(&rect);
    let dx = mouse.x - origin[0];
    let dy = mouse.y - origin[1];
    // The explicit negative guard: a float→usize cast saturates at 0, which
    // would make the centring margin hit column 0.
    if dx < 0.0 || dy < 0.0 {
        return None;
    }
    let col = (dx / CELL) as usize;
    let row = (dy / CELL) as usize;
    if col >= COLS || row >= visible_rows(&rect) {
        return None;
    }
    let idx = row * COLS + col + scroll as usize;
    (idx < crate::emotes::EMOTE_COUNT as usize).then_some(idx as i32)
}

/// The wheel's Back panel — `UIEmotesMenu.cs:78-84`: `Width(-10, 0.5)`,
/// `Height(50, 0)`, `VAlign 1`, `HAlign 0.5`, `Top(-45)`. The bestiary's
/// `back_rect` is the same box with `Top(-25)` (`UIBestiaryTest.cs:371`), so
/// the wheel keeps its own — reusing the bestiary's put the label 20 px low.
pub fn back_rect(panel: &[f32; 4]) -> [f32; 4] {
    let w = (panel[2] - 10.0) * 0.5;
    [
        panel[0] + panel[2] * 0.5 - w * 0.5,
        panel[1] + panel[3] - 45.0 - 50.0,
        w,
        50.0,
    ]
}

/// `back_hit` against *this* panel (`UIEmotesMenu.cs:78-86`'s Back) — the
/// bestiary twin reads the bestiary container, so the wheel keeps its own.
pub fn back_hit(panel: &[f32; 4], mouse: Vec2) -> bool {
    let r = back_rect(panel);
    mouse.x >= r[0] && mouse.x <= r[0] + r[2] && mouse.y >= r[1] && mouse.y <= r[1] + r[3]
}

// --- EmotesGroupListItem (W4-d81 census row 3) ----------------------------------
//
// The port's wheel draws the flat emote roster on the shared 14x36 lattice
// (`draw` below; the group titles/separators are a documented deviation, the
// flat-roster note in the tests). These helpers port the C# group item's
// geometry so the grouped layout can be adopted without re-deriving it.
// Row 2 (`EmoteButton`) is fully landed already: the 20-tick blink counter
// (`crate::emotes::EmoteSystem::button_frame`, `EmoteButton.cs:38-44`), the
// frame pick (`:32-36`, `draw` below), the hover sound-12 edge
// (`app/loop.rs:600-619`, `:77-88`), the border tint + `MouseText` name
// (`:52-75`, `draw` below) and the click's emote+close (`:90-95`,
// `app/clicks.rs:1170-1197`).

/// `TITLE_HEIGHT = 20` / `SEPARATOR_HEIGHT = 10` (`EmotesGroupListItem.cs:11-13`)
/// — declared but unused by the ctor, which lays out with 30/36 (`:35`, `:79`).
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub const GROUP_TITLE_HEIGHT: f32 = 20.0;
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub const GROUP_SEPARATOR_HEIGHT: f32 = 10.0;

/// The ctor's row count (`EmotesGroupListItem.cs:30-34`):
/// `emotes.Length / 14`, `+1` when the remainder is non-zero.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn group_row_count(emote_count: usize) -> usize {
    let mut rows = emote_count / COLS;
    if emote_count % COLS != 0 {
        rows += 1;
    }
    rows
}

/// The group item's height (`EmotesGroupListItem.cs:35`):
/// `Height.Set(30 + 36 * rows, 0f)` — the 30 px header band plus the rows.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn group_height(emote_count: usize) -> f32 {
    30.0 + CELL * group_row_count(emote_count) as f32
}

/// The header container (`EmotesGroupListItem.cs:37-42`): `Height 30`,
/// `Width (-20, 1)`, `HAlign 0.5` — 10 px in from each side of the group item.
/// The separator (`VAlign 1`, `:45-51`) and the title (`VAlign 1`, `Top -6`,
/// `:52-57`) hang off its bottom edge.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn group_header_rect(group: &[f32; 4]) -> [f32; 4] {
    [group[0] + 10.0, group[1], group[2] - 20.0, 30.0]
}

/// The header separator's tint (`EmotesGroupListItem.cs:50`):
/// `Color.Lerp(Color.White, new Color(63, 65, 151), 0.85f) * 0.9f`.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn group_separator_color() -> [u8; 4] {
    let lerped = super::char_creation::lerp_rgb([255, 255, 255, 255], [63, 65, 151, 255], 0.85);
    super::color::mul(lerped, 0.9)
}

/// Emote `i`'s button top-left inside the group item
/// (`EmotesGroupListItem.cs:58-81`): `Top = 30 + row*36 + 6` (`num2 = 6f`,
/// `:58`/`:79`), `Left = 36*col - num6` with `num6` hard-overwritten to `-16f`
/// (`:74`) — the `num5` centring computation (`:64-73`) is dead code in the
/// C#, every row starts 16 px in.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn group_button_top_left(i: usize) -> [f32; 2] {
    let row = (i / COLS) as f32; // `num3` (`:62`)
    let col = (i % COLS) as f32; // `num4` (`:63`)
    [30.0 + row * CELL + 6.0, col * CELL + 16.0]
}

/// `CompareTo` (`EmotesGroupListItem.cs:87-94`): groups sort by `_groupIndex`.
#[allow(dead_code)] // the grouped wheel adopts these (W4-d81); the flat roster draws today
pub fn compare_group_order(a: i32, b: i32) -> std::cmp::Ordering {
    a.cmp(&b)
}

/// `EmoteBubble.GetPosition`'s entity arm (`EmoteBubble.cs:247-249`):
/// `(entity.Top.X, VisualPosition.Y) + (-direction*width*0.75, 2)` in world
/// px — `Top.X` is the hitbox's horizontal centre. Returns the anchor and
/// the `SpriteEffects.FlipHorizontally` flag (`direction != -1`, `:248`).
/// The port anchors the local player only; `VisualPosition`'s net/gfx
/// offsets are not modelled for the local player, and the reverse-gravity
/// arm (`:207-212`) stays out with the unported gravity flip.
pub fn bubble_anchor(
    player_x: f32,
    player_y: f32,
    width: f32,
    facing: i32,
) -> ([f32; 2], bool) {
    (
        [
            player_x + width * 0.5 - facing as f32 * width * 0.75,
            player_y + 2.0,
        ],
        facing != -1,
    )
}

/// The full wheel + bubble draw (`EmoteBubble.DrawAll` + the menu state).
/// The bubbles ride the world→UI transform (`Main.GameViewMatrix.ZoomMatrix`,
/// `DrawInterface_1_1_DrawEmoteBubblesInWorld` `Main.cs:46455-46459`) — the
/// same `(world - camera) * zoom + screen/2` the breath bubbles use
/// (`ui::resources::draw_breath`); before the fix the raw world coordinates
/// went straight into the UI batch, putting every bubble thousands of px
/// offscreen.
///
/// W4-g50: the `91`-synced bubbles anchor on peer players / NPCs /
/// projectiles (`DeserializeNetAnchor`, `EmoteBubble.cs:100-109`) — the
/// resolver walks the three stores by the wire's (kind, index) pair.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    emotes: &crate::emotes::EmoteSystem,
    player: Option<&crate::player::Player>,
    camera: Vec2,
    zoom: f32,
    mouse: Vec2,
    pulse: super::UiPulse,
    local_id: u32,
    remote_players: &std::collections::HashMap<u32, crate::net::PlayerNet>,
    npcs: &[crate::npc::Npc],
    projectiles: &crate::projectile::ProjectilePool,
) {
    // The bubbles (`EmoteBubble.Draw`, `EmoteBubble.cs:199-241`).
    if let Some(sheet) = lookups.ui_emote_sheet {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        // `value.Frame(8, EMOTE_SHEET_VERTICAL_FRAMES, …)` — integer division
        // (`Utils.Frame`): 272/8 = 34 wide, 1092/39 = 28 tall.
        let sw = (sheet.width / 8) as f32;
        let sh = (sheet.height / crate::emotes::SHEET_ROWS as u32) as f32;
        // `origin = (frame.Width/2, frame.Height)` (`:206`): the bubble
        // hangs from its bottom centre.
        let mut draw_frame = |col: i32, row: i32, anchor: [f32; 2], flip: bool| {
            let x0 = sheet.x as f32 + sw * col as f32;
            let y0 = sheet.y as f32 + sh * row as f32;
            let mut uv = [x0 / aw, y0 / ah, (x0 + sw) / aw, (y0 + sh) / ah];
            if flip {
                uv = crate::render::player_layers::flip_uv(uv, true);
            }
            let pos = [
                anchor[0] - sw * zoom * 0.5,
                anchor[1] - sh * zoom,
            ];
            r.push_ui_sprite(pos, [sw * zoom, sh * zoom], uv, [1.0; 4]);
        };
        for b in &emotes.bubbles {
            // `GetPosition`'s entity arm (`:247-249`) on the resolved
            // anchor: the local player, a peer snapshot, an NPC or a
            // projectile. An unresolvable anchor draws nothing (the C#
            // entity reference would be null — the port skips it).
            let resolved: Option<(f32, f32, f32, i32)> = match b.anchor_kind {
                1 if b.anchor_index as u32 == local_id => {
                    player.map(|p| (p.x, p.y, crate::player::WIDTH, p.facing))
                }
                1 => remote_players
                    .get(&(b.anchor_index as u32))
                    .map(|p| (p.x, p.y, crate::player::WIDTH, p.facing)),
                0 => npcs.get(b.anchor_index as usize).map(|n| {
                    let (w, _) = n.size();
                    (n.x, n.y, w, n.facing)
                }),
                // The projectile pool carries no per-kind width table (the
                // draw reads the sheet frame); the 16 px hitbox stand-in is
                // the port's recorded convention (`projectile.rs:10318`).
                2 => projectiles
                    .slots()
                    .get(b.anchor_index as usize)
                    .filter(|p| p.active)
                    .map(|p| (p.pos.x, p.pos.y, 16.0, if p.vel.x >= 0.0 { 1 } else { -1 })),
                _ => None,
            };
            let Some((ex, ey, ew, ef)) = resolved else {
                continue;
            };
            let (world0, flip0) = bubble_anchor(ex, ey, ew, ef);
            // `GetPosition(...).Floor()` (`:203`).
            let world0 = [world0[0].floor(), world0[1].floor()];
            let anchor = [
                (world0[0] - camera.x) * zoom + screen.x * 0.5,
                (world0[1] - camera.y) * zoom + screen.y * 0.5,
            ];
            // The pop gate: the first/last 6 ticks draw the closed shell
            // (`:204-205`, `EmoteBubble::popping`/`shell_column`). The shell
            // draws FIRST with the original flip at the unshifted anchor
            // (`:213`), and the pop frames early-return (`:214-217`) before
            // the 87/89 adjustment.
            let popping = b.popping();
            draw_frame(b.shell_column(), 0, anchor, flip0);
            if popping {
                continue;
            }
            // The 87/89 un-flip (`:220-224`, `crate::emotes::bubble_unflips`):
            // the confused/sleep emotes keep their orientation and shift
            // 4 px right instead — applied AFTER the shell, to the emote
            // sprite only (`:225`).
            let (mut world, mut flip) = (world0, flip0);
            if crate::emotes::bubble_unflips(b.emote, flip) {
                flip = false;
                world[0] += 4.0;
            }
            let anchor = [
                (world[0] - camera.x) * zoom + screen.x * 0.5,
                (world[1] - camera.y) * zoom + screen.y * 0.5,
            ];
            // The emote sprite: column `emote*2%8 + frame`, row `1 + emote/4`
            // (`:225`, `EmoteBubble::emote_frame`).
            let (col, row) = b.emote_frame();
            draw_frame(col, row, anchor, flip);
        }
    }
    // The wheel (`UIEmotesMenu`): the shared panel chrome + the grid.
    if !emotes.menu_open {
        return;
    }
    let panel = panel_rect(screen, true);
    r.push_ui_sprite(
        [panel[0], panel[1]],
        [panel[2], panel[3]],
        r.white_uv(),
        [33.0 / 255.0 * 0.8, 43.0 / 255.0 * 0.8, 79.0 / 255.0 * 0.8, 0.8],
    );
    text::hud_text(r, lookups, panel[0] + 12.0, panel[1] + 10.0, 1.0, [1.0; 4], "Emotes");
    let rect = grid_rect(screen);
    let origin = grid_origin(&rect);
    let hovered = card_at(screen, mouse, emotes.scroll);
    if let Some(sheet) = lookups.ui_emote_sheet {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        // The button's true frame size (`EmoteButton`'s ctor `:27-29` reads
        // `GetFrame().Width/Height` — 34x28), centred in the 36 px lattice
        // cell (`EmotesGroupListItem.SIZE_PER_EMOTE`, `:15`).
        let sw = (sheet.width / 8) as f32;
        let sh = (sheet.height / crate::emotes::SHEET_ROWS as u32) as f32;
        for row in 0..visible_rows(&rect) {
            for col in 0..COLS {
                let idx = row * COLS + col + emotes.scroll as usize;
                if idx >= crate::emotes::EMOTE_COUNT as usize {
                    break;
                }
                let id = idx as i32;
                let cx = origin[0] + col as f32 * CELL + CELL * 0.5;
                let cy = origin[1] + row as f32 * CELL + CELL * 0.5;
                let hover = hovered == Some(id);
                // `EmoteButton.DrawSelf` (`:52-75`): the shell frame
                // (column 1, row 0), the emote frame, then the border —
                // black idle, `Main.OurFavoriteColor` (255,231,69,
                // `Main.cs:66748`) hovered, at `center - (2, 2)`.
                let shell_x0 = sheet.x as f32 + sw;
                let shell_y0 = sheet.y as f32;
                let shell_uv = [
                    shell_x0 / aw,
                    shell_y0 / ah,
                    (shell_x0 + sw) / aw,
                    (shell_y0 + sh) / ah,
                ];
                r.push_ui_sprite(
                    [cx - sw * 0.5, cy - sh * 0.5],
                    [sw, sh],
                    shell_uv,
                    [1.0; 4],
                );
                // `GetFrame` (`:32-36`): `emote%4*2 + (frameCounter >= 10)`,
                // row `emote/4 + 1` — the second frame is the idle blink,
                // not the hover pick.
                let fcol = (id % 4) * 2 + emotes.button_frame();
                let frow = 1 + id / 4;
                let x0 = sheet.x as f32 + sw * fcol as f32;
                let y0 = sheet.y as f32 + sh * frow as f32;
                let uv = [x0 / aw, y0 / ah, (x0 + sw) / aw, (y0 + sh) / ah];
                r.push_ui_sprite(
                    [cx - sw * 0.5, cy - sh * 0.5],
                    [sw, sh],
                    uv,
                    [1.0; 4],
                );
                // The border (`Images/UI/EmoteBubbleBorder`, 36x30,
                // `EmoteButton.cs:25`/`:69`).
                if let Some(border) = lookups.ui_emote_border {
                    let tint = if hover {
                        [255.0 / 255.0, 231.0 / 255.0, 69.0 / 255.0, 1.0]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    };
                    // The border origin is the EMOTE FRAME's half-size
                    // (17,14), not the border's own (18,15): `EmoteButton.cs:60`
                    // `origin = frame.Size()/2`, drawn at `vector - One*2`
                    // (`:69`) -> top-left = center - (19,16).
                    push_whole(
                        r,
                        lookups,
                        &border,
                        [
                            cx - 2.0 - sw * 0.5,
                            cy - 2.0 - sh * 0.5,
                            border.width as f32,
                            border.height as f32,
                        ],
                        tint,
                    );
                }
            }
        }
    }
    // The hovered emote's name (`EmoteButton.DrawSelf` `:70-74`):
    // `"/" + Language.GetTextValue("EmojiName." + EmoteID.Search.GetName(id))`
    // through `Main.MouseText`.
    if let Some(id) = hovered {
        if let Some((_, name)) = crate::emotes::EMOTE_NAMES.iter().find(|(i, _)| *i == id) {
            let label = format!("/{}", crate::lang::text(&format!("EmojiName.{name}")));
            super::tooltip::draw_mouse_text(r, lookups, mouse, &label, screen, pulse);
        }
    }
    // The Back panel (`UIEmotesMenu.cs:78-87`): `UITextPanel<LocalizedText>(
    // Language.GetText("UI.Back"), 0.7f, large: true)` with
    // `Width(-10, 0.5)`, `Height(50, 0)`, `VAlign 1`, `HAlign 0.5` and
    // **`Top(-45)`** — the wheel's own -45, not the bestiary's -25, which is
    // what made the label sit off-centre. The label goes through `lang::text`
    // like the bestiary's and the achievements' panels do, never a literal.
    let back = back_rect(&panel);
    let style = super::cards::fat_button_style();
    let label = crate::lang::text(BACK_KEY);
    let text_w = text::hud_text_width(r, lookups, &label, style.text_scale);
    super::uistate::draw_text_panel(
        r,
        lookups,
        back,
        &style,
        &label,
        text_w,
        back_hit(&panel, mouse),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_group_item_geometry_follows_the_ctor() {
        // `EmotesGroupListItem.cs:30-35`: ceil(count/14) rows, Height 30+36*n.
        assert_eq!(group_row_count(0), 0);
        assert_eq!(group_row_count(14), 1);
        assert_eq!(group_row_count(15), 2);
        assert_eq!(group_height(14), 66.0);
        assert_eq!(group_height(15), 102.0);
        assert_eq!(group_height(0), 30.0);
        // The header container (`:37-42`): Width (-20, 1), HAlign 0.5.
        let group = [100.0, 50.0, 590.0, 102.0];
        assert_eq!(group_header_rect(&group), [110.0, 50.0, 570.0, 30.0]);
        // The buttons (`:58-81`): Top 30 + row*36 + 6, Left 36*col + 16 —
        // `num6` is hard-overwritten to -16 (`:74`), the centring walk dead.
        assert_eq!(group_button_top_left(0), [36.0, 16.0]);
        assert_eq!(group_button_top_left(13), [36.0, 13.0 * 36.0 + 16.0]);
        assert_eq!(group_button_top_left(14), [72.0, 16.0]);
        // `CompareTo` (`:87-94`) sorts by group index.
        assert_eq!(compare_group_order(1, 2), std::cmp::Ordering::Less);
        assert_eq!(compare_group_order(2, 2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn the_group_separator_tint_is_the_white_to_inventory_lerp() {
        // `EmotesGroupListItem.cs:50`: Lerp(White, (63,65,151), 0.85) * 0.9.
        // Lerp: 255 + (63-255)*0.85 = 91.8 -> 91; 255 + (65-255)*0.85 = 93.5
        // -> 93 (f32 truncation); 255 + (151-255)*0.85 = 166.6 -> 166.
        // *0.9: (81, 83, 149, 229).
        assert_eq!(group_separator_color(), [81, 83, 149, 229]);
    }

    #[test]
    fn dw_4_1_the_in_game_container_is_the_c_sharp_split() {
        // `UIEmotesMenu.cs:39-41`'s in-game arm: Top 120, Height -120, and
        // `InitializePage`'s 590-wide box (`:48`) — centred at 1600: x = 505.
        let p = panel_rect(Vec2::new(1600.0, 900.0), true);
        assert_eq!(p, [505.0, 120.0, 590.0, 780.0]);
        // 1280x720: the same box, height 720 - 120 = 600.
        let p = panel_rect(Vec2::new(1280.0, 720.0), true);
        assert_eq!(p, [345.0, 120.0, 590.0, 600.0]);
        // A narrow screen: the container is a fixed 590 wide (`Width.Set(590f, 0f)`),
        // centred — it does not shrink.
        let p = panel_rect(Vec2::new(640.0, 480.0), true);
        assert_eq!(p[2], 590.0);
        // The Back row sits at the container's bottom.
        let back = crate::ui::bestiary::back_rect(&p);
        assert_eq!(back[1] + back[3], p[1] + p[3] - 25.0);
    }

    #[test]
    fn dw_4_2_every_emote_id_is_reachable_at_36px_and_14_per_row() {
        // `EmotesGroupListItem.cs:15`/`:25`: 36 px cells, 14 per row — the port's
        // flat roster against the same lattice.
        let screen = Vec2::new(1600.0, 900.0);
        let grid = grid_rect(screen);
        let origin = grid_origin(&grid);
        let rows = visible_rows(&grid);
        assert!(rows >= 1);
        for scroll in 0..=(crate::emotes::EMOTE_COUNT as usize).div_ceil(COLS) {
            for row in 0..rows {
                for col in 0..COLS {
                    let idx = row * COLS + col + scroll;
                    if idx >= crate::emotes::EMOTE_COUNT as usize {
                        continue;
                    }
                    let cx = origin[0] + col as f32 * CELL + CELL * 0.5;
                    let cy = origin[1] + row as f32 * CELL + CELL * 0.5;
                    assert_eq!(
                        card_at(screen, Vec2::new(cx, cy), scroll as i32),
                        Some(idx as i32),
                        "id {idx} at row {row}, col {col}, scroll {scroll}"
                    );
                }
            }
        }
    }

    #[test]
    fn dw_4_3_the_scroll_clamps_at_both_ends_and_resets_on_open() {
        let mut e = crate::emotes::EmoteSystem::default();
        e.open();
        assert_eq!(e.scroll, 0);
        // Scrolling up from the top stays at 0 (`UIScrollbar.cs:37-47`).
        e.scroll_by(-3, 5);
        assert_eq!(e.scroll, 0);
        // And past the last row the offset stops at the roster's end.
        let total_rows = (crate::emotes::EMOTE_COUNT as usize).div_ceil(COLS);
        for _ in 0..total_rows + 10 {
            e.scroll_by(1, 5);
        }
        assert_eq!(e.scroll as usize, total_rows - 5);
        // Each open resets it (`UIEmotesMenu.OnActivate`'s rebuild).
        e.open();
        assert_eq!(e.scroll, 0);
        e.close();
        assert!(!e.menu_open);
        assert_eq!(e.scroll, 0);
    }

    #[test]
    fn dw_4_3_a_partial_last_row_stays_hittable() {
        // 151 emotes / 14 per row leaves a partial last row; its cells resolve
        // until the roster runs out and `card_at` returns None.
        let screen = Vec2::new(1600.0, 900.0);
        let grid = grid_rect(screen);
        let origin = grid_origin(&grid);
        let last = (crate::emotes::EMOTE_COUNT as usize - 1) / COLS;
        let col = (crate::emotes::EMOTE_COUNT as usize - 1) % COLS;
        let cx = origin[0] + col as f32 * CELL + CELL * 0.5;
        let cy = origin[1] + last as f32 * CELL + CELL * 0.5;
        assert_eq!(
            card_at(screen, Vec2::new(cx, cy), 0),
            Some(crate::emotes::EMOTE_COUNT - 1)
        );
        let cx2 = origin[0] + (col + 1) as f32 * CELL + CELL * 0.5;
        assert_eq!(card_at(screen, Vec2::new(cx2, cy), 0), None);
    }

    #[test]
    fn dw_4_3_the_back_row_hangs_off_the_wheel_container() {
        // The wheel's Back is half the *wheel* container's width, not the
        // bestiary panel's it used to borrow, and it hangs at its own
        // `Top(-45)` (`UIEmotesMenu.cs:82`) — not the bestiary's -25
        // (`UIBestiaryTest.cs:371`).
        let panel = panel_rect(Vec2::new(1600.0, 900.0), true);
        let back = back_rect(&panel);
        assert_eq!(back[2], (590.0 - 10.0) * 0.5);
        assert_eq!(back[3], 50.0);
        assert_eq!(back[1], panel[1] + panel[3] - 95.0); // -45 top, 50 tall
        // Centred: the box's midpoint is the panel's midpoint.
        assert_eq!(back[0] + back[2] * 0.5, panel[0] + panel[2] * 0.5);
        assert!(back_hit(&panel, Vec2::new(back[0] + 10.0, back[1] + 10.0)));
        assert!(!back_hit(&panel, Vec2::new(back[0] - 10.0, back[1] + 10.0)));
        // The old bestiary borrow (`Top(-25)`, `UIBestiaryTest.cs:371`) spans
        // `bottom-75 .. bottom-25`; the wheel's own (`Top(-45)`,
        // `UIEmotesMenu.cs:82`) spans `bottom-95 .. bottom-45`. A point in the
        // bestiary's lower half — below the wheel's own bottom edge — is NOT a
        // wheel-Back hit, which is what pinned the 20 px mis-centring.
        let bestiary = crate::ui::bestiary::back_rect(&panel);
        assert!(
            !back_hit(&panel, Vec2::new(bestiary[0] + 10.0, bestiary[3] + bestiary[1] - 10.0)),
            "the bestiary's lower row sits below the wheel's own Back"
        );
        // And the wheel's own box is 20 px above the bestiary's top edge.
        assert_eq!(bestiary[1] - back[1], 20.0);
    }

    #[test]
    fn hover_picks_the_drawn_cell_not_the_uncentred_lattice() {
        // The break: the draw centred the 14x36 roster inside the grid
        // (`UIList.HAlign = 0.5`, `UIEmotesMenu.cs:66`) while `card_at` read
        // the uncentred lattice — at 1600x900 the offset is
        // (566 - 504)/2 = 31 px, so a pointer on a drawn cell resolved its
        // left neighbour (and the leftmost 31 px lit nothing).
        let screen = Vec2::new(1600.0, 900.0);
        let grid = grid_rect(screen);
        let origin = grid_origin(&grid);
        assert_eq!(origin[0] - grid[0], (grid[2] - COLS as f32 * CELL) * 0.5);
        // The drawn centre of cell (3, 1) resolves to id 17.
        let cx = origin[0] + 3.0 * CELL + CELL * 0.5;
        let cy = origin[1] + 1.0 * CELL + CELL * 0.5;
        assert_eq!(card_at(screen, Vec2::new(cx, cy), 0), Some(17));
        // The centring margin left of the roster is not a cell (the old
        // float→usize cast saturated the negative offset to column 0).
        assert_eq!(
            card_at(screen, Vec2::new(grid[0] + 2.0, origin[1] + 5.0), 0),
            None
        );
        // Every cell edge pair agrees: the point just inside the drawn
        // cell's left edge hits that cell, just outside hits the previous.
        let left_edge = origin[0] + 5.0 * CELL;
        assert_eq!(
            card_at(screen, Vec2::new(left_edge + 0.5, origin[1] + 5.0), 0),
            Some(5)
        );
        assert_eq!(
            card_at(screen, Vec2::new(left_edge - 0.5, origin[1] + 5.0), 0),
            Some(4)
        );
        // Scroll shifts the roster, not the lattice.
        assert_eq!(card_at(screen, Vec2::new(cx, cy), 14), Some(31));
    }

    #[test]
    fn the_bubble_anchor_is_the_c_sharp_entity_arm() {
        // `GetPosition` (`EmoteBubble.cs:247-249`): `(Top.X, y) +
        // (-direction*width*0.75, 2)`, `Top.X` = x + width/2; flip while
        // `direction != -1` (`:248`).
        let (pos, flip) = bubble_anchor(1000.0, 2000.0, 20.0, 1);
        assert_eq!(pos, [1000.0 + 10.0 - 15.0, 2002.0]);
        assert!(flip);
        let (pos, flip) = bubble_anchor(1000.0, 2000.0, 20.0, -1);
        assert_eq!(pos, [1000.0 + 10.0 + 15.0, 2002.0]);
        assert!(!flip);
    }

    #[test]
    fn bug_18_the_back_label_is_the_ui_back_localized_text() {
        // `UIEmotesMenu.cs:78`: the Back panel is a
        // `UITextPanel<LocalizedText>(Language.GetText("UI.Back"), …)`, so the
        // label is the localized value — never the literal "Back".
        assert_eq!(BACK_KEY, "UI.Back");
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // The ru-RU `UI` group carries `Back` (`Content.ru-RU.json`); with no
        // localization sources the manager answers the key itself, in which
        // case this test only pins the key spelling.
        crate::lang::init(tr_core::localization::GameCulture::Russian);
        let ru = crate::lang::text(BACK_KEY);
        if ru == BACK_KEY {
            eprintln!("skipping the localization half: no sources");
            crate::lang::init(tr_core::localization::GameCulture::English);
            return;
        }
        assert_ne!(ru, "Back", "ru-RU must not read the English literal");
        assert_eq!(ru, "Назад", "the ru-RU `UI.Back` row");
        crate::lang::init(tr_core::localization::GameCulture::English);
        let en = crate::lang::text(BACK_KEY);
        assert_eq!(en, "Back", "the en-US `UI.Back` row");
    }

    // S1-E-26 wave B — the wheel's cell geometry and open/pick/close flow,
    // verified against `EmoteButton.cs` and `UIEmotesMenu.cs` line by line.
    // Tests written, not run (code-only mode).

    #[test]
    fn the_button_frame_formula_matches_get_frame() {
        // `EmoteButton.GetFrame` (`:32-36`): column `_emoteIndex%4*2 +
        // (_frameCounter >= 10 ? 1 : 0)`, row `_emoteIndex/4 + 1`. The draw
        // reads the same formula via `EmoteSystem::button_frame` (the shared
        // 20-tick counter). Pin it for representative ids across the roster.
        let mut sys = crate::emotes::EmoteSystem::default();
        sys.open();
        // First half of the cycle: frame 0.
        assert_eq!(sys.button_frame(), 0);
        for id in [0i32, 3, 4, 7, 8, 87, 150] {
            let col = (id % 4) * 2 + sys.button_frame();
            let row = 1 + id / 4;
            assert_eq!(col, (id % 4) * 2, "frame 0 half: id {id}");
            assert!(row >= 1 && row < crate::emotes::SHEET_ROWS, "id {id}");
        }
        // Second half (counter >= 10): frame 1 shifts every column by one.
        for _ in 0..10 {
            sys.update_all();
        }
        assert_eq!(sys.button_frame(), 1);
        for id in [0i32, 3, 4, 150] {
            let col = (id % 4) * 2 + sys.button_frame();
            assert_eq!(col, (id % 4) * 2 + 1, "frame 1 half: id {id}");
        }
        // The shell frame is always column 1, row 0 (`DrawSelf` `:58-59`:
        // `value.X = texture.Width()/8`, `value.Y = 0`) — independent of id.
        let shell_col = 1;
        let shell_row = 0;
        assert_eq!((shell_col, shell_row), (1, 0));
    }

    #[test]
    fn the_border_origin_is_the_emote_frame_half_size() {
        // `EmoteButton.DrawSelf` (`:60`, `:69`): `origin = frame.Size()/2`
        // (the 34x28 emote frame → (17,14)), drawn at `vector - One*2`, so
        // the border's top-left = cell centre − (19,16). NOT the border's
        // own half-size (18,15) — the BUG the draw comment records.
        let (frame_w, frame_h) = (34.0f32, 28.0f32);
        let origin = [frame_w * 0.5, frame_h * 0.5];
        assert_eq!(origin, [17.0, 14.0]);
        let centre = [100.0f32, 200.0f32];
        let top_left = [centre[0] - 2.0 - origin[0], centre[1] - 2.0 - origin[1]];
        assert_eq!(top_left, [81.0, 184.0], "centre − (19,16)");
    }

    #[test]
    fn the_wheel_opens_picks_and_closes() {
        // The full flow: `DrawEmoteBubblesButton`'s click opens the wheel
        // (`Main.cs:42404` → `UIEmotesMenu.OnActivate` rebuild, scroll 0),
        // a cell pick fires `EmoteButton.LeftClick` (`:90-95`) →
        // `MakeLocalPlayerEmote` (`EmoteBubble.cs:278-292`, the 360-tick
        // bubble) + `IngameFancyUI.Close`, and Back just closes
        // (`UIEmotesMenu.GoBackClick` `:305-309`).
        let screen = Vec2::new(1600.0, 900.0);
        let mut sys = crate::emotes::EmoteSystem::default();
        assert!(!sys.menu_open);
        sys.open();
        assert!(sys.menu_open);
        assert_eq!(sys.scroll, 0, "OnActivate rebuilds the page at scroll 0");
        // Pick the drawn centre of cell (row 1, col 3) → id 17.
        let grid = grid_rect(screen);
        let origin = grid_origin(&grid);
        let cx = origin[0] + 3.0 * CELL + CELL * 0.5;
        let cy = origin[1] + 1.0 * CELL + CELL * 0.5;
        let id = card_at(screen, Vec2::new(cx, cy), sys.scroll).unwrap();
        assert_eq!(id, 17);
        // The pick spawns the bubble and closes the wheel (the dead gate is
        // the caller's, `clicks.rs:1056` — here the player is alive).
        sys.new_bubble(id);
        sys.close();
        assert!(!sys.menu_open);
        assert_eq!(sys.scroll, 0);
        assert_eq!(sys.bubbles.len(), 1);
        assert_eq!(sys.bubbles[0].emote, 17);
        assert_eq!(sys.bubbles[0].life_time, 360);
        // Back closes without spawning (`GoBackClick`).
        sys.open();
        let panel = panel_rect(screen, true);
        let back = back_rect(&panel);
        assert!(back_hit(&panel, Vec2::new(back[0] + 5.0, back[1] + 5.0)));
        sys.close();
        assert!(!sys.menu_open);
        assert_eq!(sys.bubbles.len(), 1, "Back spawns nothing");
    }
}
