//! The emote draw — `EmoteBubble.Draw` (`:209-251`) and the
//! `UIEmotesMenu` wheel: the seven `EmotesGroupListItem` category lists
//! (`UIEmotesMenu.InitializePage` `:90-97`) stacked in the `UIList` with
//! `ListPadding 14` (`:67`), the pixel `UIScrollbar` (`:70-77`) and the
//! Back panel (`:78-88`). W4-g54 replaces the flat 151-roster stand-in
//! with the C# grouped layout; the rosters + downed gates live in
//! `crate::emotes::build_groups`.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::push_whole;
use super::uistate::{self, Style};
use super::{color, panel, text};
use crate::assets::Lookups;
use crate::emotes::EmoteGroup;

/// `EmotesGroupListItem.SIZE_PER_EMOTE` (`EmotesGroupListItem.cs:15`) — the
/// `EmoteButton` cell is the sheet's 36x36 frame (`EmoteButton.cs:32-36`), the
/// port's old 44 px stand-in.
pub const CELL: f32 = 36.0;
/// `EmotesGroupListItem._maxEmotesPerRow` (`:21`), hard-overridden to 14 in the
/// ctor (`:25`) — the per-group caps `TryAddingList` passes (10/10/11/8/9/7/8,
/// `UIEmotesMenu.cs:91-97`) are dead.
pub const COLS: usize = 14;
/// `UIList.ListPadding = 14f` (`UIEmotesMenu.cs:67`).
pub const LIST_PADDING: f32 = 14.0;

/// `TITLE_HEIGHT = 20` / `SEPARATOR_HEIGHT = 10` (`EmotesGroupListItem.cs:11-13`)
/// — declared but unused by the ctor, which lays out with 30/36 (`:35`, `:79`).
#[allow(dead_code)] // the C# consts are dead too; kept for the cite
pub const GROUP_TITLE_HEIGHT: f32 = 20.0;
#[allow(dead_code)]
pub const GROUP_SEPARATOR_HEIGHT: f32 = 10.0;

/// `UI.Back` — the Back panel's label key (`UIEmotesMenu.cs:78`'s
/// `UITextPanel<LocalizedText>(Language.GetText("UI.Back"), …)`), the same key
/// the bestiary (`UIBestiaryTest.cs:366`) and achievements panels read.
pub const BACK_KEY: &str = "UI.Back";

/// `UIEmotesMenu.OnActivate`'s outer container (`UIEmotesMenu.cs:30-44`,
/// `:45-52`): a **590 px wide** box — centred, `Top 220 / Height −220` in the
/// menu and `Top 120 / Height −120` in-game (`:39-41`'s else arm). The port
/// reaches the wheel only from the in-game inventory (`clicks.rs`), so the
/// in-game branch is the live one; the menu split stays for parity.
pub fn panel_rect(screen: Vec2, in_game: bool) -> [f32; 4] {
    let (top, height) = if in_game {
        (120.0, Style::new(-120.0, 1.0))
    } else {
        (220.0, Style::new(-220.0, 1.0))
    };
    uistate::resolve(
        super::state_root::root_rect([0.0, 0.0, screen.x, screen.y]), // the `UIState` root (`UIState.cs:9-11`)
        Style::px(0.0),                // `Left` unset
        Style::px(top),                // `_outerContainer.Top` (`:39`/`:41`)
        Style::px(590.0),              // `Width.Set(590f, 0f)` (`:49`)
        height,                        // `Height.Set(-120f/-220f, 1f)` (`:40`/`:42`)
        0.5,                           // `HAlign = 0.5f` (`:52`)
        0.0,
        0.0,
        0.0,
        590.0,
        [0.0; 4],
    )
}

/// The `_container` `UIPanel` (`UIEmotesMenu.cs:55-61`): `Width (0, 1)`,
/// `Height (-110, 1)` of the outer container, `BackgroundColor
/// (33, 43, 79) * 0.8`, `PaddingTop = 0` over the `UIPanel`'s 12 px
/// `SetPadding` (`UIPanel.cs:32`).
pub fn container_rect(panel: &[f32; 4]) -> [f32; 4] {
    [panel[0], panel[1], panel[2], panel[3] - 110.0]
}

/// The container's inner rect — the 12 px corner padding with `PaddingTop`
/// zeroed (`UIEmotesMenu.cs:59`, `UIElement.cs:298-302`).
pub fn container_inner(panel: &[f32; 4]) -> [f32; 4] {
    uistate::inner(container_rect(panel), [12.0, 0.0, 12.0, 12.0])
}

