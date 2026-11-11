//! Minecart tracks — the static rail state machine `Minecart.TrackCollision`
//! (`Minecart.cs:562-941`), the 36-frame track decode `Minecart.Initialize`
//! (`Minecart.cs:103-547`) and the query helpers `GetOnTrack` (`:1200-1241`),
//! `OnTrack` (`:1243-1253`), `TrackRotation` (`:1255-1265`) and
//! `GetMagnetPosition` (`:1290-1294`).
//!
//! Pure module: reads the world and mutates only the position / velocity /
//! last-boost the caller passes in. Player integration (riding, boost apply,
//! dismount) is Phase 2 of M2-PHYS-7; wheel sparks + sounds Phase 3; track
//! placement (`FrameTrack`) is a later follow-up. The mounted player is 20×52
//! (`Mount.SetAsMinecart`, `Mount.cs:2747-2799`) and every function below is
//! calibrated against the default `Minecart.Customization` (`Minecart.cs:29-34`).
//!
//! Phase 2 consumed the ride contract (the Player tick slots call
//! `track_collision`/`on_track`/`track_rotation`/`magnet_position`); the
//! remaining pre-consumer items keep targeted allows until Phase 3 (sparks)
//! and M2-MOUNT-3 (the hook-mount snap, the sprite decode) land.

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

/// `Minecart.GetSourceRect` (`Minecart.cs:1519-1532`): the 16x16 sheet cell of
/// a frame id. The booster frames (`frameID < 36 && _trackType[frameID] == 2`)
/// animate 18 px down by `Main.tileFrame[314]`; an out-of-range id is the C#'s
/// empty `(0, 0, 0, 0)` rectangle — `None` here.
pub(crate) fn source_rect(frame: i32, animation: i32) -> Option<[i32; 4]> {
    if !(0..40).contains(&frame) {
        return None;
    }
    let (x, y) = TEXTURE_POSITION[frame as usize];
    let mut ry = y as i32;
    if frame < 36 && TRACK_FRAMES[frame as usize].track_type == 2 {
        ry += 18 * animation;
    }
    Some([x as i32, ry, 16, 16])
}

/// The sprite layers `DrawTile_MinecartTrack` (`TileDrawing.cs:3786-3824`)
/// emits for one cell, in draw order: `(source rect, row offset, paint)`.
/// Rows are offsets from the track's own row (+1 the hanging ties, −1 the
/// bumpers).
pub(crate) fn track_layers(
    world: &tr_core::World,
    i: i32,
    j: i32,
    tile: &tr_core::Tile,
) -> Vec<([i32; 4], i32, u8)> {
    let (front_paint, back_paint) = track_colors(world, i, j);
    // `Main.tileFrame[314]` animates the booster rows; the tile animator is not
    // ported, so the vanilla `t = 0` state is passed like for every other kind.
    let animation = 0;
    let mut out = Vec::with_capacity(6);
    // The back layer stands down on the `frameY == -1` cells (`:3793`).
    if tile.frame_y != -1 {
        if let Some(rect) = source_rect(tile.frame_y as i32, animation) {
            out.push((rect, 0, back_paint));
        }
    }
    if let Some(rect) = source_rect(tile.frame_x as i32, animation) {
        out.push((rect, 0, front_paint));
    }
    // The hanging ties sit in the cell BELOW (`:3797-3816`).
    if draws_left_decoration(tile.frame_y) {
        if let Some(rect) = source_rect(36, 0) {
            out.push((rect, 1, back_paint));
        }
    }
    if draws_left_decoration(tile.frame_x) {
        if let Some(rect) = source_rect(36, 0) {
            out.push((rect, 1, front_paint));
        }
    }
    if draws_right_decoration(tile.frame_y) {
        if let Some(rect) = source_rect(37, animation) {
            out.push((rect, 1, back_paint));
        }
    }
    if draws_right_decoration(tile.frame_x) {
        if let Some(rect) = source_rect(37, 0) {
            out.push((rect, 1, front_paint));
        }
    }
    // The bumper sits in the cell ABOVE (`:3817-3824`): the regular one 39,
    // the bouncy one 38.
    if draws_bumper(tile.frame_x) {
        if let Some(rect) = source_rect(39, 0) {
            out.push((rect, -1, front_paint));
        }
    } else if draws_bouncy_bumper(tile.frame_x) {
        if let Some(rect) = source_rect(38, 0) {
            out.push((rect, -1, front_paint));
        }
    }
    out
}

