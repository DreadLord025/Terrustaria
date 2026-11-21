//! The aiStyle-26 tails (S1-F-13b2 slice B) — the deferred half of the
//! Headless Horseman (315) / Hellhound (329) machine: the `FindFrame`
//! sprite arms, the Horseman's gallop dust (with its exact `Main.rand`
//! draws), its `Lighting.AddLight`, the `gfxOffY` pre-AI decay and the
//! pumpkin lob's spawn-time scale roll.
//!
//! **Disjoint-file slice.** Slice A (`npc_ai_unicorn.rs`, branch
//! `task/S1-F-13b2-unicorn`) owns the `AI_026_Unicorns` core and the
//! `Npc` field/dispatch edits; this file is pure free functions over
//! passed-in state so it compiles against `origin/main` alone. WIRED by
//! S1-D-4 census slice W-10 (the union points below name the live call
//! sites). Code-only mode: the
//! tests are written, not run (no compiler invoked on this branch).
//!
//! # Union points (merge wiring, in C# order)
//!
//! 1. **`gfx_off_y_decay`** — `NPC.UpdateNPC` (`NPC.cs:90395-90420`),
//!    AFTER the `oldTarget`/`oldDirection`/`oldDirectionY` latches
//!    (`:90393-90395`) and BEFORE `this.AI()` (`:90422`). In the port:
//!    `Npc::update_rng` (`npc.rs`), ahead of the aiStyle dispatch, on
//!    slice A's `gfx_off_y`/`step_speed` fields (`NPC.cs:6095`/`:6097`;
//!    `AI_026` sets `stepSpeed` 1f/2f at `:62796`/`:62800`). The decay
//!    reads the CURRENT `velocity.X` — the pre-AI value, verbatim.
//!    Render adds `gfxOffY` to the draw Y (`Main.cs:25414` etc.); the
//!    port's `render/entities.rs` NPC draw adds `n.gfx_off_y` at union.
//! 2. **`horseman_gallop_dust`** — inside slice A's `if horseman` block
//!    (`npc_ai_unicorn.rs`, the deferred comment after the lob), at the
//!    C# position `NPC.cs:62421-62439`: AFTER the 480-tick lob (and its
//!    three draws) and the `AddLight`, BEFORE the stuck-counter block
//!    (`:62441`). The draws ride the caller's `UpdateNPCs` stream
//!    in-step — NOT a deferred visual queue (the `player.rs` spark
//!    queue is a different seam: its C# draws happen outside the AI).
//!    Union threading: `unicorn_step` gains `pool: &mut DustPool` +
//!    `ctx: &SpawnCtx` parameters, fed from the `update_npcs` tick
//!    closure (`world_sim/npcs.rs:2687-2695`) the way `audio` is
//!    already threaded — `app.dust` and a `SpawnCtx` built from the
//!    camera/screen statics. `frame_row` is `npc.frame` (the port's
//!    row-index convention), `bottom` is `(x + w/2, y + h)`
//!    (`Entity.cs:122-125`), `velocity` is `(vx, vy)`.
//! 3. **`horseman_light`** — the C# `AddLight` (`NPC.cs:62420`) is
//!    unconditional every Horseman AI tick; the port's light feed is
//!    render-side (`app/frame/draw.rs:373-388`, the `npc_lights`
//!    S1-E-5 precedent — no RNG, so the placement moves no stream).
//!    Union: call it beside `npc_light` in that loop and
//!    `entity_lights.add` the result.
//! 4. **`horseman_find_frame` / `hellhound_find_frame`** —
//!    `Npc::find_frame` (`npc.rs`), the 315/329 arms of the C# switch
//!    (`NPC.cs:74465-74497` / `:74375-74403`). The caller is inside
//!    `impl Npc`, so slice A's private `frame_counter` needs no new
//!    accessor: pass `self.frame_counter`/`self.frame`/`self.sprite_dir`
//!    in and write the returned triple back. `frame` counts: 315 = 11
//!    rows, 329 = 10 (`Main.cs:67082`'s array — the port's
//!    `NPC_FRAME_COUNT` already carries both).
//! 5. **`horseman_pumpkin_scale`** — `Projectile.SetDefaults`'s 1001 arm
//!    (`Projectile.cs:9518`): `scale = 1f + Main.rand.Next(51) * 0.01f`,
//!    a draw at `NewProjectile` time INSIDE the AI — i.e. on the
//!    `UpdateNPCs` stream immediately after the lob's
//!    `NextFloatDirection()` draw (`NPC.cs:62417`), before any later AI
//!    draw. The port's spawn drains post-tick
//!    (`world_sim/npcs.rs:3546-3556`), so the roll must happen at the
//!    QUEUE site: union adds this call as the lob's fourth draw in
//!    slice A's arm and carries the value to the spawned slot (a
//!    `scale` field on `HostileShotAi`, applied after
//!    `spawn_hostile_ai`). Rolling it at the drain instead would move
//!    the draw past every later NPC's AI draws — a stream deviation.
//!    The `proj_data` 1001 row (this slice, `projectile/data.rs`)
//!    carries the stock 1f base; the aiStyle-194 flight
//!    (`AI_194_HorsemanPumpkin`, `Projectile.cs:41293`) is a follow-up.
//!
//! # RNG argument (the gallop dust, `NPC.cs:62421-62439`)
//!
//! `num4 = frame.Height` (clamped ≥ 1), `num5 = frame.Y / num4` — the
//! row index; the port's `frame: u32` IS `num5`, so `num` cancels. The
//! gate `4 <= num5 <= 7` guards ALL draws. When it passes:
//!
//! 1. `Main.rand.Next(3)` — ALWAYS drawn (the `!= 0` gate).
//! 2. Only when `!= 0`: `NextFloat() * 60` then `NextFloat() * 8`
//!    (the `Vector2` ctor evaluates x then y), the position jitter over
//!    the `Bottom + (-30, -8)` base, 60×8 box.
//! 3. `Dust.NewDustPerfect(pos, 6, base.velocity)` (`Dust.cs:54-63`) =
//!    `NewDust(pos, 0, 0, 6, 0, 0, 0, default, 1)` + exact position /
//!    velocity overwrites. On an empty pool (first inactive slot 0, no
//!    pressure band) the internal draws are, in order
//!    (`Dust.cs:153-193` + the type-6 tweak `:214-221`):
//!    `Next(1)` (x, box clamped 5×5), `Next(1)` (y), `Next(-20, 21)`
//!    (vx), `Next(-20, 21)` (vy), `Next(3)` (frame row),
//!    `Next(-20, 21)` (scale), `Next(-10, 6)` (the type-6 vy re-roll).
//!    Type 6 is not `DangerousDust` (`ChildSafety.cs:32`) — no
//!    ChildSafety draw. The overwrites then discard the rolled
//!    position/velocity, but the DRAWS stay on the stream. Higher
//!    slots add the pressure-band draws (`Dust.cs:114-152`) and the
//!    off-screen cull (`:97-101`) skips the internals entirely — the
//!    pool port handles both; the witness tests pin the empty-pool and
//!    culled shapes.
//! 4. Post-spawn writes (no draws): `scale = 0.6`, `fadeIn = 1.1`,
//!    `noGravity = true`, `noLight = true` — into whatever slot
//!    `NewDust` returned, the sentinel included (the C#'s own shape,
//!    `particles.rs::new_dust_perfect` docs).
//!
//! The light, the FindFrame arms and the gfxOffY decay draw nothing.

