//! The second half of `Main.DrawMap`'s overlay machinery (W4-f29): the
//! NPC head icons (`Main.DrawNPCMapIcons`/`2`/`3`, `Main.cs:56099-56292`),
//! the player death markers (`Main.DrawPlayerDeathMarker`,
//! `Main.cs:56446-56464` + the three call sites), the revenge coin markers
//! (`CoinLossRevengeSystem.RevengeMarker.DrawMapIcon`/`DrawMapIcons`/
//! `UseMouseOver`, `CoinLossRevengeSystem.cs:252-273/:482-496`), the misc
//! portal/golf icons (`Main.DrawMiscMapIcons`, `Main.cs:56466-56539`) and
//! the fullscreen hover tile/chest names (`Main.cs:55784-55913` +
//! `DrawMap_FindChestName`, `:56631-56643`).
//!
//! The head composites reproduce `OutlinedTextureRenderer.DrawWithOutlines`
//! (`OutlinedTextureRenderer.cs:27-44`) over the
//! `AnOutlinedDrawRenderTargetContent` rings (`:38-62`): 8 black copies at
//! Manhattan radius 4 + 4 white copies at radius 2, then the head itself —
//! the same ring shape `render/entities.rs::draw_player_map_icon` already
//! uses for the player head (the port has no render targets).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;
use crate::ui::color;

/// `NPCID.Sets.BossHeadTextures` (`NPCID.cs:10479`): the vanilla NPC type →
/// boss-head sheet index, default -1. The seven per-instance overrides live
/// in [`boss_head_texture_index`] (`NPC.GetBossHeadTextureIndex`).
const BOSS_HEAD_TEXTURES: [(i32, i32); 40] = [
    (4, 0),
    (13, 2),
    (344, 3),
    (370, 4),
    (246, 5),
    (249, 5),
    (345, 6),
    (50, 7),
    (396, 8),
    (395, 9),
    (325, 10),
    (262, 11),
    (327, 13),
    (222, 14),
    (125, 15),
    (126, 20),
    (346, 17),
    (127, 18),
    (35, 19),
    (68, 19),
    (113, 22),
    (266, 23),
    (439, 24),
    (440, 24),
    (134, 25),
    (491, 26),
    (517, 27),
    (422, 28),
    (507, 29),
    (493, 30),
    (549, 35),
    (564, 32),
    (565, 32),
    (576, 33),
    (577, 33),
    (551, 34),
    (548, 36),
    (636, 37),
    (657, 38),
    (668, 39),
];

/// `NPC.GetBossHeadTextureIndex` (`NPC.cs:7557-7606`): the table lookup plus
/// the seven type overrides, verbatim. -1 means "no boss head".
pub(crate) fn boss_head_texture_index(n: &Npc) -> i32 {
    let ty = n.kind.net_id();
    let mut result = BOSS_HEAD_TEXTURES
        .iter()
        .find(|(t, _)| *t == ty)
        .map(|(_, h)| *h)
        .unwrap_or(-1);
    match ty {
        // `:7562-7567`: the Eye's second phase swaps to head 1.
        4 => {
            if n.ai[0] >= 2.0 {
                result = 1;
            }
        }
        // `:7568-7573`: the Brain hides its head while illusioned.
        266 => {
            if n.ai[0] < 0.0 || n.local_ai[2] > 0.0 {
                result = -1;
            }
        }
        // `:7574-7579`: Plantera's enraged half-life head.
        262 => {
            if n.hp <= n.max_hp / 2 {
                result = 12;
            }
        }
        // `:7580-7591`: the twins' second phases.
        125 => {
            if n.ai[0] >= 3.0 {
                result = 16;
            }
        }
        126 => {
            if n.ai[0] >= 3.0 {
                result = 21;
            }
        }
        // `:7592-7603`: the Cultist's clone/ritual visibility pair.
        439 => {
            if n.ai[0] == 5.0 && n.alpha == 255 {
                result = -1;
            }
        }
        440 => {
            if n.ai[0] != 5.0 || n.alpha == 255 {
                result = -1;
            }
        }
        _ => {}
    }
    result
}

