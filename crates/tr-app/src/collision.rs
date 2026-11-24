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

/// `Collision.WalkDownSlope` (`Collision.cs:1686-1789`) — the
/// `NPC.Collision_WalkDownSlopes` head of `UpdateCollision`
/// (`NPC.cs:93342-93350`). Velocity-only: the position is returned
/// unchanged (`new Vector4(Position, ...)`), so the port returns the
/// velocity pair. The `Velocity.Y != gravity` early-out (`:1688`) means
/// the pass only fires while the body falls at EXACTLY the tick's
/// gravity — a settled walker's `vy == 0` skips it. The support scan
/// (`:1707-1757`) reads `nactive() && (tileSolid || tileSolidTop)` (the
/// RUNTIME table — 192 cleared, `WorldGen.cs:15714`; the solidTop half
/// carries NO frameY gate here, unlike `solid()`), the half-brick +8,
/// the `(i*16, j*16-17, 16, 16)` probe rect (XNA `Intersects` — strict,
/// touching edges do NOT count) and the `num5 <= num` ceiling with the
/// equal-height slope-direction preference (`num4`: 2 walking left,
/// 1 right). The tail (`:1760-1787`) adds `|Velocity.X|` to Y on the
/// matching ramp when the feet sit at/below the ramp line.
pub fn walk_down_slope(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: f32,
    height: f32,
    gravity: f32,
) -> [f32; 2] {
    // `:1688-1691` — the exact-gravity gate.
    if vel[1] != gravity {
        return vel;
    }
    let mut vy = vel[1];
    // `:1693-1698` — the scan bounds; the Y clamp's upper is
    // `maxTilesY - 2 - 40` (the Utils.Clamp order: min first).
    let value = ((pos[0] / 16.0) as i32).clamp(0, world.width - 1);
    let value2 = (((pos[0] + width) / 16.0) as i32).clamp(0, world.width - 1);
    let raw3 = ((pos[1] + height + 4.0) / 16.0) as i32;
    // The Utils.Clamp order: min first, then max — on a tiny world the
    // upper goes negative and the scan rows read null tiles (the C#'s
    // `Main.tile[i, j] == null` arm; the port's `world.tile` → None).
    let upper3 = world.height - 2 - 40;
    let value3 = if raw3 < 0 { 0 } else if raw3 > upper3 { upper3 } else { raw3 };
    let mut num = (value3 + 3) as f32 * 16.0;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let mut prev_slope = 0u8;
    let num4 = if vel[0] < 0.0 { 2u8 } else { 1u8 };
    // The int-truncated body rect (`(int)Position.X/Y`, `:1724`).
    let (bx, by) = (pos[0] as i32, pos[1] as i32);
    let (bw, bh) = (width as i32, height as i32);
    for i in value..=value2 {
        for j in value3..=value3 + 1 {
            let Some(t) = world.tile(i, j) else { continue };
            // `:1715` — nactive + the runtime solid/solidTop union.
            if !t.active() || t.inactive() {
                continue;
            }
            let solid = (td::TILE_SOLID.has(t.kind) && t.kind != 192)
                || td::TILE_SOLID_TOP.has(t.kind);
            if !solid {
                continue;
            }
            let mut num5 = (j * 16) as f32;
            if t.half_brick() {
                num5 += 8.0;
            }
            // `:1724` — the strict rect intersect + the ceiling.
            let (rx, ry) = (i * 16, j * 16 - 17);
            let intersects = bx < rx + 16 && bx + bw > rx && by < ry + 16 && by + bh > ry;
            if !intersects || num5 > num {
                continue;
            }
            if num == num5 {
                // `:1728-1749` — the equal-height preference.
                if t.slope() == 0 {
                    continue;
                }
                if num2 != -1 && num3 != -1 && prev_slope != 0 {
                    if t.slope() == num4 {
                        num = num5;
                        num2 = i;
                        num3 = j;
                        prev_slope = t.slope();
                    }
                } else {
                    num = num5;
                    num2 = i;
                    num3 = j;
                    prev_slope = t.slope();
                }
            } else {
                num = num5;
                num2 = i;
                num3 = j;
                prev_slope = t.slope();
            }
        }
    }
    // `:1758-1787` — the ramp push (the found tile's slope > 0).
    if num2 != -1 && num3 != -1 && prev_slope > 0 {
        let (tx, ty) = (num2 as f32 * 16.0, num3 as f32 * 16.0);
        match prev_slope {
            2 => {
                let num9 = tx + 16.0 - (pos[0] + width);
                if pos[1] + height >= ty + num9 && vel[0] < 0.0 {
                    vy += vel[0].abs();
                }
            }
            1 => {
                let num9 = pos[0] - tx;
                if pos[1] + height >= ty + num9 && vel[0] > 0.0 {
                    vy += vel[0].abs();
                }
            }
            _ => {}
        }
    }
    [vel[0], vy]
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

/// `WorldGen.SolidOrSlopedTile(tile, includePlatforms = false)`
/// (`WorldGen.cs:69985-70010`): an active solid tile that is not solid-top
/// (or, with `includePlatforms`, a solid-top platform), and not actuated.
/// The no-space teleport scan's floor probes read it (`TeleportHelpers.cs:32-33`).
pub fn solid_or_sloped_tile(t: &Tile, include_platforms: bool) -> bool {
    if !t.active() || !td::TILE_SOLID.has(t.kind) {
        return false;
    }
    let solid_top = td::TILE_SOLID_TOP.has(t.kind);
    if !include_platforms {
        if solid_top {
            return false;
        }
    } else if solid_top && !td::SET_PLATFORMS.has(t.kind) {
        return false;
    }
    !t.inactive()
}

/// `Collision.IsClearSpotTest` (`Collision.cs:2534-2587`) with
/// `checkCardinals = true`, `checkSlopes = false`, `gravDir = 1`: the four
/// cardinal 16-px sweeps must all pass unimpeded. `Player.CanFitInSpaceWithSize`
/// (`Player.cs:6108-6111`) probes the grown mount box this way (the
/// `fallThrough`/`fall2` pair true).
pub fn is_clear_spot_test(
    world: &World,
    pos: [f32; 2],
    test_magnitude: f32,
    width: f32,
    height: f32,
    fall_through: bool,
    fall2: bool,
) -> bool {
    // `:2538-2557`: +X, -X, +Y, -Y — each sweep starts one magnitude AWAY
    // from the probe position and moves back through it.
    for vector in [
        [test_magnitude, 0.0],
        [-test_magnitude, 0.0],
        [0.0, test_magnitude],
        [0.0, -test_magnitude],
    ] {
        let start = [pos[0] - vector[0], pos[1] - vector[1]];
        let tc = tile_collision(world, start, vector, width, height, fall_through, fall2);
        if tc.vel != vector {
            return false;
        }
    }
    true
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

/// `Tile.blockType()` (`Tile.cs:202-214`): 0 = a full block, 1 = a half
/// brick, 2-5 = the four slopes.
fn block_type(t: &Tile) -> u8 {
    if t.half_brick() {
        return 1;
    }
    let s = t.slope();
    if s > 0 {
        s + 1
    } else {
        0
    }
}

/// `Collision.DrownCollision` (`Collision.cs:1383-1432`): the head box's
/// water test — the 10x12 probe at the head (`vector`), the liquid's surface
/// offset (`(256 - liquid) / 32 * 2`, `:1421-1423`) and the solid-floor
/// exclusion on the gravity-side row (`num4`, `:1409`/`:1416`).
///
/// `grav_dir` is the C#'s parameter: the vanilla player passes its own
/// `gravDir = 1f` (`Player.cs:1956`, `CheckDrowning` `:23193`), so `1` keeps
/// the probe at the body TOP (`position.Y - 2`, the head); the `-1` default
/// lowers it by `Height/2 - 6` (the flipped-gravity arm). The NPCs' calls
/// also pass `1f`. `include_slopes` is the C#'s `includeSlopes` — the
/// solid-floor exclusion then also accepts the sloped blocks.
pub fn drown_collision(
    world: &World,
    pos_x: f32,
    pos_y: f32,
    width: i32,
    height: i32,
    grav_dir: f32,
    include_slopes: bool,
) -> bool {
    let num = 10.min(width);
    let num2 = 12.min(height);
    // `:1385-1400`: the probe's top-left — the centre column, the head row.
    let vx = pos_x + (width / 2) as f32 - (num / 2) as f32;
    let mut vy = pos_y - 2.0;
    if grav_dir == -1.0 {
        vy += (height / 2) as f32 - 6.0;
    }
    let value = (pos_x / 16.0) as i32 - 1;
    let value2 = ((pos_x + width as f32) / 16.0) as i32 + 2;
    let value3 = (pos_y / 16.0) as i32 - 1;
    let value4 = ((pos_y + height as f32) / 16.0) as i32 + 2;
    let num3 = value.clamp(0, world.width - 1);
    let value2 = value2.clamp(0, world.width - 1);
    // `Main.maxTilesY - 40` (`:1407-1408`) — the underworld boundary row.
    // The C#'s `Utils.Clamp` folds a negative bound back onto itself (the
    // scan then runs empty); the port clamps the bound at zero instead.
    let bottom = world.height.saturating_sub(40).max(0);
    let value3 = value3.clamp(0, bottom);
    let value4 = value4.clamp(0, bottom);
    let num4 = if grav_dir == 1.0 { value3 } else { value4 - 1 };
    for i in num3..value2 {
        for j in value3..value4 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            if t.liquid == 0 {
                continue;
            }
            // `!tile.lava() && !tile.shimmer()` — the honey drowns, the lava
            // and the shimmer do not.
            if matches!(t.liquid_kind(), tr_core::tile::LiquidKind::Lava | tr_core::tile::LiquidKind::Shimmer)
            {
                continue;
            }
            // `:1416`: the solid floor at the gravity-side row — the water
            // under a solid block does not drown.
            if j == num4
                && t.active()
                && solid_kind(t.kind)
                && !td::TILE_SOLID_TOP.has(t.kind)
                && !(include_slopes && block_type(t) != 0)
            {
                continue;
            }
            let mut ty = j as f32 * 16.0;
            let mut num5 = 16.0f32;
            let num6 = (256.0 - t.liquid as f32) / 32.0;
            ty += num6 * 2.0;
            num5 -= (num6 * 2.0) as i32 as f32;
            if vx + num as f32 > i as f32 * 16.0
                && vx < i as f32 * 16.0 + 16.0
                && vy + num2 as f32 > ty
                && vy < ty + num5
            {
                return true;
            }
        }
    }
    false
}

/// `Utils.FloatIntersect` (`Utils.cs:982-989`): the two boxes overlap. The
/// C#'s comparisons are strict, so a touching edge still counts as an
/// overlap.
fn float_intersect(
    r1x: f32,
    r1y: f32,
    r1w: f32,
    r1h: f32,
    r2x: f32,
    r2y: f32,
    r2w: f32,
    r2h: f32,
) -> bool {
    if r1x > r2x + r2w || r1y > r2y + r2h || r1x + r1w < r2x || r1y + r1h < r2y {
        return false;
    }
    true
}

/// The outcome of [`switch_tiles`]: the C#'s return value (whether a
/// pressure plate consumed the entity - the mine and the geyser arms
/// `continue` the scan, so they never set it), the item the broken plate
/// dropped (the C#'s `WorldGen.KillTile` drop, which the app spawns) and the
/// switch tiles that fired, so the caller can send `59 HitSwitch`
/// (`Collision.cs:3304`, `:3318`, `:3324`, `:3347`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SwitchOutcome {
    pub hit: bool,
    /// `(item, x, y)` - the plate's drop.
    pub dropped: Option<(u16, i32, i32)>,
    /// Every `Wiring.HitSwitch(i, j)` the scan issued, in scan order - the
    /// C#'s `NetMessage.SendData(59, ..., i, j)` sits on each of them.
    pub switches: Vec<(i32, i32)>,
}