// The functions land at the union points above; until the merge wires
// them, only the tests call them.
#![allow(dead_code)]

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::NpcKind;

/// `FindFrame`'s case 315 (`NPC.cs:74465-74497`) — the Headless
/// Horseman's sprite arm. Row-index convention: the C# `frame.Y` is
/// `num * row` (`num = frame.Height`, the sheet row height), so `num`
/// cancels out of every comparison; `frame_row` is the port's
/// `Npc::frame`. Returns `(sprite_dir, frame_counter, frame_row)`.
///
/// The sprite-direction block runs first and INDEPENDENTLY of the
/// airborne reset: `vy != 0 && wet` still turns the sprite, then resets
/// the row (the C# `break` at `:74485` comes after the turn block).
pub(crate) fn horseman_find_frame(
    vx: f32,
    vy: f32,
    wet: bool,
    facing: i32,
    sprite_dir: i32,
    frame_counter: f32,
    frame_row: u32,
) -> (i32, f32, u32) {
    let mut sprite_dir = sprite_dir;
    let mut counter = frame_counter;
    let mut row = frame_row;
    // `:74466-74479` — grounded or wet: the ±1 velocity dead-band, else
    // the facing (`base.direction`).
    if vy == 0.0 || wet {
        if vx < -1.0 {
            sprite_dir = -1;
        } else if vx > 1.0 {
            sprite_dir = 1;
        } else {
            sprite_dir = facing;
        }
    }
    // `:74480-74485` — airborne: row 0, counter 0, done.
    if vy != 0.0 {
        return (sprite_dir, 0.0, 0);
    }
    // `:74486-74496` — the gallop: |vx| per tick, advance at >= 8,
    // wrap past row 10 back to row 1 (row 0 is the jump frame).
    counter += vx.abs();
    if counter >= 8.0 {
        counter = 0.0;
        row += 1;
        if row > 10 {
            row = 1;
        }
    }
    (sprite_dir, counter, row)
}

