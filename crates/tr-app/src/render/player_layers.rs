//! Pure composite tables for the player draw: the frame cells, the shoulder
//! order, the head-gear bob and the weapon slot of `PlayerDrawSet` /
//! `PlayerDrawLayers`. No render types — the builders turn these tables
//! into `LayerSprite` lists and the render passes only push them.

use super::BgTex;
use super::entities::PlayerAssets;

/// The starter palette of `Player_0_*` (`colorBodySkin`/`colorShirt`/... from
/// the default skin; the M3-EQUIP equipment palette will replace these).
pub(crate) const SKIN: [f32; 3] = [1.0, 125.0 / 255.0, 90.0 / 255.0];
pub(crate) const SHIRT: [f32; 3] = [175.0 / 255.0, 165.0 / 255.0, 140.0 / 255.0];
pub(crate) const PANTS: [f32; 3] = [1.0, 230.0 / 255.0, 175.0 / 255.0];
pub(crate) const SHOES: [f32; 3] = [160.0 / 255.0, 105.0 / 255.0, 60.0 / 255.0];
pub(crate) const HAIR: [f32; 3] = [215.0 / 255.0, 90.0 / 255.0, 55.0 / 255.0];
pub(crate) const EYE_WHITE: [f32; 3] = [1.0, 1.0, 1.0];
pub(crate) const EYES: [f32; 3] = [105.0 / 255.0, 90.0 / 255.0, 75.0 / 255.0];

/// Which player part a sprite belongs to — the test seam for layer order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerPart {
    TorsoSkin,
    LegsSkin,
    BackArmSkin,
    BackUnder,
    BackSleeve,
    Pants,
    Shoes,
    Shirt,
    Head,
    EyeWhites,
    Eyes,
    Hair,
    FrontArmSkin,
    FrontSleeve,
    FrontShirt,
    FrontShoulderSkin,
    FrontShoulderSleeve,
    FrontShoulderShirt,
    HeldItem,
}

/// One drawable sprite produced by a layer builder: position and size in world
/// pixels, an atlas UV quad, a pre-multiplied tint and a top-left rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerSprite {
    pub(crate) part: PlayerPart,
    pub(crate) pos: [f32; 2],
    pub(crate) size: [f32; 2],
    pub(crate) uv: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) rotation: f32,
}

/// `Player.CompositeArmStretchAmount` (`Player.cs`, рядом с
/// `CompositeArmData:219`): how far the composite arm stretches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompositeStretch {
    Full,
    ThreeQuarters,
    Quarter,
    None,
}

impl CompositeStretch {
    /// The sheet row of the stretch (`PlayerDrawSet.cs:2254-2268`):
    /// Full→0, ThreeQuarters→1, Quarter→2, None→3.
    fn row(self) -> u32 {
        match self {
            CompositeStretch::Full => 0,
            CompositeStretch::ThreeQuarters => 1,
            CompositeStretch::Quarter => 2,
            CompositeStretch::None => 3,
        }
    }
}

/// `Player.CompositeArmData` (`Player.cs:219`): the per-arm composite state —
/// `enabled` switches the arm onto the composite sheet columns
/// (front 7 / back 8) at the stretch row, with `rotation` applied around the
/// composite pivot; disabled arms keep the per-body-frame cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompositeArmState {
    pub(crate) enabled: bool,
    pub(crate) stretch: CompositeStretch,
    pub(crate) rotation: f32,
}

impl CompositeArmState {
    /// The disabled arm — per-body-frame cells, no rotation.
    pub(crate) const DISABLED: CompositeArmState = CompositeArmState {
        enabled: false,
        stretch: CompositeStretch::None,
        rotation: 0.0,
    };
}

/// `PlayerDrawSet.UpdateCompositeArm` (`PlayerDrawSet.cs:2246-2271`): an
/// enabled arm frames from the composite column — front `targetX` 7, back 8
/// (`:2035-2036`) — at the stretch row, rotating by `data.rotation`; a
/// disabled arm keeps the body-frame cell (`front_arm_cell` / `back_arm_cell`,
/// the `frameIndex.Y = frameIndex2.Y + 2` of `:2034`) and does not rotate.
pub(crate) fn update_composite_arm(
    data: CompositeArmState,
    frame: u32,
    is_back: bool,
) -> ((u32, u32), f32) {
    match data.enabled {
        true => {
            let column = if is_back { 8 } else { 7 };
            ((column, data.stretch.row()), data.rotation)
        }
        false => {
            let cell = if is_back {
                back_arm_cell(frame)
            } else {
                front_arm_cell(frame)
            };
            (cell, 0.0)
        }
    }
}

/// `PlayerDrawLayers.GetCompositeOffset_BackArm` (`PlayerDrawLayers.cs:4082-4085`):
/// (+6, +2) before the direction mirror.
pub(crate) const COMPOSITE_PIVOT_BACK: [f32; 2] = [6.0, 2.0];

/// `PlayerDrawLayers.GetCompositeOffset_FrontArm` (`:4087-4090`): (−5, 0)
/// before the mirror.
pub(crate) const COMPOSITE_PIVOT_FRONT: [f32; 2] = [-5.0, 0.0];