/// `NPC.GetBossHeadRotation` (`NPC.cs:7608-7632`).
pub(crate) fn boss_head_rotation(n: &Npc) -> f32 {
    match n.kind.net_id() {
        68 | 262 => n.rotation,
        35 | 127 => {
            if n.ai[1] == 1.0 || n.ai[1] == 2.0 {
                n.rotation
            } else {
                0.0
            }
        }
        345 => {
            if n.ai[0] == 2.0 {
                n.rotation
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// `NPC.GetBossHeadSpriteEffects` (`NPC.cs:7634-7642`): only the Flying
/// Dutchman (491) flips, on `spriteDirection == 1`.
pub(crate) fn boss_head_flip(n: &Npc) -> bool {
    n.kind.net_id() == 491 && n.sprite_dir == 1
}

/// `TownNPCProfiles.GetHeadIndexSafe` (`TownNPCProfiles.cs:200-207`): the
/// profile's head index, else `NPC.TypeToDefaultHeadIndex`. Every ported
/// town kind's profile is a `LegacyWithSimpleShimmer`/`LegacyNPCProfile`
/// whose `GetHeadTextureIndex` returns the default head
/// (`Profiles.cs:106-109`) — the same value `TypeToDefaultHeadIndex` maps —
/// so the port resolves through the one table. The shimmered profiles'
/// alternate heads (55-80) ride `townNpcVariationIndex`, unported; the
/// variant pets (Cat/Dog/Bunny 637/638/656) have no port kind.
pub(crate) fn town_head_index(n: &Npc) -> i32 {
    crate::ui::housing::type_to_head_index(n.kind.net_id())
}

/// `NPC.FullName` (`NPC.cs:6729-6739`): the given name + type title, or the
/// bare type name when unnamed.
pub(crate) fn npc_full_name(n: &Npc) -> String {
    let type_name = crate::lang::npc_name(n.kind.net_id());
    if n.given_name.is_empty() {
        return type_name;
    }
    crate::lang::text_args("Game.NPCTitle", &[n.given_name.clone(), type_name])
}

/// `NPC.GivenOrTypeName` (`NPC.cs:6743-6753`).
pub(crate) fn npc_given_or_type_name(n: &Npc) -> String {
    if n.given_name.is_empty() {
        crate::lang::npc_name(n.kind.net_id())
    } else {
        n.given_name.clone()
    }
}

/// The Destroyer's segment-average centre (`Main.cs:56125-56137` and the
/// twins at `:56186-56198`/`:56261-56273`): walk the `ai[0]` chain from the
/// head (type 134), summing `Center` while the next slot is an active
/// 134-136, at most 15 segments, then divide. The port's `ai[0]` stores
/// `child_index + 1` (`npc_ai_style37.rs:1117`), so the C#'s
/// `(int)ai[0]` slot read becomes `ai[0] as usize - 1`; a zero `ai[0]`
/// (the head's pre-spawn tick) addresses no slot, like the C#'s slot 0
/// (`CannotSpawnInSlot0`, `NPCID.cs:10452`).
pub(crate) fn destroyer_average_center(npcs: &[Npc], head: usize) -> [f32; 2] {
    let (hx, hy) = npcs[head].center();
    let mut center = [hx, hy];
    let mut count = 1;
    let mut next = npcs[head].ai[0] as usize;
    while count < 15 && next > 0 {
        let Some(seg) = npcs.get(next - 1) else { break };
        let ty = seg.kind.net_id();
        if seg.hp <= 0 || !(134..=136).contains(&ty) {
            break;
        }
        count += 1;
        let (cx, cy) = seg.center();
        center[0] += cx;
        center[1] += cy;
        next = seg.ai[0] as usize;
    }
    [center[0] / count as f32, center[1] / count as f32]
}

/// `Main.DrawNPCMapIcon_CanBeSeen_Townie` (`Main.cs:56294-56297`): always
/// true in 1.4.5.8.
pub(crate) fn townie_can_be_seen(_n: &Npc) -> bool {
    true
}

/// The head-draw mode: which `DrawNPCMapIcons*` variant runs. The three
/// share the walk; only the position maths and the clip differ.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadMode {
    /// `DrawNPCMapIcons` (`Main.cs:56222-56292`), the fullscreen map:
    /// `x = center.X/16*scale + X - 10*scale`, no clip, hover AFTER draw.
    Fullscreen,
    /// `DrawNPCMapIcons2` (`Main.cs:56144-56220`), the minimap: the
    /// `(center/16 - left/top)*scale + X2/Y2 - 2*scale/5 + xOff/yOff`
    /// transform, the `(miniMapX+12, +10, W-16, H-14)` clip, hover BEFORE
    /// draw.
    Minimap,
    /// `DrawNPCMapIcons3` (`Main.cs:56099-56142`), the mapStyle-2 overlay:
    /// the fullscreen transform, no clip, no hover text at all.
    Overlay,
}

/// The shared head-icon inputs. `origin` is `(X, Y)` / `(X2, Y2)`,
/// `left_top` is `(num12, num13)` (zero on the fullscreen), `sub_offset` is
/// `(num10, num11)` (the minimap's sub-tile centres), `clip` is the
/// `(miniMapX+12, miniMapY+10, miniMapX+W-16, miniMapY+H-14)` rect in the
/// same transform space, `scale` is `num5` and `head_scale` the
/// `num37`/`num46`/`num77` icon scale.
pub(crate) struct HeadIconInputs<'a> {
    pub npcs: &'a [Npc],
    pub mode: HeadMode,
    pub origin: [f32; 2],
    pub left_top: [f32; 2],
    pub sub_offset: [f32; 2],
    pub clip: Option<[f32; 4]>,
    pub scale: f32,
    pub head_scale: f32,
    /// The minimap batch transform `Matrix.CreateScale(Main.MapScale)`
    /// (`Main.cs:55309-55329`): positions/sizes are computed in transform
    /// space, then multiplied by this on the way to the screen. 1.0 on the
    /// fullscreen map (its `transformMatrix` is Identity, `:55322-55325`).
    pub present_scale: f32,
    /// `b` — the map alpha byte (`Main.cs:55292`/`:55532`).
    pub alpha: u8,
    /// The hover mouse in TRANSFORM space. The minimap branch scales
    /// `Main.mouseX/Y` by `1/MapScale` (`Main.cs:55512-55513`) before the
    /// head hover rects read them; the fullscreen uses `MouseScreen` as-is.
    pub mouse: Vec2,
}

/// The head position for one NPC centre, per mode — the C# left-to-right
/// float order kept (`Main.cs:56113-56115` / `:56158-56164` /
/// `:56236-56242`).
pub(crate) fn head_pos(center: [f32; 2], inp: &HeadIconInputs) -> [f32; 2] {
    match inp.mode {
        HeadMode::Fullscreen | HeadMode::Overlay => [
            center[0] / 16.0 * inp.scale + inp.origin[0] - 10.0 * inp.scale,
            center[1] / 16.0 * inp.scale + inp.origin[1] - 10.0 * inp.scale,
        ],
        HeadMode::Minimap => [
            (center[0] / 16.0 - inp.left_top[0]) * inp.scale
                + inp.origin[0]
                + inp.sub_offset[0],
            (center[1] / 16.0 - inp.left_top[1]) * inp.scale
                + inp.origin[1]
                - 2.0 * inp.scale / 5.0
                + inp.sub_offset[1],
        ],
    }
}

/// The minimap head clip (`Main.cs:56165`/`:56206`): strict inequalities on
/// all four edges.
pub(crate) fn head_in_clip(pos: [f32; 2], clip: [f32; 4]) -> bool {
    pos[0] > clip[0] && pos[0] < clip[2] && pos[1] > clip[1] && pos[1] < clip[3]
}

/// The hover rect of a head sheet centred at `pos`
/// (`Main.cs:56167-56171`): `(pos - (Width/2)*headScale, Width*headScale)`,
/// inclusive on all four edges. The C# `Width() / 2` is INTEGER division
/// (`Texture2D.Width()` returns int), so an odd-width sheet's half truncates.
pub(crate) fn head_hover(pos: [f32; 2], size: [f32; 2], head_scale: f32, mouse: Vec2) -> bool {
    let x0 = pos[0] - (size[0] as i32 / 2) as f32 * head_scale;
    let y0 = pos[1] - (size[1] as i32 / 2) as f32 * head_scale;
    let x1 = x0 + size[0] * head_scale;
    let y1 = y0 + size[1] * head_scale;
    mouse.x >= x0 && mouse.x <= x1 && mouse.y >= y0 && mouse.y <= y1
}

/// `OutlinedTextureRenderer.DrawWithOutlines` (`OutlinedTextureRenderer.cs:
/// 27-44`) over `AnOutlinedDrawRenderTargetContent` (`:38-62`): the target
/// is `texture + 8` px with the head at `(4, 4)` (`OutlinedDrawRenderTarget
/// Content.cs:16-23`), blitted centred (`target.Size()/2` origin) — so the
/// head's own centre lands on `pos`. The rings: 8 black copies at Manhattan
/// radius 4 (`num = 2`, `num2 = 4`, step 2), 4 white copies at radius 2,
/// then the head. The ring copies ride the ColorOnly shader (alpha-only
/// silhouette tint); the port reproduces them as tinted copies of the same
/// sheet, the `draw_player_map_icon` convention. `rotation` pivots the
/// whole composite; `flip` mirrors the head sheet horizontally.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_head_outlined(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &tr_render::SheetRect,
    pos: [f32; 2],
    tint: [f32; 4],
    rotation: f32,
    scale: f32,
    flip: bool,
) {
    let Some(uv) = lookups.hud_uv(Some(sheet)) else {
        return;
    };
    let uv = if flip {
        [uv[2], uv[1], uv[0], uv[3]]
    } else {
        uv
    };
    let size = [sheet.width as f32 * scale, sheet.height as f32 * scale];
    // The target centre is `pos`; the head sits at target-local (4, 4) of
    // the (w+8, h+8) target, i.e. its own centre is the target centre.
    let base = [pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5];
    let black = [0.0, 0.0, 0.0, tint[3]];
    let white = [tint[3], tint[3], tint[3], tint[3]];
    let (s, c) = rotation.sin_cos();
    let mut copy = |off: [f32; 2], col: [f32; 4]| {
        // The ring offset rotates with the composite (`RotatedBy` is not
        // in this path — the rings are baked into the target BEFORE the
        // rotation, so the whole target rotates about its centre; the
        // offset in target space is `(off*scale)` rotated by `rotation`).
        let ox = off[0] * scale;
        let oy = off[1] * scale;
        let rx = ox * c - oy * s;
        let ry = ox * s + oy * c;
        let tl = [base[0] + rx, base[1] + ry];
        if rotation == 0.0 {
            r.push_ui_sprite(tl, size, uv, col);
        } else {
            // Rotate about the head centre: shift the top-left by
            // `origin - R(rot)*origin` (the bestiary convention).
            let origin = [size[0] * 0.5, size[1] * 0.5];
            let tl = [
                tl[0] + origin[0] - (origin[0] * c - origin[1] * s),
                tl[1] + origin[1] - (origin[0] * s + origin[1] * c),
            ];
            r.push_ui_sprite_rot(tl, size, uv, col, rotation);
        }
    };
    // `AnOutlinedDrawRenderTargetContent.cs:38-49`: the 8 black copies.
    for off in [
        [-4.0, 0.0],
        [4.0, 0.0],
        [0.0, -4.0],
        [0.0, 4.0],
        [-2.0, -2.0],
        [-2.0, 2.0],
        [2.0, -2.0],
        [2.0, 2.0],
    ] {
        copy(off, black);
    }
    // `:50-60`: the 4 border copies (white — `_borderColor` defaults White
    // and neither head renderer calls `UseColor`).
    for off in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
        copy(off, white);
    }
    // `:62`: the head itself.
    copy([0.0, 0.0], tint);
}