/// `FindFrame`'s case 329 (`NPC.cs:74375-74403`) — the Hellhound's
/// sprite arm. Same row-index convention and return triple as
/// [`horseman_find_frame`]. The vertical arms break BEFORE the
/// `spriteDirection` assignment (`:74376-74387`) — an airborne hound
/// keeps its last sprite direction, verbatim.
pub(crate) fn hellhound_find_frame(
    vx: f32,
    vy: f32,
    facing: i32,
    sprite_dir: i32,
    frame_counter: f32,
    frame_row: u32,
) -> (i32, f32, u32) {
    // `:74376-74381` — falling: row 3, counter 0.
    if vy > 0.0 {
        return (sprite_dir, 0.0, 3);
    }
    // `:74382-74387` — rising: row 2, counter 0.
    if vy < 0.0 {
        return (sprite_dir, 0.0, 2);
    }
    // `:74388-74402` — grounded: the sprite rides the facing, the
    // counter accrues |vx| * 0.4, advance at > 4 (strict), clamp the
    // walk cycle to rows 2..=9 (both ends wrap to 2).
    let mut counter = frame_counter + vx.abs() * 0.4;
    let mut row = frame_row;
    if counter > 4.0 {
        counter = 0.0;
        row += 1;
        if row < 2 {
            row = 2;
        }
        if row > 9 {
            row = 2;
        }
    }
    (facing, counter, row)
}

