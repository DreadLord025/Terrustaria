//! The map's NPC head overlays — `Main.DrawNPCMapIcons`/`2`/`3`
//! (`Main.cs:56099-56292`), the boss half (`DrawNPCHeadBoss`, `:56562-56565`
//! → `OutlinedTextureRenderer.DrawWithOutlines`) and the town half
//! (`DrawNPCHeadFriendly`, `:56553-56560`). The head-index resolvers are
//! `NPC.GetBossHeadTextureIndex`/`GetBossHeadRotation`/
//! `GetBossHeadSpriteEffects` (`NPC.cs:7557-7642`) over
//! `NPCID.Sets.BossHeadTextures` (`NPCID.cs:10479`) and
//! `TownNPCProfiles.GetHeadIndexSafe` (`TownNPCProfiles.cs:200-207`).
//!
//! W4-e17 (the f29 census's first half, items 3+4). The render-target
//! composite is reproduced as direct quads like `draw_player_map_icon`
//! (`render/entities.rs`): the outlined target is the head sheet padded 4 px
//! per side (`OutlinedDrawRenderTargetContent.cs:16-17`), blitted centred
//! (`OutlinedTextureRenderer.cs:38`), so the head itself sits centred on the
//! draw position and the baked rings ride at Manhattan 4 (black) / 2
//! (border) target pixels (`AnOutlinedDrawRenderTargetContent.cs:38-60`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::npc::{Npc, NpcKind};

/// The vanilla `NPC.type` of a boss-head kind — the head table is keyed by
/// the C# type, NOT the folded bestiary net id (the Golem parts fold onto
/// 245, but only the 246/249 heads draw; `NPCID.cs:10479` has no 245 row).
fn boss_head_type(kind: NpcKind) -> Option<i32> {
    Some(match kind {
        NpcKind::EyeOfCthulhu => 4,
        NpcKind::EaterOfWorldsHead => 13,
        NpcKind::Everscream => 344,
        NpcKind::DukeFishron => 370,
        NpcKind::GolemHead => 246,
        NpcKind::GolemHeadFree => 249,
        NpcKind::IceQueen => 345,
        NpcKind::KingSlime => 50,
        NpcKind::MoonLordHead => 396,
        NpcKind::MartianSaucerCore => 395,
        NpcKind::MourningWood => 325,
        NpcKind::Plantera => 262,
        NpcKind::Pumpking => 327,
        NpcKind::QueenBee => 222,
        NpcKind::Retinazer => 125,
        NpcKind::Spazmatism => 126,
        NpcKind::SantaNK1 => 346,
        NpcKind::SkeletronPrime => 127,
        NpcKind::SkeletronHead => 35,
        NpcKind::WallOfFlesh => 113,
        NpcKind::BrainOfCthulhu => 266,
        NpcKind::AncientCultist => 439,
        NpcKind::TheDestroyer => 134,
        NpcKind::FlyingDutchman => 491,
        NpcKind::SolarTower => 517,
        NpcKind::VortexTower => 422,
        NpcKind::NebulaTower => 507,
        NpcKind::StardustTower => 493,
        NpcKind::DD2LanePortal => 549,
        NpcKind::DD2DarkMage => 564,
        NpcKind::DD2DarkMageT3 => 565,
        NpcKind::DD2OgreT2 => 576,
        NpcKind::DD2OgreT3 => 577,
        NpcKind::DD2Betsy => 551,
        NpcKind::DD2EterniaCrystal => 548,
        NpcKind::Deerclops => 668,
        // 68 (Dungeon Guardian), 440 (Cultist Boss Clone), 636 (Empress of
        // Light) and 657 (Queen Slime) have table rows but no port kind yet.
        _ => return None,
    })
}

/// `NPCID.Sets.BossHeadTextures` (`NPCID.cs:10479`), the `CreateIntSet(-1,
/// ...)` pairs verbatim. Unlisted types answer -1.
fn boss_head_base(ty: i32) -> i32 {
    match ty {
        4 => 0,
        13 => 2,
        344 => 3,
        370 => 4,
        246 | 249 => 5,
        345 => 6,
        50 => 7,
        396 => 8,
        395 => 9,
        325 => 10,
        262 => 11,
        327 => 13,
        222 => 14,
        125 => 15,
        126 => 20,
        346 => 17,
        127 => 18,
        35 | 68 => 19,
        113 => 22,
        266 => 23,
        439 | 440 => 24,
        134 => 25,
        491 => 26,
        517 => 27,
        422 => 28,
        507 => 29,
        493 => 30,
        549 => 35,
        564 | 565 => 32,
        576 | 577 => 33,
        551 => 34,
        548 => 36,
        636 => 37,
        657 => 38,
        668 => 39,
        _ => -1,
    }
}