/// `Minecart.DrawLeftDecoration` / `DrawRightDecoration`
/// (`Minecart.cs:1442-1458`): the hanging tie is drawn when the side
/// connection is "down" (2).
pub(crate) fn draws_left_decoration(frame: i16) -> bool {
    frame_connections(frame).is_some_and(|f| f.left_connection == 2)
}

pub(crate) fn draws_right_decoration(frame: i16) -> bool {
    frame_connections(frame).is_some_and(|f| f.right_connection == 2)
}

/// `Minecart.DrawBumper` (`Minecart.cs:1460-1469`): the frame's left column
/// carries the regular bumper (−1) sentinel.
pub(crate) fn draws_bumper(frame: i16) -> bool {
    frame_connections(frame).is_some_and(|f| {
        if f.tile_height[0] != -1 {
            f.tile_height[7] == -1
        } else {
            true
        }
    })
}

/// `Minecart.DrawBouncyBumper` (`Minecart.cs:1471-1480`): same for the bouncy
/// bumper (−2).
pub(crate) fn draws_bouncy_bumper(frame: i16) -> bool {
    frame_connections(frame).is_some_and(|f| {
        if f.tile_height[0] != -2 {
            f.tile_height[7] == -2
        } else {
            true
        }
    })
}

/// The frame's table row (`_leftSideConnection`/`_rightSideConnection` index
/// space is 0..36).
fn frame_connections(frame: i16) -> Option<&'static TrackFrameData> {
    if !(0..36).contains(&frame) {
        return None;
    }
    TRACK_FRAMES.get(frame as usize)
}

/// `Tile.color()` (`Tile.cs:883-895`) — the paint byte, 0 when unpainted; the
/// track layers share it (`Minecart.cs:1325-1326`).
fn color_of(tile: Option<&tr_core::Tile>) -> u8 {
    match tile {
        Some(t) if t.active() && t.kind == 314 => t.paint(),
        _ => 0,
    }
}

/// `Minecart.TrackColors` (`Minecart.cs:1321-1437`): the front (`frameX`) and
/// back (`frameY`) track layers take the neighbours' paint so a painted line
/// keeps one colour; the side connections pick which neighbour is "in front".
/// Returns `(front, back)` as the paint byte (`push_sprite_paint`'s input).
pub(crate) fn track_colors(world: &tr_core::World, i: i32, j: i32) -> (u8, u8) {
    let Some(track) = world.tile(i, j) else {
        return (0, 0);
    };
    if track.kind != 314 {
        return (0, 0);
    }
    let mut front = track.paint();
    let mut back = front;
    if track.frame_y == -1 {
        return (front, back);
    }
    let (Some(fx_row), Some(fy_row)) = (frame_connections(track.frame_x), frame_connections(track.frame_y))
    else {
        return (front, back);
    };
    let num = fx_row.left_connection;
    let num2 = fx_row.right_connection;
    let num3 = fy_row.left_connection;
    let num4 = fy_row.right_connection;
    let mut num5 = 0u8;
    let mut num6 = 0u8;
    let mut num7 = 0u8;
    let mut num8 = 0u8;
    for k in 0..4 {
        let conn = match k {
            1 => num2,
            2 => num3,
            3 => num4,
            _ => num,
        };
        let dy = match conn {
            0 => -1,
            1 => 0,
            2 => 1,
            _ => 0,
        };
        let neighbour = if k % 2 != 0 {
            world.tile(i + 1, j + dy)
        } else {
            world.tile(i - 1, j + dy)
        };
        let paint = color_of(neighbour);
        match k {
            1 => num6 = paint,
            2 => num7 = paint,
            3 => num8 = paint,
            _ => num5 = paint,
        }
    }
    if num == num3 {
        if num6 != 0 {
            front = num6;
        } else if num5 != 0 {
            front = num5;
        }
        if num8 != 0 {
            back = num8;
        } else if num7 != 0 {
            back = num7;
        }
        return (front, back);
    }
    if num2 == num4 {
        if num5 != 0 {
            front = num5;
        } else if num6 != 0 {
            front = num6;
        }
        if num7 != 0 {
            back = num7;
        } else if num8 != 0 {
            back = num8;
        }
        return (front, back);
    }
    if num6 == 0 {
        if num5 != 0 {
            front = num5;
        }
    } else if num5 != 0 {
        front = if num2 <= num { num6 } else { num5 };
    }
    if num8 == 0 {
        if num7 != 0 {
            back = num7;
        }
    } else if num7 != 0 {
        back = if num4 <= num3 { num8 } else { num7 };
    }
    (front, back)
}

