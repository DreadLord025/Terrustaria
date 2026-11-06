//! Ports of `Terraria/Collision.cs`: `TileCollision`, `SlopeCollision`, `StepUp` and
//! `StepDown`. The player moves exactly like the original: an AABB pass that lets
//! slope triangles pass through when approached from their open side, then the
//! step up/down helpers that lift the hitbox onto 1-tile ledges while `gfxOffY`
//! keeps the sprite at the old height and eases it down afterwards.

use tr_core::tile::Tile;
use tr_core::tiledata as td;
use tr_core::{World, TILE_PX};

/// `Main.tileSolid[type] || (Main.tileSolidTop[type] && frameY == 0)` — and
/// the C#'s every solid test carries `!tile.inActive()`
/// (`Collision.cs:445/:471/:549/:574/...`): an actuated tile is
/// walk-through.
fn solid(t: &Tile) -> bool {
    if !t.active() || t.flags & tr_core::tile::FLAG_INACTIVE != 0 {
        return false;
    }
    if (t.kind as usize) >= td::TILE_COUNT {
        return false;
    }
    // `WorldGen.cs:15715`: `tileSolid[192] = false` at the end of the Living
    // Trees pass — leaf blocks are walk-through for the whole session.
    (td::TILE_SOLID.has(t.kind) && t.kind != 192)
        || (td::TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0)
}

fn solid_kind(kind: u16) -> bool {
    // `WorldGen.cs:15715`: leaf blocks (192) go non-solid after generation.
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind) && kind != 192
}

/// The support-test tile gate of `StepUp`/`StepDown`
/// (`Collision.cs:445/:461/:549/:565/:574/...`): the tile counts as support
/// only when it is active, **not actuated**, genuinely solid — the C#'s
/// `!tile.inActive() && tile.active() && Main.tileSolid[...]` conjuncts.
fn support_solid(t: &Tile) -> bool {
    t.active() && t.flags & tr_core::tile::FLAG_INACTIVE == 0 && solid_kind(t.kind)
}

/// `Tile.slope() > 0 && slope() < 3`: the two ground ramps (`topSlope()`).
fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < world.width && y < world.height
}

/// Vanilla scans up to `maxTilesY - 40`, the row where the underworld boundary
/// starts. Worlds smaller than that have no boundary, so the margin only applies
/// once a world is tall enough to own one.
fn scan_bottom(world: &World) -> i32 {
    if world.height > 400 {
        world.height - 40
    } else {
        world.height
    }
}

pub struct TileCollision {
    pub vel: [f32; 2],
    pub up: bool,
    #[allow(dead_code)]
    pub down: bool,
}