/// The pivot offset with `GetCompositeOffset_*`'s direction mirror applied
/// (X flips with `direction`, Y with `gravDir` — gravDir is always 1 here).
pub(crate) fn composite_pivot(is_back: bool, dir: f32) -> [f32; 2] {
    let raw = if is_back {
        COMPOSITE_PIVOT_BACK
    } else {
        COMPOSITE_PIVOT_FRONT
    };
    [raw[0] * dir, raw[1]]
}

/// The per-frame inputs every layer builder reads (parameter object — the
/// cc-routine-and-class-design 7±2 cap).
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerCompositeState {
    /// Top-left of the 40x56 sprite box (`player.x - 10, player.y - 10`).
    pub(crate) origin: [f32; 2],
    pub(crate) leg_frame: u32,
    pub(crate) body_frame: u32,
    pub(crate) facing: f32,
    /// `Player.itemAnimation` stand-in: the swing timer, 0 = idle.
    pub(crate) swing: f32,
    /// Vanilla `useStyle` stand-in: 1 = swing (default), 8 = thrust, 9 = bow.
    pub(crate) use_style: u8,
    /// The held item's atlas quad and frame size, `None` = empty hand.
    pub(crate) item_uv: Option<[f32; 4]>,
    pub(crate) item_size: [f32; 2],
    /// The light tint at the player tile (`style.shade(...)` output).
    pub(crate) light: [f32; 3],
    /// `Player.compositeFrontArm` (`Player.cs:1906`).
    pub(crate) front_arm: CompositeArmState,
    /// `Player.compositeBackArm`.
    pub(crate) back_arm: CompositeArmState,
}

/// `WeaponDrawOrder` (`PlayerDrawSet.cs:358`): which of the three
/// `DrawPlayer_27_HeldItem` call slots the held item occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WeaponDrawOrder {
    BehindBackArm,
    BehindFrontArm,
    OverFrontArm,
}

