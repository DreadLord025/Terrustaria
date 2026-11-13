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
/// `underShirtColor` (`Player.cs:2562`, (160,180,215)): the under-shirt sleeve
/// `Players[8]` on both arm stacks.
pub(crate) const UNDER_SHIRT: [f32; 3] = [160.0 / 255.0, 180.0 / 255.0, 215.0 / 255.0];
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
    /// The mount's back/front sprite (`Mount.Draw`, `Mount.cs:5501-5935`).
    Mount,
    TorsoSkin,
    LegsSkin,
    BackArmSkin,
    BackUnder,
    /// `Players[8]` on the back arm, the under-shirt sleeve
    /// (`PlayerDrawLayers.cs:1357`, `colorUnderShirt`).
    BackUnderArm,
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
    /// `Players[8]` between the skin arm and the sleeve
    /// (`PlayerDrawLayers.cs:3698`).
    FrontArmUnder,
    FrontSleeve,
    FrontShirt,
    FrontShoulderSkin,
    FrontShoulderUnder,
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
    /// The dye pass this layer draws with (0 = plain) — `DrawData.shader`'s
    /// source, the `c*` cache value `PlayerDrawHelper.SetShaderForData`
    /// unpacks (`PlayerDrawHelper.cs:29-60`). Assigned by `apply_dye` after
    /// assembly; the builders keep 0.
    pub(crate) dye: u32,
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
/// The mount draw inputs (`Mount.Draw` `Mount.cs:5501-5935`): the mount id
/// (the row lookup for geometry + the assets' sheet tables), the animator's
/// frame and the per-frame player offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MountDraw {
    pub(crate) id: i32,
    pub(crate) frame: i32,
    pub(crate) player_offset: i32,
    /// `Mount.IsAtIdle` (:594-608) — Rudolph hides its front strip while
    /// idle (`Mount.cs:5520-5523`).
    pub(crate) is_idle: bool,
}

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
    /// `Player.itemRotation`'s source for useStyle 5 — the aim angle.
    pub(crate) aim: f32,
    /// The held item's atlas quad and frame size, `None` = empty hand.
    pub(crate) item_uv: Option<[f32; 4]>,
    pub(crate) item_size: [f32; 2],
    /// The light tint at the player tile (`style.shade(...)` output).
    pub(crate) light: [f32; 3],
    /// The live mount's draw inputs, `None` = unmounted.
    pub(crate) mount: Option<MountDraw>,
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
    /// `UpdateDyes`' cache for the layers with draw consumers: `cHead/
    /// cBody/cLegs` read the dye slots directly (`Player.cs:9593-9595`) and
    /// `cMount` mirrors `miscDyes[3].dye` (`:9600-9604` — `Mount.currentShader`,
    /// `PlayerDrawSet.cs:432-437`). 0 = plain.
    pub(crate) c_head: u32,
    pub(crate) c_body: u32,
    pub(crate) c_legs: u32,
    pub(crate) c_mount: u32,
    /// `PlayerSittingHelper.isSitting` (`PlayerSittingHelper.cs:18`) — the
    /// legs re-slice and the torso drop switch on it.
    pub(crate) sitting: bool,
    /// `GetSittingOffsetInfo`'s `seatAdjustment` (`PlayerSittingHelper.cs:
    /// 23-24`): `-4 + (int)offsetForSeat.Y` — the torso/head drop
    /// (`PlayerDrawSet.cs:345`'s `torsoOffset`, consumed by the torso
    /// sandwich `LegacyPlayerRenderer.cs:205-225` and the composite sites
    /// `PlayerDrawLayers.cs:1216,1266`).
    pub(crate) seat_y_offset: i32,
    /// `PlayerSleepingHelper.isSleeping` (`:13`) — arms the pose rotation.
    pub(crate) sleeping: bool,
    /// `Player.fullRotation` (`PlayerSleepingHelper.cs:73`): `PI/2 *
    /// (-direction)` while asleep.
    pub(crate) full_rotation: f32,
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
    if frame == 5 { (1, 0) } else { (0, 0) }
}

/// `compShoulderOverFrontArm`: when true the shoulder stack draws after (over)
/// the arm stack. Default true; rows 1, 2 and 5 clear it
/// (`PlayerDrawSet.cs:1956,1961,1966,1971,1976`).
pub(crate) fn shoulder_over_front_arm(frame: u32) -> bool {
    !matches!(frame, 1 | 2 | 5)
}

/// `ArmorIDs.Body.Sets.showsShouldersWhileJumping` (`ID/ArmorIDs.cs:714`) — the
/// body ids whose composite shoulders stay visible while jumping. The set is
/// unordered; the list is sorted for readability.
const SHOWS_SHOULDERS_WHILE_JUMPING: [i16; 34] = [
    24, 46, 54, 55, 56, 65, 66, 67, 69, 70, 71, 73, 75, 92, 95, 105, 106, 175, 176, 177, 182, 183,
    187, 190, 194, 197, 198, 201, 204, 226, 227, 229, 239, 259,
];

/// `PlayerDrawSet.cs:1933-1937` + `:1984-1987`: `flag` is
/// `ArmorIDs.Body.Sets.showsShouldersWhileJumping[body]`, and on body row 5 an
/// armor that does NOT show its shoulders while jumping raises
/// `hideCompositeShoulders` (`:1932` clears the flag; `:1940`/`:1944` raise it
/// for a coat / a front accessory — the port has neither). The vanilla then
/// skips the shoulder pieces (`PlayerDrawLayers.cs:1307` the back shoulder,
/// `:3624` the front shoulder). Row 5 is the falling pose, so a worn chest
/// armor loses its shoulders on every fall.
pub(crate) fn hide_composite_shoulders(body_frame: u32, body_armor: i16) -> bool {
    if body_frame != 5 || body_armor < 0 {
        return false;
    }
    !SHOWS_SHOULDERS_WHILE_JUMPING.contains(&body_armor)
}