/// `Collision.TileCollision` with `gravDir = 1`, `hoik = true`.
pub fn tile_collision(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: f32,
    height: f32,
    fall_through: bool,
    fall2: bool,
) -> TileCollision {
    let mut up = false;
    let mut down = false;
    let mut result = vel;
    let target = [pos[0] + vel[0], pos[1] + vel[1]];
    let x0 = (pos[0] / TILE_PX) as i32 - 1;
    let x1 = ((pos[0] + width) / TILE_PX) as i32 + 2;
    let y0 = (pos[1] / TILE_PX) as i32 - 1;
    let y1 = ((pos[1] + height) / TILE_PX) as i32 + 2;
    let mut num = -1i32;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let mut num4 = -1i32;
    let cx0 = x0.clamp(0, world.width - 1);
    let cx1 = x1.clamp(0, world.width - 1);
    let bottom = scan_bottom(world);
    let cy0 = y0.clamp(0, bottom);
    let cy1 = y1.clamp(0, bottom);
    let mut num6 = (cy1 + 3) as f32 * TILE_PX;
    for i in cx0..cx1 {
        for j in cy0..cy1 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            if !solid(&t) {
                continue;
            }
            let rx = i as f32 * TILE_PX;
            let mut ry = j as f32 * TILE_PX;
            let mut rh = TILE_PX;
            if t.half_brick() {
                ry += 8.0;
                rh -= 8.0;
            }
            if !(target[0] + width > rx)
                || !(target[0] < rx + TILE_PX)
                || !(target[1] + height > ry)
                || !(target[1] < ry + rh)
            {
                continue;
            }
            let mut flag2 = false;
            let mut flag3 = false;
            let s = t.slope();
            if s > 2 {
                if s == 3 && pos[1] + vel[0].abs() >= ry && pos[0] >= rx {
                    flag3 = true;
                }
                if s == 4 && pos[1] + vel[0].abs() >= ry && pos[0] + width <= rx + TILE_PX {
                    flag3 = true;
                }
            } else if s > 0 {
                flag2 = true;
                if s == 1 && pos[1] + height - vel[0].abs() <= ry + rh && pos[0] >= rx {
                    flag3 = true;
                }
                if s == 2
                    && pos[1] + height - vel[0].abs() <= ry + rh
                    && pos[0] + width <= rx + TILE_PX
                {
                    flag3 = true;
                }
            }
            if flag3 {
                continue;
            }
            let solid_top = td::TILE_SOLID_TOP.has(t.kind);
            if pos[1] + height <= ry {
                down = true;
                if (!(solid_top && fall_through) || !((vel[1] <= 1.0) | fall2)) && num6 > ry {
                    num3 = i;
                    num4 = j;
                    if rh < TILE_PX {
                        num4 += 1;
                    }
                    if num3 != num && !flag2 {
                        result[1] = ry - (pos[1] + height);
                        num6 = ry;
                    }
                }
            } else if pos[0] + width <= rx && !solid_top {
                let ls = world.tile(i - 1, j).map(|t| t.slope()).unwrap_or(0);
                if ls != 2 && ls != 4 {
                    num = i;
                    num2 = j;
                    if num2 != num4 {
                        result[0] = rx - (pos[0] + width);
                    }
                    if num3 == num {
                        result[1] = vel[1];
                    }
                }
            } else if pos[0] >= rx + TILE_PX && !solid_top {
                let rs = world.tile(i + 1, j).map(|t| t.slope()).unwrap_or(0);
                if rs != 1 && rs != 3 {
                    num = i;
                    num2 = j;
                    if num2 != num4 {
                        result[0] = rx + TILE_PX - pos[0];
                    }
                    if num3 == num {
                        result[1] = vel[1];
                    }
                }
            } else if pos[1] >= ry + rh && !solid_top {
                up = true;
                num3 = i;
                num4 = j;
                result[1] = ry + rh - pos[1] + 0.01;
                if num4 == num2 {
                    result[0] = vel[0];
                }
            }
        }
    }
    TileCollision {
        vel: result,
        up,
        down,
    }
}

pub struct SlopeCollision {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub stair: bool,
    pub stair_fall: bool,
}