/// The front-arm composite cell per body frame — the `frameIndex2` switch of
/// `PlayerDrawSet.CreateCompositeData` (`PlayerDrawSet.cs:1952-2023`), with the
/// out-of-range clamp falling back to the standing frame like the original's
/// `bodyFrame` layout.
pub(crate) fn front_arm_cell(frame: u32) -> (u32, u32) {
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

/// The back arm reuses the front cell two rows down
/// (`frameIndex.Y = frameIndex2.Y + 2`, `PlayerDrawSet.cs:2034`).
pub(crate) fn back_arm_cell(frame: u32) -> (u32, u32) {
    let (col, row) = front_arm_cell(frame);
    (col, row + 2)
}

/// Front shoulder cell — `pt2 = (0, 1)` (`PlayerDrawSet.cs:1939`).
pub(crate) const FRONT_SHOULDER_CELL: (u32, u32) = (0, 1);

/// Back shoulder cell — `pt = (1, 1)` (`PlayerDrawSet.cs:1938`).
pub(crate) const BACK_SHOULDER_CELL: (u32, u32) = (1, 1);

/// Torso skin cell — `pt3 = default` with the case-5 override to column 1
/// (`PlayerDrawSet.cs:1940,1975`).
pub(crate) fn torso_cell(frame: u32) -> (u32, u32) {
    if frame == 5 {
        (1, 0)
    } else {
        (0, 0)
    }
}

/// `compShoulderOverFrontArm`: when true the shoulder stack draws after (over)
/// the arm stack. Default true; rows 1, 2 and 5 clear it
/// (`PlayerDrawSet.cs:1956,1961,1966,1971,1976`).
pub(crate) fn shoulder_over_front_arm(frame: u32) -> bool {
    !matches!(frame, 1 | 2 | 5)
}

/// The raw `OffsetsPlayerHeadgear` table row — Y is 0 exactly at rows 7-9 and
/// 14-16, 2 elsewhere (`Main.cs:66638-66662`).
pub(crate) fn headgear_offset(frame: u32) -> f32 {
    match frame {
        7..=9 | 14..=16 => 0.0,
        _ => 2.0,
    }
}

/// The shift the composite consumers apply: `vector2.Y -= 2; vector += vector2
/// * -1` for the un-flipped case (`PlayerDrawLayers.cs:1217-1219`) — so the
/// torso and both arm stacks sit 2 px lower on the zero rows, legs never shift.
pub(crate) fn headgear_composite_shift(frame: u32) -> f32 {
    2.0 - headgear_offset(frame)
}

/// The held-item slot for the starter player: `weaponDrawOrder` defaults to
/// `BehindFrontArm` (`PlayerDrawSet.cs:358`); the 4952 pet and golf-club
/// overrides (`359-365`) are out of scope.
pub(crate) fn weapon_draw_order() -> WeaponDrawOrder {
    WeaponDrawOrder::BehindFrontArm
}

/// UV of one 40x56 frame in a player part strip.
pub(crate) fn player_frame_uv(tex: &BgTex, row: u32) -> [f32; 4] {
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
pub(crate) fn player_cell_uv(tex: &BgTex, col: u32, row: u32) -> [f32; 4] {
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

/// Mirrors a UV quad horizontally (`SpriteEffects.FlipHorizontally`).
pub(crate) fn flip_uv(uv: [f32; 4], flip: bool) -> [f32; 4] {
    if flip {
        [uv[2], uv[1], uv[0], uv[3]]
    } else {
        uv
    }
}

fn tint(c: [f32; 3], part: [f32; 3]) -> [f32; 4] {
    [c[0] * part[0], c[1] * part[1], c[2] * part[2], 1.0]
}

/// The sprite box: every player part is the 40x56 sheet frame anchored at
/// `(x - 10, y - 10)` so it straddles the 20x42 hitbox like the original.
/// One body part, skipped when its sheet is missing (`Main.spriteBatch.Draw`
/// ignores null textures — DW-2.5's dirty case).
/// The sprite top-left for one part: the 40x56 box at `origin + shift`, or —
/// when a composite pivot offset and rotation are given — rotated around
/// `center + offset` (XNA rotates around the draw origin; at rotation 0 this
/// collapses to the plain top-left).
fn part_pos(s: &PlayerCompositeState, shift: f32, pivot_offset: [f32; 2], rotation: f32) -> [f32; 2] {
    let base_pos = [s.origin[0], s.origin[1] + shift];
    let origin_c = [20.0 + pivot_offset[0], 28.0 + pivot_offset[1]];
    let pivot_pos = [base_pos[0] + origin_c[0], base_pos[1] + origin_c[1]];
    let (sin, cos) = rotation.sin_cos();
    [
        pivot_pos[0] - (origin_c[0] * cos - origin_c[1] * sin),
        pivot_pos[1] - (origin_c[0] * sin + origin_c[1] * cos),
    ]
}

fn push_part(
    out: &mut Vec<LayerSprite>,
    part: PlayerPart,
    tex: Option<BgTex>,
    uv: impl Fn(&BgTex) -> [f32; 4],
    pos: [f32; 2],
    rotation: f32,
    color: [f32; 4],
) {
    if let Some(tex) = tex {
        out.push(LayerSprite {
            part,
            pos,
            size: [40.0, 56.0],
            uv: uv(&tex),
            color,
            rotation,
        });
    }
}

/// `DrawPlayer_12_Skin_Composite` → `13_Leggings` → `14_Shoes` →
/// `17_TorsoComposite` → `21_Head` (`LegacyPlayerRenderer.cs:195-215`): the
/// body parts in original order. The composite torso/arm sprites take the
/// head-gear shift (`PlayerDrawLayers.cs:1217-1219`); legs and head do not.
pub(crate) fn body_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    let mut out = Vec::with_capacity(12);
    let flip = s.facing < 0.0;
    let shift = headgear_composite_shift(s.body_frame);
    // 12_Skin_Composite: torso skin, leg skin, back-arm stack.
    push_part(
        &mut out,
        PlayerPart::TorsoSkin,
        a.torso,
        |t| flip_uv(player_cell_uv(t, torso_cell(s.body_frame).0, torso_cell(s.body_frame).1), flip),
        part_pos(s, shift, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SKIN),
    );
    push_part(
        &mut out,
        PlayerPart::LegsSkin,
        a.legs,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SKIN),
    );
    // `UpdateCompositeArm` (`PlayerDrawSet.cs:2246`): the back arm switches to
    // the composite column (8, stretch row) and rotation when enabled.
    let (back_cell, back_rotation) = update_composite_arm(s.back_arm, s.body_frame, true);
    let back_pos = part_pos(
        s,
        shift,
        composite_pivot(true, s.facing),
        back_rotation,
    );
    push_part(
        &mut out,
        PlayerPart::BackArmSkin,
        a.arm,
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SKIN),
    );
    // `DrawPlayer_12_SkinComposite_BackArmShirt`: under-skin arm layer (5) and
    // shirt sleeve (13); the undershirt-tinted `Players[8]` pass is skipped —
    // the starter skin has no underShirt colour (M3-EQUIP palette).
    push_part(
        &mut out,
        PlayerPart::BackUnder,
        a.undershirt,
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SKIN),
    );
    push_part(
        &mut out,
        PlayerPart::BackSleeve,
        a.arm_shirt,
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SHIRT),
    );
    // 13_Leggings / 14_Shoes ride the leg frame without the shift.
    push_part(
        &mut out,
        PlayerPart::Pants,
        a.pants,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, PANTS),
    );
    push_part(
        &mut out,
        PlayerPart::Shoes,
        a.shoes,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SHOES),
    );
    // 17_TorsoComposite: the shirt over the (already drawn) skin torso.
    push_part(
        &mut out,
        PlayerPart::Shirt,
        a.shirt,
        |t| flip_uv(player_cell_uv(t, torso_cell(s.body_frame).0, torso_cell(s.body_frame).1), flip),
        part_pos(s, shift, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SHIRT),
    );
    // 21_Head: head, eye whites, eyes, hair.
    push_part(
        &mut out,
        PlayerPart::Head,
        a.head,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SKIN),
    );
    push_part(
        &mut out,
        PlayerPart::EyeWhites,
        a.eye_whites,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, EYE_WHITE),
    );
    push_part(
        &mut out,
        PlayerPart::Eyes,
        a.eyes,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, EYES),
    );
    push_part(
        &mut out,
        PlayerPart::Hair,
        a.hair,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, HAIR),
    );
    out
}