/// The `UIList` (`UIEmotesMenu.cs:62-69`): `Width (-25, 1)` while the
/// scrollbar is attached, `(0, 1)` otherwise (`Recalculate` `:285-303`),
/// `Height (-50, 1)`, `Top 50`, `HAlign 0.5` — resolved against the
/// container's INNER rect.
pub fn list_rect(panel: &[f32; 4], scrollbar_attached: bool) -> [f32; 4] {
    let inner = container_inner(panel);
    let width = Style::new(if scrollbar_attached { -25.0 } else { 0.0 }, 1.0);
    uistate::resolve(
        inner,
        Style::px(0.0),
        Style::px(50.0),
        width,
        Style::new(-50.0, 1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        inner[2],
        [0.0; 4],
    )
}

/// The `UIScrollbar` (`UIEmotesMenu.cs:70-77`): `Width 20`
/// (`UIScrollbar.cs:59`), `Height (-20, 1)`, `HAlign 1`, `VAlign 1`,
/// `Top -5`, a child of the `_container` panel — resolved against the
/// container's inner rect.
pub fn scrollbar_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = container_inner(panel);
    uistate::resolve(
        inner,
        Style::px(0.0),
        Style::px(-5.0),
        Style::px(20.0),
        Style::new(-20.0, 1.0),
        1.0,
        1.0,
        0.0,
        0.0,
        20.0, // `MaxWidth.Set(20f, 0f)` (`UIScrollbar.cs:60`)
        [0.0; 4],
    )
}

/// The ctor's row count (`EmotesGroupListItem.cs:30-34`):
/// `emotes.Length / 14`, `+1` when the remainder is non-zero.
pub fn group_row_count(emote_count: usize) -> usize {
    let mut rows = emote_count / COLS;
    if emote_count % COLS != 0 {
        rows += 1;
    }
    rows
}

/// The group item's height (`EmotesGroupListItem.cs:35`):
/// `Height.Set(30 + 36 * rows, 0f)` — the 30 px header band plus the rows.
pub fn group_height(emote_count: usize) -> f32 {
    30.0 + CELL * group_row_count(emote_count) as f32
}

/// `UIList.RecalculateChildren` (`UIList.cs:116-128`): the items stack at
/// cumulative `Top`s with `ListPadding` after EACH item — including the
/// last — when there is more than one (`num2 = count == 1 ? 0 : ListPadding`,
/// `:122`); `_innerListHeight` is the running total (`:127`).
pub fn list_content_height(groups: &[EmoteGroup]) -> f32 {
    let pad = if groups.len() == 1 { 0.0 } else { LIST_PADDING };
    groups
        .iter()
        .map(|g| group_height(g.emotes.len()) + pad)
        .sum()
}

/// Group `i`'s `Top` inside the inner list (the `RecalculateChildren`
/// accumulation, `UIList.cs:119-126`).
pub fn group_top(groups: &[EmoteGroup], i: usize) -> f32 {
    let pad = if groups.len() == 1 { 0.0 } else { LIST_PADDING };
    groups[..i]
        .iter()
        .map(|g| group_height(g.emotes.len()) + pad)
        .sum()
}

/// The scroll range — `UIScrollbar.ViewPosition`'s clamp
/// (`UIScrollbar.cs:45`: `maxViewSize - viewSize`), with `SetView`'s
/// `viewSize = Clamp(viewSize, 0, maxViewSize)` (`:74`) folding a
/// shorter-than-viewport list to 0.
pub fn max_scroll(groups: &[EmoteGroup], list: &[f32; 4]) -> f32 {
    (list_content_height(groups) - list[3]).max(0.0)
}

/// `UIEmotesMenu.Recalculate`'s attach machine (`UIEmotesMenu.cs:285-303`):
/// the scrollbar attaches iff `CanScroll` (`UIScrollbar.cs:49`:
/// `_maxViewSize != _viewSize` — after `SetView`'s clamp that is
/// `content > viewport`), and the list width follows.
pub fn scrollbar_attaches(groups: &[EmoteGroup], panel: &[f32; 4]) -> bool {
    let full = list_rect(panel, false);
    list_content_height(groups) > full[3]
}

