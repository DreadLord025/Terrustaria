//! The in-world mouse-over pass — `Main.DrawMouseOver` (`Main.cs:42871-42988`,
//! interface layer 39 `DrawInterface_39_MouseOver`, `:44904-44908`) and its
//! NPC half `Main.HoverOverNPCs` (`:42990-43114`). W4-g53 doc-sweep lane 10.
//!
//! Two hover texts, both through `MouseTextHackZoom` → the plain
//! `MouseTextInner` arm (`Main.cs:20182-20319` — NO box, the box is the
//! `HoverItem.type > 0` tooltip path, `:20213-20216`):
//! - a dropped item under the pointer prints `AffixName + " (stack)"` in
//!   its rare colour (`:42884-42913`);
//! - an NPC under the pointer prints `GivenOrTypeName` plus
//!   `": life/lifeMax"` while the carrier shows life (`:43087-43104`).
//!
//! The `Main.mouseText` priority chain (`:45709` clears it at layer 24,
//! `:45188` sets it for a carried mouse item, `:42884` gates the item walk
//! on `!mouseText`, `:43087` the NPC text on `flag && !mouseText`): the
//! port's `draw_hud` slot tooltips / PVP / housing texts are the earlier
//! claimants, so this pass runs only when none of them drew.
//!
//! Unported halves (documented): the stealthed-remote-player hover
//! (`:42914-42936`, no remote stealth state), the sign hover
//! (`:42940-42986`, claim f22b), the elder-slime transform arm
//! (`:43020-43045`, no kind 685), the `cursorItemIcon*` interact-icon
//! family (`:42897`, `:43024-43026`, `:43089` — the whole
//! `DrawInterface_40_InteractItemIcon` layer is unported), and the
//! smart-interact / stinky / projectile-651 / mouseInterface gates of the
//! talk arm (`:43048-43064`) — the talk route itself already lives in
//! `world_sim::npcs::talk_to_npc`.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::item::ItemDrop;
use crate::npc::Npc;
use crate::ui::{text, tooltip, UiPulse};

/// `Main.cs:42874-42875`: the 1x1 world-pixel mouse rect
/// (`Main.MouseScreen + Main.screenPosition` under the world zoom).
pub fn mouse_world_point(mouse: Vec2, camera: Vec2, zoom: f32, screen: Vec2) -> Vec2 {
    Vec2::new(
        (mouse.x - screen.x * 0.5) / zoom + camera.x,
        (mouse.y - screen.y * 0.5) / zoom + camera.y,
    )
}

/// The dropped item's hover rect (`Main.cs:42893-42895`):
/// `Item.GetDrawHitbox(type)` bottom-centred on `item.Bottom`. The port's
/// `GetDrawHitbox` stand-in is the atlas icon quad (`item.rs::draw`'s
/// convention); with no icon the 16x16 `WorldItem` box stands in
/// (`WorldItem.cs:138` — `width = height = 16`), bottom-centred the same.
pub fn drop_hover_rect(drop: &ItemDrop, lookups: &assets::Lookups) -> [f32; 4] {
    let (w, h) = lookups
        .item_icon(drop.kind)
        .map(|(_, w, h)| (w, h))
        .unwrap_or((16.0, 16.0));
    // `item.Bottom` = position + (width/2, height) (`Entity.cs`), the
    // port's box top-left + SIZE.
    let bottom_x = drop.pos.x + crate::item::SIZE * 0.5;
    let bottom_y = drop.pos.y + crate::item::SIZE;
    [
        (bottom_x - w * 0.5) as i32 as f32,
        (bottom_y - h) as i32 as f32,
        w,
        h,
    ]
}

/// `Main.cs:42898-42907`: the hover text — `inner.AffixName()` plus
/// `" (stack)"` above one, and the rare colour (`expert` → −12's disco is
/// unmodelled; the port is single-difficulty normal, so `inner.rare`).
pub fn drop_hover_text(drop: &ItemDrop) -> (String, [f32; 4]) {
    let mut name = crate::lang::prefixed_item_name(drop.kind, drop.prefix);
    if drop.count > 1 {
        name = format!("{name} ({})", drop.count);
    }
    (name, tooltip::item_name_color(crate::registry::rare(drop.kind)))
}

