//! W4-d1 wave 3 — the aiStyle-10 falling-block family (`AI_010`,
//! `Projectile.cs:57826-58100`, dispatched from `AI()`'s
//! `else if (this.aiStyle == 10)` `:24367-24370`). The kinds
//! `WorldGen.GetSandfallProjData` (`WorldGen.cs:86514-86570`) spawns when a
//! `TileID.Sets.Falling` tile loses its support, the Dirt Rod's DirtBall
//! 17, the player Mud Ball 1081, and the rowed-but-inert sandgun balls
//! 42/65/68/354 (the S1-D-5 "42-row gap" — the rows existed but the
//! dispatch's `10 =>` arm no-op'd them; this arm closes the gap).
//!
//! The 109 hostile snowball (`ai_snowball_hostile`) and the 736-738 debris
//! trio (`ai_dungeon_debris`) keep their own arms — the dispatch gates this
//! one on [`FALLING_BLOCK_KINDS`].
//!
//! FROZEN-TAIL CARVE-OUT — RETIRED (W4-d1 retire, the documented wiring
//! follow-up): the pool's `spawn_falling` path moved to the data spawn
//! (`spawn_with_data_ai` — the rowed `SetDefaults` stats, `ai[0] = 1`,
//! vel (0, 0.41), NO `falling_tile`), so EVERY natural fall rides this
//! arm's flight + the `death_place` landing. The frozen tail (the
//! per-sub-step 0.41 gravity + the post-loop bottom-edge
//! `landed`/`can_place_back` placement) is removed; the `falling_tile`
//! field stays `None` on every slot (the save/net shape). The deltas
//! over the frozen tail are recorded in ANALYSIS W4-d1 retire (the pen
//! −1 rows, the blocked-move kill + centre-cell place, the player-damage
//! pass, the tile-cut chain, the dust-gate draws).
//!
//! C# sources read WHOLE (the golden rule): `AI_010` (`:57826-58100`),
//! `HandleMovement`'s aiStyle-10 block (`:17592-17610`) + the blocked-move
//! chain's 1081 arm (`:19829-19847`) and default (`:19849-19853`),
//! `Kill_ExplodeTiles`'s aiStyle-10 death placement (`:77387-77570`, the
//! owner gate `:77267`, the `!noDropItem` gate `:77367`),
//! `Collision.AnyCollisionWithSpecificTiles` (`Collision.cs:2813-2913`),
//! `WorldGen.SolidTile3` (`WorldGen.cs:70465-70485`),
//! `ProjectileID.Sets.FallingBlockDoesNotFallThroughPlatforms`
//! (`ProjectileID.cs:433`).
//!
//! The machine, in the C#'s own order (`AI_010`):
//! 1. The per-kind dust gate (`:57828-57941`): the `Next(2)`/`Next(3)`
//!    rolls STAY on the `UpdateProjectiles` stream (the dust bodies skip —
//!    the `projectile_ai_misc` convention); the generic tail is `Next(20)`
//!    (the 17 rides it; the 109/debris exclusions never reach this arm).
//! 2. `tileCollide = true; localAI[1] = 0` every tick (`:57951-57952`).
//! 3. The owner's first tick (`ai[0] == 0`, `:57953-57996`): tileCollide
//!    drops for the tick; the channel branch aim-steers toward the mouse
//!    at speed 12 (the 40 excluded, `:57956`), else `ai[0] = 1`.
//! 4. The 1081's own age/gravity (`:57997-58004`): `ai[0]++`, from 20
//!    `vy += 0.3`, `vx *= 0.98`. Else `ai[0] == 1` (`:58006-58021`): the
//!    sandgun balls 42/65/68/354 count `ai[1]` to 60 BEFORE gravity
//!    (`vy += 0.2` only at the cap), every other kind `vy += 0.41`.
//!    Else `ai[0] == 2` (`:58022-58037`): `vy += 0.2`, the X damp ±0.04→0.
//! 5. The owner's tile walk (`:58038-58072`): breakables die, a top slope
//!    flattens and lifts the block 16 px — the shared
//!    [`ai010_owner_tile_walk`] (the `ai_dungeon_debris` body, extracted).
//! 6. `rotation += 0.1`, the 10 px/tick fall cap (`:58092-58099`).
//!
//! Movement (`HandleMovement`): `flag6 = false` for the
//! `FallingBlockDoesNotFallThroughPlatforms` set (`:17544-17547`), the
//! plain `TileCollision(pos, vel, w, h, flag6, flag6)` (`:17609`); the
//! 411-414 chain the `AnyCollisionWithSpecificTiles(TileID.Sets.Falling,
//! evenActuated: true)` pass after it (`:17601`). A blocked move runs the
//! tile-cut chain in the C#'s order (W4-d1 retire — the bodies PORTED,
//! `cut_tiles_in_flight_box`): the 484 rolling-cactus block (`:17910-17947`) on
//! every blocked move, THEN the tile-cut `Next(3)` (`:17948` — the draw
//! is the first `&&` operand, the ai026_c precedent), THEN the
//! cracked-brick block (`:17948-17985`) on the `== 0` roll.
//! The 1081's blocked arm (`:19829-19847`): a Y-block while falling steps
//! and kills; any other block zeroes a negative vy and latches
//! `localAI[0]` with the one-shot Dig cue (`PlaySound(0)`). Every other
//! kind takes the default `position += velocity; Kill()` (`:19849-19853`)
//! — the port's single-step `kill_step` stand-in (the C# `UpdatePosition`
//! `:20445` steps a second time after the default arm; the exact death
//! pixel can differ by the resolved span while the kill does not — the
//! recorded `kill_step` deviation).
//!
//! Death (`Kill_ExplodeTiles`'s aiStyle-10 arm, `:77387-77570`): the
//! centre-cell tile place map (proj → (tile, item)), the 1081 side-slide
//! probe (`:77492-77502`), the half-brick up-shift (`:77512`), the
//! below-tile 314/421/422 + `BlockBelowMakesSandFall` refusal
//! (`:77520-77542`), the forced `PlaceTile` (`:77545` — W4-d1 retire: the
//! forced flag's exact bypass set is censused + ported,
//! `tr_core::edit::place_tile_forced`; the `mute: false` tail's Dig/Coins
//! cue rides the pool's sound intent) + the below-slope flatten
//! (`:77549-77556`), else the `Item.RequestNewItem` drop
//! (`:77562-77569`). The drop rides the pool's `pending_block_drops`
//! intent (the `pending_catches` seam) — the pass drains it on the
//! `UpdateItems` stream (`spawn_item_drop_px`).
//!
//! Damage flags: the row shape carries neither `friendly` nor `hostile`
//! (ANALYSIS "The projectile data-row deviations"). The arm holds the C#
//! `friendly` state (the coin piles' stock false, every other kind true —
//! the `ai_dungeon_debris` precedent); the pool's PVE walk refuses the
//! coin piles (`Damage_PVE_Inner`'s `!friendly` refusal `:12611`) and the
//! `!friendly` player-damage proxy reads [`FALLING_HOSTILE_KINDS`] for the
//! friendly+hostile natural falls (`Damage_EVP` runs for them in the C#).
//!
//! Skips (each noted at its point): the dust bodies, the `netUpdate`
//! writes, the wet-velocity branch (the slot's liquid flags fold dry, the
//! ai026_c precedent), the `GetCollisionParams` resize (no served kind
//! overrides it), the `SlopeCollision` pass (the port's collision helper
//! carries the slope triangles; the extra `correctSlopeCollision` pass is
//! the recorded deferred layer), the `CanPoundTile`/fail-dust tails of the
//! tile walk's `SlopeTile`.
//!
//! CODE-ONLY MODE (W4-d1): the witness tests in `projectile.rs`
//! (`w4_d1_falling_block_*`) are WRITTEN NOT RUN; every expected value is
//! hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

