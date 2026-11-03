//! Entity drawing: the player/NPC texture atlases and frame-UV helpers, the
//! world-space player and NPC passes, the held-item swing, the fullscreen-map
//! head icon and the UI-space character preview.
use super::player_layers::{
    assemble_layers, front_arm_cell, swing_stand_in, CompositeArmState, LayerSprite,
    PlayerCompositeState,
};
use super::sky::sky_tex;
use super::BgTex;
use crate::assets;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::registry;
use crate::ui;
use glam::Vec2;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::TILE_PX;
use tr_render::{Atlas, SpriteRenderer};

/// Default-player texture parts (`PlayerTextureID`): head, eyes, torso skin, shirt,
/// legs, pants, shoes, hair. Frames are 40x56 on the 40-px-wide strips.
#[derive(Default)]
pub(crate) struct PlayerAssets {
    pub(crate) head: Option<BgTex>,
    pub(crate) eye_whites: Option<BgTex>,
    pub(crate) eyes: Option<BgTex>,
    pub(crate) torso: Option<BgTex>,
    pub(crate) shirt: Option<BgTex>,
    /// Back-arm skin frame from `Player_0_7` (composite arm frame 1).
    pub(crate) arm: Option<BgTex>,
    /// `Player_0_5`: the under-skin torso layer drawn behind the back arm
    /// (`TextureAssets.Players[skinVar, 5]`, `PlayerDrawLayers.cs:1345`).
    pub(crate) undershirt: Option<BgTex>,
    /// `Player_0_13`: the shirt sleeve over the back arm
    /// (`TextureAssets.Players[skinVar, 13]`, `PlayerDrawLayers.cs:1362`).
    pub(crate) arm_shirt: Option<BgTex>,
    pub(crate) legs: Option<BgTex>,
    pub(crate) pants: Option<BgTex>,
    pub(crate) shoes: Option<BgTex>,
    pub(crate) hair: Option<BgTex>,
}

pub(crate) fn make_player_assets(atlas: &Atlas) -> PlayerAssets {
    PlayerAssets {
        head: sky_tex(atlas, "Player_0_0", None),
        eye_whites: sky_tex(atlas, "Player_0_1", None),
        eyes: sky_tex(atlas, "Player_0_2", None),
        // Torso and shirt sheets are 9 columns wide; keep only the first column.
        torso: sky_tex(atlas, "Player_0_3", Some((0, 0, 40, 224))),
        shirt: sky_tex(atlas, "Player_0_6", Some((0, 0, 40, 224))),
        arm: sky_tex(atlas, "Player_0_7", None),
        undershirt: sky_tex(atlas, "Player_0_5", None),
        arm_shirt: sky_tex(atlas, "Player_0_13", None),
        legs: sky_tex(atlas, "Player_0_10", None),
        pants: sky_tex(atlas, "Player_0_11", None),
        shoes: sky_tex(atlas, "Player_0_12", None),
        hair: sky_tex(atlas, "Player_Hair_1", None),
    }
}

/// UV of one 40x56 frame in a player part strip — tabled in
/// [`player_layers`] (single source; re-exported for the sibling passes).
pub(crate) use super::player_layers::{flip_uv, player_cell_uv, player_frame_uv};

/// Default-derivable: every sheet is an `Option` guard (the headless flow
/// tests construct the empty set).
#[derive(Default)]
pub(crate) struct NpcAssets {
    pub(crate) slime: Option<BgTex>,
    pub(crate) eye: Option<BgTex>,
    pub(crate) wandering_eye: Option<BgTex>,
    pub(crate) zombie: Option<BgTex>,
    pub(crate) king_slime: Option<BgTex>,
}

pub(crate) fn make_npc_assets(atlas: &Atlas) -> NpcAssets {
    NpcAssets {
        slime: sky_tex(atlas, "NPC_1", None),
        eye: sky_tex(atlas, "NPC_2", None),
        wandering_eye: sky_tex(atlas, "NPC_133", None),
        zombie: sky_tex(atlas, "NPC_3", None),
        king_slime: sky_tex(atlas, "NPC_50", None),
    }
}