/// The first dropped item whose hover rect contains the world mouse point
/// (`Main.cs:42886-42912`'s walk, `break` on the first hit).
pub fn hovered_drop<'a>(
    items: &'a [ItemDrop],
    lookups: &assets::Lookups,
    mouse_world: Vec2,
) -> Option<&'a ItemDrop> {
    items.iter().find(|it| {
        let r = drop_hover_rect(it, lookups);
        // `Rectangle.Intersects` (`:42895`) — the 1x1 mouse rect against
        // the frame box; the right/bottom edges are exclusive.
        mouse_world.x >= r[0]
            && mouse_world.x < r[0] + r[2]
            && mouse_world.y >= r[1]
            && mouse_world.y < r[1] + r[3]
    })
}

/// The NPC hover box (`Main.cs:43003`): the frame rect bottom-centred on
/// `nPC.Bottom` — `(int)Bottom.X - frame.Width/2, (int)Bottom.Y -
/// frame.Height`. The port's frame stand-in is `Npc::size()` (the
/// `bubble_npc` convention, `ui/npc_chat.rs`). The special boxes:
/// types 87-92 (the Scutlix/saucer family) a 64x64 centred box
/// (`:43004-43007`), 677 a 20x20 (`:43008-43011`), 695/696 a 60x40
/// bottom-offset box (`:43012-43015`) — none of those kinds ship, so the
/// arms ride the net id for when they do.
pub fn npc_hover_rect(n: &Npc) -> [f32; 4] {
    let (w, h) = n.size();
    let id = n.kind.net_id();
    if (87..=92).contains(&id) {
        // `:43006` — centre + (width/2 − 32, height/2 − 32), 64x64.
        return [
            (n.x + w * 0.5 - 32.0) as i32 as f32,
            (n.y + h * 0.5 - 32.0) as i32 as f32,
            64.0,
            64.0,
        ];
    }
    if id == 677 {
        // `Utils.CenteredRectangle(nPC.Center, (20, 20))`.
        return [
            (n.x + w * 0.5 - 10.0) as i32 as f32,
            (n.y + h * 0.5 - 10.0) as i32 as f32,
            20.0,
            20.0,
        ];
    }
    if id == 696 || id == 695 {
        // `Utils.CenteredRectangle(Bottom - (direction * -10, 20), (60, 40))`.
        let bx = n.x + w * 0.5 - (n.facing as f32 * -10.0);
        let by = n.y + h - 20.0;
        return [
            (bx - 30.0) as i32 as f32,
            (by - 20.0) as i32 as f32,
            60.0,
            40.0,
        ];
    }
    [
        (n.x + w * 0.5) as i32 as f32 - (w as i32 / 2) as f32,
        (n.y + h) as i32 as f32 - h,
        w,
        h,
    ]
}

/// `Main.cs:43018`: the hover-text exclusion gate — the buried mimics
/// (85/341/629) and aiStyle 87 hide while `ai[0] == 0`, 694 while
/// `ai[3] == 3`, and 690/488 never show. The port ships 85
/// (BrainScrambler) and 341 (PresentMimic); the rest ride the net id.
fn npc_hover_excluded(n: &Npc) -> bool {
    let id = n.kind.net_id();
    if (id == 85 || id == 341 || id == 629 || n.kind.ai_style() == 87) && n.ai[0] == 0.0 {
        return true;
    }
    if id == 694 && n.ai[3] == 3.0 {
        return true;
    }
    id == 690 || id == 488
}

/// The NPC hover pick (`Main.cs:42994-43113`'s walk): the first live,
/// non-shimmer-transparent NPC whose hover box contains the pointer and
/// that passes the exclusion gate. The `netOffset` shift (`:43002`)
/// rides `offset_clone` like the health-bar pass.
pub fn hovered_npc(npcs: &[Npc], mouse_world: Vec2) -> Option<usize> {
    npcs.iter().position(|n| {
        if n.hp <= 0 || n.shimmer_transparency > 0.0 {
            return false; // `:42997` — active && !CurrentlyShimmerTransparent
        }
        if npc_hover_excluded(n) {
            return false;
        }
        let owned = n.offset_clone();
        let r = npc_hover_rect(&owned);
        mouse_world.x >= r[0]
            && mouse_world.x <= r[0] + r[2]
            && mouse_world.y >= r[1]
            && mouse_world.y <= r[1] + r[3]
    })
}

