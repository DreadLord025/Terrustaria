//! Pure composite tables for the player draw: the frame cells, the shoulder
//! order, the head-gear bob and the weapon slot of `PlayerDrawSet` /
//! `PlayerDrawLayers`. No render types — the builders turn these tables
//! into `LayerSprite` lists and the render passes only push them.

use super::entities::PlayerAssets;
use super::BgTex;

/// The starter palette of `Player_0_*` (`colorBodySkin`/`colorShirt`/... from
/// the default skin; the M3-EQUIP equipment palette will replace these).
pub(crate) const SKIN: [f32; 3] = [1.0, 125.0 / 255.0, 90.0 / 255.0];
pub(crate) const SHIRT: [f32; 3] = [175.0 / 255.0, 165.0 / 255.0, 140.0 / 255.0];
pub(crate) const PANTS: [f32; 3] = [1.0, 230.0 / 255.0, 175.0 / 255.0];
pub(crate) const SHOES: [f32; 3] = [160.0 / 255.0, 105.0 / 255.0, 60.0 / 255.0];
/// `colorArmorHead/Body/Legs` with empty dye slots (`PlayerDrawSet.cs:860` and
/// the `Color.White` fallbacks): the armor sheets are dyed art, plain white.
pub(crate) const ARMOR_WHITE: [f32; 3] = [1.0, 1.0, 1.0];
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
    /// `DrawPlayer_21_Head`'s helmet branch (`PlayerDrawLayers.cs:2264-2276`):
    /// the `ArmorHead[head]` sprite over the face.
    Helmet,
    /// The composite chest armor pieces (`12_Skin_Composite` /
    /// `17_TorsoComposite` / `28_ArmOverItemComposite`, `PlayerDrawLayers.cs:
    /// 1298-1320,1952-1958,3661-3706`), one per body cell family.
    BackShoulderArmor,
    BackArmArmor,
    ArmorTorso,
    FrontArmArmor,
    FrontShoulderArmor,
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
    /// `Player.gfxOffY`: the step/slope visual ease. `LegacyPlayerRenderer`
    /// receives the player at `position + (0, gfxOffY)`
    /// (`LegacyPlayerRenderer.cs:457,468`), so every body part and the held
    /// item anchor ride it — without it the sprite pops vertically on steps.
    pub(crate) gfx_off_y: f32,
    pub(crate) leg_frame: u32,
    pub(crate) body_frame: u32,
    pub(crate) facing: f32,
    /// `Player.itemAnimation / itemAnimationMax` — the remaining use fraction
    /// (1 at the swing start, 0 at rest); 0 = idle.
    pub(crate) swing: f32,
    /// `Item.holdStyle` of the held kind: 1 draws the item in the hand at rest
    /// (`PlayerDrawLayers.cs:3086` `flag2 = holdStyle != 0`).
    pub(crate) hold_style: u8,
    /// Vanilla `useStyle` stand-in: 1 = swing (default), 8 = thrust, 9 = bow.
    pub(crate) use_style: u8,
    /// The held item's atlas quad and frame size, `None` = empty hand.
    pub(crate) item_uv: Option<[f32; 4]>,
    pub(crate) item_size: [f32; 2],
    /// The light tint at the player tile (`style.shade(...)` output).
    pub(crate) light: [f32; 3],
    /// `Player.head` — the equipped helmet's armor slot (−1 = none); a set
    /// slot suppresses the hair (`Head.Sets.HidesHead`,
    /// `PlayerDrawLayers.cs:2512`). Slot 0 is real (Familiar Wig).
    pub(crate) head_armor: i16,
    /// `Player.body` — the chest armor's slot; the composite `Armor/Armor_{n}`
    /// sheet replaces the shirt torso and both arm stacks' sleeve pieces.
    pub(crate) body_armor: i16,
    /// `Player.legs` — the leg armor's slot; replaces the pants sprite.
    pub(crate) leg_armor: i16,
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
fn part_pos(
    s: &PlayerCompositeState,
    shift: f32,
    pivot_offset: [f32; 2],
    rotation: f32,
) -> [f32; 2] {
    // `LegacyPlayerRenderer` receives the player at
    // `position + (0, gfxOffY)` (`LegacyPlayerRenderer.cs:457,468`).
    let base_pos = [s.origin[0], s.origin[1] + s.gfx_off_y + shift];
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
        |t| {
            flip_uv(
                player_cell_uv(t, torso_cell(s.body_frame).0, torso_cell(s.body_frame).1),
                flip,
            )
        },
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
    let back_pos = part_pos(s, shift, composite_pivot(true, s.facing), back_rotation);
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
        if s.body_armor >= 0 { None } else { a.undershirt },
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SKIN),
    );
    // With chest armor the sleeve is not drawn (`if (!flag3)`): the armor
    // pieces replace it — back shoulder at the body anchor, back arm over the
    // skin arm (`PlayerDrawLayers.cs:1298-1320`).
    push_part(
        &mut out,
        PlayerPart::BackSleeve,
        if s.body_armor >= 0 { None } else { a.arm_shirt },
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SHIRT),
    );
    if s.body_armor >= 0 {
        push_part(
            &mut out,
            PlayerPart::BackShoulderArmor,
            a.armor_body.get(s.body_armor as usize).copied().flatten(),
            |t| flip_uv(player_cell_uv(t, BACK_SHOULDER_CELL.0, BACK_SHOULDER_CELL.1), flip),
            part_pos(s, shift, [0.0, 0.0], 0.0),
            0.0,
            tint(s.light, ARMOR_WHITE),
        );
        push_part(
            &mut out,
            PlayerPart::BackArmArmor,
            a.armor_body.get(s.body_armor as usize).copied().flatten(),
            |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
            back_pos,
            back_rotation,
            tint(s.light, ARMOR_WHITE),
        );
    }
    // 13_Leggings / 14_Shoes ride the leg frame without the shift. `legs > 0`
    // draws the armor legs texture over the skin (`PlayerDrawLayers.cs:1498`),
    // tinted `colorArmorLegs` (white with empty dye slots).
    push_part(
        &mut out,
        PlayerPart::Pants,
        if s.leg_armor >= 0 {
            a.armor_legs.get(s.leg_armor as usize).copied().flatten()
        } else {
            a.pants
        },
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, if s.leg_armor >= 0 { ARMOR_WHITE } else { PANTS }),
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
    // 17_TorsoComposite: the shirt over the (already drawn) skin torso, or the
    // composite armor torso cell (`PlayerDrawLayers.cs:1952-1958`, `!flag`
    // draws the shirt instead).
    push_part(
        &mut out,
        if s.body_armor >= 0 { PlayerPart::ArmorTorso } else { PlayerPart::Shirt },
        if s.body_armor >= 0 {
            a.armor_body.get(s.body_armor as usize).copied().flatten()
        } else {
            a.shirt
        },
        |t| {
            flip_uv(
                player_cell_uv(t, torso_cell(s.body_frame).0, torso_cell(s.body_frame).1),
                flip,
            )
        },
        part_pos(s, shift, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, if s.body_armor >= 0 { ARMOR_WHITE } else { SHIRT }),
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
    // 21_Head: head, eye whites, eyes, hair — the hair suppressed while a
    // helmet covers it (`Head.Sets.HidesHead`, `PlayerDrawLayers.cs:2512`), the
    // helmet branch drawn instead (`:2264-2276`, bodyFrame source over the
    // face anchor).
    push_part(
        &mut out,
        PlayerPart::Hair,
        if s.head_armor >= 0 { None } else { a.hair },
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, HAIR),
    );
    push_part(
        &mut out,
        PlayerPart::Helmet,
        a.armor_head.get(s.head_armor as usize).copied().flatten(),
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, 0.0, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, ARMOR_WHITE),
    );
    out
}