/// UV of one slime / demon-eye frame (two frames stacked vertically).
fn npc_frame_uv(tex: &BgTex, frame_h: f32, frame: u32) -> [f32; 4] {
    let dv = (tex.uv[3] - tex.uv[1]) * (frame_h / tex.h);
    let r = frame as f32;
    [
        tex.uv[0],
        tex.uv[1] + dv * r,
        tex.uv[2],
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// Snap a sprite's top-left to an integer screen pixel so nearest sampling keeps
/// the same source texels while the camera moves (no crawling pixel grid).
#[inline]
fn snap_to_screen(pos: [f32; 2], camera: Vec2, zoom: f32, screen: Vec2) -> [f32; 2] {
    let sx = (pos[0] - camera.x) * zoom + screen.x * 0.5;
    let sy = (pos[1] - camera.y) * zoom + screen.y * 0.5;
    [
        camera.x + (sx.round() - screen.x * 0.5) / zoom,
        camera.y + (sy.round() - screen.y * 0.5) / zoom,
    ]
}

/// The held item's atlas quad and frame size — the `GetItemDrawFrame` stand-in:
/// the hotbar icon quad and its atlas dimensions while `Item.cs` textures stay
/// unframed (`ItemSlot.DrawItemIcon`'s clamp applies to the HUD slot, not the
/// world sprite).
fn held_item_quad(lookups: &assets::Lookups, kind: u16) -> Option<([f32; 4], [f32; 2])> {
    if let Some((uv, w, h)) = lookups.item_icon(kind) {
        return Some((uv, [w, h]));
    }
    if registry::is_weapon(kind) {
        return lookups.shuriken_uv().map(|uv| (uv, [16.0, 16.0]));
    }
    if let Some(wall) = registry::place_wall(kind) {
        return lookups.wall_icon_uv(wall).map(|uv| (uv, [16.0, 16.0]));
    }
    registry::place_tile(kind)
        .and_then(|t| lookups.plain_tex(t))
        .map(|f| (f.uv, [16.0, 16.0]))
}

/// Pushes a builder's sprite list. Unrotated sprites snap their top-left to an
/// integer screen pixel (the entity-snapping invariant); the rotated held item
/// keeps its sub-pixel pivot like the old arc pass. Returns the quad count.
pub(crate) fn push_layers(
    r: &mut SpriteRenderer,
    layers: &[LayerSprite],
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let mut quads = 0;
    for s in layers {
        if s.rotation != 0.0 {
            r.push_sprite_rot(s.pos, s.size, s.uv, s.color, s.rotation);
        } else {
            r.push_sprite(
                snap_to_screen(s.pos, camera, zoom, screen),
                s.size,
                s.uv,
                s.color,
            );
        }
        quads += 1;
    }
    quads
}

/// The starter player through the pure layer builders (`assemble_layers`):
/// `LegacyPlayerRenderer`'s layer order with the held item in its
/// `weaponDrawOrder` slot (`PlayerDrawSet.cs:358`). `held` carries the selected
/// item kind and the swing timer (`None` = empty hand — remote players).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_player(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    lookups: &assets::Lookups,
    player: &Player,
    held: Option<(u16, f32)>,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let (cx, cy) = player.center();
    let (tx, ty) = ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
    let light_c = style.shade(light.get(tx, ty), tx, ty, anim_time);
    let swing = held.map_or(0.0, |(_, s)| s);
    let state = PlayerCompositeState {
        origin: [player.x - 10.0, player.y - 10.0],
        leg_frame: player.leg_frame,
        body_frame: player.body_frame,
        facing: player.facing as f32,
        // `LegacyPlayerRenderer` receives the player at
        // `position + (0, gfxOffY)` (`LegacyPlayerRenderer.cs:457,468`).
        gfx_off_y: player.gfx_off_y,
        swing,
        hold_style: held.map_or(0, |(kind, _)| crate::item::hold_style(kind)),
        use_style: 1,
        item_uv: held
            .and_then(|(kind, _)| held_item_quad(lookups, kind))
            .map(|(uv, _)| uv),
        item_size: held
            .and_then(|(kind, _)| held_item_quad(lookups, kind))
            .map(|(_, size)| size)
            .unwrap_or([0.0, 0.0]),
        light: light_c,
        front_arm: swing_stand_in(1, swing, player.facing as f32),
        back_arm: CompositeArmState::DISABLED,
    };
    push_layers(r, &assemble_layers(&state, assets), camera, zoom, screen)
}

/// `Main.PlayerRenderer.DrawPlayer` in UI space (`UICharacter.cs:118`): the
/// existing frame-UV machinery of [`draw_player`] at a fixed screen rect,
/// full-white light (the preview is unlit), the draft's colours, `scale` =
/// `UICharacter._characterScale` (1.5 creation / 1.0 card), `frame_row` =
/// `UICharacter`'s animation row (0 static, `ui::char_creation::
/// animated_frame_row(time)` animated, `:85-95`). `pos` is
/// `GetPlayerPosition`'s hitbox anchor (`UICharacter.cs:122-130` — the port's
/// `preview_anchor`); the 40x56 art box rides the hitbox at the same
/// `(-10, -10)` offset the world draw uses, scaled. The preview faces right —
/// `PreparePreview_Main`'s `direction = 1` (`UICharacterCreation.cs:616`).
///
/// The port decodes only the base body sheets (no per-variant sets), so every
/// `skinVariant`/`hair` id renders the base male frame — the documented
/// preview deviation that mirrors the char-select card's.
pub(crate) fn draw_player_preview(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    pos: [f32; 2],
    scale: f32,
    frame_row: u32,
    colors: &ui::char_creation::PreviewColors,
) -> u32 {
    let byte = |c: u8| c as f32 / 255.0;
    let skin = [
        byte(colors.skin[0]),
        byte(colors.skin[1]),
        byte(colors.skin[2]),
        1.0,
    ];
    let eye_white = [1.0, 1.0, 1.0, 1.0];
    let eye = [
        byte(colors.eye[0]),
        byte(colors.eye[1]),
        byte(colors.eye[2]),
        1.0,
    ];
    let shirt = [
        byte(colors.shirt[0]),
        byte(colors.shirt[1]),
        byte(colors.shirt[2]),
        1.0,
    ];
    let undershirt = [
        byte(colors.undershirt[0]),
        byte(colors.undershirt[1]),
        byte(colors.undershirt[2]),
        1.0,
    ];
    let pants = [
        byte(colors.pants[0]),
        byte(colors.pants[1]),
        byte(colors.pants[2]),
        1.0,
    ];
    let shoes = [
        byte(colors.shoe[0]),
        byte(colors.shoe[1]),
        byte(colors.shoe[2]),
        1.0,
    ];
    let hair = [
        byte(colors.hair[0]),
        byte(colors.hair[1]),
        byte(colors.hair[2]),
        1.0,
    ];

    let pos = [pos[0] - 10.0 * scale, pos[1] - 10.0 * scale];
    let size = [40.0 * scale, 56.0 * scale];
    let body = frame_row;
    let mut quads = 0;
    let (arm_col, arm_row) = front_arm_cell(body);
    let frame_uv = |t: Option<&BgTex>, row: u32| t.map(|t| player_frame_uv(t, row));
    let cell_uv = |t: Option<&BgTex>, col: u32, row: u32| t.map(|t| player_cell_uv(t, col, row));
    let mut part = |tex: Option<&BgTex>, uv: Option<[f32; 4]>, color: [f32; 4]| {
        if tex.is_some() {
            if let Some(uv) = uv {
                r.push_ui_sprite(pos, size, uv, color);
                quads += 1;
            }
        }
    };
    // `PlayerFrame`: the legs animate on `legFrame`, the body/arms/head on
    // `bodyFrame` (`UICharacter.UpdateAnim` writes all three, `:92-93`).
    part(
        assets.legs.as_ref(),
        frame_uv(assets.legs.as_ref(), body),
        skin,
    );
    part(
        assets.pants.as_ref(),
        frame_uv(assets.pants.as_ref(), body),
        pants,
    );
    part(
        assets.shoes.as_ref(),
        frame_uv(assets.shoes.as_ref(), body),
        shoes,
    );
    // Back arm behind the torso, front arm in front of the shirt (composite
    // layers), the under-shirt between the torso and the shirt.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, (arm_row + 2).min(3)),
        skin,
    );
    part(
        assets.torso.as_ref(),
        cell_uv(assets.torso.as_ref(), 0, 0),
        skin,
    );
    part(
        assets.torso.as_ref(),
        cell_uv(assets.torso.as_ref(), 0, 0),
        undershirt,
    );
    part(
        assets.shirt.as_ref(),
        cell_uv(assets.shirt.as_ref(), 0, 0),
        shirt,
    );
    // Same `CreateCompositeData` table for the preview's front arm — column 8
    // is the composite back-arm use column, not a walking pose.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, arm_row),
        skin,
    );
    part(
        assets.head.as_ref(),
        frame_uv(assets.head.as_ref(), body),
        skin,
    );
    part(
        assets.eye_whites.as_ref(),
        frame_uv(assets.eye_whites.as_ref(), body),
        eye_white,
    );
    part(
        assets.eyes.as_ref(),
        frame_uv(assets.eyes.as_ref(), body),
        eye,
    );
    part(
        assets.hair.as_ref(),
        frame_uv(assets.hair.as_ref(), body),
        hair,
    );
    quads
}

