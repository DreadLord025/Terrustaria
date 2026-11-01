//! Entity drawing: the player/NPC texture atlases and frame-UV helpers, the
//! world-space player and NPC passes, the held-item swing, the fullscreen-map
//! head icon and the UI-space character preview.
use super::BgTex;
use super::player_layers::front_arm_cell;
use super::sky::sky_tex;
use crate::assets;
use crate::item::SWING_TIME;
use crate::npc::{Npc, NpcKind};
use crate::player::{self, Player};
use crate::registry;
use crate::ui;
use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::lighting::{LightMap, LightStyle};
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
    pub(crate) zombie: Option<BgTex>,
    pub(crate) king_slime: Option<BgTex>,
}

pub(crate) fn make_npc_assets(atlas: &Atlas) -> NpcAssets {
    NpcAssets {
        slime: sky_tex(atlas, "NPC_1", None),
        eye: sky_tex(atlas, "NPC_2", None),
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

#[allow(clippy::too_many_arguments)]
/// Draws the starter player from the original 40x56 part strips. The composition
/// mirrors `PlayerDrawLayers`: legs/pants/shoes, torso/shirt, head/eyes/hair, all at
/// `(x-10, y-10)` so the 40x56 sprite is centred on the 20x42 hitbox.
/// `PlayerDrawLayers` first pass: the selected block in the player's hand, with a
/// `Player.itemAnimation`-style swing when mining, placing or attacking.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_held_item(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    player: &Player,
    kind: u16,
    swing: f32,
    anim_time: f32,
) -> u32 {
    let uv = if let Some((uv, _, _)) = lookups.item_icon(kind) {
        Some(uv)
    } else if registry::is_weapon(kind) {
        lookups.shuriken_uv()
    } else if let Some(wall) = registry::place_wall(kind) {
        lookups.wall_icon_uv(wall)
    } else {
        registry::place_tile(kind).and_then(|t| lookups.plain_tex(t).map(|f| f.uv))
    };
    let Some(uv) = uv else {
        return 0;
    };
    let (cx, cy) = player.center();
    let tx = (cx / TILE_PX) as i32;
    let ty = (cy / TILE_PX) as i32;
    let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
    let dir = player.facing as f32;
    let progress = if swing > 0.0 {
        1.0 - (swing / SWING_TIME).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Arc from raised-behind to forward-down; mirrored for the left hand.
    let a = if progress > 0.0 {
        (-1.7 + progress * 2.5) * dir
    } else {
        0.0
    };
    let hand = [player.x + player::WIDTH * 0.5 + 5.0 * dir, player.y + 27.0];
    let reach = 10.0;
    let center = [hand[0] + a.cos() * reach * dir, hand[1] + a.sin() * reach];
    // `push_sprite_rot` rotates around the top-left, so offset the pivot by the
    // rotated half-size to spin the block around its centre.
    let half = 7.0f32;
    let (s, co) = a.sin_cos();
    let top_left = [
        center[0] - (half * co - half * s),
        center[1] - (half * s + half * co),
    ];
    r.push_sprite_rot(
        top_left,
        [half * 2.0, half * 2.0],
        uv,
        [c[0], c[1], c[2], 1.0],
        a,
    );
    1
}

pub(crate) fn draw_player(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    player: &Player,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let (cx, cy) = player.center();
    let tile_x = (cx / TILE_PX) as i32;
    let tile_y = (cy / TILE_PX) as i32;
    let l = light.get(tile_x, tile_y);
    let c = style.shade(l, tile_x, tile_y, anim_time);
    let light_c = [c[0], c[1], c[2]];
    let mul = |part: [f32; 3]| {
        [
            light_c[0] * part[0],
            light_c[1] * part[1],
            light_c[2] * part[2],
            1.0,
        ]
    };
    let skin = [1.0, 125.0 / 255.0, 90.0 / 255.0];
    let eye_white = [1.0, 1.0, 1.0];
    let eye = [105.0 / 255.0, 90.0 / 255.0, 75.0 / 255.0];
    let shirt = [175.0 / 255.0, 165.0 / 255.0, 140.0 / 255.0];
    let pants = [255.0 / 255.0, 230.0 / 255.0, 175.0 / 255.0];
    let shoes = [160.0 / 255.0, 105.0 / 255.0, 60.0 / 255.0];
    let hair = [215.0 / 255.0, 90.0 / 255.0, 55.0 / 255.0];

    let pos = snap_to_screen([player.x - 10.0, player.y - 10.0], camera, zoom, screen);
    let size = [40.0, 56.0];
    let flip = player.facing < 0;
    let leg = player.leg_frame;
    let body = player.body_frame;
    let mut quads = 0;
    let (arm_col, arm_row) = front_arm_cell(body);
    let frame_uv = |t: Option<&BgTex>, row: u32| t.map(|t| flip_uv(player_frame_uv(t, row), flip));
    let cell_uv = |t: Option<&BgTex>, col: u32, row: u32| {
        t.map(|t| flip_uv(player_cell_uv(t, col, row), flip))
    };
    let mut part = |tex: Option<&BgTex>, uv: Option<[f32; 4]>, color: [f32; 4]| {
        if tex.is_some() {
            if let Some(uv) = uv {
                r.push_sprite(pos, size, uv, color);
                quads += 1;
            }
        }
    };
    // The layer sequence of `LegacyPlayerRenderer.DrawPlayerInternal` for the
    // parts the starter player has (no armor/accessories yet). Each step cites
    // its `PlayerDrawLayers` method.
    //
    // `DrawPlayer_12_Skin_Composite` (PlayerDrawLayers.cs:1211): torso skin
    // (`compTorsoFrame`, always cell (0,0)) and leg skin (`legFrame`).
    // `PlayerFrame`: the legs animate on `legFrame`, the body/arms/head on
    // `bodyFrame` (which copies the leg row while walking).
    part(
        assets.torso.as_ref(),
        cell_uv(assets.torso.as_ref(), 0, 0),
        mul(skin),
    );
    part(
        assets.legs.as_ref(),
        frame_uv(assets.legs.as_ref(), leg),
        mul(skin),
    );
    // `DrawPlayer_12_SkinComposite_BackArmShirt` (PlayerDrawLayers.cs:1260):
    // the back arm stacks the skin (`Players[7]`), the under-skin arm layer
    // (`Players[5]`, body-skin colour) and the shirt sleeve (`Players[13]`,
    // shirt colour). The starter player has no undershirt colour, so the
    // `Players[8]` pass stays out until the equipment palette lands (M3-EQUIP).
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, (arm_row + 2).min(3)),
        mul(skin),
    );
    part(
        assets.undershirt.as_ref(),
        cell_uv(assets.undershirt.as_ref(), arm_col, (arm_row + 2).min(3)),
        mul(skin),
    );
    part(
        assets.arm_shirt.as_ref(),
        cell_uv(assets.arm_shirt.as_ref(), arm_col, (arm_row + 2).min(3)),
        mul(shirt),
    );
    // `DrawPlayer_13_Leggings` (1412) and `DrawPlayer_14_Shoes` (1709) overlay
    // the leg frame; `DrawPlayer_17_TorsoComposite` (1937) draws the shirt over
    // the skin torso.
    part(
        assets.pants.as_ref(),
        frame_uv(assets.pants.as_ref(), leg),
        mul(pants),
    );
    part(
        assets.shoes.as_ref(),
        frame_uv(assets.shoes.as_ref(), leg),
        mul(shoes),
    );
    part(
        assets.shirt.as_ref(),
        cell_uv(assets.shirt.as_ref(), 0, 0),
        mul(shirt),
    );
    // `DrawPlayer_21_Head` (2036): head, eye whites, eyes and hair.
    part(
        assets.head.as_ref(),
        frame_uv(assets.head.as_ref(), body),
        mul(skin),
    );
    part(
        assets.eye_whites.as_ref(),
        frame_uv(assets.eye_whites.as_ref(), body),
        mul(eye_white),
    );
    part(
        assets.eyes.as_ref(),
        frame_uv(assets.eyes.as_ref(), body),
        mul(eye),
    );
    part(
        assets.hair.as_ref(),
        frame_uv(assets.hair.as_ref(), body),
        mul(hair),
    );
    // The composite front arm belongs to the held-item layers
    // (`DrawPlayer_27_HeldItem`/`28_ArmOverItem`, with the shoulder rotation);
    // the cell approximation stays until M2-DRAW-2 ports the rotation.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), 8, arm_row),
        mul(skin),
    );
    quads
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
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), 8, arm_row),
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
    let blue = [0.52, 0.72, 1.0];
    let mut quads = 0;
    for n in npcs {
        let (cx, cy) = n.center();
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let (w, _) = n.kind.size();
        let (tex, frame_h, size, pos) = match n.kind {
            NpcKind::Slime => (
                assets.slime.as_ref(),
                26.0,
                [32.0, 26.0],
                // The 26 px frame has 2 transparent rows below the feet; align the
                // art bottom (row 23) with the hitbox bottom.
                [n.x - 1.0, n.y + 1.0],
            ),
            NpcKind::DemonEye => (
                assets.eye.as_ref(),
                24.0,
                [38.0, 24.0],
                [n.x - 4.0, n.y + 2.0],
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
        let base = if n.kind == NpcKind::Slime {
            [c[0] * blue[0], c[1] * blue[1], c[2] * blue[2], 1.0]
        } else {
            [c[0], c[1], c[2], 1.0]
        };
        let tint = if n.hurt_cd > 0.0 {
            [c[0].max(0.8), c[1].max(0.8), c[2].max(0.8), 1.0]
        } else {
            base
        };
        let uv = flip_uv(npc_frame_uv(tex, frame_h, n.frame()), n.facing < 0);
        r.push_sprite(snap_to_screen(pos, camera, zoom, screen), size, uv, tint);
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