/// `tile.nactive() && tile.type == 314` (`Minecart.cs:633`) — an active,
/// non-actuated track tile. `nactive()` = active and not actuator-hidden
/// (`Tile.cs:240-247`); the port follows the `liquid.rs:384` shape.
fn is_track_tile(tile: &tr_core::Tile) -> bool {
    tile.active() && tile.flags & FLAG_INACTIVE == 0 && tile.kind == 314
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

/// One `Dust` type-213 particle (`MinecartID.Spark` dust — the wheel spark).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SparkP {
    pub(crate) pos: Vec2,
    pub(crate) vel: Vec2,
    pub(crate) scale: f32,
    pub(crate) fade_in: f32,
    /// The sheet row picked at spawn (`10 * Main.rand.Next(3)`,
    /// `Dust.cs:164`).
    pub(crate) frame: u8,
    pub(crate) no_gravity: bool,
    /// The draw rotation (`Dust.cs:744/:2080` — zeroed then `+= vel.X * 0.5`).
    pub(crate) rot: f32,
}

/// The `Main.dust` stand-in for the wheel sparks: a plain pool the App owns,
/// ticked once per frame and drawn in the world layer (the rain/sand
/// emitter pattern).
pub(crate) struct Sparks {
    parts: Vec<SparkP>,
}

impl Sparks {
    pub(crate) fn new() -> Self {
        Sparks { parts: Vec::new() }
    }

    /// Appends a tick's queued particles (the `world_sim` drain). The pool cap
    /// stands in for the C# 6000-slot pool's pressure bands (`Dust.cs:101-141`):
    /// pinned to no pressure, overflow drops the new particle.
    pub(crate) fn spawn(&mut self, new: Vec<SparkP>) {
        for p in new {
            if self.parts.len() < MAX_SPARKS {
                self.parts.push(p);
            }
        }
    }

    /// The type-213 `Dust.UpdateDust` pass, in the C# source order.
    pub(crate) fn tick(&mut self, camera: Vec2, screen: Vec2, zoom: f32) {
        // `Main.screenPosition.Y + Main.screenHeight` (`Dust.cs:2343`) — the
        // world-pixel bottom of the view.
        let bottom = camera.y + screen.y * 0.5 / zoom;
        let mut i = 0;
        while i < self.parts.len() {
            let mut p = self.parts[i];
            // `Dust.cs:447-449`: runaway scale dies.
            if p.scale > 10.0 {
                self.parts.swap_remove(i);
                continue;
            }
            // `:476`: the move.
            p.pos += p.vel;
            // (`:737-756`: the type-213 arm resets rotation and adds light —
            // the port writes no dust light; the rotation value is rebuilt
            // below.)
            // `:1819-1829` (the generic `!noGravity` tail; 213 is not 107):
            // the spark falls.
            if !p.no_gravity {
                p.vel.y += 0.1;
            }
            // `:2071-2072` (the `41`/`44`/else-if chain tail): the air drag.
            p.vel.x *= 0.99;
            // `:744` + `:2080`: rotation is zeroed by the 213 arm each tick,
            // then the generic arm adds the drag — net `vel.X * 0.5`.
            p.rot = p.vel.x * 0.5;
            // `:2083-2124`: the fadeIn ramp (`:2106-2108`: the 213 arm gains
            // 0.1/tick while the spawn fadeIn stands; `:2119-2124`: then
            // −0.2/tick — the spark burns out fast).
            if p.fade_in > 0.0 && p.fade_in < 100.0 {
                p.scale += 0.1;
                if p.scale > p.fade_in {
                    p.fade_in = 0.0;
                }
            } else {
                p.scale -= 0.2;
            }
            // `:2335-2341` (the `!= 304 && noGravity` chain tail): the
            // gravity-free spark dampens; once the fadeIn stands down it also
            // shrinks.
            if p.no_gravity {
                p.vel *= 0.92;
                if p.fade_in == 0.0 {
                    p.scale -= 0.04;
                }
            }
            // `:2343-2345`: below the view it dies.
            if p.pos.y > bottom {
                self.parts.swap_remove(i);
                continue;
            }
            // `:2393-2395`: the scale floor — `num86` stays 0.1 with the
            // pool-pressure `dCount` nibbles pinned to 0 (no pressure).
            if p.scale < 0.1 {
                self.parts.swap_remove(i);
                continue;
            }
            self.parts[i] = p;
            i += 1;
        }
    }