/// `Main.MapPlayerRenderer.DrawPlayerHead` (`Main.cs:55954`): the local player's
/// 84x84 head icon for the fullscreen map.
///
/// This engine has no render targets, so the `PlayerHeadDrawRenderTargetContent`
/// composite (skin head + eye whites + eyes + hair) and the
/// `AnOutlinedDrawRenderTargetContent` black/border rings are reproduced by pushing
/// the same quads straight into the screen-space `ui` range (drawn after
/// `map`/`map_back`, matching vanilla's frame → tiles → head order).
///
/// `pos` is the icon **centre** (`head_icon_pos`). The target is blitted centred
/// with origin `(42, 42)` (`MapHeadRenderer.cs:50`), and the head frame sits at
/// target-local `(26, 28)` (`PlayerDrawHeadSet.cs:150-155` + `147`), so its screen
/// top-left is `pos + ([-16, -14]) * scale` and its size is `[40, 56] * scale`.
/// `scale` is `Main.UIScale` — this engine has no UI scale, so it is passed `1.0`.
/// `border` is `Main.GetPlayerHeadBordersColor` (White in singleplayer).
pub(crate) fn draw_player_map_icon(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    player: &Player,
    pos: [f32; 2],
    scale: f32,
    border: [f32; 4],
) -> u32 {
    // `PlayerDrawHeadSet.cs:129-133` (`BoringSetup`): the raw skin/eye/hair colours,
    // alpha 1. The icon is unlit — `MapHeadRenderer.cs:50` blits the target with
    // `Color.White`, so there is no tile-light multiply.
    let skin = [1.0, 125.0 / 255.0, 90.0 / 255.0, 1.0];
    let eye_white = [1.0, 1.0, 1.0, 1.0];
    let eye = [105.0 / 255.0, 90.0 / 255.0, 75.0 / 255.0, 1.0];
    let hair = [215.0 / 255.0, 90.0 / 255.0, 55.0 / 255.0, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];
    // `PlayerDrawHeadSet.cs:142-146`: `direction < 0` flips horizontally.
    let flip = player.facing < 0;
    // `PlayerDrawHeadSet.cs:148-149`: the neutral frame row (`bodyFrameMemory` with
    // `Y = 0`). `HairFrame` shortens this by 1px (`:82-90`), ignored.
    let frame = 0u32;
    let parts: [(Option<&BgTex>, [f32; 4]); 4] = [
        (assets.head.as_ref(), skin),
        (assets.eye_whites.as_ref(), eye_white),
        (assets.eyes.as_ref(), eye),
        (assets.hair.as_ref(), hair),
    ];
    let size = [40.0 * scale, 56.0 * scale];
    let base = [pos[0] - 16.0 * scale, pos[1] - 14.0 * scale];
    let mut quads = 0;
    // One `Draw(helperTarget, offset, tint)` — all four parts of the composite, so
    // the union of their alphas is the silhouette the outline rings are built from.
    let mut copy = |offset: [f32; 2], tint: Option<[f32; 4]>| {
        let at = [base[0] + offset[0] * scale, base[1] + offset[1] * scale];
        for &(tex, color) in parts.iter() {
            if let Some(tex) = tex {
                let uv = flip_uv(player_frame_uv(tex, frame), flip);
                r.push_ui_sprite(at, size, uv, tint.unwrap_or(color));
                quads += 1;
            }
        }
    };
    // `AnOutlinedDrawRenderTargetContent.cs:38-49`: 8 black ring copies at Manhattan
    // radius 4 (`num = 2`, `num2 = num * 2 = 4`).
    for offset in [
        [-4.0, 0.0],
        [4.0, 0.0],
        [0.0, -4.0],
        [0.0, 4.0],
        [-2.0, -2.0],
        [-2.0, 2.0],
        [2.0, -2.0],
        [2.0, 2.0],
    ] {
        copy(offset, Some(black));
    }
    // `AnOutlinedDrawRenderTargetContent.cs:50-60`: 4 border copies at Manhattan
    // radius 2 (`num2 = num = 2`).
    for offset in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
        copy(offset, Some(border));
    }
    // `AnOutlinedDrawRenderTargetContent.cs:62`: the head composite itself.
    copy([0.0, 0.0], None);
    quads
}