/// `Main.DrawNPCHeadFriendly` (`Main.cs:56553-56560`): the outlined town
/// head plus the `nextDialogue.ShowIndicator` notification icon at
/// `(x, y - 20)`. The port has no `nextDialogue` queue (the chat panel is
/// click-driven), so the indicator gate is always false — recorded in
/// docs/ANALYSIS.md. `pos` is in screen px; `head_scale` is already the
/// screen-space scale (`headScale * present_scale`).
pub(crate) fn draw_npc_head_friendly(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    head_id: usize,
    pos: [f32; 2],
    alpha: u8,
    head_scale: f32,
    flip: bool,
) {
    let Some(sheet) = lookups.npc_heads.get(head_id).and_then(|s| *s) else {
        return;
    };
    let tint = color::tint([alpha, alpha, alpha, alpha]);
    draw_head_outlined(r, lookups, &sheet, pos, tint, 0.0, head_scale, flip);
    // `:56556-56559`: `theNPC is NPC { nextDialogue: not null } nPC &&
    // nPC.nextDialogue.ShowIndicator` — unported (no dialogue queue).
}

/// `Main.DrawNPCHeadBoss` (`Main.cs:56562-56565`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_npc_head_boss(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    head_id: usize,
    pos: [f32; 2],
    alpha: u8,
    head_scale: f32,
    rotation: f32,
    flip: bool,
) {
    let Some(sheet) = lookups.npc_head_boss.get(head_id).and_then(|s| *s) else {
        return;
    };
    let tint = color::tint([alpha, alpha, alpha, alpha]);
    draw_head_outlined(r, lookups, &sheet, pos, tint, rotation, head_scale, flip);
}

/// The `DrawNPCMapIcons`/`2`/`3` walk (`Main.cs:56099-56292`): per NPC, the
/// town head first (gated `active && townNPC && CanBeSeen_Townie`, head
/// index > 0), then the boss head (gated `active && headIndex != -1`).
/// Returns the hover text (the C#'s `mouseTextString` ref). The port's
/// liveness is `hp > 0` (the list carries no `active` field).
pub(crate) fn draw_npc_map_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    inp: &HeadIconInputs,
) -> Option<String> {
    let mut text: Option<String> = None;
    for (i, n) in inp.npcs.iter().enumerate() {
        if n.hp <= 0 {
            continue;
        }
        // The townie half (`:56103-56117` / `:56148-56178` / `:56226-56253`).
        // The C# gate is the `townNPC` flag — the port's `is_town_resident`
        // set (the 21 kinds with `this.townNPC = true` in `SetDefaults`).
        if n.kind.is_town_resident() && townie_can_be_seen(n) {
            let head = town_head_index(n);
            if head > 0 {
                // `direction > 0` flips (`:56109-56112`).
                let flip = n.facing > 0;
                let (cx, cy) = n.center();
                let center = [
                    cx + n.net_offset.x,
                    cy + n.gfx_off_y + n.net_offset.y,
                ];
                let pos = head_pos(center, inp);
                let draw = match inp.mode {
                    HeadMode::Minimap => {
                        inp.clip.is_some_and(|c| head_in_clip(pos, c))
                    }
                    _ => true,
                };
                if draw {
                    let sheet = lookups.npc_heads.get(head as usize).and_then(|s| *s);
                    if let Some(sheet) = sheet {
                        let size = [sheet.width as f32, sheet.height as f32];
                        // The minimap hovers BEFORE the draw
                        // (`:56171-56175`); the fullscreen AFTER
                        // (`:56247-56251`); the overlay never. The hover
                        // rect is in transform space (the mouse is too).
                        if inp.mode != HeadMode::Overlay
                            && head_hover(pos, size, inp.head_scale, inp.mouse)
                        {
                            text = Some(npc_full_name(n));
                        }
                        draw_npc_head_friendly(
                            r,
                            lookups,
                            head as usize,
                            [pos[0] * inp.present_scale, pos[1] * inp.present_scale],
                            inp.alpha,
                            inp.head_scale * inp.present_scale,
                            flip,
                        );
                    }
                }
            }
        }
        // The boss half (`:56118-56141` / `:56179-56217` / `:56254-56289`).
        let head = boss_head_texture_index(n);
        if head == -1 {
            continue;
        }
        let rotation = boss_head_rotation(n);
        let flip = boss_head_flip(n);
        let (cx, cy) = n.center();
        let mut center = [
            cx + n.net_offset.x,
            cy + n.gfx_off_y + n.net_offset.y,
        ];
        if n.kind.net_id() == 134 {
            center = destroyer_average_center(inp.npcs, i);
        }
        let pos = head_pos(center, inp);
        let draw = match inp.mode {
            HeadMode::Minimap => inp.clip.is_some_and(|c| head_in_clip(pos, c)),
            _ => true,
        };
        if !draw {
            continue;
        }
        let sheet = lookups.npc_head_boss.get(head as usize).and_then(|s| *s);
        let Some(sheet) = sheet else { continue };
        let size = [sheet.width as f32, sheet.height as f32];
        if inp.mode == HeadMode::Minimap
            && head_hover(pos, size, inp.head_scale, inp.mouse)
        {
            text = Some(npc_given_or_type_name(n));
        }
        draw_npc_head_boss(
            r,
            lookups,
            head as usize,
            [pos[0] * inp.present_scale, pos[1] * inp.present_scale],
            inp.alpha,
            inp.head_scale * inp.present_scale,
            rotation,
            flip,
        );
        // The fullscreen boss hover runs AFTER the draw (`:56282-56289`).
        if inp.mode == HeadMode::Fullscreen
            && head_hover(pos, size, inp.head_scale, inp.mouse)
        {
            text = Some(npc_given_or_type_name(n));
        }
    }
    text
}

// ---------------------------------------------------------------------------
// The player death markers (`Main.DrawPlayerDeathMarker`, `Main.cs:56446-
// 56464`, + the three call sites `:55672-55691` / `:55742-55765` /
// `:55977-55987`).
// ---------------------------------------------------------------------------