/// One arm composite pass: skin(7) + sleeve(13) + shirt(6) at one cell
/// (`PlayerDrawLayers.cs:3661-3706`, `Players[8]` skipped — no starter colour).
/// The stand-in for `ItemCheck_ApplyUseStyle`'s composite-arm rule
/// (`Player.cs:50679`, M2-ANIM's territory): while the swing timer runs, the
/// front arm is enabled — `Full` stretch mid-swing, `ThreeQuarters` at the
/// edges — rotating along the swing arc. Idle returns the disabled arm.
pub(crate) fn swing_stand_in(swing: f32, facing: f32) -> CompositeArmState {
    if swing <= 0.0 {
        return CompositeArmState::DISABLED;
    }
    let progress = 1.0 - (swing / crate::item::SWING_TIME).clamp(0.0, 1.0);
    let stretch = if (0.25..=0.75).contains(&progress) {
        CompositeStretch::Full
    } else {
        CompositeStretch::ThreeQuarters
    };
    CompositeArmState {
        enabled: true,
        stretch,
        rotation: (-1.7 + progress * 2.5) * facing,
    }
}

/// One arm composite pass: skin(7) + sleeve(13) + shirt(6) at one cell
/// (`PlayerDrawLayers.cs:3661-3706`, `Players[8]` skipped — no starter colour).
/// `cell`/`rotation` come from `update_composite_arm`; a rotation pivots the
/// sprite around `center + composite_pivot` (XNA rotates around the draw
/// origin — `pos = pivot − R·(size/2 + offset)`; at rotation 0 this collapses
/// to the plain top-left, so disabled arms render exactly as before).
fn arm_stack(
    s: &PlayerCompositeState,
    a: &PlayerAssets,
    base: PlayerPart,
    cell: (u32, u32),
    rotation: f32,
    pivot_offset: [f32; 2],
) -> Vec<LayerSprite> {
    let flip = s.facing < 0.0;
    let shift = headgear_composite_shift(s.body_frame);
    let shoulder = base == PlayerPart::FrontShoulderSkin;
    let mut out = Vec::with_capacity(3);
    for (part, tex) in [
        (base, a.arm),
        (
            if shoulder {
                PlayerPart::FrontShoulderSleeve
            } else {
                PlayerPart::FrontSleeve
            },
            a.arm_shirt,
        ),
        (
            if shoulder {
                PlayerPart::FrontShoulderShirt
            } else {
                PlayerPart::FrontShirt
            },
            a.shirt,
        ),
    ] {
        let Some(tex) = tex else { continue };
        let color = if part == base {
            tint(s.light, SKIN)
        } else {
            tint(s.light, SHIRT)
        };
        out.push(LayerSprite {
            part,
            pos: part_pos(s, shift, pivot_offset, rotation),
            size: [40.0, 56.0],
            uv: flip_uv(player_cell_uv(&tex, cell.0, cell.1), flip),
            color,
            rotation,
        });
    }
    out
}

/// The front-arm layer of `DrawPlayer_28_ArmOverItemComposite`'s no-armor
/// branch (`PlayerDrawLayers.cs:3661-3706`): the arm and shoulder stacks, arm
/// first when `compShoulderOverFrontArm` (default true), shoulder first
/// otherwise.
pub(crate) fn front_arm_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    // `28_ArmOverItemComposite`: the arm stack rotates by
    // `compositeFrontArm.rotation` around the front pivot; the shoulder stack
    // keeps the plain body rotation.
    let (cell, rotation) = update_composite_arm(s.front_arm, s.body_frame, false);
    let arm = arm_stack(
        s,
        a,
        PlayerPart::FrontArmSkin,
        cell,
        rotation,
        composite_pivot(false, s.facing),
    );
    let shoulder = arm_stack(
        s,
        a,
        PlayerPart::FrontShoulderSkin,
        FRONT_SHOULDER_CELL,
        0.0,
        composite_pivot(false, s.facing),
    );
    let mut out = Vec::with_capacity(6);
    if shoulder_over_front_arm(s.body_frame) {
        out.extend(arm);
        out.extend(shoulder);
    } else {
        out.extend(shoulder);
        out.extend(arm);
    }
    out
}