/// `FindFrame`'s case 155 (`NPC.cs:74311-74373`) — the Wolf's sprite arm
/// (W4-d36 E4). Same row-index convention and return triple as
/// [`hellhound_find_frame`]. The vertical arms (`:74318-74325`) set rows
/// 10/11 and fall to the frame write WITHOUT touching the counter or the
/// sprite direction (unlike the 329 case's counter reset). The
/// `IsABestiaryIconDummy` row-3 clamp (`:74314-74317`) is the documented
/// skip — the port has no bestiary dummy (`npc_findframe_b.rs` module
/// docs). The 13-row strip is `Main.npcFrameCount[155]` (`Main.cs:67082`).
pub(crate) fn wolf_find_frame(
    vx: f32,
    vy: f32,
    facing: i32,
    sprite_dir: i32,
    frame_counter: f32,
    frame_row: u32,
) -> (i32, f32, u32) {
    let mut row = frame_row;
    let mut counter = frame_counter;
    // `:74318-74325` — rising row 10 / falling row 11, state untouched.
    if vy < 0.0 {
        return (sprite_dir, counter, 10);
    }
    if vy > 0.0 {
        return (sprite_dir, counter, 11);
    }
    // `:74326-74330` — the grounded standstill rests row 0.
    if vx == 0.0 {
        return (sprite_dir, 0.0, 0);
    }
    // `:74331-74352` — walking against the facing below 4: the sprite
    // rides the facing, rows > 2 reset to the 0..2 wind-up walk; the
    // counter++ clock advances at > 5, row 2 holds with counter 0.
    if ((facing > 0 && vx < 0.0) || (facing < 0 && vx > 0.0)) && vx.abs() < 4.0 {
        if row > 2 {
            row = 0;
            counter = 0.0;
        }
        if row < 2 {
            counter += 1.0;
            if counter > 5.0 {
                counter = 0.0;
                row += 1;
            }
        } else {
            counter = 0.0;
        }
        return (facing, counter, row);
    }
    // `:74353-74371` — the forward gallop: the sprite rides the velocity
    // sign, the counter accrues |vx| * 0.4; an airborne row (10/11) lands
    // on row 12 with counter 0, else advance at > 8 (SUBTRACT 8, not
    // reset) over the 3..=9 cycle.
    let dir = if vx < 0.0 { -1 } else { 1 };
    counter += vx.abs() * 0.4;
    if row == 10 || row == 11 {
        row = 12;
        counter = 0.0;
    } else if counter > 8.0 {
        counter -= 8.0;
        row += 1;
        if row > 9 {
            row = 3;
        }
    }
    (dir, counter, row)
}

/// The Horseman's gallop dust (`NPC.cs:62421-62439`) — one AI tick's
/// arm, drawing the caller's `UpdateNPCs` stream at the exact C#
/// position (module docs: the draw-order argument).
///
/// `frame_row` is `Npc::frame` (= the C# `frame.Y / frame.Height`, the
/// `num4 < 1` clamp included — a loaded sheet row is never 0). `bottom`
/// is `base.Bottom` (`Entity.cs:122-125`), `velocity` the NPC's
/// `base.velocity` (the dust inherits it, `:62433`).
pub(crate) fn horseman_gallop_dust(
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    frame_row: u32,
    bottom: Vec2,
    velocity: Vec2,
) {
    // `:62421-62428`'s gate: `num5 >= 4 && num5 <= 7` — the four gallop
    // rows only. Outside it, NO draws (the stream stays untouched).
    if !(4..=7).contains(&frame_row) {
        return;
    }
    // `:62429-62430` — `base.Bottom + (-30, -8)`, the 60×8 jitter box.
    let base = bottom + Vec2::new(-30.0, -8.0);
    let jitter_box = Vec2::new(60.0, 8.0);
    // `:62431` — the gate draw ALWAYS fires when the row gate passes.
    if rng.next_max(3) != 0 {
        // `:62433` — the two jitters, x then y (the Vector2 ctor's
        // argument order), ONLY on the spawn branch.
        let jx = rng.next_float() * jitter_box.x;
        let jy = rng.next_float() * jitter_box.y;
        // `Dust.NewDustPerfect(pos, 6, base.velocity)` — Alpha 0,
        // default Color, Scale 1 (the C# call's defaults).
        let idx = pool.new_dust_perfect(
            rng,
            ctx,
            world,
            base + Vec2::new(jx, jy),
            6,
            Some(velocity),
            0,
            Color::default(),
            1.0,
        );
        // `:62434-62437` — the post-spawn writes, sentinel included
        // (the C# writes into `Main.dust[6000]` on a refusal too).
        let d = pool.slot_mut(idx);
        d.scale = 0.6;
        d.fade_in = 1.1;
        d.no_gravity = true;
        d.no_light = true;
    }
}