use crate::projectile::{
    Projectile, ASH_BALL_FALLING, COIN_PILE_KINDS, CRIMSAND_BALL_FALLING, DIRT_BALL,
    EBONSAND_BALL_FALLING, FALLING_BLOCK_NO_PLATFORM_FALL, MUD_BALL_FALLING, MUD_BALL_PLAYER,
    PEARLSAND_BALL_FALLING, SAND_BALL_FALLING, SHELL_PILE_FALLING, SILT_BALL_FALLING,
    SLUSH_BALL_FALLING,
};

/// The port's `Kill()` stand-in for the blocked-move kill paths (the
/// slice-C convention — the death hooks run on the pool's `!active`
/// transition).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI_010`'s owner tile walk (`Projectile.cs:58038-58072`), shared with
/// `ai_dungeon_debris` (the S1-D-5 body, extracted verbatim — the
/// second-copy helper rule): the box's overlapping cells, the breakable
/// set dies (`WorldGen.KillTile`), a non-platform top slope flattens
/// (`WorldGen.SlopeTile`) and lifts the block one tile.
pub(crate) fn ai010_owner_tile_walk(p: &mut Projectile, world: &mut World, local_owner: u8) {
    // `:58038` — the owner + tileCollide gate.
    if p.owner != local_owner || !p.tile_collide {
        return;
    }
    let x0 = (p.pos.x / 16.0) as i32;
    let x1 = ((p.pos.x + p.w) / 16.0) as i32;
    let y0 = (p.pos.y / 16.0) as i32;
    let y1 = ((p.pos.y + p.h) / 16.0) as i32;
    for tx in x0..=x1 {
        for ty in y0..=y1 {
            // `WorldGen.InWorld(i, j)` (`WorldGen.cs:9688`, fluff 0) + the
            // null/active guard (`:58044-58052`).
            if !world.in_bounds(tx, ty) {
                continue;
            }
            let Some(tile) = world.tile(tx, ty) else {
                continue;
            };
            if !tile.active() {
                continue;
            }
            // `:58053-58060` — the breakable set: the small/large piles
            // 185-187, the stalactite 165, the heart 12, the life-crystal
            // boulder 665, the mana crystal 639, the statues 105, the
            // exposed gems 178.
            let kind = tile.kind;
            if (185..=187).contains(&kind)
                || kind == 165
                || kind == 12
                || kind == 665
                || kind == 639
                || kind == 105
                || kind == 178
            {
                let _ = tr_core::edit::kill_tile(world, tx, ty, false);
            } else if tile.top_slope() && !tr_core::tiledata::SET_PLATFORMS.has(kind) {
                // `:58061-58068` — `WorldGen.SlopeTile(i, j)`: clear the
                // half brick and the slope (the `slope_tile_flatten`
                // shape), then lift the block one tile. The `CanPoundTile`
                // gate and the fail-dust/sound/net tails are unported —
                // the flatten is unconditional (ANALYSIS, the debris
                // precedent).
                if let Some(t) = world.tile_mut(tx, ty) {
                    t.flags &= !tr_core::tile::FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
                p.pos.y -= 16.0;
            }
        }
    }
}

/// `AI_010` (`Projectile.cs:57826-58100`) + `HandleMovement`'s aiStyle-10
/// block for one falling-block slot, in the C#'s own order. `world` is
/// mutably reborrowed at the dispatch call site (the tile-writing
/// carve-out); `rng` is the pass's `UpdateProjectiles` stream;
/// `local_owner` is the pool's `Main.myPlayer`; `channel` is the owner's
/// latched `Player.channel`; `aim` is the mouse WORLD point
/// (`Main.mouseX + screenPosition.X`, the `YoyoInputs.aim` snapshot).
pub(crate) fn step(
    p: &mut Projectile,
    world: &mut World,
    rng: &mut dyn DropRng,
    local_owner: u8,
    channel: bool,
    aim: Option<Vec2>,
) {
    // The C# row flags the shape carries no field for: the coin piles keep
    // the stock `friendly = false` (`:788` — the `:4933-4941` arm touches
    // neither flag), every other served kind sets `friendly = true`. Held
    // explicitly for the slot's whole life (the `ai_dungeon_debris`
    // precedent); the damage-pass gates read it.
    p.friendly = !COIN_PILE_KINDS.contains(&p.kind);
    // `:57828-57941` — the per-kind dust gate: the rolls stay on the
    // stream, the `Dust.NewDust` bodies and their internal draws skip
    // (the dust-skip convention). The else-if chain order is the C#'s.
    if p.kind == SAND_BALL_FALLING && p.ai[0] != 2.0 {
        // `:57828-57835` — the 31's falling dust 32.
        let _ = rng.next_range(0, 2);
    } else if p.kind == MUD_BALL_FALLING {
        // `:57836-57843` — dust 38.
        let _ = rng.next_range(0, 2);
    } else if p.kind == MUD_BALL_PLAYER {
        // `:57844-57852` — dust 38, scale 0.8.
        let _ = rng.next_range(0, 2);
    } else if COIN_PILE_KINDS.contains(&p.kind) {
        // `:57853-57870` — the 1-in-3 gate, dust 9/11/19 per coin.
        let _ = rng.next_range(0, 3);
    } else if p.kind == ASH_BALL_FALLING {
        // `:57871-57878` — dust 36.
        let _ = rng.next_range(0, 2);
    } else if p.kind == 42 || p.kind == SAND_BALL_FALLING {
        // `:57879-57886` — the 42 + the settled 31 (`ai[0] == 2`), dust 32.
        let _ = rng.next_range(0, 2);
    } else if p.kind == EBONSAND_BALL_FALLING || p.kind == 65 {
        // `:57887-57894` — dust 14.
        let _ = rng.next_range(0, 2);
    } else if p.kind == PEARLSAND_BALL_FALLING || p.kind == 68 {
        // `:57895-57902` — dust 51.
        let _ = rng.next_range(0, 2);
    } else if p.kind == SILT_BALL_FALLING {
        // `:57903-57910` — dust 53.
        let _ = rng.next_range(0, 2);
    } else if p.kind == SLUSH_BALL_FALLING {
        // `:57911-57918` — dust 149.
        let _ = rng.next_range(0, 2);
    } else if p.kind == CRIMSAND_BALL_FALLING || p.kind == 354 {
        // `:57919-57926` — dust 36.
        let _ = rng.next_range(0, 2);
    } else {
        // `:57938-57941` — the generic `type != 109 && Next(20) == 0`
        // dust-0 tail (the 17 rides it; the 109 and the 736-738 trio never
        // reach this arm — the dispatch gates them to their own).
        let _ = rng.next_range(0, 20);
    }
    // `:57951-57952` — the per-tick reset.
    p.tile_collide = true;
    p.local_ai[1] = 0.0;
    // `:57953-57996` — the owner's first tick: tileCollide drops for the
    // tick (the tile walk below skips it).
    if p.owner == local_owner && p.ai[0] == 0.0 {
        p.tile_collide = false;
        if channel && p.kind != ASH_BALL_FALLING {
            // `:57956-57990` — the channel aim-steer: `localAI[1] = -1`,
            // the mouse-world delta capped at speed 12 (`num14`). The
            // gravDir −1 mirror folds positive (no gravity potion, the
            // pool convention); the `netUpdate` writes skip (no MP layer).
            // A `None` aim folds to the block's own centre (the zero-delta
            // arm — the pass always supplies the mouse point when a
            // player exists).
            p.local_ai[1] = -1.0;
            let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            let mut delta = aim.unwrap_or(center) - center;
            let len = delta.length();
            if len > 12.0 {
                delta *= 12.0 / len;
            }
            p.vel = delta;
        } else {
            // `:57991-57995` — the fall arm (`netUpdate` skipped).
            p.ai[0] = 1.0;
        }
    }
    // `:57997-58037` — the gravity ladder.
    if p.kind == MUD_BALL_PLAYER {
        // `:57997-58004` — the 1081's own age: from 20, +0.3 fall and the
        // 2% X damp.
        p.ai[0] += 1.0;
        if p.ai[0] >= 20.0 {
            p.vel.y += 0.3;
            p.vel.x *= 0.98;
        }
    } else if p.ai[0] == 1.0 {
        if matches!(p.kind, 42 | 65 | 68 | 354) {
            // `:58008-58016` — the sandgun balls fly straight for 60
            // ticks (`ai[1]` counts to the cap), then fall at 0.2.
            p.ai[1] += 1.0;
            if p.ai[1] >= 60.0 {
                p.ai[1] = 60.0;
                p.vel.y += 0.2;
            }
        } else {
            // `:58017-58020` — the tile-fall gravity.
            p.vel.y += 0.41;
        }
    } else if p.ai[0] == 2.0 {
        // `:58022-58037` — the settle mode: 0.2 gravity, the X damp
        // toward 0 in 0.04 steps.
        p.vel.y += 0.2;
        if p.vel.x < -0.04 {
            p.vel.x += 0.04;
        } else if p.vel.x > 0.04 {
            p.vel.x -= 0.04;
        } else {
            p.vel.x = 0.0;
        }
    }
    // `:58038-58072` — the owner's tile walk (the shared helper).
    ai010_owner_tile_walk(p, world, local_owner);
    // `:58092-58099` — the spin and the 10 px/tick fall cap (the non-debris
    // arm; the trio's frame-based spin stays in `ai_dungeon_debris`).
    p.rot += 0.1;
    if p.vel.y > 10.0 {
        p.vel.y = 10.0;
    }
    // `HandleMovement` (`:17592-17610`) + `UpdatePosition` (`:20445`).
    if !p.tile_collide {
        // The first-tick free move (the owner latch dropped tileCollide).
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    // `:17478` + `:17544-17547` — flag6 defaults true; the
    // FallingBlockDoesNotFallThroughPlatforms set drops it (these blocks
    // stop ON platforms). The `decidesManualFallThrough` override
    // (`:17583-17586`) is unset for every served row.
    let flag6 = !FALLING_BLOCK_NO_PLATFORM_FALL.contains(&p.kind);
    let tc = crate::collision::tile_collision(
        world,
        [p.pos.x, p.pos.y],
        [p.vel.x, p.vel.y],
        p.w,
        p.h,
        flag6,
        flag6,
    );
    p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    if COIN_PILE_KINDS.contains(&p.kind) {
        // `:17598-17602` — the coin piles also stop on the falling-tile
        // kinds, even actuated (`TileID.Sets.Falling`).
        let v = crate::collision::any_collision_with_specific_tiles(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            tr_core::falling::is_falling_kind,
            true,
        );
        p.vel = Vec2::new(v[0], v[1]);
    }
    if p.vel != last {
        // `HandleMovement`'s `flag` block, in the C#'s own order: the 484
        // rolling-cactus cut (`:17910-17947`) runs on EVERY blocked move FIRST (no
        // call-site rand), THEN the tile-cut `Next(3)` (`:17948` — the
        // FIRST `&&` operand, consumed regardless of the trailing gates,
        // the ai026_c precedent), THEN the cracked-brick cut
        // (`:17948-17985`) on the `== 0` roll. W4-d1 retire: the cut
        // BODIES are ported now (the shared `cut_tiles_in_flight_box`
        // walk — the box is `position + velocity`, pre-integration, the
        // C#'s HandleMovement-before-UpdatePosition shape).
        cut_tiles_in_flight_box(p, world, local_owner, false);
        let cut_roll = rng.next_range(0, 3);
        if cut_roll == 0 {
            cut_tiles_in_flight_box(p, world, local_owner, true);
        }
        if p.kind == MUD_BALL_PLAYER {
            // `:19829-19847` — the 1081's blocked arm.
            if last.y != p.vel.y && last.y > 0.0 {
                // `:19831-19835` — a Y-block while falling: one last step
                // and the kill (the single-step `kill_step` stand-in).
                p.pos += p.vel;
                kill(p);
                return;
            }
            // `:19836-19847` — any other block: a negative vy zeroes, the
            // `localAI[0]` one-shot latch plays the Dig cue
            // (`PlaySound(0)` — the id-0 drain arm).
            if p.vel.y < 0.0 {
                p.vel.y = 0.0;
            }
            if p.local_ai[0] == 0.0 {
                p.local_ai[0] = 1.0;
                p.wants_sound = Some((0, 0, p.pos.x, p.pos.y, 0.0));
            }
        } else {
            // `:19849-19853` — the blocked-move default: one last step
            // into the tile and the kill (the `kill_step` stand-in — the
            // C#'s post-chain `UpdatePosition` steps again, the recorded
            // death-pixel deviation).
            p.pos += p.vel;
            kill(p);
            return;
        }
    }
    // `UpdatePosition` (`:20445`) — the free integrate (the wet branch
    // folds dry, module doc).
    p.pos += p.vel;
}

/// `HandleMovement`'s two tile-cut blocks (`Projectile.cs:17910-17947`
/// the 484 rolling-cactus block, `:17948-17985` the cracked-brick block — the
/// second-copy helper rule: one walk, the kind test is the parameter).
/// W4-d1 retire — the cutting layer's falling-block slice. W4-d1 SEAL:
/// the 491 FlyingKnife's blocked move shares the walk (its `CanCutTiles`
/// passes — aiStyle 9 is not on the refusal list, the 491 not on the type
/// list — and `friendly || hostile` holds; the coin-pile fold is a no-op
/// for it).
///
/// The gate (both blocks): `owner == Main.myPlayer && CanCutTiles() &&
/// (friendly || hostile) && damage > 0`. `CanCutTiles` (`:15323-15334`)
/// passes every served kind (aiStyle 10 is not on the style refusal list,
/// no served kind is on the type list); `friendly || hostile` holds for
/// every served kind EXCEPT the coin piles (stock both-false — the arm
/// holds `friendly = false`, the rows touch neither flag); `damage > 0`
/// excludes the zero-damage snow variants' falls (`GetSandfallProjData`'s
/// 330-333 dmg 0 — the coin piles again). `GetTileCutIgnorance(false,
/// trap)` (`Player.cs:44632-44644`): `trap` is false for every served row
/// and `dontHurtNature` has no ported granter (the `bug_net` precedent) —
/// the ignore set is `TileCutIgnore.None` (all-false).
///
/// The box (`:17912-17931`): `(int)(pos + vel - 1) / 16` ..
/// `(int)((pos + size + vel + 1) / 16) + 1`, clamped to `[0, maxTiles]`
/// — the position + velocity box BEFORE the integration (HandleMovement
/// runs ahead of `UpdatePosition` `:20445`). Each matching cell through
/// `WorldGen.CanCutTile` (`:86828-86839`, the port's
/// `tileobject::can_cut_tile` — the context does not reach the body) and
/// `WorldGen.KillTile` (the port's full `kill_tile`, the tile-walk
/// precedent; the `netMode` sends are MP-only). ZERO call-site rand —
/// the cracked block's `Next(3)` is the caller's (the first `&&`
/// operand, drawn on every blocked move).
pub(crate) fn cut_tiles_in_flight_box(p: &Projectile, world: &mut World, local_owner: u8, cracked: bool) {
    // The shared gate (module doc): `owner == Main.myPlayer` + the
    // `friendly || hostile` fold (the coin piles are the only served kinds
    // with both flags false) + `damage > 0`. `CanCutTiles` passes every
    // served kind (the module doc's fold).
    if p.owner != local_owner || COIN_PILE_KINDS.contains(&p.kind) || p.damage <= 0 {
        return;
    }
    let x0 = ((p.pos.x + p.vel.x - 1.0) as i32 / 16).max(0);
    let x1 = (((p.pos.x + p.w + p.vel.x + 1.0) as i32) / 16 + 1).min(world.width);
    let y0 = ((p.pos.y + p.vel.y - 1.0) as i32 / 16).max(0);
    let y1 = (((p.pos.y + p.h + p.vel.y + 1.0) as i32) / 16 + 1).min(world.height);
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            let matches_kind = if cracked {
                tr_core::tiledata::SET_CRACKED_BRICKS.has(t.kind)
            } else {
                t.kind == 484
            };
            if matches_kind && tr_core::tileobject::can_cut_tile(world, tx, ty) {
                let _ = tr_core::edit::kill_tile(world, tx, ty, false);
            }
        }
    }
}