    /// The generic `Main.DrawDust` draw (`Main.cs:39003-39013`) with the
    /// type-213 pins: the draw scale is forced to 1 (`:38974-38977`), the tint
    /// is `GetAlpha`'s 213 arm — a self-lit scale-scaled gray with no light
    /// term (`Dust.cs:2498-2501`) — and the `GetColor` overlay never fires
    /// (the spawn colour is default-black, `Main.cs:39006`).
    pub(crate) fn draw(
        &mut self,
        r: &mut tr_render::sprite::SpriteRenderer,
        lookups: &crate::assets::Lookups,
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
        for p in &self.parts {
            // `Dust.frame` for 213 (`Dust.cs:177-189`): x = 10*213 − 2000 =
            // 130, y = 60 + 10*row, an 8×8 cell.
            let Some(uv) = lookups.dust_uv(130, 60 + p.frame as i32 * 10, 8, 8) else {
                continue;
            };
            // `int num2 = (int)(scale / 2.5f * 255f)` — the XNA `Color(int,…)`
            // ctor byte-casts, so scales past 2.5 wrap.
            let n = (p.scale / 2.5 * 255.0) as i32;
            let a = (n as u8) as f32 / 255.0;
            // `Main.cs:38974-38977`: draw scale 1 — the 8 px frame at zoom.
            let size = 8.0 * zoom;
            r.push_fx_sprite_rot(to_screen(p.pos), [size, size], uv, [a, a, a, a], p.rot);
        }
    }

    /// Live population (tests).
    pub(crate) fn count(&self) -> usize {
        self.parts.len()
    }
}

/// The pool cap (see [`Sparks::spawn`]).
const MAX_SPARKS: usize = 500;

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
/// point, then `spark_count` delegate calls.
pub(crate) fn wheel_sparks(
    sparks: &mut Vec<SparkP>,
    rng: &mut u64,
    position: Vec2,
    width: i32,
    height: i32,
    spark_count: u32,
    rotation: f32,
) {
    // `:1625` — `Position + (Width/2 − TextureWidth/2, Height/2) +
    // MagnetOffset`, the same arithmetic as `GetMagnetPosition` (`:1290-1294`).
    let at = magnet_position(position, width, height);
    for _ in 0..spark_count {
        sparks.push(sparks_one(rng, at, rotation));
    }
}

/// `DelegateMethods.Minecart.Sparks` (`DelegateMethods.cs:223-242`) — one
/// spark, all 14 rolls in the C# order (the two `NewDust` speed arguments, the
/// six in-`NewDust` rolls `Dust.cs:159-191`, then the emitter's own six).
fn sparks_one(rng: &mut u64, dust_position: Vec2, rotation: f32) -> SparkP {
    // `:225`: the ±13 side offset rotated by `Minecart.rotation`
    // (`Vector2.RotatedBy`: (x cosθ − y sinθ, x sinθ + y cosθ)).
    let side: f32 = if next_int(rng, 2) == 0 { -13.0 } else { 13.0 };
    let rotated = Vec2::new(13.0 * rotation.cos(), 13.0 * rotation.sin()) * side.signum();
    // `:226`: the `Dust.NewDust(dustPosition, 1, 1, 213, Next(-2,3), Next(-2,3))`
    // argument rolls first (C# evaluates the args before the call).
    let speed_x = next_range(rng, -2, 3) as f32;
    let speed_y = next_range(rng, -2, 3) as f32;
    // `Dust.cs:159-160`: the 1×1 box clamps to 5, so both jitters roll
    // `Next(1)` (always 0) and add 4 px. ChildSafety.Disabled = true pins the
    // dangerous-dust branch off (`ChildSafety.cs:38`) — no roll there.
    let _jitter_x = next_int(rng, 1);
    let _jitter_y = next_int(rng, 1);
    let mut pos = dust_position + rotated + Vec2::new(4.0, 4.0);
    // `Dust.cs:161-162`: the base velocity plus the caller's speed.
    let mut vel = Vec2::new(
        next_range(rng, -20, 21) as f32 * 0.1 + speed_x,
        next_range(rng, -20, 21) as f32 * 0.1 + speed_y,
    );
    // `Dust.cs:164`: the sheet row.
    let frame = next_int(rng, 3) as u8;
    // `Dust.cs:190-191`: `scale = 1 + Next(-20,21)*0.01` (the Scale arg is 1).
    let mut scale = 1.0 + next_range(rng, -20, 21) as f32 * 0.01;
    // `:228`: `fadeIn = scale + 1 + 0.01 * Next(0,51)`.
    let fade_in = scale + 1.0 + 0.01 * next_int(rng, 51) as f32;
    let mut no_gravity = true; // `:227/:229`
    // `:230-232`: the velocity recipe.
    vel *= next_range(rng, 15, 51) as f32 * 0.01;
    vel.x *= next_range(rng, 25, 101) as f32 * 0.01;
    vel.y -= next_range(rng, 15, 31) as f32 * 0.1;
    // `:233`.
    pos.y -= 4.0;
    // `:234-241`: 2-in-3 the spark falls; otherwise it stays weightless and
    // shrinks.
    if next_int(rng, 3) != 0 {
        no_gravity = false;
    } else {
        scale *= 0.6;
    }
    SparkP {
        pos,
        vel,
        scale,
        fade_in,
        frame,
        no_gravity,
        rot: 0.0,
    }
}