/// `Collision.SwitchTiles` (`Collision.cs:3227-3360`): the entity→switch
/// bridge — the pressure plates (135), the land mine (210), the geyser trap
/// (443) and the projectile pressure pad (442).
///
/// The scan is the C#'s: the ±1-margin box around the entity, the four
/// switch kinds, the type-442 projectile-pad branch (the four frame rects,
/// `objType == 4` only, `:3266-3307`) and the "the entity moved onto it"
/// test (`:3310`): the entity's box must reach the tile's `y + 12` line and
/// its old box must not have been over it.
///
/// `obj_type` is the C#'s: 1 = the NPC, 2 = the NPC's other call, 3 = the
/// projectile, 4 = the projectile's pad call, 5 = the player.
pub fn switch_tiles(
    wiring: &mut tr_core::wiring::Wiring,
    world: &mut World,
    pos_x: f32,
    pos_y: f32,
    old_pos_x: f32,
    old_pos_y: f32,
    width: i32,
    height: i32,
    obj_type: i32,
) -> SwitchOutcome {
    let mut num = (pos_x / 16.0) as i32 - 1;
    let mut num2 = ((pos_x + width as f32) / 16.0) as i32 + 2;
    let mut num3 = (pos_y / 16.0) as i32 - 1;
    let mut num4 = ((pos_y + height as f32) / 16.0) as i32 + 2;
    num = num.max(0);
    num2 = num2.min(world.width);
    num3 = num3.max(0);
    num4 = num4.min(world.height - 40);
    // The `59` send sites sit on every `HitSwitch` that does not return
    // (`Collision.cs:3318`, `:3324`), so the scan accumulates them.
    let mut switches: Vec<(i32, i32)> = Vec::new();
    for i in num..num2 {
        for j in num3..num4 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            let kind = t.kind;
            if !t.active() || !matches!(kind, 135 | 210 | 443 | 442) {
                continue;
            }
            let vector_x = i as f32 * 16.0;
            let vector_y = j as f32 * 16.0 + 12.0;
            let mut flag = false;
            if kind == 442 {
                if obj_type == 4 {
                    // `:3270-3300`: the four projectile-pad frames' rects.
                    let (r1x, r1y, r1w, r1h) = match t.frame_x / 22 {
                        0 => (i as f32 * 16.0, j as f32 * 16.0 + 6.0, 16.0, 10.0),
                        1 => (i as f32 * 16.0, j as f32 * 16.0, 16.0, 10.0),
                        2 => (i as f32 * 16.0, j as f32 * 16.0, 10.0, 16.0),
                        3 => (i as f32 * 16.0 + 6.0, j as f32 * 16.0, 10.0, 16.0),
                        // The C#'s switch has no default arm: the rect stays
                        // empty and the intersect test fails.
                        _ => (0.0, 0.0, 0.0, 0.0),
                    };
                    if float_intersect(
                        r1x, r1y, r1w, r1h, pos_x, pos_y, width as f32, height as f32,
                    ) && !float_intersect(
                        r1x, r1y, r1w, r1h, old_pos_x, old_pos_y, width as f32, height as f32,
                    ) {
                        wiring.hit_switch(world, i, j);
                        return SwitchOutcome {
                            hit: true,
                            switches: vec![(i, j)],
                            ..Default::default()
                        };
                    }
                }
                // The pad never takes the general path below (`:3308-3310`).
                flag = true;
            }
            if flag
                || !(pos_x + width as f32 > vector_x)
                || !(pos_x < vector_x + 16.0)
                || !(pos_y + height as f32 > vector_y)
                || !((pos_y as f64) < (vector_y as f64) + 4.01)
                || (old_pos_x + width as f32 > vector_x
                    && old_pos_x < vector_x + 16.0
                    && old_pos_y + height as f32 > vector_y
                    && (old_pos_y as f64) < (vector_y as f64) + 16.01)
            {
                continue;
            }
            match kind {
                // The land mine explodes through the switch (`:3314-3319`).
                210 => {
                    wiring.hit_switch(world, i, j);
                    switches.push((i, j)); // `^:3318`
                    continue;
                }
                // The geyser trap only answers the player and the NPCs
                // (`:3320-3326`).
                443 => {
                    if obj_type == 1 || obj_type == 5 {
                        wiring.hit_switch(world, i, j);
                        switches.push((i, j)); // `^:3324`
                    }
                    continue;
                }
                _ => {}
            }
            // `:3328-3337`: the pressure plate's frame row picks who may
            // trigger it — the rows 2/3/4/6/7 answer only the player, and row
            // 5 answers nobody.
            let num5 = t.frame_y / 18;
            let mut flag2 = true;
            if matches!(num5, 4 | 2 | 3 | 6 | 7) && obj_type != 5 {
                flag2 = false;
            }
            if num5 == 5 && (obj_type == 1 || obj_type == 4 || obj_type == 5) {
                flag2 = false;
            }
            if !flag2 {
                continue;
            }
            wiring.hit_switch(world, i, j);
            switches.push((i, j)); // `^:3347`
            let mut outcome = SwitchOutcome {
                hit: true,
                switches,
                ..Default::default()
            };
            if num5 == 7 {
                // `:3348-3352`: the row-7 plate breaks under the touch. The
                // C#'s `WorldGen.KillTile` drops the plate's item - the port's
                // core has no item pool, so the drop travels back to the app.
                let frame_y = world.tile(i, j).map_or(0, |t| t.frame_y);
                outcome.dropped = crate::registry::tile_drop(kind, frame_y).map(|item| (item, i, j));
                tr_core::edit::kill_tile(world, i, j, false);
            }
            return outcome;
        }
    }
    SwitchOutcome {
        switches,
        ..Default::default()
    }
}

/// `Collision.GetWaterLine` (`Collision.cs:1498-1544`): the water surface's
/// height in the column `x`, or `None` when the column holds no water. The
/// scan looks at the four rows around `y` (`:1521-1543`) and the height is
/// the liquid amount's shortfall from the row's bottom
/// (`waterLineHeight -= liquid / 16`, the C#'s integer division).
pub fn get_water_line(world: &World, x: i32, y: i32) -> Option<f32> {
    // `WorldGen.InWorld(X, Y, 10)` (`:1501`).
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return None;
    }
    let liquid_at = |row: i32| -> u8 {
        world
            .tile(x, row)
            .map_or(0, |t| t.liquid)
    };
    if liquid_at(y - 2) > 0 {
        return None;
    }
    if liquid_at(y - 1) > 0 {
        let l = liquid_at(y - 1);
        return Some(y as f32 * 16.0 - (l / 16) as f32);
    }
    if liquid_at(y) > 0 {
        let l = liquid_at(y);
        return Some((y + 1) as f32 * 16.0 - (l / 16) as f32);
    }
    if liquid_at(y + 1) > 0 {
        let l = liquid_at(y + 1);
        return Some((y + 2) as f32 * 16.0 - (l / 16) as f32);
    }
    None
}

/// `Collision.GetWaterLineIterate` (`Collision.cs:1551-1570`): the water
/// line of the column's surface — the walk up to the first dry row and the
/// shortfall of the row below it.
pub fn get_water_line_iterate(world: &World, x: i32, y: i32) -> Option<f32> {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return None;
    }
    let mut y = y;
    while y > 0 && world.tile(x, y).map_or(0, |t| t.liquid) > 0 {
        y -= 1;
    }
    y += 1;
    let t = world.tile(x, y)?;
    if t.liquid > 0 {
        let below = world.tile(x, y - 1).map_or(0, |t| t.liquid);
        return Some(y as f32 * 16.0 - (below / 16) as f32);
    }
    None
}

/// `Collision.StickyTiles` (`Collision.cs:3362-3435`): the sticky-surface probe
/// under the player's feet. Returns the cobweb (51) or honey-block (229) cell
/// the hitbox overlaps, plus whether the C#'s inline type-30 web puff fires
/// (`:3401`/`:3425`) — the caller owns `Main.dust`, so it queues that puff.
/// `None` is "no sticky tile" (the C#'s `(-1,-1)`).
///
/// Two arms, exactly as the source splits them:
/// * type **51** (cobweb) — any slope, the `num5 = 0` margin at `:3396`;
/// * type **229** (honey block) — `slope() == 0` only (`:3410`), the `num6 = 1`
///   margin and the half-brick's 8 px shrunken box (`:3414-3422`).
///
/// The `Next(30)` puff roll is drawn here, in the source's own order — before
/// `Player.StickyMovement`'s `Next(20,100)` break roll (`Player.cs:22946`).
/// The type-153 honey dust (`Player.cs:23030-23081`) stays the caller's.
pub fn sticky_tiles(
    world: &World,
    pos_x: f32,
    pos_y: f32,
    width: f32,
    height: f32,
    vel_x: f32,
    vel_y: f32,
    rng: &mut dyn tr_core::loot::DropRng,
) -> Option<(i32, i32, bool)> {
    // `:3365-3368`: the ±1 margin scan window.
    let mut num = (pos_x / 16.0) as i32 - 1;
    let mut num2 = ((pos_x + width) / 16.0) as i32 + 2;
    let mut num3 = (pos_y / 16.0) as i32 - 1;
    let mut num4 = ((pos_y + height) / 16.0) as i32 + 2;
    // `:3369-3384`: the clamp — the bottom is `maxTilesY - 40`.
    if num < 0 {
        num = 0;
    }
    if num2 > world.width {
        num2 = world.width;
    }
    if num3 < 0 {
        num3 = 0;
    }
    if num4 > world.height - 40 {
        num4 = world.height - 40;
    }
    for i in num..num2 {
        for j in num3..num4 {
            let Some(t) = world.tile(i, j) else { continue };
            // `:3390`: null/inactive/actuated tiles are skipped.
            if !t.active() || t.flags & tr_core::tile::FLAG_INACTIVE != 0 {
                continue;
            }
            let tx = (i * 16) as f32;
            let ty = (j * 16) as f32;
            if t.kind == 51 {
                // `:3396-3406`: the cobweb's zero margin, full 16.01 height.
                let num5 = 0.0f32;
                if pos_x + width > tx - num5
                    && pos_x < tx + 16.0 + num5
                    && pos_y + height > ty
                    && (pos_y as f64) < (ty as f64) + 16.01
                {
                    // `:3401`: the type-30 web puff, drawn before the return.
                    let puff = (vel_x.abs() + vel_y.abs()) > 0.7 && rng.next_range(0, 30) == 0;
                    return Some((i, j, puff));
                }
            } else {
                // `:3410`: only the flat honey block counts.
                if t.kind != 229 || t.slope() != 0 {
                    continue;
                }
                // `:3414-3423`: the one-pixel margin, the half-brick's 8 px shrink.
                let num6 = 1.0f32;
                let mut ty2 = ty;
                let mut num7 = 16.01f32;
                if t.half_brick() {
                    ty2 += 8.0;
                    num7 -= 8.0;
                }
                if pos_x + width > tx - num6
                    && pos_x < tx + 16.0 + num6
                    && pos_y + height > ty2
                    && pos_y < ty2 + num7
                {
                    // `:3425`: the same `if` in the honey arm tests `type == 51`
                    // — always false here, and C# `&&` short-circuits, so no
                    // `Next(30)` is drawn. Keep the dead test's semantics.
                    return Some((i, j, false));
                }
            }
        }
    }
    // `:3434`: `new Vector2(-1f, -1f)`.
    None
}

