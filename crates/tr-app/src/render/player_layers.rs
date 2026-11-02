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
    /// `Player.itemAnimation` stand-in: the swing timer, 0 = idle.
    pub(crate) swing: f32,
    /// Vanilla `useStyle` stand-in: 1 = swing (default), 8 = thrust, 9 = bow.
    pub(crate) use_style: u8,
    /// The held item's atlas quad and frame size, `None` = empty hand.
    pub(crate) item_uv: Option<[f32; 4]>,
    pub(crate) item_size: [f32; 2],
    /// The light tint at the player tile (`style.shade(...)` output).
    pub(crate) light: [f32; 3],
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
fn part_sprite(
    part: PlayerPart,
    uv: [f32; 4],
    s: &PlayerCompositeState,
    color: [f32; 4],
    y_shift: f32,
) -> LayerSprite {
    LayerSprite {
        part,
        pos: [s.origin[0], s.origin[1] + s.gfx_off_y + y_shift],
        size: [40.0, 56.0],
        uv,
        color,
        rotation: 0.0,
    }
}

/// One body part, skipped when its sheet is missing (`Main.spriteBatch.Draw`
/// ignores null textures — DW-2.5's dirty case).
fn push_part(
    out: &mut Vec<LayerSprite>,
    part: PlayerPart,
    tex: Option<BgTex>,
    uv: impl Fn(&BgTex) -> [f32; 4],
    s: &PlayerCompositeState,
    color: [f32; 4],
    shift: f32,
) {
    if let Some(tex) = tex {
        out.push(part_sprite(part, uv(&tex), s, color, shift));
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
        s,
        tint(s.light, SKIN),
        shift,
    );
    push_part(
        &mut out,
        PlayerPart::LegsSkin,
        a.legs,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        s,
        tint(s.light, SKIN),
        0.0,
    );
    let (bc, br) = back_arm_cell(s.body_frame);
    push_part(
        &mut out,
        PlayerPart::BackArmSkin,
        a.arm,
        |t| flip_uv(player_cell_uv(t, bc, br), flip),
        s,
        tint(s.light, SKIN),
        shift,
    );
    // `DrawPlayer_12_SkinComposite_BackArmShirt`: under-skin arm layer (5) and
    // shirt sleeve (13); the undershirt-tinted `Players[8]` pass is skipped —
    // the starter skin has no underShirt colour (M3-EQUIP palette).
    push_part(
        &mut out,
        PlayerPart::BackUnder,
        a.undershirt,
        |t| flip_uv(player_cell_uv(t, bc, br), flip),
        s,
        tint(s.light, SKIN),
        shift,
    );
    push_part(
        &mut out,
        PlayerPart::BackSleeve,
        a.arm_shirt,
        |t| flip_uv(player_cell_uv(t, bc, br), flip),
        s,
        tint(s.light, SHIRT),
        shift,
    );
    // 13_Leggings / 14_Shoes ride the leg frame without the shift.
    push_part(
        &mut out,
        PlayerPart::Pants,
        a.pants,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        s,
        tint(s.light, PANTS),
        0.0,
    );
    push_part(
        &mut out,
        PlayerPart::Shoes,
        a.shoes,
        |t| flip_uv(player_frame_uv(t, s.leg_frame), flip),
        s,
        tint(s.light, SHOES),
        0.0,
    );
    // 17_TorsoComposite: the shirt over the (already drawn) skin torso.
    push_part(
        &mut out,
        PlayerPart::Shirt,
        a.shirt,
        |t| flip_uv(player_cell_uv(t, torso_cell(s.body_frame).0, torso_cell(s.body_frame).1), flip),
        s,
        tint(s.light, SHIRT),
        shift,
    );
    // 21_Head: head, eye whites, eyes, hair.
    push_part(
        &mut out,
        PlayerPart::Head,
        a.head,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        s,
        tint(s.light, SKIN),
        0.0,
    );
    push_part(
        &mut out,
        PlayerPart::EyeWhites,
        a.eye_whites,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        s,
        tint(s.light, EYE_WHITE),
        0.0,
    );
    push_part(
        &mut out,
        PlayerPart::Eyes,
        a.eyes,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        s,
        tint(s.light, EYES),
        0.0,
    );
    push_part(
        &mut out,
        PlayerPart::Hair,
        a.hair,
        |t| flip_uv(player_frame_uv(t, s.body_frame), flip),
        s,
        tint(s.light, HAIR),
        0.0,
    );
    out
}

/// One arm composite pass: skin(7) + sleeve(13) + shirt(6) at one cell
/// (`PlayerDrawLayers.cs:3661-3706`, `Players[8]` skipped — no starter colour).
fn arm_stack(s: &PlayerCompositeState, a: &PlayerAssets, base: PlayerPart, cell: (u32, u32)) -> Vec<LayerSprite> {
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
            pos: [s.origin[0], s.origin[1] + shift],
            size: [40.0, 56.0],
            uv: flip_uv(player_cell_uv(&tex, cell.0, cell.1), flip),
            color,
            rotation: 0.0,
        });
    }
    out
}