/// `Main.cs:43090-43099`: the NPC hover text — `GivenOrTypeName`, then
/// `": life/lifeMax"` when the CARRIER (`realLife`, the port's
/// `carrier_index` head) has `lifeMax > 1 && ShowLife`. `ShowLife` is the
/// health-bar gate's (`NPC.cs:6832-6842`: `dontTakeDamage ?
/// dontTakeDamageStillShowLife : true`, the only true default is 412).
pub fn npc_hover_text(npcs: &[Npc], idx: usize) -> String {
    let n = &npcs[idx];
    let mut text = if n.given_name.is_empty() {
        crate::lang::npc_name(n.kind.net_id())
    } else {
        n.given_name.clone()
    };
    // `num = i; if (realLife >= 0) num = realLife` (`:43091-43095`).
    let carrier = Npc::carrier_index(npcs, idx).unwrap_or(idx);
    let c = &npcs[carrier];
    let show_life = !c.dont_take_damage || c.kind.net_id() == 412;
    if c.max_hp > 1 && show_life {
        text = format!("{text}: {}/{}", c.hp, c.max_hp);
    }
    text
}

/// `MouseTextInner`'s plain-text position (`Main.cs:20196-20243`):
/// `mouse + 14`, `+34` X while the cursor carries an item (`:20208-20211`),
/// then the `num2 = 40` on-screen clamp (`:20236-20243`, int-truncated).
/// `ThickMouse` (`:20203-20207`) is unported (no thick-cursor setting).
pub fn mouse_text_pos(mouse: Vec2, carrying: bool, text_size: [f32; 2], screen: Vec2) -> [f32; 2] {
    let mut x = mouse.x + 14.0;
    let mut y = mouse.y + 14.0;
    if carrying {
        x += 34.0;
    }
    if x + text_size[0] + 40.0 > screen.x {
        x = (screen.x - text_size[0] - 40.0) as i32 as f32;
    }
    if y + text_size[1] + 40.0 > screen.y {
        y = (screen.y - text_size[1] - 40.0) as i32 as f32;
    }
    [x, y]
}