/// `Collision.SlopeCollision`: slides the body along ground ramps and pushes it
/// out of ceiling ramps.
pub fn slope_collision(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: f32,
    height: f32,
    fall: bool,
) -> SlopeCollision {
    let mut stair = false;
    let mut stair_fall = false;
    let (mut b1, mut b2, mut b3, mut b4) = (false, false, false, false);
    let mut y = pos[1];
    let mut y2 = pos[1];
    let mut v2 = pos;
    let mut v3 = vel;
    let x0 = (pos[0] / TILE_PX) as i32 - 1;
    let x1 = ((pos[0] + width) / TILE_PX) as i32 + 2;
    let y0 = (pos[1] / TILE_PX) as i32 - 1;
    let y1 = ((pos[1] + height) / TILE_PX) as i32 + 2;
    let cx0 = x0.clamp(0, world.width - 1);
    let cx1 = x1.clamp(0, world.width - 1);
    let bottom = scan_bottom(world);
    let cy0 = y0.clamp(0, bottom);
    let cy1 = y1.clamp(0, bottom);
    for i in cx0..cx1 {
        for j in cy0..cy1 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            if !solid(&t) {
                continue;
            }
            let rx = i as f32 * TILE_PX;
            let mut ry = j as f32 * TILE_PX;
            let mut rh = TILE_PX;
            if t.half_brick() {
                ry += 8.0;
                rh -= 8.0;
            }
            if !(pos[0] + width > rx)
                || !(pos[0] < rx + TILE_PX)
                || !(pos[1] + height > ry)
                || !(pos[1] < ry + rh)
            {
                continue;
            }
            let platform = td::SET_PLATFORMS.has(t.kind);
            let mut flag2 = true;
            if platform {
                if vel[1] < 0.0 {
                    flag2 = false;
                }
                if pos[1] + height < j as f32 * TILE_PX
                    || pos[1] + height - (1.0 + vel[0].abs()) > j as f32 * TILE_PX + TILE_PX
                {
                    flag2 = false;
                }
                let s = t.slope();
                if ((s == 1 && vel[0] >= 0.0) || (s == 2 && vel[0] <= 0.0))
                    && (pos[1] + height) / TILE_PX - 1.0 == j as f32
                {
                    flag2 = false;
                }
            }
            if !flag2 {
                continue;
            }
            let flag3 = fall && platform;
            let s = t.slope();
            let ry = j as f32 * TILE_PX;
            if !(pos[0] + width > rx)
                || !(pos[0] < rx + TILE_PX)
                || !(pos[1] + height > ry)
                || !(pos[1] < ry + TILE_PX)
            {
                continue;
            }
            if s == 3 || s == 4 {
                let num4 = if s == 3 {
                    pos[0] - rx
                } else {
                    rx + TILE_PX - (pos[0] + width)
                };
                if num4 >= 0.0 {
                    if pos[1] <= ry + TILE_PX - num4 {
                        let num5 = ry + TILE_PX - pos[1] - num4;
                        if pos[1] + num5 > y2 {
                            v2[1] = pos[1] + num5;
                            y2 = v2[1];
                            if v3[1] < 0.0101 {
                                v3[1] = 0.0101;
                            }
                            if s == 3 {
                                b3 = true;
                            } else {
                                b4 = true;
                            }
                        }
                    }
                } else if pos[1] > ry && v2[1] < ry + TILE_PX {
                    v2[1] = ry + TILE_PX;
                    if v3[1] < 0.0101 {
                        v3[1] = 0.0101;
                    }
                }
            }
            if s != 1 && s != 2 {
                continue;
            }
            let num4 = if s == 1 {
                pos[0] - rx
            } else {
                rx + TILE_PX - (pos[0] + width)
            };
            if num4 >= 0.0 {
                if !(pos[1] + height >= ry + num4) {
                    continue;
                }
                let num7 = ry - (pos[1] + height) + num4;
                if !(pos[1] + num7 < y) {
                    continue;
                }
                if flag3 {
                    stair_fall = true;
                    continue;
                }
                stair = platform;
                v2[1] = pos[1] + num7;
                y = v2[1];
                if v3[1] > 0.0 {
                    v3[1] = 0.0;
                }
                if s == 1 {
                    b1 = true;
                } else {
                    b2 = true;
                }
                continue;
            }
            if platform && !(pos[1] + height - 4.0 - vel[0].abs() <= ry) {
                if flag3 {
                    stair_fall = true;
                }
                continue;
            }
            let num8 = ry - height;
            if !(v2[1] > num8) {
                continue;
            }
            if flag3 {
                stair_fall = true;
                continue;
            }
            stair = platform;
            v2[1] = num8;
            if v3[1] > 0.0 {
                v3[1] = 0.0;
            }
        }
    }
    let velocity = [v2[0] - pos[0], v2[1] - pos[1]];
    let v5 = tile_collision(world, pos, velocity, width, height, false, false).vel;
    let mut out_pos = v2;
    let mut out_vel = v3;
    if v5[1] > velocity[1] {
        let num9 = velocity[1] - v5[1];
        out_pos[1] = pos[1] + v5[1];
        if b1 {
            out_pos[0] = pos[0] - num9;
        }
        if b2 {
            out_pos[0] = pos[0] + num9;
        }
        out_vel = [0.0, 0.0];
    } else if v5[1] < velocity[1] {
        let num10 = v5[1] - velocity[1];
        out_pos[1] = pos[1] + v5[1];
        if b3 {
            out_pos[0] = pos[0] - num10;
        }
        if b4 {
            out_pos[0] = pos[0] + num10;
        }
        out_vel = [0.0, 0.0];
    }
    SlopeCollision {
        pos: out_pos,
        vel: out_vel,
        stair,
        stair_fall,
    }
}