/// The raw `OffsetsPlayerHeadgear` table row — Y is 0 exactly at rows 7-9 and
/// 14-16, 2 elsewhere (`Main.cs:66638-66662`).
pub(crate) fn headgear_offset(frame: u32) -> f32 {
    crate::player::offsets_player_headgear_y(frame as usize)
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
///
/// The row count rounds **up**: the head/eye-whites/eyes strips are 1118 px
/// tall, i.e. 19 full rows plus a 54-px row 19 (`Player_0_0`), so a `floor`
/// count clamped the walk cycle's fastest band (row 19) back onto row 18 and
/// the head lagged the legs by one frame. The quad's bottom edge is clamped to
/// the sheet so the short last row never samples the neighbouring atlas sheet.
pub(crate) fn player_frame_uv(tex: &BgTex, row: u32) -> [f32; 4] {
    let rows = (tex.h / 56.0).ceil().max(1.0);
    let r = (row as f32).min(rows - 1.0);
    let dv = (tex.uv[3] - tex.uv[1]) * (56.0 / tex.h);
    let v0 = tex.uv[1] + dv * r;
    [tex.uv[0], v0, tex.uv[2], (v0 + dv).min(tex.uv[3])]
}

/// `PlayerDrawSet.cs:1766-1786`: the hair frame is the body frame shifted up
/// six rows (`bodyFrame.Y -= 336`) and cropped 2 px (`Height -= 2`). The
/// `Player_Hair_*` strips are 14 rows (784 px), so body rows 0..6 land on hair
/// row 0 and rows 7..19 on hair rows 1..13.
pub(crate) fn hair_row(body_frame: u32) -> u32 {
    body_frame.saturating_sub(6)
}

/// The hair source rect: one 40x54 frame at [`hair_row`].
pub(crate) fn hair_uv(tex: &BgTex, body_frame: u32) -> [f32; 4] {
    let uv = player_frame_uv(tex, hair_row(body_frame));
    let dv = (uv[3] - uv[1]) * (2.0 / 56.0);
    [uv[0], uv[1], uv[2], uv[3] - dv]
}

/// The helmet source rect: the normal-hat branch draws `bodyFrame` with
/// `Height -= 4` (`PlayerDrawLayers.cs:2254-2261`), a 40x52 crop.
pub(crate) fn helmet_uv(tex: &BgTex, body_frame: u32) -> [f32; 4] {
    let uv = player_frame_uv(tex, body_frame);
    let dv = (uv[3] - uv[1]) * (4.0 / 56.0);
    [uv[0], uv[1], uv[2], uv[3] - dv]
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
    push_part_h(out, part, tex, uv, pos, rotation, color, 56.0);
}

/// [`push_part`] for the two layers the C# crops: the hair (`Height -= 2`) and
/// the helmet (`Height -= 4`). The quad takes the cropped height so the art
/// keeps its 1:1 pixel scale.
#[allow(clippy::too_many_arguments)]
fn push_part_h(
    out: &mut Vec<LayerSprite>,
    part: PlayerPart,
    tex: Option<BgTex>,
    uv: impl Fn(&BgTex) -> [f32; 4],
    pos: [f32; 2],
    rotation: f32,
    color: [f32; 4],
    height: f32,
) {
    if let Some(tex) = tex {
        out.push(LayerSprite {
            part,
            pos,
            size: [40.0, height],
            uv: uv(&tex),
            color,
            rotation,
            dye: 0,
        });
    }
}

/// `PlayerSleepingHelper.SetOffsetbyBed` (`PlayerSleepingHelper.cs:226-279`),
/// verbatim: the per-bed-style visual offset of the bed base, keyed by
/// `frameY / 36`; unlisted styles take the default.
pub(crate) fn bed_visual_offset(bed_style: u32) -> [f32; 2] {
    match bed_style {
        8 => [-11.0, 1.0],
        10 => [-9.0, -1.0],
        11 => [-11.0, 1.0],
        13 => [-11.0, -3.0],
        15 | 16 | 17 => [-7.0, -3.0],
        18 => [-9.0, -3.0],
        19 => [-3.0, -1.0],
        20 => [-9.0, -5.0],
        21 => [-9.0, 5.0],
        22 => [-7.0, 1.0],
        23 => [-5.0, -1.0],
        24 | 25 => [-7.0, 1.0],
        27 => [-9.0, 3.0],
        28 => [-9.0, 5.0],
        29 => [-11.0, -1.0],
        30 => [-9.0, 3.0],
        31 => [-7.0, 5.0],
        32 => [-7.0, -1.0],
        34 | 35 | 36 | 37 => [-13.0, 1.0],
        38 => [-11.0, -3.0],
        _ => [-9.0, 1.0],
    }
}

/// `DrawSittingLegs`' slice geometry (`PlayerDrawLayers.cs:1545-1554`,
/// `:1672-1695`) for one leg texture: three strips of the standing legs cell
/// (row 0 while seated) — the bottom block (cell rows 46..56, 10 px) fanned
/// `+4 px` along the facing plus two 2-px knee slices (44..46 at +2,
/// 42..44 at +0) — all dropped `num2` = 42 px below the sheet anchor with
/// the pre-shift `(+2·direction, +2)`, the `-2` lift and the seat drop.
/// Returns `(x_offset, y_offset, cell_y, cell_h)` per strip in draw order
/// (bottom block first, knee slices over it).
pub(crate) fn sitting_leg_strips(seat_y_offset: f32) -> [(f32, f32, f32, f32); 3] {
    // The C# constants: `num = 2, num2 = 42, num3 = 2, num4 = 2, num5 = 0`.
    let (num, num2, num3, num4) = (2.0f32, 42.0f32, 2i32, 2.0f32);
    let mut out = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 3];
    for (i, num10) in (0..=num3).rev().enumerate() {
        let n = num10 as f32;
        // `position = vector + (num, 2) * (direction, 1)`, then
        // `.X += direction * num4 * num10`, `.Y += num2 + num5`.
        let x_off = num + num4 * n;
        let y_off = -2.0 + seat_y_offset + 2.0 + num2 + 0.0;
        // `value.Y += num10 * 2 + num2; value.Height -= num2 + num10 * 2;
        //  if (num10 != num3) value.Height = 2`.
        let cell_y = n * 2.0 + num2;
        let cell_h = if num10 != num3 {
            2.0
        } else {
            56.0 - num2 - n * 2.0
        };
        out[i] = (x_off, y_off, cell_y, cell_h);
    }
    out
}

/// One leg texture's seated draw (`DrawSittingLegs`' call sites: the leg
/// skin `PlayerDrawLayers.cs:1154-1157`/`:1266-1270`, the leggings
/// `:1427-1455` and the shoes `:1719-1721`): three re-sliced strips at the
/// sheet anchor with the X fan along the facing.
fn push_sitting_legs(
    out: &mut Vec<LayerSprite>,
    part: PlayerPart,
    tex: Option<BgTex>,
    s: &PlayerCompositeState,
    color: [f32; 4],
) {
    let Some(tex) = tex else { return };
    let anchor = part_pos(s, 0.0, [0.0, 0.0], 0.0);
    for (x_off, y_off, cell_y, cell_h) in sitting_leg_strips(s.seat_y_offset as f32) {
        out.push(LayerSprite {
            part,
            pos: [anchor[0] + x_off * s.facing, anchor[1] + y_off],
            size: [40.0, cell_h],
            uv: player_leg_strip_uv(&tex, s.leg_frame, cell_y, cell_h),
            color,
            rotation: 0.0,
            dye: 0,
        });
    }
}

/// The strip sub-rect of one 40x56 cell row: `[y0, y0 + h)` in cell pixels.
fn player_leg_strip_uv(tex: &BgTex, row: u32, cell_y: f32, cell_h: f32) -> [f32; 4] {
    let full = player_frame_uv(tex, row);
    let dv = full[3] - full[1];
    let v0 = full[1] + dv * (cell_y / 56.0);
    [full[0], v0, full[2], v0 + dv * (cell_h / 56.0)]
}

/// `DrawPlayer_TransformDrawData` (`PlayerDrawLayers.cs:4092-4119`): the
/// sleeping pose rotates every sprite's position around the player's
/// `fullRotationOrigin` and adds the rotation to each sprite's own. (The
/// `ignorePlayerRotation` opt-outs — the mount strip `Mount.cs:6059` and the
/// beheaded halo — are unreachable while asleep: the sleep dismounts and
/// there is no halo.)
pub(crate) fn apply_player_rotation(sprites: &mut [LayerSprite], pivot: [f32; 2], rotation: f32) {
    if rotation == 0.0 {
        return;
    }
    let (sin, cos) = rotation.sin_cos();
    for sp in sprites.iter_mut() {
        let dx = sp.pos[0] - pivot[0];
        let dy = sp.pos[1] - pivot[1];
        sp.pos = [
            pivot[0] + dx * cos - dy * sin,
            pivot[1] + dx * sin + dy * cos,
        ];
        sp.rotation += rotation;
    }
}