/// `DrawPlayer_27_HeldItem` (`PlayerDrawLayers.cs:3043-3195`): the held item
/// rotated around the C# grip origin — bottom corner mirrored by direction
/// (default, `:3161`), `origin.Y = 2` with a quarter-turn for useStyle 8
/// (`:3181-3188`), `(0.5, 0.4)` of the frame for useStyle 9 (`:3162-3174`).
/// The rotation itself comes from the swing arc (the M2-ANIM boundary); the
/// empty hand returns no sprite.
pub(crate) fn held_item_layers(s: &PlayerCompositeState) -> Vec<LayerSprite> {
    // Guards of the original (`:3077-3092`), flattened to guard clauses.
    let Some(uv) = s.item_uv else {
        return Vec::new();
    };
    if s.swing <= 0.0 || s.item_size[0] <= 0.0 {
        return Vec::new();
    }
    let dir = if s.facing < 0.0 { -1.0 } else { 1.0 };
    let (w, h) = (s.item_size[0], s.item_size[1]);
    // `:3161`: `(width*0.5 - width*0.5*direction, height)`.
    let mut origin = [w * 0.5 - w * 0.5 * dir, h];
    let progress = 1.0 - (s.swing / crate::item::SWING_TIME).clamp(0.0, 1.0);
    let mut rotation = (-1.7 + progress * 2.5) * dir;
    match s.use_style {
        // `:3181-3188`: thrust — quarter turn, near-top grip.
        8 => {
            rotation -= std::f32::consts::FRAC_PI_2 * dir;
            origin[1] = 2.0;
            origin[0] += 2.0 * dir;
        }
        // `:3162-3174`: bow — centre-left hold.
        9 => origin = [w * 0.5, h * 0.4],
        _ => {}
    }
    // The pivot is the grip point: `ItemLocation` stands at the hand, so the
    // sprite's top-left is `pivot - R * origin`.
    let hand = [s.origin[0] + 10.0 + 5.0 * dir, s.origin[1] + 37.0];
    let (sin, cos) = rotation.sin_cos();
    let top_left = [
        hand[0] - (origin[0] * cos - origin[1] * sin),
        hand[1] - (origin[0] * sin + origin[1] * cos),
    ];
    vec![LayerSprite {
        part: PlayerPart::HeldItem,
        pos: top_left,
        size: [w, h],
        uv,
        color: [s.light[0], s.light[1], s.light[2], 1.0],
        rotation,
    }]
}

