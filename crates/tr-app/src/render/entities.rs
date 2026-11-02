//! Entity drawing: the player/NPC texture atlases and frame-UV helpers, the
//! world-space player and NPC passes, the held-item swing, the fullscreen-map
//! head icon and the UI-space character preview.
use super::sky::sky_tex;
use super::BgTex;
use crate::assets;
use crate::item::SWING_TIME;
use crate::npc::{Npc, NpcKind};
use crate::player::{self, Player};
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

/// UV of one 40x56 frame in a player part strip.
fn player_frame_uv(tex: &BgTex, row: u32) -> [f32; 4] {
    let rows = (tex.h / 56.0).floor().max(1.0);
    let r = (row as f32).min(rows - 1.0);
    let dv = (tex.uv[3] - tex.uv[1]) * (56.0 / tex.h);
    [
        tex.uv[0],
        tex.uv[1] + dv * r,
        tex.uv[2],
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// UV of a 40x56 composite cell in a 9x4 player part sheet.
fn player_cell_uv(tex: &BgTex, col: u32, row: u32) -> [f32; 4] {
    let cols = (tex.w / 40.0).floor().max(1.0);
    let rows = (tex.h / 56.0).floor().max(1.0);
    let du = (tex.uv[2] - tex.uv[0]) / cols;
    let dv = (tex.uv[3] - tex.uv[1]) / rows;
    let c = (col as f32).min(cols - 1.0);
    let r = (row as f32).min(rows - 1.0);
    [
        tex.uv[0] + du * c,
        tex.uv[1] + dv * r,
        tex.uv[0] + du * (c + 1.0),
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// `PlayerDrawSet.CreateCompositeData`: body frame index -> front-arm cell.
fn arm_cell(frame: u32) -> (u32, u32) {
    match frame {
        0 => (2, 0),
        1 => (3, 0),
        2 => (4, 0),
        3 => (5, 0),
        4 => (6, 0),
        5 => (2, 1),
        6 => (3, 1),
        7..=10 => (4, 1),
        11..=13 => (3, 1),
        14 => (5, 1),
        15..=16 => (6, 1),
        17 => (5, 1),
        18..=19 => (3, 1),
        _ => (2, 0),
    }
}

pub(crate) fn flip_uv(uv: [f32; 4], flip: bool) -> [f32; 4] {
    if flip {
        [uv[2], uv[1], uv[0], uv[3]]
    } else {
        uv
    }
}

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

/// `Player.ItemCheck` useStyle-1 swing rotation (`Player.cs:50848`):
/// `itemRotation = (itemAnimation / itemAnimationMax - 0.5) * (-direction) * 3.5
/// - direction * 0.3`. `itemAnimation` counts down from `itemAnimationMax`, so
/// `remaining` is 1 at the swing start (arm raised behind, `-2.05·dir`) and 0 at
/// the end (forward-down, `+1.45·dir`) — a 3.5 rad sweep.
fn item_rotation(remaining: f32, direction: f32) -> f32 {
    (remaining - 0.5) * -direction * 3.5 - direction * 0.3
}

/// `Player.ItemCheck` useStyle-1 `itemLocation` thirds (`Player.cs:50709-50846`):
/// the anchor the held item pivots around, per animation third, offset from
/// `position + (width/2, y)`. The `num` tables below are the C# cascades over
/// the held item's frame size; `zero` is the `UsesBetterMeleeItemLocation`
/// nudge (`Player.cs:50844`) which no ported item belongs to.
#[allow(clippy::too_many_arguments)]
fn item_location(
    remaining: f32,
    direction: f32,
    pos_x: f32,
    pos_y: f32,
    frame_w: f32,
    frame_h: f32,
) -> [f32; 2] {
    let half = frame_w * 0.5;
    let (num_x, loc_y) = if remaining >= 0.666 {
        // First third (`Player.cs:50709-50734`): `num` 10 / >32:14 / >=52:24 /
        // >=64:28 / >=92:38, `itemLocation.Y = position.Y + 24`.
        let mut num = if frame_w > 32.0 { 14.0 } else { 10.0 };
        if frame_w >= 52.0 {
            num = 24.0;
        }
        if frame_w >= 64.0 {
            num = 28.0;
        }
        if frame_w >= 92.0 {
            num = 38.0;
        }
        (num, 24.0)
    } else if remaining >= 0.333 {
        // Middle third (`:50736-50775`): X `10 / >32:18 / >=52:24 / >=64:28 /
        // >=92:38`, Y `10 / >32:8 / >52:12 / >64:14`.
        let mut num = if frame_w > 32.0 { 18.0 } else { 10.0 };
        if frame_w >= 52.0 {
            num = 24.0;
        }
        if frame_w >= 64.0 {
            num = 28.0;
        }
        if frame_w >= 92.0 {
            num = 38.0;
        }
        let num_y = if frame_h > 64.0 {
            14.0
        } else if frame_h > 52.0 {
            12.0
        } else if frame_h > 32.0 {
            8.0
        } else {
            10.0
        };
        (num, num_y)
    } else {
        // Last third (`:50776-50843`): X `6 / >32:14 / >=48:18 / >=52:24 /
        // >=64:28 / >=92:38` and it *subtracts* `direction`, Y `10 / >52:12 /
        // >64:14`.
        let mut num = if frame_w > 32.0 { 14.0 } else { 6.0 };
        if frame_w >= 48.0 {
            num = 18.0;
        }
        if frame_w >= 52.0 {
            num = 24.0;
        }
        if frame_w >= 64.0 {
            num = 28.0;
        }
        if frame_w >= 92.0 {
            num = 38.0;
        }
        let num_y = if frame_h > 64.0 {
            14.0
        } else if frame_h > 52.0 {
            12.0
        } else {
            10.0
        };
        (num, num_y)
    };
    let x = if remaining < 0.333 {
        pos_x - (half - num_x) * direction
    } else {
        pos_x + (half - num_x) * direction
    };
    [x, pos_y + loc_y]
}

/// `PlayerDrawLayers.DrawPlayer_27_HeldItem` (`PlayerDrawLayers.cs:3043-3215`)
/// for useStyle 1: the used item pivots on `itemLocation` (plus `gfxOffY`,
/// `PlayerDrawSet.cs:368`) with rotation `itemRotation`. The C# origin
/// `(W/2 - W/2·direction, H)` plus the `direction < 0` horizontal flip places
/// the pivot on the drawn sprite's bottom-left corner in both directions, so
/// the quad's top-left is `anchor + R(θ)·(0, -H)`.
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
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let (uv, frame_w, frame_h) = if let Some((uv, w, h)) = lookups.item_icon(kind) {
        (uv, w, h)
    } else if registry::is_weapon(kind) {
        match lookups.shuriken_uv() {
            Some(uv) => (uv, 16.0, 16.0),
            None => return 0,
        }
    } else if let Some(wall) = registry::place_wall(kind) {
        match lookups.wall_icon_uv(wall) {
            Some(uv) => (uv, 16.0, 16.0),
            None => return 0,
        }
    } else {
        match registry::place_tile(kind).and_then(|t| lookups.plain_tex(t)) {
            Some(f) => (f.uv, 16.0, 16.0),
            None => return 0,
        }
    };
    let (cx, cy) = player.center();
    let tx = (cx / TILE_PX) as i32;
    let ty = (cy / TILE_PX) as i32;
    let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
    let dir = player.facing as f32;
    // `itemAnimation / itemAnimationMax` with the countdown running: `swing`
    // holds the remaining seconds of the use. `SWING_TIME` stands in for the
    // per-item `useAnimation` (catalogued deviation; real per-item timings are
    // the M2-ANIM-item-timings catalog row).
    let remaining = (swing / SWING_TIME).clamp(0.0, 1.0);
    let rotation = item_rotation(remaining, dir);
    let (loc_x, loc_y) = {
        let l = item_location(
            remaining,
            dir,
            player.x + player::WIDTH * 0.5,
            player.y,
            frame_w,
            frame_h,
        );
        (l[0], l[1])
    };
    // `drawinfo.ItemLocation = Position + (itemLocation - position)`
    // (`PlayerDrawSet.cs:368`) and `Position.Y` carries `gfxOffY`
    // (`LegacyPlayerRenderer.cs:457`).
    let anchor = snap_to_screen([loc_x, loc_y + player.gfx_off_y], camera, zoom, screen);
    let (s, co) = rotation.sin_cos();
    let top_left = [anchor[0] + frame_h * s, anchor[1] - frame_h * co];
    let uv = flip_uv(uv, dir < 0.0);
    r.push_sprite_rot(
        top_left,
        [frame_w, frame_h],
        uv,
        [c[0], c[1], c[2], 1.0],
        rotation,
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

    // `LegacyPlayerRenderer.DrawPlayer` is called with
    // `position + netOffset + (0, gfxOffY)` (`LegacyPlayerRenderer.cs:457,468`),
    // and the layer math (`PlayerDrawLayers.cs:1150-1153`: `- bodyFrame.Width/2 +
    // width/2`, `+ height - bodyFrame.Height + 4`, origin `+ (W/2, H/2)`)
    // resolves to the 40x56 frame's top-left at `(x - 10, y - 10 + gfxOffY)`.
    // Without the gfx term the sprite pops vertically on steps/slopes.
    let pos = snap_to_screen(
        [player.x - 10.0, player.y - 10.0 + player.gfx_off_y],
        camera,
        zoom,
        screen,
    );
    let size = [40.0, 56.0];
    let flip = player.facing < 0;
    let leg = player.leg_frame;
    let body = player.body_frame;
    let mut quads = 0;
    let (arm_col, arm_row) = arm_cell(body);
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
    // (`DrawPlayer_27_HeldItem`/`28_ArmOverItem`). useStyle 1 never enables the
    // composite-use columns, so the front arm always rides the bodyFrame row
    // through the same `CreateCompositeData` table as the back arm
    // (`PlayerDrawSet.cs:1962-2023`); column 8 is the composite *back*-arm use
    // column and showed a mismatched pose while walking/using items.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, arm_row),
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
    let (arm_col, arm_row) = arm_cell(body);
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
        if n.kind == NpcKind::DemonEye && n.rotation != 0.0 {
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
