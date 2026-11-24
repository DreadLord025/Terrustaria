//! The chat-portrait DRAW, Detailed arm (W4-g58 retire) —
//! `Main.DrawNPCPortrait` (`Main.cs:40318-40447`) from `NPCChatPanel.Draw`
//! (`NPCChatPanel.cs:79`); the hop-state's first CONSUMER (research/240
//! §4). Scope: `Detailed` (`:40375-40377`, default preference 0) + the
//! `Disabled` gate (`:40321`); `CloseUp`/`FullBodyRetro` (`:40379-40443`)
//! deferred with the portrait-system lane (draw nothing — ANALYSIS). The
//! dead wobble/sway (`:40347-40372`) dropped. Provider:
//! [`super::chat_portrait_data`]; tests: [`super::chat_portrait_tests`].

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::chat_portrait_data::{is_shimmer_variant, portrait_sheet_name};
use super::text;
use crate::assets::Lookups;
use crate::npc::Npc;

/// The portrait anchor (`Main.cs:40336-40337` + the hop lift
/// `:40355-40359`): `npcChatTopLeft + (-62, 62)`, `y += hop_lift` — the
/// CENTRE both sprites draw around (`origin = rect.Size()/2`).
pub(crate) fn portrait_anchor(panel_top_left: Vec2, frame_counter: i32) -> Vec2 {
    let v = panel_top_left + Vec2::new(-62.0, 62.0);
    Vec2::new(v.x, v.y + super::npc_chat::portrait_hop_lift(frame_counter))
}

/// The Detailed arm (`:40318-40377` + the label `:40444-40446`).
/// `preference`: 0 Detailed / 1 CloseUp / 2 FullBodyRetro / 3 Disabled —
/// 3 returns (`:40321`), 1/2 draw nothing (deferred; the hop counter
/// ticks at the caller regardless — the `:40335` increment precedes the
/// mode branch). Returns whether a portrait drew.
pub(crate) fn draw_portrait(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    npc: &Npc,
    panel_top_left: Vec2,
    frame_counter: i32,
    preference: u8,
    lycanthrope: bool,
) -> bool {
    if preference == 3 {
        return false; // `DialoguePortraitDrawOption.Disabled` (`:40321`)
    }
    let Some(name) = portrait_sheet_name(npc.kind, is_shimmer_variant(npc), lycanthrope) else {
        return false; // no provider row (`:40326-40329`)
    };
    let Some(&sheet) = lookups.chat_portraits.get(&name) else {
        return false; // texture null-gate (`:40331-40334`)
    };
    if preference != 0 {
        return false; // CloseUp/FullBodyRetro deferred (module header)
    }
    let anchor = portrait_anchor(panel_top_left, frame_counter);
    // `:40373` — the Portrait_Window backdrop at the chatBack tint
    // (`Color(200,200,200,200)`, `NPCChatPanel.cs:62`), centre origin.
    if let Some(bg) = lookups.npc_portrait_background {
        let (bw, bh) = (bg.width as f32, bg.height as f32);
        let uv = sheet_uv(lookups, bg.x as f32, bg.y as f32, bw, bh);
        let pos = [anchor.x - bw * 0.5, anchor.y - bh * 0.5];
        r.push_ui_sprite(pos, [bw, bh], uv, [200.0 / 255.0; 4]);
    }
    // `:40377` — the portrait texture, Color.White, centre origin.
    let (pw, ph) = (sheet.width as f32, sheet.height as f32);
    let uv = sheet_uv(lookups, sheet.x as f32, sheet.y as f32, pw, ph);
    r.push_ui_sprite([anchor.x - pw * 0.5, anchor.y - ph * 0.5], [pw, ph], uv, [1.0; 4]);
    draw_name_label(r, lookups, npc, anchor);
    true
}

/// The whole-sheet UV rect in atlas space (the `push_crop` precedent's
/// full-frame case — the portrait sheets are single-frame wholes).
fn sheet_uv(lookups: &Lookups, x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    [x / aw, y / ah, (x + w) / aw, (y + h) / ah]
}

/// The name label (`Main.cs:40444-40446`): `GivenOrTypeName`
/// (`NPC.cs:6743-6753` — given name if any, else type name) in
/// `FontAssets.ItemStack`, White, origin = MeasureString/2 at
/// `anchor + (0, 57)`. Bitmap fallback: 8 px/char, 16 px line height.
fn draw_name_label(r: &mut SpriteRenderer, lookups: &Lookups, npc: &Npc, anchor: Vec2) {
    let label = if npc.given_name.is_empty() {
        crate::lang::npc_name(npc.kind.cs_id() as i32)
    } else {
        npc.given_name.clone()
    };
    let lw = lookups
        .item_font
        .as_ref()
        .map(|f| text::font_width(f, &label, 1.0))
        .unwrap_or(label.chars().count() as f32 * 8.0);
    let lh = lookups.item_font.as_ref().map(|f| f.line_spacing).unwrap_or(16.0);
    const SHADOW_RING: [[f32; 2]; 4] = [[2.0, 0.0], [-2.0, 0.0], [0.0, 2.0], [0.0, -2.0]];
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Item,
        anchor.x - lw * 0.5,
        anchor.y + 57.0 - lh * 0.5,
        1.0,
        &SHADOW_RING,
        [1.0; 4],
        &label,
    );
}