/// The pass's draw (`Main.cs:42908`/`:43100` → `MouseTextHackZoom` →
/// `MouseTextInner` `:20319`): `ChatManager.DrawColorCodedStringWithShadow`
/// at spread 2 — the four ±2 px black passes then the colour pass, the
/// rare colour multiplied by the `mouseTextColor` pulse (`:20245-20310`).
/// NO `DrawInvBG` box: that is the `HoverItem` tooltip path (`:20213`).
pub fn draw_hover_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    carrying: bool,
    label: &str,
    color: [f32; 4],
    screen: Vec2,
    pulse: UiPulse,
) {
    let w = text::hud_text_width(r, lookups, label, 1.0);
    let h = text::line_height(lookups);
    let [x, y] = mouse_text_pos(mouse, carrying, [w, h], screen);
    let t = pulse.text;
    let tint = [color[0] * t, color[1] * t, color[2] * t, t];
    for [dx, dy] in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
        text::hud_text(r, lookups, x + dx, y + dy, 1.0, [0.0, 0.0, 0.0, t], label);
    }
    text::hud_text(r, lookups, x, y, 1.0, tint, label);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// `Main.cs:42893-42895`: the drop's hover rect is the draw frame
    /// bottom-centred on `item.Bottom` — with no atlas icon the 16x16
    /// `WorldItem` box (`WorldItem.cs:138`) stands in.
    #[test]
    fn g53_the_drop_hover_rect_is_bottom_centred_on_the_box() {
        let lookups = assets::Lookups::default();
        let drop = ItemDrop::new(5, 3, 0, 100.0, 200.0, 0.0, 0.0, 0.0);
        // Bottom = (100 + 7, 200 + 14) = (107, 214); the 16x16 rect's
        // top-left = (107 − 8, 214 − 16) = (99, 198).
        assert_eq!(drop_hover_rect(&drop, &lookups), [99.0, 198.0, 16.0, 16.0]);
        // The walk picks the first hit (`:42910`'s break).
        let items = vec![drop];
        assert!(hovered_drop(&items, &lookups, Vec2::new(107.0, 210.0)).is_some());
        assert!(hovered_drop(&items, &lookups, Vec2::new(98.0, 210.0)).is_none());
    }

    /// `Main.cs:42898-42902`: `AffixName + " (stack)"` above one, the rare
    /// colour (`:42903`).
    #[test]
    fn g53_the_drop_text_carries_the_affix_stack_and_rare_colour() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let single = ItemDrop::new(5, 1, 0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (name, color) = drop_hover_text(&single);
        assert_eq!(name, "Mushroom");
        assert_eq!(color, [1.0; 4], "rare 0 is white");
        let stack = ItemDrop::new(5, 12, 0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(drop_hover_text(&stack).0, "Mushroom (12)");
        // A prefixed drop prints the affix name (`Item.AffixName`).
        let prefixed = ItemDrop::new(4, 1, 81, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (name, _) = drop_hover_text(&prefixed);
        assert!(name.ends_with("Iron Broadsword"), "{name}");
        assert_ne!(name, "Iron Broadsword", "the affix prefixes the name");
    }

    /// `Main.cs:43090-43099`: the NPC text is `GivenOrTypeName` plus the
    /// carrier's `life/lifeMax` while `lifeMax > 1 && ShowLife`.
    #[test]
    fn g53_the_npc_text_appends_the_carrier_life() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut guide = Npc::new(NpcKind::Guide, 0.0, 0.0, 1, 0.0);
        guide.hp = guide.max_hp;
        let npcs = vec![guide];
        let text = npc_hover_text(&npcs, 0);
        assert!(text.starts_with("Guide"), "{text}");
        assert!(text.contains(&format!(": {}/{}", npcs[0].hp, npcs[0].max_hp)));
        // A named NPC answers its given name (`NPC.GivenOrTypeName`).
        let mut named = npcs.clone();
        named[0].given_name = "Andrew".into();
        assert!(npc_hover_text(&named, 0).starts_with("Andrew"));
        // lifeMax == 1: no suffix (`:43096`).
        let mut one = npcs.clone();
        one[0].max_hp = 1;
        one[0].hp = 1;
        assert_eq!(npc_hover_text(&one, 0), "Guide");
    }

    /// `Main.cs:42997` + `:43018`: dead / shimmer-transparent NPCs never
    /// hover; the buried-mimic gate hides 341 while `ai[0] == 0`.
    #[test]
    fn g53_the_npc_pick_gates_on_life_shimmer_and_the_buried_state() {
        let mut n = Npc::new(NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        n.hp = n.max_hp;
        let (w, h) = n.size();
        let over = Vec2::new(n.x + w * 0.5, n.y + h * 0.5);
        assert_eq!(hovered_npc(&[n.clone()], over), Some(0));
        let mut dead = n.clone();
        dead.hp = 0;
        assert_eq!(hovered_npc(&[dead], over), None);
        let mut shimmer = n.clone();
        shimmer.shimmer_transparency = 0.5;
        assert_eq!(hovered_npc(&[shimmer], over), None);
        // The PresentMimic (341) buried (`ai[0] == 0`): excluded; dug up
        // (`ai[0] != 0`): hoverable.
        n.kind = NpcKind::PresentMimic;
        n.ai[0] = 0.0;
        assert!(npc_hover_excluded(&n));
        n.ai[0] = 1.0;
        assert!(!npc_hover_excluded(&n));
    }

    /// `Main.cs:20196-20243`: the plain MouseText position — mouse + 14,
    /// +34 X while carrying, the 40 px clamp int-truncated.
    #[test]
    fn g53_the_mouse_text_position_clamps_at_forty() {
        let screen = Vec2::new(1280.0, 720.0);
        // Room on screen: mouse + 14.
        assert_eq!(
            mouse_text_pos(Vec2::new(100.0, 100.0), false, [50.0, 20.0], screen),
            [114.0, 114.0]
        );
        // Carrying shifts X by 34 (`:20210`).
        assert_eq!(
            mouse_text_pos(Vec2::new(100.0, 100.0), true, [50.0, 20.0], screen),
            [148.0, 114.0]
        );
        // The clamp: 1270 + 50 + 40 > 1280 → x = (1280 − 50 − 40) = 1190.
        assert_eq!(
            mouse_text_pos(Vec2::new(1270.0, 700.0), false, [50.0, 20.0], screen),
            [1190.0, 660.0]
        );
    }
}