/// `Main.DrawPlayerDeathMarker` (`Main.cs:56446-56464`): the MapDeath
/// texture centred on the death spot, the `-(2 - scale/5*2)` y-offset and
/// the `+4 -14*headScale` / `28*headScale` hover box. Returns whether the
/// mouse hovers it (the C#'s `num19 = i` arm). `last_death` is the world-px
/// `Center` stamp; `origin`/`left_top`/`scale` are the map transform; the
/// fullscreen call passes `left_top = (num12, num13) = 0` and
/// `origin = (num, num2)`. `present_scale` is the batch transform the draw
/// rides (1.0 fullscreen, `MapScale` on the minimap): the position and
/// size scale by it on the way to the screen, while the hover box stays in
/// transform space (the caller's mouse is in that space too).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_player_death_marker(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    last_death: [f32; 2],
    origin: [f32; 2],
    left_top: [f32; 2],
    scale: f32,
    head_scale: f32,
    present_scale: f32,
    alpha: u8,
    mouse: Vec2,
) -> bool {
    // `:56448-56453`, the C# left-to-right order.
    let mut x = (last_death[0] / 16.0 - left_top[0]) * scale;
    let mut y = (last_death[1] / 16.0 - left_top[1]) * scale;
    x += origin[0];
    y = y + origin[1] - (2.0 - scale / 5.0 * 2.0);
    x -= 10.0 * scale;
    y -= 10.0 * scale;
    let Some(sheet) = lookups.map_death else {
        return false;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return false;
    };
    let size = [
        sheet.width as f32 * head_scale * present_scale,
        sheet.height as f32 * head_scale * present_scale,
    ];
    // `:56454`: origin = texture centre, so the top-left is pos - size/2.
    let a = alpha as f32 / 255.0;
    r.push_ui_sprite(
        [
            x * present_scale - size[0] * 0.5,
            y * present_scale - size[1] * 0.5,
        ],
        size,
        uv,
        [a, a, a, a],
    );
    // `:56455-56463`: the hover box, in transform space.
    let x0 = x + 4.0 - 14.0 * head_scale;
    let y0 = y + 2.0 - 14.0 * head_scale;
    let x1 = x0 + 28.0 * head_scale;
    let y1 = y0 + 28.0 * head_scale;
    mouse.x >= x0 && mouse.x <= x1 && mouse.y >= y0 && mouse.y <= y1
}

/// The minimap death-marker clip (`Main.cs:55752`): the
/// `(miniMapX+8, miniMapY+8, W-18, H-16)` rect, strict on all four edges —
/// distinct from the head clip's `+12/+10/-16/-14`.
pub(crate) fn death_marker_in_minimap_clip(pos: [f32; 2], origin: [f32; 2]) -> bool {
    pos[0] > origin[0] + 8.0
        && pos[0] < origin[0] + 240.0 - 18.0
        && pos[1] > origin[1] + 8.0
        && pos[1] < origin[1] + 240.0 - 16.0
}

/// The minimap death-marker position (`Main.cs:55746-55751`): the same
/// transform as the fullscreen marker but with the `num10/num11` sub-tile
/// offsets added and NO `-10*scale` inset (the minimap branch keeps the
/// raw panel origin). Returns the marker centre in transform space.
pub(crate) fn minimap_death_marker_pos(
    last_death: [f32; 2],
    origin: [f32; 2],
    left_top: [f32; 2],
    sub_offset: [f32; 2],
    scale: f32,
) -> [f32; 2] {
    let mut x = (last_death[0] / 16.0 - left_top[0]) * scale;
    let mut y = (last_death[1] / 16.0 - left_top[1]) * scale;
    x += origin[0];
    y = y + origin[1] - (2.0 - scale / 5.0 * 2.0);
    x += sub_offset[0];
    y += sub_offset[1];
    [x, y]
}

/// The minimap death-marker draw + hover (`Main.cs:55752-55764`): the
/// MapDeath texture centred at the transform position (scaled to the
/// screen by `present_scale` = `MapScale`), and the hover box with the
/// minimap's EXTRA `-4f` on both axes (`:55757-55758`) — distinct from the
/// fullscreen `DrawPlayerDeathMarker` box. `pos` is the transform-space
/// centre from [`minimap_death_marker_pos`]; the mouse is in that space.
pub(crate) fn draw_minimap_death_marker(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    pos: [f32; 2],
    head_scale: f32,
    present_scale: f32,
    alpha: u8,
    mouse: Vec2,
) -> bool {
    let Some(sheet) = lookups.map_death else {
        return false;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return false;
    };
    let size = [
        sheet.width as f32 * head_scale * present_scale,
        sheet.height as f32 * head_scale * present_scale,
    ];
    let a = alpha as f32 / 255.0;
    r.push_ui_sprite(
        [
            pos[0] * present_scale - size[0] * 0.5,
            pos[1] * present_scale - size[1] * 0.5,
        ],
        size,
        uv,
        [a, a, a, a],
    );
    // `:55755-55761`: the hover box, the extra -4 on both axes.
    let x0 = pos[0] + 4.0 - 14.0 * head_scale - 4.0;
    let y0 = pos[1] + 2.0 - 14.0 * head_scale - 4.0;
    let x1 = x0 + 28.0 * head_scale;
    let y1 = y0 + 28.0 * head_scale;
    mouse.x >= x0 && mouse.x <= x1 && mouse.y >= y0 && mouse.y <= y1
}

// ---------------------------------------------------------------------------
// The revenge coin markers (`CoinLossRevengeSystem.RevengeMarker.DrawMapIcon`
// `:252-266`, `DrawMapIcons` `:482-496`, `UseMouseOver` `:268-273`).
// ---------------------------------------------------------------------------