/// Group `i`'s rect on screen — the inner-list `Top` lifted by the
/// scrollbar (`UIList.DrawSelf` `:179-182`: `_innerList.Top = -GetValue()`),
/// `Width (0, 1)` of the list (`EmotesGroupListItem.cs:36`).
pub fn group_rect(list: &[f32; 4], groups: &[EmoteGroup], i: usize, scroll: f32) -> [f32; 4] {
    [
        list[0],
        list[1] + group_top(groups, i) - scroll,
        list[2],
        group_height(groups[i].emotes.len()),
    ]
}

/// The header container (`EmotesGroupListItem.cs:37-42`): `Height 30`,
/// `Width (-20, 1)`, `HAlign 0.5` — 10 px in from each side of the group item.
/// The separator (`VAlign 1`, `:45-51`) and the title (`VAlign 1`, `Top -6`,
/// `:52-57`) hang off its bottom edge.
pub fn group_header_rect(group: &[f32; 4]) -> [f32; 4] {
    [group[0] + 10.0, group[1], group[2] - 20.0, 30.0]
}

/// The header separator's tint (`EmotesGroupListItem.cs:50`):
/// `Color.Lerp(Color.White, new Color(63, 65, 151), 0.85f) * 0.9f` — the
/// creation screens' separator colour (`uistate::CREATION_SEPARATOR_COLOR`).
pub fn group_separator_color() -> [u8; 4] {
    let lerped = super::char_creation::lerp_rgb([255, 255, 255, 255], [63, 65, 151, 255], 0.85);
    color::mul(lerped, 0.9)
}

/// Emote `i`'s button offset inside the group item
/// (`EmotesGroupListItem.cs:58-81`): `Top = 30 + row*36 + 6` (`num2 = 6f`,
/// `:58`/`:79`), `Left = 36*col - num6` with `num6` hard-overwritten to `-16f`
/// (`:74`) — the `num5` centring computation (`:64-73`) is dead code in the
/// C#, every row starts 16 px in. Returns `[top, left]`.
pub fn group_button_top_left(i: usize) -> [f32; 2] {
    let row = (i / COLS) as f32; // `num3` (`:62`)
    let col = (i % COLS) as f32; // `num4` (`:63`)
    [30.0 + row * CELL + 6.0, col * CELL + 16.0]
}

/// The `EmoteButton`'s own rect — `Width/Height = GetFrame().Width/Height`
/// (`EmoteButton.cs:27-29`): the 34x28 `Extra_48` frame at the group-relative
/// offset, `HAlign/VAlign 0` (`EmotesGroupListItem.cs:77-78`).
pub fn button_rect(group: &[f32; 4], i: usize) -> [f32; 4] {
    let [top, left] = group_button_top_left(i);
    [group[0] + left, group[1] + top, 34.0, 28.0]
}

/// `CompareTo` (`EmotesGroupListItem.cs:87-94`): groups sort by `_groupIndex`
/// (`UIList.UpdateOrder`'s `Sort`, `UIList.cs:145-156`). `build_groups`
/// already emits them in index order; the comparator is the pin.
pub fn compare_group_order(a: i32, b: i32) -> std::cmp::Ordering {
    a.cmp(&b)
}

