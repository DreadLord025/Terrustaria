//! Minecart tracks — the static rail state machine `Minecart.TrackCollision`
//! (`Minecart.cs:562-941`), the 36-frame track decode `Minecart.Initialize`
//! (`Minecart.cs:103-547`) and the query helpers `GetOnTrack` (`:1200-1241`),
//! `OnTrack` (`:1243-1253`), `TrackRotation` (`:1255-1265`) and
//! `GetMagnetPosition` (`:1290-1294`).
//!
//! Pure module: reads the world and mutates only the position / velocity /
//! last-boost the caller passes in. Player integration (riding, boost apply,
//! dismount) is Phase 2 of M2-PHYS-7; wheel sparks + sounds Phase 3; track
//! placement (`FrameTrack` non-pound + the hammer pound) landed with the
//! research-71 §3b.4 follow-up. The mounted player is 20×52
//! (`Mount.SetAsMinecart`, `Mount.cs:2747-2799`) and every function below is
//! calibrated against the default `Minecart.Customization` (`Minecart.cs:29-34`).
//!
//! Phase 2 consumed the ride contract (the Player tick slots call
//! `track_collision`/`on_track`/`track_rotation`/`magnet_position`); the
//! remaining pre-consumer items keep targeted allows until Phase 3 (sparks)
//! and M2-MOUNT-3 (the hook-mount snap, the sprite decode) land.
//!
//! W4-f6 final 4 — the unclaimed track-side dust (research/198, renamed
//! 67→198 round-1): the four
//! non-base `MinecartDust` delegates (`DelegateMethods.Minecart.SparksFart`
//! :328 / `SparksTerraFart` :348 / `SparksMech` :373 / `SparksMeow` :394, the
//! 51/53/11/33 cart rows) ride the SAME `wheel_sparks` queue → `spawn_spark`
//! drain seam as the base 213 (the row's delegate picks the body at queue
//! time; the TerraFart 50% branch rolls BEFORE the side/speed rolls). The
//! tick + draw cover 260/211/107 beside the 213. `SpawnFartCloud` (:258, the
//! fart carts' 3-gore cloud + the 53-only 15-dust tail) is STAGED — its live
//! triggers are c14's fart sound delegates (the activation seam is documented
//! at the fn). The f11/e20 lanes are device-wiring (Wiring.cs), no claim here.

use glam::Vec2;
use tr_core::World;
use tr_core::tile::FLAG_INACTIVE;

/// `Player.defaultGravity` (`Player.cs:57118`) — read by the open-end release
/// (`Minecart.cs:907`) and the landing-effect gate (`Minecart.cs:809`).
const DEFAULT_GRAVITY: f32 = 0.4;

/// `Minecart.Customization` (`Minecart.cs:21-35`). The vanilla game re-defaults
/// `Player.MinecartSettings` every tick (`Player.cs:18829`), so the defaults
/// are the only values the riding physics ever sees.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MinecartSettings {
    /// `MinecartTextureWidth` — the sprite width the magnet math subtracts.
    pub texture_width: f32,
    /// `MagnetOffset` — the ride point inside the sprite.
    pub magnet_offset: Vec2,
    /// `WheelOffset` — the two wheel probe offsets from the player center.
    pub wheel_offset: Vec2,
}

impl Default for MinecartSettings {
    /// `Customization.Default` (`Minecart.cs:29-34`).
    fn default() -> Self {
        MinecartSettings {
            texture_width: 50.0,
            magnet_offset: Vec2::new(25.0, 26.0),
            wheel_offset: Vec2::new(12.0, 0.0),
        }
    }
}

/// `Minecart.TrackState` (`Minecart.cs:9-19`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TrackState {
    NoTrack = -1,
    AboveTrack = 0,
    OnTrack = 1,
    BelowTrack = 2,
    AboveFront = 3,
    AboveBack = 4,
    OnFront = 5,
    OnBack = 6,
}

/// The flag indices of the `BitsByte` result (`Minecart.cs:47-57`).
pub(crate) const FLAG_ON_TRACK: u8 = 1 << 0;
pub(crate) const FLAG_BOUNCY_BUMPER: u8 = 1 << 1;
pub(crate) const FLAG_USED_RAMP: u8 = 1 << 2;
pub(crate) const FLAG_HIT_SWITCH: u8 = 1 << 3;
pub(crate) const FLAG_BOOST_LEFT: u8 = 1 << 4;
pub(crate) const FLAG_BOOST_RIGHT: u8 = 1 << 5;

/// `BoosterSpeed` (`Minecart.cs:75`) — the boost the Player applies next tick
/// from the bit-4/bit-5 results (Phase 2).
pub(crate) const BOOSTER_SPEED: f32 = 4.0;

/// The `TrackCollision` result — the C# `BitsByte` flags (`Minecart.cs:47-57`,
/// filled at :589-940) plus the landing event the C# fired through
/// `delegatesData` (`Minecart.cs:809-813`, landing sound + 10 sparks). The
/// condition is internal march state (gated on the C# `flag9` stickiness) and
/// is not re-derivable by callers, so Phase 3 wires the effects from this flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrackFlags {
    /// The C# `BitsByte` bits 0-5.
    pub bits: u8,
    /// `Minecart.cs:809-813` — fell onto a rail from a real fall this call.
    pub landing: bool,
}

impl TrackFlags {
    /// `result[0]` — the final state rides a rail (`Minecart.cs:936-939`).
    pub(crate) fn on_track(&self) -> bool {
        self.bits & FLAG_ON_TRACK != 0
    }
    /// `result[1]` — bounced off a bouncy bumper (`Minecart.cs:689`).
    pub(crate) fn bouncy_bumper(&self) -> bool {
        self.bits & FLAG_BOUNCY_BUMPER != 0
    }
    /// `result[2]` — left the rail through a ramp end (`Minecart.cs:911`).
    pub(crate) fn used_ramp(&self) -> bool {
        self.bits & FLAG_USED_RAMP != 0
    }
    /// `result[3]` — crossed a pressure track (`Minecart.cs:816/:828`).
    pub(crate) fn hit_switch(&self) -> bool {
        self.bits & FLAG_HIT_SWITCH != 0
    }
    /// `result[4]` — passed a left-boosting booster tile (`Minecart.cs:840`).
    pub(crate) fn boost_left(&self) -> bool {
        self.bits & FLAG_BOOST_LEFT != 0
    }
    /// `result[5]` — passed a right-boosting booster tile (`Minecart.cs:844`).
    pub(crate) fn boost_right(&self) -> bool {
        self.bits & FLAG_BOOST_RIGHT != 0
    }
}

/// One decoded track frame: the per-frame data `Minecart.Initialize` fills
/// (`Minecart.cs:103-547`).
///
/// `tile_height` is stored **after** the Initialize epilogue transform
/// (`Minecart.cs:437-447`): a raw rail level `v` (1..8, higher = further up
/// the tile) becomes the pixel offset `(8 − v) · 2` from the tile top; unset
/// sub-columns start at level 5 (`Minecart.cs:117-121`) = 6 px. The sentinel
/// values stay negative (`Minecart.cs:67-73`): −1 regular bumper, −2 bouncy
/// bumper, −3 ramp end, −4 open end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TrackFrameData {
    /// `_leftSideConnection` (`Minecart.cs:59-65` encoding): 0 = up,
    /// 1 = straight, 2 = down, −1 = no connection.
    pub left_connection: i8,
    /// `_rightSideConnection`, same encoding.
    pub right_connection: i8,
    /// `_trackType` (`Minecart.cs:77-81`): 0 normal, 1 pressure, 2 booster.
    pub track_type: i8,
    /// `_boostLeft` (booster frames only).
    pub boost_left: bool,
    /// `_tileHeight[frame]` — the 8 two-pixel sub-columns (see above).
    pub tile_height: [i8; 8],
}

const fn frame(
    left: i8,
    right: i8,
    track_type: i8,
    boost_left: bool,
    tile_height: [i8; 8],
) -> TrackFrameData {
    TrackFrameData {
        left_connection: left,
        right_connection: right,
        track_type,
        boost_left,
        tile_height,
    }
}

/// The 36 track frames, transcribed from `Minecart.Initialize`
/// (`Minecart.cs:124-428`) in frame order.
pub(crate) const TRACK_FRAMES: [TrackFrameData; 36] = [
    // 0 — the lone piece: open ends at both edges (`Minecart.cs:125-129`).
    frame(-1, -1, 0, false, [-4, 6, 6, 6, 6, 6, 6, -4]),
    // 1 — straight (`Minecart.cs:131-133`).
    frame(1, 1, 0, false, [6; 8]),
    // 2 — bumper on the left half (`Minecart.cs:135-141`).
    frame(-1, 1, 0, false, [-1, -1, -1, -1, 6, 6, 6, 6]),
    // 3 — bumper on the right half (`Minecart.cs:143-149`).
    frame(1, -1, 0, false, [6, 6, 6, 6, -1, -1, -1, -1]),
    // 4 — corner: from below (left=2) to straight right (`Minecart.cs:151-159`).
    frame(2, 1, 0, false, [14, 12, 10, 10, 8, 8, 6, 6]),
    // 5 — corner: straight left down to below (right=2) (`Minecart.cs:161-169`).
    frame(1, 2, 0, false, [6, 6, 8, 8, 10, 10, 12, 14]),
    // 6 — corner: straight left up to above (right=0) (`Minecart.cs:171-177`).
    frame(1, 0, 0, false, [6, 6, 6, 6, 4, 4, 2, 0]),
    // 7 — corner: from above (left=0) to straight right (`Minecart.cs:179-185`).
    frame(0, 1, 0, false, [0, 2, 4, 4, 6, 6, 6, 6]),
    // 8 — diagonal down-right (`Minecart.cs:187-193`).
    frame(0, 2, 0, false, [0, 2, 4, 6, 8, 10, 12, 14]),
    // 9 — diagonal up-right (`Minecart.cs:195-201`).
    frame(2, 0, 0, false, [14, 12, 10, 8, 6, 4, 2, 0]),
    // 10 — ramp up from below, dead end (`Minecart.cs:203-211`).
    frame(2, -1, 0, false, [14, 12, -1, -1, -1, -1, -1, -1]),
    // 11 — dead end, ramp down to below (`Minecart.cs:213-221`).
    frame(-1, 2, 0, false, [-1, -1, -1, -1, -1, -1, 12, 14]),
    // 12 — from above, dead end (`Minecart.cs:223-232`).
    frame(0, -1, 0, false, [0, 2, 4, -1, -1, -1, -1, -1]),
    // 13 — dead end, drops to below (`Minecart.cs:234-243`).
    frame(-1, 0, 0, false, [-1, -1, -1, -1, -1, 4, 2, 0]),
    // 14 — open end on the left (`Minecart.cs:245-248`).
    frame(-1, 1, 0, false, [-4, 6, 6, 6, 6, 6, 6, 6]),
    // 15 — open end on the right (`Minecart.cs:250-253`).
    frame(1, -1, 0, false, [6, 6, 6, 6, 6, 6, 6, -4]),
    // 16 — ramp rising right, ramp end (−3) on top (`Minecart.cs:255-263`).
    frame(2, -1, 0, false, [14, 12, 10, 8, 6, 4, -3, -3]),
    // 17 — ramp end (−3) at the bottom left (`Minecart.cs:265-273`).
    frame(-1, 2, 0, false, [-3, -3, 4, 6, 8, 10, 12, 14]),
    // 18 — from above, ramp end (−3) (`Minecart.cs:275-283`).
    frame(0, -1, 0, false, [0, 2, 4, 6, 8, 10, -3, -3]),
    // 19 — ramp end (−3) at the top left (`Minecart.cs:285-293`).
    frame(-1, 0, 0, false, [-3, -3, 10, 8, 6, 4, 2, 0]),
    // 20 — pressure: lone piece (`Minecart.cs:295-300`).
    frame(-1, -1, 1, false, [-4, 6, 6, 6, 6, 6, 6, -4]),
    // 21 — pressure: straight (`Minecart.cs:302-305`).
    frame(1, 1, 1, false, [6; 8]),
    // 22 — pressure: open end left (`Minecart.cs:307-311`).
    frame(-1, 1, 1, false, [-4, 6, 6, 6, 6, 6, 6, 6]),
    // 23 — pressure: open end right (`Minecart.cs:313-317`).
    frame(1, -1, 1, false, [6, 6, 6, 6, 6, 6, 6, -4]),
    // 24 — bouncy bumper left (`Minecart.cs:319-325`).
    frame(-1, 1, 0, false, [-2, -2, -2, -2, -2, -2, 6, 6]),
    // 25 — bouncy bumper right (`Minecart.cs:327-333`).
    frame(1, -1, 0, false, [6, 6, -2, -2, -2, -2, -2, -2]),
    // 26 — ramp down from below into bouncy (`Minecart.cs:335-343`).
    frame(2, -1, 0, false, [14, 12, -2, -2, -2, -2, -2, -2]),
    // 27 — bouncy into ramp down (`Minecart.cs:345-353`).
    frame(-1, 2, 0, false, [-2, -2, -2, -2, -2, -2, 12, 14]),
    // 28 — from above into bouncy (`Minecart.cs:355-364`).
    frame(0, -1, 0, false, [0, 2, 4, -2, -2, -2, -2, -2]),
    // 29 — bouncy drops to below (`Minecart.cs:366-375`).
    frame(-1, 0, 0, false, [-2, -2, -2, -2, -2, 4, 2, 0]),
    // 30 — booster, straight, boosts right (`Minecart.cs:377-381`).
    frame(1, 1, 2, false, [6; 8]),
    // 31 — booster, straight, boosts left (`Minecart.cs:383-387`).
    frame(1, 1, 2, true, [6; 8]),
    // 32 — booster diagonal down-right, boosts right (`Minecart.cs:389-397`).
    frame(0, 2, 2, false, [0, 2, 4, 6, 8, 10, 12, 14]),
    // 33 — booster diagonal up-right, boosts left (`Minecart.cs:399-407`).
    frame(2, 0, 2, true, [14, 12, 10, 8, 6, 4, 2, 0]),
    // 34 — booster diagonal down-right, boosts left (`Minecart.cs:409-417`).
    frame(0, 2, 2, true, [0, 2, 4, 6, 8, 10, 12, 14]),
    // 35 — booster diagonal up-right, boosts right (`Minecart.cs:419-427`).
    frame(2, 0, 2, false, [14, 12, 10, 8, 6, 4, 2, 0]),
];

/// `_texturePosition` (`Minecart.cs:129-427`, decorations `:429-432`, the ×18
/// cell scale `:433-436`) — the sprite cell of each frame id in the 18 px
/// track sheet. Frames 36-39 are the decoration pieces (36/37 left-down and
/// right-down, 38/39 the bumpers — note the C# assigns 39 before 38).
#[allow(dead_code)] // the track-sprite decode — the rendering follow-up
pub(crate) const TEXTURE_POSITION: [(f32, f32); 40] = [
    (0.0, 0.0),
    (18.0, 0.0),
    (36.0, 18.0),
    (54.0, 18.0),
    (0.0, 36.0),
    (18.0, 36.0),
    (0.0, 18.0),
    (18.0, 18.0),
    (0.0, 54.0),
    (18.0, 54.0),
    (72.0, 18.0),
    (90.0, 18.0),
    (108.0, 18.0),
    (126.0, 18.0),
    (36.0, 0.0),
    (54.0, 0.0),
    (72.0, 0.0),
    (90.0, 0.0),
    (108.0, 0.0),
    (126.0, 0.0),
    (0.0, 72.0),
    (18.0, 72.0),
    (0.0, 90.0),
    (18.0, 90.0),
    (36.0, 36.0),
    (54.0, 36.0),
    (72.0, 36.0),
    (90.0, 36.0),
    (108.0, 36.0),
    (126.0, 36.0),
    (36.0, 54.0),
    (54.0, 54.0),
    (72.0, 54.0),
    (90.0, 54.0),
    (108.0, 54.0),
    (126.0, 54.0),
    (0.0, 108.0),
    (18.0, 108.0),
    (18.0, 126.0),
    (0.0, 126.0),
];

/// `_firstPressureFrame` / `_firstRightBoostFrame` / `_firstLeftBoostFrame`
/// (`Minecart.cs:519-546`).
#[allow(dead_code)] // the placement/orientation follow-up reads these ranges
pub(crate) const FIRST_PRESSURE_FRAME: i16 = 20;
#[allow(dead_code)] // see above
pub(crate) const FIRST_RIGHT_BOOST_FRAME: i16 = 30;
#[allow(dead_code)] // see above
pub(crate) const FIRST_LEFT_BOOST_FRAME: i16 = 31;

/// `tile.nactive() && tile.type == 314` (`Minecart.cs:633`) — an active,
/// non-actuated track tile. `nactive()` = active and not actuator-hidden
/// (`Tile.cs:240-247`); the port follows the `liquid.rs:384` shape.
fn is_track_tile(tile: &tr_core::Tile) -> bool {
    tile.active() && tile.flags & FLAG_INACTIVE == 0 && tile.kind == 314
}

/// `Minecart.IsPressurePlate` (`Minecart.cs:549-560`): an active tile 314
/// whose FRONT frame is 20 or 21 — the two unpressed pressure-track fronts
/// (`_trackType` 1 spans frames 20-23, but the pressed variants 22/23 do NOT
/// count). The null-tile arm is the caller's `Option` (a missing tile is
/// `false`). Sole consumer: `WorldGen.IsItATrigger` (`WorldGen.cs:26817-
/// 26820`) inside the `ClearBrokenTraps` walk (`:26840-26958`, the
/// `BrokenTrapCleanup` worldgen pass `:22121-22140`) — that pass has no port
/// yet (the worldgen-passes lanes), so this rides `dead_code` until it lands;
/// the cleanup's 314 arm sets `frameX = 1` instead of killing (`:26948-
/// 26951`). NOT the ride's pressure trip — that reads `TrackCollision`'s
/// flag3 over the whole type-1 range (`Minecart.cs:809-813` region).
#[allow(dead_code)] // the BrokenTrapCleanup worldgen pass is the consumer
pub(crate) fn is_pressure_plate(tile: Option<&tr_core::Tile>) -> bool {
    tile.is_some_and(|t| t.active() && t.kind == 314 && (t.frame_x == 20 || t.frame_x == 21))
}

/// `Minecart.GetMagnetPosition` (`Minecart.cs:1290-1294`) with the default
/// settings: `position + (width/2 − 25, height/2) + (25, 26)`. For the mounted
/// 20×52 body the two halves cancel to the exact bottom-center.
pub(crate) fn magnet_position(position: Vec2, width: i32, height: i32) -> Vec2 {
    let settings = MinecartSettings::default();
    position
        + Vec2::new(
            (width / 2) as f32 - settings.texture_width / 2.0,
            (height / 2) as f32,
        )
        + settings.magnet_offset
}