/// `Collision.StepUp` with `gravDir = 1`: lifts the body onto a one-tile ledge and
/// records the lift in `gfx_off_y` so the sprite eases down afterwards.
#[allow(clippy::too_many_arguments)]
pub fn step_up(
    world: &World,
    pos: &mut [f32; 2],
    vel: &mut [f32; 2],
    width: f32,
    height: f32,
    step_speed: &mut f32,
    gfx_off_y: &mut f32,
    holds_matching: bool,
) {
    let num = if vel[0] < 0.0 {
        -1i32
    } else if vel[0] > 0.0 {
        1
    } else {
        0
    };
    let vx = pos[0] + vel[0];
    let num2 = ((vx + width * 0.5 + (width * 0.5 + 1.0) * num as f32) / TILE_PX) as i32;
    let num3 = ((pos[1] + height - 1.0) / TILE_PX) as i32;
    let num4 = height as i32 / 16 + if height as i32 % 16 != 0 { 1 } else { 0 };
    if !in_world(world, num2, num3) || num3 >= scan_bottom(world) {
        return;
    }
    if world.tile(num2, num3).is_none() {
        return;
    }
    for i in 1..num4 + 2 {
        if world.tile(num2, num3 - i).is_none() {
            return;
        }
    }
    if world.tile(num2 - num, num3 - num4).is_none() {
        return;
    }
    let mut flag = true;
    for j in 2..num4 + 1 {
        let Some(t) = world.tile(num2, num3 - j) else {
            return;
        };
        flag = flag && (!support_solid(t) || td::TILE_SOLID_TOP.has(t.kind));
    }
    let t = world.tile(num2 - num, num3 - num4).unwrap();
    let flag2 = !support_solid(t) || td::TILE_SOLID_TOP.has(t.kind);
    let Some(tile_a) = world.tile(num2, num3 - 1) else {
        return;
    };
    let Some(tile_a2) = world.tile(num2, num3 - (num4 + 1)) else {
        return;
    };
    let flag3 = !support_solid(tile_a)
        || td::TILE_SOLID_TOP.has(tile_a.kind)
        || (tile_a.slope() == 1 && pos[0] + width * 0.5 > num2 as f32 * TILE_PX)
        || (tile_a.slope() == 2 && pos[0] + width * 0.5 < num2 as f32 * TILE_PX + TILE_PX)
        || (tile_a.half_brick()
            && (!support_solid(tile_a2) || td::TILE_SOLID_TOP.has(tile_a2.kind)));
    let tile_b = world.tile(num2, num3).unwrap();
    let tile_b2 = world.tile(num2, num3 - 1).unwrap();
    let holds = holds_matching
        && ((td::TILE_SOLID_TOP.has(tile_b.kind) && tile_b.frame_y == 0)
            || td::SET_PLATFORMS.has(tile_b.kind))
        && !support_solid(tile_b2);
    let mut flag4 = (tile_b.active()
        && tile_b.flags & tr_core::tile::FLAG_INACTIVE == 0
        && (!top_slope(&tile_b)
            || (tile_b.slope() == 1 && pos[0] + width * 0.5 < num2 as f32 * TILE_PX)
            || (tile_b.slope() == 2 && pos[0] + width * 0.5 > num2 as f32 * TILE_PX + TILE_PX))
        && (!top_slope(&tile_b) || pos[1] + height > num3 as f32 * TILE_PX)
        && ((solid_kind(tile_b.kind) && !td::TILE_SOLID_TOP.has(tile_b.kind)) || holds))
        || (tile_b2.half_brick()
            && tile_b2.active()
            && tile_b2.flags & tr_core::tile::FLAG_INACTIVE == 0);
    flag4 &= !td::TILE_SOLID_TOP.has(tile_b.kind) || !td::TILE_SOLID_TOP.has(tile_b2.kind);
    if !(num2 as f32 * TILE_PX < vx + width) || !(num2 as f32 * TILE_PX + TILE_PX > vx) {
        return;
    }
    if !(flag4 && flag3 && flag && flag2) {
        return;
    }
    let mut num5 = num3 as f32 * TILE_PX;
    if world.tile(num2, num3 - 1).is_some_and(|t| t.half_brick()) {
        num5 -= 8.0;
    } else if world.tile(num2, num3).is_some_and(|t| t.half_brick()) {
        num5 += 8.0;
    }
    if !(num5 < pos[1] + height) {
        return;
    }
    let num6 = pos[1] + height - num5;
    if num6 <= 16.1 {
        *gfx_off_y += pos[1] + height - num5;
        pos[1] = num5 - height;
        *step_speed = if num6 < 9.0 { 1.0 } else { 2.0 };
    }
}