/// The emote id under the cursor — the `GetElementAt` walk
/// (`UIElement.cs:324-352`): the point must be inside the `UIList`'s own
/// rect to descend (the parent chain gates), then inside a group item, then
/// inside the 34x28 `EmoteButton` rect (`ContainsPoint` `:354-357`, the
/// truncated `Rectangle.Contains`). A button straddling the list's edge is
/// hittable in its visible part — the scissor clips the DRAW, not the hit.
pub fn card_at(screen: Vec2, mouse: Vec2, groups: &[EmoteGroup], scroll: f32) -> Option<i32> {
    let panel = panel_rect(screen, true);
    let attached = scrollbar_attaches(groups, &panel);
    let list = list_rect(&panel, attached);
    if mouse.x < list[0] || mouse.x >= list[0] + list[2] {
        return None;
    }
    if mouse.y < list[1] || mouse.y >= list[1] + list[3] {
        return None;
    }
    for (i, g) in groups.iter().enumerate() {
        let gr = group_rect(&list, groups, i, scroll);
        if mouse.y < gr[1] || mouse.y >= gr[1] + gr[3] {
            continue;
        }
        for (j, &id) in g.emotes.iter().enumerate() {
            let br = button_rect(&gr, j);
            // `Rectangle.Contains(point.ToPoint())` — the int-truncated rect.
            let bx = br[0] as i32;
            let by = br[1] as i32;
            let mx = mouse.x as i32;
            let my = mouse.y as i32;
            if mx >= bx && mx < bx + br[2] as i32 && my >= by && my < by + br[3] as i32 {
                return Some(id);
            }
        }
    }
    None
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

// (W4-d81's census copy of the group geometry helpers — `GROUP_TITLE_HEIGHT`,
// `GROUP_SEPARATOR_HEIGHT`, `group_row_count`, `group_height`,
// `group_header_rect`, `group_separator_color`, `group_button_top_left`,
// `compare_group_order` — deduped into the g54 definitions above; the bodies
// were term-identical.)

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
/// (`ui::resources::draw_breath`).
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
    // The wheel (`UIEmotesMenu`): the container panel + the grouped list.
    if !emotes.menu_open {
        return;
    }
    let panel = panel_rect(screen, true);
    let container = container_rect(&panel);
    // `UIPanel.DrawSelf` (`UIPanel.cs:68-77`): the 9-slice background then
    // border; `BackgroundColor = new Color(33, 43, 79) * 0.8f`
    // (`UIEmotesMenu.cs:58`), border the `UIPanel` default black.
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_bg,
        container,
        color::tint(color::mul([33, 43, 79, 255], 0.8)),
    );
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        container,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );
    let groups = &emotes.groups;
    let attached = scrollbar_attaches(groups, &panel);
    let list = list_rect(&panel, attached);
    let hovered = card_at(screen, mouse, groups, emotes.scroll);
    // The `UIList`'s `OverflowHidden` scissor (`UIList.cs:59`,
    // `UIElement.cs:222-237`): the groups draw shifted by the scrollbar and
    // clip at the list's edges.
    r.push_clip(list);
    for (i, g) in groups.iter().enumerate() {
        let gr = group_rect(&list, groups, i, emotes.scroll);
        // `UIInnerList.DrawChildren`'s AABB gate (`UIList.cs:27`): a group
        // fully off the list draws nothing.
        if gr[1] + gr[3] <= list[1] || gr[1] >= list[1] + list[3] {
            continue;
        }
        draw_group(r, lookups, &gr, g, emotes, hovered);
    }
    r.pop_clip();
    // The scrollbar (`UIEmotesMenu.Recalculate`'s attach, `:285-303`):
    // `SetView(listInnerHeight, innerListHeight)` (`UIList.cs:130-136`),
    // drawn at the live `_viewPosition`.
    if attached {
        let sb = scrollbar_rect(&panel);
        panel::draw_scrollbar(
            r,
            lookups,
            sb,
            list[3],
            list_content_height(groups),
            emotes.scroll,
            false,
        );
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
    // **`Top(-45)`** — the wheel's own -45, not the bestiary's -25. The label
    // goes through `lang::text` like the bestiary's and the achievements'
    // panels do, never a literal.
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

/// One `EmotesGroupListItem`: the header separator + localized title, then
/// the `EmoteButton` lattice (`EmotesGroupListItem.cs:37-84`).
fn draw_group(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    gr: &[f32; 4],
    g: &EmoteGroup,
    emotes: &crate::emotes::EmoteSystem,
    hovered: Option<i32>,
) {
    let header = group_header_rect(gr);
    // The separator (`:45-51`): `Width (0, 1)`, `VAlign 1`, `HAlign 0.5` —
    // the header's bottom edge, the sheet's own 4 px height
    // (`UIHorizontalSeparator.cs:28`).
    panel::draw_horizontal_separator(
        r,
        lookups,
        header[0],
        header[1] + header[3] - 4.0,
        header[2],
        color::tint(group_separator_color()),
    );
    // The title (`:52-57`): `UIText(groupTitle)` — scale 1, small
    // (`UIText.cs:82`), `VAlign 1`, `HAlign 0.5`, `Top -6`; `MinWidth =
    // textSize.X`, `MinHeight = textSize.Y = 16` (`UIText.cs:186-189`).
    // `DrawSelf` (`:126-159`): `position.Y -= 2*scale`, `TextOriginX 0.5`
    // centres against the (text-width) box, `TextOriginY 0`.
    let title = crate::lang::text(g.title_key);
    let tw = text::hud_text_width(r, lookups, &title, 1.0);
    let tx = header[0] + (header[2] - tw) * 0.5;
    let ty = header[1] + header[3] - 6.0 - 16.0 - 2.0;
    text::border_string(r, lookups, tx, ty, [1.0; 4], &title);
    // The buttons.
    let Some(sheet) = lookups.ui_emote_sheet else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    // The button's true frame size (`EmoteButton`'s ctor `:27-29` reads
    // `GetFrame().Width/Height` — 34x28).
    let sw = (sheet.width / 8) as f32;
    let sh = (sheet.height / crate::emotes::SHEET_ROWS as u32) as f32;
    for (j, &id) in g.emotes.iter().enumerate() {
        let br = button_rect(gr, j);
        // `DrawSelf` (`EmoteButton.cs:52-75`): `vector = Position + Size/2`.
        let cx = br[0] + br[2] * 0.5;
        let cy = br[1] + br[3] * 0.5;
        let hover = hovered == Some(id);
        // The shell frame (column 1, row 0 — `:58-59`), the emote frame,
        // then the border — black idle, `Main.OurFavoriteColor` (255,231,69,
        // `Main.cs:66748`) hovered, at `vector - One*2` (`:69`).
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
            // The border origin is the EMOTE FRAME's half-size (17,14), not
            // the border's own (18,15): `EmoteButton.cs:60`
            // `origin = frame.Size()/2`, drawn at `vector - One*2` (`:69`)
            // -> top-left = center - (19,16).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emotes::{EmoteGates, EmoteSystem};

    /// The fresh-world page: 6 groups (Bosses empty), the C# rosters.
    fn fresh_groups() -> Vec<EmoteGroup> {
        crate::emotes::build_groups(&EmoteGates::default())
    }

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
    fn g54_the_in_game_container_is_the_c_sharp_split() {
        // `UIEmotesMenu.cs:39-41`'s in-game arm: Top 120, Height -120, and
        // `InitializePage`'s 590-wide box (`:49`) — centred at 1600: x = 505.
        let p = panel_rect(Vec2::new(1600.0, 900.0), true);
        assert_eq!(p, [505.0, 120.0, 590.0, 780.0]);
        // 1280x720: the same box, height 720 - 120 = 600.
        let p = panel_rect(Vec2::new(1280.0, 720.0), true);
        assert_eq!(p, [345.0, 120.0, 590.0, 600.0]);
        // The menu arm (`:35-36`): Top 220 / Height -220.
        let p = panel_rect(Vec2::new(1280.0, 720.0), false);
        assert_eq!(p, [345.0, 220.0, 590.0, 500.0]);
        // The container drops 110 (`:57`), the inner rect keeps the 12 px
        // sides/bottom with PaddingTop 0 (`:59`).
        let c = container_rect(&p);
        assert_eq!(c, [345.0, 220.0, 590.0, 390.0]);
        assert_eq!(container_inner(&p), [357.0, 220.0, 566.0, 378.0]);
    }

    #[test]
    fn g54_the_group_geometry_is_the_ctor_lattice() {
        // `EmotesGroupListItem.cs:30-35`: 29 emotes → 3 rows → 30 + 108 = 138;
        // 6 → 1 row → 66; 14 → exactly 1 row (no remainder bump).
        assert_eq!(group_row_count(29), 3);
        assert_eq!(group_height(29), 138.0);
        assert_eq!(group_row_count(6), 1);
        assert_eq!(group_height(6), 66.0);
        assert_eq!(group_row_count(14), 1);
        assert_eq!(group_height(14), 66.0);
        assert_eq!(group_row_count(15), 2);
        // The button offsets (`:58-81`): i=0 → (36, 16); i=14 wraps to
        // row 1 → (72, 16); i=17 → (72, 124). The dead num6 walk pins -16.
        assert_eq!(group_button_top_left(0), [36.0, 16.0]);
        assert_eq!(group_button_top_left(14), [72.0, 16.0]);
        assert_eq!(group_button_top_left(17), [72.0, 124.0]);
        // The header band: 10 px in, 30 tall (`:37-42`).
        let gr = [100.0, 200.0, 541.0, 138.0];
        assert_eq!(group_header_rect(&gr), [110.0, 200.0, 521.0, 30.0]);
        // The separator tint is the creation screens' Lerp*0.9 pair.
        assert_eq!(group_separator_color(), uistate::CREATION_SEPARATOR_COLOR);
        // CompareTo sorts by group index (`:87-94`).
        assert_eq!(compare_group_order(0, 1), std::cmp::Ordering::Less);
        assert_eq!(compare_group_order(2, 2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn g54_the_list_stacks_with_padding_14_and_a_trailing_pad() {
        // `UIList.RecalculateChildren` (`UIList.cs:119-127`): the fresh page
        // is General 138 / RPS 66 / Items 102 / Biomes 102 / Town 138 /
        // Critters 66 — tops 0, 152, 232, 348, 464, 616; the running total
        // keeps the LAST item's 14 px pad: 696.
        let groups = fresh_groups();
        assert_eq!(groups.len(), 6);
        let heights: Vec<f32> = groups.iter().map(|g| group_height(g.emotes.len())).collect();
        assert_eq!(heights, vec![138.0, 66.0, 102.0, 102.0, 138.0, 66.0]);
        let tops: Vec<f32> = (0..groups.len()).map(|i| group_top(&groups, i)).collect();
        assert_eq!(tops, vec![0.0, 152.0, 232.0, 348.0, 464.0, 616.0]);
        assert_eq!(list_content_height(&groups), 696.0);
        // A single group carries NO padding (`num2 = count == 1 ? 0`,
        // `UIList.cs:122`).
        let one = vec![groups[1].clone()];
        assert_eq!(list_content_height(&one), 66.0);
    }

    #[test]
    fn g54_the_scrollbar_attach_machine_and_pixel_range() {
        // At 1280x720: container inner (357, 120, 566, 478); the unattached
        // list is (357, 170, 566, 428); 696 > 428 → the scrollbar attaches
        // and the list narrows to 541, centred: x = 369.5.
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen, true);
        let groups = fresh_groups();
        assert!(scrollbar_attaches(&groups, &panel));
        let list = list_rect(&panel, true);
        assert_eq!(list, [369.5, 170.0, 541.0, 428.0]);
        // `ViewPosition` clamps to maxView - view (`UIScrollbar.cs:45`).
        assert_eq!(max_scroll(&groups, &list), 268.0);
        // The scrollbar box: HAlign 1 / VAlign 1 / Top -5 of the inner rect
        // (`UIEmotesMenu.cs:72-75`): x = 357 + 566 - 20 = 903,
        // y = 120 + 478 - 5 - 458 = 135, h = 478 - 20 = 458.
        assert_eq!(scrollbar_rect(&panel), [903.0, 135.0, 20.0, 458.0]);
        // A page that fits keeps the full-width list and no scrollbar.
        let one = vec![groups[1].clone()]; // RPS, 66 tall
        assert!(!scrollbar_attaches(&one, &panel));
        assert_eq!(list_rect(&panel, false), [357.0, 170.0, 566.0, 428.0]);
        assert_eq!(max_scroll(&one, &list_rect(&panel, false)), 0.0);
    }

    #[test]
    fn g54_every_roster_emote_is_hittable_at_its_drawn_rect() {
        // The draw and the hit share the group lattice: walk every button
        // of the fresh page at scroll 0 and at the bottom clamp, and pin
        // the centre of each drawn rect back to its id.
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen, true);
        let groups = fresh_groups();
        let list = list_rect(&panel, true);
        for scroll in [0.0, max_scroll(&groups, &list)] {
            for (i, g) in groups.iter().enumerate() {
                let gr = group_rect(&list, &groups, i, scroll);
                for (j, &id) in g.emotes.iter().enumerate() {
                    let br = button_rect(&gr, j);
                    let centre = Vec2::new(br[0] + br[2] * 0.5, br[1] + br[3] * 0.5);
                    // Only the part inside the list scissor is reachable
                    // (the `GetElementAt` parent-chain gate).
                    if centre.y < list[1] || centre.y >= list[1] + list[3] {
                        continue;
                    }
                    assert_eq!(
                        card_at(screen, centre, &groups, scroll),
                        Some(id),
                        "id {id} group {i} cell {j} scroll {scroll}"
                    );
                }
            }
        }
    }

    #[test]
    fn g54_the_hit_walk_gates_on_the_list_and_the_button_rect() {
        let screen = Vec2::new(1280.0, 720.0);
        let groups = fresh_groups();
        let panel = panel_rect(screen, true);
        let list = list_rect(&panel, true);
        // General's first button (id 0): group 0 at (369.5, 170), offset
        // (16, 36) → rect (385.5, 206, 34, 28).
        let br = button_rect(&group_rect(&list, &groups, 0, 0.0), 0);
        assert_eq!(br, [385.5, 206.0, 34.0, 28.0]);
        assert_eq!(card_at(screen, Vec2::new(386.0, 207.0), &groups, 0.0), Some(0));
        // Just outside the truncated rect (x 385..419 — `ToRectangle()`
        // truncates 385.5 to 385, so 385 itself is INSIDE) hits nothing.
        assert_eq!(card_at(screen, Vec2::new(384.0, 207.0), &groups, 0.0), None);
        assert_eq!(card_at(screen, Vec2::new(420.0, 207.0), &groups, 0.0), None);
        // The header band is not a button.
        assert_eq!(card_at(screen, Vec2::new(400.0, 180.0), &groups, 0.0), None);
        // Outside the list rect nothing descends (`GetElementAt`'s chain).
        assert_eq!(card_at(screen, Vec2::new(360.0, 207.0), &groups, 0.0), None);
        assert_eq!(card_at(screen, Vec2::new(400.0, 160.0), &groups, 0.0), None);
        // Scroll lifts the lattice: at scroll 36 the first button's rect
        // moves up 36 and the point that was its centre now reads the row
        // below's first cell (General row 1 col 0 = roster index 14 = id 87).
        assert_eq!(card_at(screen, Vec2::new(400.0, 220.0), &groups, 36.0), Some(87));
    }

    #[test]
    fn g54_the_scroll_clamps_in_pixels_and_resets_on_open() {
        // `UIList.ScrollWheel` (`:107-114`): ViewPosition -= delta; the
        // setter clamps to [0, maxView - view] (`UIScrollbar.cs:45`).
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen, true);
        let mut e = EmoteSystem::default();
        e.open();
        assert_eq!(e.scroll, 0.0);
        let max = max_scroll(&e.groups, &list_rect(&panel, true));
        assert_eq!(max, 268.0);
        // Wheel-up (XNA delta +120) at the top stays 0 (`-=` walks back).
        e.scroll_by(120.0, max);
        assert_eq!(e.scroll, 0.0);
        // One notch DOWN (delta −120) scrolls 120 px, then past the end
        // clamps at 268 (`UIScrollbar.cs:45`).
        e.scroll_by(-120.0, max);
        assert_eq!(e.scroll, 120.0);
        e.scroll_by(-1000.0, max);
        assert_eq!(e.scroll, 268.0);
        // Each open rebuilds the page at scroll 0 (`OnActivate`).
        e.open();
        assert_eq!(e.scroll, 0.0);
        e.close();
        assert!(!e.menu_open);
        assert_eq!(e.scroll, 0.0);
        // The track jump shares the clamp (`scroll_to`).
        e.scroll_to(9999.0, max);
        assert_eq!(e.scroll, 268.0);
        e.scroll_to(-5.0, max);
        assert_eq!(e.scroll, 0.0);
    }

    #[test]
    fn g54_the_back_row_hangs_off_the_wheel_container() {
        // The wheel's Back is half the *wheel* container's width and hangs
        // at its own `Top(-45)` (`UIEmotesMenu.cs:83`) — not the bestiary's
        // -25 (`UIBestiaryTest.cs:371`).
        let panel = panel_rect(Vec2::new(1600.0, 900.0), true);
        let back = back_rect(&panel);
        assert_eq!(back[2], (590.0 - 10.0) * 0.5);
        assert_eq!(back[3], 50.0);
        assert_eq!(back[1], panel[1] + panel[3] - 95.0); // -45 top, 50 tall
        assert_eq!(back[0] + back[2] * 0.5, panel[0] + panel[2] * 0.5);
        assert!(back_hit(&panel, Vec2::new(back[0] + 10.0, back[1] + 10.0)));
        assert!(!back_hit(&panel, Vec2::new(back[0] - 10.0, back[1] + 10.0)));
        let bestiary = crate::ui::bestiary::back_rect(&panel);
        assert_eq!(bestiary[1] - back[1], 20.0);
    }

    #[test]
    fn the_bubble_anchor_is_the_c_sharp_entity_arm() {
        // `GetPosition` (`EmoteBubble.cs:247-249`).
        let (pos, flip) = bubble_anchor(1000.0, 2000.0, 20.0, 1);
        assert_eq!(pos, [1000.0 + 10.0 - 15.0, 2002.0]);
        assert!(flip);
        let (pos, flip) = bubble_anchor(1000.0, 2000.0, 20.0, -1);
        assert_eq!(pos, [1000.0 + 10.0 + 15.0, 2002.0]);
        assert!(!flip);
    }

    #[test]
    fn bug_18_the_back_label_is_the_ui_back_localized_text() {
        assert_eq!(BACK_KEY, "UI.Back");
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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

    #[test]
    fn the_border_origin_is_the_emote_frame_half_size() {
        // `EmoteButton.DrawSelf` (`:60`, `:69`): origin = frame.Size()/2
        // (34x28 → (17,14)), drawn at `vector - One*2` → centre − (19,16).
        let (frame_w, frame_h) = (34.0f32, 28.0f32);
        let origin = [frame_w * 0.5, frame_h * 0.5];
        assert_eq!(origin, [17.0, 14.0]);
        let centre = [100.0f32, 200.0f32];
        let top_left = [centre[0] - 2.0 - origin[0], centre[1] - 2.0 - origin[1]];
        assert_eq!(top_left, [81.0, 184.0], "centre − (19,16)");
    }

    #[test]
    fn g54_the_wheel_opens_picks_and_closes() {
        // The full flow: the button click opens the wheel (`OnActivate`
        // rebuild, scroll 0), a cell pick fires `EmoteButton.LeftClick`
        // (`:90-95`) → `MakeLocalPlayerEmote` (the 360-tick bubble) +
        // `IngameFancyUI.Close`, Back just closes (`GoBackClick` `:305-309`).
        let screen = Vec2::new(1280.0, 720.0);
        let mut sys = EmoteSystem::default();
        assert!(!sys.menu_open);
        sys.open();
        assert!(sys.menu_open);
        assert_eq!(sys.scroll, 0.0);
        // General's cell (row 1, col 3) = roster index 17 = id 91 (EmoteRun).
        let panel = panel_rect(screen, true);
        let list = list_rect(&panel, true);
        let gr = group_rect(&list, &sys.groups, 0, 0.0);
        let br = button_rect(&gr, 17);
        let id = card_at(screen, Vec2::new(br[0] + 17.0, br[1] + 14.0), &sys.groups, 0.0).unwrap();
        assert_eq!(id, 91);
        sys.new_bubble(id);
        sys.close();
        assert!(!sys.menu_open);
        assert_eq!(sys.bubbles.len(), 1);
        assert_eq!(sys.bubbles[0].emote, 91);
        assert_eq!(sys.bubbles[0].life_time, 360);
        // Back closes without spawning.
        sys.open();
        let back = back_rect(&panel);
        assert!(back_hit(&panel, Vec2::new(back[0] + 5.0, back[1] + 5.0)));
        sys.close();
        assert_eq!(sys.bubbles.len(), 1, "Back spawns nothing");
    }

    #[test]
    fn g54_the_downed_gates_grow_the_critters_and_bosses_lists() {
        // `GetEmotesCritters` (`:153-176`) / `GetEmotesBosses` (`:178-283`).
        let fresh = fresh_groups();
        assert_eq!(fresh.len(), 6, "no bosses downed → the Bosses list is skipped");
        assert_eq!(fresh[5].emotes, vec![12, 13, 61, 62, 63, 67, 68, 69, 70, 72]);
        let mut gates = EmoteGates {
            goblins: true,
            frost: true,
            pirates: true,
            martians: true,
            ..Default::default()
        };
        let g = crate::emotes::build_groups(&gates);
        assert_eq!(g[5].emotes, vec![12, 13, 61, 62, 63, 67, 68, 69, 70, 72, 64, 66, 65, 71]);
        // Pirates + martians also feed the Bosses tail (59, 60).
        assert_eq!(g[6].emotes, vec![59, 60]);
        assert_eq!(g[6].index, 6);
        // The full ladder in the C# if-order.
        gates = EmoteGates {
            boss1: true,
            boss2: true,
            slime_king: true,
            deerclops: true,
            queen_bee: true,
            boss3: true,
            hard_mode: true,
            queen_slime: true,
            mech1: true,
            mech3: true,
            mech2: true,
            plant: true,
            golem: true,
            fishron: true,
            empress: true,
            cultist: true,
            moonlord: true,
            halloween_tree: true,
            halloween_king: true,
            christmas_tree: true,
            christmas_ice_queen: true,
            christmas_santank: true,
            pirates: true,
            martians: true,
            dd2_any: true,
            ..Default::default()
        };
        let g = crate::emotes::build_groups(&gates);
        assert_eq!(
            g[6].emotes,
            vec![
                39, 40, 41, 51, 150, 42, 43, 44, 144, 45, 46, 47, 48, 49, 50, 143, 52, 53,
                54, 55, 56, 57, 58, 59, 60, 133,
            ]
        );
        // Every id the seven lists can show is a valid emote; the five
        // ungated lists are 29+6+23+23+30 = 111 ids.
        let fixed: usize = g[..5].iter().map(|x| x.emotes.len()).sum();
        assert_eq!(fixed, 111);
        for group in &g {
            for &id in &group.emotes {
                assert!((0..crate::emotes::EMOTE_COUNT).contains(&id));
            }
        }
    }
}