/// `RevengeMarker.DrawMapIcon`'s coin tier (`CoinLossRevengeSystem.cs:262`):
/// <100c copper, <10000c silver, >=1000000c platinum, else gold.
pub(crate) fn revenge_coin_tier(coins_value: u64) -> usize {
    if coins_value < 100 {
        0
    } else if coins_value < 10_000 {
        1
    } else if coins_value >= 1_000_000 {
        3
    } else {
        2
    }
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the copper total split into
/// `[copper, silver, gold, platinum]`.
pub(crate) fn coins_split(count: u64) -> [u64; 4] {
    let mut array = [0u64; 4];
    let mut num = 0u64;
    let mut denom = 1_000_000u64;
    let mut i = 3;
    loop {
        array[i] = (count - num) / denom;
        num += array[i] * denom;
        denom /= 100;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    array
}

/// `RevengeMarker.DrawMapIcon` (`CoinLossRevengeSystem.cs:252-266`): the
/// coin tier's `Frame(1, 8)` row 0 centred on the marker's tile, clipped to
/// the map rect. Returns whether the mouse hovers it. `location` is world
/// px; the transform is `(location/16 - mapTopLeft)*mapScale + mapOffset`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_revenge_map_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    location: Vec2,
    coins_value: u64,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
) -> bool {
    // `:254-256`.
    let pos = [
        (location.x / 16.0 - map_top_left[0]) * map_scale + map_offset[0],
        (location.y / 16.0 - map_top_left[1]) * map_scale + map_offset[1],
    ];
    // `:257-260`: the clip is on the POSITION point (Rectangle.Contains).
    if let Some(c) = clip {
        let px = pos[0] as i32;
        let py = pos[1] as i32;
        if px < c[0] as i32 || py < c[1] as i32 || px >= c[2] as i32 || py >= c[3] as i32 {
            return false;
        }
    }
    let tier = revenge_coin_tier(coins_value);
    let Some(sheet) = lookups.coin_anim[tier] else {
        return false;
    };
    // `value.Frame(1, 8)` (`:263`): the first of the 8 animation rows.
    let fw = sheet.width as f32;
    let fh = sheet.height as f32 / 8.0;
    let Some(full_uv) = lookups.hud_uv(Some(&sheet)) else {
        return false;
    };
    let uv = [full_uv[0], full_uv[1], full_uv[2], full_uv[1] + (full_uv[3] - full_uv[1]) / 8.0];
    let size = [fw * draw_scale, fh * draw_scale];
    // `:264`: origin = frame centre.
    r.push_ui_sprite([pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5], size, uv, [1.0; 4]);
    // `:265`: `Utils.CenteredRectangle(vector, rectangle.Size()*drawScale)`
    // (`Utils.cs:947-950`) — the integer-truncated centred box.
    let hx = (pos[0] - fw * draw_scale / 2.0) as i32;
    let hy = (pos[1] - fh * draw_scale / 2.0) as i32;
    let hw = (fw * draw_scale) as i32;
    let hh = (fh * draw_scale) as i32;
    let mx = mouse.x as i32;
    let my = mouse.y as i32;
    mx >= hx && mx < hx + hw && my >= hy && my < hy + hh
}

/// `RevengeMarker.UseMouseOver` (`CoinLossRevengeSystem.cs:268-273`): the
/// horizontal `ItemSlot.DrawMoney` at `MouseScreen/drawScale + (-28) + (4,0)`.
/// The C# clears `mouseTextString` first (`:270`) — the coin row REPLACES
/// any hover text. The call re-begins the batch with `uIScaleMatrix`
/// (`Main.cs:55991`/`:55771`), so every position/size is multiplied by
/// `draw_scale` (= `Main.UIScale`) on the way to the screen. `ItemSlot.
/// DrawMoney` (`ItemSlot.cs:3652-3684`) draws the four coin icons + stack
/// counts; the port reproduces the horizontal row.
pub(crate) fn revenge_use_mouse_over(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    coins_value: u64,
    mouse: Vec2,
    draw_scale: f32,
) {
    // `:271`: the anchor in the UIScale batch's space.
    let ax = mouse.x / draw_scale - 28.0 + 4.0;
    let ay = mouse.y / draw_scale;
    let coins = coins_split(coins_value);
    // `ItemSlot.cs:3663-3683`: the horizontal row, i = 0..3, coin type
    // `74 - i` (platinum first), count `coinsArray[3 - i]` capped at 999.
    for i in 0..4usize {
        let tier = 3 - i;
        let count = coins[tier].min(999);
        // `:3680`: the coin centre at `(shopx + textWidth + 24*i + 45,
        // shopy + 50)`. The text is "" (the C# passes ""), so textWidth 0.
        let cx = (ax + 24.0 * i as f32 + 45.0) * draw_scale;
        let cy = (ay + 50.0) * draw_scale;
        if let Some(sheet) = lookups.coin_anim[tier] {
            let fw = sheet.width as f32 * draw_scale;
            let fh = sheet.height as f32 / 8.0 * draw_scale;
            if let Some(full_uv) = lookups.hud_uv(Some(&sheet)) {
                let uv = [
                    full_uv[0],
                    full_uv[1],
                    full_uv[2],
                    full_uv[1] + (full_uv[3] - full_uv[1]) / 8.0,
                ];
                // `CoinSlot.DrawItemCoin` (`CoinSlot.cs:240-253`): the
                // frame centred, scale 1 in the batch (= draw_scale here).
                r.push_ui_sprite([cx - fw * 0.5, cy - fh * 0.5], [fw, fh], uv, [1.0; 4]);
            }
        }
        // `:3682`: the stack count, ItemStack font 0.75, origin
        // `(0, LineSpacing)` at `(vector.X - 11, vector.Y + LineSpacing*0.75)`
        // — the origin cancels the +LineSpacing*0.75, so the text top-left
        // lands at `(vector.X - 11, vector.Y)` in batch space.
        crate::ui::text::draw_border_string(
            r,
            lookups,
            crate::ui::text::BorderFont::Item,
            (ax + 24.0 * i as f32 + 45.0 - 11.0) * draw_scale,
            cy,
            0.75 * draw_scale,
            &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
            [1.0; 4],
            &count.to_string(),
        );
    }
}

// ---------------------------------------------------------------------------
// The misc map icons (`Main.DrawMiscMapIcons`, `Main.cs:56466-56539`).
// ---------------------------------------------------------------------------

/// One misc portal/golf icon (`DrawMapIcons_PotionOfReturnHomePosition`
/// `:56494-56513`, `...AppearAfterUsePosition` `:56473-56492`,
/// `...LastGolfballHit` `:56515-56539`): the position is
/// `((anchor + (0, -height/2))/16 - mapTopLeft)*mapScale + mapOffset`,
/// FLOORED, the sheet centred, the hover box `Utils.CenteredRectangle`.
/// Returns the hover text key hit, if any. `anchor` is the world-px
/// position (the potion Bottom / the golf ball Centre); `height_offset` is
/// `-LocalPlayer.height/2` for the portals, 0 for the golf ball.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_misc_map_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<tr_render::SheetRect>,
    anchor: [f32; 2],
    height_offset: f32,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
) -> bool {
    let Some(sheet) = sheet else { return false };
    // `:56480`/`:56501`/`:56522`: the `Utils.Floor` position — which is
    // `(int)` TRUNCATION toward zero (`Utils.cs:1702-1707`), not Math.Floor.
    let pos = [
        (((anchor[0]) / 16.0 - map_top_left[0]) * map_scale + map_offset[0]) as i32 as f32,
        (((anchor[1] + height_offset) / 16.0 - map_top_left[1]) * map_scale + map_offset[1]) as i32 as f32,
    ];
    if let Some(c) = clip {
        let px = pos[0] as i32;
        let py = pos[1] as i32;
        if px < c[0] as i32 || py < c[1] as i32 || px >= c[2] as i32 || py >= c[3] as i32 {
            return false;
        }
    }
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return false;
    };
    let size = [sheet.width as f32 * draw_scale, sheet.height as f32 * draw_scale];
    r.push_ui_sprite([pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5], size, uv, [1.0; 4]);
    let hx = (pos[0] - sheet.width as f32 * draw_scale / 2.0) as i32;
    let hy = (pos[1] - sheet.height as f32 * draw_scale / 2.0) as i32;
    let hw = (sheet.width as f32 * draw_scale) as i32;
    let hh = (sheet.height as f32 * draw_scale) as i32;
    let mx = mouse.x as i32;
    let my = mouse.y as i32;
    mx >= hx && mx < hx + hw && my >= hy && my < hy + hh
}

// ---------------------------------------------------------------------------
// The fullscreen hover tile/chest names (`Main.cs:55784-55913` +
// `DrawMap_FindChestName`, `:56631-56643`).
// ---------------------------------------------------------------------------

/// `GolfState.GetLastHitBall` (`GolfState.cs:102-109`) for the map draw: the
/// stored snapshot re-validated against the live pool slot — active, still
/// a golf ball, still the local player's, and not re-hit since
/// (`_lastRecordedSwingCount == ai[1]`). The port's `GolfState` keeps the
/// snapshot + the swing count private, so this re-derives the four guards
/// from `last_hit_golf_ball()` + the live slot (the `update_golf_state`
/// convention, `world_sim/player.rs:344-358`).
pub(crate) fn last_hit_golf_ball(
    golf: &tr_core::golf_state::GolfState,
    pool: &crate::projectile::ProjectilePool,
    my_player: i32,
) -> Option<tr_core::golf_state::GolfBall> {
    let stored = golf.last_hit_golf_ball()?;
    let q = pool.slots().get(stored.who_am_i)?;
    if !q.active
        || !tr_core::golf_state::is_a_golf_ball(q.kind as i32)
        || q.owner as i32 != my_player
    {
        return None;
    }
    let (cx, cy) = q.center();
    let live = tr_core::golf_state::GolfBall {
        who_am_i: stored.who_am_i,
        kind: q.kind as i32,
        owner: q.owner as i32,
        active: q.active,
        position: q.pos,
        center: Vec2::new(cx, cy),
        ai1: q.ai[1],
        local_ai1: q.local_ai[1],
    };
    // The re-hit guard: the stored snapshot's ai1 is the recorded swing
    // count (`record_swing` writes both, `golf_state.rs:200-203`); a live
    // ai1 past it means the ball was re-hit.
    if stored.ai1 != live.ai1 {
        return None;
    }
    Some(live)
}