/// First enemy pass: the blue slime, tinted and lit like the player, with an overhead
/// health bar once damaged.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_npcs(
    r: &mut SpriteRenderer,
    assets: &NpcAssets,
    npcs: &[Npc],
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let white = r.white_uv();
    let mut quads = 0;
    for n in npcs {
        let (cx, cy) = n.center();
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let (w, _) = n.size();
        let (tex, frame_h, size, pos) = match n.kind {
            NpcKind::Slime => (
                assets.slime.as_ref(),
                26.0,
                // The 32x26 frame scales with the variant
                // (`SetDefaults_ForNetId`'s size scale) and is anchored at the
                // frame centre over the hitbox centre (`Main.DrawNPC`'s
                // `origin = frame.Size() / 2`).
                [32.0 * n.slime.stats().scale, 26.0 * n.slime.stats().scale],
                [cx - 16.0 * n.slime.stats().scale, cy - 13.0 * n.slime.stats().scale],
            ),
            NpcKind::DemonEye => (
                assets.eye.as_ref(),
                24.0,
                [38.0, 24.0],
                [n.x - 4.0, n.y + 2.0],
            ),
            NpcKind::WanderingEye => (
                assets.wandering_eye.as_ref(),
                // `Main.npcFrameCount[133] = 4` (Main.cs:67082).
                assets.wandering_eye.as_ref().map_or(24.0, |t| t.h / 4.0),
                assets
                    .wandering_eye
                    .as_ref()
                    .map_or([38.0, 24.0], |t| [38.0, t.h / 4.0]),
                [cx - 19.0, cy - 12.0],
            ),
            NpcKind::Zombie => (
                assets.zombie.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            NpcKind::KingSlime => (
                assets.king_slime.as_ref(),
                120.0,
                [174.0, 120.0],
                [n.x - 38.0, n.y - 25.0],
            ),
        };
        let Some(tex) = tex else {
            continue;
        };
        // `Main.DrawNPC` multiplies the light by `npc.color`; the slime
        // variants carry their `SetDefaults` colours (alpha included — the
        // premultiplied pipeline renders the translucency directly).
        let base = if n.kind == NpcKind::Slime {
            let k = n.slime.stats().color;
            [
                c[0] * k[0] as f32 / 255.0,
                c[1] * k[1] as f32 / 255.0,
                c[2] * k[2] as f32 / 255.0,
                k[3] as f32 / 255.0,
            ]
        } else {
            [c[0], c[1], c[2], 1.0]
        };
        let tint = if n.hurt_cd > 0.0 {
            [c[0].max(0.8), c[1].max(0.8), c[2].max(0.8), 1.0]
        } else {
            base
        };
        let uv = flip_uv(npc_frame_uv(tex, frame_h, n.frame()), n.facing < 0);
        if matches!(n.kind, NpcKind::DemonEye | NpcKind::WanderingEye) && n.rotation != 0.0
            {
            // `Main.DrawNPC` draws every NPC at its centre with
            // `origin = frame size / 2` and `rotation = npc.rotation`
            // (`NPC.cs` draw pass); the demon eye aims along its flight
            // (`NPC.cs:74019-74031`). The mirrored quad's corners are
            // symmetric about the centre, so flipping does not move them —
            // rotate the quad about its centre, exactly like XNA does.
            let center = snap_to_screen(
                [pos[0] + size[0] * 0.5, pos[1] + size[1] * 0.5],
                camera,
                zoom,
                screen,
            );
            let (hw, hh) = (size[0] * 0.5, size[1] * 0.5);
            let (s, co) = n.rotation.sin_cos();
            let top_left = [center[0] - hw * co + hh * s, center[1] - hw * s - hh * co];
            r.push_sprite_rot(top_left, size, uv, tint, n.rotation);
        } else {
            r.push_sprite(snap_to_screen(pos, camera, zoom, screen), size, uv, tint);
        }
        quads += 1;
        if n.hp < n.max_hp {
            let frac = (n.hp as f32 / n.max_hp as f32).clamp(0.0, 1.0);
            r.push_sprite([n.x, n.y - 9.0], [w, 4.0], white, [0.02, 0.02, 0.02, 0.75]);
            r.push_sprite(
                [n.x, n.y - 9.0],
                [w * frac, 4.0],
                white,
                [0.85, 0.15, 0.15, 0.95],
            );
            quads += 2;
        }
    }
    quads
}