/// `Collision.TileCollisionInStepsOf16` (`Collision.cs:2455-2482`): how far a
/// body of this size travels along the unit direction `dir` before tiles stop
/// it, probed in 16 px steps; `amount` when the whole distance is clear. The
/// slope probes use `Width − 1` (`:2460`), the tile probe the full width.
pub fn tile_collision_in_steps_of_16(
    world: &World,
    position: [f32; 2],
    dir: [f32; 2],
    amount: f32,
    width: f32,
    height: f32,
) -> f32 {
    let step = 16.0f32;
    // `vector` (`:2459`).
    let shifted = [position[0] + 1.0, position[1]];
    let slope_width = width - 1.0;
    let mut travelled = 0.0f32; // `num2`
    while travelled < amount {
        let offset = [dir[0] * travelled, dir[1] * travelled];
        // `position` / `vector3` (`:2463-2464`).
        let pos = [position[0] + offset[0], position[1] + offset[1]];
        let probe = [shifted[0] + offset[0], shifted[1] + offset[1]];
        let remain = (amount - travelled).min(step); // `num3`
        let vel = [dir[0] * remain, dir[1] * remain]; // `vector4`
        let sc = slope_collision(world, probe, vel, slope_width, height, false);
        if sc.pos != probe || sc.vel != vel {
            let dx = sc.pos[0] - probe[0];
            let dy = sc.pos[1] - probe[1];
            return travelled - dx.hypot(dy);
        }
        let probe2 = [probe[0] + vel[0], probe[1] + vel[1]];
        let sc2 = slope_collision(world, probe2, vel, slope_width, height, false);
        if sc2.pos != probe2 || sc2.vel != vel {
            let dx = sc2.pos[0] - probe2[0];
            let dy = sc2.pos[1] - probe2[1];
            return travelled + remain - dx.hypot(dy);
        }
        let tc = tile_collision(world, pos, vel, width, height, false, false);
        if tc.vel != vel {
            return travelled + tc.vel[0].hypot(tc.vel[1]); // `vector5.Length()`
        }
        travelled += step;
    }
    amount
}