/// `Minecart.TrackCollision` (`Minecart.cs:562-941`): the ride state machine.
///
/// Marches the bottom-center magnet point along the velocity in ≤ 2 px steps,
/// following rails, corners, ramps and bumpers, and returns the flag byte.
/// Mutations (the C# `ref` parameters): `position.Y` snaps onto the final rail
/// line, rounded to 2 decimals (`Minecart.cs:934-935`); `position.X` moves
/// only in the bouncy-bumper rewind (`Minecart.cs:917`, pitfall §10.3 — X
/// integration is the caller's job); `velocity` is rewritten by the
/// bumper/ramp/open-end/stop paths; `last_boost` (tile coords, `(0,0)` = none)
/// is set passing a booster and cleared when the march ends on a different
/// tile (`Minecart.cs:832-847`, `:894-897`).
///
/// Out-of-range indices behave like the C#'s unchecked `IndexOutOfRange`:
/// a track frame id outside 0..35 (or a negative magnet X, whose tile lookup
/// dies first in C#) panics on the table index.
pub(crate) fn track_collision(
    world: &World,
    position: &mut Vec2,
    velocity: &mut Vec2,
    last_boost: &mut (i32, i32),
    width: i32,
    height: i32,
    follow_down: bool,
    follow_up: bool,
    fall_start: i32,
    track_only: bool,
) -> TrackFlags {
    let settings = MinecartSettings::default();
    // `if (followDown & followUp)` — both cancels (`Minecart.cs:564-568`).
    let (mut follow_down, mut follow_up) = (follow_down, follow_up);
    if follow_down && follow_up {
        follow_down = false;
        follow_up = false;
    }
    // `vector` — the sprite-anchored offset (`Minecart.cs:570`; the C#
    // `Width / 2` is integer division).
    let anchor = Vec2::new(
        (width / 2) as f32 - settings.texture_width / 2.0,
        (height / 2) as f32,
    );
    // `vector2` — the magnet start (`Minecart.cs:571`).
    let magnet_start = *position + anchor + settings.magnet_offset;
    // `vector3`/`num` — the normalized velocity and the remaining speed
    // (`Minecart.cs:572-574`; `normalize_or_zero` — the C# NaN of a zero
    // vector is dead at every use site because `num == 0` ends the march).
    let speed_total = velocity.length();
    let mut march_dir = velocity.normalize_or_zero();
    let mut speed_left = speed_total;
    // `vector4` — the marching point (`Minecart.cs:575`).
    let mut march = magnet_start;
    // `flag` — the current tile is a track; `flag2` — first iteration;
    // `num2`/`num3` — the current tile; `num4` — the current sub-column
    // (`Minecart.cs:576-582`).
    let mut tile_frames: Option<(i16, i16)> = None;
    let mut riding_tile = false;
    let mut first_iteration = true;
    let mut tile_x = -1i32;
    let mut tile_y = -1i32;
    let mut sub_column = -1i32;
    let mut state = TrackState::NoTrack;
    // `flag3` pressure hit, `flag4` ramp used, `flag5` open-end leave,
    // `flag6` bumper/stop rewind (`Minecart.cs:583-586`).
    let mut hit_switch = false;
    let mut used_ramp = false;
    let mut open_end = false;
    let mut stopped = false;
    // `vector5` — the ramp launch velocity; `vector6` — the last march step
    // (`Minecart.cs:587-588`).
    let mut ramp_velocity = Vec2::ZERO;
    let mut last_step = Vec2::ZERO;
    let mut result = TrackFlags::default();
    loop {
        // `num5`/`num6`/`num7` — tile X, tile Y and the 2 px sub-column
        // (`Minecart.cs:592-594`). For X ≥ 0 the sub-column is bounded 0..7 by
        // construction; a negative X dies at the tile lookup below (as the
        // C#'s raw `Tile[,]` access does).
        let column = (march.x / 16.0) as i32;
        let mut row = (march.y / 16.0) as i32;
        let sub = (march.x as i32 % 16) / 2;
        if first_iteration {
            sub_column = sub;
        }
        // `flag7` — the sub-column moved since the last iteration
        // (`Minecart.cs:599`).
        let sub_moved = sub != sub_column;

        // Corner switching (`Minecart.cs:600-614`): a tile-X change while
        // already on a rail reads the previous tile's connection in the
        // travel direction and steps the march one row up/down.
        if matches!(
            state,
            TrackState::OnBack | TrackState::OnTrack | TrackState::OnFront
        ) && column != tile_x
        {
            let (front, back) = tile_frames.expect("an On* state implies a current track tile");
            // `num8` — the ridden rail: the front rail unless OnBack
            // (`FrontTrack`/`BackTrack`, `Minecart.cs:602`, `:1632-1650`).
            let rail = if state != TrackState::OnBack {
                front as usize
            } else {
                back as usize
            };
            // `!(vector3.X < 0f)` takes the right connection (`Minecart.cs:603`).
            let connection = if march_dir.x < 0.0 {
                TRACK_FRAMES[rail].left_connection
            } else {
                TRACK_FRAMES[rail].right_connection
            };
            match connection {
                // 0 = up: the rail steps into the row above (`:605-608`).
                0 => {
                    row -= 1;
                    march.y -= 2.0;
                }
                // 2 = down: into the row below (`:609-612`).
                2 => {
                    row += 1;
                    march.y += 2.0;
                }
                _ => {}
            }
        }

        // `trackState2`/`flag8` — the resolved state and "the tile changed"
        // (`Minecart.cs:615-634`).
        let mut new_state = TrackState::NoTrack;
        let mut tile_changed = false;
        if column != tile_x || row != tile_y {
            if first_iteration {
                first_iteration = false;
            } else {
                tile_changed = true;
            }
            let tile = world.tile(column, row);
            riding_tile = tile.is_some_and(is_track_tile);
            tile_frames = tile.map(|t| (t.frame_x, t.frame_y));
        }

        if riding_tile {
            let (front, back) = tile_frames.expect("riding_tile implies a tile");
            let front = front as usize;
            // `num11` — the front rail height at this sub-column
            // (`Minecart.cs:640`); an out-of-range frame id panics like the
            // C#'s unchecked index.
            match TRACK_FRAMES[front].tile_height[sub as usize] {
                // −4 open end (`Minecart.cs:643-658`).
                -4 => {
                    if state == TrackState::OnFront {
                        if track_only {
                            march -= last_step;
                            speed_left = 0.0;
                            new_state = TrackState::OnFront;
                            stopped = true;
                        } else {
                            new_state = TrackState::NoTrack;
                            open_end = true;
                        }
                    }
                }
                // −1 regular bumper: dead stop on the rail (`:660-668`).
                -1 => {
                    if state == TrackState::OnFront {
                        march -= last_step;
                        speed_left = 0.0;
                        new_state = TrackState::OnFront;
                        stopped = true;
                    }
                }
                // −2 bouncy bumper (`:669-691`).
                -2 => {
                    if state == TrackState::OnFront {
                        if track_only {
                            march -= last_step;
                            speed_left = 0.0;
                            new_state = TrackState::OnFront;
                            stopped = true;
                        } else {
                            if march_dir.x < 0.0 {
                                // Rewind to the right edge of the current
                                // sub-column (`Minecart.cs:682-687`).
                                let rewind = (column * 16 + (sub + 1) * 2) as f32 - march.x;
                                march.x += rewind;
                                speed_left += rewind / march_dir.x;
                            }
                            march_dir.x = -march_dir.x;
                            result.bits |= FLAG_BOUNCY_BUMPER;
                            new_state = TrackState::OnFront;
                        }
                    }
                }
                // −3 ramp end: leave at 45° (`:692-701`).
                -3 => {
                    if state == TrackState::OnFront {
                        state = TrackState::NoTrack;
                        // `Vector2.Transform((Velocity.X, 0), rotZ(±π/4))` =
                        // `(vx·cos, vx·sin)`; the X component is then
                        // overridden back to `Velocity.X` (`:696-697`) — the
                        // launch is `(vx, ±vx·√2/2)` (the research §6 prose
                        // says `(vx, ±vx)`; the code wins).
                        let quarter = std::f32::consts::FRAC_PI_4;
                        let angle = if velocity.x > 0.0 {
                            if TRACK_FRAMES[front].left_connection != 2 {
                                quarter
                            } else {
                                -quarter
                            }
                        } else if TRACK_FRAMES[front].right_connection != 2 {
                            -quarter
                        } else {
                            quarter
                        };
                        let (sin, cos) = angle.sin_cos();
                        ramp_velocity = Vec2::new(velocity.x * cos, velocity.x * sin);
                        ramp_velocity.x = velocity.x;
                        used_ramp = true;
                        speed_left = 0.0;
                    }
                }
                // The regular rail heights (`:702-741`).
                _ => {
                    let front_h = TRACK_FRAMES[front].tile_height[sub as usize];
                    let rail_y = row as f32 * 16.0 + front_h as f32; // `num12`
                    // The landing catch (`:705-709`): a new tile with the
                    // magnet within 2 px below the rail demotes to AboveFront
                    // so the transition ladder can land the cart.
                    if column != tile_x
                        && state == TrackState::NoTrack
                        && march.y > rail_y
                        && march.y - rail_y < 2.0
                    {
                        tile_changed = false;
                        state = TrackState::AboveFront;
                    }
                    let front_state = if march.y < rail_y {
                        TrackState::AboveTrack
                    } else if march.y > rail_y {
                        TrackState::BelowTrack
                    } else {
                        TrackState::OnTrack
                    }; // `trackState4`
                    let mut back_state = TrackState::NoTrack; // `trackState3`
                    if back != -1 {
                        let back_h = TRACK_FRAMES[back as usize].tile_height[sub as usize];
                        let back_rail_y = row as f32 * 16.0 + back_h as f32; // `num13`
                        back_state = if march.y < back_rail_y {
                            TrackState::AboveTrack
                        } else if march.y > back_rail_y {
                            TrackState::BelowTrack
                        } else {
                            TrackState::OnTrack
                        };
                    }
                    new_state = match front_state {
                        TrackState::OnTrack => {
                            if back_state == TrackState::OnTrack {
                                TrackState::OnTrack
                            } else {
                                TrackState::OnFront
                            }
                        }
                        TrackState::AboveTrack => match back_state {
                            TrackState::OnTrack => TrackState::OnBack,
                            TrackState::BelowTrack => TrackState::AboveFront,
                            TrackState::AboveTrack => TrackState::AboveTrack,
                            _ => TrackState::AboveFront,
                        },
                        TrackState::BelowTrack => match back_state {
                            TrackState::OnTrack => TrackState::OnBack,
                            TrackState::AboveTrack => TrackState::AboveBack,
                            TrackState::BelowTrack => TrackState::BelowTrack,
                            _ => TrackState::BelowTrack,
                        },
                        _ => unreachable!("front_state is Above/On/Below by construction"),
                    };
                }
            }
        }

        // The transition resolution (`Minecart.cs:744-822`) — same tile, state
        // changed, and the sub-column moved or the cart is descending.
        if !tile_changed {
            if state != new_state {
                let mut sticky = false; // `flag9`
                if sub_moved || march_dir.y > 0.0 {
                    // The demotion ladder + sticky On* mapping (`:751-794`).
                    match state {
                        TrackState::AboveTrack => {
                            new_state = match new_state {
                                TrackState::AboveFront => TrackState::OnBack,
                                TrackState::AboveBack => TrackState::OnFront,
                                TrackState::AboveTrack => TrackState::OnTrack,
                                _ => new_state,
                            };
                        }
                        TrackState::AboveFront => {
                            if new_state == TrackState::BelowTrack {
                                new_state = TrackState::OnFront;
                            }
                        }
                        TrackState::AboveBack => {
                            if new_state == TrackState::BelowTrack {
                                new_state = TrackState::OnBack;
                            }
                        }
                        TrackState::OnFront => {
                            new_state = TrackState::OnFront;
                            sticky = true;
                        }
                        TrackState::OnBack => {
                            new_state = TrackState::OnBack;
                            sticky = true;
                        }
                        TrackState::OnTrack => {
                            // The between-rails ambiguity (`:787-794`): the
                            // follow flags pick the rail to keep riding.
                            let (front, back) = tile_frames.expect("OnTrack implies a track tile");
                            let front_h = TRACK_FRAMES[front as usize].tile_height[sub as usize];
                            let back_h = TRACK_FRAMES[back as usize].tile_height[sub as usize];
                            new_state = if follow_down {
                                if front_h >= back_h {
                                    TrackState::OnFront
                                } else {
                                    TrackState::OnBack
                                }
                            } else if !follow_up {
                                TrackState::OnFront
                            } else if front_h >= back_h {
                                TrackState::OnBack
                            } else {
                                TrackState::OnFront
                            };
                            sticky = true;
                        }
                        _ => {}
                    }
                    // `num17` — the frame the resolved state rides
                    // (`Minecart.cs:796-806`).
                    let ridden = match new_state {
                        TrackState::OnTrack | TrackState::OnFront => {
                            Some(tile_frames.expect("riding state implies a tile").0)
                        }
                        TrackState::OnBack => {
                            Some(tile_frames.expect("riding state implies a tile").1)
                        }
                        _ => None,
                    };
                    if let Some(rail) = ridden {
                        let rail = rail as usize;
                        // The landing effect gate (`:809-813`): a real fall
                        // (faster than gravity, the origin at least one row
                        // above) fired the C# sound + 10-spark delegates.
                        if !sticky
                            && velocity.y > DEFAULT_GRAVITY
                            && fall_start < (position.y / 16.0) as i32 - 1
                        {
                            result.landing = true;
                        }
                        // The pressure trip from above (`:814-817`).
                        if state == TrackState::AboveFront && TRACK_FRAMES[rail].track_type == 1 {
                            hit_switch = true;
                        }
                        // The snap (`:818-819`).
                        march_dir.y = 0.0;
                        march.y =
                            row as f32 * 16.0 + TRACK_FRAMES[rail].tile_height[sub as usize] as f32;
                    }
                }
            }
        } else if matches!(
            new_state,
            TrackState::OnFront | TrackState::OnBack | TrackState::OnTrack
        ) {
            // Tile changed while already riding (`:824-831`): the pressure
            // check and a level march direction.
            if riding_tile {
                let front = tile_frames.expect("riding_tile implies a tile").0 as usize;
                if TRACK_FRAMES[front].track_type == 1 {
                    hit_switch = true;
                }
            }
            march_dir.y = 0.0;
        }

        // The boost detection (`Minecart.cs:832-847`): finishing a step on the
        // front rail of a booster tile with no pending boost marks the tile.
        if new_state == TrackState::OnFront {
            let front = tile_frames.expect("OnFront implies a tile").0 as usize;
            if TRACK_FRAMES[front].track_type == 2 && *last_boost == (0, 0) {
                *last_boost = (column, row);
                if TRACK_FRAMES[front].boost_left {
                    result.bits |= FLAG_BOOST_LEFT;
                } else {
                    result.bits |= FLAG_BOOST_RIGHT;
                }
            }
        }

        // Bookkeeping (`Minecart.cs:848-851`).
        sub_column = sub;
        state = new_state;
        tile_x = column;
        tile_y = row;

        // The 2 px march step (`Minecart.cs:852-892`).
        if speed_left > 0.0 {
            let rem_x = march.x % 2.0;
            let rem_y = march.y % 2.0;
            let mut dist_x = 3.0f32; // `num21`
            let mut dist_y = 3.0f32; // `num22`
            if march_dir.x < 0.0 {
                dist_x = rem_x + 0.125;
            } else if march_dir.x > 0.0 {
                dist_x = 2.0 - rem_x;
            }
            if march_dir.y < 0.0 {
                dist_y = rem_y + 0.125;
            } else if march_dir.y > 0.0 {
                dist_y = 2.0 - rem_y;
            }
            if dist_x == 3.0 && dist_y == 3.0 {
                // A levelled direction with speed left: stop without the
                // last-boost clear (`Minecart.cs:874-877`).
                break;
            }
            // `num23`/`num24`/`num25` — the step distance; a zero component
            // divides to ±∞ and loses the min() (`Minecart.cs:878-880`).
            let step_x = (dist_x / march_dir.x).abs();
            let step_y = (dist_y / march_dir.y).abs();
            let step = step_x.min(step_y);
            if step > speed_left {
                last_step = march_dir * speed_left;
                speed_left = 0.0;
            } else {
                last_step = march_dir * step;
                speed_left -= step;
            }
            march += last_step;
            continue;
        }
        // The march ended on a tile — drop a pending boost that belongs to a
        // different tile (`Minecart.cs:894-897`).
        if *last_boost != (tile_x, tile_y) {
            *last_boost = (0, 0);
        }
        break;
    }

    // The final resolve (`Minecart.cs:900-940`).
    if hit_switch {
        result.bits |= FLAG_HIT_SWITCH;
    }
    if open_end {
        // Left through an open end: keep the travelled X, drop into gravity
        // (`Minecart.cs:904-908`).
        velocity.x = march.x - magnet_start.x;
        velocity.y = DEFAULT_GRAVITY;
    } else if used_ramp {
        result.bits |= FLAG_USED_RAMP;
        *velocity = ramp_velocity;
    } else if result.bouncy_bumper() {
        // The bouncy rewind is the only X move (`Minecart.cs:914-922`).
        velocity.x = -velocity.x;
        position.x = march.x - settings.magnet_offset.x - anchor.x - velocity.x;
        if march_dir.y == 0.0 {
            velocity.y = 0.0;
        }
    } else {
        if stopped {
            velocity.x = march.x - magnet_start.x;
        }
        if march_dir.y == 0.0 {
            velocity.y = 0.0;
        }
    }
    // The Y snap, rounded to 2 decimals (`Minecart.cs:934-935`).
    position.y += march.y - magnet_start.y - velocity.y;
    position.y = round_y_snap(position.y);
    // `result[0]` iff the final state rides a rail (`Minecart.cs:936-939`;
    // the C# `(uint)(trackState - 5) <= 1` is OnFront/OnBack).
    if matches!(
        state,
        TrackState::OnTrack | TrackState::OnFront | TrackState::OnBack
    ) {
        result.bits |= FLAG_ON_TRACK;
    }
    result
}

/// `Math.Round(Position.Y, 2)` (`Minecart.cs:935`) — C# banker's rounding to
/// two decimals (`MidpointRounding.ToEven`); Rust's `f32::round` is
/// half-away-from-zero and has no digit form. Same emulation shape as
/// `tr_core::damage::round_bankers`.
fn round_y_snap(y: f32) -> f32 {
    let scaled = y as f64 * 100.0;
    let floor = scaled.floor();
    let diff = scaled - floor;
    let rounded = if diff > 0.5 {
        floor + 1.0
    } else if diff < 0.5 {
        floor
    } else if (floor as i64) % 2 == 0 {
        floor
    } else {
        floor + 1.0
    };
    (rounded / 100.0) as f32
}

/// `Minecart.OnTrack` (`Minecart.cs:1243-1253`): is the magnet point's tile a
/// track tile. The C# checks only the tile type (`:1248-1252` — no
/// `nactive()`), so an actuated-off track still counts.
pub(crate) fn on_track(world: &World, position: Vec2, width: i32, height: i32) -> bool {
    let magnet = magnet_position(position, width, height);
    let x = (magnet.x / 16.0) as i32;
    let y = (magnet.y / 16.0) as i32;
    world.tile(x, y).is_some_and(|t| t.kind == 314)
}

/// `Minecart.GetOnTrack` (`Minecart.cs:1200-1241`): snap the position so the
/// magnet sits on the nearest valid front-rail sub-column of the track tile at
/// (`tile_x`, `tile_y`). Returns false when the tile is not a track — the C#
/// NREs on a missing tile (`:1202` reads it unchecked), which the port treats
/// as "not a track".
#[allow(dead_code)] // the hook→cart mount snap — M2-MOUNT-3
pub(crate) fn get_on_track(
    world: &World,
    tile_x: i32,
    tile_y: i32,
    position: &mut Vec2,
    width: i32,
    height: i32,
) -> bool {
    let settings = MinecartSettings::default();
    let Some(tile) = world.tile(tile_x, tile_y) else {
        return false;
    };
    if tile.kind != 314 {
        return false;
    }
    let anchor = Vec2::new(
        (width / 2) as f32 - settings.texture_width / 2.0,
        (height / 2) as f32,
    );
    let mut magnet = *position + anchor + settings.magnet_offset;
    // The sub-column under the magnet (`Minecart.cs:1209`).
    let start_sub = (magnet.x as i32 % 16) / 2;
    let frame = tile.frame_x as usize;
    // Scan forward then backward for the first sub-column with a real rail
    // surface (height ≥ 0; the sentinels are negative, `:1210-1232`).
    let mut found = -1i32;
    let mut rail_y = 0i32;
    for sub in start_sub..8 {
        rail_y = TRACK_FRAMES[frame].tile_height[sub as usize] as i32;
        if rail_y >= 0 {
            found = sub;
            break;
        }
    }
    if found == -1 {
        for sub in (0..start_sub).rev() {
            rail_y = TRACK_FRAMES[frame].tile_height[sub as usize] as i32;
            if rail_y >= 0 {
                found = sub;
                break;
            }
        }
    }
    if found == -1 {
        return false;
    }
    // The snap onto the found sub-column (`Minecart.cs:1237-1240`).
    magnet.x = tile_x as f32 * 16.0 + found as f32 * 2.0;
    magnet.y = tile_y as f32 * 16.0 + rail_y as f32;
    *position = magnet - settings.magnet_offset - anchor;
    true
}

/// `Minecart.GetWheelsPositions` (`Minecart.cs:1267-1279`): probe the two
/// wheel points (±`WheelOffset.X`) with trackOnly `TrackCollision` passes and
/// add the residual velocity each probe kept.
fn get_wheels_positions(
    world: &World,
    position: Vec2,
    width: i32,
    height: i32,
    follow_down: bool,
    follow_up: bool,
) -> (Vec2, Vec2) {
    let settings = MinecartSettings::default();
    let mut last_boost = (0, 0);
    // The left wheel (`Minecart.cs:1273-1275`).
    let mut left = position;
    let mut velocity = settings.wheel_offset * Vec2::new(-1.0, 1.0);
    track_collision(
        world,
        &mut left,
        &mut velocity,
        &mut last_boost,
        width,
        height,
        follow_down,
        follow_up,
        0,
        true,
    );
    left += velocity;
    // The right wheel (`Minecart.cs:1276-1278`).
    let mut right = position;
    let mut velocity = settings.wheel_offset;
    track_collision(
        world,
        &mut right,
        &mut velocity,
        &mut last_boost,
        width,
        height,
        follow_down,
        follow_up,
        0,
        true,
    );
    right += velocity;
    (left, right)
}

/// `Minecart.TrackRotation` (`Minecart.cs:1255-1265`): the rail slope through
/// the two wheel probes. Returns `(gfx_off_y, rotation)` — the C# writes
/// `rotation` by ref and returns the Y correction.
pub(crate) fn track_rotation(
    world: &World,
    position: Vec2,
    width: i32,
    height: i32,
    follow_down: bool,
    follow_up: bool,
) -> (f32, f32) {
    let (left, right) =
        get_wheels_positions(world, position, width, height, follow_down, follow_up);
    let dy = right.y - left.y; // `num`
    let dx = right.x - left.x; // `num2`
    let slope = dy / dx; // `num3`
    let line_y = left.y + (position.x - left.x) * slope; // `num4`
    let frac_y = (position.x - position.x as i32 as f32) * slope; // `num5`
    let rotation = dy.atan2(dx);
    (line_y - position.y + frac_y, rotation)
}

// ===========================================================================
// Wheel sparks + minecart sounds (M2-PHYS-7 Phase 3)
// ===========================================================================

/// One queued minecart cue: the `SoundEngine.PlaySound(SoundID.ItemNN, x, y)`
/// call site's cue name and pixel position, drained by `world_sim` into
/// `App::audio::play_at` (the tick has no App access — the
/// `pending_switch_trip` handoff pattern).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CartCue {
    pub(crate) cue: &'static str,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

/// One queued spark spawn request — the pre-`NewDust` half of the cart's
/// `MinecartDust` delegate (`DelegateMethods.Minecart.Sparks`
/// `DelegateMethods.cs:225-227` and its four variants): the ±13 side offset
/// rotated by `Minecart.rotation` (the `Dust.NewDust` position argument) and
/// the two `Next(-2,3)` speed arguments the C# evaluates before the call. The
/// `world_sim` drain turns it into the pool slot ([`spawn_spark`]); the pool's
/// own rolls then ride the same stream. `variant` is the row's delegate
/// (`Mount.cs:122`/`:916`/`:1013`/`:1099`/`:1107`/`:1487`); the base `Sparks`
/// keeps the type-213 body byte-for-byte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SparkSpawn {
    pub(crate) pos: Vec2,
    pub(crate) speed_x: f32,
    pub(crate) speed_y: f32,
    body: SparkBody,
}

/// The cart's `MinecartDust` delegate (`MountDelegatesData.MinecartDust`,
/// `Mount.cs:104`). The base `Sparks` (`DelegateMethods.cs:223-242`) is the
/// type-213 wheel spark; the four variants are the unclaimed track-side dust
/// (research/198, renamed from 67): Fart 51 (`:1099`), Terra-Fart 53 (`:1107`), Mech 11
/// (`:1487`), Meowmere 33 (`:1013`). Every other cart row keeps `Sparks`
/// (`SetAsMinecart` `:2751`, the hand-written 6/13/16 rows `:916`/`:1595`/
/// `:1047`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SparkVariant {
    /// `DelegateMethods.Minecart.Sparks` — dust 213 (`:223-242`).
    Sparks,
    /// `SparksFart` — dust 211, alpha 50, scale 0.8 (`:328-346`).
    Fart,
    /// `SparksTerraFart` — a 50% `SparksFart` branch, else dust 107
    /// (`:348-371`).
    TerraFart,
    /// `SparksMech` — dust 260, the 213 body with a +0.5 fadeIn (`:373-392`).
    Mech,
    /// `SparksMeow` — the 213 body + the Meowmere shader (`:394-414`, the
    /// shader is the documented `Dust.shader` drop).
    Meow,
}

/// The row's `MinecartDust` delegate (`Mount.cs` registrations). The base
/// `Sparks` is the default for every cart row that does not override it.
pub(crate) fn spark_variant_of(mount_kind: i32) -> SparkVariant {
    match mount_kind {
        51 => SparkVariant::Fart,      // `:1099`
        53 => SparkVariant::TerraFart, // `:1107`
        11 => SparkVariant::Mech,      // `:1487`
        33 => SparkVariant::Meow,      // `:1013`
        _ => SparkVariant::Sparks,     // `:2751`/`:916`/`:1595`/`:1047`
    }
}

/// The resolved per-spawn delegate body — the TerraFart 50% branch
/// (`DelegateMethods.cs:350`) is rolled at QUEUE time (it precedes the side
/// offset + speed rolls), so the drain sees a concrete body. `Fart` here is
/// either the 51 delegate or the TerraFart branch's `SparksFart` call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SparkBody {
    /// dust 213 (`Sparks`/`SparksMeow`, the shader is a drop).
    Dust213,
    /// dust 260 (`SparksMech`).
    Dust260,
    /// dust 211 (`SparksFart`).
    Dust211,
    /// dust 107 (`SparksTerraFart`'s green arm).
    Dust107,
}

/// Live population of one dust type in the pool — the tests' read and the
/// sand budget's `Dust.SandStormCount` stand-in (`Sandstorm.cs:166`).
pub(crate) fn live_type(pool: &tr_core::particles::DustPool, ty: i32) -> usize {
    pool.slots()[..tr_core::particles::DUST_SENTINEL]
        .iter()
        .filter(|d| d.active && d.dust_type == ty)
        .count()
}

/// The drain-time half of the cart's `MinecartDust` delegate
/// (`DelegateMethods.Minecart.Sparks` `DelegateMethods.cs:227-242` and its
/// variants): the pool's verbatim spawn — its rolls ride the client's stream —
/// then the delegate's post-spawn arms. The arms roll even when the pool
/// refused: the C# blindly writes `Main.dust[6000]`, the sentinel slot
/// (`Dust.cs:79`), which is never scanned or drawn.
pub(crate) fn spawn_spark<R: tr_core::particles::DustRng>(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut R,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    spawn: SparkSpawn,
) -> usize {
    match spawn.body {
        SparkBody::Dust213 => spawn_spark_213(pool, rng, ctx, world, spawn),
        SparkBody::Dust260 => spawn_spark_mech(pool, rng, ctx, world, spawn),
        SparkBody::Dust211 => spawn_spark_fart(pool, rng, ctx, world, spawn),
        SparkBody::Dust107 => spawn_spark_terra(pool, rng, ctx, world, spawn),
    }
}

/// `DelegateMethods.Minecart.Sparks` (`DelegateMethods.cs:226-241`) — the
/// type-213 body. `SparksMeow` (`:397-413`) is the SAME body plus the
/// `dust.shader = GameShaders.Armor.GetShaderFromItemId(2870)` write (`:398`,
/// the documented `Dust.shader` drop — no draw-shader chain).
fn spawn_spark_213<R: tr_core::particles::DustRng>(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut R,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    spawn: SparkSpawn,
) -> usize {
    // `DelegateMethods.cs:226`: `Dust.NewDust(dustPosition, 1, 1, 213,
    // Next(-2,3), Next(-2,3))` — the speed args were rolled at queue time
    // ([`wheel_sparks`]); the pool rolls its own chain on the same stream.
    let idx = pool.new_dust(
        rng,
        ctx,
        world,
        spawn.pos,
        1,
        1,
        213,
        spawn.speed_x,
        spawn.speed_y,
        0,
        tr_core::particles::Color::default(),
        1.0,
    );
    // `:227-229`: `noGravity = true` twice (verbatim), then the fadeIn band
    // over the pool's rolled scale.
    let slot = pool.slot_mut(idx);
    slot.no_gravity = true;
    slot.fade_in = slot.scale + 1.0 + 0.01 * rng.next_max(51) as f32;
    slot.no_gravity = true;
    // `:230-232`: the velocity recipe — three rolls, then the `:233` lift.
    slot.velocity *= rng.next_range(15, 51) as f32 * 0.01;
    slot.velocity.x *= rng.next_range(25, 101) as f32 * 0.01;
    slot.velocity.y -= rng.next_range(15, 31) as f32 * 0.1;
    slot.position.y -= 4.0;
    // `:234-241`: 2-in-3 the spark gains gravity; otherwise it stays
    // weightless and shrinks.
    if rng.next_max(3) != 0 {
        slot.no_gravity = false;
    } else {
        slot.scale *= 0.6;
    }
    idx
}

/// The per-tick player snapshot the physics chain reads as
/// `Main.player[Main.myPlayer]` statics (`DustPhysicsEnv`'s player half:
/// 27/45 `:1679-1686`, 76 `:1749-1756`, 268 `:1799-1806`, 235 `:2091`).
/// The no-player default is all-zero velocity / dry / the default-constructed
/// `Player.Center` `(10, 21)` (`Player.cs` ctor, width 20 / height 42 at the
/// origin — the `dust_physics` module-doc fallback).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DustPlayerSnap {
    pub vx: f32,
    pub vy: f32,
    pub wet: bool,
    pub center: Vec2,
}

impl Default for DustPlayerSnap {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            wet: false,
            center: Vec2::new(10.0, 21.0),
        }
    }
}