/// One arm composite pass: skin(7) + sleeve(13) + shirt(6) at one cell
/// (`PlayerDrawLayers.cs:3661-3706`, `Players[8]` skipped — no starter colour).
/// The composite front arm is vanilla-disabled for our item set: useStyle 1
/// never calls `SetCompositeArmFront` (`Player.cs:50686-50953` sets only
/// `itemRotation`/`itemLocation`), so the arm pose rides the bodyFrame thirds
/// through `front_arm_cell` and the item swings by `itemRotation`. Styles that
/// do enable it (5/9/11) arrive with the bow/thrown-item tasks.
pub(crate) fn swing_stand_in(_use_style: u8, _swing: f32, _facing: f32) -> CompositeArmState {
    CompositeArmState::DISABLED
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
    // The armor arm/shoulder cell rides the skin piece (`PlayerDrawLayers.cs:
    // 3661-3706` armor branch); the sleeve and shirt are armor-only skips.
    let armor_tex = if s.body_armor >= 0 {
        a.armor_body.get(s.body_armor as usize).copied().flatten()
    } else {
        None
    };
    let mut out = Vec::with_capacity(4);
    for (part, tex, color) in [
        (base, a.arm, tint(s.light, SKIN)),
        (
            if shoulder {
                PlayerPart::FrontShoulderArmor
            } else {
                PlayerPart::FrontArmArmor
            },
            armor_tex,
            tint(s.light, ARMOR_WHITE),
        ),
        (
            if shoulder {
                PlayerPart::FrontShoulderSleeve
            } else {
                PlayerPart::FrontSleeve
            },
            if s.body_armor >= 0 { None } else { a.arm_shirt },
            tint(s.light, SHIRT),
        ),
        (
            if shoulder {
                PlayerPart::FrontShoulderShirt
            } else {
                PlayerPart::FrontShirt
            },
            if s.body_armor >= 0 { None } else { a.shirt },
            tint(s.light, SHIRT),
        ),
    ] {
        let Some(tex) = tex else { continue };
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

/// `Player.ItemCheck` useStyle-1 swing rotation (`Player.cs:50848`):
/// `itemRotation = (itemAnimation / itemAnimationMax - 0.5) * (-direction) * 3.5
/// - direction * 0.3`. `itemAnimation` counts down from `itemAnimationMax`, so
/// `remaining` is 1 at the swing start (arm raised behind, `-2.05·dir`) and 0
/// at the end (forward-down, `+1.45·dir`) — a 3.5 rad sweep.
fn item_rotation(remaining: f32, direction: f32) -> f32 {
    (remaining - 0.5) * -direction * 3.5 - direction * 0.3
}

/// `Player.ItemCheck` useStyle-1 `itemLocation` thirds
/// (`Player.cs:50709-50846`): the anchor the held item pivots around, per
/// animation third, offset from `position + (width/2, y)`. The `num` tables
/// are the C# cascades over the held item's frame size. The
/// `UsesBetterMeleeItemLocation` nudge (`Player.cs:50844`) is not applied —
/// no ported item belongs to that set.
fn item_location(
    remaining: f32,
    direction: f32,
    pos_x: f32,
    pos_y: f32,
    frame_w: f32,
    frame_h: f32,
) -> [f32; 2] {
    let half = frame_w * 0.5;
    // `itemAnimation` counts DOWN, so the C# chain's first branch —
    // `itemAnimation < itemAnimationMax * 0.333` (`Player.cs:50709-50734`) —
    // is the swing END: X `10 / >32:14 / >=52:24 / >=64:28 / >=92:38` added,
    // Y `position.Y + 24`. The last branch (`>= 0.666`, `:50776-50843`)
    // *subtracts* the X offset near the swing start.
    let (num_x, loc_y) = if remaining < 0.333 {
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
    } else if remaining < 0.666 {
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
        // Swing start (`:50776-50843`): X `6 / >32:14 / >=48:18 / >=52:24 /
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
        pos_x + (half - num_x) * direction
    } else {
        pos_x - (half - num_x) * direction
    };
    [x, pos_y + loc_y]
}

/// `DrawPlayer_27_HeldItem` (`PlayerDrawLayers.cs:3043-3195`): the held item
/// pivots on `itemLocation` — the useStyle-1 thirds from `Player.ItemCheck`
/// (`Player.cs:50709-50846`) while swinging, the holdStyle-1 anchor at rest
/// (`Player.cs:50275-50460`: `position + (width/2 + frame.W/2 + 2)·dir, +24`,
/// `itemRotation = 0`) — with rotation `itemRotation` (`Player.cs:50848`),
/// plus `gfxOffY` (`PlayerDrawSet.cs:368`). The C# draw origin
/// `(width*0.5 - width*0.5*direction, height)` (`:3161`) places the pivot on
/// the quad's bottom-left/bottom-right corner; the `direction < 0` flip swaps
/// only the UVs inside that quad (XNA flips the image, not the geometry).
/// useStyle 8 thrusts with a quarter-turn near-top grip (`:3181-3188`) and
/// useStyle 9 holds a bow at `(0.5, 0.4)` of the frame (`:3162-3174`). The
/// empty hand returns no sprite.
pub(crate) fn held_item_layers(s: &PlayerCompositeState) -> Vec<LayerSprite> {
    // Guards of the original (`:3077-3092`), flattened to guard clauses: the
    // swing uses the item, the rest hold keeps `holdStyle != 0` items visible.
    let Some(uv) = s.item_uv else {
        return Vec::new();
    };
    let resting = s.swing <= 0.0;
    if s.item_size[0] <= 0.0 || (resting && s.hold_style == 0) {
        return Vec::new();
    }
    let dir = if s.facing < 0.0 { -1.0 } else { 1.0 };
    let (w, h) = (s.item_size[0], s.item_size[1]);
    let mut rotation;
    let anchor;
    if resting {
        // `holdStyle == 1` rest anchor (`Player.cs:50275-50460`); the generic
        // branch is the one our item set uses (no fishing poles or specials).
        // `position.X + width/2` is `origin.x + 20`, `position.Y + 24` rides
        // `gfxOffY`.
        anchor = [
            s.origin[0] + 20.0 + (w * 0.5 + 2.0) * dir,
            s.origin[1] + 10.0 + s.gfx_off_y + 24.0,
        ];
        rotation = 0.0;
    } else {
        rotation = item_rotation(s.swing, dir);
        let loc = item_location(s.swing, dir, s.origin[0] + 20.0, s.origin[1] + 10.0, w, h);
        anchor = [loc[0], loc[1] + s.gfx_off_y];
    }
    // `:3161`: `(width*0.5 - width*0.5*direction, height)`; useStyle tweaks.
    let mut origin = [w * 0.5 - w * 0.5 * dir, h];
    match s.use_style {
        // `:3181-3188`: thrust — quarter turn, near-top grip.
        8 if !resting => {
            rotation -= std::f32::consts::FRAC_PI_2 * dir;
            origin[1] = 2.0;
            origin[0] += 2.0 * dir;
        }
        // `:3162-3174`: bow — centre-left hold.
        9 => origin = [w * 0.5, h * 0.4],
        _ => {}
    }
    let (sin, cos) = rotation.sin_cos();
    let top_left = [
        anchor[0] - (origin[0] * cos - origin[1] * sin),
        anchor[1] - (origin[0] * sin + origin[1] * cos),
    ];
    vec![LayerSprite {
        part: PlayerPart::HeldItem,
        pos: top_left,
        size: [w, h],
        uv: flip_uv(uv, dir < 0.0),
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
        // useStyle 1 never enables the composite front arm
        // (`Player.cs:50686-50953` sets only itemRotation/itemLocation): the
        // arm pose rides the bodyFrame thirds through `front_arm_cell`.
        assert_eq!(swing_stand_in(1, 0.5, 1.0), CompositeArmState::DISABLED);
        assert_eq!(swing_stand_in(1, 1.0, 1.0), CompositeArmState::DISABLED);
        assert_eq!(swing_stand_in(1, 0.0, 1.0), CompositeArmState::DISABLED);
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
            armor_head: [None; 5],
            armor_body: [None; 5],
            armor_legs: [None; 5],
        }
    }

    fn state() -> PlayerCompositeState {
        PlayerCompositeState {
            origin: [100.0, 200.0],
            gfx_off_y: 0.0,
            hold_style: 0,
            leg_frame: 0,
            body_frame: 0,
            facing: 1.0,
            swing: 0.0,
            use_style: 1,
            item_uv: None,
            item_size: [16.0, 16.0],
            light: [1.0, 1.0, 1.0],
            head_armor: -1,
            body_armor: -1,
            leg_armor: -1,
            front_arm: CompositeArmState::DISABLED,
            back_arm: CompositeArmState::DISABLED,
        }
    }

    /// The same fixture with a full armor set in slot 1 (copper).
    fn armored() -> (PlayerAssets, PlayerCompositeState) {
        let tex = || Some(sheet());
        let armor = [None, tex(), None, None, None];
        (
            PlayerAssets {
                armor_head: armor,
                armor_body: armor,
                armor_legs: armor,
                ..fixture()
            },
            PlayerCompositeState {
                head_armor: 1,
                body_armor: 1,
                leg_armor: 1,
                ..state()
            },
        )
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
    fn test_equip_1_armor_layers_replace_the_skin_pieces() {
        // A full early-metal set: the composite torso/back-shoulder/back-arm
        // pieces replace the shirt and sleeve (`PlayerDrawLayers.cs:1298-1320,
        // 1952-1958`), the helmet replaces the hair (`:2512`), the leg armor
        // replaces the pants (`:1498`). Armor always draws over its skin.
        let (a, s) = armored();
        let body = body_layers(&s, &a);
        let idx = |p: PlayerPart| body.iter().position(|sp| sp.part == p);
        assert!(parts(&body).contains(&PlayerPart::ArmorTorso));
        assert!(parts(&body).contains(&PlayerPart::BackShoulderArmor));
        assert!(parts(&body).contains(&PlayerPart::BackArmArmor));
        assert!(parts(&body).contains(&PlayerPart::Helmet));
        assert!(!parts(&body).contains(&PlayerPart::Shirt));
        assert!(!parts(&body).contains(&PlayerPart::BackSleeve));
        assert!(!parts(&body).contains(&PlayerPart::Hair));
        // Armor over skin: torso, then back shoulder, then back arm.
        assert!(idx(PlayerPart::TorsoSkin) < idx(PlayerPart::ArmorTorso));
        assert!(idx(PlayerPart::BackArmSkin) < idx(PlayerPart::BackArmArmor));
        let arms = front_arm_layers(&s, &a);
        assert!(parts(&arms).contains(&PlayerPart::FrontArmArmor));
        assert!(parts(&arms).contains(&PlayerPart::FrontShoulderArmor));
        assert!(!parts(&arms).contains(&PlayerPart::FrontShirt));
        assert!(!parts(&arms).contains(&PlayerPart::FrontSleeve));
        // The unarmored builders keep the vanilla no-armor set.
        let plain = body_layers(&state(), &fixture());
        assert!(parts(&plain).contains(&PlayerPart::Shirt));
        assert!(parts(&plain).contains(&PlayerPart::Hair));
        assert!(!parts(&plain).contains(&PlayerPart::Helmet));
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
        // Default useStyle 1 at the swing start (`swing = 1.0`,
        // `remaining = 1`): `itemRotation = (1 - 0.5) * (-1) * 3.5 - 0.3 =
        // -2.05` (`Player.cs:50858`); `itemLocation` start third for a 16x16
        // frame = `position - (8 - 6, -10)` = (100+20-2, 200+10+10) =
        // (118, 220) (`Player.cs:50776-50843`); grip origin `(0, 16)`
        // (`PlayerDrawLayers.cs:3161`).
        let mut s = state();
        s.swing = 1.0; // itemAnimation == itemAnimationMax: the swing start
        s.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        let rotation = -2.05f32;
        let (sin, cos) = rotation.sin_cos();
        let expected = [118.0 + 16.0 * sin, 220.0 - 16.0 * cos];
        let item = &held_item_layers(&s)[0];
        assert!((item.pos[0] - expected[0]).abs() < 1e-4, "{:?}", item.pos);
        assert!((item.pos[1] - expected[1]).abs() < 1e-4, "{:?}", item.pos);
        assert!((item.rotation - rotation).abs() < 1e-5);

        // The gfxOffY ease rides the anchor (`PlayerDrawSet.cs:368`).
        let mut sg = s;
        sg.gfx_off_y = 5.0;
        let itemg = &held_item_layers(&sg)[0];
        assert!((itemg.pos[1] - (expected[1] + 5.0)).abs() < 1e-4);

        // useStyle 8: quarter turn, `origin.Y = 2`, `origin.X += 2*dir`
        // (`PlayerDrawLayers.cs:3181-3188`).
        let mut s8 = s;
        s8.use_style = 8;
        let rotation8 = -2.05 - FRAC_PI_2;
        let (sin8, cos8) = rotation8.sin_cos();
        let expected8 = [
            118.0 - (2.0 * cos8 - 2.0 * sin8),
            220.0 - (2.0 * sin8 + 2.0 * cos8),
        ];
        let item8 = &held_item_layers(&s8)[0];
        assert!((item8.rotation - rotation8).abs() < 1e-5);
        assert!((item8.pos[0] - expected8[0]).abs() < 1e-4);
        assert!((item8.pos[1] - expected8[1]).abs() < 1e-4);

        // useStyle 9: the bow hold `(0.5, 0.4)` of the frame
        // (`:3162-3174`); the rotation stays the swing sweep.
        let mut s9 = s;
        s9.use_style = 9;
        let item9 = &held_item_layers(&s9)[0];
        assert!((item9.rotation - -2.05).abs() < 1e-5);

        // Facing left mirrors the item UVs and puts the pivot on the drawn
        // sprite's bottom-left (`origin.x -> width - origin.x`).
        let mut sl = s;
        sl.facing = -1.0;
        let iteml = &held_item_layers(&sl)[0];
        assert!(
            iteml.uv[0] > iteml.uv[2],
            "mirrored item quad {:?}",
            iteml.uv
        );

        // The slot: the item sits between the body (last part Hair) and the
        // front arm (`LegacyPlayerRenderer.cs:237-239`).
        let all = parts(&assemble_layers(&s, &a));
        let hair = all.iter().position(|p| *p == PlayerPart::Hair).unwrap();
        assert_eq!(all[hair + 1], PlayerPart::HeldItem);
        assert_eq!(all[hair + 2], PlayerPart::FrontArmSkin);
    }

    #[test]
    fn rest_hold_anchors_the_torch_at_the_hand() {
        // `Player.cs:50275-50460` (holdStyle == 1): the generic anchor is
        // `position.X + width/2 + (frame.W/2 + 2) * direction`,
        // `position.Y + 24`, with `itemRotation = 0`. A 20x28 torch facing
        // right: anchor = (100+20+12, 200+10+24) = (132, 234); the grip
        // origin `(0, 28)` puts the quad's bottom-left at the anchor.
        let mut s = state();
        s.swing = 0.0;
        s.hold_style = 1;
        s.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        s.item_size = [20.0, 28.0];
        let item = &held_item_layers(&s)[0];
        assert!((item.pos[0] - 132.0).abs() < 1e-4, "{:?}", item.pos);
        assert!(
            (item.pos[1] - (234.0 - 28.0)).abs() < 1e-4,
            "{:?}",
            item.pos
        );
        assert_eq!(item.rotation, 0.0);
        // Facing left mirrors the UVs and the quad spans left of the anchor.
        let mut sl = s;
        sl.facing = -1.0;
        let iteml = &held_item_layers(&sl)[0];
        assert!(iteml.uv[0] > iteml.uv[2]);
        assert!(
            (iteml.pos[0] - (100.0 + 20.0 - 12.0 - 20.0)).abs() < 1e-4,
            "{:?}",
            iteml.pos
        );
        // Without holdStyle the hand is empty at rest.
        let mut sn = state();
        sn.swing = 0.0;
        sn.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        assert!(held_item_layers(&sn).is_empty());
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
        sx.swing = 1.0;
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
        let by = |p: PlayerPart| body.iter().find(|s| s.part == p).unwrap().pos[1];
        assert_eq!(by(PlayerPart::TorsoSkin), 202.0);
        assert_eq!(by(PlayerPart::BackArmSkin), 202.0);
        assert_eq!(by(PlayerPart::LegsSkin), 200.0);
        assert_eq!(by(PlayerPart::Head), 200.0);
        // Frame 0: no shift anywhere.
        let body0 = body_layers(&state(), &a);
        assert!(body0.iter().all(|s| (s.pos[1] - 200.0).abs() < 1e-6));
    }
}