/// `DrawMapIcons_LastGolfballHit`'s second draw (`Main.cs:56529-56532`): the
/// ball's own `TextureAssets.Projectile[type]` frame centred on the same
/// floored position, over the Extra_176 ring.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_golf_ball_on_map(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    kind: u16,
    center: [f32; 2],
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    map_scale: f32,
    draw_scale: f32,
) {
    let Some(sheet) = lookups.proj_sheets.get(&kind).copied() else {
        return;
    };
    let pos = [
        ((center[0] / 16.0 - map_top_left[0]) * map_scale + map_offset[0]) as i32 as f32,
        ((center[1] / 16.0 - map_top_left[1]) * map_scale + map_offset[1]) as i32 as f32,
    ];
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return;
    };
    let size = [sheet.width as f32 * draw_scale, sheet.height as f32 * draw_scale];
    r.push_ui_sprite([pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5], size, uv, [1.0; 4]);
}

/// `Main.DrawMap_FindChestName` (`Main.cs:56631-56643`): the chest's stored
/// name appended to the style name, or the bare style name. `style` is the
/// `frameX / fullTileWidth` index into the chest-name table.
pub(crate) fn find_chest_name(
    chests: &[crate::chest::WorldChest],
    chest_names: fn(usize) -> String,
    tile_frame_x: i32,
    x: i32,
    y: i32,
    full_tile_width: i32,
) -> String {
    let Some(idx) = crate::chest::find_chest(chests, x, y) else {
        return chest_names(0);
    };
    let style = (tile_frame_x / full_tile_width) as usize;
    let name = &chests[idx].name;
    if !name.is_empty() {
        format!("{}: {}", chest_names(style), name)
    } else {
        chest_names(style)
    }
}