/// `Collision.TryChangingSizeFromBottomCenter` (`Collision.cs:2486-2545`):
/// resize the hitbox to `target` keeping the bottom-center fixed. Returns the
/// new top-left, or `None` when a growth has no room (`FailedNoSpace`). A
/// shrink probes nothing and always fits (`:2505-2510` — the branch the cart
/// dismount takes).
pub fn try_changing_size_from_bottom_center(
    world: &World,
    top_left: [f32; 2],
    size: [f32; 2],
    target: [f32; 2],
) -> Option<[f32; 2]> {
    let mut x = top_left[0];
    let mut y = top_left[1];
    let mut w = size[0];
    let mut h = size[1];
    // The vertical change (`num`, `:2489`).
    let dv = (target[1] - h) as i32;
    if dv > 0 {
        let down_room =
            tile_collision_in_steps_of_16(world, [x, y], [0.0, 1.0], dv as f32, w, h) as i32; // `num2`
        let up_room =
            tile_collision_in_steps_of_16(world, [x, y], [0.0, -1.0], dv as f32, w, h) as i32; // `num3`
        if up_room + down_room < dv {
            return None;
        }
        // The split (`:2499-2503`) — the second `num4` add is the source's
        // own arithmetic (a no-op at these bounds) and is kept verbatim.
        let mut up = dv.min(up_room); // `num4`
        let mut down = 0i32; // `num5`
        up += (dv - up - down).min(up_room - up);
        down += (dv - up - down).min(down_room - down);
        y -= up as f32;
        h += (up + down) as f32;
    } else {
        y -= dv as f32; // `Offset(0, -num)`
        h = target[1];
    }
    // The horizontal change (`num6`, `:2512`).
    let dw = (target[0] - w) as i32;
    if dw > 0 {
        let right_room =
            tile_collision_in_steps_of_16(world, [x, y], [1.0, 0.0], dw as f32, w, h) as i32; // `num7`
        let left_room =
            tile_collision_in_steps_of_16(world, [x, y], [-1.0, 0.0], dw as f32, w, h) as i32; // `num8`
        if left_room + right_room < dw {
            return None;
        }
        // The split (`:2530-2536`).
        let mut num10 = (dw / 2).min(left_room).min(right_room);
        let mut num9 = num10;
        num10 += (dw - num10 - num9).min(left_room - num10);
        num9 += (dw - num10 - num9).min(right_room - num9);
        x -= num10 as f32;
        w += (num10 + num9) as f32;
    } else {
        x += (dw / 2) as f32; // `Offset(num6 / 2, 0)` — C# int division
        w = target[0];
    }
    Some([x, y])
}

/// `Collision.StepDown` with `gravDir = 1`: drops the body when the floor falls away
/// while walking, again recording the drop in `gfx_off_y`.
#[allow(clippy::too_many_arguments)]
pub fn step_down(
    world: &World,
    pos: &mut [f32; 2],
    vel: &mut [f32; 2],
    width: f32,
    height: f32,
    step_speed: &mut f32,
    gfx_off_y: &mut f32,
) {
    let vx = pos[0] + vel[0];
    let num = (vx / TILE_PX) as i32;
    let num2 = ((vx + width) / TILE_PX) as i32;
    let num3 = ((pos[1] + height) / TILE_PX).floor() as i32;
    let num4 = height as i32 / 16 + if height as i32 % 16 != 0 { 1 } else { 0 };
    let mut num5 = (num3 + num4) as f32 * TILE_PX;
    let num6 = world.height as f32 - 42.0;
    for i in num..=num2 {
        for j in num3..=num3 + 1 {
            if !in_world(world, i, j) {
                continue;
            }
            let t = world.tile(i, j);
            let active_solid = t.is_some_and(|t| {
                t.active()
                    && t.flags & tr_core::tile::FLAG_INACTIVE == 0
                    && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
            });
            if (j as f32) >= num6 || active_solid {
                let mut num9 = j as f32 * TILE_PX;
                if t.is_some_and(|t| t.half_brick()) {
                    num9 += 8.0;
                }
                let ry = j as f32 * TILE_PX - 17.0;
                if i as f32 * TILE_PX < pos[0] + width
                    && i as f32 * TILE_PX + TILE_PX > pos[0]
                    && ry < pos[1] + height
                    && ry + TILE_PX > pos[1]
                    && num9 < num5
                {
                    num5 = num9;
                }
            }
        }
    }
    let num10 = num5 - (pos[1] + height);
    if num10 > 7.0 && num10 < 17.0 {
        *step_speed = if num10 > 9.0 { 2.5 } else { 1.5 };
        *gfx_off_y += pos[1] + height - num5;
        pos[1] = num5 - height;
    }
}