/// `Player.cs:19868-19902` (the left arm, `velocity.X >= 0`) /
/// `:19926-19958` (the right arm, `velocity.X <= 0`): the Item55 cue, then
/// the threshold chain over `|velocity.X|` vs `max_run_speed`.
pub(crate) fn rolling_wheel_sparks(
    sounds: &mut Vec<CartCue>,
    sparks: &mut Vec<SparkP>,
    rng: &mut u64,
    position: Vec2,
    width: i32,
    height: i32,
    velocity: Vec2,
    max_run_speed: f32,
    rotation: f32,
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
            );
        }
        if next_int(rng, 2) == 0 {
            wheel_sparks(sparks, rng, position, width, height, 1, rotation);
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
            );
        }
        if next_int(rng, 3) != 0 {
            wheel_sparks(sparks, rng, position, width, height, 1, rotation);
        }
    }
    // `:19900-19902`: the guaranteed single spark at `position`.
    else {
        wheel_sparks(sparks, rng, position, width, height, 1, rotation);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::Tile;
    use tr_core::tile::FLAG_ACTIVE;

    /// The mounted cart body: width 20 (unchanged), height 42 + heightBoost 10
    /// (`Mount.SetAsMinecart`, `Mount.cs:2747-2799`).
    const CART_W: i32 = 20;
    const CART_H: i32 = 52;
    /// The straight-rail surface offset — `TRACK_FRAMES[1]` is level 5
    /// everywhere → 6 px (`Minecart.cs:117-121`, `:131-133`, `:437-447`).
    const STRAIGHT_RAIL_PX: f32 = 6.0;

    // TILE-DRAW-7: `DrawTile_MinecartTrack` (`TileDrawing.cs:3786-3821`)
    // draws the track from `Minecart.GetSourceRect` (`Minecart.cs:1519-1532`),
    // not from the autotile frames — the frame ids are 18 px sheet cells.
    #[test]
    fn the_track_frames_map_onto_the_sheet_cells() {
        // The first four frames sit on the top row (`Minecart.cs:129-149`).
        assert_eq!(source_rect(0, 0), Some([0, 0, 16, 16]));
        assert_eq!(source_rect(1, 0), Some([18, 0, 16, 16]));
        assert_eq!(source_rect(2, 0), Some([36, 18, 16, 16]));
        assert_eq!(source_rect(3, 0), Some([54, 18, 16, 16]));
        // Frame 36/37 are the hanging ties, 38/39 the bumpers — and the C#
        // assigns 39 before 38, so 39 is the top cell of the pair.
        assert_eq!(source_rect(36, 0), Some([0, 108, 16, 16]));
        assert_eq!(source_rect(37, 0), Some([18, 108, 16, 16]));
        assert_eq!(source_rect(38, 0), Some([18, 126, 16, 16]));
        assert_eq!(source_rect(39, 0), Some([0, 126, 16, 16]));
        // Out of range is the empty rect (`:1521-1524`).
        assert_eq!(source_rect(-1, 0), None);
        assert_eq!(source_rect(40, 0), None);
    }

    // Only the booster frames (`_trackType == 2`, i.e. 30..35) animate; the
    // animation walks the sheet 18 px down (`Minecart.cs:1527-1530`).
    #[test]
    fn only_the_booster_frames_animate() {
        let straight = source_rect(1, 1).unwrap();
        assert_eq!(straight[1], 0, "a plain straight run does not animate");
        let booster = source_rect(30, 1).unwrap();
        assert_eq!(booster, [36, 54 + 18, 16, 16], "the booster advances a row");
        // 36+ are outside the animation gate.
        assert_eq!(source_rect(37, 2).unwrap()[1], source_rect(37, 0).unwrap()[1]);
    }

    // `DrawLeftDecoration`/`DrawRightDecoration`/`DrawBumper`/
    // `DrawBouncyBumper` (`Minecart.cs:1442-1480`) read the frame's side
    // connections and its (-1/-2) sentinels.
    #[test]
    fn the_track_decorations_and_bumpers_follow_the_frame_table() {
        // Frame 4 connects down on the left (`Minecart.cs:151-159`).
        assert!(draws_left_decoration(4) && !draws_right_decoration(4));
        // Frame 5 connects down on the right (`:161-169`).
        assert!(!draws_left_decoration(5) && draws_right_decoration(5));
        assert!(!draws_left_decoration(1) && !draws_right_decoration(1));
        // Frame 2 is the plain bumper, frame 24 the bouncy one
        // (`:135-141`, `:319-325`).
        assert!(draws_bumper(2) && !draws_bouncy_bumper(2));
        assert!(!draws_bumper(24) && draws_bouncy_bumper(24));
        assert!(!draws_bumper(1) && !draws_bouncy_bumper(1));
        for bad in [-1i16, 36, 39] {
            assert!(!draws_left_decoration(bad));
            assert!(!draws_bumper(bad));
        }
    }

    // `Minecart.TrackColors` (`Minecart.cs:1321-1437`): a painted track takes
    // its neighbours' paint per layer.
    #[test]
    fn the_track_layers_take_the_neighbour_paint() {
        let mut w = track_world();
        // A straight run: frame 1, back layer down (`frameY = 0`, the lone
        // piece) — unpainted neighbours leave both layers unpainted.
        place_track(&mut w, 10, 50, 1, -1);
        assert_eq!(track_colors(&w, 10, 50), (0, 0));
        // Paint the track itself: both layers inherit it before any lookup.
        let i = w.index(10, 50);
        w.tiles[i].set_paint(9);
        assert_eq!(track_colors(&w, 10, 50), (9, 9), "frameY == -1 keeps both");
        // With a back layer the neighbour walk runs: a left neighbour painted
        // cyan (front-left = frame's left connection, straight → same row).
        place_track(&mut w, 10, 50, 1, 0);
        let j = w.index(9, 50);
        w.tiles[j].kind = 314;
        w.tiles[j].frame_x = 1;
        w.tiles[j].frame_y = -1;
        w.tiles[j].flags = FLAG_ACTIVE;
        w.tiles[j].set_paint(3);
        let (front, back) = track_colors(&w, 10, 50);
        assert!(front == 3 || back == 3, "the painted neighbour reaches a layer");
        assert!(front != 0 || back != 0);
    }

    // TILE-DRAW-7: the layer list `DrawTile_MinecartTrack` emits per cell
    // (`TileDrawing.cs:3786-3824`).
    #[test]
    fn the_track_arm_emits_the_vanilla_layers() {
        let mut w = track_world();
        // A lone straight piece: one layer, its own row (`:3793` skips the
        // `frameY == -1` back layer).
        place_track(&mut w, 10, 50, 1, -1);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].0, [18, 0, 16, 16]);
        assert_eq!(layers[0].1, 0);
        // A left "down" frame (4) hangs a tie in the cell below.
        place_track(&mut w, 10, 50, 4, -1);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[1].0, [0, 108, 16, 16], "frame 36 is the tie");
        assert_eq!(layers[1].1, 1, "the tie draws one row down");
        // A right "down" frame (5) uses the other tie cell.
        place_track(&mut w, 10, 50, 5, -1);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers[1].0, [18, 108, 16, 16], "frame 37 is the right tie");
        // The regular bumper (frame 2) sits one row UP.
        place_track(&mut w, 10, 50, 2, -1);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[1].0, [0, 126, 16, 16], "frame 39 is the bumper");
        assert_eq!(layers[1].1, -1);
        // The bouncy bumper (frame 24) uses frame 38 instead.
        place_track(&mut w, 10, 50, 24, -1);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers[1].0, [18, 126, 16, 16]);
        assert_eq!(layers[1].1, -1);
        // A two-layer cell draws the back layer first, then the front one.
        place_track(&mut w, 10, 50, 1, 20);
        let layers = track_layers(&w, 10, 50, w.tile(10, 50).unwrap());
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].0, [0, 72, 16, 16], "frame 20 = pressure lone");
        assert_eq!(layers[1].0, [18, 0, 16, 16], "then the front straight");
        // An unplaced cell (`frame_x = -1`) draws nothing at all.
        place_track(&mut w, 10, 50, -1, -1);
        assert!(track_layers(&w, 10, 50, w.tile(10, 50).unwrap()).is_empty());
    }

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
    /// `want` sparks — the gate rolls interleave with the 14-roll sparks, so
    /// the wanted gate outcomes sit at strided stream positions only the real
    /// roll order knows.
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
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    const SPARK_POS: Vec2 = Vec2::new(100.0, 200.0);

    /// DW-3.1 — one spark reproduces the `DelegateMethods.Minecart.Sparks`
    /// spawn math (`DelegateMethods.cs:223-242`) with `Dust.NewDust`'s rolls
    /// inlined (`Dust.cs:159-191`): the ±13 side offset rotated by
    /// `Minecart.rotation` plus the +4/−4 px box/lift pair, the upward
    /// velocity recipe, the sheet row and the scale/fadeIn bands, the 2-in-3
    /// gravity flip, and exact stream replay.
    #[test]
    fn sparks_one_reproduces_the_dust_213_spawn_math() {
        let at = Vec2::new(1000.0, 2000.0);
        let rot = 0.5f32;
        let (cos, sin) = (rot.cos(), rot.sin());
        let mut weightless = 0;
        for seed in 1..=200u64 {
            let mut rng = seed;
            let p = sparks_one(&mut rng, at, rot);
            // The position: `at ± (13cos, 13sin) + (4, 4)` then `pos.Y -= 4`
            // — the +4/−4 cancel in Y (`:225` + `Dust.cs:159-160` + `:233`).
            let off = p.pos - at;
            let side_x = if off.x > 4.0 { 13.0 * cos } else { -13.0 * cos };
            assert!((off.x - 4.0 - side_x).abs() < 1e-3, "side x: {}", off.x);
            assert!((off.y.abs() - 13.0 * sin).abs() < 1e-3, "side y: {}", off.y);
            // The velocity recipe (`:230-232` over the `NewDust` starts
            // (`Dust.cs:161-162` — ±2.0 each plus the ±2 speed args) scaled
            // by 0.15..0.5, then the X scale 0.25..1 and the Y lift 1.5..3):
            // vel.Y ∈ [−5, 0.5], vel.X ∈ [−2, 2].
            assert!(
                p.vel.y >= -5.0 - 1e-3 && p.vel.y <= 0.5 + 1e-3,
                "vel.y {}",
                p.vel.y
            );
            assert!(p.vel.x.abs() <= 2.0 + 1e-3, "vel.x {}", p.vel.x);
            // The sheet row (`Dust.cs:164`) and the scale/fadeIn bands
            // (`Dust.cs:190-191`, `:228`).
            assert!(p.frame <= 2);
            assert!(p.scale >= 0.8 * 0.6 - 1e-4 && p.scale <= 1.2 + 1e-4);
            assert!((1.8..=2.7).contains(&p.fade_in), "fade {}", p.fade_in);
            // `:234-241`: 2-in-3 the spark gains gravity.
            if p.no_gravity {
                weightless += 1;
            }
        }
        assert!(
            (40..=140).contains(&weightless),
            "the 1-in-3 weightless flip: {weightless}/200"
        );
        // The same seed replays byte-identically.
        let mut rng = 42u64;
        let a = sparks_one(&mut rng, at, rot);
        let mut rng = 42u64;
        let b = sparks_one(&mut rng, at, rot);
        assert_eq!(a, b);
    }

    /// DW-3.1 — `Minecart.WheelSparks` (`Minecart.cs:1623-1630`) lands every
    /// spark on the magnet point (the `:1625` formula — the same arithmetic
    /// as `GetMagnetPosition`, `:1290-1294`) and emits exactly `spark_count`.
    #[test]
    fn wheel_sparks_lands_on_the_magnet_point_with_the_requested_count() {
        let mut sparks = Vec::new();
        let mut rng = 7u64;
        wheel_sparks(&mut sparks, &mut rng, SPARK_POS, 20, 52, 10, 0.0);
        assert_eq!(sparks.len(), 10, "the landing burst's count");
        let at = magnet_position(SPARK_POS, 20, 52);
        assert_eq!(at, Vec2::new(110.0, 252.0), "the bottom-center point");
        for p in &sparks {
            // Rotation 0: the side offset is (±13, 0) — x ∈ {at+17, at−9}
            // after the +4 box, y untouched.
            assert!(
                (p.pos.x - 127.0).abs() < 1e-4 || (p.pos.x - 101.0).abs() < 1e-4,
                "x {}",
                p.pos.x
            );
            assert_eq!(p.pos.y, 252.0);
        }
    }

    /// DW-3.1 — the pool tick runs the type-213 `Dust.UpdateDust` arms in the
    /// C# source order: the generic gravity fall (`Dust.cs:1819-1829`), the X
    /// drag (`:2071-2072`), the rotation (`:744` + `:2080`), the fadeIn ramp
    /// (`:2083-2124`) and the noGravity damp (`:2335-2341`).
    #[test]
    fn sparks_ai_runs_the_type_213_update() {
        let cam = Vec2::ZERO;
        let screen = Vec2::new(1600.0, 900.0);
        // The falling spark: gravity +0.1, X drag ×0.99, the fade-done
        // shrink −0.2, no damp.
        let mut s = Sparks::new();
        s.parts.push(SparkP {
            pos: Vec2::new(100.0, 100.0),
            vel: Vec2::new(2.0, 0.0),
            scale: 1.0,
            fade_in: 0.0,
            frame: 0,
            no_gravity: false,
            rot: 9.0,
        });
        s.tick(cam, screen, 1.0);
        let p = &s.parts[0];
        assert_eq!(p.pos, Vec2::new(102.0, 100.0), "the move precedes the AI");
        assert_eq!(p.vel, Vec2::new(1.98, 0.1));
        assert_eq!(p.rot, 0.99, "rotation = vel.X * 0.5");
        assert!((p.scale - 0.8).abs() < 1e-6, "the fade-done shrink −0.2");
        // The weightless spark with a live fadeIn: the ramp climbs 0.1/tick
        // until it passes `fadeIn`, which stands down; the damp (×0.92) runs
        // every tick and adds −0.04 once the fadeIn is gone.
        let mut s = Sparks::new();
        s.parts.push(SparkP {
            pos: Vec2::new(100.0, 100.0),
            vel: Vec2::new(2.0, 1.0),
            scale: 1.0,
            fade_in: 1.15,
            frame: 1,
            no_gravity: true,
            rot: 0.0,
        });
        s.tick(cam, screen, 1.0);
        let p = &s.parts[0];
        assert_eq!(p.vel, Vec2::new(2.0 * 0.99 * 0.92, 0.92));
        assert_eq!(p.scale, 1.1, "the ramp's first +0.1");
        assert_eq!(p.fade_in, 1.15, "still under the fadeIn");
        s.tick(cam, screen, 1.0);
        let p = &s.parts[0];
        assert_eq!(p.fade_in, 0.0, "1.2 > 1.15 stands the fadeIn down");
        assert!(
            (p.scale - 1.16).abs() < 1e-5,
            "1.2 then the fade-done −0.04"
        );
        s.tick(cam, screen, 1.0);
        let p = &s.parts[0];
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
        let mut s = Sparks::new();
        s.parts.push(SparkP {
            pos: Vec2::new(0.0, 451.0),
            vel: Vec2::ZERO,
            scale: 1.0,
            fade_in: 0.0,
            frame: 0,
            no_gravity: true,
            rot: 0.0,
        });
        s.tick(cam, screen, 1.0);
        assert_eq!(s.count(), 0, "below the view bottom (450) it dies");
        let mut s = Sparks::new();
        s.parts.push(SparkP {
            pos: Vec2::new(0.0, 100.0),
            vel: Vec2::ZERO,
            scale: 0.15,
            fade_in: 0.0,
            frame: 0,
            no_gravity: false,
            rot: 0.0,
        });
        s.tick(cam, screen, 1.0);
        assert_eq!(s.count(), 0, "the −0.2 shrink crosses the 0.1 floor");
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
        );
        assert_eq!(sparks.len(), 1);
        let mut twin = 12345u64;
        for _ in 0..14 {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "one spark = exactly the 14 emitter rolls");
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
        );
        assert_eq!(sparks.len(), 1, "0.33·max takes the guaranteed branch");
    }

    /// DW-3.2 — the gate rolls and the per-spark emitter rolls consume ONE
    /// stream in the C# order: the hi branch burns 3 gate rolls plus 14 per
    /// accepted spark (`Player.cs:19873-19902` interleaved with
    /// `DelegateMethods.cs:223-242` + `Dust.cs:159-191`).
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
        );
        assert_eq!(sparks.len(), 2);
        let mut twin = seed;
        for _ in 0..(3 + 2 * 14) {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rng, twin, "3 gate rolls + 2 sparks of 14");
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
            );
            assert_eq!(a, b);
            assert_eq!(ra, rb);
        }
    }

    /// The cue positions (`DelegateMethods.cs:248-256`): `(int)Position +
    /// (Width/2, Height/2)` — C# int casts truncate toward zero.
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
}