/// `NPC.GetBossHeadTextureIndex` (`NPC.cs:7557-7606`): the table row plus
/// the seven type overrides. -1 = no head draws.
pub(crate) fn boss_head_texture_index(n: &Npc) -> i32 {
    let Some(ty) = boss_head_type(n.kind) else {
        return -1;
    };
    let mut result = boss_head_base(ty);
    match ty {
        // `:7562-7567`: the Eye's second phase swaps to the torn head.
        4 if n.ai[0] >= 2.0 => result = 1,
        // `:7568-7573`: the Brain's illusion/teleport states hide the head.
        266 if n.ai[0] < 0.0 || n.local_ai[2] > 0.0 => result = -1,
        // `:7574-7579`: Plantera's second phase — INTEGER lifeMax/2.
        262 if n.hp <= n.max_hp / 2 => result = 12,
        // `:7580-7591`: the twins' laser/spit phases.
        125 if n.ai[0] >= 3.0 => result = 16,
        126 if n.ai[0] >= 3.0 => result = 21,
        // `:7592-7603`: the Cultist's clone swap (439 hides at ai0 5 +
        // alpha 255; 440 — unported — shows only there).
        439 if n.ai[0] == 5.0 && n.alpha == 255 => result = -1,
        440 if n.ai[0] != 5.0 || n.alpha == 255 => result = -1,
        _ => {}
    }
    result
}

/// `NPC.GetBossHeadRotation` (`NPC.cs:7608-7632`): only the spinning bosses
/// tilt the head, and only in their spin states.
pub(crate) fn boss_head_rotation(n: &Npc) -> f32 {
    let Some(ty) = boss_head_type(n.kind) else {
        return 0.0;
    };
    match ty {
        68 | 262 => n.rotation,
        35 | 127 if n.ai[1] == 1.0 || n.ai[1] == 2.0 => n.rotation,
        345 if n.ai[0] == 2.0 => n.rotation,
        _ => 0.0,
    }
}

/// `NPC.GetBossHeadSpriteEffects` (`NPC.cs:7634-7642`): the Flying
/// Dutchman flips when `spriteDirection == 1`.
pub(crate) fn boss_head_flip(n: &Npc) -> bool {
    boss_head_type(n.kind) == Some(491) && n.sprite_dir == 1
}

/// `TownNPCProfiles.GetHeadIndexSafe` (`TownNPCProfiles.cs:200-207`): the
/// profile dict's head id, else `NPC.TypeToDefaultHeadIndex`. The stacked
/// shimmer profiles pick the variant by `townNpcVariationIndex`
/// (`Profiles.cs:44-52` — index 1 = the shimmered head, out-of-range falls
/// to 0); the pet `VariantNPCProfile`s index their head-id arrays
/// (`Profiles.cs:238-241`); the town slimes are plain `LegacyNPCProfile`s
/// with fixed heads (`TownNPCProfiles.cs:150-180`).
pub(crate) fn town_head_index(n: &Npc) -> i32 {
    let ty = crate::render::entities::texture_net_id(n.kind) as i32;
    // The `LegacyWithSimpleShimmer`/`TransformableWithSimpleShimmer` rows
    // (`TownNPCProfiles.cs:21-136`): (normal, shimmered).
    let stacked: Option<(i32, i32)> = match ty {
        22 => Some((1, 72)),
        20 => Some((5, 73)),
        19 => Some((6, 74)),
        107 => Some((9, 75)),
        160 => Some((12, 76)),
        208 => Some((15, 77)),
        228 => Some((18, 78)),
        550 => Some((24, 79)),
        369 => Some((22, 55)),
        54 => Some((7, 57)),
        209 => Some((16, 58)),
        38 => Some((4, 59)),
        207 => Some((14, 60)),
        588 => Some((25, 61)),
        124 => Some((8, 62)),
        17 => Some((2, 63)),
        18 => Some((3, 64)),
        227 => Some((17, 65)),
        229 => Some((19, 66)),
        142 => Some((11, 67)),
        178 => Some((13, 68)),
        353 => Some((20, 69)),
        441 => Some((23, 70)),
        108 => Some((10, 71)),
        663 => Some((45, 54)),
        633 => Some((26, 56)),
        37 | 453 => Some((-1, -1)),
        368 => Some((21, 80)),
        _ => None,
    };
    if let Some((normal, shimmered)) = stacked {
        return if n.variation_index == 1 { shimmered } else { normal };
    }
    // The pet variants (`TownNPCProfiles.cs:138-148` + the head-id arrays
    // `:211-213`): Cat 27..32, Dog 33..38, Bunny 39..44.
    let pet_base = match ty {
        637 => Some(27),
        638 => Some(33),
        656 => Some(39),
        _ => None,
    };
    if let Some(base) = pet_base {
        // `VariantNPCProfile.GetHeadTextureIndex` indexes the array
        // directly; an out-of-range variation would throw in the C# — the
        // port clamps into the 6-row array (the variation roll is 0..6).
        return base + n.variation_index.clamp(0, 5);
    }
    // The town slimes' fixed heads (`TownNPCProfiles.cs:150-180`).
    match ty {
        670 => 46,
        678 => 47,
        679 => 48,
        680 => 49,
        681 => 50,
        682 => 51,
        683 => 52,
        684 => 53,
        // No profile: `NPC.TypeToDefaultHeadIndex` (`NPC.cs:7510-7553`).
        _ => crate::ui::housing::type_to_head_index(ty),
    }
}

/// `NPC.FullName` (`NPC.cs:6729-6739`): the given name + the type name via
/// `Game.NPCTitle` ("{0} the {1}"), or the bare `TypeName` when unnamed.
pub(crate) fn full_name(n: &Npc) -> String {
    let type_name = crate::lang::npc_name(n.kind.net_id());
    if n.given_name.is_empty() {
        type_name
    } else {
        crate::lang::text_args("Game.NPCTitle", &[n.given_name.clone(), type_name])
    }
}