// ---------------------------------------------------------------------------
// Conveyor belts (S1-E-13e): `Collision.BuildTileContacts` +
// `Collision.ApplyConveyorBeltMovementToVelocity` +
// `Collision.StepConveyorBelt` (`Collision.cs:2135-2290`, `:3918-4188`,
// `:4190-4320`) and the three geometry helpers they lean on
// (`Collision.GetEntityEdgeTiles` `:3866-3916`, `Collision.CheckAABBvLineCollision2`
// `:263-271`, `Utils.RectangleLineCollision` `Utils.cs:1894-1918`).
// ---------------------------------------------------------------------------

/// `Collision.TileContactSide` (`Collision.cs:11-19`). The variant order is
/// the C#'s enum order - nothing serialises it, but keep it aligned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileContactSide {
    Left,
    Right,
    Top,
    Bottom,
    BottomLeft,
    BottomRight,
}

/// `Collision.TileContact` (`Collision.cs:21-34`): one touching tile face.
#[derive(Clone, Copy, Debug)]
pub struct TileContact {
    pub side: TileContactSide,
    /// The overlap in pixels, already floored by the source's `(int)` cast.
    pub overlap: i32,
    pub x: i32,
    pub y: i32,
    pub slope: i32,
    pub kind: u16,
}