/// `Main.UpdateWorld_Dust`'s single ported pass (`Main.cs:18045-18047`): the
/// shared `Dust.UpdateDust` loop (`Dust.cs:420-2423`,
/// `DustPool::update_dust`) drives the head (incl. the `pre_move` shrink/lift
/// + the canonical inline lava-bubble increment, `:468-476`), the move and
/// the kills, and the hook is the W4-d50 physics chain
/// (`tr_core::dust_physics::type_chain`, `:478-2334`) over the
/// `DustPhysicsEnv` seam — the 213/268 hand-dispatches retired into it
/// (research/69 §4-6+8). ONE pass per frame: the C# calls `UpdateDust` once,
/// so every active slot integrates exactly once. The hook's bool is the
/// SECOND-chain match (`:2130-2334`), suppressing the skeleton's `:2335`
/// gravity tail exactly as the C#'s else-if does; the FIRST-ladder arms
/// (213/268 included) return false and take the tail alongside.
/// Post-pass: the `SandStormCount`/`snowDust` env deltas land on the pool
/// (the head resets ride `update_dust`, `:428-429` — the d54 ONE-EDIT hunk
/// `84eb739b` landing shape), and the child-spawn sink drains through the
/// verbatim `NewDust` allocator (the `wants_*` sink precedent; stream-neutral
/// for the current consumers — 213/268 spawn no children).
/// `spawn_ctx` is the child-drain `NewDust` ctx (the App's `spawn_ctx()`);
/// `light_color` is the `Lighting.GetColor` probe (43's death check,
/// `:1405`) — the caller passes the bright default until the S1-E-5 sample
/// seam lands (`dust_physics` module docs).
/// `DelegateMethods.Minecart.SparksMech` (`DelegateMethods.cs:376-391`) — the
/// 213 body with dust 260 and a `+0.5` fadeIn band (`:378`, vs the 213 `+1`).
fn spawn_spark_mech<R: tr_core::particles::DustRng>(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut R,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    spawn: SparkSpawn,
) -> usize {
    // `:376`: `Dust.NewDust(dustPosition, 1, 1, 260, Next(-2,3), Next(-2,3))`.
    let idx = pool.new_dust(
        rng,
        ctx,
        world,
        spawn.pos,
        1,
        1,
        260,
        spawn.speed_x,
        spawn.speed_y,
        0,
        tr_core::particles::Color::default(),
        1.0,
    );
    let slot = pool.slot_mut(idx);
    slot.no_gravity = true;
    // `:378`: the fadeIn band is `scale + 0.5 + 0.01*Next(0,51)`.
    slot.fade_in = slot.scale + 0.5 + 0.01 * rng.next_max(51) as f32;
    slot.no_gravity = true;
    // `:380-383`: the same velocity recipe + lift.
    slot.velocity *= rng.next_range(15, 51) as f32 * 0.01;
    slot.velocity.x *= rng.next_range(25, 101) as f32 * 0.01;
    slot.velocity.y -= rng.next_range(15, 31) as f32 * 0.1;
    slot.position.y -= 4.0;
    // `:384-391`: the same 2-in-3 gravity flip.
    if rng.next_max(3) != 0 {
        slot.no_gravity = false;
    } else {
        slot.scale *= 0.6;
    }
    idx
}

/// `DelegateMethods.Minecart.SparksFart` (`DelegateMethods.cs:330-345`) —
/// dust 211, alpha 50, scale 0.8; two independent 50% `alpha += 25` rolls,
/// `noLight = true`, the `Next(3) == 0` gravity pick, then the velocity
/// recipe + lift. NO fadeIn band (the 213 body's is absent here).
fn spawn_spark_fart<R: tr_core::particles::DustRng>(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut R,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    spawn: SparkSpawn,
) -> usize {
    // `:331`: `Dust.NewDust(dustPosition, 1, 1, 211, Next(-2,3), Next(-2,3),
    // 50, default, 0.8f)`.
    let idx = pool.new_dust(
        rng,
        ctx,
        world,
        spawn.pos,
        1,
        1,
        211,
        spawn.speed_x,
        spawn.speed_y,
        50,
        tr_core::particles::Color::default(),
        0.8,
    );
    let slot = pool.slot_mut(idx);
    // `:332-339`: two independent `Next(2) == 0` alpha bumps.
    if rng.next_max(2) == 0 {
        slot.alpha += 25;
    }
    if rng.next_max(2) == 0 {
        slot.alpha += 25;
    }
    // `:340-341`: `noLight = true`, then the gravity pick.
    slot.no_light = true;
    slot.no_gravity = rng.next_max(3) == 0;
    // `:342-345`: the velocity recipe + lift.
    slot.velocity *= rng.next_range(15, 51) as f32 * 0.01;
    slot.velocity.x *= rng.next_range(25, 101) as f32 * 0.01;
    slot.velocity.y -= rng.next_range(15, 31) as f32 * 0.1;
    slot.position.y -= 4.0;
    idx
}

/// `DelegateMethods.Minecart.SparksTerraFart`'s green arm
/// (`DelegateMethods.cs:355-370`) — dust 107, alpha 100, scale 0.8, the
/// `Color.Lerp(new Color(64,220,96), White, NextFloat()*0.3)` spawn colour
/// (the NextFloat draws AFTER the two speed args, inside the NewDust arg
/// list), two 50% alpha bumps, `noLightEmittance = true`, the gravity pick,
/// then the velocity recipe + lift.
fn spawn_spark_terra<R: tr_core::particles::DustRng>(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut R,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    spawn: SparkSpawn,
) -> usize {
    // `:356`: the colour's `NextFloat()*0.3f` lerp amount draws here (the C#
    // evaluates the NewDust args left-to-right: the two speeds were rolled at
    // queue time, the colour's NextFloat is the next draw).
    let amount = rng.next_float() * 0.3;
    let color = terra_fart_color(amount);
    let idx = pool.new_dust(
        rng,
        ctx,
        world,
        spawn.pos,
        1,
        1,
        107,
        spawn.speed_x,
        spawn.speed_y,
        100,
        color,
        0.8,
    );
    let slot = pool.slot_mut(idx);
    // `:357-364`: two independent `Next(2) == 0` alpha bumps.
    if rng.next_max(2) == 0 {
        slot.alpha += 25;
    }
    if rng.next_max(2) == 0 {
        slot.alpha += 25;
    }
    // `:365-366`: `noLightEmittance = true`, then the gravity pick.
    slot.no_light_emittance = true;
    slot.no_gravity = rng.next_max(3) == 0;
    // `:367-370`: the velocity recipe + lift.
    slot.velocity *= rng.next_range(15, 51) as f32 * 0.01;
    slot.velocity.x *= rng.next_range(25, 101) as f32 * 0.01;
    slot.velocity.y -= rng.next_range(15, 31) as f32 * 0.1;
    slot.position.y -= 4.0;
    idx
}

/// `Color.Lerp(new Color(64, 220, 96), Color.White, amount)` — XNA's
/// per-channel byte lerp, truncated (the `mount_dust::color_lerp` convention).
fn terra_fart_color(amount: f32) -> tr_core::particles::Color {
    let a = tr_core::particles::Color::new(64, 220, 96, 255);
    let b = tr_core::particles::Color::WHITE;
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * amount) as u8;
    tr_core::particles::Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// The cart-spark `Dust.UpdateDust` pass over the pool's 213/260/211/107 slots,
/// in the C# source order. Death deactivates the slot (`Dust.cs:449` and
/// passim) — the pool keeps its 6000 fixed slots.
/// `Main.UpdateWorld_Dust`'s single ported pass (`Main.cs:18045-18047`): the
/// shared `Dust.UpdateDust` loop (`Dust.cs:420-2423`,
/// `DustPool::update_dust`) drives the head, the move and the kills, and this
/// entry dispatches the ported per-type arms — the wheel sparks' 213 + the
/// variant delegates' 260/211/107 (this file, W4-f6 final 4) and the flying
/// sand's 268 (`sand.rs`). ONE pass per frame: the C# calls `UpdateDust` once,
/// so every active slot integrates exactly once (the two clients' separate
/// tick loops were the batch-review defect).
pub(crate) fn tick(
    pool: &mut tr_core::particles::DustPool,
    rng: &mut impl tr_core::particles::DustRng,
    max_dust_to_draw: i32,
    camera: Vec2,
    screen: Vec2,
    zoom: f32,
    world: &tr_core::World,
    player: DustPlayerSnap,
    show_sandstorm_visuals: bool,
    spawn_ctx: &tr_core::particles::SpawnCtx,
) {
    // The `Collision` probes bind the landed tr-app ports (the same seam
    // shape as the pool's `WetProbe`): `Collision.SolidCollision`
    // (`Collision.cs:2700-2734`) and `WetCollision` (`:1572-1651`).
    let solid = |pos: Vec2, w: f32, h: f32| {
        crate::player::Player::solid_collision(world, pos.x, pos.y, w, h)
    };
    let wet = |pos: Vec2, w: f32, h: f32| {
        crate::player::wet_collision(world, pos.x, pos.y, w, h)
    };
    // The bright default: 43 never spuriously dies until the real
    // `Lighting.GetColor` sample rides the light pass (S1-E-5).
    let light = |_: i32, _: i32| tr_core::particles::Color::WHITE;
    let mut spawns = Vec::new();
    let mut sand = 0;
    let mut snow = 0;
    // `Main.snowDust` (`Main.cs:621`): the head reset (`Dust.cs:428`) rides
    // `update_dust`; the type-76 arm's `++` (`:1747`) accumulates here and
    // lands on the pool after the pass (the snowfall budget reads it).
    let mut snow_dust = 0;
    pool.update_dust(
        &tr_core::particles::UpdateCtx {
            max_dust_to_draw,
            screen_position: camera,
            screen_height: screen.y * 0.5 / zoom,
        },
        rng,
        |d: &mut tr_core::particles::Dust, rng: &mut _| {
            let mut env = tr_core::dust_physics::DustPhysicsEnv {
                world,
                show_sandstorm_visuals,
                player_vx: player.vx,
                player_vy: player.vy,
                player_wet: player.wet,
                player_center: player.center,
                solid_collision: &solid,
                wet_collision: &wet,
                light_color: &light,
                spawns: &mut spawns,
                sand_storm_count: &mut sand,
                snow_dust: &mut snow,
            };
            tr_core::dust_physics::type_chain(d, rng, &mut env)
        },
    );
    // `Dust.SandStormCount` (`:1791`) + `Main.snowDust` (`:1747`): the head
    // resets zeroed both, so the post-pass landing is the exact count (the
    // snowfall budget reads `pool.snow_dust`, `Main.cs:13057`).
    pool.sand_storm_count += sand;
    pool.snow_dust = snow;
    // Drain the child `NewDust` requests (159/164/173/160/162): the verbatim
    // allocator (default Speed/Alpha/Color, Scale=1) then the C#'s post-writes
    // (`Dust.cs:1086-1088` family): `noGravity = true`, the folded scale. The
    // sentinel slot is written too on a refusal — the C#'s shape.
    for s in spawns {
        let idx = pool.new_dust(
            rng,
            spawn_ctx,
            world,
            s.position,
            s.width,
            s.height,
            s.dust_type,
            0.0,
            0.0,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
        let d = pool.slot_mut(idx);
        d.no_gravity = true;
        d.scale = s.scale;
    }
        |d: &mut tr_core::particles::Dust| {
            if d.dust_type == 213 || d.dust_type == 260 {
                // `:1819-1829`: the spark falls (260 is not 107/41/44/309).
                if !d.no_gravity {
                    d.velocity.y += 0.1;
                }
                // `:2071-2072`: the air drag.
                d.velocity.x *= 0.99;
                // `:739` + `:2080`: rotation is rebuilt from the dragged
                // velocity (the 213|260 arm zeroes it first, so `=` not `+=`).
                d.rotation = d.velocity.x * 0.5;
                // `:2083-2124`: the fadeIn ramp — +0.1/tick while the spawn
                // fadeIn stands (capped at it), then −0.2/tick (it burns out).
                if d.fade_in > 0.0 && d.fade_in < 100.0 {
                    d.scale += 0.1;
                    if d.scale > d.fade_in {
                        d.fade_in = 0.0;
                    }
                } else {
                    d.scale -= 0.2;
                }
            } else if d.dust_type == 211 {
                // `:729-732`: the SparksFart solid-kill — `noLight` (the
                // delegate sets it) over a solid tile deactivates. Runs before
                // the gravity chain (the C# standalone arm precedes `:1819`).
                if d.no_light
                    && crate::player::Player::solid_collision(world, d.position.x, d.position.y, 4.0, 4.0)
                {
                    d.active = false;
                }
                // `:1819-1829`: the spark falls (211 is not 107/41/44/309).
                if !d.no_gravity {
                    d.velocity.y += 0.1;
                }
                // `:2071-2072`: the air drag.
                d.velocity.x *= 0.99;
                // `:2079-2081`: rotation ACCUMULATES (211 is not in the
                // `:737` 213|260 zero-arm, so `+=` not `=`).
                d.rotation += d.velocity.x * 0.5;
                // `:2083-2128`: the GENERIC fadeIn ramp — 211 is in no special
                // arm: +0.03/tick while fadeIn stands, else −0.01/tick. The
                // SparksFart delegate sets no fadeIn, so it burns at −0.01.
                if d.fade_in > 0.0 && d.fade_in < 100.0 {
                    d.scale += 0.03;
                    if d.scale > d.fade_in {
                        d.fade_in = 0.0;
                    }
                } else {
                    d.scale -= 0.01;
                }
            } else if d.dust_type == 107 {
                // `:1819-1824`: the 107 gravity arm — `velocity *= 0.9` (BOTH
                // axes) instead of the +0.1 fall, gated on !noGravity.
                if !d.no_gravity {
                    d.velocity *= 0.9;
                }
                // `:2071-2072`: the air drag.
                d.velocity.x *= 0.99;
                // `:2079-2081`: rotation accumulates (107 not in the zero-arm).
                d.rotation += d.velocity.x * 0.5;
                // `:2083-2128`: the generic fadeIn ramp (+0.03 / −0.01). The
                // TerraFart spark sets no fadeIn; the SpawnFartCloud dust sets
                // fadeIn 0.2..0.3 (`DelegateMethods.cs:301`).
                if d.fade_in > 0.0 && d.fade_in < 100.0 {
                    d.scale += 0.03;
                    if d.scale > d.fade_in {
                        d.fade_in = 0.0;
                    }
                } else {
                    d.scale -= 0.01;
                }
            } else if d.dust_type == 268 {
                crate::sand::type_268_arm(d, show_sandstorm_visuals, player_vy, world);
            } else if d.dust_type == 76 {
                // `Dust.cs:1745-1763` — the ambient snowfall's flake AI
                // (W4-d130). The arm feeds `Main.snowDust` (`:1747`); the
                // shared tails (the second chain's drag `:2073`, the rotation
                // rebuild `:2081`, the fadeIn-else fade `:2127`) run after it.
                // The FIRST ladder's gravity else-if (`:1819`) is excluded by
                // the matched arm — the flake keeps its own fall velocity.
                crate::snow::type_76_arm(d, player_vy, world, &mut snow_dust);
                crate::snow::type_76_tails(d);
            } else {
                // g67 (transplanted, docs/research/83 §3): the rain collision
                // splashes (`dustWater()`'s family) ride the shared loop's
                // `Dust.cs:1834-1874` arm — it gates on the C#'s full member
                // list (308/52/266 included) and no-ops every other type.
                crate::rain::water_splash_arm(d, world);
            }
            // All arms return false: the 213/268/splash arms are standalone
            // All arms return false: the 213/260/211/107/268 arms are standalone
            // statements of the C#'s FIRST ladder (between the `:477` move and
            // the fresh `if (dust.type >= 130 ...)` chain head at `:2127`), so
            // they do NOT exclude the second chain's final
            // `else if (type != 304 && noGravity)` tail (`:2335`) — a
            // no-gravity 213 takes its arm AND the tail, exactly like the C#.
            // The 76 arm is an else-if OF the first ladder (`:1745`), so it
            // excludes the `:2071` drag — its own tails ran above.
            // no-gravity spark takes its arm AND the tail, exactly like the C#.
            false
        },
    );
    pool.snow_dust = snow_dust;
}