/// `NPC.GivenOrTypeName` (`NPC.cs:6743-6753`).
pub(crate) fn given_or_type_name(n: &Npc) -> String {
    if n.given_name.is_empty() {
        crate::lang::npc_name(n.kind.net_id())
    } else {
        n.given_name.clone()
    }
}

/// The Destroyer head's segment-average centre (`Main.cs:56125-56137`):
/// `center = head.Center`, then up to 14 more links while the `ai[0]` chain
/// stays active and in 134..=136, `vector2 = center / num2`. The C# walks
/// the `Main.npc[(int)ai[0]]` slot chain; the port's stable chain identity
/// is `worm_id` + `seg_index` (`npc_ai_style37.rs` — the pool compacts, the
/// slot links do not survive), so the walk takes the same-worm segments in
/// chain order and stops at the first inactive/missing link, exactly the
/// C#'s `active && type` gate. No gfxOffY/netOffset on this path — the C#
/// overwrites `vector2` outright.
pub(crate) fn destroyer_head_center(npcs: &[Npc], head: usize) -> [f32; 2] {
    let n = &npcs[head];
    let (hx, hy) = n.center();
    let mut center = [hx, hy];
    let mut count = 1i32;
    // The chain order: seg_index 1, 2, ... of the same worm.
    let mut seg = 1u32;
    while count < 15 {
        let link = npcs.iter().find(|l| {
            l.worm_id == n.worm_id
                && l.worm_id != 0
                && l.seg_index == seg
                && crate::npc_ai_style37::is_destroyer_family(l.kind)
        });
        match link {
            // `Main.npc[num3].active` — the port's liveness is hp > 0.
            Some(l) if l.hp > 0 => {
                count += 1;
                let (cx, cy) = l.center();
                center[0] += cx;
                center[1] += cy;
                seg += 1;
            }
            _ => break,
        }
    }
    [center[0] / count as f32, center[1] / count as f32]
}

/// The head's draw anchor: `Center + (0, gfxOffY) + netOffset`
/// (`Main.cs:56113`/`:56124` and siblings).
fn head_anchor(n: &Npc) -> [f32; 2] {
    let (cx, cy) = n.center();
    [cx + n.net_offset.x, cy + n.gfx_off_y + n.net_offset.y]
}

/// The hover rect's half extents — the C# reads `(float)(Width() / 2)`:
/// INTEGER division of the sheet width first (`Main.cs:56243-56246`).
fn hover_half(w: u32, h: u32, head_scale: f32) -> [f32; 2] {
    [(w / 2) as f32 * head_scale, (h / 2) as f32 * head_scale]
}

/// One centered, rotated, optionally flipped UI quad — the
/// `OutlinedTextureRenderer` blit's body pass (`OutlinedTextureRenderer.cs:38`:
/// origin `target.Size()/2`, the head content centred inside the 4 px pad).
fn push_centered(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    tint: [f32; 4],
    rotation: f32,
    flip: bool,
) {
    let uv = crate::render::player_layers::flip_uv(uv, flip);
    if rotation == 0.0 {
        r.push_ui_sprite(
            [pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5],
            size,
            uv,
            tint,
        );
        return;
    }
    // The instance pipeline rotates about the top-left; shift it so the
    // rotation pivots on the centre (`push_sprite_origin_rot`'s math).
    let (s, c) = rotation.sin_cos();
    let (ox, oy) = (size[0] * 0.5, size[1] * 0.5);
    let tl = [
        pos[0] - (ox * c - oy * s),
        pos[1] - (ox * s + oy * c),
    ];
    r.push_ui_sprite_rot(tl, size, uv, tint, rotation);
}

/// `OutlinedTextureRenderer.DrawWithOutlines` over the baked
/// `AnOutlinedDrawRenderTargetContent` rings: 8 black copies at Manhattan
/// radius 4 (`:38-49`), 4 border copies at radius 2 (`:50-60`), then the
/// head (`:62`). The rings are baked at full alpha and the whole target is
/// blitted with `Color(alpha, alpha, alpha, alpha)` (`Main.cs:56555`/
/// `:56564`), so every pass rides the single alpha multiply — the border
/// is White × the tint, the black ring keeps rgb 0. `pos` is the head
/// centre; the ring offsets are target pixels scaled by `head_scale` and
/// rotated with the blit.
#[allow(clippy::too_many_arguments)]
fn draw_head_outlined(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &tr_render::SheetRect,
    pos: [f32; 2],
    alpha: f32,
    head_scale: f32,
    rotation: f32,
    flip: bool,
) {
    let Some(uv) = lookups.hud_uv(Some(sheet)) else {
        return;
    };
    let size = [sheet.width as f32 * head_scale, sheet.height as f32 * head_scale];
    let body = [alpha, alpha, alpha, alpha];
    let black = [0.0, 0.0, 0.0, alpha];
    let (s, c) = rotation.sin_cos();
    let mut ring = |dx: f32, dy: f32, tint: [f32; 4]| {
        let off = [
            (dx * c - dy * s) * head_scale,
            (dx * s + dy * c) * head_scale,
        ];
        push_centered(
            r,
            [pos[0] + off[0], pos[1] + off[1]],
            size,
            uv,
            tint,
            rotation,
            flip,
        );
    };
    for [dx, dy] in [
        [-4.0, 0.0],
        [4.0, 0.0],
        [0.0, -4.0],
        [0.0, 4.0],
        [-2.0, -2.0],
        [-2.0, 2.0],
        [2.0, -2.0],
        [2.0, 2.0],
    ] {
        ring(dx, dy, black);
    }
    for [dx, dy] in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
        ring(dx, dy, body);
    }
    push_centered(r, pos, size, uv, body, rotation, flip);
}