/// The full player composition: body, then the held item in its
/// `weaponDrawOrder` slot, then the front arm — the
/// `LegacyPlayerRenderer.cs:185-255` sequence for the starter player.
pub(crate) fn assemble_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    let body = body_layers(s, a);
    let arms = front_arm_layers(s, a);
    let item = held_item_layers(s);
    let mut out = Vec::with_capacity(body.len() + arms.len() + item.len());
    match weapon_draw_order() {
        WeaponDrawOrder::BehindBackArm => {
            out.extend(item.iter().copied());
            out.extend(body);
        }
        WeaponDrawOrder::BehindFrontArm => {
            out.extend(body);
            out.extend(item.iter().copied());
        }
        WeaponDrawOrder::OverFrontArm => {}
    }
    out.extend(arms);
    if weapon_draw_order() == WeaponDrawOrder::OverFrontArm {
        out.extend(item.iter().copied());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dw_1_2_front_arm_cell_matches_c_switch() {
        // `PlayerDrawSet.cs:1952-2023`, one case per row.
        let expected: [(u32, u32); 20] = [
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (2, 1),
            (3, 1),
            (4, 1),
            (4, 1),
            (4, 1),
            (4, 1),
            (3, 1),
            (3, 1),
            (3, 1),
            (5, 1),
            (6, 1),
            (6, 1),
            (5, 1),
            (3, 1),
            (3, 1),
        ];
        for (frame, cell) in expected.iter().enumerate() {
            assert_eq!(front_arm_cell(frame as u32), *cell, "frame {frame}");
        }
        // Out-of-range clamp: standing frame.
        assert_eq!(front_arm_cell(20), (2, 0));
        // The back arm is the same cell two rows down (`:2034`).
        assert_eq!(back_arm_cell(0), (2, 2));
        assert_eq!(back_arm_cell(7), (4, 3));
    }

    #[test]
    fn test_dw_1_3_headgear_raw_table_and_composite_shift() {
        // Raw `OffsetsPlayerHeadgear` (`Main.cs:66638-66662`): zero rows.
        for frame in [7u32, 8, 9, 14, 15, 16] {
            assert_eq!(headgear_offset(frame), 0.0, "raw frame {frame}");
        }
        // Every other row is 2.
        for frame in [0u32, 6, 10, 13, 17, 19] {
            assert_eq!(headgear_offset(frame), 2.0, "raw frame {frame}");
        }
        // Composite consumers shift by `2 − y` (`PlayerDrawLayers.cs:1217-1219`).
        assert_eq!(headgear_composite_shift(7), 2.0);
        assert_eq!(headgear_composite_shift(0), 0.0);
    }

    #[test]
    fn test_dw_1_4_shoulder_order_and_weapon_slot() {
        // Rows 1, 2, 5 clear the flag; 3, 4 set it; the rest default true.
        assert!(!shoulder_over_front_arm(1));
        assert!(!shoulder_over_front_arm(2));
        assert!(!shoulder_over_front_arm(5));
        assert!(shoulder_over_front_arm(3));
        assert!(shoulder_over_front_arm(4));
        assert!(shoulder_over_front_arm(0));
        assert!(shoulder_over_front_arm(19));
        // The starter's weapon slot is the default (`PlayerDrawSet.cs:358`).
        assert_eq!(weapon_draw_order(), WeaponDrawOrder::BehindFrontArm);
    }

    #[test]
    fn torso_cell_overrides_frame_5() {
        assert_eq!(torso_cell(0), (0, 0));
        assert_eq!(torso_cell(5), (1, 0));
        assert_eq!(torso_cell(19), (0, 0));
        assert_eq!(FRONT_SHOULDER_CELL, (0, 1));
        assert_eq!(BACK_SHOULDER_CELL, (1, 1));
    }

    #[test]
    fn test_dw_1_1_enabled_cells() {
        // An enabled arm frames from the composite column — front 7, back 8 —
        // at the stretch row (`PlayerDrawSet.cs:2254-2268`, targetX `:2035-2036`).
        let data = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Full,
            rotation: 0.5,
        };
        assert_eq!(update_composite_arm(data, 0, false), ((7, 0), 0.5));
        assert_eq!(update_composite_arm(data, 0, true), ((8, 0), 0.5));
        let stretches = [
            (CompositeStretch::Full, 0),
            (CompositeStretch::ThreeQuarters, 1),
            (CompositeStretch::Quarter, 2),
            (CompositeStretch::None, 3),
        ];
        for (stretch, row) in stretches {
            let d = CompositeArmState {
                enabled: true,
                stretch,
                rotation: 0.0,
            };
            assert_eq!(update_composite_arm(d, 3, false).0, (7, row));
        }
    }

    #[test]
    fn test_dw_1_2_disabled_frame_cells() {
        // Disabled arms keep the body-frame cells and do not rotate — the
        // M2-DRAW-1 behaviour (`PlayerDrawSet.cs:2269-2270`, `:2034`).
        let disabled = CompositeArmState::DISABLED;
        assert_eq!(
            update_composite_arm(disabled, 0, false),
            (front_arm_cell(0), 0.0)
        );
        assert_eq!(
            update_composite_arm(disabled, 7, true),
            (back_arm_cell(7), 0.0)
        );
        assert_eq!(front_arm_cell(0), (2, 0));
        assert_eq!(back_arm_cell(7), (4, 3));
    }

    #[test]
    fn test_dw_1_3_pivots_and_mirror() {
        // Raw constants (`PlayerDrawLayers.cs:4082-4090`).
        assert_eq!(COMPOSITE_PIVOT_BACK, [6.0, 2.0]);
        assert_eq!(COMPOSITE_PIVOT_FRONT, [-5.0, 0.0]);
        // `GetCompositeOffset_*` mirror X by direction, Y stays.
        assert_eq!(composite_pivot(true, 1.0), [6.0, 2.0]);
        assert_eq!(composite_pivot(true, -1.0), [-6.0, 2.0]);
        assert_eq!(composite_pivot(false, 1.0), [-5.0, 0.0]);
        assert_eq!(composite_pivot(false, -1.0), [5.0, 0.0]);
    }

    #[test]
    fn test_dw_2_1_enabled_arm_cells_and_rotation() {
        let a = fixture();
        let mut s = state();
        s.front_arm = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Full,
            rotation: 0.75,
        };
        s.back_arm = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Quarter,
            rotation: -0.4,
        };
        // Front arm: composite column 7 of the 9-column sheet.
        let layers = front_arm_layers(&s, &a);
        let arm = layers
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        assert!((arm.rotation - 0.75).abs() < 1e-6);
        assert!((arm.uv[0] - 7.0 / 9.0).abs() < 1e-4, "{:?}", arm.uv);
        assert!((arm.uv[2] - 8.0 / 9.0).abs() < 1e-4);
        // Back arm: composite column 8.
        let body = body_layers(&s, &a);
        let back = body
            .iter()
            .find(|l| l.part == PlayerPart::BackArmSkin)
            .unwrap();
        assert!((back.rotation - -0.4).abs() < 1e-6);
        assert!((back.uv[0] - 8.0 / 9.0).abs() < 1e-4, "{:?}", back.uv);
    }

    #[test]
    fn test_dw_2_2_pivot_math() {
        let a = fixture();
        let mut s = state();
        s.front_arm = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Full,
            rotation: 0.75,
        };
        let layers = front_arm_layers(&s, &a);
        let arm = layers
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        // pivot = center + front offset = (100+20-5, 200+28+0);
        // origin_c = (20-5, 28); pos = pivot - R*origin_c.
        let (sin, cos) = 0.75f32.sin_cos();
        let expected = [
            115.0 - (15.0 * cos - 28.0 * sin),
            228.0 - (15.0 * sin + 28.0 * cos),
        ];
        assert!((arm.pos[0] - expected[0]).abs() < 1e-4, "{:?}", arm.pos);
        assert!((arm.pos[1] - expected[1]).abs() < 1e-4);
        // rotation = 0 collapses to the plain top-left.
        s.front_arm.rotation = 0.0;
        let layers0 = front_arm_layers(&s, &a);
        let arm0 = layers0
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        assert_eq!(arm0.pos, [100.0, 200.0]);
    }

    #[test]
    fn test_dw_2_3_idle_parity() {
        let a = fixture();
        let s = state();
        // Disabled arms: every sprite sits at the plain top-left with no
        // rotation — the M2-DRAW-1 geometry, untouched.
        for l in body_layers(&s, &a)
            .iter()
            .chain(front_arm_layers(&s, &a).iter())
        {
            assert_eq!(l.rotation, 0.0, "{:?}", l.part);
            assert_eq!(l.pos, [100.0, 200.0], "{:?}", l.part);
        }
    }

    #[test]
    fn test_dw_2_4_swing_stand_in() {
        // Mid-swing: Full stretch; the edges: ThreeQuarters; idle: disabled.
        let mid = swing_stand_in(crate::item::SWING_TIME * 0.5, 1.0);
        assert!(mid.enabled);
        assert_eq!(mid.stretch, CompositeStretch::Full);
        assert!((mid.rotation - (-1.7 + 0.5 * 2.5)).abs() < 1e-5);
        let edge = swing_stand_in(crate::item::SWING_TIME, 1.0);
        assert_eq!(edge.stretch, CompositeStretch::ThreeQuarters);
        assert_eq!(swing_stand_in(0.0, 1.0), CompositeArmState::DISABLED);
    }

    #[test]
    fn test_dw_2_5_enabled_dirty_inputs() {
        let a = fixture();
        // Enabled arm with missing textures: nothing is pushed.
        let mut s = state();
        s.front_arm = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Full,
            rotation: 0.75,
        };
        let empty = PlayerAssets::default();
        assert!(front_arm_layers(&s, &empty).is_empty());
        // facing -1 mirrors the pivot: pivot x = origin + 20 + 5.
        let mut sm = state();
        sm.facing = -1.0;
        sm.front_arm = s.front_arm;
        let layersm = front_arm_layers(&sm, &a);
        let arm = layersm
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        let (sin, cos) = 0.75f32.sin_cos();
        let expected = [
            125.0 - (25.0 * cos - 28.0 * sin),
            228.0 - (25.0 * sin + 28.0 * cos),
        ];
        assert!((arm.pos[0] - expected[0]).abs() < 1e-4, "{:?}", arm.pos);
        assert!((arm.pos[1] - expected[1]).abs() < 1e-4);
    }

    fn sheet() -> BgTex {
        BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 360.0,
            h: 224.0,
        }
    }

    fn fixture() -> PlayerAssets {
        PlayerAssets {
            head: Some(sheet()),
            eye_whites: Some(sheet()),
            eyes: Some(sheet()),
            torso: Some(sheet()),
            shirt: Some(sheet()),
            arm: Some(sheet()),
            undershirt: Some(sheet()),
            arm_shirt: Some(sheet()),
            legs: Some(sheet()),
            pants: Some(sheet()),
            shoes: Some(sheet()),
            hair: Some(sheet()),
        }
    }

    fn state() -> PlayerCompositeState {
        PlayerCompositeState {
            origin: [100.0, 200.0],
            leg_frame: 0,
            body_frame: 0,
            facing: 1.0,
            swing: 0.0,
            use_style: 1,
            item_uv: None,
            item_size: [16.0, 16.0],
            light: [1.0, 1.0, 1.0],
            front_arm: CompositeArmState::DISABLED,
            back_arm: CompositeArmState::DISABLED,
        }
    }

    fn parts(sprites: &[LayerSprite]) -> Vec<PlayerPart> {
        sprites.iter().map(|s| s.part).collect()
    }

    #[test]
    fn test_dw_2_1_builders_are_render_free() {
        // The builders are exercised here directly: no `SpriteRenderer` in the
        // signature, no GPU state — the proof is the call itself.
        let s = state();
        let a = fixture();
        assert!(!body_layers(&s, &a).is_empty());
        assert!(!front_arm_layers(&s, &a).is_empty());
        assert!(assemble_layers(&s, &a).len() >= 18);
    }

    #[test]
    fn test_dw_2_2_body_layer_order() {
        let s = state();
        let a = fixture();
        assert_eq!(
            parts(&body_layers(&s, &a)),
            vec![
                PlayerPart::TorsoSkin,
                PlayerPart::LegsSkin,
                PlayerPart::BackArmSkin,
                PlayerPart::BackUnder,
                PlayerPart::BackSleeve,
                PlayerPart::Pants,
                PlayerPart::Shoes,
                PlayerPart::Shirt,
                PlayerPart::Head,
                PlayerPart::EyeWhites,
                PlayerPart::Eyes,
                PlayerPart::Hair,
            ]
        );
    }

    #[test]
    fn test_dw_2_3_arm_stacks() {
        let a = fixture();
        // Frame 0 keeps the default: arm stack, then shoulder over it.
        let s0 = state();
        assert_eq!(
            parts(&front_arm_layers(&s0, &a)),
            vec![
                PlayerPart::FrontArmSkin,
                PlayerPart::FrontSleeve,
                PlayerPart::FrontShirt,
                PlayerPart::FrontShoulderSkin,
                PlayerPart::FrontShoulderSleeve,
                PlayerPart::FrontShoulderShirt,
            ]
        );
        // Frames 1/2/5 clear `compShoulderOverFrontArm`: shoulder first.
        let s1 = PlayerCompositeState {
            body_frame: 1,
            ..state()
        };
        assert_eq!(
            parts(&front_arm_layers(&s1, &a))[..3],
            [
                PlayerPart::FrontShoulderSkin,
                PlayerPart::FrontShoulderSleeve,
                PlayerPart::FrontShoulderShirt,
            ]
        );
    }

    #[test]
    fn test_dw_2_4_held_item_origins() {
        use std::f32::consts::FRAC_PI_2;
        let a = fixture();
        // Default: origin is the bottom corner mirrored by direction (`:3161`),
        // rotation from the swing arc (progress 0 at full swing).
        let mut s = state();
        s.swing = crate::item::SWING_TIME;
        s.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        let dir = 1.0f32;
        let rotation = -1.7 * dir;
        let (sin, cos) = rotation.sin_cos();
        // hand = (115, 237); origin = (0, 16).
        let expected = [
            115.0 - (0.0 * cos - 16.0 * sin),
            237.0 - (0.0 * sin + 16.0 * cos),
        ];
        let item = &held_item_layers(&s)[0];
        assert!((item.pos[0] - expected[0]).abs() < 1e-4, "{:?}", item.pos);
        assert!((item.pos[1] - expected[1]).abs() < 1e-4, "{:?}", item.pos);
        assert!((item.rotation - rotation).abs() < 1e-5);

        // useStyle 8: quarter turn, `origin.Y = 2`, `origin.X += 2*dir`.
        let mut s8 = s;
        s8.use_style = 8;
        let rotation8 = -1.7 - FRAC_PI_2 * dir;
        let (sin8, cos8) = rotation8.sin_cos();
        let expected8 = [115.0 - (2.0 * cos8 - 2.0 * sin8), 237.0 - (2.0 * sin8 + 2.0 * cos8)];
        let item8 = &held_item_layers(&s8)[0];
        assert!((item8.rotation - rotation8).abs() < 1e-5);
        assert!((item8.pos[0] - expected8[0]).abs() < 1e-4);
        assert!((item8.pos[1] - expected8[1]).abs() < 1e-4);

        // useStyle 9: the bow hold `(0.5, 0.4)` of the frame.
        let mut s9 = s;
        s9.use_style = 9;
        let item9 = &held_item_layers(&s9)[0];
        assert!((item9.rotation - -1.7).abs() < 1e-5);

        // The slot: the item sits between the body (last part Hair) and the
        // front arm (`LegacyPlayerRenderer.cs:237-239`).
        let all = parts(&assemble_layers(&s, &a));
        let hair = all.iter().position(|p| *p == PlayerPart::Hair).unwrap();
        assert_eq!(all[hair + 1], PlayerPart::HeldItem);
        assert_eq!(all[hair + 2], PlayerPart::FrontArmSkin);
    }

    #[test]
    fn test_dw_2_5_dirty_inputs() {
        let a = fixture();
        // swing = 0: no item layer anywhere.
        let s = state();
        assert!(!parts(&assemble_layers(&s, &a)).contains(&PlayerPart::HeldItem));
        // Missing textures: empty list instead of a panic.
        let empty = PlayerAssets::default();
        assert!(body_layers(&state(), &empty).is_empty());
        assert!(front_arm_layers(&state(), &empty).is_empty());
        // Unknown useStyle falls through to the default origin without panic.
        let mut sx = state();
        sx.swing = crate::item::SWING_TIME;
        sx.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        sx.use_style = 255;
        assert_eq!(held_item_layers(&sx).len(), 1);
        // Facing -1 mirrors the UVs.
        let mut sf = state();
        sf.facing = -1.0;
        let torso = &body_layers(&sf, &a)[0];
        assert!(torso.uv[0] > torso.uv[2], "mirrored quad {:?}", torso.uv);
    }

    #[test]
    fn test_dw_2_6_headgear_bob_scope() {
        let a = fixture();
        // Frame 7: the zero row — torso/arms sit 2 px lower, legs do not move.
        let s7 = PlayerCompositeState {
            body_frame: 7,
            ..state()
        };
        let body = body_layers(&s7, &a);
        let by = |p: PlayerPart| {
            body.iter()
                .find(|s| s.part == p)
                .unwrap()
                .pos[1]
        };
        assert_eq!(by(PlayerPart::TorsoSkin), 202.0);
        assert_eq!(by(PlayerPart::BackArmSkin), 202.0);
        assert_eq!(by(PlayerPart::LegsSkin), 200.0);
        assert_eq!(by(PlayerPart::Head), 200.0);
        // Frame 0: no shift anywhere.
        let body0 = body_layers(&state(), &a);
        assert!(body0.iter().all(|s| (s.pos[1] - 200.0).abs() < 1e-6));
    }
}