/// `WorldGen.SolidTile3(i, j)` (`WorldGen.cs:70465-70485`): in-world
/// (fluff 1) + `nactive() && tileSolid && !tileSolidTop`. An out-of-world
/// probe reads false (the C#'s `InWorld` guard).
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    if x < 1 || y < 1 || x >= world.width - 1 || y >= world.height - 1 {
        return false;
    }
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && t.flags & tr_core::tile::FLAG_INACTIVE == 0
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
    })
}

/// `Kill_ExplodeTiles`'s aiStyle-10 death placement
/// (`Projectile.cs:77387-77570`): the centre-cell tile place + the item
/// drop when the cell refuses. Returns `(drop, place_cue)` — `drop` is
/// the `Item.RequestNewItem` intent `(item, x, y)` (the pool parks it on
/// `pending_block_drops`, the pass drains it on the `UpdateItems`
/// stream); `place_cue` is the successful forced placement's
/// `PlaceTile` tail sound `(cue, tx, ty)` (`:60263-60282`, the pool
/// queues it on the sound intent). The owner gate is the caller's
/// (`:77267`); the `!noDropItem` gate (`:77367`) folds true (the stock
/// false, no served row sets it).
pub(crate) fn death_place(
    p: &Projectile,
    world: &mut World,
) -> (Option<(u16, f32, f32)>, Option<(tr_core::edit::SoundCue, i32, i32)>) {
    // `:77389-77392` — the centre cell, the default (tile 0, item 2): the
    // DirtBall 17 has NO map entry, so it places dirt and drops the dirt
    // block item — verbatim.
    let mut tx = (p.pos.x + (p.w as i32 / 2) as f32) as i32 / 16;
    let mut ty = (p.pos.y + (p.h as i32 / 2) as f32) as i32 / 16;
    let (mut tile, mut item) = (0u16, 2u16);
    // `:77398-77491` — the per-kind place map (the 109's 147/0 entry is
    // its own arm's, never reached here).
    match p.kind {
        SAND_BALL_FALLING => {
            tile = 53;
            item = if p.ai[0] == 2.0 { 0 } else { 169 };
        }
        42 => {
            tile = 53;
            item = 0;
        }
        EBONSAND_BALL_FALLING => {
            tile = 112;
            item = 370;
        }
        65 => {
            tile = 112;
            item = 0;
        }
        PEARLSAND_BALL_FALLING => {
            tile = 116;
            item = 408;
        }
        68 => {
            tile = 116;
            item = 0;
        }
        SILT_BALL_FALLING => {
            tile = 123;
            item = 424;
        }
        MUD_BALL_FALLING => {
            tile = 59;
            item = 0;
        }
        MUD_BALL_PLAYER => {
            tile = 59;
            item = 176;
        }
        ASH_BALL_FALLING => {
            tile = 57;
            item = 0;
        }
        SLUSH_BALL_FALLING => {
            tile = 224;
            item = 1103;
        }
        CRIMSAND_BALL_FALLING => {
            tile = 234;
            item = 1246;
        }
        354 => {
            tile = 234;
            item = 0;
        }
        411 => {
            tile = 330;
            item = 71;
        }
        412 => {
            tile = 331;
            item = 72;
        }
        413 => {
            tile = 332;
            item = 73;
        }
        414 => {
            tile = 333;
            item = 74;
        }
        SHELL_PILE_FALLING => {
            tile = 495;
            item = 4090;
        }
        _ => {}
    }
    // `:77492-77502` — the 1081's side-slide probe: no solid below → slide
    // onto a solid diagonal whose own cell is free.
    if p.kind == MUD_BALL_PLAYER && !solid_tile3(world, tx, ty + 1) {
        let left_free = world.tile(tx - 1, ty).is_some_and(|t| !t.active());
        let right_free = world.tile(tx + 1, ty).is_some_and(|t| !t.active());
        if solid_tile3(world, tx - 1, ty + 1) && left_free {
            tx -= 1;
        } else if solid_tile3(world, tx + 1, ty + 1) && right_free {
            tx += 1;
        }
    }
    // `:77512` — the half-brick up-shift: an active (nactive) half brick
    // under a downward-dominant fall lifts the placement one row. The
    // out-of-world read folds false (the C# would throw; the out-of-world
    // guard kills the slot first).
    let half_brick_shift = world.tile(tx, ty).is_some_and(|t| {
        t.active()
            && t.flags & tr_core::tile::FLAG_INACTIVE == 0
            && t.half_brick()
    }) && p.vel.y > 0.0
        && p.vel.y.abs() > p.vel.x.abs();
    if half_brick_shift {
        ty -= 1;
    }
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:77516` — the free-cell placement gate.
    if world.tile(tx, ty).is_some_and(|t| !t.active()) {
        let mut skip_place = false;
        // `:77520-77542` — the below-tile refusal: the planter box 314,
        // the corruption/crimson sand converters 421/422 (the 31's item
        // re-arms to the sand block 169), or `BlockBelowMakesSandFall`.
        if ty < world.height - 2 {
            if let Some(below) = world.tile(tx, ty + 1) {
                if below.active() {
                    if below.kind == 314 {
                        skip_place = true;
                    }
                    if below.kind == 421 || below.kind == 422 {
                        if p.kind == SAND_BALL_FALLING {
                            item = 169;
                        }
                        skip_place = true;
                    }
                    if tr_core::falling::block_below_makes_sand_fall(world, tx, ty) {
                        skip_place = true;
                    }
                }
            }
        }
        let mut placed = false;
        let mut place_cue = None;
        if !skip_place {
            // `:77545` — `WorldGen.PlaceTile(..., mute: false, forced:
            // true)`. W4-d1 retire: the forced flag's exact bypass set is
            // censused + ported (`tr_core::edit::place_tile_forced` — the
            // EmptyTile overlap scan AND the port's wall-unbreakable
            // refusal both bypassed; the served kinds match none of the
            // grass/moss/liquid/drip gates, `WorldGen.cs:59342-60295`).
            placed = tr_core::edit::place_tile_forced(world, tx, ty, tile);
            if placed {
                // `:60263-60282` — the `mute: false` tail switch rides the
                // `place_tile_sound` table: 330-333 → the legacy Coins 18,
                // every other served kind → the Dig 0. The intent rides
                // the pool's sound drain (the `wants_sound` seam); the
                // C# plays at `(i*16, j*16)`.
                place_cue = Some((tr_core::edit::place_tile_sound(tile), tx, ty));
            }
        }
        // `:77547-77556` — the placement stood: flatten a half-brick /
        // slope below (`WorldGen.SlopeTile`, the debris flatten shape).
        let stood = world
            .tile(tx, ty)
            .is_some_and(|t| t.active() && t.kind == tile);
        if !skip_place && stood {
            if let Some(below) = world.tile(tx, ty + 1) {
                if below.half_brick() || below.slope() != 0 {
                    if let Some(t) = world.tile_mut(tx, ty + 1) {
                        t.flags &= !tr_core::tile::FLAG_HALF_BRICK;
                        t.set_slope(0);
                    }
                }
            }
        } else if !placed && item > 0 {
            // `:77562-77565` — the refused placement drops the item at
            // `Center + (0, -2)`.
            return (Some((item, center.x, center.y - 2.0)), place_cue);
        }
        (None, place_cue)
    } else if item > 0 {
        // `:77567-77570` — the occupied cell drops the item at `Center`.
        (Some((item, center.x, center.y)), None)
    } else {
        (None, None)
    }
}