/// `Utils.DrawNotificationIcon` (`Utils.cs:442-461`) at the head's
/// `(x, y - 20)` (`Main.cs:56558`): the quickicon bubble sways
/// `sin(t % 2 / 2 * 2π) * 2π * 0.035` and its alpha lerps White ↔ White/2
/// by `sin(2π * (t % 1)) * 0.5 + 0.5`; origin `(w/2, h - 4)`, scale 1.
/// The `nextDialogue.ShowIndicator` gate (`Main.cs:56556`) is the town
/// dialogue system's — unported, so the caller passes the flag (false
/// until that system lands).
fn draw_notification_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    pos: [f32; 2],
    global_time: f32,
) {
    let Some(sheet) = lookups.ui_quickicon1 else {
        return;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return;
    };
    let (w, h) = (sheet.width as f32, sheet.height as f32);
    let amount = (std::f32::consts::TAU * (global_time % 1.0)).sin() * 0.5 + 0.5;
    let rot = (global_time % 2.0 / 2.0 * std::f32::consts::TAU).sin()
        * std::f32::consts::TAU
        * 0.035;
    // `Color.Lerp(color, value2, amount)` with value2.A /= 2: rgb stays 1,
    // alpha lerps 1 → 0.5 (the byte `/2` is 127/255).
    let a = 1.0 + (127.0 / 255.0 - 1.0) * amount;
    let tint = [1.0, 1.0, 1.0, a];
    // Origin `(w/2, h - 4)` — the pivot the rotation keeps in place.
    let (s, c) = rot.sin_cos();
    let (ox, oy) = (w * 0.5, h - 4.0);
    let tl = [pos[0] - (ox * c - oy * s), pos[1] - (ox * s + oy * c)];
    r.push_ui_sprite_rot(tl, [w, h], uv, tint, rot);
}

/// `Main.DrawNPCHeadFriendly` (`Main.cs:56553-56560`): the outlined town
/// head plus the notification bubble when the NPC has a pending dialogue.
#[allow(clippy::too_many_arguments)]
fn draw_head_friendly(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    head: i32,
    pos: [f32; 2],
    alpha: f32,
    head_scale: f32,
    flip: bool,
    show_indicator: bool,
    global_time: f32,
) {
    if let Some(sheet) = lookups.npc_heads.get(head as usize).copied().flatten() {
        draw_head_outlined(r, lookups, &sheet, pos, alpha, head_scale, 0.0, flip);
    }
    if show_indicator {
        draw_notification_icon(r, lookups, [pos[0], pos[1] - 20.0], global_time);
    }
}

/// `Main.DrawNPCHeadBoss` (`Main.cs:56562-56565`): the outlined boss head.
#[allow(clippy::too_many_arguments)]
fn draw_head_boss(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    head: i32,
    pos: [f32; 2],
    alpha: f32,
    head_scale: f32,
    rotation: f32,
    flip: bool,
) {
    if let Some(sheet) = lookups.npc_boss_heads.get(head as usize).copied().flatten() {
        draw_head_outlined(r, lookups, &sheet, pos, alpha, head_scale, rotation, flip);
    }
}

/// The per-mode geometry inputs — the C# reads the `Main.DrawMap` locals.
pub(crate) struct NpcIconView {
    /// `num12`/`num13` (`left`/`top`): 0 on the fullscreen map.
    pub map_position: [f32; 2],
    /// The minimap's sub-tile offsets `num10`/`num11` (0 elsewhere).
    pub sub_offset: [f32; 2],
    /// `Main.miniMapX/Y/Width/Height` in the draw's transform space;
    /// `None` on the fullscreen map (no clip).
    pub minimap_rect: Option<[f32; 4]>,
    /// `num5`: px per tile.
    pub map_scale: f32,
    /// `num37`/`num46`/`num77`: the head draw scale.
    pub head_scale: f32,
    /// `b / 255`.
    pub alpha: f32,
    pub mouse: Vec2,
    /// `Main.GlobalTimeWrappedHourly` — the notification sway.
    pub global_time: f32,
}

/// The town-half gate: `active && townNPC && DrawNPCMapIcon_CanBeSeen_Townie`
/// (`Main.cs:56103` — the townie visibility fn is a flat `true`, `:56294-56297`).
fn town_visible(n: &Npc) -> bool {
    n.hp > 0 && n.kind.is_town_npc_like()
}

