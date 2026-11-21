//! Player physics and collision, modelled on `Player.cs`.
//!
//! Core movement: gravity, acceleration/friction, jump, 1-tile auto-step, axis-separated
//! AABB collision against solid tiles, simple water buoyancy and ground-slope support
//! (slopes 1/2) and half bricks/ceiling slopes during movement. Survival follows
//! `Player.cs`: a 200-tick breath meter (1 unit every 7
//! ticks underwater, 2 damage once empty), fall damage over 25 tiles, lava damage and
//! out-of-combat health regeneration. Half bricks and ceiling slopes still count as full
//! cells.

use crate::registry;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};

use crate::buffs::{BuffSlots, MAX_BUFFS};
use crate::collision;
use crate::dye;
use crate::item::{INV, ItemSlot};
use crate::luck::LuckState;
use crate::minecart;
use crate::npc::Npc;
use crate::projectile::{BABY_BIRD, PROJ_COUNT, ProjectilePool};
use crate::regen::{DotDeath, RegenInputs, RegenState, update_life_regen, update_mana_regen};
use glam::Vec2;

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
/// `Player.jumpSpeed` / `Player.jumpHeight`: holding the jump key keeps the
/// upward velocity at `jumpSpeed` for `jumpHeight` ticks, so tap and hold jumps
/// reach different heights.
const JUMP_SPEED: f32 = 5.01;

/// The base minecart's mount data (`Mount.SetAsMinecart` via the `mounts[6]`
/// registration, `Mount.cs:918-931`): `runSpeed = dashSpeed = 13`,
/// `acceleration = 0.04`, `jumpHeight = 15`, `jumpSpeed = 5.15` — the C#
/// `JumpHeight`/`JumpSpeed` cases (`Mount.cs:2810-2858`) list no cart, so the
/// stats are fixed (no |vx| scaling).
const CART_RUN_SPEED: f32 = 13.0;
const CART_DASH_SPEED: f32 = 13.0;
const CART_ACCELERATION: f32 = 0.04;
const CART_JUMP_HEIGHT: i32 = 15;
const CART_JUMP_SPEED: f32 = 5.15;
/// `mountData.buff` for the minecart (`Mount.cs:920`) — `AddBuff(buff, 3600)`
/// on mount (`Mount.cs:6226`), cleared on dismount (`:6151`).
const CART_MOUNT_BUFF: i32 = 118;
/// The mounted body height: `42 + heightBoost` (`SetMount`, `Mount.cs:6270`;
/// `heightBoost = 10`, `Mount.cs:923`). The width stays 20.
const CART_MOUNT_HEIGHT: f32 = 52.0;
/// `Player.breathMax`; `breath` is in ticks.
pub const BREATH_MAX: i32 = 200;
/// `Player.breathCDMax`: ticks between breath decrements.
const BREATH_CD: i32 = 7;

/// The extra-jump options (`Player.hasJumpOption_*` / `canJumpAgain_*` /
/// `isPerformingJump_*`, `Player.cs:2664-2716`). The mount jumps
/// (Unicorn/Santank/WallOfFleshGoat/Basilisk) arm from the mounts
/// (`Mount.cs:4884-4968`), the bottles from the accessories
/// (`Player.cs:14787-15003`) — both out until M3-EQUIP-3/M2-MOUNT-1, so
/// `TR_JUMPS=` pins them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExtraJump {
    Cloud,
    Sandstorm,
    Blizzard,
    Fart,
    Sail,
    Unicorn,
    Santank,
    WallOfFleshGoat,
    Basilisk,
}

impl ExtraJump {
    /// The scan index into the flag arrays.
    pub(crate) fn index(self) -> usize {
        match self {
            ExtraJump::Cloud => 0,
            ExtraJump::Sandstorm => 1,
            ExtraJump::Blizzard => 2,
            ExtraJump::Fart => 3,
            ExtraJump::Sail => 4,
            ExtraJump::Unicorn => 5,
            ExtraJump::Santank => 6,
            ExtraJump::WallOfFleshGoat => 7,
            ExtraJump::Basilisk => 8,
        }
    }

    /// The consumption priority (`Player.cs:20698-20748`): the mount jumps
    /// first, then Sandstorm, Blizzard, Fart, Sail, Cloud as the
    /// unconditional fall-through.
    pub(crate) const PRIORITY: [ExtraJump; 9] = [
        ExtraJump::Basilisk,
        ExtraJump::WallOfFleshGoat,
        ExtraJump::Santank,
        ExtraJump::Unicorn,
        ExtraJump::Sandstorm,
        ExtraJump::Blizzard,
        ExtraJump::Fart,
        ExtraJump::Sail,
        ExtraJump::Cloud,
    ];

    /// The hold-boost counter the jump grants, in `jumpHeight` multiples
    /// (`Player.cs:20787-21006`).
    pub(crate) fn hold_multiplier(self) -> f32 {
        match self {
            ExtraJump::Sandstorm => 3.0,
            ExtraJump::Blizzard => 1.5,
            ExtraJump::Fart => 2.0,
            ExtraJump::Sail => 1.25,
            ExtraJump::Unicorn => 2.0,
            ExtraJump::Santank => 2.0,
            ExtraJump::WallOfFleshGoat => 2.0,
            ExtraJump::Basilisk => 0.75,
            ExtraJump::Cloud => 0.75,
        }
    }

    /// The `TR_JUMPS=` name.
    pub(crate) fn parse_name(name: &str) -> Option<Self> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "cloud" => ExtraJump::Cloud,
            "sandstorm" => ExtraJump::Sandstorm,
            "blizzard" => ExtraJump::Blizzard,
            "fart" => ExtraJump::Fart,
            "sail" | "tsunami" => ExtraJump::Sail,
            "unicorn" => ExtraJump::Unicorn,
            "santank" => ExtraJump::Santank,
            "goat" | "walloffleshgoat" => ExtraJump::WallOfFleshGoat,
            "basilisk" => ExtraJump::Basilisk,
            _ => return None,
        })
    }
}

#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub down: bool,
    /// `Player.controlUp` (`Main.cUp`): the up key. The vanilla folds it into
    /// `controlJump` (`Player.cs:24905`), but `GrappleMovement`'s jump-off gate
    /// (`:22717`) and `GetGrapplingForces`' steering (`:22775`/`:22792`) read
    /// it separately, so the port carries it.
    pub up: bool,
    /// `Player.controlHook` (the quick-grapple key).
    pub hook: bool,
    /// `player.itemAnimation > 0` at the movement pass — the swing-state
    /// half of the `useTurn` direction lock (`Player.cs:19837`). The
    /// world-sim fills it from `app.swing`; the plain `update` harness
    /// leaves it false (idle).
    pub item_in_use: bool,
    /// `inventory[selectedItem].useTurn` — the other half of the
    /// `Player.cs:19837` lock. False for an empty hand / unknown kind.
    pub held_use_turn: bool,
}

/// The DoT-derived half of [`RegenInputs`] for one tick — what the held buffs
/// say about the drain (`Player.cs:18949-19385`). `update_survival` copies the
/// fields into the engine's inputs; the engine (regen.rs) owns the lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DotDerivation {
    /// The summed DoT delta (honey mitigation included) → `RegenInputs::dot_regen`.
    regen: i32,
    /// Any DoT (or bleeding) held → `RegenInputs::dot_resets_time`.
    resets_time: bool,
    /// `burned || suffocating || (tongued && expert)` (`Player.cs:19313`).
    fast_lane: bool,
    /// The fast lane's kill reason (7 → 12 → 8, `Player.cs:19321-19331`).
    death_fast: DotDeath,
    /// The generic lane's kill reason (22 → 9 → 10 → 23 → 8, `:19362-19385`).
    death_generic: DotDeath,
}

/// The `WetCollision` movement speed per liquid (`Player.cs:28168-28171`):
/// water 0.5, lava 0.5, honey 0.25, shimmer 0.375, dry 1.0.
pub fn movement_speed_for(wet: bool, lava_wet: bool, honey_wet: bool, shimmer_wet: bool) -> f32 {
    if honey_wet {
        0.25
    } else if shimmer_wet {
        0.375
    } else if wet || lava_wet {
        0.5
    } else {
        1.0
    }
}

/// The wet-family flags the statics rebase reads (`Player.cs:24380-24437`).
/// The not-yet-ported writers default `false` — the C#'s pre-condition
/// state: `shimmering` (the absorbed state, the shimmer-emergence work),
/// `merman` (the buff table, S1-E-7), `trident` (`Player.cs:12896`'s use
/// arm), `control_up` (no input channel yet), `down_dash`
/// (`isPerformingJump_DownDash` — only the Dead Cells down-dash sets it,
/// `Player.cs:20697-20722`), `vortex_debuff` (the Moon Lord debuff) and
/// `portal_physics` (no portals).
#[derive(Clone, Copy, Debug, Default)]
pub struct WetStaticsFlags {
    pub wet: bool,
    pub honey_wet: bool,
    pub lava_wet: bool,
    pub shimmer_wet: bool,
    pub shimmering: bool,
    pub merman: bool,
    pub trident: bool,
    pub control_up: bool,
    pub down_dash: bool,
    pub vortex_debuff: bool,
    pub portal_physics: bool,
}

/// `Player.cs:24380-24437` — the per-tick statics rebase that runs right
/// after the movement defaults (`:24364-24369` resets `maxFallSpeed = 10f`,
/// `gravity = Player.defaultGravity`, `jumpHeight = 15`, `jumpSpeed =
/// 5.01f`): the wet/shimmer arms assign the tick's `gravity`/`maxFallSpeed`
/// and the jump statics (`Player.jumpHeight`/`jumpSpeed` are statics in the
/// C#, instance fields here) BEFORE any integration, then `vortexDebuff`
/// zeroes the gravity and every tick adds the `+0.01f` fall-cap creep. The
/// plain integration arm (`Player.cs:27352-27359`) consumes the rebased
/// values at FULL strength — the `gravity / 3` / `maxFallSpeed / 3` sites
/// (`:27008`, `:27339`) are the mount/wings flight integrators, not a wet
/// rule. Returns `(gravity, max_fall_speed, jump_height, jump_speed)`.
pub fn wet_gravity_and_cap(
    gravity: f32,
    max_fall_speed: f32,
    jump_height: i32,
    jump_speed: f32,
    flags: WetStaticsFlags,
) -> (f32, f32, i32, f32) {
    let WetStaticsFlags {
        wet,
        honey_wet,
        lava_wet,
        shimmer_wet,
        shimmering,
        merman,
        trident,
        control_up,
        down_dash,
        vortex_debuff,
        portal_physics,
    } = flags;
    let mut gravity = gravity;
    let mut max_fall_speed = max_fall_speed;
    let mut jump_height = jump_height;
    let mut jump_speed = jump_speed;
    if portal_physics {
        max_fall_speed = 35.0; // `:24380-24382`
    }
    if !shimmering && wet && down_dash {
        gravity *= 0.85; // `:24384-24387`
        max_fall_speed *= 0.85;
    } else if shimmer_wet || shimmering {
        if shimmering {
            gravity *= 0.9; // `:24389-24394`
            max_fall_speed *= 0.9;
        } else {
            gravity = 0.15;
            jump_height = 23;
            jump_speed = 5.51;
        }
    } else if wet {
        if honey_wet {
            gravity = 0.1; // `:24399-24402`
            max_fall_speed = 3.0;
        } else if merman {
            gravity = 0.3; // `:24404-24407`
            max_fall_speed = 7.0;
        } else if trident && !lava_wet {
            gravity = 0.25; // `:24409-24417`
            max_fall_speed = 6.0;
            jump_height = 25;
            jump_speed = 5.51;
            if control_up {
                gravity = 0.1;
                max_fall_speed = 2.0;
            }
        } else {
            gravity = 0.2; // `:24418-24424`
            max_fall_speed = 5.0;
            jump_height = 30;
            jump_speed = 6.01;
        }
    }
    if vortex_debuff {
        gravity = 0.0; // `:24433-24436`
    }
    max_fall_speed += 0.01; // `:24437`
    (gravity, max_fall_speed, jump_height, jump_speed)
}

/// `Main.OffsetsPlayerHeadgear[i].Y` (`Main.cs:66638-66660`): the per-body-row
/// headgear lift the draw and the Moon Lord leg rows read. Rows 7-9 and 14-16
/// are the `0` entries.
pub fn offsets_player_headgear_y(row: usize) -> f32 {
    match row.min(19) {
        7..=9 | 14..=16 => 0.0,
        _ => 2.0,
    }
}

/// `Player.cs:47371-47378`: the shooting styles' `itemRotation` —
/// `atan2(aimY * direction, aimX * direction) - fullRotation`. The port keeps
/// the aim as an angle, so the mirrored `atan2` is the angle + PI when the
/// player faces left.
pub fn item_rotation_for_aim(aim: f32, facing: i32, full_rotation: f32) -> f32 {
    let mirrored = if facing < 0 {
        std::f32::consts::PI
    } else {
        0.0
    };
    aim + mirrored - full_rotation
}

/// `Player.GetAdjustedItemScale` (`Player.cs:42566-42574`): `item.scale`,
/// times 1.1 while `meleeScaleGlove` and the item is melee
/// (`ApplyMeleeScale`, `:42576-42582`). The port carries no
/// `meleeScaleGlove` granter (the Titan Glove's scale half is a documented
/// skip, ANALYSIS "The combat accessory dedupe/wiring"), so the glove arm
/// folds to the plain prefixed scale. S1-P-i05: the thrust extension
/// (`:51043`) and the stab hitbox (`:45128-45130`) both read it.
pub fn adjusted_item_scale(scale: f32) -> f32 {
    scale
}

/// `ItemCheck_ApplyUseStyle`'s useStyle-3 (Thrust) arm
/// (`Player.cs:51031-51063`): the held item's `itemLocation` +
/// `itemRotation`. `remaining` is `itemAnimation / itemAnimationMax`
/// (1 at the use's start, 0 at its end), `frame_w` the held item frame's
/// width (`Item.GetDrawHitbox`'s Width), `scale` the
/// [`adjusted_item_scale`] value, `mount_offset` the C#'s `mountOffset`
/// (the port's `HeightOffsetHitboxCenter` stands at 0). `gravDir` is
/// always 1 in the port, so the `:51059-51063` flip folds away.
///
/// The wind-up third (`itemAnimation > itemAnimationMax * 0.666`,
/// `:51033`) hides the item offscreen (`-1000, -1000`) with the cocked
/// rotation `-1.3 * direction`. The stab arm (`:51041-51057`) extends
/// the item forward as the animation runs out: `num14` shrinks with the
/// falling `itemAnimation`, so `itemLocation.X -= num14` starts the
/// item at the hand and pushes it FORWARD through the stab. The C#'s
/// snap clamps (`:51044-51051`) pin `num14` to -8 (facing left) / 8
/// (facing right) once it crosses -4 / 4 — a SNAP, not a bound — which
/// holds the full extension through the animation's last quarter. The
/// two umbrellas (946 / 4707 — the only useStyle-3 items in 1.4.5.8,
/// `Item.cs:12320-12332` / `:38683-38695`) take the extra
/// `itemLocation.X -= 6 * direction` nudge (`:51054-51057`).
pub fn thrust_item_placement(
    pos_x: f32,
    pos_y: f32,
    width: f32,
    dir: f32,
    remaining: f32,
    frame_w: f32,
    scale: f32,
    mount_offset: f32,
    item_type: i32,
) -> ([f32; 2], f32) {
    // `:51033`: `(double)itemAnimation > (double)itemAnimationMax * 0.666`
    // — the ratio compare against the C#'s 0.666 literal, NOT 2/3.
    if remaining > 0.666 {
        // `:51035-51037`.
        return ([-1000.0, -1000.0], -1.3 * dir);
    }
    // `:51041-51042`.
    let mut loc_x = pos_x + width * 0.5 + (frame_w * 0.5 - 4.0) * dir;
    let loc_y = pos_y + 24.0 + mount_offset;
    // `:51043`: the extension term — the C# multiplies left to right:
    // `ratio * frame.W * direction * GetAdjustedItemScale(sItem) * 1.2`
    // then subtracts `10 * direction`.
    let mut num14 = remaining * frame_w * dir * adjusted_item_scale(scale) * 1.2
        - 10.0 * dir;
    // `:51044-51051`: the snap clamps (one-sided per facing).
    if num14 > -4.0 && dir == -1.0 {
        num14 = -8.0;
    }
    if num14 < 4.0 && dir == 1.0 {
        num14 = 8.0;
    }
    // `:51052-51053`.
    loc_x -= num14;
    let rotation = 0.8 * dir;
    // `:51054-51057`: the umbrella pair's nudge.
    if item_type == 946 || item_type == 4707 {
        loc_x -= 6.0 * dir;
    }
    ([loc_x, loc_y], rotation)
}

/// `ItemCheck_GetMeleeHitbox`'s useStyle-3 (Thrust) arm
/// (`Player.cs:45099-45189`): the stab's damage rectangle out of the
/// CURRENT `itemLocation` (the [`thrust_item_placement`] output).
/// `None` = the C#'s `dontAttack` — the wind-up third
/// (`itemAnimation > itemAnimationMax * 0.666`, `:45168-45172`) never
/// attacks. The rect is the held frame sized by the adjusted scale
/// (`:45105-45130`; the port's `frame` is the atlas icon quad, the
/// `GetDrawHitbox` stand-in), anchored on `(int)itemLocation` with the
/// direction/gravity corner flips (`:45131-45138`, `gravDir = 1`), then
/// the stab shape (`:45173-45179`): facing left shifts X back by the
/// 1.4-growth, Width ×1.4, Y down `Height * 0.6`, Height ×0.6. The two
/// umbrellas grow +14 Height / shrink 10 Width, facing left shifting X
/// +10 (`:45180-45188`). The 5094-5097 pre-shrink switch (`:45107-45125`)
/// never meets a useStyle-3 item — those four are useStyle 1 — so it is
/// omitted here.
pub fn thrust_melee_hitbox(
    item_location: [f32; 2],
    frame_w: f32,
    frame_h: f32,
    scale: f32,
    dir: i32,
    remaining: f32,
    item_type: i32,
) -> Option<(i32, i32, i32, i32)> {
    // `:45168-45172`: `dontAttack` during the wind-up.
    if remaining > 0.666 {
        return None;
    }
    // `:45102/:45126-45130`: the int-truncated anchor + scaled size.
    let mut x = item_location[0] as i32;
    let mut y = item_location[1] as i32;
    let mut w = (frame_w * adjusted_item_scale(scale)) as i32;
    let mut h = (frame_h * adjusted_item_scale(scale)) as i32;
    // `:45131-45138`: the corner flips (`gravDir == 1`).
    if dir == -1 {
        x -= w;
    }
    y -= h;
    // `:45173-45179`: the stab shape. The C# grows Width AFTER the
    // left-shift read, and the Y shift reads the PRE-shrink Height.
    if dir == -1 {
        x -= (w as f64 * 1.4 - w as f64) as i32;
    }
    w = (w as f64 * 1.4) as i32;
    y += (h as f64 * 0.6) as i32;
    h = (h as f64 * 0.6) as i32;
    // `:45180-45188`: the umbrella pair.
    if item_type == 946 || item_type == 4707 {
        h += 14;
        w -= 10;
        if dir == -1 {
            x += 10;
        }
    }
    Some((x, y, w, h))
}

/// `Player.CompositeArmStretchAmount` (`Player.cs:211-216`): how far the
/// composite arm stretches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeStretch {
    Full,
    None,
    Quarter,
    ThreeQuarters,
}

impl CompositeStretch {
    /// The sheet row of the stretch (`PlayerDrawSet.cs:2254-2268`):
    /// Full=0, None=3, Quarter=2, ThreeQuarters=1.
    pub fn row(self) -> u32 {
        match self {
            CompositeStretch::Full => 0,
            CompositeStretch::None => 3,
            CompositeStretch::Quarter => 2,
            CompositeStretch::ThreeQuarters => 1,
        }
    }
}

/// `Player.CompositeArmData` (`Player.cs:219`): the per-arm composite state —
/// `enabled` switches the arm onto the composite sheet columns
/// (front 7 / back 8) at the stretch row, with `rotation` applied around the
/// composite pivot; disabled arms keep the per-body-frame cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompositeArmState {
    pub enabled: bool,
    pub stretch: CompositeStretch,
    pub rotation: f32,
}

impl CompositeArmState {
    /// The disabled arm — per-body-frame cells, no rotation (the per-tick
    /// reset at `AnimatePlayerAndGetItemFrame`, `Player.cs:43376-43377`).
    pub const DISABLED: CompositeArmState = CompositeArmState {
        enabled: false,
        stretch: CompositeStretch::None,
        rotation: 0.0,
    };
}

/// The held-item inputs of `PlayerFrame()` (`Player.cs:36144-36147`). The C#
/// reads `this.inventory[this.selectedItem]` straight out of the player; the
/// port keeps the inventory in the app, so the app hands the same fields in
/// before each fixed step (`world_sim/player.rs`, beside `swing_aim`).
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInputs {
    /// `inventory[selectedItem].useStyle` — drives the item-use body chain.
    pub use_style: u8,
    /// `inventory[selectedItem].useTurn` (S1-P-i05) — the run-key direction
    /// lock's exemption (`Player.cs:19837`: `itemAnimation == 0 || useTurn`).
    pub use_turn: bool,
    /// `flag6` (`:36146`): `useStyle != 10 && != 14 && != 15`. Those three
    /// styles (the fishing rod family and the like) skip the item-use rows.
    pub use_style_blocks_frames: bool,
    /// `inventory[selectedItem].type` — the useStyle 5 torch carve-out and the
    /// `type == 0` empty-hand share of the useStyle 1 branch.
    pub item_type: i32,
    /// `inventory[selectedItem].holdStyle` — the idle hold rows.
    pub hold_style: i32,
    /// `flag5` = `CanVisuallyHoldItem(HeldItem)` (`:36143`).
    pub can_visually_hold: bool,
    /// `itemAnimation > 0`.
    pub swing_active: bool,
    /// `itemAnimation / itemAnimationMax` (1 at the start, 0 at the end).
    pub swing_remaining: f32,
    /// The raw animation counters (`Player.itemAnimation`/`itemAnimationMax`)
    /// — the composite assignments read the raw phases.
    pub swing_anim: f32,
    pub swing_anim_max: f32,
    /// `Item.useAnimation` — the style 13's rhythm base.
    pub use_animation: i32,
    /// `Player.itemRotation` for the shooting styles (`Player.cs:47371-47378`):
    /// `atan2(aimY * direction, aimX * direction) - fullRotation`. The style 13
    /// composite arm reads it (`itemRotation * Directions.Y`, `:51195`).
    pub item_rotation: f32,
    /// `itemRotation * direction` — the useStyle 5 aim (`Player.cs:36423`).
    pub aim_relative: f32,
    /// `!wet || !item.noWet` — the two held-item arms that the water kills.
    pub hold_style_dry: bool,
    /// `!happyFunTorchTime || createTile != 4` — the torch hold override.
    pub torch_hold_ok: bool,
    /// The grapple body arm's lean — the hooks' centroid minus the body centre
    /// (`Player.cs:36502-36520`).
    pub grapple_dx: f32,
    pub grapple_dy: f32,
}

/// `ApplyEquipFunctional`'s `buffImmune[...] = true` rows (`Player.cs:
/// 15300-15570`): the Ankh Shield's eleven debuffs (1612/1613), its parts
/// (885-893 one each, 901-904 two each), the Chill/Frozen pair on 1921 and the
/// 156/22 rows on 3781/5354. The `ResetEffects` head clears the table every
/// tick (`Player.Update`'s `:25563` loop, the port's `reset_buff_immune`), so
/// these re-apply inside the equipment pass.
pub const BUFF_IMMUNITY_ROWS: &[(u16, &[i32])] = &[
    (885, &[30]),
    (886, &[36]),
    (887, &[20]),
    (888, &[22]),
    (889, &[32]),
    (890, &[35]),
    (891, &[23]),
    (892, &[33]),
    (893, &[31]),
    (901, &[33, 36]),
    (902, &[30, 20]),
    (903, &[32, 31]),
    (904, &[35, 23]),
    (1612, &[33, 36, 30, 20, 32, 31, 35, 23, 22, 156]),
    (1613, &[46, 33, 36, 30, 20, 32, 31, 35, 23, 22, 156]),
    (1921, &[46, 47]),
    (3781, &[156]),
    (5354, &[22, 156]),
];

/// One queued `Dust.NewDust` from `Player.StickyMovement` / `Collision.StickyTiles`
/// (`Collision.cs:3401`, `Player.cs:23036-23080`): the port's tick only knows
/// where the puff goes; the drain in `world_sim` calls `DustPool::new_dust`
/// with these args, then — for the type-153 honey puffs only — rolls the
/// `Next(0,6)` scale arm and applies the `velocity *= 0.1` / `noGravity`
/// post-arms, so the RNG draw order matches the source (S1-A-10).
#[derive(Clone, Copy, Debug)]
pub(crate) struct StickyDust {
    pub(crate) pos: Vec2,
    pub(crate) kind: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
    /// `Player.cs:23037-23039`: the `Next(0,6)` scale + `velocity *= 0.1` +
    /// `noGravity` tail. Only the type-153 honey puffs carry it; the type-30
    /// web puff (`Collision.cs:3403`) is a bare `NewDust`.
    pub(crate) post_arms: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    // The class combat stats (`Player.cs:2430-2437`): every class starts at
    // crit 4 and a 1.0 damage multiplier; the equipment modifies them
    // (M3-EQUIP).
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    /// `Player.minionDamage` (`Player.cs:2437`, default 1.0): the summon
    /// class multiplier — the minion prefixes 69-72 add to it through
    /// `weapon_damage_multiplier`'s Summon arm.
    pub minion_damage: f32,
    // The ranged effective-damage components (`Player.cs:2442-2450` +
    // `arrowDamageAdditiveStack`, reset `:18338`/`:18396-18399`, read by
    // `GetWeaponDamageMultiplier`'s ranged arms through the computed
    // properties `:3982-3986`). S1-C-4: the arms land with the multiplier;
    // the quiver stack + the Archery buff arm them now, the Shroomite
    // helmets ride the armor walk, and the `rangedMultDamage` consumers
    // (the C#'s separate multiplier class) keep the reset 1.0.
    pub ranged_mult_damage: f32,
    pub arrow_damage: f32,
    pub arrow_damage_additive_stack: f32,
    pub bullet_damage: f32,
    pub rocket_damage: f32,
    /// The per-player summon budget (M4-PROJ-8): `Player.maxMinions = 1`
    /// (`Player.cs:1260`) — raised by the buffs 110/150 and the summon armor
    /// rows, reset to 1 by `ResetEffects` (`:18448`).
    pub max_minions: i32,
    /// `Player.numMinions` (`Player.cs:1262`) — accumulated by the pool's
    /// minion block (`Projectile.cs:16736`, Phase 3), zeroed at `:28279` and
    /// on death (`:17009`); the buff-216 arm reads it (`:10706`).
    pub num_minions: i32,
    /// `Player.slotsMinions` (`Player.cs:1264`, float) — the 0.5-slot
    /// spiders' fractional budget, accumulated beside `numMinions`.
    pub slots_minions: f32,
    /// `Player.maxTurrets = 1` (`Player.cs:2848`) — the sentry budget, raised
    /// by buff 348 and reset by `ResetEffects` (`:18449`).
    pub max_turrets: i32,
    /// `Player.maxTurretsOld` (`Player.cs:2850`) — the last observed
    /// `max_turrets`; a change drives the `UpdateMaxTurrets` sweep
    /// (`:25709-25712`, Phase 5).
    pub max_turrets_old: i32,
    /// `Player.MinionAttackTargetNPC = -1` (`Player.cs:3008`) — the
    /// right-click aim target; written by the use path (Phase 6).
    pub minion_attack_target_npc: i32,
    /// `Player.MinionRestTargetPoint = Vector2.Zero` (`Player.cs:3006`) —
    /// the armor-set-bonus rest spot the idle minions return to (the
    /// Forbidden double-tap aim, `MinionRestTargetAim` `:56935-56997`;
    /// `Vector2.Zero` = no rest target, `HasMinionRestTarget` `:4060`).
    /// Written by [`Player::minion_rest_target_aim`] and cleared by the
    /// set-bonus down-hold (`KeyHoldDown` `:56894-56912`) and the
    /// >1000-px invalidation (`UpdateMinionTarget` `:57005-57007`).
    /// (S1-P-i27 landed the writer; i56's Guardian rest states 1/3 read it.)
    pub minion_rest_target_point: Vec2,
    /// `Player.setForbidden` (`ArmorSetBonuses.cs:61-66`) — the Forbidden
    /// armor set bonus flag; the double-tap-down rest aim + storm command
    /// gate (`KeyDoubleTap` `Player.cs:56779-56786`). Re-derived every tick
    /// by the armor-set pass (`ResetEffects` clears it, `:18663`-adjacent).
    pub set_forbidden: bool,
    /// `Player.setStardust` (`Player.cs:16341`, `ApplySetBonus_Stardust`) —
    /// the Stardust armor set bonus flag; the down-hold rest-target clear
    /// gate (`KeyHoldDown` `:56903-56906`).
    pub set_stardust: bool,
    /// `Player.ownedProjectileCounts[ProjectileID.Count]` (`Player.cs:2992`)
    /// — recounted every tick from the live pool (`:25565`); the summon
    /// buff-arms and the keep-alive checks read exactly this.
    pub owned_projectile_counts: [i32; PROJ_COUNT],
    /// `Player.highestStormTigerGemOriginalDamage` (`Player.cs:1304`) — the
    /// recount's type-831 maximum (`:12194-12215`).
    pub highest_storm_tiger_gem_original_damage: i32,
    /// `Player.highestAbigailCounterOriginalDamage` (`Player.cs:1314`) — the
    /// recount's type-970 maximum; AI_062's 963 scaling reads it.
    pub highest_abigail_counter_original_damage: i32,
    /// The 29 summon keep-alive flags `ResetEffects` clears
    /// (`Player.cs:18620-18646`+`:18741`) and `UpdateBuffs`' arms re-affirm
    /// (`:10483-10972`); AI_062 reads them to pin `timeLeft = 2` (Phase 4).
    /// Fields verbatim: `Player.cs:1268-1300` (order here groups the arm
    /// table's pairs).
    pub pygmy: bool,
    pub raven: bool,
    pub slime: bool,
    pub hornet_minion: bool,
    pub imp_minion: bool,
    pub spider_minion: bool,
    /// `Player.nextCycledSpiderMinionType` (`Player.cs:1280`, int default 0):
    /// the Spider Staff's spawn cycle (`:48478-48480`) — each use spawns
    /// `shoot + nextCycledSpiderMinionType` (390/391/392) then increments mod
    /// 3. Never reset by `ResetEffects` (no C# reset site), so it persists
    /// across uses and lives.
    pub next_cycled_spider_minion_type: i32,
    pub twins_minion: bool,
    pub pirate_minion: bool,
    pub vampire_frog: bool,
    pub sharknado_minion: bool,
    pub ufo_minion: bool,
    pub stardust_minion: bool,
    pub bats_of_light: bool,
    pub baby_bird: bool,
    /// AI_026 slice A (S1-D-4b-10) — the 14 flyer pets' keep-alive flags
    /// (`Player.cs`'s `eater`/`skeletron`/`hornet`/`tiki`/`wisp`/
    /// `zephyrfish`/`petFlag*` fields). `AI_026`'s per-type blocks
    /// (`Projectile.cs:58284-58910`) clear the flag when the owner dies and
    /// pin `timeLeft = 2` while it stands. The grant side (the pet
    /// buff/item pass, `Player.cs`'s `UpdateBuffs` pet arms) is unported —
    /// the flags stay false until it lands (ANALYSIS "AI_026 slice A").
    pub eater: bool,
    pub skeletron: bool,
    pub hornet: bool,
    pub tiki: bool,
    pub wisp: bool,
    pub zephyrfish: bool,
    pub pet_flag_baby_shark: bool,
    pub pet_flag_lil_harpy: bool,
    pub pet_flag_glittery_butterfly: bool,
    pub pet_flag_queen_bee_pet: bool,
    pub pet_flag_duke_fishron_pet: bool,
    pub pet_flag_moon_lord_pet: bool,
    pub pet_flag_dd2_betsy_pet: bool,
    pub pet_flag_pufferfish: bool,
    pub dead_cells_mushroom_boi_minion: bool,
    pub palworld_cattiva_minion: bool,
    pub palworld_foxsparks_minion: bool,
    pub palworld_trusty_cattiva_minion: bool,
    pub palworld_trusty_foxsparks_minion: bool,
    pub clay_pot_minion: bool,
    pub forbidden_minion: bool,
    pub flinx_minion: bool,
    pub abigail_minion: bool,
    pub storm_tiger: bool,
    pub smolstar: bool,
    pub empress_blade: bool,
    pub stardust_guardian: bool,
    pub stardust_dragon: bool,
    /// `Player.DeadlySphereMinion` (`Player.cs:1288`) — buff 161's flag, the
    /// chain's last arm (`:10957-10972`).
    pub deadly_sphere_minion: bool,
    /// `Player.dead`: the player is dead and waiting for the respawn timer
    /// (`Player.cs:38908`).
    pub dead: bool,
    /// `Player.aggro` (`Player.cs:1446`, the int default 0): the stealth
    /// offset the shared NPC target scan subtracts (`NPC.cs:77688`) and
    /// the facing-inheritance gate reads (`:77749`). The stealth armour
    /// writers are unported (the Scout Sling's `aggro -= 400` half,
    /// `Player.cs:14579`, still skips) — stock 0 until they land; the
    /// S1-D-3p wiring reads it through `npc_target::session_candidate`.
    pub aggro: i32,
    /// `Player.npcTypeNoAggro[NPCID.Count]` (`Player.cs:2994`, the bool
    /// array default false): per-NPC-type "do not aggro" flags (the
    /// Royal Gel / calm-accessory writers are unported). Indexed by the
    /// C# NPC type (`NpcKind::cs_id`), read by the target scan's +1000
    /// penalty (`NPC.cs:77689`) and facing suppression (`:77740`).
    pub npc_type_no_aggro: Vec<bool>,
    /// `Player.tankPet` (`Player.cs:870`, default -1): the projectile
    /// slot of the player's tank pet — AI_120's Guardian pins it every
    /// non-idle tick (`Projectile.cs:44680-44684`, S1-P-i56); the target
    /// scan's steal arm reads it (`NPC.cs:77701`).
    pub tank_pet: i32,
    /// `Player.tankPetReset` (`Player.cs:872`, default false): the
    /// one-tick grace latch — `Player.Update` (`:24539-24548`) sets it
    /// on the first tick a tank pet is held and clears `tankPet` to -1
    /// on the next tick the pet did NOT re-pin it. The Guardian's AI
    /// writes false every non-idle tick (`Projectile.cs:44683`), so a
    /// live Guardian keeps its claim; an idle/despawned one lets the
    /// decay drop it. The player-side decay is ported as
    /// `Player::tank_pet_decay`, called inside both fixed-step branches
    /// of `world_sim::player::update_player` (S1-P-i56).
    pub tank_pet_reset: bool,
    /// `Player.numberOfDeathsPVE` (`Player.cs:834`, `:38844`): the PVE death
    /// counter `/death` reads (`DeathCommand`).
    pub deaths_pve: u32,
    /// `Player.numberOfDeathsPVP` (`Player.cs:836`, `:38840`): the PVP death
    /// counter `/pvpdeath` reads (`PVPDeathCommand`).
    pub deaths_pvp: u32,
    /// `Player.hostile` (`Player.cs:2570`): the PVP toggle the inventory's
    /// `DrawPVPIcons` reads and toggles (`Main.cs:42664`, `:42692`). Gates the
    /// PVP damage rules (`damage::*`, `Player.Damage_PVP`).
    pub hostile: bool,
    /// `Player.team` (`Player.cs:1430`): 0 = none, 1..=5 = Red/Green/Blue/
    /// Yellow/Pink (`PlayerTeamID`). The `DrawPVPIcons` row index.
    pub team: i32,
    /// `Player.hideVisibleAccessory[10]` (`Player.cs:1708`): per-accessory
    /// "hide the visuals" flags, toggled by the ticks beside the accessory
    /// column (`Main.cs:41426-41460`) and read by the dye/visual update
    /// (`Player.cs:9610`, `:14378`). Indices 0..2 are the armour slots and keep
    /// their ticks switched off, exactly like the vanilla's `num38 > 2` gate.
    pub hide_visible_accessory: [bool; 10],
    /// `Player.deadTime`: the ticks spent dead (`Player.cs:16923`); gates the
    /// respawn-skip (90-tick lockout, `Player.cs:57111`).
    pub dead_time: i32,
    /// `Player.anglerQuestsFinished` (`Player.cs:1910`): the count
    /// the Angler's reward table reads (`GetAnglerReward_MainReward`'s
    /// milestone switch, `Player.cs:56323`).
    pub angler_quests_finished: i32,
    /// `Player.golferScoreAccumulated` (`Player.cs:1912`): the golf score
    /// the Golfer's shop and chatter tier (`Chest.cs:3158`, `NPC.cs:94792`).
    pub golfer_score: i32,
    /// `Player.respawnTimer`: the ticks left until the respawn.
    pub respawn_timer: i32,
    /// `Player.TagEffectStack` (`Player.cs`'s field): the whip tags this
    /// player applied (`tag_effects.rs`, M4-PROJ-7).
    pub tag_effects: crate::tag_effects::TagEffectStack,
    /// `Player.whipRangeMultiplier` (`Player.cs:1140`, reset 1f `:18498`):
    /// the whip curve's reach scale (`FillWhipControlPoints` `:48301`). The
    /// Tiki head (+0.1 `:13854`), the Stardust body/legs (+0.15 `:13879`),
    /// the Tiki set (+0.2 `ArmorSetBonuses.cs:15`), the Obsidian Outlaw set
    /// (+0.3 `:123`) and the Mobius Strip (+0.1 `:12979`) add to it.
    /// (S1-P-i24/i27 both declared it; gate-merged to one field.)
    pub whip_range_multiplier: f32,
    /// `Player.whipUseTimeMultiplier` (`Player.cs:1142`, reset 1f `:18499`):
    /// the whip swing's use-time scale (`ApplyItemAnimation` `:4476`). The
    /// Obsidian Outlaw set multiplies it by `1/1.15` (`ArmorSetBonuses.cs:124`).
    pub whip_use_time_multiplier: f32,
    /// `Player.summonerWeaponSpeedBonus` (`Player.cs:2460`, reset 0 `:18341`):
    /// the whip-speed buff adds (Dark Harvest 311 +0.35, Sword Whip 308 +0.25,
    /// Thorn Whip 314 +0.12, `:10110-10121`); `CapAttackSpeeds` (`:29025`)
    /// converts `meleeSpeed + bonus` into the use-time multiplier the swing
    /// reads. Stored as the RAW bonus until the cap fold.
    pub summoner_weapon_speed_bonus: f32,
    /// `Player.coolWhipBuff` / `cobWhipBuff` (`Player.cs:2108`/`:2110`, reset
    /// `:18400-18401`): the Cool/Cob Whip's one-snowflake/one-spark latch
    /// (`Projectile.cs:13918`/`:13929`) — the buff 312/365 arms re-affirm them.
    pub cool_whip_buff: bool,
    pub cob_whip_buff: bool,
    /// `Player.accSnakeBand` / `accWickedArmlet` / `accMobiusStrip` /
    /// `accSilverBracer` (`Player.cs:1208`/`:1214`/`:1210`/`:1212`, reset
    /// `:18519-18522`): the whip accessory flags. The Snake Band repeats the
    /// swing (`:42711-42731`), the Wicked Armlet adds +1 knockback per whip
    /// throw (`:48663-48666`), the Mobius Strip / Silver Bracer / Wicked
    /// Armlet raise `maxTagEffects` (`:12974-12984`). The i27 grant rows:
    /// Silver Bracer 6156/Twilight Grasp 6163/Silver Shield 6183; Mobius
    /// Strip 6158/Ouroboros Ring 6162; Wicked Armlet 6159/Twilight Grasp
    /// 6163/Armlet of Ruin 6179/Fire Gauntlet-family 6182.
    pub acc_snake_band: bool,
    pub acc_wicked_armlet: bool,
    pub acc_mobius_strip: bool,
    pub acc_silver_bracer: bool,
    /// `Player.repeatWhipSwings` (`Player.cs:1216`, default −1) and
    /// `repeatWhipsResetCooldown` (`:1218`): the Snake Band's swing counter
    /// (`++` per whip throw `:48686`, reset to −1 after 120 idle ticks
    /// `Update_SnakeBand` `:28412-28418`).
    pub repeat_whip_swings: i32,
    pub repeat_whips_reset_cooldown: i32,
    /// The jump-input edge state for the dead-skip roll.
    prev_jump_input: bool,
    /// The `UpdateBuffs`-derived debuff flags (`ResetEffects` clears,
    /// `Player.cs:18640-18720`; the per-slot derivation `:10039-10061` and
    /// the Weak/Webbed/Darkness rows). Movement reads `slow`/`chilled` as
    /// the `strongestMoveSpeedDebuff` tiers (`:25945-25953`), `frozen`/
    /// `webbed` lock the controls and the item use (`:3381`), `blind` drops
    /// `Lighting.GlobalBrightness` 1.2 → 1 (`Lighting.cs:112`).
    /// The `Player.SceneMetrics` aura snapshot — the app refreshes it per
    /// tick from the scene scan; the buff grants and the regen inputs read
    /// it (`M2-BUFF-5`).
    pub auras: tr_core::scene::Auras,
    /// `Player.manaRegenBonus` — the star-in-bottle's +10 per slot
    /// (`Player.cs:9933`), zeroed by `ResetEffects` (`:18655`).
    pub mana_regen_bonus: i32,
    /// `Player.wellFed` — any of the fed tiers 26/206/207 held
    /// (`Player.cs:11933-11967`); gates the re-eat (`:14353`) and the
    /// expert regen halving.
    pub well_fed: bool,
    /// The WellFed mining speed bonus — the `pickSpeed -=` negative
    /// (`:11945`: the tier's -0.05/-0.1/-0.15); the mining hardness
    /// multiplies by (1 - bonus).
    pub pick_speed_bonus: f32,
    /// The WellFed melee speed add (`meleeSpeed +=`, `:11937`) — rides the
    /// swing pacing until the per-item meleeSpeed lands (M3-ITEM).
    pub melee_speed_bonus: f32,
    /// (Resync keep-one: the i12 `summoner_weapon_speed_bonus` /
    /// `whip_use_time_multiplier` declarations merged with the i24 pair
    /// above — ONE field each, the read-only
    /// [`Self::summoner_weapon_speed_multiplier`] accessor is canonical.)
    /// `Player.tileSpeed` / `Player.wallSpeed` (`Player.cs:2470`/`:2468`,
    /// reset 1f `:18532`/`:18531`): the Builder buff's +0.25 (`:10160-10161`)
    /// and the Brick Layer / Extendo Grip accessories' +0.5
    /// (`:13029`/`:13038`). `UpdateEquips`' tail (`:25913-25922`) caps them
    /// at 3f and INVERTS them — `ApplyItemAnimation`'s createTile/createWall
    /// branches (`:4486`/`:4490`) read the inverted multipliers.
    pub tile_speed_bonus: f32,
    pub wall_speed_bonus: f32,
    /// `Player.autoReuseGlove` (`Player.cs:2384`, reset `:18710`): the
    /// Feral Claws and the glove family grant it (`:15107`, `:14413`,
    /// `:14602`, `:15304`, `:15311`, `:15374`); `TryAllowingItemReuse`
    /// (`:52905-52920`) forces `releaseUseItem` for melee (except the Flying
    /// Knife 3030) and the attack-speed-scaling summon weapons.
    pub auto_reuse_glove: bool,
    /// `Player.stressBall` (`Player.cs:1814`-adjacent, reset `:18431`): the
    /// Stress Ball (5543) grants it while `CanUseStressBall` passes
    /// (`:14730-14733`); `TryAllowingItemReuse` treats it like the
    /// auto-reuse-all setting.
    pub stress_ball: bool,
    // `Player.revolverCritChanceBonus` — declared below (S1-P-i14's field;
    // the V5 gate dropped S1-P-i12's duplicate seed).
    /// `inventory[selectedItem].type` — the held kind snapshot the
    /// `CanUseStressBall` gate reads (`Player.cs:14352`); the world-sim
    /// fills it each tick before the grant pass.
    pub held_item_kind: u16,
    /// `Player.statDefense` — the armor pieces' defense + the set bonus
    /// (M3-EQUIP-2); the buff arms' `statDefense` adds join as their
    /// systems land.
    pub stat_defense: i32,
    pub slow: bool,
    pub chilled: bool,
    pub frozen: bool,
    pub webbed: bool,
    pub blind: bool,
    /// The S1-E-7 wave-A potion-buff flags (`Player.UpdateBuffs`,
    /// `Player.cs:9875-10048`; `ResetEffects` clears `:18355`, `:18367-18381`,
    /// `:18650`, `:18695`, `:18705`; `manaRegenBuff` `:18394`). Each arm
    /// re-derives its flag per tick from the buff slots.
    /// Buff 1 (ObsidianSkin): lava immunity + hot-tile walking + OnFire(24)
    /// immunity (`:9875-9880`).
    pub lava_immune: bool,
    /// Buff 4 (Gills): `CheckDrowning`'s probe inverts (`:9958`, `:23226`).
    pub gills: bool,
    /// Buff 6 (ManaRegeneration): `RegenInputs::mana_regen_buff` (`:9966`).
    pub mana_regen_buff: bool,
    /// Buff 8 (Featherfall): the fall caps + the gravity/3 arm (`:9979`).
    pub slow_fall: bool,
    /// Buff 9 (Spelunker): treasure highlighting — draw-side, flag only.
    pub find_treasure: bool,
    /// Buff 343 (BiomeSight): ore highlighting — draw-side, flag only.
    pub biome_sight: bool,
    /// Buff 10 (Invisibility): the spawner's ×1.2 rate / ×0.8 cap (`:9991`).
    pub invis: bool,
    /// Buff 12 (NightOwl): the legacy light pass's brightness lift (`:9999`).
    pub night_vision: bool,
    /// Buff 13 (Battle): the spawner's ×0.5 rate / ×2 cap (`:10003`).
    pub enemy_spawns: bool,
    /// Buff 15 (WaterWalking): the liquid-surface walk — collision half
    /// unported, flag only (`:10014`).
    pub water_walk: bool,
    /// Buff 17 (Hunter): enemy highlighting — draw-side, flag only (`:10023`).
    pub detect_creature: bool,
    /// Buff 18 (Gravitation): the gravity-flip key — unported, flag only
    /// (`:10027`).
    pub grav_control: bool,
    /// Buff 31 (Confused): the control swap (`:10035`, `:24996-25004`).
    pub confused: bool,
    /// Buff 35 (Silenced): mana items refuse (`:10043`, `:52221`).
    pub silence: bool,
    /// Buff 160 (Dazed): the jump statics /5 and /2 (`:10047`, `:19637-19641`).
    pub dazed: bool,
    /// `Player.noItems` (`Player.cs:2332`): Cursed (23) locks item use
    /// (`:11733`); `ResetEffects` clears (`:18687`). The transformation-mount
    /// exemption (`ItemCheck_CheckCanUse_Inner` `:52140-52148`) and
    /// `Dismount`'s restore (`Mount.cs:6152-6155`) read it.
    pub no_items: bool,
    /// `Player.cursed` (`Player.cs:2334`): the Cursed (23) flag pair
    /// (`:11734`); `ResetEffects` clears (`:18688`).
    pub cursed: bool,
    /// `Player.empressBrooch` (`Player.cs:14773`): the Empress Brooch (4989)
    /// grant; `ResetEffects` clears (`:18455`). The Velociraptor mount's
    /// per-tick flight refill reads it (`Mount.cs:2993-2996`).
    pub empress_brooch: bool,
    /// `Player.ignoreWater` (`Player.cs`): the water mounts' swim grant
    /// (`UpdateBuffs` `:9887`/`:9892`/`:9902`/`:9907`); `ResetEffects` clears
    /// (`:18756`). Flag only until the swim-collision consumer lands.
    pub ignore_water: bool,
    /// `Player.accFlipper` (`Player.cs`): the flipper grant the water mounts
    /// share (`:9888`/`:9893`/`:9898`/`:9903`/`:9908`); `ResetEffects` clears
    /// (`:18361`).
    pub acc_flipper: bool,
    /// `Player.canFloatInWater` (`Player.cs:1086`): the Water Strider's
    /// (buff 265) float grant (`:9895-9898`) or the mount item 4404
    /// (`GrantArmorBenefits` `:13265-13267`); `ResetEffects` clears
    /// (`:18332`). `ShouldFloatInWater` (`:3872-3885`) reads it; the fishing
    /// +5 float bonus (`GetFishingConditions` `:42194`, `canFloatInWater &&
    /// wet`) reads the same field (gate merge i35 ⨉ i41 — one field).
    pub can_float_in_water: bool,
    /// `Player.lavaVision` (`Player.cs`): the Lava Shark's (buff 305) grant
    /// (`:9910`); `ResetEffects` clears (`:18757`). Draw-side, flag only.
    pub lava_vision: bool,
    /// `Player.spikedBoots` (`Player.cs`): the Rat mount's wall-slide grant
    /// (`Mount.UpdateAfterEquips` `Mount.cs:3002`); `ResetEffects` clears
    /// (`:18660`). The slide consumer is unported — flag only.
    pub spiked_boots: i32,
    /// `Player.slideDir` (`Player.cs:28084` reset, `:28242-28252` writes):
    /// the slope-slide direction the Rat's rotation lerp reads
    /// (`Mount.cs:3004-3033`).
    pub slide_dir: i32,
    /// `Player.FailedNoSpaceCount` / `FailedNoSpaceLocation`
    /// (`Player.cs:6053-6054`): the QuickMount no-space ladder — two refused
    /// dismounts play the tink + dust, the third forces the dismount and the
    /// no-space teleport (`:6049-6070`); `UpdateFailedDismount`
    /// (`:6037-6043`) clears the count on death/dismount/moved-tile.
    pub failed_no_space_count: i32,
    pub failed_no_space_location: (i32, i32),
    /// The per-slot Weak count — `meleeDamage -= 0.051f` and
    /// `moveSpeed -= 0.1f` per slot (`Player.cs:11873-11878`); the
    /// `statDefense -= 4` half has no player-defense consumer yet
    /// (M3-EQUIP-3) and rides `weak_stacks` until it lands.
    pub weak_stacks: i32,
    /// Top-left of the 20x42 hitbox, world pixels.
    pub x: f32,
    pub y: f32,
    /// Position at the previous fixed 60 Hz step — the pressure-plate bridge's
    /// `PressurePlateHelper.PlayerLastPosition` (`PressurePlateHelper.cs:16`).
    /// NOT a render interpolation source: the C# draws `player.position` raw
    /// (`LegacyPlayerRenderer.cs:468-475`) — see `draw_region`'s draw note
    /// (fix/player-motion-blur).
    pub prev_x: f32,
    pub prev_y: f32,
    /// `Player.ActuationRodLock` / `ActuationRodLockSetting`
    /// (`Player.cs:2616-2618`): the Actuation Rod acts once per HOLD — the
    /// lock and the tile state it saw. `ResetEffects` clears the lock only
    /// while `releaseUseItem` rides (`:18802-18804`), so a held button keeps
    /// the lock across ticks and a fresh press re-arms it.
    pub actuation_rod_lock: bool,
    pub actuation_rod_lock_setting: bool,
    /// `Player.InfoAccMechShowWires` (`Player.cs:2620`): the Mechanical Lens
    /// (3619) / Wire Kite (3611) show the wires without a mech tool held.
    /// Cleared in `ResetEffects` (`:18524`), re-granted by
    /// `RefreshMechanicalAccsFromItemType` (`:12801-12804`).
    pub info_acc_mech_show_wires: bool,
    /// `Player.autoActuator` (`Player.cs:2474`): the Actuation Accessory
    /// (3624) auto-places actuators on solid tiles while building. Cleared
    /// in `ResetEffects` (`:18534`), re-granted by
    /// `RefreshMechanicalAccsFromItemType` (`:12817-12820`) and the held-item
    /// arm (`:15295-15298`). The placement consumer
    /// (`PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`, `:40594`) is a
    /// recorded follow-up (ANALYSIS "The wiring tools (S1-P-i34)").
    pub auto_actuator: bool,
    /// `Player.builderAccStatus` (`Player.cs:814`, `BuilderAccToggleIDs.Count
    /// = 12` at `:77`): the info-accessory toggle states. Slots 4/5/6/7 are
    /// the red/blue/green/yellow wire visibility, 9 the actuator visibility
    /// (`Main.DrawWires`, `Main.cs:58237-58242`); slot 8 gates the lens'
    /// wire overlay (`WiresUI.Settings.DrawWires`, `WiresUI.cs:39`). The
    /// per-slot 0=Bright/1=Normal/2=Faded cycle is UI-driven
    /// (`Main.cs:47549-47555`, wraps at 3 — the 3=Hidden arm is unreachable
    /// through the click). The C# field initialises to all-zero (`:814`);
    /// the port's spawn mirrors the player reset's `[0] = 1` (`:56092`).
    pub builder_acc_status: [i32; 12],
    /// `Entity.oldPosition` (`Player.cs:28071`): the position before this
    /// tick's movement. The switch-tile bridge (`Collision.SwitchTiles`,
    /// `Player.cs:28274-28275`) reads it to tell "the entity moved onto the
    /// plate" from "the entity stands on it".
    pub old_x: f32,
    pub old_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    /// 1 = right, -1 = left.
    pub facing: i32,
    /// The equipped item kinds — `Player.armor[0..20]`'s kinds snapshot the
    /// update's grant pass reads (`UpdateEquips`, `Player.cs:12892`). Filled
    /// from `App.equip` per tick.
    pub equipped: [u16; 20],
    /// `Player.dye[0..10]`'s kinds — `UpdateDyes`' input through
    /// `GetEffectiveDye` (`Player.cs:5742-5764`; the port has no loadouts, so
    /// the slot itself is the source). Filled from `App.equip` per tick next
    /// to `equipped`.
    pub dye_slots: [u16; 10],
    /// `Player.miscDyes[0..5]`'s kinds (`Player.cs:1494`; pet/light/cart/
    /// mount/hook). Filled from `App.equip` per tick.
    pub misc_dye_slots: [u16; 5],
    /// `Player.hideMisc` (`Player.cs:1710` — a `BitsByte`; only bits 0/1
    /// have writers, the pet/light-pet hides `:17344`/`:17353` and the
    /// buff-bar cancel `Main.cs:43401`). The port models the two live bits
    /// as a bool pair; the pet-grant arms (`:17303`/`:17318`) are unported,
    /// so no consumer re-shows the visual yet (ANALYSIS).
    pub hide_misc: [bool; 2],
    /// `UpdateDyes`' `c*` cache (`Player.cs:9587-9757`) — the dye pass each
    /// draw layer applies (the `DrawData.shader` source, 0 = plain). The port
    /// fills the fields with draw consumers: the armor trio reads its dye
    /// slots directly (`cHead/cBody/cLegs = GetEffectiveDye(0..2).dye`,
    /// `:9593-9595`) and the misc five mirror `:9600-9604` (`cMount` feeds
    /// the mount layers; the pets/minions have no draw path yet). The ~27
    /// accessory effect-slot fields of `UpdateItemDye` wait for the accessory
    /// visuals — deviation in `docs/ANALYSIS.md`.
    pub c_head: u32,
    pub c_body: u32,
    pub c_legs: u32,
    pub c_pet: u32,
    pub c_light: u32,
    pub c_minecart: u32,
    pub c_mount: u32,
    pub c_grapple: u32,
    /// The equipped items' rolled prefixes, slot-parallel to `equipped` —
    /// `GrantPrefixBenefits` (`Player.cs:13953-14044`) reads them in the
    /// accessory grant pass below.
    pub equipped_prefix: [u16; 20],
    /// The four banks (`Player.bank/bank2/bank3/bank4`,
    /// `Player.cs:1580-1591` — `Chest.CreateBank(-2..-5)`, 40 slots each;
    /// player-bounded, always unlocked per `Chest.cs:178-189`).
    pub banks: [crate::bank::BankStore; 4],
    /// `Player.chest` (`Player.cs:2876`): the single open-container index —
    /// -1 closed, -2..-5 the banks (`bank.rs`'s `bank_for_chest`), >= 0 the
    /// `App.world_chests` store index. Every open/close/tick route writes
    /// this one field (`chest.rs`'s shared path — the M3-CHEST-INT unify).
    pub chest: i32,
    /// `Player.chestX`/`chestY` (`Player.cs:2878`/:2880): the open
    /// container's tile origin — set for the banks too (`OpenChest`
    /// `:33524-33525`), the per-tick range close's probe point.
    pub chest_x: i32,
    pub chest_y: i32,
    /// `Player.editedChestName` (`Player.cs:1442`): the MP client's rename
    /// flag — `ChestUI.RenameChestSubmit` sets it on a write under netMode 1
    /// (`ChestUI.cs:855-859`), and the next `33 SyncPlayerChest` send carries
    /// the name (`Main.TrySyncingMyPlayer`, `Main.cs:17575-17593`; the open
    /// click's flush `Player.cs:33097-33101` / the dresser's `:32883-32887`
    /// are the same seam). Cleared on the send (S1-P-i43).
    pub edited_chest_name: bool,
    /// The previous tick's `chest` snapshot for `Main.TrySyncingMyPlayer`'s
    /// close-transition test (`Main.cs:17575` — `player` is the last-synced
    /// copy; the port keeps just the chest index, the one field the rename
    /// flush reads). Updated by `chest::tick_open` each player tick
    /// (S1-P-i43).
    pub prev_synced_chest: i32,
    /// `Player.IsVoidVaultEnabled` (`Player.cs:24550-24553` —
    /// `HasItem(4131)`): refreshed per tick beside the other scans.
    pub void_vault_enabled: bool,
    /// `Player.ateArtisanBread` (`Player.cs:2048`): the artisan loaf's
    /// +4-tile crafting reach (`Player.AdjTiles`, `:35671`). The item that
    /// sets it (5326, `Player.cs:45218-45221`) has no ported use effect yet,
    /// so the flag stays false in play — the reach formula reads it.
    pub ate_artisan_bread: bool,
    /// `Player.unlockedBiomeTorches` (`Player.cs:2046`): the Torch God's
    /// biome torches — the gate on the `needTorchGodsFavor` recipes
    /// (`Recipe.cs:353-356`). The unlock itself (the Torch God's dialogue)
    /// is unported, so the flag stays false in play.
    pub unlocked_biome_torches: bool,
    /// The `TR_JUMPS`/`TR_BOOTS`/`TR_WINGS` pins: the debug stand-ins for the
    /// granting equipment, re-applied by the accessory pass every tick (the
    /// vanilla `ResetEffects` clears the flags and the grants re-set them).
    pub pinned_jumps: [bool; 9],
    pub pinned_balloon: bool,
    pub pinned_rocket_boots: i32,
    /// The `TR_DASH` pin: stands in for the dash-granting equipment, re-applied
    /// after the per-tick `dashType` clear (the `pinned_*` pattern).
    pub pinned_dash: i32,
    /// `Player.extraAccessory` (`Player.cs:866`): the Demon Heart's use-grant
    /// (`:45197`) - unlocks the 6th accessory slot in expert mode
    /// (`IsItemSlotUnlockedAndUsable` `:12587`). The consumable use is
    /// unported, so the flag stays false in play.
    pub extra_accessory: bool,
    /// The `TR_MOUNT=<id|name>` pin: the debug stand-in for the saddle item
    /// (the real items equip through `Item.mountType`), re-applied after the
    /// per-tick flag clear while no mount is active (the `pinned_*` pattern,
    /// docs/ANALYSIS.md "Item / equip deviations").
    pub pinned_mount: Option<i32>,
    /// `PlayerSittingHelper.isSitting` (`PlayerSittingHelper.cs:18`) — the
    /// chair-sit state; the player's bottom is snapped to the seat.
    pub sitting: bool,
    /// `PlayerSleepingHelper.isSleeping` (`PlayerSleepingHelper.cs:13`) — the
    /// bed-sleep state; the bottom is snapped to the bed anchor and the pose
    /// rotates through `full_rotation`.
    pub sleeping: bool,
    /// `PlayerSleepingHelper.timeSleeping` (`:17`): the fully-asleep counter
    /// — `>= 120` (`TimeToFullyFallAsleep`, `:11`) arms the clock's ×5
    /// (`World::day_rate`).
    pub sleep_time: u32,
    /// `PlayerSleepingHelper.visualOffsetOfBedBase`'s key: the bed style
    /// (`frameY / 36`, `GetSleepingTargetInfo` `:222`) refreshed per tick —
    /// the renderer derives the offset from the `SetOffsetbyBed` table.
    pub sleeping_style: u32,
    /// `PlayerSittingHelper.offsetForSeat` (`:14`): the per-style seat drop
    /// the decoder returned, refreshed by `update_sitting` (`:84`).
    offset_for_seat: (f32, f32),
    /// `PlayerSittingHelper.GetSittingOffsetInfo`'s `posOffset`
    /// (`PlayerSittingHelper.cs:18-32`, `sittingIndex` 0 in SP): the seat
    /// offset mirrored to the facing — the draw pass reads it.
    pub seat_pos_offset: (f32, f32),
    /// `GetSittingOffsetInfo`'s `seatAdjustment` (`:23-24`): `-4 +
    /// (int)offsetForSeat.Y` — the torso drop while seated.
    pub seat_y_offset: i32,
    /// `Player.noKnockback` (`Player.cs:1950`) - the Cobalt Shield's grant
    /// (`Player.cs:15025-15028`); the Hurt gate reads it.
    pub no_knockback: bool,
    /// `Player.fireWalk` (`Player.cs:1878`, reset `:18355`): the ObsidianSkin
    /// buff (1) arms it per tick (`UpdateBuffs` `:9878`, S1-E-7 wave A) and
    /// the Obsidian Skull family grants it (`accessory_defense.rs`, S1-C-12
    /// wave C) — `Collision.CanTileHurt`'s hot arm skips the burning tiles
    /// (`Collision.cs:3220`).
    pub fire_walk: bool,
    /// `Player.hasPaladinShield` (`Player.cs:1408`, reset `:18777`): the
    /// Paladin's/Frozen/Hero shields above 25% life (`:15348-15353`).
    /// The hurt-side team redirect (`:38190`) rides C-7.
    pub has_paladin_shield: bool,
    /// M5-SEED-2: `Main.vampireSeed` — the `vampirism` micro seed's
    /// `InitializeSecretSeeds` write (`WorldGen.cs:555-558`). `UpdateLifeRegen`'s
    /// OnFire row reads it (`Player.cs:18983-18992`: −100 instead of −8, and the
    /// `drippingSlime` double is 100 instead of 8). Captured from the world at
    /// spawn (`Player::spawn`) — the flag is immutable post-generation, exactly
    /// like the C#'s world-level static.
    vampirism: bool,
    /// `Player.itemRotation`'s source for useStyle 5 — the aim angle the App
    /// updates per tick (`swing_aim`), mirrored onto the player for the draw.
    pub swing_aim: f32,
    /// The accessory-granted `Player.lifeRegen` base (`Player.cs:13122`'s
    /// `lifeRegen += armorPiece.lifeRegen` - the Band's `Item.lifeRegen = 2`).
    accessory_life_regen: i32,
    /// `Player.legFrame.Y / 56`: the current leg animation row (0..=19), driven by
    /// `PlayerFrame()`.
    pub leg_frame: u32,
    /// `Player.bodyFrame.Y / 56`: the current body animation row (0..=19). Mirrors the
    /// legs while walking; the app overrides it with the item-use rows during a swing.
    pub body_frame: u32,
    /// `Player.legFrameCounter`: accumulates `|velocity.X| * 1.3` while walking and
    /// advances a leg row every `8.0`.
    leg_frame_counter: f32,
    /// `Player.compositeFrontArm` / `compositeBackArm` (`Player.cs:1906/:4347`)
    /// — the composite arm poses the item-use styles assign (S1-C-4's
    /// swing parity); reset to DISABLED per frame and reassigned by the
    /// item-composite block.
    pub composite_front: CompositeArmState,
    pub composite_back: CompositeArmState,
    /// `Player.bodyFrameCounter` (`Player.cs:36594`): only the `legs == 140`
    /// walk branch accumulates it (`|velocity.X| * 0.5`); every other body arm
    /// either zeroes it or leaves it alone.
    body_frame_counter: f32,
    /// `Player.swimTime` (`Player.cs:35756-35762`): the merman/flipper swim
    /// window; `PlayerFrame` decrements it and clears it out of water.
    pub swim_time: i32,
    /// `Player.miscCounter`: ticked once per update in the original; the
    /// sandstorm leg pose turns the player every 4 ticks on it
    /// (`Player.cs:36302`).
    pub misc_counter: u32,
    /// `Player.pulley` / `Player.pulleyDir` (`Player.cs:1070`): the rope
    /// pose — body rows 1 (dir 2) and 2 (dir 1).
    pub pulley: bool,
    pub pulley_dir: u8,
    /// `Player.shieldRaised` (`Player.cs:36481`): body row 10.
    pub shield_raised: bool,
    /// `Player.sliding` (`Player.cs`): the slope/ice slide pose (body row 3 in
    /// the air). No port source sets it yet, so the arm is coded but
    /// unreachable.
    pub sliding: bool,
    /// `Player.sandStorm` (`Player.cs:36298`): the Sandstorm-in-a-Bottle dash
    /// pose (leg row 0, body row 6, a 4-tick turn cadence).
    pub sand_storm: bool,
    /// `Player.carpetFrame` (`Player.cs:1094`, init `-1`): the Magic Carpet
    /// glide pose (leg row 0, body row 6).
    pub carpet_frame: i32,
    /// `Player.legs` (`Player.cs:35766`): the effective legs armour texture id
    /// (`Item.legSlot`). The Moon Lord carve-out (`140`) and the sitting
    /// carve-out (`217`) read it; `0` = the default legs sheet.
    pub legs_armor: i32,
    /// The held-item half of `PlayerFrame()` - the C# reads
    /// `inventory[selectedItem]` directly, the port feeds the summary in from
    /// the app before the tick (`world_sim/player.rs`).
    pub frame_inputs: FrameInputs,
    /// `Player.meleeNPCHitCooldown` (`Player.cs:3180`,
    /// `int[Main.maxNPCs]`): the per-NPC melee re-hit lock in ticks.
    /// `ResetMeleeHitCooldowns` (`:44212-44218`) clears it at the use's
    /// start (`ItemCheck_StartActualUse` `:51679`),
    /// `UpdateMeleeHitCooldowns` (`:44220-44229`) decrements every entry
    /// once per damage tick (`:43664`), a landed hit re-arms it to the
    /// CURRENT `itemAnimation` (`ProcessHitAgainstNPC` `:44414`), and
    /// `CanHitNPCWithMeleeHit` (`:44231-44234`) gates the scan
    /// (`ItemCheck_MeleeHitNPCs` `:44246`). The C# indexes stable NPC
    /// slots; the port's pool compacts, so the map keys on the stable
    /// `Npc.uid` (`NPC.whoAmI`, the established carrier-link identity).
    /// S1-P-i05: the stab sweep's one-hit-per-swing lock.
    pub melee_npc_hit_cooldown: std::collections::HashMap<u32, i32>,
    /// `Player.wet` — the head/body in WATER (`Player.cs:27728`).
    pub wet: bool,
    /// `Player.lavaWet` — in LAVA.
    pub lava_wet: bool,
    /// `Player.honeyWet` — in HONEY.
    pub honey_wet: bool,
    /// `Player.shimmerWet` — in SHIMMER.
    pub shimmer_wet: bool,
    /// The `WetCollision` movement-speed scale for this tick's position delta
    /// (`Player.cs:28168-28171`): water/lava 0.5, honey 0.25, shimmer 0.375,
    /// dry 1.0.
    pub liquid_step: f32,
    pub in_water: bool,
    /// `Player.controlLeft` / `Player.controlRight`: this tick's held run keys
    /// (PlayerInput). The electrified drain tests them (Player.cs:19080-19082).
    pub control_left: bool,
    pub control_right: bool,
    /// `Player.controlJump` / `controlDown`: this tick's held jump/down keys.
    pub control_jump: bool,
    pub control_down: bool,
    /// `Player.controlUp` (`Player.cs:1738`): this tick's held up key. The
    /// grapple jump-off gate (`:22717`) and steering read it.
    pub control_up: bool,
    /// Remaining ticks of the held-jump boost (`Player.jump`).
    jump: i32,
    /// Jump was held on the previous fixed step, for press-edge detection.
    jump_held: bool,
    pub spawn_x: f32,
    pub spawn_y: f32,
    /// `Player.PotionOfReturnOriginalUsePosition` (`Player.cs:2480`): the
    /// Bottom at the Potion of Return 4870 use (`:37501`), the return leg's
    /// target (`DoPotionOfReturnReturnToOriginalUsePosition`, `:37512-37523`,
    /// the `Teleport` style-8 follow-up). Recorded by the recall cluster's
    /// 4870 arm (`recall_teleport.rs`).
    pub potion_of_return_original_use_position: Option<(f32, f32)>,
    /// `Player.PotionOfReturnHomePosition` (`Player.cs:2482`): the post-spawn
    /// Bottom the 4870 use records (`:37506`), the return-gate hitbox centre
    /// (`PotionOfReturnHelper.TryGetGateHitbox`).
    pub potion_of_return_home_position: Option<(f32, f32)>,
    pub hp: i32,
    pub max_hp: i32,
    /// `Player.statMana` / `Player.statManaMax` (`Player.cs:1930-1932`). A new
    /// character starts at `20/20` (`UICharacterCreation.cs:1721`; the default max
    /// is also 20 in `Player.Initialize`, `Player.cs:56075`).
    /// `Player.armorPenetration` / `meleeArmorPenetration` (`Player.cs:1918-1920`):
    /// the flat penetration the hit adds to `NPC.checkArmorPenetration`
    /// (`:44383`). `meleeArmorPenetration` only counts for melee hits.
    pub armor_penetration: i32,
    pub melee_armor_penetration: i32,
    /// `Player.lavaTime` / `lavaMax` (`Player.cs:1528-1530`): the lava
    /// immunity budget — the Lava Charm family adds 420 ticks to the cap
    /// (`:15168-15183`), and while the player stands in lava the timer drains
    /// one tick per frame before the 80-damage hit lands (`:27681-27722`).
    pub lava_time: i32,
    pub lava_max: i32,
    /// `Player.thorns` / `cactusThorns` / `turtleThorns` (`Player.cs:2800-2802`):
    /// the contact reflection — `thorns` is the fraction of the NPC's damage
    /// thrown back (the Thorns potion sets 1, the Dryad's Ward adds a half, the
    /// Turtle set pins 2 through `turtle_thorns`), and `cactus_thorns` is the
    /// Cactus set's flat 15.
    pub thorns: f32,
    pub cactus_thorns: bool,
    pub turtle_thorns: bool,
    /// `Player.skyStoneEffects` (`Player.cs:1120`): the Celestial Stone/Shell
    /// and the day/night Sun/Moon Stones set it, and the one block below adds
    /// the whole stat bundle (`:12988-13003`).
    pub sky_stone_effects: bool,
    /// `Player.endurance` (`Player.cs:1138`): the fraction of incoming damage
    /// the player shrugs off — `Hurt` applies `(int)((1 - endurance) * damage)`
    /// (`:38304`) after the defense reduction. `ResetEffects` zeroes it
    /// (`:18497`); the Worm Scarf, the Turtle/Chlorophyte/Solar sets and the
    /// Endurance/Ice-Barrier buffs add per tick.
    pub endurance: f32,
    pub mana: i32,
    /// `Player.manaCost` (`Player.cs:1884`): the magic items' cost
    /// multiplier — `ResetEffects` sets 1f (`:18334`) and the armor pieces,
    /// the accessory rows, the casters' set bonuses and `BuffID.MagicPower`
    /// subtract from it. `CheckMana` pays `(int)(item.mana * manaCost)`
    /// (`:5881-5884`).
    pub mana_cost: f32,
    /// `Player.statManaMax2` (`Player.cs:1932`): the effective mana cap —
    /// `ResetEffects` rebases it to the permanent `mana_max_base`
    /// (`:18389` `statManaMax2 = statManaMax`) and the per-tick grants
    /// (the Arcane prefix 66's +20 `:13971-13974`, the Magic Power buff's
    /// +20 `:11890`, the caster sets' `statManaMax2` rows) add on top.
    pub mana_max: i32,
    /// `Player.potionDelay`/`manaPotionDelay` (`Player.cs:1650-1652`): the
    /// Potion Sickness / Mana Potion cooldown clocks (S1-P-i37). Decremented
    /// once per tick (`Player.cs:24718-24724`); the buff-21 mirror re-assigns
    /// `potion_delay = buffTime[j]` from the debuff scan (`:11719-11722`);
    /// `ApplyPotionDelay` (`:51853-51886`) arms it at the drink; `UpdateDead`
    /// zeroes both (`:17018-17019`). `mana_potion_delay` stays 0 on the
    /// retail path — `DebugOptions.ManaPotionDelay` (`Testing/DebugOptions.cs:28`,
    /// static ctor `:86`) is false, its only writers (`:51941`, `:10322`)
    /// are constants-off.
    pub potion_delay: i32,
    pub mana_potion_delay: i32,
    /// `Player.manaSick`/`manaSickReduction` (`Player.cs:974-976`): the Mana
    /// Sickness (buff 94) magic-damage penalty. `ResetEffects` clears
    /// `manaSick` (`:18592`); the buff-94 arm (`:10318-10327`) re-arms it
    /// through `potions::apply_mana_sickness_arm`; the magic-damage tail
    /// (`:25900-25911`) rides `potions::apply_magic_damage_sickness_tail`
    /// (the i37 canonical fns — the i19 resync take-one). ONLY mana potions
    /// grant the sickness (`ApplyLifeAndOrMana` `:51933-51943`,
    /// `manaSickTime = 300` `:57101`) — never a weapon.
    pub mana_sick: bool,
    pub mana_sick_reduction: f32,
    /// `Player.manaHeat` (`Player.cs:980`): reset to 1f every tick
    /// (`:18593`). `ApplyManaPotionManaHeat` (`:51968-52004`) has NO caller
    /// in the retail source (the only `TryApplyManaHeat` callers are the
    /// mana-star pickups `:35071`/`:35087` — the S1-P-i52 lane owns those);
    /// drinking a mana potion grants NO ManaHeat, so the field stays 1.
    pub mana_heat: f32,
    /// `Player.pStone` (`Player.cs:2404`): the Philosopher's Stone family —
    /// 535 (`:15574-15577`), Charm of Myths 860 (`:15150-15153`), Catalyst
    /// Band 6185 (`:14615-14620`). `ResetEffects` clears (`:18720`), the
    /// accessory rows re-arm; a CHANGE rescales the live buff-21 timer
    /// (`:25640-25643` -> `AdjustRemainingPotionSickness` `:28939-28952`).
    pub p_stone: bool,
    /// The previous tick's `pStone` — the `flag9` latch (`Player.cs:25477`)
    /// the `:25640` change check compares against.
    p_stone_prev: bool,
    /// `Player.statManaMax` (`Player.cs:1931`): the PERMANENT mana cap —
    /// the character file's `manaMax` (`:55226`, clamped to 200) and the
    /// Mana Crystal's +20 use-grant (`:45276-45281`, unported). The
    /// per-tick rebase reads it; nothing else writes it in play.
    pub mana_max_base: i32,
    /// `Player.manaFlower` (`Player.cs:1876`-adjacent field, armed by the Mana
    /// Flower 555 `:15591-15594`, Arcane Flower 3991 `:14405-14406`, Magnet
    /// Flower 4000 `:14455-14457` and Mana Cloak 4001 `:14459-14460`). The
    /// `manaCost -= 0.08f` row applies ONCE after the equip loop
    /// (`UpdateEquips` `:12962-12965`) — a bool, so two flowers do not stack.
    /// `ResetEffects` clears it (`:18721`). It also gates `CheckMana`'s
    /// QuickMana lane (`:52588`), which under ManaV2 lands the same empty-bar
    /// arm as the plain lane (the potion seam rides i37's `use_potion_item`).
    pub mana_flower: bool,
    /// `Player.slowMagicUse` (`Player.cs:978`): the ManaV2 out-of-mana cast
    /// flag. `CheckMana` clears it on entry (`:52578`) and arms it when the
    /// bar drains to 0 (`DoRunOutOfManaEffect` `:52619-52622`); while armed a
    /// mana item's use animation/time stretch ×2.5 (`ApplyItemAnimation`
    /// `:4478-4483`, `ItemCheck_Shoot` `:47187-47194`, `GetSlowMagicMultiplier`
    /// `:47046-47049`). A mana potion drunk mid-swing undoes it
    /// (`UndoSlowMagicUse` `:51954-51966`).
    pub slow_magic_use: bool,
    /// `Player.cs:19420`'s `flag = itemAnimation > 0 || reuseDelay > 0` —
    /// the ManaV2 animation gate on the mana refill (`:19542-19545`). The
    /// sim caller feeds the App's swing/reuse timers each tick (the
    /// `frame_inputs` convention); the bare `update` callers keep false.
    pub mana_regen_item_in_use: bool,
    /// The 44 buff slots and `buffImmune` — `Player.buffType`/`buffTime`/`buffImmune`
    /// (`Player.cs:1510-1514`), driven by the `Player.AddBuff` family (`buffs.rs`).
    pub buffs: BuffSlots,
    pub hurt_cd: f32,
    /// The dodge-family flags (`Player.blackBelt`/`mysticSashDodge`/
    /// `brainOfConfusionItem`/`shadowDodge`/`longInvince`, `Player.cs:1178-
    /// 1184,2402`): armed per tick in the fold from items 963/984
    /// (`:15060`/`:15064`), 6189 (`:14633`), 3223 (`:15087` — the C# carries
    /// the `Item` reference, the port's bool is the armed-flag equivalent) and
    /// buff 59 (`:10075-10077`); `long_invince` arms from the Star Veil 862
    /// (`:15147`, S1-C-7 wave B) — its other sources (6187 et al.) ride later
    /// waves. `ResetEffects` clears them every
    /// frame (`:18440/:18617/:18661-18662`/`:18719`) and the `Hurt` roll block
    /// (`:38214-38241`) consumes them.
    pub black_belt: bool,
    pub mystic_sash_dodge: bool,
    pub brain_of_confusion: bool,
    pub shadow_dodge: bool,
    pub long_invince: bool,
    /// `Player.magicCuffs` (`Player.cs:2248`): the Magic Cuffs (1595,
    /// `ApplyEquipFunctional` `:14936-14939`) and the Celestial Cuffs (2221,
    /// `:14949-14953`) arm it per tick; `ResetEffects` clears it (`:18599`).
    /// `Hurt` (`:38291-38303`) restores the RAW incoming damage (the C# `num`,
    /// pre-defense, crit-doubled) as mana, capped at `statManaMax2`, and calls
    /// `ManaEffect` (`:4531-4538`) — the port queues the HealMana combat text.
    /// The items' +40 `statManaMax2` stat rows ride C-12 (the accessory stat
    /// chain); 2221's `manaMagnet` half rides `mana_magnet` below (S1-P-i52).
    pub magic_cuffs: bool,
    /// `Player.goldRing` (`Player.cs:2642`) — the Gold Ring family's coin
    /// grab-range/pull flag: 3033 Gold Ring (`:14980-14983`), 3034 Coin Ring
    /// (`:14984-14989`, + the lucky coin), 3035 Greedy Ring (`:14990-14996`,
    /// + the lucky coin + `discountEquipped` — the discount half has no port
    /// consumer yet, the shop-price lane). `ResetEffects` clears it
    /// (`:18504`); the consumers are `GetItemGrabRange` (`:35133`) and
    /// `GrabItems`' pull arm (`:34961`).
    pub gold_ring: bool,
    /// `Player.hasLuckyCoin` (`Player.cs:2638`) — the Lucky Coin family's
    /// melee-hit coin proc flag: 855 Lucky Coin (`:14975-14979`), 3034 Coin
    /// Ring (`:14987`), 3035 Greedy Ring (`:14993`). `ResetEffects` clears it
    /// (`:18760`). The consumer is `ApplyNPCOnHitEffects`' tail
    /// (`:44537-44549`): `npc.value > 0 && hasLuckyCoin && Next(5) == 0`
    /// drops 1-10 coins (the 1/10 silver + 1/100 gold upgrades). The
    /// `hasLuck_LuckyCoin` luck twin (`:2650`) is always set/cleared with
    /// this flag, so one bool carries both — the +0.05 equipment-luck add
    /// is `update_equips`' tail gate (`:13066-13069`).
    pub has_lucky_coin: bool,
    /// `Player.manaMagnet` (`Player.cs:932`): the Celestial Magnet 2219
    /// (`:14940-14943`), the Celestial Emblem 2220 (`:14944-14948`), the
    /// Celestial Cuffs 2221 (`:14949-14953`) and the Magnet Flower 4000
    /// (`:14455-14458`) arm it per tick; `ResetEffects` clears it (`:18491`).
    /// `GetItemGrabRange` adds `Item.manaGrabRange = 300` for the star family
    /// (`:35137-35140`, `Item.cs:49963`) and `GrabItems` pulls stars at
    /// speed 12 / acc 5 (`:34943-34946`).
    pub mana_magnet: bool,
    /// `Player.lifeMagnet` (`Player.cs:934`): the Heartreach Potion's buff 105
    /// arms it (`UpdateBuffs` `:10138-10141`); `ResetEffects` clears it
    /// (`:18492`). `GetItemGrabRange` adds `Item.lifeGrabRange = 250` for the
    /// heart family (`:35145-35148`, `Item.cs:49964`) and `GrabItems` pulls
    /// hearts at speed 15 / acc 5 (`:34949-34952`).
    pub life_magnet: bool,
    /// `Player.latestManaPotionDamageBonus` / `latestManaPotionDuration`
    /// (`Player.cs:982-984`): `TryApplyManaHeat` (`:52006-52014`) writes them
    /// on every mana-star pickup (`PickupItem` `:35071`) — power 0.2, time 2 s
    /// (120 ticks). The buff-396 effect arm (`:10328-10341`, the `manaHeat`
    /// decay) and its `magicDamage *= manaHeat` consumer (`:25906`) are the
    /// UpdateBuffs/damage-recalc census's seam — deferred (ANALYSIS
    /// "Heart & star pickups (S1-P-i52)").
    pub latest_mana_potion_damage_bonus: f32,
    pub latest_mana_potion_duration: f32,
    /// The S1-C-7 wave B retaliate family (`Player.Hurt`, `Player.cs:38401-
    /// 38517`). The C# carries `Item` references (`starCloakItem`,
    /// `honeyCombItem`, the four `starCloakItem_*OverrideItem`s); the port
    /// folds them to armed bools (the `brain_of_confusion` convention) — the
    /// references only feed `GetProjectileSource_Accessory`, which the port
    /// has no entity-source model for. The cloak's star type defaults to 726
    /// and the override checks run in the C#'s order starVeil (725) →
    /// beeCloak (724) → manaCloak (723) → snakeCloak (1133), LAST wins
    /// (`:38437-38456`). Arming rows: 532 (`:15585`), 862 starVeil +
    /// longInvince (`:15146-15148`), 1247 beeCloak + honeyComb
    /// (`:14961-14963`), 4001 manaCloak + manaFlower (`:14461-14462`), 6186
    /// snakeCloak (`:14622-14623`); the honeyComb family 1132 (`:15074`),
    /// 1578 (+panic, `:15078`), 1249 (`:14881`), 3251 (`:14874`), 6184
    /// (`:14613`); strongBees 3333 (`:15346`). `ResetEffects` clears:
    /// `:18322` (strongBees), `:18386` (honeyCombItem), `:18714-18718` (the
    /// cloak + overrides).
    pub star_cloak: bool,
    pub star_veil_override: bool,
    pub bee_cloak_override: bool,
    pub mana_cloak_override: bool,
    pub snake_cloak_override: bool,
    pub honey_comb: bool,
    pub strong_bees: bool,
    /// `Player.shadowDodgeTimer` (`Player.cs:1184`): the Hallowed set-bonus
    /// cooldown — set 1800 by a shadow dodge (`:38043`), decayed one per tick
    /// in `Player.Update` (`:25461-25464`).
    pub shadow_dodge_timer: i32,
    /// `Player.brainOfConfusionDodgeAnimationCounter` (`Player.cs:1158`): the
    /// 300-tick animation window a brain dodge opens (`:38049`); decayed in
    /// `ResetEffects` (`:18794-18797`), zeroed on death (`UpdateDead`,
    /// `:16932`). No render consumer yet — carried for state parity (the
    /// S1-A-9 dead-slot convention).
    pub brain_of_confusion_dodge_animation_counter: i32,
    /// `Player.creativeGodMode` (`Player.cs:1896`): refreshed every tick from
    /// the journey `GodmodePower` (`:25493-25497`; `ResetEffects` clears it at
    /// `:18789`). `Hurt` (`:38203`), `KillMe` (`:38809`) and
    /// `Update_NPCCollision` (`:31321`) all return early while it is set, and
    /// `Main.UpdateWorld_Players` refills the three pools
    /// (`Main.cs:17869-17873`).
    pub creative_god_mode: bool,
    /// S1-E-34 — the F3 panel's debug god mode. `Player.ResetEffects`
    /// rewrites `creative_god_mode` from the journey power every tick
    /// (`Player.cs:18789` / `:25493-25497`), so a debug toggle needs its own
    /// field: the tick ORs it into `creative_god_mode`, which keeps the three
    /// existing god-mode gates (`Hurt`, `KillMe`, `Update_NPCCollision` and
    /// the pool refill) working unchanged.
    pub debug_god_mode: bool,
    /// `Player.immune` (`Player.cs:2636`-ish): the spawn i-frames flag —
    /// true while `immuneTime` counts down (`UpdateImmunity`,
    /// `Player.cs:18856-18869`).
    pub immune: bool,
    /// `Player.immuneTime`: 180 on the death revive, 60 on the other spawn
    /// contexts (`Player.cs:37661-37668`).
    pub immune_time: i32,
    /// `Player.immuneNoBlink` (`:37669-37671`): true while the i-frames run
    /// and the player is not hostile (the PVP flag — always false here).
    pub immune_no_blink: bool,
    /// `Player.tileRangeX/Y`'s journey half (`Player.cs:18815-18828`): the
    /// `FarPlacementRangePower` toggle that doubles the default 5/3 range.
    pub far_placement_range: bool,
    /// Remaining breath in ticks (`Player.breath`).
    pub breath: i32,
    breath_cd: i32,
    /// The life/mana regen accumulators (`Player.lifeRegen`/`lifeRegenTime`/
    /// `lifeRegenCount`/`manaRegen*`, `Player.cs:1930-1946`) — `regen.rs`.
    pub regen: RegenState,
    /// The pending DoT death reason: the drain engine's `KillMe` attribution
    /// (`Player.cs:19319-19334,19361-19384`) rides with the player until the
    /// death pipeline (world_sim) resolves the epitaph text and clears it.
    pub dot_death: Option<DotDeath>,
    /// The luck value + its decay timers (`Player.luck`/`coinLuck`/
    /// `ladyBugLuckTimeLeft`/`torchLuck`/`luckPotion`/`kiteLuckLevel`/…,
    /// `Player.cs:3084-3096` etc.) — `luck.rs`, rebuilt every tick by
    /// `RecalculateLuck` (`Player.cs:29137-29164`).
    pub luck_state: LuckState,
    /// `Player.yoyoHitCounter` (`Player.cs:1220`: `= -1`): the lifetime
    /// yoyo-hit counter behind `EverySomeYoyoHits` (`Player.cs:4308-4316`)
    /// — the M4-PROJ-4b on-hit effects (the damage doublings and the
    /// Cascade/Chik/Kraken/Eye-of-Cthulhu spawns). Vanilla never resets it
    /// (no `ResetEffects` arm, no death/throw reset), so the port doesn't
    /// either — the -1 start makes the first hit skip every effect.
    pub yoyo_hit_counter: i32,
    /// `Player.revolverCritChanceBonus` (`Player.cs:2456`): the Revolver 2269's
    /// hold-to-charge crit bonus. The per-tick `ItemCheck` block
    /// (`Player.cs:42677-42703`) charges it while the button is released mid-
    /// swing (+1 per cancelled tick) and decays it −2 on a 1/3 `Main.rand.Next(3)`
    /// roll otherwise — the roll fires EVERY tick for EVERY held item (the
    /// non-2269 else arm, `:42700-42703`), so the draw rides the fixed-step
    /// `UpdatePlayers` stream unconditionally (`world_sim/player.rs`). The clamp
    /// (`:42697-42698`) bounds `item.crit + bonus` to `[crt, 20 + crt]` (the
    /// prefix's crit add). The shot reads it through the spawn source
    /// (`Projectile.ApplyStatsFromSource`, `Projectile.cs:11129-11132`); the port
    /// folds it into the throw-time crit roll (`world_sim/editing.rs`).
    pub revolver_crit_chance_bonus: i32,
    /// The `Player.ZoneSnow`-family flags the torch-luck table reads
    /// (`Player.cs:17924-18036`); the app copies them from `Main.SceneMetrics`
    /// next to the aura snapshot (`UpdateBiomes`, `Player.cs:16726-16729`).
    pub zones: tr_core::scene::Zones,
    /// The held torch's place style — `inventory[selectedItem].createTile == 4`
    /// (`:17917-17920`), refreshed per tick from `app.inventory[app.selected]`;
    /// `None` when nothing torch-like is held.
    pub held_torch_style: Option<u8>,
    /// `Player.fallStart` (`Player.cs:2882`): the tile row the fall started on,
    /// int-truncated like the original's `(int)(position.Y / 16f)`.
    fall_start: i32,
    /// `Player.extraFall` (`Player.cs:2726`): tiles added to the 25-tile safe
    /// fall - the Amphibian Boots add 10 (`:14403`), reset per tick (`:18788`).
    extra_fall: i32,
    /// `Player.jumpSpeedBoost` (`Player.cs:2724`): the flat jump-speed add
    /// (`UpdateJumpHeight` `:19618`), reset per tick (`:18787`). The Amphibian
    /// Boots add 1.6 (`:14402`); the frog/empress/moon-lord rows are unported.
    jump_speed_boost: f32,
    /// `Player.noFallDmg` (`Player.cs:2766`): the horseshoe/balloon family
    /// (M3-EQUIP-3). Always false until the accessory system lands.
    no_fall_dmg: bool,
    /// `Player.hasWings` (`Player.cs:1336`): any wing equipped (M3-EQUIP-3).
    pub has_wings: bool,
    /// `Player.releaseHook` (`Player.cs:27483-27489`): the "the hook key was
    /// released" latch the quick-grapple edge reads (`controlHook` +
    /// `releaseHook` → `QuickGrapple`).
    pub release_hook: bool,
    /// `Player.releaseThrow` (`Player.cs:1766`, updated `:25248-25255`):
    /// the throw-key release latch — `dropItemCheck`'s
    /// `controlThrow && releaseThrow` edge (`:5147`) reads LAST tick's
    /// value, so the drop fires on the press tick only. The field default
    /// is `false` (C# bool), the port starts `true` like `release_hook`
    /// so a key already held at world entry still edges once.
    pub release_throw: bool,
    /// `Player.JustDroppedAnItem` (`Player.cs:2426`): set by
    /// `DropSelectedItem` (`:5213`), cleared when `itemAnimation` hits 0
    /// (`:43338-43339`). Gates the held-item light/particle emissions
    /// (`:42837`/`:42847`).
    pub just_dropped_an_item: bool,
    /// `Player.grappling[0] != -1` (the per-tick rebuilt grapple state): the
    /// hook is attached and `GrappleMovement` drives the velocity.
    pub grappling_active: bool,
    /// `Player.GoingDownWithGrapple` (`Player.cs:808`): the pull points
    /// downward — the platforms fall through.
    pub going_down_with_grapple: bool,
    /// `Player.MountFishronSpecialCounter` (`Player.cs:3004`): the Cute
    /// Fishron's 60/420-tick "special" window (`Mount.cs:5001-5008`), one
    /// tick of decay per update (`Player.cs:25625-25628`). While positive it
    /// forces [`Self::mount_fishron_special`] true.
    mount_fishron_special_counter: f32,
    /// `Player.wingTime` / `wingTimeMax` (`Player.cs:1330,1338`): the flight
    /// budget in ticks and its refill cap.
    pub wing_time: f32,
    pub wing_time_max: i32,
    /// `Player.wingsLogic` (`Player.cs:1334`): the gameplay wing id
    /// (`TR_WINGS=<ArmorIDs.Wing>` pins it; the granting items land with
    /// M3-EQUIP-3).
    pub wings_logic: i32,
    /// `Player.rocketTime` / `rocketTimeMax` / `rocketDelay` / `rocketDelay2`
    /// (`Player.cs:2744-2748`): the rocket-boots fuel and its thrust cadence.
    pub rocket_time: i32,
    rocket_time_max: i32,
    pub rocket_delay: i32,
    rocket_delay2: i32,
    /// W4-c14 - `Player.rocketSoundDelay` (`Player.cs:2752`): the rocket
    /// cue's own cadence (`:26884-26896` arms it 30/15 per vanity tier,
    /// `:26903-26905` ticks it down once per flight tick).
    rocket_sound_delay: i32,
    /// `Player.rocketBoots` (`Player.cs:2756`): the boots tier
    /// (`TR_BOOTS=<1..5>` pins it).
    pub rocket_boots: i32,
    /// W4-c14 doc-sweep - `Player.vanityRocketBoots` (`Player.cs:2760`): the
    /// VISUAL boots tier the rocket cue picks its sound from
    /// (`:26886`/`:2691`). The functional equip rows set it equal to
    /// `rocketBoots` (`:14420`/`:15023`/`:15222`/`:15380-15400`); the vanity
    /// pass (`UpdateBootVisualEffects`, `:36704-36751`) re-derives it from
    /// the visible boot and the Red's Leggings 668 sets tier 6
    /// (`:13088-13091`). ResetEffects clears it (`:18354`), the mount block
    /// zeroes it (`:26630`).
    pub vanity_rocket_boots: i32,
    /// `Player.canRocket` / `rocketRelease` (`Player.cs:2762,2752`): the
    /// midair arming and the release-jump latch the trigger reads.
    can_rocket: bool,
    rocket_release: bool,
    /// (Gate merge i40 ⨉ i41: `empress_brooch` is declared ONCE, with the
    /// mount block above — the i40 duplicate folded into it.)
    /// `Player.merman` (`Player.cs:2368`): the Neptune's-Shell / Moon-Charm /
    /// Celestial-Shell wet transformation — gates `CanUseWingAbilities`
    /// (`:3990-3999`) and `WingAirLogicTweaks` (`:26496`). Derived per tick
    /// (`:25577-25589`) from `acc_merman` + the wet state.
    pub merman: bool,
    /// `Player.accMerman` (`Player.cs:2366`): the shell accessory grant
    /// (861/3110/497, `:15134-15143`/`:15402-15410`/`:15566-15573`), read by
    /// the merman derivation then cleared (`:25598`).
    acc_merman: bool,
    /// `Player.timeSinceLastDashStarted` (`Player.cs:1050`): ticks since the
    /// last dash start, capped at 300 (`:24600-24604`); the Long Trail
    /// Rainbow wings' (45) run-slowdown ×6 gates on >= 60 (`:29324-29327`).
    time_since_last_dash_started: i32,
    /// The cart mount the Track Hook hands the player to
    /// (`Player.cs:22676-22679`): the equipped cart (the misc-equip 2) when
    /// it is a cart row, else the wooden minecart (13). The app refreshes it
    /// per tick from the misc-equip slot.
    pub cart_mount: i32,
    /// `Player.RemoveAllGrapplingHooks` (`Player.cs:33562-33566`) — the
    /// request the caller applies to the hook pool. The C# calls it from the
    /// jump-off (`Player.cs:22733`), the sign interaction (`:33379`), the
    /// minecart-hook launch (`:33557`) and the TrackHook→cart transfer
    /// (`:22675`); the jump-off is the one the port models here.
    pub release_grappling_hooks: bool,
    /// `Player.releaseJump` (`Player.cs`): jump was released since the last
    /// press — the wing/rocket refills on the ground gate on it.
    release_jump: bool,
    /// `Player.tryKeepingHoveringDown/Up` (`Player.cs:1818-1820`): the
    /// press-down hover latches.
    try_keeping_hovering_down: bool,
    try_keeping_hovering_up: bool,
    down_prev: bool,
    /// `Player.doubleTapCardinalTimer[0]` (`Player.cs:25022-25062`) — the
    /// down-key double-tap window (15 ticks). S1-P-i27: the Forbidden
    /// set-bonus rest aim fires on the down double-tap (`KeyDoubleTap(0)`).
    /// The port models the down cardinal only (the up/left/right cardinals
    /// drive the dash / other set bonuses, unported here).
    double_tap_down_timer: i32,
    /// `Player.holdDownCardinalTimer[0]` (`Player.cs:25064-25072`) — the
    /// down-key hold counter. S1-P-i27: the Stardust/Forbidden set-bonus
    /// rest-target clear fires at holdTime >= 60 (`KeyHoldDown(0, ..)`).
    hold_down_timer: i32,
    /// `Player.releaseDown` (`Player.cs:25037`) — the down-key release latch
    /// (true while the key is up). The double-tap edge is
    /// `controlDown && releaseDown` (`Utils.JustBecameTrue`).
    release_down: bool,
    /// `Player.hasJumpOption_*` / `canJumpAgain_*` / `isPerformingJump_*`
    /// (`Player.cs:2664-2716`), indexed by [`ExtraJump::index`]. The has
    /// flags re-derive per tick from the equipment; the can flags re-arm on
    /// the ground (`RefreshDoubleJumps`, `Player.cs:22869-22908`).
    pub has_jump_options: [bool; 9],
    pub can_jump_again: [bool; 9],
    performing_jump: Option<ExtraJump>,
    /// `Player.jumpBoost` (`Player.cs:2764`): the balloon family's stat
    /// floor (`UpdateJumpHeight`, `Player.cs:19589-19593`).
    pub jump_boost: bool,
    /// `Player.jumpHeight` / `Player.jumpSpeed` (`Player.cs:24366-24367`):
    /// the per-tick statics the jump branches read.
    jump_height: i32,
    jump_speed: f32,
    /// `Player.mount` (`Player.cs:2126`): the mount instance state — the data
    /// table and the state core live in the `mount` module.
    pub mount: crate::mount::MountState,
    /// `Player.autoJump`: the vanilla auto-jump accessibility setting; the
    /// port has no settings surface, so this carries only the mount's
    /// per-tick head-set (`Mount.UpdateEffects`, `Mount.cs:4797`).
    auto_jump: bool,
    /// `Player.justJumped` (`Player.cs:2722`): set on a grounded jump trigger
    /// (`:20678`), cleared in `ResetEffects` (`:18786`); the wing/rocket
    /// refills read it for the auto-jump (constant-jump) mounts.
    just_jumped: bool,
    /// The live hitbox height — `Player.height`: `HEIGHT` (42) normally,
    /// `42 + heightBoost` while mounted (`SetMount` `Mount.cs:6264-6277`).
    pub height: f32,
    /// The live hitbox width — `Player.width` (`Entity.width`): `WIDTH` (20)
    /// normally; the transformation mounts' `PlayerSize` delegates shrink it
    /// (Rat 14, Pixie 8 — `Mount.cs:6244-6263`, `DelegateMethods.cs:164-194`),
    /// and `Dismount` restores 20 (`Mount.cs:6176-6193`). The sim's collision,
    /// centre and mount probes read it; the cross-system contact rects
    /// (`npc.rs` et al.) still read the `WIDTH` constant (ANALYSIS "Mount
    /// hitbox width (S1-P-i41)").
    pub width: f32,
    /// `Player.lastStoned` (`Player.cs`): the petrify-transition detector.
    last_stoned: bool,
    /// `Player.dashType` (`Player.cs:1044`): the equipped dash capability.
    /// The accessory granters (Tabi 977 `:15052`, EoCShield 3097 `:15056`,
    /// Master Ninja Gear 984 `:15065`) arm it in `apply_equip_functional`;
    /// the Solar set (type 3) rides the set rows and the Chillet mounts
    /// (type 6) their own task. `ResetEffects` clears it per tick (`:18659`);
    /// `TR_DASH=<1|2|3|6>` pins it through `pinned_dash`.
    pub dash_type: i32,
    /// `Player.dash` (`:1046`): the committed dash type of the lifecycle.
    pub dash: i32,
    /// `Player.dashTime` (`:1048`): the signed 15-tick double-tap window.
    dash_time: i32,
    /// `Player.dashDelay` (`:1052`): 0 ready, -1 dashing, >0 cooldown.
    pub dash_delay: i32,
    /// `Player.eocDash` / `eocHit` (`:1054,1056`): the Shield of Cthulhu
    /// post-hit grace (the struck-NPC exclusion + the drain on cooldown).
    pub eoc_dash: i32,
    eoc_hit: i32,
    /// The previous-tick direction keys — the press edges the double-tap
    /// reads (`Player.cs:27618-27635` recomputes the release flags at the
    /// end of the tick).
    right_prev: bool,
    left_prev: bool,
    /// `Player.maxRunSpeed` / `runAcceleration` / `runSlowdown` /
    /// `accRunSpeed` (`Player.cs:24364-24371`): the per-tick horizontal
    /// defaults the `HorizontalMovement` branches and the floor modifiers ride.
    max_run_speed: f32,
    run_acceleration: f32,
    run_slowdown: f32,
    acc_run_speed: f32,
    /// `Player.moveSpeed` / `strongestMoveSpeedDebuff` (`Player.cs:2464,2462`):
    /// 1.0 until accessories/debuffs land (M3-EQUIP-3, M2-BUFF-4).
    pub(crate) move_speed: f32,
    strongest_move_speed_debuff: f32,
    /// `Player.iceSkate` / `desertBoots` (`Player.cs:1088,2256`): Ice Skates /
    /// Dunerider Boots (M3-EQUIP-3 — no accessory system yet).
    ice_skate: bool,
    desert_boots: bool,
    /// The `FloorVisuals` flags (`Player.cs:23708-23743`), sampled after the
    /// tick's movement — one tick stale relative to the movement block,
    /// exactly like the original's call order (`:26500` vs `:28271`).
    sticky: bool,
    slippy: bool,
    slippy2: bool,
    powerrun: bool,
    running_on_sand: bool,
    /// `Player.stickyBreak` (`Player.cs:2301`): the cobweb (51) break counter
    /// `StickyMovement` accumulates against the `Next(20,100)` /
    /// `Next(80,300)` threshold (`:22938-22954`); reset when no web is touched
    /// (`:23085`).
    sticky_break: i32,
    /// `Player.suffocateDelay` (`Player.cs:2318`): the tick counter the
    /// suffocating tiles (the sands) advance — the sixth tick in the sand
    /// suffocates (`Player.ApplyTouchDamage`, `Player.cs:28996-29010`).
    pub suffocate_delay: u8,
    /// The liquid displacement fraction `Player.WetCollision` integrates free
    /// axes with (`Player.cs:23436-23451`): water/lava 0.5, honey 0.25
    /// (`Player.cs:28168-28195`), dry 1.0.
    liquid_move_speed: f32,
    pub in_lava: bool,
    /// `Player.gfxOffY`: visual offset that hides the instant lift of `StepUp` /
    /// `StepDown`; eases back to zero so steps and slopes look smooth.
    pub gfx_off_y: f32,
    /// `Player.stepSpeed`: how fast `gfxOffY` eases out.
    step_speed: f32,
    /// `Player.stairFall`: slopes may be fallen through this tick.
    stair_fall: bool,
    /// `Player.mount.Active && Mount.Minecart` — the base-cart ride state
    /// (`MountID.Sets.Cart` contains 6, `MountID.cs:43`). The full Mount
    /// framework is M2-MOUNT-1; `TR_CART` pins this state for tests and
    /// screenshots.
    pub cart_mounted: bool,
    /// `Player.cartFlip` (`Player.cs:1786`): the bouncy-bumper sprite flip,
    /// toggled on each bounce while a direction key is held
    /// (`Player.cs:28123-28130`).
    pub cart_flip: bool,
    /// `Player.unlockedSuperCart` (`Player.cs:2052`): the Super Cart is
    /// unlocked by the item 3353 (`Player.cs:45207-45211`, `:55718`) — that
    /// writer is not ported yet, so the flag stays false in play.
    pub unlocked_super_cart: bool,
    /// `Player.enabledSuperCart` (`Player.cs:2054`, default true): the
    /// player's Super Cart toggle. The UI that flips it is not ported yet.
    pub enabled_super_cart: bool,
    /// `Player.onTrack` (`Player.cs:2811`): this tick's `Minecart.TrackCollision`
    /// result bit 0 — the body rides a rail. Couples the collision heights
    /// (pitfall §10.9).
    pub on_track: bool,
    /// `Player.onWrongGround` (`Player.cs:2455`): a mounted cart off the rails —
    /// set by the on-track check (`Player.cs:26665-26674`), preserved across the
    /// tick for carts (`:24371-24374`).
    pub on_wrong_ground: bool,
    /// `Player.trackBoost` (`Player.cs:1780`): the pending booster push, applied
    /// pre-input next tick and clamped (`Player.cs:19818-19833`).
    pub track_boost: f32,
    /// `Player.lastBoost` (`Player.cs:1782`): the booster tile a boost is pending
    /// from — tile coords, `(0, 0)` = none (`Minecart.cs:832-847`, `:894-897`).
    pub last_boost: (i32, i32),
    /// `Player.cartRampTime` (`Player.cs:1802`): the gravity-freeze ticks a ramp
    /// launch grants (`Player.cs:28148-28150`), consumed by the gravity gate
    /// (`:27352-27359`).
    pub cart_ramp_time: i32,
    /// `Player.fullRotation` / `fullRotationOrigin` (`Player.cs:1838-1840`): the
    /// rail-slope tilt from `Minecart.TrackRotation` (`Player.cs:28114-28115`).
    pub full_rotation: f32,
    pub full_rotation_origin: (f32, f32),
    /// `Player.itemRotation` (`Player.cs:1836`): the held item's draw
    /// rotation. The shooting styles derive it from the aim each frame
    /// (`:47371-47378`, `FrameInputs.item_rotation`); the flail chain writes
    /// it per tick (`Projectile.cs:44162-44167`, S1-P-i07's `FlailTick`).
    pub item_rotation: f32,
    /// `Player.fallStart2` (`Player.cs:2884`): the fall origin the rail-landing
    /// gate reads (`Minecart.cs:809-813`), captured pre-movement when
    /// `velocity.Y <= 0` (`Player.cs:25295-25297`).
    pub fall_start2: i32,
    /// The `Player.cs:28237-28241` switch trip handed to the caller (the C#
    /// `HitTrackSwitch` needs `&mut World`; the tick takes `&World`).
    pending_switch_trip: Option<(i32, i32)>,
    /// `Player.PlayHurtSound` (`Player.cs:38569-38712`): the hurt cue the
    /// damage path queues (`Player.cs:38525`) — the caller owns the audio
    /// device, so the Player hands the cue over.
    pending_hurt_sound: Option<&'static str>,
    /// `Player.PlayDeathSound` (`Player.cs:38715-38744`): the death cue
    /// `Player.KillMe` queues (`Player.cs:38868`).
    pending_death_sound: Option<&'static str>,
    /// `Player.runSoundDelay` (`Player.cs:1860`): the footstep cooldown —
    /// `SpawnFastRunParticles` sets it to `hermesStepSound.IntendedCooldown`
    /// (`:20215`), `Player.Update` ticks it down (`:24706-24708`).
    pub(crate) run_sound_delay: i32,
    /// `Player.hermesStepSound` (`Player.cs:2572`, `DataStructures/SoundPlaySet`):
    /// the fast-run footstep cue — `(SoundType, SoundStyle, IntendedCooldown)`.
    /// `PlayerFrame` re-derives it every frame: the default 17/-1/9
    /// (`Player.cs:35947-35949`), the Moon Lord legs override 2/24/6
    /// (`:35959-35964`). Vanilla has no per-tile-type footstep cues — the
    /// grass/stone/snow sounds are the tile-break families (wave C).
    pub(crate) hermes_step_sound: (u16, i16, i32),
    /// `Player.SpawnFastRunParticles`' play (`Player.cs:20214`): the queued
    /// `(SoundType, SoundStyle, position.X, position.Y)` — the caller owns
    /// the audio device and the `UpdatePlayers` stream.
    pending_step_sound: Option<(u16, i16, f32, f32)>,
    /// S1-E-21f — the extra-jump chain's `PlaySound(16)` play
    /// (`Player.cs:20793`/`:20803`/`:20815`/`:20871`/`:20899`/`:20927`/
    /// `:20957`/`:21004`): `(position.X, position.Y)` — the top-left, NOT the
    /// centre. The caller owns the audio device and the `UpdatePlayers`
    /// stream (the `pending_step_sound` convention). The Fart's `SoundID.Item16`
    /// play (`:20846`) is a style façade — a separate queue below.
    pending_double_jump_sound: Option<(f32, f32)>,
    /// S1-E-21f — the Fart-in-a-Jar's `PlaySound(SoundID.Item16, position)`
    /// (`Player.cs:20846`): `(style_index, position.X, position.Y)` — the
    /// wave-A style façade draws its `GetRandomPitch` NextFloat on the
    /// audio-only stream, the case-2 pitch on `UpdatePlayers`.
    pending_fart_sound: Option<(u16, f32, f32)>,
    /// S1-E-21f — the liquid entry/exit splash play (`Player.cs:27806`/
    /// `:27819`/`:27832`/`:27846` entry, `:27900`/`:27913`/`:27926`/`:27940`
    /// exit): `(Style, position.X, position.Y)`. The style is the C#'s third
    /// argument — shimmer 2 entry / 3 exit, water 0, honey/lava the default 1.
    pending_splash_sound: Option<(i32, f32, f32)>,
    /// `Entity.wetCount` (`Entity.cs`): the splash cooldown — set to 10 on
    /// the entry/exit play (`Player.cs:27776`/`:27870`), decremented at the
    /// wet block's tail (`:27959-27961`), zeroed on spawn (`:37636`).
    wet_count: i32,
    /// S1-E-21f — the drowning gasp (`Player.CheckDrowning`'s breath-zero
    /// arm, `Player.cs:23257`: `PlaySound(23)` — unpositioned). A bare flag:
    /// the case draws nothing, the drain plays `Drown` on no stream.
    pending_drown_sound: bool,
    /// S1-E-21f — the wing-flap latch `Player.flapSound` (`Player.cs:1128`):
    /// the `wingFrame == 3` edge plays `SoundID.Item32` once (`:26848-26852`),
    /// the off-frame clears it (`:26856`). The port's flap cadence is the
    /// default arm's `wingFrameCounter` machine (`Player.cs:30466-30479`) —
    /// the hover/special wings' own frame tables are visual seams.
    flap_sound: bool,
    /// `Player.wingFrame` (`Player.cs:1340`) / `wingFrameCounter` (`:1342`):
    /// the default flap machine's state (the `flap_sound` latch above).
    wing_frame: i32,
    wing_frame_counter: i32,
    /// S1-E-21f — the wing flap's `PlaySound(SoundID.Item32, position)`
    /// (`Player.cs:26850`): `(style_index, position.X, position.Y)` — the
    /// wave-A façade (the `GetRandomPitch` NextFloat rides the audio-only
    /// stream; `Item32`'s variance is 0, the draw is still consumed).
    pending_flap_sound: Option<(u16, f32, f32)>,
    /// `Player.ManaEffect` (`Player.cs:4531-4538`): the mana-restore combat
    /// text the cuffs arm queues from `Hurt` (`:38291-38303`) — the caller
    /// owns `Main.combatText`, so the Player hands the amount over (the
    /// `pending_hurt_sound` convention). The C# `CombatText.NewText` rect is
    /// the player's own hitbox at the hurt position.
    pending_mana_effect: Option<i32>,
    /// `Player.HealEffect` (`Player.cs:4522-4529`): the heart pickup's heal
    /// text (`PickupItem` `:35062` → `Heal` `:35117`) — the same queue
    /// convention as `pending_mana_effect`, drained by the items pass (the
    /// pickup site owns the combat-text pool + the `UpdatePlayers` stream).
    pending_heal_effect: Option<i32>,
    /// S1-C-7 wave B: the accepted-hurt retaliation request (`Player.Hurt`'s
    /// `ImmunityCooldownID.Sets.Retaliate` block, `Player.cs:38401-38517`).
    /// `hurt_inner` has no NPC list / projectile pool, so it records the
    /// hit-moment snapshot (final post-floor `num2` + position/center/
    /// velocity BEFORE the knockback `:38519`) and the drain
    /// (`hurt_modifiers::drain_retaliate`, the `world_sim` seams) rolls the
    /// block on the hit lane's stream — the `pending_mana_effect` convention.
    /// The C# gates on the PRE-hit `flag` (not immune) AND the Retaliate-set
    /// membership of the cooldown counter; `hurt_inner`'s `retaliate_lane`
    /// carries the membership (the lava/tile lanes pass false).
    pending_retaliate: Option<crate::hurt_modifiers::RetaliateRequest>,
    /// S1-C-7 wave B: the dismount-on-hit request (`Player.cs:38396-38400`) —
    /// the damage source was a projectile of `ProjectileID.Sets.
    /// DismountsPlayersOnHit` (`ProjectileID.cs:546`, {877, 878, 879}) and a
    /// mount is active. `Mount.TryDismount` needs `&World`, so the tick hands
    /// the request over (the `pending_switch_trip` convention).
    pending_dismount: bool,
    /// S1-P-i41 — the Chillet mount's `SoundID.PalChillet` play
    /// (`Mount.SetMount` `Mount.cs:6283-6286`, `SoundEngine.PlaySound(
    /// SoundID.PalChillet, mountedPlayer.Center)`): `(position.X,
    /// position.Y)` — the case-42 trackable façade draws its NextFloat on
    /// the audio-only stream (the `pending_flap_sound` convention).
    pending_chillet_sound: Option<(f32, f32)>,
    /// S1-P-i41 — the QuickMount no-space ladder's `SoundEngine.PlaySound(0,
    /// (int)position.X, (int)position.Y)` (`Player.cs:6057`): `(x, y)` — the
    /// case-0 Dig play draws its variation + pitch on the `UpdatePlayers`
    /// stream (the `play_place_sound` contract).
    pending_no_space_sound: Option<(f32, f32)>,
    /// W4-c14 - the petrify-transition Dig-0 (`Player.cs:25225`:
    /// `SoundEngine.PlaySound(0, (int)position.X, (int)position.Y)` on the
    /// `stoned != lastStoned` edge, BOTH directions): `(x, y)` - the case-0
    /// variation + pitch draws ride the drain's `UpdatePlayers` stream.
    pending_stoned_sound: Option<(f32, f32)>,
    /// S1-P-i41 — the third-fail `PlayerNoSpaceTeleport` request
    /// (`Player.cs:6061-6066`): the forced dismount ran, the tick owns
    /// `&World` for the spot scan (`TeleportHelpers.FindClosestTeleportSpotNoSpace`).
    pending_no_space_teleport: bool,
    /// S1-P-i41 — the `Shellphone_Spawn` invalid-area force-clear request
    /// (`Player.cs:56169-56172`, `Spawn_ForceClearArea` `:37914-37935`):
    /// `(floor_x, floor_y)` — the kills + liquid wipes need `&mut World` and
    /// the cascade's drop rolls, so the drain runs them (`editing`).
    pending_force_clear: Option<(i32, i32)>,
    /// The wheel-spark rolls' named stream (the per-consumer convention — C#
    /// rolls the shared `Main.rand` at `Player.cs:19873-19901` and inside
    /// `DelegateMethods.Minecart.Sparks`). Seeded from the world seed.
    pub spark_rng: u64,
    /// The tick's queued spark spawn requests, drained by `world_sim` into
    /// the App's `Main.dust` pool (S1-E-1 phase 2).
    pending_sparks: Vec<crate::minecart::SparkSpawn>,
    /// The tick's queued minecart cues (Item53/55/56), drained into
    /// `App::audio::play_at` (`DelegateMethods.cs:248-256`, `Player.cs:19870`).
    pending_cart_sounds: Vec<crate::minecart::CartCue>,
    /// The tick's queued sticky dust: the `Collision.StickyTiles` type-30 web
    /// puff (`Collision.cs:3401`) and `StickyMovement`'s type-153 honey puffs
    /// (`:23036-23080`). The C# calls `Dust.NewDust` inline; the port queues the
    /// position and the drain rolls the pool exactly as the source does, so the
    /// `Next(0,6)` scale roll (`:23037`) stays *after* the `NewDust` chain and
    /// the RNG order is unchanged (S1-A-10). The type-153 `velocity *= 0.1`
    /// and `noGravity` post-arms ride the drain too.
    pending_sticky_dust: Vec<StickyDust>,
    /// `Player.cs:22956` `WorldGen.KillTile(num3, num4)` — the caller owns
    /// `&mut World`, so the tick hands the web tile over (`tr_core::edit::kill_tile`).
    pending_sticky_kill: Option<(i32, i32)>,
    /// `Player.channel` (`Player.cs:52958-52968`): a `channel` item is held
    /// and aiming — set by `StartChanneling`, cleared by `TryCancelChannel`
    /// (`:52975-52981`) and on the release. The aiStyle-9 magic spells (Magic
    /// Missile / Flamelash / Rainbow Rod) read it while aiming
    /// (`Projectile.cs`'s `AI_009_MagicMissiles` owner block).
    /// Written by the item-use path (M4-PROJ-6 Phase 5), read by the arm
    /// (Phase 4); default off.
    pub channel: bool,
    /// The shooting item's kind while `channel` — the held item's id, read by
    /// the Last Prism holder's keep-alive (`registry::item(channel_item)`,
    /// `Projectile.cs:66540-66546`); 0 = no item.
    pub channel_item: u16,
    /// The held item's `shoot` (projectile id) while `channel` — the C# block
    /// holds the aim only while `HeldItem.shoot == this.type`
    /// (`AI_009_MagicMissiles`'s owner block, `Projectile.cs:43241`), so the
    /// aiStyle-9 spells compare THIS against their own kind, not the item id.
    /// 0 = no item. (S1-P-i20: the pre-fix port compared `channel_item` — the
    /// item id — against the projectile kind, so the Magic Missile / Flamelash
    /// channel hold never fired in the real use path.)
    pub channel_shoot: u16,
    /// The aim target the channel feeds the spell: `MouseWorld` clamped by
    /// `LimitPointToPlayerReachableArea` (`Projectile.cs:43241-43249`'s
    /// `ai[0..1] = LimitPointToPlayerReachableArea(MouseWorld)`), in world
    /// pixels. `Vec2::ZERO` while not channelling.
    pub channel_target: Vec2,
    /// `Player._channelShotCache.ProjectileTypeExpected` (`Player.cs:117`,
    /// `:52963-52966`): the held channel item's `shoot` — the projectile
    /// type `TryUpdateChannel` tracks and `TryCancelChannel` matches
    /// (`ChannelCancelKey` `:121-140`). 0 = no channel armed.
    pub channel_proj_expected: u16,
    /// `Player._channelShotCache.ProjectileIndexExpected` (`Player.cs:119`,
    /// `:134-140`): the pool slot of the LAST spawned projectile of the
    /// expected type (`NewProjectile`'s tail `Projectile.cs:11097-11100`).
    /// The C# struct field defaults to 0 (`StartChanneling(Item)` news the
    /// key with only `ProjectileTypeExpected` set, `:52963-52966`), so an
    /// untracked cache matches slot 0 — the port keeps the same 0 default.
    pub channel_index_expected: u16,
    /// `Player.heldProj` (`Player.cs:1516`, stock −1): the slot index of the
    /// held-channel projectile (the golf club helper writes it every tick,
    /// `Projectile.cs:52490`/`:52535`). The port's draw/pose consumers land
    /// with the held-item layers; the field keeps the C#'s write exact.
    pub held_proj: i32,
    // ------------------------------- the ammo mastery layer (M3-ITEM-6)
    // `Player.magicQuiver` (`Player.cs:1968`): the quivers 4002/4006/6181/1321
    // (`:14465`, `:14483`, `:14596`, `:15333`) — the 1/5 roll arm (`:53595`) and
    // the ×1.1 speed fuse (`:53543`). Cleared and re-derived per tick by the
    // grant seam in `world_sim/player.rs` (`ResetEffects` `:18605`).
    pub magic_quiver: bool,
    // ------------------------------- the tooltip's player-side layers (S1-E-20)
    // `Player.minionKB` (`Player.cs`): the summon knockback bonus the
    // tooltip's knockback row adds (`Main.cs:20749`).
    pub minion_kb: f32,
    // `Player.kbGlove` / `kbBuff` (`Player.cs`): the knockback gloves' and
    // buff's bonuses (the tooltip's knockback row, `Main.cs:20333-20340`).
    pub kb_glove: bool,
    pub kb_buff: bool,
    // ---- S1-E-7 wave C: the accessory/set-bonus buff flags
    // (`buff_effects_accessory.rs`, `Player.cs:10800-11400` + `:12013-12044`).
    /// `Player.gross` (`Player.cs:10981`): Horrified (37) while the WoF lives.
    pub gross: bool,
    /// `Player.tongued` (`Player.cs:10993`): Tongued (38), pinned at 10 ticks.
    pub tongued: bool,
    /// `Player.sunflower` (`Player.cs:11003`): the Sunflower-buff flag half
    /// (the movement half rides the effect match below).
    pub sunflower: bool,
    /// `Player.rabid` (`Player.cs:11323`): the Feral Bite (148) flag.
    pub rabid: bool,
    /// `Player.meleeEnchant` (`Player.cs:1068`, arms `:12013-12044`): the
    /// weapon-imbuement id 1..8 (0 = none).
    pub melee_enchant: u8,
    /// The `petFlag*`/light-pet bools (`Player.cs:18413-18600`'s clears),
    /// indexed by the PET PROJECTILE id — the C# flags are per-pet (buff 341
    /// sets the 881 and 934 flags shared with buffs 284/317).
    pub pet_flags: [bool; PROJ_COUNT],
    // `Player.shroomiteStealth` + `stealth` (`Player.cs`): the shroomite
    // armour's ranged knockback scale (`Main.cs:20342-20345`).
    pub shroomite_stealth: bool,
    pub stealth: f32,
    // `Player.spaceGun` (`Player.cs`): the mana row's exemption
    // (`Main.cs:20917`).
    pub space_gun: bool,
    // ---- S1-C-12 wave D: the LOW-tier armor sets' flag halves
    // (`ArmorSetBonuses.Benefits`, the `armor_sets.rs` seam grants them at
    // the `UpdateArmorSets` position, `Player.cs:25685`).
    /// `Player.shadowArmor` (`ArmorSetBonuses.cs:312-315`): the Shadow Scale
    /// set's movement boost (`Player.cs:26511-26517`). `ResetEffects` clears
    /// it (`:18458`).
    pub shadow_armor: bool,
    /// `Player.crimsonRegen` (`ArmorSetBonuses.cs:248-251`): the Crimson
    /// set's regen boost — `lifeRegenTime++` (`Player.cs:19117-19120`) and
    /// the ramp ×1.5 (`:19242-19245`), fed through `RegenInputs`.
    /// `ResetEffects` clears it (`:18722`).
    pub crimson_regen: bool,
    /// `Player.ashWoodBonus` (`ArmorSetBonuses.cs:380-383`): the Ash Wood
    /// set's lava-hit halves (`Player.cs:27694-27702`). `ResetEffects`
    /// clears it (`:18325`).
    pub ash_wood_bonus: bool,
    /// `Player.frostBurn` (`ArmorSetBonuses.cs:271`): the Frost set's
    /// on-hit Frostburn flag — the melee consumer is
    /// `melee_on_hit.rs::status_to_npc` (S1-P-i10, `Player.cs:6548-6551`);
    /// the projectile consumers stay unported (ANALYSIS "S1-C-12 wave D").
    /// `ResetEffects` clears it (`:18686`).
    pub frost_burn: bool,
    /// `Player.magmaStone` (`Player.cs:18610`'s clear): the Magma Stone
    /// 1322 (`:15336-15339`), the Fire Gauntlet 1343 (`:15313`) and the
    /// Pyroclast 6178 (`:14580-14583`) arm it; the melee on-hit consumer
    /// is `melee_on_hit.rs::status_to_npc` (`:6552-6566`, S1-P-i10).
    /// `ResetEffects` clears it every tick (`:18610`).
    pub magma_stone: bool,
    /// `Player.catalystBand` (`Player.cs:18608`'s clear): the Catalyst
    /// Band 6185 (`:14615-14619`) arms it; the melee on-hit consumer is
    /// `melee_on_hit.rs::status_to_npc` (`:6536-6539`, S1-P-i10 —
    /// AcceleratePoisons 398). `ResetEffects` clears it (`:18608`).
    pub catalyst_band: bool,
    /// `Player.vampireBurningInSunlight` (`Player.cs:28709`): the vampire
    /// seed's sun-scorch flag — the gate on the Molten set's
    /// `buffImmune[24]` (`ArmorSetBonuses.cs:285-288`). The granter
    /// (`VampireSeedSunlightExposure`, `:28672-28719`) is unported, so the
    /// flag stays false in play; `ResetEffects` clears it (`:18730`).
    pub vampire_burning_in_sunlight: bool,
    /// `Player.hasMoltenQuiver` (`Player.cs:1974`): 4002/6181 (`:14467`,
    /// `:14596`) — converts the wooden arrow to the FireArrow and adds 2 ammo
    /// damage (`:53537-53541`).
    pub has_molten_quiver: bool,
    /// `Player.accSharpBarb` (`Player.cs:1224`): 6160 Sharp Barb (`:14501`) —
    /// +1 ammo damage on arrows (`:53548`).
    pub acc_sharp_barb: bool,
    // S1-P-i35 — the fishing family (`Player.cs:1088-1102` region).
    /// `Player.fishingSkill` (`Player.cs:1092`): the +15 Fishing potion
    /// (buff 121, `UpdateBuffs` `:10146-10149`), the Angler armor pieces
    /// (+5 each, `GrantArmorBenefits` `:13447-13450`), the earring/tackle
    /// combos (+10, `:14655-14683`), the bobber accessory (+10, `:12985-
    /// 12988`) and the Gummy Worm (+3, `:12857-12860`). `ResetEffects`
    /// clears it (`:18507`).
    pub fishing_skill: i32,
    /// `Player.cratePotion` (`Player.cs:1096`): buff 123 (`:10154-10157`) —
    /// the crate roll's +15 window (`Projectile.cs:21520-21523`).
    /// `ResetEffects` clears it (`:18508`).
    pub crate_potion: bool,
    /// `Player.sonarPotion` (`Player.cs:1094`): buff 122 (`:10150-10153`) —
    /// the sonar popup on a bite (`Projectile.cs:20746-20751`; the popup
    /// layer is a draw seam, the flag rides). `ResetEffects` clears it
    /// (`:18509`).
    pub sonar_potion: bool,
    /// `Player.accTackleBox` (`Player.cs:1098`): 2375/3721/5064
    /// (`:14663-14683`) — the bait conservation +1 (`:52483-52486`).
    /// `ResetEffects` clears it (`:18510`).
    pub acc_tackle_box: bool,
    /// `Player.accFishingBobber` (`Player.cs:1100`): the glow-bobber
    /// accessories 5139-5146 (`:14659-14662`) — +10 skill (`:12985-12988`).
    /// `ResetEffects` clears it (`:18511`).
    pub acc_fishing_bobber: bool,
    /// `Player.accFishingLine` (`Player.cs:1102`): 2373/3721/5064
    /// (`:14651-14683`) — the 1-in-7 line snap never fires (`:52438`).
    /// `ResetEffects` clears it (`:18512`).
    pub acc_fishing_line: bool,
    /// `Player.accLavaFishing` (`Player.cs:1104`): 4881/5064 (`:14667-
    /// 14683`) — the lava-fishing gate (`Projectile.cs:20637`).
    /// `ResetEffects` clears it (`:18513`).
    pub acc_lava_fishing: bool,
    /// `Player.overrideFishingBobber` (`Player.cs:1106`): the glow-bobber
    /// accessory's projectile override (`UpdateFishingBobber`, `:36673-
    /// 36702`: items 5139-5146 → 986-993), read by `ItemCheck_Shoot`
    /// (`:47181-47184`). Reset to -1 (`:16990`/`:18410`).
    pub override_fishing_bobber: i32,
    /// `Player.usedGummyWorm` (`Player.cs:2074`): the 5341 permanent +3
    /// fishing skill (`:45411-45416`, `:12857-12860`). NOT cleared by
    /// ResetEffects — a permanent unlock (the player-file field).
    pub used_gummy_worm: bool,
    /// (Gate merge i35 ⨉ i41: `can_float_in_water` and `no_items` are
    /// declared ONCE, with the mount block above — the i35 duplicates
    /// folded into them; the fishing float bonus / bobber kill gate read
    /// the same fields.)
    /// `Player.accSentryBackpack` (`Player.cs:1250`): the Heavy Sling family
    /// (6172/6175/6176/6177, `ApplyEquipFunctional` `:14507`/`:14565`/`:14573`/
    /// `:14577`) — the sentry-backpack pickup gate
    /// (`Projectile.CanGetPickedUpBySentryBackpack`, `Projectile.cs:78796`).
    /// `ResetEffects` clears it (`:18481`). The 6176 row's
    /// `accGolemSentryBackpack` half (`:14574`) has no port consumer yet
    /// (the Golem-backpack crit arm `Projectile.cs:12930` is unported).
    pub acc_sentry_backpack: bool,
    /// `Player.accSentryBackpackGrabCooldown` (`Player.cs:1254`): the 2-tick
    /// grab lockout `SetAsSelectedSentryBackpackTarget` sets
    /// (`Projectile.cs:78809`), decremented by `Update_SentryBackpack`
    /// (`Player.cs:28428-28434`). NOT cleared by `ResetEffects` — a countdown.
    pub acc_sentry_backpack_grab_cooldown: i32,
    /// `Player.ballistaPanic` (`Player.cs:2424`): buff 205 (Ballista Panic,
    /// `UpdateBuffs` `:11843-11846`) — the Ballista's 60/30-tick reload caps
    /// (`Projectile.cs:68563-68570`, `GetBallistraShotDelay` `:68660-68667`).
    /// `ResetEffects` clears it (`:18740`).
    pub ballista_panic: bool,
    /// `Player.setSquireT2`/`setSquireT3` (`Player.cs:2840`/`:2832`): the
    /// Squire armor sets (`ArmorSetBonuses.cs:68-72`/`:92-97`, the triples
    /// `:484`/`:488`) — the Ballista's 21-px shot speed and 100/30-tick
    /// reloads (`Projectile.cs:68521-68524`, `:68656-68667`). `ResetEffects`
    /// clears both (`:18747`/`:18751`).
    pub set_squire_t2: bool,
    pub set_squire_t3: bool,
    /// `Player.setHuntressT2`/`setHuntressT3` (`Player.cs:2842`/`:2834`): the
    /// Huntress armor sets (`ArmorSetBonuses.cs:80-84`/`:106-111`, the triples
    /// `:486`/`:490`) — the Explosive Trap's 60/30-tick cooldowns
    /// (`GetExplosiveTrapCooldown`, `Projectile.cs:69376-69388`) and the
    /// blast's Oiled debuff (`StatusNPC`, `:11434-11437`). `ResetEffects`
    /// clears both (`:18746`/`:18750`).
    pub set_huntress_t2: bool,
    pub set_huntress_t3: bool,
    /// `Player.setMonkT2`/`setMonkT3` (`Player.cs:2846`/`:2838`): the Monk
    /// armor sets (`ArmorSetBonuses.cs:86-90`/`:113-118`, the triples
    /// `:487`/`:491`) — the Lightning Aura's 25-tick zap period and its
    /// 14-tile/8-tile scan band (`Projectile.cs:68862-68870`). `ResetEffects`
    /// clears both (`:18748`/`:18752`).
    pub set_monk_t2: bool,
    pub set_monk_t3: bool,
    /// `Player.setApprenticeT2`/`setApprenticeT3` (`Player.cs:2844`/`:2836`):
    /// the Apprentice armor sets (`ArmorSetBonuses.cs:74-78`/`:99-104`, the
    /// triples `:485`/`:489`) — the Flame Burst Tower's 0.1 angle cone, the
    /// 1.5× range and the 1.4× shot speed (`Projectile.cs:68275-68280`).
    /// `ResetEffects` clears both (`:18753`/`:18749`). S1-D-4b-9.
    pub set_apprentice_t2: bool,
    pub set_apprentice_t3: bool,
    /// `Player.ammoBox` (`Player.cs:1982`): buff 93, the Ammo Box (`:10080`) —
    /// the 1/5 roll arm on ANY ammo (`:53599`).
    pub ammo_box: bool,
    /// `Player.ammoPotion` (`Player.cs:1984`): buff 112, the Ammo Reservation
    /// (`:10195`) — the 1/5 roll arm on ANY ammo (`:53603`).
    pub ammo_potion: bool,
    /// `Player.huntressAmmoCost90` (`Player.cs:1960`): 3804/4983 (`:13179`,
    /// `:13634`) — the 1/10 roll arm (`:53643`).
    pub huntress_ammo_cost90: bool,
    /// `Player.chloroAmmoCost80` (`Player.cs:1958`): 1002 (`:13649`) — the 1/5
    /// roll arm (`:53647`).
    pub chloro_ammo_cost80: bool,
    /// `Player.ammoCost80` (`Player.cs:1962`): 1549/3878 (`:13715`, `:13222`) —
    /// the 1/5 roll arm (`:53651`).
    pub ammo_cost80: bool,
    /// `Player.ammoCost75` (`Player.cs:1964`): 2758 (`:13908`) — the 1/4 roll
    /// arm (`:53655`).
    pub ammo_cost75: bool,
    /// `Player.archery` (`Player.cs:2286`): buff 16, the Archery Potion
    /// (`:10018`) — the ×1.2 speed fuse capped at 20 (`:53560`).
    pub archery: bool,
    // `Player.ammoCyclingMode`/`ammoCyclingOffset`/`ammoCyclingCooldown`
    // (`Player.cs:1244`, `:1248`, `:1246`): the cycling state.
    pub ammo_cycling: crate::ammo::AmmoCycling,
    // `Player.ammoCyclingMode`/`ammoCyclingOffset`/`ammoCyclingCooldown`
    // (`Player.cs:1244`, `:1248`, `:1246`): the cycling state — the mode is
    // granted per tick by the accessories 6168/6169/6193/6194 (`:14528-14544`),
    // the offset/cooldown are written inside a successful pick (`:53569-53577`)
}

/// One summon keep-alive arm of `Player.UpdateBuffs`' chain
/// (`Player.cs:10483-10947`): the buff id, the projectile kinds whose live
/// presence re-affirms the buff, and the `ResetEffects` flag the arm sets
/// (`Player.cs:18620-18646`+`:18741` clear, the arm sets true).
struct SummonBuffArm {
    buff: i32,
    kinds: &'static [u16],
    flag: fn(&mut Player) -> &mut bool,
}

fn flag_pygmy(p: &mut Player) -> &mut bool {
    &mut p.pygmy
}
fn flag_raven(p: &mut Player) -> &mut bool {
    &mut p.raven
}
fn flag_slime(p: &mut Player) -> &mut bool {
    &mut p.slime
}
fn flag_hornet_minion(p: &mut Player) -> &mut bool {
    &mut p.hornet_minion
}
fn flag_imp_minion(p: &mut Player) -> &mut bool {
    &mut p.imp_minion
}
fn flag_spider_minion(p: &mut Player) -> &mut bool {
    &mut p.spider_minion
}
fn flag_twins_minion(p: &mut Player) -> &mut bool {
    &mut p.twins_minion
}
fn flag_pirate_minion(p: &mut Player) -> &mut bool {
    &mut p.pirate_minion
}
fn flag_vampire_frog(p: &mut Player) -> &mut bool {
    &mut p.vampire_frog
}
fn flag_sharknado_minion(p: &mut Player) -> &mut bool {
    &mut p.sharknado_minion
}
fn flag_ufo_minion(p: &mut Player) -> &mut bool {
    &mut p.ufo_minion
}
fn flag_stardust_minion(p: &mut Player) -> &mut bool {
    &mut p.stardust_minion
}
fn flag_bats_of_light(p: &mut Player) -> &mut bool {
    &mut p.bats_of_light
}
fn flag_baby_bird(p: &mut Player) -> &mut bool {
    &mut p.baby_bird
}
fn flag_dead_cells_mushroom_boi_minion(p: &mut Player) -> &mut bool {
    &mut p.dead_cells_mushroom_boi_minion
}
fn flag_palworld_cattiva_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_cattiva_minion
}
fn flag_palworld_foxsparks_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_foxsparks_minion
}
fn flag_palworld_trusty_cattiva_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_trusty_cattiva_minion
}
fn flag_palworld_trusty_foxsparks_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_trusty_foxsparks_minion
}
fn flag_clay_pot_minion(p: &mut Player) -> &mut bool {
    &mut p.clay_pot_minion
}
fn flag_forbidden_minion(p: &mut Player) -> &mut bool {
    &mut p.forbidden_minion
}
fn flag_flinx_minion(p: &mut Player) -> &mut bool {
    &mut p.flinx_minion
}
fn flag_abigail_minion(p: &mut Player) -> &mut bool {
    &mut p.abigail_minion
}
fn flag_storm_tiger(p: &mut Player) -> &mut bool {
    &mut p.storm_tiger
}
fn flag_smolstar(p: &mut Player) -> &mut bool {
    &mut p.smolstar
}
fn flag_empress_blade(p: &mut Player) -> &mut bool {
    &mut p.empress_blade
}
fn flag_stardust_guardian(p: &mut Player) -> &mut bool {
    &mut p.stardust_guardian
}
fn flag_stardust_dragon(p: &mut Player) -> &mut bool {
    &mut p.stardust_dragon
}
fn flag_deadly_sphere_minion(p: &mut Player) -> &mut bool {
    &mut p.deadly_sphere_minion
}

/// The summon keep-alive arms in the C# chain's verbatim order
/// (`Player.cs:10483-10972`). Buff 216 carries its own respawn arm and buffs
/// 335/263 their status updaters — the driver special-cases those ids.
const SUMMON_BUFF_ARMS: [SummonBuffArm; 29] = [
    SummonBuffArm {
        buff: 49,
        kinds: &[191, 192, 193, 194],
        flag: flag_pygmy,
    }, // :10483
    SummonBuffArm {
        buff: 83,
        kinds: &[317],
        flag: flag_raven,
    }, // :10505
    SummonBuffArm {
        buff: 64,
        kinds: &[266],
        flag: flag_slime,
    }, // :10521
    SummonBuffArm {
        buff: 125,
        kinds: &[373],
        flag: flag_hornet_minion,
    }, // :10537
    SummonBuffArm {
        buff: 126,
        kinds: &[375],
        flag: flag_imp_minion,
    }, // :10553
    SummonBuffArm {
        buff: 133,
        kinds: &[390, 391, 392],
        flag: flag_spider_minion,
    }, // :10569
    SummonBuffArm {
        buff: 134,
        kinds: &[387, 388],
        flag: flag_twins_minion,
    }, // :10585
    SummonBuffArm {
        buff: 135,
        kinds: &[393, 394, 395],
        flag: flag_pirate_minion,
    }, // :10601
    SummonBuffArm {
        buff: 214,
        kinds: &[758],
        flag: flag_vampire_frog,
    }, // :10617
    SummonBuffArm {
        buff: 139,
        kinds: &[407],
        flag: flag_sharknado_minion,
    }, // :10633
    SummonBuffArm {
        buff: 140,
        kinds: &[423],
        flag: flag_ufo_minion,
    }, // :10649
    SummonBuffArm {
        buff: 182,
        kinds: &[613],
        flag: flag_stardust_minion,
    }, // :10665
    SummonBuffArm {
        buff: 213,
        kinds: &[755],
        flag: flag_bats_of_light,
    }, // :10681
    SummonBuffArm {
        buff: 216,
        kinds: &[759],
        flag: flag_baby_bird,
    }, // :10697 (+respawn)
    SummonBuffArm {
        buff: 355,
        kinds: &[1022],
        flag: flag_dead_cells_mushroom_boi_minion,
    }, // :10729
    SummonBuffArm {
        buff: 385,
        kinds: &[1093],
        flag: flag_palworld_cattiva_minion,
    }, // :10745
    SummonBuffArm {
        buff: 393,
        kinds: &[1118],
        flag: flag_clay_pot_minion,
    }, // :10761
    SummonBuffArm {
        buff: 394,
        kinds: &[1119],
        flag: flag_forbidden_minion,
    }, // :10777
    SummonBuffArm {
        buff: 386,
        kinds: &[1094],
        flag: flag_palworld_foxsparks_minion,
    }, // :10793
    SummonBuffArm {
        buff: 389,
        kinds: &[1112],
        flag: flag_palworld_trusty_cattiva_minion,
    }, // :10809
    SummonBuffArm {
        buff: 390,
        kinds: &[1113],
        flag: flag_palworld_trusty_foxsparks_minion,
    }, // :10825
    SummonBuffArm {
        buff: 325,
        kinds: &[951],
        flag: flag_flinx_minion,
    }, // :10841
    SummonBuffArm {
        buff: 335,
        kinds: &[970],
        flag: flag_abigail_minion,
    }, // :10857 (+status)
    SummonBuffArm {
        buff: 263,
        kinds: &[831],
        flag: flag_storm_tiger,
    }, // :10877 (+status)
    SummonBuffArm {
        buff: 271,
        kinds: &[864],
        flag: flag_smolstar,
    }, // :10897
    SummonBuffArm {
        buff: 322,
        kinds: &[946],
        flag: flag_empress_blade,
    }, // :10912
    SummonBuffArm {
        buff: 187,
        kinds: &[623],
        flag: flag_stardust_guardian,
    }, // :10928
    SummonBuffArm {
        buff: 188,
        kinds: &[625],
        flag: flag_stardust_dragon,
    }, // :10944
    SummonBuffArm {
        buff: 161,
        kinds: &[533],
        flag: flag_deadly_sphere_minion,
    }, // :10957
];

impl Player {
    /// `Player.EverySomeYoyoHits` (`Player.cs:4308-4316`): the counter's
    /// modulo against `amount` (C#'s truncated remainder, so the -1 start
    /// skips the very first hit) plus the optional bump. The one caller
    /// pair is `Projectile`'s on-hit arms: the damage doublings peek
    /// (`countUp = false`) before the strike, the spawns increment after
    /// it — both on the same hit, in that order.
    pub fn every_some_yoyo_hits(&mut self, amount: i32, count_up: bool) -> bool {
        let hit = self.yoyo_hit_counter % amount == 0;
        if count_up {
            self.yoyo_hit_counter += 1;
        }
        hit
    }

    pub fn spawn(world: &World) -> Self {
        // `FindSpawn` (`Player.cs:54592-54609`): the bed spawn — the port's
        // `World::bed_spawn` — overrides the world spawn when it validates
        // through the same `CheckSpawn` the revive path runs
        // (`Player.cs:37571-37575`). The load-time fallback is silent (the
        // failure chat needs the monitor, which the join path has not built
        // yet); the revive prints like the C#.
        let (sx, sy) = match world.bed_spawn {
            Some((bx, by)) if crate::world_sim::editing::bed_spawn_valid(world, bx, by).is_ok() => {
                // `Spawn_SetPosition` (`Player.cs:37987-37991`): the feet at
                // the spawn cell's top.
                (
                    bx as f32 * TILE_PX + 8.0 - WIDTH * 0.5,
                    by as f32 * TILE_PX - HEIGHT,
                )
            }
            _ => (
                world.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5,
                world.spawn_y as f32 * TILE_PX,
            ),
        };
        Self {
            dead: false,
            // The S1-D-3p stock values (`Player.cs:1446`/`:2994`/`:870`'s
            // field defaults): no stealth, no per-type calm, no tank pet.
            aggro: 0,
            npc_type_no_aggro: vec![false; crate::npc::NpcKind::CS_ID_COUNT],
            tank_pet: -1,
            tank_pet_reset: false,
            composite_front: crate::player::CompositeArmState::DISABLED,
            composite_back: crate::player::CompositeArmState::DISABLED,
            deaths_pve: 0,
            angler_quests_finished: 0,
            golfer_score: 0,
            deaths_pvp: 0,
            hostile: false,
            team: 0,
            hide_visible_accessory: [false; 10],
            dead_time: 0,
            respawn_timer: 0,
            immune: false,
            immune_time: 0,
            immune_no_blink: false,
            minion_attack_target_npc: -1,
            minion_rest_target_point: Vec2::ZERO,
            set_forbidden: false,
            set_stardust: false,
            tag_effects: crate::tag_effects::TagEffectStack::default(),
            whip_range_multiplier: 1.0,
            whip_use_time_multiplier: 1.0,
            // The RAW summoner-speed bonus starts 0 (`Player.cs:2460`,
            // `ResetEffects` re-zeroes `:18341`) — the i12 read-only
            // `summoner_weapon_speed_multiplier` folds `melee + bonus`
            // on demand, so the gate's old post-first-fold 1.0 spawn
            // stand-in is gone (the resync keep-one).
            summoner_weapon_speed_bonus: 0.0,
            cool_whip_buff: false,
            cob_whip_buff: false,
            acc_snake_band: false,
            acc_wicked_armlet: false,
            acc_mobius_strip: false,
            acc_silver_bracer: false,
            repeat_whip_swings: -1,
            repeat_whips_reset_cooldown: 0,
            prev_jump_input: false,
            auras: tr_core::scene::Auras::default(),
            mana_regen_bonus: 0,
            stat_defense: 0,
            well_fed: false,
            pick_speed_bonus: 0.0,
            melee_speed_bonus: 0.0,
            tile_speed_bonus: 0.0,
            wall_speed_bonus: 0.0,
            auto_reuse_glove: false,
            stress_ball: false,
            held_item_kind: 0,
            slow: false,
            chilled: false,
            frozen: false,
            webbed: false,
            blind: false,
            weak_stacks: 0,
            melee_crit: 4,
            ranged_crit: 4,
            magic_crit: 4,
            melee_damage: 1.0,
            ranged_damage: 1.0,
            magic_damage: 1.0,
            minion_damage: 1.0,
            ranged_mult_damage: 1.0,
            arrow_damage: 1.0,
            arrow_damage_additive_stack: 0.0,
            bullet_damage: 1.0,
            rocket_damage: 1.0,
            max_minions: 1,
            num_minions: 0,
            slots_minions: 0.0,
            max_turrets: 1,
            max_turrets_old: 1,
            owned_projectile_counts: [0; PROJ_COUNT],
            highest_storm_tiger_gem_original_damage: 0,
            highest_abigail_counter_original_damage: 0,
            pygmy: false,
            raven: false,
            slime: false,
            hornet_minion: false,
            imp_minion: false,
            spider_minion: false,
            next_cycled_spider_minion_type: 0,
            twins_minion: false,
            pirate_minion: false,
            vampire_frog: false,
            sharknado_minion: false,
            ufo_minion: false,
            stardust_minion: false,
            bats_of_light: false,
            baby_bird: false,
            // AI_026 slice A's pet keep-alive flags (see the field block).
            eater: false,
            skeletron: false,
            hornet: false,
            tiki: false,
            wisp: false,
            zephyrfish: false,
            pet_flag_baby_shark: false,
            pet_flag_lil_harpy: false,
            pet_flag_glittery_butterfly: false,
            pet_flag_queen_bee_pet: false,
            pet_flag_duke_fishron_pet: false,
            pet_flag_moon_lord_pet: false,
            pet_flag_dd2_betsy_pet: false,
            pet_flag_pufferfish: false,
            dead_cells_mushroom_boi_minion: false,
            palworld_cattiva_minion: false,
            palworld_foxsparks_minion: false,
            palworld_trusty_cattiva_minion: false,
            palworld_trusty_foxsparks_minion: false,
            clay_pot_minion: false,
            forbidden_minion: false,
            flinx_minion: false,
            abigail_minion: false,
            storm_tiger: false,
            smolstar: false,
            empress_blade: false,
            stardust_guardian: false,
            stardust_dragon: false,
            deadly_sphere_minion: false,
            x: sx,
            y: sy,
            prev_x: sx,
            prev_y: sy,
            old_x: sx,
            old_y: sy,
            actuation_rod_lock: false,
            actuation_rod_lock_setting: false,
            info_acc_mech_show_wires: false,
            auto_actuator: false,
            // `Player.cs:814`'s all-zero field init + the reset's `[0] = 1`
            // (`:56092`): the ruler-line toggle starts off, the rest on.
            builder_acc_status: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            leg_frame: 0,
            body_frame: 0,
            leg_frame_counter: 0.0,
            body_frame_counter: 0.0,
            swim_time: 0,
            misc_counter: 0,
            pulley: false,
            pulley_dir: 0,
            shield_raised: false,
            sliding: false,
            sand_storm: false,
            carpet_frame: -1,
            legs_armor: 0,
            frame_inputs: FrameInputs::default(),
            melee_npc_hit_cooldown: std::collections::HashMap::new(),
            wet: false,
            lava_wet: false,
            honey_wet: false,
            shimmer_wet: false,
            liquid_step: 1.0,
            in_water: false,
            sitting: false,
            sleeping: false,
            sleep_time: 0,
            sleeping_style: 0,
            offset_for_seat: (0.0, 0.0),
            seat_pos_offset: (0.0, 0.0),
            seat_y_offset: 0,
            no_knockback: false,
            fire_walk: false,
            has_paladin_shield: false,
            // M5-SEED-2: `Main.vampireSeed` (`WorldGen.cs:555-558`) — the
            // OnFire regen row's source (`Player.cs:18983-18992`).
            vampirism: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
            accessory_life_regen: 0,
            swing_aim: 0.0,
            equipped: [0; 20],
            dye_slots: [0; 10],
            misc_dye_slots: [0; 5],
            hide_misc: [false; 2],
            c_head: 0,
            c_body: 0,
            c_legs: 0,
            c_pet: 0,
            c_light: 0,
            c_minecart: 0,
            c_mount: 0,
            c_grapple: 0,
            release_hook: true,
            release_throw: true,
            just_dropped_an_item: false,
            equipped_prefix: [0; 20],
            banks: std::array::from_fn(|_| std::array::from_fn(|_| crate::item::EMPTY_SLOT)),
            chest: crate::bank::CHEST_CLOSED,
            chest_x: 0,
            chest_y: 0,
            edited_chest_name: false,
            prev_synced_chest: crate::bank::CHEST_CLOSED,
            void_vault_enabled: false,
            ate_artisan_bread: false,
            unlocked_biome_torches: false,
            pinned_jumps: [false; 9],
            pinned_balloon: false,
            pinned_rocket_boots: 0,
            pinned_dash: 0,
            extra_accessory: false,
            pinned_mount: None,
            control_left: false,
            control_right: false,
            control_jump: false,
            control_down: false,
            control_up: false,
            jump: 0,
            jump_held: false,
            spawn_x: sx,
            spawn_y: sy,
            potion_of_return_original_use_position: None,
            potion_of_return_home_position: None,
            hp: 100,
            max_hp: 100,
            armor_penetration: 0,
            melee_armor_penetration: 0,
            lava_time: 0,
            lava_max: 0,
            thorns: 0.0,
            cactus_thorns: false,
            turtle_thorns: false,
            sky_stone_effects: false,
            endurance: 0.0,
            mana: 20,
            mana_cost: 1.0,
            mana_max: 20,
            potion_delay: 0,
            mana_potion_delay: 0,
            mana_sick: false,
            mana_sick_reduction: 0.0,
            mana_heat: 1.0,
            p_stone: false,
            p_stone_prev: false,
            mana_max_base: 20,
            mana_flower: false,
            slow_magic_use: false,
            mana_regen_item_in_use: false,
            buffs: BuffSlots::default(),
            hurt_cd: 0.0,
            black_belt: false,
            mystic_sash_dodge: false,
            brain_of_confusion: false,
            shadow_dodge: false,
            long_invince: false,
            magic_cuffs: false,
            gold_ring: false,
            has_lucky_coin: false,
            mana_magnet: false,
            life_magnet: false,
            latest_mana_potion_damage_bonus: 0.0,
            latest_mana_potion_duration: 0.0,
            star_cloak: false,
            star_veil_override: false,
            bee_cloak_override: false,
            mana_cloak_override: false,
            snake_cloak_override: false,
            honey_comb: false,
            strong_bees: false,
            shadow_dodge_timer: 0,
            brain_of_confusion_dodge_animation_counter: 0,
            breath: BREATH_MAX,
            breath_cd: 0,
            creative_god_mode: false,
            debug_god_mode: false,
            far_placement_range: false,
            regen: RegenState::default(),
            dot_death: None,
            luck_state: LuckState::default(),
            zones: tr_core::scene::Zones::default(),
            held_torch_style: None,
            fall_start: (sy / TILE_PX) as i32,
            extra_fall: 0,
            jump_speed_boost: 0.0,
            no_fall_dmg: false,
            has_wings: false,
            grappling_active: false,
            going_down_with_grapple: false,
            mount_fishron_special_counter: 0.0,
            wing_time: 0.0,
            wing_time_max: 0,
            wings_logic: 0,
            rocket_time: 0,
            rocket_time_max: 7,
            rocket_delay: 0,
            rocket_delay2: 0,
            rocket_sound_delay: 0,
            rocket_boots: 0,
            vanity_rocket_boots: 0,
            can_rocket: false,
            rocket_release: false,
            merman: false,
            acc_merman: false,
            time_since_last_dash_started: 0,
            cart_mount: 13,
            release_grappling_hooks: false,
            release_jump: false,
            try_keeping_hovering_down: false,
            try_keeping_hovering_up: false,
            down_prev: false,
            double_tap_down_timer: 0,
            hold_down_timer: 0,
            release_down: true,
            has_jump_options: [false; 9],
            can_jump_again: [false; 9],
            performing_jump: None,
            jump_boost: false,
            jump_height: 15,
            jump_speed: 5.01,
            mount: crate::mount::MountState::default(),
            auto_jump: false,
            just_jumped: false,
            height: HEIGHT,
            width: WIDTH,
            last_stoned: false,
            dash_type: 0,
            dash: 0,
            dash_time: 0,
            dash_delay: 0,
            eoc_dash: 0,
            eoc_hit: -1,
            right_prev: false,
            left_prev: false,
            max_run_speed: 3.0,
            run_acceleration: 0.08,
            run_slowdown: 0.2,
            acc_run_speed: 3.0,
            move_speed: 1.0,
            strongest_move_speed_debuff: 1.0,
            ice_skate: false,
            desert_boots: false,
            sticky: false,
            slippy: false,
            slippy2: false,
            powerrun: false,
            running_on_sand: false,
            sticky_break: 0,
            suffocate_delay: 0,
            liquid_move_speed: 1.0,
            in_lava: false,
            gfx_off_y: 0.0,
            step_speed: 1.0,
            stair_fall: false,
            cart_mounted: false,
            cart_flip: false,
            unlocked_super_cart: false,
            enabled_super_cart: true,
            on_track: false,
            on_wrong_ground: false,
            track_boost: 0.0,
            last_boost: (0, 0),
            cart_ramp_time: 0,
            full_rotation: 0.0,
            full_rotation_origin: (0.0, 0.0),
            item_rotation: 0.0,
            fall_start2: (sy / TILE_PX) as i32,
            pending_switch_trip: None,
            pending_hurt_sound: None,
            pending_death_sound: None,
            run_sound_delay: 0,
            // `Player.cs:35947-35949` — the default until `PlayerFrame` runs.
            hermes_step_sound: (17, -1, 9),
            pending_step_sound: None,
            pending_double_jump_sound: None,
            pending_fart_sound: None,
            pending_splash_sound: None,
            wet_count: 0,
            pending_drown_sound: false,
            flap_sound: false,
            wing_frame: 0,
            wing_frame_counter: 0,
            pending_flap_sound: None,
            pending_mana_effect: None,
            pending_heal_effect: None,
            pending_retaliate: None,
            pending_dismount: false,
            pending_chillet_sound: None,
            pending_no_space_sound: None,
            pending_stoned_sound: None,
            pending_no_space_teleport: false,
            pending_force_clear: None,
            spark_rng: world.seed ^ 0x0CA4_75B1_A5E5_DD1D,
            pending_sparks: Vec::new(),
            pending_cart_sounds: Vec::new(),
            pending_sticky_dust: Vec::new(),
            pending_sticky_kill: None,
            // `Player.channel` is off until a `channel` item is held
            // (`Player.cs:52958-52968`).
            channel: false,
            channel_item: 0,
            channel_shoot: 0,
            channel_target: Vec2::ZERO,
            channel_proj_expected: 0,
            channel_index_expected: 0,
            // `Player.heldProj = -1` (`Player.cs:1516`'s stock).
            held_proj: -1,
            magic_quiver: false,
            minion_kb: 0.0,
            kb_glove: false,
            kb_buff: false,
            gross: false,
            tongued: false,
            sunflower: false,
            rabid: false,
            melee_enchant: 0,
            pet_flags: [false; PROJ_COUNT],
            shroomite_stealth: false,
            stealth: 0.0,
            space_gun: false,
            shadow_armor: false,
            crimson_regen: false,
            ash_wood_bonus: false,
            frost_burn: false,
            magma_stone: false,
            catalyst_band: false,
            vampire_burning_in_sunlight: false,
            has_molten_quiver: false,
            acc_sharp_barb: false,
            // S1-P-i35 — the fishing family's spawn defaults.
            fishing_skill: 0,
            crate_potion: false,
            sonar_potion: false,
            acc_tackle_box: false,
            acc_fishing_bobber: false,
            acc_fishing_line: false,
            acc_lava_fishing: false,
            override_fishing_bobber: -1,
            used_gummy_worm: false,
            acc_sentry_backpack: false,
            acc_sentry_backpack_grab_cooldown: 0,
            ballista_panic: false,
            set_squire_t2: false,
            set_squire_t3: false,
            set_huntress_t2: false,
            set_huntress_t3: false,
            set_monk_t2: false,
            set_monk_t3: false,
            set_apprentice_t2: false,
            set_apprentice_t3: false,
            ammo_box: false,
            ammo_potion: false,
            huntress_ammo_cost90: false,
            chloro_ammo_cost80: false,
            ammo_cost80: false,
            ammo_cost75: false,
            archery: false,
            lava_immune: false,
            gills: false,
            mana_regen_buff: false,
            slow_fall: false,
            find_treasure: false,
            biome_sight: false,
            invis: false,
            night_vision: false,
            enemy_spawns: false,
            water_walk: false,
            detect_creature: false,
            grav_control: false,
            confused: false,
            silence: false,
            dazed: false,
            no_items: false,
            cursed: false,
            empress_brooch: false,
            ignore_water: false,
            acc_flipper: false,
            can_float_in_water: false,
            lava_vision: false,
            spiked_boots: 0,
            slide_dir: 0,
            failed_no_space_count: 0,
            failed_no_space_location: (0, 0),
            ammo_cycling: crate::ammo::AmmoCycling::default(),
            yoyo_hit_counter: -1,
            // `Player.revolverCritChanceBonus` (`Player.cs:2456`): the int field
            // defaults to 0 (no charge at spawn).
            revolver_crit_chance_bonus: 0,
        }
    }

    /// `Collision.SolidCollision` (`Collision.cs:2700-2734`): any active
    /// solid non-solidTop tile overlapping the box — the `-1/+2` sample
    /// expansion, the half-brick top cut and the C#'s `maxTilesY - 40`
    /// bottom clamp included.
    pub(crate) fn solid_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
        let value = (x / 16.0) as i32 - 1;
        let value2 = ((x + w) / 16.0) as i32 + 2;
        let value3 = (y / 16.0) as i32 - 1;
        let value4 = ((y + h) / 16.0) as i32 + 2;
        let num = value.clamp(0, world.width as i32 - 1);
        let value2 = value2.clamp(0, world.width as i32 - 1);
        let value3 = value3.clamp(0, world.height as i32 - 40);
        let value4 = value4.clamp(0, world.height as i32 - 40);
        for i in num..value2 {
            for j in value3..value4 {
                let Some(t) = world.tile(i, j) else {
                    continue;
                };
                if !t.active()
                    || t.inactive()
                    || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                    || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    continue;
                }
                let vx = i as f32 * 16.0;
                let mut vy = j as f32 * 16.0;
                let mut vh = 16.0f32;
                if t.half_brick() {
                    vy += 8.0;
                    vh -= 8.0;
                }
                if x + w > vx && x < vx + 16.0 && y + h > vy && y < vy + vh {
                    return true;
                }
            }
        }
        false
    }

    /// `Player.CanSnapToPosition` (`Player.cs:33401-33417`): the snap box
    /// (`position + offset`, the LIVE width/height) must be free of solid
    /// tiles. A mounted transformation row (PlayerSize < 20×42,
    /// `:33404-33414`) additionally probes the DEFAULT 20×42 box through
    /// `CanFitInSpaceWithSize`. (The `Collision.CanHit` reach half `:33420`
    /// and its fallbacks are the standing stand-in deviation.)
    pub fn can_snap_to(&self, world: &World, offset_x: f32, offset_y: f32) -> bool {
        if Player::solid_collision(
            world,
            self.x + offset_x,
            self.y + offset_y,
            self.width,
            self.height,
        ) {
            return false;
        }
        // `:33404-33414` — the shrunken transformation mounts re-probe the
        // default box (Rat 14×14, Bat 20×18, Pixie 8×14 all qualify).
        if self.mount.is_active()
            && self
                .mount
                .data()
                .and_then(|d| d.player_size_override)
                .is_some_and(|(w, h)| w < WIDTH || h < HEIGHT)
        {
            // `CanFitInSpaceWithSize(DefaultSize, offset)` (`:6108-6111`).
            let probe_x = self.x + self.width * 0.5 - WIDTH / 2.0 + self.vx + offset_x;
            let probe_y = self.y + self.height - HEIGHT + self.vy + offset_y;
            if !collision::is_clear_spot_test(world, [probe_x, probe_y], 16.0, WIDTH, HEIGHT, true, true)
            {
                return false;
            }
        }
        true
    }

    /// `PlayerSittingHelper.SitDown` (`PlayerSittingHelper.cs:104-142`): the
    /// snap box is validated (`CanSnapToPosition`, `:110`), a repeat click
    /// at the same seat stands up (`:119-123`), then the bottom snaps to
    /// the seat, the facing flips to the seat's, the velocity zeroes and
    /// the seat offset stores (`:130-137`). The grapple/mount teardown
    /// stands in for `RemoveAllGrapplingHooks`/`mount.TryDismount`.
    pub fn sit_down(
        &mut self,
        world: &World,
        seat_x: f32,
        seat_y: f32,
        direction: i32,
        seat_offset: (f32, f32),
    ) {
        let bottom_x = self.x + self.width * 0.5;
        let bottom_y = self.y + self.height;
        if !self.can_snap_to(world, seat_x - bottom_x, seat_y - bottom_y) {
            return;
        }
        if self.sitting && (bottom_x - seat_x).abs() < 0.01 && (bottom_y - seat_y).abs() < 0.01 {
            self.sit_up();
            return;
        }
        self.grappling_active = false;
        self.x = seat_x - self.width * 0.5;
        self.y = seat_y - self.height;
        self.facing = direction;
        self.sitting = true;
        self.offset_for_seat = seat_offset;
        // `GetSittingOffsetInfo` (`:22-25`) with `sittingIndex` 0 (SP):
        // `posOffset = offsetForSeat * (direction, gravDir)`,
        // `seatAdjustment = -4 + (int)offsetForSeat.Y`.
        self.seat_pos_offset = (seat_offset.0 * direction as f32, seat_offset.1);
        self.seat_y_offset = -4 + seat_offset.1 as i32;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// `PlayerSittingHelper.SitUp` (`:89-102`).
    pub fn sit_up(&mut self) {
        self.sitting = false;
        self.offset_for_seat = (0.0, 0.0);
        self.seat_pos_offset = (0.0, 0.0);
        self.seat_y_offset = 0;
    }

    /// `PlayerSleepingHelper.StartSleeping` (`PlayerSleepingHelper.cs:155-190`):
    /// the snap box is validated (`:158`), a repeat click at the same anchor
    /// wakes up (`:167-171`), the vanity/teardown strip runs (`:172-178`),
    /// the bottom snaps to the bed anchor, the facing flips to the bed's and
    /// the pose rotation arms (`:184` → `SetIsSleepingAndAdjustPlayerRotation`,
    /// `:66-83`: `fullRotation = PI/2 * (-direction)`, origin `Size / 2`).
    pub fn sleep_down(
        &mut self,
        world: &World,
        anchor_x: f32,
        anchor_y: f32,
        direction: i32,
        bed_style: u32,
    ) {
        let bottom_x = self.x + self.width * 0.5;
        let bottom_y = self.y + self.height;
        if !self.can_snap_to(world, anchor_x - bottom_x, anchor_y - bottom_y) {
            return;
        }
        if self.sleeping && (bottom_x - anchor_x).abs() < 0.01 && (bottom_y - anchor_y).abs() < 0.01
        {
            self.stop_sleeping();
            return;
        }
        // `player.StopVanityActions()` (`:172`): SitUp + StopSleeping (+ the
        // petting stop — not ported).
        self.sit_up();
        self.stop_sleeping();
        self.grappling_active = false;
        // `RemoveAllFishingBobbers` (`PlayerSleepingHelper.cs:174`) runs at
        // the interaction caller (`world_sim/editing/doors.rs`'s BedSleep arm,
        // S1-P-i35) — the pool is not reachable from this pure player method;
        // `mount.TryDismount` (`:175-178`).
        if self.cart_mounted || self.mount.is_active() {
            let _ = self.dismount(world, false);
        }
        self.x = anchor_x - self.width * 0.5;
        self.y = anchor_y - self.height;
        self.facing = direction;
        self.sleeping = true;
        self.sleeping_style = bed_style;
        self.vx = 0.0;
        self.vy = 0.0;
        self.full_rotation = std::f32::consts::FRAC_PI_2 * -(direction as f32);
        self.full_rotation_origin = (WIDTH / 2.0, HEIGHT / 2.0);
    }

    /// `PlayerSleepingHelper.StopSleeping` (`:140-153`) +
    /// `SetIsSleepingAndAdjustPlayerRotation(false)` (`:76-81`).
    pub fn stop_sleeping(&mut self) {
        if self.sleeping {
            self.sleeping = false;
            self.full_rotation = 0.0;
            self.full_rotation_origin = (0.0, 0.0);
            self.sleep_time = 0;
        }
    }

    /// `PlayerSleepingHelper.UpdateState` (`PlayerSleepingHelper.cs:85-138`).
    /// `item_in_use` is the caller's `player.itemAnimation > 0`; the
    /// `danger_near` gate rides the M2-DEATH-2 stand-in (false — the port
    /// has no `DangerThatPreventsOtherDangers` members), and the eclipse
    /// arm (`:55-58`) waits for the hardmode state.
    pub fn update_sleeping(&mut self, world: &World, item_in_use: bool, danger_near: bool) {
        if !self.sleeping {
            self.sleep_time = 0;
            return;
        }
        self.sleep_time += 1;
        // `DoesPlayerHaveReasonToActUpInBed` (`:45-64`): the danger/blood
        // moon/item checks only reset the fully-asleep counter.
        if danger_near || (world.blood_moon && !world.day_time) || item_in_use {
            self.sleep_time = 0;
        }
        let bx = self.x + self.width * 0.5;
        let by = self.y + self.height - 2.0;
        let tx = (bx / 16.0).floor() as i32;
        let ty = (by / 16.0).floor() as i32;
        // `:97-102`: the bed tile re-decodes; a gone bed stops the sleep.
        let Some((_, _, target_direction, bed_style)) =
            tr_core::edit::sleeping_target_info(world, tx, ty)
        else {
            self.stop_sleeping();
            return;
        };
        // `:103-106`: the same exit mask as sitting (`controlUp` rides the
        // port's merged jump binding — W/ArrowUp feed `control_jump`).
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.stop_sleeping();
            return;
        }
        // `:103-106`: the same exit mask as sitting (`controlUp` rides the
        // port's merged jump binding — W/ArrowUp feed `control_jump`).
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.stop_sleeping();
            return;
        }
        // `:107-128`: a damaging/fishing/`ForcesBreaksSleeping` held item
        // breaks the sleep — the item stats land with M3-ITEM; the swing
        // itself already reset the counter above.
        // `:129-132`: the 2-players-per-tile stack (`Main.sleepingManager`)
        // is the multiplayer registry — SP no-op. `:133-137`: the offset
        // refresh — the port stores the style; the renderer derives the
        // table value.
        self.sleeping_style = bed_style;
    }

    /// The body height the sim feeds `base.height` with — 52 while the cart is
    /// mounted (`SetMount`, `Mount.cs:6270`: `height = 42 + heightBoost`), the
    /// live hitbox height otherwise (the mount framework's equip mutates it for
    /// its own mounts; unmounted 42). The width stays 20 (`:6271`).
    pub fn body_height(&self) -> f32 {
        if self.cart_mounted {
            CART_MOUNT_HEIGHT
        } else {
            self.height
        }
    }

    /// `Mount.SetMount(MountID.Minecart)` (`Mount.cs:6196-6305`) — the mount
    /// entry this phase exposes to the `TR_CART` pin (the item/QuickMount/hook
    /// paths are M2-MOUNT-3). A same-type re-mount is a no-op (`:6198`); a
    /// mount change resets the ride state (`:6205-6211`, pitfall §10.13).
    pub fn mount_cart(&mut self) {
        if self.cart_mounted {
            return;
        }
        self.cart_flip = false;
        self.last_boost = (0, 0);
        self.full_rotation = 0.0;
        self.full_rotation_origin = (0.0, 0.0);
        self.cart_mounted = true;
        // M2-MOUNT-3: through the mount framework — `SetMount(6)` grants the
        // row's buff 118 for 3600 (`:6226`) and applies the heightBoost 10
        // bottom-center growth 42 -> 52 (`:6266-6272`). The same-id guard
        // (`:6198`) keeps a re-mount a no-op, so the ride state stays.
        self.set_mount(crate::mount::MOUNT_MINECART, self.wet);
    }

    /// `Mount.CanDismountWithResult` (`Mount.cs:6611-6624`): the cart is not in
    /// `DontDismountWhenCCed` (`MountID.cs:47` — CCed never blocks it), so only
    /// the unmount-size room check can refuse. For the base cart the check is
    /// the 52→42 bottom-anchored shrink, which probes no tiles and always fits
    /// (`Collision.cs:2505-2510`).
    fn can_dismount(&self, world: &World) -> bool {
        collision::try_changing_size_from_bottom_center(
            world,
            [self.x, self.y],
            [self.width, self.body_height()],
            [WIDTH, HEIGHT],
        )
        .is_some()
    }

    /// `Mount.Dismount` (`Mount.cs:6142-6194`) for the cart ride: clears the
    /// ride state, the mount buff, and resizes 20×52 → 20×42 bottom-center.
    /// Both resize branches (`:6182-6193`) anchor the bottom for a shrink, so
    /// the port lifts the top by the freed heightBoost. The dismount dust is
    /// Phase 3. (Named apart from the mount-framework `dismount` — the cart
    /// ride sits outside `MountState` until M2-MOUNT-3 bridges them.)
    fn cart_dismount(&mut self) {
        if !self.cart_mounted {
            return; // `:6146-6149` — the `!_active` guard
        }
        self.cart_mounted = false;
        // M2-MOUNT-3: the framework's `Dismount` clears the mount buff
        // (`:6151`); the caller has already passed `CanDismountWithResult`,
        // so the space probe inside would also succeed.
        if let Some(buff) = self.mount.dismount() {
            self.clear_buff_type(buff);
        }
        self.cart_flip = false; // `:6157-6160`
        self.last_boost = (0, 0);
        self.full_rotation = 0.0; // `:6161-6162`
        self.full_rotation_origin = (0.0, 0.0);
        self.on_track = false;
        // The box restore: `mount_cart` now grows through the framework's
        // `grow_hitbox` (`self.height = 52`), so the drop returns it to 42
        // bottom-anchored (`:6182-6185`).
        self.y += CART_MOUNT_HEIGHT - HEIGHT;
        self.height = HEIGHT;
    }

    /// `Mount.TryDismount` (`Mount.cs:6137-6140` → `TryDismountWithResult`
    /// `:6127-6135`) for the cart ride: dismount only when the space check
    /// succeeds. (Named apart from the mount-framework `try_dismount`.)
    fn try_cart_dismount(&mut self, world: &World) {
        if self.can_dismount(world) {
            self.cart_dismount();
        }
    }

    /// Takes the pending pressure-track trip (the `Player.cs:28237-28241`
    /// `HitTrackSwitch` handoff — the caller owns `&mut World`).
    pub fn take_switch_trip(&mut self) -> Option<(i32, i32)> {
        self.pending_switch_trip.take()
    }

/// Takes the queued hurt cue (`Player.PlayHurtSound`, `Player.cs:38525`)
    /// — the caller owns the audio device.
    pub(crate) fn take_hurt_sound(&mut self) -> Option<&'static str> {
        self.pending_hurt_sound.take()
    }

    /// Takes the queued death cue (`Player.PlayDeathSound`,
    /// `Player.cs:38868`).
    pub(crate) fn take_death_sound(&mut self) -> Option<&'static str> {
        self.pending_death_sound.take()
    }

    /// Takes the queued footstep play (`Player.SpawnFastRunParticles`,
    /// `Player.cs:20214`) — `(SoundType, SoundStyle, position.X, position.Y)`.
    pub(crate) fn take_step_sound(&mut self) -> Option<(u16, i16, f32, f32)> {
        self.pending_step_sound.take()
    }

    /// S1-E-21f — takes the extra-jump `PlaySound(16)` play (the
    /// `Player.cs:20698-21022` chain's eight sites) — `(position.X,
    /// position.Y)`, the top-left.
    pub(crate) fn take_double_jump_sound(&mut self) -> Option<(f32, f32)> {
        self.pending_double_jump_sound.take()
    }

    /// S1-E-21f — takes the Fart's `SoundID.Item16` play (`Player.cs:20846`)
    /// — `(style_index, position.X, position.Y)` for the wave-A façade.
    pub(crate) fn take_fart_sound(&mut self) -> Option<(u16, f32, f32)> {
        self.pending_fart_sound.take()
    }

    /// S1-E-21f — takes the liquid entry/exit splash play
    /// (`Player.cs:27774-27944`) — `(Style, position.X, position.Y)`.
    pub(crate) fn take_splash_sound(&mut self) -> Option<(i32, f32, f32)> {
        self.pending_splash_sound.take()
    }

    /// S1-E-21f — takes the drowning gasp flag (`Player.cs:23257`,
    /// `PlaySound(23)` — unpositioned, no draws).
    pub(crate) fn take_drown_sound(&mut self) -> bool {
        std::mem::take(&mut self.pending_drown_sound)
    }

    /// S1-E-21f — takes the wing-flap `SoundID.Item32` play
    /// (`Player.cs:26850`) — `(style_index, position.X, position.Y)`.
    pub(crate) fn take_flap_sound(&mut self) -> Option<(u16, f32, f32)> {
        self.pending_flap_sound.take()
    }

    /// S1-P-i41 — takes the Chillet mount's `SoundID.PalChillet` play
    /// (`Mount.cs:6285`) — `(position.X, position.Y)` = `mountedPlayer.Center`.
    pub(crate) fn take_chillet_sound(&mut self) -> Option<(f32, f32)> {
        self.pending_chillet_sound.take()
    }

    /// S1-P-i41 - takes the QuickMount no-space tink (`Player.cs:6057`) -
    /// `(x, y)`, the truncated position; the case-0 draws ride the drain.
    pub(crate) fn take_no_space_sound(&mut self) -> Option<(f32, f32)> {
        self.pending_no_space_sound.take()
    }

    /// W4-c14 - takes the petrify-transition Dig-0 (`Player.cs:25225`) -
    /// `(x, y)`, the truncated position; the case-0 draws ride the drain.
    pub(crate) fn take_stoned_sound(&mut self) -> Option<(f32, f32)> {
        self.pending_stoned_sound.take()
    }

    /// S1-P-i41 — takes the third-fail no-space teleport request
    /// (`Player.cs:6061-6066`); the caller owns `&World` for the spot scan.
    pub(crate) fn take_no_space_teleport(&mut self) -> bool {
        std::mem::take(&mut self.pending_no_space_teleport)
    }

    /// S1-P-i41 — takes the `Shellphone_Spawn` force-clear request
    /// (`Player.cs:56171`) — `(floor_x, floor_y)` of the invalid spawn area.
    pub(crate) fn take_force_clear(&mut self) -> Option<(i32, i32)> {
        self.pending_force_clear.take()
    }

    /// Takes the queued mana-restore text amount (`Player.ManaEffect`,
    /// `Player.cs:4531-4538`) — the caller owns `Main.combatText`.
    pub(crate) fn take_mana_effect(&mut self) -> Option<i32> {
        self.pending_mana_effect.take()
    }

    /// `Player.ManaEffect` (`Player.cs:4531-4538`): queue the HealMana
    /// combat-text amount. The cuffs' `Hurt` arm writes the field directly
    /// (`:38299`); the potion drink (`ApplyLifeAndOrMana` `:51950`) and the
    /// star pickups (`:35072`) call the method — this is its port.
    pub(crate) fn queue_mana_effect(&mut self, amount: i32) {
        self.pending_mana_effect = Some(amount);
    }

    /// Takes the queued heal text amount (`Player.HealEffect`,
    /// `Player.cs:4522-4529`) — the caller owns `Main.combatText`
    /// (the `take_mana_effect` convention).
    pub(crate) fn take_heal_effect(&mut self) -> Option<i32> {
        self.pending_heal_effect.take()
    }

    /// `Player.Heal(int)` (`Player.cs:35112-35123`): `statLife += amount`,
    /// the local-player `HealEffect(amount)` text, THEN the `statLifeMax2`
    /// cap — the text carries the RAW amount even when the cap eats part of
    /// the heal (the heart at 95/100 heals 5 but prints 20). The NetMessage
    /// 35 broadcast (`:4525-4528`) is MP-only.
    pub(crate) fn heal(&mut self, amount: i32) {
        self.hp += amount;
        self.pending_heal_effect = Some(amount);
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
    }

    /// `Player.TryApplyManaHeat(float power, float time)`
    /// (`Player.cs:52006-52014`): the latest-bonus pair is written first,
    /// then `AddBuff(396, (int)duration)` when both are non-zero. The
    /// buff-396 arm's `manaHeat` decay (`:10328-10341`) is the deferred
    /// UpdateBuffs census seam.
    pub(crate) fn try_apply_mana_heat(&mut self, power: f32, time: f32) {
        self.latest_mana_potion_damage_bonus = power;
        self.latest_mana_potion_duration = time * 60.0;
        if time != 0.0 && power != 0.0 {
            self.buffs.add_buff(396, self.latest_mana_potion_duration as i32);
        }
    }


    /// S1-C-7 wave B: takes the accepted-hurt retaliation request
    /// (`Player.cs:38401-38517`) — the hit-moment snapshot (final post-floor
    /// `num2` + pre-knockback position/center/velocity). The caller owns the
    /// NPC list + projectile pool + the hit lane's RNG
    /// (`hurt_modifiers::drain_retaliate`).
    pub(crate) fn take_retaliate(&mut self) -> Option<crate::hurt_modifiers::RetaliateRequest> {
        self.pending_retaliate.take()
    }

    /// S1-C-7 wave B: takes the dismount-on-hit request (`Player.cs:38396-
    /// 38400`) — the caller owns `&World` (`try_dismount`).
    pub(crate) fn take_dismount_request(&mut self) -> bool {
        std::mem::take(&mut self.pending_dismount)
    }

    /// Takes the tick's queued spark spawn requests (the `Main.dust`
    /// handoff — the caller owns the pool).
    pub(crate) fn take_cart_sparks(&mut self) -> Vec<crate::minecart::SparkSpawn> {
        std::mem::take(&mut self.pending_sparks)
    }

    /// Takes the tick's queued minecart cues (the `SoundEngine.PlaySound`
    /// handoff — the caller owns the audio device).
    pub(crate) fn take_cart_sounds(&mut self) -> Vec<crate::minecart::CartCue> {
        std::mem::take(&mut self.pending_cart_sounds)
    }

    /// Takes the cobweb tile `StickyMovement` broke (`Player.cs:22949`'s
    /// `WorldGen.KillTile`) — the caller owns `&mut World`.
    pub(crate) fn take_sticky_kill(&mut self) -> Option<(i32, i32)> {
        self.pending_sticky_kill.take()
    }

    /// Takes the tick's queued web puffs (`Player.cs:23036-23080`) — the caller
    /// owns `Main.dust`.
    pub(crate) fn take_sticky_dust(&mut self) -> Vec<StickyDust> {
        std::mem::take(&mut self.pending_sticky_dust)
    }

    /// `PlayerSittingHelper.UpdateSitting` (`PlayerSittingHelper.cs:50-87`):
    /// the seat tile re-decodes through `GetSittingTargetInfo` (`:56-61`),
    /// any control input / the mount / a facing mismatch stands up
    /// (`:62-65`), and the seat offset refreshes (`:84-85`). The 2-per-tile
    /// seat stack (`Main.sittingManager`, `:66-69`) and the Skeletron-hat
    /// bench easter egg (`:74-83`) are the multiplayer/NPC sides — SP no-op /
    /// not ported. (`controlUp` rides the port's merged jump binding —
    /// W/ArrowUp feed `control_jump`.)
    pub fn update_sitting(&mut self, world: &World) {
        if !self.sitting {
            return;
        }
        // `TryGetSittingBlock` (`:33-48`): the tile at the bottom - 2 px
        // re-decodes as a sit-able seat.
        let bx = self.x + self.width * 0.5;
        let by = self.y + self.height - 2.0;
        let tx = (bx / 16.0).floor() as i32;
        let ty = (by / 16.0).floor() as i32;
        let Some((_, _, target_direction, seat_down_offset)) =
            tr_core::edit::sitting_target_info(world, tx, ty, self.facing)
        else {
            self.sit_up();
            return;
        };
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.sit_up();
            return;
        }
        self.offset_for_seat = seat_down_offset;
        self.seat_pos_offset = (seat_down_offset.0 * self.facing as f32, seat_down_offset.1);
        self.seat_y_offset = -4 + seat_down_offset.1 as i32;
    }

    /// The test/inspection reads for the private speed fields.
    pub(crate) fn acc_run_speed_value(&self) -> f32 {
        self.acc_run_speed
    }

    pub(crate) fn max_run_speed_value(&self) -> f32 {
        self.max_run_speed
    }

    /// S1-C-12 wave D — the run-acceleration/slowdown inspection reads (the
    /// Shadow Scale boost's witnesses, `Player.cs:26511-26517`).
    pub(crate) fn run_acceleration_value(&self) -> f32 {
        self.run_acceleration
    }

    pub(crate) fn run_slowdown_value(&self) -> f32 {
        self.run_slowdown
    }

    pub(crate) fn accessory_life_regen_value(&self) -> i32 {
        self.accessory_life_regen
    }

    /// Used by the `TR_XY` debug hook and future teleport effects.
    pub fn teleport(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.prev_x = x;
        self.prev_y = y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.fall_start = (y / TILE_PX) as i32;
        self.gfx_off_y = 0.0;
    }

    /// `Player.ResetFallDamage` (`Player.cs:37712-37716`): `fallStart =
    /// fallStart2 = (int)(position.Y / 16f)` — the spawn/recall reset the
    /// fall-damage row to the new position.
    pub fn reset_fall_damage(&mut self) {
        self.fall_start = (self.y / TILE_PX) as i32;
        self.fall_start2 = self.fall_start;
    }

    /// `Entity.wetCount = 0` on spawn (`Player.cs:37636`) — the splash
    /// cooldown the recall spawn clears with the wet flags.
    pub fn reset_wet_count(&mut self) {
        self.wet_count = 0;
    }

    /// `Player.Spawn`'s `jump = 0` (`Player.cs:37641`) — the recall/teleport
    /// spawn clears the held-jump boost (gate V3: the i38 module reaches the
    /// private counter through this seam, the `reset_fall_damage` pattern).
    pub fn reset_jump_boost(&mut self) {
        self.jump = 0;
    }

    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.jump_held = false;
        self.leg_frame = 0;
        self.body_frame = 0;
        self.leg_frame_counter = 0.0;
        self.body_frame_counter = 0.0;
        self.swim_time = 0;
        self.pulley = false;
        self.pulley_dir = 0;
        self.shield_raised = false;
        self.sliding = false;
        self.sand_storm = false;
        self.carpet_frame = -1;
        // `Player.Spawn`'s revive life (`Player.cs:37598-37608`): the pool
        // is `max(statLifeMax, statLifeMax2)` — the port's single `max_hp`
        // is the effective max — and the life is `max(pool / 2, 100)`. A
        // fresh 100-life character revives FULL; the `spawnMax` Journey arm
        // (`:37609-37611`) has no port counterpart.
        self.hp = (self.max_hp / 2).max(100).min(self.max_hp.max(100));
        // `:37658-37666`: the i-frames — `immune = true` (`:37594`) and the
        // `PlayerSpawnContext.ReviveFromDeath` arm lands 180 (the port's
        // respawn is always the death revive; the 60 arm rides the other
        // `PlayerSpawnContext`s).
        self.immune = true;
        self.immune_time = 180;
        self.immune_no_blink = true;
        self.hurt_cd = 0.0;
        // `UpdateDead`'s brain-counter zero (`Player.cs:16932`): the death tail
        // clears the animation window with the rest of the per-life state.
        self.brain_of_confusion_dodge_animation_counter = 0;
        self.breath = BREATH_MAX;
        self.breath_cd = 0;
        self.regen = RegenState::default();
        self.dot_death = None;
        self.fall_start = (self.y / TILE_PX) as i32;
        self.wet = false;
        self.lava_wet = false;
        self.honey_wet = false;
        self.shimmer_wet = false;
        // `Player.Spawn` (`Player.cs:37636`): `base.wetCount = 0`.
        self.wet_count = 0;
        self.in_lava = false;
        // `Player.UpdateDead` (`Player.cs:16929`): death zeroes the special.
        self.mount_fishron_special_counter = 0.0;
        self.gfx_off_y = 0.0;
        // `Player.KillMe` dismounts the mount (`Player.cs:38907`); the respawn
        // resets the ride state with it (the position is rebuilt from spawn
        // below, so no bottom-center resize is needed here).
        self.cart_mounted = false;
        self.cart_flip = false;
        self.on_track = false;
        self.on_wrong_ground = false;
        self.track_boost = 0.0;
        self.last_boost = (0, 0);
        self.cart_ramp_time = 0;
        self.full_rotation = 0.0;
        self.full_rotation_origin = (0.0, 0.0);
        self.fall_start2 = (self.y / TILE_PX) as i32;
        self.pending_switch_trip = None;
        self.pending_sparks.clear();
        self.pending_cart_sounds.clear();
        self.pending_sticky_dust.clear();
        self.pending_sticky_kill = None;
        self.pending_mana_effect = None;
        self.pending_heal_effect = None;
        self.pending_retaliate = None;
        self.pending_double_jump_sound = None;
        self.pending_fart_sound = None;
        self.pending_splash_sound = None;
        self.pending_drown_sound = false;
        self.pending_flap_sound = None;
        self.pending_stoned_sound = None;
        self.pending_dismount = false;
        // `Player.spawn`/`Spawn` never resets `stickyBreak` (`Player.cs:1966` —
        // the only writers are `StickyMovement`'s own four sites), so the port
        // keeps the counter across a revive too.
        self.buffs.clear_buff(CART_MOUNT_BUFF);
        // M2-MOUNT-3: the cart row lives in `MountState` now, so the revive
        // strip clears it like every other mount (`Player.cs:38908`) — the
        // framework's dismount returns the box with it, and the spawn
        // position is rebuilt below anyway.
        if let Some(buff) = self.mount.dismount() {
            self.clear_buff_type(buff);
        }
        self.height = HEIGHT;
        self.width = WIDTH;
        // `Player.Spawn`'s `StopVanityActions` (`Player.cs:37556`): the
        // revive clears the sit/sleep state.
        self.sit_up();
        self.stop_sleeping();
    }

    /// `Player.KillMe`'s respawn setup (`Player.cs:38908-38910`): the dead
    /// flag, `deadTime` reset and `respawnTimer = GetRespawnTime(pvp)`.
    pub fn start_death(&mut self, world: &World, boss_near: bool, expert: bool) {
        // `Player.KillMe`'s head (`Player.cs:38809-38812`): the godmode power
        // (and an already-dead player) refuses the death entirely.
        if self.creative_god_mode || self.dead {
            return;
        }
        // `Player.KillMe`'s counter arm (`Player.cs:38840-38844`): the PVP
        // flag splits the two counters; the port has no PVP, so every
        // death is PVE.
        self.deaths_pve = self.deaths_pve.saturating_add(1);
        self.dead = true;
        self.dead_time = 0;
        self.respawn_timer = Player::get_respawn_time(boss_near, expert);
        // `Player.KillMe` -> `mount.Dismount` (`Player.cs:38907`).
        let _ = self.dismount(world, true);
        // `Player.KillMe`'s tail (`Player.cs:38868`): the death cue — the
        // legacy "Player Killed" id 5 (`Player.cs:38737`, the default arm).
        self.pending_death_sound = Some("Player_Killed");
    }

    /// `Player.GetRespawnTime` (`Player.cs:39060-39082`): 600 base, +600 when
    /// a boss hinders the respawn, x1.5 in expert. The FTW-multiplier x2 and
    /// the master-mode path stay with M5-SEED-1 / the master difficulty.
    pub fn get_respawn_time(boss_near: bool, expert: bool) -> i32 {
        let mut time = 600;
        if boss_near {
            time += 600;
        }
        if expert {
            time = (time as f64 * 1.5) as i32;
        }
        time
    }

    /// `NPC.AnyBossHindersRespawnTime` shape (`NPC.cs`): a boss within the
    /// 4000 px Manhattan range of the death spot. The King Slime is the
    /// ported boss.
    pub fn boss_hinders(&self, npcs: &[crate::npc::Npc]) -> bool {
        let (cx, cy) = self.center();
        npcs.iter().any(|n| {
            n.kind == crate::npc::NpcKind::KingSlime && {
                let (nx, ny) = n.center();
                (cx - nx).abs() + (cy - ny).abs() < 4000.0
            }
        })
    }

    /// `Player.NearAnyNPCsThatBlockRespawn` (`Player.cs:38987-39018`): a live
    /// NPC inside the centred `Main.MaxWorldViewSize` (1920x1080) rect inflated
    /// by 320 px blocks the respawn skip — an invasion NPC while its invasion
    /// runs (`NPC.GetNPCInvasionGroup(type) == Main.invasionType`), or a
    /// `NPCID.Sets.DangerThatPreventsOtherDangers` member (the port has none
    /// yet, so only the invasion arm fires).
    pub fn near_any_npcs_that_block_respawn(
        &self,
        npcs: &[crate::npc::Npc],
        invasion_live: bool,
    ) -> bool {
        // `Utils.CenteredRectangle(Center, Main.MaxWorldViewSize)` then
        // `Inflate(320, 320)` (`:38992-38998`).
        const VIEW_W: f32 = 1920.0;
        const VIEW_H: f32 = 1080.0;
        const INFLATE: f32 = 320.0;
        let (cx, cy) = self.center();
        let half_w = VIEW_W * 0.5 + INFLATE;
        let half_h = VIEW_H * 0.5 + INFLATE;
        npcs.iter().any(|n| {
            let (nx, ny) = n.center();
            let (w, h) = n.size();
            let overlaps =
                (nx - cx).abs() <= half_w + w * 0.5 && (ny - cy).abs() <= half_h + h * 0.5;
            if !overlaps {
                return false;
            }
            // `NPC.GetNPCInvasionGroup(nPC.type) > 0 && == Main.invasionType`
            // (`:39005-39008`); the danger-set arm (`:39009-39012`) has no
            // ported members.
            invasion_live && n.kind.is_invasion_npc()
        })
    }

    /// `Player.AllowsRespawnTimerSkip` (`Player.cs:38962-38987`): after the
    /// 90-tick lockout, with no boss hindering, no danger NPCs nearby and
    /// no Frost/Pumpkin/OOA event running (`:38984-38986`). The
    /// `ghost`/`pvpDeath` branches stay unreachable in the port.
    pub fn allows_respawn_timer_skip(
        &self,
        boss_near: bool,
        danger_near: bool,
        moon_event_live: bool,
    ) -> bool {
        !(self.ghost_like() || self.dead_time < 90 || boss_near || danger_near || moon_event_live)
    }

    /// The port has no ghosts (hardcore) or PVP deaths; both gates are false.
    fn ghost_like(&self) -> bool {
        false
    }

    /// `Player.UpdateDead` (`Player.cs:16919-17070`): `deadTime++`, the
    /// respawn decrement clamped to `respawnTimerMax` (3600), and the
    /// jump-press skip while `AllowsRespawnTimerSkip`. Returns true when the
    /// player should respawn now.
    pub fn update_dead(
        &mut self,
        jump: bool,
        boss_near: bool,
        danger_near: bool,
        moon_event_live: bool,
    ) -> bool {
        self.dead_time += 1;
        // `Player.cs:17016`: the dead tick zeroes the life pool.
        self.hp = 0;
        // `Player.cs:17017`: the dead tick drops the channel.
        self.channel = false;
        self.channel_item = 0;
        self.channel_target = Vec2::ZERO;
        self.channel_proj_expected = 0;
        self.channel_index_expected = 0;
        // S1-P-i20 seam: the shoot-id field clears with the latch.
        self.channel_shoot = 0;
        // `Player.cs:17018-17019`: the dead tick zeroes both sickness clocks
        // (the buff-21 slot itself survives until expiry — only the counters
        // reset, exactly like the source).
        self.potion_delay = 0;
        self.mana_potion_delay = 0;
        self.respawn_timer = (self.respawn_timer - 1).clamp(0, 3600);
        // `Player.UpdateDead`'s counter reset (`Player.cs:17009-17010`): the
        // budget zeroes while dead — the pool stays until AI_062's dead-owner
        // flag read (M4-PROJ-8 phase 4) despawns the minions.
        self.num_minions = 0;
        self.slots_minions = 0.0;
        // `Player.cs:17020` — `this.chest = -1`: the dead tick force-closes
        // the open container SILENTLY (no sound — the range close's 11 never
        // fires; S1-P-i43). `chestX/Y` stay, exactly like the C#.
        self.chest = crate::bank::CHEST_CLOSED;
        let jump_edge = jump && !self.prev_jump_input;
        self.prev_jump_input = jump;
        let can_skip = self.allows_respawn_timer_skip(boss_near, danger_near, moon_event_live);
        if can_skip && jump_edge && self.respawn_timer > 0 {
            return true;
        }
        self.respawn_timer <= 0
    }

    /// `Player.Spawn(ReviveFromDeath)`: the dead flags clear, then the
    /// standard spawn-point respawn.
    pub fn revive(&mut self) {
        self.dead = false;
        self.dead_time = 0;
        self.respawn_timer = 0;
        self.respawn();
    }

/// `Main.TryRemovingBuff` (`Main.cs:43370-43389`): the buff bar's
    /// right-click removal — non-debuffs only, and not Petrified 60 /
    /// Webbed 151; the mount's own buff dismounts instead of removing the
    /// buff (`Mount.CheckBuff`, `Mount.cs:2933-2936` — `data.buff == b`).
    /// The dismount is `mount.TryDismount` (`:43377` → `Mount.cs:6137-6140`
    /// → `CanDismountWithResult` `:6611-6624`): a refused dismount (CCed on
    /// 55/56/61, or no room for the 20×42 shrink) keeps the buff — the C#
    /// sets `flag = true` regardless of the `TryDismount` result
    /// (`:43378`), so `DelBuff` never runs on the mount arm. The cart ride
    /// hosts its own state (`cart_mounted`), so the cart rows route through
    /// `try_cart_dismount` — the same `TryDismount` gates plus the ride
    /// flags. `misc_buffs` carries `miscEquips[0..2].buffType` (the App
    /// owns the arrays) for the `TryRemovingBuff_CheckBuffHideMisc` arms
    /// (`:43380-43381`). Returns whether the arm entered (the caller plays
    /// `SoundEngine.PlaySound(12)`'s MenuTick cue, `:43382`).
    pub fn try_removing_buff(
        &mut self,
        world: &World,
        i: usize,
        b: i32,
        misc_buffs: [i32; 2],
    ) -> bool {
        if !crate::buffs::is_debuff(b) && b != 60 && b != 151 {
            // `:43375-43379` — the mount arm; `flag` latches on the
            // CheckBuff match, not on the dismount's success.
            let mut mount_arm = false;
            if self.mount.is_active() && self.mount.data().map(|d| d.buff) == Some(b) {
                if self.cart_mounted {
                    self.try_cart_dismount(world);
                } else {
                    self.try_dismount(world);
                }
                mount_arm = true;
            }
            // `:43380-43381` — the pet / light-pet hide arms.
            self.check_buff_hide_misc(0, b, misc_buffs[0]);
            self.check_buff_hide_misc(1, b, misc_buffs[1]);
            if !mount_arm {
                self.buffs.del_buff(i); // `:43385`
            }
            return true;
        }
        false
    }

        /// `Main.TryRemovingBuff_CheckBuffHideMisc` (`Main.cs:43390-43404`):
    /// cancelling the equipped pet's / light pet's buff hides the misc
    /// slot's visual (`hideMisc[slot] = true`) — but only while it is not
    /// already hidden. The 101/102 light-pet buffs also match the Fairy
    /// Bell's legacy `buffType == 27` (`:43395-43398`; item 425,
    /// `Item.cs:6342`).
    fn check_buff_hide_misc(&mut self, slot: usize, buff_id: i32, misc_buff: i32) {
        if !self.hide_misc[slot] {
            let mut matches = misc_buff == buff_id;
            if !matches && (buff_id == 102 || buff_id == 101) {
                matches = misc_buff == 27;
            }
            if matches {
                self.hide_misc[slot] = true;
            }
        }
    }

    /// `GetNurseHealCost`'s slot count (`Main.cs:40058-40065`): a debuff
    /// slot counts when `Main.debuff[b] && buffTime > 60 &&
    /// !NurseCannotRemoveDebuff[b]`.
    pub fn remountable_debuff_count(&self) -> i32 {
        let mut count = 0;
        for i in 0..crate::buffs::MAX_BUFFS {
            let b = self.buffs.buff_type[i];
            if crate::buffs::is_debuff(b)
                && self.buffs.buff_time[i] > 60
                && !crate::buffs::nurse_cannot_remove(b)
            {
                count += 1;
            }
        }
        count
    }

    /// `NPCChatText_DoNurseHeal`'s removal pass (`Main.cs:40147`): the
    /// debuff slots with `buffTime > 0` and the Nurse's protected list
    /// clear.
    pub fn clear_nurse_removable_debuffs(&mut self) {
        for i in 0..crate::buffs::MAX_BUFFS {
            let b = self.buffs.buff_type[i];
            if crate::buffs::is_debuff(b)
                && self.buffs.buff_time[i] > 0
                && !crate::buffs::nurse_cannot_remove(b)
            {
                self.buffs.clear_buff(b);
            }
        }
    }

    /// `Player.ResetProjectileCaches` (`Player.cs:12217-12226`): the recount
    /// input zeroed — called from the `ResetEffects` seam (`:18810`) and the
    /// dead path (`:24752`).
    pub fn reset_projectile_caches(&mut self) {
        self.highest_storm_tiger_gem_original_damage = 0;
        self.highest_abigail_counter_original_damage = 0;
        self.owned_projectile_counts = [0; PROJ_COUNT];
    }

    /// `Player.UpdateProjectileCaches` (`Player.cs:12184-12215`): the per-tick
    /// recount of the live pool's owned projectiles into
    /// `owned_projectile_counts` plus the 831/970 `originalDamage` maxima.
    /// The C# walks `Main.projectile[1000]` for `owner == i`; the port's
    /// whoAmI stand-in is 0 (the single-player owner the spawner folds to).
    pub fn update_projectile_caches(&mut self, pool: &ProjectilePool) {
        for p in pool.slots() {
            if !p.active || p.owner != 0 {
                continue;
            }
            // A kind at or above `ProjectileID.Count` cannot exist in the C#
            // id space; the count skips it instead of indexing out of range.
            if let Some(count) = self.owned_projectile_counts.get_mut(usize::from(p.kind)) {
                *count += 1;
            }
            match p.kind {
                831 => {
                    if self.highest_storm_tiger_gem_original_damage < p.original_damage {
                        self.highest_storm_tiger_gem_original_damage = p.original_damage;
                    }
                }
                970 => {
                    if self.highest_abigail_counter_original_damage < p.original_damage {
                        self.highest_abigail_counter_original_damage = p.original_damage;
                    }
                }
                _ => {}
            }
        }
    }

    /// `Player.UpdateBuffs`' summon chain (`Player.cs:10483-10947`): the
    /// slot-ordered keep-alive arms (with the cap effects `:10175-10183` and
    /// the buff-216 respawn arm `:10697-10726`), driven by
    /// `owned_projectile_counts`. Runs inside the fixed step at the
    /// `UpdateBuffs` seam — the pool and the inventory ride in as parameters
    /// because `Player::update` cannot see the App's.
    ///
    /// The ported order, pinned (the plan constraint — verified against the
    /// decompile this phase):
    /// - `Main.Update` → `UpdatePlayers` (`Main.cs:17754`) → `Player.Update`:
    ///   `ResetEffects` (`:25491` — `maxMinions`/`maxTurrets` = 1 at
    ///   `:18448-18449`, the summon flags `:18620-18646`+`:18741`,
    ///   `ResetProjectileCaches` `:18810`) → the buffImmune reset
    ///   (`:25562-25564`) → `UpdateProjectileCaches(i)` (`:25565`) →
    ///   `UpdateBuffs(i)` (`:25566` — ONE `for j` loop: the timer head
    ///   `:9869-9876`, then the else-if chain: the cap effects
    ///   `:10175-10183`, the summon arms `:10483-10947`, `DelBuff(j); j--`
    ///   rescanning the slot the compaction filled) → `UpdateEquips`
    ///   (`:25633` — `grant_summon_caps`) → `numMinions = 0; slotsMinions =
    ///   0f` (`:28279-28280`, before ItemCheck).
    /// - `Main.Update` → `UpdateNPCs` (`:17767`) → `UpdateProjectiles`
    ///   (`:17775` — the pool pass accumulates `numMinions`/`slotsMinions`,
    ///   `Projectile.cs:16736-16737`, M4-PROJ-8 phase 3). So the value this
    ///   chain reads for `numMinions` is the last pool pass's accumulation —
    ///   the live pool's owned-minion count (the pool is unchanged between
    ///   the passes).
    /// The port realizes the same order across `update_with_grapple` (the
    /// ResetEffects seam + the buff timer/effects), this method, the
    /// `grant_*` seams and the pool's update — each counter's read order
    /// matches. The buff timers decrement and the expired slots purge in
    /// `update_with_grapple` before the arms run here; the C# arms run
    /// before the `:25608` purge, but a summon buff's time is pinned at
    /// 18000 by the refresh while its minion lives (and a dead-keep-alive
    /// buff is DelBuff'd by the arm either way), so the swap is
    /// unobservable.
    pub fn update_summon_buff_arms(
        &mut self,
        pool: &mut ProjectilePool,
        inventory: &[ItemSlot; INV],
    ) {
        // The slot-ordered chain with the C# `j--` rescan: after `DelBuff(j)`
        // the compaction shifts the later buffs left, so index `j` is
        // re-examined instead of advanced (the shifted-in buff's arm runs in
        // the same tick, `Player.cs:10483-10947`).
        let mut j = 0usize;
        while j < MAX_BUFFS {
            let buff = self.buffs.buff_type[j];
            // The C# guard (`:9871-9874`) — the decrement already ran in
            // `update_with_grapple`'s timer tick.
            if buff <= 0 || self.buffs.buff_time[j] <= 0 {
                j += 1;
                continue;
            }
            let survives = match buff {
                // `Player.cs:10175-10183` — the cap effects share the chain,
                // slot-ordered with the arms (the buff-216 arm's `maxMinions`
                // read at `:10706` sees the earlier slots' raises).
                110 | 150 => {
                    self.max_minions += 1;
                    true
                }
                348 => {
                    self.max_turrets += 1;
                    true
                }
                216 => self.run_baby_bird_arm(pool, inventory, j),
                _ => match SUMMON_BUFF_ARMS.iter().find(|a| a.buff == buff) {
                    Some(arm) => {
                        let kept = arm
                            .kinds
                            .iter()
                            .any(|&k| self.owned_projectile_counts[usize::from(k)] > 0);
                        if kept {
                            *(arm.flag)(self) = true;
                            self.buffs.buff_time[j] = 18000;
                            true
                        } else {
                            self.buffs.del_buff(j);
                            false
                        }
                    }
                    // Not a summon arm — the chain leaves the slot alone.
                    None => true,
                },
            };
            if survives {
                // `Player.cs:10874-10877` / `:10893-10896`: the companion
                // updaters run after their arm kept the buff.
                match buff {
                    335 => self.update_abigail_status(),
                    263 => self.update_storm_tiger_status(),
                    _ => {}
                }
                j += 1;
            }
            // else: the slot was DelBuff'd — rescan index j.
        }
        // `Player.cs:28279-28280`: the per-tick zero before ItemCheck — the
        // pool pass (Phase 3) accumulates the values the next tick's arms
        // read.
        self.num_minions = 0;
        self.slots_minions = 0.0;
    }

    /// Buff 216's arm (`Player.cs:10697-10726`): keep-alive on the 759 count;
    /// otherwise the respawn arm — below cap with the Baby Bird Staff (4281)
    /// in the inventory, spawn the item's shoot at the player's top with the
    /// item's damage/knockBack (`originalDamage = item.damage` at `:10713` —
    /// the data-row spawn seam writes it for minion rows); at cap (or without
    /// the staff) `DelBuff(j)` and no refresh (the C# `flag2` guard). Returns
    /// false when the slot was deleted — the caller rescans the index.
    fn run_baby_bird_arm(
        &mut self,
        pool: &mut ProjectilePool,
        inventory: &[ItemSlot; INV],
        j: usize,
    ) -> bool {
        if self.owned_projectile_counts[usize::from(BABY_BIRD)] > 0 {
            self.baby_bird = true;
        } else {
            // `base.whoAmI == Main.myPlayer` always holds in the
            // single-player port.
            if self.num_minions < self.max_minions {
                // `Player.FindItem` (`Player.cs:55972-55981`): the first
                // inventory slot with the type and a stack.
                if let Some(idx) = inventory.iter().position(|s| s.kind == 4281 && s.count > 0) {
                    if let Some(def) = registry::get(inventory[idx].kind) {
                        // `Projectile.NewProjectile(..., base.Top,
                        // Vector2.Zero, item.shoot, item.damage,
                        // item.knockBack, whoAmI)` — `Entity.Top` is the
                        // hitbox's top center; item 4281's `Item.shoot` is
                        // 759 (the generated row), so the verbatim data-row
                        // slice (minion flag, 1.0 slots, timeLeft 18000,
                        // the local pair) rides the spawn seam. The C#'s
                        // pool-full sentinel crash stands in as "not
                        // spawned".
                        if pool
                            .spawn_with_data_ai_owned(
                                BABY_BIRD,
                                0,
                                self.x + self.width * 0.5,
                                self.y,
                                0.0,
                                0.0,
                                def.damage,
                                def.knock_back,
                                0.0,
                                0.0,
                            )
                            .is_some()
                        {
                            self.baby_bird = true;
                        }
                    }
                }
            }
            if !self.baby_bird {
                self.buffs.del_buff(j);
                return false;
            }
        }
        self.buffs.buff_time[j] = 18000;
        true
    }

    /// `Player.UpdateAbigailStatus` (`Player.cs:12274-12293`) — STUB. Kills
    /// the 963 companions when the 970 count drops to 0 and spawns one beside
    /// the newest Abigail otherwise; both halves need the pool spawn/AI work
    /// (`FindNewestAI_164Minion`, the aiStyle-62 family) landing in M4-PROJ-8
    /// phases 4/6. Until then an orphan companion self-despawns through
    /// AI_062's flag read (`Projectile.cs:65045-65051`, Phase 4). Deviation
    /// recorded for ANALYSIS.
    fn update_abigail_status(&mut self) {}

    /// `Player.UpdateStormTigerStatus` (`Player.cs:12295-12328`) — STUB, the
    /// 833-835 rank management (`GetDesiredStormTigerMinionRank` + the
    /// `StormTigerIds` sets); lands with phases 4/6 like the Abigail stub.
    fn update_storm_tiger_status(&mut self) {}

    /// The `ResetEffects` summon-flag clear (`Player.cs:18620-18646`,
    /// `:18741`) — table-driven so the arms table stays the single source.
    fn clear_summon_flags(&mut self) {
        for arm in &SUMMON_BUFF_ARMS {
            *(arm.flag)(self) = false;
        }
    }

    /// Contact damage / future combat. Returns true when the hit was accepted,
    /// false when it was dodged or swallowed. `rng` is the caller's active
    /// stream (the C# dodge rolls draw from `Main.rand`, the S1-A-10 contract)
    /// and `dodgeable` is `Hurt`'s parameter (`Player.cs:38197` — the
    /// contact path passes `IsDamageDodgeable`, which rides Phase 2).
    pub fn hurt(
        &mut self,
        amount: i32,
        from_x: f32,
        rng: &mut dyn tr_core::loot::DropRng,
        dodgeable: bool,
    ) -> bool {
        // Every single-player C# `Hurt` lane passes `Crit: false` and no
        // projectile source (contact `Player.cs:31423`, lava `:27708`, tile
        // `:28981`, fall/petrify `:25223/:25352/:25370`); the projectile lane
        // (`Projectile.cs:14868`, also `Crit: false`) carries its type through
        // `hurt_with_source` for the dismount-on-hit gate (`:38396-38400`).
        self.hurt_inner(amount, from_x, true, rng, dodgeable, false, None, true)
    }

    /// The projectile-source `Hurt` (`PlayerDeathReason.ByProjectile`,
    /// `Projectile.cs:14868`): identical to [`Player::hurt`] plus the source
    /// projectile's type — the dismount-on-hit gate (`Player.cs:38396-38400`)
    /// reads `damageSource.SourceProjectileType`. `Crit: false` like the C#
    /// caller.
    pub(crate) fn hurt_with_source(
        &mut self,
        amount: i32,
        from_x: f32,
        rng: &mut dyn tr_core::loot::DropRng,
        dodgeable: bool,
        source_projectile: u16,
    ) -> bool {
        self.hurt_inner(
            amount,
            from_x,
            true,
            rng,
            dodgeable,
            false,
            Some(source_projectile),
            true,
        )
    }

    /// `Player.SetImmuneTimeForAllTypes` (`Player.cs:38013-38021`): the whole
    /// write rides the port's fields — `immune = true` + `immuneTime = time`
    /// (the fields R-03-5 ported; `immuneNoBlink` stays untouched, like the
    /// C#) and the per-cooldown array collapsed onto the one `hurt_cd` (the
    /// M4-HAZARD-1 admission). The dodge handlers pass the 120/80 pair
    /// directly; `iframes_for` (the normal-hurt window) is not involved.
    /// `pub(crate)`: the Strange Brew's bonus i-frames (`ApplyLifeAndOrMana`,
    /// `Player.cs:51912-51915`) call it from `potions.rs` (S1-P-i37).
    pub(crate) fn set_immune_time_for_all_types(&mut self, time: i32) {
        self.immune = true;
        self.immune_time = time;
        self.hurt_cd = time as f32 / 60.0;
    }

    /// `Player.ShadowDodge` (`Player.cs:38023-38042`) minus the NetMessage:
    /// the 120/80 immunity, buff 59 deleted by the same single-pass `for` with
    /// `DelBuff`'s compaction (`:38030-38040`), the Hallowed set-bonus
    /// cooldown armed (`PutHallowedArmorSetBonusOnCooldown` `:38037-38039` —
    /// it keeps the 30 s buff-59 cycle from re-granting).
    fn do_shadow_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
        for i in 0..crate::buffs::MAX_BUFFS {
            if self.buffs.buff_time[i] > 0 && self.buffs.buff_type[i] == 59 {
                self.buffs.del_buff(i);
            }
        }
        self.shadow_dodge_timer = 1800;
    }

    /// `Player.BrainOfConfusionDodge` (`Player.cs:38046-38055`) minus the
    /// NetMessage: the 120/80 immunity, the 300-tick animation window and the
    /// brain's own cooldown buff 321 (240 ticks — the roll's
    /// `FindBuffIndex(321)` gate).
    fn brain_of_confusion_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
        self.brain_of_confusion_dodge_animation_counter = 300;
        self.buffs.add_buff(321, 240);
    }

    /// `Player.DoMysticSashDodge` (`Player.cs:38057-38059`): the 120/80
    /// immunity; the 50x dust + gore burst is deferred to S1-E-1.
    fn do_mystic_sash_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
    }

    /// `Player.NinjaDodge` (`Player.cs:38119-38121`): the 120/80 immunity; the
    /// 100x dust + gore burst is deferred to S1-E-1.
    fn ninja_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
    }

    /// `Player.UpdateImmunity` (`Player.cs:18856-18869`): the i-frame
    /// countdown — `immune` clears at 0 and `immuneNoBlink` rides with it;
    /// the `immuneAlpha` blinkVisual half is draw-side and stays out.
    /// (R-03-5's port; the dodge handlers above write the same fields via
    /// `SetImmuneTimeForAllTypes`, so both windows decay through here.)
    pub fn update_immunity(&mut self) {
        if self.immune {
            self.immune_time -= 1;
            if self.immune_time <= 0 {
                self.immune = false;
                self.immune_no_blink = false;
            }
        }
    }

    /// The shared hurt body. `knock_back` mirrors the original's
    /// `hitDirection != 0` gate (`Player.cs:38519`): the fall-damage and
    /// petrify calls pass hitDirection 0 and skip the knock entirely (the
    /// fallStart reset inside that branch rides with it).
    /// `Player.Update`'s contact reflection (`Player.cs:31373-31415`): the
    /// damage an NPC takes for touching the player — `turtleThorns` pins the
    /// multiplier at 2, else `thorns` scales the NPC's own damage, capped at
    /// 1000; the Cactus set adds a flat 15 more.
    /// `Player.GetArmorPenetration` (`Player.cs:4365-4373`): the flat sum,
    /// plus the melee-only row when the hit is melee (`sItem.melee`).
    pub fn get_armor_penetration(&self, melee: bool) -> i32 {
        self.armor_penetration
            + if melee {
                self.melee_armor_penetration
            } else {
                0
            }
    }

    pub fn thorns_damage(&self, incoming: i32) -> i32 {
        let multiplier = if self.turtle_thorns { 2.0 } else { self.thorns };
        let mut total = 0;
        if multiplier > 0.0 {
            total += ((incoming as f32 * multiplier) as i32).min(1000);
        }
        if self.cactus_thorns {
            total += 15;
        }
        total
    }

    pub(crate) fn hurt_inner(
        &mut self,
        amount: i32,
        from_x: f32,
        knock_back: bool,
        rng: &mut dyn tr_core::loot::DropRng,
        dodgeable: bool,
        crit: bool,
        source_projectile: Option<u16>,
        retaliate_lane: bool,
    ) -> bool {
        // `Player.Hurt`'s creative-godmode arm (`Player.cs:38203-38206`): the
        // journey godmode power swallows the hit outright — before the
        // iframes, the defense reduction and the knockback.
        if self.creative_god_mode {
            return false;
        }
        // `Player.Hurt`'s General-cooldown gate (`Player.cs:38210-38216`):
        // the port's single damage lane rides `ImmunityCooldownID.General`,
        // whose flag is `!immune` — the spawn i-frames swallow the hit.
        if self.immune {
            return false;
        }
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        // The dodge roll block (`Player.cs:38214-38241`), slotted after the
        // cooldown early-return (the port's `hurt_cd`-gate IS the C# `flag`
        // for the General counter) and before the defense reduction.
        // `whoAmI == Main.myPlayer` collapses in the single-player port. The
        // roll order is sash -> belt -> brain -> shadow; each `Next` draws
        // only while its source is armed and the previous source did not
        // dodge (the early returns short-circuit the rest). The brain's
        // `Next(6)` precedes the buff-321 check in the C# `&&` chain — the
        // draw happens whenever the item is armed, buff 321 only gates the
        // dodge itself; shadow draws nothing.
        if dodgeable {
            if self.mystic_sash_dodge && rng.next_range(0, 10) == 0 {
                self.do_mystic_sash_dodge();
                return false;
            }
            if self.black_belt && rng.next_range(0, 10) == 0 {
                self.ninja_dodge();
                return false;
            }
            if self.brain_of_confusion
                && rng.next_range(0, 6) == 0
                && self.buffs.find_buff_index(321).is_none()
            {
                self.brain_of_confusion_dodge();
                return false;
            }
            if self.shadow_dodge {
                self.do_shadow_dodge();
                return false;
            }
        }
        // `Player.Hurt`'s defense reduction (the classic
        // `damage - statDefense * 0.5`, floored at 1 — M3-EQUIP-2). The
        // incoming raw damage stays the C# `num` (`Player.cs:38246`) — the
        // cuffs below restore it, NOT the reduced `num2` (`:38247`).
        // `Player.cs:38247-38251`: `num2` is computed from the PRE-crit
        // `Damage`, THEN `if (Crit) num *= 2` doubles the raw `num` the cuffs
        // restore (and the MP-only PVP return recompute `:38562-38565`).
        // Every single-player C# caller passes `Crit: false` (contact
        // `:31423`, projectile `Projectile.cs:14868`, lava `:27708`, tile
        // `:28981`, fall/petrify `:25223/:25352/:25370`); the crit-true
        // callers are the PVP pair (`Player.cs:44098`, `Projectile.cs:14672`)
        // — unreachable here, so the port's callers all pass false and the
        // flag rides for parity + the maths test.
        let raw = amount;
        let amount = tr_core::armor_sets::reduce_damage(amount, self.stat_defense);
        let raw = if crit { raw * 2 } else { raw };
        // `Player.Hurt`'s magicCuffs arm (`Player.cs:38291-38303`): after the
        // invis strip (`:38281-38290`, unported) and BEFORE the endurance step
        // (`:38304`) — `num3 = (int)(num * 1f)` is the raw damage (crit-doubled
        // at `:38248-38251`; every ported caller passes `crit = false`, the
        // single-player C# lanes all do), the
        // mana restores and caps at `statManaMax2`, then `ManaEffect(num3)`
        // queues the HealMana text (the NetMessage 43 send is MP-only).
        if self.magic_cuffs {
            self.mana += raw;
            if self.mana > self.mana_max {
                self.mana = self.mana_max;
            }
            self.pending_mana_effect = Some(raw);
        }
        // `Player.cs:38304`: the endurance step — `(int)((1 - endurance) *
        // damage)` after the defense reduction.
        let amount = ((1.0 - self.endurance) * amount as f32) as i32;
        // `Player.cs:38378-38381`: the `num2 < 1 -> 1` floor. The endurance
        // truncation can drive a hit to 0; the C# lifts it back to 1 (which
        // also selects the 20-tick immunity window below, not 40).
        let amount = amount.max(1);
        self.hp = (self.hp - amount).max(0);
        // `Player.Hurt`'s immunity window (`Player.cs:38382-38385`):
        // `int num7 = (pvp ? 8 : ((num2 != 1.0) ? (this.longInvince ? 80 : 40)
        // : (this.longInvince ? 40 : 20)));` then, on the General lane,
        // `this.immune = true; this.immuneTime = num7;`. The write lands on
        // the same `immune`/`immuneTime` fields the dodge handlers arm, so
        // `UpdateImmunity` (`:18856-18869`) runs the window down. PVP 8 is
        // unreachable in single-player. `hurt_cd` keeps carrying the window
        // in seconds for the single collapsed damage lane.
        let iframes = tr_core::damage::iframes_for(amount, false, self.long_invince);
        self.immune = true;
        self.immune_time = iframes;
        self.hurt_cd = iframes as f32 / 60.0;
        // `Player.Hurt` resets the natural-ramp timer (`Player.cs:38395`).
        self.regen.life_regen_time = 0.0;
        // `Player.cs:38396-38400`: the dismount-on-hit — the damage source's
        // projectile type is in `ProjectileID.Sets.DismountsPlayersOnHit`
        // (`ProjectileID.cs:546`: 877/878/879, the three golf-ball returns)
        // and a mount is active → `Mount.TryDismount(this)`. The port's
        // `try_dismount` needs `&World`, so the request queues for the
        // `world_sim` drain (the `pending_switch_trip` convention).
        if let Some(kind) = source_projectile {
            if matches!(kind, 877 | 878 | 879) && self.mount.is_active() {
                self.pending_dismount = true;
            }
        }
        // `Player.cs:38401`: `(whoAmI == Main.myPlayer && ImmunityCooldownID.
        // Sets.Retaliate[cooldownCounter]) & flag` — `flag` is the PRE-hit
        // not-immune state (`:38208`), which holds for every hit that reaches
        // this tail. The Retaliate set is {General, BossNoCheese,
        // PaladinsShield} (`ImmunityCooldownID.cs:49`); the port's collapsed
        // lane keeps the membership through `retaliate_lane` — the contact
        // and projectile lanes (General/BossNoCheese) and the fall/petrify
        // defaults pass true, the lava (`Lava`, `:27708`) and tile-contact
        // (`TileContactDamage`, `:28981`) lanes pass false (NOT members —
        // vanilla lava/spike hits never retaliate). The block needs the NPC
        // list + projectile pool + the hit lane's RNG: `hurt_modifiers::
        // drain_retaliate` rolls it at the `world_sim` seams with the final
        // post-floor damage (the C# `num2` after `:38378-38381`).
        if retaliate_lane {
            // The hit-moment snapshot: the C# block runs BEFORE `Hurt`'s own
            // knockback (`:38519-38524`), so the dart's `velocity * 0.3`, the
            // stars' aim and the bees' spawn read the pre-knockback state.
            self.pending_retaliate = Some(crate::hurt_modifiers::RetaliateRequest {
                damage: amount,
                x: self.x,
                y: self.y,
                center: self.center(),
                vx: self.vx,
                vy: self.vy,
            });
        }
        // `Player.Hurt`'s tail (`Player.cs:38525`): the hurt cue. The C#'s
        // voice system picks the family (`voiceVariant` 1 = the legacy
        // "Player Hit" id 1, 2 = the female id 20, 3 = the default trackable)
        // — the port has no voice setting, so the default variant 1 stands
        // (`Player.cs:56045`).
        self.pending_hurt_sound = Some("Player_Hit_0");
        // The knockback gate (`Player.cs:38519`): `!noKnockback &&
        // hitDirection != 0 && (!mount.Active || !mount.Cart)` — a mounted
        // CART rider takes no pop (the cart keeps the rails); every other
        // mount does. The C# dismount-on-hit (`:38396-38400`) runs BEFORE
        // this gate, so a golf-ball return pops the rider off the cart and
        // the knockback lands the same hit; the port's deferred drain
        // (ANALYSIS, wave B) dismounts at the pass tail, so the cart rider's
        // pop is suppressed on the hit tick instead (recorded deviation).
        if knock_back
            && !self.no_knockback
            && (!self.mount.is_active() || !crate::mount::is_cart(self.mount.kind()))
        {
            let dir = if self.x + self.width * 0.5 >= from_x {
                1.0
            } else {
                -1.0
            };
            // The fixed contact knockback (`Player.cs:38519-38524`):
            // `base.velocity.X = 4.5f * hitDirection`, `Y = -3.5f` and the
            // fall origin reset to the current row (`this.fallStart =
            // (int)(base.position.Y / 16f)`) so the pop never books a fall.
            self.vx = dir * 4.5;
            self.vy = -3.5;
            self.fall_start = (self.y / TILE_PX) as i32;
            self.on_ground = false;
        }
        true
    }

    /// The fall-damage landing hit (`Player.cs:25361-25370`): the normal branch
    /// forces `immune = false` BEFORE the Hurt — the hit lands through any
    /// iframes — and hitDirection 0 means no knockback and no fallStart reset.
    fn hurt_fall(&mut self, amount: i32, rng: &mut dyn tr_core::loot::DropRng) -> bool {
        // `Player.cs:25361`: `this.immune = false;` BEFORE the Hurt — a
        // landing ignores the i-frames the contact/lava lanes armed.
        self.immune = false;
        self.hurt_cd = 0.0;
        let cx = self.x + self.width * 0.5;
        // The landing hit keeps `Hurt`'s default `dodgeable = true` — a
        // black-belt dodge eats fall damage in the vanilla (research S1-C-1
        // §1.4).
        self.hurt_inner(amount, cx, false, rng, true, false, None, true)
    }

    /// `Player.tileRangeX` (`Player.cs:2496`, `DefaultTileRangeX = 5` at
    /// `:57113`) after `Player.ResetEffects`' journey block
    /// (`:18815-18828`): the base 5, or `5 * 2 + 8 = 18` while the
    /// `FarPlacementRangePower` is enabled. The C# gates the doubling on
    /// `Main.IsJourneyMode`; the port has no journey-difficulty flag, so the
    /// power's own toggle is the gate.
    ///
    /// S1-P-i43: the per-tick static writes AFTER the journey block ride in
    /// too — `UpdateEquips`' `equippedAnyTileRangeAcc` +3/+2
    /// (`Player.cs:13040-13044`, set by the Extendo Grip 2215 `:15267-15270`,
    /// the Architect's Gizmo Pack 3061 `:15279-15285` and the Ancient
    /// Manipulator-family 5126 `:15286-15294`) and `ApplyEquipFunctional`'s
    /// held Toolbox 1923 +1/+1 (`:14954-14958`). The chest range close reads
    /// this through `craft::tile_reach`'s `TileReachLimit = 20` clamp
    /// (`TileReachCheckSettings.cs:24-33`), the same seam the crafting scan
    /// uses (`craft.rs:547-578`).
    pub fn tile_range_x(&self) -> f32 {
        let base = 5.0;
        if self.far_placement_range {
            base * 2.0 + 8.0
        } else {
            base
        }
    }

    /// `Player.tileRangeY` (`DefaultTileRangeY = 3`, `Player.cs:57114`) with
    /// the same journey doubling (`:18823-18825`): `3` or `14`.
    pub fn tile_range_y(&self) -> f32 {
        let base = 3.0;
        if self.far_placement_range {
            base * 2.0 + 8.0
        } else {
            base
        }
    }

    /// `Player.pickSpeed` (`Player.cs:2466`, reset to 1f at `:18690`): the
    /// base 1f minus the positive bonus the buff/equipment grants write
    /// (WellFed `:11945`, Mining 104 `:10136`, the Ancient Chisel
    /// `:13032-13035`, the sky-stone bundle `:13000`), clamped at the
    /// `Player.Update` floor `if (pickSpeed < 0.3) pickSpeed = 0.3f`
    /// (`Player.cs:25972-25975`). A LOWER pickSpeed mines faster — it
    /// multiplies `useTime` into the mining `itemTime`
    /// (`Player.cs:45779`). The Hungry/Starving debuffs ADD to pickSpeed
    /// (`:11992`/`:12007`); the port's signed `pick_speed_bonus` keeps that
    /// direction (a negative bonus slows the dig).
    pub fn pick_speed(&self) -> f32 {
        (1.0 - self.pick_speed_bonus).max(0.3)

    }

    /// The full per-tick `Player.tileRangeX/Y` pair after the journey block,
    /// the `equippedAnyTileRangeAcc` accessory grant and the equipped Toolbox
    /// (`Player.cs:18815-18828` → `:13040-13044` → `:14954-14958`), clamped
    /// by `TileReachCheckSettings.Simple`'s `TileReachLimit = 20`
    /// (`TileReachCheckSettings.cs:24-33`, `:81-85`) — the exact reach
    /// `IsInTileInteractionRange` reads (`Player.cs:32025-32029`). The
    /// accessory scan is the C#'s `UpdateEquips` loop 3..10 over the
    /// equipped kinds (`:12946-12952`); the Toolbox 1923's +1/+1 fires from
    /// the SAME loop (`ApplyEquipFunctional`, `:14954` — an equipped
    /// accessory, not the held item). Delegates to `craft::tile_reach` —
    /// one formula, no second copy (S1-P-i43).
    pub fn tile_interaction_reach(&self) -> (i32, i32) {
        let range_acc = self.equipped[3..10]
            .iter()
            .any(|k| matches!(k, 2215 | 3061 | 5126));
        let toolbox = self.equipped[3..10].contains(&1923);
        crate::craft::tile_reach(
            range_acc,
            toolbox,
            self.far_placement_range,
            false, // the artisan loaf's TB is the AdjTiles scan's, not Simple's
        )
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// `Entity.Bottom` (`Entity.cs:122-132`):
    /// `(position.X + width/2, position.Y + height)` — the golf shot's
    /// legality-box anchor (`GolfHelper.ValidateShot`, `GolfHelper.cs:408`).
    pub fn bottom(&self) -> (f32, f32) {
        (self.x + WIDTH * 0.5, self.y + self.height)
    }

    /// `Player.meleeSpeed` AS READ by the game logic — the INVERTED,
    /// held-item-scaled use-time multiplier, not the raw speed sum.
    /// `UpdateEquips`' tail (`Player.cs:25912`) scales the accumulated speed
    /// by `ItemID.Sets.BonusMeleeSpeedMultiplier[selectedItem.type]`
    /// (`ItemID.cs:1350`: default 1f, Bladed Glove 1827 → 0.5f, Fetid
    /// Baghnakhs 3013 → 0.25f, Psycho Knife 3106 → 0.33f, Terra Blade 757 →
    /// 0.75f), then `CapAttackSpeeds` (`:29021-29026`) runs
    /// `TurnAttackSpeedToUseTimeMultiplier` (`:29028-29039`): cap at 3f,
    /// then `1f / speed` (a 0 speed stays 0). Every consumer reads THIS
    /// value: `ApplyItemAnimation`'s melee branch (`:4472`), the melee
    /// shoot's `speed /= meleeSpeed` (`:47063`) and the flail family's
    /// `1f / player.meleeSpeed` scales (`Projectile.cs:43727`,
    /// `:43824-43831`). The raw 1f + grants sum lives in
    /// [`Self::melee_speed_bonus`] (`Player.cs:2458`, reset `:18335`).
    pub fn melee_speed(&self, held_kind: u16) -> f32 {
        let bonus_mult = match held_kind {
            1827 => 0.5,
            3013 => 0.25,
            3106 => 0.33,
            757 => 0.75,
            _ => 1.0,
        };
        let mut speed = 1.0 + self.melee_speed_bonus;
        speed = 1.0 + (speed - 1.0) * bonus_mult;
        if speed > 3.0 {
            speed = 3.0;
        }
        if speed != 0.0 {
            speed = 1.0 / speed;
        }
        speed
    }

    /// `CapAttackSpeeds`' summoner half (`Player.cs:29025`):
    /// `summonerWeaponSpeedBonus = TurnAttackSpeedToUseTimeMultiplier(
    /// meleeSpeed + summonerWeaponSpeedBonus)`. The `meleeSpeed` it reads is
    /// the ALREADY held-item-scaled raw speed — `UpdateEquips`' tail rewrites
    /// `meleeSpeed = 1f + (meleeSpeed - 1f) * BonusMeleeSpeedMultiplier[type]`
    /// at `:25912` BEFORE `CapAttackSpeeds` runs (`:25976`), so the summoner
    /// fold carries that scale too (it just hasn't been inverted yet). Capped
    /// and inverted, then `ApplyItemAnimation`'s scaling-whip branch multiplies
    /// by `whipUseTimeMultiplier` (`:4476`).
    pub fn summoner_weapon_speed_multiplier(&self, held_kind: u16) -> f32 {
        let bonus_mult = match held_kind {
            1827 => 0.5,
            3013 => 0.25,
            3106 => 0.33,
            757 => 0.75,
            _ => 1.0,
        };
        let scaled_melee = 1.0 + self.melee_speed_bonus * bonus_mult;
        let mut speed = scaled_melee + self.summoner_weapon_speed_bonus;
        if speed > 3.0 {
            speed = 3.0;
        }
        if speed != 0.0 {
            speed = 1.0 / speed;
        }
        speed * self.whip_use_time_multiplier
    }

    /// `Player.tileSpeed` as `ApplyItemAnimation` reads it (`:4486`):
    /// `UpdateEquips`' tail caps the accumulated speed at 3f and inverts
    /// (`Player.cs:25913-25917`).
    pub fn tile_speed_multiplier(&self) -> f32 {
        1.0 / (1.0 + self.tile_speed_bonus).min(3.0)
    }

    /// `Player.wallSpeed` as `ApplyItemAnimation` reads it (`:4490`):
    /// the same cap-and-invert (`Player.cs:25918-25922`).
    pub fn wall_speed_multiplier(&self) -> f32 {
        1.0 / (1.0 + self.wall_speed_bonus).min(3.0)
    }

    /// Resync keep-one: main's i12 `melee_speed(held_kind)` /
    /// `summoner_weapon_speed_multiplier(held_kind)` read-only accessors
    /// above are canonical (the landed V4 form — `CapAttackSpeeds`
    /// `Player.cs:29021-29026` folded on demand instead of mutating the
    /// accumulator). The gate's mutating `cap_attack_speeds` +
    /// `turn_attack_speed_to_use_time_multiplier` copies were dropped;
    /// `weapon_knockback` keeps ONE def — the i25 `&ItemDef` form below
    /// (the census-corrected minionKB-first order, `Player.cs:53346-53369`).

    /// The whip swing's animation multiplier (`Player.ApplyItemAnimation`
    /// `:4474-4476`): `summonerWeaponSpeedBonus * whipUseTimeMultiplier` —
    /// the i12 read-only fold over the live held item (the gate's i24
    /// swing-tail seam, re-armed onto the canonical accessor at the resync).
    pub fn whip_animation_multiplier(&self) -> f32 {
        self.summoner_weapon_speed_multiplier(self.held_item_kind)
    }

    /// `Player.Update_SnakeBand` (`Player.cs:28412-28418`): the whip-repeat
    /// reset cooldown counts down one per tick; at 0 the swing counter
    /// returns to −1 (the next whip throw is swing #0). Called from the
    /// world-sim player pass at the `:24567` position.
    pub fn update_snake_band(&mut self) {
        if self.repeat_whips_reset_cooldown > 0 {
            self.repeat_whips_reset_cooldown -= 1;
            if self.repeat_whips_reset_cooldown == 0 {
                self.repeat_whip_swings = -1;
            }
        }
    }

    /// `Player.MountedCenter` (`Player.cs:3335-3343`):
    /// `position + (width/2, BaseHeight/2 + HeightOffsetHitboxCenter)`. The
    /// port has no `HeightOffsetBoost`/`HeightOffsetHitboxCenter`/mount
    /// offsets (no mounts yet), so both fold to 0 and the vector equals
    /// `center()` — kept as its own accessor so the flail chain anchor cites
    /// the C# point and survives the mount port.
    pub fn mounted_center(&self) -> (f32, f32) {
        self.center()
    }

    /// `Player.MountedTop` (`Player.cs:3349-3360`):
    /// `(position.X + width/2, position.Y + HeightOffsetHitboxCenter)`, or
    /// `Bottom.Y - HeightOffsetHitboxCenter` under reversed gravity. The port
    /// keeps `gravDir = 1` and `HeightOffsetHitboxCenter = 0` (no mount height
    /// offsets), so both fold to the body's top-centre — kept as its own
    /// accessor so the sentry-backpack reposition cites the C# point.
    pub fn mounted_top(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y)
    }

    /// S1-P-i35 — `Player.GetFishingConditions` (`Player.cs:42176-42206`)
    /// over the App's inventory layout: the grid 0-49, the coins 50-53 and
    /// the ammo 54-57 (the C# `inventory[58]`). Returns
    /// `(pole_power, pole_type, bait_power, bait_type, final_level)`.
    ///
    /// `Fishing_GetBestFishingPole` (`:42282-42298`): the held item first,
    /// then the strongest pole across 0..58. `Fishing_GetBait`
    /// (`:42254-42280`): the ammo slots 54-58 first, then the grid 0-50.
    /// The final level rides `tr_core::fishing::final_fishing_level` — the
    /// Tipsy buff 25 (`:42190`), the water-float (`canFloatInWater && wet`,
    /// `:42194`), the sitting bonus (`:42198`) and the power multiplier.
    pub fn fishing_conditions(
        &self,
        inventory: &[crate::item::ItemSlot],
        coins: &[crate::item::ItemSlot],
        ammo: &[crate::item::ItemSlot],
        selected: usize,
        world: &World,
        raining: bool,
        cloud_bg_alpha: f32,
    ) -> (i32, u16, i32, u16, i32) {
        let stat = |kind: u16| crate::registry::get(kind);
        // The held pole (`:42284-42289`).
        let held = inventory[selected];
        let mut pole_power = stat(held.kind).map_or(0, |d| d.fishing_pole);
        let mut pole_type = held.kind;
        if pole_power == 0 {
            // `:42290-42297`: the 0..58 scan (grid + coins + ammo).
            for sl in inventory.iter().chain(coins.iter()).chain(ammo.iter()) {
                let p = stat(sl.kind).map_or(0, |d| d.fishing_pole);
                if p > pole_power {
                    pole_power = p;
                    pole_type = sl.kind;
                }
            }
        }
        // The bait (`:42258-42279`): ammo first, then the grid.
        let mut bait_power = 0i32;
        let mut bait_type = 0u16;
        for sl in ammo.iter() {
            if sl.count > 0 {
                let b = stat(sl.kind).map_or(0, |d| d.bait as i32);
                if b > 0 {
                    bait_power = b;
                    bait_type = sl.kind;
                    break;
                }
            }
        }
        if bait_power == 0 && bait_type == 0 {
            for sl in inventory.iter() {
                if sl.count > 0 {
                    let b = stat(sl.kind).map_or(0, |d| d.bait as i32);
                    if b > 0 {
                        bait_power = b;
                        bait_type = sl.kind;
                        break;
                    }
                }
            }
        }
        let level = tr_core::fishing::final_fishing_level(
            pole_power,
            bait_type,
            bait_power,
            self.fishing_skill,
            // `:42190` — `FindBuffIndex(25) != -1` (Tipsy).
            self.buffs.find_buff_index(25).is_some(),
            // `:42194` — `canFloatInWater && wet`: the merman buff 265
            // (`:9895-9898`) or the Water Strider mount item 4404
            // (`:13265-13267`); the field is granted in the buff loop.
            self.can_float_in_water && self.wet,
            self.sitting,
            raining,
            cloud_bg_alpha,
            world.day_time,
            world.time,
            world.moon_phase as i32,
            world.blood_moon,
        );
        (pole_power, pole_type, bait_power, bait_type, level)
    }

    /// `Player.ApplyHeadOffsetFromMount` (`Player.cs:51544-51645`): the
    /// per-mount head offsets — the whole table is the four types 54/55/56/61
    /// (the other mounts fall through to the zero vector), each with its
    /// frame ladder. `Directions` is `(direction, gravDir)` (`Player.cs:3849`);
    /// the port keeps `gravDir = 1`, so the Y halves are plain adds.
    pub fn apply_head_offset_from_mount(&self, pos: Vec2) -> Vec2 {
        if !self.mount.is_active() {
            return pos;
        }
        let frame = self.mount.frame();
        let d = self.facing as f32;
        // `v * Directions` — the X half mirrors with the facing.
        let dirs = |v: Vec2| Vec2::new(v.x * d, v.y);
        let vector = match self.mount.kind() {
            // `:51552-51555`
            61 => dirs(Vec2::new(-2.0, 20.0)),
            // `:51556-51572`
            56 => {
                let mut v = dirs(Vec2::new(6.0, 18.0));
                if matches!(frame, 1 | 2 | 6 | 7 | 8 | 9) {
                    v += dirs(Vec2::new(0.0, 2.0));
                }
                v
            }
            // `:51573-51608`
            55 => {
                let mut v = dirs(Vec2::new(0.0, 18.0));
                v += match frame {
                    1 => dirs(Vec2::new(-2.0, 0.0)),
                    2 | 3 => dirs(Vec2::new(-4.0, -2.0)),
                    4..=11 => dirs(Vec2::new(4.0, 2.0)),
                    12 | 13 => dirs(Vec2::new(6.0, 0.0)),
                    14 => dirs(Vec2::new(6.0, 2.0)),
                    15 => dirs(Vec2::new(4.0, 2.0)),
                    _ => Vec2::ZERO,
                };
                v
            }
            // `:51609-51643`
            54 => {
                let mut v = dirs(Vec2::new(23.0, -8.0));
                v += match frame {
                    5 => dirs(Vec2::new(0.0, 4.0)),
                    6 => dirs(Vec2::new(2.0, 2.0)),
                    7 => dirs(Vec2::new(2.0, 0.0)),
                    8 | 9 | 11 | 12 | 13 | 15 => dirs(Vec2::new(4.0, 4.0)),
                    10 | 14 => dirs(Vec2::new(4.0, 2.0)),
                    16..=20 => dirs(Vec2::new(4.0, 4.0)),
                    _ => Vec2::ZERO,
                };
                v
            }
            _ => Vec2::ZERO,
        };
        pos + vector
    }

    /// `Player.RotatedRelativePoint(pos)` (`Player.cs:4225-4251`) with the
    /// default arguments — the full-rotation pivot every held-item anchor and
    /// the minion home spot ride. The sleeping/sitting offset arms
    /// (`:4240-4249`) are not modelled: the port's sleep is a pose flag with no
    /// offset info, and the C# zeroes the angle for a sleeping player anyway
    /// (`:4228-4231`).
    pub fn rotated_relative_point(&self, pos: Vec2, push_from_origin_hack: i32) -> Vec2 {
        let num = if self.sleeping { 0.0 } else { self.full_rotation };
        // `base.Bottom + (0, gfxOffY)`.
        let bottom = Vec2::new(self.x + self.width * 0.5, self.y + self.height);
        let vector = bottom + Vec2::new(0.0, self.gfx_off_y);
        // `mount.PlayerOffset / 2 + 4 + pushFromOriginHack` — integer division.
        let num2 = (self.mount.player_offset() / 2 + 4 + push_from_origin_hack) as f32;
        let vector2 =
            Vec2::new(0.0, -num2) + Vec2::new(0.0, num2).rotate(Vec2::from_angle(num));
        let p = pos + Vec2::new(0.0, self.gfx_off_y);
        vector + (p - vector).rotate(Vec2::from_angle(num)) + vector2
    }

    /// `Player.killGuide` / `Player.killClothier` (`Player.cs:2770`/`:2772`):
    /// `ApplyEquipFunctional` arms them for the worn accessory slots 3..10
    /// (`:14368`, the `UpdateEquips` loop `:12948-12954` — `IsItemSlotUnlockedAndUsable`
    /// is not modelled, every slot counts as unlocked) and `ResetEffects`
    /// clears them every tick (`:18365-18366`). The melee gate
    /// (`ProcessHitAgainstNPC` `:44274`) and the projectile gate
    /// (`Damage_PVE_Inner` `:12601`) both read this pair.
    pub fn doll_flags(&self) -> (bool, bool) {
        let worn = |k: u16| self.equipped[3..10].contains(&k);
        (worn(267), worn(1307))
    }

    /// `Player.IsItemSlotUnlockedAndUsable` (`Player.cs:12579-12604`): the
    /// slot 8 needs the Demon Heart's `extraAccessory` AND expert mode, the
    /// slot 9 needs master mode. Deviations: the `Main.gameMenu` escape arms
    /// read false in the sim; the port has no master difficulty yet, so the
    /// slot 9 stays locked (docs/ANALYSIS.md "The UpdateEquip seam"). The
    /// Demon Heart's use-grant (`:45197`) is unported - the flag stays false
    /// in play until the consumable use lands.
    fn is_item_slot_unlocked_and_usable(&self, slot: usize) -> bool {
        match slot {
            8 | 18 => self.extra_accessory && self.buffs.expert_mode,
            9 | 19 => false, // `Main.masterMode` - unported difficulty
            _ => true,
        }
    }

    /// W4-c14 doc-sweep - `Player.UpdateBootVisualEffects`
    /// (`Player.cs:36704-36751`): the boot kind -> VISUAL rocket tier. Only
    /// the tier-setting arms land (the rocket cue reads the tier); the
    /// `CancelAllBootRunVisualEffects` dash flags (`sailDash`/`coldDash`/
    /// `desertDash`/`fairyBoots`/`hellfireTreads`, `:36664-36671`) and the
    /// Hermes-54 cancel are run-particle visual seams. `None` = the kind
    /// sets no visual tier (the C# switch falls through).
    fn boot_visual_tier(kind: u16) -> Option<i32> {
        match kind {
            128 => Some(1),          // `:36708-36710` Rocket Boots
            405 | 898 | 3993 => Some(2), // `:36714-36718` Spectre/Lightning + `:36745-36749` Fairy
            1862 => Some(3),         // `:36724-36727` Frostspark
            5000 => Some(4),         // `:36728-36731` Terraspark
            4874 => Some(5),         // `:36719-36723` Hellfire Treads
            _ => None,
        }
    }

    /// `Player.UpdateEquips`' functional-accessory loop (`Player.cs:12946-12952`):
    /// for the slots 3..10, `IsItemSlotUnlockedAndUsable` gates the slot and
    /// `ApplyEquipFunctional` applies the worn kind. `GetEffectiveArmor`
    /// (`:5684-5710`) collapses to `equipped[slot]` - no loadout system in the
    /// port. The `:12922` gates ride along: `IsAir` (kind 0),
    /// `UpdateEquips_CanItemGrantBenefits` (`:12869-12890` - the slots 3..9
    /// demand `item.accessory`) and the accessory-only `GrantPrefixBenefits`
    /// (`:12924-12927`). The `expertOnly` gate (`:14370-14373`) is unmodelled:
    /// `ItemDef` carries no `expert_only` and every ported grant kind is
    /// classic (deviation, docs/ANALYSIS.md "The UpdateEquip seam").
    fn update_equips(&mut self) {
        for m in 3..10 {
            if !self.is_item_slot_unlocked_and_usable(m) {
                continue;
            }
            let kind = self.equipped[m];
            if kind == 0 {
                continue;
            }
            if !crate::registry::get(kind).is_some_and(|d| d.accessory) {
                continue;
            }
            crate::prefix::grant_prefix_benefits(self.equipped_prefix[m], self);
            // W4-c14 doc-sweep - `ApplyEquipFunctional`'s visible-boot half
            // (`Player.cs:14378-14380`): `UpdateBootVisualEffects` runs only
            // when the slot is NOT hidden. The functional grant below is
            // unconditional (the C# switch sits outside the gate).
            if !self.hide_visible_accessory[m] {
                if let Some(tier) = Self::boot_visual_tier(kind) {
                    self.vanity_rocket_boots = tier;
                }
            }
            self.apply_equip_functional(m, kind);
        }
        // W4-c14 doc-sweep - `UpdateEquips`' vanity-accessory loop
        // (`Player.cs:13009-13015` -> `ApplyEquipVanity` `:14197` ->
        // `UpdateBootVisualEffects` `:14234`): the slots 13..20 re-derive the
        // visual boot tier from the SOCIAL accessory (no hide gate on this
        // loop). The other `ApplyEquipVanity` grants (shader/wings/merman/
        // bobber/voice) stay draw seams; only the rocket-cue tier lands.
        for n in 13..20 {
            if !self.is_item_slot_unlocked_and_usable(n) {
                continue;
            }
            if let Some(tier) = Self::boot_visual_tier(self.equipped[n]) {
                self.vanity_rocket_boots = tier;
            }
        }
        // W4-c14 doc-sweep - `Player.cs:13088-13091`: the Red's Leggings 668
        // in the leg armour slot (2) OR its vanity slot (12) sets the visual
        // rocket tier 6 (the bee-costume rocket cue, Item24 cadence 15).
        // `GetEffectiveArmor` collapses to `equipped[slot]` (no loadouts).
        // This runs AFTER the accessory loops, so it wins like the C#.
        if self.equipped[2] == 668 || self.equipped[12] == 668 {
            self.vanity_rocket_boots = 6;
        }
        // `UpdateEquips`' luck tail (`Player.cs:13066-13069`): ONE +0.05
        // `equipmentBasedLuckBonus` add when `hasLuck_LuckyCoin` — the flag
        // the Lucky Coin family arms (855/3034/3035); two rings still add
        // 0.05 once. (`Main.myPlayer`-only in the C# — the port's single
        // player.) The clover/horseshoe/raven-feather siblings
        // (`:13050-13065`) are unported families.
        if self.has_lucky_coin {
            self.luck_state.equipment_based_luck_bonus += 0.05;

        }
        // `Player.UpdateEquips`' tail (`Player.cs:13097-13100`):
        // `mount.UpdateAfterEquips(this)` — the Velociraptor's selective-fly
        // permission (`Mount.cs:2989-2997`, the empress-brooch refill reads
        // `empressBrooch` + `wingTimeMax`) and the Rat's wall-slide grants
        // (`:2998-3033`).
        if self.mount.is_active() {
            self.mount.update_selective_flight(
                self.wings_logic,
                self.empress_brooch,
                self.wing_time_max,
            );
            // `:3002-3003` — the Rat (55) grants a spiked-boot level and
            // centres the full rotation on the (shrunk) body.
            if self.mount.kind() == 55 {
                self.spiked_boots += 1;
                self.full_rotation_origin = (self.width / 2.0, self.height / 2.0);
                // `:3004-3033` — the slide-direction rotation lerp. The
                // port's `slide_dir` rides the slope collision's writes.
                let half_pi = std::f32::consts::FRAC_PI_2;
                if self.slide_dir >= 1 {
                    if half_pi - self.full_rotation.abs() <= 0.1 {
                        self.full_rotation = -half_pi;
                    } else {
                        self.full_rotation =
                            crate::npc::angle_lerp(self.full_rotation, -half_pi, 0.5);
                    }
                } else if self.slide_dir <= -1 {
                    if half_pi - self.full_rotation.abs() <= 0.1 {
                        self.full_rotation = half_pi;
                    } else {
                        self.full_rotation =
                            crate::npc::angle_lerp(self.full_rotation, half_pi, 0.5);
                    }
                } else if self.full_rotation.abs() <= 0.1 {
                    self.full_rotation = 0.0;
                } else {
                    self.full_rotation = crate::npc::angle_lerp(self.full_rotation, 0.0, 0.5);
                }
            }
        }
    }

    /// `Player.ApplyEquipFunctional` (`Player.cs:14368-15560`): the per-kind
    /// grant chain for one functional accessory slot. The ported arms keep
    /// the C#'s grants; the visual half (`UpdateBootVisualEffects`/bobber/
    /// voice, `:14378-14390`) and the `hideVisibleAccessory` gate are draw
    /// seams, out of scope. `_item_slot` is the C# `itemSlot` (the visibility
    /// lookups only).
    fn apply_equip_functional(&mut self, _item_slot: usize, kind: u16) {
        // S1-C-12 wave E — the combat/damage family has ONE grant path:
        // `accessory_combat::apply_combat_accessory` claims its kinds here
        // and the former inline arms (489/490/491/2998/935/897/936/1343/
        // 1301/3015) are deleted from the match below, so nothing grants
        // twice. The family kinds are disjoint from every remaining arm
        // (the C# rows are independent per-type `if`s, `Player.cs:14368-
        // 15450`), so claiming them before the match is order-identical.
        if crate::accessory_combat::apply_combat_accessory(kind, self) {
            return;
        }
        // `ApplyEquipFunctional`'s mana stat switch (`Player.cs:14922-14935`,
        // a `switch` INDEPENDENT of the per-type `if` chain the match below
        // ports — both run per slot): the Band of Starpower 111 and the
        // cuffs 1595/2221 add +40 `statManaMax2`; the Mana Regeneration Band
        // 982, the Restoration Shield 6188 and the Mystic Arts Sash 6189 add
        // +40 AND `manaRegenBonus += 60` (`:14933`).
        match kind {
            111 | 1595 | 2221 => self.mana_max += 40,
            982 | 6188 | 6189 => {
                self.mana_max += 40;
                self.mana_regen_bonus += 60;
            }
            _ => {}
        }
        match kind {
            // S1-P-i29 — the Ancient Chisel (`Player.cs:14396-14398`):
            // `chiselSpeed = true`, folded by `UpdateEquips` into
            // `pickSpeed -= 0.25f` (`:13032-13035`). The port keeps the
            // positive-bonus convention (a lower pickSpeed mines faster),
            // so the fold lands here directly — the same sum the C#
            // reaches, one grant path. The Architect's Gizmo Pack (5126)
            // carries the same `chiselSpeed` (`:15293`) plus the range
            // flag the reach scan reads (`craft.rs`).
            4056 | 5126 => self.pick_speed_bonus += 0.25,
            // S1-P-i35 — `ApplyEquipFunctional`' fishing rows
            // (`Player.cs:14651-14683`), in the C#'s order: the High Test
            // Line 2373, the Angler Earring 2374 (+10), the glow-bobber
            // family 5139-5146 (the flag + the projectile override via
            // `UpdateFishingBobber` `:36673-36702`), the Tackle Box 2375,
            // the Lavaproof Hook 4881, the combos 3721 (line+box+10) and
            // 5064 (line+box+10+lava).
            2373 => self.acc_fishing_line = true,
            2374 => self.fishing_skill += 10,
            5139..=5146 => {
                self.acc_fishing_bobber = true;
                // `UpdateFishingBobber` (`:36677-36700`): 5139→986 …
                // 5146→993 (the C# calls it from the same equip pass,
                // `:14381`).
                self.override_fishing_bobber = 986 + (kind - 5139) as i32;
            }
            2375 => self.acc_tackle_box = true,
            4881 => self.acc_lava_fishing = true,
            3721 => {
                self.acc_fishing_line = true;
                self.acc_tackle_box = true;
                self.fishing_skill += 10;
            }
            5064 => {
                self.acc_fishing_line = true;
                self.acc_tackle_box = true;
                self.fishing_skill += 10;
                self.acc_lava_fishing = true;
            }
            // The bottles (`Player.cs:14997-15001`, :14793-14811, :15001):
            53 => self.has_jump_options[ExtraJump::Cloud.index()] = true,
            857 => self.has_jump_options[ExtraJump::Sandstorm.index()] = true,
            987 => self.has_jump_options[ExtraJump::Blizzard.index()] = true,
            1724 => self.has_jump_options[ExtraJump::Fart.index()] = true,
            3201 => self.has_jump_options[ExtraJump::Sail.index()] = true,
            // The bottle combos (`:14822-14835`): 1164 (Bundle of
            // Balloons) and 5331 (its Amber/Glass variant) carry all three
            // bottles at once.
            1164 | 5331 => {
                self.has_jump_options[ExtraJump::Cloud.index()] = true;
                self.has_jump_options[ExtraJump::Sandstorm.index()] = true;
                self.has_jump_options[ExtraJump::Blizzard.index()] = true;
                self.jump_boost = true;
            }
            // Every other bottle/balloon carries its own option *and*
            // `jumpBoost` (`:14806/:15093-15103`, `:14797-14860`): `399`
            // Shiny Red Balloon and `1250` Cloud in a Balloon the Cloud
            // row, `1251`/`1163` the Blizzard one, `983`/`1252` the
            // Sandstorm one, `1863`/`3250` the Fart one, `3252`/`3241` the
            // Sail one. `3251` (honeycomb balloon) and `1249` (Honey
            // Comb) carry the boost AND `honeyCombItem` (`:14871-14882`,
            // S1-C-7 wave B); `159`/`3225` (the Aglet-style `jumpSpeedBoost`
            // rows) carry only the boost.
            399 | 1250 => {
                self.has_jump_options[ExtraJump::Cloud.index()] = true;
                self.jump_boost = true;
            }
            1251 | 1163 => {
                self.has_jump_options[ExtraJump::Blizzard.index()] = true;
                self.jump_boost = true;
            }
            983 | 1252 => {
                self.has_jump_options[ExtraJump::Sandstorm.index()] = true;
                self.jump_boost = true;
            }
            1863 | 3250 => {
                self.has_jump_options[ExtraJump::Fart.index()] = true;
                self.jump_boost = true;
            }
            3252 | 3241 => {
                self.has_jump_options[ExtraJump::Sail.index()] = true;
                self.jump_boost = true;
            }
            159 | 3225 => self.jump_boost = true,
            // The Honey Comb 1249 (`Player.cs:14878-14882`): `jumpBoost` +
            // `honeyCombItem` only.
            1249 => {
                self.jump_boost = true;
                self.honey_comb = true;
            }
            // The Honey Balloon 3251 (`:14871-14877`): the same pair PLUS
            // `noFallDmg` (its `hasLuck_LuckyHorseshoe` half `:14876` has no
            // port field - deferred). The `noFallDmg` row lives HERE, not in
            // the horseshoe arm below: this arm matches first, so a 3251
            // pattern there would be unreachable.
            3251 => {
                self.jump_boost = true;
                self.honey_comb = true;
                self.no_fall_dmg = true;
            }
            // The base boots (`:15005-15008` Hermes / `:15013-15016` Flurry /
            // `:15017-15020` Sailfish - `accRunSpeed = 6f`).
            54 | 1579 | 3200 => self.acc_run_speed = 6.0,
            // The Amphibian Boots (`:14399-14404`): the base run speed plus
            // the auto-jump, the +1.6 jump-speed add and +10 safe-fall tiles.
            // (`autoJump` re-derives per tick; a mount's own `constantJump`
            // head-set still overwrites it in `update_mount`.)
            3990 => {
                self.acc_run_speed = 6.0;
                self.auto_jump = true;
                self.jump_speed_boost += 1.6;
                self.extra_fall += 10;
            }
            // The Fairy Boots (`:14418-14421`): the Spectre row (6f + tier 2).
            3993 => {
                self.acc_run_speed = 6.0;
                self.rocket_boots = 2;
                self.vanity_rocket_boots = 2; // `:14420` (W4-c14 doc-sweep)
            }
            // The Sand Boots (`:14422-14425`): 6f plus the desert-run flag
            // (the sand-floor 1.75x rides the floor-modifier chain).
            4055 => {
                self.acc_run_speed = 6.0;
                self.desert_boots = true;
            }
            // The Rocket Boots (`:15021-15024` - `rocketBoots = 1`).
            128 => {
                self.rocket_boots = 1;
                self.vanity_rocket_boots = 1; // `:15023` (W4-c14 doc-sweep)
            }
            // The Spectre Boots (`:15219-15223` - tier 2).
            405 => {
                self.acc_run_speed = 6.0;
                self.rocket_boots = 2;
                self.vanity_rocket_boots = 2; // `:15222` (W4-c14 doc-sweep)
            }
            // The Lightning Boots (`:15376-15380` - tier 2, 6.75 speed,
            // +8% move).
            898 => {
                self.acc_run_speed = 6.75;
                self.rocket_boots = 2;
                self.vanity_rocket_boots = 2; // `:15380` (W4-c14 doc-sweep)
                self.move_speed += 0.08;
            }
            // The Frostspark Boots (`:15382-15388` - tier 3 and the ice
            // skates).
            1862 => {
                self.acc_run_speed = 6.75;
                self.rocket_boots = 3;
                self.vanity_rocket_boots = 3; // `:15386` (W4-c14 doc-sweep)
                self.move_speed += 0.08;
                self.ice_skate = true;
            }
            // The Terraspark Boots (`:15390-15396` - tier 4).
            5000 => {
                self.acc_run_speed = 6.75;
                self.rocket_boots = 4;
                self.vanity_rocket_boots = 4; // `:15393` (W4-c14 doc-sweep)
                self.move_speed += 0.08;
                self.ice_skate = true;
                // The Terraspark row (`:15168-15174`): the water/fire walk
                // and the lava charm's 420 ticks.
                self.lava_max += 420;
            }
            // The Hellfire Treads (`:15397-15401` - tier 5, 6f).
            4874 => {
                self.acc_run_speed = 6.0;
                self.rocket_boots = 5;
                self.vanity_rocket_boots = 5; // `:15400` (W4-c14 doc-sweep)
            }
            _ if Self::wing_slot(kind as i32) > 0 => {
                // `:15593-15602`: the accessory's `wingSlot` is both the
                // gameplay wing (`wingsLogic`) and the look (`wings`, same
                // field in the port), and `:15453` takes the flight budget
                // from the same id's `ArmorIDs.Wing.Sets.Stats`.
                let slot_id = Self::wing_slot(kind as i32);
                self.wings_logic = slot_id;
                self.has_wings = true;
                self.wing_time_max = Self::wing_stats(slot_id).0;
            }
            // The Titan Glove's melee-speed row (`:15103-15107`) plus its
            // `autoReuseGlove` grant — `TryAllowingItemReuse`
            // (`:52905-52920`) forces the reuse edge for melee weapons.
            211 => {
                self.melee_speed_bonus += 0.12;
                self.auto_reuse_glove = true;
            }
            // The Wicked Claws (`:14600-14604`): the armlet flag + the
            // Titan-Glove pair (`autoReuseGlove` + meleeSpeed +0.12). The
            // resync keep-one: the i24 whip block below ALSO carried a 6182
            // arm (armlet + meleeSpeed, no glove) — first-match-wins shadowed
            // it; this i12 arm is the C#-complete one, the duplicate dropped.
            6182 => {
                self.acc_wicked_armlet = true;
                self.melee_speed_bonus += 0.12;
                self.auto_reuse_glove = true;
            }
            // The Stress Ball (`:14730-14733`): `CanUseStressBall` gates the
            // grant (`:14342-14366`). The port stands in for the extractinate
            // hover (false) and the food-in-hand arm (the food buff check
            // rides the use path); the standing-still + damage arms are the
            // reachable ones. `TryAllowingItemReuse` reads the flag.
            5543 => {
                let standing_still = self.vx.abs() < 0.05 && self.vy.abs() < 0.05;
                let held = self.held_item_kind;
                let held_def = crate::registry::get(held);
                let food_arm = !self.well_fed
                    && held_def.is_some_and(|d| matches!(d.buff_type, 26 | 206 | 207));
                let coin_or_bomb = (71..=74).contains(&held) || held == 1338;
                let damage_arm = held_def.is_some_and(|d| d.damage > 0) || held == 1124;
                if standing_still && (food_arm || (!coin_or_bomb && damage_arm)) {
                    self.stress_ball = true;
                }
            }
            // The damage/crit accessories OUTSIDE the combat family
            // (`ApplyEquipFunctional`): the Sniper Scope's +10 ranged
            // crit/+10% ranged damage (1858, `Player.cs:14907-14911`),
            // the Celestial Emblem's +15% magic + `manaMagnet` (2220,
            // `:14944-14948`, S1-P-i52), the minion
            // emblems +15%/+10% (1167/1864/1845, `:15315-15330` — their
            // `minionKB += 2f` halves have no live consumer yet, the
            // `maxMinions++` budget rides `grant_summon_caps`) and the
            // Empress Brooch's +7.5% move (4989). The class emblems
            // (489/490/491/2998), the Avenger (935), the gloves
            // (897/936/1343/3992), the Destroyer Emblem (1301), the
            // Putrid Scent (3015) and the Flesh Knuckles (3016) are
            // claimed by `accessory_combat::apply_combat_accessory`
            // ABOVE — their inline arms were deleted in wave E
            // (S1-C-12e) so each kind grants exactly once.
            1858 => {
                self.ranged_crit += 10;
                self.ranged_damage += 0.1;
            }
            2220 => {
                self.magic_damage += 0.15;
                // `:14946`: the Celestial Emblem's `manaMagnet` (S1-P-i52).
                self.mana_magnet = true;
            }
            // The Celestial Magnet 2219 (`Player.cs:14940-14943`): the bare
            // `manaMagnet` flag (S1-P-i52).
            2219 => self.mana_magnet = true,
            1167 => self.minion_damage += 0.15,
            // 1864/1845 also carry `maxMinions++` in the C#; the budget
            // half is `grant_summon_caps`' (`world_sim/player.rs`), so only
            // the damage row lands here (the C# total counts once).
            1864 => self.minion_damage += 0.15,
            1845 => self.minion_damage += 0.1,
            4989 => {
                // `Player.cs:14771-14775`: the Soaring Insignia — the brooch
                // flag (infinite flight + the jump/acceleration adds, the
                // mount refill reads it) and the +7.5% move.
                self.empress_brooch = true;
                self.move_speed += 0.075;
            }
            // The merman shells (`:15134-15143` Moon Charm 861,
            // `:15402-15410` Celestial Shell 3110, `:15566-15573` Neptune's
            // Shell 497): `accMerman` arms the wet transformation the flight
            // gates read (`CanUseWingAbilities` `:3994`, `WingAirLogicTweaks`
            // `:26496`). The `wolfAcc`/`hideMerman` halves are the werewolf /
            // visibility families (out of flight scope).
            861 | 3110 | 497 => self.acc_merman = true,
            // The penetration rows: the Shark Tooth Necklace (3212,
            // `:13286-13289`) and the Honey Comb (4007, `:14485-14489`)
            // add a flat 5 each.
            3212 | 4007 => self.armor_penetration += 5,
            // The Lava Charm family (`:15168-15183`): 906 (Lava Charm),
            // 4038 (Molten Charm), 3999 (Molten Skull Rose) and 4003
            // (Lava Waders) each add 420 ticks of lava immunity; 908
            // (Lava Waders' older row) and 5000 (Terraspark Boots) also
            // set the water/fire walk and the lava rose.
            906 | 3999 | 4003 | 4038 => self.lava_max += 420,
            // (The Terraspark Boots' own row rides its boot arm above —
            // a duplicate pattern here would be unreachable.)
            908 => self.lava_max += 420,
            // The Worm Scarf (`:15081-15084`): +17% endurance.
            3224 => self.endurance += 0.17,
            // The Mana Flower (`:15591-15594`), the Arcane Flower
            // (`:14405-14410` — also +5 magic crit / +5% magic damage; the
            // aggro -400 has no port consumer), the Magnet Flower
            // (`:14455-14457`, its manaMagnet half unported) and the Mana
            // Cloak (`:14459-14460`) arm `manaFlower`; the ONE -0.08
            // `manaCost` row applies after the equip loop (`:12962-12965`),
            // so two flowers do not stack. Nature's Gift (`:15110-15113`)
            // keeps its own per-item -0.06 row.
            555 => self.mana_flower = true,
            // The Magnet Flower 4000 (`:14455-14458`): `manaFlower` +
            // `manaMagnet` (S1-P-i52) — ONE arm, the i19 flag + the i52
            // magnet merged (a second 4000 arm below would be unreachable).
            4000 => {
                self.mana_flower = true;
                self.mana_magnet = true;
            }
            3991 => {
                self.mana_flower = true;
                self.magic_crit += 5;
                self.magic_damage += 0.05;
            }
            223 => self.mana_cost -= 0.06,
            // The Philosopher's Stone family (S1-P-i37): 535
            // (`:15574-15577`) and the Charm of Myths 860 (`:15150-15153`)
            // arm `pStone` — the Potion Sickness durations ×0.75
            // (`Player.cs:25481-25486`).
            535 | 860 => self.p_stone = true,
            // The Ice Skates (`:15089-15092`).
            950 => self.ice_skate = true,
            // The Aglet / the Anklet of the Wind (`:15114-15121`).
            285 => self.move_speed += 0.05,
            212 => self.move_speed += 0.1,
            // `Player.cs:15029-15033` + the horseshoe balloons: the fall
            // damage immunity. (3251's row lives in its Honey Balloon arm
            // above - a pattern here would be unreachable.)
            158 | 1250 | 1251 | 1252 | 3250 | 3252 => self.no_fall_dmg = true,
            // `Player.cs:15025-15028` - the Cobalt Shield.
            156 => self.no_knockback = true,
            // The sentry-backpack slings (`ApplyEquipFunctional`,
            // `Player.cs:14506-14508`/`:14564-14579`): the Heavy Sling
            // (6172) grants the bare flag; the Scout Sling (6175) adds
            // +5% to all four damage classes (its `aggro -= 400` half has
            // no port field — skipped, no aggro consumer); the Templar
            // Sling (6176) also arms `accGolemSentryBackpack` (no port
            // consumer — the Golem-backpack crit arm is unported); the
            // Royal Guard's Harness (6177) adds the Cobalt Shield's
            // knockback immunity.
            6172 => self.acc_sentry_backpack = true,
            6175 => {
                self.acc_sentry_backpack = true;
                self.melee_damage += 0.05;
                self.magic_damage += 0.05;
                self.ranged_damage += 0.05;
                self.minion_damage += 0.05;
            }
            6176 => self.acc_sentry_backpack = true,
            6177 => {
                self.acc_sentry_backpack = true;
                self.no_knockback = true;
            }
            // (Resync keep-one: the i27 whip-accessory flag block that sat
            // here was the STALE subset — the i24 block below carries the
            // same rows PLUS the 6157 Snake Band / 6183 Silver Shield arms
            // and the live `acc_snake_band` grants. First-match-wins made
            // these arms shadow the fuller ones; dropped.)
            // The dodge family (`ApplyEquipFunctional`): the Black Belt
            // 963 (`:15058-15061`) and the Master Ninja Gear 984
            // (`:15062-15067`) both arm `blackBelt`; the Mystic Arts Sash
            // 6189 (`:14632-14635`) arms `mysticSashDodge` and the Brain
            // of Confusion 3223 (`:15085-15088`) arms the (bool
            // stand-in for the) `brainOfConfusionItem` reference.
            963 => self.black_belt = true,
            984 => {
                self.black_belt = true;
                // `:15065`: the Tabi's dash. (`spikedBoots = 2` at `:15066`
                // has no port field - the wall-cling is unported.)
                self.dash_type = 1;
            }
            // The dash granters: the Tabi (`:15050-15053`, type 1) and the
            // Shield of Cthulhu (`:15054-15057`, type 2).
            977 => self.dash_type = 1,
            3097 => self.dash_type = 2,
            6189 => self.mystic_sash_dodge = true,
            3223 => self.brain_of_confusion = true,
            // The Magic Cuffs (1595, `ApplyEquipFunctional`
            // `Player.cs:14936-14939`) and the Celestial Cuffs (2221,
            // `:14949-14953`) arm `magicCuffs`. Their +40 `statManaMax2`
            // rows (`:14925-14934`) ride C-12; 2221's `manaMagnet` half
            // (`:14951`) arms the star magnet (S1-P-i52).
            1595 => self.magic_cuffs = true,
            2221 => {
                self.magic_cuffs = true;
                self.mana_magnet = true;
            }
            // The Lucky Coin family (`Player.cs:14975-14996`): 855 Lucky
            // Coin arms `hasLuckyCoin` + `hasLuck_LuckyCoin`; 3033 Gold Ring
            // arms `goldRing` alone; 3034 Coin Ring arms goldRing + the
            // lucky coin; 3035 Greedy Ring adds `discountEquipped` (no port
            // consumer — the shop-price lane). The two lucky flags are
            // always set/cleared together (`:14977-14978`/`:14987-14988`/
            // `:14993-14994`, `:18760`/`:18762`), so one bool carries both;
            // the +0.05 luck add is the UpdateEquips TAIL's single
            // `hasLuck_LuckyCoin` gate (`:13066-13069`), applied once after
            // the slot loop — NOT per accessory (two rings still add 0.05).
            855 => self.has_lucky_coin = true,
            3033 => self.gold_ring = true,
            3034 => {
                self.gold_ring = true;
                self.has_lucky_coin = true;
            }
            3035 => {
                self.gold_ring = true;
                self.has_lucky_coin = true;
            }
            // The Magnet Flower 4000's manaMagnet + manaFlower arm lives in
            // the flower-flag block above (deduped keep-one — the per-item
            // `manaCost -= 0.08` i52 stand-in was dropped at the i19 resync;
            // the ONE -0.08 fold rides the post-loop `mana_flower` gate,
            // `:12962-12965`).
            // The S1-C-7 wave B retaliate family (`Player.Hurt`
            // `:38401-38517`): the star cloak + its four override kinds, the
            // honeycomb family and the strong-bee flag. The C# carries Item
            // references; the port's bools are the armed-flag equivalents
            // (the `brain_of_confusion` convention).
            // Star Cloak 532 (`:15583-15586`).
            532 => self.star_cloak = true,
            // Star Veil 862 (`:15144-15149`): the cloak + `longInvince` +
            // the starVeil override (star type 725).
            862 => {
                self.star_cloak = true;
                self.long_invince = true;
                self.star_veil_override = true;
            }
            // Bee Cloak 1247 (`:14959-14964`): the cloak + the honeycomb +
            // the beeCloak override (star type 724).
            1247 => {
                self.star_cloak = true;
                self.honey_comb = true;
                self.bee_cloak_override = true;
            }
            // Mana Cloak 4001 (`:14459-14463`): `manaFlower` + the cloak +
            // the manaCloak override (star type 723). The flower flag arms
            // here; the ONE -0.08 `manaCost` row rides the post-loop
            // `mana_flower` fold (`:12962-12965`, the 555 convention).
            4001 => {
                self.mana_flower = true;
                self.star_cloak = true;
                self.mana_cloak_override = true;
            }
            // Serpent's Coil 6186 (`:14620-14624`): the cloak + the
            // snakeCloak override (star type 1133) + `accSnakeBand` (the
            // whip-repeat flag, `:14621`).
            6186 => {
                self.star_cloak = true;
                self.snake_cloak_override = true;
                self.acc_snake_band = true;
            }
            // The whip accessory flags (`ApplyEquipFunctional`,
            // `Player.cs:14509-14624`): Silver Bracer 6156 (`:14510`),
            // Mobius Strip 6158 (`:14513`), Wicked Armlet 6159 (`:14516`),
            // Ouroboros Ring 6162 (`:14519-14520`, Mobius + Snake), Twilight
            // Grasp 6163 (`:14523`, Silver Bracer + Wicked), Snake Band 6157
            // (`:14526`), Armlet of Ruin 6179 (`:14585`, Wicked), Wicked
            // Claws 6182 (`:14601-14603`, Wicked + meleeSpeed +0.12), Silver
            // Shield 6183 (`:14606`, Silver Bracer — its knockback/fire-walk
            // halves ride `accessory_defense`). The `maxTagEffects` /
            // `whipRangeMultiplier` / `tagEffectDuration` consumers fold in
            // `update_equips`' tail (`:12970-12984`).
            6156 => self.acc_silver_bracer = true,
            6158 => self.acc_mobius_strip = true,
            6159 => self.acc_wicked_armlet = true,
            6162 => {
                self.acc_mobius_strip = true;
                self.acc_snake_band = true;
            }
            6163 => {
                self.acc_silver_bracer = true;
                self.acc_wicked_armlet = true;
            }
            6157 => self.acc_snake_band = true,
            6179 => self.acc_wicked_armlet = true,
            // (Resync keep-one: the 6182 Wicked Claws arm lives in the i12
            // Titan-Glove block above — armlet + autoReuseGlove + meleeSpeed,
            // the C#-complete `:14600-14604` set. This block's old copy
            // lacked the glove grant and was shadowed anyway.)
            6183 => self.acc_silver_bracer = true,
            // Honey Comb 1132 (`:15072-15075`).
            1132 => self.honey_comb = true,
            // Panic Necklace 1578 (`:15076-15080`): the honeycomb + `panic`.
            // The port has no `panic` field — the hurt buff `AddBuff(63,
            // 480)` (`:38237-38240`) stays deferred (ANALYSIS, wave B).
            1578 => self.honey_comb = true,
            // Stinger Necklace 6184 (`:14611-14615`): the honeycomb
            // (`accPoisonBarb`/`accHoneyedBarb` are unported families).
            6184 => self.honey_comb = true,
            // S1-P-i10 + i37 keep-both (M1 re-sync): the Catalyst Band grants
            // BOTH flags — `Player.cs:14615-14619`: `accPoisonBarb`, `pStone`,
            // `catalystBand`. Magma Stone 1322 (`:15336-15339`): the bare flag;
            // the OnFire3 infliction is `melee_on_hit.rs` (`:6552-6566`).
            1322 => self.magma_stone = true,
            // Pyroclast 6178 (`:14580-14583`): magmaStone + accPyroclast
            // (the snapping-stone strike `:44338-44341` is an unported
            // projectile family — only the flag half lands here).
            6178 => self.magma_stone = true,
            // Catalyst Band 6185 (`:14615-14619`): catalystBand + pStone +
            // accPoisonBarb (the AcceleratePoisons infliction is
            // `melee_on_hit.rs`, `:6536-6539`; the Stone family's third
            // member rides i37's potion-delay rescale).
            6185 => {
                self.catalyst_band = true;
                self.p_stone = true;
            }
            // Hive Pack 3333 (`:15344-15347`): `strongBees`.
            3333 => self.strong_bees = true,
            // `UpdateLifeRegen`'s `lifeRegen` base (the Band's
            // `Item.lifeRegen = 2`, `Item.cs:1974` -> `Player.cs:13122`).
            // (The `Item.lifeRegen` rows are generic now — the loop after
            // the accessory match adds every worn item's value.)
            _ => {}
        }
    }

    /// `Player.GetWeaponDamageMultiplier` (`Player.cs:53386`) narrowed to
    /// the ported classes: the classless items pass the 1f default.
    pub fn weapon_damage_multiplier(&self, class: registry::DamageClass) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => self.ranged_damage,
            registry::DamageClass::Magic => self.magic_damage,
            registry::DamageClass::Summon => self.minion_damage,
            _ => 1.0,
        }
    }

    /// `Player.GetWeaponDamageMultiplier` in full (`Player.cs:53386-53419`),
    /// the item-aware form: the ranged dispatch keys on the ammo category —
    /// arrows → `bowEffectiveDamage` (`:3982`), bullets →
    /// `gunEffectiveDamage` (`:3984`), the specialist rockets/everything
    /// else → `specialistEffectiveDamage` (`:3986`) — and falls to
    /// `rangedDamage` for the plain guns. `use_ammo` is the weapon's
    /// `useAmmo` (the `PickAmmo_PickAmmoItem` equivalence makes it the
    /// resolved ammo's `ammo` field too — the proof `is_arrow_ammo`'s doc
    /// carries); `item_type` feeds `ItemID.Sets.IsRangedSpecialistWeapon`
    /// (`ItemID.cs:1472`). The fuse sites call this; the class-only callers
    /// keep [`Player::weapon_damage_multiplier`].
    pub fn weapon_damage_multiplier_with_ammo(
        &self,
        class: registry::DamageClass,
        item_type: u16,
        use_ammo: i32,
    ) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => {
                if crate::ammo::is_arrow_ammo(use_ammo) {
                    return (self.ranged_damage / self.ranged_mult_damage
                        + self.arrow_damage_additive_stack)
                        * self.ranged_mult_damage
                        * self.arrow_damage;
                }
                if crate::ammo::is_bullet_ammo(use_ammo) {
                    return self.ranged_damage * self.bullet_damage;
                }
                if crate::ammo::is_specialist_ammo(use_ammo)
                    || crate::ammo::IS_RANGED_SPECIALIST_WEAPONS.contains(&item_type)
                {
                    return self.ranged_damage * self.rocket_damage;
                }
                self.ranged_damage
            }
            registry::DamageClass::Magic => self.magic_damage,
            registry::DamageClass::Summon => self.minion_damage,
            _ => 1.0,
        }
    }

    /// The class crit source (`Player.meleeCrit`/`rangedCrit`/`magicCrit`,
    /// `Player.cs:2430-2432`; base 4 per class).
    pub fn class_crit(&self, class: registry::DamageClass) -> i32 {
        match class {
            registry::DamageClass::Melee => self.melee_crit,
            registry::DamageClass::Ranged => self.ranged_crit,
            registry::DamageClass::Magic => self.magic_crit,
            _ => 0,
        }
    }

    /// `Player.GetWeaponKnockback` (`Player.cs:53346-53369`): the summon
    /// staff's shot knockback takes the `minionKB` bonus FIRST (`:53348-53351`,
    /// `if (sItem.summon) KnockBack += this.minionKB`), then the melee glove's
    /// ×2 (`:53352-53355`), the Titan buff's ×1.5 (`:53356-53359`) and the
    /// shroomite-stealth ranged scale (`:53360-53363`). The `setVortex` ranged
    /// arm (`:53364-53367`) has no port field (the vortex stealth is unported)
    /// and folds out. `ItemCheck_Shoot` calls this at `:47151` before the
    /// spawn, so the minion/sentry shots carry the bonus — the port's throw
    /// path previously passed the raw `fx_kb` and dropped it.
    pub fn weapon_knockback(&self, def: &registry::ItemDef, knock_back: f32) -> f32 {
        let mut kb = knock_back;
        if def.summon {
            kb += self.minion_kb;
        }
        if def.melee && self.kb_glove {
            kb *= 2.0;
        }
        if self.kb_buff {
            kb *= 1.5;
        }
        if def.ranged && self.shroomite_stealth {
            kb *= 1.0 + (1.0 - self.stealth) * 0.5;
        }
        kb
    }

    /// `Player.ItemCheck`'s food consumption side (`:14353`'s gate): the
    /// eat plays through the swing; the flag refreshes next tick's scan.
    pub fn eat_food(&mut self) {
        // The vanilla re-eat gate reads `wellFed` (the scan-derived flag);
        // the caller already gated on `!well_fed`, so nothing extra here —
        // the slot's buff grant is the state change.
    }

    /// Direct health loss with no knockback or i-frames, used by drowning and fall
    /// damage (`statLife -= ...` in `Player.cs`).
    fn lose_health(&mut self, amount: i32) {
        if self.hp <= 0 {
            return;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = self.hurt_cd.max(0.8);
        // No ramp reset here: the C# direct `statLife -=` paths (drowning, fall
        // damage) leave `lifeRegenTime` untouched.
    }

    /// `UpdateDyes` (`Player.cs:9587-9614`): the per-tick dye cache, refilled
    /// after the `ResetEffects` clears and before the `UpdateEquips` grants
    /// (`Player.Update` `:25491-25633`). The C# zero-fills ~36 `c*` fields
    /// (`:9589-9592`) and `UpdateItemDye` (`:9616-9757`) re-derives the
    /// accessory effect-slot fields from the armor items; the port keeps the
    /// fields with draw consumers — the armor trio reads its dye slots
    /// directly (`cHead/cBody/cLegs = GetEffectiveDye(0..2).dye`,
    /// `:9593-9595`) and the misc five mirror `:9600-9604`. The `wearsRobe →
    /// cLegs = cBody` rule (`:9596-9599`) and the accessory mappings wait for
    /// their ported visuals (docs/ANALYSIS.md "Dye deviations").
    pub(crate) fn update_dye_cache(&mut self) {
        self.c_head = dye::shader_id_for(self.dye_slots[0]);
        self.c_body = dye::shader_id_for(self.dye_slots[1]);
        self.c_legs = dye::shader_id_for(self.dye_slots[2]);
        // `cPet/cLight/cMinecart/cMount/cGrapple = miscDyes[0..4].dye`
        // (`:9600-9604`); `cYorai = cPet` (`:9613`) has no draw consumer.
        self.c_pet = dye::shader_id_for(self.misc_dye_slots[0]);
        self.c_light = dye::shader_id_for(self.misc_dye_slots[1]);
        self.c_minecart = dye::shader_id_for(self.misc_dye_slots[2]);
        self.c_mount = dye::shader_id_for(self.misc_dye_slots[3]);
        self.c_grapple = dye::shader_id_for(self.misc_dye_slots[4]);
    }

    /// One fixed 60 Hz step.
    /// The test-facing tick wrapper: the C# `Player.Update` reads the global
    /// `Main.rand`; the port threads the caller's stream (`update_with_grapple`).
    /// Production always goes through the world-sim's `UpdatePlayers` stage
    /// stream, so this wrapper supplies a scratch seed-0 stream for the
    /// gear-less test players (an armed dodge source would draw from it
    /// deterministically — nothing else).
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        let mut scratch = tr_core::UnifiedRandom::default();
        self.update_with_grapple(world, input, &[], false, false, false, None, &mut scratch)
    }

    /// The full tick with the optional grappling pull (`GrappleMovement`):
    /// while a hook is attached the pull REPLACES the normal movement
    /// (`Player.cs:27395-27399` gates the whole run/jump/wing/dash block).
    /// `item_in_use` is `player.itemAnimation > 0` and `danger_near` the
    /// `NPC.AnyDanger` stand-in — both feed the sleeping state's checks.
    pub fn update_with_grapple(
        &mut self,
        world: &World,
        input: PlayerInput,
        // `Player.grappling[]`/`grapCount` as a `(ProjectileID, center,
        // rotation)` triple per registered hook (`Player.cs:2742`/`:2744`) —
        // the pool rebuilds it on its own pass, so the caller only hands the
        // set over. The rotation is read by the Squirrel Hook's steering alone.
        grapple: &[(u16, Vec2, f32)],
        item_in_use: bool,
        danger_near: bool,
        // `Main.raining` + the wind model: the Cute Fishron's special getter
        // (`Player.cs:4048-4056`) needs both; `update` passes calm defaults.
        raining: bool,
        wind: Option<&tr_core::wind::Wind>,
        // The stage stream the C# swaps in for the player update
        // (`Main.cs:17755-17757`): the world-sim passes the `UpdatePlayers`
        // named stream; the dodge rolls and the lava `Hurt` inside the tick
        // draw from it (S1-A-10 — no new persistent stream).
        rng: &mut dyn tr_core::loot::DropRng,
    ) {
        // `Player.grappling[0] != -1` is rebuilt per tick by the attached
        // hook's AI (`Player.cs:28300` + `Projectile.cs:52860-52864`) — the
        // caller passes the registered set, so derive it here.
        self.grappling_active = !grapple.is_empty();
        // `Player.Update`'s sickness clocks (`Player.cs:24718-24724`): both
        // cooldowns decrement once per tick BEFORE `UpdateBuffs` (`:25566`)
        // re-mirrors `potionDelay` from the live buff-21 timer (S1-P-i37).
        if self.potion_delay > 0 {
            self.potion_delay -= 1;
        }
        if self.mana_potion_delay > 0 {
            self.mana_potion_delay -= 1;
        }
        // The `flag9` pStone latch (`Player.cs:25477`): snapshotted BEFORE
        // `ResetEffects` (`:25491`) clears it; the post-`UpdateEquips` change
        // check (`:25640-25643`) rescales the live buff-21 timer.
        self.p_stone_prev = self.p_stone;
        // The buff-frame head of `Player.Update` (`Player.cs:25555-25616`): the
        // `buffImmune` reset precedes `UpdateBuffs` and every re-grant; the
        // end-of-update purge consumes the table at `:26011-26016`.
        self.buffs.reset_buff_immune();
        // `Player.Update` → `UpdateBuffs(i)` (`Player.cs:24535`): the timer head runs
        // early in the tick. Nothing grants buffs yet, so this only decrements.
        self.buffs.update_buffs_tick();
        // `Player.Update` → `UpdateImmunity` (`Player.cs:25456`): the spawn
        // i-frames count down before the AI.
        self.update_immunity();
        // `Player.cs:24728` — `UpdateFailedDismount`: the QuickMount no-space
        // strike count clears on death / dismount / moving off the tile.
        self.update_failed_dismount();
        // `Player.cs:25625-25628`: the Cute Fishron's special window decays
        // one per tick; `Mount.UpdateEffects` re-arms it later in the tick.
        if self.mount_fishron_special_counter > 0.0 {
            self.mount_fishron_special_counter -= 1.0;
        }
        // `Player.cs:24591`: the tag stack's update right after the buff head —
        // the `maxTagEffects` cap plus the per-NPC mark/proc timers.
        self.tag_effects.update_tick();
        // `Player.cs:24600-24604`: the dash-age counter ticks and caps at 300
        // (the wings-45 run-slowdown ×6 reads it at >= 60).
        self.time_since_last_dash_started += 1;
        if self.time_since_last_dash_started >= 300 {
            self.time_since_last_dash_started = 300;
        }
        // `Player.cs:24761-24764`: every resting tick recovers 2 hover
        // fatigue — the read is the PREVIOUS tick's velocity (the head runs
        // before this tick's movement).
        if self.vy == 0.0 {
            self.mount.fatigue_recovery();
        }
        // `Player.cs:9881-9913`: a mount-type buff re-affirms its mount every
        // tick (`SetMount` — the `Mount.cs:6198` same-type no-op guard makes
        // the re-call a no-op while it is already up) and holds
        // `buffTime = 10` — the buff is the source of truth. The five
        // water-mount sub-arms (`:9885-9912`) grant their swim flags.
        for bt in self.buffs.active_types() {
            if let (Some(m), Some(i)) = (
                crate::mount::mount_type_of_buff(bt),
                self.buffs.find_buff_index(bt),
            ) {
                let wet = self.wet;
                self.set_mount(m, wet);
                self.buffs.buff_time[i] = 10;
                // The per-buff water grants (`:9885-9912`) run in the main
                // buff loop below — AFTER the `ResetEffects` clears (the C#
                // `ResetEffects` `:25491` precedes `UpdateBuffs` `:25566`);
                // granting them here at the tick head would be wiped by the
                // `ignoreWater`/`accFlipper`/`canFloatInWater`/`lavaVision`
                // resets later in this same tick (gate V3, S1-P-i41).
            }
        }
        // Buffs whose timer ran out are deleted right after the tick
        // (`Player.cs:25608-25616`) — before any effect can read them.
        self.remove_expired_buffs();
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
        // `Player.Update`'s Hallowed-cooldown decay (`Player.cs:25461-25464`,
        // right after `UpdateImmunity`): one tick off the 1800 shadow-dodge
        // cycle timer per frame.
        if self.shadow_dodge_timer > 0 {
            self.shadow_dodge_timer -= 1;
        }
        // `Player.Update`'s frozen/webbed control lock: the frozen flag
        // zeroes the movement controls (`Player.cs:10057`'s flag gates the
        // run keys), the webbed one zeroes velocity outright (the scan
        // above). Gravity still applies to a frozen player.
        let input = if self.frozen || self.webbed {
            PlayerInput::default()
        } else {
            input
        };
        // The run keys ride on the player like the C#'s `controlLeft`/`controlRight`
        // (input struct → fields early in `Player.Update`).
        self.control_left = input.left;
        self.control_right = input.right;
        self.control_jump = input.jump;
        self.control_down = input.down;
        self.control_up = input.up;
        // `Player.cs:24996-25004`: Confused (31) swaps the controls AFTER
        // they land: `flag2 = controlLeft`, `flag3 = controlUp`, then
        // left↔right, `controlUp = controlRight` (post-swap, i.e. the old
        // left) and `controlDown = flag3` (the pre-swap up).
        if self.confused {
            let flag3 = self.control_up;
            std::mem::swap(&mut self.control_left, &mut self.control_right);
            self.control_up = self.control_right;
            self.control_down = flag3;
        }
        // `PlayerSittingHelper.UpdateSitting` (`Player.cs:26125`) and
        // `PlayerSleepingHelper.UpdateState` (`:26126`): the stand-up/wake
        // gates run after the controls land, before the movement applies.
        self.update_sitting(world);
        self.update_sleeping(world, item_in_use, danger_near);
        let (cx, cy) = self.center();
        let mid = world.tile((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
        // `Collision.WetCollision` / `LavaCollision` set the four wet flags
        // (`Player.cs:27728-27910`); the mid-tile liquid kind picks them.
        let liquid = mid.filter(|t| t.liquid > 0).map(|t| t.liquid_kind());
        // S1-E-21f — the entry/exit splash plays (`Player.cs:27772-27944`).
        // The C#'s `base.wet` is WetCollision (ANY liquid); the port's four
        // flags split the kinds, so the edge is the any-liquid union. The
        // `wetCount == 0` gate (`:27774`/`:27868`) arms the 10-tick cooldown
        // (`:27776`/`:27870`) and suppresses re-entry churn; the tail
        // decrement is `:27959-27961`. Styles: entry shimmer 2 (`:27806`),
        // honey/lava the default 1 (`:27819`/`:27846`), water 0 (`:27832`);
        // exit shimmer 3 (`:27900`), honey 1 (`:27913`), water 0 (`:27926`),
        // the old-lava arm 1 (`:27940`). The dust bursts are visual seams.
        let was_any_wet = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        let now_wet = liquid == Some(tr_core::tile::LiquidKind::Water);
        let now_lava = liquid == Some(tr_core::tile::LiquidKind::Lava);
        let now_honey = liquid == Some(tr_core::tile::LiquidKind::Honey);
        let now_shimmer = liquid == Some(tr_core::tile::LiquidKind::Shimmer);
        let now_any_wet = now_wet || now_lava || now_honey || now_shimmer;
        if now_any_wet && !was_any_wet && self.wet_count == 0 {
            self.wet_count = 10;
            // `:27777` — the shimmering (absorbed) state suppresses the play;
            // the port has no absorbed state (S1-E-16), so the guard is vacuous.
            // `:27779` — `flag25` is the NEW `Collision.LavaCollision`
            // (`:27671`): a lava entry plays the `:27846` default-1 arm;
            // the dry arms read the freshly-set shimmer/honey flags.
            let style = if now_lava {
                1 // `:27846` — the lava arm, default Style 1
            } else if now_shimmer {
                2 // `:27806`
            } else if now_honey {
                1 // `:27819`
            } else {
                0 // `:27832`
            };
            self.pending_splash_sound = Some((style, self.x, self.y));
        } else if !now_any_wet && was_any_wet {
            // `:27864` — the exit jump clamp (`jump > jumpHeight/5`) is a
            // movement seam, not a sound one; the splash gate stands alone.
            if self.wet_count == 0 {
                self.wet_count = 10;
                let style = if self.lava_wet {
                    1 // `:27940` — flag24, the OLD lava flag
                } else if self.shimmer_wet {
                    3 // `:27900`
                } else if self.honey_wet {
                    1 // `:27913`
                } else {
                    0 // `:27926`
                };
                self.pending_splash_sound = Some((style, self.x, self.y));
            }
        }
        self.wet = now_wet;
        self.lava_wet = now_lava;
        self.honey_wet = now_honey;
        self.shimmer_wet = now_shimmer;
        self.in_water = self.wet || self.honey_wet;
        self.in_lava = self.lava_wet;
        // `:27959-27961` — the splash cooldown ticks at the wet block's tail.
        if self.wet_count > 0 {
            self.wet_count -= 1;
        }

        // The fall origin is only reset post-movement now: the landing/rest
        // block (`velocity.Y == 0`, `Player.cs:25383`) and the continuous
        // condition (`Player.cs:25379`). The old pre-movement `vy <= 0` reset
        // measured falls from every upward apex — the vanilla code never does.

        // `Player.cs:25295-25297`: `fallStart2` captures the pre-movement row
        // while rising or resting — it feeds the rail-landing gate
        // (`Minecart.cs:809-813`) and nothing else.
        if self.vy <= 0.0 {
            self.fall_start2 = (self.y / TILE_PX) as i32;
        }

        // `Player.cs:24364-24371`: the per-tick movement defaults;
        // `moveSpeed` multiplies acceleration and the primary gate while
        // `accRunSpeed` takes only the debuff (`Player.cs:26004-26010`).
        self.max_run_speed = 3.0;
        self.run_acceleration = 0.08;
        self.run_slowdown = 0.2;
        self.acc_run_speed = self.max_run_speed;
        // `Player.cs:24366-24367`: the jump statics reset per tick (the C#
        // writes the statics `Player.jumpHeight = 15` / `jumpSpeed = 5.01f`).
        self.jump_height = 15;
        self.jump_speed = 5.01;
        // `Player.cs:24380-24437`: the wet/shimmer statics rebase — the C#
        // runs it right after the defaults and BEFORE `UpdateJumpHeight`'s
        // max-lift (`:25656`), so a wet assignment (30/6.01 water) survives
        // the lift and the later max can only raise it. The rebased pair
        // feeds the plain integration arm below at full strength.
        let (tick_gravity, tick_max_fall) = {
            let flags = WetStaticsFlags {
                wet: self.wet,
                honey_wet: self.honey_wet,
                lava_wet: self.lava_wet,
                shimmer_wet: self.shimmer_wet,
                ..WetStaticsFlags::default()
            };
            let (g, mf, jh, js) =
                wet_gravity_and_cap(GRAVITY, MAX_FALL, self.jump_height, self.jump_speed, flags);
            self.jump_height = jh;
            self.jump_speed = js;
            (g, mf)
        };
        // `UpdateJumpHeight` (`:19587-19593`): the balloon/jump-boost lift is
        // a MAX over the rebased statics, not an assignment.
        if self.jump_boost {
            self.jump_height = self.jump_height.max(20);
            self.jump_speed = self.jump_speed.max(6.51);
        }
        self.move_speed = 1.0;
        self.strongest_move_speed_debuff = 1.0;
        // `Player.cs:24371-24374`: a mounted cart keeps `onWrongGround` across
        // the tick (the on-track check below rewrites it); everyone else
        // clears it at the defaults.
        if !self.cart_mounted {
            self.on_wrong_ground = false;
        }
        // `Player.ResetEffects` + `UpdateBuffs`' slot scan (`Player.cs:18640/
        // :9855`, the debuff rows `:10039-10061`, Weak `:11873-11878`, Webbed
        // `:11865-11883`, Darkness→blind `:9969-9971`): the flags derive per
        // slot, Weak's stat mods subtract per slot. The melee base recalc
        // (`UpdateDamageRecalc`) has no other ported modifier yet, so the
        // reset stands in for it (deviation until M3-EQUIP-3).
        self.slow = false;
        self.chilled = false;
        self.frozen = false;
        self.webbed = false;
        // S1-P-i41 — the water-mount / Cursed flag clears of `ResetEffects`:
        // `canFloatInWater = false` (`Player.cs:18332`), `accFlipper = false`
        // (`:18361`), `noItems = false` / `cursed = false` (`:18687-18688`),
        // `spikedBoots = 0` (`:18660`), `ignoreWater = false` /
        // `lavaVision = false` (`:18756-18757`). The mount buff arm
        // (`:9881-9913`) and the Cursed (23) debuff arm (`:11733-11734`)
        // re-grant them per tick.
        self.can_float_in_water = false;
        self.acc_flipper = false;
        self.no_items = false;
        self.cursed = false;
        // `Player.cs:18455`: `empressBrooch = false`.
        self.empress_brooch = false;
        self.spiked_boots = 0;
        self.ignore_water = false;
        self.lava_vision = false;

        // `Player.cs:18506` — `ResetEffects` clears `GoingDownWithGrapple` per
        // tick; `GrappleMovement` (`:22655-22658`) re-sets it only on a down pull.
        self.going_down_with_grapple = false;
        // `Player.cs:18786`: justJumped clears at the tick head; the auto-jump
        // flag re-derives per tick from the mount head-set (`Mount.cs:4797`).
        self.just_jumped = false;
        self.auto_jump = false;
        // `Player.cs:18787-18788`: the jump-boost add and the safe-fall tiles
        // clear per tick; the Amphibian Boots re-grant them below.
        self.jump_speed_boost = 0.0;
        self.extra_fall = 0;
        // `Player.cs:18802-18804`: `ResetEffects` clears the Actuation Rod's
        // lock ONLY while `releaseUseItem` rides. The port runs the clear at
        // the fixed-step seam (`world_sim/player.rs`, the `mine_released`
        // latch = the use button's release state), NOT unconditionally here:
        // a held button keeps the lock across ticks (one actuation per hold).
        // `Player.cs:18524`/`:18534`: the mechanical-accessory flags clear
        // with the rest of `ResetEffects`; `UpdateEquips`' 58-slot scan and
        // the armor/held grants re-set them per tick
        // (`RefreshMechanicalAccsFromItemType`, `:12799-12820`).
        self.info_acc_mech_show_wires = false;
        self.auto_actuator = false;
        // `Mount.CheckMountBuff` via `ResetEffects` (`Player.cs:18832` ->
        // `Mount.cs:5275-5281`): the buff gone means dismount.
        if self.mount.is_active()
            && self
                .mount
                .data()
                .is_some_and(|d| self.buffs.find_buff_index(d.buff).is_none())
        {
            self.try_dismount(world);
        }
        self.blind = false;
        // The S1-E-7 wave-A potion flags' `ResetEffects` clears
        // (`Player.cs:18355` fireWalk, `:18367-18374` lavaImmune/gills/
        // slowFall/findTreasure/biomeSight/invis/nightVision/enemySpawns,
        // `:18378` waterWalk, `:18380-18381` detectCreature/gravControl,
        // `:18394` manaRegenBuff, `:18650` dazed, `:18695` confused,
        // `:18705` silence) — the buff arms below re-set them per tick.
        self.lava_immune = false;
        self.fire_walk = false;
        // S1-C-12 wave D — the armor-set flag clears of `ResetEffects`
        // (`Player.cs:18325` ashWoodBonus, `:18364` spaceGun, `:18458`
        // shadowArmor, `:18686` frostBurn, `:18722` crimsonRegen, `:18730`
        // vampireBurningInSunlight): the `armor_sets.rs` seam re-grants
        // them per tick at the `UpdateArmorSets` position (`:25685`).
        // `noKnockback`'s `:18356` clear rides the accessory-flag block
        // below (the shields share it).
        self.ash_wood_bonus = false;
        self.space_gun = false;
        self.shadow_armor = false;
        self.frost_burn = false;
        // S1-P-i10 — `ResetEffects`' on-hit accessory flag clears
        // (`Player.cs:18608` catalystBand, `:18610` magmaStone): the
        // `apply_equip_functional` arms (1322/1343/6178/6185) re-grant
        // them per tick from the worn slots.
        self.magma_stone = false;
        self.catalyst_band = false;
        self.crimson_regen = false;
        self.vampire_burning_in_sunlight = false;
        self.gills = false;
        self.mana_regen_buff = false;
        self.slow_fall = false;
        self.find_treasure = false;
        self.biome_sight = false;
        self.invis = false;
        self.night_vision = false;
        self.enemy_spawns = false;
        self.water_walk = false;
        self.detect_creature = false;
        self.grav_control = false;
        self.confused = false;
        self.silence = false;
        self.dazed = false;
        self.weak_stacks = 0;
        self.melee_damage = 1.0;
        self.ranged_damage = 1.0;
        self.magic_damage = 1.0;
        self.minion_damage = 1.0;
        // `ResetEffects`' Mana Sickness pair (`Player.cs:18592-18593`):
        // `manaSick = false` (the buff-94 arm below re-arms per tick);
        // `manaHeat = 1f` (no use-path writer exists — the star-pickup lane
        // is unported, `potions.rs`'s tail note).
        self.mana_sick = false;
        self.mana_heat = 1.0;
        // `:18338`/`:18396-18399` — the ranged effective-damage components.
        self.ranged_mult_damage = 1.0;
        self.arrow_damage = 1.0;
        self.arrow_damage_additive_stack = 0.0;
        self.bullet_damage = 1.0;
        self.rocket_damage = 1.0;
        // `ResetEffects`' summon budget bases (`Player.cs:18448-18449`): the
        // cap buffs 110/150/348 (`:10175-10183`, the summon arms pass) and
        // the armor/held rows (`:13258-13948`/`:15324-15329`,
        // `grant_summon_caps`) re-add per tick.
        self.max_minions = 1;
        self.max_turrets = 1;
        // `ResetEffects`' whip rows (`Player.cs:18498-18499` whipRange/
        // whipUseTime = 1f, `:18341` summonerWeaponSpeedBonus = 0f,
        // `:18400-18401` cool/cobWhipBuff = false, `:18517-18518`
        // maxTagEffects = 1 / tagEffectDuration = 1f, `:18519-18522` the four
        // whip accessory flags = false): the buff arms, the armor rows and
        // the accessory match below re-derive them per tick. The Snake Band
        // counters (`repeatWhipSwings`/`repeatWhipsResetCooldown`) are NOT
        // reset here — they are a countdown (`Update_SnakeBand`, `:28412`).
        self.whip_range_multiplier = 1.0;
        self.whip_use_time_multiplier = 1.0;
        self.summoner_weapon_speed_bonus = 0.0;
        self.cool_whip_buff = false;
        self.cob_whip_buff = false;
        self.acc_snake_band = false;
        self.acc_wicked_armlet = false;
        self.acc_mobius_strip = false;
        self.acc_silver_bracer = false;
        self.tag_effects.set_player_caps(1, 1.0);
        // The 28 summon keep-alive flags (`:18620-18646`+`:18741`) — the
        // arms table is the single source of the flag set.
        self.clear_summon_flags();
        // `ResetProjectileCaches` (`:18810`): the recount inputs zeroed;
        // `UpdateProjectileCaches` recounts them at the `:25565` seam (the
        // summon arms pass in `world_sim/player.rs`).
        self.reset_projectile_caches();
        self.mana_regen_bonus = 0;
        // `ResetEffects`' `magicCuffs = false` (`Player.cs:18599`): the cuffs'
        // accessory rows below re-arm it from the live items each tick.
        self.magic_cuffs = false;
        // `ResetEffects`' `manaMagnet = false` / `lifeMagnet = false`
        // (`Player.cs:18491-18492`): the magnet accessories and buff 105
        // re-arm them per tick (S1-P-i52).
        self.mana_magnet = false;
        self.life_magnet = false;
        // `ResetEffects`' retaliate-family clears (S1-C-7 wave B):
        // `strongBees = false` (`Player.cs:18322`), `honeyCombItem = null`
        // (`:18386`), `starCloakItem` + the four overrides = null
        // (`:18714-18718`). The accessory rows below re-arm per tick.
        self.star_cloak = false;
        self.star_veil_override = false;
        self.bee_cloak_override = false;
        self.mana_cloak_override = false;
        self.snake_cloak_override = false;
        self.honey_comb = false;
        self.strong_bees = false;
        self.well_fed = false;
        self.pick_speed_bonus = 0.0;
        self.melee_speed_bonus = 0.0;
        // (Resync keep-one: the summoner/whip attack-speed clears live in
        // the i24/i27 ResetEffects block above — `:18341`/`:18499`.)
        // `ResetEffects`' attack-speed clears (`Player.cs:18531-18532`
        // wallSpeed/tileSpeed = 1f, `:18710` autoReuseGlove = false,
        // `:18431` stressBall = false) — the buff/equipment grants
        // below re-add them per tick, like the melee speed sum.
        self.tile_speed_bonus = 0.0;
        self.wall_speed_bonus = 0.0;
        self.auto_reuse_glove = false;
        self.stress_ball = false;
        // `ResetEffects`' `statDefense = 0` (`Player.cs:18327`): the buff arms
        // below and `GrantArmorBenefits` (`:13121`, the equipment pass at the
        // bottom of this block) re-add per tick. Before M3-EQUIP-5 the field
        // was assigned *outside* the tick (from `world_sim`), which silently
        // dropped every buff and prefix defense arm.
        self.stat_defense = 0;
        // `:18334`: `manaCost = 1f` — the reductions re-apply below.
        self.mana_cost = 1.0;
        // `:18389`: `statManaMax2 = statManaMax` — the effective cap rebases
        // to the permanent one every tick, then the per-tick grants re-add —
        // the Arcane prefix 66's +20 (`GrantPrefixBenefits` `:13971-13974`),
        // the Magic Power buff's +20 (`:11890`) and the caster sets' rows.
        // Without the rebase the grants compound every tick and stick after
        // the item comes off. (ONE rebase — both sides carried it.)
        self.mana_max = self.mana_max_base;
        // `:18720`: `pStone = false` — the Stone family's accessory rows
        // (535/860/6185) re-arm below (`:15576`/`:15152`/`:14617`).
        self.p_stone = false;
        // `:18721`: `manaFlower = false` — the flower accessories re-arm it
        // in the equip pass; the ONE -0.08 row applies after the loop.
        // (`manaSick = false` `:18592` already ran at the damage-reset head
        // above — the i37 form, take-one.)
        self.mana_flower = false;
        // `:18497`: `endurance = 0f` — the rows below re-apply.
        self.endurance = 0.0;
        // S1-P-i27 — `ResetEffects`' summoner-tag resets (`Player.cs:18498`,
        // `:18517-18522`): the whip range base, the tag-stack cap/duration
        // defaults and the three accessory flags re-derive per tick from
        // the equip match below (`apply_equip_functional`) and the
        // `UpdateEquips` fold. The stack's own fields mirror the C#'s
        // `_owner.maxTagEffects`/`tagEffectDuration` reads
        // (`TagEffectStack.cs:47`/`:133` — the stack reads the PLAYER, so
        // the port syncs the copies here).
        self.whip_range_multiplier = 1.0;
        self.acc_silver_bracer = false;
        self.acc_mobius_strip = false;
        self.acc_wicked_armlet = false;
        self.tag_effects.reset_player_scalars(
            crate::tag_effects::MAX_TAG_EFFECTS_DEFAULT,
            crate::tag_effects::TAG_EFFECT_DURATION_DEFAULT,
        );
        // `:18323-18324`: the penetration reset.
        self.armor_penetration = 0;
        self.melee_armor_penetration = 0;
        // `ResetEffects`' sentry-backpack clears (`Player.cs:18481`,
        // `:18740`, `:18746-18752`): the accessory flag and the six DD2
        // armor-set flags re-derive below (the accessory match and the
        // armor-set block); `ballistaPanic` re-derives in the buff loop.
        // The grab cooldown is NOT cleared here — it is a countdown
        // (`Update_SentryBackpack`, `:28428-28434`).
        self.acc_sentry_backpack = false;
        self.ballista_panic = false;
        // S1-P-i35 — `ResetEffects`' fishing clears (`Player.cs:18507-18513`):
        // the skill and the six accessory/potion flags re-derive in the buff
        // loop (121/122/123) and the equip grants below. The bobber override
        // resets to -1 (`:18410`); `usedGummyWorm` is permanent (NOT cleared).
        self.fishing_skill = 0;
        self.crate_potion = false;
        self.sonar_potion = false;
        self.acc_tackle_box = false;
        self.acc_fishing_bobber = false;
        self.acc_fishing_line = false;
        self.acc_lava_fishing = false;
        self.override_fishing_bobber = -1;
        // (Gate merge i35 ⨉ i41: the `canFloatInWater`/`noItems` clears run
        // ONCE, in the i41 mount-flag block above — `Player.cs:18332`/`:18687`.)
        self.set_squire_t2 = false;
        self.set_squire_t3 = false;
        self.set_huntress_t2 = false;
        self.set_huntress_t3 = false;
        self.set_monk_t2 = false;
        self.set_monk_t3 = false;
        self.set_apprentice_t2 = false;
        self.set_apprentice_t3 = false;
        // S1-P-i27 — `ResetEffects`' `setStardust = false` (`Player.cs:18743`)
        // and `setForbidden = false` (`:18745`): the armor-set block above
        // re-derives both per tick.
        self.set_stardust = false;
        self.set_forbidden = false;
        // `:18663`: `lavaMax = 0` — the charm rows below re-add it.
        self.lava_max = 0;
        // `:18659`: `dashType = 0` - the armor-set rows and the accessory
        // granters (Tabi/EoC Shield/Master Ninja Gear) below re-arm it per
        // tick (ResetEffects runs before UpdateEquips, `:25305` < `:25633`).
        self.dash_type = 0;
        // `:18376` + `:18726-18727`: the thorns flag clear.
        self.thorns = 0.0;
        self.cactus_thorns = false;
        self.turtle_thorns = false;
        // The dodge-family clears (`ResetEffects`): `shadowDodge = false`
        // (`:18440`), `brainOfConfusionItem = null` (`:18617`),
        // `blackBelt = false` / `mysticSashDodge = false` (`:18661-18662`),
        // `longInvince = false` (`:18719`) — the fold's equipped/buff arms
        // below re-arm from the live sources. The brain's animation counter
        // decays here like the C# (`:18794-18797`).
        self.shadow_dodge = false;
        self.brain_of_confusion = false;
        self.black_belt = false;
        self.mystic_sash_dodge = false;
        self.long_invince = false;
        if self.brain_of_confusion_dodge_animation_counter > 0 {
            self.brain_of_confusion_dodge_animation_counter -= 1;
        }
        // The luck inputs ResetEffects owns (`Player.cs:18732` stinky,
        // `:18812` equipmentBasedLuckBonus, `:18814` luckPotion): zeroed here,
        // re-derived in the buff scan below (the potion/kite/stinky arms).
        self.luck_state.stinky = false;
        self.luck_state.equipment_based_luck_bonus = 0.0;
        self.luck_state.luck_potion = 0;
        // `ResetEffects`' crit bases (`Player.cs:18640+`): the per-slot adds
        // (the fed tiers, the accessory states) stack on top per tick.
        self.melee_crit = 4;
        self.ranged_crit = 4;
        self.magic_crit = 4;
        // S1-C-12 wave E — the combat family's per-tick clears the wired
        // grants need: `minionKB = 0f` (`Player.cs:18347`) and `kbGlove
        // = false` (`:18709`). Without them the sky-stone bundle's
        // `minionKB += 0.5f` and the gloves' `kbGlove` would accumulate
        // / stick after the item comes off. (`autoReuseGlove`/
        // `meleeScaleGlove` `:18710-18711` have no port fields.)
        self.minion_kb = 0.0;
        self.kb_glove = false;
        // `GrantArmorBenefits` (' armor/accessory rows, `Player.cs:13121`):
        // every equipped wearable's `Item.defense` — the three armor slots AND
        // the seven functional accessory slots (Shackle +1, Cobalt Shield +1,
        // Ankh Shield +4, …), then the completed set's bonus
        // (`ArmorSetBonuses.GetCompleteSet(...).Effect`).
        for &kind in self.equipped[..10].iter() {
            if kind == 0 {
                continue;
            }
            if let Some(def) = crate::registry::get(kind) {
                self.stat_defense += def.defense as i32;
            }
            // The piece's own damage/crit/speed row (`GrantArmorBenefits`'
            // `switch (armorPiece.type)`, `Player.cs:13126-13950`).
            if let Some(piece) = tr_core::armor_sets::armor_stats(kind) {
                self.melee_damage += piece.melee_damage;
                self.ranged_damage += piece.ranged_damage;
                self.magic_damage += piece.magic_damage;
                self.minion_damage += piece.minion_damage;
                self.melee_crit += piece.melee_crit;
                self.ranged_crit += piece.ranged_crit;
                self.magic_crit += piece.magic_crit;
                self.move_speed += piece.move_speed;
                self.melee_speed_bonus += piece.melee_speed;
                // Gate-merge: the piece `whipRangeMultiplier` halves (Tiki Mask
                // 1159 +0.1, Stardust 3382/3383 +0.15, `Player.cs:13854`/`:13879`)
                // ride `grant_summon_caps` beside the minion-cap rows (the i27
                // seam) — NOT here. i24's `armor_stats.whip_range` fold was the
                // duplicate; the set half stays in the `set_stats` block below.
                self.whip_use_time_multiplier *= piece.whip_use_time_mult;
            }
            // S1-P-i35 — `GrantArmorBenefits`' Angler pieces
            // (`Player.cs:13447-13450`): 2367-2369 (the Angler set) and
            // 5591-5593 (the Lava Angler set) each +5 fishing skill.
            if matches!(kind, 2367 | 2368 | 2369 | 5591 | 5592 | 5593) {
                self.fishing_skill += 5;
            }
            // `GrantArmorBenefits`' `switch (armorPiece.type)` rows that touch
            // `manaCost` (`Player.cs:13118-13760`): the Magic Hat and Wizard
            // Hat robes (1282-1287, −0.05…−0.15 by tier; the Amber Robe 4256
            // shares the 1286 row, `:13742-13746`), the Crystal Assassin
            // head (4982, −0.1), the Spectre hood/mask (1003 −0.17, 2189
            // −0.13), the Mystic Robe (2279, −0.1), the Apprentice/Dark
            // Artist rows (3797 −0.1, 3875 −0.15), the Nebula Helmet (2760,
            // −0.15, `:13918-13924`).
            self.mana_cost -= match kind {
                1282 => 0.05,
                1283 => 0.07,
                1284 => 0.09,
                1285 => 0.11,
                1286 | 4256 => 0.13,
                1287 => 0.15,
                2279 | 3797 | 4982 => 0.1,
                2189 => 0.13,
                3875 | 2760 => 0.15,
                1003 => 0.17,
                _ => 0.0,
            };
            // `GrantArmorBenefits`' per-piece `statManaMax2 +=` rows
            // (`Player.cs:13274-13924`): the caster heads and the jungle/
            // ancient-cobalt pieces. The set bonuses' own +60 rows ride
            // `set_stats` below (once per completed set).
            self.mana_max += match kind {
                228 | 960 => 40,   // `:13343-13347` Jungle Hat / Ancient Cobalt Helmet
                229 | 961 => 20,   // `:13348-13352` Jungle Shirt / Ancient Cobalt Breastplate
                230 | 962 => 20,   // `:13353-13357` Jungle Pants / Ancient Cobalt Leggings
                371 => 40,         // `:13389-13394` Cobalt Hat
                376 => 60,         // `:13419-13423` Mythril Hood
                400 => 80,         // `:13451-13456` Adamantite Headgear
                1207 => 60,        // `:13491-13496` Palladium Headgear
                1212 => 80,        // `:13528-13532` Orichalcum Headgear
                1217 => 100,       // `:13558-13563` Titanium Headgear
                558 | 4898 => 100, // `:13585-13590` Hallowed Headgear / Ancient Hallowed Headgear
                1003 => 80,        // `:13651-13656` Spectre Hood
                2189 => 60,        // `:13679-13685` Spectre Mask
                1282 => 20,        // `:13722-13726` Amethyst Robe
                1283 => 40,        // `:13727-13731` Topaz Robe
                1284 => 40,        // `:13732-13736` Sapphire Robe
                1285 => 60,        // `:13737-13741` Emerald Robe
                1286 | 4256 => 60, // `:13742-13746` Ruby Robe / Amber Robe
                1287 => 80,        // `:13747-13751` Diamond Robe
                2760 => 60,        // `:13918-13924` Nebula Helmet
                3777 | 3778 => 40, // `:13274-13285` Ancient Battle Armor Shirt/Pants
                _ => 0,
            };
        }
        // `:12988-13003`: the Celestial Stone bundle — S1-C-12 wave E
        // dedupe: both halves now ride `accessory_combat` (the former
        // inline copies are deleted; ONE grant path per row). The flag
        // is set by the stones (`:15412-15423`): 1865 (Celestial Stone)
        // and 3110 (Celestial Shell) always, 899 (Sun Stone) by day,
        // 900 (Moon Stone) by night or during an eclipse. The fold adds
        // the bundle's stats incl. `minionKB += 0.5f` (`:13002` — the
        // inline copy had missed it); its `lifeRegen += 2` half rides
        // the `accessory_life_regen` fold after `update_equips`.
        self.sky_stone_effects = crate::accessory_combat::set_sky_stone_flag(
            &self.equipped[..10],
            world.day_time,
            world.eclipse,
        );
        if self.sky_stone_effects {
            crate::accessory_combat::apply_sky_stone_effects(self);
        }
        // The immunity rows: the worn accessories (`BUFF_IMMUNITY_ROWS`) and
        // the completed Snow set (`set_buff_immunity` — the Molten row is
        // gated and rides the wave-D `armor_sets.rs` seam below).
        for (kind, ids) in BUFF_IMMUNITY_ROWS {
            if self.equipped[..10].contains(kind) {
                for id in *ids {
                    if let Some(slot) = self.buffs.buff_immune.get_mut(*id as usize) {
                        *slot = true;
                    }
                }
            }
        }
        let (armor_head, armor_body, armor_legs) = (
            self.equipped[0] as i32,
            self.equipped[1] as i32,
            self.equipped[2] as i32,
        );
        self.stat_defense +=
            tr_core::armor_sets::set_bonus_defense(armor_head, armor_body, armor_legs);
        for id in tr_core::armor_sets::set_buff_immunity(armor_head, armor_body, armor_legs) {
            if let Some(slot) = self.buffs.buff_immune.get_mut(*id as usize) {
                *slot = true;
            }
        }
        // The DD2 sentry armor sets' flags (`ArmorSetBonuses.Benefits`,
        // `DataStructures/ArmorSetBonuses.cs:68-118`, the registration triples
        // `:484-491`): the Squire/Huntress/Monk T2/T3 rows each set their tier
        // flag and raise `maxTurrets` by 1; the T3 rows also set the T2 flag
        // (`:95`/`:102`/`:109`/`:116`). The Apprentice rows (`:74-78`/`:99-104`)
        // feed the Flame Burst Tower's aim cone/range/speed buffs
        // (`Projectile.cs:68275-68280`, S1-D-4b-9); the aiStyle-140/142 towers
        // stay unported. `ResetEffects` clears the flags and the `max_turrets`
        // base (`:18449`) each tick, so this re-derives per tick.
        match (armor_head, armor_body, armor_legs) {
            (3800, 3801, 3802) => {
                // `:68-72` Benefits.SquireTier2.
                self.set_squire_t2 = true;
                self.max_turrets += 1;
            }
            (3871, 3872, 3873) => {
                // `:92-97` Benefits.SquireTier3.
                self.set_squire_t3 = true;
                self.set_squire_t2 = true;
                self.max_turrets += 1;
            }
            (3803, 3804, 3805) => {
                // `:80-84` Benefits.HuntressTier2.
                self.set_huntress_t2 = true;
                self.max_turrets += 1;
            }
            (3877, 3878, 3879) => {
                // `:106-111` Benefits.HuntressTier3.
                self.set_huntress_t3 = true;
                self.set_huntress_t2 = true;
                self.max_turrets += 1;
            }
            (3806, 3807, 3808) => {
                // `:86-90` Benefits.MonkTier2.
                self.set_monk_t2 = true;
                self.max_turrets += 1;
            }
            (3880, 3881, 3882) => {
                // `:113-118` Benefits.MonkTier3.
                self.set_monk_t3 = true;
                self.set_monk_t2 = true;
                self.max_turrets += 1;
            }
            (3797, 3798, 3799) => {
                // `:74-78` Benefits.ApprenticeTier2 (the triple `:485`).
                self.set_apprentice_t2 = true;
                self.max_turrets += 1;
            }
            (3874, 3875, 3876) => {
                // `:99-104` Benefits.ApprenticeTier3 (the triple `:489`).
                self.set_apprentice_t3 = true;
                self.set_apprentice_t2 = true;
                self.max_turrets += 1;
            }
            // S1-P-i27 — the minion-rest-target set bonuses. The Forbidden
            // triple (`ArmorSetBonuses.cs:483`, `Benefits.Forbidden` `:61-66`)
            // arms `setForbidden` — the double-tap-down rest aim + storm
            // command gate (`KeyDoubleTap` `Player.cs:56779-56786`). The
            // `UpdateForbiddenSetLock`/`Lighting.AddLight` halves are the
            // storm command's and the light layer's (deferred, ANALYSIS).
            (3776, 3777, 3778) => {
                self.set_forbidden = true;
            }
            // The Stardust triple (`:482`, `Benefits.Stardust` `:56-59` →
            // `ApplySetBonus_Stardust` `Player.cs:16339-16356`): `setStardust`
            // gates the down-hold rest-target clear (`KeyHoldDown`
            // `:56903-56906`). The buff-187 grant + the 623 guardian spawn
            // are the guardian's own lane (deferred, ANALYSIS).
            (3381, 3382, 3383) => {
                self.set_stardust = true;
            }
            _ => {}
        }
        // The completed set's *stat* row (`ArmorSetBonuses.Benefits` — the
        // `ArmorSetBonuses.Initialize` triples, `:396-500`): the damage/crit/
        // speed/mana/minion scalars of Necro, Pumpkin, Ninja, Wizard, Magic Hat,
        // Molten, Frost, Mining, the Cobalt/Mythril/Adamantite head variants,
        // Crystal Assassin, Tiki, Spooky, Spider and Bee.
        if let Some((_set, set)) =
            tr_core::armor_sets::set_stats(armor_head, armor_body, armor_legs)
        {
            self.melee_damage += set.melee_damage;
            self.ranged_damage += set.ranged_damage;
            self.magic_damage += set.magic_damage;
            self.minion_damage += set.minion_damage;
            self.melee_crit += set.melee_crit;
            self.ranged_crit += set.ranged_crit;
            self.magic_crit += set.magic_crit;
            self.move_speed += set.move_speed;
            self.melee_speed_bonus += set.melee_speed;
            self.pick_speed_bonus += set.pick_speed;
            self.mana_max += set.mana_max;
            self.max_minions += set.max_minions;
            self.mana_cost += set.mana_cost;
            // S1-P-i27 — the Tiki set's `whipRangeMultiplier += 0.2f`
            // (`ArmorSetBonuses.cs:15`); the whip sweep now reads the field.
            // Gate-merge: ONE application (i24 and i27 both added the line).
            self.whip_range_multiplier += set.whip_range;
            self.endurance += set.endurance;
            self.thorns += set.thorns;
            self.cactus_thorns |= set.cactus_thorns;
            self.turtle_thorns |= set.turtle_thorns;
            self.whip_use_time_multiplier *= set.whip_use_time_mult;
            if set.dash_type != 0 {
                self.dash_type = set.dash_type;
            }
        }
        for bt in self.buffs.active_types() {
            match bt {
                // S1-E-7 wave A — the potion-buff block, in the C# chain's
                // order (`Player.cs:9875-10048`).
                // `:9875-9880` — ObsidianSkin (1): lava immunity, hot-tile
                // walking and OnFire(24) immunity.
                1 => {
                    self.lava_immune = true;
                    self.fire_walk = true;
                    self.buffs.buff_immune[24] = true;
                }
                // S1-P-i41 — `:11731-11735` — Cursed (23): `noItems = true;
                // cursed = true`. The item-use lock (`ItemCheck_CheckCanUse_Inner`
                // `:52145-52148`) and the transformation-mount exemption read
                // the pair. (The debuff block's own arm — the flag grant is
                // the same write either way.)
                23 => {
                    self.no_items = true;
                    self.cursed = true;
                }
                // `:9952-9955` — Swiftness (3): `moveSpeed += 0.25f`.
                3 => self.move_speed += 0.25,
                // S1-P-i29 — `:10134-10137`: Mining (104) `pickSpeed -= 0.25f`
                // (the port's positive bonus convention: a lower pickSpeed
                // mines faster).
                104 => self.pick_speed_bonus += 0.25,
                // `:9956-9959` — Gills (4): `CheckDrowning` inverts the probe.
                4 => self.gills = true,
                // `:9964-9967` — ManaRegeneration (6): `manaRegenBuff = true`.
                6 => self.mana_regen_buff = true,
                // `:9968-9976` — MagicPower (7): `magicDamage += 0.2f` (the
                // `DebugOptions.ManaV2` branch assigns the same 0.2).
                7 => self.magic_damage += 0.2,
                // `:9977-9980` — Featherfall (8): the fall caps/gravity arm.
                8 => self.slow_fall = true,
                // `:9981-9984` — Spelunker (9): treasure highlighting (draw).
                9 => self.find_treasure = true,
                // `:9985-9988` — BiomeSight (343): ore highlighting (draw).
                343 => self.biome_sight = true,
                // `:9989-9992` — Invisibility (10): the spawner rate modifier.
                10 => self.invis = true,
                // `:9993-9996` — Shine (11): the AddLight half rides the
                // frame's entity-light feed (`entity_lights::shine_light`);
                // no flag exists in the C# either.
                // `:9997-10000` — NightOwl (12): the legacy brightness lift.
                12 => self.night_vision = true,
                // `:10001-10004` — Battle (13): the spawner rate modifier.
                13 => self.enemy_spawns = true,
                // `:10012-10015` — WaterWalking (15): the surface walk
                // (collision half unported — flag only).
                15 => self.water_walk = true,
                // `:10021-10024` — Hunter (17): enemy highlighting (draw).
                17 => self.detect_creature = true,
                // `:10025-10028` — Gravitation (18): the gravity-flip key
                // (unported — flag only).
                18 => self.grav_control = true,
                // `:10033-10036` — Confused (31): the control swap below.
                31 => self.confused = true,
                // `:10041-10044` — Silenced (35): mana items refuse.
                35 => self.silence = true,
                // `:11719-11722` — Potion Sickness (21): the debuff timer IS
                // the cooldown — `potionDelay = buffTime[j]` every scan tick
                // (the drink gates read it, S1-P-i37). The assignment (not a
                // max) mirrors the source: the last live slot wins.
                21 => {
                    if let Some(i) = self.buffs.find_buff_index(21) {
                        self.potion_delay = self.buffs.buff_time[i];
                    }
                }
                // `:10318-10327` — Mana Sickness (94): the mana-potion magic
                // penalty (`manaSick = true`, `manaSickReduction = 0.25 *
                // buffTime / manaSickTime`). The `DebugOptions.ManaPotionDelay`
                // mirror (`:10320-10323`) is constants-off.
                94 => {
                    if let Some(i) = self.buffs.find_buff_index(94) {
                        let t = self.buffs.buff_time[i];
                        crate::potions::apply_mana_sickness_arm(self, t);
                    }
                }
                // `:10045-10048` — Dazed (160): the jump-static clamp.
                160 => self.dazed = true,
                22 => self.blind = true, // `BuffID.Darkness`
                32 => self.slow = true,  // `BuffID.Slow`
                // `Player.UpdateBuffs`' shadow-dodge arm (`Player.cs:10075-
                // 10077`): the Hallowed set's buff 59 arms `shadowDodge` for
                // the tick; the buff is granted by the set bonus's
                // `onHitDodge` scan, which is an armor-set row (out of scope).
                59 => self.shadow_dodge = true,
                // `:10138-10141` — Heartreach (105): `lifeMagnet = true`
                // (S1-P-i52 — the heart grab-range bonus + the 15/5 pull).
                105 => self.life_magnet = true,
                // `:10201-10204` — `BuffID.Endurance` (114): +10%.
                114 => self.endurance += 0.1,
                // `:10015-10017` — `BuffID.Sharpened` (159): the melee-only
                // +12 armor penetration.
                159 => self.melee_armor_penetration += 12,
                // `:10461-10470` — the Ice Barrier (62, the Frozen Turtle
                // Shell): +25% while at or below half life.
                62 => {
                    if self.hp as f32 <= self.max_hp as f32 * 0.5 {
                        self.endurance += 0.25;
                    }
                }
                // `:11887-11892` — `BuffID.MagicPower` (29): the crit, the
                // damage, the 20 mana and the −0.02 cost.
                29 => {
                    self.magic_crit += 2;
                    self.magic_damage += 0.05;
                    self.mana_max += 20;
                    self.mana_cost -= 0.02;
                }
                33 => {
                    self.weak_stacks += 1; // `BuffID.Weak`
                    self.melee_damage -= 0.051;
                    self.move_speed -= 0.1;
                    // `:11915`: `statDefense -= 4` per Weak slot.
                    self.stat_defense -= 4;
                }
                // `:9962` — `BuffID.Ironskin`: `statDefense += 8`.
                5 => self.stat_defense += 8,
                // `:10064` — `BuffID.Ichor`: `statDefense -= 15`.
                69 => self.stat_defense -= 15,
                // `:10108` — buff 215 (`BuffID.Warmth`'s neighbour row in the
                // C#: `statDefense += 5`).
                215 => self.stat_defense += 5,
                // `:10110-10121` — the whip-speed buffs: Dark Harvest (311)
                // +0.35, Sword Whip (308) +0.25, Thorn Whip (314) +0.12, per
                // slot. `CapAttackSpeeds` (`:29025`) folds `meleeSpeed + bonus`
                // into the use-time multiplier the whip swing reads.
                311 => self.summoner_weapon_speed_bonus += 0.35,
                308 => self.summoner_weapon_speed_bonus += 0.25,
                314 => self.summoner_weapon_speed_bonus += 0.12,
                // `:10122-10129` — the Cool/Cob Whip buff flags: the on-hit
                // latch (`Projectile.cs:13918`/`:13929`) reads them to spawn
                // one snowflake/spark; the buff re-affirms the flag per tick.
                312 => self.cool_whip_buff = true,
                365 => self.cob_whip_buff = true,
                // `:10306-10312` — `BuffID.DryadsWard`: `statDefense += 8`,
                // `lifeRegen += 6` (the regen half has no consumer yet) and
                // `thorns += 0.5` while under 1.
                165 => {
                    self.stat_defense += 8;
                    if self.thorns < 1.0 {
                        self.thorns += 0.5;
                    }
                }
                // `:10005-10011` — `BuffID.Thorns` (14): `thorns = 1f` if lower.
                14 => {
                    if self.thorns < 1.0 {
                        self.thorns = 1.0;
                    }
                }
                // `:11921` — `BuffID.Tipsy`: `statDefense -= 4`.
                25 => self.stat_defense -= 4,
                // The whip attack-speed buffs (`Player.cs:10110-10121`):
                // 311 +0.35, 308 +0.25, 314 +0.12 onto
                // `summonerWeaponSpeedBonus` — `ApplyItemAnimation`'s
                // scaling-whip branch reads the capped inverse (`:4476`).
                311 => self.summoner_weapon_speed_bonus += 0.35,
                308 => self.summoner_weapon_speed_bonus += 0.25,
                314 => self.summoner_weapon_speed_bonus += 0.12,
                // `:10158-10163` — the Builder Potion (107): tileSpeed /
                // wallSpeed += 0.25 (the blockRange half rides the reach).
                107 => {
                    self.tile_speed_bonus += 0.25;
                    self.wall_speed_bonus += 0.25;
                }

                // S1-P-i35 — `UpdateBuffs`' fishing potions
                // (`Player.cs:10146-10157`): Fishing (121) +15 skill,
                // Sonar (122) and Crate (123) flags.
                121 => self.fishing_skill += 15,
                122 => self.sonar_potion = true,
                123 => self.crate_potion = true,
                // (Gate merge i35 ⨉ i41: the Merman 265 and Cursed 23 arms
                // live ONCE in this loop — the i41 arms above/below carry
                // `accFlipper`/`cursed` too; the i35 duplicates folded in.)
                46 => self.chilled = true, // `BuffID.Chilled`
                47 => self.frozen = true,  // `BuffID.Frozen`
                // `:11001-11004`: the Sunflower buff's movement tier —
                // `moveSpeed += 0.1f; moveSpeed *= 1.1f` per slot.
                146 => {
                    self.move_speed += 0.1;
                    self.move_speed *= 1.1;
                }
                // `:9932-9934`: the Star in a Bottle's mana-regen add.
                158 => self.mana_regen_bonus += 10,
                // (Gate resync take-one: the Mana Sickness 94 arm lives ONCE
                // above through `potions::apply_mana_sickness_arm` — the i37
                // canonical fn; the i19 inline copy was identical maths
                // (`0.25 * buffTime / 300`) and dropped, an unreachable arm.)
                // The WellFed tiers (`:11931-11969`): the per-slot stat adds —
                // the defense rows (26 +2, 206 +3, 207 +4) since M3-EQUIP-5;
                // the minion halves wait for M4-MINION.
                26 => {
                    self.well_fed = true;
                    // `:11934`: `statDefense += 2`.
                    self.stat_defense += 2;
                    self.melee_crit += 2;
                    self.melee_damage += 0.05;
                    self.melee_speed_bonus += 0.05;
                    self.ranged_crit += 2;
                    self.ranged_damage += 0.05;
                    self.magic_crit += 2;
                    self.magic_damage += 0.05;
                    self.move_speed += 0.2;
                    self.pick_speed_bonus += 0.05;
                }
                206 => {
                    self.well_fed = true;
                    // `:11950`: `statDefense += 3`.
                    self.stat_defense += 3;
                    self.melee_crit += 3;
                    self.melee_damage += 0.075;
                    self.melee_speed_bonus += 0.075;
                    self.ranged_crit += 3;
                    self.ranged_damage += 0.075;
                    self.magic_crit += 3;
                    self.magic_damage += 0.075;
                    self.move_speed += 0.3;
                    self.pick_speed_bonus += 0.1;
                }
                207 => {
                    self.well_fed = true;
                    // `:11966`: `statDefense += 4`.
                    self.stat_defense += 4;
                    self.melee_crit += 4;
                    self.melee_damage += 0.1;
                    self.melee_speed_bonus += 0.1;
                    self.ranged_crit += 4;
                    self.ranged_damage += 0.1;
                    self.magic_crit += 4;
                    self.magic_damage += 0.1;
                    self.move_speed += 0.4;
                    self.pick_speed_bonus += 0.15;
                }
                // `BuffID.Webbed` (`Player.cs:11865-11881`): the webbed arm also
                // calls `RemoveAllGrapplingHooks` (`:11880`) — the web snaps the
                // hook. The port raises the same removal request the jump-off
                // uses (`release_grappling_hooks`), drained by the world-sim.
                149 => {
                    self.webbed = true;
                    self.release_grappling_hooks = true;
                }
                // `:11843-11846` — buff 205 (Ballista Panic): the Ballista's
                // reload caps (`Projectile.cs:68563-68570`).
                205 => self.ballista_panic = true,
                // `:10276-10280`: buff 120 (Stinky) — the `talkNPC = -1`
                // dialogue half is not ported; the luck −0.25 flag is.
                120 => self.luck_state.stinky = true,
                // `:10285-10301`: buff 257 (Lucky) — the potion tier by
                // *remaining* time; `RecalculateLuck` weights it
                // `(int)luckPotion · 0.1` (max +0.3).
                257 => {
                    if let Some(i) = self.buffs.find_buff_index(257) {
                        let t = self.buffs.buff_time[i];
                        self.luck_state.luck_potion = if t > 36_000 {
                            3
                        } else if t > 18_000 {
                            2
                        } else {
                            1
                        };
                    }
                }
                // `:9920-9924`: buff 383 (Kite) — the tier is
                // `buffTime / 10800` clamped 0..3, assigned only while the
                // buff is active: the C# never zeroes the field, so it stays
                // stale after expiry (no `kiteLuckLevel = 0` exists).
                383 => {
                    if let Some(i) = self.buffs.find_buff_index(383) {
                        self.luck_state.kite_luck_level =
                            (self.buffs.buff_time[i] / 10_800).clamp(0, 3) as u8;
                    }
                }
                // S1-P-i41 — `UpdateBuffs`' mount sub-arms (`Player.cs:9885-
                // 9912`), in the main buff loop so they run AFTER the
                // `ResetEffects` clears (`:25491` ResetEffects precedes
                // `:25566` UpdateBuffs). The mount re-affirm + buffTime hold
                // stays at the tick head (the `:9881-9884` loop); only the
                // swim-flag grants move here (gate V3 ordering fix).
                // `:9885-9889` (131), `:9890-9894` (168), `:9900-9904` (279).
                131 | 168 | 279 => {
                    self.ignore_water = true;
                    self.acc_flipper = true;
                }
                // `:9895-9899` (265, the Water Strider).
                265 => {
                    self.can_float_in_water = true;
                    self.acc_flipper = true;
                }
                // `:9905-9912` (305, the Lava Shark): the full lava suite.
                305 => {
                    self.ignore_water = true;
                    self.acc_flipper = true;
                    self.lava_immune = true;
                    self.lava_vision = true;
                    self.fire_walk = true;
                }
                _ => {}
            }
        }
        if self.slow {
            self.strongest_move_speed_debuff = self.strongest_move_speed_debuff.min(0.5);
        }
        if self.chilled {
            self.strongest_move_speed_debuff = self.strongest_move_speed_debuff.min(0.75);
        }
        if self.webbed {
            // `:11866-11872`: velocity zeroed (airborne keeps a 1e-6 fall),
            // jump height 0, gravity 0 — the player hangs in the web.
            self.vx = 0.0;
            self.vy = if self.vy != 0.0 { 1e-6 } else { 0.0 };
        }
        // `ResetEffects`' equipment-flag clears (`Player.cs:18348-18788`) and
        // the accessory grant pass - `UpdateEquips` -> `ApplyEquipFunctional`
        // (`Player.cs:12892`/`:14368`, the slots 3..10 loop :12946-12952):
        // the flags reset, then the equipped kinds re-set them per tick.
        // `UpdateDyes` sits between the two in `Player.Update`
        // (`:25492` before `UpdateEquips`' `:25633`), so the dye cache
        // refills here, against this tick's slots.
        self.update_dye_cache();
        self.jump_boost = false;
        self.no_fall_dmg = false;
        // `:18504` goldRing, `:18760` hasLuckyCoin — the Gold/Lucky Coin
        // family's per-tick clears (the equip pass below re-arms them).
        self.gold_ring = false;
        self.has_lucky_coin = false;
        self.ice_skate = false;
        self.desert_boots = false;
        self.rocket_boots = 0;
        // W4-c14 doc-sweep - `:18354`: `vanityRocketBoots = 0` clears with
        // the gameplay tier; the equip rows + the vanity pass re-arm it.
        self.vanity_rocket_boots = 0;
        self.no_knockback = false;
        // `:18777`: `hasPaladinShield = false` — the wave-C pass
        // (`accessory_defense.rs`) re-grants per tick. `fireWalk`'s `:18355`
        // clear lives in the ResetEffects block above (before the buff
        // arms), matching the C#'s ResetEffects -> UpdateBuffs ->
        // UpdateEquips order: clearing it here would wipe the ObsidianSkin
        // buff's grant (S1-E-7 wave A).
        self.has_paladin_shield = false;
        self.has_jump_options = [false; 9];
        // `:18701-18703`: `wingsLogic = 0; hasWings = false; wingTimeMax = 0`
        // — the granting wing re-sets all three below, so taking the wings off
        // ends the flight (and its budget) on the next tick. `:18455`:
        // `empressBrooch = false` clears with the rest of ResetEffects.
        self.wings_logic = 0;
        self.has_wings = false;
        self.wing_time_max = 0;
        self.empress_brooch = false;
        // `:25598`: `accMerman = false` clears after the derivation read it;
        // the shell grants below re-arm it. `merman` itself is re-derived
        // per tick (`:25577-25589`), not reset here.
        self.acc_merman = false;
        self.accessory_life_regen = 0;
        // `UpdateEquips`' second loop (`Player.cs:12946-12952`): the functional
        // accessory pass over the slots 3..10, gated per slot - the seam below.
        self.update_equips();
        // `UpdateEquips`' whip-accessory tail (`Player.cs:12970-12984`): the
        // Silver Bracer adds `tagEffectDuration += 2f` and `maxTagEffects++`
        // (`minionDamage += 0f` is a no-op), the Mobius Strip adds
        // `maxTagEffects++` and `whipRangeMultiplier += 0.1f`, the Wicked
        // Armlet adds `maxTagEffects++`. The base cap/duration reset to 1/1f
        // in `ResetEffects` (`:18517-18518`); the stack reads them per tick.
        // (Gate i24/i27 deduped form — ONE application, V2's fix.)
        let mut max_tag_effects = 1usize;
        let mut tag_effect_duration = 1.0f32;
        if self.acc_silver_bracer {
            tag_effect_duration += 2.0;
            max_tag_effects += 1;
        }
        if self.acc_mobius_strip {
            max_tag_effects += 1;
            self.whip_range_multiplier += 0.1;
        }
        if self.acc_wicked_armlet {
            max_tag_effects += 1;
        }
        self.tag_effects.set_player_caps(max_tag_effects, tag_effect_duration);
        // The placement-speed accessories (`Player.cs:15263-15292` set the
        // flags — 2214 Brick Layer → tile, 2217 Portable Cement Mixer → wall,
        // 3061 Architect Gizmo Pack and 5126 Hand Of Creation → both;
        // `:13024-13039` folds them): each grant adds +0.5 to its speed — the
        // Brick Layer's tile half skips torch placements (`TileID.Sets.Torches`,
        // `:13027`). The flags ride `ApplyEquipFunctional`, which runs the
        // accessory slots 3..10 only (`:12946-12952`) — armor-slot copies
        // never grant.
        let held_tile = crate::registry::place_tile(self.held_item_kind);
        let tile_acc = self.equipped[3..10].iter().any(|k| matches!(k, 2214 | 3061 | 5126));
        let wall_acc = self.equipped[3..10].iter().any(|k| matches!(k, 2217 | 3061 | 5126));
        if tile_acc && held_tile.is_some_and(|t| !td::SET_TORCHES.has(t)) {
            self.tile_speed_bonus += 0.5;
        }
        if wall_acc {
            self.wall_speed_bonus += 0.5;
        }
        // (Resync keep-one: the Obsidian Outlaw ×(1/1.15) triple-match that
        // sat here was the i12 stand-in from before the set joined
        // `armor_sets::SETS` — the set-stats fold above now carries
        // `whip_use_time_mult` (`ArmorSetBonuses.cs:124`), so the direct
        // match DOUBLE-applied the scale. Dropped.)
        // `Player.cs:25640-25643`: a pStone CHANGE across `UpdateEquips`
        // rescales the live Potion Sickness timer (`AdjustRemainingPotionSickness`,
        // `:28939-28952`) — ×0.75 equipping the Stone family, ÷0.75 removing it.
        if self.p_stone_prev != self.p_stone {
            crate::potions::adjust_remaining_potion_sickness(self);
        }

        // Gate merge (i40 ⨉ i41): the `Mount.UpdateAfterEquips` type-54 arm
        // (permission latch + Soaring Insignia refill, `Mount.cs:2989-2997`)
        // runs ONCE, inside `update_equips` above (`update_selective_flight`,
        // S1-P-i41) — the i40 duplicate refill call is folded into it.
        // S1-C-12 wave C — the defense/immunity accessory rows kept in
        // their own module (the Obsidian/Paladin-family shields' knockback
        // + fire-walk grants, `Player.cs:14605-14631`/`:15209-15213`/
        // `:15348-15353`/`:15552-15554` + the fireWalk granters). Boolean
        // OR grants with no RNG draw — a second pass over the same slots
        // is order-identical to the C#'s interleaved rows.
        crate::accessory_defense::apply_defense_accessories(self);
        // `UpdateEquips`' post-loop `manaFlower` fold (`Player.cs:12962-
        // 12965`): ONE `manaCost -= 0.08f` for the flag the four flower
        // accessories armed (555/3991/4000/4001) — a bool, so stacking two
        // flowers still costs only 0.08.
        if self.mana_flower {
            self.mana_cost -= 0.08;
        }
        // Gate-merge note (i24 vs i27): the summoner-tag fold
        // (`Player.cs:12970-12984`, the Silver Bracer/Mobius Strip/Wicked
        // Armlet adds) lives ONCE above — i24's local-fold +
        // `set_player_caps` form (the same three C# `if`s, same order,
        // same values). i27's `add_player_scalars` twin was dropped at the
        // gate (it would double the adds); the seam stays available.
        // S1-P-i35 — `UpdatePermanentBoosters`' Gummy Worm row
        // (`Player.cs:12857-12860`): the permanent +3 fishing skill.
        if self.used_gummy_worm {
            self.fishing_skill += 3;
        }
        // S1-P-i35 — `UpdateEquips`' post-loop bobber row
        // (`Player.cs:12985-12988`): the glow-bobber accessory's +10 skill,
        // AFTER the functional pass set the flag.
        if self.acc_fishing_bobber {
            self.fishing_skill += 10;
        }
        // S1-C-12 wave D — `Player.UpdateArmorSets` (`Player.cs:25685` →
        // `:16304-16306`: `ArmorSetBonuses.GetCompleteSet(...)?.Effect`):
        // the completed LOW-tier set's FLAG halves (shadow/crimson/meteor/
        // molten/frost/ash-wood/gladiator). The stat halves already applied
        // above (`set_stats`/`set_bonus_defense`/`set_buff_immunity`) are
        // NOT re-applied — the seam's rows are disjoint. The ammo-cost
        // flags ride `grant_set_ammo_flags` in the ammo-mastery seam (its
        // per-tick clear runs after this update). The C# call sits after
        // `UpdateEquips` (`:25633`) and the equipment-flag clears
        // (`ResetEffects` `:25491`), matching this position.
        crate::armor_sets::apply_armor_set_bonuses(self);
        // `Player.cs:25900-25911` — the Mana Sickness damage cut: after every
        // `magicDamage` grant (the buff scan, the equip/set rows), the recalc
        // multiplies `magicDamage *= 1 - manaSickReduction` then `*= manaHeat`
        // (the i37 canonical tail fn — the i19 inline cut dropped at the
        // resync take-one; the heat multiply stays ×1 on the drink path).
        crate::potions::apply_magic_damage_sickness_tail(self);
        // `Player.CapAttackSpeeds` (`Player.cs:25976`): the i12 canonical
        // read-only form folds the cap/invert on demand —
        // `melee_speed(held_kind)` / `summoner_weapon_speed_multiplier`
        // read the live grants, so no mutating pass runs here (the gate's
        // `cap_attack_speeds` accumulator rewrite was the superseded form).
        // `UpdateJumpHeight` (`Player.cs:19618`): the flat jump-speed add
        // rides after the grants (the Amphibian Boots' +1.6); a non-additive
        // mount still overwrites the pair in `update_mount` (`:19628-19629`).
        // `:19594-19597`: the Soaring Insignia's +1.8 joins the boost here —
        // the C# adds it inside UpdateJumpHeight, which runs AFTER
        // UpdateEquips granted the brooch this tick.
        if self.empress_brooch {
            self.jump_speed_boost += 1.8;
        }
        self.jump_speed += self.jump_speed_boost;
        // `Player.cs:25577-25589`: the merman transformation — a shell
        // (`accMerman`) while wet (not lava, not a slime mount) turns the
        // player merman; the flight gates read it (`CanUseWingAbilities`
        // `:3994`, `WingAirLogicTweaks` `:26496`). The port folds the C#'s
        // `base.wet` (WetCollision, any liquid) + `!lavaWet` into the
        // water/honey/shimmer union. The `releaseJump`/`wings = 0`/`accFlipper`
        // /`AddBuff(34)` siblings are the swim/visual families (out of flight
        // scope); `accMerman` clears at the reset above (`:25598`).
        let merman_wet = (self.wet || self.honey_wet || self.shimmer_wet)
            && !self.lava_wet
            && (!self.mount.is_active() || !self.mount.is_considered_a_slime_mount());
        self.merman = self.acc_merman && merman_wet;
        // `Player.cs:25927-25930`: the defense floor — a pile of debuffs (Ichor
        // −15, Weak −4 a slot, Tipsy −4) may overshoot the armor's total, and
        // the C# never lets `statDefense` go negative (a negative value would
        // *raise* incoming damage through `CalculateDamagePlayersTake`).
        self.stat_defense = self.stat_defense.max(0);
        // `Player.lifeRegen`'s equipment rows (`UpdateLifeRegen`'s input, fed
        // through `accessory_life_regen` -> `extra_regen`): the armor pieces
        // (`:13148` Stardust/Solar +4, `:13192` Spooky +8, `:13886-13900` the
        // Solar Flare pieces +2 each — the C# also adds `Item.lifeRegen`,
        // which the item table does not carry yet), the Regeneration buff
        // (2, +4 at `:9950`) and the Dryad's Ward (165, +6 at `:10305`), and
        // the Celestial Stone bundle's +2 (`:12991`).
        for &kind in self.equipped[..10].iter() {
            // `:13122`: `lifeRegen += armorPiece.lifeRegen` — the item's own
            // row (the Band of Regeneration 2, Charm of Myths 2, Mana
            // Regeneration Band 2, Restoration Shield 2, Mystic Arts Sash 2
            // are the whole stock table).
            if let Some(def) = crate::registry::get(kind) {
                self.accessory_life_regen += def.life_regen;
            }
            self.accessory_life_regen += match kind {
                3800 => 4,
                3872 => 8,
                2763 | 2764 | 2765 => 2,
                _ => 0,
            };
        }
        for bt in self.buffs.active_types() {
            match bt {
                2 => self.accessory_life_regen += 4,
                165 => self.accessory_life_regen += 6,
                _ => {}
            }
        }
        if self.sky_stone_effects {
            self.accessory_life_regen += 2;
        }
        // The debug pins stand in for the granting equipment - re-applied
        // after the clear (the vanilla: the grants re-set per tick).
        for i in 0..9 {
            if self.pinned_jumps[i] {
                self.has_jump_options[i] = true;
            }
        }
        if self.pinned_balloon {
            self.jump_boost = true;
        }
        if self.pinned_rocket_boots > self.rocket_boots {
            self.rocket_boots = self.pinned_rocket_boots;
            // W4-c14 doc-sweep - the pin stands in for the equip row, which
            // sets both tiers equal (`Player.cs:14420` etc.).
            self.vanity_rocket_boots = self.pinned_rocket_boots;
        }
        if self.pinned_dash != 0 {
            self.dash_type = self.pinned_dash;
        }
        // The `TR_MOUNT` pin equips while nothing else did this tick — a real
        // saddle (or its still-live buff) wins, matching the refresh order.
        if let Some(m) = self.pinned_mount {
            if !self.mount.is_active() {
                let wet = self.wet;
                self.set_mount(m, wet);
            }
        }
        // `Player.UpdateBuffs`' scene-buff grants (`Player.cs:25525-25555`):
        // the 2-tick indicator buffs refresh from the scan; the effect cases
        // above read the previous tick's grant, exactly like the source.
        if self.auras.shadow_candle {
            self.buffs.add_buff(350, 2);
        }
        if self.auras.campfire {
            self.buffs.add_buff(87, 2);
        }
        if self.auras.cat_bast {
            self.buffs.add_buff(215, 2);
        }
        if self.auras.star_in_bottle {
            self.buffs.add_buff(158, 2);
        }
        if self.auras.heart_lantern {
            self.buffs.add_buff(89, 2);
        }
        if self.auras.sunflower {
            self.buffs.add_buff(146, 2);
        }
        // `Player.Update` → `UpdateLuck` (`Player.cs:25645` → `:29041`): the
        // decay timers + `RecalculateLuck`, after the buff effects derived
        // stinky/luckPotion/kiteLuckLevel above and before movement.
        self.update_luck(world);
        // `Player.cs:26004-26009`: the run-speed merge runs AFTER `UpdateEquips`
        // (`:25633`) - the accessory/prefix `moveSpeed` adds ride the debuff
        // multiply too. `oldStyleParkour` (the `:26007` gate) is unported
        // (item 6190's info-acc row), so the accRunSpeed debuff always applies.
        self.move_speed *= self.strongest_move_speed_debuff;
        self.run_acceleration *= self.move_speed;
        self.max_run_speed *= self.move_speed;
        self.acc_run_speed *= self.strongest_move_speed_debuff;
        // `Player.cs:26511-26517` — the Shadow Scale set's movement boost
        // (S1-C-12 wave D): acceleration ×1.75, max/acc run speed ×1.15,
        // slowdown ×1.75. The C# gates the whole block on
        // `grappling[0] == -1 && !tongued` (`:26494`); the port's grapple
        // bypass below covers the first half, the tongue flag (E-7c's
        // buff arm) the second. The empressBrooch/Magiluminescence arms
        // (`:26500-26510`) have no port fields and stay out (ANALYSIS).
        if !self.grappling_active && !self.tongued && self.shadow_armor {
            self.run_acceleration *= 1.75;
            self.max_run_speed *= 1.15;
            self.acc_run_speed *= 1.15;
            self.run_slowdown *= 1.75;
        }
        // `Player.cs:26522-26579`: the floor-tile modifiers, one else-if chain.
        // The flags come from the previous tick's `FloorVisuals` sample — one
        // tick stale, like the original's call order. The empress-brooch /
        // Magiluminescence boosts have no port fields (ANALYSIS "S1-C-12
        // wave D"); the shadow-armor boost landed above.
        if self.sticky {
            self.max_run_speed *= 0.25;
            self.run_acceleration *= 0.25;
            self.run_slowdown *= 2.0;
            self.vx = self.vx.clamp(-self.max_run_speed, self.max_run_speed);
        } else if self.powerrun {
            self.max_run_speed *= 3.5;
            self.run_slowdown *= 2.0;
        } else if self.running_on_sand && self.desert_boots {
            let n = 1.75;
            self.max_run_speed *= n;
            self.acc_run_speed *= n;
            self.run_acceleration *= n;
            self.run_slowdown *= n;
        } else if self.slippy2 {
            self.run_acceleration *= 0.6;
            self.run_slowdown = 0.0;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            }
        } else if self.slippy {
            self.run_acceleration *= 0.7;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            } else {
                self.run_slowdown *= 0.1;
            }
        }
        // `Player.cs:25256-25287`: the hover latches — a down press edge
        // latches hover-down, an up edge swaps to hover-up; landing clears
        // both. The vanilla default is `Settings.HoverControl == Hold`
        // (`:407`), which clears the latches EVERY tick (`:25283-25287`) so
        // hover needs the key held; the port models the latch-persist
        // (Toggle-like) behaviour and does not yet read the setting — a
        // documented deviation (ANALYSIS "The wings/flight accessory block").
        // The up-edge arm (`:25267-25277`) needs `controlUp`, which the port
        // merges into `control_jump` (no separate channel).
        if input.down && !self.down_prev {
            if self.try_keeping_hovering_up {
                self.try_keeping_hovering_up = false;
            } else {
                self.try_keeping_hovering_down = true;
            }
        }
        self.down_prev = input.down;
        if self.vy == 0.0 {
            self.try_keeping_hovering_up = false;
            self.try_keeping_hovering_down = false;
        }
        // `Player.cs:27395-27399`: the whole run/jump/wing/dash movement
        // block is bypassed while grappling — `GrappleMovement` replaces the
        // velocity below.
        let grappling = self.grappling_active;
        let dir = input.right as i32 - input.left as i32;
        // `Player.cs:26688` calls `HorizontalMovement()` unconditioned, so its
        // direction block (`:19835-19843`) applies even on the ticks the
        // port's grapple gate bypasses the call — the facing update for those
        // ticks stays here (the pull replaces the velocity afterwards, the
        // facing keeps following, with the cart lock intact).
        if grappling && dir != 0 {
            let roll = (self.vx > 0.0) as i32 - (self.vx < 0.0) as i32;
            if !self.cart_mounted || roll == dir {
                self.facing = dir;
            }
        }
        // `Player.cs:26623-26687`: the mount stat override. The base cart is
        // not additive and cannot use wings (`MovementStatsAreAdditive` false,
        // `CanUseWings` false), so the stats are replaced outright and the
        // wing/rocket gear is stripped for the tick.
        if self.cart_mounted {
            self.run_slowdown = 0.2; // `:26625`
            self.rocket_boots = 0; // `:26628`
            self.vanity_rocket_boots = 0; // `:26630` (W4-c14 doc-sweep)
            self.wings_logic = 0; // `:26631-26633` (`wings = 0` has no port field)
            self.max_run_speed = CART_RUN_SPEED; // `:26652` mount.RunSpeed
            self.acc_run_speed = CART_DASH_SPEED; // `:26653` mount.DashSpeed
            self.run_acceleration = CART_ACCELERATION; // `:26654` mount.Acceleration
            // `Player.cs:19628-19629` + `UpdateJumpHeight`: the cart is in no
            // `JumpHeight`/`JumpSpeed` case (`Mount.cs:2810-2858`) — fixed
            // 15/5.15, no |vx| scaling.
            self.jump_height = CART_JUMP_HEIGHT;
            self.jump_speed = CART_JUMP_SPEED;
            // `Player.cs:26665-26681`: the on-track check while resting — the
            // friction switch and the dismount-on-direction-press edge.
            if self.vy == 0.0 {
                if !minecart::on_track(
                    world,
                    Vec2::new(self.x, self.y),
                    WIDTH as i32,
                    CART_MOUNT_HEIGHT as i32,
                ) {
                    self.full_rotation = 0.0; // `:26668`
                    self.on_wrong_ground = true; // `:26669`
                    self.run_slowdown = 0.2; // `:26670`
                    // `:26672` — `releaseLeft`/`releaseRight` are `!controlX`
                    // recomputed at tick end (`:27624-27635`), so the pair is
                    // a direction-key press edge.
                    if (self.control_left && !self.left_prev)
                        || (self.control_right && !self.right_prev)
                    {
                        self.try_cart_dismount(world);
                    }
                } else {
                    self.run_slowdown = self.run_acceleration; // `:26675` — 0.04
                    self.on_wrong_ground = false; // `:26676`
                }
            }
        }
        if !grappling {
            // `:26496-26499`: the wing run-speed tweaks run before the
            // movement — vanilla gates them on `!merman` and
            // `mount.CanUseWings` too.
            if self.wings_logic > 0
                && self.vy != 0.0
                && !self.merman
                && self.mount.can_use_wings()
            {
                self.wing_air_logic_tweaks();
            }
            // `:26500-26503`: the Soaring Insignia's ×1.75 run acceleration
            // (a sibling of the tweaks, same pre-movement slot — the C# does
            // NOT gate it on airborne).
            if self.empress_brooch {
                self.run_acceleration *= 1.75;
            }
            // `Player.cs:26623-26688`: the per-tick mount block (with the
            // `UpdateEffects` head and the `UpdateJumpHeight` merge — vanilla
            // runs those at `:25656`/`:26011`, both earlier in the same tick).
            // A no-op without a mount; the cart ride's own stat block (the
            // same C# lines, `cart_mounted`-gated) runs where Phase 2 placed
            // it — the two never fire together (no cart row in the mount
            // table; M2-MOUNT-3 bridges them).
            self.update_mount(world, raining, wind);
            self.horizontal_movement(input);
        }
        // `Player.cs:19637-19641` (`UpdateJumpHeight`'s tail): Dazed (160)
        // divides the jump statics — height /5, speed /2 — after the sticky
        // clamp (the mount block above carries that one).
        if self.dazed {
            self.jump_height /= 5;
            self.jump_speed /= 2.0;
        }

        let in_liquid = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        // `Player.cs:27352-27359`: the plain arm integrates the rebased
        // gravity (the `:24380-24437` rebase above owns the liquid values)
        // at FULL strength and caps the fall at the rebased `maxFallSpeed`
        // — the `gravity / 3` / `maxFallSpeed / 3` sites (`:27008`/`:27339`)
        // are the mount/wings flight integrators, not a wet rule.
        let gravity = tick_gravity;
        let fall_cap = tick_max_fall;
        // `WetCollision`'s `movementSpeed` (`Player.cs:28168-28171` + :23436):
        // the position delta scales by the liquid speed — water/lava 0.5,
        // honey 0.25, shimmer 0.375. The scaled step feeds the collision; the
        // velocity itself keeps integrating (the exit speed is preserved).
        self.liquid_step =
            movement_speed_for(self.wet, self.lava_wet, self.honey_wet, self.shimmer_wet);

        let jump_pressed = input.jump && !self.jump_held;
        self.jump_held = input.jump;
        // `:21024` clears the latch at the END of the held block, so the
        // jump branches read the PREVIOUS tick's latch — capture it first.
        let released = self.release_jump;
        if input.jump && !grappling {
            // `:21024`: any held frame clears the release latch.
            self.release_jump = false;
            if self.on_ground && jump_pressed {
                // `Player.cs:20671-20675`: the cart nudge — the mount jump
                // lifts the magnet a hair off the rail line so the ride state
                // machine cannot re-catch the jump.
                if self.cart_mounted {
                    self.y -= 0.001;
                }
                // `Player.cs:20752-20754`: leaving the rail levels the tilt.
                if !self.on_track {
                    self.full_rotation = 0.0;
                }
                // `Player.cs:20766-20767`: the ground jump; the landing re-arm
                // follows (`:20755-20758` - `RefreshDoubleJumps`). `:20678`
                // sets `justJumped` on the grounded trigger. `:20749-20750`:
                // the trigger disarms the rocket (it re-arms midair).
                // W4-c14 - `:20762-20764`: the Pogo Stick's (43) ground jump
                // plays Item168 at the CENTRE before the velocity writes
                // (the `pending_cart_sounds` drain, the hop arm above).
                if self.mount.is_active() && self.mount.kind() == 43 {
                    let (cx, cy) = self.center();
                    self.pending_cart_sounds.push(crate::minecart::CartCue {
                        cue: "Item_168",
                        x: cx,
                        y: cy,
                    });
                }
                self.just_jumped = true;
                self.can_rocket = false;
                self.rocket_release = false;
                self.vy = -self.jump_speed;
                self.jump = self.jump_height;
                self.on_ground = false;
                self.refresh_double_jumps();
            } else if self.jump > 0 {
                if self.vy == 0.0 {
                    self.jump = 0;
                } else {
                    self.vy = -self.jump_speed;
                    self.jump -= 1;
                }
            } else if self.in_water {
                self.vy = -2.0;
            } else if self.jump == 0
                && released
                && self.can_jump_again.iter().any(|&c| c)
                && !self.in_water
            {
                // `Player.cs:20698-21022`: the extra-jump priority chain —
                // the mount jumps first, then Sandstorm, Blizzard, Fart,
                // Sail, Cloud as the unconditional fall-through. Each sets
                // the standard ascent velocity and its own hold-boost
                // counter; the performing flag is the visual state.
                let jumped = ExtraJump::PRIORITY
                    .iter()
                    .copied()
                    .find(|j| self.can_jump_again[j.index()]);
                if let Some(j) = jumped {
                    self.can_jump_again[j.index()] = false;
                    self.performing_jump = Some(j);
                    // `:20678`: only a grounded trigger counts as justJumped.
                    if self.vy == 0.0 {
                        self.just_jumped = true;
                    }
                    // `:20749-20750`: the trigger disarms the rocket.
                    self.can_rocket = false;
                    self.rocket_release = false;
                    // S1-E-21f — the chain's sound plays (S1-E-21f): every
                    // arm but the Fart plays `PlaySound(16, (int)position.X,
                    // (int)position.Y)` (`:20793` Sandstorm, `:20803`
                    // Blizzard, `:20815` Sail, `:20871` Unicorn, `:20899`
                    // WallOfFleshGoat, `:20927` Basilisk, `:20957` Santank,
                    // `:21004` Cloud) BEFORE the velocity/jump writes; the
                    // Fart plays `SoundID.Item16` at `position` (`:20846`).
                    // The int overload draws no GetRandomPitch; the case-16
                    // pitch roll rides the drain's `UpdatePlayers` stream.
                    if j == ExtraJump::Fart {
                        if let Some(idx) = crate::soundid::SOUND_STYLES
                            .iter()
                            .position(|s| s.name == "Item16")
                        {
                            self.pending_fart_sound = Some((idx as u16, self.x, self.y));
                        }
                    } else {
                        self.pending_double_jump_sound = Some((self.x, self.y));
                    }
                    self.vy = -self.jump_speed;
                    self.jump = (self.jump_height as f32 * j.hold_multiplier()) as i32;
                } else {
                    // `:20746`: the Cloud fall-through consumed even when
                    // unarmed — the can flag clears all the same.
                    if let Some(cloud) = self.can_jump_again.get_mut(0) {
                        *cloud = false;
                    }
                }
            }
        } else {
            self.jump = 0;
            // `:21026-21031`: jump released — the rocket re-arm latch and the
            // wing/rocket ground refills gate on it.
            self.release_jump = true;
            self.rocket_release = true;
        }

        // `Player.cs:26061-26099`: the stale option clear while airborne.
        self.clear_stale_jumps();

        // `Player.Update` → `DashMovement` (`Main` call order `:26757`): the
        // velocity-only dash pass before the collision step.
        if !grappling {
            self.dash_movement(world, input);
        }
        // `Player.cs:26799-26812`: while CCed the mount dismounts and the
        // plain gravity takes over (the port's CC input zeroing and the
        // flight gate below already cover the rest of that branch).
        if !grappling && self.cced() && self.mount.is_active() {
            self.try_dismount(world);
        }

        // `Player.Update` → `GrappleMovement` (`Main` call order `:27653`,
        // `Player.cs:22613-22740`): the pull REPLACES the velocity — the
        // vector to the attached hook center clamped to the pull speed. The
        // fall origin refreshes per tick (no fall damage accrues).
        if grappling {
            self.grapple_movement(world, grapple, input, item_in_use);
        }

        // `Player.cs:27654`: `StickyMovement()` right after `GrappleMovement()`
        // and before `CheckDrowning()` (`:27655`). `flag` is the cart above
        // 5 px/tick (`:22918-22921`), `flag2` the gel/slime mounts (`:22922-22925`).
        let sticky_cart = self.mount.is_active()
            && self.mount.kind() > 0
            && crate::mount::is_cart(self.mount.kind())
            && self.vx.abs() > 5.0;
        let sticky_half = (self.mount.is_active() && self.mount.kind() == 56)
            || self.mount.kind() == 61;
        self.sticky_movement(world, sticky_cart, sticky_half, rng);

        // The flight section (`Player.cs:26744-27360`): the wings and the
        // rocket boots REPLACE the gravity integration on the ticks they run.
        // The plain gravity below applies only in the fallback. The wet ticks
        // skip the flight (the vanilla water physics dominate; the swim cap
        // stays).
        let flew = if !in_liquid && !self.webbed && !grappling {
            self.flight_section(gravity, fall_cap)
        } else {
            false
        };
        // `Player.cs:26821-26824`: a rest tick clears the ramp freeze.
        if self.vy == 0.0 {
            self.cart_ramp_time = 0;
        }
        if !flew && !self.webbed && !grappling {
            // `Player.cs:27352-27359`: the ramp freeze — gravity applies only
            // once the ramp timer has run out; otherwise it ticks down.
            if self.cart_ramp_time <= 0 {
                self.vy = (self.vy + gravity).min(fall_cap);
            } else {
                self.cart_ramp_time -= 1;
            }
        }
        // `Player.cs:27361-27393`: the fall-cap block runs AFTER the whole
        // else-if chain (the flight arms included) — the plain maxFallSpeed
        // clamp first (`:27365-27368`), then Featherfall caps the fall at
        // maxFallSpeed/3 (no hover-down), and the hover-up latch tightens it
        // to /10 once past /5. The gate is `!mount.Active || mount.Type != 5`;
        // the gravDir == -1 mirror (`:27380-27391`) has no ported gravity flip.
        if !self.mount.is_active() || self.mount.kind() != 5 {
            if self.vy > fall_cap {
                self.vy = fall_cap;
            }
            if self.slow_fall && self.vy > fall_cap / 3.0 && !self.trying_to_hover_down() {
                self.vy = fall_cap / 3.0;
            }
            if self.slow_fall && self.vy > fall_cap / 5.0 && self.trying_to_hover_up() {
                self.vy = fall_cap / 10.0;
            }
        }
        // `Player.cs:27400-27403`: a mount that cannot use wings zeroes the
        // flap frame (the draw pose).
        if !self.mount.can_use_wings() {
            self.wing_frame = 0;
        }
        // `Player.cs:27404-27425`: the hover dampening runs AFTER the whole
        // chain AND the fall-cap block — every path reaches it (the flying
        // tick, the glide, the plain fall), gated on CanUseWingAbilities.
        if !grappling {
            self.hover_dampening();
        }
        self.vy = self.vy.max(if in_liquid {
            -(MAX_FALL / 3.0)
        } else {
            -MAX_FALL
        });

        // `Player.cs:28093-28161`: the `Minecart.TrackCollision` slot — it runs
        // before the integration because it snaps Y itself and rewrites the
        // velocity (pitfall §10.3: X moves in the integration below).
        let was_on_track = self.on_track; // `:28093` `flag32` - the off-rail JumpingSound delegate (the fart carts' Item16, see below)
        self.on_track = false; // `:28092`
        let mut hit_switch_flag = false; // `flag33`
        if self.cart_mounted {
            // `:28099-28106`: the water scale wraps only this call
            // (`ignoreWater` has no port field; `merman` now exists but the
            // cart water-scale arm is a minecart seam — both treated false).
            let scale = if self.shimmer_wet {
                0.25
            } else if self.honey_wet {
                0.25
            } else if !self.wet {
                1.0
            } else {
                0.5
            };
            let body_h = self.body_height() as i32;
            // `Player.cs:28110-28111` + `Minecart.cs:809-813`: the landing
            // event fires mid-call with the pre-resolve `Position` and the
            // rotation the caller set before the call — both captured here.
            let pre_pos = Vec2::new(self.x, self.y);
            let pre_rotation = self.full_rotation;
            let mut pos = Vec2::new(self.x, self.y);
            let mut vel = Vec2::new(self.vx, self.vy);
            vel *= scale;
            // `:28109-28110` — `controlDown`/`controlUp` are the follow flags
            // (the port's input has no up bit; single-rail rides never read it).
            let flags = minecart::track_collision(
                world,
                &mut pos,
                &mut vel,
                &mut self.last_boost,
                WIDTH as i32,
                body_h,
                self.control_down,
                false,
                self.fall_start2,
                false,
            );
            vel /= scale;
            // `:28116-28119` — the non-cart position restore never fires for
            // carts (`!this.mount.Cart` gate).
            if flags.on_track() {
                self.on_track = true; // `:28113`
                // `:28114-28115` — the rail-slope visual correction.
                let (gfx, rot) = minecart::track_rotation(
                    world,
                    pos + vel,
                    WIDTH as i32,
                    body_h,
                    self.control_down,
                    false,
                );
                self.gfx_off_y = gfx;
                self.full_rotation = rot;
                self.full_rotation_origin = ((WIDTH / 2.0), body_h as f32);
            }
            // `:28117-28119`: the riding-off-rail `MinecartJumpingSound`
            // delegate - EMPTY for the base cart (`DelegateMethods.cs:244-
            // 246`); W4-c14: the Fart/Terra-Fart carts (51/53) play Item16
            // (`JumpingSoundFart`, `:308-312`) at the POST-call position.
            if was_on_track && !self.on_track && matches!(self.cart_mount, 51 | 53) {
                minecart::minecart_jumping_sound_fart(
                    &mut self.pending_cart_sounds,
                    pos,
                    WIDTH as i32,
                    body_h,
                );
            }
            if flags.landing {
                // `Minecart.cs:809-813`: the landing check -
                // `MinecartLandingSound` (Item53) + a 10-spark burst, both
                // with the pre-call position and rotation captured above.
                // W4-c14: the fart carts' delegate plays Item16 THEN
                // Item53 (`LandingSoundFart`, `DelegateMethods.cs:314-319`);
                // the Meowmere cart (33) plays the case-37 style-5 cue
                // (`Mount.MeowcartLandingSound`, `Mount.cs:659-662`).
                if matches!(self.cart_mount, 51 | 53) {
                    minecart::minecart_landing_sound_fart(
                        &mut self.pending_cart_sounds,
                        pre_pos,
                        WIDTH as i32,
                        body_h,
                    );
                } else if self.cart_mount == 33 {
                    minecart::meowcart_landing_sound(
                        &mut self.pending_cart_sounds,
                        pre_pos,
                        WIDTH as i32,
                        body_h,
                    );
                } else {
                    minecart::minecart_landing_sound(
                        &mut self.pending_cart_sounds,
                        pre_pos,
                        WIDTH as i32,
                        body_h,
                    );
                }
                minecart::wheel_sparks(
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pre_pos,
                    WIDTH as i32,
                    body_h,
                    10,
                    pre_rotation,
                );
            }
            if flags.bouncy_bumper() {
                // `:28123-28136`: the sprite flip follows the held direction
                // keys; the body direction follows the flipped roll.
                if self.control_left || self.control_right {
                    self.cart_flip = !self.cart_flip;
                }
                if vel.x > 0.0 {
                    self.facing = 1;
                } else if vel.x < 0.0 {
                    self.facing = -1;
                }
                // `:28140`: `MinecartBumperSound` → Item56, at the
                // post-TrackCollision position (`base.position`, pre-integration).
                // W4-c14: the Meowmere cart (33) plays the case-37 style-3
                // cue instead (`Mount.MeowcartBumperSound`, `Mount.cs:664-667`);
                // the fart carts play Item16 THEN Item56 (`BumperSoundFart`,
                // `DelegateMethods.cs:321-326`).
                if self.cart_mount == 33 {
                    minecart::meowcart_bumper_sound(
                        &mut self.pending_cart_sounds,
                        pos,
                        WIDTH as i32,
                        body_h,
                    );
                } else if matches!(self.cart_mount, 51 | 53) {
                    minecart::minecart_bumper_sound_fart(
                        &mut self.pending_cart_sounds,
                        pos,
                        WIDTH as i32,
                        body_h,
                    );
                } else {
                    minecart::minecart_bumper_sound(
                        &mut self.pending_cart_sounds,
                        pos,
                        WIDTH as i32,
                        body_h,
                    );
                }
            }
            if flags.hit_switch() {
                hit_switch_flag = true; // `:28138-28140`
            }
            if flags.used_ramp() {
                // `:28148-28150` — the gravity-freeze grant, truncated.
                self.cart_ramp_time = (1.0f32.min(self.vx.abs() / CART_RUN_SPEED) * 20.0) as i32;
            }
            if flags.boost_left() {
                self.track_boost -= minecart::BOOSTER_SPEED; // `:28151-28153`
            }
            if flags.boost_right() {
                self.track_boost += minecart::BOOSTER_SPEED; // `:28155-28157`
            }
            self.x = pos.x;
            self.y = pos.y;
            self.vx = vel.x;
            self.vy = vel.y;
        }
        // The AABB resolve (`TileCollision` + the step/slope helpers): the
        // velocity phase above only shaped `velocity`; this integrates it.
        // `base.oldPosition = base.position` (`Player.cs:28071`): the position
        // before this tick's movement — the switch-tile bridge reads it to
        // tell "moved onto the plate" from "stands on it".
        self.old_x = self.x;
        self.old_y = self.y;
        self.move_vanilla(world, input.down, gravity);
        // `Player.cs:28237-28241` + `Minecart.HitTrackSwitch`
        // (`Minecart.cs:1281-1288`): the pressure-track trip fires at the
        // post-integration magnet tile (net message 59 is out of scope; the
        // caller applies `Wiring::hit_switch`).
        if hit_switch_flag {
            let m = minecart::magnet_position(
                Vec2::new(self.x, self.y),
                WIDTH as i32,
                self.body_height() as i32,
            );
            self.pending_switch_trip = Some(((m.x / 16.0) as i32, (m.y / 16.0) as i32));
        }

        // `Player.cs:25219-25224`: the tick petrification lands, a flat hit -
        // the difficulty multiplier is 1 (classic) until M6-CREATIVE lands.
        let stoned = self.buffs.find_buff_index(156).is_some();
        if stoned != self.last_stoned {
            if stoned && !self.last_stoned {
                let cx = self.x + self.width * 0.5;
                self.hurt_inner(20, cx, false, rng, true, false, None, true);
            }
            // W4-c14 - `:25225`: the transition's Dig-0 plays on BOTH edges
            // (the C# site sits outside the `stoned` arm), at the truncated
            // top-left; the case-0 draws ride the drain. The 20 dust 1
            // (`:25226-25233`) are the visual seam.
            self.pending_stoned_sound = Some((self.x.trunc(), self.y.trunc()));
        }
        self.last_stoned = stoned;

        // `Player.cs:25293-25385`: the fall-damage block — the whole thing runs
        // whenever the player has come to rest this tick (`velocity.Y == 0`).
        // `:26057-26060`: every rest tick re-arms the granted extra jumps.
        if self.vy == 0.0 {
            self.refresh_double_jumps();
            // `num10 = 25 + extraFall` (`Player.cs:25301`); the mount addendum
            // (`Player.cs:25301-25304`): `25 + extraFall` + the mount's own.
            let threshold = 25 + self.extra_fall + self.mount.extra_fall();
            // `num11` is int-truncated on both ends (`Player.cs:25306`).
            let mut fell = (self.y / TILE_PX) as i32 - self.fall_start;
            // `:25307-25318`: the zeroing — a flying mount and the bunny
            // (type 1) nullify.
            if self.mount.can_fly() || self.mount.kind() == 1 {
                fell = 0;
            }
            // `Player.cs:25311-25314`: a track rider resting on rails takes no
            // fall damage — `Minecart.OnTrack` reads the magnet tile.
            if self.cart_mounted
                && minecart::on_track(
                    world,
                    Vec2::new(self.x, self.y),
                    WIDTH as i32,
                    self.body_height() as i32,
                )
            {
                fell = 0;
            }
            // The dash zero of `:25319-25323` waits for its own task; the
            // cloud/soft-tile scan does
            // (`Player.cs:25325-25345`): the tile row under the feet across
            // the hitbox's x-extent.
            let tx0 = (self.x / TILE_PX) as i32;
            let tx1 = ((self.x + self.width) / TILE_PX) as i32;
            // `Player.cs:25321` reads `base.height` — the live hitbox height
            // (the mount framework's equip for its mounts; the cart's 52) scans
            // the tile row, not the row above it.
            let ty = ((self.y + self.body_height() + 1.0) / TILE_PX) as i32;
            for tx in tx0..=tx1 {
                if let Some(t) = world.tile(tx, ty) {
                    if t.active()
                        && (tr_core::tiledata::SET_CLOUDS.has(t.kind)
                            || t.kind == 666
                            || (t.kind == 19 && (t.frame_y / 18) == 49))
                    {
                        fell = 0;
                        break;
                    }
                }
            }
            if stoned {
                // `Player.cs:25347-25359`: petrified — a fixed 2-tile grace and
                // 20 damage per tile; extraFall and the mount multiplier are
                // ignored, and the immunity clears AFTER the hit.
                let dmg = (fell - 2) * 20;
                if dmg > 0 {
                    let cx = self.x + self.width * 0.5;
                    self.hurt_inner(dmg, cx, false, rng, true, false, None, true);
                    self.hurt_cd = 0.0;
                }
            } else if (fell > threshold) && !self.no_fall_dmg && !self.has_wings {
                // `Player.cs:25360-25374`: `(fell - threshold) * 10` damage with
                // hitDirection 0 — no knockback — and `immune = false` forced
                // BEFORE the Hurt so the hit lands through any iframes.
                // `:25363-25367`: the mount's `FallDamage` scales the hit
                // (only while a mount is active — the C# gates on
                // `mount.Active`).
                let dmg = if self.mount.is_active() {
                    (((fell - threshold) * 10) as f32 * self.mount.fall_damage()) as i32
                } else {
                    (fell - threshold) * 10
                };
                self.hurt_fall(dmg, rng);
            }
            self.fall_start = (self.y / TILE_PX) as i32;
        }
        // `Player.cs:25379-25382`: the continuous reset — jumping, liquid,
        // slowFall keep the fall origin at the current row (the C#'s
        // `base.wet` covers honey too — the port's water-only `in_water`
        // stand-in stays, docs/ANALYSIS.md Phase 4; the sky-gravity/tongued
        // terms are unported).
        if self.jump > 0 || self.rocket_delay > 0 || self.in_water || self.slow_fall {
            self.fall_start = (self.y / TILE_PX) as i32;
        }

        // `Player.cs:28269-28272`: `FloorVisuals` after this tick's movement —
        // the movement block reads the flags next tick (one tick stale).
        if self.vy == 0.0 && !grappling {
            self.floor_visuals(world);
        }

        self.update_survival(world, rng);

        if self.x < 0.0 {
            self.x = 0.0;
            self.vx = 0.0;
        }
        let max_x = (world.width as f32 * TILE_PX) - self.width;
        if self.x > max_x {
            self.x = max_x;
            self.vx = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 300.0 {
            self.respawn();
        }

        // The immune purge runs near the end of the original update, after movement
        // and effects (`Player.cs:26012-26018`).
        self.buffs.purge_immune_buffs();

        // `Player.cs:24706-24708`: the footstep cooldown ticks down (the
        // C# runs it before `PlayerFrame` `:28286`, which reads the value).
        if self.run_sound_delay > 0 {
            self.run_sound_delay -= 1;
        }

        self.update_frames(world);

        // `Player.cs:27618-27635`: the release flags recompute at the end of
        // the tick — the double-tap reads the press edges against them.
        self.right_prev = input.right;
        self.left_prev = input.left;
    }

    /// `Mount.TryLanding` (`Mount.cs:4562-4568`): while the frame state is
    /// still air/fly, a 4px-down tile probe snapping shut pulls the frame to
    /// the grounded set on the touchdown tick (the C# re-probes
    /// `TileCollision` with `UnitY * gravDir * 4`).
    fn try_landing(&mut self, world: &World) {
        if !matches!(
            self.mount.frame_state(),
            crate::mount::FRAME_FLYING | crate::mount::FRAME_IN_AIR
        ) {
            return;
        }
        // The `!controlUp && !controlDown && !controlJump` gate — the port
        // has no controlUp input yet.
        if self.control_down || self.control_jump {
            return;
        }
        let probe = collision::tile_collision(
            world,
            [self.x, self.y],
            [0.0, 4.0],
            self.width,
            self.height,
            false,
            false,
        );
        if probe.vel[1] == 0.0 {
            // `GetIntendedGroundedFrame` (`Mount.cs:4547-4560`).
            let state = if self.vx == 0.0
                || ((self.slippy || self.slippy2) && !self.control_left && !self.control_right)
            {
                crate::mount::FRAME_STANDING
            } else {
                crate::mount::FRAME_RUNNING
            };
            self.mount
                .update_frame(
                    state,
                    self.vx,
                    self.vy,
                    self.just_jumped,
                    self.control_down,
                    self.sliding,
                    self.dash,
                    self.dash_delay,
                );
        }
    }

    /// `PlayerFrame()` (`Player.cs:35755-36647`) — the legs and body rows of
    /// every pose. The chain order is the vanilla one: the swim window, the
    /// mount animator (`flag7` plus `DoesNotOverrideLegFrames`), the leg rows,
    /// then the body rows (item use → pulley → hold styles → shield → mount →
    /// grapple → water → swim → air → walk → idle).
    ///
    /// The C# reads the held item out of `inventory[selectedItem]`; the port
    /// keeps the inventory in the app, which hands the same fields in through
    /// [`FrameInputs`] before each fixed step (`world_sim/player.rs`, beside
    /// `swing_aim`).
    ///
    /// Arms whose granting equipment has no port field yet stay coded but
    /// unreachable until it lands: Sandstorm in a Bottle (`sand_storm`), the
    /// Magic Carpet (`carpet_frame`), flippers (`ShouldFloatInWater`), the
    /// pulley/rope item (`pulley`) and the raised shield (`shield_raised`).
    fn update_frames(&mut self, world: &World) {
        let held = self.frame_inputs;
        // `:35947-35949`: `PlayerFrame` re-derives the fast-run footstep cue
        // every frame — the default is the legacy Run (17, style -1, cooldown
        // 9); the Moon Lord legs (140) swap in the Item-24 click at cooldown 6
        // (`:35959-35964`). The port's legs read is `legs_armor` (the C#'s
        // `:35767` effective-legSlot assignment has no port producer yet —
        // the override is coded, unreachable until the armor visuals land).
        self.hermes_step_sound = if self.legs_armor == 140 {
            (2, 24, 6)
        } else {
            (17, -1, 9)
        };
        // `:35756-35762`: the swim window ticks down and dies out of water.
        if self.swim_time > 0 {
            self.swim_time -= 1;
            if !self.wet {
                self.swim_time = 0;
            }
        }
        // `:36148-36240`: `flag7` — the mount owns the leg frames unless it is
        // one of the 57-60 roller-skate mounts
        // (`MountID.Sets.DoesNotOverrideLegFrames`, `MountID.cs:39`). Every
        // mount still runs its own animator, so this block does not return.
        let mut mount_owns_legs = false;
        if self.mount.is_active() {
            let kind = self.mount.kind();
            if !crate::mount::does_not_override_leg_frames(kind) {
                mount_owns_legs = true;
                // `:36150-36161`: row 6; the 23/45/48 carve-out forces row 0.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 6;
                if matches!(kind, 23 | 45 | 48) {
                    self.leg_frame = 0;
                }
            }
            if self.vy != 0.0 && self.mount.running_grace_time() <= 0 {
                if self.mount.fly_time() > 0
                    && self.jump == 0
                    && self.control_jump
                    && !self.mount.can_hover()
                {
                    // `:36199-36200`: `TryBeginningFlight` only fires for
                    // hover mounts (`Mount.cs:4537` — none in this table);
                    // the Rudolph dust trail is a visual seam.
                    self.mount.update_frame(
                        crate::mount::FRAME_FLYING,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                        self.sliding,
                        self.dash,
                        self.dash_delay,
                    );
                    self.try_landing(world);
                } else if self.wet {
                    // `:36203-36206`: the swim state (the ground mounts'
                    // swim sets reuse their in-air frames).
                    self.mount.update_frame(
                        crate::mount::FRAME_SWIMMING,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                        self.sliding,
                        self.dash,
                        self.dash_delay,
                    );
                } else {
                    // `:36207-36212`: plain air.
                    self.mount.update_frame(
                        crate::mount::FRAME_IN_AIR,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                        self.sliding,
                        self.dash,
                        self.dash_delay,
                    );
                    self.try_landing(world);
                }
            } else {
                // `:36216` -> `GetIntendedGroundedFrame` (`Mount.cs:4547-4560`):
                // the hooks-grapple arm cannot fire (no rows in CanUseHooks);
                // the `windPushed` term has no port counterpart.
                let state = if self.vx == 0.0
                    || ((self.slippy || self.slippy2) && !self.control_left && !self.control_right)
                {
                    crate::mount::FRAME_STANDING
                } else {
                    crate::mount::FRAME_RUNNING
                };
                self.mount.update_frame(
                    state,
                    self.vx,
                    self.vy,
                    self.just_jumped,
                    self.control_down,
                    self.sliding,
                    self.dash,
                    self.dash_delay,
                );
            }
        }

        // `:36219-36291`: the leg rows.
        if !mount_owns_legs && self.legs_armor != 140 {
            if self.swim_time > 0 {
                // `:36221-36233`: the swim paddle — a flat 2 per tick.
                self.leg_frame_counter += 2.0;
                while self.leg_frame_counter > 8.0 {
                    self.leg_frame_counter -= 8.0;
                    self.leg_frame += 1;
                }
                if self.leg_frame < 7 {
                    self.leg_frame = 19;
                } else if self.leg_frame > 19 {
                    self.leg_frame = 7;
                }
            } else if self.vy != 0.0 || self.grappling_active {
                // `:36234-36240`: the jump/fall/hang row 5; the three
                // always-animated wing sets keep row 0.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 5;
                if matches!(self.wings_logic, 22 | 28 | 45)
                    && self.should_draw_wings_that_are_always_animated()
                {
                    self.leg_frame = 0;
                }
            } else if self.vx != 0.0 {
                // `:36242-36275`: the walk cycle. The slippery no-input arm
                // (and the roller skates, `MountID.Sets.IsRollerSkates`) pins
                // the rest row.
                let roller_skates =
                    self.mount.is_active() && crate::mount::is_roller_skates(self.mount.kind());
                if ((self.slippy || self.slippy2) | roller_skates)
                    && !self.control_left
                    && !self.control_right
                {
                    self.leg_frame_counter = 0.0;
                    self.leg_frame = 0;
                } else {
                    let mut num19 = self.vx.abs() * 1.3;
                    // `:36255-36258`: the skates lock to 2 at full speed.
                    if roller_skates && self.vx.abs() >= self.max_run_speed {
                        num19 = 2.0;
                    }
                    self.leg_frame_counter += num19;
                    while self.leg_frame_counter > 8.0 {
                        self.leg_frame_counter -= 8.0;
                        self.leg_frame += 1;
                    }
                    if self.leg_frame < 7 {
                        self.leg_frame = 19;
                    } else if self.leg_frame > 19 {
                        self.leg_frame = 7;
                    }
                }
            } else {
                // `:36276-36282`: the rest row.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 0;
            }
        }
        // `:36292-36297`: the Magic Carpet glide pins the rest row.
        if self.carpet_frame >= 0 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        }
        // `:36298-36311`: the sandstorm dash pins the rest row and turns the
        // player every 4 ticks while no item is in use.
        if self.sand_storm {
            if self.grappling_active {
                self.sand_storm = false;
            }
            if self.misc_counter % 4 == 0 && !held.swing_active {
                self.facing = -self.facing;
            }
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        } else if held.swing_active && held.use_style_blocks_frames {
            // `:36312-36444`: the item-use rows per useStyle.
            self.apply_item_use_frame(held.swing_remaining, held.use_style, held.aim_relative);
        } else if self.pulley {
            // `:36445-36454`: the rope pose.
            self.body_frame = if self.pulley_dir == 2 { 1 } else { 2 };
        } else if held.can_visually_hold
            && held.hold_style == 1
            && held.hold_style_dry
            && held.torch_hold_ok
        {
            // `:36455-36459`: the torch hold (row 3).
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 2 && held.hold_style_dry {
            // `:36460-36464`: the held-up item (row 2).
            self.body_frame = 2;
        } else if held.can_visually_hold && held.hold_style == 3 {
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 5 {
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 7 {
            // `:36477-36481`: the holdStyle 7 row 11.
            self.body_frame = 11;
        } else if held.can_visually_hold && held.hold_style == 4 && self.vy == 0.0 {
            // `:36483-36488`: the umbrella hold rests on row 0. (The C# also
            // requires `gravDir == 1`; the port has no gravity flip.)
            self.body_frame = 0;
        } else if self.shield_raised {
            // `:36489-36492`: the raised shield (row 10).
            self.body_frame = 10;
        } else if self.mount.is_active()
            && !crate::mount::does_not_override_body_frames(self.mount.kind())
        {
            // `:36493-36497`: the mount body row.
            self.body_frame_counter = 0.0;
            self.body_frame = self.mount.data().map_or(0, |d| d.body_frame) as u32;
        } else if self.grappling_active {
            // `:36498-36535`: the grapple pose leans with the hooks' centroid.
            self.sand_storm = false;
            let dx = held.grapple_dx;
            let dy = held.grapple_dy;
            if dy < 0.0 && dy.abs() > dx.abs() {
                self.body_frame = 2;
            } else if dy > 0.0 && dy.abs() > dx.abs() {
                self.body_frame = 4;
            } else {
                self.body_frame = 3;
            }
        } else if self.wet && self.should_float_in_water() {
            // `:36536-36539`: the float pose (row 10).
            self.body_frame = 10;
        } else if self.swim_time > 0 {
            // `:36540-36558`: the swim window — row 0 at the edges, row 5 in
            // the middle.
            self.body_frame = if self.swim_time > 20 {
                0
            } else if self.swim_time > 10 {
                5
            } else {
                0
            };
        } else if self.vy != 0.0 {
            // `:36559-36600`: the airborne rows.
            if self.sliding {
                self.body_frame = 3;
            } else if self.sand_storm || self.carpet_frame >= 0 {
                self.body_frame = 6;
            } else if self.eoc_dash > 0 {
                self.body_frame = 6;
            } else if self.wings_logic > 0 {
                self.body_frame = if matches!(self.wings_logic, 22 | 28 | 45) {
                    0
                } else if self.vy > 0.0 {
                    if self.control_jump { 6 } else { 5 }
                } else {
                    6
                };
            } else {
                self.body_frame = 5;
            }
            self.body_frame_counter = 0.0;
        } else if self.vx != 0.0 {
            // `:36602-36623`: the walk rows. The Moon Lord legs (140) run
            // their own counter at `|velocity.X| * 0.5`; everything else
            // copies the leg row.
            if self.legs_armor == 140 {
                self.body_frame_counter += self.vx.abs() * 0.5;
                while self.body_frame_counter > 8.0 {
                    self.body_frame_counter -= 8.0;
                    self.body_frame += 1;
                }
                if self.body_frame < 7 {
                    self.body_frame = 19;
                } else if self.body_frame > 19 {
                    self.body_frame = 7;
                }
            } else {
                self.body_frame_counter += self.vx.abs() * 1.5;
                self.body_frame = self.leg_frame;
            }
        } else {
            // `:36624-36633`: the rest row.
            self.body_frame_counter = 0.0;
            self.body_frame = 0;
        }

        // `:36634-36644`: the Moon Lord legs override their own rows after the
        // body chain — row 5 in the air, row 0 at rest, and row 7 whenever the
        // body row's headgear offset is zero.
        if self.legs_armor == 140 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = if self.vy != 0.0 { 5 } else { 0 };
            if offsets_player_headgear_y(self.body_frame as usize) == 0.0 {
                self.leg_frame = 7;
            }
            if matches!(self.wings_logic, 22 | 28) {
                self.leg_frame = 0;
            }
        }
        // `:36645-36651`: the sitting/sleeping legs (217) pin row 5.
        if self.legs_armor == 217 && (self.sitting || self.sleeping) {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 5;
        }
        // `AnimatePlayerAndGetItemFrame` (`Player.cs:43372-43380`): the
        // composite arms reset to DISABLED per frame and
        // `ItemCheck_ApplyUseStyle` reassigns them. The hold arms run when
        // `ItemTimeIsZero` (no swing); the use arms when `itemAnimation > 0`.
        self.composite_front = CompositeArmState::DISABLED;
        self.composite_back = CompositeArmState::DISABLED;
        if held.swing_active {
            self.apply_item_composite_arms(&held);
        } else {
            self.apply_hold_composite_arms(&held);
        }
    }

    /// `ItemCheck_ApplyUseStyle`'s composite-arm assignments
    /// (`Player.cs:50686-51320`), per `useStyle`. The rotations/curves are
    /// verbatim; `dir` is `Player.direction` (`gravDir` is always 1).
    fn apply_item_composite_arms(&mut self, held: &FrameInputs) {
        let dir = self.facing as f32;
        let progress = 1.0 - held.swing_remaining;
        match held.use_style {
            // `:50936-50955` (useStyle 2, EatFood): both arms at -PI/2 * dir;
            // the stretch walks Full -> None by `num10` quarters
            // (`num10 = 1 - itemAnimation / itemAnimationMax`).
            2 => {
                let st = if progress <= 0.25 {
                    CompositeStretch::Full
                } else if progress <= 0.5 {
                    CompositeStretch::ThreeQuarters
                } else if progress <= 0.75 {
                    CompositeStretch::Quarter
                } else {
                    CompositeStretch::None
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: st,
                    rotation: -std::f32::consts::FRAC_PI_2 * dir,
                };
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: st,
                    rotation: -std::f32::consts::FRAC_PI_2 * dir,
                };
            }
            // `:50885-50911` (useStyle 9): the front arm Full along the
            // `num7 -> num8` arc — `num6 = 2*PI/5`, `num7 = -num6/2 +
            // (1 - lerp) * num6 + PI/10`, `num8 = num7 - PI/2`, mirrored for
            // `direction == -1`; `lerp = GetLerpValue(0, 0.7, progress)`
            // clamped.
            9 => {
                let lerp = (progress / 0.7).clamp(0.0, 1.0);
                let num6 = std::f32::consts::PI * 2.0 / 5.0;
                let num7 = -num6 * 0.5 + (1.0 - lerp) * num6 + (std::f32::consts::PI / 10.0);
                let mut num8 = num7 - std::f32::consts::FRAC_PI_2;
                if dir < 0.0 {
                    num8 = -num8;
                }
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: num8,
                };
            }
            // `:50978-51007` (useStyle 11, MowTheLawn): the front arm walks
            // `num12 = progress * 2` through the two-cycle table; the back
            // arm pins Full at -PI/16 (no direction term).
            11 => {
                let num12 = progress * 2.0;
                let st = if num12 > 1.75 {
                    CompositeStretch::Full
                } else if num12 > 1.5 {
                    CompositeStretch::ThreeQuarters
                } else if num12 > 1.25 {
                    CompositeStretch::Quarter
                } else if num12 > 0.75 {
                    CompositeStretch::None
                } else if num12 > 0.5 {
                    CompositeStretch::Quarter
                } else if num12 > 0.25 {
                    CompositeStretch::ThreeQuarters
                } else {
                    CompositeStretch::Full
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: st,
                    rotation: -std::f32::consts::FRAC_PI_4 * dir,
                };
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: -(std::f32::consts::PI / 16.0),
                };
            }
            // `:51174-51196` (useStyle 13, Rapier): the front arm at
            // `itemRotation * gravDir - PI/2 * dir`, the stretch walking
            // Quarter -> ThreeQuarters -> Full by thirds of
            // `1 - (itemAnimation % useAnim) / useAnim`.
            13 => {
                let anim = held.swing_anim.max(0.0);
                let max = held.swing_anim_max.max(1.0);
                let num20 = 1.0 - (anim % max) / max;
                let st = if num20 > 0.66 {
                    CompositeStretch::Full
                } else if num20 > 0.33 {
                    CompositeStretch::ThreeQuarters
                } else {
                    CompositeStretch::Quarter
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: st,
                    // `itemRotation * Directions.Y` — `Directions.Y` is the
                    // gravity direction, always 1 in the port.
                    rotation: held.item_rotation - std::f32::consts::FRAC_PI_2 * dir,
                };
            }
            // `:51349-51360` (useStyle 14): the back arm Full at
            // `-3*PI/4 * dir`; the front arm untouched.
            14 => {
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: std::f32::consts::PI * -3.0 / 4.0 * dir,
                };
            }
            // `:51361-51375` (useStyle 15): both arms at -2*PI/5 * dir — the
            // back ThreeQuarters, the front Full.
            15 => {
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::ThreeQuarters,
                    rotation: std::f32::consts::PI * -2.0 / 5.0 * dir,
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: std::f32::consts::PI * -2.0 / 5.0 * dir,
                };
            }
            // `:51197-51274` (useStyle 8, the throw): three phases — the
            // wind-up (the first tick), the swing arc, the follow-through.
            // The port's swing never overshoots `itemAnimationMax`, so the
            // `itemAnimation > useAnimation` arm is unreachable (documented).
            8 => {
                let anim = held.swing_anim;
                let use_anim = held.use_animation as f32;
                if anim >= use_anim - 1.0 && anim <= use_anim {
                    // `:51205-51229`: the wind-up.
                    let mut item_rot = 10.0f32.atan2(10.0 * dir) + std::f32::consts::FRAC_PI_4;
                    if dir < 0.0 {
                        item_rot += std::f32::consts::FRAC_PI_2;
                    }
                    let num21 = -(std::f32::consts::PI / 10.0) * if dir < 0.0 { -1.0 } else { 1.0 };
                    let mut num22 = item_rot - std::f32::consts::FRAC_PI_4 + std::f32::consts::PI;
                    if dir > 0.0 {
                        num22 += std::f32::consts::FRAC_PI_2;
                    }
                    self.composite_front = CompositeArmState {
                        enabled: true,
                        stretch: CompositeStretch::Full,
                        rotation: num22 + num21,
                    };
                    self.composite_back = CompositeArmState {
                        enabled: true,
                        stretch: CompositeStretch::ThreeQuarters,
                        rotation: num22 - num21,
                    };
                } else {
                    // `:51246-51273`: the swing arc
                    // (`num26 = 1 - itemAnimation / itemAnimationMax`).
                    let num26 = 1.0 - held.swing_remaining;
                    let st = if num26 > 0.6 {
                        CompositeStretch::Quarter
                    } else {
                        CompositeStretch::ThreeQuarters
                    };
                    let num27 = -(std::f32::consts::PI / 10.0)
                        * (1.0 - num26 * 2.2)
                        * if dir < 0.0 { -1.0 } else { 1.0 };
                    let num28 = (num26 * std::f32::consts::PI * 0.6
                        + std::f32::consts::PI * 2.0 / 5.0)
                        * -dir
                        + std::f32::consts::PI
                        - std::f32::consts::FRAC_PI_4
                        + std::f32::consts::PI;
                    let num28 = if dir > 0.0 {
                        num28 + std::f32::consts::FRAC_PI_2
                    } else {
                        num28
                    };
                    self.composite_front = CompositeArmState {
                        enabled: true,
                        stretch: CompositeStretch::Full,
                        rotation: num28 + num27,
                    };
                    self.composite_back = CompositeArmState {
                        enabled: true,
                        stretch: st,
                        rotation: num28 - num27,
                    };
                }
            }
            // `:51275-51320` (useStyle 12, Guitar): the front arm walks
            // `num30 = progress * 2` (the two-cycle table with None at
            // 0.75..1.25) with `num31 = cos(num30 * PI) * 0.2`; the back arm
            // Quarter at `(-PI/4 - num31/2) * dir`.
            12 => {
                let num30 = progress * 2.0;
                let num31 = (num30 * std::f32::consts::PI).cos() * 0.2;
                let st = if num30 > 1.75 {
                    CompositeStretch::Full
                } else if num30 > 1.5 {
                    CompositeStretch::ThreeQuarters
                } else if num30 > 1.25 {
                    CompositeStretch::Quarter
                } else if num30 > 0.75 {
                    CompositeStretch::None
                } else if num30 > 0.5 {
                    CompositeStretch::Quarter
                } else if num30 > 0.25 {
                    CompositeStretch::ThreeQuarters
                } else {
                    CompositeStretch::Full
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: st,
                    rotation: (-std::f32::consts::FRAC_PI_4 + num31) * dir,
                };
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Quarter,
                    rotation: (-std::f32::consts::FRAC_PI_4 - num31 * 0.5) * dir,
                };
            }
            _ => {}
        }
    }

    /// `HoldItem`'s holdStyle composite arms (`Player.cs:50520-50582`). The
    /// holdStyle 1/2/3/7 rows ride the body-frame chain (no composite); the
    /// holdStyle 5/6 arms gate on `!pulley`; `can_visually_hold` gates the
    /// whole chain (`Player.cs:50271`).
    fn apply_hold_composite_arms(&mut self, held: &FrameInputs) {
        if !held.can_visually_hold || held.swing_active {
            return;
        }
        let dir = self.facing as f32;
        match held.hold_style {
            // `:50520-50549` (holdStyle 4, the umbrella): the front arm Full
            // and the back ThreeQuarters around `num9 +/- num8`.
            4 => {
                let mut item_rot = 10.0f32.atan2(10.0 * dir) + std::f32::consts::FRAC_PI_4;
                if dir < 0.0 {
                    item_rot += std::f32::consts::FRAC_PI_2;
                }
                let num8 = -(std::f32::consts::PI / 10.0) * if dir < 0.0 { -1.0 } else { 1.0 };
                let mut num9 = item_rot - std::f32::consts::FRAC_PI_4 + std::f32::consts::PI;
                if dir > 0.0 {
                    num9 += std::f32::consts::FRAC_PI_2;
                }
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: num9 + num8,
                };
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::ThreeQuarters,
                    rotation: num9 - num8,
                };
            }
            // `:50550-50562` (holdStyle 5): the back Quarter at -PI/4 * dir
            // and the front Full at -0.39269912 * dir.
            5 if !self.pulley => {
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Quarter,
                    rotation: -std::f32::consts::FRAC_PI_4 * dir,
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: -0.39269912 * dir,
                };
            }
            // `:50563-50570` (holdStyle 6): the back ThreeQuarters at
            // -3*PI/5 * dir.
            6 if !self.pulley => {
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::ThreeQuarters,
                    rotation: std::f32::consts::PI * -3.0 / 5.0 * dir,
                };
            }
            // `:50575-50582` (holdStyle 8): the back ThreeQuarters and the
            // front Full at -2*PI/5 * dir.
            8 => {
                self.composite_back = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::ThreeQuarters,
                    rotation: std::f32::consts::PI * -2.0 / 5.0 * dir,
                };
                self.composite_front = CompositeArmState {
                    enabled: true,
                    stretch: CompositeStretch::Full,
                    rotation: std::f32::consts::PI * -2.0 / 5.0 * dir,
                };
            }
            _ => {}
        }
    }


    /// `Player.ShouldDrawWingsThatAreAlwaysAnimated` (`Player.cs:30737-30748`):
    /// the three always-animated wing sets keep the rest legs while airborne.
    fn should_draw_wings_that_are_always_animated(&self) -> bool {
        self.vy != 0.0
            && !self.grappling_active
            && (!self.wet || !self.should_float_in_water())
            && self.mount.data().is_some_and(|d| d.can_use_wings)
    }

    /// `Player.ShouldFloatInWater` (`Player.cs:3872-3885`): `canFloatInWater`
    /// (the Water Strider's buff-265 grant, `:9897`, or the flippers/merman)
    /// while not holding down; mounted, only the Water Strider's mount (37)
    /// floats. The buff arm now grants `can_float_in_water`, so the pose arm
    /// is reachable.
    fn should_float_in_water(&self) -> bool {
        if self.can_float_in_water && !self.control_down {
            if self.mount.is_active() {
                return self.mount.kind() == 37;
            }
            return true;
        }
        false
    }

    /// `PlayerFrame()`'s item-use branch (`Player.cs:36312-36444`): the body
    /// rows per useStyle while `itemAnimation > 0`. `remaining` is
    /// `itemAnimation / itemAnimationMax` (1 at the swing's start, 0 at the
    /// end — the C# compares `itemAnimation` against the fractions directly);
    /// `aim_relative` is `itemRotation * direction` (the shooting styles' rows
    /// read it). The quirks are verbatim: style 3's both branches are row 3,
    /// style 5 reads the aim, style 13 mirrors style 1.
    fn apply_item_use_frame(&mut self, remaining: f32, use_style: u8, aim_relative: f32) {
        // The inverted remaining: `itemAnimation > max * frac` ==
        // `remaining < (1 - frac)` (itemAnimation runs 1 -> 0).
        let inv = 1.0 - remaining;
        match use_style {
            // `:36312-36326` (blocks/placeables; `type == 0` shares it) —
            // rows 1 -> 2 -> 3 by thirds.
            1 | 0 => {
                self.body_frame = if remaining < 0.333 {
                    3
                } else if remaining < 0.666 {
                    2
                } else {
                    1
                };
            }
            // `:36327-36331` — DrinkOld: the first half row 3, then row 2.
            7 => {
                self.body_frame = if remaining > 0.5 { 3 } else { 2 };
            }
            // `:36332-36334` — EatFood: row 3 through the use.
            2 => self.body_frame = 3,
            // `:36335-36345` — MowTheLawn: >half row 4, >15% row 3, else 2.
            11 => {
                self.body_frame = if remaining > 0.5 {
                    4
                } else if remaining > 0.15 {
                    3
                } else {
                    2
                };
            }
            // `:36346-36348` — DrinkLiquid: row 0.
            9 => self.body_frame = 0,
            // `:36349-36362` — DrinkLong: the progress `* 6` clamped at 1,
            // >= 0.5 row 3 else row 2.
            6 => {
                let num20 = (inv * 6.0).min(1.0);
                self.body_frame = if num20 >= 0.5 { 3 } else { 2 };
            }
            // `:36377-36387` — Thrust: BOTH branches row 3 (the vanilla
            // quirk — `:36385`'s else is dead; S1-P-i05 re-cite).
            3 => self.body_frame = 3,
            // `:36373-36375` — HoldUp: row 2.
            4 => self.body_frame = 2,
            // `:36376-36378` — GolfPlay: row 0.
            8 => self.body_frame = 0,
            // `:36379-36381` — Guitar: row 3.
            12 => self.body_frame = 3,
            // `:36382-36392` — Rapier: the same thirds as style 1.
            13 => {
                self.body_frame = if remaining < 0.333 {
                    3
                } else if remaining < 0.666 {
                    2
                } else {
                    1
                };
            }
            // `:36393-36444` — Shoot: the rows read `itemRotation *
            // direction` (the aim): the default row 3, past -0.75 row 2
            // (gravDir -1: row 4), past 0.6 row 4 (gravDir -1: row 2).
            5 => {
                self.body_frame = 3;
                let num21 = aim_relative;
                if num21 < -0.75 {
                    self.body_frame = 2;
                }
                if num21 > 0.6 {
                    self.body_frame = 4;
                }
            }
            _ => {}
        }
    }

    /// The expired-buff sweep of `Player.Update` (`Player.cs:25608-25616`): a
    /// slot holding a type with a non-positive timer is deleted. Like the C#,
    /// the loop does not rescan after `DelBuff` compaction.
    fn remove_expired_buffs(&mut self) {
        for i in 0..MAX_BUFFS {
            if self.buffs.buff_type[i] > 0 && self.buffs.buff_time[i] <= 0 {
                self.buffs.del_buff(i);
            }
        }
    }

    /// The slot scan behind the C# per-buff flag chain: a buff counts while it
    /// occupies a slot with a positive timer (`Player.cs:9869-9870`). The scan
    /// ignores `buffImmune` on purpose — an immune-held buff still sets its
    /// flag for the one frame before the purge deletes it (`:26011-26016`).
    fn holding_index(&self, buff_type: i32) -> Option<usize> {
        (0..MAX_BUFFS)
            .find(|&i| self.buffs.buff_time[i] > 0 && self.buffs.buff_type[i] == buff_type)
    }

    /// `true` while `buff_type` is held on a live slot (see [`Self::holding_index`]).
    fn holding(&self, buff_type: i32) -> bool {
        self.holding_index(buff_type).is_some()
    }

    /// The Wet-buff (103) player-side effects of the `UpdateBuffs` chain
    /// (`Player.cs:11740-11751`): while held, lava contact deletes the buff;
    /// otherwise it re-grants this frame's fire immunities (OnFire 24, OnFire3
    /// 323, Burning 67).
    fn refresh_wet_state(&mut self) {
        // The C# branch runs inside the per-buff slot loop and uses the
        // previous frame's `lavaWet`; the port's `in_lava` probe is this
        // frame's, so the removal lands one tick earlier than the original's
        // lagging flag (docs/ANALYSIS.md, Phase 4).
        if let Some(i) = self.holding_index(103) {
            if self.in_lava {
                self.buffs.del_buff(i);
            } else {
                self.buffs.buff_immune[24] = true;
                self.buffs.buff_immune[323] = true;
                self.buffs.buff_immune[67] = true;
            }
        }
    }

    /// The DoT branches of `Player.UpdateLifeRegen` as a pure derivation.
    ///
    /// Pseudocode (branch order = C# `Player.cs:18949→19095`; each hit branch
    /// clamps a positive regen and zeroes the ramp time, folded into
    /// `resets_time`):
    /// ```text
    /// regen ← 0; resets_time ← false
    /// poisoned(20): −4 | venom(70): −30 | chlorophyteSpore(397): −30
    /// onFire(24): −8, ×2 while drippingSlime(137)
    ///             (−100/×−100 while `Main.vampireSeed`, the `vampirism` micro seed)
    /// onFire3(323): −8 ×2 | frostburn(44): −16 ×2 | frostburn2(324): −16 ×2
    /// cursedInferno(39): −24 ×2 | burned(67): −60 ×2
    /// suffocating(68): −40
    /// electrified(144): −8, −32 more while controlLeft || controlRight
    /// tongued(38) && expert: −100
    /// bleeding(30): zeroes the ramp time only (no delta, `:19178-19181`)
    /// honey(48) while regen < 0: regen += 4, clamped at 0 (never positive)
    /// fast_lane ← burned || suffocating || (tongued && expert)      (:19313)
    /// death_fast ← suffocating → 7, else tongued → 12, else 8       (:19321-19331)
    /// death_generic ← vampire(22, unported) → poisoned||venom → 9 →
    ///                 electrified → 10 → chlorophyte → 23 → else 8  (:19362-19385)
    /// ```
    fn derive_dot_inputs(&self, expert: bool) -> DotDerivation {
        let mut regen = 0i32;
        let mut resets_time = false;
        // The DoT branches in the exact C# order (`Player.cs:18949-19095`).
        // Each branch clamps a positive regen and zeroes `lifeRegenTime`
        // before subtracting (`:18951-18956` shape) — the engine's
        // `dot_resets_time` covers the clamp+reset for the folded sum.
        if self.holding(20) {
            // Poisoned — `:18949-18957`.
            resets_time = true;
            regen -= 4;
        }
        if self.holding(70) {
            // Venom — `:18958-18966`.
            resets_time = true;
            regen -= 30;
        }
        if self.holding(397) {
            // Chlorophyte Spore — `:18967-18975`.
            resets_time = true;
            regen -= 30;
        }
        let drip = self.holding(137); // drippingSlime doubles the fire family
        if self.holding(24) {
            // OnFire — `:18976-18998`. M5-SEED-2: the `vampirism` world trades
            // the drain for its own (`Main.vampireSeed` → −100, and the
            // `drippingSlime` double is −100 instead of −8, `:18983-18992`) —
            // the flag comes from the world's micro seeds (`WorldGen.cs:555-558`).
            resets_time = true;
            if self.vampirism {
                regen -= 100;
                if drip {
                    regen -= 100;
                }
            } else {
                regen -= 8;
                if drip {
                    regen -= 8;
                }
            }
        }
        if self.holding(323) {
            // OnFire3 (hellfire) — `:19000-19012`.
            resets_time = true;
            regen -= 8;
            if drip {
                regen -= 8;
            }
        }
        if self.holding(44) {
            // Frostburn — `:19013-19025`.
            resets_time = true;
            regen -= 16;
            if drip {
                regen -= 16;
            }
        }
        if self.holding(324) {
            // Frostburn2 — `:19026-19038`.
            resets_time = true;
            regen -= 16;
            if drip {
                regen -= 16;
            }
        }
        if self.holding(39) {
            // Cursed Inferno (C# `onFire2`) — `:19039-19051`.
            resets_time = true;
            regen -= 24;
            if drip {
                regen -= 24;
            }
        }
        if self.holding(67) {
            // Burning (C# `burned`) — `:19052-19064`.
            resets_time = true;
            regen -= 60;
            if drip {
                regen -= 60;
            }
        }
        if self.holding(68) {
            // Suffocation — `:19065-19073`.
            resets_time = true;
            regen -= 40;
        }
        if self.holding(144) {
            // Electrified — `:19074-19086`; the extra 32 needs the run keys.
            resets_time = true;
            regen -= 8;
            if self.control_left || self.control_right {
                regen -= 32;
            }
        }
        let tongued = self.holding(38);
        if tongued && expert {
            // Tongued, expert-only — `:19087-19095`.
            resets_time = true;
            regen -= 100;
        }
        // Bleeding zeroes the ramp time without draining — `:19178-19181`.
        if self.holding(30) {
            resets_time = true;
        }
        // Honey softens a drain by 4, clamped at 0 so it never heals —
        // `:19096-19103`. At this point the C# `lifeRegen` holds only the DoT
        // sum (the flat adds come later), so folding here is exact.
        if self.holding(48) && regen < 0 {
            regen = (regen + 4).min(0);
        }
        // The fast-lane entry condition — `:19313`.
        let fast_lane = self.holding(67) || self.holding(68) || (tongued && expert);
        // The fast lane's kill reason — `:19319-19333` (suffocation 7 →
        // tongued 12 → the fire fallback 8).
        let death_fast = if self.holding(68) {
            DotDeath::Suffocation
        } else if tongued {
            DotDeath::Tongued
        } else {
            DotDeath::Burned
        };
        // The generic lane's kill reason — `:19361-19384`. The C# tests
        // `vampireBurningInSunlight` (22) first; nothing grants it in the
        // port, so the chain starts at poisoned/venom (9).
        let death_generic = if self.holding(20) || self.holding(70) {
            DotDeath::Poison
        } else if self.holding(144) {
            DotDeath::Electrified
        } else if self.holding(397) {
            DotDeath::ChlorophyteSpore
        } else {
            DotDeath::Burned
        };
        DotDerivation {
            regen,
            resets_time,
            fast_lane,
            death_fast,
            death_generic,
        }
    }

    /// `Player.RefreshDoubleJumps` (`Player.cs:22869-22908`): every option
    /// the equipment grants re-arms.
    fn refresh_double_jumps(&mut self) {
        for i in 0..9 {
            if self.has_jump_options[i] {
                self.can_jump_again[i] = true;
            }
        }
    }

    /// `Player.cs:26061-26099`: while airborne, an extra jump whose option
    /// vanished (an accessory swap) force-clears.
    fn clear_stale_jumps(&mut self) {
        if self.vy == 0.0 {
            return;
        }
        for i in 0..9 {
            if !self.has_jump_options[i] {
                self.can_jump_again[i] = false;
            }
        }
    }

    /// `Player.SpawnFastRunParticles`' sound half (`Player.cs:20205-20216`):
    /// the fast-run footstep — `runSoundDelay == 0` and grounded plays
    /// `hermesStepSound` at `position` (the top-left, NOT the centre) and
    /// re-arms the cooldown (`:20212-20215`). The caller drains the queue
    /// through `Audio::play_step_sound`. The dust trails (`:20217-20300`:
    /// the wings-3/rocket-6, sailDash, shoe-dye arms) are visual seams.
    fn spawn_fast_run_particles(&mut self) {
        if self.run_sound_delay == 0 && self.vy == 0.0 {
            let (sound_type, style, cooldown) = self.hermes_step_sound;
            self.pending_step_sound = Some((sound_type, style, self.x, self.y));
            self.run_sound_delay = cooldown;
        }
    }

    /// `Player.HorizontalMovement` (`Player.cs:19772-20088`): the primary
    /// acceleration gates on `maxRunSpeed`; between `maxRunSpeed` and
    /// `accRunSpeed` (the boot top speeds) acceleration continues at 0.2x —
    /// ground-only here, the wings/flying-mount terms land with PHYS-5/M2-
    /// MOUNT-1, and `dashDelay >= 0` always holds until M2-PHYS-4. No input
    /// decays by `runSlowdown` on the ground and half of it in the air. There
    /// is no hard clamp — the decay bounds the speed (knockback rides out).
    /// `Player.DashMovement` (`Player.cs:21034-21538`), dashTypes 1/2/3/6.
    /// The accessory dash granters (Tabi 977, EoCShield 3097, Master Ninja
    /// Gear 984 → type 1/2/1) arm `dash_type` in `apply_equip_functional`
    /// (S1-C-12a); the Solar set (type 3) rides the set rows and the Chillet
    /// mounts (type 6) their own task — `TR_DASH=<1|2|3|6>` pins the
    /// capability through `pinned_dash`. The dust/gore trails are visual seams;
    /// the vanilla plays no dash sounds.
    fn dash_movement(&mut self, world: &World, input: PlayerInput) {
        // `:21040-21048`: the lifecycle re-sync.
        if self.dash_delay == 0 {
            self.dash = self.dash_type;
        }
        if self.dash == 0 {
            self.dash_time = 0;
            self.dash_delay = 0;
            return;
        }
        if self.dash_delay > 0 {
            // `:21237-21248`: the cooldown ticks; the EoC grace drains with it.
            if self.eoc_dash > 0 {
                self.eoc_dash -= 1;
            }
            if self.eoc_dash == 0 {
                self.eoc_hit = -1;
            }
            self.dash_delay -= 1;
            return;
        }
        if self.dash_delay < 0 {
            // `:21249-21399`: the active dash — the duration is a speed-decay
            // model, not a tick timer.
            let mut threshold = 12.0f32;
            let mut fast = 0.992f32;
            let run_band = self.acc_run_speed.max(self.max_run_speed);
            let mut mid = 0.96f32;
            let mut cooldown = 20;
            match self.dash {
                // `:21271-21285`: the shield decays faster, cooldowns longer.
                2 => {
                    fast = 0.985;
                    mid = 0.94;
                    cooldown = 30;
                }
                // `:21286-21307`: the Solar dash rides higher and longer.
                3 => {
                    threshold = 14.0;
                    fast = 0.985;
                    mid = 0.94;
                }
                // `:21346-21351`: a direction flip or losing the capability
                // ends a mount dash at 30 with a damp.
                6 => {
                    if self.vx.signum() as i32 != self.facing || self.dash_type != 6 {
                        self.dash_delay = 30;
                        self.vx *= 0.3;
                        self.dash_time = 0;
                        return;
                    }
                }
                _ => {}
            }
            if self.dash <= 0 {
                return;
            }
            if self.vx > threshold || self.vx < -threshold {
                self.vx *= fast;
                return;
            }
            if self.vx > run_band || self.vx < -run_band {
                self.vx *= mid;
                return;
            }
            // `:21390-21398`: the dash ends — the cooldown starts and the
            // velocity pins to the run band in the held/facing direction.
            self.dash_delay = cooldown;
            if self.vx < 0.0 || (self.vx == 0.0 && input.left) {
                self.vx = -run_band;
            } else if self.vx > 0.0 || (self.vx == 0.0 && input.right) {
                self.vx = run_band;
            }
            return;
        }
        // `:21400-21537`: the dash start (`dashDelay == 0`), per type.
        let Some((dir, dashing)) = self.do_common_dash_handle(input) else {
            return;
        };
        if !dashing {
            return;
        }
        let speed = match self.dash {
            1 => 16.9,
            2 => 14.5,
            3 => 21.9,
            // `:21520-21523`: the mount hop at the start.
            6 => {
                if self.vy == 0.0 {
                    self.vy = -4.0;
                }
                16.0
            }
            _ => return,
        };
        self.vx = speed * dir as f32;
        // `:21414-21417`: a solid or sloped tile ahead halves the launch.
        if self.solid_ahead(world, dir) {
            self.vx /= 2.0;
        }
        self.dash_delay = -1;
        if self.dash == 2 {
            self.eoc_dash = 15;
        }
    }

    /// `Player.DoCommonDashHandle` (`Player.cs:21546-21605`): the 15-tick
    /// double-tap window (`Settings.DashControl == AllowDoubleTap`, the
    /// default) — the first tap arms `dashTime = ±15`, the same-direction
    /// press edge inside the window fires. The dedicated dash button
    /// (`controlDash`) has no port input binding yet.
    fn do_common_dash_handle(&mut self, input: PlayerInput) -> Option<(i32, bool)> {
        if self.dash_time > 0 {
            self.dash_time -= 1;
        }
        if self.dash_time < 0 {
            self.dash_time += 1;
        }
        let right_edge = input.right && !self.right_prev;
        let left_edge = input.left && !self.left_prev;
        if right_edge {
            if self.dash_time > 0 {
                // `:21578`: the dash start zeroes the dash-age counter.
                self.time_since_last_dash_started = 0;
                return Some((1, true));
            }
            self.dash_time = 15;
        } else if left_edge {
            if self.dash_time < 0 {
                // `:21593`: the dash start zeroes the dash-age counter.
                self.time_since_last_dash_started = 0;
                return Some((-1, true));
            }
            self.dash_time = -15;
        }
        None
    }

    /// `WorldGen.SolidOrSlopedTile` on the two launch-probe points
    /// (`Player.cs:21412-21414`): the hitbox's leading edge at the upper
    /// body and the center rows.
    fn solid_ahead(&self, world: &World, dir: i32) -> bool {
        let cx = self.x + self.width * 0.5;
        let cy = self.y + self.height * 0.5;
        let tx = ((cx + dir as f32 * (self.width * 0.5 + 2.0)) / TILE_PX) as i32;
        let ty0 = ((cy - self.height * 0.5 + 2.0) / TILE_PX) as i32;
        let ty1 = (cy / TILE_PX) as i32;
        [ty0, ty1].into_iter().any(|ty| {
            world.tile(tx, ty).is_some_and(|t| {
                t.active() && (tr_core::tiledata::TILE_SOLID.has(t.kind) || t.slope() != 0)
            })
        })
    }

    /// `Item.wingSlot` (`Item.cs`'s `SetDefaults` cases — the 47 wings whose
    /// `case` carries the assignment; the three that set it from a shared
    /// helper stay 0 for now). The port reads it where the C# reads
    /// `currentItem.wingSlot` (`Player.cs:15593-15602`).
    pub fn wing_slot(item: i32) -> i32 {
        match item {
            492 => 1,
            493 => 2,
            665 => 3,
            748 => 4,
            749 => 5,
            761 => 6,
            785 => 7,
            786 => 8,
            821 => 9,
            822 => 10,
            823 => 11,
            948 => 12,
            1162 => 13,
            1165 => 14,
            1515 => 15,
            1583 => 16,
            1584 => 17,
            1585 => 18,
            1586 => 19,
            1797 => 20,
            1830 => 21,
            1866 => 22,
            1871 => 23,
            2280 => 24,
            2494 => 25,
            2609 => 26,
            2770 => 27,
            3228 => 28,
            // `Item.cs:31096-31105`: the four Nebula wing cases share
            // `wingSlot = 29 + type - 3468` (S1-P-i40).
            3468 => 29,
            3469 => 30,
            3470 => 31,
            3471 => 32,
            3580 => 33,
            3582 => 34,
            3588 => 35,
            3592 => 36,
            3883 => 37,
            3924 => 38,
            3928 => 39,
            4730 => 40,
            4746 => 41,
            4750 => 42,
            4754 => 43,
            4823 => 44,
            4954 => 45,
            4978 => 46,
            5586 => 47,
            5627 => 48,
            5659 => 49,
            5686 => 50,
            6140 => 51,
            _ => 0,
        }
    }

    /// `ArmorIDs.Wing.Sets.Stats` via `WingStatsInitializer.cs:11-61`:
    /// `(fly_time, acc_run_override, accel_mult, has_hover, hover_speed,
    /// hover_accel)`. `-1` = no run-speed override. The dev/misc wings share
    /// the 150/7 config.
    pub fn wing_stats(id: i32) -> (i32, f32, f32, bool, f32, f32) {
        match id {
            46 => (25, 3.0, 1.0, false, -1.0, 1.0),
            1 | 2 => (100, 6.25, 1.0, false, -1.0, 1.0),
            6 | 7 | 10 | 25 => (130, 6.75, 1.0, false, -1.0, 1.0),
            4 => (150, 6.5, 1.0, false, -1.0, 1.0),
            5 | 9 | 13 | 14 | 15 => (160, 7.5, 1.0, false, -1.0, 1.0),
            8 | 11 | 24 | 27 => (170, 7.5, 1.0, false, -1.0, 1.0),
            22 => (170, 6.5, 1.0, true, 10.0, 10.0),
            12 | 20 | 21 | 23 => (180, 7.5, 1.0, false, -1.0, 1.0),
            26 => (180, 8.0, 2.0, false, -1.0, 1.0),
            37 => (150, 7.0, 2.5, true, 12.0, 12.0),
            44 => (150, 8.0, 2.0, false, -1.0, 1.0),
            45 => (180, 8.0, 4.5, true, 16.0, 16.0),
            29 | 32 => (180, 9.0, 2.5, false, -1.0, 1.0),
            30 | 31 => (180, 6.5, 1.5, true, 12.0, 12.0),
            // `WingStatsInitializer.cs:61`: the 150/7 chain covers
            // 3, 16-19, 28, 33-43 AND 47/50/51 (Chicken Bones/Kazzymodus/
            // Lunas) plus 48/49 (`:59-60`).
            3 | 16..=19 | 28 | 33..=43 | 47..=51 => (150, 7.0, 1.0, false, -1.0, 1.0),
            // `WingStats.cs:3`: the struct default is `flyTime = 100` —
            // `GetWingStats` (`Player.cs:14335-14338`) returns `default` for
            // an out-of-range id, NOT a zero budget (S1-P-i40).
            _ => (100, -1.0, 1.0, false, -1.0, 1.0),
        }
    }

    /// `WingMovement`'s per-tier constants (`Player.cs:22023-22243`):
    /// `(num, num2, num3, num4, num5)` — the ascent impulse, the falling
    /// brake, the upward cap multiplier, the rising brake threshold, the
    /// rising impulse.
    fn wing_constants(id: i32) -> (f32, f32, f32, f32, f32) {
        // The C# assigns the tier ifs in order (`:22062-22198`): the base
        // 0.1/0.5/1.5/0.5/0.1, then 26 (`:22067-22074`), the 1.66 cap group
        // (`:22075-22078`), the 1.805 cap group (`:22079-22082`), 37
        // (`:22083-22090`), 44 (`:22091-22097` — its OWN row: 0.125 impulse,
        // 0.85 brake, 2.75 cap, NOT the 45 numbers), 45 (`:22137-22142` —
        // keeps the 0.1 impulse), 29/32 (`:22182-22189`), 30/31
        // (`:22190-22198` — only num4/num3 here; the 0.15 impulse is
        // hover-down-gated at the call site, `:22194-22197`).
        match id {
            26 | 37 => (0.125, 0.75, 2.5, 1.0, 0.15),
            8 | 11 | 22 | 24 | 27 => (0.1, 0.5, 1.66, 0.5, 0.1),
            12 | 20 | 21 | 23 => (0.1, 0.5, 1.805, 0.5, 0.1),
            29 | 32 => (0.135, 0.85, 3.0, 1.0, 0.15),
            30 | 31 => (0.1, 0.5, 2.45, 1.0, 0.1),
            44 => (0.125, 0.85, 2.75, 1.0, 0.15),
            45 => (0.1, 0.95, 4.5, 1.0, 0.15),
            _ => (0.1, 0.5, 1.5, 0.5, 0.1),
        }
    }

    /// The hold-down hover wing set (`Player.cs:27405`, the dampening list).
    fn hover_dampening_wing(id: i32) -> bool {
        matches!(id, 22 | 28 | 30 | 31 | 33 | 35 | 37 | 45)
    }

    /// `Player.WingAirLogicTweaks` (`Player.cs:29304-29328`): the wing's run
    /// speed override and acceleration multiplier, the hover variants while
    /// hold-down-hovering, and the Long Trail Rainbow wings' (45) ×6
    /// run-slowdown once 60 ticks passed since the last dash. Runs before
    /// `HorizontalMovement`.
    fn wing_air_logic_tweaks(&mut self) {
        let (_fly, acc_override, accel_mult, has_hover, hover_speed, hover_accel) =
            Self::wing_stats(self.wings_logic);
        let hovering = self.trying_to_hover_down()
            && self.control_jump
            && self.wing_time > 0.0
            && !self.is_performing_down_dash();
        if has_hover && hovering {
            if hover_speed != -1.0 {
                self.acc_run_speed = hover_speed;
            }
            self.run_acceleration *= hover_accel;
        } else {
            if acc_override != -1.0 && acc_override > self.acc_run_speed {
                self.acc_run_speed = acc_override;
            }
            self.run_acceleration *= accel_mult;
        }
        // `:29324-29327`: the wings-45 dash-cooldown slowdown.
        if self.wings_logic == 45 && self.time_since_last_dash_started >= 60 {
            self.run_slowdown *= 6.0;
        }
    }

    /// `Player.TryingToHoverDown` (`Player.cs:3934-3941`): controlDown or the
    /// latch.
    fn trying_to_hover_down(&self) -> bool {
        self.control_down || self.try_keeping_hovering_down
    }

    /// `Player.TryingToHoverUp` (`Player.cs:3924-3933`): the up latch.
    fn trying_to_hover_up(&self) -> bool {
        self.try_keeping_hovering_up
    }

    /// The `isPerformingJump_DownDash` stand-in: the flag lands with the
    /// Dead-Cells dash (not in this task's scope), so it is always false.
    fn is_performing_down_dash(&self) -> bool {
        false
    }

    /// `Player.CCed` (`Player.cs:3377-3385`): frozen, webbed or stoned.
    pub fn cced(&self) -> bool {
        self.frozen || self.webbed || self.buffs.find_buff_index(156).is_some()
    }

    /// `Player.ClearBuff` (`Player.cs:5472-5481`): every slot whose
    /// `buffType == type` dels — NO `buffImmune` or `buffTime >= 1` gate
    /// (those are `FindBuffIndex`'s, `:5249-5255`; the old port looped
    /// `find_buff_index`, which skipped immune/expired slots the C# clears).
    fn clear_buff_type(&mut self, bt: i32) {
        self.buffs.clear_buff(bt);
    }

    /// `Mount.SetMount` (`Mount.cs:6196-6305`) via the player: the state
    /// swap, the buff clear/add (`:6203`/`:6226`), the swap resets
    /// (`:6205-6213`), the pirate-ship dash reset (`:6228-6235`), the
    /// PlayerSize-delegate resize (`:6244-6263` — Rat 14×14, Bat 20×18,
    /// Pixie 8×14) or the bottom-anchored `42 + heightBoost` growth
    /// (`:6264-6277`), and the Chillet sound queue (`:6283-6286`).
    pub fn set_mount(&mut self, m: i32, wet: bool) -> bool {
        let Some(equip) = self.mount.set_mount(m, wet) else {
            return false;
        };
        if let Some(old) = equip.clear_buff {
            self.clear_buff_type(old);
        }
        // `:6205-6213` — the swap branch: a track rider clears the cart flip
        // and the last boost, every mount clears the full rotation.
        if equip.was_active {
            if equip.track_rider {
                self.cart_flip = false;
                self.last_boost = (0, 0);
            }
            self.full_rotation = 0.0;
            self.full_rotation_origin = (0.0, 0.0);
        }
        self.buffs.add_buff(equip.add_buff, 3600);
        // `:6228-6235` — the pirate ship (44) drops the dash and damps the
        // velocity on mount.
        if m == 44 {
            self.vx *= 0.2;
            self.vy *= 0.2;
            self.dash = 0;
            self.dash_type = 0;
            self.dash_delay = 0;
            self.dash_time = 0;
        }
        if let Some((w, h)) = equip.size_override {
            // `:6244-6263` — the PlayerSize delegate branch: the bottom
            // point is captured BEFORE the resize, then the new box is
            // bottom-center anchored on it (`position = (bottom.X - w/2,
            // bottom.Y - h)`). The shadowPos loops have no port counterpart
            // (no advanced shadows) — the position maths is the observable.
            let bottom_x = self.x + self.width * 0.5;
            let bottom_y = self.y + self.height;
            self.width = w;
            self.height = h;
            self.x = bottom_x - w / 2.0;
            self.y = bottom_y - h;
        } else {
            // `:6264-6277` — the height-boost growth, bottom-anchored.
            self.grow_hitbox(HEIGHT + equip.height_boost as f32);
        }
        // `:6279-6282` — the UFO (7) and the drill (8) centre the full
        // rotation on the grown box (`width / 2, height / 2`).
        if m == 7 || m == 8 {
            self.full_rotation_origin = (self.width / 2.0, self.height / 2.0);
        }
        // `:6283-6286` — the Chillemt family (62-65) plays
        // `SoundID.PalChillet` at `mountedPlayer.Center`; the queue drains
        // on the tick's audio seam (the `pending_flap_sound` convention).
        if matches!(m, 62..=65) {
            let (cx, cy) = self.center();
            self.pending_chillet_sound = Some((cx, cy));
        }
        true
    }

    /// `:6270-6275`: the height change anchors the bottom (position.Y += h;
    /// height = new; position.Y -= h).
    fn grow_hitbox(&mut self, new_height: f32) {
        self.y += self.height;
        self.height = new_height;
        self.y -= self.height;
    }

    /// `Mount.Dismount` (`Mount.cs:6142-6194`) via the player: the buff clear
    /// (`:6150`), the transformation mounts' `noItems` restore (`:6152-6155`),
    /// the cart flip/boost clears (`:6156-6160`), the full-rotation reset
    /// (`:6161-6162`) and the 20×42 hitbox restore (`:6176-6193`).
    /// `ignore_effect` stands in for the spawn dust (`:6167-6170` — no dust
    /// system yet). The petting stop (`:6163-6166`) has no port counterpart.
    pub fn dismount(&mut self, world: &World, ignore_effect: bool) -> bool {
        // `:6148` — `bool cart = this.Cart` is read BEFORE the state clears
        // (`Mount.Cart` = `Minecart && _active`, `Mount.cs:530-540`).
        let was_cart = self.mount.is_active()
            && self
                .mount
                .data()
                .is_some_and(|d| d.minecart);
        let was_type = self.mount.kind();
        let Some(buff) = self.mount.dismount() else {
            return false;
        };
        let _ = ignore_effect;
        self.clear_buff_type(buff);
        // `:6152-6155` — the Rat (55) / Bat (56) transformation mounts free
        // the item lock unless Cursed keeps it.
        if matches!(was_type, 55 | 56) && self.no_items && !self.cursed {
            self.no_items = false;
        }
        // `:6156-6160` — a cart dismount clears the flip and the last boost.
        if was_cart {
            self.cart_flip = false;
            self.last_boost = (0, 0);
        }
        // `:6161-6162` — the dismount clears the full rotation.
        self.full_rotation = 0.0;
        self.full_rotation_origin = (0.0, 0.0);
        // `:6176-6193`: the restore — the 20x42 box from the bottom center;
        // on failure the Y patch accepts the overlap (the collision step
        // resolves it next tick, as in the original). The resize goes through
        // the single `TryChangingSizeFromBottomCenter` port (world-aware).
        // The C# restores BOTH width and height (`:6180-6181`/`:6190-6191`).
        if let Some([nx, ny]) = collision::try_changing_size_from_bottom_center(
            world,
            [self.x, self.y],
            [self.width, self.height],
            [WIDTH, HEIGHT],
        ) {
            self.x = nx;
            self.y = ny;
            self.width = WIDTH;
            self.height = HEIGHT;
        } else {
            self.y += self.height;
            self.width = WIDTH;
            self.height = HEIGHT;
            self.y -= self.height;
        }
        true
    }

    /// `Mount.CanDismountWithResult` (`Mount.cs:6611-6624`): the
    /// `DontDismountWhenCCed` gate ({55, 56, 61}, `MountID.cs:47`) refuses
    /// while CCed; otherwise the 20×42 shrink probe decides. A shrink always
    /// fits vertically, but the transformation mounts GROW in width on the
    /// restore (Pixie 8 → 20), so the probe can genuinely fail.
    pub fn can_dismount_with_result(&self, world: &World) -> crate::mount::DismountCheck {
        if crate::mount::dont_dismount_when_cced(self.mount.kind()) && self.cced() {
            return crate::mount::DismountCheck::FailedCCed;
        }
        if collision::try_changing_size_from_bottom_center(
            world,
            [self.x, self.y],
            [self.width, self.height],
            [WIDTH, HEIGHT],
        )
        .is_none()
        {
            return crate::mount::DismountCheck::FailedNoSpace;
        }
        crate::mount::DismountCheck::Succeeded
    }

    /// `Mount.TryDismount` (`Mount.cs:6127-6140`) → `TryDismountWithResult`:
    /// dismount only when `CanDismountWithResult` succeeds.
    pub fn try_dismount(&mut self, world: &World) -> bool {
        if self.can_dismount_with_result(world) == crate::mount::DismountCheck::Succeeded {
            self.dismount(world, false)
        } else {
            false
        }
    }

    /// `Mount.TryEarlyDismount` (`Mount.cs:6626-6632`): the item-use dismount
    /// — active, `dismountsOnItemUse` and the space check passes.
    pub fn try_early_dismount(&mut self, world: &World) {
        if self.mount.is_active()
            && self.mount.dismount_on_item_use()
            && self.can_dismount_with_result(world) == crate::mount::DismountCheck::Succeeded
        {
            self.dismount(world, false);
        }
    }

    /// `Mount.CanMount` (`Mount.cs:6581-6604`): the grown box must fit in
    /// space (`CanFitInSpaceWithSize` → `Collision.IsClearSpotTest`, the four
    /// cardinal 16-px sweeps), the bat (56) refuses water (`:6595`) and a
    /// grapple in flight blocks every mount outside `Cart`/`CanUseHooks`
    /// (`:6599-6601`). The PlayerSize delegate (Rat/Bat/Pixie) overrides the
    /// probe size (`:6585-6590`).
    pub(crate) fn can_mount(&self, world: &World, m: i32) -> bool {
        let Some(row) = crate::mount::mount_data(m) else {
            return false;
        };
        // `:6584` — `Size + (0, heightBoost)`; the delegate (`:6585-6590`)
        // replaces both components with its own size.
        let (size_w, size_h) = match row.player_size_override {
            Some((w, h)) => (w, h),
            None => (self.width, self.height + row.height_boost as f32),
        };
        // `Player.CanFitInSpaceWithSize` (`Player.cs:6108-6111`): the probe
        // top-left is `(Center.X - size.X/2, Bottom.Y - size.Y) + velocity`,
        // the four cardinal 16-px sweeps with fallThrough/fall2 true.
        let probe_x = self.x + self.width * 0.5 - size_w / 2.0 + self.vx;
        let probe_y = self.y + self.height - size_h + self.vy;
        if !collision::is_clear_spot_test(world, [probe_x, probe_y], 16.0, size_w, size_h, true, true)
        {
            return false;
        }
        // `:6595`: `Collision.WetCollision(Bottom - size * 0.5, size)` — the
        // probe is the sized box centred on the body's bottom point.
        if m == 56
            && (self.wet
                || self.dripping()
                || wet_collision(
                    world,
                    self.x + self.width * 0.5 - size_w / 2.0,
                    self.y + self.height - size_h / 2.0,
                    size_w,
                    size_h,
                ))
        {
            return false;
        }
        // `:6599`: `!Cart[m] && !CanUseHooks[m] && grappling[0] >= 0`.
        if !crate::mount::is_cart(m) && !crate::mount::can_use_hooks(m) && self.grappling_active {
            return false;
        }
        true
    }

    /// `Player.dripping` (`Player.cs:11743-11747`): buff 103 (`Wet`) keeps the
    /// player dripping until lava deletes it; the bat's wet gate reads it.
    fn dripping(&self) -> bool {
        self.holding(103) && !self.lava_wet
    }

    /// `WorldGen.InAPlaceWithWind(position, width, height)`
    /// (`WorldGen.cs:86841-86846`): the tile rectangle the body covers.
    fn wind_here(&self, world: &World, wind: Option<&tr_core::wind::Wind>) -> bool {
        let Some(wind) = wind else {
            return false;
        };
        let x0 = (self.x / TILE_PX) as i32;
        let y0 = (self.y / TILE_PX) as i32;
        let x1 = ((self.x + self.width) / TILE_PX) as i32;
        let y1 = ((self.y + self.height) / TILE_PX) as i32;
        wind.in_a_place_with_wind(world, x0, y0, 1 + x1 - x0, 1 + y1 - y0)
    }

    /// `Player.MountFishronSpecial` (`Player.cs:4044-4056`): full run stats for
    /// the Cute Fishron (12) while hurt, not in plain water (lava/honey are
    /// exempt), dripping, inside the 60/420-tick window, or under rain that
    /// reaches the body.
    fn mount_fishron_special(
        &self,
        world: &World,
        raining: bool,
        wind: Option<&tr_core::wind::Wind>,
    ) -> bool {
        if self.hp >= self.max_hp / 2
            && (!self.wet || self.lava_wet || self.honey_wet)
            && !self.dripping()
            && !(self.mount_fishron_special_counter > 0.0)
        {
            return raining && self.wind_here(world, wind);
        }
        true
    }

    /// `Player.ItemCheck_CheckCanUse_Inner`'s noItems gate (`Player.cs:52140-
    /// 52148`): `flag3 = ignoreCursed`, widened when a dismounts-on-item-use
    /// mount is up, can dismount, and the player is Cursed-lock free; the
    /// `noItems && !flag3` pair refuses the use. The mount item-use arm and
    /// `QuickMount`'s `ItemCheck_TryStartUse` both ride it.
    pub(crate) fn can_use_item_past_no_items(&self, world: &World, ignore_cursed: bool) -> bool {
        let mut flag3 = ignore_cursed;
        if self.mount.is_active()
            && self.mount.dismount_on_item_use()
            && self.can_dismount_with_result(world) == crate::mount::DismountCheck::Succeeded
            && self.no_items
            && !self.cursed
        {
            flag3 = true;
        }
        !(self.no_items && !flag3)
    }

    /// `Player.QuickMount` (`Player.cs:6045-6100`): the toggle — dismount
    /// (with the FailedNoSpace 3-strike ladder, `:6049-6070`), or equip the
    /// first non-cart saddle (`QuickMount_GetItemToUse` `:6179-6198`: the
    /// `miscEquips[3]` slot first, then the inventory). The `QuickMinecartSnap`
    /// track-hook branch (`:6081`, `:6145-6177`) and the `QuickMinecart`
    /// fallback (`:6097`) are unported (M2-MOUNT-3 / grapple) — constants-off,
    /// so `flag` stays true. `trying_to_use_item` is the caller's
    /// `TryingToUseItem()` read (`:6082`); the `DismountsOnItemUse` mount
    /// refuses to equip while an item use is in flight. Returns the saddle
    /// item kind used (for the use animation).
    pub fn quick_mount(
        &mut self,
        world: &World,
        inventory: &[crate::item::ItemSlot],
        misc_equips: &[crate::item::ItemSlot],
        trying_to_use_item: bool,
    ) -> Option<u16> {
        if self.mount.is_active() {
            // `:6049-6052`: a successful (or CCed-refused) dismount returns;
            // only FailedNoSpace falls into the ladder.
            let result = self.can_dismount_with_result(world);
            if result == crate::mount::DismountCheck::Succeeded {
                self.dismount(world, false);
                return None;
            }
            if result != crate::mount::DismountCheck::FailedNoSpace {
                return None;
            }
            // `:6053-6060`: the no-space ladder — record the tile, count the
            // strike; the first two play the tink + dust and keep the mount.
            self.failed_no_space_location =
                ((self.x / TILE_PX) as i32, (self.y / TILE_PX) as i32);
            self.failed_no_space_count += 1;
            if self.failed_no_space_count < 3 {
                // `:6057` — `PlaySound(0, (int)position.X, (int)position.Y)`;
                // the case-0 Dig draws ride the drain's stream. The
                // `DoFailedDismountDust` (`:6058`) is a visual seam (no dust
                // request channel for mounts yet — ANALYSIS).
                self.pending_no_space_sound = Some((self.x.trunc(), self.y.trunc()));
                return None;
            }
            // `:6061-6066`: the third strike forces the dismount
            // (ignoreEffect) and teleports to the nearest open spot.
            self.dismount(world, true);
            self.failed_no_space_count = 0;
            self.pending_no_space_teleport = true;
            return None;
        }
        // `:6074-6077`: CCed / tongued / gravDir==-1 / dead / noItems refuse.
        // The port keeps gravDir = 1 (no flip), so that arm folds out.
        if self.cced() || self.tongued || self.dead || self.no_items {
            return None;
        }
        // `QuickMount_GetItemToUse` (`:6179-6198`): miscEquips[3] first (a
        // non-cart mount), then the inventory scan skipping cart rows.
        let pick = misc_equips
            .get(3)
            .filter(|s| s.count > 0)
            .and_then(|s| crate::mount::mount_type_of_item(s.kind).map(|m| (s.kind, m)))
            .filter(|(_, m)| !crate::mount::is_cart(*m))
            .or_else(|| {
                inventory.iter().find_map(|s| {
                    if s.count == 0 {
                        return None;
                    }
                    let m = crate::mount::mount_type_of_item(s.kind)?;
                    if crate::mount::is_cart(m) {
                        return None;
                    }
                    Some((s.kind, m))
                })
            });
        let Some((kind, m)) = pick else {
            return None; // `:6097` `QuickMinecart` — M2-MOUNT-3
        };
        // `:6079`: `CanMount && ItemCheck_TryStartUse`. For a saddle the
        // TryStartUse cascade reduces to the noItems gate (no mana/potion/
        // shoot) — the full cascade rides the editing.rs use path.
        if !self.can_mount(world, m) || !self.can_use_item_past_no_items(world, false) {
            return None;
        }
        // `:6081-6085`: `flag = !QuickMinecartSnap()` (unported → true); a
        // dismounts-on-item-use mount refuses while an item use is in flight.
        let mut flag = true;
        if crate::mount::mount_data(m).is_some_and(|d| d.dismounts_on_item_use)
            && trying_to_use_item
        {
            flag = false;
        }
        if flag && self.set_mount(m, self.wet) {
            // `:6089-6092` — `item.UseSound` plays at `base.Center`; the
            // caller's use-animation seam plays it (`start_swing_extras`).
            Some(kind)
        } else {
            None
        }
    }

    /// `Player.UpdateFailedDismount` (`Player.cs:6037-6043`): the no-space
    /// strike count clears on death, a dismount, or moving off the recorded
    /// tile. Called from the player tick (`:24728`).
    pub fn update_failed_dismount(&mut self) {
        if self.failed_no_space_count > 0
            && (self.dead
                || !self.mount.is_active()
                || self.failed_no_space_location
                    != ((self.x / TILE_PX) as i32, (self.y / TILE_PX) as i32))
        {
            self.failed_no_space_count = 0;
        }
    }

    /// `Player.PlayerNoSpaceTeleport` (`Player.cs:6019-6035`): the third
    /// no-space strike — find the closest open spot and teleport there
    /// (style 11), else fall back to the world spawn (`Shellphone_Spawn`).
    /// The net sends (`:6025-6029`) have no port counterpart.
    /// The invalid-area force-clear (`Spawn_ForceClearArea`) rewrites tiles
    /// and rolls drops, so it rides the pending-request convention — the
    /// drain (`world_sim/player.rs`) runs `editing::spawn_force_clear_area`
    /// the same tick; the teleport reads no tiles, so the deferral is
    /// observation-identical.
    pub fn player_no_space_teleport(&mut self, world: &World) {
        self.failed_no_space_count = 0;
        if let Some((nx, ny)) = find_closest_teleport_spot_no_space(world, self) {
            self.teleport_style(nx, ny, 11);
        } else {
            self.shellphone_spawn(world);
        }
    }

    /// `Player.Teleport` (`Player.cs:37414-37496`) — the movement-relevant
    /// core: `RemoveAllGrapplingHooks` (style != 10), `StopVanityActions`,
    /// the shimmer strip, the position write, the fall-origin reset and the
    /// `oldPosition` sync. The camera/effect/map halves are render seams.
    /// The velocity is NOT zeroed here — the C# scan zeroes it before the
    /// call (`TeleportHelpers.cs:12`) and `Shellphone_Spawn` after (`:56175`).
    pub fn teleport_style(&mut self, new_x: f32, new_y: f32, style: i32) {
        // `:37420-37423` — every style but 10 removes the hooks.
        if style != 10 {
            self.release_grappling_hooks = true;
            self.grappling_active = false;
        }
        // `:37424` — `StopVanityActions` (the petting stop is unported).
        self.sit_up();
        self.stop_sleeping();
        // `:37425-37431` — the shimmering / shimmerWet strip. The port has no
        // `Player.shimmering` (the absorbed state, S1-E-16), so only the
        // ported `shimmer_wet` half fires; the `shimmering = false` write and
        // the buff-353 clear ride that state's own task (ANALYSIS).
        if self.shimmer_wet {
            self.shimmer_wet = false;
            self.wet = false;
            self.clear_buff_type(353);
        }
        // `:37446`/`:37455`/`:37488` — position, fall origin, oldPosition.
        self.x = new_x;
        self.y = new_y;
        self.fall_start = (new_y / TILE_PX) as i32;
        self.prev_x = new_x;
        self.prev_y = new_y;
    }

    /// `Player.Shellphone_Spawn` (`Player.cs:56164-56181`): the world-spawn
    /// fallback — `Spawn_GetPositionAtWorldSpawn` finds the floor, the area
    /// is force-cleared when invalid, then the teleport (style 11) and the
    /// velocity zero. The port's `World` spawn fields are the tile coords;
    /// the `netMode != 1` gate folds to always-clear (SP only). The
    /// force-clear rewrites tiles + rolls drops, so it queues as a pending
    /// request (`pending_force_clear`) — the drain runs it the same tick.
    pub fn shellphone_spawn(&mut self, world: &World) {
        let (floor_x, floor_y) =
            spawn_get_position_at_world_spawn(world, world.spawn_x, world.spawn_y);
        // `:56169-56172` — the invalid area force-clears BEFORE the teleport.
        if !spawn_is_area_valid_spawn(world, floor_x, floor_y) {
            self.pending_force_clear = Some((floor_x, floor_y));
        }
        // `:56173` — `ToWorldCoordinates(8, 0) - (width/2, height)`.
        let new_x = floor_x as f32 * TILE_PX + 8.0 - self.width / 2.0;
        let new_y = floor_y as f32 * TILE_PX - self.height;
        self.teleport_style(new_x, new_y, 11);
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// `Player.UsingSuperCart` (`Player.cs:3966-3980`): the Super Cart counts
    /// only when it is both unlocked and enabled.
    pub fn using_super_cart(&self) -> bool {
        self.unlocked_super_cart && self.enabled_super_cart
    }

    /// The per-tick mount block — `Player.cs:26623-26688` with the
    /// `UpdateEffects` head (`Mount.cs:4795-4803`, `:4949-5008`) and the
    /// `UpdateJumpHeight` mount merge (`:26011` → `:19619-19637`). Vanilla
    /// runs those at :25656/:26011, both before the run-speed merge — the
    /// combined method keeps their relative order. Only reachable while not
    /// grappling (the caller's `:26622` branch).
    fn update_mount(&mut self, world: &World, raining: bool, wind: Option<&tr_core::wind::Wind>) {
        if !self.mount.is_active() {
            return;
        }
        // `Mount.cs:4797`: the mount head-sets the auto-jump flag per tick.
        self.auto_jump = self.mount.auto_jump();
        // `Mount.UpdateEffects` (`Mount.cs:4798`): the Super Cart flag rides
        // the cart set (`MountID.Sets.Cart`) and the player's unlock/enable
        // pair (`Player.UsingSuperCart`, `Player.cs:3966-3980`).
        self.mount.set_should_super_cart(
            crate::mount::is_cart(self.mount.kind()) && self.using_super_cart(),
        );
        // `Mount.UpdateEffects` (`Mount.cs:4806-4814`): the Chillemt mounts
        // add 0.1 to every damage class. The vanilla applies this after the
        // stat recompute (`Player.ResetEffects`), which is the port's order
        // too — `update_mount` runs after the equip pass.
        if matches!(self.mount.kind(), 62..=65) {
            self.melee_damage += 0.1;
            self.ranged_damage += 0.1;
            self.magic_damage += 0.1;
            self.minion_damage += 0.1;
        }
        // `Mount.cs:4982-5008` (`UpdateEffects` case 12): the Cute Fishron
        // re-arms its special window when hurt or standing in water/rain.
        if self.mount.kind() == 12 {
            // `:4983-5000` — while `MountFishronSpecial`, every damage class
            // gains 0.15 (the AddLight halves are the draw's light seam).
            if self.mount_fishron_special(world, raining, wind) {
                self.melee_damage += 0.15;
                self.ranged_damage += 0.15;
                self.magic_damage += 0.15;
                self.minion_damage += 0.15;
            }
            if self.hp <= self.max_hp / 2 {
                self.mount_fishron_special_counter = 60.0;
            }
            if self.wet || (raining && self.wind_here(world, wind)) {
                self.mount_fishron_special_counter = 420.0;
            }
        }
        // `Mount.cs:5137-5145` (`UpdateEffects` cases 55/56/61): the
        // transformation mounts lock item use every tick
        // (`IsAllowedToHoldItems = false` rides the draw's
        // `mounted_blocks_hold`; the Pixie's dust tail `:5146-5160` is the
        // visual seam).
        if matches!(self.mount.kind(), 55 | 56 | 61) {
            self.no_items = true;
        }
        // `Mount.cs:4949-4981`: the ground-dash mounts arm their extra jumps
        // (the PHYS-8 priority chain consumes them) and pin knockback off
        // above the charge speed. The Santank's (46) jump rides the
        // ability case's head (`:4882-4884`), before the target scan.
        match self.mount.kind() {
            10 => self.has_jump_options[ExtraJump::Unicorn.index()] = true,
            46 => self.has_jump_options[ExtraJump::Santank.index()] = true,
            47 => self.has_jump_options[ExtraJump::WallOfFleshGoat.index()] = true,
            14 => self.has_jump_options[ExtraJump::Basilisk.index()] = true,
            _ => {}
        }
        if matches!(self.mount.kind(), 10 | 47 | 14 | 40 | 41 | 42)
            && self.vx.abs() > self.mount.dash_speed() - self.mount.run_speed() / 2.0
        {
            self.no_knockback = true;
        }
        // `Player.cs:19619-19631`: the jump statics — replace, or additive
        // when the mount says so; `:19635-19637` the sticky clamp after (the
        // `dazed` clamp `:19638-19641` has no port counterpart).
        if self.mount.movement_stats_are_additive() {
            self.jump_height += self.mount.jump_height(self.vx);
            self.jump_speed += self.mount.jump_speed(self.vx);
        } else {
            self.jump_height = self.mount.jump_height(self.vx);
            self.jump_speed = self.mount.jump_speed(self.vx);
        }
        if self.sticky {
            self.jump_height /= 10;
            self.jump_speed /= 5.0;
        }
        // `:26625-26635`: the wings/rocket suppression + the slowdown floor.
        if !self.mount.can_use_wings() {
            self.run_slowdown = 0.2;
        }
        self.rocket_boots = 0;
        self.vanity_rocket_boots = 0; // `:26630` (W4-c14 doc-sweep)
        if !self.mount.can_use_wings() {
            self.has_wings = false;
            self.wings_logic = 0;
        }
        // `:26636` + `:26637-26654`: the three-way run-stat merge.
        let flag17 = self.mount.can_use_wings() && self.wings_logic > 0 && self.vy != 0.0;
        if self.mount.movement_stats_are_additive() {
            self.max_run_speed += self.mount.run_speed();
            self.acc_run_speed += self.mount.dash_speed();
            self.run_acceleration += self.mount.acceleration();
        } else if flag17 {
            self.max_run_speed = self.max_run_speed.max(self.mount.run_speed());
            self.acc_run_speed = self.acc_run_speed.max(self.mount.dash_speed());
            self.run_acceleration = self.run_acceleration.max(self.mount.acceleration());
        } else {
            self.max_run_speed = self.mount.run_speed();
            self.acc_run_speed = self.mount.dash_speed();
            self.run_acceleration = self.mount.acceleration();
        }
        // `:26655-26659`: the Cute Fishron runs at half acceleration/top
        // speed unless `MountFishronSpecial` is true.
        if self.mount.kind() == 12 && !self.mount_fishron_special(world, raining, wind) {
            self.run_acceleration /= 2.0;
            self.max_run_speed /= 2.0;
        }
        // `:26660-26663` (the roller skates), `:26665-26682` (the cart/OnTrack
        // block — M2-MOUNT-3) and `:26683-26686` (`UpdateDrill` —
        // M2-MOUNT-4) have no rows in this table yet; the ability counters
        // still tick (`:26663`).
        self.mount.ability_recovery();
    }

    /// The flight section (`Player.cs:26745-27359`): the budget zeroing,
    /// `canRocket`, the ground refills, `flag19`, the CCed arm, `WingMovement`,
    /// `WingFrame`, the rocket→wing conversion, the flap edge, the rocket
    /// trigger + thrust, and the !flag19 else-if chain (hover mount, flying
    /// mount, Featherfall, wing glide). Returns true when it integrated vy
    /// itself (the plain gravity is skipped this tick, like the original's
    /// else-if chain). The post-chain fall-cap block, the `wingFrame = 0`
    /// mount gate and the hover dampening (`:27361-27425`) run in the caller
    /// AFTER the plain-gravity fallback, in the C# order.
    fn flight_section(&mut self, gravity: f32, fall_cap: f32) -> bool {
        // `:26745-26752`: an unequipped tier zeroes its budget.
        if self.wings_logic == 0 {
            self.wing_time = 0.0;
        }
        if self.rocket_boots == 0 {
            self.rocket_time = 0;
        }
        // `:26768-26771`: wings or a mount cancel the sandstorm hover.
        if self.wings_logic > 0 || self.mount.is_active() {
            self.sand_storm = false;
        }
        // `:26772-26775`: airborne outside the initial jump burst.
        if self.vy > -self.jump_speed && self.vy != 0.0 {
            self.can_rocket = true;
        }
        // `:26776-26781`: the wing refill on the ground needs a jump release —
        // or an auto-jump (constant-jump) mount re-jumping this tick; the
        // SLIDING state counts as resting too (`this.sliding`, `:26777`); the
        // mount flight budget refills with it (`mount.ResetFlightTime`).
        if ((self.vy == 0.0 || self.sliding) && self.release_jump)
            || (self.auto_jump && self.just_jumped)
        {
            self.wing_time = self.wing_time_max as f32;
            self.mount.reset_flight_time(self.vx, self.wing_time_max);
        }
        // `:26782-26785`: holding jump with wings levels the tilt.
        if self.wings_logic > 0 && self.control_jump {
            self.full_rotation = 0.0;
        }
        // `:26786-26793`: flag19 — actively flying this tick. The second
        // condition is the hold-down-hover extension for the hover wings
        // (the set includes 29/32 but NOT 31 — the C#'s own list).
        let mut flag19 = self.wings_logic > 0
            && self.control_jump
            && self.wing_time > 0.0
            && self.jump == 0
            && self.vy != 0.0;
        if self.wings_logic > 0
            && self.control_jump
            && self.trying_to_hover_down()
            && self.wing_time > 0.0
            && matches!(self.wings_logic, 22 | 28 | 30 | 32 | 29 | 33 | 35 | 37 | 45)
        {
            flag19 = true;
        }
        // `:26794-26798`: flag20 (the Dead-Cells down-dash) cancels flag19 —
        // `is_performing_down_dash` is always false in the port (the dash is
        // unported), so the cancel never fires.
        if self.is_performing_down_dash() {
            flag19 = false;
        }
        // `:26799-26812`: the CCed arm — the mount dismounts (the caller's
        // :26801-26804 site), plain gravity + the fall cap, no flight at all
        // (the rocket thrust included). The visual-effect cancels are seams.
        if self.cced() {
            self.sand_storm = false;
            self.vy += gravity;
            if self.vy > fall_cap {
                self.vy = fall_cap;
            }
            return true;
        }
        // `:26815-26818`: the flying tick — `WingAirVisuals` (the per-wing
        // dust rolls, `:30750-30963`) is a visual seam the port defers (its
        // `Main.rand` draws would ride the UpdatePlayers stream; see
        // docs/ANALYSIS.md "The wings/flight accessory block").
        if flag19 {
            self.wing_movement();
        }
        // S1-E-21f - `WingFrame(flag19)` (`Player.cs:26820`, `:29468-30527`).
        // W4-c14 - the wings-4 (jetpack) sound arm (`:29471-29480`): while
        // flapping or mid-jump the rocketDelay2 cadence plays Item13 every
        // 60 ticks (the frame table + the dust are visual seams). The C#
        // decrements INSIDE the arm; the `:26907` per-tick decrement runs
        // again below - both, verbatim.
        if self.wings_logic == 4 && (flag19 || self.jump > 0) {
            self.rocket_delay2 -= 1;
            if self.rocket_delay2 <= 0 {
                self.pending_cart_sounds.push(crate::minecart::CartCue {
                    cue: "Item_13",
                    x: self.x,
                    y: self.y,
                });
                self.rocket_delay2 = 60;
            }
        }
        // W4-c14 - the wings-45 (Celestial Starboard) sound arm
        // (`:30165-30174`): `ShouldDrawWingsThatAreAlwaysAnimated`
        // (`:30737-30748`: airborne, ungrappled, not water-floating, the
        // mount allows wings) + budget left: Item24 every 30 ticks.
        if self.wings_logic == 45
            && self.vy != 0.0
            && !self.grappling_active
            && (!self.wet || !self.should_float_in_water())
            && self.mount.can_use_wings()
            && self.wing_time > 0.0
        {
            self.rocket_delay2 -= 1;
            if self.rocket_delay2 <= 0 {
                self.pending_cart_sounds.push(crate::minecart::CartCue {
                    cue: "Item_24",
                    x: self.x,
                    y: self.y,
                });
                self.rocket_delay2 = 30;
            }
        }
        // The port runs the DEFAULT arm's frame machine (`:30448-30526`:
        // num35 = 4, num36 = 4, num37 = 0, num38 = 0, num39 = 1 — the
        // wings-43/48/32 constants and the hover wings' own tables are visual
        // seams): counter > 4 wraps the frame 0..3, the airborne rest frame 1,
        // the grounded 0. The machine sits behind the C#'s exclusion gate
        // (`:26835`: wings 4/22/0/24/28/30/33/45 — the early-return arms own
        // their frames). The `wingFrame == 3` flap edge (`:26835-26858`) runs
        // AFTER the rocket→wing conversion below, in the C# order; wings
        // 43/44 read frame 4 and 49 reads frame 5 — their frame tables are
        // unported, so their flap arms stay silent (the documented deviation).
        if !matches!(self.wings_logic, 4 | 22 | 0 | 24 | 28 | 30 | 33 | 45) {
            if flag19 || self.jump > 0 {
                self.wing_frame_counter += 1;
                if self.wing_frame_counter > 4 {
                    self.wing_frame += 1;
                    self.wing_frame_counter = 0;
                    if self.wing_frame >= 4 {
                        self.wing_frame = 0;
                    }
                }
            } else if self.vy != 0.0 {
                // `:30482-30484` — the airborne rest frame (num39 = 1).
                self.wing_frame = 1;
            } else {
                // `:30523-30526` — the grounded rest frame (num38 = 0).
                self.wing_frame = 0;
            }
        }
        // `:26825-26834`: with both equipped the rocket fuel converts into
        // wing time (capped past the max) — AFTER WingMovement drained this
        // tick's point, in the C# order.
        if self.wings_logic > 0 && self.rocket_boots != 0 && self.vy != 0.0 && self.rocket_time != 0
        {
            let add = self.rocket_time * 6;
            self.wing_time += add as f32;
            if self.wing_time > (self.wing_time_max + add) as f32 {
                self.wing_time = (self.wing_time_max + add) as f32;
            }
            self.rocket_time = 0;
        }
        // `:26835-26858`: the flap-sound edge — `wingFrame == 3` plays
        // `SoundID.Item32` once through the `flapSound` latch; the façade
        // draw rides the drain.
        if flag19 && !matches!(self.wings_logic, 4 | 22 | 0 | 24 | 28 | 30 | 33 | 45) {
            if self.wing_frame == 3 {
                if !self.flap_sound {
                    if let Some(idx) = crate::soundid::SOUND_STYLES
                        .iter()
                        .position(|s| s.name == "Item32")
                    {
                        self.pending_flap_sound = Some((idx as u16, self.x, self.y));
                    }
                }
                self.flap_sound = true;
            } else {
                self.flap_sound = false;
            }
        }
        // `:26859-26862`: the rocket fuel refills on any rest tick (sliding
        // counts) or an auto-jump mount's re-jump.
        if self.vy == 0.0 || self.sliding || (self.auto_jump && self.just_jumped) {
            self.rocket_time = self.rocket_time_max;
        }
        // `:26863-26866`: the Soaring Insignia refills the fuel every tick.
        if self.empress_brooch {
            self.rocket_time = self.rocket_time_max;
        }
        // `:26867-26902`: the rocket trigger — only once the wing budget is
        // spent (or absent); the arming needs a midair jump release and
        // `CanUseBootFlyingAbilities` (`:3988`: !down-dash, always true here).
        if (self.wing_time == 0.0 || self.wings_logic == 0)
            && self.rocket_boots != 0
            && self.control_jump
            && self.rocket_delay == 0
            && self.can_rocket
            && self.rocket_release
            && !self.is_performing_down_dash()
        {
            if self.rocket_time > 0 {
                self.rocket_time -= 1;
                self.rocket_delay = 10;
                if self.rocket_delay2 <= 0 {
                    // `:26875-26882`: tier 1 cadence 30, tiers 2/3/4/5 15.
                    self.rocket_delay2 = if self.rocket_boots == 1 { 30 } else { 15 };
                }
                // W4-c14 - `:26884-26896`: the rocketSoundDelay cue - the
                // VANITY tier picks the cue (1/5 Item13 cadence 30,
                // 2/3/4/6 Item24 cadence 15). W4-c14 doc-sweep: the vanity
                // producer lands (`vanity_rocket_boots` - the equip rows set
                // both tiers equal, `Player.cs:14420`/`:15023`/`:15222`/
                // `:15380-15400`, the vanity pass `UpdateBootVisualEffects`
                // `:36704-36751` re-derives it, the Red's Leggings 668 sets
                // tier 6 `:13088-13091`), so the cue reads the vanity tier
                // like the C#. The play queues on the movement-cue drain
                // (the Item55 convention).
                if self.rocket_sound_delay <= 0 {
                    let cue = match self.vanity_rocket_boots {
                        1 | 5 => Some(("Item_13", 30)),
                        2 | 3 | 4 | 6 => Some(("Item_24", 15)),
                        _ => None,
                    };
                    if let Some((cue, delay)) = cue {
                        self.rocket_sound_delay = delay;
                        self.pending_cart_sounds.push(crate::minecart::CartCue {
                            cue,
                            x: self.x,
                            y: self.y,
                        });
                    }
                }
            } else {
                self.can_rocket = false;
            }
        }
        // W4-c14 - `:26903-26905`: the rocket cue cadence ticks once per
        // flight tick, before the flame cadence below.
        if self.rocket_sound_delay > 0 {
            self.rocket_sound_delay -= 1;
        }
        // `:26907-26910`: the flame cadence delay ticks every tick.
        if self.rocket_delay2 > 0 {
            self.rocket_delay2 -= 1;
        }
        // `:26915-26955`: the rocket thrust — 10 ticks per fuel tick. The
        // fall origin refreshes via the caller's `:25379-25382` block (the
        // `rocketDelay > 0` term) — the port keeps its own refresh here so
        // the last thrust tick (delay decremented to 0) still resets it.
        if self.rocket_delay > 0 {
            self.rocket_delay -= 1;
            self.vy -= 0.1;
            if self.vy > 0.0 {
                self.vy -= 0.5;
            } else if self.vy > -self.jump_speed * 0.5 {
                self.vy -= 0.1;
            }
            if self.vy < -self.jump_speed * 1.5 {
                self.vy = -self.jump_speed * 1.5;
            }
            self.fall_start = (self.y / TILE_PX) as i32;
            return true;
        }
        if !flag19 {
            // `:26958-26961`: the hover mounts (`Mount.Hover`, `Mount.cs:3340`)
            // — the branch carries the vertical move itself, so it is
            // consumed like the vanilla `else if` chain.
            if self.mount.can_hover() {
                self.vy = self.mount.hover_integrate(
                    self.vy,
                    false, // no `controlUp` key ported (M2-MOUNT-4)
                    self.control_jump,
                    self.control_down,
                    gravity,
                );
                // `Mount.cs:3479`: the fall origin refreshes every hover tick
                // (no fall damage accrues while hovering).
                self.fall_start = (self.y / TILE_PX) as i32;
                return true;
            }
            // `:26962-27020`: the mount flight thrust — the flying mounts'
            // replacement for the wing ascent while the budget lasts.
            if self.mount.can_fly() && self.control_jump && self.jump == 0 {
                if self.mount.flight() {
                    if self.trying_to_hover_down() {
                        // `:26966-26973`: the hover-down dampening.
                        self.vy *= 0.9;
                        if self.vy > -1.0 && self.vy < 0.5 {
                            self.vy = 1e-5;
                        }
                    } else {
                        // `:26975-27003`: the rise thrust — the falling
                        // brake, the rising assist, the upward cap.
                        let mut num40 = 0.1;
                        let mut num41 = self.jump_speed;
                        if self.mount.kind() == 50 {
                            num41 *= 0.5;
                        }
                        // `:26982-26985`: the Bat/Pixie rise cap is 2/3.
                        if matches!(self.mount.kind(), 56 | 61) {
                            num41 /= 1.5;
                        }
                        // `:26986-26991`: the Velociraptor (54) borrows the
                        // equipped wing's run-speed override / 1.5 as its cap
                        // and scales the thrust by the wing's accel mult.
                        if self.mount.kind() == 54 && self.wings_logic > 0 {
                            let stats = Self::wing_stats(self.wings_logic);
                            num41 = stats.1 / 1.5;
                            num40 *= stats.2;
                        }
                        if self.vy > 0.0 {
                            self.vy -= num40 * 5.0;
                        } else if self.vy > -num41 * 1.5 {
                            self.vy -= num40;
                        }
                        if self.vy < -num41 * 1.5 {
                            self.vy = -num41 * 1.5;
                        }
                    }
                    return true;
                }
                // `:27006-27019`: the budget spent — gravity / 3 with the
                // fall cap / 3 unless hover-down (the gravDir -1 mirror is
                // unported).
                self.vy += gravity / 3.0;
                if self.vy > fall_cap / 3.0 && !self.trying_to_hover_down() {
                    self.vy = fall_cap / 3.0;
                }
                return true;
            }
            // `:27022-27033`: Featherfall's airborne arm — gravity /3 (or
            // /10 with the hover-up latch), applied BEFORE the wing glide
            // in the C# else-if chain.
            if self.slow_fall
                && !self.trying_to_hover_down()
                && !self.is_performing_down_dash()
            {
                let g = if self.trying_to_hover_up() {
                    gravity / 10.0
                } else {
                    gravity / 3.0
                };
                self.vy += g;
                return true;
            }
            // `:27034-27350`: the wing glide — jump held while falling: the
            // fall origin refreshes per tick (`:27037`, no fall damage
            // accrues), the per-wing glide dust rolls (`:27040-27337`) are
            // visual seams, then gravity / 3 and the fall cap / 3 — the cap
            // SKIPS while hover-down (`:27342`, the dampening owns the float).
            if self.wings_logic > 0
                && self.control_jump
                && self.vy > 0.0
                && !self.is_performing_down_dash()
            {
                self.fall_start = (self.y / TILE_PX) as i32;
                // W4-c14 - the wings-4 glide sound (`Player.cs:27149-27156`,
                // inside the `velocity.Y > 0` dust block): the rocketDelay2
                // cadence plays Item13 every 60 ticks while gliding (the
                // per-wing dust rolls stay visual seams).
                if self.wings_logic == 4 {
                    self.rocket_delay2 -= 1;
                    if self.rocket_delay2 <= 0 {
                        self.pending_cart_sounds.push(crate::minecart::CartCue {
                            cue: "Item_13",
                            x: self.x,
                            y: self.y,
                        });
                        self.rocket_delay2 = 60;
                    }
                }
                self.vy += gravity / 3.0;
                if self.vy > fall_cap / 3.0 && !self.trying_to_hover_down() {
                    self.vy = fall_cap / 3.0;
                }
                return true;
            }
            return false;
        }
        true
    }

    /// `Player.WingMovement` (`Player.cs:22023-22243`): the ascent physics —
    /// the impulse, the falling brake, the rising brake, the upward cap, the
    /// budget countdown (half while hover-down with no horizontal input, 2x
    /// for the jetpack hover-up), the wings-44/45 hover thrusts and the
    /// Empress Brooch's infinite flight (`:22239-22242`).
    fn wing_movement(&mut self) {
        let (mut num, num2, num3, num4, num5) = Self::wing_constants(self.wings_logic);
        // `:22230`: the half-drain reads the CONTROL fields (post-Confused
        // swap), not the raw input.
        let hover_down =
            self.trying_to_hover_down() && !self.control_left && !self.control_right;
        if self.wings_logic == 4 && self.trying_to_hover_up() {
            // `:22025-22058`: the jetpack hover-up — the 0.2 impulse, the
            // FULL 1.0 falling brake (`:22032` `velocity.Y--`), the 0.2
            // rising assist above `-jumpSpeed` (`:22034`, the full static,
            // not half), the 3x cap — burns 2 ticks per tick. The arm
            // RETURNS out of the shared ascent (the C# else at `:22060`).
            self.vy -= 0.2;
            if self.vy > 0.0 {
                self.vy -= 1.0;
            } else if self.vy > -self.jump_speed {
                self.vy -= 0.2;
            }
            if self.vy < -self.jump_speed * 3.0 {
                self.vy = -self.jump_speed * 3.0;
            }
            self.wing_time -= 2.0;
            // `:22239-22242`: the brooch refill runs for the jetpack arm too.
            if self.empress_brooch && self.wing_time != 0.0 {
                self.wing_time = self.wing_time_max as f32;
            }
            return;
        }
        // `:22098-22135` (44) / `:22143-22180` (45): the hover thrusts run
        // INSIDE the tier arms, BEFORE the shared ascent (which still
        // applies after them). The falling brake is the FULL 1.0
        // (`:22105`/`:22150` `velocity.Y--`), the rising assist 0.2 above
        // `-jumpSpeed`, the cap 3x — all on the per-tick rebased
        // `Player.jumpSpeed` static (`UpdateJumpHeight`, `:19618-19641`).
        if matches!(self.wings_logic, 44 | 45) {
            if self.trying_to_hover_up() {
                self.vy -= 0.4;
                if self.vy > 0.0 {
                    self.vy -= 1.0;
                } else if self.vy > -self.jump_speed {
                    self.vy -= 0.2;
                }
                if self.vy < -self.jump_speed * 3.0 {
                    self.vy = -self.jump_speed * 3.0;
                }
            }
            // `:22132`/`:22177`: the hover-down nudge needs jump RELEASED.
            if self.trying_to_hover_down() && !self.control_jump && self.vy != 0.0 {
                self.vy += 0.4;
            }
        }
        // `:22194-22197`: the 30/31 boosters only take the 0.15 impulse
        // while NOT hover-down (the base 0.1 otherwise).
        if matches!(self.wings_logic, 30 | 31) && !self.trying_to_hover_down() {
            num = 0.15;
        }
        // `:22199-22213`: the shared ascent — impulse, falling brake,
        // rising assist above `-jumpSpeed * num4`, the `-jumpSpeed * num3`
        // cap. The C# reads the `Player.jumpSpeed` STATIC, which
        // `UpdateJumpHeight` (`:19618-19641`) has already rebased this tick
        // (boosts/mount/sticky/dazed) — the port's per-tick `jump_speed`.
        self.vy -= num;
        if self.vy > 0.0 {
            self.vy -= num2;
        } else if self.vy > -self.jump_speed * num4 {
            self.vy -= num5;
        }
        if self.vy < -self.jump_speed * num3 {
            self.vy = -self.jump_speed * num3;
        }
        // `:22230-22237`: the budget countdown — half for the hover set
        // while hover-down with no horizontal input, else 1.
        if hover_down && matches!(self.wings_logic, 22 | 28 | 30 | 31 | 37 | 45) {
            self.wing_time -= 0.5;
        } else {
            self.wing_time -= 1.0;
        }
        // `:22239-22242`: the Empress Brooch refills the budget to max on
        // every flying tick it stays non-zero (infinite flight; at exactly
        // 0 the refill does NOT fire — the tick that spends the last point
        // ends the flight).
        if self.empress_brooch && self.wing_time != 0.0 {
            self.wing_time = self.wing_time_max as f32;
        }
    }

    /// `Player.cs:27404-27425`: the post-block hover dampening — the hover
    /// wings holding down multiply vy toward the float. Runs AFTER the whole
    /// flight else-if chain (both the flying and the glide paths reach it),
    /// gated on `CanUseWingAbilities` (`:3990-4000`: `!merman &&
    /// !isPerformingJump_DownDash`). The wings-37 arm (`:27418-27425`)
    /// applies a SECOND ×0.92 pass after the shared ×0.9.
    fn hover_dampening(&mut self) {
        let can_use = !self.merman && !self.is_performing_down_dash();
        if !can_use || !self.trying_to_hover_down() || !self.control_jump || self.wing_time <= 0.0 {
            return;
        }
        if Self::hover_dampening_wing(self.wings_logic) {
            let num67 = if self.wings_logic == 45 { 0.8 } else { 0.9 };
            self.vy *= num67;
            if self.vy > -2.0 && self.vy < 1.0 {
                self.vy = 1e-5;
            }
        }
        // `:27418-27425`: Betsy's wings (37) damp AGAIN at 0.92.
        if self.wings_logic == 37 {
            self.vy *= 0.92;
            if self.vy > -2.0 && self.vy < 1.0 {
                self.vy = 1e-5;
            }
        }
    }

    /// `Player.GrappleMovement` (`Player.cs:22613-22740`): the pull replaces
    /// the velocity; the jump-off detaches. `hook` is `(center, type)` of the
    /// attached hook (the param already filters to the attached state).
    /// `Player.GrappleMovement`'s Track Hook hand-over (`Player.cs:22659-22693`):
    /// the Track Hook (403) gives the player to the minecart when its box
    /// overlaps the player over a track tile.
    ///
    /// The C# removes every grappling hook first (`:22675`), snaps the player
    /// onto the track's rail sub-column (`Minecart.GetOnTrack` `:1200-1241`),
    /// refuses a solid body (`Collision.SolidCollision` `:22683`) and then
    /// mounts the cart (`Mount.SetMount` `:22687`) with the wheel-sparks
    /// burst (`:22689-22690`).
    pub(crate) fn track_hook_cart_transfer(
        &mut self,
        world: &World,
        hooks: &[(u16, Vec2, f32)],
    ) -> bool {
        let Some((_, center, _)) = hooks
            .iter()
            .find(|(kind, ..)| *kind == crate::projectile::TRACK_HOOK)
        else {
            return false;
        };
        let Some(data) = crate::projectile::proj_data(crate::projectile::TRACK_HOOK) else {
            return false;
        };
        let hook_x = center.x - data.w * 0.5;
        let hook_y = center.y - data.h * 0.5;
        // `:22664-22666` — the hook's box against the player's box.
        if !(hook_x < self.x + WIDTH as f32
            && hook_x + data.w >= self.x
            && hook_y < self.y + self.height
            && hook_y + data.h >= self.y)
        {
            return false;
        }
        // `:22668-22670` — the tile under the hook's centre.
        let tx = (center.x as i32) / 16;
        let ty = (center.y as i32) / 16;
        self.vx = 0.0;
        self.vy = 0.0;
        if !world
            .tile(tx, ty)
            .is_some_and(|t| t.kind == tr_core::smart_cursor::TRACK_TILE)
        {
            return false;
        }
        // `:22675` — the hooks go before the probe.
        self.release_grappling_hooks = true;
        let mut position = Vec2::new(
            center.x - WIDTH as f32 * 0.5,
            center.y - self.height as f32 * 0.5,
        );
        let height_boost = crate::mount::mount_data(self.cart_mount)
            .map(|row| row.height_boost)
            .unwrap_or(0);
        let height = self.height + height_boost as f32;
        // `:22680-22683` — the on-track snap and the solid-body refusal.
        if !crate::minecart::get_on_track(
            world,
            tx,
            ty,
            &mut position,
            WIDTH as i32,
            height as i32,
        ) {
            return false;
        }
        if collides_rect(world, position.x, position.y, WIDTH as f32, height - 20.0) {
            return false;
        }
        // `:22684-22688` — the position, the rotation and the mount.
        self.x = position.x;
        self.y = position.y;
        self.prev_x = position.x;
        self.prev_y = position.y;
        self.cart_mounted = true;
        self.set_mount(self.cart_mount, self.wet);
        // `:22689-22690` — the wheel-sparks burst (25 sparks).
        minecart::wheel_sparks(
            &mut self.pending_sparks,
            &mut self.spark_rng,
            position,
            WIDTH as i32,
            height as i32,
            25,
            self.full_rotation,
        );
        true
    }

    fn grapple_movement(
        &mut self,
        world: &World,
        hooks: &[(u16, Vec2, f32)],
        input: PlayerInput,
        item_in_use: bool,
    ) {
        // `:22615-22618` — `grappling[0] < 0` returns. The caller hands the
        // registered (attached) set, so an empty set is the early return.
        if hooks.is_empty() {
            return;
        }
        // `:22619` — `StopVanityActions` (`Player.cs:33458-33463`): stop the
        // petting animal (unported), sit up and stop sleeping. The port carries
        // sitting/sleeping, so both run; petting has no field.
        self.sit_up();
        self.stop_sleeping();
        // `:22620-22623` — a mount not in the `MountID.Sets.CanUseHooks` set
        // ({54, 57-60}) dismounts while the hooks are out.
        if self.mount.is_active() && !crate::mount::can_use_hooks(self.mount.kind()) {
            self.try_dismount(world);
        }
        // `:22624-22640` — the carpet/wing frame head. `canCarpet`/`rocketFrame`
        // have no port fields; `carpetFrame = -1` and the wing frame ladder are
        // the visual seam (the wings-4/30 overrides ride the wing draw).
        self.carpet_frame = -1;
        self.wing_frame = 1;
        // `:22627-22628` — `flag` (nearly stationary) and `flag2` (floating in
        // ANY liquid with near-zero vy) read the PRE-pull velocity. The C#
        // `base.wet` is WetCollision (any liquid), so the port uses the union.
        let flag = Vec2::new(self.vx, self.vy).length() < 2.0;
        let any_wet = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        let flag2 = any_wet && self.vy > -0.02 && self.vy < 0.02;
        // `:22629-22632` — the wing frame zeroes when nearly still / floating /
        // grounded (visual; the port keeps the field current).
        if (flag2 | flag) || self.vy == 0.0 {
            self.wing_frame = 0;
        }
        // `:22633-22640` — the wings-4/30 frame overrides. The port folds
        // `wings` (the look) and `wingsLogic` (the gameplay) into one field
        // (`wings_logic`, the accessory's `wingSlot`, `Player.cs:15599`).
        if self.wings_logic == 4 {
            self.wing_frame = 3;
        }
        if self.wings_logic == 30 {
            self.wing_frame = 0;
        }
        // `:22641` — `RefreshMovementAbilities` (`:22858-22867`): the flight
        // budgets refill every tick while grappling.

        // `RefreshMovementAbilities` (`Player.cs:22858-22867`): the flight
        // budgets refill every tick while grappling; `:22642-22644` clears
        // the rocket state with them.
        self.wing_time = self.wing_time_max as f32;
        self.rocket_time = self.rocket_time_max;
        self.rocket_delay = 0;
        self.can_rocket = false;
        self.rocket_release = false;
        self.refresh_double_jumps();
        // `:22642-22644` — the rocket flags clear.
        self.can_rocket = false;
        self.rocket_release = false;
        // `:22645`: the fall origin refreshes per tick — no fall damage.
        self.fall_start = (self.y / TILE_PX) as i32;

        // `:22654` — `GetGrapplingForces` (`Player.cs:22757-22856`): the
        // steering-adjusted average of every registered hook's center, clamped
        // to `grappling[0]`'s type pull speed. The steering reads `controlUp`
        // (`:22775`/`:22792`); the tuple's third element is the hook's rotation
        // (only the Squirrel Hook reads it).
        let pull = crate::grapple::grapple_pull(
            hooks,
            Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5),
            crate::grapple::steering_stick(input.left, input.right, input.up, input.down),
        );
        let Some(pull) = pull else {
            // `:22820-22824`: no attached hook carries the velocity through.
            return;
        };
        // `:22655-22658` — `GoingDownWithGrapple` is SET only when the pull
        // points down; `ResetEffects` (`:18506`) clears it per tick, so a
        // non-down pull leaves it false (the reset runs at the tick head).
        if pull.vel.y > 0.0 {
            self.going_down_with_grapple = true;
        }
        // `:22659`: `flag3` = the pull points down while the PRE-pull velocity
        // Y is zero (a grounded pull) — read before the replacement.
        let flag3 = pull.vel.y > 0.0 && self.vy == 0.0;
        // `:22660-22661`: the pull replaces the velocity outright.
        self.vx = pull.vel.x;
        self.vy = pull.vel.y;
        // `:22662-22692`: the Track Hook (403) hands the player to the minecart
        // when its box overlaps the player over a track tile.
        self.track_hook_cart_transfer(world, hooks);

        // `:22693-22707`: the direction follows the pull; an exactly-zero X
        // takes the Squirrel Hook's preferred direction instead. The C# gates
        // the block on `itemAnimation == 0`, which the sim passes in.
        if !item_in_use {
            if self.vx == 0.0 {
                if let Some(d) = pull.dir {
                    self.facing = d;
                }
            }
            if self.vx > 0.0 {
                self.facing = 1;
            }
            if self.vx < 0.0 {
                self.facing = -1;
            }
        }

        // `:22708-22739`: the jump-off.
        if input.jump {
            if self.release_jump {
                // `:22712` — `flag4 = flag2 | flag` (the PRE-pull reads above).
                let mut flag4 = flag2 | flag;
                // `:22713-22716` — holding DOWN cancels the stationary jump.
                if input.down {
                    flag4 = false;
                }
                // `:22717-22720` — a grounded down-pull cancels it unless UP is
                // held (`!controlUp`, NOT `!controlJump`).
                if flag3 && !input.up {
                    flag4 = false;
                }
                if flag4 {
                    // `:22723-22724` — the full jump impulse. `oldStyleParkour`
                    // halves the height; the parkour gate is unported (false).
                    self.vy = -self.jump_speed;
                    self.jump = self.jump_height;
                    self.release_jump = false;
                } else {
                    // `:22729` — the nudge path detaches without the jump.
                    self.vy += 0.01;
                    self.release_jump = false;
                }
                // `:22732` — `RefreshDoubleJumps`.
                self.refresh_double_jumps();
                // `:22733` — `RemoveAllGrapplingHooks`: the caller drops the
                // owner's hooks (`Player.cs:33562-33574`).
                self.release_grappling_hooks = true;
                self.grappling_active = false;
            }
        } else {
            // `:22738` — `releaseJump = true`.
            self.release_jump = true;
        }
    }

    /// `Player.HorizontalMovement` (`Player.cs:19772-20090`), the exact if/else
    /// chain: the pending booster push applies pre-input and clamps
    /// (`:19818-19833`), the cart arms gate their accel on `velocity.Y == 0`
    /// (`:19849/:19907`), the off-track brakes pull the roll toward rest
    /// (`:19858-19865`, `:19948-19956`), the cart coast arm clamps ±max without
    /// on-track friction (`:20023-20058`), and the grounded/airborne friction
    /// tails close the chain (`:20059-20087`).
    fn horizontal_movement(&mut self, input: PlayerInput) {
        // W4-c14 - `Player.cs:19782-19793`: the Pogo Stick's (mount 43)
        // grounded hop - control key held, resting, jump not held: the
        // Item168 cue at the CENTRE (`base.Center`, the Vector2 facade
        // truncates), then the half-speed hop and the early return. The
        // port queues the cue on the `pending_cart_sounds` drain (the
        // Item55 convention - the drain's facade truncates too).
        if (self.control_left || self.control_right)
            && self.mount.is_active()
            && self.mount.kind() == 43
            && self.vy == 0.0
            && !self.control_jump
        {
            let (cx, cy) = self.center();
            self.pending_cart_sounds.push(crate::minecart::CartCue {
                cue: "Item_168",
                x: cx,
                y: cy,
            });
            let mut hop = 5.01 * 0.5; // `Player.jumpSpeed` static × gravDir 1
            if hop < 2.0 {
                hop = 2.0;
            }
            self.vy = -(hop + 0.01);
            self.jump = 15; // `Player.jumpHeight` - the STATIC 15 (`:19791`)
            self.full_rotation = 0.0;
            return;
        }
        // `Player.cs:19779`: the fast-run threshold - halfway between the
        // boot top speed and the base run speed.
        let fast_run_threshold = (self.acc_run_speed + self.max_run_speed) / 2.0;
        // `Player.cs:19818-19833`: the boost applies before any input reads
        // the velocity, then clamps.
        if self.track_boost != 0.0 {
            self.vx += self.track_boost;
            self.track_boost = 0.0;
            if self.vx < 0.0 {
                if self.vx < -self.max_run_speed {
                    self.vx = -self.max_run_speed;
                }
            } else if self.vx > self.max_run_speed {
                self.vx = self.max_run_speed;
            }
        }
        // `Player.cs:19835-19845`: the direction lock — `flag3 =
        // (itemAnimation == 0 || inventory[selectedItem].useTurn) &&
        // mount.AllowDirectionChange && !sandStorm`; a cart additionally
        // demands the roll direction match (`Math.Sign(velocity.X) == num5`,
        // `Sign(0) = 0` locks a stationary cart), read AFTER the boost above.
        // So a NON-useTurn weapon locks the facing for the whole swing
        // (`useTurn` is what lets a pickaxe/hammer turn mid-swing).
        // M1 keep-one (i04/i05 × i12): the i12 PlayerInput channel
        // (`item_in_use`/`held_use_turn`, filled by world_sim from app.swing
        // + the registry row) is the live source — the gate's own i04/i05
        // frame_inputs form reads the identical predicate.
        let dir_keys = self.control_right as i32 - self.control_left as i32;
        if dir_keys != 0 {
            let mut allow = (!input.item_in_use || input.held_use_turn)
                && self.mount.allow_direction_change()
                && !self.sand_storm;
            if self.cart_mounted {
                let roll = (self.vx > 0.0) as i32 - (self.vx < 0.0) as i32;
                allow &= roll == dir_keys;
            }
            if allow {
                self.facing = dir_keys;
            }
        }
        if self.control_left && self.vx > -self.max_run_speed {
            // `:19848-19857`: the cart accel gate — airborne carts get no
            // input accel that tick.
            if !self.cart_mounted || self.vy == 0.0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration;
            }
            // `:19858-19865`: the off-track brake.
            if self.on_wrong_ground {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `Player.cs:19868-19902`: the rolling wheel sparks + Item55 —
            // the arm's gate is the direction-OPPOSITE sign (`velocity.X >=
            // 0f`): the sound + sparks fire while braking a rightward roll or
            // at rest, not while accelerating left. (`itemAnimation == 0`
            // holds — the port's item swing never gates movement.)
            if self.cart_mounted
                && !self.on_wrong_ground
                && self.on_track
                && self.vy == 0.0
                && self.vx >= 0.0
            {
                let pos = Vec2::new(self.x, self.y);
                let vel = Vec2::new(self.vx, self.vy);
                let body_h = self.body_height() as i32;
                let max_run = self.max_run_speed;
                let rotation = self.full_rotation;
                minecart::rolling_wheel_sparks(
                    &mut self.pending_cart_sounds,
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pos,
                    WIDTH as i32,
                    body_h,
                    vel,
                    max_run,
                    rotation,
                );
            }
        } else if self.control_right && self.vx < self.max_run_speed {
            // `:19904-19916`: the mirror arm.
            if !self.cart_mounted || self.vy == 0.0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration;
            }
            // `:19948-19956`: the off-track brake.
            if self.on_wrong_ground {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `Player.cs:19926-19958`: the mirror arm — the gate is
            // `velocity.X <= 0f` (braking a leftward roll or at rest).
            if self.cart_mounted
                && !self.on_wrong_ground
                && self.on_track
                && self.vy == 0.0
                && self.vx <= 0.0
            {
                let pos = Vec2::new(self.x, self.y);
                let vel = Vec2::new(self.vx, self.vy);
                let body_h = self.body_height() as i32;
                let max_run = self.max_run_speed;
                let rotation = self.full_rotation;
                minecart::rolling_wheel_sparks(
                    &mut self.pending_cart_sounds,
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pos,
                    WIDTH as i32,
                    body_h,
                    vel,
                    max_run,
                    rotation,
                );
            }
        } else if self.control_left && self.vx > -self.acc_run_speed && self.dash_delay >= 0 {
            // `:19960-19972`: the over-max refine — the inner gate carries the
            // airborne-wings term (`wingsLogic > 0`; carts zero it above).
            if self.vy == 0.0 || self.wings_logic > 0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration * 0.2;
                if self.wings_logic > 0 {
                    self.vx -= self.run_acceleration * 0.2;
                }
            }
            // `:19973-19980`: the off-track brake — unreachable for carts
            // (maxRunSpeed == accRunSpeed, `:26652-26653`), kept verbatim.
            if self.on_wrong_ground {
                if self.vx < self.run_slowdown {
                    self.vx += self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `:19987-19990`: past the fast-run threshold, grounded and
            // unmounted — the Hermes-boot footstep + dust trail.
            if self.vx < -fast_run_threshold && self.vy == 0.0 && !self.mount.is_active() {
                self.spawn_fast_run_particles();
            }
        } else if self.control_right && self.vx < self.acc_run_speed && self.dash_delay >= 0 {
            // `:19992-20005`: the mirror arm.
            if self.vy == 0.0 || self.wings_logic > 0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration * 0.2;
                if self.wings_logic > 0 {
                    self.vx += self.run_acceleration * 0.2;
                }
            }
            // `:20006-20013`: the off-track brake (see above).
            if self.on_wrong_ground {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `:20017-20020`: the mirror fast-run arm.
            if self.vx > fast_run_threshold && self.vy == 0.0 && !self.mount.is_active() {
                self.spawn_fast_run_particles();
            }
        } else if self.cart_mounted && self.vx.abs() >= 1.0 {
            // `:20023-20058`: the cart coast — friction only off-track (rails
            // have no rolling resistance; the on-track 0.04 applies below
            // 1 px/tick through the `velocity.Y == 0` arm), then the clamp.
            if self.on_wrong_ground {
                if self.vx > 0.0 {
                    if self.vx > self.run_slowdown {
                        self.vx -= self.run_slowdown;
                    } else {
                        self.vx = 0.0;
                    }
                } else if self.vx < 0.0 {
                    if self.vx < -self.run_slowdown {
                        self.vx += self.run_slowdown;
                    } else {
                        self.vx = 0.0;
                    }
                }
            }
            if self.vx > self.max_run_speed {
                self.vx = self.max_run_speed;
            }
            if self.vx < -self.max_run_speed {
                self.vx = -self.max_run_speed;
            }
        } else if self.vy == 0.0 {
            // `:20059-20072`: the grounded friction — `runSlowdown` is 0.04
            // on-track and 0.2 off-track via the on-track check.
            if self.vx > self.run_slowdown {
                self.vx -= self.run_slowdown;
            } else if self.vx < -self.run_slowdown {
                self.vx += self.run_slowdown;
            } else {
                self.vx = 0.0;
            }
        } else {
            // `:20074-20087`: the airborne friction, halved.
            // (`PortalPhysicsEnabled` has no port system.)
            if self.vx > self.run_slowdown * 0.5 {
                self.vx -= self.run_slowdown * 0.5;
            } else if self.vx < -self.run_slowdown * 0.5 {
                self.vx += self.run_slowdown * 0.5;
            } else {
                self.vx = 0.0;
            }
        }
    }

    /// `Player.FloorVisuals` (`Player.cs:23708-23743`) + `GetFloorTile`
    /// (`:23755-23783`): the tile row at the feet, center column first, then
    /// the neighbors, solid tiles only. The dust visuals (MakeFloorDust) and
    /// the slope dust carve-out stay with the particle tasks.
    fn floor_visuals(&mut self, world: &World) {
        let x = ((self.x + self.width * 0.5) / TILE_PX) as i32;
        // `Player.GetFloorTile` (`Player.cs:23755-23783`) reads `base.height` —
        // the live hitbox height (the mount framework's equip; the cart's 52).
        let y = ((self.y + self.body_height()) / TILE_PX) as i32;
        let mut kind = None;
        for tx in [x, x - 1, x + 1] {
            if let Some(t) = world.tile(tx, y) {
                if t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind) {
                    kind = Some(t.kind);
                    break;
                }
            }
        }
        let Some(kind) = kind else {
            self.sticky = false;
            self.slippy = false;
            self.slippy2 = false;
            self.powerrun = false;
            self.running_on_sand = false;
            return;
        };
        // `Player.cs:23727-23731`: cobweb 229, the IceSkateSlippery set
        // (`TileID.cs:481`), FrozenSlimeBlock 197, Asphalt 198, and the sand
        // conversions (`TileID.cs:92-94`).
        self.sticky = kind == 229;
        self.slippy = tr_core::tiledata::SET_ICE_SKATE_SLIPPERY.has(kind);
        self.slippy2 = kind == 197;
        self.powerrun = kind == 198;
        self.running_on_sand = tr_core::tiledata::SET_CONVERSION_SAND.has(kind)
            || tr_core::tiledata::SET_CONVERSION_SANDSTONE.has(kind)
            || tr_core::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind);
    }

    /// `Player.luck` (`Player.cs:3086`) — the value the roll consumers read
    /// (`Luck.RollLuck(this.luck, range)`, `Player.cs:18188`; `DamageVar`
    /// `:44342`; `NPCLoot_DropMoney` `NPC.cs:79622`). Never clamped — the
    /// −0.7/1.0 caps are UI-only (`NormalizedLuck`, `:4107-4122`).
    pub fn luck(&self) -> f32 {
        self.luck_state.luck
    }

    /// `Player.UpdateLuck` → `UpdateLuckFactors` (`Player.cs:29041-29135`),
    /// called from the post-buff-effects position (`:25645`): the ladybug and
    /// coin decays run for every player, the torch scan + the broken-mirror
    /// decay for `Main.myPlayer` only (the port's single player). The torch
    /// slot fills with the 1-row-per-tick scan in the M2-BUFF-6 phase 2; the
    /// `luckNeedsSync` tail is net-only.
    fn update_luck(&mut self, world: &World) {
        let day_rate = world.day_rate();
        self.luck_state.update_lady_bug_decay(day_rate);
        self.luck_state.update_coin_decay(day_rate);
        // `TryRecalculatingTorchLuck` + the diff check (`Player.cs:29126-29130`
        // — the diff half is net-only): `Main.myPlayer`-only in the C#, which
        // is the port's single player.
        let center = self.center();
        crate::luck::torch_scan_step(
            &mut self.luck_state,
            world,
            center,
            self.dead,
            self.held_torch_style,
            self.zones,
        );
        self.luck_state.update_broken_mirror_decay(day_rate);
        self.luck_state.recalculate_luck();
    }

    /// `Player.AddCoinLuck` (`Player.cs:18090-18101`): the shimmer-coin feed
    /// (`WorldItem.cs:1692-1716` is its only C# caller; the coin-shimmer chain
    /// is unported — the callable is unit-tested per the plan). The
    /// `luckNeedsSync` tail is net-only.
    pub fn add_coin_luck(&mut self, coin_position: (f32, f32), coin_amount: i32) {
        let (cx, cy) = self.center();
        let dist = ((coin_position.0 - cx).powi(2) + (coin_position.1 - cy).powi(2)).sqrt();
        if !self.dead && dist < 1000.0 {
            self.luck_state.coin_luck += coin_amount as f32;
            if self.luck_state.coin_luck > 1_000_000.0 {
                self.luck_state.coin_luck = 1_000_000.0;
            }
        }
    }

    /// `NPC.LadyBugKilled`'s player half (`NPC.cs:81566-81598`) — see
    /// `luck::on_lady_bug_killed`; returns the `ladyBugRainBoost` add for the
    /// future Rain writer (the critter row, M4).
    pub fn on_lady_bug_killed(&mut self, position: (f32, f32), gold: bool) -> i32 {
        let (cx, cy) = self.center();
        crate::luck::on_lady_bug_killed(&mut self.luck_state, position, (cx, cy), self.dead, gold)
    }

    /// `NPC.LadyBugLuck`'s player half (`NPC.cs:81600-81626`) — see
    /// `luck::on_lady_bug_nearby`; the `releaseOwner == 255` wild-only gate
    /// stays with the NPC (M4 critter row).
    pub fn on_lady_bug_nearby(&mut self, position: (f32, f32), gold: bool) {
        let (cx, cy) = self.center();
        crate::luck::on_lady_bug_nearby(&mut self.luck_state, position, (cx, cy), gold);
    }

    /// `NPC.NPCLuck` (`NPC.cs:81628-81652`) — see `luck::on_town_npc_death`.
    pub fn on_town_npc_death(&mut self, npc_type: i32) {
        crate::luck::on_town_npc_death(&mut self.luck_state, npc_type, self.dead);
    }

    /// `Player.MinionNPCTargetAim` (`Player.cs:56914-56933`): the summon
    /// staff's right-click picks the nearest chaseable NPC to the mouse world
    /// point; a second click on the same target clears it.
    ///
    /// S1-P-i27 fixes (verbatim against the C#):
    /// * the distance is `NPC.Hitbox.Distance(mouseWorld)` — the
    ///   rectangle-to-point distance (`Utils.cs:1507-1542`, 0 inside the box),
    ///   NOT a centre-to-centre distance. A wide NPC's edge can be nearer to
    ///   the cursor than a small NPC's centre even when the small one's centre
    ///   is closer.
    /// * the scan re-evaluates the CURRENT best's hitbox distance each
    ///   iteration (`Main.npc[num].Hitbox.Distance(mouseWorld)`, `:56920`) —
    ///   equivalent to tracking it, kept as a re-read for clarity.
    /// * `do_not_disable_if_the_target_is_the_same` (`:56925`): the toggle-off
    ///   only fires when the flag is false. The use path passes false
    ///   (`ItemCheck_MinionAltFeatureUse`, `:49777`). The whip's aiStyle-165
    ///   on-hit retarget is a PLAIN WRITE (`Projectile.cs:14261`,
    ///   `whip_on_hit` — it never calls this method), so no caller passes
    ///   true today; the parameter mirrors the C# signature.
    pub fn minion_npc_target_aim(
        &mut self,
        npcs: &[Npc],
        mouse_world: Vec2,
        do_not_disable_if_the_target_is_the_same: bool,
    ) {
        let hitbox_dist = |n: &Npc| -> f32 {
            let (w, h) = n.size();
            crate::npc_ai_style64::rect_distance(
                n.x as i32,
                n.y as i32,
                w as i32,
                h as i32,
                mouse_world.x,
                mouse_world.y,
            )
        };
        let mut best: i32 = -1;
        for (i, n) in npcs.iter().enumerate() {
            if !n.can_be_chased_by() {
                continue;
            }
            // `:56920` — `num == -1 || Main.npc[i].Hitbox.Distance(mouseWorld)
            // < Main.npc[num].Hitbox.Distance(mouseWorld)`.
            if best == -1 || hitbox_dist(n) < hitbox_dist(&npcs[best as usize]) {
                best = i as i32;
            }
        }
        if self.minion_attack_target_npc == best && !do_not_disable_if_the_target_is_the_same {
            self.minion_attack_target_npc = -1;
        } else {
            self.minion_attack_target_npc = best;
        }
    }

    /// `Player.Update`'s tank-pet decay (`Player.cs:24539-24548`): while
    /// `tankPet >= 0`, the first tick arms the one-tick grace latch
    /// (`tankPetReset = true`); a tick where the pet did NOT re-pin the
    /// claim (AI_120 writes `tankPetReset = false` every non-idle tick,
    /// `Projectile.cs:44683`) drops it to -1. The callers run it inside
    /// both fixed-step branches — the C# block sits ahead of the
    /// dead-player early return (`:24749-24754`), so the dead player's
    /// claim decays too. S1-P-i56.
    pub fn tank_pet_decay(&mut self) {
        if self.tank_pet >= 0 {
            if !self.tank_pet_reset {
                self.tank_pet_reset = true;
            } else {
                self.tank_pet = -1;
            }
        }
    }

    /// `Player.UpdateMinionTarget` (`Player.cs:56999-57028`) — the per-tick
    /// minion-target maintenance (the `whoAmI == Main.myPlayer` gate,
    /// `:57001-57004`, folds to the local player: the port runs one).
    ///
    /// S1-P-i27 fixes (verbatim against the C#):
    /// * the rest target drops when the player drifts past 1000 px
    ///   (`base.Distance(MinionRestTargetPoint) > 1000f`, `:57005-57007`) —
    ///   `base.Distance` is CENTRE-to-point (`Entity.cs:195-198`); the C#
    ///   runs the compare UNCONDITIONALLY (a ZERO point is >1000 px from
    ///   any real player and re-zeroes — no `!= ZERO` gate).
    /// * the attack target drops when unchaseable OR its HITBOX is past
    ///   3000 px (`Main.npc[..].Hitbox.Distance(base.Center) > 3000f`,
    ///   `:57009-57011`) — rectangle-to-CENTRE, not centre-to-position.
    ///
    /// The stardust-guardian rest-spot dust ring (`:57013-57027`) is the
    /// visual layer (deferred, ANALYSIS). Gate-merge: supersedes i56's
    /// `invalidate_minion_target` (same halves; the i56 name stays as an
    /// alias for its witnesses).
    pub fn update_minion_target(&mut self, npcs: &[Npc]) {
        let (cx, cy) = self.center();
        let player_center = Vec2::new(cx, cy);
        // `:57005-57007` — the rest target's 1000-px centre-distance drop.
        if (player_center - self.minion_rest_target_point).length() > 1000.0 {
            self.minion_rest_target_point = Vec2::ZERO;
        }
        // `:57009-57011` — the attack target's chaseable + 3000-px hitbox drop.
        if self.minion_attack_target_npc != -1 {
            let ok = npcs
                .get(self.minion_attack_target_npc as usize)
                .is_some_and(|n| {
                    if !n.can_be_chased_by() {
                        return false;
                    }
                    let (w, h) = n.size();
                    crate::npc_ai_style64::rect_distance(
                        n.x as i32,
                        n.y as i32,
                        w as i32,
                        h as i32,
                        cx,
                        cy,
                    ) <= 3000.0
                });
            if !ok {
                self.minion_attack_target_npc = -1;
            }
        }
    }

    /// S1-P-i56's name for the same `UpdateMinionTarget` maintenance —
    /// kept as an alias for the Guardian witnesses (the live tick calls
    /// [`Player::update_minion_target`], S1-P-i27's canonical form).
    pub fn invalidate_minion_target(&mut self, npcs: &[Npc]) {
        self.update_minion_target(npcs);
    }

    /// `Player.HasMinionRestTarget` (`Player.cs:4060`): the rest spot is set
    /// (non-zero). The idle minions read it to choose their return point.
    pub fn has_minion_rest_target(&self) -> bool {
        self.minion_rest_target_point != Vec2::ZERO
    }

    /// `Player.HasMinionAttackTargetNPC` (`Player.cs:4062`).
    pub fn has_minion_attack_target_npc(&self) -> bool {
        self.minion_attack_target_npc != -1
    }

    /// `Player.MinionRestTargetAim` (`Player.cs:56935-56997`): the Forbidden
    /// set bonus's rest-spot aim — the mouse world point is projected onto the
    /// ground below/above it and stored as the idle minions' return point.
    ///
    /// The C# walks the tile column at the mouse's X: if the mouse tile is a
    /// full solid, it scans up to 20 tiles up and 20 down for the solid run's
    /// edges and picks a row 2-3 tiles clear of the run (`:56942-56971`); then
    /// it drops straight down until a 5-tile window holds a
    /// `SolidTileAllowBottomSlope` (`:56973-56991`). The spot is
    /// `(tileX*16+8, row*16)` and is kept only within 1000 px of the player
    /// (`base.Distance(vector) <= 1000f`, `:56993-56996`).
    ///
    /// S1-P-i27: ported verbatim. The `WorldGen.SolidTileAllowBottomSlope`
    /// probe is the full form (platform-top-frame clause + inactive test,
    /// `WorldGen.cs:70167-70189`), mirrored from the AI_026 slice-C copy.
    pub fn minion_rest_target_aim(&mut self, world: &World, mouse_world: Vec2) {
        let num = mouse_world.x as i32 / 16;
        let num2 = mouse_world.y as i32 / 16;
        // The full-solid test the column scan uses (`Main.tile[..].nactive()
        // && tileSolid[type] && !tileSolidTop[type]`, `:56942`/`:56949`/
        // `:56962`). Out-of-world reads non-solid here (the C# would throw →
        // the try/catch folds to the scan's bounds).
        let full_solid_at = |x: i32, y: i32| -> bool {
            world.tile(x, y).is_some_and(|t| {
                t.active()
                    && !t.inactive()
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
            })
        };
        let mut num3 = 0;
        if full_solid_at(num, num2) {
            // `:56944-56957` — the upward scan (num4 = the topmost solid
            // offset, ≤ 0).
            let mut num4 = 0;
            let mut num5 = 0;
            while num5 > -20 && num2 + num5 > 1 {
                let num6 = num2 + num5;
                if full_solid_at(num, num6) {
                    num4 = num5;
                    num5 -= 1;
                    continue;
                }
                num4 = num5;
                break;
            }
            // `:56958-56969` — the downward scan (num7 = the bottommost solid
            // offset, ≥ 0).
            let mut num7 = 0;
            for i in 0..20 {
                if num2 + i >= world.height {
                    break;
                }
                let num8 = num2 + i;
                if full_solid_at(num, num8) {
                    num7 = i;
                    continue;
                }
                num7 = i;
                break;
            }
            // `:56970` — pick the side with more clearance.
            num3 = if num7 <= -num4 { num7 + 3 } else { num4 - 2 };
        }
        let mut num9 = num2 + num3;
        // `:56973-56991` — drop until a 5-tile window holds a bottom-slope-
        // allowed solid. The C#'s `while (!flag)` has no bound; the port caps
        // at the world's bottom row (out-of-world reads solid, so the loop
        // terminates there anyway).
        let solid_allow_bottom_slope = |x: i32, y: i32| -> bool {
            crate::projectile_ai026_c::solid_tile_allow_bottom_slope(world, x, y)
        };
        let mut flag = false;
        for j in num9..num9 + 5 {
            if solid_allow_bottom_slope(num, j) {
                flag = true;
            }
        }
        while !flag && num9 < world.height {
            num9 += 1;
            for k in num9..num9 + 5 {
                if solid_allow_bottom_slope(num, k) {
                    flag = true;
                }
            }
        }
        let vector = Vec2::new((num * 16 + 8) as f32, (num9 * 16) as f32);
        // `:56993-56996` — the 1000-px centre-distance gate.
        let (cx, cy) = self.center();
        if (Vec2::new(cx, cy) - vector).length() <= 1000.0 {
            self.minion_rest_target_point = vector;
        }
    }

    /// `Player.KeyDoubleTap` (`Player.cs:56764-56787`) — the down-cardinal
    /// double-tap. S1-P-i27: the Forbidden set bonus's rest aim fires here
    /// (`:56779-56786`). The `CommandForbiddenStorm` half (`:56784`) is the
    /// storm command's own lane (deferred, ANALYSIS — it spawns the 656
    /// projectile + pays mana). The Vortex stealth toggle (`:56775-56778`)
    /// is the stealth system's lane (unported).
    ///
    /// `reversed` is `Main.ReversedUpDownArmorSetBonuses` (`Main.cs:273`):
    /// when true the set-bonus cardinal is UP (keyDir 1), so the DOWN
    /// double-tap (keyDir 0) does nothing.
    pub fn key_double_tap_down(&mut self, world: &World, mouse_world: Vec2, reversed: bool) {
        let num = if reversed { 1 } else { 0 };
        // keyDir 0 (down) != num when reversed → early return (`:56771-56774`).
        if 0 != num {
            return;
        }
        if self.set_forbidden {
            self.minion_rest_target_aim(world, mouse_world);
            // `CommandForbiddenStorm` (`:56782-56785`) — deferred (ANALYSIS).
        }
    }

    /// `Player.KeyHoldDown` (`Player.cs:56894-56912`) — the down-cardinal
    /// hold. S1-P-i27: at holdTime >= 60 the Stardust/Forbidden set bonuses
    /// clear the rest target (`:56903-56910`). `reversed` flips the cardinal
    /// to UP, so the DOWN hold does nothing.
    pub fn key_hold_down(&mut self, hold_time: i32, reversed: bool) {
        let num = if reversed { 1 } else { 0 };
        if 0 != num {
            return;
        }
        if self.set_stardust && hold_time >= 60 {
            self.minion_rest_target_point = Vec2::ZERO;
        }
        if self.set_forbidden && hold_time >= 60 {
            self.minion_rest_target_point = Vec2::ZERO;
        }
    }

    /// The down-cardinal double-tap/hold driver (`Player.cs:25022-25073`),
    /// down-cardinal half. S1-P-i27: the rest-target command's input edge.
    /// Called once per tick with the live down control; the 15-tick
    /// double-tap window and the hold counter mirror the C#'s
    /// `doubleTapCardinalTimer[0]`/`holdDownCardinalTimer[0]`.
    pub fn update_minion_rest_command(
        &mut self,
        world: &World,
        mouse_world: Vec2,
        control_down: bool,
        reversed: bool,
    ) {
        // `:25024-25028` — the double-tap timer decays toward 0.
        self.double_tap_down_timer -= 1;
        if self.double_tap_down_timer < 0 {
            self.double_tap_down_timer = 0;
        }
        // `:25037` — flag5 = controlDown && releaseDown (the press edge).
        let flag5 = control_down && self.release_down;
        if flag5 {
            if self.double_tap_down_timer > 0 {
                self.key_double_tap_down(world, mouse_world, reversed);
            } else {
                self.double_tap_down_timer = 15;
            }
        }
        // `:25064-25072` — the hold counter.
        if control_down {
            self.hold_down_timer += 1;
            self.key_hold_down(self.hold_down_timer, reversed);
        } else {
            self.hold_down_timer = 0;
        }
        // `Player.releaseDown = !controlDown` (the per-tick latch refresh).
        self.release_down = !control_down;
    }

    /// `Player.CheckDrowning` (`Player.cs:23191-23281`): the breath only
    /// drains while the **head** is under the water — the C#'s
    /// `Collision.DrownCollision` probe (`:23193`), not the "any water"
    /// stand-in the port used before S1-E-13b. The drain's shape is the
    /// C#'s: the breath counter (`breathCD`) reaches `breathCDMax` (7) before
    /// a tick is taken, and the refill is 3 ticks per tick out of the water.
    ///
    /// The C#'s accessory arms (the Merman armour 250/4275, the breathing
    /// reed, the shimmering and the mount 4) have no ported counterparts
    /// yet; the Gills buff (4) arm IS ported (`Player.cs:23226-23229`,
    /// S1-E-7 wave A): `flag = Main.getGoodWorld && !flag` — on the For the
    /// Worthy seed the gills invert the probe (air drowns, water breathes).
    fn check_drowning(&mut self, world: &World) {
        // `this.gravDir` is -1 for the vanilla gravity — the probe sits at the
        // head; the NPCs' call passes 1 (the feet).
        let mut drowning = crate::collision::drown_collision(
            world,
            self.x,
            self.y,
            self.width as i32,
            self.body_height() as i32,
            -1.0,
            false,
        );
        // `Player.cs:23226-23229`: the Gills buff flips the probe — only the
        // For-the-Worthy seed keeps a drain (in the dry).
        if self.gills {
            let good_world = world
                .seeds
                .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
            drowning = good_world && !drowning;
        }
        if drowning {
            self.breath_cd += 1;
            if self.breath_cd >= BREATH_CD {
                self.breath_cd = 0;
                self.breath -= 1;
                // `Player.cs:23255-23258`: the breath-zero gasp —
                // `PlaySound(23)` unpositioned, ONCE on the 1→0 edge (the
                // `<= 0` damage arm below keeps firing every drain).
                if self.breath == 0 {
                    self.pending_drown_sound = true;
                }
                if self.breath <= 0 {
                    self.breath = 0;
                    self.lose_health(2);
                }
            }
        } else {
            self.breath = (self.breath + 3).min(BREATH_MAX);
            self.breath_cd = 0;
        }
    }

    /// `Player.StickyMovement` (`Player.cs:22910-23087`): the cobweb (51) /
    /// honey-block (229) drag. Reads the surface via `Collision.StickyTiles`
    /// (offset 0.01 against the gravity direction, `:22929`), then
    ///
    /// * breaks a cobweb once `stickyBreak` beats `Next(20,100)` — `Next(80,300)`
    ///   on a slime/gel mount (`:22938-22954`), killed through the caller;
    /// * clamps the velocity (the `flag2` mount halves it, `:22965-22972`; the
    ///   rest take the `±1` cap and the `0.85`/`0.6` X damp, `:22975-23028`);
    /// * puffs type-153 web dust on the hit row and the two below it
    ///   (`:23030-23081`).
    ///
    /// `cart_ignore` is the C#'s `flag` (a cart above 5 px/tick ignores the
    /// drag, `:22918-22921`); `half_speed` is `flag2` (mounts 56/61,
    /// `:22922-22925`). The tile kill and the dust are queued for the caller.
    fn sticky_movement(&mut self, world: &World, cart_ignore: bool, half_speed: bool, rng: &mut dyn tr_core::loot::DropRng) {
        // `:22912-22915`: the C# returns while `Player.shimmering` (the absorbed
        // state). No port counterpart yet (the shimmer absorption pass is
        // unported), so the gate is absent rather than always-false.
        // `:22929-22930`: the probe sits 0.01 px against gravity. The port has
        // no gravity flip, so `gravDir` is the vanilla `1f` (`Player.cs:1956`).
        let Some((num3, num4, web_puff)) = crate::collision::sticky_tiles(
            world,
            self.x,
            self.y - 0.01,
            self.width,
            self.body_height(),
            self.vx,
            self.vy,
            rng,
        ) else {
            // `:23083-23086`: no web under the feet clears the counter.
            self.sticky_break = 0;
            return;
        };
        // `Collision.cs:3403`: the `Next(30)` web puff (`DustID.Web` = 30), a
        // 16x16 spawn on the tile corner.
        if web_puff {
            self.pending_sticky_dust.push(StickyDust {
                pos: Vec2::new((num3 * 16) as f32, (num4 * 16) as f32),
                kind: 30,
                w: 16,
                h: 16,
                post_arms: false,
            });
        }
        let tile_type = world.tile(num3, num4).map_or(0, |t| t.kind as i32);
        // `:22936-22955`: the cobweb's random break. `whoAmI == Main.myPlayer`
        // collapses (the port is single-player, `Player.cs:2835`-style).
        if tile_type == 51 && (self.vx != 0.0 || self.vy != 0.0) {
            self.sticky_break += 1;
            // `:22939-22945`: the threshold band, wider on the gel/slime mounts.
            let (min_value, max_value) = if half_speed { (80, 300) } else { (20, 100) };
            // `:22946`: the C#'s bitwise `|` — both sides evaluate, so the draw
            // happens even when `flag` short-circuits the assignment.
            let roll = rng.next_range(min_value, max_value);
            if self.sticky_break > roll || cart_ignore {
                self.sticky_break = 0;
                self.pending_sticky_kill = Some((num3, num4));
            }
        }
        // `:22956-22959`: a cart above the drag speed leaves the velocity alone.
        if cart_ignore {
            return;
        }
        // `:22960`: the fall origin refreshes on the sticky surface.
        self.fall_start = (self.y / TILE_PX) as i32;
        // `:22961-22964`: a honey block keeps the jump; the web cancels it.
        if tile_type != 229 {
            self.jump = 0;
        }
        if half_speed {
            // `:22965-22972`: the gel/slime mounts slow to half, with the tiny
            // floor so the body never freezes completely.
            self.vx *= 0.5;
            self.vy *= 0.5;
            let len = (self.vx * self.vx + self.vy * self.vy).sqrt();
            if len < 0.0025 {
                // `Velocity.SafeNormalize(Vector2.Zero) * 0.0025`: a zero
                // vector normalises to zero, so the product is zero.
                if len > 0.0 {
                    self.vx = self.vx / len * 0.0025;
                    self.vy = self.vy / len * 0.0025;
                } else {
                    self.vx = 0.0;
                    self.vy = 0.0;
                }
            }
        } else {
            // `:22975-22990`: the X drag — the ±1 cap then the 0.85/0.6 damp.
            if self.vx > 1.0 {
                self.vx = 1.0;
            }
            if self.vx < -1.0 {
                self.vx = -1.0;
            }
            if self.vx > 0.75 || self.vx < -0.75 {
                self.vx *= 0.85;
            } else {
                self.vx *= 0.6;
            }
            // `:22991-23028`: the Y drag, mirrored for the two gravity signs.
            // The port has no gravity flip, so only the `gravDir == 1` arm runs.
            if self.vy > 1.0 {
                self.vy = 1.0;
            }
            if self.vy < -5.0 {
                self.vy = -5.0;
            }
            if self.vy < 0.0 {
                self.vy *= 0.96;
            } else {
                self.vy *= 0.3;
            }
        }
        // `:23030-23033`: the dust only for the honey block, one tick in five,
        // and only while moving vertically.
        if tile_type != 229 || rng.next_range(0, 5) != 0 || !(self.vy > 0.15 || self.vy < 0.0) {
            return;
        }
        // `:23034-23081`: the puff sits on the side the body's centre is away
        // from; one row, then the two rows below it while they are still 229.
        let centre_x = self.x + self.width * 0.5;
        // `(num3*16) < position.X + width/2` picks the left puff (`:23034`).
        let puff_x = if ((num3 * 16) as f32) < centre_x {
            self.x - 4.0
        } else {
            self.x + self.width - 2.0
        };
        // `Dust.NewDust(pos, 4, 16, 153, 0f, 0f, 50)` (`:23036`) + the
        // `Next(0,6)`/`velocity *= 0.1`/`noGravity` tail (`:23037-23039`).
        let honey = |row: i32| StickyDust {
            pos: Vec2::new(puff_x, (row * 16) as f32),
            kind: 153,
            w: 4,
            h: 16,
            post_arms: true,
        };
        self.pending_sticky_dust.push(honey(num4));
        // `:23048-23064`: the row below, gated on `type == 229` and the body
        // still overlapping it.
        if world.tile(num3, num4 + 1).map_or(0, |t| t.kind as i32) == 229
            && self.y + self.body_height() > ((num4 + 1) * 16) as f32
        {
            self.pending_sticky_dust.push(honey(num4 + 1));
        }
        // `:23065-23080`: the second row below, same gates.
        if world.tile(num3, num4 + 2).map_or(0, |t| t.kind as i32) == 229
            && self.y + self.body_height() > ((num4 + 2) * 16) as f32
        {
            self.pending_sticky_dust.push(honey(num4 + 2));
        }
    }

    pub(crate) fn update_survival(&mut self, world: &World, rng: &mut dyn tr_core::loot::DropRng) {
        // The Wet-buff effects and the DoT derivation are the flag sources the
        // C#'s `UpdateBuffs` chain leaves behind for `UpdateLifeRegen`
        // (`Player.cs:11740-11751, 18949-19103`).
        self.refresh_wet_state();
        let dots = self.derive_dot_inputs(self.buffs.expert_mode);

        self.check_drowning(world);

        // `Player.Update` → `UpdateLifeRegen` / `UpdateManaRegen`
        // (`Player.cs:25977,25995`). The aura/item/grapple inputs stay at their
        // defaults until those systems land (`regen.rs`); the hp gate stands in
        // for the original's dead-player check (M2-DEATH).
        if self.hp > 0 {
            let in_ = RegenInputs {
                max_life: self.max_hp,
                max_mana: self.mana_max,
                mana_regen_bonus: self.mana_regen_bonus,
                mana_regen_buff: self.mana_regen_buff,
                well_fed: self.well_fed,
                campfire: self.auras.campfire,
                heart_lantern: self.auras.heart_lantern,
                moving_x: self.vx != 0.0,
                standing_still: self.vx.abs() < 0.05 && self.vy.abs() < 0.05,
                dot_regen: dots.regen,
                dot_resets_time: dots.resets_time,
                dot_fast_lane: dots.fast_lane,
                dot_death_fast: dots.death_fast,
                dot_death_generic: dots.death_generic,
                sitting_or_sleeping: self.sitting || self.sleeping,
                extra_regen: self.accessory_life_regen,
                // S1-C-12 wave D — the Crimson set's regen halves
                // (`ArmorSetBonuses.cs:248-251` grants the flag):
                // `lifeRegenTime++` (`Player.cs:19117-19120`) and the ramp
                // ×1.5 (`:19242-19245`), in the engine's own input slots.
                extra_time: if self.crimson_regen { 1.0 } else { 0.0 },
                extra_ramp_scale: if self.crimson_regen { 1.5 } else { 1.0 },
                ..RegenInputs::default()
            };
            if let Some(death) = update_life_regen(&mut self.regen, &in_, &mut self.hp) {
                self.dot_death = Some(death);
            }
            update_mana_regen(
                &mut self.regen,
                &in_,
                &mut self.mana,
                crate::regen::ManaRegenFlags {
                    item_in_use: self.mana_regen_item_in_use,
                },
            );
        }

        // The lava hit and the water extinguish both trail `UpdateLifeRegen` in
        // the original frame (`Player.cs:27630-27761` > `:25977`): a DoT kill
        // this tick wins over the lava hit.
        // `Player.cs:27681-27722`: the lava timer. Standing in lava drains one
        // tick of the immunity budget per frame; only when it is empty does the
        // 80-damage hit land (and grant On Fire for 420). Out of the lava the
        // budget refills up to `lavaMax` — the Lava Charm family's 420 ticks.
        // `Player.cs:27676`: `flag25 && !lavaImmune` — ObsidianSkin (buff 1)
        // keeps the flag true, so the drain/hit arm is skipped AND `flag26`
        // stays true (the budget refills while standing in lava).
        if self.in_lava && !self.lava_immune {
            if self.lava_time > 0 {
                self.lava_time -= 1;
            } else if self.hurt_cd <= 0.0 {
                let cx = self.x + self.width * 0.5;
                // `Player.cs:27687-27707`: the hit's damage/burn pair —
                // 80/420 classic, halved by the Ash Wood set
                // (`ashWoodBonus`: `num79 /= 2`, `num80 -= 210`,
                // `:27694-27702`, S1-C-12 wave D). The remix-world
                // variants (`:27689-27693`, `:27696-27699`) and the
                // `lavaRose` halves (`:27703-27707`) have no port fields
                // and stay out (ANALYSIS "S1-C-12 wave D").
                let (mut num79, mut num80) = (80, 420);
                if self.ash_wood_bonus {
                    num79 /= 2;
                    num80 -= 210;
                }
                // The lava hit rolls the dodge block like any `Hurt` — the
                // stream rides in from the `UpdatePlayers` stage through the
                // update chain, `dodgeable` keeps `Hurt`'s default `true`
                // (research S1-C-1 §1.4). `Crit: false` and the
                // `ImmunityCooldownID.Lava` counter (`Player.cs:27708`) — Lava
                // is NOT in `Sets.Retaliate` (`ImmunityCooldownID.cs:49`), so
                // the S1-C-7 wave B retaliate lane stays off.
                if self.hurt_inner(num79, cx, true, rng, true, false, None, false) {
                    // The Hurt return `num81 > 0.0` gates the OnFire grant
                    // (`Player.cs:27709-27711`). `AddBuff` applies the expert
                    // ×2 itself (24 is in the difficulty set, `BuffID.cs:50`).
                    self.buffs.add_buff(24, num80);
                }
            }
        } else if self.lava_time < self.lava_max {
            self.lava_time += 1;
        }
        if self.lava_time > self.lava_max {
            self.lava_time = self.lava_max;
        }
        // Water extinguishes only OnFire (24) and OnFire3 (323) — Frostburn,
        // CursedInferno and Burning are immune to it — and only while not
        // standing in lava (`Player.cs:27750-27761`). Deviation (Assumption-2
        // fallback): the C# `WetCollision` flag covers honey too; the port
        // conditions on the water-only `wet` flag, distinct from honey
        // (docs/ANALYSIS.md, Phase 4). The C# loop does not rescan after
        // `DelBuff` compaction.
        if self.wet && !self.lava_wet {
            for i in 0..MAX_BUFFS {
                let t = self.buffs.buff_type[i];
                if t == 24 || t == 323 {
                    self.buffs.del_buff(i);
                }
            }
        }
    }

    /// The original's movement pipeline (`Player.cs`): `TileCollision` resolves the
    /// AABB against solid tiles and slope triangles, `StepDown`/`StepUp` lift the
    /// hitbox over one-tile ledges while `gfxOffY` hides the pop, and
    /// `SlopeCollision` slides the feet along ramps.
    fn move_vanilla(&mut self, world: &World, down: bool, gravity: f32) {
        // `Player.cs`: the visual offset eases back to zero at walk speed.
        let decay = (1.0 + self.vx.abs() / 3.0) * self.step_speed;
        if self.gfx_off_y > 0.0 {
            self.gfx_off_y = (self.gfx_off_y - decay).max(0.0);
        } else if self.gfx_off_y < 0.0 {
            self.gfx_off_y = (self.gfx_off_y + decay).min(0.0);
        }
        self.gfx_off_y = self.gfx_off_y.clamp(-32.0, 32.0);

        let mut pos = [self.x, self.y];
        let mut vel = [self.vx, self.vy];
        // `Player.cs:28082-28084`: the pre-movement velocity snapshot and the
        // slideDir reset — the post-SlopingCollision compare (`:28242-28252`)
        // re-arms the Rat mount's wall-slide direction.
        let pre_move_vx = self.vx;
        self.slide_dir = 0;
        // The live hitbox width (`Player.width`): the transformation mounts
        // shrink it (Rat 14, Pixie 8 — `Mount.cs:6244-6263`), so the collision
        // body reads it, not the 20 constant.
        let body_w = self.width;
        // `Player.cs` runs the step helpers on the pre-collision velocity (which
        // still carries this tick's gravity), then resolves the AABB. The wet
        // path never steps — `WetCollision` goes straight to `TileCollision`
        // (`Player.cs:23443`), the step helpers are `DryCollision` internals.
        // Both helpers are skipped while riding tracks (`IsRidingTracks`,
        // `Player.cs:28057-28068` — a mounted cart always is) and inside
        // `DryCollision` for any cart (`:23529-23541`).
        let body_h = self.body_height();
        let dry = self.liquid_step == 1.0;
        if dry && vel[1] == gravity && !self.cart_mounted {
            collision::step_down(
                world,
                &mut pos,
                &mut vel,
                body_w,
                body_h,
                &mut self.step_speed,
                &mut self.gfx_off_y,
            );
        }
        if dry && vel[1] >= gravity && !down && !self.cart_mounted {
            collision::step_up(
                world,
                &mut pos,
                &mut vel,
                body_w,
                body_h,
                &mut self.step_speed,
                &mut self.gfx_off_y,
                false,
            );
        }
        // `Player.TileCollision` (`Player.cs:23596-23603`): the collision body
        // rides `height − 10` while on tracks — the cart sprite owns the bottom
        // 10 px (pitfall §10.9).
        let col_h = if self.on_track { body_h - 10.0 } else { body_h };
        // Player.cs:28085-28092: allThrough = controlDown, then
        // lag31 |= gravDir == -1 | cart-mount | GoingDownWithGrapple | pulley
        // and that arm sets BOTH true. The cart always falls through platforms;
        // a downward grapple pull (GoingDownWithGrapple, :22655-22658) and
        // the pulley do too. The port keeps gravDir = 1; the down key feeds
        // allThrough only (lag31 stays false unless a force is on).
        // (M3 sync: the collision width is the LIVE ody_w - the mount's
        // PlayerSize delegates resize it, S1-P-i41.)
        let (fall_through, fall2) =
            if self.cart_mounted || self.going_down_with_grapple || self.pulley {
                (true, true)
            } else {
                (down, false)
            };
        let tc = collision::tile_collision(world, pos, vel, body_w, col_h, fall_through, fall2);
        if tc.up {
            self.jump = 0;
        }
        // `Player.WetCollision` (`Player.cs:23436-23451`): the collision sees
        // the unscaled velocity; the free axes then integrate at the liquid
        // fraction (`liquid_step` — water/lava 0.5, honey 0.25, shimmer 0.375)
        // while an axis the collision overrode keeps its full velocity, so
        // momentum survives the water.
        let pre = vel;
        vel = tc.vel;
        let ms = self.liquid_step;
        pos[0] += if vel[0] != pre[0] {
            vel[0]
        } else {
            vel[0] * ms
        };
        pos[1] += if vel[1] != pre[1] {
            vel[1]
        } else {
            vel[1] * ms
        };
        // `Player.SlopingCollision` (`Player.cs:23677-23683`): `stairFall` arms
        // on `ignorePlats || controlDown || grappling[0] >= 0 || gravDir == -1`;
        // the port keeps `gravDir = 1`, `ignorePlats` is `fall2` above.
        if fall2 || down || self.grappling_active {
            self.stair_fall = true;
        }
        // `Player.SlopingCollision(fallThrough, flag31)` (`Player.cs:28210`):
        // the cart's platform pass-through rides here too.
        let sc = collision::slope_collision(
            world,
            pos,
            vel,
            body_w,
            body_h,
            self.stair_fall,
        );
        self.stair_fall = sc.stair_fall;
        if sc.stair && (sc.pos[1] - pos[1]).abs() > 8.0 + vel[0].abs() {
            self.gfx_off_y -= sc.pos[1] - pos[1];
            self.step_speed = 4.0;
        }
        pos = sc.pos;
        vel = sc.vel;
        // `Player.cs:28225-28232`: the belt pass rides right after
        // `SlopingCollision`, skipped while shimmering/tongued (and while
        // locked to a tile - a state the port has no field for).
        // ponytail: `!shimmering` guard dropped - the port has no
        // `Player.shimmering` (the absorbed state, S1-E-16); add it there.
        if !self.holding(38) {
            collision::step_conveyor_belt_player(
                world,
                &mut pos,
                body_w as i32,
                body_h as i32,
                1.0,
                self.gfx_off_y,
                self.grappling_active,
                self.pulley,
                self.on_track,
            );
        }
        // `Player.cs:28242-28252`: the slide direction — the pre-movement
        // velocity's sign when the collision changed the X axis.
        if pre_move_vx != vel[0] {
            if pre_move_vx < 0.0 {
                self.slide_dir = -1;
            } else if pre_move_vx > 0.0 {
                self.slide_dir = 1;
            }
        }
        self.x = pos[0];
        self.y = pos[1];
        self.vx = vel[0];
        self.vy = vel[1];
        // The original gates jumps on `velocity.Y == 0f` after collision.
        self.on_ground = self.vy == 0.0;
    }

    // fix/player-motion-blur: the old `interpolated(alpha)` render blend was
    // REMOVED — it had no C# counterpart. `Main.DrawPlayers` hands the raw
    // `player.position` to `LegacyPlayerRenderer` (`Main.cs:22298`,
    // `LegacyPlayerRenderer.cs:468-475`: `position + netOffset + (0, gfxOffY)`,
    // no `oldPosition` lerp), the layers truncate the screen position per
    // component (`(int)(drawinfo.Position.X - Main.screenPosition.X + ...)`,
    // `PlayerDrawLayers.cs:1907/1939`), and `Main.screenPosition` is itself
    // `(int)`-truncated (`Main.cs:62646-62647`). Drawing between-tick blends
    // under nearest sampling re-rasterised the sprite on a shifted pixel grid
    // every frame — the user-reported "character smears during any movement".
    // The draw path (`app/frame/draw.rs::draw_region`) now clones the raw
    // fixed-step state; `prev_x`/`prev_y` remain for the pressure-plate bridge
    // (`PressurePlateHelper.PlayerLastPosition`, `PressurePlateHelper.cs:16`).
    pub fn ammo_flags(&self) -> crate::ammo::AmmoFlags {
        crate::ammo::AmmoFlags {
            has_molten_quiver: self.has_molten_quiver,
            magic_quiver: self.magic_quiver,
            acc_sharp_barb: self.acc_sharp_barb,
            archery: self.archery,
            ammo_box: self.ammo_box,
            ammo_potion: self.ammo_potion,
            huntress_ammo_cost90: self.huntress_ammo_cost90,
            chloro_ammo_cost80: self.chloro_ammo_cost80,
            ammo_cost80: self.ammo_cost80,
            ammo_cost75: self.ammo_cost75,
        }
    }

    /// Deterministic identity for the G6 world-range cache.
    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.facing as u32) << 1;
        h ^= self.on_ground as u32;
        h ^= self.leg_frame << 8;
        h ^= self.body_frame << 16;
        // The draw bakes `gfxOffY` into the sprite's screen position
        // (`LegacyPlayerRenderer.cs:457,468`), so its ease after a step must
        // invalidate the cached range or the offset animation freezes.
        h ^= self.gfx_off_y.to_bits().wrapping_mul(0x27D4_EB2F);
        // The mount's animator state feeds the mount sprites' pixels
        // (`Mount.Draw`'s frame rect + the per-frame player offset).
        if self.mount.is_active() {
            h ^= (self.mount.kind() as u32) << 3;
            h ^= (self.mount.frame() as u32).wrapping_mul(0x9E37_79B1);
            h ^= (self.mount.frame_state() as u32) << 20;
            h ^= (self.mount.player_offset() as u32) << 26;
        }
        h
    }
}

/// Player movement collision: ground slopes (slope 1/2) are handled by `slope_floor`.
pub fn collides_player_rect(world: &World, x: f32, y: f32) -> bool {
    collides_filtered(world, x, y, WIDTH, HEIGHT, true)
}

pub fn collides_player(world: &World, x: f32, y: f32) -> bool {
    collides_player_rect(world, x, y)
}

/// Generic solid-tile AABB test used by NPCs and the player.
pub fn collides_rect(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    collides_filtered(world, x, y, w, h, false)
}

/// `Collision.WetCollision` (`Collision.cs:1572-1651`): true when a liquid
/// tile's surface overlaps the inset probe rectangle. The C# `honey`/`shimmer`
/// out-params have no consumer here and are not modelled. The bat mount's
/// activation gate calls this (`Mount.cs:6595`).
pub fn wet_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let wi = w as i32;
    let hi = h as i32;
    let inset_w = 10.min(wi);
    let inset_h = (hi / 2).min(hi);
    let px = x + (wi / 2) as f32 - (inset_w / 2) as f32;
    let py = y + (hi / 2) as f32 - (inset_h / 2) as f32;
    let max_y = (world.height - 40).max(0);
    let x0 = (((x / TILE_PX) as i32) - 1).clamp(0, world.width - 1);
    let x1 = ((((x + wi as f32) / TILE_PX) as i32) + 2).clamp(0, world.width - 1);
    let y0 = (((y / TILE_PX) as i32) - 1).clamp(0, max_y);
    let y1 = ((((y + hi as f32) / TILE_PX) as i32) + 2).clamp(0, max_y);
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            let left = tx as f32 * TILE_PX;
            if t.liquid > 0 {
                // `num5 = (256 - liquid) / 32`; the surface rises by
                // `num5 * 2` and the solid part shrinks by the same amount.
                let num5 = (256 - t.liquid as i32) as f32 / 32.0;
                let top = ty as f32 * TILE_PX + num5 * 2.0;
                let depth = 16.0 - (num5 * 2.0) as i32 as f32;
                if px + inset_w as f32 > left
                    && px < left + 16.0
                    && py + inset_h as f32 > top
                    && py < top + depth
                {
                    return true;
                }
            } else if t.active()
                && t.slope() != 0
                && ty > 0
                && world.tile(tx, ty - 1).is_some_and(|above| above.liquid > 0)
            {
                // The sloped-tile branch (`:1625-1645`): the liquid above a
                // slope still counts even though this cell is dry.
                if px + inset_w as f32 > left
                    && px < left + 16.0
                    && py + inset_h as f32 > ty as f32 * TILE_PX
                    && py < ty as f32 * TILE_PX + 16.0
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `Collision.LavaCollision` (`Collision.cs:1653-1684`): true when a LAVA
/// tile's surface overlaps the full probe rectangle (no inset — unlike
/// `WetCollision`). The no-space teleport scan refuses lava spots
/// (`TeleportHelpers.cs:34`).
pub fn lava_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let max_y = (world.height - 40).max(0);
    let x0 = (((x / TILE_PX) as i32) - 1).clamp(0, world.width - 1);
    let x1 = ((((x + w) / TILE_PX) as i32) + 2).clamp(0, world.width - 1);
    let y0 = (((y / TILE_PX) as i32) - 1).clamp(0, max_y);
    let y1 = ((((y + h) / TILE_PX) as i32) + 2).clamp(0, max_y);
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava {
                let left = tx as f32 * TILE_PX;
                let num3 = (256 - t.liquid as i32) as f32 / 32.0;
                let top = ty as f32 * TILE_PX + num3 * 2.0;
                let depth = 16.0 - (num3 * 2.0) as i32 as f32;
                if x + w > left && x < left + 16.0 && y + h > top && y < top + depth {
                    return true;
                }
            }
        }
    }
    false
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Main.wallDungeon` (`Main.cs:10598-10606`): the nine dungeon wall ids.
fn wall_dungeon(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94 | 95 | 96 | 97 | 98 | 99)
}

/// `TeleportHelpers.TileIsDangerous` (`TeleportHelpers.cs:241-261`): lava,
/// the pre-Plantera dungeon wall 87, the pre-Skeletron dungeon walls, and
/// any hurting tile. The world's seed flags feed `CanTileHurt`'s 230/80 arms.
fn teleport_tile_is_dangerous(world: &World, x: i32, y: i32, fire_walk: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava {
        return true;
    }
    if t.wall == 87 && (y as f64) > world.world_surface && !world.downed_plant_boss {
        return true;
    }
    if wall_dungeon(t.wall) && (y as f64) > world.world_surface && !world.downed_skeletron {
        return true;
    }
    let good_world = world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world.seeds.contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    if t.active() && crate::touch_damage::can_tile_hurt(t.kind, good_world, dont_starve, fire_walk) {
        return true;
    }
    false
}

/// `Utils.Clamp` (`Utils.cs`): the C# compares against MAX first, then min —
/// unlike Rust's `.clamp` it never panics when `min > max` (a small world's
/// `40 .. width-40` window can invert).
fn utils_clamp(value: i32, min: i32, max: i32) -> i32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// `TeleportHelpers.FindClosestTeleportSpotNoSpace` (`TeleportHelpers.cs:8-47`):
/// the ±25-tile scan around the body's bottom for the closest open landing
/// spot — a free cell over a solid dry floor, no lava / hurting tiles / solid
/// overlap. Zeroes the velocity as a side effect (`:12`), like the C#.
pub fn find_closest_teleport_spot_no_space(world: &World, player: &mut Player) -> Option<(f32, f32)> {
    player.vx = 0.0;
    player.vy = 0.0;
    let half_w = player.width * 0.5;
    let h = player.height;
    let bottom_x = player.x + half_w;
    let bottom_y = player.y + h;
    let px = (bottom_x / TILE_PX) as i32;
    let py = (bottom_y / TILE_PX) as i32;
    let x0 = utils_clamp(px - 25, 40, world.width - 40);
    let x1 = utils_clamp(px + 25, 40, world.width - 40);
    let y0 = utils_clamp(py - 25, 40, world.height - 40);
    let y1 = utils_clamp(py + 25, 40, world.height - 40);
    let good_world = world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world.seeds.contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    let mut best: Option<(f32, f32)> = None;
    let mut best_dist = f32::MAX;
    for i in x0..x1 {
        for j in y0..y1 {
            // `:29` — the candidate top-left: cell centre-x, floor-top-y.
            let cx = i as f32 * TILE_PX + 8.0 - half_w;
            let cy = j as f32 * TILE_PX + 15.0 - h;
            let Some(tile) = world.tile(i, j) else { continue };
            let Some(tile2) = world.tile(i, j + 1) else { continue };
            let flag = collision::solid_or_sloped_tile(tile, false) || tile.liquid > 0;
            let flag2 = collision::solid_or_sloped_tile(tile2, false) && tile2.liquid == 0;
            if !teleport_tile_is_dangerous(world, i, j, player.fire_walk)
                && !flag
                && flag2
                && !lava_collision(world, cx, cy, player.width, h)
                // `Collision.AnyHurtingTiles` (`Collision.cs:3110-3113`)
                // calls `HurtTiles(..., null)` — a NULL player: the hot-tile
                // arm (`CanTileHurt` `:3220`, `player == null || !fireWalk`)
                // always hurts, so fire_walk = false here.
                && crate::touch_damage::hurt_tiles(
                    world,
                    cx,
                    cy,
                    player.width as i32,
                    h as i32,
                    good_world,
                    dont_starve,
                    false,
                )
                .is_none()
                && !Player::solid_collision(world, cx, cy, player.width, h)
            {
                let dist = Vec2::new(cx - bottom_x, cy - bottom_y).length();
                if dist < best_dist {
                    best = Some((cx, cy));
                    best_dist = dist;
                }
            }
        }
    }
    best
}

/// `Player.Spawn_IsAreaValidSpawn` (`Player.cs:37937-37985`): the 3×3 area
/// above the floor cell free of solids / liquid / pressure plates / hurting
/// tiles. The `generatingSpawn` dual-dungeons arm (`:37978`) is unported.
fn spawn_is_area_valid_spawn(world: &World, floor_x: i32, floor_y: i32) -> bool {
    let good_world = world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world.seeds.contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    for i in floor_x - 1..floor_x + 2 {
        for j in floor_y - 3..floor_y {
            if !in_world(world, i, j, 5) {
                continue;
            }
            let Some(tile) = world.tile(i, j) else { continue };
            if j == floor_y - 1 {
                if let Some(tile2) = world.tile(i, j + 1) {
                    if tile2.active() {
                        if tile2.kind == 135 {
                            return false;
                        }
                        // `CanTileHurt(..., null)` — fire_walk = false.
                        if crate::touch_damage::can_tile_hurt(tile2.kind, good_world, dont_starve, false)
                        {
                            return false;
                        }
                    }
                }
            }
            if tile.active() && !tile.inactive() {
                if tile.kind == 135 {
                    return false;
                }
                if td::TILE_SOLID.has(tile.kind) && !td::TILE_SOLID_TOP.has(tile.kind) {
                    return false;
                }
            }
            if tile.liquid > 0 {
                return false;
            }
        }
    }
    true
}

/// `Player.Spawn_ValidSpawnFloor` (`Player.cs:37896-37912`): a solid
/// non-solid-top tile, a platform, or tile 380 — but never 379 or a plate.
/// The C# reads `tile.nactive()` (active && !inActive, `Tile.cs:240-247`).
fn spawn_valid_spawn_floor(world: &World, tx: i32, ty: i32) -> bool {
    let Some(tile) = world.tile(tx, ty) else {
        return false;
    };
    if tile.active() && tile.kind == 135 {
        return false;
    }
    let nactive = tile.active() && !tile.inactive();
    if nactive
        && ((td::TILE_SOLID.has(tile.kind) && !td::TILE_SOLID_TOP.has(tile.kind))
            || td::SET_PLATFORMS.has(tile.kind)
            || tile.kind == 380)
    {
        return tile.kind != 379;
    }
    false
}

/// `Player.Spawn_DescendFromDefaultSpace_Inner` (`Player.cs:37831-37894`):
/// the 50-row descent for a valid 3-wide floor, with the single-block
/// fallback. Returns `(point, valid_space)`.
fn spawn_descend_inner(world: &World, x: i32, y: i32, single_block_allowed: bool) -> ((i32, i32), bool) {
    let mut x = x;
    for i in 0..50 {
        let mut flag = false;
        let mut flag2 = false;
        let mut flag3 = false;
        let num = y + i;
        let mut num2 = 0;
        for j in -1..=1 {
            let num3 = x + j + num2;
            if !in_world(world, num3, num, 5) || world.tile(num3, num - 1).is_none() {
                continue;
            }
            if let Some(tile) = world.tile(num3, num) {
                if tile.active() && !tile.inactive() && tile.kind == 135 {
                    return ((x, y), false);
                }
            }
            if spawn_valid_spawn_floor(world, num3, num) && spawn_is_area_valid_spawn(world, num3, num) {
                match j {
                    -1 => flag = true,
                    1 => flag3 = true,
                    _ => flag2 = true,
                }
            }
        }
        if !flag2
            && (single_block_allowed || flag)
            && spawn_valid_spawn_floor(world, x - 2, num)
            && spawn_is_area_valid_spawn(world, x - 1, num)
        {
            flag = true;
            flag2 = true;
            flag3 = false;
            num2 -= 1;
        } else if !flag2
            && (single_block_allowed || flag3)
            && spawn_valid_spawn_floor(world, x + 2, num)
            && spawn_is_area_valid_spawn(world, x + 1, num)
        {
            flag = false;
            flag2 = true;
            flag3 = true;
            num2 += 1;
        }
        if flag2 && (single_block_allowed || flag || flag3) {
            x += num2;
            return ((x, y + i), true);
        }
    }
    ((x, y), false)
}

/// `Player.Spawn_DescendFromDefaultSpace` (`Player.cs:37818-37829`).
fn spawn_descend_from_default_space(world: &World, x: i32, y: i32) -> (i32, i32) {
    let (point, valid) = spawn_descend_inner(world, x, y, false);
    if !valid {
        let (point, _) = spawn_descend_inner(world, x, y, true);
        return point;
    }
    point
}

/// `Player.Spawn_GetPositionAtSpawn` (`Player.cs:37723-37804`): the spawn
/// floor search — the ±30-row up/down scan when the area is invalid, then
/// the descent. Returns the floor cell.
fn spawn_get_position_at_spawn(world: &World, spawn_x: i32, spawn_y: i32) -> (i32, i32) {
    let mut num = spawn_x;
    let mut num2 = spawn_y;
    if !spawn_is_area_valid_spawn(world, num, num2) {
        let mut flag2 = false;
        for i in 0..30 {
            if spawn_is_area_valid_spawn(world, num, num2 - i) {
                num2 -= i;
                flag2 = true;
                break;
            }
            if spawn_is_area_valid_spawn(world, num - 1, num2 - i) {
                num -= 1;
                num2 -= i;
                flag2 = true;
                break;
            }
            if spawn_is_area_valid_spawn(world, num + 1, num2 - i) {
                num += 1;
                num2 -= i;
                flag2 = true;
                break;
            }
        }
        if !flag2 {
            for j in 0..30 {
                if spawn_is_area_valid_spawn(world, num, num2 + j) {
                    num2 += j;
                    flag2 = true;
                    break;
                }
                if spawn_is_area_valid_spawn(world, num - 1, num2 + j) {
                    num -= 1;
                    num2 += j;
                    flag2 = true;
                    break;
                }
                if spawn_is_area_valid_spawn(world, num + 1, num2 + j) {
                    num += 1;
                    num2 += j;
                    flag2 = true;
                    break;
                }
            }
        }
        if flag2 {
            return spawn_descend_from_default_space(world, num, num2);
        }
        return (num, num2);
    }
    spawn_descend_from_default_space(world, num, num2)
}

/// `Player.Spawn_GetPositionAtWorldSpawn` (`Player.cs:37718-37721`).
fn spawn_get_position_at_world_spawn(world: &World, spawn_x: i32, spawn_y: i32) -> (i32, i32) {
    spawn_get_position_at_spawn(world, spawn_x, spawn_y)
}

fn collides_filtered(world: &World, x: f32, y: f32, w: f32, h: f32, ignore_slopes: bool) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if (t.kind as usize) >= td::TILE_COUNT || !td::TILE_SOLID.has(t.kind) {
                continue;
            }
            if ignore_slopes {
                if t.slope() == 1 || t.slope() == 2 {
                    continue; // ground ramps are handled by `slope_floor`
                }
                if t.half_brick() {
                    // Only the bottom 8 px are solid.
                    if y + h > ty as f32 * TILE_PX + 8.0 {
                        return true;
                    }
                    continue;
                }
                if t.slope() >= 3 {
                    // Ceiling ramp: solid above the diagonal, empty below it.
                    let base_x = tx as f32 * TILE_PX;
                    let base_y = ty as f32 * TILE_PX;
                    let sx0 = x.max(base_x);
                    let sx1 = (x + w).min(base_x + TILE_PX);
                    let mut cx = sx0;
                    while cx < sx1 {
                        let local = cx - base_x;
                        let boundary = if t.slope() == 3 {
                            base_y + TILE_PX - local
                        } else {
                            base_y + local
                        };
                        if y < boundary && y + h > base_y {
                            return true;
                        }
                        cx += 2.0;
                    }
                    continue;
                }
            }
            return true;
        }
    }
    false
}

#[cfg(test)]

mod tests {
    #[test]
    fn dye_cache_mirrors_update_dyes() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        // Empty slots → the C# zero-fill (`UpdateDyes`, `:9589-9592`).
        p.update_dye_cache();
        assert_eq!(p.c_head, 0);
        // `cHead/cBody/cLegs = GetEffectiveDye(0..2).dye` (`:9593-9595`): red
        // dye (1007) on the head slot, blue (1015) on the legs, nothing on
        // the body.
        p.dye_slots[0] = 1007;
        p.dye_slots[2] = 1015;
        p.misc_dye_slots[3] = 1007;
        p.update_dye_cache();
        assert!(p.c_head > 0);
        assert_eq!(p.c_body, 0);
        assert!(p.c_legs > 0 && p.c_legs != p.c_head);
        // The same dye lands on the same slot everywhere (`cMount` mirrors
        // `miscDyes[3].dye`, `:9600-9604`).
        assert_eq!(p.c_mount, p.c_head);
        // The misc five keep `:9600-9604`'s pet/light/cart/mount/hook order.
        p.misc_dye_slots[0] = 1015;
        p.update_dye_cache();
        assert!(p.c_pet > 0);
        assert_eq!(p.c_light, 0);
    }

    /// S1-C-4 — `GetWeaponDamageMultiplier`'s ranged dispatch
    /// (`Player.cs:53396-53398`) keys on the ammo category, and the
    /// effective-damage components (`:3982-3986`) move it off the plain
    /// `rangedDamage` once the C#'s arming sites fire: the Archery buff
    /// (`:10019`), the quivers (`:14466`/`:14484`/`:14598`/`:15334`) and the
    /// Shroomite helmets (`:13697-13712`).
    #[test]
    fn s1_c_4_the_ranged_multiplier_dispatches_by_ammo_category() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // A plain gun (bullet ammo 97): the plain rangedDamage — every
        // component at its reset value.
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.0
        );
        // An arrow weapon (ammo 40): bowEffectiveDamage
        // `(ranged / mult + stack) * mult * arrowDamage` = 1.0 at the resets.
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40),
            1.0
        );
        // The arming: the Archery buff (×1.1) and a Magic Quiver (+0.1 stack)
        // move the bow arm off 1.0 while the gun arm stays on rangedDamage.
        p.arrow_damage *= 1.1; // the buff arm (`:10019`)
        p.arrow_damage_additive_stack += 0.1; // the quiver arm (`:14466`)
        let bow = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40);
        assert!((bow - (1.0 / 1.0 + 0.1) * 1.0 * 1.1).abs() < 1e-6, "{bow}");
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.0,
            "the gun arm does not read the arrow components"
        );
        // The Shroomite Bullet helmet (`:13704`): the gun arm reads
        // bulletDamage; the bow arm does not.
        p.bullet_damage *= 1.12;
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.12
        );
        let bow2 = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40);
        assert!(
            (bow2 - bow).abs() < 1e-6,
            "the helmet does not touch the bow"
        );
        // The specialist arm: rocket ammo 771 and the
        // `IsRangedSpecialistWeapon` ids ride rocketDamage (`:3986`) — but
        // only after the arrow/bullet checks (the C#'s `if` order), so a
        // specialist weapon on arrow ammo stays on the bow arm.
        p.rocket_damage *= 1.12;
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 771),
            1.12
        );
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 1156, 931),
            1.12,
            "the specialist weapon on flare ammo rides rocketDamage"
        );
        let bow3 = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 1156, 40);
        assert!((bow3 - bow).abs() < 1e-6, "arrow ammo wins over the id");
        // The non-ranged classes keep their table (`:53390`/`:53410-53419`).
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Melee, 100, 40),
            1.0
        );
    }

    /// M3-EQUIP-5 — `Player.statDefense` is derived inside the tick, in the
    /// vanilla order: `ResetEffects` zeroes it (`Player.cs:18327`), the buff
    /// arms add, `GrantArmorBenefits` adds the piece rows (`:13121`) and the
    /// set bonus, `GrantPrefixBenefits` the accessory prefixes, then the floor
    /// (`:25927`).
    #[test]
    fn stat_defense_sums_pieces_set_bonus_and_buffs() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Nothing worn, nothing buffed: 0.
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0);
        // The copper set (89/80/76): 1 + 2 + 1 pieces + the MetalTier1 bonus 2.
        p.equipped[0] = 89;
        p.equipped[1] = 80;
        p.equipped[2] = 76;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 6, "copper set: 4 pieces + 2");
        // The accessories count too (`GetEffectiveArmor(0..10)`): the Shackle
        // (216) +1 and the Cobalt Shield (156) +1 sit in slots 3/4. (The Ankh
        // Shield would be +5 but it also grants immunity to Weak — see the
        // immunity test — which would change the debuff rows below.)
        p.equipped[3] = 216;
        p.equipped[4] = 156;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 8, "8 = 4 + 2 + 1 + 1");
        // Ironskin (buff 5) +8 and the fed tier (26) +2.
        p.buffs.add_buff(5, 3600);
        p.buffs.add_buff(26, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 18, "8 + 8 + 2");
        // Ichor (69) −15 and Weak (33) −4: the sum can go below the armor, and
        // the floor clamps it at 0 (`:25927`) instead of inflating damage.
        p.buffs.add_buff(69, 3600);
        p.buffs.add_buff(33, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0, "18 - 15 - 4 = -1, floored");
        p.equipped[0] = 0;
        p.equipped[1] = 0;
        p.equipped[2] = 0;
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0, "the floor holds at 0");
    }

    /// M3-EQUIP-20 — the armor-penetration rows and the melee-only split.
    #[test]
    fn the_penetration_rows_add_up() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 0);
        // The Shark Tooth Necklace (3212) and the Honey Comb (4007): +5 each,
        // both class-independent.
        p.equipped[3] = 3212;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 5);
        assert_eq!(p.get_armor_penetration(false), 5);
        p.equipped[4] = 4007;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(false), 10);
        // The Sharpened buff (159) adds 12 that only melee hits see.
        p.buffs.add_buff(159, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 22);
        assert_eq!(p.get_armor_penetration(false), 10);
    }

    /// S1-D-4b-2 — the sentry-backpack accessory grants (`ApplyEquipFunctional`,
    /// `Player.cs:14506-14508`/`:14564-14579`): the four slings arm
    /// `accSentryBackpack`; the Scout Sling (6175) adds +5% to all four damage
    /// classes and the Royal Guard's Harness (6177) the knockback immunity.
    /// `ResetEffects` clears the flag each tick (`:18481`), so taking the
    /// accessory off drops it.
    #[test]
    fn the_sentry_backpack_slings_grant_the_flag() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!(!p.acc_sentry_backpack, "nothing worn");
        // The Heavy Sling (6172): the bare flag.
        p.equipped[3] = 6172;
        p.update(&w, PlayerInput::default());
        assert!(p.acc_sentry_backpack, "6172 grants the flag");
        // The Scout Sling (6175): the flag + the four +5% damage adds.
        p.equipped[3] = 6175;
        p.update(&w, PlayerInput::default());
        assert!(p.acc_sentry_backpack);
        assert!((p.melee_damage - 1.05).abs() < 1e-4, "melee +5%");
        assert!((p.magic_damage - 1.05).abs() < 1e-4, "magic +5%");
        assert!((p.ranged_damage - 1.05).abs() < 1e-4, "ranged +5%");
        assert!((p.minion_damage - 1.05).abs() < 1e-4, "minion +5%");
        // The Royal Guard's Harness (6177): the flag + noKnockback.
        p.equipped[3] = 6177;
        p.update(&w, PlayerInput::default());
        assert!(p.acc_sentry_backpack);
        assert!(p.no_knockback, "6177 grants the Cobalt Shield's immunity");
        // Taking it off clears the flag (ResetEffects `:18481`).
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.acc_sentry_backpack, "the flag clears");
    }

    /// S1-P-i10 — the melee on-hit accessory flags
    /// (`ApplyEquipFunctional`, `Player.cs:15336-15339` Magma Stone 1322,
    /// `:14580-14583` Pyroclast 6178, `:15306-15314` Fire Gauntlet 1343,
    /// `:14615-14619` Catalyst Band 6185). `ResetEffects` clears both
    /// flags every tick (`:18608`/`:18610`), so taking the accessory off
    /// drops them.
    #[test]
    fn s1_p_i10_the_on_hit_accessories_grant_and_clear_their_flags() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!(!p.magma_stone && !p.catalyst_band, "nothing worn");
        // The Magma Stone (1322): the bare flag.
        p.equipped[3] = 1322;
        p.update(&w, PlayerInput::default());
        assert!(p.magma_stone, "1322 grants magmaStone (:15338)");
        assert!(!p.catalyst_band);
        // The Pyroclast (6178): the same flag (its accPyroclast half is
        // an unported projectile family).
        p.equipped[3] = 6178;
        p.update(&w, PlayerInput::default());
        assert!(p.magma_stone, "6178 grants magmaStone (:14581)");
        // The Fire Gauntlet (1343): the flag through the combat-family
        // grant path (`accessory_combat.rs::fire_gauntlet`).
        p.equipped[3] = 1343;
        p.update(&w, PlayerInput::default());
        assert!(p.magma_stone, "1343 grants magmaStone (:15313)");
        // Taking it off clears the flag (ResetEffects `:18610`).
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.magma_stone, "the flag clears");
        // The Catalyst Band (6185): catalystBand (its accPoisonBarb /
        // pStone halves are unported families).
        p.equipped[3] = 6185;
        p.update(&w, PlayerInput::default());
        assert!(p.catalyst_band, "6185 grants catalystBand (:14618)");
        assert!(!p.magma_stone);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.catalyst_band, "the flag clears (:18608)");
    }

    /// S1-D-4b-2 — the DD2 sentry armor sets' flags (`ArmorSetBonuses.Benefits`,
    /// `DataStructures/ArmorSetBonuses.cs:68-118`, the triples `:484-491`): each
    /// T2/T3 set sets its tier flag and raises `maxTurrets` by 1; the T3 rows
    /// also set the T2 flag. The Ballista Panic buff (205) arms `ballistaPanic`.
    #[test]
    fn the_dd2_sentry_sets_grant_their_flags() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!(!p.set_squire_t2 && !p.set_squire_t3);
        assert_eq!(p.max_turrets, 1, "the base budget");
        // The Squire T2 set (3800/3801/3802).
        p.equipped[0] = 3800;
        p.equipped[1] = 3801;
        p.equipped[2] = 3802;
        p.update(&w, PlayerInput::default());
        assert!(p.set_squire_t2 && !p.set_squire_t3);
        assert_eq!(p.max_turrets, 2, "+1 turret");
        // The Squire T3 set (3871/3872/3873): both flags.
        p.equipped[0] = 3871;
        p.equipped[1] = 3872;
        p.equipped[2] = 3873;
        p.update(&w, PlayerInput::default());
        assert!(p.set_squire_t3 && p.set_squire_t2, "T3 sets T2 too");
        assert_eq!(p.max_turrets, 2);
        // The Huntress T3 (3877/3878/3879) and the Monk T2 (3806/3807/3808).
        p.equipped[0] = 3877;
        p.equipped[1] = 3878;
        p.equipped[2] = 3879;
        p.update(&w, PlayerInput::default());
        assert!(p.set_huntress_t3 && p.set_huntress_t2);
        p.equipped[0] = 3806;
        p.equipped[1] = 3807;
        p.equipped[2] = 3808;
        p.update(&w, PlayerInput::default());
        assert!(p.set_monk_t2 && !p.set_monk_t3);
        // The Apprentice T2 (3797/3798/3799) and T3 (3874/3875/3876) —
        // S1-D-4b-9, `ArmorSetBonuses.cs:74-78`/`:99-104`, the triples
        // `:485`/`:489`.
        p.equipped[0] = 3797;
        p.equipped[1] = 3798;
        p.equipped[2] = 3799;
        p.update(&w, PlayerInput::default());
        assert!(p.set_apprentice_t2 && !p.set_apprentice_t3);
        assert_eq!(p.max_turrets, 2, "+1 turret");
        p.equipped[0] = 3874;
        p.equipped[1] = 3875;
        p.equipped[2] = 3876;
        p.update(&w, PlayerInput::default());
        assert!(p.set_apprentice_t3 && p.set_apprentice_t2, "T3 sets T2 too");
        assert_eq!(p.max_turrets, 2);
        // The Ballista Panic buff (205).
        assert!(!p.ballista_panic);
        p.buffs.add_buff(205, 3600);
        p.update(&w, PlayerInput::default());
        assert!(p.ballista_panic, "buff 205 arms the panic");
    }

    /// M3-EQUIP-17 — the life-regen rows feed `extra_regen`.
    #[test]
    fn the_life_regen_rows_add_up() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 0);
        // The Band of Regeneration (49): the item's own `lifeRegen` 2.
        p.equipped[3] = 49;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 2);
        // The Charm of Myths (860) carries the same `Item.lifeRegen` 2, and the
        // row is the item's own field now, not a hardcoded match arm.
        p.equipped[4] = 860;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 4);
        p.equipped[4] = 0;
        // The Regeneration buff (2) +4 and the Dryad's Ward (165) +6.
        p.buffs.add_buff(2, 3600);
        p.buffs.add_buff(165, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 12);
        // The Solar Flare Breastplate (2764) +2 and the Celestial Stone +2.
        p.equipped[1] = 2764;
        p.equipped[4] = 1865;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 16);
        // The Spooky Breastplate (3872) is the +8 row.
        p.equipped[1] = 3872;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 22);
    }

    /// M3-EQUIP-16 — the lava immunity budget.
    #[test]
    fn the_lava_charm_buys_immunity_ticks() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 0);
        // The Lava Charm (906) adds 420 ticks; the Terraspark Boots (5000) too.
        p.equipped[3] = 906;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 420);
        p.equipped[4] = 5000;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 840, "the rows stack");
        // Out of the lava the budget refills one tick per frame.
        p.lava_time = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_time, 1);
        // Standing in lava drains it instead of burning: five frames of the
        // budget leave the player untouched (`update_survival` is the sub-step
        // the lava hit lives in).
        p.lava_time = 5;
        p.hp = 100;
        p.hurt_cd = 0.0;
        for _ in 0..5 {
            p.in_lava = true;
            p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        }
        assert_eq!(p.hp, 100, "the budget absorbed the lava");
        assert_eq!(p.lava_time, 0);
        // The next frame burns for the vanilla 80.
        p.in_lava = true;
        p.hurt_cd = 0.0;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 20, "the 80-damage lava hit");
        assert!(p.buffs.find_buff_index(24).is_some(), "On Fire");
    }

    /// M3-EQUIP-15 — the thorns rows and the reflection arithmetic.
    #[test]
    fn the_thorns_rows_scale_the_reflection() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.thorns_damage(20), 0);
        // The Thorns potion (14): the NPC's own damage comes back whole.
        p.buffs.add_buff(14, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.thorns - 1.0).abs() < 1e-4);
        assert_eq!(p.thorns_damage(20), 20);
        // The Dryad's Ward (165) pushes it to 1.5 while under 1… it is already
        // 1.0, so the half does not land — the C# `if (thorns < 1f)` guard.
        p.buffs.add_buff(165, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.thorns - 1.0).abs() < 1e-4);
        // The Turtle set pins the multiplier at 2 regardless of the potion.
        p.equipped[0] = 1316;
        p.equipped[1] = 1317;
        p.equipped[2] = 1318;
        p.update(&w, PlayerInput::default());
        assert!(p.turtle_thorns);
        assert_eq!(p.thorns_damage(20), 40);
        assert_eq!(p.thorns_damage(2000), 1000, "the 1000 cap");
        // The Cactus set adds its flat 15 on top.
        p.equipped[0] = 894;
        p.equipped[1] = 895;
        p.equipped[2] = 896;
        p.update(&w, PlayerInput::default());
        assert!(p.cactus_thorns);
        assert!(!p.turtle_thorns, "the turtle row is gone");
        assert_eq!(p.thorns_damage(20), 35, "20 from the potion + the flat 15");
    }

    /// M3-EQUIP-14 — the Celestial Stone bundle and the day/night stones.
    #[test]
    fn the_sky_stones_add_their_bundle() {
        let mut w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The Celestial Stone (1865): every row of the C# block.
        p.equipped[3] = 1865;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        assert_eq!(p.stat_defense, 4);
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        assert!((p.minion_damage - 1.1).abs() < 1e-4);
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (6, 6, 6));
        assert!((p.pick_speed_bonus - 0.15).abs() < 1e-4);
        // The Sun Stone (899) only works by day; the Moon Stone (900) by night.
        p.equipped[3] = 899;
        w.day_time = true;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        w.day_time = false;
        p.update(&w, PlayerInput::default());
        assert!(!p.sky_stone_effects);
        p.equipped[3] = 900;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        w.day_time = true;
        p.update(&w, PlayerInput::default());
        assert!(!p.sky_stone_effects);
        // An eclipse makes the Moon Stone work in daylight.
        w.eclipse = true;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
    }

    /// M3-EQUIP-13 — the armor pieces carry their own damage/crit/speed rows
    /// (before this the worn armor only contributed defense).
    #[test]
    fn the_armor_pieces_add_their_stat_rows() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The Cobalt Mage Hat (371): +10% magic damage.
        p.equipped[0] = 371;
        p.update(&w, PlayerInput::default());
        assert!((p.magic_damage - 1.1).abs() < 1e-4);
        assert!((p.melee_damage - 1.0).abs() < 1e-4);
        // The Fossil Greaves (552) add 7% to every class and +8% move; the
        // Yoyo Bag-style rows stack per piece, so two pieces add twice.
        p.equipped[1] = 552;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.07).abs() < 1e-4);
        assert!((p.minion_damage - 1.07).abs() < 1e-4);
        assert!((p.move_speed - 1.08).abs() < 1e-4);
        // The Solar Flare Breastplate (2764) alone is +29% melee damage.
        p.equipped[0] = 0;
        p.equipped[1] = 2764;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.29).abs() < 1e-4);
    }

    /// M3-EQUIP-12 — the immunity rows of the accessories and the Snow/Molten
    /// sets, re-applied after `ResetEffects`' clear.
    #[test]
    fn the_immunity_rows_block_their_debuffs() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!(!p.buffs.buff_immune[46]);
        // The Snow set (803/804/805) is immune to Chilled (46) and Frozen (47).
        p.equipped[0] = 803;
        p.equipped[1] = 804;
        p.equipped[2] = 805;
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.buff_immune[46]);
        assert!(p.buffs.buff_immune[47]);
        // The Ankh Shield (1613): On Fire is *not* on its list, Chilled is.
        p.equipped[0] = 0;
        p.equipped[1] = 0;
        p.equipped[2] = 0;
        p.equipped[3] = 1613;
        p.update(&w, PlayerInput::default());
        for id in [46, 33, 36, 30, 20, 32, 31, 35, 23, 22, 156] {
            assert!(p.buffs.buff_immune[id], "the ankh row {}", id);
        }
        assert!(!p.buffs.buff_immune[24], "On Fire is not an ankh immunity");
        // The Molten set (231/232/233) is — the gated row rides the wave-D
        // `armor_sets.rs` seam (`SetFlag::MoltenFireImmune`); the vampire
        // sun-scorch gate always passes today (the granter is unported).
        p.equipped[3] = 0;
        p.equipped[0] = 231;
        p.equipped[1] = 232;
        p.equipped[2] = 233;
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.buff_immune[24]);
        // Cheap proof the table gates the add: the debuff never lands.
        p.buffs.add_buff(24, 600);
        assert_eq!(p.buffs.find_buff_index(24), None);
    }

    /// M3-EQUIP-11 — `endurance` folds the rows and the hurt applies it.
    #[test]
    fn the_endurance_rows_cut_the_incoming_damage() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.0).abs() < 1e-4);
        // The Worm Scarf (3224) is +17%.
        p.equipped[3] = 3224;
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.17).abs() < 1e-4);
        // The Turtle set (1316/1317/1318) is +15%.
        p.equipped[0] = 1316;
        p.equipped[1] = 1317;
        p.equipped[2] = 1318;
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.15).abs() < 1e-4);
        // The Endurance buff (114) adds its 10% on top.
        p.buffs.add_buff(114, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.25).abs() < 1e-4);
        // 40 damage with 15+10% endurance: 40 * 0.75 = 30 past the defense 0.
        p.stat_defense = 0;
        p.hp = 100;
        p.hurt_cd = 0.0;
        p.hurt_inner(
            40,
            p.x - 10.0,
            false,
            &mut tr_core::UnifiedRandom::default(),
            true,
            false,
            None,
            true,
        );
        assert_eq!(p.hp, 70, "the cut, not the raw 40");
        // A 1-damage hit past 25% endurance truncates to 0, then the
        // `num2 < 1 -> 1` floor (`Player.cs:38378-38381`) lifts it back to 1.
        p.hurt_cd = 0.0;
        p.immune = false; // the first hit armed the 40-tick window
        p.immune_time = 0;
        p.hurt_inner(
            1,
            p.x - 10.0,
            false,
            &mut tr_core::UnifiedRandom::default(),
            true,
            false,
            None,
            true,
        );
        assert_eq!(p.hp, 69, "the sub-one hit floors to 1, not 0");
    }

    /// S1-C-3 — the `num2 < 1 -> 1` floor (`Player.cs:38378-38381`): the
    /// endurance multiply can truncate a hit to 0; the C# floors it to 1, which
    /// also shortens the immunity window from 40 to 20 ticks (`:38385`:
    /// `num2 != 1.0 ? 40 : 20`, via `damage.rs::iframes_for`).
    #[test]
    fn a_sub_one_hit_floors_to_one_and_uses_the_short_iframe() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // 25% endurance: (1 - 0.25) * 1 = 0.75, truncated to 0 before the floor.
        p.endurance = 0.25;
        p.stat_defense = 0;
        p.hp = 100;
        p.hurt_cd = 0.0;
        assert!(p.hurt_inner(
            1,
            p.x - 10.0,
            false,
            &mut tr_core::UnifiedRandom::default(),
            true,
            false,
            None,
            true
        ));
        assert_eq!(p.hp, 99, "the hit lands for the floored 1, not 0");
        assert!(
            (p.hurt_cd - 20.0 / 60.0).abs() < 1e-6,
            "the 1-damage immunity window is 20 ticks, not 40"
        );
    }

    /// M3-EQUIP-10 — the class emblems and the crit/move accessories.
    #[test]
    fn the_damage_accessories_raise_their_class() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Warrior Emblem (490): melee +15%, nothing else.
        p.equipped[3] = 490;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.15).abs() < 1e-4);
        assert!((p.magic_damage - 1.0).abs() < 1e-4);
        // Avenger Emblem (935): +12% to all four classes.
        p.equipped[3] = 935;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.12).abs() < 1e-4);
        assert!((p.minion_damage - 1.12).abs() < 1e-4);
        // Destroyer Emblem (1301, `ItemID.cs:4285` — the Necromantic
        // Scroll is 1845): +8 to three crits and +10% to four classes
        // on top of the 4-point crit bases. Wave E: granted by
        // `accessory_combat::destroyer_emblem`, exactly once.
        p.equipped[3] = 1301;
        p.update(&w, PlayerInput::default());
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (12, 12, 12));
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        // Papyrus Scarab (1864): +15% minion damage (its `maxMinions++` half
        // belongs to `grant_summon_caps`).
        p.equipped[3] = 1864;
        p.update(&w, PlayerInput::default());
        assert!((p.minion_damage - 1.15).abs() < 1e-4);
    }

    /// S1-C-12 wave E — the no-double-grant witnesses: every combat-family
    /// kind grants EXACTLY ONCE through the wired seam
    /// (`apply_equip_functional` -> `accessory_combat::apply_combat_accessory`;
    /// the former inline arms are deleted). Each row pins the FULL C#
    /// effect set (`Player.cs:14411-14417`/`:14636-14650`/`:14912-14921`/
    /// `:15299-15314`/`:15369-15376`/`:15428-15450`) on a fresh spawn
    /// (damage classes 1.0, crits 4, melee-speed bonus 0), then re-ticks:
    /// a second grant path would show up either as a doubled first-tick
    /// value or as drift on the second tick.
    #[test]
    fn each_combat_accessory_grants_exactly_once_through_the_wired_seam() {
        let w = World::new("t".into(), 64, 800);
        // (kind, melee/ranged/magic/minion damage, melee/ranged/magic crit,
        //  melee-speed bonus, kbGlove)
        let rows: [(u16, [f32; 4], [i32; 3], f32, bool); 12] = [
            // Sorcerer Emblem `:15428-15431`.
            (489, [1.0, 1.0, 1.15, 1.0], [4, 4, 4], 0.0, false),
            // Warrior Emblem `:15432-15435`.
            (490, [1.15, 1.0, 1.0, 1.0], [4, 4, 4], 0.0, false),
            // Ranger Emblem `:15436-15439`.
            (491, [1.0, 1.15, 1.0, 1.0], [4, 4, 4], 0.0, false),
            // Summoner Emblem `:15440-15443`.
            (2998, [1.0, 1.0, 1.0, 1.15], [4, 4, 4], 0.0, false),
            // Avenger Emblem `:15444-15450`.
            (935, [1.12, 1.12, 1.12, 1.12], [4, 4, 4], 0.0, false),
            // Destroyer Emblem `:14912-14921`.
            (1301, [1.1, 1.1, 1.1, 1.1], [12, 12, 12], 0.0, false),
            // Putrid Scent `:14636-14646` (aggro −400 unported).
            (3015, [1.05, 1.05, 1.05, 1.05], [9, 9, 9], 0.0, false),
            // Flesh Knuckles `:14647-14650` (aggro +400 only — unported).
            (3016, [1.0, 1.0, 1.0, 1.0], [4, 4, 4], 0.0, false),
            // Power Glove `:15299-15305`.
            (897, [1.0, 1.0, 1.0, 1.0], [4, 4, 4], 0.12, true),
            // Mechanical Glove `:15369-15376`.
            (936, [1.12, 1.0, 1.0, 1.0], [4, 4, 4], 0.12, true),
            // Fire Gauntlet `:15306-15314` (magmaStone unported).
            (1343, [1.12, 1.0, 1.0, 1.0], [4, 4, 4], 0.12, true),
            // Berserker's Glove `:14411-14417` — NO meleeDamage row
            // (wave B's +0.12 copy was removed in the wave-E re-read).
            (3992, [1.0, 1.0, 1.0, 1.0], [4, 4, 4], 0.12, true),
        ];
        for (kind, dmg, crit, speed, kb) in rows {
            let mut p = Player::spawn(&w);
            p.equipped[3] = kind;
            for tick in 0..2 {
                p.update(&w, PlayerInput::default());
                let msg = format!("kind {kind}, tick {tick}");
                assert!((p.melee_damage - dmg[0]).abs() < 1e-4, "melee {msg}");
                assert!((p.ranged_damage - dmg[1]).abs() < 1e-4, "ranged {msg}");
                assert!((p.magic_damage - dmg[2]).abs() < 1e-4, "magic {msg}");
                assert!((p.minion_damage - dmg[3]).abs() < 1e-4, "minion {msg}");
                assert_eq!(
                    (p.melee_crit, p.ranged_crit, p.magic_crit),
                    (crit[0], crit[1], crit[2]),
                    "crits {msg}"
                );
                assert!((p.melee_speed_bonus - speed).abs() < 1e-4, "speed {msg}");
                assert_eq!(p.kb_glove, kb, "kbGlove {msg}");
            }
            // Taking the accessory off ends the grant next tick (the
            // per-tick clears: damage/crit/speed bases + `kbGlove = false`
            // `Player.cs:18709`).
            p.equipped[3] = 0;
            p.update(&w, PlayerInput::default());
            assert!((p.melee_damage - 1.0).abs() < 1e-4, "off {kind}");
            assert_eq!(p.melee_crit, 4, "off {kind}");
            assert!(!p.kb_glove, "kbGlove off {kind}");
        }
        // Two slots with the same emblem stack per slot (the C# loop
        // applies each slot's row once, `Player.cs:12946-12952`).
        let mut p = Player::spawn(&w);
        p.equipped[3] = 490;
        p.equipped[4] = 490;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.30).abs() < 1e-4);
    }

    /// S1-C-12 wave E — the sky-stone bundle's single grant path: the
    /// flag scan + stat fold ride `accessory_combat` (the inline copies
    /// are deleted), the `minionKB += 0.5f` row (`Player.cs:13002`) lands
    /// once per tick thanks to the new `minionKB = 0f` reset (`:18347`),
    /// and a combat emblem beside the stone grants each family once.
    #[test]
    fn the_sky_stone_bundle_grants_once_including_minion_knockback() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.equipped[3] = 1865; // Celestial Stone — always on.
        for tick in 0..2 {
            p.update(&w, PlayerInput::default());
            assert!(p.sky_stone_effects, "tick {tick}");
            assert_eq!(p.stat_defense, 4, "tick {tick}");
            assert!((p.melee_damage - 1.1).abs() < 1e-4, "tick {tick}");
            assert!((p.melee_speed_bonus - 0.1).abs() < 1e-4, "tick {tick}");
            assert_eq!(p.melee_crit, 6, "tick {tick}");
            assert!((p.minion_kb - 0.5).abs() < 1e-4, "minionKB tick {tick}");
        }
        // The stone + a Warrior Emblem: each family's row once (melee
        // 1.0 + 0.1 + 0.15, crit 4 + 2).
        p.equipped[4] = 490;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.25).abs() < 1e-4);
        assert_eq!(p.melee_crit, 6);
        assert!((p.minion_kb - 0.5).abs() < 1e-4);
        // Everything off: the bundle and its knockback end next tick.
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.sky_stone_effects);
        assert!((p.minion_kb - 0.0).abs() < 1e-4);
        assert!((p.melee_damage - 1.0).abs() < 1e-4);
    }

    /// M3-EQUIP-9 — `manaCost` folds the armor pieces, the accessories, the
    /// casters' sets and Magic Power.
    #[test]
    fn the_mana_cost_multiplier_folds_every_row() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 1.0).abs() < 1e-4);
        // Nature's Gift (223) −0.06 and the Mana Flower (555) −0.08.
        p.equipped[3] = 223;
        p.equipped[4] = 555;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.86).abs() < 1e-4);
        // The Mystic Robe (2279) −0.1 as an armor piece.
        p.equipped[1] = 2279;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.76).abs() < 1e-4);
        // Magic Power (buff 29) −0.02 plus its other rows.
        p.buffs.add_buff(29, 3600);
        let mana_max_before = p.mana_max;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.74).abs() < 1e-4);
        assert!(p.mana_max > mana_max_before, "the 20-mana row");
        // The Mythril Caster set (376/379/380) −0.17.
        p.equipped[0] = 376;
        p.equipped[1] = 379;
        p.equipped[2] = 380;
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - (1.0 - 0.17 - 0.02)).abs() < 1e-4);
    }

    /// S1-P-i19 — `statManaMax2` rebases to `statManaMax` every tick
    /// (`ResetEffects` `Player.cs:18389`); the piece/accessory/set/buff rows
    /// re-add once per tick, never accumulate.
    #[test]
    fn s1_p_i19_the_mana_max_rebases_every_tick() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The Band of Starpower (+40, `:14924-14928`) + Magic Power (+20,
        // `:11890`): 20 base + 40 + 20 = 80, stable across ticks.
        p.equipped[3] = 111;
        p.buffs.add_buff(29, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 80);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 80, "no per-tick accumulation");
        // Taking the band off drops the max back to base + buff next tick.
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 40);
        // The base itself is the owned 20 (`:56075`); the crystal-use arm
        // (`:45274-45288`) is unported, so it never grows here.
        assert_eq!(p.mana_max_base, 20);
    }

    /// S1-P-i19 — the `manaFlower` flag's ONE −0.08 row (`UpdateEquips`
    /// `Player.cs:12962-12965`): two flowers do not stack, and the Arcane
    /// Flower's own +5 crit / +5% damage rows (`:14405-14410`) ride along.
    #[test]
    fn s1_p_i19_the_flower_fold_applies_once_and_the_arcane_rows_ride() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.equipped[3] = 555; // Mana Flower
        p.equipped[4] = 4001; // Mana Cloak (also manaFlower)
        p.update(&w, PlayerInput::default());
        assert!(p.mana_flower);
        assert!(
            (p.mana_cost - 0.92).abs() < 1e-4,
            "two flowers still cost only 0.08, got {}",
            p.mana_cost
        );
        // The Arcane Flower keeps its crit/damage halves.
        let mut q = Player::spawn(&w);
        q.equipped[3] = 3991;
        q.update(&w, PlayerInput::default());
        assert!(q.mana_flower);
        assert!((q.mana_cost - 0.92).abs() < 1e-4);
        assert_eq!(q.magic_crit, 4 + 5);
        assert!((q.magic_damage - 1.05).abs() < 1e-4);
        // The flag clears when the flowers come off (`:18721`).
        q.equipped[3] = 0;
        q.update(&w, PlayerInput::default());
        assert!(!q.mana_flower);
        assert!((q.mana_cost - 1.0).abs() < 1e-4);
    }

    /// S1-P-i19 — the mana-regen accessory rows (`ApplyEquipFunctional`
    /// `Player.cs:14929-14934`): the Mana Regeneration Band 982 adds +40
    /// max AND +60 `manaRegenBonus`; the cuffs' +40 max rows landed too.
    #[test]
    fn s1_p_i19_the_mana_regen_band_grants_max_and_bonus() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.equipped[3] = 982;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 20 + 40);
        assert_eq!(p.mana_regen_bonus, 60);
        // The Magic Cuffs keep their flag AND gain the +40 max row.
        let mut q = Player::spawn(&w);
        q.equipped[3] = 1595;
        q.update(&w, PlayerInput::default());
        assert!(q.magic_cuffs);
        assert_eq!(q.mana_max, 20 + 40);
    }

    /// M3-EQUIP-8 — the wings grant `wingsLogic`/`wingTimeMax` themselves (the
    /// `TR_WINGS` pin is no longer the only source).
    #[test]
    fn the_wing_accessories_grant_flight() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 0);
        assert_eq!(p.wing_time_max, 0);
        // Demon Wings (492 → `ArmorIDs.Wing` 1): 100 ticks of flight.
        p.equipped[3] = 492;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 1);
        assert!(p.has_wings);
        assert_eq!(p.wing_time_max, Player::wing_stats(1).0);
        // Bee Wings (1866 → 22) is the hover row: its stats carry the hover
        // speeds, and the budget is its own.
        p.equipped[3] = 1866;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 22);
        assert_eq!(p.wing_time_max, Player::wing_stats(22).0);
        assert!(Player::wing_stats(22).3, "the hover flag");
        // Taking the wings off ends both.
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 0);
        assert_eq!(p.wing_time_max, 0);
        assert!(!p.has_wings);
    }

    /// M3-EQUIP-7 — the movement accessories grant the live flags (the
    /// `TR_JUMPS`/`TR_BOOTS` pins stop being the only source).
    #[test]
    fn the_movement_accessories_grant_their_flags() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Bundle of Balloons (1164): Cloud + Sandstorm + Blizzard + jumpBoost.
        // Before M3-EQUIP-7 only the jump boost was ported, so the item gave
        // one extra jump instead of three.
        p.equipped[3] = 1164;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        assert!(p.has_jump_options[ExtraJump::Blizzard.index()]);
        assert!(p.jump_boost);
        // Frostspark Boots (1862): tier 3, 6.75 accel, +8% move, ice skates.
        p.equipped[3] = 1862;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_boots, 3);
        assert!((p.acc_run_speed_value() - 6.75).abs() < 1e-4);
        assert!((p.move_speed - 1.08).abs() < 1e-4);
        assert!(p.ice_skate);
        assert!(!p.has_jump_options[ExtraJump::Cloud.index()]);
        // Hellfire Treads (4874): the tier-5 boots at the plain 6f.
        p.equipped[3] = 4874;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_boots, 5);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
        // A Sandstorm balloon (1252): the bottle's option and the boost.
        p.equipped[3] = 0;
        p.equipped[4] = 1252;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        assert!(p.jump_boost);
    }

    /// M3-EQUIP-5 — `ArmorSetBonuses.Benefits`' scalar rows ride the same pass.
    #[test]
    fn the_completed_set_grants_its_stat_row() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Crystal Assassin (4982/4983/4984): +10% every damage, +10 every crit,
        // `dashType = 5`. The crit bases are 4 (`ResetEffects`), so 14.
        p.equipped[0] = 4982;
        p.equipped[1] = 4983;
        p.equipped[2] = 4984;
        p.update(&w, PlayerInput::default());
        // The damage scalars start at 1.0 in the reset (`:2535-2538`): the
        // set's `+= 0.1f` plus the slate's own +5% (4983, M3-EQUIP-13's piece
        // row) read as 1.15.
        assert!((p.melee_damage - 1.15).abs() < 1e-4);
        assert!((p.minion_damage - 1.15).abs() < 1e-4);
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (14, 14, 14));
        assert_eq!(p.dash_type, 5);
        // Swapping to the Ninja set (256/257/258) keeps only its own row:
        // `moveSpeed += 0.2f`.
        p.equipped[0] = 256;
        p.equipped[1] = 257;
        p.equipped[2] = 258;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.0).abs() < 1e-4, "the base 1.0, no add");
        assert_eq!(p.melee_crit, 4);
        assert!((p.move_speed - 1.2).abs() < 1e-4);
        // The Mining set (88/410/411) is the `pickSpeed -= 0.1f` negation.
        p.equipped[0] = 88;
        p.equipped[1] = 410;
        p.equipped[2] = 411;
        p.update(&w, PlayerInput::default());
        assert!((p.pick_speed_bonus - 0.1).abs() < 1e-4);
        // A partial set grants nothing.
        p.equipped[2] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.pick_speed_bonus - 0.0).abs() < 1e-4);
    }

    #[test]
    fn dye_cache_ignores_non_dye_kinds() {
        // Dirty: a non-dye in a dye slot renders plain — the C# reads
        // `GetShaderIdFromItemId` (`Item.cs:48713`), which is 0 outside the
        // `DyeInitializer` bindings.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.dye_slots[1] = 89; // the Copper Helmet
        p.update_dye_cache();
        assert_eq!(p.c_body, 0);
    }

    #[test]
    fn dot_death_survival_drain_records_the_reason() {
        // DW-1.3 — Player.cs:19361-19384: a drain kill carries its reason out
        // of UpdateLifeRegen; update_survival parks it on the player for the
        // death pipeline (the default generic attribution here = Burned, 8).
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.dot_death, None); // a healthy tick records nothing
        p.hp = 8;
        p.regen.life_regen_count = -1200;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Burned));
    }

    #[test]
    fn the_defense_reduces_the_hurt_damage() {
        // `Player.Hurt`'s classic reduction: the damage minus
        // `statDefense * 0.5`, floored at 1 (M3-EQUIP-2).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.hurt_cd = 0.0;
        p.immune = false; // the window `UpdateImmunity` would have run out
        p.stat_defense = 20; // 10 damage reduction
        p.hp = 100;
        p.hurt(20, 0.0, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 90, "20 - 20*0.5 = 10 taken");
        // The floor: a huge defense leaves 1.
        p.hurt_cd = 0.0;
        p.immune = false;
        p.immune_time = 0;
        p.hp = 100;
        p.stat_defense = 400;
        p.hurt(20, 0.0, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 99);
    }

    #[test]
    fn the_potion_sickness_buff_gates_and_expires() {
        // The buff 21 (`BuffID.PotionSickness`): the drink arm gates on it,
        // and the 3600-tick timer expires like any buff.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(21, 3600);
        assert!(p.buffs.find_buff_index(21).is_some());
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.buffs.find_buff_index(21).is_some());
        // Expire it.
        for _ in 0..3600 {
            p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        }
        assert!(p.buffs.find_buff_index(21).is_none());
    }

    #[test]
    fn the_fed_tiers_stack_their_stat_adds() {
        // `Player.cs:11931-11969`: the three tiers' per-slot adds — the
        // crits/damage/speed ride the scan, the defense waits for
        // M3-EQUIP-3, the minion halves for M4-MINION.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(26, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.well_fed);
        assert_eq!(p.melee_crit, 4 + 2);
        assert!((p.melee_damage - 1.05).abs() < 1e-4);
        assert!((p.move_speed - 1.2).abs() < 1e-4);
        assert!((p.pick_speed_bonus - 0.05).abs() < 1e-4);
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(207, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.melee_crit, 4 + 4);
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        assert!((p.move_speed - 1.4).abs() < 1e-4);
    }

    #[test]
    fn the_fed_flag_clears_when_the_buff_expires() {
        // ResetEffects' per-tick derivation: the flag rides the slots.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(26, 2);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.well_fed);
        for _ in 0..3 {
            p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        }
        assert!(!p.well_fed);
    }

    #[test]
    fn the_sunflower_buff_moves_and_the_star_buffs_mana_regen() {
        // `Player.cs:11001-11004`: the 146 slot — `moveSpeed += 0.1; *= 1.1`
        // (1.0 -> 1.21); `:9932-9934`: the 158 slot — `manaRegenBonus += 10`.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(146, 60);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!((p.move_speed - 1.21).abs() < 1e-4);
        assert!((p.max_run_speed - 3.0 * 1.21).abs() < 1e-3);
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(158, 60);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.mana_regen_bonus, 10);
    }

    // ---- S1-E-7 wave A: the potion-buff block (`Player.cs:9875-10048`) ----

    /// A pool world of the given liquid from row 40 down to the ground (the
    /// `drowning_drains_breath_and_damages` shape).
    pub(super) fn liquid_pool_world(kind: tr_core::tile::LiquidKind) -> World {
        let mut w = ground_world();
        for x in 0..64 {
            for y in 40..60 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(kind);
            }
        }
        w.spawn_y = 44;
        w
    }

    #[test]
    fn s1_e_7a_obsidian_skin_arms_lava_fire_and_onfire_immunity() {
        // `Player.cs:9875-9880`: lavaImmune + fireWalk + buffImmune[24].
        let w = liquid_pool_world(tr_core::tile::LiquidKind::Lava);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(1, 36000);
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.in_lava, "the player settles in the lava");
        assert!(p.lava_immune && p.fire_walk);
        assert!(p.buffs.buff_immune[24], "OnFire immunity re-armed per tick");
        assert_eq!(p.hp, 100, "the lava hit is skipped (`:27676`)");
        assert!(p.buffs.find_buff_index(24).is_none());
        p.buffs.add_buff(24, 420);
        assert!(p.buffs.find_buff_index(24).is_none(), "the immune guard rejects OnFire");
        // Without the buff the same pool hurts (the pre-existing lava arm).
        let mut q = Player::spawn(&w);
        for _ in 0..300 {
            q.update(&w, PlayerInput::default());
        }
        assert!(q.hp < 100, "no ObsidianSkin, the lava burns");
    }

    #[test]
    fn s1_e_7a_swiftness_adds_a_quarter_move_speed() {
        // `Player.cs:9952-9955`: `moveSpeed += 0.25f`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(3, 600);
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.25).abs() < 1e-4);
        assert!((p.max_run_speed - 3.0 * 1.25).abs() < 1e-3);
    }

    #[test]
    fn s1_e_7a_gills_breathe_underwater_and_invert_on_the_good_world() {
        // `Player.cs:9958` + `:23226-23229`: the probe flips — underwater the
        // breath stays full; on For the Worthy the DRY air drowns instead.
        let w = liquid_pool_world(tr_core::tile::LiquidKind::Water);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(4, 36000);
        for _ in 0..1500 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.in_water);
        assert_eq!(p.breath, BREATH_MAX, "the gills breathe water");
        assert_eq!(p.hp, 100);
        // The good-world inversion: dry ground, the gills drain the breath.
        let mut gw = ground_world();
        gw.seeds.insert(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
        let mut q = Player::spawn(&gw);
        q.buffs.add_buff(4, 36000);
        for _ in 0..1500 {
            q.update(&gw, PlayerInput::default());
        }
        assert!(!q.in_water);
        assert_eq!(q.breath, 0, "getGoodWorld && !flag — the air drowns");
        assert!(q.hp < 100);
    }

    #[test]
    fn s1_e_7a_mana_regeneration_buff_feeds_the_regen_engine() {
        // `Player.cs:9964-9967` → `UpdateManaRegen` (`:19418-19562`): the buff
        // doubles the base regen and pins the mana-fraction scaler at 1.0.
        // Moving (vx != 0) so `standing_still` cannot double it on its own:
        // num5 = 20/3 + 0 + 1 = 7; with the buff 14 × 1.0 = 14, without
        // 7 × (0.5 + 0.5·0.5) = 5.25 → 5.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(6, 600);
        p.mana = 10;
        p.vx = 3.0;
        p.regen.mana_regen_delay = 0.0;
        p.update(&w, PlayerInput::default());
        assert!(p.mana_regen_buff);
        assert_eq!(p.regen.mana_regen, 14);
        let mut q = Player::spawn(&w);
        q.mana = 10;
        q.vx = 3.0;
        q.regen.mana_regen_delay = 0.0;
        q.update(&w, PlayerInput::default());
        assert_eq!(q.regen.mana_regen, 5);
    }

    #[test]
    fn s1_e_7a_magic_power_adds_two_tenths_magic_damage() {
        // `Player.cs:9968-9976`: `magicDamage += 0.2f`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(7, 600);
        p.update(&w, PlayerInput::default());
        assert!((p.magic_damage - 1.2).abs() < 1e-4);
    }

    #[test]
    fn s1_e_7a_featherfall_caps_the_fall_and_refreshes_the_fall_origin() {
        // `Player.cs:9979` + `:27369-27376`: the /3 cap, the hover-up /10
        // tighten; `:25379`: slowFall keeps the fall origin at the row.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(8, 600);
        for _ in 0..40 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.slow_fall && !p.on_ground, "still falling after 40 ticks");
        let cap = MAX_FALL + 0.01; // the `:24437` rebase tail
        assert!((p.vy - cap / 3.0).abs() < 1e-4, "vy {}", p.vy);
        assert_eq!(p.fall_start, (p.y / TILE_PX) as i32, "no fall accrues");
        // The hover-up latch: one tick past the /5 gate snaps to /10.
        p.try_keeping_hovering_up = true;
        p.update(&w, PlayerInput::default());
        assert!((p.vy - cap / 10.0).abs() < 1e-4, "vy {}", p.vy);
        // Let it land — the refreshed origin means no fall damage.
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        assert_eq!(p.hp, 100);
    }

    #[test]
    fn s1_e_7a_the_flag_buffs_ride_their_slots_and_clear_on_expiry() {
        // `Player.cs:9981-10028`: Spelunker 9, BiomeSight 343, Invisibility
        // 10, NightOwl 12, Battle 13, WaterWalking 15, Hunter 17,
        // Gravitation 18 — one flag per arm, re-derived per tick.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for id in [9, 343, 10, 12, 13, 15, 17, 18] {
            p.buffs.add_buff(id, 600);
        }
        p.update(&w, PlayerInput::default());
        assert!(p.find_treasure, "9");
        assert!(p.biome_sight, "343");
        assert!(p.invis, "10");
        assert!(p.night_vision, "12");
        assert!(p.enemy_spawns, "13");
        assert!(p.water_walk, "15");
        assert!(p.detect_creature, "17");
        assert!(p.grav_control, "18");
        // The three flag debuffs of the same block derive the same way
        // (`:10035`/`:10043`/`:10047`; their consumers have dedicated tests).
        for id in [31, 35, 160] {
            p.buffs.add_buff(id, 600);
        }
        p.update(&w, PlayerInput::default());
        assert!(p.confused && p.silence && p.dazed);
        // Expire them all — the ResetEffects clears leave every flag down.
        let mut q = Player::spawn(&w);
        for id in [9, 343, 10, 12, 13, 15, 17, 18] {
            q.buffs.add_buff(id, 2);
        }
        for _ in 0..4 {
            q.update(&w, PlayerInput::default());
        }
        assert!(
            !q.find_treasure && !q.biome_sight && !q.invis && !q.night_vision
                && !q.enemy_spawns && !q.water_walk && !q.detect_creature
                && !q.grav_control
        );
    }

    #[test]
    fn s1_e_7a_confused_swaps_the_run_keys() {
        // `Player.cs:10035` + `:24996-25004`: left↔right after the controls
        // land — a right press runs the player LEFT.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(31, 600);
        let input = PlayerInput { right: true, ..PlayerInput::default() };
        for _ in 0..30 {
            p.update(&w, input);
        }
        assert!(p.confused);
        assert!(p.control_left && !p.control_right, "the swap");
        assert!(!p.control_down, "the pre-swap up (unported) is false");
        assert!(p.vx < 0.0, "runs left, vx {}", p.vx);
    }

    #[test]
    fn s1_e_7a_dazed_divides_the_jump_statics() {
        // `Player.cs:10047` + `:19637-19641`: jumpHeight /5, jumpSpeed /2.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(160, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.dazed);
        assert_eq!(p.jump_height, 15 / 5);
        assert!((p.jump_speed - 5.01 / 2.0).abs() < 1e-4);
    }

    #[test]
    fn the_aura_buffs_grant_from_the_scene_snapshot() {
        // `Player.cs:25525-25555`: the 2-tick indicator buffs refresh from
        // the scan; the effect cases read the previous tick's grant (the
        // sunflower's movement starts one tick after the aura appears).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.auras.campfire = true;
        p.auras.heart_lantern = true;
        p.auras.star_in_bottle = true;
        p.auras.sunflower = true;
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        let types = p.buffs.active_types();
        assert!(types.contains(&87) && types.contains(&89) && types.contains(&158));
        // The grant happened after this tick's scan, so 146 applies NEXT
        // tick — the move_speed is still 1.0 here.
        assert!((p.move_speed - 1.0).abs() < 1e-4);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!((p.move_speed - 1.21).abs() < 1e-4);
    }

    #[test]
    fn movement_speed_per_liquid() {
        // Player.cs:28168-28171: water/lava 0.5, honey 0.25, shimmer 0.375,
        // dry 1.0.
        assert_eq!(movement_speed_for(true, false, false, false), 0.5);
        assert_eq!(movement_speed_for(false, true, false, false), 0.5);
        assert_eq!(movement_speed_for(false, false, true, false), 0.25);
        assert_eq!(movement_speed_for(false, false, false, true), 0.375);
        assert_eq!(movement_speed_for(false, false, false, false), 1.0);
    }

    #[test]
    #[test]
    fn the_wet_statics_rebase_feeds_the_tick() {
        // The wiring: a water tick rebase-assigns the jump statics (the C#
        // `Player.jumpHeight = 30` / `jumpSpeed = 6.01f`, `:24418-24424`) —
        // the values the jump section consumes.
        let mut w = ground_world();
        for y in 58..60 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(
            p.wet,
            "the tick must be wet: center row {}",
            (p.y / TILE_PX) as i32
        );
        assert_eq!(p.jump_height, 30, "the water jump height");
        assert!((p.jump_speed - 6.01).abs() < 1e-5, "the water jump speed");
    }

    #[test]
    fn the_wet_statics_rebase_assigns_the_per_liquid_arms() {
        // `Player.cs:24418-24424` (water): 0.2/5 + the jump statics 30/6.01.
        let water = WetStaticsFlags {
            wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, water);
        assert!((g - 0.2).abs() < 1e-5 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24399-24402` (honey): 0.1/3, the jump statics stay at the reset.
        let honey = WetStaticsFlags {
            wet: true,
            honey_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, honey);
        assert!((g - 0.1).abs() < 1e-5 && (mf - 3.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24404-24407` (merman): 0.3/7.
        let merman = WetStaticsFlags {
            wet: true,
            merman: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, merman);
        assert!((g - 0.3).abs() < 1e-5 && (mf - 7.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24409-24417` (trident, not lava): 0.25/6 + 25/5.51; controlUp
        // drops it to 0.1/2.
        let trident = WetStaticsFlags {
            wet: true,
            trident: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident);
        assert!((g - 0.25).abs() < 1e-5 && (mf - 6.01).abs() < 1e-5);
        assert_eq!((jh, js), (25, 5.51));
        let trident_up = WetStaticsFlags {
            control_up: true,
            ..trident
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident_up);
        assert!((g - 0.1).abs() < 1e-5 && (mf - 2.01).abs() < 1e-5);
        assert_eq!((jh, js), (25, 5.51));
        // Trident in lava falls through to the plain water arm.
        let trident_lava = WetStaticsFlags {
            lava_wet: true,
            ..trident
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident_lava);
        assert!((g - 0.2).abs() < 1e-5 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24389-24394` (shimmerWet, not absorbed): 0.15 + 23/5.51 — the
        // arm never touches the cap, so it stays at the 10 default.
        let shimmer = WetStaticsFlags {
            shimmer_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer);
        assert!((g - 0.15).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh, js), (23, 5.51));
    }

    #[test]
    fn the_rebase_multipliers_gates_and_creep() {
        // `:24384-24387`: the Dead Cells down-dash arm ×0.85 (wet, not
        // shimmering) — 0.4×0.85 / 10×0.85 + 0.01.
        let dash = WetStaticsFlags {
            wet: true,
            down_dash: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, dash);
        assert!((g - 0.4 * 0.85).abs() < 1e-5 && (mf - (10.0 * 0.85 + 0.01)).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24389-24393`: the absorbed shimmering arm ×0.9 — 0.36 / 9.01.
        let shimmering = WetStaticsFlags {
            shimmering: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmering);
        assert!((g - 0.36).abs() < 1e-5 && (mf - 9.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // The down-dash arm loses to the shimmer arm when shimmering.
        let shimmer_dash = WetStaticsFlags {
            wet: true,
            shimmering: true,
            down_dash: true,
            ..Default::default()
        };
        let (g, mf, _, _) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer_dash);
        assert!((g - 0.36).abs() < 1e-5 && (mf - 9.01).abs() < 1e-5);
        // `:24433-24436`: the vortex debuff zeroes the gravity AFTER the
        // wet arm assigned it.
        let vortex = WetStaticsFlags {
            wet: true,
            vortex_debuff: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, vortex);
        assert!(g == 0.0 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24380-24382`: portal physics pins the cap at 35 before the wet
        // arm re-assigns it (dry here, so 35 survives + the creep).
        let portal = WetStaticsFlags {
            portal_physics: true,
            ..Default::default()
        };
        let (g, mf, _, _) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, portal);
        assert!((g - 0.4).abs() < 1e-5 && (mf - 35.01).abs() < 1e-5);
        // Dry: the plain arm rides the defaults + the creep — the C#'s
        // every-tick `maxFallSpeed += 0.01f` (`:24437`).
        let (g, mf, jh, js) =
            wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, WetStaticsFlags::default());
        assert!((g - 0.4).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
    }

    #[test]
    fn the_wet_rebase_runs_before_the_jump_boost_lift() {
        // The C# order: the reset (`:24366`) → the rebase (`:24380-24437`)
        // → `UpdateJumpHeight`'s max-lift (`:25656`), so the wet 30/6.01
        // survives the lift and a jump-boosted shimmer keeps the wet 23
        // height while the lift raises the speed to 6.51.
        let shimmer = WetStaticsFlags {
            shimmer_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer);
        assert!((g - 0.15).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh.max(20), js.max(6.51)), (23, 6.51));
    }

    #[test]
    fn respawn_time_base_boss_and_expert() {
        // Player.cs:39060-39082: 600 base, +600 boss-near, x1.5 expert.
        assert_eq!(Player::get_respawn_time(false, false), 600);
        assert_eq!(Player::get_respawn_time(true, false), 1200);
        assert_eq!(Player::get_respawn_time(false, true), 900);
        assert_eq!(Player::get_respawn_time(true, true), 1800);
    }

    #[test]
    fn update_dead_counts_down_and_revives_at_zero() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        assert!(p.dead);
        assert_eq!(p.respawn_timer, 600);
        // 600 ticks of the dead state (no jump) -> the timer drains to 0 and
        // the last tick reports the respawn.
        let mut revived_at = None;
        for tick in 0..700 {
            if p.update_dead(false, false, false, false) {
                revived_at = Some(tick);
                break;
            }
        }
        assert_eq!(revived_at, Some(599));
        p.revive();
        assert!(!p.dead);
        assert_eq!(p.respawn_timer, 0);
    }

    #[test]
    fn respawn_timer_clamps_at_3600() {
        // Player.respawnTimerMax = 3600 (Player.cs:57109): a boss-near expert
        // death (1800) halved by nothing still clamps; the clamp matters for
        // the >3600 totals (the FTW x2 stays out). A forced high timer holds.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, true, true);
        p.respawn_timer = 4000; // the save-load can exceed the cap
        p.update_dead(false, false, false, false);
        // The clamp catches the decrement result (Player.cs:17049).
        assert_eq!(p.respawn_timer, 3600);
    }

    /// S1-P-i43 — `Player.UpdateDead`'s `this.chest = -1` (`Player.cs:17020`):
    /// the dead tick force-closes the open container silently (no sound 11),
    /// `chestX/Y` stay. The world chest AND the banks both close.
    #[test]
    fn s1_p_i43_the_dead_tick_closes_the_open_container() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.chest = 3;
        p.chest_x = 20;
        p.chest_y = 10;
        p.start_death(&w, false, false);
        p.update_dead(false, false, false, false);
        assert_eq!(p.chest, -1, "the world chest closes (:17020)");
        assert_eq!((p.chest_x, p.chest_y), (20, 10), "the origin stays");
        // A bank closes too (the C# writes -1 unconditionally).
        p.chest = -4;
        p.update_dead(false, false, false, false);
        assert_eq!(p.chest, -1, "the safe closes too");
    }

    #[test]
    fn respawn_life_is_half_the_pool_floored_at_100() {
        // `Player.cs:37598-37608`: the pool is `max(statLifeMax,
        // statLifeMax2)` — the port's single `max_hp` is the effective max —
        // and the life is `max(pool / 2, 100)`: a fresh 100-life character
        // revives FULL, 400 revives at 200.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.max_hp = 400;
        p.hp = 0;
        p.respawn();
        assert_eq!(p.hp, 200);
        assert_eq!(p.breath, BREATH_MAX);
        // The i-frames (`:37614-37618` + `:37661-37668`): the death revive
        // arms 180 with `immuneNoBlink` (the player is never hostile here).
        assert!(p.immune);
        assert_eq!(p.immune_time, 180);
        assert!(p.immune_no_blink);
    }

    #[test]
    fn spawn_iframes_swallow_hits_and_expire() {
        // `Player.cs:38210-38216`: the General cooldown flag is `!immune` —
        // the spawn i-frames swallow the hit; `UpdateImmunity`
        // (`:18856-18869`) clears it at 0 and the damage lands again.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.respawn();
        let mut scratch = tr_core::UnifiedRandom::default();
        assert!(
            !p.hurt(50, 0.0, &mut scratch, true),
            "the i-frames swallow the hit"
        );
        p.immune_time = 1;
        p.update_immunity();
        assert!(!p.immune, "the countdown cleared the flag");
        assert!(!p.immune_no_blink);
        assert!(
            p.hurt(50, 0.0, &mut scratch, true),
            "the damage lands after the i-frames"
        );
    }

    #[test]
    fn the_nurse_skips_protected_debuffs_in_both_gates() {
        // `BuffID.Sets.NurseCannotRemoveDebuff` (`BuffID.cs:52`): the
        // protected Potion Sickness 21 neither counts in the price
        // (`Main.cs:40061-40064`, the `> 60` time gate too) nor clears at
        // the heal (`:40147`); the unprotected Poisoned 20 does both.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(21, 3600); // protected
        p.buffs.add_buff(20, 3600); // removable
        assert_eq!(
            p.remountable_debuff_count(),
            1,
            "the protected one stays out of the price"
        );
        p.clear_nurse_removable_debuffs();
        assert!(
            p.buffs.find_buff_index(21).is_some(),
            "the protected one survives"
        );
        assert_eq!(p.buffs.find_buff_index(20), None, "the removable clears");
    }

    #[test]
    fn the_nurse_price_ignores_young_debuffs() {
        // `Main.cs:40062`: the cost counts only `buffTime > 60` slots; the
        // removal (`:40147`) takes any `buffTime > 0`.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(20, 50); // under the 60-tick price gate
        assert_eq!(p.remountable_debuff_count(), 0);
        p.buffs.buff_time[p.buffs.find_buff_index(20).unwrap()] = 1;
        p.clear_nurse_removable_debuffs();
        assert_eq!(
            p.buffs.find_buff_index(20),
            None,
            "the removal clears at time 1"
        );
    }

    #[test]
    fn buff_removal_refuses_debuffs_and_dels_the_rest() {
        // `Main.cs:43370-43389`: the non-debuff arm dels the slot; a debuff
        // (or 60/151) refuses.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(5, 3600); // Ironskin — a non-debuff
        p.buffs.add_buff(20, 3600); // Poisoned — a debuff
        let poison = p.buffs.find_buff_index(20).unwrap();
        let iron = p.buffs.find_buff_index(5).unwrap();
        assert!(!p.try_removing_buff(&w, poison, 20, [0, 0]), "a debuff refuses");
        assert!(p.try_removing_buff(&w, iron, 5, [0, 0]), "the non-debuff removes");
        assert_eq!(p.buffs.find_buff_index(5), None);
    }

    #[test]
    fn buff_removal_dismounts_the_mount_buff_without_del() {
        // `Main.cs:43376-43380`: the mount's own buff (`Mount.CheckBuff` =
        // `data.buff == b`) dismounts and clears the buff — no `DelBuff`.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.mount_cart();
        let mount_buff = p.mount.data().unwrap().buff;
        p.buffs.add_buff(mount_buff, 3600);
        let slot = p.buffs.find_buff_index(mount_buff).unwrap();
        assert!(
            p.try_removing_buff(&w, slot, mount_buff, [0, 0]),
            "the arm entered"
        );
        assert!(!p.mount.is_active(), "the mount dismounted");
    }

    /// S1-P-i53 — `TryRemovingBuff_CheckBuffHideMisc` (`Main.cs:43390-43404`):
    /// cancelling the equipped pet's buff hides the misc visual; the 101/102
    /// light buffs also match the Fairy Bell's legacy `buffType 27`
    /// (`:43395-43398`); an already-hidden slot stays hidden (no flip), an
    /// unrelated buff never hides.
    #[test]
    fn s1_p_i53_the_buff_cancel_hides_the_matching_misc_pet() {
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(40, 3600); // a vanity pet buff
        let slot = p.buffs.find_buff_index(40).unwrap();
        assert!(p.try_removing_buff(&w, slot, 40, [40, 0]));
        assert!(p.hide_misc[0], "the pet visual hid (`:43401`)");
        assert!(!p.hide_misc[1], "the light slot untouched");
        assert_eq!(p.buffs.find_buff_index(40), None, "the pet buff del'd");
        // The light-pet 101 cancel matches a `buffType == 27` equip.
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(101, 3600);
        let slot = p.buffs.find_buff_index(101).unwrap();
        assert!(p.try_removing_buff(&w, slot, 101, [0, 27]));
        assert!(p.hide_misc[1], "the 27 legacy alias matched (`:43397`)");
        // A non-matching buff leaves both flags down; the buff still dels.
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(5, 3600);
        let slot = p.buffs.find_buff_index(5).unwrap();
        assert!(p.try_removing_buff(&w, slot, 5, [40, 27]));
        assert_eq!(p.hide_misc, [false, false]);
    }

    /// S1-P-i53 — the mount arm latches `flag = true` on the CheckBuff match
    /// even when `TryDismount` refuses (`Main.cs:43377-43378`): a CCed rider
    /// on the Bat (56, `DontDismountWhenCCed`, `MountID.cs:47`) keeps the
    /// mount AND the buff — no `DelBuff` (`:43383-43386`).
    #[test]
    fn s1_p_i53_a_refused_dismount_keeps_the_mount_buff() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(56, false)); // the Bat — buff 377
        p.frozen = true; // CCed (`Player.cs:3377-3385`)
        let slot = p.buffs.find_buff_index(377).unwrap();
        assert!(
            p.try_removing_buff(&w, slot, 377, [0, 0]),
            "the arm entered — the sound plays (`:43382`)"
        );
        assert!(p.mount.is_active(), "the CC gate refused the dismount");
        assert!(
            p.buffs.find_buff_index(377).is_some(),
            "flag=true keeps DelBuff out (`:43383`)"
        );
        // Un-CCed, the same cancel dismounts and clears.
        p.frozen = false;
        let slot = p.buffs.find_buff_index(377).unwrap();
        assert!(p.try_removing_buff(&w, slot, 377, [0, 0]));
        assert!(!p.mount.is_active());
        assert_eq!(p.buffs.find_buff_index(377), None);
    }

    /// S1-P-i53 — the cart ride's cancel routes through `try_cart_dismount`
    /// (the ride flags live outside `MountState`): `cart_mounted`, the ride
    /// flip/boost and `on_track` all reset, and the buff clears
    /// (`Mount.cs:6150-6162`).
    #[test]
    fn s1_p_i53_the_cart_cancel_resets_the_ride_flags() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        p.mount_cart();
        p.cart_flip = true;
        p.last_boost = (2, 3);
        p.on_track = true;
        let mount_buff = p.mount.data().unwrap().buff;
        let slot = p.buffs.find_buff_index(mount_buff).unwrap();
        assert!(p.try_removing_buff(&w, slot, mount_buff, [0, 0]));
        assert!(!p.cart_mounted, "the ride state cleared");
        assert!(!p.mount.is_active());
        assert!(!p.cart_flip);
        assert_eq!(p.last_boost, (0, 0));
        assert!(!p.on_track);
        assert_eq!(p.buffs.find_buff_index(mount_buff), None);
    }

    /// S1-P-i53 — `Player.ClearBuff` (`Player.cs:5472-5481`) walks the raw
    /// `buffType` slots: an IMMUNE type's live slot still clears (the
    /// `FindBuffIndex` gate `:5249` must not leak into it), and a slot whose
    /// timer already expired (`buffTime == 0`) clears too.
    #[test]
    fn s1_p_i53_clear_buff_type_ignores_the_immune_and_time_gates() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(44, false)); // the pirate ship grants its buff
        let buff = p.mount.data().unwrap().buff;
        let slot = p.buffs.find_buff_index(buff).unwrap();
        // The immunity lands AFTER the grant — `find_buff_index` now blinds.
        p.buffs.buff_immune[buff as usize] = true;
        assert_eq!(p.buffs.find_buff_index(buff), None, "the gate blinding");
        assert!(p.dismount(&w, true));
        assert_eq!(p.buffs.buff_type[slot], 0, "ClearBuff walked the raw slot");
        // The expired-timer slot: type set, time 0 — still cleared.
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(44, false));
        let buff = p.mount.data().unwrap().buff;
        let slot = p.buffs.find_buff_index(buff).unwrap();
        p.buffs.buff_time[slot] = 0;
        assert!(p.dismount(&w, true));
        assert_eq!(p.buffs.buff_type[slot], 0);
    }

    /// S1-P-i53 — `Mount.Dismount`'s cart arm (`Mount.cs:6156-6160`): the
    /// framework dismount clears `cartFlip`/`lastBoost` on the Cart rows —
    /// the port used to reset them only in the ride-only `cart_dismount`.
    #[test]
    fn s1_p_i53_the_framework_dismount_clears_the_cart_ride_state() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.mount_cart(); // SetMount(6) — a Cart row
        p.cart_flip = true;
        p.last_boost = (7, 9);
        assert!(p.dismount(&w, true));
        assert!(!p.cart_flip, "`:6158`");
        assert_eq!(p.last_boost, (0, 0), "`:6159`");
        // A non-cart mount leaves the ride fields alone (`:6156` gate).
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(44, false)); // the pirate ship
        p.cart_flip = true;
        p.last_boost = (7, 9);
        assert!(p.dismount(&w, true));
        assert!(p.cart_flip);
        assert_eq!(p.last_boost, (7, 9));
    }

    #[test]
    fn the_moon_events_block_the_respawn_skip() {
        // `Player.cs:38984-38986`: `Main.snowMoon || Main.pumpkinMoon ||
        // DD2Event.Ongoing` refuses the timer skip.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.dead_time = 200;
        p.respawn_timer = 600;
        assert!(p.allows_respawn_timer_skip(false, false, false));
        assert!(!p.allows_respawn_timer_skip(false, false, true));
    }

    /// BUG-06b: `Player.NearAnyNPCsThatBlockRespawn` (`Player.cs:38987-39018`)
    /// — the live invasion's members inside the view rect inflated by 320 px
    /// block the skip; the same NPCs do not while the invasion is over, and a
    /// non-invasion NPC never does (the danger-set arm has no ported members).
    #[test]
    fn bug_06b_only_the_live_invasions_npcs_block_the_respawn_skip() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let (px, py) = p.center();
        let goblin =
            crate::npc::Npc::new(crate::npc::NpcKind::GoblinWarrior, px + 200.0, py, 1, 0.0);
        let slime = crate::npc::Npc::new(crate::npc::NpcKind::Slime, px + 200.0, py, 1, 0.0);
        // Inside the rect while the invasion runs — blocked.
        assert!(p.near_any_npcs_that_block_respawn(&[goblin.clone()], true));
        // The invasion ended: the same goblin no longer blocks.
        assert!(!p.near_any_npcs_that_block_respawn(&[goblin.clone()], false));
        // A commoner NPC is not an invasion group member.
        assert!(!p.near_any_npcs_that_block_respawn(&[slime], true));
        // Out of the inflated view rect (1920x1080 + 320 px): no block.
        let far = crate::npc::Npc::new(crate::npc::NpcKind::GoblinWarrior, px + 4000.0, py, 1, 0.0);
        assert!(!p.near_any_npcs_that_block_respawn(&[far], true));
        // The gate itself: a nearby danger NPC vetoes the skip.
        p.dead_time = 200;
        assert!(p.allows_respawn_timer_skip(false, false, false));
        assert!(!p.allows_respawn_timer_skip(false, true, false));
    }

    #[test]
    fn the_skip_needs_the_90_tick_lockout_and_a_jump_edge() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        // Inside the 90-tick lockout: no skip even on the jump edge (the
        // held jump's edge is consumed while the lockout runs).
        for _ in 0..89 {
            assert!(!p.update_dead(true, false, false, false));
        }
        // The lockout expired but the held jump has no fresh edge; a release
        // and a new press do.
        assert!(!p.update_dead(true, false, false, false));
        assert!(!p.update_dead(false, false, false, false));
        assert!(p.update_dead(true, false, false, false));
    }

    #[test]
    fn the_skip_is_blocked_while_a_boss_hinders() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        for _ in 0..95 {
            p.update_dead(false, false, false, false);
        }
        assert!(!p.allows_respawn_timer_skip(true, false, false));
        assert!(p.allows_respawn_timer_skip(false, false, false));
    }

    #[test]
    fn the_jump_edge_is_required_for_the_skip() {
        // Holding jump through the lockout does not skip on expiry; a fresh
        // press does (the JustBecameTrue semantics of PlayerInput.Triggers).
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        for _ in 0..95 {
            p.update_dead(true, false, false, false);
        }
        // The held jump consumed its edge at the lockout expiry...
        assert!(!p.update_dead(true, false, false, false));
        // ...release and press again: the edge fires.
        p.update_dead(false, false, false, false);
        assert!(p.update_dead(true, false, false, false));
    }

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    pub(super) fn ground_world() -> World {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        w
    }

    #[test]
    fn falls_and_lands_on_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "player should land, y={}", p.y);
        assert!((p.y - (60.0 * TILE_PX - HEIGHT)).abs() < 1.0, "y={}", p.y);
    }

    /// Drops the player from `from_tile` onto the ground_world surface (top
    /// row 60) and simulates until rest. Returns (hp lost, landed tile row).
    fn drop_and_settle(w: &World, from_tile: i32) -> (i32, i32) {
        let mut p = Player::spawn(w);
        let hp0 = p.hp;
        p.y = (from_tile * 16) as f32;
        p.fall_start = from_tile;
        for _ in 0..600 {
            p.update(w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        (hp0 - p.hp, (p.y / TILE_PX) as i32)
    }

    /// A `ground_world` whose surface top row is `kind` (ice 161, cobweb 229,
    /// asphalt 198 — all solid in `TILE_SOLID`).
    fn floor_world(kind: u16) -> World {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = kind;
        }
        w
    }

    /// Walks right for `ticks` with the input held, returning the final vx.
    fn walk(w: &World, ticks: u32) -> f32 {
        let mut p = Player::spawn(w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..ticks {
            p.update(w, input);
        }
        p.vx
    }

    /// `Player.cs:26560-26570`: plain ice multiplies `runSlowdown` by 0.1 when
    /// nobody skates — releasing the key, the player slides ten times further
    /// than on dirt.
    #[test]
    fn ice_slides_without_skates() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..60 {
            p.update(&ice, input);
        }
        let vx0 = p.vx;
        for _ in 0..30 {
            p.update(&ice, PlayerInput::default());
        }
        let on_ice = p.vx;
        let mut q = Player::spawn(&ground_world());
        q.vx = vx0;
        for _ in 0..30 {
            q.update(&ground_world(), PlayerInput::default());
        }
        let on_dirt = q.vx;
        assert!(vx0 > 2.5, "should have accelerated, vx={vx0}");
        assert!(on_ice > 1.5, "ice keeps sliding: {on_ice}");
        assert!(on_dirt < 0.05, "dirt friction stops the player: {on_dirt}");
    }

    /// `Player.cs:26562-26565`: with `iceSkate` the ice multiplies
    /// `runAcceleration` by 3.5 and `maxRunSpeed` by 1.25 — the top speed on
    /// ice exceeds the dirt one (3.0).
    #[test]
    fn ice_skates_raise_the_top_speed() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        p.equipped[3] = 950; // the Ice Skates (`Player.cs:15089-15092`)
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..120 {
            p.update(&ice, input);
        }
        assert!(
            p.vx > 3.7 && p.vx < 4.0,
            "skate top speed ≈ 3.75, vx={}",
            p.vx
        );
        let dirt_top = walk(&ground_world(), 120);
        assert!(
            dirt_top > 2.9 && dirt_top < 3.2,
            "dirt top speed = 3, vx={dirt_top}"
        );
    }

    /// `Player.cs:26522-26534`: the cobweb multiplies speed 0.25x, slowdown
    /// 2x, and hard-clamps the current velocity to the reduced max.
    #[test]
    fn cobweb_slow_and_clamps() {
        let web = floor_world(229);
        let mut p = Player::spawn(&web);
        for _ in 0..300 {
            p.update(&web, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        // Standing on the web, the flags are live: an entry speed at
        // knockback strength clamps to the reduced max on the next tick.
        p.vx = 4.5;
        p.update(&web, PlayerInput::default());
        assert!(p.vx < 1.0, "web clamps and drags: vx={}", p.vx);
    }

    /// `Player.cs:23436-23451`: in water the free axes integrate at 0.5, in
    /// honey at 0.25 — momentum (vx) is untouched, the stride halves.
    #[test]
    fn water_halves_the_stride() {
        let mut w = ground_world();
        for y in 58..60 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let mut p = Player::spawn(&w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..90 {
            p.update(&w, input);
        }
        // Both runners are settled into their gait; measure the stride over
        // the next 30 ticks only.
        let x0 = p.x;
        let mut q = Player::spawn(&ground_world());
        for _ in 0..90 {
            q.update(&ground_world(), input);
        }
        let q0 = q.x;
        for _ in 0..30 {
            p.update(&w, input);
            q.update(&ground_world(), input);
        }
        let ratio = (p.x - x0) / (q.x - q0);
        assert!(
            (0.4..0.65).contains(&ratio),
            "wet stride ≈ half the dry one: {ratio}"
        );
        // Momentum survives: the velocity is not scaled, only the stride.
        assert!(p.vx > 2.0, "vx preserved in water: {}", p.vx);
    }

    /// `Player.cs:20059-20087`: no input decays by `runSlowdown` on the ground
    /// and half of it in the air (0.2 vs 0.1 per tick).
    #[test]
    fn air_friction_is_half_the_ground_friction() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 2.0;
        p.on_ground = false;
        p.vy = -3.0; // airborne
        p.update(&w, PlayerInput::default());
        let air = 2.0 - p.vx;
        let mut q = Player::spawn(&w);
        q.vx = 2.0;
        q.on_ground = true;
        q.vy = 0.0;
        q.update(&w, PlayerInput::default());
        let ground = 2.0 - q.vx;
        assert!(
            (air - ground * 0.5).abs() < 1e-4,
            "air {air} vs ground {ground}"
        );
    }

    /// The port used to hard-clamp vx to ±3 every tick — the vanilla has no
    /// clamp: a knockback-speed vx (4.5) decays by friction instead.
    #[test]
    fn overspeed_rides_out() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 4.5;
        p.update(&w, PlayerInput::default());
        assert!(
            p.vx > 4.0 && p.vx < 4.5,
            "no clamp, friction only: vx={}",
            p.vx
        );
    }

    /// Simulates the double-tap: a right press edge, a release, a second
    /// press `gap` ticks later. Returns (dashDelay, eocDash, vx) one tick
    /// after the second press.
    fn double_tap(w: &World, gap: u32) -> (i32, i32, f32) {
        let mut p = Player::spawn(w);
        for _ in 0..120 {
            p.update(w, PlayerInput::default());
        }
        p.pinned_dash = 2;
        p.update(
            w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        for _ in 0..gap {
            p.update(w, PlayerInput::default());
        }
        p.update(
            w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        (p.dash_delay, p.eoc_dash, p.vx)
    }

    /// `Player.cs:21571-21600`: a second same-direction tap inside the 15-tick
    /// window fires the dash — velocity 14.5 (dash 2), `dashDelay = -1`, the
    /// EoC grace armed at 15.
    #[test]
    fn double_tap_fires_the_dash() {
        let w = ground_world();
        let (delay, eoc, vx) = double_tap(&w, 5);
        assert_eq!(delay, -1, "dashing");
        assert_eq!(eoc, 15, "the EoC grace arms at the start");
        // The launch tick already took the ground friction AND the first
        // decay: (14.5 - 0.2) x 0.985.
        assert!(
            (vx - 14.3 * 0.985).abs() < 1e-4,
            "shield dash speed, vx={vx}"
        );
    }

    /// The window: a second tap after 15 ticks is too late — `dashTime` has
    /// decayed to 0 and re-arms instead of firing.
    #[test]
    fn double_tap_window_is_15_ticks() {
        let w = ground_world();
        let (delay, _, _) = double_tap(&w, 20);
        assert_eq!(delay, 0, "outside the window");
    }

    /// `Player.cs:21374-21398`: the dash decays 0.985/tick above the 12-tile
    /// threshold; the end starts the 30-tick shield cooldown and pins the
    /// velocity to the run band.
    #[test]
    fn shield_dash_decays_into_cooldown() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.pinned_dash = 2;
        p.dash = 2;
        p.dash_delay = -1;
        p.eoc_dash = 15;
        p.vx = 14.5;
        let mut max_delay = 0;
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
            max_delay = max_delay.max(p.dash_delay);
        }
        // The dash ended into the 30-tick shield cooldown (the highest count
        // observed) and the cooldown has fully run out by now.
        assert_eq!(max_delay, 30, "the shield cooldown");
        assert_eq!(p.dash_delay, 0, "the cooldown expired");
        assert!(
            p.vx.abs() <= 3.0 + 0.09,
            "pinned to the run band: vx={}",
            p.vx
        );
        assert_eq!(p.eoc_dash, 0, "the grace drained on the cooldown");
    }

    /// `Player.cs:21414-21417`: a solid tile ahead halves the launch speed.
    #[test]
    fn wall_halves_the_launch() {
        let mut w = ground_world();
        for y in 55..62 {
            for x in [12, 13] {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = Player::spawn(&w);
        p.pinned_dash = 2;
        let held = PlayerInput {
            right: true,
            ..Default::default()
        };
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        for _ in 0..3 {
            p.update(&w, PlayerInput::default());
        }
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        assert!(
            p.vx.abs() < 14.5,
            "the launch halved against the wall: vx={}",
            p.vx
        );
    }

    /// The wings flight: `TR_WINGS`-shaped state — wings_logic 1 (100-tick
    /// budget), jump held midair after the normal jump drains: the ascent
    /// caps at `-jumpSpeed * 1.66` and the budget drains 1/tick.
    #[test]
    fn wings_lift_and_drain_the_budget() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let ground_y = p.y;
        // Demon Wings grant `wings_logic`/`wing_time_max` inside the tick.
        p.equipped[3] = 492;
        p.wing_time = 100.0;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        // Jump first (the flight needs `jump == 0`), then hold.
        p.update(&w, held);
        let mut min_vy = 0.0f32;
        for _ in 0..60 {
            p.update(&w, held);
            min_vy = min_vy.min(p.vy);
        }
        assert!(p.y < ground_y - 200.0, "the player rose: y={}", p.y);
        assert!(
            min_vy >= -JUMP_SPEED * 1.66 - 0.5 && min_vy < 0.0,
            "the ascent cap: {min_vy}"
        );
        assert!(p.wing_time < 100.0, "the budget drains: {}", p.wing_time);
    }

    /// `Player.cs:26776-26781`: the wing refill on the ground needs the jump
    /// RELEASE — holding jump on the ground does not refill.
    #[test]
    fn wing_refill_needs_the_jump_release() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // Demon Wings grant `wings_logic`/`wing_time_max` inside the tick.
        p.equipped[3] = 492;
        p.wing_time = 30.0;
        // Land and release: the refill.
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wing_time, 100.0, "released on the ground");
        // Hold jump: no refill while grounded.
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.wing_time = 30.0;
        p.update(&w, held);
        assert_eq!(p.wing_time, 30.0, "holding jump blocks the refill");
    }

    /// `Player.cs:26867-26955`: the rocket boots — a midair jump release
    /// arms `rocketRelease`; holding jump again burns a fuel tick per 10
    /// thrust ticks at 0.1 + brake impulses, capped at `-jumpSpeed * 1.5`.
    #[test]
    fn rocket_boots_thrust_after_the_rearm() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.rocket_time = 7;
        // Jump and release midair: the arming.
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        assert!(p.rocket_release, "released midair");
        // Hold again: the thrust (the ascent cap -5.01 * 1.5 = -7.515).
        let mut min_vy = 0.0f32;
        for _ in 0..30 {
            p.update(&w, held);
            min_vy = min_vy.min(p.vy);
        }
        assert!(
            min_vy <= -JUMP_SPEED * 1.5 + 0.5,
            "the thrust cap: {min_vy}"
        );
        assert!(p.rocket_time < 7, "the fuel drains: {}", p.rocket_time);
    }

    /// `Player.cs:26825-26834`: with wings AND boots the rocket fuel
    /// converts into wing time (+6 per fuel tick, capped past the max)
    /// before the rocket can trigger.
    #[test]
    fn rocket_fuel_converts_into_wing_time() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The wings come from the item now (Demon Wings = `ArmorIDs.Wing` 1,
        // 100 ticks): the grant runs inside the tick, so the budget starts
        // empty and the rocket fuel is the only source this tick.
        p.equipped[3] = 492;
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.rocket_time = 7;
        // Airborne with both: the conversion consumed the fuel.
        p.vy = 1.0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_time, 0, "the fuel converted");
        assert_eq!(p.wing_time, 42.0, "7 fuel ticks x 6 = 42 wing ticks");
    }

    /// `Player.GrappleMovement` (`Player.cs:22613-22740`): while a hook is
    /// attached the pull replaces the velocity — the vector to the hook
    /// center clamped to the pull speed.
    #[test]
    fn grapple_pull_replaces_the_velocity() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        let hook_center = Vec2::new(p.x + 80.0, p.y - 60.0);
        // One registered Hook — the multi-hook seam's single-member case.
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, hook_center, 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        // The pull toward the hook: dx +, dy -.
        assert!(p.vx > 0.0 && p.vy < 0.0, "pulling toward the hook");
        // The speed cap 11 px/tick for the Hook (13).
        assert!(
            Vec2::new(p.vx, p.vy).length() <= 11.0 + 1e-4,
            "the pull speed capped: {}",
            Vec2::new(p.vx, p.vy).length()
        );
        // GoingDownWithGrapple: the pull points up here.
        assert!(!p.going_down_with_grapple);
    }

    /// `Player.GrappleMovement`'s dismount (`Player.cs:22620-22623`): a mount
    /// outside `MountID.Sets.CanUseHooks` ({54, 57-60}) dismounts while the
    /// hooks are out — the port dismounted every mount.
    #[test]
    fn s1_l_10_a_mount_outside_can_use_hooks_dismounts() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // The Unicorn (10) is not in the CanUseHooks set.
        p.set_mount(10, false);
        assert!(p.mount.is_active(), "the mount is up");
        let grapple: &[(u16, Vec2, f32)] =
            &[(crate::projectile::HOOK, Vec2::new(p.x, p.y - 100.0), 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.mount.is_active(), "the mount dismounted");
    }

    /// The same gate at the fire edge (`Player.QuickGrapple`,
    /// `Player.cs:6334-6337`): the set's own rows stay mounted.
    #[test]
    fn s1_l_10_a_mount_inside_can_use_hooks_stays() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // The row 54 is in `MountID.Sets.CanUseHooks`.
        p.set_mount(54, false);
        assert!(p.mount.is_active(), "the mount is up");
        let grapple: &[(u16, Vec2, f32)] =
            &[(crate::projectile::HOOK, Vec2::new(p.x, p.y - 100.0), 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(p.mount.is_active(), "the mount stays mounted");
    }

    /// `Player.cs:22645`: the fall origin refreshes per tick while
    /// grappling — a long pull never accrues fall damage.
    #[test]
    fn grapple_refreshes_the_fall_origin() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        let start_row = (p.y / TILE_PX) as i32;
        let center = Vec2::new(p.x, p.y - 300.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        for _ in 0..40 {
            p.update_with_grapple(
                &w,
                PlayerInput::default(),
                grapple,
                false,
                false,
                false,
                None,
                &mut tr_core::UnifiedRandom::default(),
            );
            // GrappleMovement refreshes the origin BEFORE the position
            // integrates — one row behind the current position, like the
            // original's call order (:27645 vs the collision block).
            let row = (p.y / TILE_PX) as i32;
            assert!(
                (p.fall_start - row).abs() <= 1,
                "the origin rides the pull: {} vs {row}",
                p.fall_start
            );
        }
        let _ = start_row;
    }

    /// `Player.cs:22705-22739`: the jump-off — a fresh jump press while
    /// nearly stationary grants the full jump and detaches.
    #[test]
    fn grapple_jump_off_detaches() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Settle, then latch the hook right above (a near-stationary hang).
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The registered set (`grappling[]`/`grapCount`): one attached Hook.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        // The press edge: the jump-off.
        p.update_with_grapple(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.grappling_active, "detached");
        assert!(p.vy < 0.0, "the full jump impulse: vy={}", p.vy);
    }

    /// `Player.cs:22713-22717`: holding DOWN cancels the stationary jump-off
    /// — the nudge path detaches without the jump.
    #[test]
    fn grapple_jump_off_down_is_a_nudge() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The registered set (`grappling[]`/`grapCount`): one attached Hook.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput {
                jump: true,
                down: true,
                ..Default::default()
            },
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.grappling_active, "detached");
        // The nudge path (`velocity.Y += 0.01f`) does NOT grant the jump
        // counter — the pull velocity carries (the vanilla does not zero it
        // in RemoveAllGrapplingHooks).
        assert_eq!(p.jump, 0, "no jump counter on the nudge path");
    }

    /// `Player.cs:22660-22661`: while pulling DOWN the platforms fall
    /// through (`GoingDownWithGrapple`).
    #[test]
    fn downward_pull_sets_going_down() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // One attached Hook below the player.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y + 120.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(p.going_down_with_grapple, "the pull points down");
    }

    /// W3-i39 — `Player.cs:18506` + `:22655-22658`: `GoingDownWithGrapple` is
    /// cleared per tick by `ResetEffects` and SET only on a down pull. An UP
    /// pull therefore leaves it false (the old port assigned `pull.going_down`,
    /// which also cleared it — but never reset it on a non-grapple tick).
    /// WRITTEN NOT RUN.
    #[test]
    fn i39_going_down_resets_on_an_up_pull() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Prime the flag true (a down pull).
        let below = Vec2::new(p.x + WIDTH * 0.5, p.y + 120.0);
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            &[(crate::projectile::HOOK, below, 0.0)],
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(p.going_down_with_grapple, "the down pull sets it");
        // Now an UP pull: the per-tick reset clears it, the up pull does not re-set.
        let above = Vec2::new(p.x + WIDTH * 0.5, p.y - 120.0);
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            &[(crate::projectile::HOOK, above, 0.0)],
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.going_down_with_grapple, "the up pull leaves it cleared");
    }

    /// W3-i39 — `Player.cs:22717`: the grounded down-pull jump-off is cancelled
    /// unless `controlUp` is held (NOT `controlJump`). With UP held the full
    /// jump impulse fires even though the pull points down. WRITTEN NOT RUN.
    #[test]
    fn i39_the_jump_off_reads_control_up() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // A hook below the player: the pull points down (flag3 when grounded).
        let below = Vec2::new(p.x + WIDTH * 0.5, p.y + 120.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, below, 0.0)];
        // Jump + UP held: flag3 && !up is false → flag4 survives → full jump.
        p.update_with_grapple(
            &w,
            PlayerInput { jump: true, up: true, ..Default::default() },
            grapple,
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(p.vy < 0.0, "UP held grants the jump impulse: vy={}", p.vy);
        assert!(p.release_grappling_hooks, "the jump-off detaches");
    }

    /// W3-i39 retry — `Player.cs:11880`: the Webbed (149) buff arm calls
    /// `RemoveAllGrapplingHooks` — the web snaps the hook. The port raises the
    /// same `release_grappling_hooks` request the jump-off uses. WRITTEN NOT RUN.
    #[test]
    fn i39_webbed_detaches_the_hooks() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(!p.release_grappling_hooks, "no removal before the web");
        p.buffs.add_buff(149, 60);
        p.update(&w, PlayerInput::default());
        assert!(p.webbed, "the webbed flag is set");
        assert!(p.release_grappling_hooks, "the web raises the hook removal");
    }

    /// W3-i39 retry — `Player.cs:22619` (`StopVanityActions`, `:33458-33463`):
    /// `GrappleMovement` sits the player up and stops the sleep. WRITTEN NOT RUN.
    #[test]
    fn i39_grapple_stops_the_vanity_actions() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.sitting = true;
        p.sleeping = true;
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            &[(crate::projectile::HOOK, center, 0.0)],
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.sitting, "the grapple sits the player up");
        assert!(!p.sleeping, "the grapple wakes the player");
    }

    /// W3-i39 retry — `Player.cs:28085-28092` + `DryCollision` (`:23500`): a
    /// downward grapple pull (`GoingDownWithGrapple`) forces platform
    /// fall-through. A player resting on a platform (tile 19, solid-top) with a
    /// hook below drops through it; without the down pull the platform holds.
    /// WRITTEN NOT RUN.
    #[test]
    fn i39_a_down_pull_falls_through_a_platform() {
        let mut w = World::new("t".into(), 64, 128);
        // A platform row at y=60 across the width (tile 19 is solid-top).
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile { kind: 19, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        let mut p = Player::spawn(&w);
        // Settle onto the platform (no grapple): the solid-top holds the player.
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let rest_y = p.y;
        assert!(p.on_ground, "the platform holds the player at rest");
        // A hook far below: the pull points down → GoingDownWithGrapple.
        let below = Vec2::new(p.x + WIDTH * 0.5, p.y + 400.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, below, 0.0)];
        for _ in 0..30 {
            p.update_with_grapple(
                &w,
                PlayerInput::default(),
                grapple,
                false, false, false, None,
                &mut tr_core::UnifiedRandom::default(),
            );
        }
        assert!(p.going_down_with_grapple, "the pull points down");
        assert!(p.y > rest_y + 16.0, "the down pull drops through the platform: {} vs {rest_y}", p.y);
    }

    /// `Player.cs:19963,19993`: the overspeed continuation is gated on
    /// `dashDelay >= 0` — no input acceleration while dashing (the decay
    /// runs instead).
    #[test]
    fn no_input_acceleration_while_dashing() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 492; // Demon Wings
        p.fall_start = 40;
        p.vy = 5.0;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        // The glide cap is the wings arm's `maxFallSpeed / 3` (`:27339`) on
        // the rebased `maxFallSpeed` — `:24437`'s every-tick `+0.01f` rides
        // in (`R-03-1`).
        assert!(
            p.vy <= (MAX_FALL + 0.01) / 3.0 + 1e-4,
            "the glide cap: {}",
            p.vy
        );
        assert_eq!(p.fall_start, (p.y / TILE_PX) as i32, "the origin refreshes");
    }

    /// `Player.cs:25360`: `!hasWings` gates the fall damage — any equipped
    /// wings nullify it (PHYS-2's field, armed here).
    #[test]
    fn wings_nullify_the_fall_damage() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 492; // Demon Wings
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.on_ground && p.hp == 100, "no fall damage with wings");
    }

    /// `Player.cs:27404-27425`: the hover wings holding down dampen vy
    /// toward the float.
    #[test]
    fn hover_wings_dampen_the_descent() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Bee Wings (1866 → `ArmorIDs.Wing` 22, the `HasDownHoverStats` row).
        p.equipped[3] = 1866;
        p.wing_time = 170.0;
        p.vy = 3.0;
        let held = PlayerInput {
            jump: true,
            down: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert!(
            p.vy.abs() < 3.0 && p.vy >= 0.0,
            "the descent dampened: vy={}",
            p.vy
        );
    }

    /// Places `kind` in the accessory slot 0 (= `armor[3]`), runs the tick's
    /// reset + grant pass by calling `update`, then reports the flags.
    fn player_with_accessory(kind: u16) -> Player {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = kind;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    #[test]
    fn the_whip_speed_buffs_fold_through_cap_attack_speeds() {
        // `Player.cs:10110-10121`: Dark Harvest (311) +0.35, Sword Whip (308)
        // +0.25, Thorn Whip (314) +0.12 per slot. `CapAttackSpeeds` (`:29025`)
        // folds `meleeSpeed(1) + bonus` through `TurnAttackSpeedToUseTimeMultiplier`
        // (cap 3, invert) into the use-time multiplier the whip swing reads.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(311, 600); // Dark Harvest
        p.update(&w, PlayerInput::default());
        // bonus 0.35 → 1/(1+0.35) = 0.7407…; whipUseTimeMultiplier 1.
        let want = 1.0 / 1.35;
        assert!(
            (p.whip_animation_multiplier() - want).abs() < 1e-5,
            "the Dark Harvest swing scales to {want}, got {}",
            p.whip_animation_multiplier()
        );
        // The three buffs stack: 0.35 + 0.25 + 0.12 = 0.72 → 1/1.72.
        let mut p2 = Player::spawn(&w);
        p2.buffs.add_buff(311, 600);
        p2.buffs.add_buff(308, 600);
        p2.buffs.add_buff(314, 600);
        p2.update(&w, PlayerInput::default());
        let want2 = 1.0 / 1.72;
        assert!(
            (p2.whip_animation_multiplier() - want2).abs() < 1e-5,
            "the stacked whip speed scales to {want2}, got {}",
            p2.whip_animation_multiplier()
        );
    }

    #[test]
    fn the_obsidian_outlaw_set_speeds_the_whip_and_stretches_its_reach() {
        // `ArmorSetBonuses.cs:120-125`: minionDamage +0.15, whipRange +0.3,
        // whipUseTimeMultiplier ×1/1.15. The set triple is (3266, 3267, 3268).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[0] = 3266;
        p.equipped[1] = 3267;
        p.equipped[2] = 3268;
        p.update(&w, PlayerInput::default());
        assert!(
            (p.whip_range_multiplier - 1.3).abs() < 1e-5,
            "the set adds +0.3 reach, got {}",
            p.whip_range_multiplier
        );
        // No speed buff: summonerWeaponSpeedBonus folds 1/(1+0) = 1, times
        // the set's 1/1.15.
        let want = 1.0 / 1.15;
        assert!(
            (p.whip_animation_multiplier() - want).abs() < 1e-5,
            "the set's use-time scale {want}, got {}",
            p.whip_animation_multiplier()
        );
    }

    #[test]
    fn the_whip_accessories_raise_the_tag_cap_and_duration() {
        // `Player.cs:12970-12984`: Silver Bracer +1 cap / +2 duration, Mobius
        // Strip +1 cap / +0.1 range, Wicked Armlet +1 cap. The base is 1/1f.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 6156; // Silver Bracer
        p.equipped[4] = 6158; // Mobius Strip
        p.equipped[5] = 6159; // Wicked Armlet
        p.update(&w, PlayerInput::default());
        assert!(p.acc_silver_bracer && p.acc_mobius_strip && p.acc_wicked_armlet);
        // cap 1 + 3 = 4; duration 1 + 2 = 3; range 1 + 0.1 = 1.1.
        assert!(
            (p.whip_range_multiplier - 1.1).abs() < 1e-5,
            "the Mobius Strip adds +0.1 reach"
        );
        // The stack caps at 4: four distinct tags all stay active.
        for item in [4672u16, 5074, 4913, 4678] {
            p.tag_effects.try_apply_tag_to_npc(item, 0, crate::npc::NpcKind::Zombie);
        }
        assert_eq!(p.tag_effects.active_types().len(), 4, "the cap is 4");
        // The Silver Bracer's +2 duration stretches the mark: 240 × 3 = 720.
        assert!(p.tag_effects.is_npc_tagged(4678, 0));
        for _ in 0..240 {
            p.tag_effects.update_tick();
        }
        assert!(
            p.tag_effects.is_npc_tagged(4678, 0),
            "the 720-tick mark outlives the base 240"
        );
    }

    #[test]
    fn the_snake_band_cooldown_resets_the_swing_counter() {
        // `Update_SnakeBand` (`Player.cs:28412-28418`): the reset cooldown
        // counts down one per tick; at 0 the swing counter returns to −1.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.repeat_whip_swings = 5;
        p.repeat_whips_reset_cooldown = 2;
        p.update_snake_band();
        assert_eq!(p.repeat_whip_swings, 5, "not yet reset");
        assert_eq!(p.repeat_whips_reset_cooldown, 1);
        p.update_snake_band();
        assert_eq!(p.repeat_whip_swings, -1, "the counter resets at 0");
        assert_eq!(p.repeat_whips_reset_cooldown, 0);
        // A zero cooldown is inert (the `> 0` gate).
        p.repeat_whip_swings = 3;
        p.update_snake_band();
        assert_eq!(p.repeat_whip_swings, 3, "the idle cooldown changes nothing");
    }

    #[test]
    fn the_accessory_grants_arm_the_jumps() {
        // `ApplyEquipFunctional`'s bottle grants (`Player.cs:14997-15001`,
        // :14793-14811): the bottles arm their jump options; the reset clears
        // and the pass re-arms every tick.
        let p = player_with_accessory(53);
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert!(!p.has_jump_options[ExtraJump::Sandstorm.index()]);
        let p = player_with_accessory(857);
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        let p = player_with_accessory(987);
        assert!(p.has_jump_options[ExtraJump::Blizzard.index()]);
        let p = player_with_accessory(1724);
        assert!(p.has_jump_options[ExtraJump::Fart.index()]);
        let p = player_with_accessory(3201);
        assert!(p.has_jump_options[ExtraJump::Sail.index()]);
        // No accessory: nothing armed.
        let p = player_with_accessory(0);
        assert!(p.has_jump_options.iter().all(|&b| !b));
    }

    // ---- S1-P-i52 — the heart/star magnet grants (written, NOT run) ----

    /// `ApplyEquipFunctional`'s manaMagnet rows (`Player.cs:14940-14953`,
    /// `:14455-14458`): the Celestial Magnet 2219 (bare flag), the Celestial
    /// Emblem 2220 (+15% magic AND the flag), the Celestial Cuffs 2221
    /// (magicCuffs AND the flag), the Magnet Flower 4000 (the 555-style
    /// −0.08 manaCost fold AND the flag). The Magic Cuffs 1595 arm
    /// magicCuffs but NOT the magnet (`:14936-14939`). `ResetEffects`
    /// (`:18491`) clears the flag when the item comes off.
    #[test]
    fn s1_p_i52_the_magnet_accessories_arm_mana_magnet() {
        let p = player_with_accessory(2219);
        assert!(p.mana_magnet, "the Celestial Magnet");
        let p = player_with_accessory(2220);
        assert!(p.mana_magnet, "the Celestial Emblem");
        assert!((p.magic_damage - 1.15).abs() < 1e-4, "its +15% magic");
        let p = player_with_accessory(2221);
        assert!(p.mana_magnet, "the Celestial Cuffs");
        assert!(p.magic_cuffs, "the cuffs half");
        let p = player_with_accessory(4000);
        assert!(p.mana_magnet, "the Magnet Flower");
        assert!((p.mana_cost - 0.92).abs() < 1e-4, "the manaFlower fold");
        let p = player_with_accessory(1595);
        assert!(p.magic_cuffs);
        assert!(!p.mana_magnet, "the plain Magic Cuffs carry no magnet");
        let p = player_with_accessory(0);
        assert!(!p.mana_magnet, "ResetEffects cleared it");
        assert!(!p.life_magnet);
    }

    /// `UpdateBuffs`' Heartreach arm (`Player.cs:10138-10141`): buff 105
    /// arms `lifeMagnet` per tick; the expiry + `ResetEffects` (`:18492`)
    /// clear it.
    #[test]
    fn s1_p_i52_the_heartreach_buff_arms_life_magnet() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(105, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.life_magnet, "buff 105 armed the heart magnet");
        // The buff gone: the next tick's reset clears the flag.
        p.buffs.clear_buff(105);
        p.update(&w, PlayerInput::default());
        assert!(!p.life_magnet, "the expiry cleared it");
    }

    /// `Player.Heal` (`Player.cs:35112-35123`): the cap AFTER the queue —
    /// the text carries the raw amount. `TryApplyManaHeat`
    /// (`:52006-52014`): the pair writes, the 120-tick buff, and the
    /// zero-power/zero-time arm that writes the pair but adds NO buff.
    #[test]
    fn s1_p_i52_heal_caps_after_the_text_and_mana_heat_gates_the_buff() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.max_hp = 100;
        p.hp = 95;
        p.heal(20);
        assert_eq!(p.hp, 100, "capped");
        assert_eq!(p.take_heal_effect(), Some(20), "the RAW amount queued");
        assert_eq!(p.take_heal_effect(), None, "the text is a one-shot");
        p.try_apply_mana_heat(0.2, 2.0);
        assert_eq!(p.latest_mana_potion_damage_bonus, 0.2);
        assert!(
            (p.latest_mana_potion_duration - 120.0).abs() < 1e-3,
            "2 s * 60 = 120 ticks: {}",
            p.latest_mana_potion_duration
        );
        assert!(p.buffs.find_buff_index(396).is_some());
        // The zero arm: the pair still writes, no buff added.
        let mut p = Player::spawn(&w);
        p.try_apply_mana_heat(0.0, 0.0);
        assert_eq!(p.latest_mana_potion_duration, 0.0);
        assert!(p.buffs.find_buff_index(396).is_none(), "the gate refused");
    }

    #[test]
    fn the_hermes_boots_lift_the_acceleration_gate() {
        // `Player.cs:15005-15008`: `accRunSpeed = 6f` — the base reset (3.0)
        // is lifted, the debuff multiplier still applies afterwards.
        let p = player_with_accessory(54);
        assert!(
            (p.acc_run_speed_value() - 6.0).abs() < 1e-4,
            "{}",
            p.acc_run_speed_value()
        );
        // The vanilla leaves `maxRunSpeed` at 3.0 — the boots only widen the
        // acceleration window (the continuation to accRunSpeed).
        assert!((p.max_run_speed_value() - 3.0).abs() < 1e-4);
    }

    /// S1-C-12 wave A — the movement-speed accessory family, per item.
    #[test]
    fn the_movement_speed_accessories_stack_their_rows() {
        // Hermes (54, `:15005-15008`), Flurry (1579, `:15013-15016`) and
        // Sailfish (3200, `:15017-15020`): each sets `accRunSpeed = 6f`.
        for kind in [54u16, 1579, 3200] {
            let p = player_with_accessory(kind);
            assert!(
                (p.acc_run_speed_value() - 6.0).abs() < 1e-4,
                "kind {kind}: {}",
                p.acc_run_speed_value()
            );
            assert!((p.move_speed - 1.0).abs() < 1e-4, "no moveSpeed row");
        }
        // The Aglet (285, `:15114-15117`): +5% moveSpeed, no run-speed row.
        let p = player_with_accessory(285);
        assert!((p.move_speed - 1.05).abs() < 1e-4);
        assert!((p.acc_run_speed_value() - 3.0).abs() < 1e-4);
        assert!((p.max_run_speed_value() - 3.0 * 1.05).abs() < 1e-4);
        // The Anklet of the Wind (212, `:15118-15121`): +10%.
        let p = player_with_accessory(212);
        assert!((p.move_speed - 1.1).abs() < 1e-4);
        assert!((p.max_run_speed_value() - 3.0 * 1.1).abs() < 1e-4);
        // Both together (`:12946-12952` walks every slot): 1 + 0.05 + 0.1.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 285;
        p.equipped[4] = 212;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.15).abs() < 1e-4);
        // The boot tiers: Lightning 898 (`:15377-15382`) and Frostspark 1862
        // (`:15383-15389`) / Terraspark 5000 (`:15390-15396`): 6.75 + 8%.
        for kind in [898u16, 1862, 5000] {
            let p = player_with_accessory(kind);
            assert!((p.acc_run_speed_value() - 6.75).abs() < 1e-4, "kind {kind}");
            assert!((p.move_speed - 1.08).abs() < 1e-4, "kind {kind}");
        }
        // Spectre 405 (`:15219-15223`) / Fairy 3993 (`:14418-14421`): 6f + tier 2.
        for kind in [405u16, 3993] {
            let p = player_with_accessory(kind);
            assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4, "kind {kind}");
            assert_eq!(p.rocket_boots, 2, "kind {kind}");
        }
        // Hellfire Treads 4874 (`:15397-15401`): 6f + tier 5.
        let p = player_with_accessory(4874);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
        assert_eq!(p.rocket_boots, 5);
        // Sand Boots 4055 (`:14422-14425`): 6f + the desert-run flag.
        let p = player_with_accessory(4055);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
        assert!(p.desert_boots);
        // Amphibian Boots 3990 (`:14399-14404`): 6f, autoJump, +1.6 jump
        // speed, +10 safe-fall tiles.
        let p = player_with_accessory(3990);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
        assert!(p.auto_jump);
        assert!((p.jump_speed - (5.01 + 1.6)).abs() < 1e-4);
        assert_eq!(p.extra_fall, 10);
        // The Empress Brooch 4989 (`:14771-14775`): +7.5%.
        let p = player_with_accessory(4989);
        assert!((p.move_speed - 1.075).abs() < 1e-4);
    }

    #[test]
    fn s1_p_i49_the_arcane_mana_grant_rebases_every_tick() {
        // `GrantPrefixBenefits`' Arcane (66) adds `statManaMax2 += 20`
        // (`Player.cs:13971-13974`); `ResetEffects` rebases
        // `statManaMax2 = statManaMax` (`:18389`) every tick, so the grant
        // is a steady +20 — not a per-tick compound, and it leaves when
        // the accessory comes off.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 285; // the Aglet — a functional accessory
        p.equipped_prefix[3] = 66;
        for _ in 0..10 {
            p.update(&w, PlayerInput::default());
        }
        assert_eq!(p.mana_max, 40, "20 base + 20 Arcane, steady over 10 ticks");
        assert_eq!(p.mana_max_base, 20, "the permanent cap is untouched");
        // Unequip: the next tick's rebase drops the grant.
        p.equipped[3] = 0;
        p.equipped_prefix[3] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 20, "the grant leaves with the accessory");
        // The Magic Power buff's +20 (`:11890`) stacks on the same rebase.
        p.buffs.add_buff(29, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mana_max, 40, "the buff's +20 on the rebased cap");
    }

    /// S1-C-12 wave A — the prefix speed rows ride the accessory's rolled
    /// prefix (`GrantPrefixBenefits`, `Player.cs:14015-14030`).
    #[test]
    fn the_speed_prefixes_ride_the_accessory_slot() {
        let w = ground_world();
        for (prefix, add) in [(73u16, 0.01f32), (74, 0.02), (75, 0.03), (76, 0.04)] {
            let mut p = Player::spawn(&w);
            p.equipped[3] = 285; // the Aglet's own +5%
            p.equipped_prefix[3] = prefix;
            p.update(&w, PlayerInput::default());
            assert!(
                (p.move_speed - (1.05 + add)).abs() < 1e-4,
                "prefix {prefix}: {}",
                p.move_speed
            );
        }
        // The prefix on an empty slot grants nothing (`:12922` IsAir gate).
        let mut p = Player::spawn(&w);
        p.equipped_prefix[3] = 76;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.0).abs() < 1e-4);
    }

    /// S1-C-12 wave A — the debuff multiply runs AFTER the equip grants
    /// (`Player.cs:26004-26009` follows UpdateEquips' `:25633`).
    #[test]
    fn the_slow_debuff_damps_the_accessory_speeds() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 54; // Hermes: accRunSpeed 6
        p.equipped[4] = 212; // Anklet: +10% moveSpeed
        p.buffs.add_buff(32, 600); // Slow: strongestMoveSpeedDebuff 0.5
        p.update(&w, PlayerInput::default());
        assert!((p.acc_run_speed_value() - 3.0).abs() < 1e-4, "6 * 0.5");
        assert!((p.move_speed - 0.55).abs() < 1e-4, "1.1 * 0.5");
        assert!((p.max_run_speed_value() - 1.65).abs() < 1e-4, "3 * 0.55");
    }

    /// S1-C-12 wave A — the dash granters arm `dashType` per tick and the
    /// reset clears it when they come off (`ResetEffects` `:18659`).
    #[test]
    fn the_dash_accessories_arm_the_dash_type() {
        // Tabi 977 (`:15050-15053`): type 1.
        let p = player_with_accessory(977);
        assert_eq!(p.dash_type, 1);
        // Shield of Cthulhu 3097 (`:15054-15057`): type 2.
        let p = player_with_accessory(3097);
        assert_eq!(p.dash_type, 2);
        // Master Ninja Gear 984 (`:15062-15067`): type 1 + blackBelt.
        let p = player_with_accessory(984);
        assert_eq!(p.dash_type, 1);
        assert!(p.black_belt);
        // Taking it off clears the capability on the next tick.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 977;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.dash_type, 1);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.dash_type, 0);
    }

    /// S1-C-12 wave A — the dispatch seam's slot gates
    /// (`IsItemSlotUnlockedAndUsable` `:12579-12604`, the `:12922` accessory
    /// gate via `UpdateEquips_CanItemGrantBenefits` `:12869-12890`).
    #[test]
    fn the_equip_dispatch_gates_the_slots() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Slot 8 needs the Demon Heart flag AND expert mode.
        p.equipped[8] = 212;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.0).abs() < 1e-4, "locked slot 8");
        p.extra_accessory = true;
        p.update(&w, PlayerInput::default());
        assert!(
            (p.move_speed - 1.0).abs() < 1e-4,
            "classic mode stays locked"
        );
        p.buffs.expert_mode = true;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.1).abs() < 1e-4, "expert + Demon Heart");
        // Slot 9 needs master mode - unported, always locked.
        p.equipped[8] = 0;
        p.equipped[9] = 212;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.0).abs() < 1e-4, "slot 9 locked");
        // A non-accessory kind in an accessory slot grants nothing (88 =
        // the Mining Helmet, `headSlot` only). The generic defense loop
        // still reads it - the slot-typed GrantArmorBenefits gate is a
        // separate deviation, untouched here.
        p.equipped[9] = 0;
        p.equipped[3] = 88;
        p.update(&w, PlayerInput::default());
        assert!((p.move_speed - 1.0).abs() < 1e-4);
    }

    #[test]
    fn the_hurt_and_the_death_queue_their_voice_cues() {
        // S1-E-21b — `Player.Hurt`'s tail plays the hurt cue
        // (`Player.cs:38525`) and `Player.KillMe` the death cue
        // (`Player.cs:38868`).
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        assert!(p.take_hurt_sound().is_none(), "nothing queued yet");
        p.hurt_cd = 0.0;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.take_hurt_sound(), Some("Player_Hit_0"));
        // The cue is a one-shot: the caller drains it.
        assert!(p.take_hurt_sound().is_none());
        // The death queues its own cue (`Player.cs:38868`).
        p.start_death(&w, false, false);
        assert_eq!(p.take_death_sound(), Some("Player_Killed"));
    }

    /// S1-E-21 wave D — the fast-run footstep queue
    /// (`Player.SpawnFastRunParticles`, `Player.cs:20205-20216`): past the
    /// `(accRunSpeed + maxRunSpeed) / 2` threshold (`:19779`), grounded and
    /// unmounted, the boot run queues `hermesStepSound` at the top-left
    /// position (`:20214`) and re-arms the 9-tick cooldown (`:20215`);
    /// `Player.Update` ticks it down (`:24706-24708`). The per-cue selection
    /// is `PlayerFrame`'s (`:35947-35964`): default Run 17/-1/9, the Moon
    /// Lord legs (140) Item 2/24/6. Vanilla has NO per-tile-type footstep
    /// cues — grass/stone/snow are the tile-break families (wave C).
    #[test]
    fn s1_e_21d_the_fast_run_queues_the_footstep_cue_and_cooldown() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "landed first");
        assert_eq!(p.hermes_step_sound, (17, -1, 9), "the default cue");
        // Below the threshold (no boots yet: acc == max == 3, threshold 3):
        // the fast-run gate refuses.
        p.vx = -2.5;
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.take_step_sound().is_none(), "2.5 < 3.0: no footstep");
        // Past the threshold (the Hermes boots lift accRunSpeed to 6 every
        // tick — `Player.cs:15005-15008` — so the threshold is 4.5): the cue
        // queues at the top-left position, cooldown 9.
        p.equipped[3] = 54;
        p.update(&w, PlayerInput::default());
        p.vx = -5.0;
        let (x0, y0) = (p.x, p.y);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        let (sound_type, style, sx, sy) = p.take_step_sound().expect("the footstep queued");
        assert_eq!((sound_type, style), (17, -1));
        assert_eq!((sx, sy), (x0, y0), "the position is captured pre-movement");
        // The cooldown re-armed to 9, then the same tick's `:24706-24708`
        // decrement brought it to 8 (the C# order: movement, then Update).
        assert_eq!(p.run_sound_delay, 8);
        for _ in 0..8 {
            p.update(
                &w,
                PlayerInput {
                    left: true,
                    ..Default::default()
                },
            );
            assert!(p.take_step_sound().is_none(), "the cooldown blocks replays");
        }
        assert_eq!(p.run_sound_delay, 0);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        // The movement re-fired at delay 0, then the decrement: 9 → 8.
        assert!(p.take_step_sound().is_some(), "re-armed after the cooldown");
        assert_eq!(p.run_sound_delay, 8);
        // Airborne: the `velocity.Y == 0` gate refuses (`:20212`).
        p.run_sound_delay = 0;
        p.vy = 1.0;
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.take_step_sound().is_none(), "airborne: no footstep");
        // The Moon Lord legs override the cue selection (`:35959-35964`).
        p.legs_armor = 140;
        p.vy = 0.0;
        p.update_frames(&w);
        assert_eq!(p.hermes_step_sound, (2, 24, 6), "the legs-140 override");
        p.run_sound_delay = 0;
        p.vx = -5.0;
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        let (sound_type, style, _, _) = p.take_step_sound().expect("the override cue");
        assert_eq!((sound_type, style), (2, 24));
        assert_eq!(p.run_sound_delay, 5, "the override cooldown 6, minus the tick");
        // Back to the default when the legs come off.
        p.legs_armor = 0;
        p.update_frames(&w);
        assert_eq!(p.hermes_step_sound, (17, -1, 9));
    }

    #[test]
    fn s1_c_7_magic_cuffs_arm_from_the_two_cuffs_and_clear() {
        // `ApplyEquipFunctional`: the Magic Cuffs 1595 (`Player.cs:14936-
        // 14939`) and the Celestial Cuffs 2221 (`:14949-14953`) arm
        // `magicCuffs`; `ResetEffects` clears it per frame (`:18599`) and the
        // fold re-arms from the live items.
        let p = player_with_accessory(1595);
        assert!(p.magic_cuffs);
        let p = player_with_accessory(2221);
        assert!(p.magic_cuffs, "the Celestial Cuffs arm the same flag");
        let p = player_with_accessory(0);
        assert!(!p.magic_cuffs);
        // Armed while equipped, cleared by the next fold without the source.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 1595;
        p.update(&w, PlayerInput::default());
        assert!(p.magic_cuffs);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.magic_cuffs, "the next fold without the source clears it");
    }

    #[test]
    fn s1_c_7_magic_cuffs_restore_the_raw_damage_and_queue_the_text() {
        // `Hurt`'s magicCuffs arm (`Player.cs:38291-38303`): `num3` is the RAW
        // incoming damage (`num`, `:38246` — NOT the defense-reduced `num2`),
        // `statMana += num3` and `ManaEffect(num3)` queues the text.
        let mut p = player_with_accessory(1595);
        p.mana = 0;
        p.mana_max = 100;
        p.stat_defense = 10; // reduces the hit to 15 — the restore stays 20
        p.hurt_cd = 0.0;
        p.immune = false;
        let hp = p.hp;
        assert!(p.hurt(20, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.hp, hp - 15, "the life loss is the reduced amount");
        assert_eq!(p.mana, 20, "the restore is the raw pre-defense damage");
        assert_eq!(p.take_mana_effect(), Some(20));
        assert!(p.take_mana_effect().is_none(), "the text is a one-shot");
    }

    #[test]
    fn s1_c_7_magic_cuffs_cap_the_restore_at_the_mana_max() {
        // `Player.cs:38295-38298`: `if (statMana > statManaMax2) statMana =
        // statManaMax2`. The `ManaEffect` amount stays the uncapped `num3`.
        let mut p = player_with_accessory(2221);
        p.mana = 95;
        p.mana_max = 100;
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(20, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.mana, 100, "capped at statManaMax2");
        assert_eq!(p.take_mana_effect(), Some(20), "the text keeps the raw num3");
    }

    #[test]
    fn s1_c_7_without_the_cuffs_the_hit_restores_nothing() {
        let mut p = player_with_accessory(0);
        p.mana = 10;
        p.mana_max = 100;
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(20, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.mana, 10);
        assert!(p.take_mana_effect().is_none());
    }

    #[test]
    fn s1_c_7b_the_crit_doubles_the_raw_num_not_the_reduced_damage() {
        // `Player.cs:38246-38251`: `num2` is computed from the PRE-crit
        // `Damage` (`:38247`), THEN `if (Crit) num *= 2` — the cuffs restore
        // the doubled raw `num`, the life loss stays the pre-crit reduction.
        let mut p = player_with_accessory(1595);
        p.mana = 0;
        p.mana_max = 200;
        p.stat_defense = 10; // 20 - 5 = 15 either way
        p.hurt_cd = 0.0;
        p.immune = false;
        let hp = p.hp;
        assert!(p.hurt_inner(
            20,
            p.x + 100.0,
            true,
            &mut tr_core::UnifiedRandom::default(),
            true,
            true, // Crit
            None,
            true
        ));
        assert_eq!(p.hp, hp - 15, "num2 used the pre-crit Damage");
        assert_eq!(p.mana, 40, "the cuffs restore the crit-doubled num");
        assert_eq!(p.take_mana_effect(), Some(40));
    }

    #[test]
    fn s1_c_7b_an_accepted_hit_queues_the_retaliate_with_the_final_damage() {
        // `Player.cs:38401`: the Retaliate gate's `flag` is the PRE-hit
        // not-immune state — the port queues only on an accepted hit, with
        // the final post-floor `num2` (`:38378-38383`).
        let mut p = player_with_accessory(0);
        p.stat_defense = 10;
        p.endurance = 0.5; // (20 - 5) * 0.5 = 7.5 → 7
        p.hurt_cd = 0.0;
        p.immune = false;
        let (px, py) = (p.x, p.y);
        let center = p.center();
        let (pvx, pvy) = (p.vx, p.vy);
        assert!(p.hurt(20, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        let req = p.take_retaliate().expect("the accepted hit queues");
        assert_eq!(req.damage, 7, "the post-floor num2");
        assert_eq!((req.x, req.y), (px, py), "the hit-moment position");
        assert_eq!(req.center, center);
        // The snapshot carries the PRE-knockback velocity (`:38430`'s dart
        // reads it before `Hurt`'s own `:38519` knockback overwrites it).
        assert_eq!((req.vx, req.vy), (pvx, pvy));
        assert_eq!(p.vx, -4.5, "the knockback landed on the live player");
        assert!(p.take_retaliate().is_none(), "the request is a one-shot");
        // The iframe-swallowed hit queues nothing.
        assert!(!p.hurt(20, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert!(p.take_retaliate().is_none());
        // A dodged hit queues nothing either (the C# returns before :38401).
        let mut q = player_with_accessory(963); // the black belt
        q.hurt_cd = 0.0;
        q.immune = false;
        struct AlwaysZero;
        impl tr_core::loot::DropRng for AlwaysZero {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                0
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }
        assert!(!q.hurt(20, q.x + 100.0, &mut AlwaysZero, true));
        assert!(q.take_retaliate().is_none());
    }

    #[test]
    fn s1_c_7b_the_dismount_on_hit_queues_only_for_the_set_while_mounted() {
        // `Player.cs:38396-38400` + `ProjectileID.cs:546`: the source type
        // must be in {877, 878, 879} AND `mount.Active`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(2, false), "the slime mount equips");
        for kind in [877u16, 878, 879] {
            p.hurt_cd = 0.0;
            p.immune = false;
            assert!(p.hurt_with_source(
                10,
                p.x + 100.0,
                &mut tr_core::UnifiedRandom::default(),
                true,
                kind
            ));
            assert!(p.take_dismount_request(), "kind {kind} dismounts");
        }
        // Outside the set: no request.
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt_with_source(
            10,
            p.x + 100.0,
            &mut tr_core::UnifiedRandom::default(),
            true,
            100
        ));
        assert!(!p.take_dismount_request());
        // The plain hurt lanes carry no projectile source.
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        assert!(!p.take_dismount_request());
        // Unmounted: the set member queues nothing.
        assert!(p.dismount(&w, false));
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt_with_source(
            10,
            p.x + 100.0,
            &mut tr_core::UnifiedRandom::default(),
            true,
            877
        ));
        assert!(!p.take_dismount_request());
    }

    #[test]
    fn s1_c_7b_the_retaliate_family_arms_from_the_equips_and_clears() {
        // `ApplyEquipFunctional`'s retaliate rows (S1-C-7 wave B): the cloak
        // family, the honeycomb family and the strong-bee flag; the
        // `ResetEffects` clears (`:18322/:18386/:18714-18718`) re-fold.
        let p = player_with_accessory(532);
        assert!(p.star_cloak && !p.star_veil_override);
        let p = player_with_accessory(862);
        assert!(p.star_cloak && p.star_veil_override);
        assert!(p.long_invince, "the Star Veil arms longInvince (:15147)");
        let p = player_with_accessory(1247);
        assert!(p.star_cloak && p.bee_cloak_override && p.honey_comb);
        let p = player_with_accessory(4001);
        assert!(p.star_cloak && p.mana_cloak_override);
        assert!((p.mana_cost - 0.92).abs() < 1e-4, "the manaFlower -0.08 row");
        let p = player_with_accessory(6186);
        assert!(p.star_cloak && p.snake_cloak_override);
        for kind in [1132u16, 1578, 6184, 1249, 3251] {
            let p = player_with_accessory(kind);
            assert!(p.honey_comb, "kind {kind} arms the honeycomb");
        }
        let p = player_with_accessory(1249);
        assert!(p.jump_boost, "the Honey Comb keeps its jump boost");
        let p = player_with_accessory(3251);
        assert!(p.jump_boost && p.no_fall_dmg, "the Honey Balloon's halves");
        let p = player_with_accessory(3333);
        assert!(p.strong_bees);
        // The fold clears without the sources.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 1247;
        p.update(&w, PlayerInput::default());
        assert!(p.star_cloak && p.honey_comb && p.bee_cloak_override);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.star_cloak && !p.honey_comb && !p.bee_cloak_override);
        assert!(!p.strong_bees && !p.snake_cloak_override);
    }

    #[test]
    fn the_godmode_power_swallows_hits_death_and_refills() {
        // `GodmodePower` -> `Player.creativeGodMode` (`Player.cs:25493-25497`);
        // `Hurt` returns 0 before the iframes (`:38203-38206`) and `KillMe`
        // refuses outright (`:38809-38812`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.creative_god_mode = true;
        p.hurt_cd = 0.0;
        let hp = p.hp;
        assert!(
            !p.hurt(
                50,
                p.x + 100.0,
                &mut tr_core::UnifiedRandom::default(),
                true
            ),
            "the hit is swallowed"
        );
        assert_eq!(p.hp, hp, "no damage");
        assert_eq!(p.vx, 0.0, "no knockback");
        p.start_death(&w, false, false);
        assert!(!p.dead, "KillMe refuses while the power is on");
        // Switching the power off restores both arms.
        p.creative_god_mode = false;
        assert!(p.hurt(
            10,
            p.x + 100.0,
            &mut tr_core::UnifiedRandom::default(),
            true
        ));
        p.hp = 0;
        p.start_death(&w, false, false);
        assert!(p.dead);
    }

    #[test]
    fn the_far_placement_power_doubles_the_tile_range() {
        // `Player.ResetEffects` (`Player.cs:18815-18828`): the default
        // `tileRangeX/Y` 5/3 (`:57113-57114`) become `* 2 + 8` = 18/14 while
        // `FarPlacementRangePower` is enabled (`CreativePowers.cs:730-767`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert_eq!(p.tile_range_x(), 5.0);
        assert_eq!(p.tile_range_y(), 3.0);
        p.far_placement_range = true;
        assert_eq!(p.tile_range_x(), 18.0);
        assert_eq!(p.tile_range_y(), 14.0);
    }

    #[test]
    fn the_horseshoe_and_the_shield_grant_their_flags() {
        // `Player.cs:15029-15033` (noFallDmg) + `:15025-15028` (noKnockback).
        let p = player_with_accessory(158);
        assert!(p.no_fall_dmg);
        let p = player_with_accessory(156);
        assert!(p.no_knockback);
        let w = ground_world();

        // The hurt gate: no knockback while the shield is on (the player
        // settles first so the vy assert reads the hurt, not the spawn fall).
        let mut p = player_with_accessory(156);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let from = p.x + 100.0;
        assert!(p.hurt_inner(10, from, true, &mut tr_core::UnifiedRandom::default(), true, false, None, true));
        assert_eq!(p.vx, 0.0, "the shield eats the knockback");
        // Without the shield: the knockback lands.
        let mut p = player_with_accessory(0);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let from = p.x + 100.0;
        assert!(p.hurt_inner(10, from, true, &mut tr_core::UnifiedRandom::default(), true, false, None, true));
        assert!(p.vx != 0.0);
    }

    /// S1-P-i50 — the Gold/Lucky Coin family's equip grants
    /// (`ApplyEquipFunctional`, `Player.cs:14975-14996`): 855 Lucky Coin
    /// arms `hasLuckyCoin` only; 3033 Gold Ring arms `goldRing` only; 3034
    /// Coin Ring and 3035 Greedy Ring arm both. The +0.05 equipment-luck
    /// add is the UpdateEquips TAIL's single `hasLuck_LuckyCoin` gate
    /// (`:13066-13069`) — once, not per ring.
    #[test]
    fn s1_p_i50_the_coin_ring_family_arms_its_flags() {
        let p = player_with_accessory(855);
        assert!(p.has_lucky_coin, "Lucky Coin");
        assert!(!p.gold_ring, "no grab range from the Lucky Coin");
        let p = player_with_accessory(3033);
        assert!(p.gold_ring, "Gold Ring");
        assert!(!p.has_lucky_coin, "no proc from the Gold Ring");
        for kind in [3034u16, 3035] {
            let p = player_with_accessory(kind);
            assert!(p.gold_ring, "{kind} arms goldRing");
            assert!(p.has_lucky_coin, "{kind} arms the lucky coin");
        }
        // The luck add: +0.05 once for any lucky-coin carrier.
        let p = player_with_accessory(855);
        assert!((p.luck_state.equipment_based_luck_bonus - 0.05).abs() < 1e-6);
        // Two rings still add 0.05 ONCE (the tail gate is a bool, not a sum).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 3034;
        p.equipped[4] = 3035;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        assert!(
            (p.luck_state.equipment_based_luck_bonus - 0.05).abs() < 1e-6,
            "two rings, one +0.05: {}",
            p.luck_state.equipment_based_luck_bonus
        );
        // The per-tick reset clears both flags when the ring comes off
        // (`ResetEffects`, `:18504`/`:18760`).
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.gold_ring);
        assert!(!p.has_lucky_coin);
        assert_eq!(p.luck_state.equipment_based_luck_bonus, 0.0);
    }

    #[test]
    fn the_balloon_lifts_the_jump_statics() {
        // `UpdateJumpHeight` (`Player.cs:19589-19593`): jumpBoost floors
        // jumpHeight 20 / jumpSpeed 6.51 — the Cloud Balloon's grant
        // (`Player.cs:15093-15101`).
        let p = player_with_accessory(399);
        assert!(p.jump_boost);
        let w = ground_world();
        let mut q = Player::spawn(&w);
        q.update(&w, PlayerInput::default());
        assert!(q.jump_height < p.jump_height || q.jump_speed < p.jump_speed);
    }

    #[test]
    fn the_ice_skates_and_the_boots_feed_the_floor_modifiers() {
        // `Player.cs:15089-15092` (iceSkate) — the consumer at the port's
        // HorizontalMovement (player.rs's `if self.ice_skate`).
        let p = player_with_accessory(950);
        assert!(p.ice_skate);
        let p = player_with_accessory(54);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
    }

    #[test]
    fn the_rocket_boots_tier_applies() {
        // `Player.cs:15021-15024`: `rocketBoots = 1`.
        let p = player_with_accessory(128);
        assert_eq!(p.rocket_boots, 1);
    }

    #[test]
    fn the_band_feeds_the_regen_base() {
        // `Player.cs:13122` (`lifeRegen += armorPiece.lifeRegen`, the Band's
        // `Item.lifeRegen = 2` — `Item.cs:1974`): the regen input carries it.
        let p = player_with_accessory(49);
        assert_eq!(p.accessory_life_regen_value(), 2);
    }

    #[test]
    fn the_pins_survive_the_tick_clear() {
        // The TR_JUMPS/TR_BOOTS pins stand in for the granting equipment: the
        // pass clears the flags and re-applies the pins (the vanilla: the
        // grants re-set per tick after ResetEffects).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.pinned_jumps[ExtraJump::Cloud.index()] = true;
        p.pinned_rocket_boots = 3;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert_eq!(p.rocket_boots, 3);
        assert!(!p.has_jump_options[ExtraJump::Sandstorm.index()]);
    }

    /// DW-3.3's field half — the ten ammo-mastery flags a fresh `Player` carries
    /// (`player.rs`'s ammo block) plus the cycling struct: every one defaults
    /// false/None/0/0 like the C#'s `ResetEffects`-cleared fields
    /// (`Player.cs:18390-18664`, `:18479`), so the rolls stay inert until the
    /// per-tick grant seam in `world_sim/player.rs` derives them from the
    /// equipment and the buffs. The `ammo_flags()` projection is the single seam
    /// `ammo::not_consume` / `ammo::resolve_projectile` read.
    #[test]
    fn the_ammo_fields_default_to_false_and_none() {
        let w = crate::test_world_helpers::ground();
        let p = Player::spawn(&w);
        // The four fuse flags (Phase 1's fields).
        assert!(!p.has_molten_quiver && !p.magic_quiver);
        assert!(!p.acc_sharp_barb && !p.archery);
        // The six roll flags Phase 3 appends.
        assert!(!p.ammo_box && !p.ammo_potion && !p.huntress_ammo_cost90);
        assert!(!p.chloro_ammo_cost80 && !p.ammo_cost80 && !p.ammo_cost75);
        // The cycling state: None/0/0.
        assert_eq!(p.ammo_cycling.mode, crate::ammo::AmmoCyclingMode::None);
        assert_eq!((p.ammo_cycling.offset, p.ammo_cycling.cooldown), (0, 0));
        // The projection round-trips: an all-false Player gives an all-false
        // AmmoFlags, so the chain is the plain vanilla one.
        let flags = p.ammo_flags();
        assert_eq!(flags, crate::ammo::AmmoFlags::default());
        // A granted flag reaches the struct the rolls read.
        let mut granted = Player::spawn(&w);
        granted.ammo_box = true;
        granted.magic_quiver = true;
        assert!(granted.ammo_flags().ammo_box && granted.ammo_flags().magic_quiver);
    }

    /// Spawns on flat ground and settles (the first update is the spawn tick).
    fn settled() -> (World, Player) {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        (w, p)
    }

    #[test]
    fn the_leg_rows_cover_the_missing_playerframe_arms() {
        // `Player.cs:36219-36291`: the leg chain. Each arm is driven from the
        // state the C# reads, in the vanilla order.
        let (w, mut p) = settled();
        // The swim paddle (`:36221-36233`): a flat 2 per tick regardless of
        // velocity, cycling rows 7..=19.
        p.swim_time = 10;
        p.wet = true;
        p.vy = 0.0;
        p.leg_frame = 7;
        p.leg_frame_counter = 0.0;
        // `while (counter > 8.0)` is strict, so the 8.0 sum needs a fifth tick.
        for _ in 0..6 {
            p.update_frames(&w);
        }
        assert_eq!(p.leg_frame, 8, "2 per tick crosses 8.0 on the fifth tick");
        // Out of water the window dies (`:35759-35761`).
        p.wet = false;
        p.update_frames(&w);
        assert_eq!(p.swim_time, 0);
        // The hover set pins row 5 (`:36234-36240`): the grapple holds the
        // legs at the fall row even at rest.
        let (w, mut p) = settled();
        p.grappling_active = true;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 5, "the hooks hold the fall row");
        // The rest row (`:36276-36282`).
        p.grappling_active = false;
        p.vy = 0.0;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0);
        // The slippery no-input arm (`:36244-36250`): on ice with no steering
        // the legs rest even while sliding.
        p.vy = 0.0;
        p.vx = 6.0;
        p.slippy = true;
        p.control_left = false;
        p.control_right = false;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0, "the ice slide pins the rest row");
        // The walk cycle (`:36251-36275`) once the player steers.
        p.control_right = true;
        p.leg_frame = 7;
        p.leg_frame_counter = 0.0;
        for _ in 0..2 {
            p.update_frames(&w);
        }
        assert!(p.leg_frame > 7, "the counter advanced the row");
    }

    #[test]
    fn the_moon_lord_legs_overrun_the_body_rows() {
        // `Player.cs:36634-36644`: the 140 legs re-derive their row from the
        // body row's headgear offset after the body chain.
        let (w, mut p) = settled();
        p.legs_armor = 140;
        assert_eq!(offsets_player_headgear_y(7), 0.0);
        // Body row 7-9 has a zero headgear offset -> the leg row is pinned 7.
        p.vy = 0.0;
        p.vx = 4.0;
        p.leg_frame = 9;
        p.body_frame = 9;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 9, "the legs-140 walk keeps its own rows");
        assert_eq!(p.leg_frame, 7, "the zero offset pins the leg row");
        // Airborne the leg row follows `velocity.Y != 0` -> 5.
        p.vy = 3.0;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 5);
        // The always-animated wings (22/28) still force row 0.
        p.wings_logic = 22;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0);
    }

    #[test]
    fn the_body_chain_order_matches_playerframe() {
        // `Player.cs:36445-36633`: the body rows after the item-use arm, in the
        // vanilla `else if` order.
        let (w, mut p) = settled();
        p.vx = 0.0;
        // The rope pose (`:36445-36454`).
        p.pulley = true;
        p.pulley_dir = 2;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 1);
        p.pulley_dir = 1;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 2);
        p.pulley = false;
        // holdStyle 1 (the torch) -> row 3 (`:36455-36459`); it beats the walk
        // row because the arm comes first.
        p.frame_inputs = FrameInputs {
            hold_style: 1,
            can_visually_hold: true,
            hold_style_dry: true,
            torch_hold_ok: true,
            use_style_blocks_frames: true,
            ..FrameInputs::default()
        };
        p.vx = 3.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 3, "the torch hold wins over the walk row");
        // holdStyle 2 -> row 2 (`:36460-36464`).
        p.frame_inputs.hold_style = 2;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 2);
        // holdStyle 7 -> row 11 (`:36477-36481`).
        p.frame_inputs.hold_style = 7;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 11);
        // The umbrella (4) only rests on the ground.
        p.frame_inputs.hold_style = 4;
        p.vy = 0.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 0);
        // The raised shield -> row 10 (`:36489-36492`).
        p.frame_inputs = FrameInputs::default();
        p.shield_raised = true;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 10);
        p.shield_raised = false;
        // With nothing held the walk row copies the legs (`:36619-36622`).
        p.vy = 0.0;
        p.leg_frame = 12;
        p.vx = 3.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, p.leg_frame);
        assert!(p.leg_frame >= 12, "the walk advanced the leg row");
        assert!(p.body_frame_counter > 0.0, "the 1.5x counter accumulated");
        // At rest the body rests (`:36624-36633`).
        p.vx = 0.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 0);
    }

    #[test]
    fn the_airborne_body_rows_follow_the_wings() {
        // `Player.cs:36559-36600`: the airborne rows per wing state.
        let (_, mut p) = settled();
        p.vy = 2.0;
        p.wings_logic = 0;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 5, "no wings: the plain fall row");
        p.wings_logic = 4;
        p.control_jump = true;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 6, "rising under thrust: row 6");
        p.vy = -2.0;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 6, "rising: row 6");
        p.wings_logic = 22;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 0, "the always-animated set rests in the air");
    }

    #[test]
    fn the_use_style_body_dispatch_matches_the_playerframe_chain() {
        // `Player.cs:36307-36444`: the body rows per useStyle. `remaining` is
        // `itemAnimation / itemAnimationMax` (1 at the start, 0 at the end).
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        // Style 1 (Swing): the thirds 1 -> 2 -> 3.
        p.apply_item_use_frame(0.9, 1, 0.0);
        assert_eq!(p.body_frame, 1, "style 1 first third");
        p.apply_item_use_frame(0.5, 1, 0.0);
        assert_eq!(p.body_frame, 2, "style 1 middle third");
        p.apply_item_use_frame(0.1, 1, 0.0);
        assert_eq!(p.body_frame, 3, "style 1 last third");
        // Style 7 (DrinkOld): the first half row 3, then 2.
        p.apply_item_use_frame(0.75, 7, 0.0);
        assert_eq!(p.body_frame, 3, "style 7 first half");
        p.apply_item_use_frame(0.25, 7, 0.0);
        assert_eq!(p.body_frame, 2, "style 7 second half");
        // Style 2 (EatFood): row 3 through the use.
        p.apply_item_use_frame(0.5, 2, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 11 (MowTheLawn): >half 4, >15% 3, else 2.
        p.apply_item_use_frame(0.75, 11, 0.0);
        assert_eq!(p.body_frame, 4, "style 11 first half");
        p.apply_item_use_frame(0.3, 11, 0.0);
        assert_eq!(p.body_frame, 3, "style 11 middle");
        p.apply_item_use_frame(0.05, 11, 0.0);
        assert_eq!(p.body_frame, 2, "style 11 tail");
        // Style 9 (DrinkLiquid): row 0.
        p.apply_item_use_frame(0.5, 9, 0.0);
        assert_eq!(p.body_frame, 0);
        // Style 6 (DrinkLong): `num20 = progress * 6` clamped at 1 - the
        // row flips to 3 once num20 passes 0.5 (progress > 1/12).
        p.apply_item_use_frame(0.99, 6, 0.0);
        assert_eq!(p.body_frame, 2, "style 6 start - num20 = 0.06");
        p.apply_item_use_frame(0.9, 6, 0.0);
        assert_eq!(p.body_frame, 3, "style 6 mid - num20 = 0.6");
        p.apply_item_use_frame(0.05, 6, 0.0);
        assert_eq!(p.body_frame, 3, "style 6 late - num20 clamps at 1");
        // Style 3 (Thrust): BOTH branches row 3 - the vanilla quirk.
        p.apply_item_use_frame(0.9, 3, 0.0);
        assert_eq!(p.body_frame, 3, "style 3 first third");
        p.apply_item_use_frame(0.1, 3, 0.0);
        assert_eq!(p.body_frame, 3, "style 3 last third");
        // Style 4 (HoldUp): row 2 through the use.
        p.apply_item_use_frame(0.5, 4, 0.0);
        assert_eq!(p.body_frame, 2);
        // Style 8 (GolfPlay): row 0. Style 12 (Guitar): row 3.
        p.apply_item_use_frame(0.5, 8, 0.0);
        assert_eq!(p.body_frame, 0);
        p.apply_item_use_frame(0.5, 12, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 13 (Rapier): the thirds mirror style 1.
        p.apply_item_use_frame(0.9, 13, 0.0);
        assert_eq!(p.body_frame, 1);
        p.apply_item_use_frame(0.1, 13, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 5 (Shoot): the aim-relative rows - the default 3, past
        // -0.75 row 2, past 0.6 row 4.
        p.apply_item_use_frame(0.5, 5, 0.0);
        assert_eq!(p.body_frame, 3, "style 5 aim centred");
        p.apply_item_use_frame(0.5, 5, -0.9);
        assert_eq!(p.body_frame, 2, "style 5 aim high");
        p.apply_item_use_frame(0.5, 5, 0.9);
        assert_eq!(p.body_frame, 4, "style 5 aim low");
    }

    #[test]
    fn the_use_style_carries_from_the_held_item() {
        // `item::use_style` reads the ItemDef's `useStyle`.
        assert_eq!(crate::item::use_style(36), 1, "the wooden sword swings");
        assert_eq!(crate::item::use_style(31), 1, "the bottle places");
        assert_eq!(crate::item::use_style(0), 0, "the empty hand");
    }

    /// Arms the Cloud + Sandstorm options on a settled player — through the
    /// pins (the grant pass clears the direct sets every tick; the pins are
    /// the stand-in for the granting equipment).
    pub(super) fn settled_with_jumps(w: &World, opts: &[ExtraJump]) -> Player {
        let mut p = Player::spawn(w);
        for j in opts {
            p.pinned_jumps[j.index()] = true;
        }
        for _ in 0..120 {
            p.update(w, PlayerInput::default());
        }
        for j in opts {
            p.can_jump_again[j.index()] = true;
        }
        p
    }

    /// Fires a jump edge (press + release + gap) while airborne.
    fn tap_air(w: &World, p: &mut Player) {
        p.update(
            w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        for _ in 0..3 {
            p.update(w, PlayerInput::default());
        }
        p.update(
            w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
    }

    /// `Player.cs:20698-20748`: the priority chain — with Sandstorm + Cloud
    /// armed, the Sandstorm consumes first (hold-boost 3x).
    #[test]
    fn sandstorm_beats_cloud_in_the_chain() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Sandstorm, ExtraJump::Cloud]);
        // Jump off the ground; the ground jump re-arms everything.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.can_jump_again[ExtraJump::Sandstorm.index()], "re-armed");
        tap_air(&w, &mut p);
        assert!(
            !p.can_jump_again[ExtraJump::Sandstorm.index()],
            "sandstorm consumed first"
        );
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "cloud still armed"
        );
        assert_eq!(p.performing_jump, Some(ExtraJump::Sandstorm));
        // The hold-boost counter: jumpHeight (15) x 3 = 45.
        assert_eq!(p.jump, 45, "sandstorm 3x boost");
        // The next tap: the Cloud fall-through (0.75x = 11).
        tap_air(&w, &mut p);
        assert_eq!(p.performing_jump, Some(ExtraJump::Cloud));
        assert_eq!(p.jump, (15.0 * 0.75) as i32, "cloud 0.75x boost");
    }

    /// `Player.cs:20755-20758`: the ground jump re-arms every granted option.
    #[test]
    fn ground_jump_rearms_the_options() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // Consume the cloud.
        tap_air(&w, &mut p);
        assert!(!p.can_jump_again[ExtraJump::Cloud.index()]);
        // Land: the re-arm.
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        p.update(&w, PlayerInput::default());
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "re-armed on land"
        );
    }

    /// `Player.cs:26061-26099`: while airborne, an option that vanished
    /// force-clears its armed flag.
    #[test]
    fn stale_options_clear_while_airborne() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // Airborne.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        p.update(&w, PlayerInput::default());
        assert!(p.vy != 0.0 || !p.on_ground);
        // The accessory vanishes mid-air (`UpdateEquips`' next pass sees no
        // granting equipment — the pin is the stand-in and it clears).
        p.pinned_jumps[ExtraJump::Cloud.index()] = false;
        p.update(&w, PlayerInput::default());
        assert!(
            !p.can_jump_again[ExtraJump::Cloud.index()],
            "the stale option clears"
        );
    }

    /// `Player.cs:21023-21031`: the second airborne jump requires the button
    /// released since the previous one — holding through does not re-fire.
    #[test]
    fn the_second_jump_needs_the_release() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // The ground jump.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.can_jump_again[ExtraJump::Cloud.index()]);
        // Hold through: no consumption (releaseJump stays false).
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        for _ in 0..20 {
            p.update(&w, held);
        }
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "holding does not consume"
        );
        assert!(p.jump > 0 || p.vy < 0.0, "still in the first jump");
    }

    /// `Player.cs:19589-19593`: the balloon floor lifts the jump statics —
    /// the ground jump height 20 instead of 15.
    #[test]
    fn balloon_lifts_the_jump_height() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.jump_boost = true;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert_eq!(p.jump, 20, "jumpHeight floor 20 with jumpBoost");
    }

    /// `Player.cs:19963,19993`: the overspeed continuation is gated on
    /// `dashDelay >= 0` — no input acceleration while dashing (the decay
    /// runs instead).
    #[test]
    fn dash_overspeed_continuation_gated() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.pinned_dash = 2;
        p.dash = 2;
        p.dash_delay = -1;
        p.vx = 14.5;
        let held = PlayerInput {
            right: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert!(p.vx > 14.0, "decay, not acceleration: vx={}", p.vx);
    }

    /// `Player.cs:23755-23783`: `GetFloorTile` samples the center column
    /// first, then the neighbors — the flags come from the tile under the
    /// feet (one tick stale, so a settled player has live flags).
    #[test]
    fn floor_flags_sample_the_feet_row() {
        let w = floor_world(161);
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "must settle first");
        assert!(p.slippy, "ice floor flags the slippery set");
        let mut q = Player::spawn(&ground_world());
        for _ in 0..120 {
            q.update(&ground_world(), PlayerInput::default());
        }
        q.update(&ground_world(), PlayerInput::default());
        assert!(!q.slippy, "dirt is not slippery");
    }

    /// `Player.cs:25363`: `(fell - (25 + extraFall)) * 10`, `fell` truncated on
    /// both ends. Dropping from tile 25 lands at row 57 → fell = 32 → 70.
    #[test]
    fn fall_damage_matches_the_vanilla_formula() {
        let w = ground_world();
        let (lost, landed) = drop_and_settle(&w, 25);
        assert_eq!(landed, 57, "feet rest on row 57");
        assert_eq!(lost, (landed - 25 - 25) * 10, "fell = 57 - 25 = 32 tiles");
    }

    /// `Player.cs:25360`: `num11 > num10` is strict — a 25-tile fall hurts for
    /// nothing; the first damaging tile is 26.
    #[test]
    fn the_safe_fall_threshold_is_strict() {
        let w = ground_world();
        // From tile 32 the feet land at row 57 → fell 25 → no damage.
        let (lost, _) = drop_and_settle(&w, 32);
        assert_eq!(lost, 0, "25-tile fall is free");
    }

    /// `Player.cs:25361`: the normal branch forces `immune = false` BEFORE the
    /// Hurt — a fall lands through any iframes the contact damage granted.
    #[test]
    fn fall_damage_ignores_iframes() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.hurt_cd = 5.0; // mid-iframes
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.hp < 100, "fall must land through iframes");
    }

    /// The liquid reset (`Player.cs:25379`): while `wet` the fall origin rides
    /// the player, so a fall through water lands for nothing.
    #[test]
    fn water_forgives_the_fall() {
        let mut w = ground_world();
        for y in 50..58 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let (lost, _) = drop_and_settle(&w, 20);
        assert_eq!(lost, 0, "a fall through water is free");
    }

    /// `Player.cs:25338-25345`: a landing on the cloud set (tile 189 here)
    /// zeroes the fall distance.
    #[test]
    fn clouds_forgive_the_fall() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = 189;
        }
        let (lost, _) = drop_and_settle(&w, 25);
        assert_eq!(lost, 0, "a landing on clouds is free");
    }

    /// `Player.cs:25347-25359`: petrified replaces the whole formula — a fixed
    /// 2-tile grace at 20 damage per tile, ignoring extraFall.
    #[test]
    fn stoned_landing_uses_the_petrified_formula() {
        let mut w = ground_world();
        // Clouds out of the picture: the plain dirt surface again.
        let mut p = Player::spawn(&w);
        p.max_hp = 1000;
        p.hp = 1000;
        p.buffs.add_buff(156, 600); // Petrified
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        let landed = (p.y / TILE_PX) as i32;
        // (fell - 2) * 20 = (32 - 2) * 20 = 600, minus the flat petrify hit of
        // 20 taken the moment the buff appeared.
        assert_eq!(p.hp, 1000 - (landed - 25 - 2) * 20 - 20, "petrified hp");
    }

    /// `Player.cs:25370`: fall damage calls Hurt with hitDirection 0 — no
    /// knockback, the player stays planted where they landed.
    #[test]
    fn fall_landing_has_no_knockback() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.on_ground, "must rest after landing");
        assert_eq!(p.vy, 0.0, "no upward knock");
    }

    #[test]
    fn idle_uses_the_rest_pose_row_zero() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // `PlayerFrame`: standing still is leg/body frame row 0, never the jump
        // pose (row 5) that made the character look like it was about to leap.
        assert_eq!(p.leg_frame, 0, "idle legs must be row 0");
        assert_eq!(p.body_frame, 0, "idle body must be row 0");
    }

    #[test]
    fn airborne_uses_row_five() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..8 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.vy != 0.0, "should be airborne, vy={}", p.vy);
        assert_eq!(p.leg_frame, 5, "airborne legs must be row 5");
        assert_eq!(p.body_frame, 5, "airborne body must be row 5");
    }

    #[test]
    fn walking_cycles_the_walk_rows() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            assert!(
                (7..=19).contains(&p.leg_frame),
                "walk row {} outside 7..=19",
                p.leg_frame
            );
            assert_eq!(p.body_frame, p.leg_frame, "body must copy the leg row");
            seen.insert(p.leg_frame);
        }
        assert!(seen.len() >= 4, "walk cycle did not advance: {seen:?}");
    }

    #[test]
    fn walks_and_is_blocked_by_walls() {
        let mut w = ground_world();
        for y in 50..60 {
            let i = w.index(20, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
        }
        assert!(p.x < 20.0 * TILE_PX, "player passed the wall: x={}", p.x);
    }

    #[test]
    fn jump_leaves_the_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let grounded_y = p.y;
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..10 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.y < grounded_y - 4.0, "jump did not rise: {}", p.y);
    }

    #[test]
    fn walks_down_a_slope_smoothly() {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            let top = if x < 20 { 60 } else { 61 };
            for y in top..100 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(19, 60);
        w.tiles[i].set_slope(1);
        w.spawn_x = 10;
        w.spawn_y = 58;
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        for _ in 0..180 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
        }
        assert!(p.y > 56.0 * TILE_PX, "should have descended, y={}", p.y);
        assert!(max_step < 8.0, "slope caused a {max_step} px visual jump");
    }

    #[test]
    fn walks_up_a_one_tile_step_smoothly() {
        let mut w = ground_world();
        // A one-tile step at x=20: `Collision.StepUp` lifts the hitbox while
        // `gfxOffY` keeps the sprite at the old height, so the climb is smooth.
        for x in 20..30 {
            let i = w.index(x, 59);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        let mut stood_on_step = false;
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
            if p.x + WIDTH > 20.0 * TILE_PX && p.x < 30.0 * TILE_PX {
                stood_on_step |= (p.y + HEIGHT - 59.0 * TILE_PX).abs() < 1.0;
            }
        }
        assert!(
            p.x + WIDTH > 21.0 * TILE_PX,
            "player never climbed the step: x={}",
            p.x
        );
        assert!(stood_on_step, "feet never rested on the step top");
        // The original eases `gfxOffY` at `(1 + |vx| / 3) * stepSpeed` px per tick
        // (5 px/tick at walk speed with stepSpeed 2.5); what must never happen is
        // the instant 16 px teleport of a naive snap.
        assert!(max_step < 8.0, "step caused a {max_step} px visual jump");
    }

    #[test]
    fn holding_jump_rises_higher_than_tapping() {
        let w = ground_world();
        let mut tap = Player::spawn(&w);
        for _ in 0..120 {
            tap.update(&w, PlayerInput::default());
        }
        let mut tap_peak = tap.y;
        tap.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..60 {
            tap.update(&w, PlayerInput::default());
            tap_peak = tap_peak.min(tap.y);
        }

        let mut hold = Player::spawn(&w);
        for _ in 0..120 {
            hold.update(&w, PlayerInput::default());
        }
        let mut hold_peak = hold.y;
        for _ in 0..30 {
            hold.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
            hold_peak = hold_peak.min(hold.y);
        }
        for _ in 0..60 {
            hold.update(&w, PlayerInput::default());
            hold_peak = hold_peak.min(hold.y);
        }
        assert!(
            hold_peak + 20.0 < tap_peak,
            "hold peak {hold_peak} vs tap peak {tap_peak}"
        );
    }

    #[test]
    fn long_falls_hurt_but_short_ones_do_not() {
        let mut w = ground_world();
        w.spawn_y = 30;
        let mut p = Player::spawn(&w);
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
        }
        // The hitbox top lands at 60 - 42/16 = 57.4 tiles, so the drop is ~27:
        // 2.4 tiles over the safe distance.
        assert!(p.on_ground);
        assert!(p.hp < 90 && p.hp > 0, "hp {}", p.hp);

        let mut w = ground_world();
        w.spawn_y = 50;
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        assert_eq!(p.hp, 100, "a 10-tile fall must not hurt");
    }

    #[test]
    fn drowning_drains_breath_and_damages() {
        let mut w = ground_world();
        for x in 0..64 {
            // A pool from y=40 down to the ground, so the player settles on the floor.
            for y in 40..60 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        w.spawn_y = 44;
        let mut p = Player::spawn(&w);
        for _ in 0..1500 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.in_water);
        assert_eq!(p.breath, 0, "breath {}", p.breath);
        assert!(p.hp < 100, "drowning should hurt, hp {}", p.hp);
    }

    /// `Player.CheckDrowning` (`Player.cs:23191-23281`): the breath follows
    /// `Collision.DrownCollision` — the **head** probe, not "any water".
    #[test]
    fn the_breath_drains_only_while_the_head_is_under() {
        let pool = |rows: std::ops::Range<i32>| {
            let mut w = ground_world();
            for x in 0..64 {
                for y in rows.clone() {
                    let i = w.index(x, y);
                    w.tiles[i].liquid = 255;
                }
            }
            w
        };
        // The player stands on the ground (row 60): the feet at 960, the head
        // probe at [931, 943].
        // A shallow pool (row 59 only) — the water's surface is at 944, so
        // the head stays dry and the breath is untouched.
        let mut w = pool(59..60);
        let mut p = Player::spawn(&w);
        p.x = 10.0 * 16.0 + 8.0;
        p.y = 60.0 * 16.0 - 42.0;
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        assert_eq!(p.breath, BREATH_MAX, "the head stays dry");
        // A deep pool — the head goes under and the breath runs down.
        let mut w = pool(50..60);
        let mut p = Player::spawn(&w);
        p.x = 10.0 * 16.0 + 8.0;
        p.y = 60.0 * 16.0 - 42.0;
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        assert!(
            p.breath < BREATH_MAX,
            "the head is under the water, breath {}",
            p.breath
        );
    }

    #[test]
    fn health_regenerates_after_a_delay() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.hp = 50;
        p.regen.life_regen_time = 0.0;
        for _ in 0..1200 {
            p.update(&w, PlayerInput::default());
        }
        // The faithful engine at `statLifeMax2 = 100` is slow: the ramp scale is
        // `100/400·0.85 + 0.15 = 0.3625`, so tiers 1-2 round banker's-style to
        // 0 and the first points arrive from tick ~600 on (≈ +4 hp over 20 s
        // standing). The old placeholder granted 1 hp per half-second — the
        // `> 60` bar belonged to it.
        assert!(p.hp > 50, "regen only reached {} (sit still, 20 s)", p.hp);
        assert!(p.hp <= 56, "regen overshot the faithful pace: {}", p.hp);
    }

    #[test]
    fn stands_on_half_bricks_at_their_own_height() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
                ..Default::default()
            };
        }
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        let expected = 60.0 * TILE_PX + 8.0 - HEIGHT;
        assert!(
            (p.y - expected).abs() < 1.0,
            "y {} expected {expected}",
            p.y
        );
    }

    #[test]
    fn half_bricks_only_block_their_bottom_half() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
            ..Default::default()
        };
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0 - HEIGHT
        ));
    }

    #[test]
    fn ceiling_slopes_block_by_their_triangle() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        w.tiles[i].set_slope(3);
        // Slope 3 is solid above the diagonal from (0, 16) to (16, 0).
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            50.0 * TILE_PX
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            51.0 * TILE_PX
        ));
    }

    #[test]
    fn standing_on_a_ground_slope() {
        let mut w = World::new("t".into(), 32, 64);
        for x in 0..32 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(10, 29);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].set_slope(1);
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.spawn_x = 8;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();

        let mut p = Player::spawn(&w);
        // Left edge six pixels into the ramp: `SlopeCollision` rests slope 1 on
        // the diagonal at the hitbox's left edge.
        p.x = 10.0 * TILE_PX + 6.0;
        p.y = 29.0 * TILE_PX - HEIGHT;
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
        }
        let feet = p.y + HEIGHT;
        let expected = 29.0 * TILE_PX + (p.x - 10.0 * TILE_PX);
        assert!(
            (feet - expected).abs() <= 1.5,
            "feet {feet} expected {expected}"
        );
        assert!(p.on_ground);
    }

    // — Phase 2 (M2-BUFF-3): DoT derivation, water/lava/honey treatment —

    /// A player with a large life pool for 300-tick drain runs; `update_survival`
    /// only (the buff timers are not decremented there, so the buff holds).
    fn drain_rig() -> (Player, World) {
        let w = World::new("t".into(), 64, 800);
        let p = drain_rig_in(&w);
        (p, w)
    }

    /// The world-parameterized twin (M5-SEED-2's vampirism rigs pass a world
    /// whose micro seeds carry `Main.vampireSeed`'s bit).
    fn drain_rig_in(world: &World) -> Player {
        let mut p = Player::spawn(world);
        p.max_hp = 1000;
        p.hp = 1000;
        p
    }

    // — M2-BUFF-6 (phase 1): the luck value + the buff-driven inputs —

    #[test]
    fn buff_257_tiers_follow_the_remaining_time() {
        // DW-1.3 — `Player.cs:10285-10301`: `>36000 → 3, >18000 → 2, else 1`,
        // read after `UpdateBuffs`' timer head decremented the slot.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 36_002);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 3);

        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 36_001);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 2, "36000 → not > 36000");

        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 18_001);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 1, "18000 → not > 18000");
    }

    #[test]
    fn buff_383_tier_is_stale_after_expiry_never_zeroed() {
        // DW-1.3 — `Player.cs:9920-9924`: tier = buffTime/10800 clamp 0..3,
        // assigned only while the buff is active; no `kiteLuckLevel = 0`
        // exists in the C#, so the field keeps its last value after expiry.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(383, 32_401);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.kite_luck_level, 3, "32400/10800");

        p.buffs.clear_buff(383);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.kite_luck_level, 3, "stale-until-recomputed");
    }

    #[test]
    fn buff_120_sets_stinky_and_the_reset_clears_it() {
        // DW-1.3 — `Player.cs:10276-10280` sets it, `ResetEffects` `:18732`
        // clears it every frame; RecalculateLuck weights it −0.25.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(120, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.luck_state.stinky);
        assert!((p.luck() + 0.25).abs() < 1e-6);

        p.buffs.clear_buff(120);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(!p.luck_state.stinky);
        assert_eq!(p.luck(), 0.0);
    }

    #[test]
    fn one_tick_order_decay_then_recalculate() {
        // DW-1.4 — `UpdateLuckFactors` (`Player.cs:29121`) drains the ladybug
        // timer before `RecalculateLuck` reads it, in the same tick the buff
        // scan derived the potion tier; `p.luck()` is the consumer seam.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 20_002); // 20_001 after the timer head → tier 2
        p.luck_state.lady_bug_luck_time_left = -10_800;
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 2);
        assert_eq!(p.luck_state.lady_bug_luck_time_left, -10_799, "dayRate 1");
        let want = (10_799.0f32 / -10_800.0) * 0.2 + 2.0f32 * 0.1;
        assert!((p.luck() - want).abs() < 1e-6, "{} vs {want}", p.luck());
    }

    #[test]
    fn add_coin_luck_caps_at_a_million_within_a_thousand_px() {
        // DW-1.1 — `Player.AddCoinLuck` (`Player.cs:18090-18101`): `!dead`,
        // within 1000 px of the center, capped at 1 000 000 copper
        // equivalents (the shimmer feed is the only C# writer; unported).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.add_coin_luck((p.x + 500.0, p.y), 999_000);
        assert_eq!(p.luck_state.coin_luck, 999_000.0);
        p.add_coin_luck((p.x + 500.0, p.y), 1_000_000);
        assert_eq!(p.luck_state.coin_luck, 1_000_000.0, "capped");

        p.luck_state.coin_luck = 100.0;
        p.add_coin_luck((p.x + 1200.0, p.y), 5);
        assert_eq!(p.luck_state.coin_luck, 100.0, "out of range");

        p.dead = true;
        p.add_coin_luck((p.x, p.y), 5);
        assert_eq!(p.luck_state.coin_luck, 100.0, "dead bails");
    }

    #[test]
    fn dot_table_rows_drain_exact_life_regen() {
        // DW-2.1 — the §2.1 table rows as exact `lifeRegen` values, branch order
        // and constants per `Player.cs:18949-19095`; bleeding resets the ramp
        // only (`:19178-19181`). `loss` is the exact hp drop over 300 ticks
        // where the chunk math is closed-form (None = assert "drained").
        let rows: &[(i32, bool, i32, Option<i32>)] = &[
            (20, false, -4, Some(10)),   // Poisoned: −4×300 = −1200 → 10×1 hp
            (70, false, -30, Some(75)),  // Venom: −120 every 4 ticks → 75×1
            (397, false, -30, None),     // Chlorophyte Spore
            (24, false, -8, Some(20)),   // OnFire: −120 every 15 ticks → 20×1
            (323, false, -8, Some(20)),  // OnFire3
            (44, false, -16, None),      // Frostburn
            (324, false, -16, None),     // Frostburn2
            (39, false, -24, Some(60)),  // Cursed Inferno: −120 every 5 ticks
            (67, false, -60, Some(150)), // Burning: fast lane −600/10 ticks → 30×5
            (68, false, -40, Some(100)), // Suffocation: fast lane −600/15 → 20×5
            (144, false, -8, Some(20)),  // Electrified standing
            (38, true, -100, Some(250)), // Tongued expert: fast lane −600/6 → 50×5
            (30, false, 0, Some(0)),     // Bleeding: ramp reset only, no drain
        ];
        for &(id, expert, regen, loss) in rows {
            let (mut p, dw) = drain_rig();
            p.buffs.expert_mode = expert;
            p.buffs.add_buff(id, 600);
            for _ in 0..300 {
                p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
            }
            assert_eq!(
                p.regen.life_regen, regen,
                "buff {id} must drain exactly {regen}/tick"
            );
            match loss {
                Some(lost) => assert_eq!(1000 - p.hp, lost, "buff {id} total loss"),
                None => assert!(p.hp < 1000, "buff {id} must drain"),
            }
            assert!(p.hp > 0, "buff {id} rig must survive 300 ticks");
        }
    }

    #[test]
    fn water_extinguishes_only_onfire_and_onfire3() {
        // DW-2.2 — `Player.cs:27750-27761`: only 24 and 323 are deleted; the
        // rest of the fire/poison family rides through the water.
        for id in [24, 323] {
            let (mut p, dw) = drain_rig();
            p.buffs.add_buff(id, 600);
            p.in_water = true;
            p.wet = true;
            p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
            assert!(
                p.buffs.find_buff_index(id).is_none(),
                "water must cure buff {id}"
            );
        }
        for id in [44, 324, 39, 67, 20, 68, 144, 30] {
            let (mut p, dw) = drain_rig();
            p.buffs.add_buff(id, 600);
            p.in_water = true;
            p.wet = true;
            p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
            assert!(
                p.buffs.find_buff_index(id).is_some(),
                "water must not cure buff {id}"
            );
        }
    }

    #[test]
    fn lava_wet_blocks_the_water_extinguish() {
        // DW-2.2 — `!base.lavaWet` guard (`Player.cs:27751`): standing in lava
        // the water cannot cure OnFire. (The lava hit also re-hurt: hp 70.)
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600);
        p.in_water = true;
        p.wet = true;
        p.in_lava = true;
        p.lava_wet = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.find_buff_index(24).is_some(),
            "lavaWet blocks the cure"
        );
        assert_eq!(p.buffs.buff_time[p.buffs.find_buff_index(24).unwrap()], 600);
    }

    #[test]
    fn honey_does_not_extinguish_fire() {
        // DW-2.2 (Assumption-2 fallback) — the port conditions the cure on the
        // water-only `wet` flag; the C# `WetCollision` flag would cover honey
        // too (Collision.cs:1608-1618) — deviation recorded for
        // docs/ANALYSIS.md (Phase 4).
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600);
        p.in_water = true; // the physics probe covers water | honey
        p.honey_wet = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.find_buff_index(24).is_some(),
            "honey must not cure OnFire"
        );
    }

    #[test]
    fn water_cures_both_fire_buffs_over_two_ticks_despite_the_no_rescan_quirk() {
        // DW-2.2 quirk — the C# extinguish loop does not rescan after `DelBuff`
        // compaction (`Player.cs:27753-27761`): with 24 and 323 both held, one
        // tick deletes the first and skips the shifted-in second.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(323, 600);
        p.in_water = true;
        p.wet = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        let left = p.buffs.active_types();
        assert_eq!(left.len(), 1, "one fire buff survives the shift-skip");
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.active_types().is_empty(),
            "the second falls next tick"
        );
    }

    #[test]
    fn lava_contact_grants_onfire_420() {
        // DW-2.3 — `Player.cs:27710-27711`: a landed lava Hurt grants
        // AddBuff(24, 420); while the iframes run (`num81 == 0`) nothing lands.
        let (mut p, dw) = drain_rig();
        p.in_lava = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        let idx = p.buffs.find_buff_index(24).expect("OnFire granted on lava");
        assert_eq!(p.buffs.buff_time[idx], 420);
        // `:27705` — the classic lava hit is 80 (`num79`), the 420-tick On
        // Fire rides it.
        assert_eq!(p.hp, 1000 - 80);
        // Inside the iframes the hit (and therefore the grant) repeats nothing.
        let hp = p.hp;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, hp, "no second lava hit inside the iframes");
        // Expert doubles the buff time through the AddBuff chain
        // (`AddBuff_DetermineBuffTimeToAdd`, 24 is in the difficulty set).
        let (mut p, dw) = drain_rig();
        p.buffs.expert_mode = true;
        p.in_lava = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        let idx = p.buffs.find_buff_index(24).unwrap();
        assert_eq!(p.buffs.buff_time[idx], 840);
    }

    #[test]
    fn wet_immunity_removes_held_fire_within_one_tick() {
        // DW-2.4 — Wet (103) re-grants `buffImmune[24/323/67]` for the frame
        // (`Player.cs:11740-11751`) and the end-of-update purge deletes the
        // held fire buff (`:26011-26016`): one full update tick clears it.
        // The C# still drains one frame from the flags (the chain runs before
        // the purge): −8 OnFire on the counter.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(103, 600);
        p.update(&w, PlayerInput::default());
        assert!(
            p.buffs.find_buff_index(24).is_none(),
            "OnFire purged in 1 tick"
        );
        assert!(
            p.buffs.find_buff_index(103).is_some(),
            "Wet itself survives"
        );
        assert_eq!(p.regen.life_regen_count, -8, "one verbatim drain frame");
    }

    #[test]
    fn immune_purge_skips_the_compacted_fire_buff_for_a_frame() {
        // The purge's no-rescan quirk (`Player.cs:26012-26018`, pinned in
        // buffs.rs): deleting slot 0 shifts the second fire buff into it and
        // the loop never rescans — Burning falls one tick later.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(67, 600);
        p.buffs.add_buff(103, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(24).is_none());
        assert!(
            p.buffs.active_types().contains(&67),
            "the compacted-in Burning is skipped this frame (still held; \
             find_buff_index hides it behind the live immunity)"
        );
        p.update(&w, PlayerInput::default());
        assert!(
            !p.buffs.active_types().contains(&67),
            "Burning purged the next frame"
        );
        assert!(p.buffs.find_buff_index(103).is_some());
    }

    #[test]
    fn wet_immunity_vanishes_after_wet_expires() {
        // DW-2.4 — the per-frame reset (`Player.cs:25562-25565`) means the
        // immunities live exactly as long as the Wet buff does.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(103, 2);
        for _ in 0..3 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.buffs.find_buff_index(103).is_none(), "Wet expired");
        assert!(!p.buffs.buff_immune[24], "OnFire immunity gone");
        assert!(!p.buffs.buff_immune[323], "OnFire3 immunity gone");
        assert!(!p.buffs.buff_immune[67], "Burning immunity gone");
        // A fresh OnFire now sticks — the immunity is really gone.
        p.buffs.add_buff(24, 100);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(24).is_some());
    }

    #[test]
    fn wet_self_removes_on_lava_contact() {
        // `Player.cs:11741-11744`: the Wet branch deletes itself while
        // `lavaWet`, granting no immunities that frame.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(103, 600);
        p.in_lava = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert!(p.buffs.find_buff_index(103).is_none());
        assert!(!p.buffs.buff_immune[24], "no fire immunity without Wet");
    }

    #[test]
    fn honey_softens_a_drain_by_four_and_never_heals() {
        // DW-2.5 — `Player.cs:19096-19103`: honey +4 to a negative regen,
        // clamped at 0 — never positive, no heal from the mitigation itself.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(20, 600); // Poisoned −4
        p.buffs.add_buff(48, 600); // Honey
        for _ in 0..300 {
            p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        }
        assert_eq!(p.regen.life_regen, 0, "−4 + 4 clamps at 0");
        assert_eq!(p.hp, 1000, "a softened −4 drains nothing");
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(70, 600); // Venom −30
        p.buffs.add_buff(48, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -26, "−30 + 4 stays negative");
        // Honey without a drain never adds anything (the +2 aura is M2-BUFF-5).
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(48, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, 0);
        assert_eq!(p.hp, 1000);
    }

    #[test]
    fn fire_family_deltas_sum_with_dripping_slime() {
        // Dirty — fire debuffs coexist and sum (`Player.cs` has no replace
        // chain); drippingSlime (137) doubles the fire family only.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600); // OnFire −8
        p.buffs.add_buff(324, 600); // Frostburn2 −16
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -24);
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(324, 600);
        p.buffs.add_buff(137, 600); // drippingSlime
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -48, "drip doubles both fire parts");
        // A poison part is not doubled by the drip.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(20, 600); // Poisoned −4
        p.buffs.add_buff(24, 600); // OnFire −8
        p.buffs.add_buff(137, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -20, "−4 poison + (−8×2) fire");
    }

    // M5-SEED-2: the `vampirism` world's OnFire row (`Player.cs:18983-18992`)
    // drains −100 instead of −8, with the drippingSlime double at −100.
    #[test]
    fn the_vampirism_world_trades_the_on_fire_drain_for_its_own() {
        // The classic row first — the flag is off on a classic world.
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(24, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -8);
        // The seeded world: the flag rides the world's micro seeds, captured
        // at `Player::spawn` (`WorldGen.cs:555-558`).
        let mut w = World::new("t".into(), 64, 800);
        w.micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
        let mut p = drain_rig_in(&w);
        assert!(p.vampirism, "the flag reaches the player");
        p.buffs.add_buff(24, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -100, "the vampirism OnFire drain");
        // The drip double is 100 too, and only the fire family is affected.
        let mut p = drain_rig_in(&w);
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(137, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -200, "drip doubles the 100");
        let mut p = drain_rig_in(&w);
        p.buffs.add_buff(20, 600); // Poisoned −4 (unchanged)
        p.buffs.add_buff(24, 600); // OnFire −100
        p.buffs.add_buff(137, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -204, "−4 poison + (−100×2) fire");
    }

    #[test]
    fn electrified_drains_more_while_moving() {
        // Dirty — `Player.cs:19074-19086`: −8, −32 more while `controlLeft ||
        // controlRight` (the run keys, not velocity).
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(144, 600);
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -8);
        p.control_left = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -40, "−8 −32 with the left key");
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(144, 600);
        p.control_right = true;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -40, "−8 −32 with the right key");
    }

    #[test]
    fn dot_death_reasons_follow_the_derivation_priority() {
        // Decision #5 — the caller-side reason selection (fast 7→12→8,
        // generic 22→9→10→23→8) is this phase's job; unit checks on the
        // derivation against `Player.cs:19313-19331,19361-19385`.
        let derive = |buffs: &[i32], expert: bool| -> DotDerivation {
            let (mut p, _dw) = drain_rig();
            p.buffs.expert_mode = expert;
            for &id in buffs {
                p.buffs.add_buff(id, 600);
            }
            p.derive_dot_inputs(expert)
        };
        let d = derive(&[20], false);
        assert!(!d.fast_lane);
        assert_eq!(d.death_generic, DotDeath::Poison); // 9 before 10/23
        let d = derive(&[70, 144], false);
        assert_eq!(d.death_generic, DotDeath::Poison); // poisoned||venom first
        let d = derive(&[144, 397], false);
        assert_eq!(d.death_generic, DotDeath::Electrified); // 10 before 23
        let d = derive(&[397], false);
        assert_eq!(d.death_generic, DotDeath::ChlorophyteSpore);
        let d = derive(&[67, 20], false);
        assert!(d.fast_lane);
        assert_eq!(d.death_fast, DotDeath::Burned); // the fast fallback 8
        assert_eq!(d.death_generic, DotDeath::Poison); // the generic lane too
        let d = derive(&[68, 67], false);
        assert_eq!(d.death_fast, DotDeath::Suffocation); // 7 before 8
        let d = derive(&[68, 38], true);
        assert_eq!(d.death_fast, DotDeath::Suffocation); // 7 before 12
        let d = derive(&[38], true);
        assert!(d.fast_lane);
        assert_eq!(d.death_fast, DotDeath::Tongued); // 12
        let d = derive(&[38], false);
        assert!(!d.fast_lane, "tongued needs expert for the fast lane");
        assert_eq!(d.regen, 0, "non-expert tongued drains nothing");
    }

    #[test]
    fn fast_lane_kill_carries_the_derived_reason() {
        // The derivation → engine seam end-to-end: the drain kill returns the
        // reason the held buffs selected (world_sim resolves the text).
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(68, 600); // suffocating → fast lane, reason 7
        p.regen.life_regen_count = -600;
        p.hp = 3;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Suffocation));
        let (mut p, dw) = drain_rig();
        p.buffs.add_buff(20, 600); // poisoned → generic lane, reason 9
        p.regen.life_regen_count = -120;
        p.hp = 1;
        p.update_survival(&dw, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Poison));
    }

    #[test]
    fn expired_buffs_are_swept_after_the_tick() {
        // `Player.cs:25608-25616`: slots with a type but no timer are deleted
        // right after `UpdateBuffs`, before any effect reads them.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(20, 1);
        p.buffs.add_buff(70, 1);
        p.update(&w, PlayerInput::default());
        assert!(
            p.buffs.active_types().is_empty(),
            "both expired buffs swept: {:?}",
            p.buffs.active_types()
        );
        // And a swept id can be re-added fresh afterwards.
        p.buffs.add_buff(20, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(20).is_some());
    }

    // ------------------------------------------------------------------
    // M2-PHYS-7 Phase 2 — the minecart ride (DW-2.1 .. DW-2.5)
    // ------------------------------------------------------------------

    /// The ride-test world: open sky, no ground — every fixture hand-places
    /// its tiles.
    fn cart_world() -> World {
        World::new("cart".into(), 200, 200)
    }

    /// Hand-places one track tile with explicit frame indices — the Phase-1
    /// convention (`minecart.rs` tests): `frame_x` is the front rail frame id,
    /// `frame_y` the back rail (−1 = single rail). Track placement
    /// (`FrameTrack`) is out of scope.
    fn place_track(w: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 314,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// A straight-rail run (frame 1) on `row` from `x0` to `x1`.
    fn rail_run(w: &mut World, row: i32, x0: i32, x1: i32) {
        for x in x0..=x1 {
            place_track(w, x, row, 1, -1);
        }
    }

    fn place_stone(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The mounted player position whose bottom-center magnet sits at
    /// `magnet`: `magnet = position + (width/2 − 25 + 25, 52/2 + 26)` =
    /// `position + (10, 52)` (`Minecart.GetMagnetPosition`,
    /// `Minecart.cs:1290-1294`).
    fn cart_at(magnet: Vec2) -> (f32, f32) {
        (magnet.x - 10.0, magnet.y - CART_MOUNT_HEIGHT)
    }

    /// Mounts and seats the cart on the straight rail of `row` (the frame-1
    /// rail line is `row * 16 + 6`, `Minecart.cs:117-121`) at magnet X `mx`
    /// with the given roll.
    fn cart_on_rail(w: &World, row: i32, mx: f32, vx: f32) -> Player {
        let mut p = Player::spawn(w);
        p.mount_cart();
        let (x, y) = cart_at(Vec2::new(mx, row as f32 * 16.0 + 6.0));
        p.x = x;
        p.y = y;
        p.prev_x = x;
        p.prev_y = y;
        p.vx = vx;
        p.fall_start = row;
        p.fall_start2 = row;
        p
    }

    /// The magnet's world point of the current position.
    fn magnet_of(p: &Player) -> Vec2 {
        Vec2::new(p.x + 10.0, p.y + CART_MOUNT_HEIGHT)
    }

    /// DW-2.1 — the `TR_CART` pin's body, `Mount.SetMount(MountID.Minecart)`
    /// (`Mount.cs:6196-6305`): the Produced ride-field set lands at its
    /// defaults, the mount buff is granted, and the body grows 42 -> 52
    /// bottom-center. A same-type re-mount is a no-op (`:6198`); the death
    /// path strips the ride state (`Player.cs:38907`).
    /// S1-L-10b — `Player.GrappleMovement`'s Track Hook hand-over
    /// (`Player.cs:22659-22693`): the hook over a track tile gives the
    /// player to the minecart.
    #[test]
    fn the_track_hook_hands_the_player_to_the_cart() {
        let mut w = cart_world();
        // A straight rail on row 30; the player stands above it.
        rail_run(&mut w, 30, 20, 30);
        let mut p = Player::spawn(&w);
        p.x = 25.0 * 16.0;
        p.y = 28.0 * 16.0;
        // The Track Hook (403) sits on the rail, overlapping the player.
        let hook_center = Vec2::new(25.0 * 16.0 + 8.0, 30.0 * 16.0 + 8.0);
        let hooks = [(crate::projectile::TRACK_HOOK, hook_center, 0.0)];
        assert!(p.track_hook_cart_transfer(&w, &hooks));
        assert!(p.cart_mounted, "the cart ride starts");
        assert_eq!(p.mount.kind(), 13, "the wooden minecart (:22676)");
        // The hooks go before the probe (:22675).
        assert!(p.release_grappling_hooks);
        // The player snapped onto the rail (the magnet rides the rail
        // surface, so the body sits above it).
        assert!(p.y < 30.0 * 16.0, "the snapped y: {}", p.y);
    }

    #[test]
    fn mount_cart_sets_the_ride_state_fields() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        let top = p.y;
        p.mount_cart();
        assert!(p.cart_mounted);
        assert!(!p.cart_flip);
        assert!(!p.on_track);
        assert!(!p.on_wrong_ground);
        assert_eq!(p.track_boost, 0.0);
        assert_eq!(p.last_boost, (0, 0));
        assert_eq!(p.cart_ramp_time, 0);
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
        assert_eq!(p.body_height(), 52.0, "the mounted body is 20x52");
        assert_eq!(p.y, top - 10.0, "the grow lifts the top, the bottom stays");
        assert!(
            p.buffs.find_buff_index(CART_MOUNT_BUFF).is_some(),
            "the mount buff 118 for 3600 ticks"
        );
        // The same-type re-mount early-returns (`Mount.cs:6198`).
        let y = p.y;
        p.mount_cart();
        assert_eq!(p.y, y);
        // The respawn (post-death) path resets the ride state.
        p.respawn();
        assert!(!p.cart_mounted);
        assert_eq!(p.body_height(), 42.0);
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
    }

    /// DW-2.2 — the cart top speed is the mount's 13 (`Mount.cs:926`) on a
    /// straight rail: the input accel (`Player.cs:19904-19916`) may overshoot
    /// by one step and the coast clamp (`:20043-20057`) pins it exactly.
    #[test]
    fn cart_top_speed_is_thirteen_on_straight_track() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        for _ in 0..30 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            assert!(p.on_track, "riding the straight");
            // The accel arm may overshoot by one step (`:19904-19916` has no
            // clamp); the coast clamp erases it the next tick.
            assert!(p.vx <= 13.0 + 0.04 + 1e-4, "bounded overshoot: {}", p.vx);
        }
        assert_eq!(p.vx, 13.0, "the coast clamp pins the top speed");
    }

    /// DW-2.2 — the booster's ±4 applies NEXT tick pre-input, then the input
    /// step, and the clamp caps at 13 (`Player.cs:19818-19833`; the detection
    /// `Minecart.cs:832-847`).
    #[test]
    fn booster_applies_four_next_tick_pre_input_and_clamps() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        // Right-boost booster (frame 30) mid-run.
        place_track(&mut w, 50, 40, 30, -1);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 5.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 120 {
            p.update(&w, input);
            ticks += 1;
        }
        assert!(ticks < 120, "the booster was crossed");
        assert_eq!(p.track_boost, 4.0, "the booster tile arms the push");
        let before = p.vx;
        p.update(&w, input);
        assert!(
            (p.vx - (before + 4.0 + 0.04)).abs() < 1e-4,
            "the boost applied pre-input, then the accel step: {}",
            p.vx
        );
        assert_eq!(p.track_boost, 0.0, "the push is spent");
        // The clamp: a fast cart caps at 13 instead of gaining the full 4.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.9);
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 120 {
            p.update(&w, input);
            ticks += 1;
        }
        p.update(&w, input);
        assert_eq!(p.vx, 13.0, "the boost clamp: {}", p.vx);
        assert_eq!(p.track_boost, 0.0);
        // The left-boost booster (frame 31) pushes a leftward roll harder.
        place_track(&mut w, 50, 40, 31, -1);
        let mut p = cart_on_rail(&w, 40, 65.0 * 16.0 + 8.0, -5.0);
        let input = PlayerInput {
            left: true,
            ..Default::default()
        };
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 200 {
            p.update(&w, input);
            ticks += 1;
        }
        assert_eq!(p.track_boost, -4.0, "the left booster arms a negative push");
        let before = p.vx;
        p.update(&w, input);
        assert!((p.vx - (before - 4.0 - 0.04)).abs() < 1e-4, "{}", p.vx);
    }

    /// DW-2.2 — the ramp end launches at 45° (`Minecart.cs:692-701`:
    /// `(vx, −vx·√2/2)` for an up-right exit) and freezes gravity for
    /// `(int)(min(1, |vx|/13) · 20)` ticks (`Player.cs:28148-28150`,
    /// `:27352-27359`). The freeze also proves the input-accel gate
    /// (`velocity.Y == 0`, `Player.cs:19849`) — the DW-2.5 mechanism.
    #[test]
    fn ramp_launch_suppresses_gravity_and_leaves_at_45_degrees() {
        let mut w = cart_world();
        rail_run(&mut w, 41, 30, 38);
        place_track(&mut w, 39, 41, 6, -1); // corner: straight left, up right
        place_track(&mut w, 40, 40, 16, -1); // ramp rising right, ramp end
        let mut p = cart_on_rail(&w, 41, 33.0 * 16.0 + 8.0, 13.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut launched = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.cart_ramp_time > 0 {
                launched = true;
                break;
            }
        }
        assert!(launched, "the ramp end launched the cart");
        let launch_vy = -13.0 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((p.vy - launch_vy).abs() < 1e-3, "45 degrees up: {}", p.vy);
        assert!((p.vx - 13.0).abs() < 1e-4, "the roll carries through");
        assert_eq!(p.cart_ramp_time, 20, "full speed freezes 20 ticks");
        assert!(!p.on_track, "airborne off the ramp end");
        // The freeze: gravity and input accel both silent while vy != 0.
        let (vx0, vy0) = (p.vx, p.vy);
        for _ in 0..5 {
            p.update(&w, input);
            assert!(
                (p.vx - vx0).abs() < 1e-4,
                "no input accel airborne: {}",
                p.vx
            );
            assert!((p.vy - vy0).abs() < 1e-4, "gravity frozen: {}", p.vy);
        }
        // Gravity resumes once the timer runs out.
        for _ in 0..25 {
            p.update(&w, input);
        }
        assert!(p.vy > vy0, "gravity resumed: {} -> {}", vy0, p.vy);
    }

    /// DW-2.2 — the corner hand-off: the cart follows an up-right corner at
    /// speed (`Minecart.cs:600-614` corner switch) and ends on the upper rail
    /// line (the `:818-819` snap).
    #[test]
    fn corner_follows_the_rail_at_speed() {
        let mut w = cart_world();
        rail_run(&mut w, 41, 30, 38);
        place_track(&mut w, 39, 41, 6, -1); // corner: straight left, up right
        place_track(&mut w, 40, 40, 4, -1); // corner: from below, straight right
        rail_run(&mut w, 40, 41, 60);
        let mut p = cart_on_rail(&w, 41, 33.0 * 16.0 + 8.0, 10.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..12 {
            p.update(&w, input);
            assert!(p.on_track, "the rail is never lost through the corner");
        }
        let magnet_y = magnet_of(&p).y;
        assert!(
            (magnet_y - (40.0 * 16.0 + 6.0)).abs() < 0.5,
            "snapped to the upper rail: {}",
            magnet_y
        );
    }

    /// DW-2.3 — off the rails the coast friction is `runSlowdown` = 0.2 per
    /// tick (`Player.cs:26670` + `:20024-20041`) and a direction-key press
    /// edge dismounts (`:26672-26674` → `Mount.TryDismount`).
    #[test]
    fn off_track_friction_is_point_two_and_direction_press_dismounts() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 44); // the run ends in open air
        for x in 36..80 {
            place_stone(&mut w, x, 43); // the landing floor 3 rows down
        }
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        // Ride off the end and fall to the floor; the settle is complete when
        // the off-track-at-rest flag is up with the body at rest.
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
            if !p.on_track && p.vy == 0.0 && p.on_wrong_ground {
                break;
            }
        }
        assert!(!p.on_track, "off the rails");
        assert_eq!(p.vy, 0.0, "settled on the floor");
        assert!(p.on_wrong_ground, "the on-track check flagged wrong ground");
        // The coast bleeds exactly 0.2 per tick while |vx| >= 1 (the fall
        // decayed 6 by 0.2 per fall tick — about 3 remain).
        assert!(p.vx >= 1.0, "measurable roll remains: {}", p.vx);
        p.update(&w, PlayerInput::default());
        let v1 = p.vx;
        p.update(&w, PlayerInput::default());
        let v2 = p.vx;
        assert!(
            (v1 - v2 - 0.2).abs() < 1e-4,
            "0.2/tick off-track coast: {} -> {}",
            v1,
            v2
        );
        // A direction-key press edge dismounts; the shrink keeps the bottom.
        let y_before = p.y;
        p.update(
            &w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        assert!(!p.cart_mounted, "the press edge dismounted");
        assert_eq!(p.body_height(), 42.0);
        assert_eq!(p.y, y_before + 10.0, "the shrink anchors the bottom");
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
    }

    /// DW-2.3 — the cart jump: the −0.001 magnet nudge
    /// (`Player.cs:20671-20675`) then the fixed cart jump stats 15/5.15
    /// (`Mount.cs:929-930`; no `JumpHeight` case for carts) — the rider
    /// leaves the rail, the mount stays, and the rider lands back.
    #[test]
    fn jump_from_the_cart_uses_the_cart_jump_stats() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        p.update(&w, PlayerInput::default());
        assert!(p.on_track, "seated");
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert_eq!(p.jump, CART_JUMP_HEIGHT, "the cart jump height 15");
        assert!(
            (p.vy - (-CART_JUMP_SPEED + GRAVITY)).abs() < 1e-4,
            "the 5.15 impulse minus this tick's gravity: {}",
            p.vy
        );
        assert!(p.cart_mounted, "still mounted");
        assert!(!p.on_track, "left the rail");
        // Hold the jump through the rise, then fall back onto the rail.
        for _ in 0..30 {
            p.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
        }
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_track, "landed back on the rail");
        assert!(p.cart_mounted, "the mount survived the hop");
    }

    /// DW-2.3 — `Player.cs:25311-25314`: a track rider resting on rails takes
    /// no fall damage; the same drop onto stone does.
    #[test]
    fn landing_on_rails_zeroes_fall_damage() {
        // The rail catch: a 30-row drop lands harmlessly.
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        let mut p = cart_on_rail(&w, 40, 40.0 * 16.0 + 8.0, 0.0);
        p.y = 10.0 * 16.0;
        p.prev_y = p.y;
        p.fall_start = 10;
        p.fall_start2 = 10;
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_track, "landed on the rail");
        assert_eq!(p.hp, 100, "no fall damage on rails");
        // The same drop onto stone hurts.
        let mut w2 = cart_world();
        for x in 30..60 {
            place_stone(&mut w2, x, 41);
        }
        let mut p2 = Player::spawn(&w2);
        p2.mount_cart();
        p2.x = 40.0 * 16.0 - 10.0;
        p2.y = 10.0 * 16.0;
        p2.prev_y = p2.y;
        p2.fall_start = 10;
        p2.fall_start2 = 10;
        for _ in 0..90 {
            p2.update(&w2, PlayerInput::default());
        }
        assert!(p2.hp < 100, "the same drop onto stone hurts: {}", p2.hp);
    }

    /// DW-2.4 — riding over a pressure track (frame 21, `_trackType == 1`)
    /// trips the switch once per pass: the magnet tile is handed to
    /// `Wiring::hit_switch` by the caller (`Player.cs:28237-28241`,
    /// `Minecart.cs:1281-1288`).
    #[test]
    fn riding_over_a_pressure_track_trips_the_switch_once_per_pass() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        place_track(&mut w, 50, 40, 21, -1); // the pressure straight
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut trips = 0;
        let mut trip_tile = None;
        for _ in 0..40 {
            p.update(&w, input);
            if let Some((i, j)) = p.take_switch_trip() {
                trips += 1;
                trip_tile = Some((i, j));
            }
        }
        assert_eq!(trips, 1, "once per pass");
        assert_eq!(trip_tile, Some((50, 40)), "the magnet tile");
        // A plain-rail pass never trips.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        for _ in 0..10 {
            p.update(&w, input);
            assert!(
                p.take_switch_trip().is_none(),
                "no trip off the pressure tile"
            );
        }
    }

    /// DW-2.5 — the bouncy bumper (frame 24): the rewind flips the roll and
    /// the rider keeps the rail (`Minecart.cs:669-691`, `:914-922`;
    /// `Player.cs:28123-28136` toggles `cartFlip` on the held key). The
    /// input-accel suppression half of this item is the `velocity.Y == 0`
    /// gate, exercised on the ramp flight in `ramp_launch_...` — a flat-rail
    /// bounce itself resolves with vy == 0 (the ride snaps zero the march Y
    /// before the resolve, `Minecart.cs:920`).
    #[test]
    fn bouncy_bumper_flips_velocity_and_the_rider_keeps_the_rail() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 24, -1); // bouncy bumper, rail on the right half
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 8.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut flipped = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.vx < 0.0 {
                flipped = true;
                break;
            }
        }
        assert!(flipped, "the bounce flipped the roll");
        assert!(p.cart_flip, "the held direction key toggled the flip");
        assert!(p.on_track, "still riding the bouncy tile's rail");
        assert_eq!(p.vy, 0.0, "the ride resolve zeroes vy");
        let magnet_y = magnet_of(&p).y;
        assert!(
            (magnet_y - (40.0 * 16.0 + 6.0)).abs() < 0.5,
            "still on the rail line: {}",
            magnet_y
        );
    }

    /// Review probe (Phase 2 gate fix 1) — the direction lock reads the
    /// POST-boost roll: the boost applies at `Player.cs:19818-19833` and the
    /// lock sits at `:19835-19843`, both inside `HorizontalMovement`. Rolling
    /// left over a right-boost booster with the right key held flips the
    /// facing on the boost-apply tick, not one tick later.
    #[test]
    fn boost_tick_flips_the_facing_to_the_boosted_direction() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 30, -1); // right-boost booster
        let mut p = cart_on_rail(&w, 40, 46.0 * 16.0 + 8.0, -3.0);
        p.facing = -1;
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        // The crossing tick: the booster arms the push against the roll; the
        // lock (pre-boost roll = negative) keeps the facing.
        let mut crossed = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.track_boost != 0.0 {
                crossed = true;
                break;
            }
        }
        assert!(crossed, "the booster was crossed rolling left");
        assert_eq!(p.facing, -1, "the crossing tick keeps the roll facing");
        // The boost-apply tick: the roll turns positive pre-input and the
        // lock (post-boost) flips the facing the same tick.
        p.update(&w, input);
        assert!(p.vx > 0.0, "the boost turned the roll: {}", p.vx);
        assert_eq!(p.facing, 1, "the boosted direction, same tick");
    }

    /// Review probe (Phase 2 gate fix 1) — the dismount edge fires in the
    /// stat override (`Player.cs:26672-26674`) and the now-unmounted
    /// direction block inside `HorizontalMovement` (`:19835-19843`) flips
    /// the facing the SAME tick.
    #[test]
    fn dismount_tick_flips_the_facing_to_the_pressed_direction() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 44);
        for x in 36..80 {
            place_stone(&mut w, x, 43);
        }
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        // Roll off the end and settle off-track.
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
            if !p.on_track && p.vy == 0.0 && p.on_wrong_ground {
                break;
            }
        }
        assert!(p.on_wrong_ground && !p.on_track && p.vy == 0.0, "settled");
        assert_eq!(p.facing, 1, "the keyless roll-off kept the spawn facing");
        // The left press edge: the dismount and the facing flip land together.
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(!p.cart_mounted, "the press edge dismounted");
        assert_eq!(p.facing, -1, "the pressed direction, same tick");
    }

    /// The regular bumper (frame 2) dead-stops the cart at the rail line
    /// (`Minecart.cs:660-668` — the `stopped` resolve keeps only the travelled
    /// X). With the key held the C# vibrates in place: the accel arm grants
    /// one 0.04 step on the vy == 0 ticks and the boundary sentinel eats it
    /// when the march crosses the sub edge, so |vx| never exceeds one step.
    #[test]
    fn bumper_stops_the_cart_on_the_rail() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 2, -1); // bumper on the left half
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 8.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut reached = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.vx.abs() < 1e-4 {
                reached = true;
            }
            if reached {
                // The held key vibrates in place: one 0.04 accel step on the
                // vy == 0 ticks, eaten by the boundary sentinel's stop.
                assert!(
                    p.vx.abs() <= 0.04 + 1e-4,
                    "held against the bumper: {}",
                    p.vx
                );
                assert!(p.on_track, "resting on the rail, never unseated");
            }
        }
        assert!(reached, "the stop zeroed the roll at the boundary");
        let magnet = magnet_of(&p);
        assert!((magnet.y - (40.0 * 16.0 + 6.0)).abs() < 0.5);
        assert!(magnet.x < 45.0 * 16.0 + 0.5, "never inside the bumper sub");
    }

    /// `Player.cs:28088-28092`: the cart always falls through platforms —
    /// `fallThrough`/`flag31` are forced for `Mount.Cart`.
    #[test]
    fn the_cart_falls_through_platforms() {
        let mut w = cart_world();
        for x in 30..60 {
            let i = w.index(x, 38);
            w.tiles[i] = Tile {
                kind: 19, // platform
                frame_y: 0,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        for x in 30..60 {
            place_stone(&mut w, x, 42);
        }
        let mut p = Player::spawn(&w);
        p.mount_cart();
        let (x, y) = cart_at(Vec2::new(40.0 * 16.0 + 8.0, 30.0 * 16.0));
        p.x = x;
        p.y = y;
        p.prev_x = x;
        p.prev_y = y;
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
        }
        let bottom = p.y + CART_MOUNT_HEIGHT;
        assert!(
            (bottom - 42.0 * 16.0).abs() < 1.0,
            "fell through to the floor: {}",
            bottom
        );
        assert!(!p.on_track, "stone is not a rail");
    }

    /// Dirty (plan P2) — `TR_CART` with no rails under the magnet: the
    /// mounted cart just falls under gravity; no ride state, no panic. The
    /// first rest tick flags `on_wrong_ground` (the C# off-track-at-rest
    /// check, `Player.cs:26665-26674`) and the cart keeps it through the fall
    /// (`:24371-24374` preserves it for carts).
    #[test]
    fn cart_with_no_rails_falls_normally() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        p.mount_cart();
        p.y = 30.0 * 16.0;
        p.update(&w, PlayerInput::default());
        assert!(p.on_wrong_ground, "resting off-track flags wrong ground");
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
            assert!(!p.on_track);
        }
        assert!(p.vy > 0.0, "falling under gravity: {}", p.vy);
    }

    // ------------------------------------------------------------------
    // M2-PHYS-7 Phase 3 — wheel sparks + sounds
    // ------------------------------------------------------------------

    /// Finds a stream seed where one rolling call at `|vx| = vel_x` queues
    /// exactly `want` sparks — the gate rolls interleave with the 14-roll
    /// sparks (the rain.rs seed-search idiom, run against the same emitter
    /// constants the tick passes).
    fn seed_with_spark_count(vel_x: f32, want: usize) -> u64 {
        for s in 1u64.. {
            let mut sparks = Vec::new();
            let mut rng = s;
            crate::minecart::rolling_wheel_sparks(
                &mut Vec::new(),
                &mut sparks,
                &mut rng,
                Vec2::new(100.0, 200.0),
                20,
                52,
                Vec2::new(vel_x, 0.0),
                13.0,
                0.0,
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    /// W4-c14 - the rocket cadence cue (`Player.cs:26884-26896` +
    /// `:26903-26905`): the first fuel tick queues the tier's Item cue at
    /// the top-left and arms rocketSoundDelay (30 tier 1 / 15 tiers 2-4,
    /// the same tick's decrement brings it to 29/14); the held thrust
    /// stays silent until the cadence expires. W4-c14 doc-sweep: the cue
    /// reads the VANITY tier (the producer lands - the pin/equip rows set
    /// both tiers equal, so this witness's pinned tiers drive it unchanged).
    #[test]
    fn w4_c14_the_rocket_cue_rides_its_own_cadence() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // Arm the midair trigger state: fuel, the release latch, canRocket.
        // The PIN survives ResetEffects' per-tick `rocket_boots = 0` clear
        // (`:18760`-area; the `:26628` re-grant reads it). `jump = 1` keeps
        // the held-jump branch out of the extra-jump chain.
        p.pinned_rocket_boots = 1;
        p.rocket_boots = 1;
        p.rocket_time = 7;
        p.rocket_time_max = 7;
        p.can_rocket = true;
        p.rocket_release = true;
        p.jump = 1;
        p.vy = 2.0; // airborne
        p.on_ground = false;
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        let cues = p.take_cart_sounds();
        assert_eq!(cues.len(), 1, "the first fuel tick plays");
        assert_eq!(cues[0].cue, "Item_13", "tier 1 -> Item13");
        assert_eq!(p.rocket_sound_delay, 29, "30 armed, one tick spent");
        // The held thrust: the cadence blocks replays.
        for _ in 0..10 {
            p.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
            assert!(p.take_cart_sounds().is_empty(), "the cadence blocks");
        }
        // Tier 2 arms 15 (14 after the same tick's decrement).
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.rocket_time = 7;
        p.rocket_time_max = 7;
        p.can_rocket = true;
        p.rocket_release = true;
        p.jump = 1;
        p.vy = 2.0;
        p.on_ground = false;
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        let cues = p.take_cart_sounds();
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].cue, "Item_24", "tier 2 -> Item24");
        assert_eq!(p.rocket_sound_delay, 14);
    }

    /// W4-c14 doc-sweep - the vanity-rocket producer
    /// (`Player.cs:2760`/`:14420`/`:15023`/`:15222`/`:15380-15400` +
    /// `UpdateBootVisualEffects` `:36704-36751` + the Red's Leggings 668
    /// `:13088-13091`): the equip rows set BOTH tiers equal, the social
    /// slot re-derives the visual tier, and tier 6 plays the Item24 cue.
    /// The cue arm reads `vanity_rocket_boots`, so a divergent vanity tier
    /// picks its OWN cue while the gameplay tier stays the thrust source.
    #[test]
    fn w4_c14_the_vanity_rocket_tier_produces_the_cue() {
        let w = ground_world();
        // The functional equip rows set both tiers equal.
        let mut p = Player::spawn(&w);
        p.equipped[3] = 128; // Rocket Boots -> tier 1
        p.update_equips();
        assert_eq!(p.rocket_boots, 1);
        assert_eq!(p.vanity_rocket_boots, 1, "the equip row sets both equal");
        p.equipped[3] = 4874; // Hellfire Treads -> tier 5
        p.update_equips();
        assert_eq!(p.vanity_rocket_boots, 5);
        // The Red's Leggings 668 (leg slot 2) sets tier 6 AFTER the loops.
        p.equipped[2] = 668;
        p.update_equips();
        assert_eq!(p.vanity_rocket_boots, 6, "the bee-costume tier 6");
        assert_eq!(p.rocket_boots, 5, "the gameplay tier is untouched");
        // The social slot (13) re-derives the visual tier (no hide gate).
        let mut p = Player::spawn(&w);
        p.equipped[13] = 1862; // Frostspark in the vanity slot -> tier 3
        p.update_equips();
        assert_eq!(p.vanity_rocket_boots, 3, "the vanity pass re-derives");
        assert_eq!(p.rocket_boots, 0, "no functional grant from the social slot");
        // The cue reads the VANITY tier: the Red's Leggings 668 sets tier 6
        // AFTER the pin re-arms tier 1, so the cue plays Item24 cadence 15
        // while the gameplay tier stays 1 (the thrust source). The 668 row
        // re-arms every tick, so the divergence survives reset_effects.
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_rocket_boots = 1; // the gameplay thrust tier
        p.equipped[2] = 668; // the Red's Leggings -> vanity tier 6
        p.rocket_time = 7;
        p.rocket_time_max = 7;
        p.can_rocket = true;
        p.rocket_release = true;
        p.jump = 1;
        p.vy = 2.0;
        p.on_ground = false;
        p.update(&w, PlayerInput { jump: true, ..Default::default() });
        assert_eq!(p.rocket_boots, 1, "the gameplay tier stays the thrust source");
        assert_eq!(p.vanity_rocket_boots, 6, "the 668 row wins the visual tier");
        let cues = p.take_cart_sounds();
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].cue, "Item_24", "vanity tier 6 -> Item24");
        assert_eq!(p.rocket_sound_delay, 14, "cadence 15, one tick spent");
    }

    /// W4-c14 - the Pogo Stick's (mount 43) hop cue (`Player.cs:19782-19793`):
    /// a direction key held while grounded with jump free queues Item168 at
    /// the CENTRE and takes the half-speed hop (the early return - no run
    /// accel that tick); the ground jump (`:20762-20764`) plays it too.
    #[test]
    fn w4_c14_the_pogo_hop_and_jump_queue_item168_at_the_centre() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(43, false), "the pogo equips");
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "landed first");
        let (cx, cy) = p.center();
        p.update(
            &w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        let cues = p.take_cart_sounds();
        assert_eq!(cues.len(), 1, "the hop plays once");
        assert_eq!(cues[0].cue, "Item_168");
        assert_eq!((cues[0].x, cues[0].y), (cx, cy), "the centre position");
        assert!(p.vy < 0.0, "the hop lifted");
        // (`jump = 15` is written by the hop, then the same tick's
        // jump-released branch clears it - `Player.cs:21026`, the C# order.)
        // The ground jump (jump held on a resting tick) plays it too.
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(43, false));
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        let cues = p.take_cart_sounds();
        assert_eq!(cues.len(), 1, "the ground jump plays once");
        assert_eq!(cues[0].cue, "Item_168");
    }

    /// DW-3.2 - the landing burst (`Minecart.cs:809-813` via the Phase-1
    /// `TrackFlags::landing`): a real fall onto a rail queues the Item53 cue
    /// and exactly 10 sparks at the pre-call magnet point, with the
    /// pre-call rotation (`Player.cs:28110-28111` sets the emitter statics
    /// before `TrackCollision` runs).
    #[test]
    fn landing_on_a_rail_bursts_ten_sparks_and_item53() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 0.0);
        p.y -= 64.0; // four rows above the rail line
        p.prev_y = p.y;
        p.vy = 5.0; // the gate needs `velocity.Y > defaultGravity` (0.4)
        p.fall_start2 = (p.y / 16.0) as i32 - 3; // a real fall origin
        // The fall takes a handful of ticks to cross the 64 px to the rail;
        // the landing tick queues the burst with its pre-call position.
        let mut landed_at = None;
        for _ in 0..20 {
            let (pre_x, pre_y) = (p.x, p.y);
            p.update(&w, PlayerInput::default());
            if !p.pending_sparks.is_empty() {
                landed_at = Some((pre_x, pre_y));
                break;
            }
        }
        let (pre_x, pre_y) = landed_at.expect("the landing fired");
        assert!(p.on_track, "the fall ended on the rail");
        assert_eq!(p.pending_sparks.len(), 10, "the 10-spark burst");
        assert_eq!(p.pending_cart_sounds.len(), 1);
        assert_eq!(p.pending_cart_sounds[0].cue, "Item_53");
        // Every request sits at the pre-call magnet point (rotation 0 on the
        // flat rail): y exactly `pre_y + 52`, x = pre_x + 10 ± 13 — the +4
        // box jitter lands in the pool slot at the drain (`Next(1)` = 0,
        // `Dust.cs:159-160`).
        for s in &p.pending_sparks {
            assert_eq!(s.pos.y, pre_y + 52.0);
            assert!(
                (s.pos.x - (pre_x + 23.0)).abs() < 1e-3 || (s.pos.x - (pre_x - 3.0)).abs() < 1e-3,
                "x {}",
                s.pos.x
            );
        }
    }

    /// DW-3.3 (dirty) — the rolling gates are direction-opposite
    /// (`Player.cs:19868`: the left arm needs `velocity.X >= 0f`; `:19926`:
    /// the right arm `velocity.X <= 0f`): riding forward is silent, braking
    /// a same-sign roll sparks.
    #[test]
    fn riding_forward_does_not_spark_the_braking_arms_do() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 140);
        // Settle one tick so the previous tick's `onTrack` is up.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        p.update(&w, PlayerInput::default());
        // All three 50% gates would pass — the arm's sign gate is the only
        // thing that can stop them.
        p.spark_rng = seed_with_spark_count(10.4, 3);
        // Riding forward (right held, vx > 0): the right arm's `vx <= 0`
        // gate fails — no sparks, no Item55.
        p.update(
            &w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        assert!(p.pending_sparks.is_empty(), "forward riding is silent");
        assert!(p.pending_cart_sounds.is_empty());
        // Braking (left held against the rightward roll): the left arm's
        // `vx >= 0` gate fires the three rolls → 3 sparks + Item55.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 13.0);
        p.update(&w, PlayerInput::default());
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert_eq!(p.pending_sparks.len(), 3, "the braking arm sparks");
        assert_eq!(p.pending_cart_sounds.len(), 1);
        assert_eq!(p.pending_cart_sounds[0].cue, "Item_55");
        // At rest the arm's own accel (`Player.cs:19855`: `velocity.X -=
        // runAcceleration`, unconditional for a grounded cart) flips vx to
        // −0.04 before the gate reads it — the at-rest press never sparks
        // (the C# order; the gate is the arm's post-accel velocity).
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 0.0);
        p.update(&w, PlayerInput::default());
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(
            p.pending_sparks.is_empty(),
            "the at-rest press never sparks"
        );
    }

    /// DW-3.3 (dirty) — no sparks off-track or airborne: the gate needs the
    /// previous tick's `onTrack` with `!onWrongGround` and `velocity.Y == 0`
    /// (`Player.cs:19868`).
    #[test]
    fn sparks_are_absent_off_track_and_airborne() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        // Off-track: a mounted cart on plain ground (no rails) never passes
        // the gate — the on-track check flags `on_wrong_ground` and the
        // previous tick's `onTrack` is down.
        let mut w2 = cart_world();
        for x in 20..80 {
            place_stone(&mut w2, x, 45);
        }
        let mut p = Player::spawn(&w2);
        p.mount_cart();
        p.y = 45.0 * 16.0 - 52.0;
        p.prev_y = p.y;
        p.vx = 10.0;
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w2,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.on_wrong_ground, "the off-track flag is up");
        assert!(p.pending_sparks.is_empty(), "off-track: no sparks");
        assert!(p.pending_cart_sounds.is_empty(), "off-track: no Item55");
        // Airborne: jump off the rail, then brake mid-air — `onTrack` (the
        // previous tick's) is down and `velocity.Y != 0`.
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        p.update(&w, PlayerInput::default());
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.vy != 0.0, "the jump launched the cart");
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.pending_sparks.is_empty(), "airborne: no sparks");
        assert!(p.pending_cart_sounds.is_empty(), "airborne: no Item55");
    }

    /// DW-3.3 (the replay half, through the tick) — the same world, seed and
    /// inputs give byte-equal spark queues and identical stream positions.
    #[test]
    fn the_player_spark_stream_replays_identically() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 140);
        let mut a = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        let mut b = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        let seed = seed_with_spark_count(10.4, 3);
        a.spark_rng = seed;
        b.spark_rng = seed;
        for _ in 0..10 {
            let input = PlayerInput {
                left: true,
                ..Default::default()
            };
            a.update(&w, input);
            b.update(&w, input);
            assert_eq!(a.pending_sparks, b.pending_sparks, "byte-equal queues");
            assert_eq!(a.pending_cart_sounds, b.pending_cart_sounds);
            assert_eq!(a.spark_rng, b.spark_rng, "identical stream positions");
        }
    }

    /// Dirty (pitfall §10.13) — the dismount resets `cartFlip`/`lastBoost`/
    /// `fullRotation` and clears the mount buff (`Mount.cs:6151-6162`), and a
    /// fresh mount starts clean (`:6205-6211`).
    #[test]
    fn dismount_resets_the_ride_state() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        p.mount_cart();
        p.cart_flip = true;
        p.last_boost = (3, 4);
        p.full_rotation = 0.5;
        p.full_rotation_origin = (10.0, 52.0);
        p.track_boost = 2.0;
        p.cart_dismount();
        assert!(!p.cart_mounted);
        assert!(!p.cart_flip);
        assert_eq!(p.last_boost, (0, 0));
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
        p.mount_cart();
        assert!(p.cart_mounted);
        assert_eq!(p.last_boost, (0, 0), "the fresh mount starts clean");
        assert!(!p.cart_flip);
    }
    // -- M2-MOUNT-1 phase 2: the per-tick mount block ------------------------

    #[test]
    fn the_mount_set_and_dismount_extras_follow_the_c() {
        // S1-L-9 — `Mount.SetMount` (`Mount.cs:6196-6305`) / `Dismount`
        // (`:6142-6194`): the pirate-ship dash reset (`:6228-6235`), the
        // UFO/drill rotation origin (`:6279-6282`), the swap resets
        // (`:6205-6213`) and the dismount's rotation clear (`:6161-6162`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 10.0;
        p.vy = -4.0;
        p.dash = 2;
        p.dash_type = 3;
        p.dash_delay = 5;
        assert!(p.set_mount(44, false));
        assert!((p.vx - 2.0).abs() < 0.001, "velocity *= 0.2 (:6230)");
        assert!((p.vy + 0.8).abs() < 0.001, "velocity *= 0.2 (:6230)");
        assert_eq!(p.dash, 0, ":6231");
        assert_eq!(p.dash_type, 0, ":6232");
        assert_eq!(p.dash_delay, 0, ":6233");
        // The UFO (7) centres the full rotation on the (grown) box.
        p.full_rotation = 0.5;
        assert!(p.set_mount(7, false));
        assert_eq!(p.full_rotation_origin, (10.0, p.height / 2.0), ":6281");
        // A swap clears the rotation (`:6210-6211`).
        p.full_rotation = 0.5;
        p.full_rotation_origin = (3.0, 4.0);
        assert!(p.set_mount(10, false));
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
        // The dismount clears it too (`:6161-6162`).
        p.full_rotation = 0.25;
        assert!(p.dismount(&w, true));
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
    }

    #[test]
    fn the_chillemt_mounts_add_a_tenth_to_every_damage_class() {
        // `Mount.UpdateEffects` (`Mount.cs:4806-4814`) — the Chillemt mounts.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(62, false);
        p.update_mount(&w, false, None);
        assert!((p.melee_damage - 1.1).abs() < 0.0001, "{}", p.melee_damage);
        assert!((p.ranged_damage - 1.1).abs() < 0.0001);
        assert!((p.magic_damage - 1.1).abs() < 0.0001);
        assert!((p.minion_damage - 1.1).abs() < 0.0001);
    }

    #[test]
    fn the_mount_replaces_the_run_stats_and_suppresses_the_wings() {
        // DW-2.1/2.2 — the replace mode `Player.cs:26649-26654` and the
        // wings/rocket suppression `:26625-26635`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.wings_logic = 22; // pretend wings so the suppression has a target
        p.has_wings = true;
        p.rocket_boots = 1;
        p.set_mount(10, false); // Unicorn 4/12/0.3
        p.update(&w, PlayerInput::default());
        assert_eq!(p.max_run_speed_value(), 4.0, "maxRunSpeed replaced");
        assert_eq!(p.acc_run_speed_value(), 12.0, "accRunSpeed = DashSpeed");
        assert!(!p.has_wings && p.wings_logic == 0, "wings suppressed");
        assert_eq!(p.rocket_boots, 0, "rocketBoots zeroed");
    }

    #[test]
    fn flying_with_wings_max_merges_instead_of_replacing() {
        // `:26643-26648` — flag17 keeps the better stat (Hermes' 6.0 beats
        // the Slime mount's 4.0 DashSpeed).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.equipped[3] = 54; // Hermes — `accRunSpeed = 6f`
        p.update(&w, PlayerInput::default()); // the grant pass runs
        assert_eq!(p.acc_run_speed_value(), 6.0);
        p.set_mount(3, false);
        p.wings_logic = 22;
        p.vy = -1.0; // airborne so flag17 can arm
        p.mount.set_test_overrides(Some(true), None); // the M2-MOUNT-2 row
        p.update_mount(&w, false, None);
        assert_eq!(p.acc_run_speed_value(), 6.0, "max(6.0, 4.0)");
        // without the wings the same merge replaces: 4.0
        p.mount.set_test_overrides(Some(false), None);
        p.wings_logic = 22;
        p.update_mount(&w, false, None);
        assert_eq!(p.acc_run_speed_value(), 4.0, "replaced while grounded");
    }

    #[test]
    fn the_additive_mode_adds_instead_of_replacing() {
        // `:26637-26641` — the `MovementStatsAreAdditive` branch (roller
        // skates in vanilla; the row lands with M2-MOUNT-4).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(3, false);
        p.mount.set_test_overrides(None, Some(true));
        p.update_mount(&w, false, None);
        assert_eq!(p.max_run_speed_value(), 3.0 + 4.0, "3 + Slime runSpeed");
        assert_eq!(p.acc_run_speed_value(), 3.0 + 4.0, "3 + Slime dashSpeed");
    }

    #[test]
    fn the_mount_jumps_replace_and_scale_with_x_velocity() {
        // DW-2.3 — `Player.cs:19619-19631` + `Mount.cs:2810-2858`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false); // Unicorn — no scaling branch
        p.vx = 16.0;
        p.update_mount(&w, false, None);
        assert_eq!(p.jump_height, 10);
        assert_eq!(p.jump_speed, 8.01);
        p.dismount(&w, false);
        p.set_mount(0, false); // Rudolph, type 0
        p.vx = 4.0;
        p.update_mount(&w, false, None);
        assert_eq!(p.jump_height, 18, "17 + (int)(4/4f)");
        assert_eq!(p.jump_speed, 5.31 + 4.0 / 7.0);
    }

    #[test]
    fn the_unicorn_arms_its_extra_jump_and_the_chain_consumes_it() {
        // DW-2.3 — `Mount.cs:4950-4953` arms the option; the PHYS-8 chain
        // (`Player.cs:20698-21022`) consumes it after the vanilla re-arm.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(10, false);
        // the ground jump
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        assert!(p.vy < 0.0, "the ground jump fired");
        // release mid-air (the else branch zeroes `jump` and latches the
        // release), then press again — the extra jump fires while airborne
        let release = PlayerInput::default();
        p.update(&w, release);
        p.update(&w, press);
        assert_eq!(
            p.performing_jump,
            Some(ExtraJump::Unicorn),
            "the mount jump consumed"
        );
        assert!(!p.can_jump_again[ExtraJump::Unicorn.index()]);
    }

    #[test]
    fn the_bunny_nullifies_fall_damage_and_the_unicorn_scales_it() {
        // DW-2.4 — `Player.cs:25315-25318` (bunny zero) + `:25363-25367`
        // (the FallDamage multiplier) + `:25307-25310` (CanFly zero).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(1, false); // Bunny — the nullify
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "the bunny nullifies the fall");
        // Rudolph can fly — `CanFly` zeroes the fall distance too
        p.dismount(&w, false);
        p.set_mount(0, false);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        p.hp = 100;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "the flying mount zeroes the fall");
        // Unicorn — 0.2x: (32 - 25) * 10 = 70 -> 14
        p.dismount(&w, false);
        p.set_mount(10, false);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        p.hp = 100;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        // The 76px unicorn hitbox rests with its top on row 55 (960-76=884),
        // so `num11` measures 55-25 = 30: (30-25)*10 = 50, x0.2 = 10.
        assert_eq!(p.hp, 90, "(50) * 0.2 = 10");
    }

    #[test]
    fn the_slime_extra_fall_lifts_the_threshold() {
        // `:25301-25304` + `Mount.cs:869` (extraFall 10): a 30-tile fall with
        // the Slime mount stays under `25 + 10`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(3, false);
        p.y = 30.0 * TILE_PX;
        p.fall_start = 30;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "35 threshold eats the 30-tile fall");
    }

    #[test]
    fn equipping_grows_the_hitbox_and_dismounting_restores_it() {
        // DW-2.5 — `Mount.cs:6264-6277` grow, `:6176-6193` restore.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let y_before = p.y;
        assert_eq!(p.height, HEIGHT);
        assert!(p.set_mount(10, false)); // Unicorn boost 34
        assert_eq!(p.height, HEIGHT + 34.0);
        assert!(
            (p.y - (y_before - 34.0)).abs() < 0.01,
            "the bottom stays anchored"
        );
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the mount buff added (:6226)"
        );
        assert!(p.dismount(&w, false));
        assert_eq!(p.height, HEIGHT);
        assert!((p.y - y_before).abs() < 0.01, "the restore returns the top");
        assert!(
            p.buffs.find_buff_index(162).is_none(),
            "the buff cleared (:6150)"
        );
    }

    #[test]
    fn cced_dismounts_the_mount_mid_tick() {
        // DW-2.6 — `Player.cs:26799-26812`: the Frozen buff CCs, the mount
        // dismounts during the same tick.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(10, false);
        p.buffs.add_buff(47, 100); // BuffID.Frozen
        p.update(&w, PlayerInput::default());
        assert!(!p.mount.is_active(), "CCed dismounts");
        assert_eq!(p.height, HEIGHT, "the hitbox restored");
    }

    #[test]
    fn the_auto_jump_refill_uses_just_jumped() {
        // `:26776-26781` — a constant-jump mount's re-jump refills the flight
        // budget (`autoJump && justJumped`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(50, false); // Queen Slime: constantJump + flightTimeMax 80
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        assert!(p.auto_jump, "the mount head-sets autoJump (:4797)");
        assert!(p.just_jumped, "the grounded trigger sets justJumped");
        assert!(p.mount.fly_time() > 0, "ResetFlightTime refilled (:26777)");
    }

    const EMPTY_ITEM_SLOT: crate::item::ItemSlot = crate::item::ItemSlot {
        kind: 0,
        name: "",
        count: 0,
        prefix: 0,
    };

    // -- M2-MOUNT-1 phase 3: triggers, buff lifecycle, quick-mount ----------

    fn saddle(kind: u16) -> crate::item::ItemSlot {
        crate::item::ItemSlot {
            kind,
            name: "",
            count: 1,
            prefix: 0,
        }
    }

    #[test]
    fn quick_mount_equips_the_first_saddle_and_the_buff_persists() {
        // DW-3.1 — `QuickMount` (`Player.cs:6045-6100`) + the refresh
        // (`:9881-9884`, `buffTime = 10`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let inv = [saddle(3260), EMPTY_ITEM_SLOT];
        assert_eq!(
            p.quick_mount(&w, &inv, &[], false),
            Some(3260),
            "the Blessed Apple equips"
        );
        assert!(p.mount.is_active() && p.mount.kind() == 10);
        assert_eq!(p.height, HEIGHT + 34.0, "the hitbox grew");
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the mount buff added (:6226)"
        );
        p.update(&w, PlayerInput::default());
        let i = p.buffs.find_buff_index(162).unwrap();
        assert_eq!(p.buffs.buff_time[i], 10, "the refresh holds buffTime at 10");
        assert!(p.mount.is_active(), "the buff keeps the mount alive");
    }

    #[test]
    fn the_buff_expiring_dismounts() {
        // DW-3.2 — `CheckMountBuff` (`Mount.cs:5275-5281` via `:18832`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false);
        assert!(p.mount.is_active());
        let i = p.buffs.find_buff_index(162).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert!(!p.mount.is_active(), "the buff gone -> dismount");
        assert_eq!(p.height, HEIGHT, "the hitbox restored");
    }

    #[test]
    fn dismount_on_item_use_respects_the_same_mount_exemption() {
        // DW-3.3 — the `DismountOnItemUse` predicate (`Mount.cs:642-652`);
        // the item gate itself is the editing.rs arm (`Player.cs:31695-31698`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false);
        assert!(
            !p.mount.dismount_on_item_use(),
            "no ground row sets the flag"
        );
        p.mount.set_test_dismount_on_item_use(Some(true));
        assert!(p.mount.dismount_on_item_use());
        // the exemption: the held item being the same mount does not clear
        let same = crate::mount::mount_type_of_item(3260) == Some(p.mount.kind());
        assert!(same, "Blessed Apple is the Unicorn's own saddle");
        // a different item dismounts through the vanilla gate
        assert!(p.try_dismount(&w));
        assert!(!p.mount.is_active());
    }

    #[test]
    fn quick_mount_toggles_and_guards() {
        // DW-3.4 — the toggle, the no-saddle fallback, the CC guard, the
        // space check (`Mount.CanMount`, `Mount.cs:6581-6604`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let none = [EMPTY_ITEM_SLOT];
        assert_eq!(p.quick_mount(&w, &none, &[], false), None, "no saddle -> nothing");
        let inv = [saddle(2430)]; // Slimy Saddle -> Slime 3
        assert_eq!(p.quick_mount(&w, &inv, &[], false), Some(2430));
        assert!(p.mount.is_active());
        assert_eq!(p.quick_mount(&w, &inv, &[], false), None, "mounted -> dismount");
        assert!(!p.mount.is_active());
        // the CC guard (`:6074-6077`)
        p.buffs.add_buff(47, 100); // Frozen
        p.update(&w, PlayerInput::default());
        assert_eq!(p.quick_mount(&w, &inv, &[], false), None, "CCed cannot mount");
    }

    #[test]
    fn can_mount_needs_the_grown_box_to_fit() {
        // `Mount.cs:6593-6595` — the Unicorn's 76px box does not fit under a
        // 3-tile ceiling where the Slime's 62px does.
        let mut w = ground_world();
        // A 64px cavity: solid rows 54-55, air rows 56-59, ground row 60.
        for x in 0..64 {
            for y in 54..56 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = Player::spawn(&w);
        p.y = 918.0; // resting on the row-60 ground inside the cavity
        assert!(!p.can_mount(&w, 10), "76px does not fit in the 64px cavity");
        assert!(p.can_mount(&w, 3), "62px fits");
    }

    #[test]
    fn s1_l_8_can_mount_applies_the_bat_and_grapple_sets() {
        // `Mount.cs:6595` (the bat wet rule) and `:6599-6601` (the
        // `Cart`/`CanUseHooks` grapple exemption).
        let mut w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // Dry and grounded: the bat (56) fits.
        assert!(p.can_mount(&w, 56));
        // `wet` blocks it; so does the Wet buff's `dripping` flag.
        p.wet = true;
        assert!(!p.can_mount(&w, 56), "the bat refuses water");
        p.wet = false;
        p.buffs.add_buff(103, 600); // `Player.cs:11743-11747`
        assert!(!p.can_mount(&w, 56), "dripping blocks the bat");
        p.clear_buff_type(103);
        // `Collision.WetCollision` on the `Bottom - size/2` probe: a puddle
        // in the feet row still blocks while the mid tile stays dry.
        let feet = ((p.y + p.height) / TILE_PX) as i32;
        let col = ((p.x + WIDTH / 2.0) / TILE_PX) as i32;
        let i = w.index(col, feet);
        w.tiles[i].liquid = 255;
        assert!(!p.wet, "the body itself is dry");
        assert!(!p.can_mount(&w, 56), "the wet probe reaches the feet row");
        w.tiles[i].liquid = 0;
        assert!(p.can_mount(&w, 56));
        // A live grapple refuses the ordinary mounts but not the carts or the
        // CanUseHooks set.
        p.grappling_active = true;
        assert!(!p.can_mount(&w, 0), "Rudolph refuses a live grapple");
        assert!(p.can_mount(&w, 6), "the minecart keeps the grapple");
        assert!(p.can_mount(&w, 13), "the wood cart keeps it");
        assert!(p.can_mount(&w, 53), "the terra fart cart keeps it");
        assert!(crate::mount::can_use_hooks(54));
        assert!(crate::mount::can_use_hooks(57) && crate::mount::can_use_hooks(60));
        assert!(!crate::mount::can_use_hooks(56));
        assert!(!crate::mount::is_cart(56));
    }

    #[test]
    fn s1_l_8_fishron_halves_run_stats_outside_the_special() {
        // `Player.cs:26655-26659` with the getter at `:4044-4056`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(12, false); // Cute Fishron
        p.hp = p.max_hp;
        p.mount_fishron_special_counter = 0.0;
        p.update_mount(&w, false, None);
        let row = crate::mount::mount_data(12).unwrap();
        assert_eq!(p.max_run_speed_value(), row.run_speed / 2.0);
        assert_eq!(p.run_acceleration, row.acceleration / 2.0);
        // Hurt (`hp <= statLifeMax2 / 2`) arms the 60-tick window: no halving.
        p.hp = p.max_hp / 2;
        p.update_mount(&w, false, None);
        assert_eq!(p.mount_fishron_special_counter, 60.0);
        assert_eq!(p.max_run_speed_value(), row.run_speed);
        // The window survives the tick's one-point decay (`:25625-25628`).
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            &[],
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(p.mount_fishron_special_counter > 0.0);
        // The full tick leaves the mount in its air frame; the `+13.5` arm
        // (`Mount.cs:461-464`) would mask the halving probe, so put it back
        // on the ground state the wading case runs in.
        p.mount.test_set_frame_state(crate::mount::FRAME_STANDING);
        // Wading re-arms the 420-tick window (`Mount.cs:5005-5008`).
        p.mount_fishron_special_counter = 0.0;
        p.hp = p.max_hp;
        p.wet = true;
        p.update_mount(&w, false, None);
        assert_eq!(p.mount_fishron_special_counter, 420.0);
        assert_eq!(p.max_run_speed_value(), row.run_speed);
    }

    #[test]
    fn the_tr_mount_pin_equips_and_a_real_saddle_wins() {
        // DW-3.5 — the pin re-applies after the flag clear; a real mount
        // (or its buff) suppresses it, matching the refresh order.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_mount = Some(10);
        p.update(&w, PlayerInput::default());
        assert!(
            p.mount.is_active() && p.mount.kind() == 10,
            "the pin equipped"
        );
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the buff sustains it"
        );
        // a real saddle overrides the pin while it is live
        p.set_mount(3, false);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mount.kind(), 3, "the real saddle wins");
        // removing the saddle hands control back to the pin
        let i = p.buffs.find_buff_index(130).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mount.kind(), 10, "the pin re-equipped after the clear");
    }

    // -- M2-MOUNT-1 phase 4: the frame animator ------------------------------

    #[test]
    fn the_queen_slime_run_counter_advances_at_half_speed() {
        // DW-4.1 — `Mount.cs:4277-4279` (type 50: `|vx| * 0.5`) with the
        // 16-tick delay and the 5-frame wrap (`:2194-2244` row).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(50, false);
        p.vx = 4.0;
        // 2.0 per tick, one crossing per tick — 8 ticks per frame; the first
        // crossing lands on tick 9, the second on 17.
        for _ in 0..17 {
            p.update_frames(&w);
        }
        assert_eq!(p.mount.frame_state(), crate::mount::FRAME_RUNNING);
        assert_eq!(p.mount.frame(), 2, "two steps in 17 ticks");
        // crossings at 25/33/41: the fourth step exits the 5-frame band...
        for _ in 0..(8 * 3) {
            p.update_frames(&w);
        }
        assert_eq!(p.mount.frame(), 0, "wrapped back into the running band");
    }

    #[test]
    fn the_rudolph_idle_pick_is_deterministic_and_reaches_the_idle_band() {
        // DW-4.2 — the idle timer (`Mount.cs:4145-4181`) through the
        // splitmix stand-in: same tick sequence, same picks.
        let w = ground_world();
        let mut a = Player::spawn(&w);
        let mut b = Player::spawn(&w);
        a.set_mount(0, false);
        b.set_mount(0, false);
        let mut reached_idle = false;
        for tick in 0..2500 {
            a.update_frames(&w);
            b.update_frames(&w);
            assert_eq!(a.mount.frame(), b.mount.frame(), "tick {tick}: same pick");
            assert!(a.mount.frame() != 1, "standing still never runs");
            if a.mount.frame() >= 2 && a.mount.frame() <= 5 {
                reached_idle = true;
            }
        }
        assert!(reached_idle, "the idle band (frames 2-5) was reached");
    }

    #[test]
    fn the_air_state_picks_the_in_air_row_and_rudolph_reaches_fly_frames() {
        // DW-4.3 — `Player.cs:36155-36216`: the in-air row right after the
        // jump, then the flying band while the budget lasts.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(1, false); // Bunny: in-air start row 5
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        let release = PlayerInput::default();
        p.update(&w, release);
        assert_eq!(p.mount.frame_state(), crate::mount::FRAME_IN_AIR);
        assert_eq!(p.mount.frame(), 5, "the bunny's in-air row");
        // Rudolph: the ground refill arms the budget, the held jump after the
        // jump budget runs out switches to the flying band (start row 6).
        p.dismount(&w, false);
        p.set_mount(0, false);
        for _ in 0..240 {
            p.update(&w, release); // settle back down; rest ticks refill
            if p.on_ground && p.mount.fly_time() > 0 {
                break;
            }
        }
        assert!(p.on_ground && p.mount.fly_time() > 0, "the refill armed");
        p.update(&w, press); // the ground jump
        for _ in 0..30 {
            p.update(&w, press); // hold: the jump budget (17) runs out
        }
        assert_eq!(
            p.mount.frame_state(),
            crate::mount::FRAME_FLYING,
            "fly time drives state 3"
        );
        assert!((6..=11).contains(&p.mount.frame()), "the flying band");
    }

    #[test]
    fn the_walking_grace_stays_inert_for_the_ground_rows() {
        // DW-4.4 — `Mount.cs:3620-3631`: every ground row carries
        // `walkingGraceTimeMax = 0`, so the grace never blocks the air state
        // and never goes negative.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(1, false);
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        let mut airborne_ticks = 0;
        for _ in 0..40 {
            p.update(&w, press);
            assert_eq!(p.mount.running_grace_time(), 0, "max 0 -> always 0");
            if p.vy != 0.0 {
                airborne_ticks += 1;
                assert_eq!(
                    p.mount.frame_state(),
                    crate::mount::FRAME_IN_AIR,
                    "the grace never blocks the air state"
                );
            }
        }
        assert!(airborne_ticks > 10, "the hop stayed airborne");
    }

    /// A 40x120 world with a dirt floor at y 20 — the sit/sleep rigs (the
    /// height clears `SolidCollision`'s `maxTilesY - 40` bottom clamp).
    fn sit_world() -> World {
        let mut w = World::new("t".into(), 40, 120);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_sit_state_exits_on_a_facing_flip_or_a_gone_seat() {
        // DW-1.1 — `PlayerSittingHelper.UpdateSitting` (`:50-87`): the seat
        // re-decode at `Bottom + (0, -2)` (`:56-61`), the input mask
        // (`:62-65` — `controlUp` rides the port's merged jump binding) and
        // the facing mismatch stand the player up.
        let mut w = sit_world();
        // The chair's BOTTOM cell at (10, 19) — one row above the floor —
        // with `frameY % 40 == 18`: the decode reads the row above and
        // anchors the seat to this cell's bottom edge — `(168 - 6, 320)`
        // for the left-facing frame.
        let i = w.index(10, 19);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 15,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = Player::spawn(&w);
        p.sit_down(&w, 162.0, 320.0, -1, (0.0, 0.0));
        assert!(p.sitting);
        // The seat re-decode still resolves (the mask holds) — the state
        // survives a quiet tick.
        p.update_sitting(&w);
        assert!(p.sitting);
        // The facing mismatch (`targetDirection != player.direction`).
        p.facing = 1;
        p.update_sitting(&w);
        assert!(!p.sitting);
        // The seat tile going away fails the decode → stand up.
        p.sit_down(&w, 162.0, 320.0, -1, (0.0, 0.0));
        assert!(p.sitting);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        p.update_sitting(&w);
        assert!(!p.sitting);
    }

    #[test]
    fn the_sleep_state_arms_the_rotation_and_wakes_on_input() {
        // DW-1.5 — `PlayerSleepingHelper`'s `StartSleeping` (`:155-190`),
        // `UpdateState` (`:85-138`) and `StopSleeping` (`:140-153`): the
        // anchor snap, the π/2 rotation (`:73-74`), the fully-asleep counter
        // with the ActUp resets (`:45-64`) and the input wake (`:103-106`).
        let mut w = sit_world();
        // The bed's BOTTOM cell at (10, 19) — one row above the floor —
        // style 0 (`frameY 18`): the anchor is column `10 - 18/18 + 1 = 10`,
        // row `19` → `(168, 320)`.
        let bed = w.index(10, 19);
        w.tiles[bed] = tr_core::tile::Tile {
            kind: 79,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[bed].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = Player::spawn(&w);
        p.sleep_down(&w, 168.0, 320.0, -1, 0);
        assert!(p.sleeping);
        assert_eq!(p.sleeping_style, 0);
        assert_eq!(p.facing, -1);
        assert_eq!(p.vx, 0.0);
        assert_eq!(p.vy, 0.0);
        // `fullRotation = PI/2 * (-direction)` = +π/2, origin `Size / 2`.
        assert!((p.full_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(p.full_rotation_origin, (10.0, 21.0));
        // The counter climbs while quiet and resets on an item swing.
        p.update_sleeping(&w, false, false);
        assert_eq!(p.sleep_time, 1);
        p.update_sleeping(&w, true, false);
        assert_eq!(p.sleep_time, 0);
        for _ in 0..120 {
            p.update_sleeping(&w, false, false);
        }
        assert_eq!(p.sleep_time, 120);
        // The blood-moon night keeps resetting the counter (`:51-54`).
        w.blood_moon = true;
        w.day_time = false;
        p.update_sleeping(&w, false, false);
        assert_eq!(p.sleep_time, 0);
        assert!(p.sleeping);
        w.blood_moon = false;
        // The input mask wakes (`:103-106`).
        p.control_jump = true;
        p.update_sleeping(&w, false, false);
        assert!(!p.sleeping);
        assert_eq!(p.sleep_time, 0);
        assert_eq!(p.full_rotation, 0.0);
        // The bed going away wakes too (`:97-102`).
        p.sleep_down(&w, 168.0, 320.0, -1, 0);
        assert!(p.sleeping);
        w.tiles[bed].flags &= !tr_core::tile::FLAG_ACTIVE;
        p.update_sleeping(&w, false, false);
        assert!(!p.sleeping);
    }

    /// A minimal valid NPC room (the `housing` tests' shape): the wood
    /// frame, the background walls everywhere, the interior air ≥ the
    /// 60-tile room minimum.
    fn room_box(w: &mut World, x0: i32, y0: i32, rw: i32, rh: i32) {
        for x in x0..x0 + rw {
            for y in y0..y0 + rh {
                let i = w.index(x, y);
                w.tiles[i].wall = 27;
                let border = x == x0 || x == x0 + rw - 1 || y == y0 || y == y0 + rh - 1;
                if border {
                    w.tiles[i] = tr_core::tile::Tile {
                        kind: 9,
                        frame_x: -1,
                        frame_y: -1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
    }

    #[test]
    fn the_load_rebuilds_the_spawn_from_a_valid_bed() {
        // DW-3.2 — `FindSpawn` (`Player.cs:54592-54609`) over the port's
        // world-side bed spawn: a validated bed cell overrides the world
        // spawn; a destroyed bed falls back (the silent load-time arm).
        let mut w = sit_world();
        // The room sits on the floor row (20): the frame rows 12..20, the
        // interior 13..19. The bed's bottom cell rides (20, 19) and the
        // spawn cell = (20, 20) — the box floor.
        room_box(&mut w, 14, 12, 12, 9);
        let bed = w.index(20, 19);
        w.tiles[bed] = tr_core::tile::Tile {
            kind: 79,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[bed].flags |= tr_core::tile::FLAG_ACTIVE;
        w.bed_spawn = Some((20, 20));
        let p = Player::spawn(&w);
        assert_eq!(p.spawn_x, 20.0 * TILE_PX + 8.0 - WIDTH * 0.5);
        assert_eq!(p.spawn_y, 20.0 * TILE_PX - HEIGHT);
        // The bed gone → the world spawn (`SpawnX = -1`'s fallback).
        w.tiles[bed].flags &= !tr_core::tile::FLAG_ACTIVE;
        let p = Player::spawn(&w);
        assert_eq!(p.spawn_x, w.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5);
    }

    #[test]
    fn the_snap_refuses_a_solid_box_and_respawn_clears_the_states() {
        // `Player.CanSnapToPosition` (`Player.cs:33401-33403`) and `Spawn`'s
        // `StopVanityActions` (`:37556`) via `respawn`.
        let mut w = sit_world();
        let mut p = Player::spawn(&w);
        p.x = 160.0;
        p.y = 278.0; // the box spans rows 17..20 at column 10 — all air
        assert!(p.can_snap_to(&w, 0.0, 0.0));
        let i = w.index(10, 19);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(!p.can_snap_to(&w, 0.0, 0.0));
        // The respawn clears the sit/sleep state (`Player.cs:37556`).
        p.sitting = true;
        p.sleeping = true;
        p.full_rotation = 1.5;
        p.respawn();
        assert!(!p.sitting);
        assert!(!p.sleeping);
        assert_eq!(p.full_rotation, 0.0);
    }
    // ---- S1-C-1: the dodge family (`Player.cs:38214-38241` + the fold) ----

    /// The scripted stream the dodge-roll tests drive: `rolls` feed
    /// `next_range` in order (exhausted -> the largest in-range value, which
    /// never rolls `== 0`), `draws` records every call so the S1-A-10
    /// draw-order contract is asserted directly.
    struct DodgeRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl DodgeRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self {
                rolls: rolls.to_vec(),
                draws: Vec::new(),
            }
        }
    }
    impl tr_core::loot::DropRng for DodgeRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            if self.rolls.is_empty() {
                return max - 1;
            }
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.5
        }
    }

    /// A ground player with `items` in the accessory slots, the fold run
    /// twice (`player_with_accessory`'s multi-source form).
    fn dodge_player_with(items: &[u16]) -> Player {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for (i, &kind) in items.iter().enumerate() {
            p.equipped[3 + i] = kind;
        }
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    /// Runs the dodge window out the way `UpdateImmunity` does in the C# —
    /// the dodge's `SetImmuneTimeForAllTypes` writes `immune`/`immuneTime`
    /// (R-03-5's ported fields, now gating `hurt_inner`'s General flag), so a
    /// next hit in these tests needs the expired window, not just a cleared
    /// `hurt_cd`.
    fn expire_dodge_window(p: &mut Player) {
        p.immune = false;
        p.immune_time = 0;
        p.hurt_cd = 0.0;
    }

    #[test]
    fn s1_c_1_dw_1_1_dodge_flags_arm_from_items_and_buff_and_clear() {
        // `ApplyEquipFunctional`: 963 (`:15060`) and 984 (`:15064`) arm
        // `blackBelt`, 6189 (`:14633`) arms `mysticSashDodge`, 3223
        // (`:15087`) arms the brain; `ResetEffects` clears them per frame
        // (`:18661-18662`/`:18617`) and the fold re-arms from the live
        // sources.
        let p = dodge_player_with(&[963]);
        assert!(p.black_belt);
        let p = dodge_player_with(&[984]);
        assert!(p.black_belt, "the Master Ninja Gear arms the same flag");
        let p = dodge_player_with(&[6189]);
        assert!(p.mystic_sash_dodge);
        let p = dodge_player_with(&[3223]);
        assert!(p.brain_of_confusion);
        let p = dodge_player_with(&[0]);
        assert!(!p.black_belt && !p.mystic_sash_dodge && !p.brain_of_confusion);
        // Buff 59 arms `shadowDodge` through the buff scan
        // (`UpdateBuffs`, `:10075-10077`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(!p.shadow_dodge);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge, "buff 59 arms the shadow dodge");
        let i = p.buffs.find_buff_index(59).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert!(!p.shadow_dodge, "the fold clears it without the buff");
        // The fold-order half: armed while equipped, cleared by the next
        // fold without the source.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 963;
        p.update(&w, PlayerInput::default());
        assert!(p.black_belt);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.black_belt, "the next fold without the source clears it");
    }

    #[test]
    fn s1_c_1_dw_1_2_sash_roll_dodges_on_next_10_zero() {
        // `mysticSashDodge && Main.rand.Next(10) == 0` (`:38215-38219`).
        let mut p = dodge_player_with(&[6189]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            !p.hurt(20, p.x + 100.0, &mut rng, true),
            "the sash dodge fires"
        );
        assert_eq!(p.hp, hp, "no damage on the dodge");
        assert_eq!(rng.draws, vec![(0, 10)]);
        // A missed roll lands the hit through the same source.
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_dw_1_2_belt_roll_dodges_on_next_10_zero() {
        // `blackBelt && Main.rand.Next(10) == 0` -> `NinjaDodge()`
        // (`:38220-38223`).
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            !p.hurt(20, p.x + 100.0, &mut rng, true),
            "the belt dodge fires"
        );
        assert_eq!(p.hp, hp);
        assert_eq!(rng.draws, vec![(0, 10)]);
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_dw_1_2_brain_roll_gates_on_armed_flag_and_buff_321() {
        // `brainOfConfusionItem != null && !IsAir && Main.rand.Next(6) == 0
        // && FindBuffIndex(321) == -1` (`:38224-38227`): the armed flag folds
        // the two item checks into one bool, and the roll PRECEDES the
        // buff-321 check in the C# `&&` chain — the draw happens while armed
        // even with 321 active, the buff only gates the dodge itself.
        let mut p = dodge_player_with(&[3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            !p.hurt(20, p.x + 100.0, &mut rng, true),
            "the brain dodge fires"
        );
        assert_eq!(rng.draws, vec![(0, 6)]);
        // The dodge armed buff 321 itself (240 ticks): the next hit roll is
        // still drawn, but no dodge comes out of it.
        assert!(p.buffs.find_buff_index(321).is_some());
        expire_dodge_window(&mut p);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            p.hurt(20, p.x + 100.0, &mut rng, true),
            "buff 321 gates the dodge"
        );
        assert_eq!(
            rng.draws,
            vec![(0, 6)],
            "the draw happened anyway (C# order)"
        );
        assert!(p.hp < hp);
        // Unarmed: a would-be-hitting roll draws nothing at all.
        let mut p = dodge_player_with(&[0]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "no brain draw without the item");
    }

    #[test]
    fn s1_c_1_dw_1_2_shadow_dodges_without_a_roll() {
        // `shadowDodge -> ShadowDodge()` is deterministic (`:38229-38232`) —
        // no `Next` draw at all.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "the shadow dodge draws nothing");
        assert_eq!(p.hp, hp);
    }

    #[test]
    fn s1_c_1_dw_1_6_draw_order_sash_belt_brain_when_nothing_hits() {
        // The S1-A-10 draw-order contract, asserted on the recorded stream:
        // all three rolled sources armed + a stream that never hits draws
        // exactly sash -> belt -> brain, one `Next` each, in that order
        // (`:38215-38227`); shadow (not armed here) draws nothing.
        let mut p = dodge_player_with(&[6189, 963, 3223]);
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(
            p.hurt(20, p.x + 100.0, &mut rng, true),
            "no dodge, the hit lands"
        );
        assert_eq!(rng.draws, vec![(0, 10), (0, 10), (0, 6)]);
    }

    #[test]
    fn s1_c_1_dw_1_6_first_source_hit_consumes_no_later_draw() {
        // The early return short-circuits: a sash hit consumes no belt/brain
        // draw (`:38217-38218`).
        let mut p = dodge_player_with(&[6189, 963, 3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(rng.draws, vec![(0, 10)]);
    }

    #[test]
    fn s1_c_1_dw_1_3_dodge_sets_the_80_tick_window_and_leaves_hp() {
        // The handlers' `SetImmuneTimeForAllTypes(longInvince ? 120 : 80)`
        // (`:38013-38021`, `:38025` et al.) — the dodge pair writes the
        // collapsed General cooldown directly, NOT `iframes_for`'s 40/20
        // normal-hurt window.
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(50, p.x + 100.0, &mut rng, true));
        assert_eq!(p.hp, hp);
        assert!(
            (p.hurt_cd - 80.0 / 60.0).abs() < 1e-4,
            "the 80-tick dodge window"
        );
        // `longInvince` lifts the pair to 120.
        expire_dodge_window(&mut p);
        p.long_invince = true;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(50, p.x + 100.0, &mut rng, true));
        assert!(
            (p.hurt_cd - 120.0 / 60.0).abs() < 1e-4,
            "the 120-tick window"
        );
    }

    #[test]
    fn s1_c_1_dw_1_3_brain_dodge_bookkeeping_runs_exactly_once() {
        // `BrainOfConfusionDodge` (`:38046-38055`): the 300 counter, ONE
        // buff-321 add (240 ticks), the 120/80 window.
        let mut p = dodge_player_with(&[3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 300);
        let count = p.buffs.active_types().iter().filter(|&&b| b == 321).count();
        assert_eq!(count, 1, "one buff-321 add");
        let i = p.buffs.find_buff_index(321).unwrap();
        assert_eq!(p.buffs.buff_time[i], 240);
        assert!((p.hurt_cd - 80.0 / 60.0).abs() < 1e-4);
    }

    #[test]
    fn s1_c_1_dw_1_3_shadow_dodge_bookkeeping_runs_exactly_once() {
        // `ShadowDodge` (`:38023-38042`): buff 59 deleted, the Hallowed
        // cooldown set to 1800, the 120/80 window.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge);
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(p.buffs.find_buff_index(59), None, "buff 59 deleted");
        assert_eq!(p.shadow_dodge_timer, 1800);
        assert!((p.hurt_cd - 80.0 / 60.0).abs() < 1e-4);
    }

    #[test]
    fn s1_c_1_dw_1_4_no_draw_no_dodge_while_hurt_cd_live() {
        // The cooldown early-return (`:38207-38213`) precedes the roll block:
        // with `hurt_cd` live no `Next` is drawn and no dodge fires (the C#
        // `flag` gate). Dirty test: the hitting stream would dodge if the
        // block ran.
        let mut p = dodge_player_with(&[963]);
        p.hurt_cd = 1.0;
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "no draw while the cooldown is live");
        assert_eq!(p.hp, hp);
        assert!((p.hurt_cd - 1.0).abs() < 1e-6, "the cooldown untouched");
    }

    #[test]
    fn s1_c_1_dw_1_5_dodge_short_circuits_before_defense_endurance_and_iframes() {
        // The roll block sits after the cooldown gate and before the defense
        // reduction (`Player.cs:38214` > `:38304`): a dodged lethal hit
        // leaves hp/regen/knockback untouched and carries the DODGE window
        // (80/60), not `iframes_for`'s 40/60 — the early return proves the
        // defense/endurance/immunity steps never ran.
        let mut p = dodge_player_with(&[963]);
        p.hp = 5;
        p.endurance = 0.5;
        p.stat_defense = 100;
        let regen_time = p.regen.life_regen_time;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            !p.hurt(50, p.x + 100.0, &mut rng, true),
            "the belt eats the lethal hit"
        );
        assert_eq!(p.hp, 5, "no damage, no death pipeline");
        assert!(
            (p.hurt_cd - 80.0 / 60.0).abs() < 1e-4,
            "the dodge window, not the 40-tick hurt window"
        );
        assert_eq!(
            p.regen.life_regen_time, regen_time,
            "the regen reset did not run"
        );
        assert_eq!(p.vx, 0.0, "no knockback on the dodge");
    }

    #[test]
    fn s1_c_1_godmode_returns_before_the_dodge_rolls() {
        // `creativeGodMode` returns before the roll block (`:38205-38207`) —
        // an armed belt draws nothing under the power.
        let mut p = dodge_player_with(&[963]);
        p.creative_god_mode = true;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "godmode precedes the rolls");
        assert_eq!(p.hurt_cd, 0.0, "no immunity window written");
    }

    #[test]
    fn s1_c_1_no_rolls_when_dodgeable_is_false() {
        // `Hurt`'s `dodgeable` parameter gates the whole block (`:38214`):
        // false -> no draw, no dodge. Phase 2 threads the real
        // `IsDamageDodgeable` values through this parameter.
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, false));
        assert!(rng.draws.is_empty());
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_a_dodged_lava_hit_skips_the_on_fire_grant() {
        // The lava site (`update_survival`, `Player.cs:27681-27722`): the
        // OnFire grant rides the `Hurt` return (`:27710-27711`) — a dodge
        // returns false, so no buff 24; the counterpart roll lands and
        // grants, proving the UpdatePlayers stream reaches the lava hurt.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 963;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p.in_lava = true;
        p.lava_time = 0;
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        p.update_survival(&w, &mut rng);
        assert_eq!(p.hp, hp, "the belt ate the lava hit");
        assert!(
            p.buffs.find_buff_index(24).is_none(),
            "no OnFire on a dodge"
        );
        // The counterpart: a missed roll lands the 80 damage and grants.
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        p.update_survival(&w, &mut rng);
        assert!(p.hp < hp);
        assert!(
            p.buffs.find_buff_index(24).is_some(),
            "OnFire on the landed hit"
        );
    }

    #[test]
    fn s1_c_1_the_shadow_dodge_timer_decays_one_per_tick() {
        // `Player.Update`'s decay (`:25461-25464`), riding the tick head.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.shadow_dodge_timer = 1800;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.shadow_dodge_timer, 1799);
        p.shadow_dodge_timer = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.shadow_dodge_timer, 0, "no underflow");
    }

    #[test]
    fn s1_c_1_the_brain_counter_decays_and_clears_on_respawn() {
        // The fold decay (`ResetEffects`, `:18794-18797`) and `UpdateDead`'s
        // zero (`:16932`, the port's `respawn`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.brain_of_confusion_dodge_animation_counter = 300;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 299);
        p.respawn();
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 0);
    }

    // -- S1-P-i41: mount item use + data witnesses ---------------------------

    #[test]
    fn s1_p_i41_the_player_size_delegates_resize_the_hitbox_bottom_center() {
        // `Mount.SetMount`'s PlayerSize branch (`Mount.cs:6244-6263`) + the
        // three delegates (`DelegateMethods.cs:164-194`): Rat 14×14, Bat
        // 20×18, Pixie 8×14 — bottom-center anchored, NOT 42+heightBoost.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let bottom_x = p.x + p.width * 0.5;
        let bottom_y = p.y + p.height;
        // The Pixie (61) shrinks to 8×14.
        assert!(p.set_mount(61, false));
        assert_eq!((p.width, p.height), (8.0, 14.0));
        assert!((p.x - (bottom_x - 4.0)).abs() < 1e-4, "bottom-center X");
        assert!((p.y - (bottom_y - 14.0)).abs() < 1e-4, "bottom-center Y");
        // The buff is granted (`:6226`).
        assert!(p.buffs.find_buff_index(384).is_some());
        // Dismount restores 20×42 (`Mount.cs:6176-6193`).
        assert!(p.dismount(&w, false));
        assert_eq!((p.width, p.height), (WIDTH, HEIGHT));
        // The Rat (55) is 14×14; the Bat (56) is 20×18.
        assert!(p.set_mount(55, false));
        assert_eq!((p.width, p.height), (14.0, 14.0));
        p.dismount(&w, false);
        assert!(p.set_mount(56, false));
        assert_eq!((p.width, p.height), (20.0, 18.0));
        p.dismount(&w, false);
        // A non-delegate mount still grows by heightBoost (Unicorn 34).
        assert!(p.set_mount(10, false));
        assert_eq!((p.width, p.height), (WIDTH, HEIGHT + 34.0));
    }

    #[test]
    fn s1_p_i41_the_cc_set_refuses_the_transformation_dismount() {
        // `Mount.CanDismountWithResult` (`Mount.cs:6611-6624`): the
        // `DontDismountWhenCCed` set {55,56,61} (`MountID.cs:47`) refuses
        // while CCed; the buff-removal path then DelBuff's the slot anyway
        // (`Main.cs:43383-43386`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(55, false)); // the Rat
        assert!(crate::mount::dont_dismount_when_cced(55));
        assert!(!crate::mount::dont_dismount_when_cced(10));
        // Frozen (47) → CCed.
        p.frozen = true;
        assert_eq!(
            p.can_dismount_with_result(&w),
            crate::mount::DismountCheck::FailedCCed
        );
        assert!(!p.try_dismount(&w), "CCed refuses the dismount");
        assert!(p.mount.is_active(), "the mount stays up");
        // The buff-bar removal: `flag` latches on the CheckBuff match
        // (`Main.cs:43378`), so even a FAILED TryDismount keeps the slot
        // (M3 sync onto the landed i53 form — DelBuff never runs on the
        // mount arm).
        let slot = p.buffs.find_buff_index(374).unwrap();
        assert!(p.try_removing_buff(&w, slot, 374, [0, 0]));
        assert!(p.buffs.find_buff_index(374).is_some(), "the mount arm keeps the buff");
        assert!(p.mount.is_active(), "the mount survived the failed dismount");
        // Un-CCed: the dismount succeeds.
        p.frozen = false;
        assert!(p.try_dismount(&w));
        assert!(!p.mount.is_active());
    }

    #[test]
    fn s1_p_i41_the_velociraptor_borrows_the_wing_budget() {
        // `Mount.CanFly`'s type-54 branch (`Mount.cs:2866-2869`) +
        // `UpdateAfterEquips` (`:2989-2992`) + `ResetFlightTime` (`:5269-5272`).
        let mut m = crate::mount::MountState::default();
        m.set_mount(54, false);
        // No wings → not allowed to fly (flightTimeMax is 0).
        assert!(!m.can_fly());
        m.update_selective_flight(0, false, 0);
        assert!(!m.can_fly());
        // Wings equipped → allowed, and the budget borrows wingTimeMax.
        m.update_selective_flight(22, false, 42);
        assert!(m.can_fly());
        m.reset_flight_time(0.0, 42);
        assert_eq!(m.fly_time(), 42, "the raptor borrows the wing budget");
        // The empress brooch refills the budget every tick (`:2993-2996`).
        m.flight();
        assert_eq!(m.fly_time(), 41);
        m.update_selective_flight(22, true, 42);
        assert_eq!(m.fly_time(), 42, "the brooch refills");
        // A non-54 mount ignores the selective permission.
        m.dismount();
        m.set_mount(10, false);
        m.update_selective_flight(22, true, 42);
        assert!(!m.can_fly(), "the Unicorn has flightTimeMax 0");
    }

    #[test]
    fn s1_p_i41_the_water_mount_buffs_grant_their_swim_flags() {
        // `Player.UpdateBuffs`' mount sub-arms (`Player.cs:9885-9912`): the
        // five water buffs grant ignoreWater/accFlipper (131/168/279),
        // canFloatInWater/accFlipper (265), the full lava suite (305).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // The Turtle (buff 131): ignoreWater + accFlipper.
        p.buffs.add_buff(131, 3600);
        p.update(&w, PlayerInput::default());
        assert!(p.ignore_water && p.acc_flipper);
        assert!(p.mount.is_active() && p.mount.kind() == 4);
        assert!(!p.can_float_in_water);
        // The Water Strider (buff 265): canFloatInWater + accFlipper.
        p.buffs.add_buff(265, 3600);
        p.update(&w, PlayerInput::default());
        assert!(p.can_float_in_water && p.acc_flipper);
        // The Lava Shark (buff 305): the full lava suite.
        p.buffs.add_buff(305, 3600);
        p.update(&w, PlayerInput::default());
        assert!(p.ignore_water && p.acc_flipper);
        assert!(p.lava_immune && p.lava_vision && p.fire_walk);
        // ShouldFloatInWater (`:3872-3885`): the Strider's canFloatInWater
        // grant floats unmounted; mounted, only type 37 does.
        p.dismount(&w, false);
        p.can_float_in_water = true;
        p.control_down = false;
        assert!(p.should_float_in_water());
        p.control_down = true;
        assert!(!p.should_float_in_water(), "holding down sinks");
    }

    #[test]
    fn s1_p_i41_the_dismount_restores_no_items_unless_cursed() {
        // `Mount.Dismount`'s noItems restore (`Mount.cs:6152-6155`): the Rat
        // (55) / Bat (56) free the item lock unless Cursed keeps it.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(55, false));
        p.no_items = true;
        p.cursed = false;
        p.dismount(&w, false);
        assert!(!p.no_items, "the Rat dismount frees the lock");
        // Cursed keeps it.
        assert!(p.set_mount(56, false));
        p.no_items = true;
        p.cursed = true;
        p.dismount(&w, false);
        assert!(p.no_items, "Cursed keeps the lock");
        // A non-transformation mount never touches noItems.
        assert!(p.set_mount(10, false));
        p.no_items = true;
        p.cursed = false;
        p.dismount(&w, false);
        assert!(p.no_items, "the Unicorn leaves the lock alone");
    }

    #[test]
    fn s1_p_i41_the_cursed_buff_locks_item_use_with_the_mount_exemption() {
        // `Player.cs:11731-11735` (Cursed 23 → noItems+cursed) + the
        // `ItemCheck_CheckCanUse_Inner` exemption (`:52140-52148`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(23, 3600);
        p.update(&w, PlayerInput::default());
        assert!(p.no_items && p.cursed);
        // No mount: the lock refuses (ignoreCursed = false).
        assert!(!p.can_use_item_past_no_items(&w, false));
        // ignoreCursed = true bypasses.
        assert!(p.can_use_item_past_no_items(&w, true));
        // A dismounts-on-item-use mount that can dismount widens the gate —
        // but Cursed keeps it locked (`!this.cursed` conjunct).
        p.set_mount(55, false);
        assert!(!p.can_use_item_past_no_items(&w, false), "cursed keeps it");
        // Drop the cursed flag (keep noItems): the exemption opens.
        p.cursed = false;
        assert!(p.can_use_item_past_no_items(&w, false));
    }

    #[test]
    fn s1_p_i41_quick_mount_skips_cart_rows_and_reads_misc_equip_3() {
        // `QuickMount_GetItemToUse` (`Player.cs:6179-6198`): miscEquips[3]
        // first (non-cart), then the inventory skipping cart rows.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // A cart item in the inventory is skipped; the saddle behind it wins.
        let inv = [saddle(2343), saddle(3260)]; // Minecart 6, then Unicorn 10
        assert_eq!(p.quick_mount(&w, &inv, &[], false), Some(3260));
        assert_eq!(p.mount.kind(), 10);
        p.dismount(&w, false);
        // miscEquips[3] wins over the inventory.
        let misc = [EMPTY_ITEM_SLOT, EMPTY_ITEM_SLOT, EMPTY_ITEM_SLOT, saddle(2430)];
        assert_eq!(p.quick_mount(&w, &inv, &misc, false), Some(2430));
        assert_eq!(p.mount.kind(), 3);
    }

    #[test]
    fn s1_p_i41_quick_mount_refuses_a_dismounts_on_use_mount_mid_use() {
        // `Player.QuickMount` (`Player.cs:6082-6085`): a dismounts-on-item-use
        // mount refuses to equip while `TryingToUseItem()` is true.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let inv = [saddle(5525)]; // the Rat (55), dismountsOnItemUse
        assert_eq!(p.quick_mount(&w, &inv, &[], true), None, "mid-use refuses");
        assert!(!p.mount.is_active());
        assert_eq!(p.quick_mount(&w, &inv, &[], false), Some(5525));
        assert!(p.mount.is_active());
    }

    #[test]
    fn s1_p_i41_the_no_space_ladder_counts_three_strikes_then_teleports() {
        // `Player.QuickMount`'s FailedNoSpace ladder (`Player.cs:6049-6070`):
        // two refused dismounts queue the tink, the third forces the dismount
        // and the no-space teleport. `UpdateFailedDismount` (`:6037-6043`)
        // clears the count on a moved tile.
        let mut w = ground_world();
        let mut p = Player::spawn(&w);
        // Force a FailedNoSpace: a Pixie (8 wide) restoring to 20 in a slot
        // too narrow — build walls hugging the body.
        assert!(p.set_mount(61, false));
        // Wall the sides so the 8→20 width growth has no room.
        let col_l = ((p.x - 16.0) / TILE_PX) as i32;
        let col_r = ((p.x + p.width + 16.0) / TILE_PX) as i32;
        let row = ((p.y + p.height - 1.0) / TILE_PX) as i32;
        for c in [col_l, col_r] {
            for r in row - 3..=row {
                let i = w.index(c, r);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The probe now fails (the 20-wide restore overlaps the walls).
        assert_eq!(
            p.can_dismount_with_result(&w),
            crate::mount::DismountCheck::FailedNoSpace
        );
        // Strike 1 and 2: the tink queues, the mount stays.
        assert_eq!(p.quick_mount(&w, &[], &[], false), None);
        assert_eq!(p.failed_no_space_count, 1);
        assert!(p.take_no_space_sound().is_some());
        assert!(p.mount.is_active());
        assert_eq!(p.quick_mount(&w, &[], &[], false), None);
        assert_eq!(p.failed_no_space_count, 2);
        assert!(p.take_no_space_sound().is_some());
        // Strike 3: the forced dismount + the teleport request.
        assert_eq!(p.quick_mount(&w, &[], &[], false), None);
        assert!(!p.mount.is_active(), "the third strike forces the dismount");
        assert_eq!(p.failed_no_space_count, 0);
        assert!(p.take_no_space_teleport(), "the teleport request queued");
    }

    #[test]
    fn s1_p_i41_update_failed_dismount_clears_on_a_moved_tile() {
        // `Player.UpdateFailedDismount` (`Player.cs:6037-6043`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false);
        p.failed_no_space_count = 2;
        p.failed_no_space_location = ((p.x / TILE_PX) as i32, (p.y / TILE_PX) as i32);
        p.update_failed_dismount();
        assert_eq!(p.failed_no_space_count, 2, "same tile keeps the count");
        p.x += TILE_PX * 2.0;
        p.update_failed_dismount();
        assert_eq!(p.failed_no_space_count, 0, "a moved tile clears it");
        // A dismount clears it too.
        p.failed_no_space_count = 2;
        p.failed_no_space_location = ((p.x / TILE_PX) as i32, (p.y / TILE_PX) as i32);
        p.dismount(&w, false);
        p.update_failed_dismount();
        assert_eq!(p.failed_no_space_count, 0);
    }

    #[test]
    fn s1_p_i41_the_chillet_equip_queues_the_pal_chillet_cue() {
        // `Mount.SetMount` (`Mount.cs:6283-6286`): types 62-65 play
        // `SoundID.PalChillet` at `mountedPlayer.Center`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(62, false));
        let (cx, cy) = p.center();
        assert_eq!(p.take_chillet_sound(), Some((cx, cy)));
        assert!(p.take_chillet_sound().is_none(), "the queue drains once");
        // A non-Chillet mount queues nothing.
        p.dismount(&w, false);
        assert!(p.set_mount(10, false));
        assert!(p.take_chillet_sound().is_none());
    }

    #[test]
    fn s1_p_i41_the_no_space_teleport_finds_an_open_spot() {
        // `TeleportHelpers.FindClosestTeleportSpotNoSpace`
        // (`TeleportHelpers.cs:8-47`): the closest free cell over a solid dry
        // floor; the velocity zeroes (`:12`). The scan window clamps to
        // `40 .. width-40` (`:20-21`), so the fixture needs a world wider
        // than 80 tiles (the 64-wide `ground_world` collapses it to empty).
        let mut w = World::new("wide".into(), 128, 128);
        for x in 0..128 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 64;
        w.spawn_y = 40;
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.vx = 5.0;
        p.vy = 3.0;
        let spot = find_closest_teleport_spot_no_space(&w, &mut p);
        assert!(spot.is_some(), "an open spot exists over the ground");
        assert_eq!((p.vx, p.vy), (0.0, 0.0), "the scan zeroes the velocity");
        let (sx, sy) = spot.unwrap();
        // The spot rests on the ground row 60: the candidate bottom is
        // `j*16+15` with `j = 59` (`:29`) — one pixel above the floor top.
        assert!((sy + p.height - (60.0 * TILE_PX - 1.0)).abs() < TILE_PX);
        // The teleport writes the position + fall origin (`Player.cs:37446`).
        p.teleport_style(sx, sy, 11);
        assert_eq!((p.x, p.y), (sx, sy));
        assert_eq!(p.fall_start, (sy / TILE_PX) as i32);
    }

    #[test]
    fn s1_p_i41_the_transformation_mounts_lock_items_every_tick() {
        // `Mount.UpdateEffects` cases 55/56/61 (`Mount.cs:5137-5145`):
        // `noItems = true` re-arms every tick (ResetEffects clears it,
        // `Player.cs:18687`). The use gate (`:52140-52148`) widens only when
        // the dismounts-on-item-use mount CAN dismount — CCed closes it
        // (`DontDismountWhenCCed`, `MountID.cs:47`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.set_mount(61, false)); // the Pixie
        p.update(&w, PlayerInput::default());
        assert!(p.no_items, "the Pixie locks item use");
        // Open space: the exemption widens (the use dismounts first).
        assert!(p.can_use_item_past_no_items(&w, false));
        assert!(p.can_use_item_past_no_items(&w, true), "ignoreCursed widens");
        // CCed: the dismount refuses, so the lock holds every use.
        p.frozen = true;
        assert!(!p.can_use_item_past_no_items(&w, false));
        assert!(p.can_use_item_past_no_items(&w, true));
        // The quick-mount toggle refuses the CCed dismount (`:6049-6052`).
        let inv = [saddle(3260), EMPTY_ITEM_SLOT];
        assert_eq!(p.quick_mount(&w, &inv, &[], false), None);
        assert!(p.mount.is_active(), "the CCed mount stays up");
        p.frozen = false;
        // The Rat (55) and Bat (56) lock too; the Unicorn (10) does not.
        p.dismount(&w, false);
        assert!(p.set_mount(55, false));
        p.update(&w, PlayerInput::default());
        assert!(p.no_items);
        p.dismount(&w, false);
        assert!(p.set_mount(56, false));
        p.update(&w, PlayerInput::default());
        assert!(p.no_items);
        p.dismount(&w, false);
        assert!(p.set_mount(10, false));
        p.update(&w, PlayerInput::default());
        assert!(!p.no_items, "the Unicorn holds items");
    }

    #[test]
    fn s1_p_i41_the_cute_fishron_special_grants_the_damage_bonus() {
        // `Mount.UpdateEffects` case 12 (`Mount.cs:4983-4999`): while
        // `MountFishronSpecial` (`Player.cs:4044-4056`), every class gains
        // 0.15. Healthy + dry + no window + no rain → NOT special: no grant.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(12, false));
        p.hp = p.max_hp;
        p.mount_fishron_special_counter = 0.0;
        p.update_mount(&w, false, None);
        assert_eq!(p.melee_damage, 1.0, "no special → no bonus");
        assert_eq!(p.minion_damage, 1.0);
        // Hurt arms the special (the getter's `statLife >= statLifeMax2 / 2`
        // fails only STRICTLY below half; `:5001-5003` sets the 60-tick
        // window after the damage read, like the C# case order).
        p.melee_damage = 1.0;
        p.ranged_damage = 1.0;
        p.magic_damage = 1.0;
        p.minion_damage = 1.0;
        p.hp = p.max_hp / 2 - 1;
        p.update_mount(&w, false, None);
        assert_eq!(p.melee_damage, 1.15, "the special grants +0.15 melee");
        assert_eq!(p.ranged_damage, 1.15);
        assert_eq!(p.magic_damage, 1.15);
        assert_eq!(p.minion_damage, 1.15);
        assert_eq!(p.mount_fishron_special_counter, 60.0, "hurt arms 60");
    }

    #[test]
    fn s1_p_i41_the_santank_arms_its_extra_jump() {
        // `Mount.UpdateEffects` case 46's head (`Mount.cs:4882-4884`):
        // `hasJumpOption_Santank = true` before the target scan.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(46, false));
        p.has_jump_options[ExtraJump::Santank.index()] = false;
        p.update_mount(&w, false, None);
        assert!(
            p.has_jump_options[ExtraJump::Santank.index()],
            "the Santank jump armed"
        );
    }

    #[test]
    fn s1_p_i41_the_empress_brooch_grants_and_the_tick_clears_it() {
        // `ApplyEquipFunctional` (`Player.cs:14771-14774`): the worn Empress
        // Brooch (4989) sets `empressBrooch` + `moveSpeed += 0.075`;
        // `ResetEffects` (`:18455`) clears the flag every tick.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 4989;
        p.update(&w, PlayerInput::default());
        assert!(p.empress_brooch, "the worn brooch grants");
        // The raptor's refill reads it (`Mount.cs:2993-2996`): wings + the
        // brooch refill the borrowed budget every UpdateEquips tail. The
        // worn wings (1866, slot 22) re-derive the budget from the slot.
        assert!(p.set_mount(54, false));
        p.equipped[4] = 1866;
        p.update_equips();
        let budget = p.wing_time_max;
        assert!(budget > 0, "the worn wings set the budget");
        p.mount.reset_flight_time(0.0, budget);
        p.mount.flight();
        p.update_equips();
        assert_eq!(p.mount.fly_time(), budget, "the brooch refills the raptor");
        // Unequipped: the flag clears and the refill stops (the wing slot
        // still holds the budget; only the brooch flag went away). The tail
        // call avoids the tick's resting wing refill masking the probe.
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.empress_brooch);
        p.mount.reset_flight_time(0.0, p.wing_time_max);
        p.mount.flight();
        p.update_equips();
        assert_eq!(
            p.mount.fly_time(),
            p.wing_time_max - 1,
            "no refill without the brooch"
        );
    }

    #[test]
    fn s1_p_i41_shellphone_spawn_falls_back_and_queues_the_force_clear() {
        // `Player.Shellphone_Spawn` (`Player.cs:56164-56181`): the world
        // spawn floor search, the invalid-area force-clear request, the
        // style-11 teleport and the velocity zero.
        let mut w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 4.0;
        p.vy = -2.0;
        p.shellphone_spawn(&w);
        // The spawn (10, 40) sits over open air down to the row-60 ground:
        // the descent lands the feet at the floor top.
        assert_eq!(p.y, 60.0 * TILE_PX - p.height);
        assert_eq!((p.vx, p.vy), (0.0, 0.0));
        assert!(p.take_force_clear().is_none(), "the valid area needs no clear");
        // Wall the spawn column solid from row 0 to the ground: the ±30
        // scan and both descents fail, the floor stays (10, 40) — invalid →
        // the force-clear queues (`Spawn_GetPositionAtSpawn` `:37723-37804`).
        for x in 8..=12 {
            for y in 0..60 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        p.shellphone_spawn(&w);
        assert!(p.take_force_clear().is_some(), "the invalid area queues");
        assert!(p.take_force_clear().is_none(), "the queue drains once");
    }

    /// The player-vitals slice: `Player.Hurt`'s i-frame write, the fall/lava
    /// hazards, the death coin drop and the respawn timer — each pinned to
    /// the decompiled C# blocks (see the per-test citations).
    mod player_vitals {
        use super::*;

        /// The tick head's counters (`Player.Update` `:25456` `UpdateImmunity`
        /// and the hurt-cooldown decay) replayed, so the survival arm can be
        /// driven without the movement block re-deriving `in_lava` from the
        /// liquid scan (`:27728`).
        fn tick_counters(p: &mut Player) {
            p.update_immunity();
            p.hurt_cd = (p.hurt_cd - 1.0 / 60.0).max(0.0);
        }

        /// t1 — `Player.Hurt`'s immunity write (`Player.cs:38382-38385`):
        /// `num7 = pvp ? 8 : (num2 != 1 ? (longInvince ? 80 : 40) : (longInvince
        /// ? 40 : 20))`, then `immune = true` and `immuneTime = num7` on the
        /// General lane. The write rides the same `immunity` fields the dodge
        /// handlers use, not `hurtCooldowns` alone.
        #[test]
        fn hit_gives_csharp_iframes() {
            let w = ground_world();
            let mut rng = DodgeRng::with_rolls(&[]);
            let mut p = Player::spawn(&w);
            p.stat_defense = 0;
            p.immune = false;
            p.immune_time = 0;
            p.hurt_cd = 0.0;
            assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
            assert_eq!(p.hp, 80, "20 damage, no defense");
            assert!(p.immune, "the hit arms the immunity flag");
            assert_eq!(p.immune_time, 40, "num7 = 40 for damage != 1");
            assert_eq!(p.hurt_cd, 40.0 / 60.0, "the same 40-tick window");
            // The knockback (`:38519-38524`): `4.5 * hitDirection`, `-3.5` Y.
            assert_eq!((p.vx, p.vy), (-4.5, -3.5), "hit from the right pushes left");
            // `num2 != 1.0` picks the 40 window; an exact 1 selects 20.
            let mut q = Player::spawn(&w);
            q.stat_defense = 0;
            q.immune = false;
            q.immune_time = 0;
            q.hurt_cd = 0.0;
            assert!(q.hurt(1, q.x + 100.0, &mut rng, true));
            assert_eq!(q.hp, 99);
            assert_eq!(q.immune_time, 20, "num7 = 20 for exactly 1 damage");
            // `longInvince` doubles the pair (`:38382`).
            let mut r = Player::spawn(&w);
            r.stat_defense = 0;
            r.immune = false;
            r.immune_time = 0;
            r.hurt_cd = 0.0;
            r.long_invince = true;
            assert!(r.hurt(20, r.x + 100.0, &mut rng, true));
            assert_eq!(r.immune_time, 80, "longInvince: 80 ticks");
        }

        /// t2 — the landing curve: `num10 = 25 + extraFall` (`:25301`), the
        /// strict `num11 > num10` gate (`:25360`) and
        /// `num17 = (num11 - num10) * 10` (`:25363`); plus the `immune = false`
        /// forced BEFORE the Hurt (`:25361`) so a fall lands through i-frames.
        #[test]
        fn fall_damage_threshold_and_curve() {
            let w = ground_world();
            // The feet rest on row 57 in `ground_world`.
            let (lost, landed) = drop_and_settle(&w, 32); // fell 25
            assert_eq!((lost, landed), (0, 57), "25 tiles is the free edge");
            assert_eq!(drop_and_settle(&w, 31).0, 10, "fell 26 -> 1 tile * 10");
            assert_eq!(drop_and_settle(&w, 25).0, 70, "fell 32 -> 7 tiles * 10");
            assert_eq!(drop_and_settle(&w, 23).0, 90, "fell 34 -> 9 tiles * 10");
            // Mid-iframes (a contact hit landed a moment ago): `:25361`
            // clears `immune` before the Hurt, so the fall still lands.
            let mut p = Player::spawn(&w);
            p.immune = true;
            p.immune_time = 600;
            p.hurt_cd = 10.0;
            p.y = 25.0 * TILE_PX;
            p.fall_start = 25;
            for _ in 0..600 {
                p.update(&w, PlayerInput::default());
                if p.on_ground {
                    break;
                }
            }
            assert!(p.hp < 100, "the fall lands through iframes, hp {}", p.hp);
        }

        /// t3 — `Player.cs:27696-27712`: 80 damage per lava hit (`num79`), the
        /// hit armed only once the Lava lane's window is free — `num7 = 40`
        /// (`:38382`) — and OnFire (24) for 420 ticks on a landed hit.
        #[test]
        fn lava_ticks_per_second() {
            let w = ground_world();
            let mut rng = tr_core::UnifiedRandom::default();
            let mut p = Player::spawn(&w);
            p.max_hp = 500;
            p.hp = 500;
            let mut hits: Vec<(i32, i32)> = Vec::new();
            let mut prev = p.hp;
            for tick in 0..80 {
                if tick > 0 {
                    tick_counters(&mut p);
                }
                p.in_lava = true;
                p.lava_wet = true;
                p.update_survival(&w, &mut rng);
                // Only the 80-damage hits count: the OnFire DoT also drains.
                if prev - p.hp >= 40 {
                    hits.push((tick, prev - p.hp));
                }
                prev = p.hp;
            }
            assert_eq!(
                hits.iter().map(|h| h.0).collect::<Vec<_>>(),
                vec![0, 40],
                "one lava hit per 40 ticks"
            );
            assert_eq!(hits[0].1, 80, "80 damage per landing hit");
            assert!(p.buffs.find_buff_index(24).is_some(), "OnFire granted");
        }

        /// t4 — `Player.DropCoins` (`Player.cs:54261-54294`): per coin stack
        /// `num2 = stack / 2` is KEPT and `num3 = stack - num2` (the ceiling
        /// half) is dropped — classic; expert `stack / 4`, master `0`.
        #[test]
        fn death_drops_coins_per_csharp_rule() {
            let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
            app.coins = 7;
            assert_eq!(
                crate::death::drop_coins(&mut app, 0.0, 0.0),
                4,
                "7 copper: 3 kept, 4 dropped"
            );
            assert_eq!(app.coins, 3, "the kept half stays");
            app.coins = 1000;
            assert_eq!(crate::death::drop_coins(&mut app, 0.0, 0.0), 500);
            assert_eq!(app.coins, 500);
            // The death entry point books the same loss (`:38931-38936`).
            app.coins = 10_050;
            crate::death::on_player_death(&mut app, 0.0, 0.0, "x");
            assert_eq!(app.lost_coins, 5025);
            assert_eq!(app.coins, 5025);
        }

        /// t5 — `Player.GetRespawnTime` (`:39060-39082`) and the
        /// `UpdateDead` tick (`:17016` `statLife = 0`, `:17055-17058`
        /// `respawnTimer` clamped 0..3600), then `Spawn`'s revive
        /// (`:37600-37618`, `:37658-37666`: `max(max/2, 100)` life, 180
        /// i-frames).
        #[test]
        fn respawn_timer_matches() {
            assert_eq!(Player::get_respawn_time(false, false), 600);
            assert_eq!(Player::get_respawn_time(true, false), 1200, "boss +600");
            assert_eq!(Player::get_respawn_time(false, true), 900, "expert x1.5");
            assert_eq!(Player::get_respawn_time(true, true), 1800);
            let w = ground_world();
            let mut p = Player::spawn(&w);
            p.start_death(&w, false, false);
            assert!(p.dead);
            assert_eq!(p.respawn_timer, 600);
            for _ in 0..599 {
                assert!(!p.update_dead(false, false, false, false));
            }
            assert_eq!(p.respawn_timer, 1);
            assert!(p.update_dead(false, false, false, false), "revive on tick 600");
            // The dead tick zeroes the life (`:17016`).
            p.hp = 40;
            p.update_dead(false, false, false, false);
            assert_eq!(p.hp, 0, "UpdateDead sets statLife = 0");
            p.revive();
            assert!(!p.dead);
            assert_eq!(p.hp, 100, "max(max/2, 100)");
            assert!(p.immune && p.immune_time == 180, "180 revive i-frames");
        }
    }
}

/// M4-PROJ-8 phase 2 — the summon keep-alive arms (`Player.cs:10483-10947`),
/// the recount caches (`:12184-12226`) and the buff-216 respawn arm, driven
/// on a bare Player + pool (the world_sim tick wires the same calls).
#[cfg(test)]
mod summon_tests {
    use super::*;
    // S1-P-i12's mirror witnesses reuse the main test module's world fixture
    // (the V4/V5 first-compile fix — the coder referenced it without the import;
    // both gates added the same import, deduped to one at the M5 sync merge).
    use super::tests::ground_world;
    use crate::item::EMPTY_SLOT;

    /// `Player.FindItem`'s stand-in: the inventory with the staff (4281) in
    /// slot 0 (the `ItemDef` verified this phase: shoot 759, damage 7,
    /// knockBack 4.0).
    fn staff_inventory() -> [ItemSlot; INV] {
        let mut inv = [EMPTY_SLOT; INV];
        inv[0] = ItemSlot {
            kind: 4281,
            name: registry::name(4281),
            count: 1,
            prefix: 0,
        };
        inv
    }

    fn empty_inventory() -> [ItemSlot; INV] {
        [EMPTY_SLOT; INV]
    }

    /// Deactivates a slot through the public key path (`key_of` +
    /// `kill_remote`, the `29` despawn) — the pool's `Kill` stand-in.
    fn kill_pool_slot(pool: &mut ProjectilePool, slot: usize) {
        let key = pool.key_of(slot);
        let (owner, slot, generation) = crate::net::projectile_key_parts(key);
        pool.kill_remote(slot, owner, generation);
    }

    /// DW-2.2 — buff 64 (`Slime`) tracks projectile 266 exactly: kept and
    /// refreshed to 18000 while the pool holds it, dropped on the first pass
    /// whose recount sees it gone (the C# reads the pool the previous frame's
    /// pool pass left, `Player.cs:25565` → the arms `:10521-10534`).
    #[test]
    fn test_DW_2_2_buff_64_tracks_projectile_266() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        let minion = pool
            .spawn(266, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        p.buffs.add_buff(64, 100);
        // The live order: `ResetProjectileCaches` (`:18810`, the ResetEffects
        // seam) then the recount (`:25565`) then the arms.
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(p.slime, "266 in the pool re-affirms the flag");
        let b = p.buffs.find_buff_index(64).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000, "the arm pins the refresh");
        assert_eq!(p.owned_projectile_counts[266], 1);
        // The boundary: the minion dies between ticks (the pool pass's kill)
        // — the drop lands on exactly the next pass, not earlier.
        kill_pool_slot(&mut pool, minion);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(
            p.buffs.find_buff_index(64),
            None,
            "the buff drops the tick the recount sees 266 gone"
        );
        // The flag lifecycle: the arm only SETS it — `ResetEffects` clears it
        // per tick (the full tick's seam, `:18622`). One real tick clears it.
        p.update(&w, PlayerInput::default());
        assert!(!p.slime, "the flag is only true while the arm re-affirms");
        // A buff with no minion ever spawns drops immediately.
        p.buffs.add_buff(64, 100);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(64), None);
    }

    /// DW-2.2 boundary variant — buff 216's baby-bird respawn arm
    /// (`Player.cs:10697-10726`): below cap with the staff spawns 759 from
    /// the item's shoot/damage/knockBack (+ `originalDamage = item.damage`,
    /// `:10713`); at cap it DelBuffs; without the staff it DelBuffs.
    #[test]
    fn test_DW_2_2_buff_216_respawn_arm() {
        let w = World::new("t".into(), 64, 800);
        let inv = staff_inventory();
        // Below cap: the respawn arm fires.
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        p.buffs.add_buff(216, 100);
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(p.baby_bird, "the spawn re-affirms the flag");
        let bird = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 759)
            .expect("759 spawned from the staff");
        assert_eq!(bird.damage, 7);
        assert_eq!(bird.original_damage, 7, "Player.cs:10713");
        assert_eq!(bird.knock_back, 4.0);
        assert_eq!(bird.owner, 0);
        assert_eq!(bird.pos.x, p.x + WIDTH * 0.5);
        assert_eq!(bird.pos.y, p.y, "Entity.Top — the hitbox's top center");
        assert_eq!(bird.vel, Vec2::ZERO);
        // The verbatim 759 slice rides the data-row spawn seam (the phase-1
        // review fix): the 10×10 box and tile collide from the row, the
        // 18000 lifetime, the minion flag and its 1.0 slot budget.
        assert_eq!((bird.w, bird.h), (10.0, 10.0));
        assert!(bird.tile_collide);
        assert_eq!(bird.time_left, 18000);
        assert!(bird.minion);
        assert_eq!(bird.minion_slots, 1.0);
        let b = p.buffs.find_buff_index(216).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000);
        // At cap (`numMinions == maxMinions`): DelBuff, no spawn. The
        // counter is the pool pass's stand-in until Phase 3 writes it.
        let mut p2 = Player::spawn(&w);
        let mut pool2 = ProjectilePool::new();
        p2.num_minions = 1;
        p2.buffs.add_buff(216, 100);
        p2.update_projectile_caches(&pool2);
        p2.update_summon_buff_arms(&mut pool2, &inv);
        assert!(
            !pool2.slots().iter().any(|s| s.active && s.kind == 759),
            "at cap the arm spawns nothing"
        );
        assert_eq!(p2.buffs.find_buff_index(216), None, "Player.cs:10719-10722");
        // Below cap without the staff: DelBuff too.
        let mut p3 = Player::spawn(&w);
        let mut pool3 = ProjectilePool::new();
        p3.buffs.add_buff(216, 100);
        p3.update_projectile_caches(&pool3);
        p3.update_summon_buff_arms(&mut pool3, &empty_inventory());
        assert_eq!(p3.buffs.find_buff_index(216), None);
    }

    /// The C# `DelBuff(j); j--;` rescan (`:10483-10947`): a dropped arm's
    /// neighbour shifts into its slot and still gets its arm the same tick.
    #[test]
    fn test_edge_the_arms_rescan_after_delbuff() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        pool.spawn(317, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        // Buff 64 at slot 0 (no 266 — dropped), buff 83 at slot 1 (317 alive
        // — kept): after the drop, 83 shifts into slot 0 and is re-examined.
        p.buffs.add_buff(64, 100);
        p.buffs.add_buff(83, 100);
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(64), None);
        let raven = p.buffs.find_buff_index(83).expect("the rescan kept 83");
        assert!(p.raven);
        assert_eq!(p.buffs.buff_time[raven], 18000);
    }

    /// The cap effects share the arms' slot-ordered chain
    /// (`Player.cs:10175-10183` before `:10706`): buff 110 in an earlier slot
    /// raises `maxMinions` before buff 216's arm reads it, so a minion count
    /// at the base cap still spawns.
    #[test]
    fn test_edge_cap_effects_are_slot_ordered_before_the_216_arm() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = staff_inventory();
        p.buffs.add_buff(110, 100); // slot 0 — the raise
        p.buffs.add_buff(216, 100); // slot 1 — the read
        p.num_minions = 1; // at the base cap of 1
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.max_minions, 2, "110 raised the cap");
        assert!(
            pool.slots().iter().any(|s| s.active && s.kind == 759),
            "216 read the raised cap and spawned"
        );
    }

    /// The chain's 29th arm — buff 161 (`DeadlySphereMinion`, kind 533,
    /// `Player.cs:10957-10972`, review fix): the plain keep-alive shape, kept
    /// and refreshed while 533 is owned, dropped the pass the recount sees
    /// it gone.
    #[test]
    fn test_edge_buff_161_tracks_projectile_533() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        let sphere = pool
            .spawn(533, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        p.buffs.add_buff(161, 100);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(
            p.deadly_sphere_minion,
            "533 in the pool re-affirms the flag"
        );
        let b = p.buffs.find_buff_index(161).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000);
        assert_eq!(p.owned_projectile_counts[533], 1);
        // The drop: the minion leaves the pool, the buff follows one pass
        // later — and the flag clears on the next tick's ResetEffects.
        kill_pool_slot(&mut pool, sphere);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(161), None);
        p.update(&w, PlayerInput::default());
        assert!(!p.deadly_sphere_minion);
    }

    /// S1-P-i25 round 2 — `Player.GetWeaponKnockback` (`Player.cs:53346-53369`):
    /// the summon `minionKB` add FIRST, then the melee glove's ×2, the Titan
    /// buff's ×1.5 and the shroomite-stealth ranged scale, in that order.
    #[test]
    fn i25_get_weapon_knockback_folds_the_bonuses_in_order() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let summon = crate::registry::get(2364).unwrap(); // Hornet Staff (summon)
        let melee = crate::registry::get(1).unwrap(); // Iron Pickaxe (melee)
        let ranged = crate::registry::get(435).unwrap(); // Cobalt Repeater (ranged)
        // The summon add: minionKB rides a summon item, not a melee one.
        p.minion_kb = 2.0;
        assert_eq!(p.weapon_knockback(summon, 5.0), 7.0, "the summon minionKB add");
        assert_eq!(p.weapon_knockback(melee, 5.0), 5.0, "no add for melee");
        // The melee glove ×2 (melee only).
        p.kb_glove = true;
        assert_eq!(p.weapon_knockback(melee, 5.0), 10.0, "the glove doubles melee");
        assert_eq!(p.weapon_knockback(summon, 5.0), 7.0, "the glove skips summon");
        // The Titan buff ×1.5 rides everything, AFTER the add/glove.
        p.kb_buff = true;
        assert_eq!(p.weapon_knockback(summon, 5.0), 10.5, "(5+2)*1.5");
        assert_eq!(p.weapon_knockback(melee, 5.0), 15.0, "5*2*1.5");
        // The shroomite-stealth ranged scale (ranged only): 1 + (1-stealth)*0.5.
        p.kb_buff = false;
        p.kb_glove = false;
        p.shroomite_stealth = true;
        p.stealth = 0.5;
        assert_eq!(
            p.weapon_knockback(ranged, 4.0),
            4.0 * 1.25,
            "the stealth scale at stealth 0.5"
        );
        assert_eq!(p.weapon_knockback(summon, 5.0), 7.0, "the stealth scale skips summon");
    }

    // ---- S1-P-i12: the useTime/useAnimation speed machinery ----------------

    /// `Player.meleeSpeed` as the game logic reads it — the `:25912`
    /// held-item scale, then `CapAttackSpeeds`' cap-and-invert
    /// (`:29021-29039`). Mirror clone of the C# chain, written not run.
    #[test]
    fn i12_melee_speed_is_the_capped_inverted_held_item_scaled_multiplier() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // No grants: 1f speed → 1f multiplier (`TurnAttackSpeedToUseTimeMultiplier(1)`).
        assert_eq!(p.melee_speed(1), 1.0);
        // The Feral Claws' +0.12 (`:15108`): 1.12 → 1/1.12.
        p.melee_speed_bonus = 0.12;
        assert!((p.melee_speed(1) - 1.0 / 1.12).abs() < 1e-6);
        // `BonusMeleeSpeedMultiplier` (`ItemID.cs:1350`): the Fetid
        // Baghnakhs (3013) scale the BONUS by 0.25 — 1 + 0.12*0.25 = 1.03.
        assert!((p.melee_speed(3013) - 1.0 / 1.03).abs() < 1e-6);
        // The Bladed Glove (1827) 0.5, Psycho Knife (3106) 0.33, Terra
        // Blade (757) 0.75 — each scales only the bonus, never the base 1f.
        assert!((p.melee_speed(1827) - 1.0 / 1.06).abs() < 1e-6);
        assert!((p.melee_speed(3106) - 1.0 / (1.0 + 0.12 * 0.33)).abs() < 1e-6);
        assert!((p.melee_speed(757) - 1.0 / 1.09).abs() < 1e-6);
        // The 3f cap (`:29030-29033`): a +5 bonus clamps to 1/3.
        p.melee_speed_bonus = 5.0;
        assert!((p.melee_speed(1) - 1.0 / 3.0).abs() < 1e-6);
        // A zero speed stays zero (`:29034`'s `speed != 0f` guard).
        p.melee_speed_bonus = -1.0;
        assert_eq!(p.melee_speed(1), 0.0);
    }

    /// `CapAttackSpeeds`' summoner half (`:29025`) folds the ALREADY
    /// held-item-scaled melee speed (`:25912` runs before `:25976`) plus the
    /// whip-buff sum, caps, inverts; `ApplyItemAnimation` (`:4476`) then
    /// multiplies by `whipUseTimeMultiplier`. Mirror clone, written not run.
    #[test]
    fn i12_summoner_speed_folds_the_scaled_melee_and_whip_buffs() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Baseline: no grants → 1f.
        assert_eq!(p.summoner_weapon_speed_multiplier(4672), 1.0);
        // The 311 buff's +0.35 (`:10112`): 1.35 → 1/1.35.
        p.summoner_weapon_speed_bonus = 0.35;
        assert!((p.summoner_weapon_speed_multiplier(4672) - 1.0 / 1.35).abs() < 1e-6);
        // The melee sum joins it (`num + summonerWeaponSpeedBonus`): the
        // Feral Claws' +0.12 makes 1.47.
        p.melee_speed_bonus = 0.12;
        assert!((p.summoner_weapon_speed_multiplier(4672) - 1.0 / 1.47).abs() < 1e-6);
        // The held-item scale rides the melee half too (`:25912` precedes
        // `CapAttackSpeeds`): holding the Fetid Baghnakhs (0.25) makes the
        // melee contribution 0.03 → 1.38.
        assert!((p.summoner_weapon_speed_multiplier(3013) - 1.0 / 1.38).abs() < 1e-6);
        // The Obsidian Outlaw set's ×(1/1.15) (`ArmorSetBonuses.cs:124`)
        // multiplies AFTER the inversion (`Player.cs:4476`).
        p.whip_use_time_multiplier = 1.0 / 1.15;
        assert!(
            (p.summoner_weapon_speed_multiplier(4672) - (1.0 / 1.47) * (1.0 / 1.15)).abs() < 1e-6
        );
    }

    /// `UpdateEquips`' tail caps tileSpeed/wallSpeed at 3f and INVERTS them
    /// (`Player.cs:25913-25922`) — `ApplyItemAnimation`'s createTile/
    /// createWall branches (`:4486`/`:4490`) read the inverted values.
    /// Mirror clone, written not run.
    #[test]
    fn i12_tile_and_wall_speed_cap_and_invert() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert_eq!(p.tile_speed_multiplier(), 1.0);
        assert_eq!(p.wall_speed_multiplier(), 1.0);
        // The Builder buff's +0.25 (`:10160-10161`): 1.25 → 1/1.25 = 0.8.
        p.tile_speed_bonus = 0.25;
        p.wall_speed_bonus = 0.25;
        assert!((p.tile_speed_multiplier() - 0.8).abs() < 1e-6);
        assert!((p.wall_speed_multiplier() - 0.8).abs() < 1e-6);
        // The cap: +5 clamps to 3f → 1/3 (NOT 1/6).
        p.tile_speed_bonus = 5.0;
        p.wall_speed_bonus = 5.0;
        assert!((p.tile_speed_multiplier() - 1.0 / 3.0).abs() < 1e-6);
        assert!((p.wall_speed_multiplier() - 1.0 / 3.0).abs() < 1e-6);
    }

    /// The glove family + Feral Claws + Wicked Claws grant `autoReuseGlove`
    /// (`Player.cs:14413`/`:14602`/`:15107`/`:15302`/`:15309`/`:15372`) —
    /// `TryAllowingItemReuse` (`:52905-52920`) reads it. The per-tick
    /// `ResetEffects` clear (`:18710`) re-arms from the live slots.
    /// Mirror clone, written not run.
    #[test]
    fn i12_the_glove_family_grants_auto_reuse_glove() {
        let w = ground_world();
        for kind in [211u16, 6182, 897, 936, 1343, 3992] {
            let mut p = Player::spawn(&w);
            p.equipped[3] = kind;
            p.update(&w, PlayerInput::default());
            assert!(p.auto_reuse_glove, "kind {kind} grants autoReuseGlove");
            assert!((p.melee_speed_bonus - 0.12).abs() < 1e-6, "kind {kind} +0.12");
        }
        // Taking the glove off clears the flag on the next tick
        // (`ResetEffects` `:18710`, no re-grant).
        let mut p = Player::spawn(&w);
        p.equipped[3] = 211;
        p.update(&w, PlayerInput::default());
        assert!(p.auto_reuse_glove);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.auto_reuse_glove, "the reset clears it without a re-grant");
    }

    /// The whip attack-speed buffs (`Player.cs:10110-10121`: 311 +0.35,
    /// 308 +0.25, 314 +0.12) and the Builder buff's tile/wall halves
    /// (`:10158-10163`) land on the per-tick accumulators. Mirror clone,
    /// written not run.
    #[test]
    fn i12_the_whip_and_builder_buffs_feed_the_speed_accumulators() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(311, 600);
        p.buffs.add_buff(308, 600);
        p.buffs.add_buff(314, 600);
        p.buffs.add_buff(107, 600);
        p.update(&w, PlayerInput::default());
        assert!((p.summoner_weapon_speed_bonus - 0.72).abs() < 1e-6, "0.35+0.25+0.12");
        assert!((p.tile_speed_bonus - 0.25).abs() < 1e-6);
        assert!((p.wall_speed_bonus - 0.25).abs() < 1e-6);
    }

    /// The placement-speed accessories (`Player.cs:15263-15292` flags,
    /// `:13024-13039` fold): Brick Layer 2214 → tile +0.5 (skipping torch
    /// placements, `:13027`), Cement Mixer 2217 → wall +0.5, Gizmo Pack
    /// 3061 / Hand Of Creation 5126 → both. The flags ride
    /// `ApplyEquipFunctional` (slots 3..10, `:12946`). Mirror clone,
    /// written not run.
    #[test]
    fn i12_the_placement_speed_accessories_fold_half_a_speed() {
        let w = ground_world();
        // The Brick Layer with a stone block in hand (createTile 1, not a
        // torch): tile +0.5, wall untouched.
        let mut p = Player::spawn(&w);
        p.equipped[3] = 2214;
        p.held_item_kind = 3; // Stone Block
        p.update(&w, PlayerInput::default());
        assert!((p.tile_speed_bonus - 0.5).abs() < 1e-6);
        assert_eq!(p.wall_speed_bonus, 0.0);
        // A torch in hand (createTile 4, `TileID.Sets.Torches`): the tile
        // half skips (`:13027`).
        let mut p = Player::spawn(&w);
        p.equipped[3] = 2214;
        p.held_item_kind = 8; // Torch
        p.update(&w, PlayerInput::default());
        assert_eq!(p.tile_speed_bonus, 0.0, "the torch placement skips the Brick Layer");
        // The Cement Mixer: wall only.
        let mut p = Player::spawn(&w);
        p.equipped[3] = 2217;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.tile_speed_bonus, 0.0);
        assert!((p.wall_speed_bonus - 0.5).abs() < 1e-6);
        // The Gizmo Pack: both.
        let mut p = Player::spawn(&w);
        p.equipped[3] = 3061;
        p.held_item_kind = 3;
        p.update(&w, PlayerInput::default());
        assert!((p.tile_speed_bonus - 0.5).abs() < 1e-6);
        assert!((p.wall_speed_bonus - 0.5).abs() < 1e-6);
    }

    /// The Obsidian Outlaw set (`ArmorSetBonuses.cs:120-125`, the
    /// 3266/3267/3268 triple registered at `:492`):
    /// `whipUseTimeMultiplier *= 1f/1.15f`. Mirror clone, written not run.
    #[test]
    fn i12_the_obsidian_outlaw_triple_scales_the_whip_use_time() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[0] = 3266;
        p.equipped[1] = 3267;
        p.equipped[2] = 3268;
        p.update(&w, PlayerInput::default());
        assert!((p.whip_use_time_multiplier - 1.0 / 1.15).abs() < 1e-6);
        // A broken triple grants nothing (the set bonus needs all three).
        let mut p = Player::spawn(&w);
        p.equipped[0] = 3266;
        p.equipped[1] = 3267;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.whip_use_time_multiplier, 1.0);
    }

    /// `Mount.AllowDirectionChange` (`Mount.cs:630-640`): the Scutlix
    /// (type 9) turns only while its ability cooldown is under half the
    /// row's max (20 → `< 10`); every other mount always allows.
    /// Mirror clone, written not run.
    #[test]
    fn i12_the_scutlix_direction_lock_rides_the_ability_cooldown() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(9, false));
        // `SetMount` floors the Scutlix cooldown at the row max 20
        // (`Mount.cs:6236-6243`, the port's `mount.rs` `:2459-2461`): the
        // lock reads `< max/2` = `< 10`, so a fresh mount locks the turn.
        assert!(!p.mount.allow_direction_change(), "the fresh 20 >= 10 locks the turn");
        // A non-Scutlix mount always allows (the `:638` fallback).
        let mut q = Player::spawn(&w);
        assert!(q.set_mount(10, false));
        assert!(q.mount.allow_direction_change());
    }
}

#[cfg(test)]
mod s1_player_composite_tests {
    use super::*;
    // S1-E-21f gate fix: the sound-trigger tests below reuse the movement
    // fixtures from the sibling `tests` module (private there — the
    // code-only branch could not compile the cross-module reference).
    use super::tests::{ground_world, liquid_pool_world, settled_with_jumps};

    /// A frame-inputs builder with the raw counters the composite styles read.
    fn inputs(style: u8, remaining: f32, anim: f32, max: f32) -> FrameInputs {
        FrameInputs {
            use_style: style,
            use_turn: false,
            use_style_blocks_frames: true,
            item_type: 0,
            hold_style: 0,
            can_visually_hold: true,
            swing_active: true,
            swing_remaining: remaining,
            swing_anim: anim,
            swing_anim_max: max,
            use_animation: max as i32,
            item_rotation: 0.0,
            aim_relative: 0.0,
            hold_style_dry: true,
            torch_hold_ok: true,
            grapple_dx: 0.0,
            grapple_dy: 0.0,
        }
    }

    #[test]
    fn s1_pc_the_eat_style_assigns_both_arms_by_quarters() {
        // `Player.cs:50936-50955`: the both-arms -PI/2 pose, the stretch
        // walking Full -> None by `num10` quarters.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let held = inputs(2, 0.0, 20.0, 20.0); // progress = 1 -> None
        p.apply_item_composite_arms(&held);
        // `Player.cs:50953-50954`: the arm stays enabled with the `None`
        // stretch (the relaxed row) — it is not turned off.
        assert!(
            p.composite_front.enabled,
            "the arm stays on the relaxed row"
        );
        assert_eq!(p.composite_front.stretch, CompositeStretch::None);
        assert_eq!(p.composite_back.stretch, CompositeStretch::None);
        let held = inputs(2, 1.0, 1.0, 20.0); // progress = 0 -> Full
        p.apply_item_composite_arms(&held);
        assert!(p.composite_front.enabled);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Full);
        // The back arm rides the same stretch (the C# sets both).
        assert!(p.composite_back.enabled);
        assert_eq!(p.composite_back.stretch, CompositeStretch::Full);
        assert!(
            (p.composite_front.rotation + std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "the eat pose lifts the arms"
        );
    }

    #[test]
    fn s1_pc_the_mow_style_walks_the_two_cycle_and_pins_the_back_arm() {
        // `Player.cs:50978-51007`: the front arm's `num12 = progress * 2`
        // two-cycle table; the back arm Full at -PI/16 (no direction term).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let held = inputs(11, 0.0, 40.0, 20.0); // progress 1 -> num12 2 -> Full
        p.apply_item_composite_arms(&held);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Full);
        assert_eq!(p.composite_back.stretch, CompositeStretch::Full);
        assert!(
            (p.composite_back.rotation + std::f32::consts::PI / 16.0).abs() < 1e-6,
            "the back arm's constant pose"
        );
        let held = inputs(11, 0.75, 40.0, 20.0); // progress 0.25 -> num12 0.5 -> ThreeQuarters
        p.apply_item_composite_arms(&held);
        assert_eq!(p.composite_front.stretch, CompositeStretch::ThreeQuarters);
    }

    #[test]
    fn s1_pc_the_rapier_stretch_walks_quarter_to_full() {
        // `Player.cs:51174-51196`: `num20 = 1 - (anim % useAnim)/useAnim`;
        // the stretch Quarter (<=0.33) -> ThreeQuarters -> Full.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        // The fresh swing: anim = max -> num20 = 1 -> Full.
        let held = inputs(13, 1.0, 20.0, 20.0);
        p.apply_item_composite_arms(&held);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Full);
        // The late swing: anim = 5 of 20 -> num20 = 0.75 -> Full.
        let held = inputs(13, 0.25, 5.0, 20.0);
        p.apply_item_composite_arms(&held);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Full);
        // The mid swing: anim = 15 of 20 -> num20 = 0.25 -> Quarter.
        let held = inputs(13, 0.25, 15.0, 20.0);
        p.apply_item_composite_arms(&held);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Quarter);
    }

    #[test]
    fn s1_pc_the_rapier_arm_reads_the_item_rotation() {
        // `Player.cs:51195`: the front arm at `itemRotation * Directions.Y -
        // PI/2 * direction` (`Directions.Y` is the gravity direction, 1 in
        // the port).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let mut held = inputs(13, 1.0, 20.0, 20.0);
        held.item_rotation = 0.5;
        p.apply_item_composite_arms(&held);
        assert!(
            (p.composite_front.rotation - (0.5 - std::f32::consts::FRAC_PI_2)).abs() < 1e-6,
            "the rapier arm follows the item rotation"
        );
        // Facing left: the C#'s `- PI/2 * direction` flips the lift.
        p.facing = -1;
        p.apply_item_composite_arms(&held);
        assert!(
            (p.composite_front.rotation - (0.5 + std::f32::consts::FRAC_PI_2)).abs() < 1e-6,
            "the rapier arm mirrors with the direction"
        );
    }

    #[test]
    fn s1_pc_the_item_rotation_mirrors_the_aim_with_the_direction() {
        // `Player.cs:47378`: `itemRotation = atan2(aimY * direction, aimX *
        // direction) - fullRotation` — the mirrored `atan2` is the aim + PI
        // when the player faces left (not the negated angle).
        let right = item_rotation_for_aim(0.75, 1, 0.0);
        assert!((right - 0.75).abs() < 1e-6, "facing right keeps the aim");
        let left = item_rotation_for_aim(0.75, -1, 0.0);
        assert!(
            (left - (0.75 + std::f32::consts::PI)).abs() < 1e-6,
            "facing left mirrors the aim vector"
        );
        // The sleep pose's `fullRotation` rides the subtraction.
        let asleep = item_rotation_for_aim(0.75, 1, std::f32::consts::FRAC_PI_2);
        assert!((asleep - (0.75 - std::f32::consts::FRAC_PI_2)).abs() < 1e-6);
    }

    /// S1-P-i04 — the swing direction gate (`Player.cs:19837`):
    /// `(itemAnimation == 0 || inventory[selectedItem].useTurn) &&
    /// mount.AllowDirectionChange && !sandStorm`. The port's
    /// `horizontal_movement` flipped the facing on every run-key press
    /// mid-swing; a broadsword (`useTurn = false`) locks it.
    #[test]
    fn s1_p_i04_a_broadsword_swing_locks_the_facing() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.facing = 1;
        // Mid-swing with the Iron Broadsword (useTurn false): the left key
        // accelerates but does NOT turn the player. (M1 re-sync: the i12
        // PlayerInput channel carries the swing state — the gate's
        // frame_inputs form moved onto it, same predicate.)
        let input = PlayerInput {
            left: true,
            item_in_use: true,
            held_use_turn: false,
            ..Default::default()
        };
        p.update(&w, input);
        assert_eq!(p.facing, 1, "the swing locks the facing");
        assert!(p.vx < 0.0, "the run key still accelerates");
        // The swing ending (`itemAnimation == 0`) unlocks the turn.
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert_eq!(p.facing, -1, "the turn lands once the swing ends");
    }

    /// S1-P-i04 — a `useTurn` item (the pickaxe) keeps turning mid-swing
    /// (`Player.cs:19837`'s second term).
    #[test]
    fn s1_p_i04_a_use_turn_item_keeps_turning_mid_swing() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.facing = 1;
        p.update(
            &w,
            PlayerInput {
                left: true,
                item_in_use: true,
                held_use_turn: true,
                ..Default::default()
            },
        );
        assert_eq!(p.facing, -1, "useTurn keeps the direction live");
    }

    /// S1-P-i04 — the sandstorm locks the direction even at rest
    /// (`Player.cs:19837`'s `!this.sandStorm` term). Probed at the movement
    /// seam directly: the frame pass's sandstorm 4-tick flip
    /// (`Player.cs:36298-36311`) is a separate consumer of the same flag.
    #[test]
    fn s1_p_i04_the_sandstorm_locks_the_direction() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.facing = 1;
        p.control_left = true;
        p.control_right = false;
        p.sand_storm = true;
        p.horizontal_movement(PlayerInput::default());
        assert_eq!(p.facing, 1, "the sandstorm keeps the facing");
        // The flag down, the same press turns.
        p.sand_storm = false;
        p.horizontal_movement(PlayerInput::default());
        assert_eq!(p.facing, -1);
    }

    #[test]
    fn s1_pc_the_shoot_style_pins_the_back_arm_full() {
        // `Player.cs:51349-51360`: the useStyle 14 pose — the back arm Full
        // at -3*PI/4 * dir; the front arm untouched (DISABLED).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let held = inputs(14, 1.0, 20.0, 20.0);
        p.apply_item_composite_arms(&held);
        assert!(
            !p.composite_front.enabled,
            "the front arm rides the body rows"
        );
        assert!(p.composite_back.enabled);
        assert_eq!(p.composite_back.stretch, CompositeStretch::Full);
        assert!(
            (p.composite_back.rotation - std::f32::consts::PI * -3.0 / 4.0).abs() < 1e-6,
            "the -3*PI/4 aim"
        );
    }

    #[test]
    fn s1_pc_the_hold_composite_arms_follow_the_hold_styles() {
        // `Player.cs:50520-50582`: the holdStyle 4 umbrella pose (the front
        // Full + the back ThreeQuarters), the holdStyle 5's back Quarter at
        // -PI/4 * dir, and the holdStyle 8's both arms at -2*PI/5 * dir.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let mut held = inputs(0, 0.0, 0.0, 0.0);
        held.swing_active = false;
        held.hold_style = 4;
        p.apply_hold_composite_arms(&held);
        assert!(p.composite_front.enabled && p.composite_back.enabled);
        assert_eq!(p.composite_front.stretch, CompositeStretch::Full);
        assert_eq!(p.composite_back.stretch, CompositeStretch::ThreeQuarters);
        held.hold_style = 5;
        p.apply_hold_composite_arms(&held);
        assert_eq!(p.composite_back.stretch, CompositeStretch::Quarter);
        assert!((p.composite_back.rotation + std::f32::consts::FRAC_PI_4).abs() < 1e-6);
        held.hold_style = 8;
        p.apply_hold_composite_arms(&held);
        assert!((p.composite_front.rotation - std::f32::consts::PI * -2.0 / 5.0).abs() < 1e-6);
    }

    #[test]
    fn s1_g_18_the_umbrella_rotation_is_the_vector_rotation() {
        // `Player.cs:50524`: the umbrella's `itemRotation =
        // new Vector2(10 * direction, 10f).ToRotation() + PI/4` — the XNA
        // `ToRotation` is `atan2(Y, X)`, NOT the vector's length (the port
        // used `hypot`, which produced a nonsense angle).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        let mut held = inputs(0, 0.0, 0.0, 0.0);
        held.swing_active = false;
        held.hold_style = 4;
        p.apply_hold_composite_arms(&held);
        // Facing right: `itemRotation = atan2(10, 10) + PI/4 = PI/2`,
        // `num9 = itemRotation - PI/4 + PI (+ PI/2 when facing right)`,
        // `num8 = -PI/10`; the front arm is at `num9 + num8`.
        let item_rot = 10.0f32.atan2(10.0) + std::f32::consts::FRAC_PI_4;
        let num9 = item_rot - std::f32::consts::FRAC_PI_4
            + std::f32::consts::PI
            + std::f32::consts::FRAC_PI_2;
        let expect = num9 - std::f32::consts::PI / 10.0;
        assert!(
            (p.composite_front.rotation - expect).abs() < 1e-6,
            "{} vs {}",
            p.composite_front.rotation,
            expect
        );
        // Facing left: `atan2(10, -10) + PI/4 + PI/2`, and `num8` flips.
        p.facing = -1;
        p.apply_hold_composite_arms(&held);
        let item_rot = 10.0f32.atan2(-10.0) + std::f32::consts::FRAC_PI_4
            + std::f32::consts::FRAC_PI_2;
        let num9 = item_rot - std::f32::consts::FRAC_PI_4 + std::f32::consts::PI;
        let expect = num9 + std::f32::consts::PI / 10.0;
        assert!(
            (p.composite_front.rotation - expect).abs() < 1e-6,
            "{} vs {}",
            p.composite_front.rotation,
            expect
        );
    }

    /// `Player.StickyMovement` (`Player.cs:22910-23087`): the honey block's drag
    /// — the ±1 X cap with the 0.85 damp and the ±1/-5 Y cap with the 0.3 damp
    /// (`:22975-23028`), the gel mount's halving (`:22965-22972`), and the
    /// cobweb's `stickyBreak` break (`:22946`).
    fn sticky_world(kind: u16) -> World {
        use tr_core::tile::FLAG_ACTIVE;
        let mut w = World::new("t".into(), 64, 64);
        let i = w.index(10, 20);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    /// The body whose feet rest on tile row 20 (the sticky probe reaches it
    /// with `pos_y - 0.01`, exactly like `Collision.cs:3399`).
    fn sticky_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 164.0;
        p.y = 20.0 * 16.0 - HEIGHT + 1.0;
        p
    }

    #[test]
    fn sticky_movement_damps_the_honey_block_velocity() {
        let w = sticky_world(229);
        let mut rng = tr_core::UnifiedRandom::default();
        let mut p = sticky_player(&w);
        p.vx = 5.0;
        p.vy = 5.0;
        p.sticky_movement(&w, false, false, &mut rng);
        // `:22975-22990`: X capped to 1 then ×0.85.
        assert!((p.vx - 0.85).abs() < 1e-6, "vx={}", p.vx);
        // `:23012-23027`: Y capped to 1 then ×0.3 (the falling arm).
        assert!((p.vy - 0.3).abs() < 1e-6, "vy={}", p.vy);
        // A honey block keeps the jump (`:22961-22964`); the web clears it.
        let mut p = sticky_player(&w);
        p.jump = 10;
        p.sticky_movement(&w, false, false, &mut rng);
        assert_eq!(p.jump, 10, "honey keeps the jump");
        // The gel/slime mount (`flag2`) halves instead (`:22967`).
        let mut p = sticky_player(&w);
        p.vx = 5.0;
        p.vy = 5.0;
        p.sticky_movement(&w, false, true, &mut rng);
        assert!((p.vx - 2.5).abs() < 1e-6, "vx={}", p.vx);
        assert!((p.vy - 2.5).abs() < 1e-6, "vy={}", p.vy);
    }

    #[test]
    fn sticky_movement_breaks_a_cobweb_and_clears_the_counter() {
        let w = sticky_world(51);
        let mut rng = tr_core::UnifiedRandom::default();
        let mut p = sticky_player(&w);
        p.vx = 1.0;
        // One tick sets the counter; the roll is `[20,100)` so a counter at
        // 150 always beats it (`:22946`).
        p.sticky_movement(&w, false, false, &mut rng);
        assert_eq!(p.sticky_break, 1);
        assert!(p.take_sticky_kill().is_none());
        p.sticky_break = 150;
        p.sticky_movement(&w, false, false, &mut rng);
        assert_eq!(p.take_sticky_kill(), Some((10, 20)));
        assert_eq!(p.sticky_break, 0);
        // The web also cancels the jump and damps X the same way
        // (`:22961-22990`), and no stick clears the counter (`:23085`).
        assert_eq!(p.jump, 0);
        let mut p = sticky_player(&World::new("t".into(), 64, 64));
        p.sticky_break = 40;
        p.sticky_movement(&World::new("t".into(), 64, 64), false, false, &mut rng);
        assert_eq!(p.sticky_break, 0);
    }

    /// `:22918-22921` + `:22956-22958`: a cart above 5 px/tick skips the drag
    /// (but the break roll still happens — the C#'s bitwise `|`).
    #[test]
    fn a_fast_cart_ignores_the_sticky_drag() {
        let w = sticky_world(229);
        let mut rng = tr_core::UnifiedRandom::default();
        let mut p = sticky_player(&w);
        p.vx = 6.0;
        p.vy = 4.0;
        p.sticky_movement(&w, true, false, &mut rng);
        assert_eq!((p.vx, p.vy), (6.0, 4.0), "the drag is skipped");
    }

    /// `:23030-23080`: the honey puffs land on the side away from the body's
    /// centre, once per five vertical-moving ticks, and the two rows below when
    /// they are still honey.
    #[test]
    fn the_honey_puffs_sit_beside_the_body() {
        use tr_core::tile::FLAG_ACTIVE;
        let mut w = World::new("t".into(), 64, 64);
        for row in 20..=22 {
            let i = w.index(10, row);
            w.tiles[i].kind = 229;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut rng = tr_core::UnifiedRandom::default();
        let mut p = sticky_player(&w);
        // 60 ticks guarantees several `Next(5) == 0` hits on a fixed seed.
        let mut puffs = Vec::new();
        for _ in 0..60 {
            p.vy = 1.0;
            p.sticky_movement(&w, false, false, &mut rng);
            puffs.append(&mut p.take_sticky_dust());
        }
        assert!(!puffs.is_empty(), "the honey block puffs");
        let p0 = puffs[0];
        assert_eq!((p0.kind, p0.w, p0.h), (153, 4, 16));
        assert!(p0.post_arms);
        // `tile.x * 16 = 160 < centre 174` picks the left arm (`:23034`).
        assert_eq!(p0.pos.x, p.x - 4.0);
        assert!(puffs.iter().any(|d| d.pos.y == 21.0 * 16.0) || puffs.len() >= 1);
    }

    /// S1-E-21f — the extra-jump chain queues the case-16 play at the
    /// top-left (`Player.cs:20793` et al.); the Fart queues the Item-16
    /// style façade instead (`:20846`). The unarmed Cloud fall-through
    /// (`:20746`) stays silent — it clears the flag without a jump.
    #[test]
    fn s1_e_21f_the_extra_jump_queues_the_double_jump_play() {
        let w = ground_world();
        // The air-tap preamble (the `tap_air` shape) with the position
        // captured before the triggering tick — the queue stores the
        // probe-time top-left, the tick's integration moves the body after.
        let air_tap = |w: &World, p: &mut Player| -> (f32, f32) {
            p.update(w, PlayerInput { jump: true, ..Default::default() });
            p.update(w, PlayerInput::default());
            for _ in 0..3 {
                p.update(w, PlayerInput::default());
            }
            let pos = (p.x, p.y);
            p.update(w, PlayerInput { jump: true, ..Default::default() });
            pos
        };
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // The ground jump re-arms the chain; no extra-jump play yet.
        p.update(&w, PlayerInput { jump: true, ..Default::default() });
        assert!(p.take_double_jump_sound().is_none(), "the ground jump is silent");
        let pos = air_tap(&w, &mut p);
        assert_eq!(
            p.take_double_jump_sound(),
            Some(pos),
            "the Cloud arm queues the top-left position"
        );
        assert!(p.take_double_jump_sound().is_none(), "one-shot");
        assert!(p.take_fart_sound().is_none(), "the Cloud is not the Fart");
        // The Fart arm queues the Item-16 façade, not the case-16 play.
        let mut p = settled_with_jumps(&w, &[ExtraJump::Fart]);
        p.update(&w, PlayerInput { jump: true, ..Default::default() });
        let pos = air_tap(&w, &mut p);
        let (idx, fx, fy) = p.take_fart_sound().expect("the Fart queues Item16");
        assert_eq!(crate::soundid::SOUND_STYLES[idx as usize].name, "Item16");
        assert_eq!((fx, fy), pos);
        assert!(p.take_double_jump_sound().is_none());
        // Unarmed: no option means the chain gate never opens — silent.
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.update(&w, PlayerInput { jump: true, ..Default::default() });
        let _ = air_tap(&w, &mut p);
        assert!(p.take_double_jump_sound().is_none(), "no option, no play");
        assert!(p.take_fart_sound().is_none());
    }

    /// S1-E-21f — the liquid entry/exit splash queue (`Player.cs:27772-27944`):
    /// the wet edge plays once per `wetCount` cycle (10 ticks), water style 0
    /// entry / 0 exit, honey 1, shimmer 2 entry / 3 exit, lava 1. The tail
    /// decrement re-arms the gate after the cooldown.
    #[test]
    fn s1_e_21f_the_wet_edges_queue_the_splash_styles() {
        let pool = |kind: tr_core::tile::LiquidKind| {
            let mut w = ground_world();
            for x in 0..64 {
                for y in 40..60 {
                    let i = w.index(x, y);
                    w.tiles[i].liquid = 255;
                    w.tiles[i].set_liquid_kind(kind);
                }
            }
            w.spawn_y = 44;
            w
        };
        use tr_core::tile::LiquidKind;
        // Water: entry style 0, and after leaving (teleport to dry air above
        // the pool) the exit style 0. The queue stores the probe-time
        // top-left — capture before the tick's integration moves the body.
        let w = pool(LiquidKind::Water);
        let mut p = Player::spawn(&w);
        let spawn_pos = (p.x, p.y);
        p.update(&w, PlayerInput::default());
        assert_eq!(
            p.take_splash_sound(),
            Some((0, spawn_pos.0, spawn_pos.1)),
            "the water entry plays style 0 at the top-left"
        );
        // The cooldown suppresses re-entry churn: hop out and back within
        // the window — no second play until wetCount runs out.
        let dry_y = 20.0 * 16.0;
        let wet_y = p.y;
        p.y = dry_y;
        p.update(&w, PlayerInput::default());
        assert!(p.take_splash_sound().is_none(), "wetCount still armed");
        p.y = wet_y;
        p.update(&w, PlayerInput::default());
        assert!(p.take_splash_sound().is_none(), "the cooldown holds");
        // Run the cooldown out in the dry, then the exit edge fired earlier
        // is gone — a fresh entry plays again.
        p.y = dry_y;
        for _ in 0..12 {
            p.update(&w, PlayerInput::default());
            p.y = dry_y; // hold above the pool against gravity
        }
        let _ = p.take_splash_sound();
        p.y = wet_y;
        p.vy = 0.0;
        let reentry_pos = (p.x, p.y);
        p.update(&w, PlayerInput::default());
        assert_eq!(
            p.take_splash_sound(),
            Some((0, reentry_pos.0, reentry_pos.1)),
            "re-entry after the cooldown"
        );
        // Honey: style 1 entry. Shimmer: style 2 entry / style 3 exit.
        for (kind, style) in [(LiquidKind::Honey, 1), (LiquidKind::Shimmer, 2), (LiquidKind::Lava, 1)] {
            let w = pool(kind);
            let mut p = Player::spawn(&w);
            p.update(&w, PlayerInput::default());
            assert_eq!(
                p.take_splash_sound().map(|(s, ..)| s),
                Some(style),
                "{kind:?} entry style"
            );
        }
        // The shimmer exit reads the OLD shimmerWet flag — style 3. The
        // exit edge needs wetCount == 0: enter, wait the cooldown out UNDER
        // water (the counter ticks regardless), then leave.
        let w = pool(LiquidKind::Shimmer);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        let _ = p.take_splash_sound(); // entry, style 2
        for _ in 0..12 {
            p.update(&w, PlayerInput::default());
        }
        p.y = dry_y;
        p.update(&w, PlayerInput::default());
        assert_eq!(
            p.take_splash_sound().map(|(s, ..)| s),
            Some(3),
            "the shimmer exit plays style 3"
        );
    }

    /// S1-E-21f — the drowning gasp fires ONCE on the breath 1→0 edge
    /// (`Player.cs:23255-23258`), not on every subsequent drain tick.
    #[test]
    fn s1_e_21f_the_drown_gasp_fires_once_on_the_zero_edge() {
        let mut w = ground_world();
        for x in 0..64 {
            for y in 40..60 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        w.spawn_y = 44;
        let mut p = Player::spawn(&w);
        let mut gasps = 0;
        for _ in 0..1600 {
            p.update(&w, PlayerInput::default());
            if p.take_drown_sound() {
                gasps += 1;
            }
        }
        assert_eq!(p.breath, 0);
        assert_eq!(gasps, 1, "the gasp is the 1->0 edge, not the damage loop");
    }

    /// S1-E-21f — the wing flap's Item-32 play (`Player.cs:26835-26858`):
    /// the default frame machine (`:30466-30479`) wraps 0..3 every 5 flying
    /// ticks; the frame-3 edge plays ONCE through the `flapSound` latch.
    /// The excluded wings (4/22/24/28/30/33/45) never play.
    #[test]
    fn s1_e_21f_the_wing_flap_plays_once_per_frame_three_edge() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.equipped[3] = 492; // Demon Wings — wings_logic 1, the plain machine
        p.wing_time = 100.0;
        let held = PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held); // the ground jump
        let mut flaps = 0;
        // The machine ticks through the 15-tick jump phase too (the C#'s
        // `wingFlap || jump > 0`), so the frame-3 edges land every 20 held
        // ticks — the audible ones need flag19 (the jump drained).
        for _ in 0..80 {
            p.update(&w, held);
            if let Some((idx, _, _)) = p.take_flap_sound() {
                assert_eq!(crate::soundid::SOUND_STYLES[idx as usize].name, "Item32");
                flaps += 1;
            }
        }
        // 80 held ticks: the frame-3 edges at ~35/55/75 — each played
        // exactly once (the latch).
        assert!(flaps >= 2, "the flap edges played: {flaps}");
        assert!(flaps <= 4, "the latch holds one play per edge: {flaps}");
        // The excluded wings stay silent through the same flight. Item 1866
        // (the Festive Wings row) grants wings_logic 22 — excluded (`:26835`).
        let mut q = Player::spawn(&w);
        for _ in 0..120 {
            q.update(&w, PlayerInput::default());
        }
        q.equipped[3] = 1866;
        q.wing_time = 100.0;
        q.update(&w, held);
        assert_eq!(q.wings_logic, 22, "the wing row grants wings 22");
        for _ in 0..50 {
            q.wing_time = 100.0;
            q.update(&w, held);
            assert!(q.take_flap_sound().is_none(), "wings 22 never flap-play");
        }
    }

    // -- S1-P-i40: the wings/flight accessory interaction fixes -------------
    // Witnesses WRITTEN NOT RUN (code-only mode). Each pins a C#-verified
    // correction against `Player.cs` / `WingStatsInitializer.cs` / `Item.cs`.

    /// `WingStats.cs:3` + `Player.cs:14335-14338`: `GetWingStats` returns the
    /// struct `default` (FlyTime 100) for an out-of-range id, NOT a zero
    /// budget. The port's `_ => (0, ..)` gave wings 5586/5686/6140 (ids
    /// 47/50/51, now in the 150/7 chain) and any unknown id zero flight.
    #[test]
    fn s1_p_i40_wing_stats_default_is_the_struct_fly_time() {
        // Out-of-range ids take the WingStats default FlyTime = 100.
        assert_eq!(Player::wing_stats(0).0, 100, "id 0 -> default 100");
        assert_eq!(Player::wing_stats(52).0, 100, "id 52 (Count) -> default");
        assert_eq!(Player::wing_stats(99).0, 100, "unknown id -> default");
        // The 150/7 chain covers 47/50/51 (WingStatsInitializer.cs:61).
        for id in [47, 50, 51] {
            assert_eq!(Player::wing_stats(id).0, 150, "id {id} fly time");
            assert_eq!(Player::wing_stats(id).1, 7.0, "id {id} run override");
        }
    }

    /// `Item.cs:31096-31105`: the four Nebula wing cases share
    /// `wingSlot = 29 + type - 3468`. The port dropped them, so the 29/32
    /// ascent constants were unreachable via equip.
    #[test]
    fn s1_p_i40_the_nebula_wings_map_to_slots_29_to_32() {
        assert_eq!(Player::wing_slot(3468), 29);
        assert_eq!(Player::wing_slot(3469), 30);
        assert_eq!(Player::wing_slot(3470), 31);
        assert_eq!(Player::wing_slot(3471), 32);
        // The grant reads the slot's budget (29/32 = 180, 30/31 = 180).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 3468;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 29);
        assert_eq!(p.wing_time_max, 180);
    }

    /// `Player.cs:22091-22097` (44) vs `:22137-22142` (45): the two hover
    /// wings have DISTINCT ascent rows — 44 = (0.125, 0.85, 2.75, 1, 0.15),
    /// 45 = (0.1, 0.95, 4.5, 1, 0.15). The port shared 45's row for both.
    #[test]
    fn s1_p_i40_wings_44_and_45_have_distinct_ascent_rows() {
        assert_eq!(Player::wing_constants(44), (0.125, 0.85, 2.75, 1.0, 0.15));
        assert_eq!(Player::wing_constants(45), (0.1, 0.95, 4.5, 1.0, 0.15));
        // 30/31 keep the base 0.1 impulse here; the 0.15 boost is
        // hover-down-gated at the call site (`:22194-22197`).
        assert_eq!(Player::wing_constants(30).0, 0.1);
        assert_eq!(Player::wing_constants(31).0, 0.1);
    }

    /// `Player.cs:14771-14775` + `:22239-22242`: the Soaring Insignia (4989)
    /// arms `empressBrooch`, which refills the wing budget to max on every
    /// flying tick it stays non-zero (infinite flight).
    #[test]
    fn s1_p_i40_the_soaring_insignia_grants_infinite_flight() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.equipped[3] = 492; // Demon Wings (slot 1, 100 ticks)
        p.equipped[4] = 4989; // Soaring Insignia
        p.update(&w, PlayerInput::default());
        assert!(p.empress_brooch, "the brooch flag arms");
        // Fly: the budget drains to max each tick, never below max-1. The
        // initial jump burst (jump > 0) does NOT fly — wait it out first
        // (the C# flag19 needs `jump == 0`, `:26786`).
        let held = PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held); // the ground jump
        for _ in 0..30 {
            p.update(&w, held); // the burst drains (jumpHeight 15)
        }
        for _ in 0..200 {
            p.update(&w, held);
            assert!(
                p.wing_time >= p.wing_time_max as f32 - 1.0,
                "the brooch refills: wing_time={} max={}",
                p.wing_time,
                p.wing_time_max
            );
        }
    }

    /// `Player.cs:25577-25589` + `:3990-3999`: a shell (861/3110/497) while
    /// wet turns the player merman, which gates `CanUseWingAbilities` (the
    /// hover dampening) and `WingAirLogicTweaks`.
    #[test]
    fn s1_p_i40_the_merman_shell_gates_the_wing_abilities() {
        // Dry: the shell arms accMerman but the player is not merman.
        let dry = ground_world();
        let mut p = Player::spawn(&dry);
        p.equipped[3] = 497; // Neptune's Shell
        p.update(&dry, PlayerInput::default());
        assert!(p.acc_merman, "the shell arms accMerman");
        assert!(!p.merman, "dry: not merman");
        // Wet (a water pool): the shell turns the player merman.
        let wet = liquid_pool_world(tr_core::tile::LiquidKind::Water);
        let mut q = Player::spawn(&wet);
        q.equipped[3] = 497;
        for _ in 0..30 {
            q.update(&wet, PlayerInput::default());
        }
        assert!(q.wet, "the player settles in the water");
        assert!(q.merman, "wet + shell: merman");
        // Gate V3: probe the hover-damp gate DIRECTLY (`hover_dampening`,
        // `Player.cs:27404-27425` gated on `CanUseWingAbilities` `:3994`).
        // A full `update` tick would run the merman swim (the wet jump arm
        // rewrites vy), masking the damp gate this witness pins. Set the
        // hover state by hand and call the damp fn: merman refuses it, so
        // the descent vy is NOT dampened into the float band.
        q.equipped[3] = 1866; // Bee Wings (slot 22, hover)
        q.equipped[4] = 497;
        q.update(&wet, PlayerInput::default());
        assert!(q.merman, "still merman");
        q.wings_logic = 22;
        q.wing_time = 170.0;
        q.control_jump = true;
        q.control_down = true;
        q.vy = 3.0;
        q.hover_dampening();
        // Merman blocks the damp: vy stays 3.0 (a non-merman hover would
        // damp it to 3.0*0.9 = 2.7, then the float band snaps it to 1e-5).
        assert_eq!(q.vy, 3.0, "merman blocks the hover damp: vy={}", q.vy);
        // Control: a non-merman player with the same hover state IS dampened.
        q.merman = false;
        q.vy = 3.0;
        q.hover_dampening();
        assert!(q.vy < 3.0, "non-merman hover damps the descent: vy={}", q.vy);
    }

    /// `Player.cs:27418-27425`: Betsy's wings (37) apply a SECOND ×0.92 damp
    /// after the shared ×0.9 — the port dropped it.
    #[test]
    fn s1_p_i40_betsy_wings_damp_twice() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 3883; // Betsy's Wings (slot 37)
        p.update(&w, PlayerInput::default()); // the grant runs
        assert_eq!(p.wings_logic, 37);
        p.wing_time = 150.0;
        p.vy = 3.0;
        let held = PlayerInput { jump: true, down: true, ..Default::default() };
        p.update(&w, held);
        // 3.0 * 0.9 * 0.92 = 2.484 (both passes), vs 2.7 with only the first.
        assert!(p.vy < 2.6, "the second damp fires: vy={}", p.vy);
    }
}

/// S1-P-i05 — the useStyle-3 (Thrust) witnesses: the stab arm's
/// `itemLocation` maths (`Player.cs:51031-51063`), the
/// `ItemCheck_GetMeleeHitbox` rect (`:45164-45189`) and the run-key
/// direction lock (`:19837`). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod s1_p_i05_thrust_tests {
    use super::*;

    /// The wind-up third hides the item offscreen with the cocked
    /// rotation (`Player.cs:51033-51038`) — the gate is the C#'s 0.666
    /// literal, NOT 2/3: remaining 0.6665 is still wind-up, 0.666 is not.
    #[test]
    fn s1_p_i05_the_windup_hides_the_item_at_the_csharp_0666_gate() {
        // remaining > 0.666: offscreen, rotation -1.3 * dir.
        let (loc, rot) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.9, 44.0, 1.0, 0.0, 946);
        assert_eq!(loc, [-1000.0, -1000.0]);
        assert!((rot - -1.3).abs() < 1e-6);
        let (loc, rot) = thrust_item_placement(100.0, 200.0, 20.0, -1.0, 0.7, 44.0, 1.0, 0.0, 946);
        assert_eq!(loc, [-1000.0, -1000.0]);
        assert!((rot - 1.3).abs() < 1e-6, "facing left mirrors the cock");
        // The boundary: 0.6665 > 0.666 (wind-up), 0.666 == 0.666 (stab).
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.6665, 44.0, 1.0, 0.0, 946);
        assert_eq!(loc, [-1000.0, -1000.0], "0.6665 is still the wind-up");
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.666, 44.0, 1.0, 0.0, 946);
        assert_ne!(loc, [-1000.0, -1000.0], "0.666 is the stab arm");
        // The discriminator: 0.6661 sits between the C# literal 0.666 and
        // 2/3 (0.6667). The correct 0.666 gate treats it as wind-up
        // (HIDDEN); a 2/3 gate would wrongly REVEAL it (stab). So the
        // correct port hides 0.6661 — assert_eq, the opposite of the
        // pre-fix 2/3 behaviour. (0.6665 above pins the same band.)
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.6661, 44.0, 1.0, 0.0, 946);
        assert_eq!(loc, [-1000.0, -1000.0], "0.6661 > 0.666: the C# gate hides it, a 2/3 gate would not");
    }

    /// The extension maths (`Player.cs:51041-51053`), facing right:
    /// `loc.X = pos.X + width/2 + (frame.W/2 - 4)*dir - num14` with
    /// `num14 = remaining * frame.W * dir * scale * 1.2 - 10*dir` — the
    /// item pushes FORWARD as `itemAnimation` falls. The umbrella 946's
    /// 44x44 frame (`Item.cs:12321`), scale 1: at remaining 0.5,
    /// num14 = 0.5*44*1.2 - 10 = 16.4 (> 4, no snap), loc.X = 100 + 10 +
    /// 18 - 16.4 = 111.6; the umbrella nudge -6 (`:51056`) lands 105.6.
    /// Y = pos.Y + 24 (`:51042`).
    #[test]
    fn s1_p_i05_the_stab_extends_then_retracts_verbatim() {
        // Mid-stab (remaining 0.5): num14 = 16.4, no snap.
        let (loc, rot) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 0.0, 946);
        assert!((loc[0] - 105.6).abs() < 1e-4, "the mid-stab tip: {}", loc[0]);
        assert!((loc[1] - 224.0).abs() < 1e-6);
        assert!((rot - 0.8).abs() < 1e-6, "the stab rotation 0.8*dir");
        // The extension tail (remaining 0.1): num14 = 0.1*44*1.2 - 10 =
        // -4.72 (< 4, facing right) SNAPS to 8 (`:51048-51051`) — the
        // full extension holds: loc.X = 128 - 8 - 6 = 114.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.1, 44.0, 1.0, 0.0, 946);
        assert!((loc[0] - 114.0).abs() < 1e-4, "the snap-to-8 extension: {}", loc[0]);
        // The no-snap arm near the boundary: at remaining 0.3, num14 =
        // 0.3*44*1.2 - 10 = 5.84 (>= 4, no snap, `:51048`'s `< 4f`);
        // loc.X = 128 - 5.84 - 6 = 116.16. (The exact-4 boundary itself is
        // not f32-pinnable: remaining = 14/52.8 rounds DOWN in f32 and
        // num14 lands 3.9999990 — the C# float would snap it too, so no
        // witness may claim "num14 == 4 keeps the raw value".)
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.3, 44.0, 1.0, 0.0, 946);
        assert!((loc[0] - 116.16).abs() < 1e-3, "num14 >= 4 keeps the raw value");
        // Facing left mirrors: num14 = 0.5*44*(-1)*1.2 - 10*(-1) = -16.4
        // (> -4? no: -16.4 < -4, no snap); loc.X = 100 + 10 - 18 + 16.4 + 6 = 114.4.
        let (loc, rot) = thrust_item_placement(100.0, 200.0, 20.0, -1.0, 0.5, 44.0, 1.0, 0.0, 946);
        assert!((loc[0] - 114.4).abs() < 1e-4, "the mirrored tip: {}", loc[0]);
        assert!((rot - -0.8).abs() < 1e-6);
        // Facing left's snap: num14 > -4 snaps to -8 (`:51044-51047`).
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, -1.0, 0.1, 44.0, 1.0, 0.0, 946);
        assert!((loc[0] - (92.0 + 8.0 + 6.0)).abs() < 1e-4, "the left snap-to--8");
        // The scale factor (`:51043` GetAdjustedItemScale): scale 1.1
        // stretches the extension — num14 = 0.5*44*1.1*1.2 - 10 = 19.04.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.1, 0.0, 946);
        assert!((loc[0] - (128.0 - 19.04 - 6.0)).abs() < 1e-3, "the scaled extension");
        // The mountOffset rides Y (`:51042`).
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 5.0, 946);
        assert!((loc[1] - 229.0).abs() < 1e-6);
        // A non-umbrella style-3 item takes no nudge (`:51054` gates on
        // 946/4707) — the hypothetical row keeps loc.X = 111.6.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 0.0, 1);
        assert!((loc[0] - 111.6).abs() < 1e-4, "no nudge off the umbrella pair");
        // The Tragic Umbrella 4707 shares the nudge.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 0.0, 4707);
        assert!((loc[0] - 105.6).abs() < 1e-4);
    }

    /// `ItemCheck_GetMeleeHitbox`'s style-3 rect (`Player.cs:45164-45189`)
    /// — the wind-up `dontAttack`, the corner flips, the 1.4/0.6 shape
    /// and the umbrella +14/-10 pair, on the 44x44 frame at scale 1.
    #[test]
    fn s1_p_i05_the_stab_hitbox_matches_the_csharp_rect() {
        // The wind-up never attacks (`:45168-45172`).
        assert!(thrust_melee_hitbox([100.0, 200.0], 44.0, 44.0, 1.0, 1, 0.9, 946).is_none());
        assert!(thrust_melee_hitbox([100.0, 200.0], 44.0, 44.0, 1.0, 1, 0.6665, 946).is_none());
        // Facing right, remaining 0.5: the placement lands (105.6, 224)
        // (the witness above); the rect anchors (105, 224), 44x44;
        // gravDir 1: Y -= 44 -> 180; the stab shape: W = 61 (44*1.4 =
        // 61.6 truncated), Y += 26 (44*0.6 = 26.4), H = 26; the umbrella
        // pair: H += 14 -> 40, W -= 10 -> 51.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 0.0, 946);
        let r = thrust_melee_hitbox(loc, 44.0, 44.0, 1.0, 1, 0.5, 946).expect("the stab attacks");
        assert_eq!(r, (105, 180 + 26, 51, 40), "the right-facing umbrella rect");
        // Facing left: the placement lands 114.4; X -= W (44) -> 70;
        // the left 1.4-shift: X -= (61.6 - 44) as int = 17 -> 53; the
        // umbrella's +10 -> 63. Y/H identical.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, -1.0, 0.5, 44.0, 1.0, 0.0, 946);
        let r = thrust_melee_hitbox(loc, 44.0, 44.0, 1.0, -1, 0.5, 946).expect("the stab attacks");
        assert_eq!(r, (53 + 10, 206, 51, 40), "the left-facing umbrella rect");
        // A non-umbrella style-3 rect keeps the plain 1.4/0.6 shape
        // (`:45180` gates on 946/4707): W 61, H 26, no +14/-10.
        let (loc, _) = thrust_item_placement(100.0, 200.0, 20.0, 1.0, 0.5, 44.0, 1.0, 0.0, 1);
        let r = thrust_melee_hitbox(loc, 44.0, 44.0, 1.0, 1, 0.5, 1).expect("the stab attacks");
        assert_eq!(r, (111, 206, 61, 26), "the plain stab shape");
        // The scale sizes the rect BEFORE the shape (`:45128-45130`):
        // scale 1.1 -> 48x48 (44*1.1 = 48.4 truncated); W = 67 (48*1.4 =
        // 67.2), Y += 28 (48*0.6 = 28.8), H = 28; umbrella H 42, W 57.
        let r = thrust_melee_hitbox([100.0, 200.0], 44.0, 44.0, 1.1, 1, 0.5, 946).expect("attacks");
        assert_eq!(r, (100, 200 - 48 + 28, 57, 42), "the scaled rect");
    }

    /// The direction lock (`Player.cs:19837`): a non-useTurn swing locks
    /// the facing against the run keys; useTurn items and the idle state
    /// turn freely; the sandstorm owns its cadence.
    #[test]
    fn s1_p_i05_the_swing_locks_the_facing_unless_useturn() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        p.facing = 1;
        // Mid-swing with the umbrella (useTurn false): the left key does
        // NOT turn the player. (M1 re-sync: the i12 PlayerInput channel.)
        p.control_left = true;
        p.control_right = false;
        p.horizontal_movement(PlayerInput {
            item_in_use: true,
            held_use_turn: false,
            ..Default::default()
        });
        assert_eq!(p.facing, 1, "the stab locks the facing");
        // useTurn (the pickaxe's row) turns mid-swing.
        p.horizontal_movement(PlayerInput {
            item_in_use: true,
            held_use_turn: true,
            ..Default::default()
        });
        assert_eq!(p.facing, -1, "useTurn exempts the lock");
        // Idle (itemAnimation == 0) turns freely.
        p.facing = 1;
        p.horizontal_movement(PlayerInput::default());
        assert_eq!(p.facing, -1, "the idle turn is unlocked");
        // The sandstorm dash owns the turn cadence (`:19837` !sandStorm).
        p.facing = 1;
        p.sand_storm = true;
        p.horizontal_movement(PlayerInput::default());
        assert_eq!(p.facing, 1, "the sandstorm gates the run-key turn");
    }
}

/// S1-P-i27 — the minion target command witnesses (written, NOT run — the
/// code-only branch carries no compiler): the summoner accessory
/// target-priority fold (`Player.cs:12970-12984` + the grant rows
/// `:14509-14527`/`:14585`/`:14601-14609`), the whip-range consumers
/// (`:13852-13855`, `:13876-13881`, `ArmorSetBonuses.cs:15`), the rest-spot
/// aim (`MinionRestTargetAim`, `:56935-56997`) and the down-cardinal
/// double-tap/hold driver (`:25022-25073`, `KeyDoubleTap` `:56764-56787`,
/// `KeyHoldDown` `:56894-56912`).
#[cfg(test)]
mod s1_p_i27_tests {
    use super::*;
    use super::tests::ground_world;

    /// Two settled ticks with the accessory worn — the same fixture shape
    /// `player_with_accessory` uses (the reset clears, the pass re-grants).
    fn worn(kind: u16) -> Player {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = kind;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    /// `Player.cs:14510` + `:12970-12975` — the Silver Bracer: the fold
    /// adds +1 tag slot and +2f duration (the `minionDamage += 0f` half
    /// changes nothing).
    #[test]
    fn s1_p_i27_the_silver_bracer_scales_the_tag_stack() {
        let p = worn(6156);
        assert!(p.acc_silver_bracer, "the grant row (:14510)");
        assert_eq!(p.tag_effects.max_tag_effects(), 2, "(:12974)");
        assert!((p.tag_effects.tag_effect_duration() - 3.0).abs() < 1e-4, "(:12973)");
        assert!((p.minion_damage - 1.0).abs() < 1e-4, "the += 0f no-op (:12972)");
        assert!((p.whip_range_multiplier - 1.0).abs() < 1e-4, "no whip-range row");
    }

    /// `Player.cs:14513` + `:12976-12980` — the Mobius Strip: +1 tag slot
    /// and +0.1 whip range.
    #[test]
    fn s1_p_i27_the_mobius_strip_scales_the_tags_and_the_whip_range() {
        let p = worn(6158);
        assert!(p.acc_mobius_strip);
        assert_eq!(p.tag_effects.max_tag_effects(), 2, "(:12978)");
        assert!((p.tag_effects.tag_effect_duration() - 1.0).abs() < 1e-4);
        assert!((p.whip_range_multiplier - 1.1).abs() < 1e-4, "(:12979)");
    }

    /// `Player.cs:14516` + `:12981-12984` — the Wicked Armlet: +1 tag slot
    /// only.
    #[test]
    fn s1_p_i27_the_wicked_armlet_adds_one_tag_slot() {
        let p = worn(6159);
        assert!(p.acc_wicked_armlet);
        assert_eq!(p.tag_effects.max_tag_effects(), 2, "(:12983)");
        assert!((p.whip_range_multiplier - 1.0).abs() < 1e-4);
    }

    /// `Player.cs:14523` — the Twilight Grasp arms BOTH flags: the fold
    /// stacks +2 slots and +2f duration (the bracer's rows then the
    /// armlet's, `:12970-12984`'s if-order).
    #[test]
    fn s1_p_i27_the_twilight_grasp_arms_both_folds() {
        let p = worn(6163);
        assert!(p.acc_silver_bracer && p.acc_wicked_armlet);
        assert_eq!(p.tag_effects.max_tag_effects(), 3);
        assert!((p.tag_effects.tag_effect_duration() - 3.0).abs() < 1e-4);
    }

    /// `Player.cs:14519-14520` — the Ouroboros Ring grants the Mobius flag
    /// (the `accSnakeBand` half has no port consumer).
    #[test]
    fn s1_p_i27_the_ouroboros_ring_rides_the_mobius_fold() {
        let p = worn(6162);
        assert!(p.acc_mobius_strip);
        assert_eq!(p.tag_effects.max_tag_effects(), 2);
        assert!((p.whip_range_multiplier - 1.1).abs() < 1e-4);
    }

    /// `Player.cs:14605-14609` — the Silver Shield's bracer flag joins its
    /// knockback/fire-walk row (the defense pass grants it).
    #[test]
    fn s1_p_i27_the_silver_shield_arms_the_bracer_fold() {
        let p = worn(6183);
        assert!(p.no_knockback && p.fire_walk, "the defense halves");
        assert!(p.acc_silver_bracer, "(:14606)");
        assert_eq!(p.tag_effects.max_tag_effects(), 2);
        assert!((p.tag_effects.tag_effect_duration() - 3.0).abs() < 1e-4);
    }

    /// `Player.cs:14585`/`:14601-14603` — the Armlet of Ruin and the Wicked
    /// Claws both arm the armlet fold; the Claws add +0.12 melee speed.
    #[test]
    fn s1_p_i27_the_armlet_combos_arm_the_wicked_fold() {
        let p = worn(6179);
        assert!(p.acc_wicked_armlet);
        assert_eq!(p.tag_effects.max_tag_effects(), 2);
        let p = worn(6182);
        assert!(p.acc_wicked_armlet);
        assert_eq!(p.tag_effects.max_tag_effects(), 2);
        assert!(
            (p.melee_speed_bonus - 0.12).abs() < 1e-4,
            "(:14603) — the raw grant; the i12 melee_speed(held_kind) accessor inverts it"
        );
    }

    /// The reset half (`Player.cs:18498`/`:18517-18522`): removing the
    /// accessory drops the scalars back to the bases on the next tick.
    #[test]
    fn s1_p_i27_the_scalars_reset_when_the_accessory_leaves() {
        let w = ground_world();
        let mut p = worn(6156);
        assert_eq!(p.tag_effects.max_tag_effects(), 2);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.acc_silver_bracer);
        assert_eq!(p.tag_effects.max_tag_effects(), 1, "(:18517)");
        assert!((p.tag_effects.tag_effect_duration() - 1.0).abs() < 1e-4, "(:18518)");
        assert!((p.whip_range_multiplier - 1.0).abs() < 1e-4, "(:18498)");
    }

    /// `Player.cs:13852-13855` — the Tiki Mask's +0.1 whip range rides the
    /// armor-piece pass (`grant_summon_caps`' seam — here the tr-core
    /// `set_stats` row carries only the SET half, the piece half is the
    /// world_sim grant; this witness pins the set half via `set_stats`).
    #[test]
    fn s1_p_i27_the_tiki_set_bonus_lifts_the_whip_range() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[0] = 1159;
        p.equipped[1] = 1160;
        p.equipped[2] = 1161;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        // The set row's +0.2 (`ArmorSetBonuses.cs:15`); the piece +0.1
        // (`:13854`) rides `grant_summon_caps` (the world_sim seam, not
        // this update), so the player-update alone sees the set half.
        assert!((p.whip_range_multiplier - 1.2).abs() < 1e-4);
        assert_eq!(p.max_minions, 2, "the set's maxMinions++ (:14)");
    }

    /// `Player.MinionRestTargetAim` (`Player.cs:56935-56997`) over the
    /// ground fixture (solid from row 60): the mouse inside the ground run
    /// (row 62) scans up — rows 62/61/60 solid, row 59 air → num4 = −3 —
    /// and down (the whole 20-row run is solid → num7 = 19), so
    /// num7(19) > −num4(3) picks the UP side: num3 = num4 − 2 = −5, the
    /// row 57 candidate; the 5-tile drop window (57..62) finds the ground
    /// at 60 → the spot is `(x*16+8, 57*16)` — three tiles above the
    /// surface.
    #[test]
    fn s1_p_i27_the_rest_aim_hovers_above_the_ground_run() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // The player near the column so the 1000-px gate passes.
        p.x = 20.0 * 16.0;
        p.y = 55.0 * 16.0;
        // The mouse inside the ground run (row 62).
        p.minion_rest_target_aim(&w, Vec2::new(20.0 * 16.0 + 8.0, 62.0 * 16.0 + 4.0));
        assert_eq!(
            p.minion_rest_target_point,
            Vec2::new(20.0 * 16.0 + 8.0, 57.0 * 16.0),
            "the up-side clearance pick (:56970) + the drop window (:56974)"
        );
        assert!(p.has_minion_rest_target(), "(:4060)");
    }

    /// The air-mouse case (`:56942` false → num3 stays 0): the spot drops
    /// straight down from the mouse row until the 5-window catches the
    /// ground — row 56's window 56..61 finds row 60.
    #[test]
    fn s1_p_i27_the_rest_aim_in_air_drops_to_the_first_window() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0;
        p.y = 50.0 * 16.0;
        p.minion_rest_target_aim(&w, Vec2::new(20.0 * 16.0 + 8.0, 56.0 * 16.0 + 4.0));
        assert_eq!(
            p.minion_rest_target_point,
            Vec2::new(20.0 * 16.0 + 8.0, 56.0 * 16.0),
            "the window 56..61 already holds the ground at 60 (:56974-56980)"
        );
    }

    /// The 1000-px gate (`:56993-56996`): a far column's spot is refused —
    /// the previous rest point stays.
    #[test]
    fn s1_p_i27_the_rest_aim_refuses_a_far_spot() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0;
        p.y = 50.0 * 16.0;
        p.minion_rest_target_point = Vec2::new(1.0, 2.0);
        // Column 60 is > 1000 px away horizontally? 40 tiles = 640 px —
        // use the vertical: the mouse deep under the player's reach is
        // still in-world; instead pin the player far LEFT of the world's
        // ground columns and aim at the far right edge (63*16 ≈ 1008 px
        // from x=0 minus the centre offset — over the gate).
        p.x = 0.0;
        p.y = 50.0 * 16.0;
        p.minion_rest_target_aim(&w, Vec2::new(63.0 * 16.0 + 15.0, 56.0 * 16.0));
        assert_eq!(
            p.minion_rest_target_point,
            Vec2::new(1.0, 2.0),
            "the >1000-px spot is refused (:56993)"
        );
    }

    /// `Player.UpdateMinionTarget`'s rest half (`:57005-57007`): drifting
    /// past 1000 px from the rest spot zeroes it.
    #[test]
    fn s1_p_i27_the_rest_target_drops_past_1000px() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.x = 100.0;
        p.y = 100.0;
        p.minion_rest_target_point = Vec2::new(100.0 + 999.0, 100.0 + 21.0);
        p.update_minion_target(&[]);
        assert!(p.has_minion_rest_target(), "inside the 1000-px centre distance");
        p.minion_rest_target_point = Vec2::new(100.0 + 1200.0, 100.0);
        p.update_minion_target(&[]);
        assert_eq!(p.minion_rest_target_point, Vec2::ZERO, "(:57007)");
    }

    /// `KeyHoldDown` (`Player.cs:56894-56912`): the 60-tick down hold
    /// clears the rest spot for the Stardust/Forbidden sets; 59 does not;
    /// the reversed cardinal disables the DOWN hold entirely.
    #[test]
    fn s1_p_i27_the_down_hold_clears_the_rest_spot_at_60() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_stardust = true;
        p.minion_rest_target_point = Vec2::new(10.0, 10.0);
        p.key_hold_down(59, false);
        assert!(p.has_minion_rest_target(), "59 < 60 (:56903)");
        p.key_hold_down(60, false);
        assert_eq!(p.minion_rest_target_point, Vec2::ZERO);
        // The Forbidden half (:56907-56910).
        p.set_stardust = false;
        p.set_forbidden = true;
        p.minion_rest_target_point = Vec2::new(10.0, 10.0);
        p.key_hold_down(60, false);
        assert_eq!(p.minion_rest_target_point, Vec2::ZERO);
        // Reversed: keyDir 0 != num 1 → the early return (:56901).
        p.minion_rest_target_point = Vec2::new(10.0, 10.0);
        p.key_hold_down(120, true);
        assert!(p.has_minion_rest_target(), "the reversed cardinal skips DOWN");
        // No set flag: nothing clears.
        p.set_forbidden = false;
        p.key_hold_down(120, false);
        assert!(p.has_minion_rest_target());
        let _ = w;
    }

    /// `KeyDoubleTap` (`Player.cs:56764-56787`): the Forbidden set's down
    /// double-tap aims the rest spot; without the flag nothing happens;
    /// the reversed cardinal skips the DOWN tap.
    #[test]
    fn s1_p_i27_the_forbidden_double_tap_aims_the_rest_spot() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0;
        p.y = 50.0 * 16.0;
        let mouse = Vec2::new(20.0 * 16.0 + 8.0, 56.0 * 16.0 + 4.0);
        // No flag: no aim.
        p.key_double_tap_down(&w, mouse, false);
        assert!(!p.has_minion_rest_target());
        // The flag arms the aim (the storm command is deferred).
        p.set_forbidden = true;
        p.key_double_tap_down(&w, mouse, false);
        assert_eq!(
            p.minion_rest_target_point,
            Vec2::new(20.0 * 16.0 + 8.0, 56.0 * 16.0)
        );
        // Reversed: the early return.
        p.minion_rest_target_point = Vec2::ZERO;
        p.key_double_tap_down(&w, mouse, true);
        assert!(!p.has_minion_rest_target());
    }

    /// The input driver (`Player.cs:25022-25073`): the first down press
    /// arms the 15-tick window; a second press INSIDE it fires the
    /// double-tap; a press after the window lapses re-arms instead. The
    /// hold counter clears the rest spot at 60 held ticks.
    #[test]
    fn s1_p_i27_the_double_tap_window_and_the_hold_counter() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0;
        p.y = 50.0 * 16.0;
        p.set_forbidden = true;
        let mouse = Vec2::new(20.0 * 16.0 + 8.0, 56.0 * 16.0 + 4.0);
        let tick = |p: &mut Player, down: bool| {
            p.update_minion_rest_command(&w, mouse, down, false);
        };
        // Press 1: arms the window, no aim yet.
        tick(&mut p, true);
        assert!(!p.has_minion_rest_target(), "the first press only arms");
        // Release + press 2 within the window: the double-tap aims.
        tick(&mut p, false);
        tick(&mut p, true);
        assert!(p.has_minion_rest_target(), "the second press inside 15 ticks");
        // Clear, then let the window lapse (16 idle ticks) — the next
        // press re-arms instead of firing.
        p.minion_rest_target_point = Vec2::ZERO;
        tick(&mut p, false);
        for _ in 0..16 {
            tick(&mut p, false);
        }
        tick(&mut p, true);
        assert!(!p.has_minion_rest_target(), "the lapsed window re-arms");
        // The hold: 60 held ticks clear the spot (the counter runs from
        // the press above — 1 — so 59 more reach 60).
        p.minion_rest_target_point = Vec2::new(10.0, 10.0);
        for _ in 0..59 {
            tick(&mut p, true);
        }
        assert_eq!(p.minion_rest_target_point, Vec2::ZERO, "hold 60 (:56907)");
    }

    /// The set-flag grants + the per-tick reset (`Player.cs:18743`/
    /// `:18745`, the armor-set block): the Forbidden triple arms
    /// `setForbidden`, the Stardust triple `setStardust`; taking the
    /// armor off clears both on the next tick.
    #[test]
    fn s1_p_i27_the_set_flags_grant_and_reset() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[0] = 3776;
        p.equipped[1] = 3777;
        p.equipped[2] = 3778;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.set_forbidden, "the Forbidden triple (ArmorSetBonuses.cs:483)");
        assert!(!p.set_stardust);
        p.equipped[0] = 3381;
        p.equipped[1] = 3382;
        p.equipped[2] = 3383;
        p.update(&w, PlayerInput::default());
        assert!(p.set_stardust, "the Stardust triple (:482)");
        assert!(!p.set_forbidden, "the reset clears the old set (:18745)");
        p.equipped[0] = 0;
        p.equipped[1] = 0;
        p.equipped[2] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.set_stardust, "(:18743)");
    }
}