/// The front-arm layer of `DrawPlayer_28_ArmOverItemComposite`'s no-armor
/// branch (`PlayerDrawLayers.cs:3661-3706`): the arm and shoulder stacks, arm
/// first when `compShoulderOverFrontArm` (default true), shoulder first
/// otherwise.
pub(crate) fn front_arm_layers(s: &PlayerCompositeState, a: &PlayerAssets) -> Vec<LayerSprite> {
    let arm = arm_stack(s, a, PlayerPart::FrontArmSkin, front_arm_cell(s.body_frame));
    let shoulder = arm_stack(s, a, PlayerPart::FrontShoulderSkin, FRONT_SHOULDER_CELL);
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
    let (num_x, loc_y) = if remaining >= 0.666 {
        // First third (`Player.cs:50709-50734`): X `10 / >32:14 / >=52:24 /
        // >=64:28 / >=92:38`, Y `position.Y + 24`.
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

/// `DrawPlayer_27_HeldItem` (`PlayerDrawLayers.cs:3043-3195`): the held item
/// pivots on `itemLocation` — the useStyle-1 thirds from `Player.ItemCheck`
/// (`Player.cs:50709-50846`) — with rotation `itemRotation`
/// (`Player.cs:50848`), plus `gfxOffY` (`PlayerDrawSet.cs:368`). The C# draw
/// origin `(width*0.5 - width*0.5*direction, height)` (`:3161`) plus the
/// `direction < 0` horizontal flip puts the pivot on the *drawn* sprite's
/// bottom-left corner in both directions; useStyle 8 thrusts with a
/// quarter-turn near-top grip (`:3181-3188`) and useStyle 9 holds a bow at
/// `(0.5, 0.4)` of the frame (`:3162-3174`). The empty hand returns no sprite.
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
    // `itemAnimation / itemAnimationMax` with the countdown running: `swing`
    // holds the remaining seconds of the use. `SWING_TIME` stands in for the
    // per-item `useAnimation` (catalogued deviation; real per-item timings are
    // the M2-ANIM-item-timings catalog row).
    let remaining = (s.swing / crate::item::SWING_TIME).clamp(0.0, 1.0);
    let mut rotation = item_rotation(remaining, dir);
    // `:3161`: `(width*0.5 - width*0.5*direction, height)`.
    let mut origin = [w * 0.5 - w * 0.5 * dir, h];
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
    // The flip mirrors the texture about the source-rect centre, so the pivot
    // offset maps into drawn space as `(width - origin.x, origin.y)` — for the
    // default origin that is the bottom-left corner either way.
    let mut origin_drawn = origin;
    if dir < 0.0 {
        origin_drawn[0] = w - origin[0];
    }
    // `drawinfo.ItemLocation = Position + (itemLocation - position)`
    // (`PlayerDrawSet.cs:368`) and `Position` carries `gfxOffY`
    // (`LegacyPlayerRenderer.cs:457,468`); `position.X + width/2` is
    // `origin.x + 20`, `position.Y` is `origin.y + 10`.
    let loc = item_location(
        remaining,
        dir,
        s.origin[0] + 20.0,
        s.origin[1] + 10.0,
        w,
        h,
    );
    let anchor = [loc[0], loc[1] + s.gfx_off_y];
    let (sin, cos) = rotation.sin_cos();
    let top_left = [
        anchor[0] - (origin_drawn[0] * cos - origin_drawn[1] * sin),
        anchor[1] - (origin_drawn[0] * sin + origin_drawn[1] * cos),
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
            gfx_off_y: 0.0,
            leg_frame: 0,
            body_frame: 0,
            facing: 1.0,
            swing: 0.0,
            use_style: 1,
            item_uv: None,
            item_size: [16.0, 16.0],
            light: [1.0, 1.0, 1.0],
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
        // Default useStyle 1 at the swing start (`swing = SWING_TIME`,
        // `remaining = 1`): `itemRotation = (1 - 0.5) * (-1) * 3.5 - 0.3 =
        // -2.05` (`Player.cs:50848`); `itemLocation` first third for a 16x16
        // frame = `position + (8 - 10, 24)` = (100+20-2, 200+10+24) =
        // (118, 234) (`Player.cs:50709-50734`); grip origin `(0, 16)`
        // (`PlayerDrawLayers.cs:3161`).
        let mut s = state();
        s.swing = crate::item::SWING_TIME;
        s.item_uv = Some([0.0, 0.0, 1.0, 1.0]);
        let rotation = -2.05f32;
        let (sin, cos) = rotation.sin_cos();
        let expected = [118.0 + 16.0 * sin, 234.0 - 16.0 * cos];
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
        let expected8 = [118.0 - (2.0 * cos8 - 2.0 * sin8), 234.0 - (2.0 * sin8 + 2.0 * cos8)];
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
        assert!(iteml.uv[0] > iteml.uv[2], "mirrored item quad {:?}", iteml.uv);

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