/// `Main.DrawDust`'s draw over the pool's cart-spark slots (213/260/211/107).
/// The 213/260 sparks are self-lit (`Dust.cs:2498-2501`: a scale-scaled gray
/// with no light term; the `GetColor` overlay never fires — the spawn colour is
/// default-black, `Main.cs:39006`); 213 forces draw scale 1 (`:38974-38977`),
/// 260 keeps `GetVisualScale`. The 211/107 variants are light-tinted: the
/// generic base + `GetColor` overlay path (`Main.cs:39005-39013`) with the
/// full-dark kill (`:39015-39017`), the `sand::draw` convention.
pub(crate) fn draw(
    pool: &mut tr_core::particles::DustPool,
    r: &mut tr_render::sprite::SpriteRenderer,
    lookups: &crate::assets::Lookups,
    light: &tr_core::lighting::LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    let to_screen = |p: Vec2| {
        [
            (p.x - camera.x) * zoom + screen.x * 0.5,
            (p.y - camera.y) * zoom + screen.y * 0.5,
        ]
    };
    for i in 0..tr_core::particles::DUST_SENTINEL {
        let d = pool.slot(i);
        if !d.active {
            continue;
        }
        match d.dust_type {
            213 | 260 => {
                // `Dust.frame` (`Dust.cs:177-189`): the pool's per-100 shift
                // gives the 8×8 cell (213: x=130; 260: x=600, y=60+10*row).
                let Some(uv) =
                    lookups.dust_uv(d.frame_x, d.frame_y, d.frame_w as u32, d.frame_h as u32)
                else {
                    continue;
                };
                // `Dust.cs:2498-2501` — the 213|260 arm: `int num2 =
                // (int)(scale / 2.5f * 255f)` through the ctor's byte cast. The
                // arm ignores the light colour, so the argument is arbitrary.
                let a = tr_core::dust_table::get_alpha(d, tr_core::particles::Color::WHITE).a
                    as f32
                    / 255.0;
                // `Main.cs:38974-38977`: 213 forces draw scale 1; 260 keeps
                // `GetVisualScale` (= scale, `Dust.cs:2782-2789`).
                let scale = if d.dust_type == 213 { 1.0 } else { d.scale };
                let size = 8.0 * scale * zoom;
                r.push_fx_sprite_rot(to_screen(d.position), [size, size], uv, [a, a, a, a], d.rotation);
            }
            211 | 107 => {
                // The light-tinted generic path (`Main.cs:39005-39017`).
                let pos = d.position;
                let c = crate::rain::light_at(light, pos);
                let light_color = tr_core::particles::Color::new(
                    (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );
                let base = tr_core::dust_table::get_alpha(d, light_color);
                // `Main.cs:39015-39017`: `newColor == Color.Black` deactivates.
                if base.r == 0 && base.g == 0 && base.b == 0 && base.a == 0 {
                    pool.slot_mut(i).active = false;
                    continue;
                }
                let Some(uv) =
                    lookups.dust_uv(d.frame_x, d.frame_y, d.frame_w as u32, d.frame_h as u32)
                else {
                    continue;
                };
                let size = 8.0 * d.scale * zoom;
                let tint = [
                    base.r as f32 / 255.0,
                    base.g as f32 / 255.0,
                    base.b as f32 / 255.0,
                    base.a as f32 / 255.0,
                ];
                r.push_fx_sprite_rot(to_screen(pos), [size, size], uv, tint, d.rotation);
                // `Main.cs:39006-39013`: the `GetColor` overlay when the spawn
                // colour is non-zero (the 107 green lerp; 211 keeps default).
                if d.color.r != 0 || d.color.g != 0 || d.color.b != 0 || d.color.a != 0 {
                    let oc = tr_core::dust_table::get_color(d, base);
                    if oc.r != 0 || oc.g != 0 || oc.b != 0 || oc.a != 0 {
                        let ot = [
                            oc.r as f32 / 255.0,
                            oc.g as f32 / 255.0,
                            oc.b as f32 / 255.0,
                            oc.a as f32 / 255.0,
                        ];
                        r.push_fx_sprite_rot(to_screen(pos), [size, size], uv, ot, d.rotation);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `Main.rand.Next(max)` over the port's splitmix stream (the sand.rs
/// convention: `Next(0)` rolls 0).
fn next_int(rng: &mut u64, max: u32) -> u32 {
    (crate::app::next_rand(rng) % max.max(1) as u64) as u32
}

/// `Main.rand.Next(min, max)` — `min + roll % (max - min)`.
fn next_range(rng: &mut u64, min: i32, max_exclusive: i32) -> i32 {
    min + (crate::app::next_rand(rng) % (max_exclusive - min) as u64) as i32
}

/// `Minecart.WheelSparks` (`Minecart.cs:1623-1630`): the magnet-formula spark
/// point, then `spark_count` delegate calls. Each call queues one request —
/// the pre-`NewDust` rolls of the row's `MinecartDust` delegate
/// (`DelegateMethods.cs:225-226` and variants) on the client's stream; the
/// pool's rolls ride the same stream at drain time ([`spawn_spark`]).
/// `variant` is the row's delegate ([`spark_variant_of`]); the TerraFart 50%
/// branch (`:350`) rolls HERE, before the side offset + speed rolls (the C#
/// `if (Main.rand.Next(2) == 0)` precedes the `SparksFart`/green-arm body).
pub(crate) fn wheel_sparks(
    sparks: &mut Vec<SparkSpawn>,
    rng: &mut u64,
    position: Vec2,
    width: i32,
    height: i32,
    spark_count: u32,
    rotation: f32,
    variant: SparkVariant,
) {
    // `:1625` — `Position + (Width/2 − TextureWidth/2, Height/2) +
    // MagnetOffset`, the same arithmetic as `GetMagnetPosition` (`:1290-1294`).
    let at = magnet_position(position, width, height);
    for _ in 0..spark_count {
        // The TerraFart branch pick (`DelegateMethods.cs:350`) draws FIRST;
        // the Fart arm then re-rolls its OWN side offset (`:330`, the C#
        // calls `SparksFart(dustPosition)` with the un-offset point).
        let body = match variant {
            SparkVariant::Sparks | SparkVariant::Meow => SparkBody::Dust213,
            SparkVariant::Mech => SparkBody::Dust260,
            SparkVariant::Fart => SparkBody::Dust211,
            SparkVariant::TerraFart => {
                if next_int(rng, 2) == 0 {
                    SparkBody::Dust211
                } else {
                    SparkBody::Dust107
                }
            }
        };
        // `DelegateMethods.cs:225`: the ±13 side offset rotated by
        // `Minecart.rotation` (`Vector2.RotatedBy`: (x cosθ − y sinθ,
        // x sinθ + y cosθ)).
        let side: f32 = if next_int(rng, 2) == 0 { -13.0 } else { 13.0 };
        let rotated = Vec2::new(13.0 * rotation.cos(), 13.0 * rotation.sin()) * side.signum();
        // `:226`: the `Next(-2,3)` speed arguments — the C# evaluates the
        // `NewDust` args before the call.
        let speed_x = next_range(rng, -2, 3) as f32;
        let speed_y = next_range(rng, -2, 3) as f32;
        sparks.push(SparkSpawn {
            pos: at + rotated,
            speed_x,
            speed_y,
            body,
        });
    }
}

/// `Player.cs:19868-19902` (the left arm, `velocity.X >= 0`) /
/// `:19926-19958` (the right arm, `velocity.X <= 0`): the Item55 cue, then
/// the threshold chain over `|velocity.X|` vs `max_run_speed`.
pub(crate) fn rolling_wheel_sparks(
    sounds: &mut Vec<CartCue>,
    sparks: &mut Vec<SparkSpawn>,
    rng: &mut u64,
    position: Vec2,
    width: i32,
    height: i32,
    velocity: Vec2,
    max_run_speed: f32,
    rotation: f32,
    variant: SparkVariant,
) {
    // `:19870/:19928`: Item55 plays on every gate-passing tick, before the
    // rolls, at `(int)position + (width/2, height/2)`.
    sounds.push(CartCue {
        cue: "Item_55",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
    // `:19871-19872/:19929-19930`: the emitter statics take the ride tilt
    // first (`Minecart.rotation = fullRotation`) — each `sparks_one` below
    // receives it as its `rotation` argument.
    // `:19873-19886`: the f64 threshold compare — `(double)|vx| >
    // (double)max * 0.66` — three independent 50% rolls, each one spark at
    // `position + velocity*0.66/0.33/position`.
    if (velocity.x.abs() as f64) > (max_run_speed as f64) * 0.66 {
        if next_int(rng, 2) == 0 {
            wheel_sparks(
                sparks,
                rng,
                position + velocity * 0.66,
                width,
                height,
                1,
                rotation,
                variant,
            );
        }
        if next_int(rng, 2) == 0 {
            wheel_sparks(
                sparks,
                rng,
                position + velocity * 0.33,
                width,
                height,
                1,
                rotation,
                variant,
            );
        }
        if next_int(rng, 2) == 0 {
            wheel_sparks(sparks, rng, position, width, height, 1, rotation, variant);
        }
    }
    // `:19888-19898`: `> 0.33·max` — two 2-in-3 rolls at `*0.5`/`position`.
    else if (velocity.x.abs() as f64) > (max_run_speed as f64) * 0.33 {
        if next_int(rng, 3) != 0 {
            wheel_sparks(
                sparks,
                rng,
                position + velocity * 0.5,
                width,
                height,
                1,
                rotation,
                variant,
            );
        }
        if next_int(rng, 3) != 0 {
            wheel_sparks(sparks, rng, position, width, height, 1, rotation, variant);
        }
    }
    // `:19900-19902`: the guaranteed single spark at `position`.
    else {
        wheel_sparks(sparks, rng, position, width, height, 1, rotation, variant);
    }
}

/// `DelegateMethods.Minecart.LandingSound` (`DelegateMethods.cs:248-251`):
/// Item53 at `(int)Position + (Width/2, Height/2)`.
pub(crate) fn minecart_landing_sound(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    sounds.push(CartCue {
        cue: "Item_53",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
}

/// `DelegateMethods.Minecart.BumperSound` (`DelegateMethods.cs:253-256`):
/// Item56, same position rule.
pub(crate) fn minecart_bumper_sound(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    sounds.push(CartCue {
        cue: "Item_56",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
}

/// W4-c14 - `DelegateMethods.Minecart.BumperSoundFart`
/// (`DelegateMethods.cs:321-326`, `Mount.cs:1100`/`:1108`): Item16 THEN
/// Item56 - two plays in the C# order, both at the centre rule; the fart
/// cloud (the `useDelay` default true) is the visual seam.
pub(crate) fn minecart_bumper_sound_fart(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    let x = (position.x as i32 + width / 2) as f32;
    let y = (position.y as i32 + height / 2) as f32;
    sounds.push(CartCue {
        cue: "Item_16",
        x,
        y,
    });
    sounds.push(CartCue {
        cue: "Item_56",
        x,
        y,
    });
}

/// W4-c14 - `Mount.MeowcartLandingSound` (`Mount.cs:659-662`, the Meowmere
/// cart 33's delegate `:1014`): legacy id 37 style 5 at the centre rule.
/// The cue name routes the drain to `Audio::play_meowcart_sound` (the
/// case-37 slot pick, NOT an Item-style play).
pub(crate) fn meowcart_landing_sound(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    sounds.push(CartCue {
        cue: "MeowcartLanding",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
}

/// W4-c14 - `Mount.MeowcartBumperSound` (`Mount.cs:664-667`, `:1015`):
/// legacy id 37 style 3, same position rule.
pub(crate) fn meowcart_bumper_sound(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    sounds.push(CartCue {
        cue: "MeowcartBumper",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
}

/// W4-c14 - `DelegateMethods.Minecart.JumpingSoundFart`
/// (`DelegateMethods.cs:308-312`, the Fart/Terra-Fart carts' delegate,
/// `Mount.cs:1102`/`:1110`): Item16 at the same centre rule, then the
/// fart cloud (the gore/dust visuals are the recorded seam). The base
/// cart's `JumpingSound` is EMPTY (`:244-246`) - only the fart carts play.
pub(crate) fn minecart_jumping_sound_fart(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    sounds.push(CartCue {
        cue: "Item_16",
        x: (position.x as i32 + width / 2) as f32,
        y: (position.y as i32 + height / 2) as f32,
    });
}

/// W4-c14 - `DelegateMethods.Minecart.LandingSoundFart`
/// (`DelegateMethods.cs:314-319`, `Mount.cs:1101`/`:1109`): Item16 THEN
/// Item53 - two plays in the C# order, both at the centre rule; the fart
/// cloud is the visual seam.
pub(crate) fn minecart_landing_sound_fart(
    sounds: &mut Vec<CartCue>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    let x = (position.x as i32 + width / 2) as f32;
    let y = (position.y as i32 + height / 2) as f32;
    sounds.push(CartCue {
        cue: "Item_16",
        x,
        y,
    });
    sounds.push(CartCue {
        cue: "Item_53",
        x,
        y,
    });
/// `Minecart._trackSwitchOptions` (`Minecart.cs:449-518`): for each of the 64
/// nearby-track bitmasks, the ascending list of frame ids whose side-connection
/// mask fits the neighbourhood. A frame's mask is `left` (0→1, 1→2, 2→4) |
/// `right` (0→8, 1→0x10, 2→0x20); a dead-end side (−1) contributes 0. With
/// fewer than 2 neighbour tracks the mask must match exactly; otherwise it
/// must be a non-zero subset. `None` when no frame fits (the C# null array).
/// Built ONCE for all 64 masks (the C# `Initialize` loop; the W4-f11 OnceLock
/// shape merged per research-71 §3b.2 — output identical, no per-hit Vec).
fn track_switch_options(mask: u32) -> Option<&'static [i32]> {
    static OPTIONS: std::sync::OnceLock<Vec<Option<Vec<i32>>>> = std::sync::OnceLock::new();
    OPTIONS.get_or_init(|| {
        (0..64u32)
            .map(|mask| {
                let neighbour_count = mask.count_ones();
                let mut options = Vec::new();
                for (frame, data) in TRACK_FRAMES.iter().enumerate() {
                    let mut frame_mask = 0u32;
                    match data.left_connection {
                        0 => frame_mask |= 1,
                        1 => frame_mask |= 2,
                        2 => frame_mask |= 4,
                        _ => {}
                    }
                    match data.right_connection {
                        0 => frame_mask |= 8,
                        1 => frame_mask |= 0x10,
                        2 => frame_mask |= 0x20,
                        _ => {}
                    }
                    let fits = if neighbour_count < 2 {
                        mask == frame_mask
                    } else {
                        frame_mask != 0 && (mask & frame_mask) == frame_mask
                    };
                    if fits {
                        options.push(frame as i32);
                    }
                }
                if options.is_empty() { None } else { Some(options) }
            })
            .collect()
    })[mask as usize]
        .as_deref()
}

/// `Minecart.GetNearbyTilesSetLookupIndex` (`Minecart.cs:1170-1198`): the
/// 6-bit mask of the surrounding 314 tracks — left column (x−1) rows y−1/y/y+1
/// are bits 1/2/4, right column (x+1) are bits 8/16/32. The C# tests
/// `type == 314` without an activity check.
fn nearby_track_mask(world: &World, i: i32, j: i32) -> u32 {
    let is_track = |x: i32, y: i32| world.tile(x, y).is_some_and(|t| t.kind == 314);
    let mut mask = 0u32;
    if is_track(i - 1, j - 1) { mask += 1; }
    if is_track(i - 1, j) { mask += 2; }
    if is_track(i - 1, j + 1) { mask += 4; }
    if is_track(i + 1, j - 1) { mask += 8; }
    if is_track(i + 1, j) { mask += 16; }
    if is_track(i + 1, j + 1) { mask += 32; }
    mask
}

/// `Minecart._firstPressureFrame` (`Minecart.cs:519-527`): the first frame whose
/// `_trackType` is 1 (pressure) — frame 20 in the table.
fn first_pressure_frame() -> i32 {
    TRACK_FRAMES
        .iter()
        .position(|d| d.track_type == 1)
        .map_or(-1, |f| f as i32)
}

/// `Minecart.FrameTrack(i, j, pound, mute)` (`Minecart.cs:943-1168`) — the
/// FULL body (research-71 §3b.4 follow-up): the pound switch-cycle branch
/// (`:1040-1108`), the non-pound placement-refit branch (`:976-1038`), the
/// no-options reset (`:966-975`) and the `flag4` change census (`:1110-1142`)
/// + write tail (`:1143-1162`). Returns `(reframed, fail_kill)`: `reframed`
/// is the C# `bool`; `fail_kill` is the `(pound & flag4) && !mute` condition
/// of `:1163-1166` — the C# calls `WorldGen.KillTile(i, j, fail: true)` there,
/// whose 314 slice is the Item52 cue (`WorldGen.cs:66531-66533`) + the
/// `SquareTileFrame` wake (`:63735`); the port hands the condition to the app
/// caller (the sound/cascade seam lives app-side, the `edit::hammer`
/// precedent). Callers: the wiring flip (`pound: true, mute: true`,
/// `Minecart.cs:1315`), the hammer pound (`pound: true`, mute FALSE —
/// `Player.cs:46058`) and the placement refit (`pound: false`,
/// `WorldGen.cs:85292`, `TileFrameImportant` case 314).
///
/// The null-options arm (`:966-975`) is C#-defensive: every one of the 64
/// masks has a fitting frame (each single-bit mask matches a dead-end ramp),
/// so `_trackSwitchOptions[mask]` is never null — kept verbatim, pinned total
/// by the table witness.
fn frame_track(world: &mut World, i: i32, j: i32, pound: bool, mute: bool) -> (bool, bool) {
    // `:955-958`: `mute && tile.type != 314` → false. (The C# null-tile arm
    // `:950-954` allocates a fresh tile; the port's out-of-world read refuses.)
    let Some(tile) = world.tile(i, j).copied() else {
        return (false, false);
    };
    if mute && tile.kind != 314 {
        return (false, false);
    }
    let options = match track_switch_options(nearby_track_mask(world, i, j)) {
        Some(o) => o,
        None => {
            // `:966-975`: a null option list — pound refuses; the placement
            // path resets the front/back to the lone-piece frames (0, −1).
            if pound {
                return (false, false);
            }
            if let Some(t) = world.tile_mut(i, j) {
                t.frame_x = 0;
                t.frame_y = -1;
            }
            return (false, false);
        }
    };
    let front = i32::from(tile.frame_x); // `tile.FrontTrack()` (`:1632-1635`)
    let back = i32::from(tile.frame_y); // `tile.BackTrack()` (`:1642-1645`)
    let front_type = if front >= 0 && (front as usize) < TRACK_FRAMES.len() {
        i32::from(TRACK_FRAMES[front as usize].track_type)
    } else {
        0
    };
    let is_dead_end = |f: i32| {
        let d = &TRACK_FRAMES[f as usize];
        d.left_connection == -1 || d.right_connection == -1
    };
    // `:963-964`: the chosen front/back indices in the option list (−1 = absent).
    let mut num4: i32 = -1;
    let mut num5: i32 = -1;
    if !pound {
        // `:976-1038` — the placement refit: find the current front's index
        // (`num4`, `flag` = it is a dead end) and the first same-type dead-end
        // (`num6`) / through (`num7`) options; a through option WINS over a
        // dead-end front (`:1017-1023`), else the front keeps or falls back to
        // the first dead end — pressure/booster fronts without a same-type
        // option refuse (`:1028-1034`).
        let mut num6: i32 = -1;
        let mut num7: i32 = -1;
        let mut flag = false;
        for (k, &opt) in options.iter().enumerate() {
            if back == opt {
                num5 = k as i32;
            }
            if i32::from(TRACK_FRAMES[opt as usize].track_type) != front_type {
                continue;
            }
            if is_dead_end(opt) {
                if front == opt {
                    num4 = k as i32;
                    flag = true;
                }
                if num6 == -1 {
                    num6 = k as i32;
                }
            } else {
                if front == opt {
                    num4 = k as i32;
                    flag = false;
                }
                if num7 == -1 {
                    num7 = k as i32;
                }
            }
        }
        if num7 != -1 {
            // `:1019` — the C# bitwise `|` on bools (both sides effect-free).
            if (num4 == -1) | flag {
                num4 = num7;
            }
        } else {
            if num4 == -1 {
                match front_type {
                    1 | 2 => return (false, false),
                    _ => {}
                }
                num4 = num6;
            }
            num5 = -1;
        }
    } else {
        // `:1042-1052`: the front/back indices in the option list.
        for (l, &opt) in options.iter().enumerate() {
            if front == opt { num4 = l as i32; }
            if back == opt { num5 = l as i32; }
        }
        // `:1053-1068`: count the same-type options — through (both sides
        // connected) vs dead-end (either side −1).
        let mut through = 0;
        let mut dead_end = 0;
        for &opt in options {
            if i32::from(TRACK_FRAMES[opt as usize].track_type) == front_type {
                if is_dead_end(opt) { dead_end += 1; } else { through += 1; }
            }
        }
        // `:1069-1072`: fewer than two of either kind → no flip.
        if through < 2 && dead_end < 2 {
            return (false, false);
        }
        let flag2 = through == 0; // `:1073`: only dead-ends remain.
        let mut flag3 = false;
        // `:1075-1090`: with through options, advance the BACK index past the
        // current front to the next through option with a different connection.
        // The C# reads `array[num4]` here; a front frame absent from the option
        // list leaves `num4 == -1` and the C# throws `IndexOutOfRangeException`
        // (the module's out-of-range convention — NOT f11's `.max(0)` clamp,
        // research-71 §3b.4).
        if !flag2 {
            assert!(num4 >= 0, "FrameTrack: front frame absent from the switch options (C# IndexOutOfRange)");
            while !flag3 {
                num5 += 1;
                if num5 >= options.len() as i32 {
                    num5 = -1;
                    break;
                }
                let cand = options[num5 as usize];
                let cur = options[num4 as usize];
                let cd = &TRACK_FRAMES[cand as usize];
                let ud = &TRACK_FRAMES[cur as usize];
                if (cd.left_connection != ud.left_connection
                    || cd.right_connection != ud.right_connection)
                    && i32::from(cd.track_type) == front_type
                    && cd.left_connection != -1
                    && cd.right_connection != -1
                {
                    flag3 = true;
                }
            }
        }
        // `:1091-1108`: otherwise advance the FRONT index to the next same-type
        // option whose dead-end-ness matches `flag2` (wrapping once).
        if !flag3 {
            loop {
                num4 += 1;
                if num4 >= options.len() as i32 {
                    num4 = -1;
                    loop {
                        num4 += 1;
                        let opt = options[num4 as usize];
                        if i32::from(TRACK_FRAMES[opt as usize].track_type) == front_type
                            && is_dead_end(opt) == flag2
                        {
                            break;
                        }
                    }
                    break;
                }
                let opt = options[num4 as usize];
                if i32::from(TRACK_FRAMES[opt as usize].track_type) == front_type
                    && is_dead_end(opt) == flag2
                {
                    break;
                }
            }
        }
    }
    // `:1110-1142` — `flag4`: did the front/back CHANGE? (the fail-tile gate).
    let new_front = if num4 == -2 {
        first_pressure_frame()
    } else if num4 == -1 {
        0
    } else {
        options[num4 as usize]
    };
    let new_back = if num5 == -1 { -1 } else { options[num5 as usize] };
    let flag4 = front != new_front || back != new_back;
    // `:1143-1162`: write the new front/back frames.
    if let Some(t) = world.tile_mut(i, j) {
        t.frame_x = new_front as i16;
        t.frame_y = new_back as i16;
    }
    // `:1163-1166`: `(pound & flag4) && !mute` → KillTile(fail: true) — the
    // app caller runs the 314 fail slice (the Item52 cue + the SquareTileFrame
    // wake + the case-314 non-pound refit, `WorldGen.cs:85290-85294`).
    (true, pound && flag4 && !mute)
}

/// The wiring caller's slice — `FrameTrack(pound: true, mute: true)`
/// (`Minecart.cs:1315`, `FlipSwitchTrack` case 2): the fail-kill tail is
/// mute-suppressed, so only the C# bool matters.
fn frame_track_pound(world: &mut World, i: i32, j: i32) -> bool {
    frame_track(world, i, j, true, true).0
}

/// The hammer caller's slice — `FrameTrack(x, y, pound: true)` with mute
/// FALSE (`Player.cs:46058`, `ItemCheck_UseMiningTools_TryPoundingTile`'s
/// 314 arm). Returns the `fail_kill` condition: the app runs the
/// `KillTile(fail: true)` 314 slice when true (the C# bool itself only feeds
/// the `netMode == 1` SendData, `:46058-46061` — SP-discarded).
pub(crate) fn frame_track_hammer(world: &mut World, i: i32, j: i32) -> bool {
    frame_track(world, i, j, true, false).1
}

/// The placement/framing caller's slice — `FrameTrack(i, j, pound: false)`
/// (`WorldGen.cs:85292`, `TileFrameImportant` case 314, reached from
/// `PlaceTile`'s `SquareTileFrame` tail `:60261` and the fail-kill wake
/// `:63735`): refit the front to the neighbourhood, preferring a through
/// connection. The C# bool is unused at the call site; `HandleRopeEndFraming`
/// (`:85293`) is the rope-framing seam (unported — the rope draw module reads
/// the frames live).
pub(crate) fn frame_track_place(world: &mut World, i: i32, j: i32) {
    let _ = frame_track(world, i, j, false, false);
}

/// The `SquareTileFrame` wake's case-314 slice (`WorldGen.cs:80185-80196` →
/// `TileFrame` → `TileFrameImportant` case 314 `:85290-85294`): the 3×3
/// neighbourhood walk refits EVERY active track through
/// `FrameTrack(pound: false)` — the placement tail (`PlaceTile` `:60261`)
/// and the hammer fail-kill wake (`KillTile(fail)` `:63735`) both reach it.
/// The port's `reframe_area` skips frame-important kinds, so this app seam
/// carries the 314 arm (the `SQUARE_FRAME_ORDER` row-major walk, self last
/// of its row — the refit is order-independent per cell: each reads the
/// neighbours' KINDS only, `GetNearbyTilesSetLookupIndex` `:1170-1198`).
/// `HandleRopeEndFraming` (`:85293`) is the rope-framing half — unported
/// (the rope draw module reads the frames live).
pub(crate) fn refit_tracks_around(world: &mut World, x: i32, y: i32) {
    for (dx, dy) in [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1), (0, 0), (0, 1),
        (1, -1), (1, 0), (1, 1),
    ] {
        let (cx, cy) = (x + dx, y + dy);
        // `TileFrame`'s 5-tile edge guard (`WorldGen.cs:81330`).
        if cx <= 5 || cy <= 5 || cx >= world.width - 5 || cy >= world.height - 5 {
            continue;
        }
        if world.tile(cx, cy).is_some_and(|t| t.active() && t.kind == 314) {
            frame_track_place(world, cx, cy);
        }
    }
}

/// `Minecart.GetTrackItem` (`Minecart.cs:1508-1517`): the item a track piece
/// drops when broken — `_trackType[frameX]` 0 → 2340 (Minecart Track),
/// 1 → 2492 (Pressure Track), 2 → 2739 (Booster Track), else 0. The C#
/// indexes `frameX` UNGUARDED (a −1 front throws); the port's out-of-range
/// read returns 0 (the drop-nothing arm — a −1 front only exists between
/// `PlaceTrack` and its refit, never at break time).
pub(crate) fn track_item(frame_x: i16) -> u16 {
    let f = i32::from(frame_x);
    if f < 0 || f as usize >= TRACK_FRAMES.len() {
        return 0;
    }
    match TRACK_FRAMES[f as usize].track_type {
        0 => 2340,
        1 => 2492,
        2 => 2739,
        _ => 0,
    }
}

/// `Minecart.FlipSwitchTrack` (`Minecart.cs:1296-1319`): the wired switch-track
/// flip (`HitWireSingle` case 314, `Wiring.cs:1744-1748`). A normal track
/// (`_trackType` 0) swaps its front/back frames when a back exists; a booster
/// (`_trackType` 2) re-frames through `FrameTrack(pound, mute)`; a pressure
/// track (1) is a no-op. `FrontTrack`/`BackTrack` are `frameX`/`frameY`
/// (`Minecart.cs:1632-1650`).
pub(crate) fn flip_switch_track(world: &mut World, i: i32, j: i32) {
    let Some(tile) = world.tile(i, j).copied() else {
        return;
    };
    let front = i32::from(tile.frame_x);
    if front == -1 {
        return;
    }
    if front < 0 || front as usize >= TRACK_FRAMES.len() {
        return;
    }
    match TRACK_FRAMES[front as usize].track_type {
        0 => {
            let back = i32::from(tile.frame_y);
            if back != -1 {
                if let Some(t) = world.tile_mut(i, j) {
                    t.frame_x = back as i16;
                    t.frame_y = front as i16;
                }
            }
        }
        2 => {
            frame_track_pound(world, i, j);
        }
        _ => {}
/// `DelegateMethods.Minecart.SpawnFartCloud` (`DelegateMethods.cs:258-306`) —
/// the fart carts' (51/53) cloud visual, STAGED (research/198, renamed from 67): the live
/// triggers are c14's fart sound delegates (`JumpingSoundFart` :308-312 /
/// `LandingSoundFart` :314-319 / `BumperSoundFart` :321-326, which call this
/// with `useDelay` false/true respectively) — those land on
/// `task/W4-c14-tile-break-footstep-sounds`, not this branch. `delay` is
/// `Player.fartKartCloudDelay` (`Player.cs:1614`, decremented per track-rider
/// tick at `:28098`, max 20 `:1616`). All rolls ride the caller's
/// `UpdatePlayers` stream in C# order (the delegate runs inline inside the
/// player tick): per gore the `Next(435,438)` type arg evaluates BEFORE the
/// `NewGore` internals (Y lift, X kick); the 53-only dust tail rolls 2
/// position NextFloats + the colour NextFloat + the pool chain + 3 velocity
/// + fadeIn + gravity per dust.
#[allow(dead_code)] // STAGED - c14's fart sound delegates are the live callers
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fart_cloud(
    gore: &mut tr_core::gore::GorePool,
    dust: &mut tr_core::particles::DustPool,
    rng: &mut tr_core::UnifiedRandom,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    position: Vec2,
    width: i32,
    height: i32,
    velocity: Vec2,
    mount_kind: i32,
    delay: &mut i32,
    use_delay: bool,
) {
    // `:260-267`: the delay gate (the bumper arm's useDelay: true).
    if use_delay {
        if *delay > 0 {
            return;
        }
        *delay = 20;
    }
    // `:268-270`: the anchor — `Width / 2` is INTEGER halving.
    let num = 10.0f32;
    let y = -4.0f32;
    let vector = position + Vec2::new((width / 2 - 18) as f32, (height - 16) as f32);
    // `:271-275`: the velocity carry, clamped to length 2.
    let mut vector2 = velocity * 0.1;
    if vector2.length() > 2.0 {
        vector2 = crate::bow_launch::safe_normalize(vector2, Vec2::ZERO) * 2.0;
    }
    // `:276-287`: three gores at (0, -num, +num) X offsets, each the type pick
    // then NewGore(Zero velocity) then the ×0.2 / +carry / Y×0.75 post-writes.
    for off in [0.0, -num, num] {
        let ty = rng.next_range(435, 438); // `:276`/`:280`/`:284` arg
        let idx = gore.new_gore(rng, world, vector + Vec2::new(off, y), Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.2;
        g.velocity += vector2;
        g.velocity.y *= 0.75;
    }
    // `:288-305`: the Terra-Fart (53) only — 15 green 107 dusts.
    if mount_kind == 53 {
        let vector3 = position + Vec2::new((width / 2) as f32, (height + 10) as f32);
        let num3 = 30.0f32;
        let num4 = -16.0f32;
        for _ in 0..15 {
            // `:295`: the position floats draw FIRST (the NewDustPerfect
            // position arg), then the colour's NextFloat inside the arg list.
            let px = -num3 + num3 * 2.0 * rng.next_float();
            let py = num4 * rng.next_float();
            let color = terra_fart_color(rng.next_float() * 0.3);
            let idx = dust.new_dust_perfect(
                rng, ctx, world, vector3 + Vec2::new(px, py),
                107, Some(Vec2::ZERO), 100, color, 0.6,
            );
            let d = dust.slot_mut(idx);
            // `:296-303`: the velocity recipe + carry + the fadeIn/gravity.
            d.velocity *= rng.next_range(15, 51) as f32 * 0.01;
            d.velocity.x *= rng.next_range(25, 101) as f32 * 0.01;
            d.velocity.y -= rng.next_range(15, 31) as f32 * 0.1;
            d.velocity += vector2;
            d.velocity.y *= 0.75;
            d.fade_in = 0.2 + rng.next_float() * 0.1;
            d.no_gravity = rng.next_max(3) == 0;
            d.no_light_emittance = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic stream for the shared tick's ChildSafety arm
    /// (S1-E-3): a fixed seed, no production stream touched.
    fn tr7() -> tr_core::unified_random::UnifiedRandom {
        tr_core::unified_random::UnifiedRandom::new(7)
    }

    /// The solid-ground world the 213 ticks run against.
    fn tworld() -> World {
        World::new("t".into(), 8, 8)
    }
    use tr_core::Tile;
    use tr_core::tile::FLAG_ACTIVE;

    /// The mounted cart body: width 20 (unchanged), height 42 + heightBoost 10
    /// (`Mount.SetAsMinecart`, `Mount.cs:2747-2799`).
    const CART_W: i32 = 20;
    const CART_H: i32 = 52;
    /// The straight-rail surface offset — `TRACK_FRAMES[1]` is level 5
    /// everywhere → 6 px (`Minecart.cs:117-121`, `:131-133`, `:437-447`).
    const STRAIGHT_RAIL_PX: f32 = 6.0;

    fn track_world() -> World {
        World::new("cart".into(), 200, 200)
    }

    /// Hand-places one track tile with explicit frames — placement
    /// (`FrameTrack`) is out of scope, so tests pin the frame ids directly
    /// (the research scope decisions).
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

    #[test]
    fn flip_switch_track_swaps_a_normal_tracks_front_and_back() {
        // `Minecart.cs:1306-1313`: a normal track (type 0) with a back frame
        // swaps front/back. Frame 1 (straight) front, frame 5 back.
        let mut w = track_world();
        place_track(&mut w, 10, 10, 1, 5);
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 5);
        assert_eq!(w.tile(10, 10).unwrap().frame_y, 1);
        // A second flip swaps back.
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 1);
        assert_eq!(w.tile(10, 10).unwrap().frame_y, 5);
    }

    #[test]
    fn flip_switch_track_no_back_is_a_no_op() {
        // `Minecart.cs:1307`: `BackTrack() != -1` gates the swap — a lone
        // normal track (back −1) does not flip.
        let mut w = track_world();
        place_track(&mut w, 10, 10, 1, -1);
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 1);
        assert_eq!(w.tile(10, 10).unwrap().frame_y, -1);
    }

    #[test]
    fn flip_switch_track_pressure_is_a_no_op() {
        // `Minecart.cs:1304-1318`: only types 0 and 2 act; a pressure track
        // (type 1, frame 21) is untouched.
        let mut w = track_world();
        place_track(&mut w, 10, 10, 21, -1);
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 21);
    }

    /// W4-e20 final census — `Minecart.IsPressurePlate`
    /// (`Minecart.cs:549-560`): ONLY the unpressed fronts 20/21 count; the
    /// pressed variants 22/23, every other frame, an inactive tile, a
    /// non-314 kind and the null tile all refuse.
    #[test]
    fn dw_e20_is_pressure_plate_counts_only_the_unpressed_20_21_fronts() {
        let mut w = track_world();
        for fx in [20i16, 21] {
            place_track(&mut w, 10, 10, fx, -1);
            assert!(is_pressure_plate(w.tile(10, 10)), "front {fx} is a plate");
        }
        for fx in [22i16, 23, 0, 1, 30, 31, -1] {
            place_track(&mut w, 10, 10, fx, -1);
            assert!(!is_pressure_plate(w.tile(10, 10)), "front {fx} is not");
        }
        // The actuator-hidden tile still counts: C# reads `tile.active()`,
        // NOT `nactive()` (`:555`).
        place_track(&mut w, 10, 10, 20, -1);
        w.tiles[w.index(10, 10)].flags |= FLAG_INACTIVE;
        assert!(is_pressure_plate(w.tile(10, 10)), "active() ignores actuation");
        // An inactive tile refuses; a non-314 kind refuses; null refuses.
        w.tiles[w.index(10, 10)].flags &= !FLAG_ACTIVE;
        assert!(!is_pressure_plate(w.tile(10, 10)));
        w.tiles[w.index(10, 10)].flags |= FLAG_ACTIVE;
        w.tiles[w.index(10, 10)].kind = 135;
        assert!(!is_pressure_plate(w.tile(10, 10)), "the 135 plate is not 314");
        assert!(!is_pressure_plate(None));
    }

    #[test]
    fn flip_switch_track_booster_reframes_to_the_next_option() {
        // `Minecart.cs:1314-1317`: a booster (type 2) re-frames through
        // `FrameTrack(pound, mute)`. A straight booster (frame 30) flanked by
        // straight tracks left/right (mask 18) fits the through options
        // {1, 30, 31} among others; the two boosters 30/31 share connections,
        // so the back-index search (`:1075-1090`) finds no different-connection
        // candidate and the front wraps (`:1091-1108`) to the next same-type
        // through option — frame 31. The back stays −1.
        let mut w = track_world();
        place_track(&mut w, 10, 10, 30, -1);
        place_track(&mut w, 9, 10, 1, -1);
        place_track(&mut w, 11, 10, 1, -1);
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 31, "30 → 31, the next booster");
        assert_eq!(w.tile(10, 10).unwrap().frame_y, -1);
        // A second flip wraps 31 → 30 (the only other same-type through option).
        flip_switch_track(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 30);
    }

    /// Research-71 §3b.4 — the hammer pound (`Player.cs:46058`,
    /// `FrameTrack(pound: true)` mute FALSE): a straight end with ONE left
    /// neighbour (mask 2, options {3, 15, 23, 25}) pounds into the first
    /// same-type dead end — frame 3 — and reports the fail-kill condition
    /// (`Minecart.cs:1163-1166`); the bumper then cycles 3 → 15 → 25 → 3
    /// (the front-index wrap, `:1091-1108`). The wake's non-pound refit
    /// (`WorldGen.cs:85292`) keeps each dead end (no through option fits
    /// mask 2 — `num7 == -1`, `:1024-1038`).
    #[test]
    fn dw_e20_the_hammer_pound_cycles_the_dead_end_bumpers() {
        let mut w = track_world();
        place_track(&mut w, 10, 10, 1, -1); // the straight front
        place_track(&mut w, 9, 10, 1, -1); // the left neighbour (mask 2)
        assert!(frame_track_hammer(&mut w, 10, 10), "1 → 3 changes the front");
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 3);
        frame_track_place(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 3, "the refit keeps the dead end");
        for want in [15i16, 25, 3] {
            assert!(frame_track_hammer(&mut w, 10, 10));
            assert_eq!(w.tile(10, 10).unwrap().frame_x, want);
            frame_track_place(&mut w, 10, 10);
            assert_eq!(w.tile(10, 10).unwrap().frame_x, want);
        }
        assert_eq!(w.tile(10, 10).unwrap().frame_y, -1, "no back through the cycle");
    }

    /// Research-71 §3b.4 — the placement refit (`FrameTrack(pound: false)`,
    /// `Minecart.cs:976-1038`): a dead-end front BETWEEN two rails (mask 18)
    /// prefers the through option (`num7`, `:1017-1023`) — bumper 3 refits to
    /// the straight 1; the pound on that straight wraps the front to ITSELF
    /// (the only same-type through option, `:1091-1108`) — `flag4` false, NO
    /// fail-kill, the frame unchanged.
    #[test]
    fn dw_e20_the_refit_prefers_through_and_the_lone_straight_refuses() {
        let mut w = track_world();
        place_track(&mut w, 10, 10, 3, -1);
        place_track(&mut w, 9, 10, 1, -1);
        place_track(&mut w, 11, 10, 1, -1);
        frame_track_place(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 1, "the through option wins");
        assert!(!frame_track_hammer(&mut w, 10, 10), "the front wraps to itself — no change");
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 1, "nothing changed — no fail-kill");
        // The booster between the same rails DOES cycle (two type-2 through
        // options 30/31) and reports the fail-kill; the refit keeps 31.
        place_track(&mut w, 10, 10, 30, -1);
        assert!(frame_track_hammer(&mut w, 10, 10));
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 31);
        frame_track_place(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 31);
        // A pressure front (21) wraps to itself too — the only type-1 through
        // option; no fail-kill.
        place_track(&mut w, 10, 10, 21, -1);
        assert!(!frame_track_hammer(&mut w, 10, 10));
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 21);
    }

    /// Research-71 §3b.4 — the option table is TOTAL: every one of the 64
    /// masks has a fitting frame (each single-bit mask matches a dead-end
    /// ramp), so the C# null-array reset (`Minecart.cs:966-975`) is
    /// unreachable defensive code — ported verbatim, pinned here.
    #[test]
    fn dw_e20_the_switch_options_table_is_total_over_all_64_masks() {
        for mask in 0..64u32 {
            assert!(track_switch_options(mask).is_some(), "mask {mask} has options");
        }
    }

    /// Research-71 §6 census — `Minecart.GetTrackItem`
    /// (`Minecart.cs:1508-1517`): the break drop follows the FRONT frame's
    /// type — the straight/bumper family (0/1/3/15/25) → 2340, the pressure
    /// frames (20-23) → 2492, the boosters (30-35) → 2739; an unframed −1
    /// front drops nothing (the port's guarded read; the C# would throw).
    #[test]
    fn dw_e20_get_track_item_maps_the_front_type_to_the_three_items() {
        for f in [0i16, 1, 3, 15, 25] {
            assert_eq!(track_item(f), 2340, "frame {f}");
        }
        for f in 20i16..=23 {
            assert_eq!(track_item(f), 2492, "frame {f}");
        }
        for f in 30i16..=35 {
            assert_eq!(track_item(f), 2739, "frame {f}");
        }
        assert_eq!(track_item(-1), 0);
    }

    /// Research-71 §6 census — the placement wake
    /// (`WorldGen.cs:60261` → `:85290-85294`): `refit_tracks_around` refits
    /// the placed −1 front AND its neighbours. A fresh piece between two
    /// straights takes the through frame 1; the left neighbour's open-end
    /// front (15, right −1) re-fits to the straight 1 against the new
    /// right-side rail (the through preference, `Minecart.cs:1017-1023`).
    /// The outer supports (8/12) sit OUTSIDE the 3×3 walk and keep their
    /// frames — they give the inner neighbours their two-sided masks (18).
    #[test]
    fn dw_e20_the_placement_wake_refits_the_new_piece_and_its_neighbours() {
        let mut w = track_world();
        place_track(&mut w, 8, 10, 1, -1); // outer support, outside the walk
        place_track(&mut w, 9, 10, 15, -1); // open end, right dead
        place_track(&mut w, 11, 10, 1, -1);
        place_track(&mut w, 12, 10, 1, -1); // outer support, outside the walk
        // The placed piece: PlaceTrack's style-0 write (front −1, back −1).
        place_track(&mut w, 10, 10, -1, -1);
        refit_tracks_around(&mut w, 10, 10);
        assert_eq!(w.tile(10, 10).unwrap().frame_x, 1, "the −1 front took the straight");
        assert_eq!(w.tile(9, 10).unwrap().frame_x, 1, "the left neighbour re-fit through");
        assert_eq!(w.tile(11, 10).unwrap().frame_x, 1, "the right straight keeps");
        assert_eq!(w.tile(8, 10).unwrap().frame_x, 1, "outside the walk — untouched");
        assert_eq!(w.tile(12, 10).unwrap().frame_x, 1, "outside the walk — untouched");
        // A lone placed piece (no neighbours) refits to the isolated frame 0.
        let mut w = track_world();
        place_track(&mut w, 20, 20, -1, -1);
        refit_tracks_around(&mut w, 20, 20);
        assert_eq!(w.tile(20, 20).unwrap().frame_x, 0);
        assert_eq!(w.tile(20, 20).unwrap().frame_y, -1);
    }

    /// W4-f11 port (research-71 §3c), re-pointed at the kept `TRACK_FRAMES`
    /// superset: the connection tables' popcount sanity — frame 0 is the
    /// isolated rail (both sides −1), frame 1 the level straight (both 1);
    /// the pressure frames start at 20, the boosters at 30 (30 boosts right,
    /// 31 boosts left). Also pins the OnceLock cache against the direct fit
    /// rule on the boundary masks (0 isolated, 18 two straight neighbours).
    #[test]
    fn w4_f11_the_track_tables_transcribe_the_isolated_and_straight_frames() {
        assert_eq!(TRACK_FRAMES[0].left_connection, -1);
        assert_eq!(TRACK_FRAMES[0].right_connection, -1);
        assert_eq!(TRACK_FRAMES[0].track_type, 0);
        assert_eq!(TRACK_FRAMES[1].left_connection, 1);
        assert_eq!(TRACK_FRAMES[1].right_connection, 1);
        // The pressure frames 20..23 are type 1; the boosters 30..35 type 2.
        assert_eq!(TRACK_FRAMES[20].track_type, 1);
        assert_eq!(FIRST_PRESSURE_FRAME, 20);
        assert_eq!(TRACK_FRAMES[30].track_type, 2);
        assert!(!TRACK_FRAMES[30].boost_left);
        assert!(TRACK_FRAMES[31].boost_left);
        // Mask 0 (no neighbours, popcount 0) needs an EXACT frame-mask match:
        // only the isolated frame 0 (mask 0) fits.
        assert_eq!(track_switch_options(0), Some(&[0i32][..]));
        // Mask 18 (bits 2+16, popcount 2): the frame's mask must be a
        // non-zero subset — frame 1 (2|16 = 18) fits; frame 0 (mask 0) never.
        let opts = track_switch_options(18).expect("the straight fits");
        assert!(opts.contains(&1) && !opts.contains(&0));
    }

    /// The f11 lookup-mask pin (research/71 §3c:
    /// `w4_f11_the_lookup_index_masks_the_six_track_neighbours`, re-pointed
    /// at the kept e20 `nearby_track_mask`; carried from the c42 union tree
    /// at the batch-H merge — the witness union keeps c42's pins).
    /// `Minecart.cs:1170-1198`: the six-neighbour mask, `kind == 314` with
    /// NO activity gate. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f11_the_lookup_index_masks_the_six_track_neighbours() {
        let mut w = track_world();
        place_track(&mut w, 10, 20, 1, -1);
        assert_eq!(nearby_track_mask(&w, 10, 20), 0);
        place_track(&mut w, 9, 20, 1, -1); // (i−1, j) → bit 2
        place_track(&mut w, 11, 20, 1, -1); // (i+1, j) → bit 16
        assert_eq!(nearby_track_mask(&w, 10, 20), 2 + 16);
    }

    /// A player position whose bottom-center magnet sits at `magnet`:
    /// `magnet = position + (width/2, height/2 + 26)` (`Minecart.cs:570-571`).
    fn cart_at(magnet: Vec2) -> Vec2 {
        Vec2::new(
            magnet.x - CART_W as f32 / 2.0,
            magnet.y - CART_H as f32 / 2.0 - 26.0,
        )
    }

    /// A position riding the straight rail of `tile_y` at magnet X `mx`.
    fn cart_on_rail(mx: f32, tile_y: i32) -> Vec2 {
        cart_at(Vec2::new(mx, tile_y as f32 * 16.0 + STRAIGHT_RAIL_PX))
    }

    /// One ride tick in the Phase-2 call order: `TrackCollision` first (it
    /// snaps Y and rewrites the velocity, `Player.cs:28093-28161`), then the
    /// caller integrates `position += velocity` (the X move happens here,
    /// `Player.cs:23589` — pitfall §10.3).
    fn ride_tick(
        w: &World,
        position: &mut Vec2,
        velocity: &mut Vec2,
        last_boost: &mut (i32, i32),
    ) -> TrackFlags {
        let flags = track_collision(
            w, position, velocity, last_boost, CART_W, CART_H, false, false, 0, false,
        );
        *position += *velocity;
        flags
    }

    fn magnet_of(position: Vec2) -> Vec2 {
        magnet_position(position, CART_W, CART_H)
    }

    // ------------------------------------------------------------------
    // DW-1.1 — the decode tables
    // ------------------------------------------------------------------

    /// DW-1.1 — the decoded tables against hand-derived C# values
    /// (`Minecart.cs:124-428` + the epilogue transform `:437-447`).
    #[test]
    fn frame_decode_matches_the_csharp_tables() {
        // Straight (frame 1, `Minecart.cs:131-133`).
        let f = &TRACK_FRAMES[1];
        assert_eq!((f.left_connection, f.right_connection), (1, 1));
        assert_eq!(f.tile_height, [6; 8]);
        assert_eq!(f.track_type, 0);
        assert!(!f.boost_left);
        // Corners (frames 4-7, `Minecart.cs:151-185`).
        let f = &TRACK_FRAMES[4];
        assert_eq!((f.left_connection, f.right_connection), (2, 1));
        assert_eq!(f.tile_height, [14, 12, 10, 10, 8, 8, 6, 6]);
        assert_eq!(TRACK_FRAMES[5].tile_height, [6, 6, 8, 8, 10, 10, 12, 14]);
        assert_eq!(
            (
                TRACK_FRAMES[6].left_connection,
                TRACK_FRAMES[6].right_connection
            ),
            (1, 0)
        );
        assert_eq!(TRACK_FRAMES[6].tile_height, [6, 6, 6, 6, 4, 4, 2, 0]);
        assert_eq!(TRACK_FRAMES[7].tile_height, [0, 2, 4, 4, 6, 6, 6, 6]);
        // Diagonals (frames 8-9, `Minecart.cs:187-201`): levels 8..1 → 0..14.
        assert_eq!(
            (
                TRACK_FRAMES[8].left_connection,
                TRACK_FRAMES[8].right_connection
            ),
            (0, 2)
        );
        assert_eq!(TRACK_FRAMES[8].tile_height, [0, 2, 4, 6, 8, 10, 12, 14]);
        assert_eq!(TRACK_FRAMES[9].tile_height, [14, 12, 10, 8, 6, 4, 2, 0]);
        // Bumper ends (frames 2/3, `Minecart.cs:135-149`).
        assert_eq!(TRACK_FRAMES[2].tile_height, [-1, -1, -1, -1, 6, 6, 6, 6]);
        assert_eq!(TRACK_FRAMES[3].tile_height, [6, 6, 6, 6, -1, -1, -1, -1]);
        // Ramp ends (frames 16-19, `Minecart.cs:255-293`): the −3 sentinels.
        assert_eq!(TRACK_FRAMES[16].tile_height, [14, 12, 10, 8, 6, 4, -3, -3]);
        assert_eq!(TRACK_FRAMES[17].tile_height, [-3, -3, 4, 6, 8, 10, 12, 14]);
        assert_eq!(TRACK_FRAMES[18].tile_height, [0, 2, 4, 6, 8, 10, -3, -3]);
        assert_eq!(TRACK_FRAMES[19].tile_height, [-3, -3, 10, 8, 6, 4, 2, 0]);
        // Bouncy bumpers (frames 24-29, `Minecart.cs:319-375`).
        assert_eq!(TRACK_FRAMES[24].tile_height, [-2, -2, -2, -2, -2, -2, 6, 6]);
        assert_eq!(TRACK_FRAMES[25].tile_height, [6, 6, -2, -2, -2, -2, -2, -2]);
        assert_eq!(
            TRACK_FRAMES[26].tile_height,
            [14, 12, -2, -2, -2, -2, -2, -2]
        );
        assert_eq!(
            TRACK_FRAMES[27].tile_height,
            [-2, -2, -2, -2, -2, -2, 12, 14]
        );
        assert_eq!(TRACK_FRAMES[28].tile_height, [0, 2, 4, -2, -2, -2, -2, -2]);
        assert_eq!(TRACK_FRAMES[29].tile_height, [-2, -2, -2, -2, -2, 4, 2, 0]);
        // Pressure (frames 20-23, `Minecart.cs:295-317`): type 1 + open ends.
        for f in &TRACK_FRAMES[20..24] {
            assert_eq!(f.track_type, 1);
        }
        assert_eq!(TRACK_FRAMES[20].tile_height, [-4, 6, 6, 6, 6, 6, 6, -4]);
        assert_eq!(TRACK_FRAMES[22].tile_height, [-4, 6, 6, 6, 6, 6, 6, 6]);
        assert_eq!(TRACK_FRAMES[23].tile_height, [6, 6, 6, 6, 6, 6, 6, -4]);
        // Boosters (frames 30-35, `Minecart.cs:377-427`): type 2, boost sides
        // 31/33/34 left and 30/32/35 right (`Minecart.cs:380-426`).
        for (i, left) in [
            (30, false),
            (31, true),
            (32, false),
            (33, true),
            (34, true),
            (35, false),
        ] {
            assert_eq!(TRACK_FRAMES[i].track_type, 2, "frame {i}");
            assert_eq!(TRACK_FRAMES[i].boost_left, left, "frame {i}");
        }
        assert_eq!(TRACK_FRAMES[30].tile_height, [6; 8]);
        assert_eq!(TRACK_FRAMES[33].tile_height, [14, 12, 10, 8, 6, 4, 2, 0]);
        // Open ends (frames 0/14/15, `Minecart.cs:125-129`, `:245-253`).
        assert_eq!(TRACK_FRAMES[0].tile_height, [-4, 6, 6, 6, 6, 6, 6, -4]);
        assert_eq!(TRACK_FRAMES[14].tile_height, [-4, 6, 6, 6, 6, 6, 6, 6]);
        assert_eq!(TRACK_FRAMES[15].tile_height, [6, 6, 6, 6, 6, 6, 6, -4]);
        // The first-frame markers (`Minecart.cs:519-546`).
        assert_eq!(
            (
                FIRST_PRESSURE_FRAME,
                FIRST_RIGHT_BOOST_FRAME,
                FIRST_LEFT_BOOST_FRAME
            ),
            (20, 30, 31)
        );
        // Texture cells ×18 (`Minecart.cs:429-436`).
        assert_eq!(TEXTURE_POSITION[0], (0.0, 0.0));
        assert_eq!(TEXTURE_POSITION[1], (18.0, 0.0));
        assert_eq!(TEXTURE_POSITION[21], (18.0, 72.0));
        assert_eq!(TEXTURE_POSITION[36], (0.0, 108.0));
        assert_eq!(TEXTURE_POSITION[38], (18.0, 126.0));
        assert_eq!(TEXTURE_POSITION[39], (0.0, 126.0));
    }

    /// The assumption check: with the default `MinecartSettings`
    /// (`Minecart.cs:29-34`) the magnet of the mounted 20×52 body is the exact
    /// bottom-center (`Minecart.cs:570-571`).
    #[test]
    fn magnet_point_is_the_bottom_center_for_the_defaults() {
        let position = Vec2::new(100.0, 200.0);
        let magnet = magnet_position(position, CART_W, CART_H);
        assert_eq!(magnet.x, 110.0); // position.x + width/2
        assert_eq!(magnet.y, 252.0); // position.y + height — the bottom edge
    }

    // ------------------------------------------------------------------
    // DW-1.2 — track_collision flags + pitfalls §10.3/§10.10
    // ------------------------------------------------------------------

    /// DW-1.2, pitfall §10.3 — the snap touches only Y; X stays for the later
    /// integration (`Minecart.cs:934-935` moves Y, nothing else moves X).
    #[test]
    fn x_is_never_moved_by_track_collision() {
        let mut w = track_world();
        for x in 30..40 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let mut position = cart_on_rail(500.0, 20);
        let mut velocity = Vec2::new(4.0, 0.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(flags.on_track());
        assert_eq!(position.x, cart_on_rail(500.0, 20).x);
        // The Y snap is a no-op when the magnet already sits on the rail.
        assert_eq!(position.y, cart_on_rail(500.0, 20).y);
        assert_eq!(velocity, Vec2::new(4.0, 0.0));
    }

    /// DW-1.2 — the boost bits: a left booster sets bit 4 and marks
    /// `last_boost` with the tile (`Minecart.cs:832-847`).
    #[test]
    fn boost_left_and_right_flags_fire_on_booster_tiles() {
        let mut w = track_world();
        for x in 44..48 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 45, 20, 30, -1); // the right-boosting straight
        let mut position = cart_on_rail((44 * 16) as f32 + 2.0, 20);
        let mut velocity = Vec2::new(2.0, 0.0);
        let mut last_boost = (0, 0);
        let mut flags = TrackFlags::default();
        // Ride until the booster tile is passed.
        for _ in 0..20 {
            flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            if flags.boost_right() {
                break;
            }
        }
        assert!(flags.boost_right(), "the right booster fired");
        assert!(!flags.boost_left());
        assert_eq!(last_boost, (45, 20));
        // Ride off the tile — the pending boost clears when the march ends on
        // a different tile (`Minecart.cs:894-897`). The booster spans
        // x 720..735, so the 2 px/tick ride needs several ticks to leave it.
        for _ in 0..10 {
            ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
        }
        assert_eq!(last_boost, (0, 0));
    }

    /// DW-1.2 — riding over a pressure track trips the switch flag once per
    /// pass (`Minecart.cs:826-829` on the tile change; `:814-817` covers the
    /// from-above case).
    #[test]
    fn riding_over_pressure_trips_the_switch_flag() {
        let mut w = track_world();
        for x in 34..38 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 35, 20, 21, -1); // the straight pressure track
        let mut position = cart_on_rail((34 * 16) as f32 + 2.0, 20);
        let mut velocity = Vec2::new(2.0, 0.0);
        let mut last_boost = (0, 0);
        let mut entered = false;
        let mut after = false;
        for _ in 0..10 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            if flags.hit_switch() {
                entered = true;
                assert!(flags.on_track(), "the trip happens while riding");
            } else if entered {
                after = true;
            }
        }
        assert!(entered, "the pressure tile tripped the switch");
        assert!(after, "the flag fires once per pass, not every tick");
    }

    /// DW-1.2 — the exactly-at-threshold boundary: a magnet exactly on the
    /// rail line is OnTrack (→ OnFront for a single rail); one pixel above is
    /// AboveTrack (→ AboveFront → off the rail) (`Minecart.cs:710`, `:716-739`).
    #[test]
    fn magnet_exactly_on_the_rail_is_on_track_one_pixel_off_is_not() {
        let mut w = track_world();
        for x in 30..40 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let rail_y = 20.0 * 16.0 + STRAIGHT_RAIL_PX;
        // Exactly on the rail.
        let mut position = cart_at(Vec2::new(500.0, rail_y));
        let mut velocity = Vec2::new(0.0, 0.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(flags.on_track());
        // One pixel above the rail — above, not on.
        let mut position = cart_at(Vec2::new(500.0, rail_y - 1.0));
        let mut velocity = Vec2::new(0.0, 0.0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(!flags.on_track(), "1 px above the rail is not riding");
    }

    // ------------------------------------------------------------------
    // DW-1.3 — the helpers
    // ------------------------------------------------------------------

    /// DW-1.3 — `OnTrack` checks only the tile type under the magnet
    /// (`Minecart.cs:1243-1253`): no `nactive()`, so an actuated-off track
    /// still counts; dirt or nothing does not.
    #[test]
    fn on_track_checks_the_tile_under_the_magnet() {
        let mut w = track_world();
        place_track(&mut w, 30, 20, 1, 1);
        let on_it = cart_on_rail((30 * 16) as f32 + 8.0, 20);
        assert!(on_track(&w, on_it, CART_W, CART_H));
        // Nothing there.
        let off_it = cart_on_rail((40 * 16) as f32 + 8.0, 20);
        assert!(!on_track(&w, off_it, CART_W, CART_H));
        // Dirt under the magnet.
        let i = w.index(41, 20);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!on_track(
            &w,
            cart_on_rail((41 * 16) as f32 + 8.0, 20),
            CART_W,
            CART_H
        ));
        // An inactive (actuated-off) track still counts — the C# reads only
        // `tile.type` (`Minecart.cs:1248-1252`).
        place_track(&mut w, 42, 20, 1, 1);
        let i = w.index(42, 20);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(on_track(
            &w,
            cart_on_rail((42 * 16) as f32 + 8.0, 20),
            CART_W,
            CART_H
        ));
    }

    /// DW-1.3 — `GetOnTrack` snaps the bottom-center magnet onto the nearest
    /// valid front-rail sub-column, forward first then backward
    /// (`Minecart.cs:1200-1241`).
    #[test]
    fn get_on_track_snaps_onto_the_nearest_valid_sub_column() {
        let mut w = track_world();
        // Frame 14: sub 0 is the open end, subs 1-7 ride at 6 px.
        place_track(&mut w, 60, 25, 14, -1);
        // The magnet mid-tile (sub 5): snaps onto its own sub-column.
        let mut position = cart_at(Vec2::new(60.0 * 16.0 + 10.0, 25.0 * 16.0 + 6.0));
        assert!(get_on_track(&w, 60, 25, &mut position, CART_W, CART_H));
        assert_eq!(magnet_of(position).x, 60.0 * 16.0 + 10.0);
        assert_eq!(magnet_of(position).y, 25.0 * 16.0 + 6.0);
        // The magnet on the open-end sub (0): scans forward to sub 1.
        let mut position = cart_at(Vec2::new(60.0 * 16.0 + 0.0, 25.0 * 16.0 + 6.0));
        assert!(get_on_track(&w, 60, 25, &mut position, CART_W, CART_H));
        assert_eq!(magnet_of(position).x, 60.0 * 16.0 + 2.0);
        // Frame 3 rides only subs 0-3: a magnet at sub 6 finds nothing forward
        // and scans backward to sub 3.
        place_track(&mut w, 61, 25, 3, -1);
        let mut position = cart_at(Vec2::new(61.0 * 16.0 + 13.0, 25.0 * 16.0 + 6.0));
        assert!(get_on_track(&w, 61, 25, &mut position, CART_W, CART_H));
        assert_eq!(magnet_of(position).x, 61.0 * 16.0 + 6.0);
        assert_eq!(magnet_of(position).y, 25.0 * 16.0 + 6.0);
        // Not a track, or nothing at all: false.
        let mut position = cart_at(Vec2::new(62.0 * 16.0 + 8.0, 25.0 * 16.0 + 6.0));
        assert!(!get_on_track(&w, 62, 25, &mut position, CART_W, CART_H));
        assert!(!get_on_track(&w, 63, 25, &mut position, CART_W, CART_H));
    }

    /// DW-1.3 — `TrackRotation` measures the rail slope through the two
    /// trackOnly wheel probes (`Minecart.cs:1255-1279`): flat rails give
    /// rotation 0 and no correction; a rising corner gives a negative
    /// (upward-slope) angle and a matching gfx offset.
    #[test]
    fn track_rotation_measures_the_rail_slope() {
        let mut w = track_world();
        // Flat: 48..51 all straight.
        for x in 48..52 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let flat = cart_on_rail(808.0, 20);
        assert_eq!(
            track_rotation(&w, flat, CART_W, CART_H, false, false),
            (0.0, 0.0)
        );
        // A climb: flat into the frame-6 corner (straight left, exits up) at
        // (50,20), continued by the frame-4 piece (enters from below, exits
        // straight) diagonally up-right at (51,19) — the corner step
        // (`Minecart.cs:605-608`) moves the march one row up at that boundary.
        // The player magnet rides mid-climb at (808, 324): the right probe
        // climbs to 314 px, the left one drops back to the flat 6 px rail
        // (326 px).
        place_track(&mut w, 50, 20, 6, -1);
        place_track(&mut w, 51, 19, 4, -1);
        place_track(&mut w, 52, 19, 1, 1);
        let rising = cart_at(Vec2::new(808.0, 324.0));
        let (gfx_off_y, rotation) = track_rotation(&w, rising, CART_W, CART_H, false, false);
        // The left probe rests 12 px lower than the right one (274 vs 262):
        // atan2(−12, 24); the slope line through them sits 4 px above the
        // player position.
        assert!(
            (rotation - (-12.0f32).atan2(24.0)).abs() < 1e-5,
            "{rotation}"
        );
        assert!((gfx_off_y - (-4.0)).abs() < 1e-4, "{gfx_off_y}");
    }

    // ------------------------------------------------------------------
    // DW-1.4 — the scenarios
    // ------------------------------------------------------------------

    /// DW-1.4 — a straight run holds the rail: the magnet rides the 6 px line,
    /// X is the only motion and `velocity.Y` stays zero (`Minecart.cs:818`,
    /// `:929-932`).
    #[test]
    fn straight_run_holds_the_rail() {
        let mut w = track_world();
        for x in 30..60 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let mut position = cart_on_rail(30.0 * 16.0 + 2.0, 20);
        let mut velocity = Vec2::new(4.0, 0.0);
        let mut last_boost = (0, 0);
        for _ in 0..25 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            assert!(flags.on_track(), "still riding");
            assert_eq!(velocity.y, 0.0);
        }
        // The magnet never left the rail line.
        assert_eq!(magnet_of(position).y, 20.0 * 16.0 + STRAIGHT_RAIL_PX);
    }

    /// DW-1.4 — a corner switches the axis of travel: straight right into a
    /// frame-5 corner (right=2) then frame-8 diagonals — the march stays
    /// riding while the row steps down one tile per column
    /// (`Minecart.cs:600-614` corner step + `:818-819` snap).
    #[test]
    fn corner_switches_the_axis_of_travel() {
        let mut w = track_world();
        for x in 30..33 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 33, 20, 5, -1); // straight left, down right
        place_track(&mut w, 34, 21, 8, -1); // diagonal down-right
        place_track(&mut w, 35, 22, 8, -1);
        place_track(&mut w, 36, 23, 8, -1);
        let mut position = cart_on_rail(30.0 * 16.0 + 2.0, 20);
        let mut velocity = Vec2::new(4.0, 0.0);
        let mut last_boost = (0, 0);
        for _ in 0..20 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            assert!(flags.on_track(), "riding through the corner");
            assert_eq!(velocity.y, 0.0, "the snap keeps velocity.Y level");
            assert_eq!(velocity.x, 4.0, "X is the ride speed");
        }
        // Two rows down the diagonal, on its rail line.
        let magnet = magnet_of(position);
        assert_eq!(magnet.y, 22.0 * 16.0 + 2.0);
        assert_eq!((magnet.x / 16.0) as i32, 35);
    }

    /// DW-1.4 — a ramp end (−3) launches at 45°: the C# rotates `(Velocity.X,
    /// 0)` by ∓π/4 and then overrides X back to `Velocity.X`
    /// (`Minecart.cs:692-701`) — the launch is `(vx, ±vx·√2/2)`, bit 2 set,
    /// bit 0 clear.
    #[test]
    fn ramp_end_launches_at_45_degrees() {
        let mut w = track_world();
        place_track(&mut w, 40, 25, 16, -1); // rising ramp, ramp end on top
        // The magnet sits on the ramp's sub 4 (6 px rail), moving right; the
        // 4 px march reaches the −3 sentinel at sub 6.
        let start = cart_at(Vec2::new(40.0 * 16.0 + 8.0, 25.0 * 16.0 + 6.0));
        let mut position = start;
        let mut velocity = Vec2::new(4.0, 0.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(flags.used_ramp(), "bit 2");
        assert!(!flags.on_track(), "the ramp released the cart");
        assert_eq!(velocity.x, 4.0, "X keeps the ride speed");
        assert!(
            (velocity.y - (4.0 * -std::f32::consts::FRAC_PI_4.sin())).abs() < 1e-6,
            "the 45° launch: {}",
            velocity.y
        );
        // The Y snap compensates the upcoming launch (`Minecart.cs:934-935`):
        // position.Y += marchΔ − velocity.Y = −2 + 2.828… → +0.83.
        assert!((position.y - start.y - 0.83).abs() < 1e-4);
    }

    /// DW-1.4 — a regular bumper (−1) dead-stops the cart on the rail: the
    /// march rewinds one step, the residual `velocity.X` is the travelled
    /// distance (`Minecart.cs:660-668`, `:925-928`) and the cart stays
    /// `on_track` at the stop.
    #[test]
    fn bumper_ends_the_run_on_the_rail() {
        let mut w = track_world();
        for x in 30..34 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 34, 20, 3, -1); // straight left, bumper right
        let mut position = cart_on_rail(30.0 * 16.0 + 2.0, 20);
        let mut velocity = Vec2::new(6.0, 0.0);
        let mut last_boost = (0, 0);
        let mut max_magnet_x = 0.0f32;
        for _ in 0..15 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            assert!(flags.on_track(), "the stop happens on the rail");
            assert!(!flags.bouncy_bumper(), "a regular bumper never flips");
            max_magnet_x = max_magnet_x.max(magnet_of(position).x);
        }
        // The cart stops short of the bumper sub-column (x 552) and sits still.
        assert!(max_magnet_x <= 552.0, "{max_magnet_x}");
        assert_eq!(velocity, Vec2::new(0.0, 0.0));
    }

    /// DW-1.4 — a bouncy bumper (−2) rewinds the magnet to the sub-column's
    /// right edge and flips the velocity: bit 1 set, `velocity.X` negated and
    /// `position.X` moved (the §10.3 exception, `Minecart.cs:669-691`,
    /// `:914-922`).
    #[test]
    fn bouncy_bumper_rewinds_and_flips_vx() {
        let mut w = track_world();
        for x in 30..33 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 33, 20, 24, -1); // bouncy on the left half
        place_track(&mut w, 34, 20, 1, 1);
        // Ride left from tile 34 into the bouncy half of tile 33.
        let mut position = cart_on_rail(556.0, 20);
        let mut velocity = Vec2::new(-6.0, 0.0);
        let mut last_boost = (0, 0);
        let mut bounced = false;
        for _ in 0..5 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            if flags.bouncy_bumper() {
                bounced = true;
                assert!(flags.on_track(), "still riding the bouncy tile");
                assert_eq!(velocity.x, 6.0, "the velocity flipped");
                // The bounce rewinds the magnet to the sub-column's right edge
                // (offset 12 of tile 33 = x 540) and then marches on with the
                // leftover 1.75 px of speed, ending at x 541.75; the
                // pre-compensated position integrates back onto it
                // (`Minecart.cs:682-690`, `:917`).
                assert_eq!(magnet_of(position).x, 541.75);
                assert_eq!(velocity.y, 0.0);
            }
        }
        assert!(bounced, "the bouncy bumper flipped the cart");
    }

    /// DW-1.4 — an open end (−4) releases the cart into a fall: bit 0 clear,
    /// `velocity.X` becomes the travelled distance and `velocity.Y` the
    /// default gravity (`Minecart.cs:643-658`, `:904-908`).
    #[test]
    fn open_end_releases_into_a_fall() {
        let mut w = track_world();
        place_track(&mut w, 33, 20, 14, -1); // open end on the left
        for x in 34..40 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let mut position = cart_on_rail(556.0, 20);
        let mut velocity = Vec2::new(-6.0, 0.0);
        let mut last_boost = (0, 0);
        let mut released = false;
        for _ in 0..6 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            if !flags.on_track() {
                released = true;
                assert!(!flags.used_ramp());
                assert_eq!(velocity.x, -6.0, "the travelled X");
                assert_eq!(velocity.y, DEFAULT_GRAVITY, "the fall speed");
                // The Y snap pre-compensated the new fall speed
                // (`Minecart.cs:934-935`).
                assert!((velocity.y - 0.4).abs() < 1e-6);
                break;
            }
        }
        assert!(released, "the open end let go of the cart");
        // The next tick (gravity accumulated) is off-track and a no-op.
        velocity.y += DEFAULT_GRAVITY;
        let before = position;
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(!flags.on_track());
        assert_eq!(position, before);
        assert_eq!(velocity, Vec2::new(-6.0, 0.8));
    }

    /// DW-1.4 — the boost fires once per booster tile: passing marks
    /// `last_boost`, lingering on the tile never re-boosts, and leaving clears
    /// the mark (`Minecart.cs:832-847`, `:894-897`).
    #[test]
    fn boost_fires_once_per_booster_tile() {
        let mut w = track_world();
        for x in 30..35 {
            place_track(&mut w, x, 20, 1, 1);
        }
        place_track(&mut w, 35, 20, 31, -1); // the left-boosting straight
        for x in 36..40 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let mut position = cart_on_rail(30.0 * 16.0 + 2.0, 20);
        let mut velocity = Vec2::new(2.0, 0.0);
        let mut last_boost = (0, 0);
        let mut boost_ticks = 0;
        let mut seen_pending = false;
        let mut seen_cleared = false;
        for _ in 0..50 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            if flags.boost_left() || flags.boost_right() {
                boost_ticks += 1;
                assert_eq!(last_boost, (35, 20), "marked with the booster tile");
            } else if last_boost == (35, 20) {
                seen_pending = true; // lingering on the tile, no re-boost
            } else if seen_pending {
                seen_cleared = true; // past the tile, the mark cleared
            }
        }
        assert_eq!(boost_ticks, 1, "exactly one boost per pass");
        assert!(seen_pending, "the mark survives while on the tile");
        assert!(seen_cleared, "the mark clears after leaving the tile");
    }

    /// DW-1.4 dirty — a track frame id outside 0..35 panics on the table
    /// index exactly like the C#'s unchecked `_tileHeight[frameX]`
    /// (`Minecart.cs:640`).
    #[test]
    #[should_panic]
    fn out_of_range_frame_index_panics_like_csharp() {
        let mut w = track_world();
        place_track(&mut w, 30, 20, 99, -1);
        let mut position = cart_on_rail(30.0 * 16.0 + 4.0, 20);
        let mut velocity = Vec2::new(2.0, 0.0);
        let mut last_boost = (0, 0);
        let _ = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
    }

    /// DW-1.4 dirty — the sub-column index `(int)X % 16 / 2`
    /// (`Minecart.cs:594`) is structurally bounded to 0..7 for every
    /// reachable magnet X: riding across a tile at every 1 px offset decodes
    /// without a panic. (A negative X dies at the C#'s raw `Tile[,]` lookup
    /// first — `Main.cs:66984` — which the port mirrors as an out-of-bounds
    /// `None` → non-track.)
    #[test]
    fn sub_column_index_stays_in_bounds_across_the_tile() {
        let mut w = track_world();
        for x in 30..32 {
            place_track(&mut w, x, 20, 1, 1);
        }
        for offset in 0..16i32 {
            let mut position = cart_on_rail(30.0 * 16.0 + offset as f32, 20);
            let mut velocity = Vec2::new(2.0, 0.0);
            let mut last_boost = (0, 0);
            let flags = track_collision(
                &w,
                &mut position,
                &mut velocity,
                &mut last_boost,
                CART_W,
                CART_H,
                false,
                false,
                0,
                false,
            );
            assert!(flags.on_track(), "offset {offset} rides");
        }
    }

    /// P1 dirty — a track run interrupted mid-march (a missing tile in the
    /// row) reads as NoTrack while over the gap and re-lands on the far side
    /// without a landing event (`Minecart.cs:633` flag, `:744-831` branches).
    #[test]
    fn missing_tile_mid_march_reads_as_notrack() {
        let mut w = track_world();
        for x in 30..34 {
            place_track(&mut w, x, 20, 1, 1);
        }
        // (34, 20) left empty — the gap.
        for x in 35..39 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let mut position = cart_on_rail(30.0 * 16.0 + 2.0, 20);
        let mut velocity = Vec2::new(4.0, 0.0);
        let mut last_boost = (0, 0);
        let mut over_gap = false;
        let mut re_landed = false;
        for _ in 0..20 {
            let flags = ride_tick(&w, &mut position, &mut velocity, &mut last_boost);
            let magnet = magnet_of(position);
            if (magnet.x / 16.0) as i32 == 34 {
                over_gap = true;
                assert!(!flags.on_track(), "the gap is not a rail");
                assert!(!flags.landing, "no landing event over a gap");
                assert_eq!(velocity.y, 0.0, "the gap injects no gravity");
            } else if over_gap && (magnet.x / 16.0) as i32 >= 35 {
                re_landed = true;
                assert!(flags.on_track(), "the run resumes on the far side");
                assert!(!flags.landing);
            }
        }
        assert!(over_gap && re_landed);
    }

    // ------------------------------------------------------------------
    // Beyond the DW floor (Test Coverage 100%)
    // ------------------------------------------------------------------

    /// The landing flag needs a real fall: faster than gravity and the fall
    /// origin at least one row above (`Minecart.cs:809-813`).
    #[test]
    fn landing_flag_needs_falling_speed_and_a_real_fall_origin() {
        let mut w = track_world();
        for x in 30..40 {
            place_track(&mut w, x, 20, 1, 1);
        }
        let rail_y = 20.0 * 16.0 + STRAIGHT_RAIL_PX;
        // A 4 px drop at fall speed 6 (one call reaches the rail line), origin
        // far above: the magnet crosses the rail mid-tile, so the transition
        // ladder resolves AboveFront → OnFront with `flag9` unset.
        let mut position = cart_at(Vec2::new(500.0, rail_y - 4.0));
        let mut velocity = Vec2::new(0.0, 6.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(flags.landing, "the fall onto the rail fires the event");
        assert!(flags.on_track());
        assert_eq!(velocity.y, 0.0, "the snap levels the fall");
        assert!(
            (position.y - (rail_y - CART_H as f32)).abs() < 1e-4,
            "on the rail"
        );
        // The same fall with the origin on the current row: no event
        // (`(int)(position.Y/16) − 1` = 15 rejects `fallStart` 16).
        let mut position = cart_at(Vec2::new(500.0, rail_y - 4.0));
        let mut velocity = Vec2::new(0.0, 6.0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            16,
            false,
        );
        assert!(!flags.landing, "no fall origin, no landing event");
        assert!(flags.on_track());
    }

    /// The `followDown`/`followUp` ambiguity (`Minecart.cs:787-794`): riding a
    /// two-rail tile whose rails diverge — `followDown` keeps the lower rail,
    /// `followUp` the upper rail; the snap lands on the chosen rail's height.
    #[test]
    fn follow_flags_resolve_the_between_rails_ambiguity() {
        let mut w = track_world();
        // Front = the flat straight (1, 6 px everywhere), back = the down
        // diagonal (8, 0..14 px): both rails sit at 6 px at sub 3 (the magnet
        // is OnTrack on both) and diverge at sub 4 (front 6 px, back 8 px).
        place_track(&mut w, 50, 20, 1, 8);
        let start = cart_at(Vec2::new(50.0 * 16.0 + 6.0, 20.0 * 16.0 + 6.0));
        // followDown: the lower rail is the back (8 px) → OnBack → snap 328.
        let mut position = start;
        let mut velocity = Vec2::new(2.0, 0.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            true,
            false,
            0,
            false,
        );
        assert!(flags.on_track());
        assert_eq!(magnet_of(position).y, 20.0 * 16.0 + 8.0);
        // followUp: the upper rail is the front (6 px) → OnFront → snap 326.
        let mut position = start;
        let mut velocity = Vec2::new(2.0, 0.0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            true,
            0,
            false,
        );
        assert!(flags.on_track());
        assert_eq!(magnet_of(position).y, 20.0 * 16.0 + 6.0);
        // Neither: the front rail by default (`Minecart.cs:791`).
        let mut position = start;
        let mut velocity = Vec2::new(2.0, 0.0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            false,
        );
        assert!(flags.on_track());
        assert_eq!(magnet_of(position).y, 20.0 * 16.0 + 6.0);
    }

    /// The trackOnly probes (the wheel queries) stop at sentinels without the
    /// gameplay effects: no flip on a bouncy tile, no fall on an open end
    /// (`Minecart.cs:646-657`, `:674-681`).
    #[test]
    fn track_only_probes_stop_at_sentinels_without_effects() {
        let mut w = track_world();
        // The bouncy tile at 33 approached from the right, probe-only.
        place_track(&mut w, 33, 20, 24, -1);
        place_track(&mut w, 34, 20, 1, 1);
        let mut position = cart_on_rail(556.0, 20);
        let mut velocity = Vec2::new(-18.0, 0.0);
        let mut last_boost = (0, 0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            true,
        );
        assert!(flags.on_track(), "the probe rests on the tile");
        assert!(!flags.bouncy_bumper(), "no flip in a probe");
        assert_eq!(velocity.y, 0.0, "no fall in a probe");
        // The open end at 33 approached from the right, probe-only.
        place_track(&mut w, 33, 20, 14, -1);
        let mut position = cart_on_rail(556.0, 20);
        let mut velocity = Vec2::new(-18.0, 0.0);
        let flags = track_collision(
            &w,
            &mut position,
            &mut velocity,
            &mut last_boost,
            CART_W,
            CART_H,
            false,
            false,
            0,
            true,
        );
        assert!(flags.on_track(), "the probe rests on the tile");
        assert!(!flags.used_ramp());
        assert_eq!(velocity.y, 0.0, "the probe never falls");
    }

    /// The banker's Y snap (`Minecart.cs:935`): scaled midpoints round to
    /// even, like every other C# `Math.Round` port in this repo. True
    /// midpoints need an exactly-representable value ×100 ending in .5 — odd
    /// eighths — while a literal like 1.135 is stored just below the midpoint
    /// and rounds down in C# too.
    #[test]
    fn y_snap_rounds_half_to_even() {
        assert_eq!(round_y_snap(1.125), 1.12); // 112.5 → 112 (even)
        assert_eq!(round_y_snap(1.375), 1.38); // 137.5 → 138 (even up)
        assert_eq!(round_y_snap(1.625), 1.62); // 162.5 → 162 (even)
        assert_eq!(round_y_snap(-1.125), -1.12);
        assert_eq!(round_y_snap(1.135), 1.13); // below the midpoint in f32
        assert_eq!(round_y_snap(354.0), 354.0);
        assert_eq!(round_y_snap(-1.171573), -1.17);
    }

    // ------------------------------------------------------------------
    // M2-PHYS-7 Phase 3 — wheel sparks + sounds
    // ------------------------------------------------------------------

    /// Finds a seed whose first `wants.len()` splitmix rolls satisfy
    /// `roll % mods[i] == wants[i]` (the rain.rs seed-search idiom) so the
    /// spark gates roll on cue.
    fn seed_with_rolls(mods: &[u64], wants: &[u64]) -> u64 {
        for s in 1u64.. {
            let mut st = s;
            if mods
                .iter()
                .zip(wants)
                .all(|(&m, &w)| crate::app::next_rand(&mut st) % m == w)
            {
                return s;
            }
        }
        unreachable!()
    }

    /// Finds a seed where one rolling call at `|vx| = vel_x` queues exactly
    /// `want` sparks — the gate rolls interleave with the per-spark request
    /// rolls (side + the two speed args), so the wanted gate outcomes sit at
    /// strided stream positions only the real roll order knows.
    fn seed_with_spark_count(vel_x: f32, want: usize) -> u64 {
        for s in 1u64.. {
            let mut sparks = Vec::new();
            let mut rng = s;
            rolling_wheel_sparks(
                &mut Vec::new(),
                &mut sparks,
                &mut rng,
                SPARK_POS,
                20,
                52,
                Vec2::new(vel_x, 0.0),
                13.0,
                0.0,
                SparkVariant::Sparks,
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    const SPARK_POS: Vec2 = Vec2::new(100.0, 200.0);

    /// A spawn ctx that passes every guard with no cull pressure: the world
    /// view is far larger than the test positions need.
    fn spark_ctx() -> tr_core::particles::SpawnCtx {
        tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::new(-1000.0, -1000.0),
            screen_size: Vec2::new(8000.0, 8000.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The shared tick under the tests' default snapshot: no player
    /// (`DustPlayerSnap::default`), the wide-open `spark_ctx` for the child
    /// drain (the W4-d50 wiring shape; the 213 arm spawns no children).
    fn tick<R: tr_core::particles::DustRng>(
        pool: &mut tr_core::particles::DustPool,
        rng: &mut R,
        max: i32,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        world: &World,
        player_vy: f32,
        show: bool,
    ) {
        super::tick(
            pool,
            rng,
            max,
            camera,
            screen,
            zoom,
            world,
            DustPlayerSnap { vy: player_vy, ..Default::default() },
            show,
            &spark_ctx(),
        );
    }

    /// Queues one spark at `at` with rotation `rot` and spawns it through a
    /// fresh pool; returns the slot index. The request roll and the drain
    /// share one stream state, exactly like the tick → drain handoff.
    fn spawn_one(at: Vec2, rot: f32, seed: u64) -> (tr_core::particles::DustPool, usize) {
        let mut queue = Vec::new();
        let mut rng = seed;
        wheel_sparks(&mut queue, &mut rng, at, 20, 52, 1, rot, SparkVariant::Sparks);
        let mut pool = tr_core::particles::DustPool::new();
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut idx = tr_core::particles::DUST_SENTINEL;
        for s in &queue {
            idx = spawn_spark(
                &mut pool,
                &mut crate::app::SplitmixRand(&mut rng),
                &ctx,
                &world,
                *s,
            );
        }
        (pool, idx)
    }

    /// DW-3.1 (repointed to the pool, S1-E-1 phase 2) — one spark reproduces
    /// the `DelegateMethods.Minecart.Sparks` spawn math
    /// (`DelegateMethods.cs:223-242`): the emitter's request rolls (the ±13
    /// side offset, the two `Next(-2,3)` speed args) then the pool's verbatim
    /// `NewDust` block (`Dust.cs:159-191`) and the delegate arms on the same
    /// stream — the +4/−4 px box/lift pair, the upward velocity recipe, the
    /// sheet row and the scale/fadeIn bands, the 2-in-3 gravity flip, and
    /// exact stream replay.
    #[test]
    fn the_pool_spawn_reproduces_the_dust_213_math() {
        let at = Vec2::new(1000.0, 2000.0);
        let rot = 0.5f32;
        let (cos, sin) = (rot.cos(), rot.sin());
        // `wheel_sparks` applies the magnet formula first (`Minecart.cs:1625`):
        // the delegate's `dustPosition` is the magnet point.
        let mag = magnet_position(at, 20, 52);
        let mut weightless = 0;
        for seed in 1..=200u64 {
            let (pool, idx) = spawn_one(at, rot, seed);
            assert!(idx != tr_core::particles::DUST_SENTINEL, "spawned");
            let d = pool.slot(idx);
            // The position: `mag ± (13cos, 13sin) + (4, 4)` then `pos.Y -= 4`
            // — the +4/−4 cancel in Y (`:225` + `Dust.cs:159-160` + `:233`).
            // The 1×1 box clamps to 5, so both jitters roll `Next(1)` —
            // always 0 (`Dust.cs:159-160`).
            let off = d.position - mag;
            let side_x = if off.x > 4.0 { 13.0 * cos } else { -13.0 * cos };
            assert!((off.x - 4.0 - side_x).abs() < 1e-3, "side x: {}", off.x);
            assert!((off.y.abs() - 13.0 * sin).abs() < 1e-3, "side y: {}", off.y);
            // The velocity recipe (`:230-232` over the `NewDust` starts
            // (`Dust.cs:161-162` — ±2.0 each plus the ±2 speed args) scaled
            // by 0.15..0.5, then the X scale 0.25..1 and the Y lift 1.5..3):
            // vel.Y ∈ [−5, 0.5], vel.X ∈ [−2, 2].
            assert!(
                d.velocity.y >= -5.0 - 1e-3 && d.velocity.y <= 0.5 + 1e-3,
                "vel.y {}",
                d.velocity.y
            );
            assert!(d.velocity.x.abs() <= 2.0 + 1e-3, "vel.x {}", d.velocity.x);
            // The sheet row (`Dust.cs:164`) and the scale/fadeIn bands
            // (`Dust.cs:190-191`, `:228`): the pool's per-100 shift puts
            // type 213 at frame_x 130, frame_y 60 + 10*row.
            let row = (d.frame_y - 60) / 10;
            assert!((0..=2).contains(&row), "row {}", row);
            assert_eq!(d.frame_x, 130);
            assert!(d.scale >= 0.8 * 0.6 - 1e-4 && d.scale <= 1.2 + 1e-4);
            assert!((1.8..=2.7).contains(&d.fade_in), "fade {}", d.fade_in);
            // `:234-241`: 2-in-3 the spark gains gravity.
            if d.no_gravity {
                weightless += 1;
            }
            assert!(d.active, "the slot stays live");
            assert_eq!(d.dust_type, 213);
        }
        assert!(
            (40..=140).contains(&weightless),
            "the 1-in-3 weightless flip: {weightless}/200"
        );
        // The same seed replays byte-identically.
        let (a, ia) = spawn_one(at, rot, 42);
        let (b, ib) = spawn_one(at, rot, 42);
        assert_eq!(ia, ib);
        let da = a.slot(ia);
        let db = b.slot(ib);
        assert_eq!(da.position, db.position);
        assert_eq!(da.velocity, db.velocity);
        assert_eq!(da.scale, db.scale);
        assert_eq!(da.fade_in, db.fade_in);
        assert_eq!(da.frame_x, db.frame_x);
        assert_eq!(da.frame_y, db.frame_y);
        assert_eq!(da.no_gravity, db.no_gravity);
    }

    /// DW-3.1 (repointed) — `Minecart.WheelSparks` (`Minecart.cs:1623-1630`)
    /// lands every spark on the magnet point (the `:1625` formula — the same
    /// arithmetic as `GetMagnetPosition`, `:1290-1294`) and emits exactly
    /// `spark_count` requests, which the pool spawn places at `at ± 13 + 4`.
    #[test]
    fn wheel_sparks_lands_on_the_magnet_point_with_the_requested_count() {
        let mut queue = Vec::new();
        let mut rng = 7u64;
        wheel_sparks(&mut queue, &mut rng, SPARK_POS, 20, 52, 10, 0.0, SparkVariant::Sparks);
        assert_eq!(queue.len(), 10, "the landing burst's count");
        let at = magnet_position(SPARK_POS, 20, 52);
        assert_eq!(at, Vec2::new(110.0, 252.0), "the bottom-center point");
        // Rotation 0: the side offset is (±13, 0) — x ∈ {at+17, at−9} after
        // the pool's +4 box jitter (`Next(1)` = 0), y untouched before the
        // delegate's −4 lift.
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut pool = tr_core::particles::DustPool::new();
        for s in &queue {
            spawn_spark(
                &mut pool,
                &mut crate::app::SplitmixRand(&mut rng),
                &ctx,
                &world,
                *s,
            );
        }
        for i in 0..10 {
            let d = pool.slot(i);
            assert!(
                (d.position.x - 127.0).abs() < 1e-4 || (d.position.x - 101.0).abs() < 1e-4,
                "x {}",
                d.position.x
            );
            assert_eq!(d.position.y, 252.0);
        }
    }

    /// Hand-places an active type-213 slot at (`x`, 100) — the AI tests'
    /// fixture (the old `s.parts.push(SparkP{..})` shape, now the pool's).
    fn spark_slot(
        pool: &mut tr_core::particles::DustPool,
        slot: usize,
        x: f32,
        set: impl FnOnce(&mut tr_core::particles::Dust),
    ) {
        let d = pool.slot_mut(slot);
        d.active = true;
        d.dust_type = 213;
        d.position = Vec2::new(x, 100.0);
        d.velocity = Vec2::ZERO;
        d.scale = 1.0;
        d.fade_in = 0.0;
        d.frame_x = 130;
        d.frame_y = 60;
        d.frame_w = 8;
        d.frame_h = 8;
        set(d);
    }

    /// DW-3.1 (repointed) — the pool tick runs the type-213
    /// `Dust.UpdateDust` arms in the C# source order: the generic gravity
    /// fall (`Dust.cs:1819-1829`), the X drag (`:2071-2072`), the rotation
    /// (`:744` + `:2080`), the fadeIn ramp (`:2083-2124`) and the noGravity
    /// damp (`:2335-2341`).
    #[test]
    fn sparks_ai_runs_the_type_213_update() {
        let cam = Vec2::ZERO;
        let screen = Vec2::new(1600.0, 900.0);
        // The falling spark: gravity +0.1, X drag ×0.99, the fade-done
        // shrink −0.2, no damp.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 100.0, |d| {
            d.velocity = Vec2::new(2.0, 0.0);
            d.no_gravity = false;
            d.rotation = 9.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        let p = pool.slot(0);
        assert_eq!(
            p.position,
            Vec2::new(102.0, 100.0),
            "the move precedes the AI"
        );
        assert_eq!(p.velocity, Vec2::new(1.98, 0.1));
        assert_eq!(p.rotation, 0.99, "rotation = vel.X * 0.5");
        assert!((p.scale - 0.8).abs() < 1e-6, "the fade-done shrink −0.2");
        // The weightless spark with a live fadeIn: the ramp climbs 0.1/tick
        // until it passes `fadeIn`, which stands down, and the generic gravity
        // tail ALSO runs — the 213 arm is a standalone statement of the C#'s
        // first ladder (between the `:477` move and the fresh chain head at
        // `:2127`), so it does not exclude the second chain's final
        // `else if (type != 304 && noGravity)` tail (`Dust.cs:2335`): the
        // ×0.92 damp and the fadeIn-gated −0.04 ride every tick.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 100.0, |d| {
            d.velocity = Vec2::new(2.0, 1.0);
            d.fade_in = 1.15;
            d.no_gravity = true;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        let p = pool.slot(0);
        assert_eq!(p.velocity, Vec2::new(2.0 * 0.99 * 0.92, 0.92), "the drag AND the tail damp");
        assert_eq!(p.scale, 1.1, "the ramp's first +0.1 (fadeIn still live: no −0.04)");
        assert_eq!(p.fade_in, 1.15, "still under the fadeIn");
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        let p = pool.slot(0);
        assert_eq!(p.fade_in, 0.0, "1.2 > 1.15 stands the fadeIn down");
        assert!(
            (p.scale - 1.16).abs() < 1e-5,
            "1.2 then the fade-done −0.04"
        );
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        let p = pool.slot(0);
        assert!(
            (p.scale - 0.92).abs() < 1e-5,
            "the burn-out −0.2 then the damp −0.04"
        );
    }

    /// The type-213 deaths (`Dust.cs:447-449`, `:2343-2345`, `:2393-2395` —
    /// the scale floor is 0.1 with the pool-pressure nibbles pinned to 0).
    #[test]
    fn the_type_213_ai_dies_below_the_view_and_at_the_scale_floor() {
        let cam = Vec2::ZERO;
        let screen = Vec2::new(1600.0, 900.0);
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 0.0, |d| {
            d.position.y = 451.0;
            d.no_gravity = true;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        assert_eq!(
            live_type(&pool, 213),
            0,
            "below the view bottom (450) it dies"
        );
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 0.0, |d| {
            d.position.y = 100.0;
            d.scale = 0.15;
            d.no_gravity = false;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &tworld(), 0.0, true);
        assert_eq!(
            live_type(&pool, 213),
            0,
            "the −0.2 shrink crosses the 0.1 floor"
        );
    }

    /// DW-2.3 (the sparks half) — a paused or menu ctx refuses the drain's
    /// spawn: the pool's guard chain (`Dust.cs:77-96`) gates it; the
    /// delegate's post-spawn arms still roll onto the refused sentinel
    /// (`Main.dust[6000]`), which never becomes a live slot.
    #[test]
    /// DW-2.1/2.3 — the sparks tick is the shared `Dust.UpdateDust` loop with
    /// the 213 chain arms as the hook: the shared head moves the slot first
    /// (`Dust.cs:477`), the 213 arm runs (the fall, the drag, the burn-out),
    /// and the generic gravity tail does NOT (the arm matched — `:2335` is the
    /// chain's final `else if`, `:2330-2336`), and `dCount` lands at 0 (the
    /// dead recompute).
    #[test]
    fn s1_e_3_the_sparks_tick_rides_the_shared_update_dust_loop() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut pool = tr_core::particles::DustPool::new();
        let mut rng = 42u64;
        let idx = spawn_spark(
            &mut pool,
            &mut crate::app::SplitmixRand(&mut rng),
            &ctx,
            &world,
            SparkSpawn { pos: SPARK_POS, speed_x: 0.0, speed_y: 0.0, body: SparkBody::Dust213 },
        );
        assert_ne!(idx, tr_core::particles::DUST_SENTINEL, "the spawn landed");
        {
            let d = pool.slot_mut(idx);
            d.velocity = Vec2::new(2.0, 0.0);
            d.no_gravity = false;
            d.fade_in = 0.0;
            d.scale = 1.0;
        }
        let pos0 = pool.slot(idx).position;
        tick(
            &mut pool,
            &mut crate::app::SplitmixRand(&mut rng),
            6000,
            Vec2::ZERO,
            Vec2::new(1600.0, 900.0),
            1.0,
            &world,
            0.0,
            true,
        );
        let p = pool.slot(idx);
        assert_eq!(p.position, pos0 + Vec2::new(2.0, 0.0), "the shared move precedes the arm");
        assert_eq!(p.velocity, Vec2::new(1.98, 0.1), "the 213 arm: the drag and the fall (gravity dust: the tail arm needs noGravity)");
        assert!((p.scale - 0.8).abs() < 1e-6, "the burn-out −0.2 (fadeIn 0 at the tick head)");
        assert_eq!(pool.d_count, 0.0, "one active slot, then the dead recompute");
        // The no-gravity spark proves the tail runs alongside the arm: the
        // 213 arms are standalone statements of the C#'s first ladder (they
        // sit between the `:477` move and the fresh chain head at `:2127`),
        // so they do not exclude the second chain's final
        // `else if (type != 304 && noGravity)` tail (`Dust.cs:2335`).
        {
            let d = pool.slot_mut(idx);
            d.no_gravity = true;
            d.fade_in = 5.0;
            d.scale = 1.0;
            d.velocity = Vec2::new(2.0, 1.0);
        }
        tick(
            &mut pool,
            &mut crate::app::SplitmixRand(&mut rng),
            6000,
            Vec2::ZERO,
            Vec2::new(1600.0, 900.0),
            1.0,
            &world,
            0.0,
            true,
        );
        let p = pool.slot(idx);
        assert_eq!(p.velocity, Vec2::new(2.0 * 0.99 * 0.92, 0.92), "the drag AND the tail damp run together");
    }

    /// W4-d50 WIRING — the live tick's chain feeds the counters and the
    /// pre_move shrink/lift rides the skeleton: a 76 flake bumps
    /// `pool.snow_dust` (the d54 `84eb739b` env-delta hunk, `Dust.cs:1747`
    /// + the `:428` head reset), a 268 speck bumps `pool.sand_storm_count`
    /// (`:1791`), and a 299 spark takes the `:470-471` ×0.96 / −0.01 BEFORE
    /// the `:477` move (the g13-canonical 35 increment stays single: a 35
    /// slot counts exactly one).
    #[test]
    fn w4_d50_the_wired_tick_lands_the_snow_sand_deltas_and_the_pre_move() {
        let world = World::new("wire".into(), 64, 64);
        let mut pool = tr_core::particles::DustPool::new();
        let mk = |pool: &mut tr_core::particles::DustPool, i: usize, ty: i32| {
            let d = pool.slot_mut(i);
            d.active = true;
            d.dust_type = ty;
            d.scale = 1.0;
            d.fade_in = 0.0;
            d.position = Vec2::new(100.0, 100.0);
            d.velocity = Vec2::new(0.0, 0.0);
        };
        mk(&mut pool, 0, 76);
        mk(&mut pool, 1, 268);
        mk(&mut pool, 2, 299);
        mk(&mut pool, 3, 35);
        pool.snow_dust = 9; // the head reset must zero it (`:428`)
        pool.sand_storm_count = 9;
        tick(&mut pool, &mut tr7(), 6000, Vec2::ZERO, Vec2::new(1600.0, 900.0), 1.0, &world, 0.0, true);
        assert_eq!(pool.snow_dust, 1, "one live 76 — the env delta lands post-pass");
        assert_eq!(pool.sand_storm_count, 1, "one live 268 — SandStormCount++");
        assert_eq!(pool.lava_bubbles, 1, "one live 35 — the canonical inline increment, ONCE");
        // The 299 pre_move ran BEFORE the move: scale ×0.96 then the chain's
        // generic burn-out −0.01 (`:2127`); vy lifted −0.01 then the generic
        // gravity fall +0.1 (`:1827`).
        let d = pool.slot(2);
        assert!((d.scale - (0.96 - 0.01)).abs() < 1e-5, "the :470 shrink precedes the burn-out");
        assert!((d.velocity.y - (-0.01 + 0.1)).abs() < 1e-5, "the :471 lift precedes the fall");
    }

    fn s1_e_1_the_ctx_gates_stop_the_spark_drain() {
        let world = World::new("spark".into(), 8, 8);
        for menu in [true, false] {
            let ctx = tr_core::particles::SpawnCtx {
                game_menu: menu,
                game_paused: !menu,
                ..spark_ctx()
            };
            let mut pool = tr_core::particles::DustPool::new();
            let mut rng = 42u64;
            let idx = spawn_spark(
                &mut pool,
                &mut crate::app::SplitmixRand(&mut rng),
                &ctx,
                &world,
                SparkSpawn {
                    pos: SPARK_POS,
                    speed_x: 0.0,
                    speed_y: 0.0,
                    body: SparkBody::Dust213,
                },
            );
            assert_eq!(
                idx,
                tr_core::particles::DUST_SENTINEL,
                "the ctx gate refused the spawn"
            );
            assert_eq!(live_type(&pool, 213), 0);
            assert!(!pool.slot(tr_core::particles::DUST_SENTINEL).active);
        }
    }

    /// A counting `DustRng` over the splitmix state — DW-2.5's probe: the
    /// `SplitmixRand` semantics with a draw counter.
    struct CountingRand {
        state: u64,
        draws: u32,
    }

    impl CountingRand {
        fn sample(&mut self) -> f64 {
            self.draws += 1;
            (crate::app::next_rand(&mut self.state) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    impl tr_core::particles::DustRng for CountingRand {
        fn next_max(&mut self, max: i32) -> i32 {
            (self.sample() * max as f64) as i32
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            (self.sample() * (max - min) as f64) as i32 + min
        }
        fn next_float(&mut self) -> f32 {
            self.sample() as f32
        }
    }

    /// A stream state whose first splitmix draw samples ≥ `floor` — the
    /// refuse-band probe (any draw of `Next(4)` with sample ≥ 0.25 refuses).
    fn state_with_first_sample_at_least(floor: f64) -> u64 {
        for s in 1u64.. {
            let mut st = s;
            let sample = (crate::app::next_rand(&mut st) >> 11) as f64 / (1u64 << 53) as f64;
            if sample >= floor {
                return s;
            }
        }
        unreachable!()
    }

    /// A stream state whose first splitmix draw samples < `ceiling` — the
    /// pass-band probe (`Next(4)` with sample < 0.25 rolls 0).
    fn state_with_first_sample_below(ceiling: f64) -> u64 {
        for s in 1u64.. {
            let mut st = s;
            let sample = (crate::app::next_rand(&mut st) >> 11) as f64 / (1u64 << 53) as f64;
            if sample < ceiling {
                return s;
            }
        }
        unreachable!()
    }

    /// DW-2.1 — the queued spark surfaces as a type-213 pool entry: the
    /// request → `spawn_spark` → slot path, the drain the C#'s
    /// `Main.dust` writes ride.
    #[test]
    fn s1_e_1_the_queued_spark_surfaces_as_a_type_213_pool_entry() {
        let (pool, idx) = spawn_one(SPARK_POS, 0.0, 9);
        assert_ne!(idx, tr_core::particles::DUST_SENTINEL);
        let d = pool.slot(idx);
        assert!(d.active && d.dust_type == 213, "the 213 entry");
        assert_eq!(live_type(&pool, 213), 1);
    }

    /// DW-2.1 — the stand-in's 500-spark cap retires: the pool pressure
    /// governs (the C# 6000-slot scan + the bands, `Dust.cs:103-152`).
    #[test]
    fn s1_e_1_the_500_cap_retires_and_pool_pressure_governs() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        // 600 sparks in a fresh pool all land (slots 0..599, far below the
        // pressure bands) — the stand-in's MAX_SPARKS = 500 dropped the
        // last 100.
        let mut pool = tr_core::particles::DustPool::new();
        let mut queue = Vec::new();
        let mut rng = 11u64;
        wheel_sparks(&mut queue, &mut rng, SPARK_POS, 20, 52, 600, 0.0, SparkVariant::Sparks);
        for s in &queue {
            spawn_spark(
                &mut pool,
                &mut crate::app::SplitmixRand(&mut rng),
                &ctx,
                &world,
                *s,
            );
        }
        assert_eq!(live_type(&pool, 213), 600, "the 500 cap is gone");
        // Pressure governs: past the 0.9 band (slots 0..=5400 active) a
        // refusing stream adds no live slot — the pool's own refusal.
        let mut pool = tr_core::particles::DustPool::new();
        for i in 0..=5400 {
            pool.slot_mut(i).active = true;
        }
        let refuse_state = state_with_first_sample_at_least(0.25);
        let mut counting = CountingRand {
            state: refuse_state,
            draws: 0,
        };
        spawn_spark(
            &mut pool,
            &mut counting,
            &ctx,
            &world,
            SparkSpawn {
                pos: SPARK_POS,
                speed_x: 0.0,
                speed_y: 0.0,
                body: SparkBody::Dust213,
            },
        );
        assert_eq!(live_type(&pool, 213), 0, "the band refused the spawn");
    }

    /// DW-2.5 (the minecart site) — the pool's band rolls are the ONLY new
    /// draws on the spark stream: with the bands inactive the spawn consumes
    /// exactly the C#'s 11 drain rolls (the pool's 6 `NewDust` rolls + the
    /// delegate's 5; the stand-in drew the same 11 inline), a crossed band
    /// adds exactly the one conditional `Next(4)` roll, and a band refusal
    /// leaves the delegate's 5 rolling (the C# writes `Main.dust[6000]`
    /// blindly, `Dust.cs:79`).
    #[test]
    fn s1_e_1_the_pool_rolls_are_the_only_new_spark_draws() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let spawn = SparkSpawn {
            pos: SPARK_POS,
            speed_x: 0.0,
            speed_y: 0.0,
            body: SparkBody::Dust213,
        };
        // Bands inactive: 6 + 5 = 11 draws, no band roll.
        let mut pool = tr_core::particles::DustPool::new();
        let mut counting = CountingRand {
            state: 42,
            draws: 0,
        };
        spawn_spark(&mut pool, &mut counting, &ctx, &world, spawn);
        assert_eq!(counting.draws, 11, "pool 6 + delegate 5, bands inactive");
        // A crossed 0.9 band that accepts: the one `Next(4)` roll added.
        let pass_state = state_with_first_sample_below(0.25);
        let mut pool = tr_core::particles::DustPool::new();
        for i in 0..=5400 {
            pool.slot_mut(i).active = true;
        }
        let mut counting = CountingRand {
            state: pass_state,
            draws: 0,
        };
        spawn_spark(&mut pool, &mut counting, &ctx, &world, spawn);
        assert_eq!(
            counting.draws, 12,
            "the band's Next(4) + the 11 spawn rolls"
        );
        // A crossed 0.9 band that refuses: one band roll + the delegate's 5,
        // and no live slot.
        let refuse_state = state_with_first_sample_at_least(0.25);
        let mut pool = tr_core::particles::DustPool::new();
        for i in 0..=5400 {
            pool.slot_mut(i).active = true;
        }
        let mut counting = CountingRand {
            state: refuse_state,
            draws: 0,
        };
        spawn_spark(&mut pool, &mut counting, &ctx, &world, spawn);
        assert_eq!(counting.draws, 6, "the band roll + the delegate's 5");
        assert_eq!(live_type(&pool, 213), 0);
    }

    /// DW-3.2 — the threshold chain (`Player.cs:19873-19902`, mirrored at
    /// `:19931-19959`): three 50% rolls above 0.66·max, two 2-in-3 rolls in
    /// the middle band, one guaranteed below — each accepted roll one spark,
    /// and Item55 queued once per call regardless of the rolls.
    #[test]
    fn threshold_rolls_follow_the_c_sharp_counts() {
        let max = 13.0f32;
        // Above 0.66·max: all three gates pass → 3 sparks.
        let seed = seed_with_spark_count(10.4, 3);
        let mut sparks = Vec::new();
        let mut sounds = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut sounds,
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(10.4, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 3);
        assert_eq!(
            sounds,
            vec![CartCue {
                cue: "Item_55",
                x: 110.0,
                y: 226.0
            }]
        );
        // All three gates fail → 0 sparks, and the stream moved exactly the
        // three gate rolls (the twin check).
        let seed = seed_with_spark_count(10.4, 0);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(10.4, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert!(sparks.is_empty());
        let mut twin = seed;
        for _ in 0..3 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "three failed gates = three rolls");
        // The middle band: both 2-in-3 gates pass → 2 sparks; both fail → 0
        // (exactly the two gate rolls).
        let seed = seed_with_spark_count(6.5, 2);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(6.5, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 2);
        let seed = seed_with_spark_count(6.5, 0);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(6.5, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert!(sparks.is_empty());
        let mut twin = seed;
        for _ in 0..2 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "two failed gates = two rolls");
        // Below 0.33·max: the guaranteed single spark.
        let mut sparks = Vec::new();
        let mut rng = 12345u64;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(2.0, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 1);
        let mut twin = 12345u64;
        for _ in 0..3 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(
            rng, twin,
            "one spark = the 3 request rolls (side + speeds); the pool's              11 rolls ride the same stream at drain time (S1-E-1 phase 2)"
        );
    }

    /// M2-MOUNT-4 census (research/371 §3) — the Meowcart/track rattle's pitch
    /// does NOT scale with speed. The Item55 cue is identical at every speed
    /// (same asset, same position — the cue carries no pitch term); the pitch
    /// is the FIXED case-2 window `-Next(-20, -11) * 0.02`
    /// (`LegacySoundPlayer.cs:616`), drawn at play time on the audio path,
    /// never from the cart velocity. Speed only gates the SPARK density (the
    /// threshold chain above). This witness pins the cue's speed-invariance
    /// across the three density tiers.
    #[test]
    fn m4_the_rattle_cue_is_speed_invariant_no_pitch_scaling() {
        let max = 13.0f32;
        let mut cues = Vec::new();
        for vx in [2.0f32, 6.5, 10.4] {
            // low / mid / high density tiers
            let mut sounds = Vec::new();
            let mut sparks = Vec::new();
            let mut rng = 999u64;
            rolling_wheel_sparks(
                &mut sounds,
                &mut sparks,
                &mut rng,
                SPARK_POS,
                20,
                52,
                Vec2::new(vx, 0.0),
                max,
                0.0,
            );
            assert_eq!(sounds.len(), 1, "one rattle per gate-passing tick");
            cues.push(sounds[0].clone());
        }
        // The cue is byte-identical across the tiers — no velocity term.
        assert_eq!(cues[0], cues[1]);
        assert_eq!(cues[1], cues[2]);
        assert_eq!(cues[0].cue, "Item_55");
        // The fixed pitch window's bounds: -Next(-20,-11)*0.02 spans
        // [0.22, 0.40] (Next(-20,-11) ∈ [-20,-12], negated ×0.02) — a
        // constant band, independent of any speed.
        let lo = -(-12i32) as f32 * 0.02;
        let hi = -(-20i32) as f32 * 0.02;
        assert!((lo - 0.24).abs() < 1e-6 && (hi - 0.40).abs() < 1e-6);
    }

    /// DW-3.2 + the plan's P3 boundary — the f64 compares
    /// (`Player.cs:19873/:19888`): the f32 products at exactly 0.66·max
    /// (8.579999923706055) and 0.33·max (4.289999961853027) both sit below
    /// their f64 thresholds, so both fall through the strict `>` into the
    /// lower branch (2 rolls / the guaranteed spark).
    #[test]
    fn the_boundary_speeds_fall_through_the_strict_greater() {
        let max = 13.0f32;
        let vx66 = 13.0f32 * 0.66f32;
        assert!((vx66 as f64) <= (13.0f64) * 0.66, "the boundary fact");
        // The mid branch fires: a seed giving 2 sparks exists there, and the
        // hi branch (3 gates) would cap the queue at 3 — assert the exact
        // mid-branch signature via the guaranteed-position spark offsets is
        // overkill; the count with a 2-spark seed is the branch proof.
        let seed = seed_with_spark_count(6.5, 2);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(vx66, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 2, "0.66·max takes the mid branch");
        let vx33 = 13.0f32 * 0.33f32;
        assert!((vx33 as f64) <= (13.0f64) * 0.33, "the boundary fact");
        let mut sparks = Vec::new();
        let mut rng = 777u64;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            Vec2::new(vx33, 0.0),
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 1, "0.33·max takes the guaranteed branch");
    }

    /// DW-3.2 — the gate rolls and the per-spark request rolls consume ONE
    /// stream in the C# order: the hi branch burns 3 gate rolls plus 3 per
    /// accepted spark (`Player.cs:19873-19902` interleaved with
    /// `DelegateMethods.cs:223-227`); the pool's 11 spawn rolls continue the
    /// same stream at drain time (S1-E-1 phase 2).
    #[test]
    fn the_gate_rolls_and_spark_rolls_share_one_stream_in_order() {
        let max = 13.0f32;
        let vel = Vec2::new(10.4, 0.0);
        // hi branch, 2 of the 3 gates pass → 2 sparks.
        let seed = seed_with_spark_count(10.4, 2);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            vel,
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert_eq!(sparks.len(), 2);
        let mut twin = seed;
        for _ in 0..(3 + 2 * 3) {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(
            rng, twin,
            "3 gate rolls + 2 sparks of 3 request rolls; the pool's rolls              continue the same stream at drain time"
        );
        // hi branch, all gates fail → the 3 rolls only.
        let seed = seed_with_spark_count(10.4, 0);
        let mut sparks = Vec::new();
        let mut rng = seed;
        rolling_wheel_sparks(
            &mut Vec::new(),
            &mut sparks,
            &mut rng,
            SPARK_POS,
            20,
            52,
            vel,
            max,
            0.0,
            SparkVariant::Sparks,
        );
        assert!(sparks.is_empty());
        let mut twin = seed;
        for _ in 0..3 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "3 failed gates = 3 rolls");
    }

    /// DW-3.3 (the replay half) — the named stream is seed-deterministic:
    /// the same seed and inputs give byte-equal spark queues.
    #[test]
    fn the_spark_stream_replays_identically() {
        let max = 13.0f32;
        for seed in [1u64, 0xDEAD_BEEF, u64::MAX] {
            let mut a = Vec::new();
            let mut ra = seed;
            rolling_wheel_sparks(
                &mut Vec::new(),
                &mut a,
                &mut ra,
                SPARK_POS,
                20,
                52,
                Vec2::new(11.0, 0.0),
                max,
                0.25,
                SparkVariant::Sparks,
            );
            let mut b = Vec::new();
            let mut rb = seed;
            rolling_wheel_sparks(
                &mut Vec::new(),
                &mut b,
                &mut rb,
                SPARK_POS,
                20,
                52,
                Vec2::new(11.0, 0.0),
                max,
                0.25,
                SparkVariant::Sparks,
            );
            assert_eq!(a, b);
            assert_eq!(ra, rb);
        }
    }

    /// The cue positions (`DelegateMethods.cs:248-256`): `(int)Position +
    /// (Width/2, Height/2)` - C# int casts truncate toward zero.
    #[test]
    fn landing_and_bumper_sounds_carry_the_c_sharp_positions() {
        let mut sounds = Vec::new();
        minecart_landing_sound(&mut sounds, Vec2::new(100.7, 208.9), 20, 52);
        assert_eq!(sounds[0].cue, "Item_53");
        assert_eq!(sounds[0].x, 110.0);
        assert_eq!(sounds[0].y, 234.0);
        minecart_bumper_sound(&mut sounds, Vec2::new(-0.5, -0.5), 20, 52);
        assert_eq!(sounds[1].cue, "Item_56");
        assert_eq!(sounds[1].x, 10.0, "(int)(-0.5) truncates to 0");
        assert_eq!(sounds[1].y, 26.0);
    }

    /// W4-c14 - the Meowcart delegates (`Mount.cs:659-667`): the landing
    /// and bumper queue their own case-37 cue names (NOT Item-style names)
    /// at the truncated centre; the drain routes them to the case-37 play.
    #[test]
    fn w4_c14_the_meowcart_delegates_queue_the_case37_cues() {
        let mut sounds = Vec::new();
        meowcart_landing_sound(&mut sounds, Vec2::new(100.7, 208.9), 20, 52);
        assert_eq!(sounds[0].cue, "MeowcartLanding");
        assert_eq!((sounds[0].x, sounds[0].y), (110.0, 234.0));
        meowcart_bumper_sound(&mut sounds, Vec2::new(-0.5, -0.5), 20, 52);
        assert_eq!(sounds[1].cue, "MeowcartBumper");
        assert_eq!((sounds[1].x, sounds[1].y), (10.0, 26.0));
    }

    /// W4-c14 - the fart carts' delegates (`DelegateMethods.cs:308-326`):
    /// the jump plays ONE Item16; the landing plays Item16 THEN Item53 and
    /// the bumper Item16 THEN Item56, in the C# order, all at the same
    /// truncated centre.
    #[test]
    fn w4_c14_the_fart_delegates_play_their_cue_pairs_in_order() {
        let mut sounds = Vec::new();
        minecart_jumping_sound_fart(&mut sounds, Vec2::new(100.7, 208.9), 20, 52);
        assert_eq!(
            sounds,
            vec![CartCue {
                cue: "Item_16",
                x: 110.0,
                y: 234.0
            }]
        );
        sounds.clear();
        minecart_landing_sound_fart(&mut sounds, Vec2::new(100.7, 208.9), 20, 52);
        assert_eq!(sounds.len(), 2);
        assert_eq!(sounds[0].cue, "Item_16", "the fart first (`:316`)");
        assert_eq!(sounds[1].cue, "Item_53", "then the landing (`:317`)");
        assert_eq!((sounds[0].x, sounds[0].y), (110.0, 234.0));
        assert_eq!((sounds[1].x, sounds[1].y), (110.0, 234.0));
        sounds.clear();
        minecart_bumper_sound_fart(&mut sounds, Vec2::new(100.7, 208.9), 20, 52);
        assert_eq!(sounds.len(), 2);
        assert_eq!(sounds[0].cue, "Item_16", "the fart first (`:323`)");
        assert_eq!(sounds[1].cue, "Item_56", "then the bumper (`:324`)");
        assert_eq!((sounds[0].x, sounds[0].y), (110.0, 234.0));
        assert_eq!((sounds[1].x, sounds[1].y), (110.0, 234.0));
    // -- W4-f6 final 4: the unclaimed track-side spark variants ---------------
    // `DelegateMethods.Minecart.SparksFart/SparksTerraFart/SparksMech/SparksMeow`
    // (the 51/53/11/33 cart rows' `MinecartDust` delegates, `Mount.cs:1099`/
    // `:1107`/`:1487`/`:1013`). The base `Sparks` (213) keeps its own witnesses
    // above; these pin the four variant bodies + the queue-time TerraFart branch.

    /// The row→delegate map (`Mount.cs` registrations): 51 Fart, 53 TerraFart,
    /// 11 Mech, 33 Meow, every other cart row the base Sparks.
    #[test]
    fn w4_f6_spark_variant_of_maps_the_cart_rows() {
        assert_eq!(spark_variant_of(51), SparkVariant::Fart);
        assert_eq!(spark_variant_of(53), SparkVariant::TerraFart);
        assert_eq!(spark_variant_of(11), SparkVariant::Mech);
        assert_eq!(spark_variant_of(33), SparkVariant::Meow);
        // The base cart + the hand-written 6/13/16 rows keep Sparks.
        for k in [6, 13, 16, 15, 18, 25, 34, 38] {
            assert_eq!(spark_variant_of(k), SparkVariant::Sparks, "row {k}");
        }
    }

    /// `SparksMech` (`DelegateMethods.cs:376-391`): dust 260, the 213 body with
    /// a `+0.5` fadeIn band (vs the 213 `+1`). Same 11 drain rolls as the 213.
    #[test]
    fn w4_f6_mech_spark_is_dust_260_with_the_half_fade_band() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut pool = tr_core::particles::DustPool::new();
        let mut counting = CountingRand { state: 42, draws: 0 };
        let idx = spawn_spark(
            &mut pool,
            &mut counting,
            &ctx,
            &world,
            SparkSpawn { pos: SPARK_POS, speed_x: 0.0, speed_y: 0.0, body: SparkBody::Dust260 },
        );
        assert_ne!(idx, tr_core::particles::DUST_SENTINEL);
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 260);
        // fadeIn = scale + 0.5 + 0.01*Next(0,51) — the band sits in [scale+0.5,
        // scale+1.0], strictly below the 213's [scale+1, scale+1.5].
        assert!(d.fade_in >= d.scale + 0.5 - 1e-4 && d.fade_in <= d.scale + 1.0 + 1e-4);
        assert_eq!(counting.draws, 11, "the same 6 pool + 5 delegate rolls");
        // The 260 sheet cell: x = 10*260 − 2000 = 600, y = 60 + 10*row (two
        // per-100 shifts, `Dust.cs:180-186`).
        assert_eq!(d.frame_x, 600);
        assert!((60..=80).contains(&d.frame_y) && d.frame_y % 10 == 0);
    }

    /// `SparksFart` (`DelegateMethods.cs:330-345`): dust 211, alpha 50 base +
    /// two independent 50% `+25` bumps, scale 0.8, `noLight = true`, the
    /// `Next(3) == 0` gravity pick, NO fadeIn band. 12 drain rolls (6 pool +
    /// 2 alpha + 1 gravity + 3 velocity).
    #[test]
    fn w4_f6_fart_spark_is_dust_211_with_the_alpha_bumps() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut pool = tr_core::particles::DustPool::new();
        let mut counting = CountingRand { state: 42, draws: 0 };
        let idx = spawn_spark(
            &mut pool,
            &mut counting,
            &ctx,
            &world,
            SparkSpawn { pos: SPARK_POS, speed_x: 0.0, speed_y: 0.0, body: SparkBody::Dust211 },
        );
        assert_ne!(idx, tr_core::particles::DUST_SENTINEL);
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 211);
        assert!(d.no_light, "the delegate sets noLight");
        assert_eq!(d.fade_in, 0.0, "no fadeIn band");
        // alpha ∈ {50, 75, 100} — the base 50 plus 0/1/2 bumps of 25.
        assert!(matches!(d.alpha, 50 | 75 | 100), "alpha {}", d.alpha);
        // scale = (1 + Next(-20,21)*0.01) * 0.8 ∈ [0.64, 0.96].
        assert!(d.scale >= 0.64 - 1e-4 && d.scale <= 0.96 + 1e-4);
        assert_eq!(counting.draws, 12, "6 pool + 2 alpha + 1 gravity + 3 velocity");
        // The 211 sheet cell: x = 10*211 − 2000 = 110, y = 60 + 10*row.
        assert_eq!(d.frame_x, 110);
    }

    /// `SparksTerraFart`'s green arm (`DelegateMethods.cs:355-370`): dust 107,
    /// alpha 100, scale 0.8, the green→white lerp colour, `noLightEmittance`,
    /// the gravity pick. 13 drain rolls (1 colour NextFloat + 6 pool + 2 alpha
    /// + 1 gravity + 3 velocity).
    #[test]
    fn w4_f6_terra_spark_green_arm_is_dust_107_with_the_lerp_colour() {
        let world = World::new("spark".into(), 8, 8);
        let ctx = spark_ctx();
        let mut pool = tr_core::particles::DustPool::new();
        let mut counting = CountingRand { state: 42, draws: 0 };
        let idx = spawn_spark(
            &mut pool,
            &mut counting,
            &ctx,
            &world,
            SparkSpawn { pos: SPARK_POS, speed_x: 0.0, speed_y: 0.0, body: SparkBody::Dust107 },
        );
        assert_ne!(idx, tr_core::particles::DUST_SENTINEL);
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 107);
        assert!(d.no_light_emittance, "the delegate sets noLightEmittance");
        assert_eq!(d.fade_in, 0.0, "no fadeIn band");
        assert!(matches!(d.alpha, 100 | 125 | 150), "alpha {}", d.alpha);
        // The colour lerps (64,220,96) toward white by NextFloat()*0.3 ∈ [0,0.3]:
        // R ∈ [64,121], G ∈ [220,230], B ∈ [96,143].
        assert!(d.color.r >= 64 && d.color.r <= 121, "R {}", d.color.r);
        assert!(d.color.g >= 220 && d.color.g <= 231, "G {}", d.color.g);
        assert!(d.color.b >= 96 && d.color.b <= 144, "B {}", d.color.b);
        assert_eq!(counting.draws, 13, "1 colour + 6 pool + 2 alpha + 1 gravity + 3 velocity");
        // The 107 sheet cell: x = 10*107 − 1000 = 70, y = 30 + 10*row.
        assert_eq!(d.frame_x, 70);
    }

    /// The TerraFart 50% branch (`DelegateMethods.cs:350`) rolls at QUEUE time,
    /// BEFORE the side offset + speed rolls: the branch pick is the first draw,
    /// then the chosen body's side/speeds. A seed whose first `Next(2)` is 0
    /// takes the Fart (211) arm; otherwise the green (107) arm.
    #[test]
    fn w4_f6_terra_fart_branch_rolls_before_the_side_and_speeds() {
        // Find a seed whose first splitmix Next(2) == 0 (the Fart branch).
        let fart_seed = (1u64..)
            .find(|&s| {
                let mut st = s;
                (crate::app::next_rand(&mut st) % 2) == 0
            })
            .unwrap();
        let mut queue = Vec::new();
        let mut rng = fart_seed;
        wheel_sparks(&mut queue, &mut rng, SPARK_POS, 20, 52, 1, 0.0, SparkVariant::TerraFart);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].body, SparkBody::Dust211, "first Next(2)==0 → Fart");
        // The stream moved 4 rolls: branch + side + 2 speeds.
        let mut twin = fart_seed;
        for _ in 0..4 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "branch + side + 2 speeds = 4 queue rolls");
        // A seed whose first Next(2) == 1 takes the green arm.
        let green_seed = (1u64..)
            .find(|&s| {
                let mut st = s;
                (crate::app::next_rand(&mut st) % 2) == 1
            })
            .unwrap();
        let mut queue = Vec::new();
        let mut rng = green_seed;
        wheel_sparks(&mut queue, &mut rng, SPARK_POS, 20, 52, 1, 0.0, SparkVariant::TerraFart);
        assert_eq!(queue[0].body, SparkBody::Dust107, "first Next(2)==1 → green");
    }

    /// The variant tick arms (`Dust.cs`): 260 shares the 213 fall/drag/rotation/
    /// fade; 211 falls + accumulates rotation + the generic −0.01 fade; 107
    /// takes the `velocity *= 0.9` gravity arm (both axes) instead of the fall.
    #[test]
    fn w4_f6_variant_ticks_follow_the_c_sharp_arms() {
        let cam = Vec2::ZERO;
        let screen = Vec2::new(1600.0, 900.0);
        // 64×64 — the 211 arm's SolidCollision clamps to `maxTilesY - 40`.
        let world = World::new("spark".into(), 64, 64);
        // 260: the 213 body — fall +0.1, drag ×0.99, rotation = vel.X*0.5,
        // fade-done −0.2.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 100.0, |d| {
            d.dust_type = 260;
            d.velocity = Vec2::new(2.0, 0.0);
            d.no_gravity = false;
            d.rotation = 9.0;
            d.fade_in = 0.0;
            d.scale = 1.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &world, 0.0, true);
        let p = pool.slot(0);
        assert_eq!(p.velocity, Vec2::new(1.98, 0.1), "260 falls like 213");
        assert_eq!(p.rotation, 0.99, "260 rotation = vel.X*0.5 (the zero-arm)");
        assert!((p.scale - 0.8).abs() < 1e-6, "260 fade-done −0.2");
        // 211: fall +0.1, drag ×0.99, rotation ACCUMULATES (+=), generic −0.01.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 100.0, |d| {
            d.dust_type = 211;
            d.velocity = Vec2::new(2.0, 0.0);
            d.no_gravity = false;
            d.no_light = false; // avoid the solid-kill (empty world anyway)
            d.rotation = 9.0;
            d.fade_in = 0.0;
            d.scale = 1.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &world, 0.0, true);
        let p = pool.slot(0);
        assert_eq!(p.velocity, Vec2::new(1.98, 0.1), "211 falls");
        assert!((p.rotation - (9.0 + 0.99)).abs() < 1e-5, "211 rotation += vel.X*0.5");
        assert!((p.scale - 0.99).abs() < 1e-6, "211 generic fade −0.01");
        // 107: the gravity arm is velocity *= 0.9 (BOTH axes), not the +0.1 fall.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 100.0, |d| {
            d.dust_type = 107;
            d.velocity = Vec2::new(2.0, 1.0);
            d.no_gravity = false;
            d.rotation = 0.0;
            d.fade_in = 0.0;
            d.scale = 1.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &world, 0.0, true);
        let p = pool.slot(0);
        // gravity ×0.9 → (1.8, 0.9), then drag ×0.99 on X → 1.782.
        assert!((p.velocity.x - 1.8 * 0.99).abs() < 1e-5, "107 X: ×0.9 then ×0.99");
        assert!((p.velocity.y - 0.9).abs() < 1e-5, "107 Y: ×0.9, no +0.1 fall");
        assert!((p.scale - 0.99).abs() < 1e-6, "107 generic fade −0.01");
    }

    /// The 211 solid-kill (`Dust.cs:729-732`): a `noLight` 211 over a solid tile
    /// deactivates; without `noLight` it survives. (The world is 64×64 — the
    /// `SolidCollision` port clamps to `maxTilesY - 40`.)
    #[test]
    fn w4_f6_fart_spark_dies_over_solid_when_no_light() {
        let mut world = World::new("spark".into(), 64, 64);
        // A solid stone tile at (3,3) → world px (48..64, 48..64).
        let i = world.index(3, 3);
        world.tiles[i] = Tile { kind: 1, ..Default::default() };
        world.tiles[i].flags |= FLAG_ACTIVE;
        let cam = Vec2::ZERO;
        let screen = Vec2::new(1600.0, 900.0);
        // noLight 211 sitting on the stone → dies.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 52.0, |d| {
            d.dust_type = 211;
            d.position = Vec2::new(52.0, 52.0);
            d.velocity = Vec2::ZERO;
            d.no_light = true;
            d.no_gravity = true;
            d.scale = 1.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &world, 0.0, true);
        assert!(!pool.slot(0).active, "noLight 211 over solid dies");
        // Same position, noLight false → survives.
        let mut pool = tr_core::particles::DustPool::new();
        spark_slot(&mut pool, 0, 52.0, |d| {
            d.dust_type = 211;
            d.position = Vec2::new(52.0, 52.0);
            d.velocity = Vec2::ZERO;
            d.no_light = false;
            d.no_gravity = true;
            d.scale = 1.0;
        });
        tick(&mut pool, &mut tr7(), 6000, cam, screen, 1.0, &world, 0.0, true);
        assert!(pool.slot(0).active, "without noLight the 211 survives");
    }

    /// `SpawnFartCloud` (`DelegateMethods.cs:258-306`) — the 51 cart's cloud:
    /// three 435-437 gores at the `(Width/2 − 18, Height − 16)` anchor with
    /// the (0, −10, +10) X offsets and the −4 Y lift, the ×0.2/+carry/Y×0.75
    /// post-writes, the 20-tick delay gate, NO dust tail (53-only). The gore
    /// types are SAFE_GORE + special_ai 0, so each gore costs exactly 3 rolls
    /// (the type pick + the 2 NewGore velocity rolls) — 9 total.
    #[test]
    fn w4_f6_fart_cloud_spawns_three_gores_and_gates_on_the_delay() {
        let world = World::new("cloud".into(), 64, 64);
        let ctx = spark_ctx();
        let mut gore = tr_core::gore::GorePool::new();
        let mut dust = tr_core::particles::DustPool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let mut delay = 0i32;
        let pos = Vec2::new(300.0, 300.0);
        spawn_fart_cloud(
            &mut gore, &mut dust, &mut rng, &ctx, &world,
            pos, 20, 52, Vec2::new(5.0, 0.0), 51, &mut delay, true,
        );
        assert_eq!(delay, 20, "the useDelay arm re-arms the 20-tick gate");
        let live: Vec<_> = gore.slots()[..tr_core::gore::GORE_SENTINEL]
            .iter()
            .filter(|g| g.active)
            .collect();
        assert_eq!(live.len(), 3, "three cloud gores");
        // The anchor: (300 + 20/2 − 18, 300 + 52 − 16 − 4) = (292, 332).
        let anchor = Vec2::new(292.0, 332.0);
        for (g, off) in live.iter().zip([0.0f32, -10.0, 10.0]) {
            assert!((435..=437).contains(&g.gore_type), "type {}", g.gore_type);
            assert_eq!(g.position, anchor + Vec2::new(off, 0.0));
            assert!(!g.sticky, "the 435-437 sticky=false chain");
            // velocity = (NewGore rolls) ×0.2 + carry(0.5, 0), Y ×0.75.
            assert!((g.velocity.x - 0.5).abs() <= 0.4 + 1e-4, "vx {}", g.velocity.x);
        }
        assert_eq!(live_type(&dust, 107), 0, "the 51 cart has no dust tail");
        // The gated call: delay > 0 refuses with ZERO draws.
        let mut rng2 = tr_core::UnifiedRandom::new(7);
        let mut gore2 = tr_core::gore::GorePool::new();
        let mut dust2 = tr_core::particles::DustPool::new();
        spawn_fart_cloud(
            &mut gore2, &mut dust2, &mut rng2, &ctx, &world,
            pos, 20, 52, Vec2::ZERO, 51, &mut delay, true,
        );
        assert!(gore2.slots()[..tr_core::gore::GORE_SENTINEL].iter().all(|g| !g.active));
        assert_eq!(rng2.peek(), tr_core::UnifiedRandom::new(7).peek(), "no draws");
        // useDelay: false (the jump/landing delegates) bypasses the gate.
        spawn_fart_cloud(
            &mut gore2, &mut dust2, &mut rng2, &ctx, &world,
            pos, 20, 52, Vec2::ZERO, 51, &mut delay, false,
        );
        assert_eq!(
            gore2.slots()[..tr_core::gore::GORE_SENTINEL].iter().filter(|g| g.active).count(),
            3,
            "the useDelay:false arm fires despite the gate"
        );
    }

    /// `SpawnFartCloud`'s Terra-Fart tail (`DelegateMethods.cs:288-305`): the
    /// 53 cart adds 15 type-107 dusts at `(Width/2, Height+10)` ± the 30×16
    /// float box, alpha 100, scale 0.6, the green lerp colour, fadeIn
    /// 0.2..0.3, `noLightEmittance`, the `Next(3)==0` gravity pick.
    #[test]
    fn w4_f6_terra_fart_cloud_adds_the_fifteen_green_dusts() {
        let world = World::new("cloud".into(), 64, 64);
        let ctx = spark_ctx();
        let mut gore = tr_core::gore::GorePool::new();
        let mut dust = tr_core::particles::DustPool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let mut delay = 0i32;
        spawn_fart_cloud(
            &mut gore, &mut dust, &mut rng, &ctx, &world,
            Vec2::new(300.0, 300.0), 20, 52, Vec2::ZERO, 53, &mut delay, false,
        );
        assert_eq!(live_type(&dust, 107), 15, "the 53 tail's 15 dusts");
        // The tail anchor: (300 + 10, 300 + 62) = (310, 362); the box spans
        // X ±30, Y −16..0 (the pool's perfect-position overwrite is exact).
        for d in dust.slots()[..tr_core::particles::DUST_SENTINEL].iter().filter(|d| d.active) {
            assert_eq!(d.dust_type, 107);
            assert!(d.position.x >= 280.0 && d.position.x <= 340.0, "x {}", d.position.x);
            assert!(d.position.y >= 346.0 && d.position.y <= 362.0, "y {}", d.position.y);
            assert!(d.alpha >= 100, "alpha {}", d.alpha);
            assert!(d.no_light_emittance);
            assert!(d.fade_in >= 0.2 - 1e-5 && d.fade_in <= 0.3 + 1e-5, "fade {}", d.fade_in);
            // scale = pool roll × 0.6 ∈ [0.48, 0.72].
            assert!(d.scale >= 0.48 - 1e-4 && d.scale <= 0.72 + 1e-4);
            assert!(d.color.g >= 220, "the green lerp G {}", d.color.g);
        }
    }
}