/// `Collision.BuildTileContacts` (`Collision.cs:2135-2290`).
///
/// The entity box is the C#'s `Width`/`Height`; the source's own `+1`/`+3`
/// margins, the `0.1f` touch epsilon and the slope-triangle arms are ported
/// verbatim - including the `(int)` truncations of the overlap.
pub fn build_tile_contacts(
    world: &World,
    pos: [f32; 2],
    width: i32,
    height: i32,
    contacts: &mut Vec<TileContact>,
) {
    // `:2137-2138`: `Position.X = (int)Position.X` - truncation, not rounding.
    let pos_x = pos[0] as i32 as f32;
    let pos_y = pos[1] as i32 as f32;
    contacts.clear();
    // `:2140-2147`: the scan window and its clamps.
    let value = ((pos_x - 1.0) / 16.0) as i32 - 1;
    let value2 = ((pos_x + 1.0 + width as f32) / 16.0) as i32 + 1;
    let value3 = ((pos_y - 1.0) / 16.0) as i32 - 1;
    let value4 = ((pos_y + 3.0 + height as f32) / 16.0) as i32 + 1;
    let num = value.clamp(0, world.width - 1);
    let value2 = value2.clamp(0, world.width - 1);
    let bottom = scan_bottom(world);
    let value3 = value3.clamp(0, (bottom - 1).max(0));
    let value4 = value4.clamp(0, (bottom - 1).max(0));
    for i in num..value2 {
        for j in value3..value4 {
            // `:2153-2157`: null / inactive / actuated are skipped.
            let Some(t) = world.tile(i, j) else { continue };
            if !t.active() || t.inactive() {
                continue;
            }
            let kind = t.kind;
            // `:2159-2167`: the solid gate, with the solid-top `frameY == 0` arm.
            let mut flag = (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind);
            if (kind as usize) < td::TILE_COUNT && td::TILE_SOLID_TOP.has(kind) && t.frame_y == 0 {
                flag = true;
            }
            if !flag {
                continue;
            }
            let vx = i as f32 * 16.0;
            let mut vy = j as f32 * 16.0;
            let mut num2 = 16;
            if t.half_brick() {
                vy += 8.0;
                num2 -= 8;
            }
            let b = t.slope() as i32;
            // `:2177-2184`: the two vertical faces.
            if (pos_x - (vx + 16.0)).abs() < 0.1
                && pos_y + height as f32 > vy
                && pos_y < vy + num2 as f32
                && b != 3
                && b != 1
            {
                let ov = ((pos_y + height as f32).min(vy + num2 as f32) - pos_y.max(vy) + 0.5)
                    .max(1.0);
                contacts.push(TileContact {
                    side: TileContactSide::Left,
                    overlap: ov as i32,
                    x: i,
                    y: j,
                    slope: b,
                    kind,
                });
            }
            if (pos_x + width as f32 - vx).abs() < 0.1
                && pos_y + height as f32 > vy
                && pos_y < vy + num2 as f32
                && b != 4
                && b != 2
            {
                let ov = ((pos_y + height as f32).min(vy + num2 as f32) - pos_y.max(vy) + 0.5)
                    .max(1.0);
                contacts.push(TileContact {
                    side: TileContactSide::Right,
                    overlap: ov as i32,
                    x: i,
                    y: j,
                    slope: b,
                    kind,
                });
            }
            // `:2185-2188`: the vertical zone, or nothing below.
            if !(pos_y + 3.0 + height as f32 > vy) || !(pos_y - 1.0 < vy + num2 as f32) {
                continue;
            }
            if pos_x + width as f32 > vx && pos_x < vx + 16.0 {
                // `:2191`: the horizontal overlap, floored.
                let num3 =
                    ((pos_x + width as f32).min(vx + 16.0) - pos_x.max(vx) + 0.5).max(1.0) as i32;
                // `:2192-2258`: the per-slope top/bottom arms.
                match b {
                    0 => {
                        if (pos_y - (vy + num2 as f32)).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Top,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                        if (pos_y + height as f32 - vy).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Bottom,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    1 => {
                        if (pos_y - (vy + num2 as f32)).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Top,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                        let num7 = (pos_x - vx).max(0.0);
                        let num8 = pos_y + height as f32;
                        if num8 - vy > -0.1 && num8 - (vy + num7) < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Bottom,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    2 => {
                        if (pos_y - (vy + num2 as f32)).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Top,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                        let num5 = (vx + 16.0 - (pos_x + width as f32)).max(0.0);
                        let num6 = pos_y + height as f32;
                        if num6 - vy > -0.1 && num6 - (vy + num5) < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Bottom,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    3 => {
                        let num9 = (pos_x - vx).max(0.0);
                        if (pos_y - (vy + num2 as f32 - num9)).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Top,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                        if (pos_y + height as f32 - vy).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Bottom,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    4 => {
                        let num4 = (vx + 16.0 - (pos_x + width as f32)).max(0.0);
                        if (pos_y - (vy + num2 as f32 - num4)).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Top,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                        if (pos_y + height as f32 - vy).abs() < 0.1 {
                            contacts.push(TileContact {
                                side: TileContactSide::Bottom,
                                overlap: num3,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    _ => {}
                }
            } else {
                // `:2260-2287`: the corner arms - the box hangs off the side.
                if !(pos_x + 3.0 + width as f32 > vx)
                    || !(pos_x - 3.0 < vx + 16.0)
                    || !(pos_y < vy)
                {
                    continue;
                }
                let side = if vx < pos_x {
                    TileContactSide::BottomLeft
                } else {
                    TileContactSide::BottomRight
                };
                match b {
                    0 | 3 | 4 => contacts.push(TileContact {
                        side,
                        overlap: 0,
                        x: i,
                        y: j,
                        slope: b,
                        kind,
                    }),
                    1 => {
                        if side == TileContactSide::BottomRight {
                            contacts.push(TileContact {
                                side,
                                overlap: 0,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    2 => {
                        if side == TileContactSide::BottomLeft {
                            contacts.push(TileContact {
                                side,
                                overlap: 0,
                                x: i,
                                y: j,
                                slope: b,
                                kind,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// `TileContactSide` as `TileID.Sets.ConveyorDirection` reads it - 0 = not a
/// belt, +1 = the left belt (421, pushes one way), -1 = the right (422).
fn conveyor_direction(kind: u16) -> i32 {
    if (kind as usize) < td::TILE_COUNT {
        td::SET_CONVEYOR_DIRECTION[kind as usize]
    } else {
        0
    }
}

/// `Collision.ApplyConveyorBeltMovementToVelocity` (`Collision.cs:3918-4188`):
/// the belt contacts pull the drop's velocity toward the belt's ±2.5 px/tick
/// ramp and snap it onto the belt line. Returns the C#'s `flag` (whether a
/// belt grabbed it).
///
/// `pos` is the box's top-left and `vel` its velocity; the C#'s `item.Bottom` /
/// `item.Right` setters are the same mutation through the box centre, kept
/// inline so no `Entity` type is needed here.
pub fn apply_conveyor_belt_movement_to_velocity(
    world: &World,
    pos: &mut [f32; 2],
    vel: &mut [f32; 2],
    width: i32,
    height: i32,
) -> bool {
    let mut flag = false;
    // ponytail: a fresh Vec per call - belts are a handful of tiles per world;
    // swap in a reusable buffer if a profile ever shows it.
    let mut contacts: Vec<TileContact> = Vec::new();
    build_tile_contacts(world, *pos, width, height, &mut contacts);
    if contacts.is_empty() {
        return flag;
    }
    let mut num = -1i32;
    let mut num2 = -1i32;
    let mut flag2 = false;
    let mut side = TileContactSide::Top;
    let mut side2 = TileContactSide::Top;
    let mut zero = [0.0f32, 0.0f32];
    let mut flag3 = false;
    // `:3930-3932`: `flag4` is *overwritten* by each Top/Bottom contact (last
    // one wins), so it starts false and is assigned, never OR'd.
    let mut flag4 = false;
    let mut flag5 = false;
    for (k, c) in contacts.iter().enumerate() {
        let num3 = conveyor_direction(c.kind);
        match c.side {
            TileContactSide::Left => {
                zero[1] += num3 as f32 * c.overlap as f32;
                flag3 |= num3 != 0;
            }
            TileContactSide::Right => {
                zero[1] += -(num3 as f32) * c.overlap as f32;
                flag3 |= num3 != 0;
            }
            TileContactSide::Top => {
                zero[0] += -(num3 as f32) * c.overlap as f32;
                flag4 = num3 != 0;
            }
            TileContactSide::Bottom => {
                flag5 = true;
                zero[0] += num3 as f32 * c.overlap as f32;
                flag4 = num3 != 0;
                // `:3954-3972`: the neighbour above decides the wall snap.
                if c.slope == 1 {
                    if ((c.x * 16) as f32) < pos[0] {
                        if let Some(t3) = world.tile(c.x, c.y - 1) {
                            if t3.active() && t3.slope() == 3 {
                                num2 = k as i32;
                            }
                        }
                    }
                } else if c.slope == 2 && (c.x * 16 + 16) as f32 > pos[0] + width as f32 {
                    if let Some(t4) = world.tile(c.x, c.y - 1) {
                        if t4.active() && t4.slope() == 4 {
                            num = k as i32;
                        }
                    }
                }
            }
            TileContactSide::BottomLeft => {
                if num3 == -1 {
                    let (x2, y2) = (c.x, c.y);
                    let (ab_active, ab_slope, ab_dir) = match world.tile(x2, y2 - 1) {
                        Some(t) => (t.active(), t.slope(), conveyor_direction(t.kind)),
                        None => (false, 0, 0),
                    };
                    if !ab_active || (ab_slope == 1 && ab_dir == -1) {
                        side = TileContactSide::BottomLeft;
                        flag2 = world.tile(x2, y2).is_some_and(|t| t.half_brick());
                    }
                }
                if num3 == 1 {
                    side2 = TileContactSide::BottomLeft;
                }
            }
            TileContactSide::BottomRight => {
                if num3 == 1 {
                    let (x, y) = (c.x, c.y);
                    let (ab_active, ab_slope, ab_dir) = match world.tile(x, y - 1) {
                        Some(t) => (t.active(), t.slope(), conveyor_direction(t.kind)),
                        None => (false, 0, 0),
                    };
                    if !ab_active || (ab_slope == 2 && ab_dir == 1) {
                        side = TileContactSide::BottomRight;
                        flag2 = world.tile(x, y).is_some_and(|t| t.half_brick());
                    }
                }
                if num3 == -1 {
                    side2 = TileContactSide::BottomRight;
                }
            }
        }
    }
    // `:4012-4063`: the magnitude clamps. The `zero.Y < 0` arm's inner test is
    // `zero.Y < 8f`, which is always true for a negative value - a real vanilla
    // quirk, ported as written.
    if zero[0] < 0.0 {
        zero[0] = if zero[0] < -8.0 {
            -2.5
        } else if zero[0] < -4.0 {
            -1.25
        } else {
            -0.75
        };
    } else if zero[0] > 0.0 {
        zero[0] = if zero[0] > 8.0 {
            2.5
        } else if zero[0] > 4.0 {
            1.25
        } else {
            0.75
        };
    }
    if zero[1] < 0.0 {
        zero[1] = if zero[1] < 8.0 { -2.5 } else { -1.25 };
    } else if zero[1] > 0.0 {
        zero[1] = if zero[1] > 8.0 { 2.5 } else { 1.25 };
    } else if flag3 && vel[1] <= 1.0 {
        vel[1] = 0.0;
    }
    if zero[1] == 0.0 {
        if vel[1] < 0.11 && side != TileContactSide::Top && !flag5 {
            // `:4072-4101`: the corner lift - snap onto the belt's top edge.
            let dx = if side == TileContactSide::BottomLeft {
                Some(-1.25f32)
            } else if side == TileContactSide::BottomRight {
                Some(1.25)
            } else {
                None
            };
            if let Some(vx) = dx {
                flag = true;
                let mut bottom_y = (pos[1] + height as f32) as i32;
                let mut num4 = (bottom_y + 8) / 16 * 16;
                if flag2 {
                    num4 += 8;
                }
                pos[1] = num4 as f32 - height as f32;
                bottom_y = num4;
                let _ = bottom_y;
                vel[1] = 0.0;
                vel[0] = vx;
                zero[0] = if vx < 0.0 { -2.5 } else { 2.5 };
            }
        } else if vel[1] < 0.3 && !flag5 {
            // `:4103-4131`: the side snap onto a belt's face.
            match side2 {
                TileContactSide::BottomRight => {
                    if vel[0] <= 0.75 && vel[0] >= -2.5 && !flag3 {
                        flag = true;
                        let num6 = (pos[0] as i32 + 8) / 16;
                        pos[0] = (num6 * 16) as f32;
                        vel[0] = 0.0;
                        vel[1] = 0.75;
                        zero[1] = 2.5;
                    }
                }
                TileContactSide::BottomLeft => {
                    if vel[0] >= -0.75 && vel[0] <= 2.5 && !flag3 {
                        flag = true;
                        let right = (pos[0] + width as f32) as i32;
                        let right = (right + 8) / 16 * 16;
                        pos[0] = right as f32 - width as f32;
                        vel[0] = 0.0;
                        vel[1] = 0.75;
                        zero[1] = 2.5;
                    }
                }
                _ => {}
            }
        }
    }
    flag |= flag4 | flag3;
    // `:4134-4149`: the ramp. `* 6f / 60f` keeps the source's two-step float
    // rounding rather than folding it to `* 0.1`.
    if zero[1] < 0.0 {
        vel[1] = (vel[1] + zero[1] * 6.0 / 60.0).max(zero[1]);
    } else if zero[1] > 0.0 {
        vel[1] = (vel[1] + zero[1] * 6.0 / 60.0).min(zero[1]);
    }
    if zero[0] < 0.0 && vel[0] > zero[0] {
        vel[0] = (vel[0] + zero[0] * 6.0 / 60.0).max(zero[0]);
    }
    if zero[0] > 0.0 && vel[0] < zero[0] {
        vel[0] = (vel[0] + zero[0] * 6.0 / 60.0).min(zero[0]);
    }
    // `:4150-4185`: the slope-quarter wall snaps.
    if num != -1 && vel[0] > 0.0 {
        let num7 = contacts[num as usize].x * 16 + (16 - height / 2).max(0);
        let right = pos[0] + width as f32;
        if right + vel[0] > num7 as f32 {
            if right >= num7 as f32 {
                pos[0] = num7 as f32 - width as f32;
                vel[0] = 0.0;
            } else {
                vel[0] = num7 as f32 - right;
            }
        }
    }
    if num2 != -1 && vel[0] < 0.0 {
        let num8 = contacts[num2 as usize].x * 16 + (height / 2).min(16);
        if pos[0] + vel[0] < num8 as f32 {
            if pos[0] <= num8 as f32 {
                pos[0] = num8 as f32;
                vel[0] = 0.0;
            } else {
                vel[0] = num8 as f32 - pos[0];
            }
        }
    }
    flag
}

/// `Utils.Between` (`Utils.cs:1718-1725`): the inclusive box test.
fn between(v: [f32; 2], min: [f32; 2], max: [f32; 2]) -> bool {
    v[0] >= min[0] && v[0] <= max[0] && v[1] >= min[1] && v[1] <= max[1]
}

/// `Vector2.ClosestPointOnLine` (`Utils.cs:1860-1873`).
fn closest_point_on_line(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let value = [p[0] - a[0], p[1] - a[1]];
    let v = [b[0] - a[0], b[1] - a[1]];
    let num = v[0] * v[0] + v[1] * v[1];
    let num2 = (value[0] * v[0] + value[1] * v[1]) / num;
    if num2 < 0.0 {
        return a;
    }
    if num2 > 1.0 {
        return b;
    }
    [a[0] + v[0] * num2, a[1] + v[1] * num2]
}

/// `Utils.RectangleLineCollision` (`Utils.cs:1894-1918`).
///
/// The source builds four closest points but the loop tests `array[0]` every
/// iteration - a vanilla bug that makes the other three dead. Ported as
/// written so the belt line tests agree with the original exactly.
fn rectangle_line_collision(
    rect_tl: [f32; 2],
    rect_br: [f32; 2],
    line_start: [f32; 2],
    line_end: [f32; 2],
) -> bool {
    if between(line_start, rect_tl, rect_br) || between(line_end, rect_tl, rect_br) {
        return true;
    }
    let p = [rect_br[0], rect_tl[1]];
    let v = [rect_tl[0], rect_br[1]];
    let a0 = closest_point_on_line(rect_tl, line_start, line_end);
    let _ = closest_point_on_line(p, line_start, line_end);
    let _ = closest_point_on_line(v, line_start, line_end);
    let _ = closest_point_on_line(rect_br, line_start, line_end);
    between(a0, rect_tl, v)
}

/// `Collision.CheckAABBvAABBCollision` (`Collision.cs:184-191`).
fn check_aabb_v_aabb(p1: [f32; 2], d1: [f32; 2], p2: [f32; 2], d2: [f32; 2]) -> bool {
    p1[0] < p2[0] + d2[0] && p1[1] < p2[1] + d2[1] && p1[0] + d1[0] > p2[0] && p1[1] + d1[1] > p2[1]
}

fn rotate(v: [f32; 2], rad: f32) -> [f32; 2] {
    let (s, c) = rad.sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c]
}

/// `Collision.CheckAABBvLineCollision` (`Collision.cs:273-369`) at the
/// caller's line width (the `…2` wrapper passes `0.0001f`; the Moon Lord
/// turret laser's `Colliding` arm passes `30f * scale`,
/// `Projectile.cs:15947-15949`).
pub(crate) fn check_aabb_v_line_collision(
    aabb_pos: [f32; 2],
    aabb_dim: [f32; 2],
    line_start: [f32; 2],
    line_end: [f32; 2],
    line_width: f32,
) -> bool {
    let num = line_width * 0.5;
    let mut position = line_start;
    let mut dimensions = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    if dimensions[0] > 0.0 {
        dimensions[0] += line_width;
        position[0] -= num;
    } else {
        position[0] += dimensions[0] - num;
        dimensions[0] = -dimensions[0] + line_width;
    }
    if dimensions[1] > 0.0 {
        dimensions[1] += line_width;
        position[1] -= num;
    } else {
        position[1] += dimensions[1] - num;
        dimensions[1] = -dimensions[1] + line_width;
    }
    if !check_aabb_v_aabb(aabb_pos, aabb_dim, position, dimensions) {
        return false;
    }
    let v = [aabb_pos[0] - line_start[0], aabb_pos[1] - line_start[1]];
    let sp = [v[0] + aabb_dim[0], v[1] + aabb_dim[1]];
    let sp2 = [v[0], sp[1]];
    let sp3 = [sp[0], v[1]];
    let v2 = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let num2 = (v2[0] * v2[0] + v2[1] * v2[1]).sqrt();
    let num3 = v2[1].atan2(v2[0]);
    let arr = [
        rotate(v, -num3),
        rotate(sp3, -num3),
        rotate(sp, -num3),
        rotate(sp2, -num3),
    ];
    let mut collision_point = num2;
    let mut result = false;
    for a in arr {
        if a[1].abs() < num && a[0] < collision_point && a[0] >= 0.0 {
            collision_point = a[0];
            result = true;
        }
    }
    let vector3 = [0.0f32, num];
    let vector4 = [num2, num];
    let vector5 = [0.0f32, -num];
    let vector6 = [num2, -num];
    for j in 0..arr.len() {
        let num4 = (j + 1) % arr.len();
        let mut vector7 = [vector4[0] - vector3[0], vector4[1] - vector3[1]];
        let vector8 = [arr[num4][0] - arr[j][0], arr[num4][1] - arr[j][1]];
        let mut num5 = vector7[0] * vector8[1] - vector7[1] * vector8[0];
        if num5 != 0.0 {
            let vector9 = [arr[j][0] - vector3[0], arr[j][1] - vector3[1]];
            let num6 = (vector9[0] * vector8[1] - vector9[1] * vector8[0]) / num5;
            if (0.0..=1.0).contains(&num6) {
                let num7 = (vector9[0] * vector7[1] - vector9[1] * vector7[0]) / num5;
                if (0.0..=1.0).contains(&num7) {
                    result = true;
                    collision_point = collision_point.min(vector3[0] + num6 * vector7[0]);
                }
            }
        }
        vector7 = [vector6[0] - vector5[0], vector6[1] - vector5[1]];
        num5 = vector7[0] * vector8[1] - vector7[1] * vector8[0];
        if num5 == 0.0 {
            continue;
        }
        let vector10 = [arr[j][0] - vector5[0], arr[j][1] - vector5[1]];
        let num8 = (vector10[0] * vector8[1] - vector10[1] * vector8[0]) / num5;
        if (0.0..=1.0).contains(&num8) {
            let num9 = (vector10[0] * vector7[1] - vector10[1] * vector7[0]) / num5;
            if (0.0..=1.0).contains(&num9) {
                result = true;
                collision_point = collision_point.min(vector5[0] + num8 * vector7[0]);
            }
        }
    }
    let _ = collision_point;
    result
}

/// `Collision.CheckAABBvLineCollision2` (`Collision.cs:263-271`).
fn check_aabb_v_line_collision2(
    aabb_pos: [f32; 2],
    aabb_dim: [f32; 2],
    line_start: [f32; 2],
    line_end: [f32; 2],
) -> bool {
    if !rectangle_line_collision(aabb_pos, [aabb_pos[0] + aabb_dim[0], aabb_pos[1] + aabb_dim[1]], line_start, line_end)
    {
        return check_aabb_v_line_collision(aabb_pos, aabb_dim, line_start, line_end, 0.0001);
    }
    true
}

/// `Collision.GetEntityEdgeTiles` (`Collision.cs:3866-3916`): the ring of tile
/// coordinates the box's edges sit in. The `% 16 == 0` nudges keep a flush edge
/// from landing in the next tile along.
fn get_entity_edge_tiles(
    pos: [f32; 2],
    width: i32,
    height: i32,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
) -> Vec<(i32, i32)> {
    let mut p = Vec::new();
    let mut num = pos[0] as i32;
    let mut num2 = pos[1] as i32;
    let mut num3 = (pos[0] + width as f32) as i32;
    let mut num4 = (pos[1] + height as f32) as i32;
    if num % 16 == 0 {
        num -= 1;
    }
    if num2 % 16 == 0 {
        num2 -= 1;
    }
    if num3 % 16 == 0 {
        num3 += 1;
    }
    if num4 % 16 == 0 {
        num4 += 1;
    }
    let num5 = num3 / 16 - num / 16;
    let num6 = num4 / 16 - num2 / 16;
    num /= 16;
    num2 /= 16;
    for i in num..=num + num5 {
        if up {
            p.push((i, num2));
        }
        if down {
            p.push((i, num2 + num6));
        }
    }
    for j in num2..num2 + num6 {
        if left {
            p.push((num, j));
        }
        if right {
            p.push((num + num5, j));
        }
    }
    p
}

/// `Collision.StepConveyorBelt` (`Collision.cs:4190-4320`) for a Player: the
/// belt under the feet walks the body along it.
///
/// `pos` is the box top-left and is mutated in place; the C#'s
/// `entity.position += Collision.TileCollision(...)` moves it. `grav_dir` is
/// kept because it scales both the direction sum and the fall probe, though the
/// port never flips gravity (so every caller passes `1.0`).
///
/// ponytail: `artificialRising` has no port caller (it is the golf/reserved
/// path) and is not carried.
pub fn step_conveyor_belt_player(
    world: &World,
    pos: &mut [f32; 2],
    width: i32,
    height: i32,
    grav_dir: f32,
    gfx_off_y: f32,
    grappling: bool,
    pulley: bool,
    on_track: bool,
) {
    // `:4192-4202`: the Player guards and the 5 px shrink.
    if gfx_off_y.abs() > 2.0 || grappling || pulley {
        return;
    }
    let full_height = height;
    let height = height - 5;
    pos[1] += 5.0;
    let mut num = 0i32;
    let mut num2 = 0i32;
    let mut flag = false;
    // `:4206`: the row the on-track skip compares against.
    let num3 = pos[1] as i32 + height;
    let edges = get_entity_edge_tiles(*pos, width, height, false, false, true, true);
    let vector = 0.0001f32;
    for (px, py) in edges {
        // `:4223`: out of world, or the cart rides over the belt.
        if !world.in_bounds(px, py) || (on_track && py < num3) {
            continue;
        }
        let Some(t) = world.tile(px, py) else { continue };
        if !t.active() || t.inactive() {
            continue;
        }
        let num4 = conveyor_direction(t.kind);
        if num4 == 0 {
            continue;
        }
        // `:4237-4268`: the belt's top line (lineStart→lineEnd, the physical
        // surface) and its offset twin (lineStart2→lineEnd2, 8 px up on a
        // half brick).
        let mut ls = [0.0f32; 2];
        let mut ls2 = [0.0f32; 2];
        let mut le = [0.0f32; 2];
        let mut le2 = [0.0f32; 2];
        ls[0] = (px * 16) as f32;
        ls2[0] = ls[0];
        le[0] = (px * 16 + 16) as f32;
        le2[0] = le[0];
        let top = (py * 16) as f32;
        let bot = (py * 16 + 16) as f32;
        match t.slope() {
            1 => {
                ls2[1] = top;
                le2[1] = bot;
                le[1] = bot;
                ls[1] = bot;
            }
            2 => {
                le2[1] = top;
                ls2[1] = bot;
                le[1] = bot;
                ls[1] = bot;
            }
            3 => {
                le[1] = top;
                ls2[1] = top;
                le2[1] = top;
                ls[1] = bot;
            }
            4 => {
                ls[1] = top;
                ls2[1] = top;
                le2[1] = top;
                le[1] = bot;
            }
            _ => {
                if t.half_brick() {
                    ls2[1] = top + 8.0;
                    le2[1] = top + 8.0;
                } else {
                    ls2[1] = top;
                    le2[1] = top;
                }
                ls[1] = bot;
                le[1] = bot;
            }
        }
        let mut num5 = 0i32;
        let aabb_pos = [pos[0] - vector, pos[1] - vector];
        let aabb_dim = [width as f32 + vector * 2.0, height as f32 + vector * 2.0];
        if !((t.kind as usize) < td::TILE_COUNT && td::SET_PLATFORMS.has(t.kind))
            && check_aabb_v_line_collision2(aabb_pos, aabb_dim, ls, le)
        {
            num5 -= 1;
        }
        if check_aabb_v_line_collision2(aabb_pos, aabb_dim, ls2, le2) {
            num5 += 1;
        }
        if num5 != 0 {
            flag = true;
            num += num4 * num5 * grav_dir as i32;
            // `:4282-4289`: the slope sides also push along the tile edge
            // (`leftSlope` = 2|4, `rightSlope` = 1|3).
            let s = t.slope();
            if s == 2 || s == 4 {
                num2 += grav_dir as i32 * -num4;
            }
            if s == 1 || s == 3 {
                num2 -= grav_dir as i32 * -num4;
            }
        }
    }
    // `:4292-4296`: the height comes back before the movement probes.
    pos[1] -= 5.0;
    let height = full_height;
    if !flag {
        return;
    }
    if num != 0 {
        let num = num.signum();
        let num2 = num2.signum();
        // `:4309-4312`: the diagonal step, then the fall probe.
        let dir = [num as f32 * grav_dir, num2 as f32];
        let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
        let vel = [dir[0] / len * 2.5, dir[1] / len * 2.5];
        let tc = tile_collision(world, *pos, vel, width as f32, height as f32, false, false);
        pos[0] += tc.vel[0];
        pos[1] += tc.vel[1];
        let tc = tile_collision(
            world,
            *pos,
            [0.0, 2.5 * grav_dir],
            width as f32,
            height as f32,
            false,
            false,
        );
        pos[0] += tc.vel[0];
        pos[1] += tc.vel[1];
    }
}

/// `Collision.noSlopeCollision` (`Collision.cs:2033-2133`): `TileCollision`
/// with the slope gates removed — the caller is on a slope and must not have
/// the ramp refuse the wall. The up arm keeps the `+0.01`, the down arm has no
/// `gravDir` nudge (`:2088` vs `:2124`).
// ponytail: its caller is `NPC.Collision_MoveBlazingWheel`
// (`NPC.cs:92775-92783`), ported by `npc_ai_style21.rs` (the snail family's
// `Collision_MoveSnailOnSlopes` uses `SlopeCollision` instead, `:92704-92721`).
#[allow(dead_code)]
pub fn no_slope_collision(
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
    let mut num = -1i32;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let mut num4 = -1i32;
    // `:2041-2052`: the scan window and the four clamps.
    let cx0 = ((pos[0] / TILE_PX) as i32 - 1).clamp(0, world.width - 1);
    let cx1 = (((pos[0] + width) / TILE_PX) as i32 + 2).clamp(0, world.width - 1);
    let bottom = scan_bottom(world);
    let cy0 = ((pos[1] / TILE_PX) as i32 - 1).clamp(0, bottom);
    let cy1 = (((pos[1] + height) / TILE_PX) as i32 + 2).clamp(0, bottom);
    let mut num6 = (cy1 + 3) as f32 * TILE_PX;
    for i in cx0..cx1 {
        for j in cy0..cy1 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            // `:2059`: the solid gate — no slope test at all.
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
            let solid_top = td::TILE_SOLID_TOP.has(t.kind);
            if pos[1] + height <= ry {
                down = true;
                if (!(solid_top && fall_through) || !((vel[1] <= 1.0) | fall2)) && num6 > ry {
                    num3 = i;
                    num4 = j;
                    if rh < TILE_PX {
                        num4 += 1;
                    }
                    if num3 != num {
                        result[1] = ry - (pos[1] + height);
                        num6 = ry;
                    }
                }
            } else if pos[0] + width <= rx && !solid_top {
                num = i;
                num2 = j;
                if num2 != num4 {
                    result[0] = rx - (pos[0] + width);
                }
                if num3 == num {
                    result[1] = vel[1];
                }
            } else if pos[0] >= rx + TILE_PX && !solid_top {
                num = i;
                num2 = j;
                if num2 != num4 {
                    result[0] = rx + TILE_PX - pos[0];
                }
                if num3 == num {
                    result[1] = vel[1];
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

/// `Collision.AdvancedTileCollision` (`Collision.cs:4422-4566`): the sandshark
/// rule set — slope gates kept, `forcedIgnoredTiles` skipped (`:4449`), the
/// neighbour-slope exemption on the side arms (`:4514-4518`, `:4534-4538`) and
/// the `gravDir`-dependent `0.01` nudges (`:4507`, `:4557`).
// ponytail: its only caller is `NPC.Collision_MoveSandshark`
// (`NPC.cs:92732`), and the sandshark family is not ported yet.
#[allow(dead_code)]
pub fn advanced_tile_collision(
    world: &World,
    forced_ignored: &td::TileSet,
    pos: [f32; 2],
    vel: [f32; 2],
    width: f32,
    height: f32,
    fall_through: bool,
    fall2: bool,
    grav_dir: i32,
) -> TileCollision {
    let mut up = false;
    let mut down = false;
    let mut result = vel;
    let target = [pos[0] + vel[0], pos[1] + vel[1]];
    let mut num = -1i32;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let mut num4 = -1i32;
    let cx0 = ((pos[0] / TILE_PX) as i32 - 1).clamp(0, world.width - 1);
    let cx1 = (((pos[0] + width) / TILE_PX) as i32 + 2).clamp(0, world.width - 1);
    let bottom = scan_bottom(world);
    let cy0 = ((pos[1] / TILE_PX) as i32 - 1).clamp(0, bottom);
    let cy1 = (((pos[1] + height) / TILE_PX) as i32 + 2).clamp(0, bottom);
    let mut num6 = (cy1 + 3) as f32 * TILE_PX;
    for i in cx0..cx1 {
        for j in cy0..cy1 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            // `:4449`: the ignored-set skip sits in the same conjunct as the
            // solid gate.
            if forced_ignored.has(t.kind) || !solid(&t) {
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
            let mut flag = false;
            let mut flag2 = false;
            let s = t.slope();
            if s > 2 {
                if s == 3 && pos[1] + vel[0].abs() >= ry && pos[0] >= rx {
                    flag2 = true;
                }
                if s == 4 && pos[1] + vel[0].abs() >= ry && pos[0] + width <= rx + TILE_PX {
                    flag2 = true;
                }
            } else if s > 0 {
                flag = true;
                if s == 1 && pos[1] + height - vel[0].abs() <= ry + rh && pos[0] >= rx {
                    flag2 = true;
                }
                if s == 2
                    && pos[1] + height - vel[0].abs() <= ry + rh
                    && pos[0] + width <= rx + TILE_PX
                {
                    flag2 = true;
                }
            }
            if flag2 {
                continue;
            }
            let solid_top = td::TILE_SOLID_TOP.has(t.kind);
            if pos[1] + height <= ry {
                down = true;
                // `:4497-4508`: the roof arm, minus the `!flag` slope guard's
                // own `gravDir == -1` nudge.
                if (!(solid_top && fall_through) || !((vel[1] <= 1.0) | fall2)) && num6 > ry {
                    num3 = i;
                    num4 = j;
                    if rh < TILE_PX {
                        num4 += 1;
                    }
                    if num3 != num && !flag {
                        result[1] = ry - (pos[1] + height) + if grav_dir == -1 { -0.01 } else { 0.0 };
                        num6 = ry;
                    }
                }
            } else if pos[0] + width <= rx && !solid_top {
                // `:4514-4530`: skip when the left neighbour is a right-hand ramp.
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
                // `:4534-4550`: skip when the right neighbour is a left ramp.
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
                result[1] = ry + rh - pos[1] + if grav_dir == 1 { 0.01 } else { 0.0 };
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

/// `Collision.HitTiles` (`Collision.cs:3063-3108`) / `HitTilesInACircle`
/// (`:3016-3061`): the swept box's solid overlap, inclusive on both ends
/// (unlike `TileCollision`'s strict comparisons). The C# answers every hit
/// with `WorldGen.KillTile(fail: true, effectOnly: true)`, which returns at
/// `WorldGen.cs:63704-63707` after the dust loop — no tile is removed. The
/// port has no tile-dust layer, so the scan returns the hit cells and
/// [`tile_dust_amount`] supplies the counts the C# would spawn.
///
/// `circle` is the `HitTilesInACircle` arm: the centre-distance gate
/// `(Width + Height) / 4` at `:3054`.
fn hit_tiles_scan(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: i32,
    height: i32,
    circle: bool,
) -> Vec<(i32, i32)> {
    let target = [pos[0] + vel[0], pos[1] + vel[1]];
    let mut num = (pos[0] / TILE_PX) as i32 - 1;
    let mut num2 = ((pos[0] + width as f32) / TILE_PX) as i32 + 2;
    let mut num3 = (pos[1] / TILE_PX) as i32 - 1;
    let mut num4 = ((pos[1] + height as f32) / TILE_PX) as i32 + 2;
    // `:3023-3038` / `:3070-3085`: the four clamps — note `num2` clamps to
    // `maxTilesX`, not `maxTilesX - 1`.
    if num < 0 {
        num = 0;
    }
    if num2 > world.width {
        num2 = world.width;
    }
    if num3 < 0 {
        num3 = 0;
    }
    let bottom = scan_bottom(world);
    if num4 > bottom {
        num4 = bottom;
    }
    let (w, h) = (width as f32, height as f32);
    let centre = [target[0] + (width / 2) as f32, target[1] + (height / 2) as f32];
    let radius = ((width + height) / 4) as f32;
    let mut hits = Vec::new();
    for i in num..num2 {
        for j in num3..num4 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            // `:3044`/`:3091`: active, not actuated, solid or a platform top.
            if !t.active()
                || t.flags & tr_core::tile::FLAG_INACTIVE != 0
                || !((td::TILE_SOLID.has(t.kind) && t.kind != 192)
                    || (td::TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0))
            {
                continue;
            }
            let mut ty = j as f32 * TILE_PX;
            let mut rh = TILE_PX;
            if t.half_brick() {
                ty += 8.0;
                rh -= 8.0;
            }
            let tx = i as f32 * TILE_PX;
            if target[0] + w >= tx
                && target[0] <= tx + TILE_PX
                && target[1] + h >= ty
                && target[1] <= ty + rh
            {
                if circle {
                    let dx = centre[0] - (tx + 8.0);
                    let dy = centre[1] - (ty + 8.0);
                    if (dx * dx + dy * dy).sqrt() >= radius {
                        continue;
                    }
                }
                hits.push((i, j));
            }
        }
    }
    hits
}

/// `Collision.HitTiles` (`Collision.cs:3063-3108`).
// ponytail: the ~30 `Projectile.cs` call sites have no generic
// projectile-to-tile seam in the port (ANALYSIS, E-13).
#[allow(dead_code)]
pub fn hit_tiles(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: i32,
    height: i32,
) -> Vec<(i32, i32)> {
    hit_tiles_scan(world, pos, vel, width, height, false)
}

/// `Collision.HitTilesInACircle` (`Collision.cs:3016-3061`).
#[allow(dead_code)]
pub fn hit_tiles_in_a_circle(
    world: &World,
    pos: [f32; 2],
    vel: [f32; 2],
    width: i32,
    height: i32,
) -> Vec<(i32, i32)> {
    hit_tiles_scan(world, pos, vel, width, height, true)
}

/// `WorldGen.KillTile_GetTileDustAmount` (`WorldGen.cs:66597-66685`): the dust
/// count `KillTile` spawns for a hit cell, which is the whole observable effect
/// of `HitTiles`'s `effectOnly: true` call.
#[allow(dead_code)]
pub fn tile_dust_amount(kind: u16, fail: bool, frame_x: i16) -> i32 {
    match kind {
        // `:66607-66611`
        654 | 630 | 631 => 0,
        // `:66612-66617`
        231 => {
            if fail {
                3
            } else {
                6
            }
        }
        // `:66618-66621`
        481 | 482 | 483 => 3,
        // `:66622-66625`
        541 => 1,
        549 => 2,
        // `:66626-66663`: the whole no-dust list.
        125 | 138 | 172 | 287 | 300 | 301 | 302 | 303 | 304 | 305 | 306 | 307 | 308 | 354 | 355 | 373
        | 374 | 375 | 376 | 461 | 484 | 499 | 564 | 565 | 593 | 594 | 617 | 664 | 665 | 699 | 709 | 711
        | 712 | 713 | 714 | 715 | 716 => 0,
        // `:66664-66673`
        534 | 536 | 539 | 625 | 627 => {
            if fail {
                3
            } else {
                5
            }
        }
        // `:66674-66683`: the gem tree's frame decides.
        184 => {
            if frame_x / 22 >= 5 {
                5
            } else if fail {
                3
            } else {
                10
            }
        }
        // `:66601-66606`
        _ => {
            if fail {
                3
            } else {
                10
            }
        }
    }
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

    /// The head probe's geometry (`Collision.cs:1385-1400`): the 10x12 box
    /// at the player's head (the gravity -1 case).
    fn head_probe(pos_x: f32, pos_y: f32) -> (f32, f32) {
        let width = 20;
        let height = 42;
        let num = 10.min(width);
        let num2 = 12.min(height);
        let vx = pos_x + (width / 2) as f32 - (num / 2) as f32;
        let vy = pos_y - 2.0 + (height / 2) as f32 - 6.0;
        (vx, vy)
    }

    fn put_water(w: &mut World, x: i32, y: i32, liquid: u8, kind: tr_core::tile::LiquidKind) {
        let i = w.index(x, y);
        w.tiles[i].liquid = liquid;
        w.tiles[i].set_liquid_kind(kind);
    }

    /// `Collision.DrownCollision` (`:1383-1432`): the head under the water
    /// drowns, the head above it does not.
    #[test]
    fn the_drown_probe_tests_the_head_only() {
        use tr_core::tile::LiquidKind;
        let mut w = world();
        put_water(&mut w, 10, 7, 255, LiquidKind::Water);
        // The player's head (the probe at [155, 165] x [113, 125]) is under
        // the water of row 7 (112..128).
        assert!(
            drown_collision(&w, 150.0, 100.0, 20, 42, -1.0, false),
            "the head under the water drowns"
        );
        // The same player 40 px higher: the head is above the surface.
        assert!(
            !drown_collision(&w, 150.0, 60.0, 20, 42, -1.0, false),
            "the head above the water does not drown"
        );
        // The probe's own geometry — the C#'s centre column, 10 px wide.
        let (vx, vy) = head_probe(150.0, 100.0);
        assert_eq!(vx, 155.0);
        assert_eq!(vy, 113.0);
    }

    /// The liquid's surface offset (`:1421-1423`): a half-full tile only
    /// drowns when the head reaches its lower half.
    #[test]
    fn the_drown_probe_follows_the_liquid_surface() {
        use tr_core::tile::LiquidKind;
        let mut w = world();
        // Half full (128): the water fills the tile's lower 8 px (120..128).
        put_water(&mut w, 10, 7, 128, LiquidKind::Water);
        assert!(
            drown_collision(&w, 150.0, 100.0, 20, 42, -1.0, false),
            "the probe reaches the lower half"
        );
        // The probe ends at 125 in the case above; 8 px higher it ends at
        // 117 and stays clear of the surface.
        assert!(
            !drown_collision(&w, 150.0, 92.0, 20, 42, -1.0, false),
            "the probe stays above the surface"
        );
    }

    /// The liquid kinds (`:1416`): the lava and the shimmer do not drown, the
    /// honey does.
    #[test]
    fn only_the_water_and_the_honey_drown() {
        use tr_core::tile::LiquidKind;
        for (kind, drowns) in [
            (LiquidKind::Water, true),
            (LiquidKind::Honey, true),
            (LiquidKind::Lava, false),
            (LiquidKind::Shimmer, false),
        ] {
            let mut w = world();
            put_water(&mut w, 10, 7, 255, kind);
            assert_eq!(
                drown_collision(&w, 150.0, 100.0, 20, 42, -1.0, false),
                drowns,
                "{kind:?}"
            );
        }
    }

    /// The solid-floor exclusion (`:1409`/`:1416`): the water inside a solid
    /// block on the gravity-side row does not drown — unless the sloped
    /// blocks count as solid (`includeSlopes`).
    ///
    /// The gravity-side row is `value3` for the vanilla `gravDir = 1` probe
    /// (the player's and the NPCs' calls) and `value4 - 1` for the `-1`
    /// default (the flipped-gravity arm) — the lowered `-1` probe never
    /// reaches its own gravity-side row, so the arm is exercised through the
    /// `1` probe.
    #[test]
    fn the_solid_floor_excludes_the_water_under_it() {
        use tr_core::tile::LiquidKind;
        let mut w = world();
        // The vanilla `1` probe at y = 16 reaches row 0 (the scan's first row).
        put(&mut w, 10, 0, 1);
        put_water(&mut w, 10, 0, 255, LiquidKind::Water);
        assert!(
            !drown_collision(&w, 150.0, 16.0, 20, 42, 1.0, false),
            "the solid block shields the water"
        );
        assert!(
            !drown_collision(&w, 150.0, 16.0, 20, 42, 1.0, true),
            "a full block is a floor in both modes"
        );
        // The half brick's `blockType` is 1 (`Tile.cs:202-214`), so the
        // `includeSlopes` mode stops treating it as a floor.
        let mut w = world();
        put(&mut w, 10, 0, 1);
        let hi = w.index(10, 0);
        w.tiles[hi].flags |= tr_core::tile::FLAG_HALF_BRICK;
        put_water(&mut w, 10, 0, 255, LiquidKind::Water);
        assert!(
            !drown_collision(&w, 150.0, 16.0, 20, 42, 1.0, false),
            "the half brick is a floor too"
        );
        assert!(
            drown_collision(&w, 150.0, 16.0, 20, 42, 1.0, true),
            "the sloped blocks are not a floor"
        );
    }

    /// The switch-tile bridge (`Collision.cs:3227-3360`): a world with one
    /// switch tile of the given kind and frame row.
    fn switch_world(kind: u16, frame_y: i16) -> World {
        let mut w = world();
        let i = w.index(10, 20);
        w.tiles[i].kind = kind;
        w.tiles[i].frame_y = frame_y;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    /// The player walks onto the pressure plate: the switch trips
    /// (`Collision.cs:3310`'s "moved onto it" test).
    #[test]
    fn the_player_walking_onto_a_pressure_plate_trips_it() {
        let mut w = switch_world(135, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        // The player's box [168, 188] x [300, 342]; the plate's hit line is
        // at 20 * 16 + 12 = 332 and its old box was above it.
        let outcome = switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5);
        assert!(outcome.hit, "the plate trips");
    }

    /// The player who already stands on the plate does not re-trip it
    /// (`Collision.cs:3310`'s old-position arm).
    #[test]
    fn standing_on_the_plate_does_not_trip_it_again() {
        let mut w = switch_world(135, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        let outcome = switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 300.0, 20, 42, 5);
        assert!(!outcome.hit, "the old position was over the plate");
    }

    /// The pressure plate's frame row picks who may trigger it
    /// (`Collision.cs:3328-3337`): the rows 2/3/4/6/7 answer only the
    /// player, and row 5 answers nobody.
    #[test]
    fn the_plate_rows_pick_who_may_trigger_them() {
        // Row 2: the player trips it, the NPC does not.
        let mut w = switch_world(135, 2 * 18);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5).hit,
            "the player trips the row-2 plate"
        );
        let mut w = switch_world(135, 2 * 18);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            !switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 1).hit,
            "the NPC does not trip the row-2 plate"
        );
        // Row 5: nobody answers it.
        let mut w = switch_world(135, 5 * 18);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            !switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5).hit,
            "the row-5 plate answers nobody"
        );
    }

    /// The NPC's call site (`NPC.cs:90504`, the `objType` 2 call): the
    /// row-0 plate answers every entity.
    #[test]
    fn the_npc_call_trips_the_row_zero_plate() {
        let mut w = switch_world(135, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 2).hit,
            "the NPC trips the plate"
        );
    }

    /// The land mine (210) answers every entity and explodes through the
    /// switch (`Collision.cs:3314-3319` → `Wiring.HitSwitch` → the mine's
    /// shot).
    #[test]
    fn the_land_mine_explodes_when_an_entity_reaches_it() {
        let mut w = switch_world(210, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        let outcome = switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5);
        // The C#'s mine arm `continue`s the scan (`Collision.cs:3319`), so the
        // scan's answer stays false — only the pressure plates consume it.
        assert!(!outcome.hit, "the mine does not consume the scan");
        let shots = wiring.take_device_shots();
        assert_eq!(shots.len(), 1, "the mine fires once");
        assert_eq!(shots[0].kind, 164, "the mine's explosion projectile");
    }

    /// The geyser trap (443) answers the player and the NPCs only
    /// (`Collision.cs:3320-3326`).
    #[test]
    fn the_geyser_trap_answers_the_player_only() {
        let mut w = switch_world(443, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            !switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5).hit,
            "the geyser does not consume the scan"
        );
        let mut w = switch_world(443, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        assert!(
            !switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 3).hit,
            "the projectile does not trip the geyser"
        );
        // The projectile's call would also answer the geyser's shot; the
        // player's call fires it.
        let mut w = switch_world(443, 0);
        let mut wiring = tr_core::wiring::Wiring::new();
        switch_tiles(&mut wiring, &mut w, 168.0, 300.0, 168.0, 280.0, 20, 42, 5);
        assert!(
            !wiring.take_device_shots().is_empty(),
            "the geyser fires for the player"
        );
    }

    /// `Collision.GetWaterLine` (`:1498-1544`): the surface height of the
    /// column, read from the four rows around the query.
    #[test]
    fn the_water_line_reads_the_column_surface() {
        let mut w = world();
        put_water(&mut w, 10, 5, 255, tr_core::tile::LiquidKind::Water);
        // The surface of a full tile: the row's bottom (96) less the
        // liquid's shortfall (255 / 16 = 15).
        assert_eq!(get_water_line(&w, 10, 5), Some(81.0));
        assert_eq!(get_water_line(&w, 10, 6), Some(81.0), "one row below");
        assert_eq!(get_water_line(&w, 10, 4), Some(81.0), "one row above");
        assert_eq!(get_water_line(&w, 10, 3), None, "clear of the water");
        // Half full: the shortfall is 128 / 16 = 8.
        let mut w = world();
        put_water(&mut w, 10, 5, 128, tr_core::tile::LiquidKind::Water);
        assert_eq!(get_water_line(&w, 10, 5), Some(88.0));
    }

    /// `Collision.GetWaterLineIterate` (`:1551-1570`): the walk up to the
    /// column's surface.
    #[test]
    fn the_iterated_water_line_walks_up_the_column() {
        let mut w = world();
        put_water(&mut w, 10, 5, 255, tr_core::tile::LiquidKind::Water);
        put_water(&mut w, 10, 6, 255, tr_core::tile::LiquidKind::Water);
        // The walk climbs to the first dry row and reports the surface of the
        // row below it.
        assert_eq!(get_water_line_iterate(&w, 10, 6), Some(80.0));
        assert_eq!(get_water_line_iterate(&w, 10, 5), Some(80.0));
        assert_eq!(get_water_line_iterate(&w, 10, 7), None, "a dry column");
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

    /// `Collision.StickyTiles` (`:3362-3435`): the cobweb (51) is found under a
    /// resting body, the honey block (229) only when flat, and the half brick's
    /// 8 px shrink makes the probe miss a body standing on the thin lip.
    #[test]
    fn the_sticky_probe_finds_cobwebs_and_flat_honey_only() {
        // At rest (`Velocity == 0`) the `:3401` puff gate short-circuits before
        // the `Next(30)` draw, so the probe is deterministic here.
        let mut rng = tr_core::UnifiedRandom::default();
        let (w, h) = (20.0f32, 42.0f32);
        // A cobweb at row 20, the body standing on it.
        let mut w_web = world();
        put(&mut w_web, 10, 20, 51);
        let hit = sticky_tiles(&w_web, 164.0, 20.0 * 16.0 - h + 1.0, w, h, 0.0, 0.0, &mut rng);
        assert_eq!(hit, Some((10, 20, false)), "the cobweb is under the feet");
        // The honey block: flat is sticky, any slope is not.
        let mut w_honey = world();
        put(&mut w_honey, 10, 20, 229);
        assert_eq!(
            sticky_tiles(&w_honey, 164.0, 20.0 * 16.0 - h + 1.0, w, h, 0.0, 0.0, &mut rng),
            Some((10, 20, false)),
            "the flat honey block is sticky"
        );
        let i = w_honey.index(10, 20);
        w_honey.tiles[i].set_slope(1);
        assert_eq!(
            sticky_tiles(&w_honey, 164.0, 20.0 * 16.0 - h + 1.0, w, h, 0.0, 0.0, &mut rng),
            None,
            "a sloped honey block is not (`:3410`)"
        );
        // Nothing sticky under an empty floor.
        assert_eq!(
            sticky_tiles(&world(), 164.0, 20.0 * 16.0 - h + 1.0, w, h, 0.0, 0.0, &mut rng),
            None
        );
    }

    /// `:3418-3422`: the half brick lifts the box' top 8 px and shortens it to
    /// 8.01 — a body whose bottom rests on the tile's top edge misses the
    /// shrunken box.
    #[test]
    fn the_half_brick_shrinks_the_honey_box() {
        let mut rng = tr_core::UnifiedRandom::default();
        let (w, h) = (20.0f32, 42.0f32);
        let mut world_hb = world();
        put(&mut world_hb, 10, 20, 229);
        let i = world_hb.index(10, 20);
        world_hb.tiles[i].flags |= tr_core::tile::FLAG_HALF_BRICK;
        // The body's bottom sits at the tile's top (320): the half brick lifts
        // the box' top to 328, so `pos_y + height > ty2` is `320 > 328` — false.
        assert_eq!(
            sticky_tiles(&world_hb, 164.0, 20.0 * 16.0 - h + 1.0, w, h, 0.0, 0.0, &mut rng),
            None,
            "the half brick's box starts 8 px down"
        );
        // Ten pixels lower the body reaches into the shrunk box.
        assert_eq!(
            sticky_tiles(
                &world_hb,
                164.0,
                20.0 * 16.0 - h + 10.0,
                w,
                h,
                0.0,
                0.0,
                &mut rng
            ),
            Some((10, 20, false))
        );
    }

    /// `Collision.cs:3918-4188`: a 16x16 box resting on a belt's top face takes
    /// the belt's direction - the 421 (left) belt pushes +X, the 422 (right)
    /// belt pushes -X - and the ramp is `zero * 6f / 60f` per tick, capped at
    /// the 2.5 clamp.
    #[test]
    fn the_conveyor_belt_ramps_the_item_velocity() {
        let (w, h) = (14i32, 14i32);
        // The box' bottom sits exactly on the tile's top edge (`y = 320`).
        let rest = |kind: u16| {
            let mut wd = world();
            put(&mut wd, 10, 20, kind);
            wd
        };
        let step = |wd: &World, pos: &mut [f32; 2], vel: &mut [f32; 2]| {
            apply_conveyor_belt_movement_to_velocity(wd, pos, vel, w, h)
        };
        // 421 = +1: the first tick adds 2.5 * 6 / 60 = 0.25 px/tick.
        let wd = rest(421);
        let (mut pos, mut vel) = ([160.0f32, 320.0 - h as f32], [0.0f32, 0.0f32]);
        step(&wd, &mut pos, &mut vel);
        assert_eq!(vel[0], 0.25, "the belt ramp is zero.X * 6f / 60f");
        for _ in 0..30 {
            step(&wd, &mut pos, &mut vel);
        }
        assert_eq!(vel[0], 2.5, "the ramp saturates at the 2.5 clamp");
        // 422 = -1: the mirror.
        let wd = rest(422);
        let (mut pos, mut vel) = ([160.0f32, 320.0 - h as f32], [0.0f32, 0.0f32]);
        step(&wd, &mut pos, &mut vel);
        assert_eq!(vel[0], -0.25);
        // A plain solid tile is not a belt: the velocity is untouched.
        let wd = rest(1);
        let (mut pos, mut vel) = ([160.0f32, 320.0 - h as f32], [0.0f32, 0.0f32]);
        step(&wd, &mut pos, &mut vel);
        assert_eq!(vel, [0.0, 0.0], "dirt does not carry the drop");
    }

    /// `Collision.StepConveyorBelt` (`:4190-4320`): a 20x42 body standing on a
    /// left belt (421) is walked +X by the 2.5 px diagonal step; the 422 belt
    /// mirrors it. The `gfxOffY`/grapple/pulley guards bail out before any
    /// probing.
    #[test]
    fn the_conveyor_belt_walks_the_player() {
        let mut wd = world();
        put(&mut wd, 10, 20, 421);
        let step = |wd: &World, pos: &mut [f32; 2], gfx: f32| {
            step_conveyor_belt_player(wd, pos, 20, 42, 1.0, gfx, false, false, false)
        };
        let (mut pos, mut vel) = ([160.0f32, 320.0 - 42.0], [0.0f32, 0.0f32]);
        // Sanity: a plain solid tile leaves the body alone.
        let mut plain = world();
        put(&mut plain, 10, 20, 1);
        let mut pp = pos;
        step(&plain, &mut pp, 0.0);
        assert_eq!(pp, pos, "dirt does not walk the player");
        // The belt moves it. The position is the only channel (the C# moves
        // `entity.position` directly, no velocity), so the test measures that.
        step(&wd, &mut pos, 0.0);
        assert!(
            pos[0] > 160.0,
            "the left belt walks the body +X, got {}",
            pos[0]
        );
        assert!((pos[0] - 160.0) < 2.6, "one belt step is 2.5 px");
        // The right belt (422) mirrors it.
        let mut wr = world();
        put(&mut wr, 10, 20, 422);
        let mut rp = [160.0f32, 320.0 - 42.0];
        step(&wr, &mut rp, 0.0);
        assert!(rp[0] < 160.0, "the right belt walks the body -X");
        // The `gfxOffY > 2` guard: a stepped-up body is not carried.
        let mut gp = [160.0f32, 320.0 - 42.0];
        step(&wd, &mut gp, 2.5);
        assert_eq!(gp, [160.0, 320.0 - 42.0], "a step-up offset bails out");
        let _ = vel;
    }

    /// `Collision.noSlopeCollision` (`:2033-2133`) vs `Collision.TileCollision`:
    /// the slope-1 ramp refuses a body sliding into it through the `flag3` gate
    /// (`:131`-`143` / C# `:4481-4483`), the slope-free variant does not.
    #[test]
    fn the_slope_free_collision_ignores_the_ramp_gate() {
        let mut w = world();
        put(&mut w, 10, 20, 1);
        let i = w.index(10, 20);
        w.tiles[i].set_slope(1);
        // A 20x20 body resting exactly on the ramp's row, moving down.
        let gated = tile_collision(&w, [160.0, 300.0], [0.0, 4.0], 20.0, 20.0, false, false);
        assert!(!gated.down, "the slope gate lets the ramp pass through");
        assert_eq!(gated.vel[1], 4.0, "the velocity is untouched");
        let free = no_slope_collision(&w, [160.0, 300.0], [0.0, 4.0], 20.0, 20.0, false, false);
        assert!(free.down, "without the gate the tile stops the body");
        assert_eq!(free.vel[1], 0.0, "the body lands on the tile's top");
    }

    /// `Collision.AdvancedTileCollision` (`:4422-4566`): the `forcedIgnoredTiles`
    /// conjunct (`:4449`) and the `gravDir`-dependent `0.01` nudge (`:4507`).

    /// `Collision.AdvancedTileCollision` (`:4422-4566`): the `forcedIgnoredTiles`
    /// conjunct (`:4449`) and the `gravDir`-dependent `0.01` nudge (`:4507`).
    #[test]
    fn the_sandshark_collision_skips_its_set_and_nudges_by_gravity() {
        // A 20x20 body 4 px above the floor row, moving 5 px down: the swept
        // box reaches the tile's top at 320.
        let (pos, vel, w, h) = ([128.0f32, 296.0], [0.0f32, 5.0], 20.0f32, 20.0f32);
        // Sand (53) is in `ForAdvancedCollision.ForSandshark`
        // (`TileID.cs:125`): the shark swims through it.
        let mut sand = world();
        for x in 8..12 {
            put(&mut sand, x, 20, 53);
        }
        let skipped = advanced_tile_collision(
            &sand,
            &td::SET_FOR_SANDSHARK,
            pos,
            vel,
            w,
            h,
            false,
            false,
            1,
        );
        assert!(!skipped.down, "the sandshark set is ignored");
        assert_eq!(skipped.vel[1], 5.0, "the body keeps falling");
        // The same floor outside the set stops it — a concrete ledge.
        let mut stone = world();
        for x in 8..12 {
            put(&mut stone, x, 20, 1);
        }
        let landed =
            advanced_tile_collision(&stone, &td::SET_FOR_SANDSHARK, pos, vel, w, h, false, false, 1);
        assert!(landed.down, "stone is not in the set and stops the body");
        // `:4507`: `vector4.Y - (vector3.Y + Height)` = 320 - 316, and
        // `gravDir == 1` adds no nudge.
        assert_eq!(landed.vel[1], 4.0, "the body moves exactly onto the tile");
        // `:4507`: `+ ((gravDir == -1) ? (-0.01f) : 0f)`.
        let flipped = advanced_tile_collision(
            &stone,
            &td::SET_FOR_SANDSHARK,
            pos,
            vel,
            w,
            h,
            false,
            false,
            -1,
        );
        assert!(flipped.down);
        assert!(
            (flipped.vel[1] - 3.99).abs() < 1e-4,
            "the inverted-gravity nudge: {}",
            flipped.vel[1]
        );
    }

    /// `Collision.HitTiles` (`:3063-3108`): the swept box's solid overlap is
    /// inclusive, an empty cell answers nothing.

    /// `Collision.HitTiles` (`:3063-3108`): the swept box's solid overlap is
    /// inclusive, an empty cell answers nothing.
    #[test]
    fn the_hit_scan_reports_the_solid_cells_under_the_swept_box() {
        let mut w = world();
        put(&mut w, 10, 20, 1);
        // The box [160,180] x [280,322] reaches the tile's top row.
        assert_eq!(
            hit_tiles(&w, [160.0, 280.0], [0.0, 0.0], 20, 42),
            vec![(10, 20)],
            "the floor cell is hit"
        );
        // One row up: nothing solid in reach.
        assert!(hit_tiles(&w, [160.0, 240.0], [0.0, 0.0], 20, 42).is_empty());
    }

    /// `Collision.HitTilesInACircle` (`:3016-3061`): the same scan plus the
    /// `(Width + Height) / 4` centre-distance gate (`:3054`). The corner cell
    /// the box still grazes falls outside the radius.

    /// `Collision.HitTilesInACircle` (`:3016-3061`): the same scan plus the
    /// `(Width + Height) / 4` centre-distance gate (`:3054`). The corner cell
    /// the box still grazes falls outside the radius.
    #[test]
    fn the_circle_hit_scan_drops_the_box_corner() {
        let mut w = world();
        put(&mut w, 10, 17, 1);
        put(&mut w, 9, 16, 1);
        let pos = [160.0, 250.0];
        let (width, height) = (20, 42);
        assert_eq!(
            hit_tiles(&w, pos, [0.0, 0.0], width, height),
            vec![(9, 16), (10, 17)],
            "the box reaches both corner cells"
        );
        assert_eq!(
            hit_tiles_in_a_circle(&w, pos, [0.0, 0.0], width, height),
            vec![(10, 17)],
            "only the cell inside the radius survives"
        );
    }

    /// `Collision.BuildTileContacts` (`:2135-2290`): the four side contacts,
    /// their pixel overlap and the corner fallback.

    /// `WorldGen.KillTile_GetTileDustAmount` (`:66597-66685`): the switch's
    /// spot values, including the `fail` split and the gem tree's frame gate.
    #[test]
    fn the_tile_dust_amount_table_matches_the_source() {
        // The default arm (`:66601-66606`).
        assert_eq!(tile_dust_amount(1, false, 0), 10);
        assert_eq!(tile_dust_amount(1, true, 0), 3);
        // The explicit no-dust cases (`:66607-66611`) and a listed row
        // (`:66626-66663`).
        assert_eq!(tile_dust_amount(654, false, 0), 0);
        assert_eq!(tile_dust_amount(630, true, 0), 0);
        assert_eq!(tile_dust_amount(125, false, 0), 0);
        // The fail-split arms (`:66612-66617`, `:66664-66673`).
        assert_eq!(tile_dust_amount(231, false, 0), 6);
        assert_eq!(tile_dust_amount(231, true, 0), 3);
        assert_eq!(tile_dust_amount(534, false, 0), 5);
        assert_eq!(tile_dust_amount(534, true, 0), 3);
        // The fixed counts (`:66618-66625`).
        assert_eq!(tile_dust_amount(481, true, 0), 3);
        assert_eq!(tile_dust_amount(541, false, 0), 1);
        assert_eq!(tile_dust_amount(549, false, 0), 2);
        // The gem tree (`:66674-66683`): `frameX / 22 >= 5` wins over `fail`.
        assert_eq!(tile_dust_amount(184, false, 110), 5);
        assert_eq!(tile_dust_amount(184, true, 110), 5);
        assert_eq!(tile_dust_amount(184, false, 0), 10);
        assert_eq!(tile_dust_amount(184, true, 0), 3);
    }
}