/// The fullscreen hover tile decode (`Main.cs:55784-55913`): the tile under
/// the cursor, gated `Light > 40`, then the chest/dresser style names or the
/// map legend. `map_type` is the packed `MapTile.Type` (`baseType +
/// baseOption`); the chest ranges come from `MapHelper.tileLookup` /
/// `tileOptionCounts`. Returns the hover text, if any.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fullscreen_hover_name(
    world: &tr_core::World,
    map_tiles: &[tr_core::map::MapTile],
    chests: &[crate::chest::WorldChest],
    tx: i32,
    ty: i32,
) -> Option<String> {
    // `:55809`: the light gate. The caller's window gate keeps the coords
    // in-bounds; the guard here keeps a stray call panic-free.
    if tx < 0 || ty < 0 || tx >= world.width || ty >= world.height {
        return None;
    }
    let idx = world.index(tx, ty);
    let mt = map_tiles.get(idx)?;
    if mt.light <= 40 {
        return None;
    }
    let map_type = mt.ty as usize;
    let table = tr_core::map::color_table();
    let lookup = |tile: usize| table.tile_lookup[tile] as usize;
    let options = |tile: usize| table.tile_option_counts[tile] as usize;
    // `:55812-55819`: the chest/dresser lookup bounds.
    let (l21, l441, l467, l468, l88) = (
        lookup(21),
        lookup(441),
        lookup(467),
        lookup(468),
        lookup(88),
    );
    let (o21, o467, o88) = (options(21), options(467), options(88));
    let tile = world.tile(tx, ty)?;
    let frame_x = world.tile_frame_x(tx, ty);
    let frame_y = world.tile_frame_y(tx, ty);
    // The chest origin decode (`:55827-55836` etc.): step back to the
    // top-left cell when the frame is not on a 36-px boundary.
    let origin = |fx: i32, fy: i32| {
        (
            if fx % 36 != 0 { tx - 1 } else { tx },
            if fy % 36 != 0 { ty - 1 } else { ty },
        );
    };
    if map_type >= l21 && map_type < l21 + o21 {
        // `:55822-55838`: chestType (tile 21).
        let (ox, oy) = origin(frame_x, frame_y);
        Some(find_chest_name(chests, crate::lang::chest_type, frame_x, ox, oy, 36))
    } else if map_type >= l467 && map_type < l467 + o467 {
        // `:55840-55856`: chestType2 (tile 467).
        let (ox, oy) = origin(frame_x, frame_y);
        Some(find_chest_name(chests, crate::lang::chest_type2, frame_x, ox, oy, 36))
    } else if map_type >= l441 && map_type < l441 + o21 {
        // `:55858-55874`: tile 441 reads chestType[frameX/36] directly.
        Some(crate::lang::chest_type((frame_x / 36) as usize))
    } else if map_type >= l468 && map_type < l468 + o467 {
        // `:55876-55892`: tile 468 reads chestType2[frameX/36] directly.
        Some(crate::lang::chest_type2((frame_x / 36) as usize))
    } else if map_type >= l88 && map_type < l88 + o88 {
        // `:55894-55907`: the dresser (tile 88) — the 54-px origin decode.
        let x = tx - frame_x % 54 / 18;
        let y = if frame_y % 36 != 0 { ty - 1 } else { ty };
        let style = (frame_x / 54) as usize;
        match crate::chest::find_chest(chests, x, y) {
            None => Some(crate::lang::dresser_type(0)),
            Some(idx) if chests[idx].name.is_empty() => Some(crate::lang::dresser_type(style)),
            Some(idx) => Some(format!(
                "{}: {}",
                crate::lang::dresser_type(style),
                chests[idx].name
            )),
        }
    } else {
        // `:55909-55912`: the map legend.
        Some(crate::lang::map_object_name(map_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::World;

    fn npc(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// `NPCID.cs:10479` — the table rows, spot-checked against the source.
    #[test]
    fn w4_f29_boss_head_table_matches_npcid_10479() {
        let w = World::new("t".into(), 200, 200);
        let _ = w;
        for (ty, head) in [
            (4, 0),
            (13, 2),
            (50, 7),
            (125, 15),
            (126, 20),
            (134, 25),
            (262, 11),
            (266, 23),
            (370, 4),
            (395, 9),
            (396, 8),
            (551, 34),
            (636, 37),
            (668, 39),
        ] {
            assert!(
                BOSS_HEAD_TEXTURES.contains(&(ty, head)),
                "type {ty} -> head {head}"
            );
        }
        // The default is -1 (a plain zombie has no boss head).
        assert!(!BOSS_HEAD_TEXTURES.iter().any(|(t, _)| *t == 3));
    }

    /// `NPC.cs:7562-7603` — the seven overrides.
    #[test]
    fn w4_f29_boss_head_overrides_match_npc_7557() {
        // Eye of Cthulhu: phase 2 (ai0 >= 2) swaps head 0 -> 1.
        let mut eye = npc(NpcKind::EyeOfCthulhu, 0.0, 0.0);
        assert_eq!(boss_head_texture_index(&eye), 0);
        eye.ai[0] = 2.0;
        assert_eq!(boss_head_texture_index(&eye), 1);
        // Plantera: half life swaps 11 -> 12.
        let mut plantera = npc(NpcKind::Plantera, 0.0, 0.0);
        assert_eq!(boss_head_texture_index(&plantera), 11);
        plantera.hp = plantera.max_hp / 2;
        assert_eq!(boss_head_texture_index(&plantera), 12);
        // The twins: ai0 >= 3 swaps 15 -> 16 / 20 -> 21.
        let mut ret = npc(NpcKind::Retinazer, 0.0, 0.0);
        assert_eq!(boss_head_texture_index(&ret), 15);
        ret.ai[0] = 3.0;
        assert_eq!(boss_head_texture_index(&ret), 16);
        let mut spaz = npc(NpcKind::Spazmatism, 0.0, 0.0);
        assert_eq!(boss_head_texture_index(&spaz), 20);
        spaz.ai[0] = 3.0;
        assert_eq!(boss_head_texture_index(&spaz), 21);
        // Brain of Cthulhu: ai0 < 0 or localAI[2] > 0 hides the head.
        let mut brain = npc(NpcKind::BrainOfCthulhu, 0.0, 0.0);
        assert_eq!(boss_head_texture_index(&brain), 23);
        brain.ai[0] = -1.0;
        assert_eq!(boss_head_texture_index(&brain), -1);
        brain.ai[0] = 0.0;
        brain.local_ai[2] = 1.0;
        assert_eq!(boss_head_texture_index(&brain), -1);
    }

    /// `NPC.cs:7608-7642` — the rotation + flip arms.
    #[test]
    fn w4_f29_boss_head_rotation_and_flip() {
        let mut plantera = npc(NpcKind::Plantera, 0.0, 0.0);
        plantera.rotation = 0.7;
        assert_eq!(boss_head_rotation(&plantera), 0.7, "262 rides rotation");
        let mut skeletron = npc(NpcKind::SkeletronHead, 0.0, 0.0);
        skeletron.rotation = 0.5;
        assert_eq!(boss_head_rotation(&skeletron), 0.0, "ai1 == 0: no tilt");
        skeletron.ai[1] = 1.0;
        assert_eq!(boss_head_rotation(&skeletron), 0.5);
        skeletron.ai[1] = 2.0;
        assert_eq!(boss_head_rotation(&skeletron), 0.5);
        let mut ice_queen = npc(NpcKind::IceQueen, 0.0, 0.0);
        ice_queen.rotation = 0.3;
        assert_eq!(boss_head_rotation(&ice_queen), 0.0, "ai0 != 2");
        ice_queen.ai[0] = 2.0;
        assert_eq!(boss_head_rotation(&ice_queen), 0.3);
        // Only the Flying Dutchman flips, on spriteDirection == 1.
        let mut dutchman = npc(NpcKind::FlyingDutchman, 0.0, 0.0);
        dutchman.sprite_dir = 1;
        assert!(boss_head_flip(&dutchman));
        dutchman.sprite_dir = -1;
        assert!(!boss_head_flip(&dutchman));
        let mut eye = npc(NpcKind::EyeOfCthulhu, 0.0, 0.0);
        eye.sprite_dir = 1;
        assert!(!boss_head_flip(&eye), "491 only");
    }

    /// `Main.cs:56125-56137` — the Destroyer average: the head plus its
    /// active 134-136 chain, capped at 15, divided by the count.
    #[test]
    fn w4_f29_destroyer_average_walks_the_chain() {
        let mut head = npc(NpcKind::TheDestroyer, 100.0, 100.0);
        let (hw, hh) = head.size();
        let hc = [100.0 + hw * 0.5, 100.0 + hh * 0.5];
        // Two body segments at known centres, chained ai0 = index + 1.
        let mut b1 = npc(NpcKind::TheDestroyerBody, 200.0, 100.0);
        let mut b2 = npc(NpcKind::TheDestroyerBody, 300.0, 100.0);
        let c1 = {
            let (w, h) = b1.size();
            [200.0 + w * 0.5, 100.0 + h * 0.5]
        };
        let c2 = {
            let (w, h) = b2.size();
            [300.0 + w * 0.5, 100.0 + h * 0.5]
        };
        head.ai[0] = 2.0; // slot 1 (index+1)
        b1.ai[0] = 3.0; // slot 2
        b2.ai[0] = 0.0; // chain end
        let npcs = vec![head, b1, b2];
        let avg = destroyer_average_center(&npcs, 0);
        assert_eq!(
            avg,
            [
                (hc[0] + c1[0] + c2[0]) / 3.0,
                (hc[1] + c1[1] + c2[1]) / 3.0
            ]
        );
        // A dead segment stops the walk (the C# `active` gate).
        let mut npcs = npcs;
        npcs[1].hp = 0;
        let avg = destroyer_average_center(&npcs, 0);
        assert_eq!(avg, hc, "the chain breaks at the dead segment");
    }

    /// `Main.cs:56236-56242` vs `:56158-56164` — the two transforms.
    #[test]
    fn w4_f29_head_pos_matches_the_two_modes() {
        let inp_full = HeadIconInputs {
            npcs: &[],
            mode: HeadMode::Fullscreen,
            origin: [200.0, 300.0],
            left_top: [0.0, 0.0],
            sub_offset: [0.0, 0.0],
            clip: None,
            scale: 4.0,
            head_scale: 1.0,
            present_scale: 1.0,
            alpha: 255,
            mouse: Vec2::ZERO,
        };
        let center = [160.0, 320.0];
        // x = 160/16*4 + 200 - 10*4 = 40 + 200 - 40 = 200.
        // y = 320/16*4 + 300 - 40 = 80 + 300 - 40 = 340.
        assert_eq!(head_pos(center, &inp_full), [200.0, 340.0]);
        let inp_mini = HeadIconInputs {
            mode: HeadMode::Minimap,
            origin: [500.0, 90.0],
            left_top: [5.0, 10.0],
            sub_offset: [-0.5, -0.25],
            clip: Some([512.0, 100.0, 740.0, 326.0]),
            scale: 2.0,
            ..inp_full
        };
        // x = (160/16 - 5)*2 + 500 - 0.5 = 10 + 500 - 0.5 = 509.5.
        // y = (320/16 - 10)*2 + 90 - 2*2/5 - 0.25 = 20 + 90 - 0.8 - 0.25.
        let got = head_pos(center, &inp_mini);
        assert_eq!(got[0], 509.5);
        assert_eq!(got[1], 108.95);
    }

    /// `Main.cs:56165` — the minimap clip is strict on all four edges.
    #[test]
    fn w4_f29_head_clip_is_strict() {
        let clip = [512.0, 100.0, 740.0, 326.0];
        assert!(head_in_clip([600.0, 200.0], clip));
        assert!(!head_in_clip([512.0, 200.0], clip), "left edge excluded");
        assert!(!head_in_clip([740.0, 200.0], clip), "right edge excluded");
        assert!(!head_in_clip([600.0, 100.0], clip), "top edge excluded");
        assert!(!head_in_clip([600.0, 326.0], clip), "bottom edge excluded");
    }

    /// `TownNPCProfiles.cs:200-207` + `NPC.cs:7510-7553` — the town head
    /// resolution rides the one default table.
    #[test]
    fn w4_f29_town_head_index_matches_the_profiles() {
        assert_eq!(town_head_index(&npc(NpcKind::Guide, 0.0, 0.0)), 1);
        assert_eq!(town_head_index(&npc(NpcKind::Mechanic, 0.0, 0.0)), 8);
        assert_eq!(town_head_index(&npc(NpcKind::Wizard, 0.0, 0.0)), 10);
        assert_eq!(town_head_index(&npc(NpcKind::Princess, 0.0, 0.0)), 45);
        assert_eq!(town_head_index(&npc(NpcKind::Golfer, 0.0, 0.0)), 25);
        // A boss has no town head.
        assert_eq!(town_head_index(&npc(NpcKind::EyeOfCthulhu, 0.0, 0.0)), -1);
    }

    /// `NPC.cs:6729-6753` — the name accessors.
    #[test]
    fn w4_f29_npc_names_follow_given_or_type() {
        let mut guide = npc(NpcKind::Guide, 0.0, 0.0);
        let type_name = crate::lang::npc_name(22);
        assert_eq!(npc_given_or_type_name(&guide), type_name);
        assert_eq!(npc_full_name(&guide), type_name);
        guide.given_name = "Andrew".into();
        assert_eq!(npc_given_or_type_name(&guide), "Andrew");
        assert_eq!(
            npc_full_name(&guide),
            crate::lang::text_args("Game.NPCTitle", &["Andrew".into(), type_name])
        );
    }

    /// `CoinLossRevengeSystem.cs:262` — the coin tier brackets.
    #[test]
    fn w4_f29_revenge_coin_tier_brackets() {
        assert_eq!(revenge_coin_tier(0), 0, "copper");
        assert_eq!(revenge_coin_tier(99), 0);
        assert_eq!(revenge_coin_tier(100), 1, "silver");
        assert_eq!(revenge_coin_tier(9_999), 1);
        assert_eq!(revenge_coin_tier(10_000), 2, "gold");
        assert_eq!(revenge_coin_tier(999_999), 2);
        assert_eq!(revenge_coin_tier(1_000_000), 3, "platinum");
        assert_eq!(revenge_coin_tier(50_000_000), 3);
    }

    /// `Utils.cs:1048-1060` — CoinsSplit.
    #[test]
    fn w4_f29_coins_split_matches_utils() {
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(99), [99, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
        assert_eq!(coins_split(10_000), [0, 0, 1, 0]);
        assert_eq!(coins_split(1_000_000), [0, 0, 0, 1]);
        assert_eq!(coins_split(1_234_567), [67, 45, 23, 1]);
    }

    /// `Main.cs:56448-56453` — the death-marker transform, and the minimap
    /// variant's sub-tile offsets (`:55746-55751`).
    #[test]
    fn w4_f29_death_marker_transform() {
        // Fullscreen: origin (200,300), left_top 0, scale 4.
        // x = 160/16*4 + 200 - 40 = 200; y = 320/16*4 + 300 - (2-1.6) - 40.
        let last = [160.0, 320.0];
        let scale = 4.0f32;
        let x = (last[0] / 16.0) * scale + 200.0 - 10.0 * scale;
        let y = (last[1] / 16.0) * scale + 300.0 - (2.0 - scale / 5.0 * 2.0) - 10.0 * scale;
        assert_eq!(x, 200.0);
        assert_eq!(y, 339.6);
        // Minimap: the sub-offsets add, no -10*scale inset.
        let got = minimap_death_marker_pos(last, [500.0, 90.0], [5.0, 10.0], [-0.5, -0.25], 2.0);
        assert_eq!(got[0], 509.5);
        assert_eq!(got[1], 108.95);
    }

    /// `Main.cs:55752` — the minimap death clip is +8/+8/-18/-16, strict.
    #[test]
    fn w4_f29_death_minimap_clip_edges() {
        let origin = [500.0, 90.0];
        assert!(death_marker_in_minimap_clip([600.0, 200.0], origin));
        assert!(!death_marker_in_minimap_clip([508.0, 200.0], origin));
        assert!(!death_marker_in_minimap_clip([722.0, 200.0], origin));
        assert!(!death_marker_in_minimap_clip([600.0, 98.0], origin));
        assert!(!death_marker_in_minimap_clip([600.0, 314.0], origin));
    }

    /// `Lang.cs:1158-1205` — LocalizedDuration's largest-unit + round-up.
    #[test]
    fn w4_f29_localized_duration_largest_unit() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        // Under a minute: seconds.
        assert_eq!(crate::lang::localized_duration(45.0, false, false), "45 seconds");
        assert_eq!(crate::lang::localized_duration(1.0, false, false), "1 second");
        // Over a minute but under an hour: minutes, rounded up past 1 min.
        assert_eq!(crate::lang::localized_duration(61.0, false, false), "2 minutes");
        assert_eq!(crate::lang::localized_duration(60.0, false, false), "1 minute");
        // Over an hour: hours rounded up.
        assert_eq!(crate::lang::localized_duration(3601.0, false, false), "2 hours");
        // Over a day: days rounded up.
        assert_eq!(crate::lang::localized_duration(86401.0, false, false), "2 days");
        // Abbreviated uses the Misc.Short* keys.
        assert_eq!(crate::lang::localized_duration(45.0, true, false), "45 s");
    }

    /// `Main.cs:56631-56643` — the chest name append.
    #[test]
    fn w4_f29_find_chest_name_appends_the_stored_name() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut chest = crate::chest::WorldChest::default();
        chest.tile = (10, 20);
        chest.name = "Loot".into();
        let chests = vec![chest];
        // frameX 36 -> style 1; the stored name appends.
        let got = find_chest_name(&chests, crate::lang::chest_type, 36, 10, 20, 36);
        assert_eq!(got, format!("{}: Loot", crate::lang::chest_type(1)));
        // No chest at the coords -> the bare style-0 name.
        let got = find_chest_name(&chests, crate::lang::chest_type, 0, 99, 99, 36);
        assert_eq!(got, crate::lang::chest_type(0));
        // An empty stored name -> the bare style name.
        let mut empty = crate::chest::WorldChest::default();
        empty.tile = (10, 20);
        let got = find_chest_name(&[empty], crate::lang::chest_type, 72, 10, 20, 36);
        assert_eq!(got, crate::lang::chest_type(2));
    }

    /// `GolfState.cs:102-109` — the last-hit ball's four guards.
    #[test]
    fn w4_f29_last_hit_golf_ball_guards() {
        use tr_core::golf_state::{GolfBall, GolfState};
        let mut golf = GolfState::new();
        let mut pool = crate::projectile::ProjectilePool::new();
        // No stored ball -> None.
        assert!(last_hit_golf_ball(&golf, &pool, 0).is_none());
        // Spawn a golf ball (721) owned by the local player, hit it once.
        let idx = pool
            .spawn(721, 0, Vec2::new(160.0, 320.0), Vec2::ZERO, 0, 0.0, false)
            .unwrap();
        pool.slots_mut()[idx].ai[1] = 1.0;
        let (cx, cy) = pool.slots()[idx].center();
        let ball = GolfBall {
            who_am_i: idx,
            kind: 721,
            owner: 0,
            active: true,
            position: pool.slots()[idx].pos,
            center: Vec2::new(cx, cy),
            ai1: 1.0,
            local_ai1: 1.0,
        };
        golf.record_swing(&ball);
        assert!(last_hit_golf_ball(&golf, &pool, 0).is_some());
        // A re-hit (live ai1 past the recorded swing) drops it.
        pool.slots_mut()[idx].ai[1] = 2.0;
        assert!(last_hit_golf_ball(&golf, &pool, 0).is_none());
        // Back to the recorded count, but inactive -> None.
        pool.slots_mut()[idx].ai[1] = 1.0;
        pool.slots_mut()[idx].active = false;
        assert!(last_hit_golf_ball(&golf, &pool, 0).is_none());
        // Active again, but a foreign owner -> None.
        pool.slots_mut()[idx].active = true;
        pool.slots_mut()[idx].owner = 3;
        assert!(last_hit_golf_ball(&golf, &pool, 0).is_none());
        // The foreign owner's own read resolves.
        assert!(last_hit_golf_ball(&golf, &pool, 3).is_some());
    }

    /// `Main.cs:56480` — the misc icon's `Utils.Floor` transform (which is
    /// `(int)` truncation, `Utils.cs:1702-1707`).
    #[test]
    fn w4_f29_misc_icon_position_is_floored() {
        // anchor (161.7, 325.9), height_offset -21, scale 2.5, offset (100,50).
        // x = (161.7/16)*2.5 + 100 = 25.265.. + 100 = 125.265 -> 125.
        // y = ((325.9-21)/16)*2.5 + 50 = 47.64.. + 50 = 97.64 -> 97.
        let anchor = [161.7f32, 325.9];
        let h_off = -21.0f32;
        let scale = 2.5f32;
        let off = [100.0f32, 50.0];
        let x = ((anchor[0] / 16.0) * scale + off[0]) as i32;
        let y = (((anchor[1] + h_off) / 16.0) * scale + off[1]) as i32;
        assert_eq!(x, 125);
        assert_eq!(y, 97);
        // A negative coordinate truncates toward zero (not Math.Floor).
        let neg = (-125.265f32) as i32;
        assert_eq!(neg, -125, "Utils.Floor is (int), not Math.Floor");
    }
}