/// `Main.DrawNPCMapIcons` (`Main.cs:56222-56292`) — the fullscreen map's
/// head pass. `origin` is `num`/`num2` (the map's screen origin); the heads
/// ride `v/16*scale + origin - 10*scale`. Returns the hovered NPC's text
/// (the C# `ref mouseTextString`): town hover `FullName`, boss hover
/// `GivenOrTypeName`; the boss head draws BEFORE its hover test
/// (`:56281-56289`), the town hover runs BEFORE its draw (`:56247-56251`).
pub(crate) fn draw_npc_map_icons_fullscreen(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    npcs: &[Npc],
    origin: [f32; 2],
    view: &NpcIconView,
) -> Option<String> {
    let mut text: Option<String> = None;
    let (x, y) = (origin[0], origin[1]);
    let (scale, hs, a) = (view.map_scale, view.head_scale, view.alpha);
    for (i, n) in npcs.iter().enumerate() {
        if town_visible(n) {
            let head = town_head_index(n);
            if head > 0 {
                let flip = n.facing > 0;
                let v = head_anchor(n);
                let px = v[0] / 16.0 * scale + x - 10.0 * scale;
                let py = v[1] / 16.0 * scale + y - 10.0 * scale;
                if let Some(sheet) = lookups.npc_heads.get(head as usize).copied().flatten() {
                    let (hw, hh) = hover_half(sheet.width, sheet.height, hs);
                    let (hx, hy) = (px - hw, py - hh);
                    if view.mouse.x >= hx
                        && view.mouse.x <= hx + sheet.width as f32 * hs
                        && view.mouse.y >= hy
                        && view.mouse.y <= hy + sheet.height as f32 * hs
                    {
                        text = Some(full_name(n));
                    }
                }
                // `nextDialogue` is unported — the indicator stays off.
                draw_head_friendly(r, lookups, head, [px, py], a, hs, flip, false, view.global_time);
            }
        }
        let boss = boss_head_texture_index(n);
        if n.hp <= 0 || boss == -1 {
            continue;
        }
        let rotation = boss_head_rotation(n);
        let flip = boss_head_flip(n);
        let v = if n.kind == NpcKind::TheDestroyer {
            destroyer_head_center(npcs, i)
        } else {
            head_anchor(n)
        };
        let px = v[0] / 16.0 * scale + x - 10.0 * scale;
        let py = v[1] / 16.0 * scale + y - 10.0 * scale;
        draw_head_boss(r, lookups, boss, [px, py], a, hs, rotation, flip);
        if let Some(sheet) = lookups.npc_boss_heads.get(boss as usize).copied().flatten() {
            let (hw, hh) = hover_half(sheet.width, sheet.height, hs);
            let (hx, hy) = (px - hw, py - hh);
            if view.mouse.x >= hx
                && view.mouse.x <= hx + sheet.width as f32 * hs
                && view.mouse.y >= hy
                && view.mouse.y <= hy + sheet.height as f32 * hs
            {
                text = Some(given_or_type_name(n));
            }
        }
    }
    text
}

/// `Main.DrawNPCMapIcons2` (`Main.cs:56144-56220`) — the minimap's head
/// pass: `(v/16 - left)*scale + X2 + xOff`, the y rides `- 2*scale/5`, and
/// both halves clip to `(miniMapX+12, miniMapY+10, W-16, H-14)` with the
/// C#'s strict `>`/`<` edges (`:56165`/`:56206`). Hover runs BEFORE the
/// draw on both halves here.
pub(crate) fn draw_npc_map_icons_minimap(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    npcs: &[Npc],
    panel_origin: [f32; 2],
    view: &NpcIconView,
) -> Option<String> {
    let mut text: Option<String> = None;
    let Some(rect) = view.minimap_rect else {
        return None;
    };
    let (mx, my, mw, mh) = (rect[0], rect[1], rect[2], rect[3]);
    let (left, top) = (view.map_position[0], view.map_position[1]);
    let (x2, y2) = (panel_origin[0], panel_origin[1]);
    let (xoff, yoff) = (view.sub_offset[0], view.sub_offset[1]);
    let (scale, hs, a) = (view.map_scale, view.head_scale, view.alpha);
    let in_clip = |px: f32, py: f32| {
        px > mx + 12.0 && px < mx + mw - 16.0 && py > my + 10.0 && py < my + mh - 14.0
    };
    for (i, n) in npcs.iter().enumerate() {
        if town_visible(n) {
            let head = town_head_index(n);
            if head > 0 {
                let flip = n.facing > 0;
                let v = head_anchor(n);
                let px = (v[0] / 16.0 - left) * scale + x2 + xoff;
                let py = (v[1] / 16.0 - top) * scale + y2 - 2.0 * scale / 5.0 + yoff;
                if in_clip(px, py) {
                    if let Some(sheet) = lookups.npc_heads.get(head as usize).copied().flatten() {
                        let (hw, hh) = hover_half(sheet.width, sheet.height, hs);
                        let (hx, hy) = (px - hw, py - hh);
                        if view.mouse.x >= hx
                            && view.mouse.x <= hx + sheet.width as f32 * hs
                            && view.mouse.y >= hy
                            && view.mouse.y <= hy + sheet.height as f32 * hs
                        {
                            text = Some(full_name(n));
                        }
                    }
                    draw_head_friendly(r, lookups, head, [px, py], a, hs, flip, false, view.global_time);
                }
            }
        }
        let boss = boss_head_texture_index(n);
        if n.hp <= 0 || boss == -1 {
            continue;
        }
        let rotation = boss_head_rotation(n);
        let flip = boss_head_flip(n);
        let v = if n.kind == NpcKind::TheDestroyer {
            destroyer_head_center(npcs, i)
        } else {
            head_anchor(n)
        };
        let px = (v[0] / 16.0 - left) * scale + x2 + xoff;
        let py = (v[1] / 16.0 - top) * scale + y2 - 2.0 * scale / 5.0 + yoff;
        if in_clip(px, py) {
            if let Some(sheet) = lookups.npc_boss_heads.get(boss as usize).copied().flatten() {
                let (hw, hh) = hover_half(sheet.width, sheet.height, hs);
                let (hx, hy) = (px - hw, py - hh);
                if view.mouse.x >= hx
                    && view.mouse.x <= hx + sheet.width as f32 * hs
                    && view.mouse.y >= hy
                    && view.mouse.y <= hy + sheet.height as f32 * hs
                {
                    text = Some(given_or_type_name(n));
                }
            }
            draw_head_boss(r, lookups, boss, [px, py], a, hs, rotation, flip);
        }
    }
    text
}