/// `Lighting.AddLight(base.Center, 0.4f, 0.36f, 0.2f)`
/// (`NPC.cs:62420`) — unconditional every Horseman AI tick. The port's
/// feed is the render-side `EntityLights` pair (the `npc_lights.rs`
/// S1-E-5 precedent): `pos` is the NPC top-left (netOffset included at
/// the C# site; the port's single-player netOffset is zero), `w`/`h`
/// the hitbox. `Center` truncates the quotient per axis
/// (`Lighting.cs:140-143`). No RNG.
pub(crate) fn horseman_light(
    kind: NpcKind,
    pos: Vec2,
    w: f32,
    h: f32,
) -> Option<(i32, i32, [f32; 3])> {
    if kind != NpcKind::HeadlessHorseman {
        return None;
    }
    Some((
        tile_x(pos, w, XAnchor::Center),
        tile_y(pos, h, XAnchor::Center),
        [0.4, 0.36, 0.2],
    ))
}

/// `NPC.UpdateNPC`'s `gfxOffY` decay (`NPC.cs:90395-90420`) — runs
/// BEFORE the AI, after the old-target/direction latches: drift toward
/// 0 at `(1 + |velocity.X| / 3) * stepSpeed` per tick, snap at the
/// zero crossing, then clamp ±16. Pure: the caller owns the field
/// (slice A's `gfx_off_y`; `stepSpeed` is the AI's 1f/2f,
/// `:62796`/`:62800`). The render adds the result to the draw Y
/// (`Main.cs:25414` etc.).
pub(crate) fn gfx_off_y_decay(gfx_off_y: f32, vx: f32, step_speed: f32) -> f32 {
    // `:90396` — num6, off the PRE-AI velocity.
    let num6 = 1.0 + vx.abs() / 3.0;
    let mut g = gfx_off_y;
    // `:90397-90412` — the two signed drift arms, each snapping at 0.
    if g > 0.0 {
        g -= num6 * step_speed;
        if g < 0.0 {
            g = 0.0;
        }
    } else if g < 0.0 {
        g += num6 * step_speed;
        if g > 0.0 {
            g = 0.0;
        }
    }
    // `:90413-90420` — the ±16 clamp.
    if g > 16.0 {
        g = 16.0;
    }
    if g < -16.0 {
        g = -16.0;
    }
    g
}