/// `DrawPlayer_12_Skin_Composite` → `13_Leggings` → `14_Shoes` →
/// `17_TorsoComposite` → `21_Head` (`LegacyPlayerRenderer.cs:195-215`): the
/// body parts in original order. The composite torso/arm sprites take the
/// head-gear shift (`PlayerDrawLayers.cs:1217-1219`); legs and head do not.
pub(crate) fn body_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    let mut out = Vec::with_capacity(12);
    let flip = s.facing < 0.0;
    // `PlayerDrawSet.cs:343-347`: while seated the torso sandwich layers
    // (torso, back arm, head — `LegacyPlayerRenderer.cs:183-225`) ride
    // `torsoOffset = seatYOffset`; the legs re-slice carries its own drop.
    let seat_drop = if s.sitting {
        s.seat_y_offset as f32
    } else {
        0.0
    };
    let shift = headgear_composite_shift(s.body_frame) + seat_drop;
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
    // `DrawPlayer_12_Skin`'s bottom-skin block (`PlayerDrawLayers.cs:1150-1157`):
    // seated the leg-skin piece goes through `DrawSittingLegs`.
    if s.sitting {
        push_sitting_legs(
            &mut out,
            PlayerPart::LegsSkin,
            a.legs,
            s,
            tint(s.light, SKIN),
        );
    } else {
        push_part(
            &mut out,
            PlayerPart::LegsSkin,
            a.legs,
            |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
            part_pos(s, 0.0, [0.0, 0.0], 0.0),
            0.0,
            tint(s.light, SKIN),
        );
    }
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
    // The under-skin layer (`Players[skinVar, 5]`, body-skin tint) rides on
    // BOTH armor paths, right behind the arm skin: with a chest armor the
    // `flag2 && missingArm` block draws it (`PlayerDrawLayers.cs:1294-1302` -
    // `missingArm` is true for every body id but 83, `PlayerDrawSet.cs:
    // 377-384`), without one the `!flag6` block draws it (`:1346-1353`). The
    // pre-fix draw skipped it under armor, so the armor's under-arm art showed
    // through where the vanilla paints the skin.
    push_part(
        &mut out,
        PlayerPart::BackUnder,
        a.undershirt,
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SKIN),
    );
    // The under-shirt sleeve (`Players[8]`, `colorUnderShirt`) and the shirt
    // sleeve (`Players[13]`) live in the `!flag3` block
    // (`PlayerDrawLayers.cs:1355-1360`), so a chest armor drops them: the
    // armor pieces replace them.
    push_part(
        &mut out,
        PlayerPart::BackUnderArm,
        if s.body_armor >= 0 { None } else { a.under_arm },
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, UNDER_SHIRT),
    );
    push_part(
        &mut out,
        PlayerPart::BackSleeve,
        if s.body_armor >= 0 { None } else { a.arm_shirt },
        |t| flip_uv(player_cell_uv(t, back_cell.0, back_cell.1), flip),
        back_pos,
        back_rotation,
        tint(s.light, SHIRT),
    );
    // The armor pieces close the back-arm stack: the back shoulder at the
    // body anchor and the back arm over the skin arm
    // (`PlayerDrawLayers.cs:1304-1333`, the `DrawCompositeArmorPiece` calls
    // at `:1309` and `:1322`). The shoulder is skipped when
    // `hideCompositeShoulders` is raised (`:1307`).
    if s.body_armor >= 0 {
        if !hide_composite_shoulders(s.body_frame, s.body_armor) {
            push_part(
                &mut out,
                PlayerPart::BackShoulderArmor,
                a.armor_body.get(s.body_armor as usize).copied().flatten(),
                |t| {
                    flip_uv(
                        player_cell_uv(t, BACK_SHOULDER_CELL.0, BACK_SHOULDER_CELL.1),
                        flip,
                    )
                },
                part_pos(s, shift, [0.0, 0.0], 0.0),
                0.0,
                tint(s.light, ARMOR_WHITE),
            );
        }
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
    // tinted `colorArmorLegs` (white with empty dye slots). Seated, both go
    // through `DrawSittingLegs` (`:1427-1455`, `:1719-1721` — the armor
    // overrides for legs 106/143/226/217 etc. stay with the armor sets).
    let legs_tex = if s.leg_armor >= 0 {
        a.armor_legs.get(s.leg_armor as usize).copied().flatten()
    } else {
        a.pants
    };
    let legs_tint = tint(s.light, if s.leg_armor >= 0 { ARMOR_WHITE } else { PANTS });
    if s.sitting {
        push_sitting_legs(&mut out, PlayerPart::Pants, legs_tex, s, legs_tint);
    } else {
        push_part(
            &mut out,
            PlayerPart::Pants,
            legs_tex,
            |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
            part_pos(s, 0.0, [0.0, 0.0], 0.0),
            0.0,
            legs_tint,
        );
    }
    if s.sitting {
        push_sitting_legs(
            &mut out,
            PlayerPart::Shoes,
            a.shoes,
            s,
            tint(s.light, SHOES),
        );
    } else {
        push_part(
            &mut out,
            PlayerPart::Shoes,
            a.shoes,
            |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
            part_pos(s, 0.0, [0.0, 0.0], 0.0),
            0.0,
            tint(s.light, SHOES),
        );
    }
    // 17_TorsoComposite: the shirt over the (already drawn) skin torso, or the
    // composite armor torso cell (`PlayerDrawLayers.cs:1952-1958`, `!flag`
    // draws the shirt instead).
    push_part(
        &mut out,
        if s.body_armor >= 0 {
            PlayerPart::ArmorTorso
        } else {
            PlayerPart::Shirt
        },
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
        tint(
            s.light,
            if s.body_armor >= 0 {
                ARMOR_WHITE
            } else {
                SHIRT
            },
        ),
    );
    // 21_Head: head, eye whites, eyes, hair — the whole block sits inside
    // the torso sandwich (`LegacyPlayerRenderer.cs:205-225`), so the seated
    // drop rides along.
    push_part(
        &mut out,
        PlayerPart::Head,
        a.head,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, seat_drop, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, SKIN),
    );
    push_part(
        &mut out,
        PlayerPart::EyeWhites,
        a.eye_whites,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, seat_drop, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, EYE_WHITE),
    );
    push_part(
        &mut out,
        PlayerPart::Eyes,
        a.eyes,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        part_pos(s, seat_drop, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, EYES),
    );
    // 21_Head: head, eye whites, eyes, hair. The hair rides its own trimmed
    // frame (`hairFrontFrame`, `PlayerDrawSet.cs:1766-1786`); the helmet the
    // normal-hat branch's 52-px crop (`PlayerDrawLayers.cs:2254-2261`). The
    // hair stays suppressed while a helmet covers it: the ported metal sets
    // are plain normal hats (neither `fullHair` nor `hatHair`), and that
    // branch draws no hair at all (`:2262-2380`).
    push_part_h(
        &mut out,
        PlayerPart::Hair,
        if s.head_armor >= 0 { None } else { a.hair },
        |t| flip_uv(hair_uv(t, s.body_frame), flip),
        part_pos(s, seat_drop, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, HAIR),
        54.0,
    );
    push_part_h(
        &mut out,
        PlayerPart::Helmet,
        a.armor_head.get(s.head_armor as usize).copied().flatten(),
        |t| flip_uv(helmet_uv(t, s.body_frame), flip),
        part_pos(s, seat_drop, [0.0, 0.0], 0.0),
        0.0,
        tint(s.light, ARMOR_WHITE),
        52.0,
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
pub(crate) fn swing_stand_in(use_style: u8, swing: f32, facing: f32) -> CompositeArmState {
    // The per-style composite arms (`ApplyItemDrawData`'s
    // `SetCompositeArmFront/Back` calls): style 1 never enables them
    // (`Player.cs:50686-50953`), styles 2/11/13 do.
    let progress = 1.0 - swing; // itemAnimationMax - itemAnimation, normalized
    let stretch = |p: f32| {
        if p <= 0.25 {
            CompositeStretch::Full
        } else if p <= 0.5 {
            CompositeStretch::ThreeQuarters
        } else if p <= 0.75 {
            CompositeStretch::Quarter
        } else {
            CompositeStretch::None
        }
    };
    match use_style {
        // `Player.cs:50936-50956` (EatFood): both arms at -pi/2 * dir, the
        // stretch walking Full -> None as the use runs out.
        2 => {
            let st = stretch(progress);
            CompositeArmState {
                enabled: true,
                stretch: st,
                rotation: -std::f32::consts::FRAC_PI_2 * facing,
            }
        }
        // `Player.cs:51006-51007` (MowTheLawn): the front arm Quarter at
        // -pi/4 * dir, the back arm Full at -pi/16.
        11 => CompositeArmState {
            enabled: true,
            stretch: {
                let p = progress * 2.0;
                if p <= 0.25 {
                    CompositeStretch::Full
                } else if p <= 0.5 {
                    CompositeStretch::ThreeQuarters
                } else if p <= 0.75 {
                    CompositeStretch::Quarter
                } else {
                    CompositeStretch::None
                }
            },
            rotation: -std::f32::consts::FRAC_PI_4 * facing,
        },
        // `Player.cs:51195` (Rapier): the front arm at the item rotation
        // minus pi/2 * dir, the stretch by the swing thirds.
        13 => {
            let st = if progress <= 0.33 {
                CompositeStretch::Full
            } else if progress <= 0.66 {
                CompositeStretch::ThreeQuarters
            } else {
                CompositeStretch::Quarter
            };
            CompositeArmState {
                enabled: true,
                stretch: st,
                rotation: -std::f32::consts::FRAC_PI_2 * facing,
            }
        }
        _ => CompositeArmState::DISABLED,
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
    // The armor arm/shoulder cell rides the skin piece (`PlayerDrawLayers.cs:
    // 3661-3706` armor branch); the sleeve and shirt are armor-only skips.
    let armor_tex = if s.body_armor >= 0 {
        a.armor_body.get(s.body_armor as usize).copied().flatten()
    } else {
        None
    };
    let mut out = Vec::with_capacity(5);
    for (part, tex, color) in [
        (
            base,
            // `DrawPlayer_28_ArmOverItemComposite`'s armor branch
            // (`PlayerDrawLayers.cs:3600-3661`) paints the skin arm
            // (`Players[7]` at the ARM frame, `:3611` - drawn because
            // `missingArm` is true for every body id but 83, `PlayerDrawSet.cs:
            // 377-384`) and the hand (`Players[9]`, `:3618`), but never a skin
            // piece at the SHOULDER frame: the shoulder skin
            // (`Players[7]` at `compFrontShoulderFrame`) only exists in the
            // no-armor branch (`:3670`). Under a chest armor the shoulder
            // slot carries the armor piece alone - the pre-fix draw kept the
            // skin, so a bare shoulder stripe showed through the armor.
            if shoulder && s.body_armor >= 0 {
                None
            } else {
                a.arm
            },
            tint(s.light, SKIN),
        ),
        (
            if shoulder {
                PlayerPart::FrontShoulderArmor
            } else {
                PlayerPart::FrontArmArmor
            },
            // The shoulder piece is skipped when `hideCompositeShoulders` is
            // raised (`PlayerDrawLayers.cs:3624`).
            if shoulder && hide_composite_shoulders(s.body_frame, s.body_armor) {
                None
            } else {
                armor_tex
            },
            tint(s.light, ARMOR_WHITE),
        ),
        (
            if shoulder {
                PlayerPart::FrontShoulderUnder
            } else {
                PlayerPart::FrontArmUnder
            },
            if s.body_armor >= 0 { None } else { a.under_arm },
            tint(s.light, UNDER_SHIRT),
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
            dye: 0,
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
    let mut arm = arm_stack(
        s,
        a,
        PlayerPart::FrontArmSkin,
        cell,
        rotation,
        composite_pivot(false, s.facing),
    );
    // `PlayerDrawLayers.cs:3604-3608`: an enabled composite front arm (column
    // 7) shifts the arm stack one pixel towards the facing and one down
    // (`vector += (+/-1, +/-1)`); the shoulder keeps its own anchor, built
    // before the nudge.
    if cell.0 >= 7 {
        for sprite in arm.iter_mut() {
            sprite.pos[0] += s.facing;
            sprite.pos[1] += 1.0;
        }
    }
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
        // The per-style held-item placement — `Player.ItemDraw`'s
        // useStyle branches (`Player.cs:50686-51243`): 1's arc, 3's thrust,
        // 4's hold-up, 5's aim. Styles 2/7/9/11/13 pin the item near the
        // hand and let the composite arms carry the pose.
        match s.use_style {
            // `:51065-51088` — HoldUp: rotation 0, the item beside the
            // shoulder (`width*0.5 - 9` right of centre, mid-height + 4).
            4 => {
                rotation = 0.0;
                anchor = [
                    s.origin[0] + 20.0 + (w * 0.5 - 9.0) * dir,
                    s.origin[1] + 10.0 + s.gfx_off_y + h * 0.5 + 4.0,
                ];
            }
            // `:51091-51136` — Shoot: the rotation is the aim
            // (`Player.itemRotation`); the generic anchor centres the item
            // `direction * -2` off the body and half-height above the
            // centre. The staffs (Item.staff) anchor at the MountedCenter +
            // `rotation.ToRotationVector2() * 6` — the port's staff set is
            // empty, so the generic arm only.
            5 => {
                rotation = s.aim;
                anchor = [
                    s.origin[0] + 20.0 - dir * 2.0,
                    s.origin[1] + 10.0 + s.gfx_off_y - h * 0.5 + 16.0,
                ];
            }
            // `:51031-51064` — Thrust: the first third tucks the item
            // offscreen (`itemLocation = -1000`), then rotation `0.8 * dir`
            // with the X-thrust by the swing progress.
            3 => {
                if s.swing > 2.0 / 3.0 {
                    anchor = [-1000.0, -1000.0];
                    rotation = -1.3 * dir;
                } else {
                    rotation = 0.8 * dir;
                    let num14 = (s.swing * w * dir * 1.2 - 10.0 * dir).clamp(
                        if dir < 0.0 { -8.0 } else { f32::MIN },
                        if dir > 0.0 { 8.0 } else { f32::MAX },
                    );
                    anchor = [
                        s.origin[0] + 20.0 + (w * 0.5 - 4.0) * dir - num14,
                        s.origin[1] + 10.0 + s.gfx_off_y + 24.0,
                    ];
                }
            }
            // The default arc — useStyle 1 (`:50686-50935`): the port's
            // `item_rotation`/`item_location` pair is the third-walk + the
            // `(anim/max - 0.5) * -dir * 3.5 - dir * 0.3` rotation.
            _ => {
                rotation = item_rotation(s.swing, dir);
                let loc = item_location(s.swing, dir, s.origin[0] + 20.0, s.origin[1] + 10.0, w, h);
                anchor = [loc[0], loc[1] + s.gfx_off_y];
            }
        }
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
        dye: 0,
    }]
}

/// The mount sprite (`Mount.Draw` `Mount.cs:5501-5735`): one vertical-strip
/// frame, `frameH = textureHeight / totalFrames`, the drawn band cropped by
/// the vanilla `Height -= 2` (`:5722`). The frame center lands at
/// `player.center + (xOffset * direction, yOffset + PlayerOffset)`
/// (`:5554-5561`, origin `:622`).
fn mount_layer(s: &PlayerCompositeState, a: &PlayerAssets, front: bool) -> Option<LayerSprite> {
    let m = s.mount?;
    let idx = crate::mount::mount_index(m.id)?;
    let row = crate::mount::mount_data(m.id)?;
    let tex = if front {
        &a.mount_front[idx]
    } else {
        &a.mount_back[idx]
    }
    .as_ref()?;
    // An empty sheet name is the C# `Asset<Texture2D>.Empty` default — the
    // drawType draws nothing (`Mount.cs:5509-5535`).
    let sheet = if front {
        row.front_sheet
    } else {
        row.back_sheet
    };
    if sheet.is_empty() {
        return None;
    }
    // `:5520-5523`: Rudolph hides the front strip while idle.
    if front && m.id == 0 && m.is_idle {
        return None;
    }
    let frame_h = tex.h / row.total_frames.max(1) as f32;
    let band_h = frame_h - 2.0;
    let frame = (m.frame.max(0) as f32).min((row.total_frames.max(1) - 1) as f32);
    let dv = (tex.uv[3] - tex.uv[1]) / row.total_frames.max(1) as f32;
    let v0 = tex.uv[1] + dv * frame;
    let uv = [
        tex.uv[0],
        v0,
        tex.uv[2],
        v0 + (tex.uv[3] - tex.uv[1]) * (band_h / tex.h),
    ];
    let dir = if s.facing < 0.0 { -1.0 } else { 1.0 };
    let height = 42.0 + row.height_boost as f32;
    let cx = s.origin[0] + 20.0 + row.x_offset as f32 * dir;
    let cy = s.origin[1]
        + 10.0
        + height / 2.0
        + row.y_offset as f32
        + m.player_offset as f32
        + s.gfx_off_y;
    let pos = [cx - tex.w / 2.0, cy - frame_h / 2.0];
    Some(LayerSprite {
        part: PlayerPart::Mount,
        pos,
        size: [tex.w, band_h],
        uv: flip_uv(uv, dir < 0.0),
        color: [s.light[0], s.light[1], s.light[2], 1.0],
        rotation: 0.0,
        dye: 0,
    })
}

/// The per-layer dye assignment (`DrawData.shader`'s sources): the helmet
/// pieces take `cHead`, the composite torso/shoulder/arm armor pieces
/// `cBody`, the armor legs `cLegs` and the mount strips `cMount`
/// (`Mount.currentShader`). Skin/shirt/held-item pieces draw plain — the C#
/// skin passes `shader = 0` explicitly (`PlayerDrawLayers.cs:1504`).
fn apply_dye(layers: &mut [LayerSprite], s: &PlayerCompositeState) {
    for l in layers {
        l.dye = match l.part {
            PlayerPart::Helmet => s.c_head,
            PlayerPart::ArmorTorso
            | PlayerPart::BackShoulderArmor
            | PlayerPart::BackArmArmor
            | PlayerPart::FrontArmArmor
            | PlayerPart::FrontShoulderArmor => s.c_body,
            PlayerPart::Pants => s.c_legs,
            PlayerPart::Mount => s.c_mount,
            _ => 0,
        };
    }
}

/// The full player composition: body, then the held item in its
/// `weaponDrawOrder` slot, then the front arm — the
/// `LegacyPlayerRenderer.cs:185-255` sequence for the starter player. The
/// mount rides the vanilla layer order: `DrawPlayer_02_MountBehindPlayer`
/// (`PlayerDrawLayers.cs:211`, behind everything) and
/// `DrawPlayer_23_MountFront` (`:2912`, after the arms).
pub(crate) fn assemble_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    let body = body_layers(s, a);
    let arms = front_arm_layers(s, a);
    let item = held_item_layers(s);
    let mount_back = mount_layer(s, a, false);
    let mount_front = mount_layer(s, a, true);
    let mut out = Vec::with_capacity(body.len() + arms.len() + item.len() + 2);
    if let Some(back) = mount_back {
        out.push(back);
    }
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
    if let Some(front) = mount_front {
        out.push(front);
    }
    apply_dye(&mut out, s);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_the_frame_chain_can_emit_has_draw_tables() {
        // M2-DRAW-3 made the chain emit rows the old three-arm approximation
        // never produced (1, 2, 4, 6, 10, 11, 19 ...). Every consumer table is
        // keyed on the row, so walk all 20 `PlayerFrame` rows and pin the C#
        // values: the front-arm switch (`PlayerDrawSet.cs:1952-2023`), the
        // shoulder-over-arm clears at 1/2/5 (`:1961,1966,1976`), the torso
        // column-1 override at row 5 (`:1975`) and the headgear Y table
        // (`Main.cs:66638-66660`).
        for row in 0..=19u32 {
            let (col, line) = front_arm_cell(row);
            assert!(col >= 2 && col <= 6, "row {row} arm column {col}");
            assert!(line <= 1, "row {row} arm line {line}");
            let (bcol, bline) = back_arm_cell(row);
            assert_eq!((bcol, bline), (col, line + 2), "row {row} back arm");
            assert_eq!(
                shoulder_over_front_arm(row),
                !matches!(row, 1 | 2 | 5),
                "row {row} shoulder order"
            );
            assert_eq!(
                torso_cell(row),
                if row == 5 { (1, 0) } else { (0, 0) },
                "row {row} torso cell"
            );
            let expect_y = if matches!(row, 7..=9 | 14..=16) { 0.0 } else { 2.0 };
            assert_eq!(headgear_offset(row), expect_y, "row {row} headgear Y");
            assert_eq!(
                headgear_composite_shift(row),
                2.0 - expect_y,
                "row {row} composite shift"
            );
        }
    }

    #[test]
    fn the_composite_pivots_and_offsets_are_the_c_sharp_pairs() {
        // `GetCompositeOffset_BackArm` (`PlayerDrawLayers.cs:4082-4085`) and
        // `_FrontArm` (`:4087-4090`) applied to both the position and the
        // origin (`:1272-1277`, `:1296-1298`), so the arm anchor rides the
        // unrotated pivot — the port's `part_pos` adds `origin_c` to the
        // position on purpose.
        assert_eq!(composite_pivot(true, 1.0), [6.0, 2.0]);
        assert_eq!(composite_pivot(true, -1.0), [-6.0, 2.0]);
        assert_eq!(composite_pivot(false, 1.0), [-5.0, 0.0]);
        assert_eq!(composite_pivot(false, -1.0), [5.0, 0.0]);
    }

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
        // origin_c = (20-5, 28); pos = pivot - R*origin_c, plus the
        // column-7 nudge `(facing, +1)` (`PlayerDrawLayers.cs:3604-3608`).
        let (sin, cos) = 0.75f32.sin_cos();
        let expected = [
            115.0 - (15.0 * cos - 28.0 * sin) + 1.0,
            228.0 - (15.0 * sin + 28.0 * cos) + 1.0,
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
        // The enabled arm keeps its column-7 nudge even at rotation 0.
        assert_eq!(arm0.pos, [101.0, 201.0]);
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
        // Mirrored pivot (125) plus the `(-1, +1)` nudge of the enabled arm.
        let expected = [
            125.0 - (25.0 * cos - 28.0 * sin) - 1.0,
            228.0 - (25.0 * sin + 28.0 * cos) + 1.0,
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
            under_arm: Some(sheet()),
            arm_shirt: Some(sheet()),
            legs: Some(sheet()),
            pants: Some(sheet()),
            shoes: Some(sheet()),
            hair: Some(sheet()),
            armor_head: [None; 5],
            armor_body: [None; 5],
            armor_legs: [None; 5],
            mount_back: [Some(sheet()); 50],
            mount_front: [Some(sheet()); 50],
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
            aim: 0.0,
            item_uv: None,
            item_size: [16.0, 16.0],
            light: [1.0, 1.0, 1.0],
            head_armor: -1,
            body_armor: -1,
            leg_armor: -1,
            front_arm: CompositeArmState::DISABLED,
            back_arm: CompositeArmState::DISABLED,
            mount: None,
            c_head: 0,
            c_body: 0,
            c_legs: 0,
            c_mount: 0,
            sitting: false,
            seat_y_offset: 0,
            sleeping: false,
            full_rotation: 0.0,
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
    fn s1_g_15_the_back_arm_keeps_the_under_skin_layer_under_a_chest_armor() {
        // `PlayerDrawLayers.cs:1294-1302`: the `Players[5]` under-skin layer
        // is drawn on the ARMOR path too - the `flag2 && missingArm` block
        // paints it behind the arm skin, and `missingArm` is true for every
        // body id but 83 (`PlayerDrawSet.cs:377-384`). The pre-fix draw
        // skipped it under armor, so the armor's under-arm art showed through
        // where the vanilla paints the skin.
        let (a, s) = armored();
        let body = body_layers(&s, &a);
        let idx = |p: PlayerPart| body.iter().position(|sp| sp.part == p);
        assert!(parts(&body).contains(&PlayerPart::BackUnder));
        assert!(idx(PlayerPart::BackArmSkin) < idx(PlayerPart::BackUnder));
        assert!(idx(PlayerPart::BackUnder) < idx(PlayerPart::BackShoulderArmor));
        assert!(idx(PlayerPart::BackShoulderArmor) < idx(PlayerPart::BackArmArmor));
        // The under-shirt sleeve and the shirt sleeve stay armor-only skips
        // (`PlayerDrawLayers.cs:1355-1360`, the `!flag3` block).
        assert!(!parts(&body).contains(&PlayerPart::BackUnderArm));
        assert!(!parts(&body).contains(&PlayerPart::BackSleeve));
    }

    #[test]
    fn s1_g_15_the_front_shoulder_skin_drops_under_a_chest_armor() {
        // `DrawPlayer_28_ArmOverItemComposite`'s armor branch
        // (`PlayerDrawLayers.cs:3600-3661`) paints the skin arm at the ARM
        // frame (`:3611`) and the armor pieces; the shoulder-frame skin
        // (`Players[7]` at `compFrontShoulderFrame`) only exists in the
        // no-armor branch (`:3670`). The pre-fix draw kept it, so a bare
        // shoulder stripe showed through the armor.
        let (a, s) = armored();
        let arms = front_arm_layers(&s, &a);
        assert!(!parts(&arms).contains(&PlayerPart::FrontShoulderSkin));
        assert!(parts(&arms).contains(&PlayerPart::FrontArmSkin));
        let idx = |p: PlayerPart| arms.iter().position(|sp| sp.part == p);
        // Row 0 keeps `compShoulderOverFrontArm`: the C#'s `i` loop draws the
        // arm skin and then the armor arm (`i == num3 == 0`), and the armor
        // shoulder last (`i == num2 == 1`).
        assert!(idx(PlayerPart::FrontArmSkin) < idx(PlayerPart::FrontArmArmor));
        assert!(idx(PlayerPart::FrontArmArmor) < idx(PlayerPart::FrontShoulderArmor));
        // Row 1 clears it: the armor shoulder (`i == num2 == 0`) comes first.
        let s1 = PlayerCompositeState { body_frame: 1, ..s };
        let arms1 = front_arm_layers(&s1, &a);
        let idx1 = |p: PlayerPart| arms1.iter().position(|sp| sp.part == p);
        assert!(!parts(&arms1).contains(&PlayerPart::FrontShoulderSkin));
        assert!(idx1(PlayerPart::FrontShoulderArmor) < idx1(PlayerPart::FrontArmSkin));
        assert!(idx1(PlayerPart::FrontArmSkin) < idx1(PlayerPart::FrontArmArmor));
    }

    #[test]
    fn s1_g_15_the_shoulders_hide_on_the_falling_row_under_a_chest_armor() {
        // `PlayerDrawSet.cs:1933-1937` + `:1984-1987`: on body row 5 an armor
        // that does not show its shoulders while jumping raises
        // `hideCompositeShoulders`, and the vanilla then skips the shoulder
        // pieces (`PlayerDrawLayers.cs:1307` the back shoulder, `:3624` the
        // front one). None of the port's armor ids (1-4) is in
        // `showsShouldersWhileJumping` (`ID/ArmorIDs.cs:714`), so a worn
        // chest armor loses its shoulders on the falling row.
        assert!(hide_composite_shoulders(5, 1));
        assert!(!hide_composite_shoulders(0, 1));
        assert!(!hide_composite_shoulders(5, -1));
        let (a, s) = armored();
        let s5 = PlayerCompositeState { body_frame: 5, ..s };
        let body = body_layers(&s5, &a);
        assert!(!parts(&body).contains(&PlayerPart::BackShoulderArmor));
        assert!(parts(&body).contains(&PlayerPart::BackArmArmor));
        let arms = front_arm_layers(&s5, &a);
        assert!(!parts(&arms).contains(&PlayerPart::FrontShoulderArmor));
        assert!(parts(&arms).contains(&PlayerPart::FrontArmArmor));
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
                PlayerPart::BackUnderArm,
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
        // Frame 0 keeps the default: arm stack, then shoulder over it. Each
        // stack is skin, `Players[8]`, sleeve, shirt (`:3661-3706`).
        let s0 = state();
        assert_eq!(
            parts(&front_arm_layers(&s0, &a)),
            vec![
                PlayerPart::FrontArmSkin,
                PlayerPart::FrontArmUnder,
                PlayerPart::FrontSleeve,
                PlayerPart::FrontShirt,
                PlayerPart::FrontShoulderSkin,
                PlayerPart::FrontShoulderUnder,
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
            parts(&front_arm_layers(&s1, &a))[..4],
            [
                PlayerPart::FrontShoulderSkin,
                PlayerPart::FrontShoulderUnder,
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
    fn test_dye_follows_the_c_cache() {
        // `apply_dye` stamps the `DrawData.shader` sources: the helmet takes
        // cHead, the composite torso/shoulder/arm armor cBody, the armor legs
        // cLegs and the mount strips cMount; skin/shirt/held-item pieces stay
        // plain (the C# skin passes `shader = 0`, `PlayerDrawLayers.cs:1504`).
        let tex = || Some(sheet());
        let armor = [None, tex(), None, None, None];
        let a = PlayerAssets {
            armor_head: armor,
            armor_body: armor,
            armor_legs: armor,
            ..fixture()
        };
        let mut s = state();
        s.head_armor = 1;
        s.body_armor = 1;
        s.leg_armor = 1;
        s.mount = Some(MountDraw {
            id: 0,
            frame: 0,
            player_offset: 0,
            is_idle: false,
        });
        s.c_head = 7;
        s.c_body = 5;
        s.c_legs = 3;
        s.c_mount = 9;
        for l in assemble_layers(&s, &a) {
            let expected = match l.part {
                PlayerPart::Helmet => 7,
                PlayerPart::ArmorTorso
                | PlayerPart::BackShoulderArmor
                | PlayerPart::BackArmArmor
                | PlayerPart::FrontArmArmor
                | PlayerPart::FrontShoulderArmor => 5,
                PlayerPart::Pants => 3,
                PlayerPart::Mount => 9,
                _ => 0,
            };
            assert_eq!(l.dye, expected, "part {:?}", l.part);
        }
        // Zero cache → the plain path everywhere (DW-3.3's pixel identity).
        let mut plain = state();
        plain.head_armor = 1;
        plain.body_armor = 1;
        plain.leg_armor = 1;
        for l in assemble_layers(&plain, &a) {
            assert_eq!(l.dye, 0, "part {:?}", l.part);
        }
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

    #[test]
    fn mount_layers_order_behind_and_in_front() {
        // DW-5.1 — `PlayerDrawLayers.cs:211` (behind everything) and `:2912`
        // (after the arms).
        let a = fixture();
        let mut s = state();
        s.mount = Some(MountDraw {
            id: 10,
            frame: 3,
            player_offset: 30,
            is_idle: false,
        });
        let out = assemble_layers(&s, &a);
        assert_eq!(
            out.first().unwrap().part,
            PlayerPart::Mount,
            "back layer first"
        );
        // The Unicorn has no front strip (`Mount.cs:1469-1471` — only the
        // back texture), so exactly one mount sprite draws.
        assert_eq!(
            out.iter().filter(|sp| sp.part == PlayerPart::Mount).count(),
            1
        );
        // Rudolph draws the front strip over the arms (idle hides it).
        s.mount = Some(MountDraw {
            id: 0,
            frame: 0,
            player_offset: 30,
            is_idle: false,
        });
        let out = assemble_layers(&s, &a);
        assert_eq!(out.first().unwrap().part, PlayerPart::Mount, "back first");
        assert_eq!(out.last().unwrap().part, PlayerPart::Mount, "front last");
        s.mount = Some(MountDraw {
            id: 0,
            frame: 0,
            player_offset: 30,
            is_idle: true,
        });
        let out = assemble_layers(&s, &a);
        assert_eq!(
            out.iter().filter(|sp| sp.part == PlayerPart::Mount).count(),
            1,
            "the front hides while idle, the back stays first"
        );
        let s = state();
        assert!(
            assemble_layers(&s, &a)
                .iter()
                .all(|sp| sp.part != PlayerPart::Mount)
        );
    }

    #[test]
    fn mount_frame_math_matches_the_c_draw() {
        // DW-5.2 — `Mount.cs:5554-5561` position, `:5717-5722` the frame
        // band (`frameH = textureHeight / totalFrames`, `Height -= 2`), the
        // flip mirrors both the UV and the `xOffset * direction` offset.
        let a = fixture(); // sheet(): uv [0,0,1,1], 360x224
        let mut s = state(); // origin [100, 200], facing 1
        s.mount = Some(MountDraw {
            id: 10,
            frame: 3,
            player_offset: 30,
            is_idle: false,
        });
        let sprite = mount_layer(&s, &a, false).expect("the unicorn draws its back strip");
        // frameH = 224 / 16 = 14; the drawn band is 12 tall.
        assert_eq!(sprite.size, [360.0, 12.0]);
        // The band: v from 3/16 to 3/16 + 12/224.
        let dv = 1.0 / 16.0;
        assert!((sprite.uv[1] - dv * 3.0).abs() < 1e-6);
        assert!((sprite.uv[3] - (dv * 3.0 + 12.0 / 224.0)).abs() < 1e-6);
        // The center: origin.x + 20 + xOffset(5) * dir(1), origin.y + 10 +
        // height(76)/2 + yOffset(1) + PlayerOffset(30).
        let cx = 100.0 + 20.0 + 5.0;
        let cy = 200.0 + 10.0 + 38.0 + 1.0 + 30.0;
        assert!((sprite.pos[0] - (cx - 180.0)).abs() < 1e-5);
        assert!(
            (sprite.pos[1] - (cy - 7.0)).abs() < 1e-5,
            "the frame center anchors the quad"
        );
        // Facing away: the offset mirrors and the UV flips.
        s.facing = -1.0;
        let flipped = mount_layer(&s, &a, false).unwrap();
        assert!((flipped.pos[0] - (100.0 + 20.0 - 5.0 - 180.0)).abs() < 1e-5);
        assert_eq!(flipped.uv[0], sprite.uv[2], "the UV mirrors");
        // The empty-back rows (Pogo 43 draws front only) skip the back pass.
        let mut s = state();
        s.mount = Some(MountDraw {
            id: 43,
            frame: 0,
            player_offset: 14,
            is_idle: false,
        });
        assert!(mount_layer(&s, &a, false).is_none(), "no back sheet");
        assert!(mount_layer(&s, &a, true).is_some(), "the front strip draws");
    }

    #[test]
    fn the_seated_legs_draw_three_resliced_strips() {
        // DW-2.1 — `DrawSittingLegs` (`PlayerDrawLayers.cs:1545-1554`,
        // `:1672-1695`): each leg texture becomes three strips of the
        // standing cell — the 10-px bottom block (cell rows 46..56) fanned
        // +4 px along the facing over two 2-px knee slices (44..46 at +2,
        // 42..44 at +0) — dropped 42 px below the sheet anchor.
        let a = fixture();
        let mut s = state();
        s.sitting = true;
        s.seat_y_offset = -4;
        let base_y = s.origin[1] + s.gfx_off_y;
        for facing in [1.0, -1.0] {
            s.facing = facing;
            let out = body_layers(&s, &a);
            let pants: Vec<_> = out
                .iter()
                .filter(|sp| sp.part == PlayerPart::Pants)
                .collect();
            assert_eq!(pants.len(), 3, "three strips per leg texture");
            // Draw order: bottom block (num10 = 2) first, knee slices over.
            let expect = [
                (2.0 + 2.0 * 2.0, 10.0),
                (2.0 + 2.0 * 1.0, 2.0),
                (2.0 + 2.0 * 0.0, 2.0),
            ];
            for (sp, (x_fan, cell_h)) in pants.iter().zip(expect) {
                assert_eq!(sp.size, [40.0, cell_h]);
                assert!((sp.pos[0] - (s.origin[0] + x_fan * facing)).abs() < 1e-4);
                assert!((sp.pos[1] - (base_y + 42.0 + s.seat_y_offset as f32)).abs() < 1e-4);
            }
            // The v-spans track the cell sub-rects: the 10-px block spans
            // five times a 2-px slice, and the block sits lowest in the cell
            // (the largest v).
            let v = |sp: &LayerSprite| sp.uv[3] - sp.uv[1];
            assert!((v(pants[0]) - v(pants[1]) * 5.0).abs() < 1e-6);
            assert!(pants[0].uv[1] > pants[1].uv[1]);
            assert!(pants[1].uv[1] > pants[2].uv[1]);
        }
    }

    #[test]
    fn the_seat_drop_shifts_the_torso_sandwich_only() {
        // DW-2.2 — `PlayerDrawSet.cs:343-347` + the sandwich
        // (`LegacyPlayerRenderer.cs:183-225`): the torso skin, the back arm
        // and the head drop by `seatAdjustment`; the front arm does not.
        let a = fixture();
        let mut s = state();
        s.sitting = true;
        s.seat_y_offset = -4;
        let out = body_layers(&s, &a);
        let drop = |part: PlayerPart| -> f32 {
            let plain = body_layers(&state(), &a);
            let y = |v: &[LayerSprite]| v.iter().find(|sp| sp.part == part).unwrap().pos[1];
            y(&out) - y(&plain)
        };
        assert!((drop(PlayerPart::TorsoSkin) - (-4.0)).abs() < 1e-4);
        assert!((drop(PlayerPart::BackArmSkin) - (-4.0)).abs() < 1e-4);
        assert!((drop(PlayerPart::Head) - (-4.0)).abs() < 1e-4);
        // The front arm draws in `28_ArmOverItemComposite` — outside the
        // sandwich (`LegacyPlayerRenderer.cs:205-225` ends before it).
        let arm_y = |sitting: bool, seat: i32| {
            let mut st = state();
            st.sitting = sitting;
            st.seat_y_offset = seat;
            front_arm_layers(&st, &a)
                .iter()
                .find(|sp| sp.part == PlayerPart::FrontArmSkin)
                .unwrap()
                .pos[1]
        };
        assert!((arm_y(true, -4) - arm_y(false, 0)).abs() < 1e-4);
        // The bench's `seatDownOffset` (0, -1) gives `seatAdjustment` -5
        // (`PlayerSittingHelper.cs:23-24`): `-4 + (int)(-1)`.
        let mut s = state();
        s.sitting = true;
        s.seat_y_offset = -5;
        let out = body_layers(&s, &a);
        let torso = out
            .iter()
            .find(|sp| sp.part == PlayerPart::TorsoSkin)
            .unwrap();
        assert!((torso.pos[1] - (s.origin[1] + s.gfx_off_y - 5.0)).abs() < 1e-4);
    }

    #[test]
    fn the_bed_offset_table_is_verbatim() {
        // DW-2.3 — `SetOffsetbyBed` (`PlayerSleepingHelper.cs:226-279`).
        assert_eq!(bed_visual_offset(0), [-9.0, 1.0], "the default arm");
        assert_eq!(bed_visual_offset(8), [-11.0, 1.0]);
        assert_eq!(bed_visual_offset(13), [-11.0, -3.0]);
        assert_eq!(bed_visual_offset(16), [-7.0, -3.0]);
        assert_eq!(bed_visual_offset(19), [-3.0, -1.0]);
        assert_eq!(bed_visual_offset(21), [-9.0, 5.0]);
        assert_eq!(bed_visual_offset(24), [-7.0, 1.0]);
        assert_eq!(bed_visual_offset(36), [-13.0, 1.0]);
        assert_eq!(bed_visual_offset(38), [-11.0, -3.0]);
        assert_eq!(bed_visual_offset(999), [-9.0, 1.0], "unlisted → default");
    }

    #[test]
    fn the_sleep_rotation_spins_the_set_around_the_pivot() {
        // DW-2.3 — `DrawPlayer_TransformDrawData` (`PlayerDrawLayers.cs:
        // 4092-4119`): every sprite's position rotates around the pivot and
        // its own rotation gains the full rotation.
        let mut sprites = vec![LayerSprite {
            part: PlayerPart::TorsoSkin,
            pos: [110.0, 210.0],
            size: [40.0, 56.0],
            uv: [0.0, 0.0, 1.0, 1.0],
            color: [1.0; 4],
            rotation: 0.0,
            dye: 0,
        }];
        let pivot = [100.0, 200.0];
        let rot = std::f32::consts::FRAC_PI_2;
        apply_player_rotation(&mut sprites, pivot, rot);
        let sp = &sprites[0];
        // (10, 10) rotated 90° → (-10, 10) around the pivot.
        assert!((sp.pos[0] - (pivot[0] - 10.0)).abs() < 1e-4);
        assert!((sp.pos[1] - (pivot[1] + 10.0)).abs() < 1e-4);
        assert!((sp.rotation - rot).abs() < 1e-6);
        // The zero rotation pass is a no-op (the standing player).
        apply_player_rotation(&mut sprites, pivot, 0.0);
        assert!((sprites[0].rotation - rot).abs() < 1e-6);
    }

    #[test]
    fn the_hair_frame_is_the_body_frame_shifted_six_rows() {
        // `PlayerDrawSet.cs:1766-1786`: `hairFrame = bodyFrame; Height -= 2;
        // Y -= 336` — the `Player_Hair_*` strips are 14 rows (784 px), so body
        // rows 0..6 share hair row 0 and row 19 lands on row 13. Drawing the
        // body row directly (the pre-fix draw) walked the hair out of its own
        // strip on every pose but standing.
        assert_eq!(hair_row(0), 0);
        assert_eq!(hair_row(5), 0);
        assert_eq!(hair_row(6), 0);
        assert_eq!(hair_row(7), 1);
        assert_eq!(hair_row(10), 4);
        assert_eq!(hair_row(19), 13);
        // The hair strip itself: 40x784 = 14 frames of 56.
        let hair = BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 40.0,
            h: 784.0,
        };
        let uv = hair_uv(&hair, 19);
        // Row 13 of 14, cropped 2 px of its 56 (`Height -= 2`).
        assert!((uv[1] - 13.0 / 14.0).abs() < 1e-6);
        assert!((uv[3] - (13.0 / 14.0 + 54.0 / 784.0)).abs() < 1e-6);
        // The drawn hair sprite keeps the 40x54 art box.
        let layers = body_layers(&state(), &fixture());
        let h = layers.iter().find(|s| s.part == PlayerPart::Hair).unwrap();
        assert_eq!(h.size, [40.0, 54.0]);
    }

    #[test]
    fn the_last_player_part_row_is_reachable() {
        // `Player_0_0`/`_0_1`/`_0_2` are 1118 px = 19 full rows + a 54-px row
        // 19. A `floor(h/56)` row count clamped the walk cycle's fastest band
        // (row 19) back to row 18, so the head lagged the legs by one frame.
        let head = BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 40.0,
            h: 1118.0,
        };
        let uv18 = player_frame_uv(&head, 18);
        let uv19 = player_frame_uv(&head, 19);
        assert!((uv18[1] - 18.0 * 56.0 / 1118.0).abs() < 1e-6);
        assert!((uv19[1] - 19.0 * 56.0 / 1118.0).abs() < 1e-6);
        // The quad's bottom edge never samples past the sheet (the atlas
        // neighbour bleed at the chin).
        assert!((uv19[3] - 1.0).abs() < 1e-6);
        // A well-formed 20x56 strip still reaches row 19 exactly.
        let strip = BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 40.0,
            h: 1120.0,
        };
        assert!((player_frame_uv(&strip, 19)[1] - 19.0 / 20.0).abs() < 1e-6);
        assert!((player_frame_uv(&strip, 19)[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_torso_column_one_override_is_reachable() {
        // `PlayerDrawSet.cs:1940,1975`: the row-5 torso cell is column 1. The
        // sheet is 9x4 but the port crops it to the columns the draw can
        // reach — one column made the override collapse onto column 0.
        let sheet = BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 80.0,
            h: 224.0,
        };
        let cell = torso_cell(5);
        assert_eq!(cell, (1, 0));
        let uv = player_cell_uv(&sheet, cell.0, cell.1);
        assert!((uv[0] - 0.5).abs() < 1e-6, "column 1 of 2: {uv:?}");
        assert!((uv[2] - 1.0).abs() < 1e-6);
        assert_eq!(torso_cell(4), (0, 0));
    }

    #[test]
    fn the_enabled_front_arm_carries_the_one_pixel_nudge() {
        // `PlayerDrawLayers.cs:3604-3608`: an enabled composite front arm
        // (column 7) shifts the arm stack one pixel along the facing and one
        // down; the shoulder keeps its anchor. The API exposes the nudge
        // through the arm sprite positions only.
        let a = fixture();
        let mut s = state();
        s.front_arm = CompositeArmState {
            enabled: true,
            stretch: CompositeStretch::Full,
            rotation: 0.0,
        };
        let layers = front_arm_layers(&s, &a);
        let arm = layers
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        let shoulder = layers
            .iter()
            .find(|l| l.part == PlayerPart::FrontShoulderSkin)
            .unwrap();
        // Row 0 carries no head-gear shift: the arm sits on `origin` plus the
        // nudge, the shoulder on `origin` itself.
        assert!((arm.pos[0] - 101.0).abs() < 1e-6, "{:?}", arm.pos);
        assert!((arm.pos[1] - 201.0).abs() < 1e-6, "{:?}", arm.pos);
        assert!(
            (shoulder.pos[0] - 100.0).abs() < 1e-6,
            "no nudge on the shoulder"
        );
        assert!(
            (shoulder.pos[1] - 200.0).abs() < 1e-6,
            "no nudge on the shoulder"
        );
        // Facing left mirrors the nudge, the Y stays down.
        let mut sl = state();
        sl.facing = -1.0;
        sl.front_arm = s.front_arm;
        let left = front_arm_layers(&sl, &a);
        let arm = left
            .iter()
            .find(|l| l.part == PlayerPart::FrontArmSkin)
            .unwrap();
        assert!((arm.pos[0] - 99.0).abs() < 1e-6, "{:?}", arm.pos);
        assert!((arm.pos[1] - 201.0).abs() < 1e-6, "{:?}", arm.pos);
    }
}