/// `Collision.TryChangingSizeFromBottomCenter` (`Collision.cs:2486-2545`):
/// resize the box keeping its bottom center. The shrink path (`num <= 0`)
/// is pure geometry and always fits; the grow/widen paths probe tile
/// clearance in 16 px steps (`TileCollisionInStepsOf16`) — not reachable
/// from the mount code (the only caller shrinks the hitbox back to 20x42),
/// so they return `None` until a consumer appears.
pub fn try_changing_size_from_bottom_center(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    target_w: f32,
    target_h: f32,
) -> Option<(f32, f32, f32, f32)> {
    let num = target_h - h;
    if num > 0.0 {
        // `:2490-2508`: the grow path — `TileCollisionInStepsOf16` both ways.
        return None;
    }
    // `:2509-2513`: `Offset(0, -num); Height = targetHeight` — the bottom
    // stays anchored, the top drops to the new height.
    let ny = y - num;
    let num6 = target_w - w;
    if num6 != 0.0 {
        // `:2514-2545`: the widen/narrow paths probe clearance too.
        return None;
    }
    Some((x, ny, w, target_h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_actuated_block_is_walk_through() {
        // `Collision`'s every solid test carries `!tile.inActive()`
        // (`Collision.cs:445/:471/:549/:574/...`): a falling body passes an
        // actuated floor and lands on the row below it.
        let mut w = world();
        for x in 8..12 {
            put(&mut w, x, 20, 1);
            put(&mut w, x, 21, 1);
        }
        // The 40 px body sits above the floor (the bottom at 316 < 320) and
        // moves 5 px down into row 20 — the landing clamps the velocity.
        let solid_hit = tile_collision(
            &w,
            [8.0 * 16.0, 276.0],
            [0.0, 5.0],
            20.0,
            40.0,
            false,
            false,
        );
        assert!(solid_hit.down, "the active floor stops the body");
        assert!(solid_hit.vel[1] < 5.0, "the velocity clamped to the floor");
        for x in 8..12 {
            let i = w.index(x, 20);
            w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        }
        let actuated_pass = tile_collision(
            &w,
            [8.0 * 16.0, 276.0],
            [0.0, 5.0],
            20.0,
            40.0,
            false,
            false,
        );
        assert!(
            !actuated_pass.down,
            "the actuated row does not stop the body"
        );
        assert_eq!(actuated_pass.vel[1], 5.0, "the velocity kept falling");
    }

    /// DW-2.3 (dirty, mechanism level) — `Collision.TryChangingSizeFromBottomCenter`
    /// (`Collision.cs:2486-2545`): a 42→52 growth under a two-tile ceiling has no
    /// room (`FailedNoSpace`), the same growth with headroom shifts the top up,
    /// and the 52→42 dismount shrink always fits (the shrink branch probes no
    /// tiles, `:2505-2510`).
    #[test]
    fn dismount_space_check_blocks_growth_under_a_two_tile_ceiling() {
        let mut w = world();
        // A stone floor (row 20) with a ceiling (row 16) over it — a 3-tile
        // cavity a 42-tall body fits in but a 52-tall growth does not.
        for x in 4..16 {
            put(&mut w, x, 20, 1);
            put(&mut w, x, 16, 1);
        }
        // A body standing in the cavity cannot grow 10 px up into the ceiling.
        let blocked = try_changing_size_from_bottom_center(
            &w,
            [80.0, 20.0 * 16.0 - 42.0],
            [20.0, 42.0],
            [20.0, 52.0],
        );
        assert!(blocked.is_none(), "the ceiling blocks the growth");
        // The same body under open sky grows, keeping the bottom anchored.
        let mut w_open = world();
        for x in 4..16 {
            put(&mut w_open, x, 20, 1);
        }
        let free = try_changing_size_from_bottom_center(
            &w_open,
            [80.0, 20.0 * 16.0 - 42.0],
            [20.0, 42.0],
            [20.0, 52.0],
        );
        let [fx, fy] = free.expect("open sky leaves room");
        assert_eq!(fx, 80.0);
        assert_eq!(fy, 20.0 * 16.0 - 52.0, "the top lifts, the bottom stays");
        // The dismount shrink 52 -> 42 always fits, even in the cavity.
        let shrink = try_changing_size_from_bottom_center(
            &w,
            [80.0, 20.0 * 16.0 - 52.0],
            [20.0, 52.0],
            [20.0, 42.0],
        );
        let [sx, sy] = shrink.expect("the shrink never probes tiles");
        assert_eq!(sx, 80.0);
        assert_eq!(sy, 20.0 * 16.0 - 42.0, "the bottom-center is kept");
    }
}