/// `Projectile.SetDefaults`'s 1001 scale (`Projectile.cs:9518`):
/// `1f + (float)Main.rand.Next(51) * 0.01f` — ONE `Next(51)` draw at
/// `NewProjectile` time, i.e. on the `UpdateNPCs` stream right after
/// the lob's `NextFloatDirection()` (`NPC.cs:62417`). Union point 5 in
/// the module docs: roll at the lob's queue site, carry the value to
/// the spawned slot — never at the post-tick drain.
pub(crate) fn horseman_pumpkin_scale(rng: &mut UnifiedRandom) -> f32 {
    1.0 + rng.next_max(51) as f32 * 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1920×1080 screen at the origin, full dust budget — the
    /// `particles.rs` tests' ctx shape (slot 0, no pressure bands).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
            ..SpawnCtx::default()
        }
    }

    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    /// Case 315's airborne reset (`:74480-74485`): row 0, counter 0 —
    /// and the wet+airborne tick STILL turns the sprite first.
    #[test]
    fn s1_f_13b2b_horseman_airborne_resets_the_row() {
        // Dry airborne: no turn (the sprite keeps its last value).
        let (sd, c, r) = horseman_find_frame(3.0, -5.0, false, -1, -1, 6.5, 7);
        assert_eq!((sd, c, r), (-1, 0.0, 0));
        // Wet airborne: the turn block runs, THEN the reset.
        let (sd, c, r) = horseman_find_frame(3.0, -5.0, true, -1, -1, 6.5, 7);
        assert_eq!((sd, c, r), (1, 0.0, 0));
    }

    /// Case 315's turn dead-band (`:74466-74479`): |vx| <= 1 rides the
    /// facing; beyond ±1 the velocity wins.
    #[test]
    fn s1_f_13b2b_horseman_turn_dead_band() {
        let (sd, _, _) = horseman_find_frame(0.5, 0.0, false, -1, 1, 0.0, 1);
        assert_eq!(sd, -1, "inside the band the facing wins");
        let (sd, _, _) = horseman_find_frame(-1.0, 0.0, false, 1, 1, 0.0, 1);
        assert_eq!(sd, 1, "-1.0 is NOT < -1 — the band edge rides the facing");
        let (sd, _, _) = horseman_find_frame(-1.5, 0.0, false, 1, 1, 0.0, 1);
        assert_eq!(sd, -1);
        let (sd, _, _) = horseman_find_frame(1.5, 0.0, false, -1, -1, 0.0, 1);
        assert_eq!(sd, 1);
    }

    /// Case 315's gallop cadence (`:74486-74496`): |vx| per tick,
    /// advance at >= 8, wrap past row 10 to row 1. At vx 3 the advance
    /// lands every 3 ticks (3, 6, 9 >= 8); 11 advances wrap.
    #[test]
    fn s1_f_13b2b_horseman_gallop_cycle_wraps_to_row_one() {
        let (mut sd, mut c, mut r) = (1, 0.0f32, 0u32);
        for tick in 1..=33 {
            (sd, c, r) = horseman_find_frame(3.0, 0.0, false, 1, sd, c, r);
            if tick % 3 != 0 {
                assert_eq!(c, 3.0 * (tick % 3) as f32, "tick {tick} accrues");
            }
        }
        // 33 ticks = 11 advances: rows 1..10 then 11 -> 1.
        assert_eq!((r, c), (1, 0.0), "the wrap goes to row 1, not 0");
        // The >= 8 edge: exactly 8 advances (vx 4, two ticks).
        let (_, c, r) = horseman_find_frame(4.0, 0.0, false, 1, 1, 4.0, 5);
        assert_eq!((c, r), (0.0, 6), "8.0 >= 8.0 advances");
    }

    /// Case 329's vertical arms (`:74376-74387`): rows 3/2, counter 0,
    /// and the sprite direction is NOT touched (the breaks come first).
    #[test]
    fn s1_f_13b2b_hellhound_vertical_rows_keep_the_sprite() {
        let (sd, c, r) = hellhound_find_frame(5.0, 2.0, 1, -1, 3.9, 7);
        assert_eq!((sd, c, r), (-1, 0.0, 3), "falling: row 3, sprite kept");
        let (sd, c, r) = hellhound_find_frame(-5.0, -2.0, -1, 1, 3.9, 7);
        assert_eq!((sd, c, r), (1, 0.0, 2), "rising: row 2, sprite kept");
    }

    /// Case 329's walk cycle (`:74388-74402`): |vx| * 0.4 per tick,
    /// advance at > 4 (strict), clamp rows 2..=9. At vx 5 the counter
    /// steps 2.0: ticks 2, 4, then 6 > 4 advances.
    #[test]
    fn s1_f_13b2b_hellhound_walk_clamps_to_rows_two_through_nine() {
        // From row 0: the first advance lands on 1, clamped up to 2.
        let (mut sd, mut c, mut r) = (-1, 0.0f32, 0u32);
        for _ in 0..3 {
            (sd, c, r) = hellhound_find_frame(5.0, 0.0, 1, sd, c, r);
        }
        assert_eq!((sd, c, r), (1, 0.0, 2), "the low clamp lifts 1 to 2");
        // 8 more advances (24 ticks): 3..9 then 10 -> 2.
        for _ in 0..24 {
            (sd, c, r) = hellhound_find_frame(5.0, 0.0, 1, sd, c, r);
        }
        assert_eq!((r, c), (2, 0.0), "the high clamp wraps 10 to 2");
        // The strict > 4 edge: exactly 4.0 does NOT advance.
        let (_, c, r) = hellhound_find_frame(5.0, 0.0, 1, 1, 2.0, 5);
        assert_eq!((c, r), (4.0, 5), "4.0 is not > 4.0");
    }

    /// The gallop-dust stream witness: replay the exact C# draw order
    /// on a cloned stream and require state equality — the gate draw
    /// always, the two jitters + the seven empty-pool `NewDust`
    /// internals only on the spawn branch, NOTHING outside rows 4..=7.
    /// Seeds 0..8 cover both gate outcomes without pinning one roll.
    #[test]
    fn s1_f_13b2b_gallop_dust_rides_the_stream_in_c_sharp_order() {
        let w = world();
        let bottom = Vec2::new(400.0, 300.0);
        let vel = Vec2::new(2.5, 0.0);
        for seed in 0..8 {
            for row in [3u32, 4, 7, 8] {
                let mut pool = DustPool::new();
                let mut rng = UnifiedRandom::new(seed);
                let mut replay = rng.clone();
                horseman_gallop_dust(&mut rng, &mut pool, &ctx(), &w, row, bottom, vel);
                if (4..=7).contains(&row) {
                    // 1. The gate draw — always.
                    let gate = replay.next_max(3);
                    if gate != 0 {
                        // 2. The jitters, x then y.
                        let jx = replay.next_float() * 60.0;
                        let jy = replay.next_float() * 8.0;
                        // 3. NewDust's internals at slot 0 (box clamped
                        //    5×5, no pressure band, type 6 not dangerous).
                        replay.next_max(1); // position x
                        replay.next_max(1); // position y
                        replay.next_range(-20, 21); // velocity x
                        replay.next_range(-20, 21); // velocity y
                        replay.next_max(3); // frame row
                        replay.next_range(-20, 21); // scale
                        replay.next_range(-10, 6); // the type-6 vy re-roll
                        // 4. The overwrites + the post-spawn writes.
                        let d = pool.slot(0);
                        assert!(d.active, "seed {seed} row {row}: the dust spawned");
                        assert_eq!(d.dust_type, 6);
                        assert_eq!(
                            d.position,
                            bottom + Vec2::new(-30.0, -8.0) + Vec2::new(jx, jy),
                            "the exact jitter position"
                        );
                        assert_eq!(d.velocity, vel, "the NPC velocity override");
                        assert_eq!(d.scale, 0.6);
                        assert_eq!(d.fade_in, 1.1);
                        assert!(d.no_gravity);
                        assert!(d.no_light);
                    }
                }
                assert_eq!(rng, replay, "seed {seed} row {row}: the stream matches");
            }
        }
    }

    /// The off-screen cull (`Dust.cs:97-101`): the gate + jitters still
    /// draw, the internals never run, and the post-spawn writes land on
    /// the SENTINEL slot — the C#'s own refusal shape.
    #[test]
    fn s1_f_13b2b_gallop_dust_culled_spawn_writes_the_sentinel() {
        let w = world();
        let mut pool = DustPool::new();
        // The screen far from the dust: NewDust refuses at the cull.
        let far_ctx = SpawnCtx {
            screen_position: Vec2::new(10000.0, 10000.0),
            ..ctx()
        };
        let bottom = Vec2::new(400.0, 300.0);
        let vel = Vec2::new(2.5, 0.0);
        let mut rng = UnifiedRandom::new(0);
        let mut replay = rng.clone();
        horseman_gallop_dust(&mut rng, &mut pool, &far_ctx, &w, 5, bottom, vel);
        let gate = replay.next_max(3);
        if gate != 0 {
            replay.next_float();
            replay.next_float();
            let sentinel = pool.slot(tr_core::particles::DUST_SENTINEL);
            assert_eq!(sentinel.scale, 0.6, "the refusal writes still land");
            assert_eq!(sentinel.fade_in, 1.1);
            assert!(sentinel.no_gravity && sentinel.no_light);
            assert!(!pool.slot(0).active, "no real slot was taken");
        }
        assert_eq!(rng, replay, "only the gate + jitters drew");
    }

    /// The 1001 scale roll (`Projectile.cs:9518`): exactly one
    /// `Next(51)` draw, `1 + roll * 0.01` in [1.0, 1.5].
    #[test]
    fn s1_f_13b2b_pumpkin_scale_is_one_next_51_draw() {
        for seed in 0..4 {
            let mut rng = UnifiedRandom::new(seed);
            let mut replay = rng.clone();
            let scale = horseman_pumpkin_scale(&mut rng);
            let roll = replay.next_max(51);
            assert_eq!(scale, 1.0 + roll as f32 * 0.01);
            assert!((1.0..=1.5).contains(&scale));
            assert_eq!(rng, replay, "seed {seed}: exactly one draw");
        }
    }

    /// The gfxOffY decay (`NPC.cs:90395-90420`): the (1 + |vx|/3)
    /// rate, the zero-crossing snap, the ±16 clamp, zero stays zero.
    #[test]
    fn s1_f_13b2b_gfx_off_y_decays_and_clamps() {
        // vx 0, step 1: rate 1 — plain decay.
        assert_eq!(gfx_off_y_decay(10.0, 0.0, 1.0), 9.0);
        assert_eq!(gfx_off_y_decay(-10.0, 0.0, 1.0), -9.0);
        // vx 3, step 1: rate 2 — the crossing snaps to 0, not -1.5.
        assert_eq!(gfx_off_y_decay(0.5, 3.0, 1.0), 0.0);
        assert_eq!(gfx_off_y_decay(-0.5, -3.0, 1.0), 0.0);
        // vx 6, step 2 (the AI's :62800 arm): rate (1+2)*2 = 6.
        assert_eq!(gfx_off_y_decay(10.0, 6.0, 2.0), 4.0);
        // Zero is a fixed point (neither signed arm runs).
        assert_eq!(gfx_off_y_decay(0.0, 5.0, 2.0), 0.0);
        // The ±16 clamp fires after the drift.
        assert_eq!(gfx_off_y_decay(17.0, 0.0, 0.0), 16.0);
        assert_eq!(gfx_off_y_decay(-17.0, 0.0, 0.0), -16.0);
    }

    /// The Horseman light (`NPC.cs:62420`): Center anchor, quotient
    /// truncation, (0.4, 0.36, 0.2); the Hellhound emits nothing.
    #[test]
    fn s1_f_13b2b_horseman_light_at_the_center_tile() {
        // The 74×70 box (`npc/data.rs`'s size row) at (160, 176):
        // Center = (197, 211); (int)(197/16) = 12, (int)(211/16) = 13.
        let (x, y, rgb) =
            horseman_light(NpcKind::HeadlessHorseman, Vec2::new(160.0, 176.0), 74.0, 70.0)
                .expect("the Horseman lights every tick");
        assert_eq!((x, y), (12, 13));
        assert_eq!(rgb, [0.4, 0.36, 0.2]);
        assert!(horseman_light(NpcKind::Hellhound, Vec2::new(160.0, 176.0), 60.0, 46.0).is_none());
    }

    /// The proj_data 1001 row (`Projectile.cs:9511-9522` + the
    /// SetDefaults reset): the box, style 194, the flags, the stock
    /// penetrate 1 / scale 1f base (the scale ROLL is the union's).
    #[test]
    fn s1_f_13b2b_proj_data_1001_row() {
        let d = crate::projectile::proj_data(crate::projectile::HORSEMAN_PUMPKIN)
            .expect("the 1001 row exists");
        assert_eq!((d.w, d.h), (26.0, 26.0));
        assert_eq!(d.ai_style, 194);
        assert_eq!(d.penetrate, 1, "the stock :774 penetrate");
        assert_eq!(d.time_left, 360);
        assert_eq!(d.extra_updates, 0);
        assert!(!d.tile_collide);
        assert!(d.ignore_water);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.scale, 1.0, "the row carries the base; the roll is the union's");
        assert!(!d.arrow && !d.trap && !d.minion && !d.sentry && !d.net_important);
    }
}