/// `Main.DrawNPCMapIcons3` (`Main.cs:56099-56142`) — the map-style-2
/// overlay's head pass: the fullscreen geometry without any hover text.
/// UNWIRED: the port's `mapStyle == 2` branch draws nothing (the P4 plan's
/// Uncertainty 7, `app/frame/draw.rs`); the fn rides the same helpers so
/// the overlay branch can call it when it lands.
#[allow(dead_code)]
pub(crate) fn draw_npc_map_icons_overlay(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    npcs: &[Npc],
    origin: [f32; 2],
    view: &NpcIconView,
) {
    let (x, y) = (origin[0], origin[1]);
    let (scale, hs, a) = (view.map_scale, view.head_scale, view.alpha);
    for (i, n) in npcs.iter().enumerate() {
        if town_visible(n) {
            let head = town_head_index(n);
            if head > 0 {
                let flip = n.facing > 0;
                let v = head_anchor(n);
                let px = v[0] / 16.0 * scale + x - 10.0 * scale;
                let py = v[1] / 16.0 * scale + y - 10.0 * scale;
                draw_head_friendly(r, lookups, head, [px, py], a, hs, flip, false, view.global_time);
            }
        }
        let boss = boss_head_texture_index(n);
        if n.hp <= 0 || boss == -1 {
            continue;
        }
        let rotation = boss_head_rotation(n);
        let flip = boss_head_flip(n);
        let v = if n.kind == NpcKind::TheDestroyer {
            destroyer_head_center(npcs, i)
        } else {
            head_anchor(n)
        };
        let px = v[0] / 16.0 * scale + x - 10.0 * scale;
        let py = v[1] / 16.0 * scale + y - 10.0 * scale;
        draw_head_boss(r, lookups, boss, [px, py], a, hs, rotation, flip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    fn npc(kind: NpcKind) -> Npc {
        let mut n = Npc::new(kind, 100.0, 200.0, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// W4-e17 — `NPCID.Sets.BossHeadTextures` (`NPCID.cs:10479`): the base
    /// rows the port's kinds resolve to, and the unlisted types' -1.
    #[test]
    fn w4_e17_the_boss_head_table_matches_npcid_10479() {
        for (kind, head) in [
            (NpcKind::EyeOfCthulhu, 0),
            (NpcKind::EaterOfWorldsHead, 2),
            (NpcKind::KingSlime, 7),
            (NpcKind::QueenBee, 14),
            (NpcKind::SkeletronHead, 19),
            (NpcKind::WallOfFlesh, 22),
            (NpcKind::BrainOfCthulhu, 23),
            (NpcKind::TheDestroyer, 25),
            (NpcKind::Retinazer, 15),
            (NpcKind::Spazmatism, 20),
            (NpcKind::SkeletronPrime, 18),
            (NpcKind::Plantera, 11),
            (NpcKind::GolemHead, 5),
            (NpcKind::GolemHeadFree, 5),
            (NpcKind::DukeFishron, 4),
            (NpcKind::MoonLordHead, 8),
            (NpcKind::MartianSaucerCore, 9),
            (NpcKind::AncientCultist, 24),
            (NpcKind::FlyingDutchman, 26),
            (NpcKind::Deerclops, 39),
            (NpcKind::DD2Betsy, 34),
            (NpcKind::DD2EterniaCrystal, 36),
            (NpcKind::DD2LanePortal, 35),
            (NpcKind::StardustTower, 30),
            (NpcKind::NebulaTower, 29),
            (NpcKind::SolarTower, 27),
            (NpcKind::VortexTower, 28),
        ] {
            assert_eq!(boss_head_texture_index(&npc(kind)), head, "{kind:?}");
        }
        // The Golem BODY (245) has no table row — only its heads draw.
        assert_eq!(boss_head_texture_index(&npc(NpcKind::Golem)), -1);
        // A plain mob has no head either.
        assert_eq!(boss_head_texture_index(&npc(NpcKind::Zombie)), -1);
        // The WoF's mirror eye (114) is not in the table.
        assert_eq!(boss_head_texture_index(&npc(NpcKind::WallOfFleshEye)), -1);
    }

    /// W4-e17 — `GetBossHeadTextureIndex`'s seven overrides
    /// (`NPC.cs:7560-7604`).
    #[test]
    fn w4_e17_the_boss_head_overrides_follow_the_state() {
        // The Eye: ai0 >= 2 swaps to head 1.
        let mut eye = npc(NpcKind::EyeOfCthulhu);
        eye.ai[0] = 1.0;
        assert_eq!(boss_head_texture_index(&eye), 0);
        eye.ai[0] = 2.0;
        assert_eq!(boss_head_texture_index(&eye), 1);
        // The Brain: ai0 < 0 OR localAI[2] > 0 hides it.
        let mut brain = npc(NpcKind::BrainOfCthulhu);
        assert_eq!(boss_head_texture_index(&brain), 23);
        brain.ai[0] = -1.0;
        assert_eq!(boss_head_texture_index(&brain), -1);
        brain.ai[0] = 0.0;
        brain.local_ai[2] = 1.0;
        assert_eq!(boss_head_texture_index(&brain), -1);
        // Plantera: INTEGER lifeMax/2 — at exactly half the head swaps.
        let mut plantera = npc(NpcKind::Plantera);
        plantera.max_hp = 30001; // odd: /2 truncates to 15000
        plantera.hp = 15001;
        assert_eq!(boss_head_texture_index(&plantera), 11);
        plantera.hp = 15000;
        assert_eq!(boss_head_texture_index(&plantera), 12);
        // The twins: ai0 >= 3.
        let mut ret = npc(NpcKind::Retinazer);
        ret.ai[0] = 3.0;
        assert_eq!(boss_head_texture_index(&ret), 16);
        let mut spaz = npc(NpcKind::Spazmatism);
        spaz.ai[0] = 2.0;
        assert_eq!(boss_head_texture_index(&spaz), 20);
        spaz.ai[0] = 3.0;
        assert_eq!(boss_head_texture_index(&spaz), 21);
        // The Cultist: ai0 == 5 AND alpha 255 hides it.
        let mut cultist = npc(NpcKind::AncientCultist);
        cultist.ai[0] = 5.0;
        cultist.alpha = 254;
        assert_eq!(boss_head_texture_index(&cultist), 24);
        cultist.alpha = 255;
        assert_eq!(boss_head_texture_index(&cultist), -1);
    }

    /// W4-e17 — `GetBossHeadRotation` (`NPC.cs:7608-7632`) and
    /// `GetBossHeadSpriteEffects` (`:7634-7642`).
    #[test]
    fn w4_e17_the_boss_head_rotation_and_flip_gates() {
        // Plantera always tilts.
        let mut p = npc(NpcKind::Plantera);
        p.rotation = 0.7;
        assert_eq!(boss_head_rotation(&p), 0.7);
        // Skeletron only in the spin states (ai1 1 or 2).
        let mut s = npc(NpcKind::SkeletronHead);
        s.rotation = 0.5;
        s.ai[1] = 0.0;
        assert_eq!(boss_head_rotation(&s), 0.0);
        s.ai[1] = 2.0;
        assert_eq!(boss_head_rotation(&s), 0.5);
        // The Ice Queen only at ai0 == 2.
        let mut iq = npc(NpcKind::IceQueen);
        iq.rotation = 0.3;
        iq.ai[0] = 1.0;
        assert_eq!(boss_head_rotation(&iq), 0.0);
        iq.ai[0] = 2.0;
        assert_eq!(boss_head_rotation(&iq), 0.3);
        // The Eye never tilts its head.
        let mut e = npc(NpcKind::EyeOfCthulhu);
        e.rotation = 1.0;
        assert_eq!(boss_head_rotation(&e), 0.0);
        // The Dutchman flips at spriteDirection == 1 only.
        let mut d = npc(NpcKind::FlyingDutchman);
        d.sprite_dir = -1;
        assert!(!boss_head_flip(&d));
        d.sprite_dir = 1;
        assert!(boss_head_flip(&d));
        assert!(!boss_head_flip(&npc(NpcKind::KingSlime)));
    }

    /// W4-e17 — `TownNPCProfiles.GetHeadIndexSafe`
    /// (`TownNPCProfiles.cs:200-207`): the stacked shimmer rows, the fixed
    /// slime heads and the `TypeToDefaultHeadIndex` fallback.
    #[test]
    fn w4_e17_the_town_head_profiles_resolve() {
        let mut guide = npc(NpcKind::Guide);
        assert_eq!(town_head_index(&guide), 1);
        guide.variation_index = 1;
        assert_eq!(town_head_index(&guide), 72, "the shimmered Guide head");
        guide.variation_index = 2; // out of range falls to profile 0
        assert_eq!(town_head_index(&guide), 1);
        // The Princess's shimmer head is 54 (not 72+).
        let mut princess = npc(NpcKind::Princess);
        assert_eq!(town_head_index(&princess), 45);
        princess.variation_index = 1;
        assert_eq!(town_head_index(&princess), 54);
        // The Travelling Merchant keeps head 21 / shimmer 80.
        let mut tm = npc(NpcKind::TravellingMerchant);
        assert_eq!(town_head_index(&tm), 21);
        tm.variation_index = 1;
        assert_eq!(town_head_index(&tm), 80);
        // A kind with no profile row falls to TypeToDefaultHeadIndex.
        assert_eq!(
            town_head_index(&npc(NpcKind::Merchant)),
            crate::ui::housing::type_to_head_index(17)
        );
    }

    /// W4-e17 — the Destroyer's segment-average centre
    /// (`Main.cs:56125-56137`): head + up to 14 active links, divided by
    /// the count; an inactive link ends the walk.
    #[test]
    fn w4_e17_the_destroyer_head_averages_its_chain() {
        let mut head = npc(NpcKind::TheDestroyer);
        head.worm_id = 7;
        head.seg_index = 0;
        head.x = 0.0;
        head.y = 0.0;
        let hc = head.center();
        let mut npcs = vec![head];
        // Two live links at +100/+200 px on x.
        for (seg, dx) in [(1u32, 100.0f32), (2, 200.0)] {
            let mut link = npc(NpcKind::TheDestroyerBody);
            link.worm_id = 7;
            link.seg_index = seg;
            link.x = dx;
            link.y = 0.0;
            npcs.push(link);
        }
        let lc1 = npcs[1].center();
        let lc2 = npcs[2].center();
        let got = destroyer_head_center(&npcs, 0);
        let want = [
            (hc.0 + lc1.0 + lc2.0) / 3.0,
            (hc.1 + lc1.1 + lc2.1) / 3.0,
        ];
        assert_eq!(got, want);
        // A dead second link stops the walk after the first.
        npcs[2].hp = 0;
        let got = destroyer_head_center(&npcs, 0);
        assert_eq!(got, [(hc.0 + lc1.0) / 2.0, (hc.1 + lc1.1) / 2.0]);
        // A foreign worm's link never joins the average.
        npcs[2].hp = npcs[2].max_hp;
        npcs[2].worm_id = 8;
        let got = destroyer_head_center(&npcs, 0);
        assert_eq!(got, [(hc.0 + lc1.0) / 2.0, (hc.1 + lc1.1) / 2.0]);
        // The 15-cap: 20 live links still average only head + 14.
        let mut many = vec![npcs[0].clone()];
        for seg in 1..=20u32 {
            let mut link = npc(NpcKind::TheDestroyerBody);
            link.worm_id = 7;
            link.seg_index = seg;
            link.x = seg as f32 * 10.0;
            link.y = 0.0;
            many.push(link);
        }
        let got = destroyer_head_center(&many, 0);
        let mut sum = hc.0;
        for seg in 1..=14u32 {
            sum += many[seg as usize].center().0;
        }
        assert_eq!(got[0], sum / 15.0);
    }

    /// W4-e17 — `NPC.FullName`/`GivenOrTypeName` (`NPC.cs:6729-6753`):
    /// the unnamed NPC answers its type name; the named one formats
    /// `Game.NPCTitle` ("{0} the {1}") / answers the given name.
    #[test]
    fn w4_e17_the_head_hover_names_follow_the_given_name() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut guide = npc(NpcKind::Guide);
        assert_eq!(full_name(&guide), "Guide");
        assert_eq!(given_or_type_name(&guide), "Guide");
        guide.given_name = "Andrew".into();
        assert_eq!(full_name(&guide), "Andrew the Guide");
        assert_eq!(given_or_type_name(&guide), "Andrew");
    }

    /// W4-e17 — the hover rect's INTEGER half-width (`Main.cs:56243`:
    /// `(float)(Width() / 2)`) and the head anchor's gfxOffY/netOffset add
    /// (`Main.cs:56236`).
    #[test]
    fn w4_e17_the_hover_half_truncates_and_the_anchor_offsets() {
        // A 25-px-wide sheet halves to 12, not 12.5.
        assert_eq!(hover_half(25, 23, 1.0), [12.0, 11.0]);
        assert_eq!(hover_half(25, 23, 2.0), [24.0, 22.0]);
        let mut n = npc(NpcKind::Guide);
        n.gfx_off_y = 3.0;
        n.net_offset = Vec2::new(2.0, -1.0);
        let (cx, cy) = n.center();
        assert_eq!(head_anchor(&n), [cx + 2.0, cy + 3.0 - 1.0]);
    }

    /// W4-e17 — the minimap clip is the C#'s strict-edged
    /// `(miniMapX+12, miniMapY+10, W-16, H-14)` window
    /// (`Main.cs:56165`/`:56206`): the edges themselves are OUT.
    #[test]
    fn w4_e17_the_minimap_clip_edges_are_strict() {
        let (mx, my, mw, mh) = (100.0f32, 50.0f32, 240.0f32, 240.0f32);
        let in_clip = |px: f32, py: f32| {
            px > mx + 12.0 && px < mx + mw - 16.0 && py > my + 10.0 && py < my + mh - 14.0
        };
        assert!(in_clip(113.0, 61.0));
        assert!(!in_clip(112.0, 61.0), "the left edge is out");
        assert!(!in_clip(324.0, 61.0), "the right edge is out");
        assert!(!in_clip(113.0, 60.0), "the top edge is out");
        assert!(!in_clip(113.0, 276.0), "the bottom edge is out");
        let _ = World::new("t".into(), 64, 64);
    }
}
