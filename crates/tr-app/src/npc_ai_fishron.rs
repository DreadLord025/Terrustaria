//! Duke Fishron's machine — `NPC.AI_069_DukeFishron`
//! (`NPC.cs:48774-49754`), aiStyle 69, `NPCID.DukeFishron` (370).
//! S1-D-3 fishron audit/gap-fill, **code-only mode**: the whole arm is
//! ported here as pure functions over a local state struct + witness
//! tests; the `NpcKind` roster row and the live dispatch wiring are the
//! documented remainder (the S1-F-9b pattern). No other kind's arm is
//! touched.
//!
//! The three-phase machine (one `ai[0]` counter, hp-band hand-offs):
//! * **phase 1** (`ai[0]` -1..4, `NPC.cs:48995-49309`): the materialize
//!   (-1, 75 ticks), the hover/charge cycle (0 ↔ 1, five charges), the
//!   bubble volley (2, the 371 Detonating Bubbles), the sharknado bolt
//!   (3, the two 385 Sharknados), the transform pause (4 → 5). The
//!   `life <= lifeMax * 0.5` band forces state 4 on any cycle boundary.
//! * **phase 2** (`ai[0]` 5..9, `:49310-49561`): damage ×1.2, defense
//!   ×0.8; the faster hover/charge (5 ↔ 6, three charges), the Cthulunado
//!   spin (7, the bubble trail + the 20-speed launch), the aimed
//!   sharknado (8, one 385 with `ai[0]=1, ai[1]=target+1`), the vanish
//!   pause (9 → 10). The expert `life <= lifeMax * 0.15` band forces 9.
//! * **phase 3** (`ai[0]` 10..13, `:49562-49752`): damage ×1.1, defense
//!   0, `chaseable` flips with the teleport cycle; the charge (11), the
//!   teleport (12, re-centres over the player), the spin (13).
//!
//! The enrage gate (`flag6`, `:48868-48875`) — the player above y=800,
//! below the surface line, or inside the horizontal 6400-px margins —
//! doubles damage/defense, drops the cycle to 10 ticks, +6 charge speed,
//! and swaps the bubble volley for the sharknado bolt (the aimed
//! sharknado carries the flag as its third spawn arg).
//!
//! **RNG by construction**: `AI_069` itself draws ZERO `Main.rand`
//! values — every draw in the arm is inside the charge dust loops
//! (`:49180`, `:49448`, `:49660`: per dust j, `Main.rand.NextDouble()`
//! THEN `Main.rand.Next(3, 8)`, 7 dust per charge tick). The port
//! skips the dust (the documented deviation, below) and therefore
//! consumes no stream at all; the bubble's scale draw
//! `Main.rand.Next(80, 121)/100f` happens in the SPAWNED 371's own
//! `AI()` (`:34889`) / the phase-2 trail spawn (`:49481`) — carried on
//! the request, drawn by the wiring when it lands.
//!
//! Documented deviations (also in `docs/ANALYSIS.md`, S1-D-3 fishron):
//! * the dust loops (172-water dust), the sounds (`SoundEngine` 29/4)
//!   and the `oldPos` trail wipe (`:49584-49587`, `:49708-49711`) are
//!   skipped — cosmetic;
//! * `EncourageDespawn(10)` (`:48857`, `NPC.cs:7249-7256`) has no port
//!   `NPC.timeLeft` (the King Slime precedent) — the despawn request is
//!   carried on the output, the timer stays for the wiring;
//! * `TargetClosest` (`:48850`) rides the S1-D-3p helper
//!   (`npc_target.rs`) when wired; here the target re-pick is a
//!   request flag and the caller's target rides unchanged;
//! * the 371/385 spawns are REQUESTS (`FishronRequest`) — the 371
//!   Detonating Bubble and the 385 Sharknado have no port kinds yet;
//!   the wiring spawns them through `NPC.NewNPC`'s bottom-centre
//!   convention (the S1-D-3k servant precedent).

#![allow(dead_code)] // wiring deferred (S1-D-3 fishron, code-only mode)

use std::f32::consts::PI;

/// `NPC.SetDefaults` case 370 (`NPC.cs:13701-13718`): 150x100, aiStyle
/// 69, damage 100, defense 50, lifeMax 60000, knockBackResist 0,
/// noTileCollide + noGravity, npcSlots 10, value 25 gold, boss,
/// netAlways. `defDamage`/`defDefense` latch these at spawn
/// (`NPC.SetDefaults`' tail).
pub const FISHRON_WIDTH: f32 = 150.0;
pub const FISHRON_HEIGHT: f32 = 100.0;
pub const FISHRON_DEF_DAMAGE: i32 = 100;
pub const FISHRON_DEF_DEFENSE: i32 = 50;
pub const FISHRON_LIFE_MAX: i32 = 60000;

/// The machine's live state — the fields `AI_069` reads/writes on the
/// NPC (`ai[0..3]`, `localAI[0]`, position/velocity/direction/rotation/
/// alpha/chaseable/damage/defense/life). One fixed step per call.
#[derive(Clone, Copy, Debug)]
pub struct FishronState {
    /// `this.ai[0]` — the phase state (-1 materialize, 0..13).
    pub ai0: f32,
    /// `this.ai[1]` — the hover side offset (±300, phase 3 ±360).
    pub ai1: f32,
    /// `this.ai[2]` — the state timer.
    pub ai2: f32,
    /// `this.ai[3]` — the cycle counter (charges/bubbles/sharknado).
    pub ai3: f32,
    /// `this.localAI[0]` — the first-tick latch (`:48877-48887`).
    pub local_ai0: f32,
    /// `base.position` (top-left) — `base.Center` derives from it.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `base.direction` (+1 right / -1 left).
    pub direction: i32,
    /// `this.spriteDirection` — Fishron faces AGAINST `direction`.
    pub sprite_direction: i32,
    pub rotation: f32,
    /// `this.alpha` (0 opaque .. 255 invisible).
    pub alpha: i32,
    /// `this.chaseable` — phase 3 flips it with the teleport cycle.
    pub chaseable: bool,
    /// `this.damage` — rewritten every tick from `defDamage`.
    pub damage: i32,
    /// `this.defense` — rewritten every tick from `defDefense`.
    pub defense: i32,
    pub life: i32,
    pub life_max: i32,
    /// `this.defDamage`/`this.defDefense` (the `SetDefaults` latches).
    pub def_damage: i32,
    pub def_defense: i32,
}

impl Default for FishronState {
    fn default() -> Self {
        Self {
            ai0: 0.0,
            ai1: 0.0,
            ai2: 0.0,
            ai3: 0.0,
            local_ai0: 0.0,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            sprite_direction: -1,
            rotation: 0.0,
            alpha: 0,
            chaseable: true,
            damage: FISHRON_DEF_DAMAGE,
            defense: FISHRON_DEF_DEFENSE,
            life: FISHRON_LIFE_MAX,
            life_max: FISHRON_LIFE_MAX,
            def_damage: FISHRON_DEF_DAMAGE,
            def_defense: FISHRON_DEF_DEFENSE,
        }
    }
}

/// The per-tick environment the arm reads from `Main`/the target.
#[derive(Clone, Copy, Debug)]
pub struct FishronEnv {
    /// `Main.expertMode` (`:48776`).
    pub expert: bool,
    /// The target player's centre (`Main.player[this.target].Center`).
    pub player_cx: f32,
    pub player_cy: f32,
    /// The target player's top-left (`player.position`, the enrage gate).
    pub player_x: f32,
    pub player_y: f32,
    pub player_dead: bool,
    /// `this.target` — the re-pick gate reads it (`:48848`).
    pub target: i32,
    /// `player.active` (`:48848`).
    pub player_active: bool,
    /// `Main.worldSurface` in TILES (the gate multiplies by 16, `:48868`).
    pub world_surface: f64,
    /// `Main.maxTilesX`.
    pub max_tiles_x: i32,
    /// `Collision.SolidCollision(position, width, height)` (`:48978`) —
    /// the wiring probes the world; the pure core takes the answer.
    pub solid_collision: bool,
}

/// A spawn the arm requests (`NPC.NewNPC`/`Projectile.NewProjectile`).
/// The wiring resolves them; the pure core never touches the pools.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FishronRequest {
    /// `NPC.NewNPC(..., (int)p.X, (int)p.Y + 45, 371)` (`:49246`) —
    /// the Detonating Bubble at the aim point (phase-1 volley).
    Bubble { x: i32, y: i32 },
    /// The phase-2 spin trail's bubble (`:49476-49481`): the spawned
    /// 371 inherits the target + the rotated 6-speed velocity + the
    /// `ai[3] = Main.rand.Next(80, 121) / 100f` scale draw (the wiring
    /// draws it — the RNG order cite is `:49481`).
    TrailBubble {
        x: i32,
        y: i32,
        vx: f32,
        vy: f32,
        target: i32,
    },
    /// `Projectile.NewProjectile(..., 385)` — the Sharknado bolt.
    /// Phase 1 (`:49279-49280`): two bolts at ±2/8 from the muzzle;
    /// phase 2 (`:49505`): one still bolt at the centre with
    /// `ai[0] = 1`, `ai[1] = target + 1`, `localAI[0] = enrage`.
    Sharknado {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        ai0: f32,
        ai1: f32,
        local_ai0: f32,
    },
    /// `this.TargetClosest()` (`:48850`) — the wiring runs the S1-D-3p
    /// `npc_target::target_closest` helper.
    TargetClosest,
    /// `this.EncourageDespawn(10)` (`:48857`) — no port `timeLeft`
    /// yet (the documented deviation).
    EncourageDespawn(i32),
}

/// The per-tick output beside the state mutations.
#[derive(Clone, Debug, Default)]
pub struct FishronOut {
    pub requests: Vec<FishronRequest>,
    /// `this.netUpdate = true` was written this tick.
    pub net_update: bool,
    /// `this.dontTakeDamage = !flag7` (`:49753`) — the tail write.
    pub dont_take_damage: bool,
}

/// `Vector2.Normalize` — XNA divides by the length UNGUARDED (a zero
/// input yields NaN). The port guards the degenerate exact-overlap
/// case to the zero vector instead (the documented deviation — it
/// only differs when the aim/hover vector is exactly zero, and keeps
/// the witness tests NaN-free).
fn normalize(x: f32, y: f32) -> (f32, f32) {
    let len = (x * x + y * y).sqrt();
    if len > 0.0 && len.is_finite() {
        (x / len, y / len)
    } else {
        (0.0, 0.0)
    }
}

/// `Math.Sign(float)` — -1/0/+1.
fn sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `MathHelper.Lerp(a, b, t)` — `a + (b - a) * t`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Vector2.RotatedBy(r)` — the XNA rotation matrix
/// (`cos`/`sin`, `x' = x·cos − y·sin`, `y' = x·sin + y·cos`).
fn rotated(x: f32, y: f32, r: f32) -> (f32, f32) {
    let c = r.cos();
    let s = r.sin();
    (x * c - y * s, x * s + y * c)
}

/// `NPC.SimpleFlyMovement` (`NPC.cs:77397-77431`) — the per-axis
/// approach with the sign-reversal double-add. The inline copies in
/// states 0/2/5 (`:49052-49083` etc.) are byte-identical to this.
pub fn simple_fly_movement(vx: &mut f32, vy: &mut f32, dx: f32, dy: f32, speed: f32) {
    if *vx < dx {
        *vx += speed;
        if *vx < 0.0 && dx > 0.0 {
            *vx += speed;
        }
    } else if *vx > dx {
        *vx -= speed;
        if *vx > 0.0 && dx < 0.0 {
            *vx -= speed;
        }
    }
    if *vy < dy {
        *vy += speed;
        if *vy < 0.0 && dy > 0.0 {
            *vy += speed;
        }
    } else if *vy > dy {
        *vy -= speed;
        if *vy > 0.0 && dy < 0.0 {
            *vy -= speed;
        }
    }
}

/// The hover approach toward `player + (side, -200)` at the cycle speed
/// (`:49051`/`:49203`/`:49316`/`:49577` — the desired vector subtracts
/// the CURRENT velocity, then `SimpleFlyMovement` walks toward it).
fn hover_toward(s: &mut FishronState, env: &FishronEnv, side: f32, speed: f32, accel: f32) {
    let (cx, cy) = center(s);
    let (nx, ny) = normalize(
        env.player_cx + side - cx - s.vx,
        env.player_cy - 200.0 - cy - s.vy,
    );
    simple_fly_movement(&mut s.vx, &mut s.vy, nx * speed, ny * speed, accel);
}

fn center(s: &FishronState) -> (f32, f32) {
    (s.x + FISHRON_WIDTH / 2.0, s.y + FISHRON_HEIGHT / 2.0)
}

/// The charge launch shared by states 0→1 (`:49135-49150`), 5→6
/// (`:49396-49411`) and 10→11 (`:49618-49633`): the velocity snaps to
/// the player aim × the charge speed, the rotation follows the
/// velocity, the facing flips add π when the sprite was facing right.
fn launch_charge(s: &mut FishronState, env: &FishronEnv, charge_speed: f32, facing: i32) {
    let (cx, cy) = center(s);
    let (nx, ny) = normalize(env.player_cx - cx, env.player_cy - cy);
    s.vx = nx * charge_speed;
    s.vy = ny * charge_speed;
    s.rotation = s.vy.atan2(s.vx);
    if facing != 0 {
        s.direction = facing;
        if s.sprite_direction == 1 {
            s.rotation += PI;
        }
        s.sprite_direction = -s.direction;
    }
}

/// The facing block shared by the hover states (`:49084-49097`,
/// `:49349-49362`, `:49578-49595`): the first-tick direction change
/// adds π, every sprite flip adds π. Returns the facing sign (`num26`/
/// `num31`/`num36`) for the charge launch.
fn hover_facing(s: &mut FishronState, env: &FishronEnv, first_tick_flip: bool) -> i32 {
    let (cx, _) = center(s);
    let facing = sign(env.player_cx - cx);
    if facing != 0 {
        if first_tick_flip && s.ai2 == 0.0 && facing != s.direction {
            s.rotation += PI;
        }
        s.direction = facing;
        if s.sprite_direction != -s.direction {
            s.rotation += PI;
        }
        s.sprite_direction = -s.direction;
    }
    facing
}

/// One tick of `AI_069_DukeFishron` (`NPC.cs:48774-49754`). The order
/// is the source's: the hp-band stat block, the tunables, the target
/// re-pick + dead/despawn arm, the enrage gate, the first-tick latch,
/// the rotation tracking, the solid-collision alpha, the state switch,
/// the `dontTakeDamage` tail.
pub fn fishron_ai_step(s: &mut FishronState, env: &FishronEnv) -> FishronOut {
    let mut out = FishronOut::default();
    let expert = env.expert; // `:48776`
    let num = if expert { 1.2f32 } else { 1.0 }; // `:48777`
    let flag = (s.life as f64) <= (s.life_max as f64) * 0.5; // `:48778`
    let flag2 = expert && (s.life as f64) <= (s.life_max as f64) * 0.15; // `:48779`
    let flag3 = s.ai0 > 4.0; // `:48780`
    let flag4 = s.ai0 > 9.0; // `:48781`
    let flag5 = s.ai3 < ((if flag3 { 3 } else { 5 }) * 2) as f32; // `:48782`

    // The phase stat block (`:48783-48797`) — f32 multiplies then the
    // truncating int cast, exactly as the C#.
    if flag4 {
        s.damage = (s.def_damage as f32 * 1.1 * num) as i32;
        s.defense = 0;
    } else if flag3 {
        s.damage = (s.def_damage as f32 * 1.2 * num) as i32;
        s.defense = (s.def_defense as f32 * 0.8) as i32;
    } else {
        s.damage = s.def_damage;
        s.defense = s.def_defense;
    }

    // The cycle tunables (`:48798-48816`): num2 the hover ticks, num3
    // the accel, num4 the hover speed.
    let mut num2: i32 = if expert { 40 } else { 60 };
    let mut num3: f32 = if expert { 0.55 } else { 0.45 };
    let mut num4: f32 = if expert { 8.5 } else { 7.5 };
    if flag4 {
        num3 = 0.7;
        num4 = 12.0;
        num2 = 30;
    } else if flag3 & flag5 {
        // the C#'s non-short-circuit `&` (`:48807`) — same result.
        num3 = if expert { 0.6 } else { 0.5 };
        num4 = if expert { 10.0 } else { 8.0 };
        num2 = if expert { 40 } else { 20 };
    } else if flag5 && !flag3 && !flag4 {
        num2 = 30;
    }

    // The charge tunables (`:48817-48831`): num5 the charge ticks,
    // num6 the charge speed.
    let mut num5: i32 = if expert { 28 } else { 30 };
    let mut num6: f32 = if expert { 17.0 } else { 16.0 };
    if flag4 {
        num5 = 25;
        num6 = 27.0;
    } else if flag5 & flag3 {
        num5 = if expert { 27 } else { 30 };
        if expert {
            num6 = 21.0;
        }
    }

    // The fixed constants (`:48832-48845`).
    let num7: i32 = 80; // the bubble volley ticks
    let num8: i32 = 4; // the volley's spawn cadence
    let num9: f32 = 0.3; // the volley's accel
    let num10: f32 = 5.0; // the volley's speed
    let num11: i32 = 90; // the sharknado bolt ticks
    let num12: i32 = 180; // the phase-1 transform ticks
    let num13: i32 = 180; // the phase-2 vanish ticks
    let num14: i32 = 30; // the teleport ticks
    let num15: i32 = 120; // the spin ticks
    let num16: i32 = 4; // the spin trail's spawn cadence
    let num17: f32 = 6.0; // the trail bubble's speed
    let num18: f32 = 20.0; // the Cthulunado launch speed
    let num19: f32 = PI * 2.0 / (num15 / 2) as f32; // the spin step
    let num20: i32 = 75; // the materialize ticks

    // `:48846` — the C# refreshes `center` in the teleport (`:49701`);
    // only the X rides a later read (num40), so only cx is mutable here.
    let (mut cx, cy) = center(s);

    // The target re-pick gate (`:48848-48853`). The C# re-reads the
    // player after TargetClosest; the pure core carries the caller's
    // target (the request rides the wiring).
    let dist = ((env.player_cx - cx).powi(2) + (env.player_cy - cy).powi(2)).sqrt();
    if env.target < 0 || env.target == 255 || env.player_dead || !env.player_active || dist > 5600.0
    {
        out.requests.push(FishronRequest::TargetClosest);
        out.net_update = true;
    }

    // The dead/far despawn arm (`:48854-48867`).
    if env.player_dead || dist > 5600.0 {
        s.vy -= 0.4;
        out.requests.push(FishronRequest::EncourageDespawn(10));
        if s.ai0 > 4.0 {
            s.ai0 = 5.0;
        } else {
            s.ai0 = 0.0;
        }
        s.ai2 = 0.0;
    }

    // The enrage gate (`:48868-48875`) — flag6.
    let flag6 = env.player_y < 800.0
        || (env.player_y as f64) > env.world_surface * 16.0
        || (env.player_x > 6400.0 && env.player_x < (env.max_tiles_x * 16 - 6400) as f32);
    if flag6 {
        num2 = 10;
        s.damage = s.def_damage * 2;
        s.defense = s.def_defense * 2;
        num6 += 6.0;
    }

    // flag7 — the damageable latch (`:48876`, the tail `:49753`).
    let mut flag7 = true;

    // The first-tick latch (`:48877-48887`) — single-player takes the
    // `Main.netMode != 1` branch.
    if s.local_ai0 == 0.0 {
        s.local_ai0 = 1.0;
        s.alpha = 255;
        s.rotation = 0.0;
        s.ai0 = -1.0;
        out.net_update = true;
    }

    // The rotation tracking (`:48888-48975`).
    let mut num21 = (env.player_cy - cy).atan2(env.player_cx - cx);
    if s.sprite_direction == 1 {
        num21 += PI;
    }
    if num21 < 0.0 {
        num21 += PI * 2.0;
    }
    if num21 > PI * 2.0 {
        num21 -= PI * 2.0;
    }
    if s.ai0 == -1.0 || s.ai0 == 3.0 || s.ai0 == 4.0 || s.ai0 == 8.0 {
        num21 = 0.0; // `:48901-48916`
    }
    let mut num22 = 0.04f32;
    if s.ai0 == 1.0 || s.ai0 == 6.0 || s.ai0 == 7.0 {
        num22 = 0.0; // `:48918-48925`
    }
    if s.ai0 == 3.0 || s.ai0 == 4.0 || s.ai0 == 8.0 {
        num22 = 0.01; // `:48926-48937`
    }
    if s.rotation < num21 {
        if (num21 - s.rotation) as f64 > std::f64::consts::PI {
            s.rotation -= num22;
        } else {
            s.rotation += num22;
        }
    }
    if s.rotation > num21 {
        if (s.rotation - num21) as f64 > std::f64::consts::PI {
            s.rotation += num22;
        } else {
            s.rotation -= num22;
        }
    }
    if s.rotation > num21 - num22 && s.rotation < num21 + num22 {
        s.rotation = num21;
    }
    if s.rotation < 0.0 {
        s.rotation += PI * 2.0;
    }
    if s.rotation > PI * 2.0 {
        s.rotation -= PI * 2.0;
    }
    if s.rotation > num21 - num22 && s.rotation < num21 + num22 {
        s.rotation = num21;
    }

    // The solid-collision alpha (`:48976-48994`) — every state except
    // the materialize and phase 3 (>= 9).
    if s.ai0 != -1.0 && s.ai0 < 9.0 {
        if env.solid_collision {
            s.alpha += 15;
        } else {
            s.alpha -= 15;
        }
        s.alpha = s.alpha.clamp(0, 150);
    }

    // ---- the state switch (`:48995-49752`) ----
    if s.ai0 == -1.0 {
        // The materialize (`:48995-49044`): damp, face the player,
        // rise after tick 20, the 36-dust ring at tick 60 (skipped),
        // hand off to the hover at tick 75.
        flag7 = false;
        s.vx *= 0.98;
        s.vy *= 0.98;
        let num23 = sign(env.player_cx - cx);
        if num23 != 0 {
            s.direction = num23;
            s.sprite_direction = -s.direction;
        }
        if s.ai2 > 20.0 {
            s.vy = -2.0;
            s.alpha -= 5;
            if env.solid_collision {
                s.alpha += 15;
            }
            s.alpha = s.alpha.clamp(0, 150);
        }
        // `:49022-49035`: the tick-60 dust ring + sound 29 — skipped
        // (cosmetic, no RNG: the ring is deterministic geometry).
        s.ai2 += 1.0;
        if s.ai2 >= num20 as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 0.0 && !env.player_dead {
        // The phase-1 hover + cycle picker (`:49045-49173`).
        if s.ai1 == 0.0 {
            s.ai1 = 300.0 * sign(cx - env.player_cx) as f32;
        }
        hover_toward(s, env, s.ai1, num4, num3);
        let num26 = hover_facing(s, env, true);
        s.ai2 += 1.0;
        if s.ai2 >= num2 as f32 {
            // The cycle table (`:49101-49132`): ai[3] 0..9 → charge,
            // 10 → bubbles (reset 1), 11 → sharknado (reset 0); the
            // enrage swaps bubbles → sharknado; the half-hp band
            // forces the transform.
            let mut num27 = 0;
            match s.ai3 as i32 {
                0..=9 => num27 = 1,
                10 => {
                    s.ai3 = 1.0;
                    num27 = 2;
                }
                11 => {
                    s.ai3 = 0.0;
                    num27 = 3;
                }
                _ => {}
            }
            if flag6 && num27 == 2 {
                num27 = 3;
            }
            if flag {
                num27 = 4;
            }
            match num27 {
                1 => {
                    s.ai0 = 1.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                    launch_charge(s, env, num6, num26);
                }
                2 => {
                    s.ai0 = 2.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                3 => {
                    s.ai0 = 3.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                    if flag6 {
                        s.ai2 = (num11 - 40) as f32;
                    }
                }
                4 => {
                    s.ai0 = 4.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                _ => {}
            }
            out.net_update = true;
        }
    } else if s.ai0 == 1.0 {
        // The phase-1 charge (`:49174-49196`): the 7-dust trail
        // (skipped — the arm's ONLY RNG draws live there), then the
        // num5-tick hand-off with ai[3] += 2.
        s.ai2 += 1.0;
        if s.ai2 >= num5 as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 += 2.0;
            out.net_update = true;
        }
    } else if s.ai0 == 2.0 {
        // The bubble volley (`:49197-49267`): the slower hover
        // (0.3/5), a 371 every 4 ticks at the aim muzzle + 45 down,
        // the facing flip WITHOUT the first-tick π (`:49249-49258`),
        // 80 ticks then back to the hover.
        if s.ai1 == 0.0 {
            s.ai1 = 300.0 * sign(cx - env.player_cx) as f32;
        }
        hover_toward(s, env, s.ai1, num10, num9);
        if s.ai2 % num8 as f32 == 0.0 {
            let (nx, ny) = normalize(env.player_cx - cx, env.player_cy - cy);
            let mx = nx * (FISHRON_WIDTH + 20.0) / 2.0 + cx;
            let my = ny * (FISHRON_WIDTH + 20.0) / 2.0 + cy;
            out.requests.push(FishronRequest::Bubble {
                x: mx as i32,
                y: my as i32 + 45,
            });
        }
        let num30 = sign(env.player_cx - cx);
        if num30 != 0 {
            s.direction = num30;
            if s.sprite_direction != -s.direction {
                s.rotation += PI;
            }
            s.sprite_direction = -s.direction;
        }
        s.ai2 += 1.0;
        if s.ai2 >= num7 as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 3.0 {
        // The sharknado bolt (`:49268-49290`): damp + the Y lerp, the
        // two 385 bolts at tick 60 from the rotation muzzle, 90 ticks.
        s.vx *= 0.98;
        s.vy *= 0.98;
        s.vy = lerp(s.vy, 0.0, 0.02);
        if s.ai2 == (num11 - 30) as f32 {
            // `:49278`: rotation.ToRotationVector2() * (UnitX *
            // direction) — the COMPONENT-WISE multiply keeps only the
            // X term: (cos(rot) * direction, 0).
            let mx = s.rotation.cos() * s.direction as f32 * (FISHRON_WIDTH + 20.0) / 2.0 + cx;
            let my = cy;
            out.requests.push(FishronRequest::Sharknado {
                x: mx,
                y: my,
                vx: s.direction as f32 * 2.0,
                vy: 8.0,
                ai0: 0.0,
                ai1: 0.0,
                local_ai0: 0.0,
            });
            out.requests.push(FishronRequest::Sharknado {
                x: mx,
                y: my,
                vx: -s.direction as f32 * 2.0,
                vy: 8.0,
                ai0: 0.0,
                ai1: 0.0,
                local_ai0: 0.0,
            });
        }
        s.ai2 += 1.0;
        if s.ai2 >= num11 as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 4.0 {
        // The phase-1 transform pause (`:49291-49309`): invulnerable,
        // damp + lerp, 180 ticks then phase 2 (ai[3] reset).
        flag7 = false;
        s.vx *= 0.98;
        s.vy *= 0.98;
        s.vy = lerp(s.vy, 0.0, 0.02);
        s.ai2 += 1.0;
        if s.ai2 >= num12 as f32 {
            s.ai0 = 5.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 5.0 && !env.player_dead {
        // The phase-2 hover + cycle picker (`:49310-49441`).
        if s.ai1 == 0.0 {
            s.ai1 = 300.0 * sign(cx - env.player_cx) as f32;
        }
        hover_toward(s, env, s.ai1, num4, num3);
        let num31 = hover_facing(s, env, true);
        s.ai2 += 1.0;
        if s.ai2 >= num2 as f32 {
            // The cycle table (`:49366-49393`): ai[3] 0..5 → charge,
            // 6 → Cthulunado (reset 1), 7 → aimed sharknado (reset 0);
            // the expert 15 % band forces the vanish BEFORE the
            // enrage swap (the source's order).
            let mut num32 = 0;
            match s.ai3 as i32 {
                0..=5 => num32 = 1,
                6 => {
                    s.ai3 = 1.0;
                    num32 = 2;
                }
                7 => {
                    s.ai3 = 0.0;
                    num32 = 3;
                }
                _ => {}
            }
            if flag2 {
                num32 = 4;
            }
            if flag6 && num32 == 2 {
                num32 = 3;
            }
            match num32 {
                1 => {
                    s.ai0 = 6.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                    launch_charge(s, env, num6, num31);
                }
                2 => {
                    // The Cthulunado launch (`:49412-49427`): the
                    // 20-speed snap BEFORE the state write.
                    launch_charge(s, env, num18, num31);
                    s.ai0 = 7.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                3 => {
                    s.ai0 = 8.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                4 => {
                    s.ai0 = 9.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                _ => {}
            }
            out.net_update = true;
        }
    } else if s.ai0 == 6.0 {
        // The phase-2 charge (`:49442-49464`): the dust trail skipped,
        // num5 ticks then ai[3] += 2 back to 5.
        s.ai2 += 1.0;
        if s.ai2 >= num5 as f32 {
            s.ai0 = 5.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 += 2.0;
            out.net_update = true;
        }
    } else if s.ai0 == 7.0 {
        // The Cthulunado spin (`:49465-49494`): a trail bubble every 4
        // ticks (the rotated 6-speed + the target inherit + the scale
        // draw), the velocity/rotation spin by −num19·direction, 120
        // ticks.
        if s.ai2 % num16 as f32 == 0.0 {
            let (nx, ny) = normalize(s.vx, s.vy);
            let mx = nx * (FISHRON_WIDTH + 20.0) / 2.0 + cx;
            let my = ny * (FISHRON_WIDTH + 20.0) / 2.0 + cy;
            let (bx, by) = rotated(nx, ny, PI / 2.0 * s.direction as f32);
            out.requests.push(FishronRequest::TrailBubble {
                x: mx as i32,
                y: my as i32 + 45,
                vx: bx * num17,
                vy: by * num17,
                target: env.target,
            });
        }
        let (rx, ry) = rotated(s.vx, s.vy, (0.0 - num19) * s.direction as f32);
        s.vx = rx;
        s.vy = ry;
        s.rotation -= num19 * s.direction as f32;
        s.ai2 += 1.0;
        if s.ai2 >= num15 as f32 {
            s.ai0 = 5.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 8.0 {
        // The aimed sharknado (`:49495-49515`): damp + lerp, ONE still
        // 385 at the centre at tick 60 carrying ai[0]=1,
        // ai[1]=target+1, localAI[0]=enrage, 90 ticks.
        s.vx *= 0.98;
        s.vy *= 0.98;
        s.vy = lerp(s.vy, 0.0, 0.02);
        if s.ai2 == (num11 - 30) as f32 {
            out.requests.push(FishronRequest::Sharknado {
                x: cx,
                y: cy,
                vx: 0.0,
                vy: 0.0,
                ai0: 1.0,
                ai1: (env.target + 1) as f32,
                local_ai0: if flag6 { 1.0 } else { 0.0 },
            });
        }
        s.ai2 += 1.0;
        if s.ai2 >= num11 as f32 {
            s.ai0 = 5.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 9.0 {
        // The phase-2 vanish (`:49516-49561`): invulnerable; the
        // solid-alpha until tick 90, then the +4 fade to 255; damp +
        // lerp; 180 ticks then phase 3 (ai[3] reset).
        flag7 = false;
        if s.ai2 < (num13 - 90) as f32 {
            if env.solid_collision {
                s.alpha += 15;
            } else {
                s.alpha -= 15;
            }
            s.alpha = s.alpha.clamp(0, 150);
        } else if s.alpha < 255 {
            s.alpha += 4;
            if s.alpha > 255 {
                s.alpha = 255;
            }
        }
        s.vx *= 0.98;
        s.vy *= 0.98;
        s.vy = lerp(s.vy, 0.0, 0.02);
        s.ai2 += 1.0;
        if s.ai2 >= num13 as f32 {
            s.ai0 = 10.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 10.0 && !env.player_dead {
        // The phase-3 hover + cycle picker (`:49562-49647`):
        // unchaseable, the +25 fade, the ±360 side, SimpleFlyMovement,
        // the cycle table (0/2/3/5/6/7 → charge, 1/4/8 → teleport;
        // num37 == 3 is DEAD — no table row writes 3).
        s.chaseable = false;
        if s.alpha < 255 {
            s.alpha += 25;
            if s.alpha > 255 {
                s.alpha = 255;
            }
        }
        if s.ai1 == 0.0 {
            s.ai1 = 360.0 * sign(cx - env.player_cx) as f32;
        }
        hover_toward(s, env, s.ai1, num4, num3);
        let num36 = hover_facing(s, env, true);
        // (`:49584-49587`: the first-tick flip also wipes oldPos —
        // the trail is unported, skipped.)
        s.ai2 += 1.0;
        if s.ai2 >= num2 as f32 {
            let mut num37 = 0;
            match s.ai3 as i32 {
                0 | 2 | 3 | 5 | 6 | 7 => num37 = 1,
                1 | 4 | 8 => num37 = 2,
                _ => {}
            }
            match num37 {
                1 => {
                    s.ai0 = 11.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                    launch_charge(s, env, num6, num36);
                }
                2 => {
                    s.ai0 = 12.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                // `:49639-49643`: case 3 → state 13 exists but no
                // table row picks it (the spin is unreachable in
                // vanilla phase 3) — ported as-is.
                3 => {
                    s.ai0 = 13.0;
                    s.ai1 = 0.0;
                    s.ai2 = 0.0;
                }
                _ => {}
            }
            out.net_update = true;
        }
    } else if s.ai0 == 11.0 {
        // The phase-3 charge (`:49648-49676`): chaseable again, the
        // −25 fade-in, the dust skipped, num5 ticks then ai[3]++ (NO
        // wrap here — the wrap lives in the teleport).
        s.chaseable = true;
        s.alpha -= 25;
        if s.alpha < 0 {
            s.alpha = 0;
        }
        s.ai2 += 1.0;
        if s.ai2 >= num5 as f32 {
            s.ai0 = 10.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 += 1.0;
            out.net_update = true;
        }
    } else if s.ai0 == 12.0 {
        // The teleport (`:49677-49734`): invulnerable + unchaseable,
        // the +17 fade, damp + lerp; at tick 15 re-centre over the
        // player (the ±300 side latched HERE if still 0), the facing
        // block, 30 ticks then ai[3]++ with the 9-wrap.
        flag7 = false;
        s.chaseable = false;
        if s.alpha < 255 {
            s.alpha += 17;
            if s.alpha > 255 {
                s.alpha = 255;
            }
        }
        s.vx *= 0.98;
        s.vy *= 0.98;
        s.vy = lerp(s.vy, 0.0, 0.02);
        if s.ai2 == (num14 / 2) as f32 {
            if s.ai1 == 0.0 {
                s.ai1 = 300.0 * sign(cx - env.player_cx) as f32;
            }
            // `:49701` — base.Center = player + (−ai[1], −200); the
            // position follows the centre.
            let ncx = env.player_cx - s.ai1;
            let ncy = env.player_cy - 200.0;
            s.x = ncx - FISHRON_WIDTH / 2.0;
            s.y = ncy - FISHRON_HEIGHT / 2.0;
            cx = ncx; // the C#'s `center = ...` refresh (`:49701`)
            let num40 = sign(env.player_cx - cx);
            if num40 != 0 {
                // (`:49705-49712`: the ai2==0 flip is dead here —
                // ai2 is 15 — and the oldPos wipe is skipped.)
                if s.ai2 == 0.0 && num40 != s.direction {
                    s.rotation += PI;
                }
                s.direction = num40;
                if s.sprite_direction != -s.direction {
                    s.rotation += PI;
                }
                s.sprite_direction = -s.direction;
            }
        }
        s.ai2 += 1.0;
        if s.ai2 >= num14 as f32 {
            s.ai0 = 10.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 += 1.0;
            if s.ai3 >= 9.0 {
                s.ai3 = 0.0;
            }
            out.net_update = true;
        }
    } else if s.ai0 == 13.0 {
        // The phase-3 spin (`:49735-49752`) — reachable only through
        // the dead case-3 row above; ported for completeness.
        let (rx, ry) = rotated(s.vx, s.vy, (0.0 - num19) * s.direction as f32);
        s.vx = rx;
        s.vy = ry;
        s.rotation -= num19 * s.direction as f32;
        s.ai2 += 1.0;
        if s.ai2 >= num15 as f32 {
            s.ai0 = 10.0;
            s.ai1 = 0.0;
            s.ai2 = 0.0;
            s.ai3 += 1.0;
            out.net_update = true;
        }
    }

    // The tail (`:49753`).
    out.dont_take_damage = !flag7;
    out
}

/// The kind-370-gated seam: the ONE place the live dispatch will call
/// once `NpcKind::DukeFishron` lands (`update_kind_ai`'s new arm —
/// `npc.rs`, the wiring follow-up). The gate keys on the vanilla
/// `NPC.type` 370 ONLY, so no `aiStyle` arm for another kind is
/// affected; every other type returns false without touching state.
pub fn fishron_seam_applies(vanilla_type: u16) -> bool {
    vanilla_type == 370
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN NOT RUN (code-only mode, no compiler
    //! invoked). Each pins one cited block of `AI_069_DukeFishron`.
    use super::*;

    /// An env with the player at the given centre, ALIVE and inside
    /// every non-enrage gate: position y = 4000 (800 <= y <=
    /// worldSurface*16 = 19200) and x < 6400 (outside the interior
    /// margin band, `:48868`). The enrage tests override `player_y`.
    fn env_at(player_cx: f32, player_cy: f32) -> FishronEnv {
        FishronEnv {
            expert: false,
            player_cx,
            player_cy,
            player_x: player_cx - 9.0,
            player_y: 4000.0,
            player_dead: false,
            target: 0,
            player_active: true,
            world_surface: 1200.0,
            max_tiles_x: 8400,
            solid_collision: false,
        }
    }

    fn state_at(x: f32, y: f32) -> FishronState {
        FishronState {
            x,
            y,
            ..Default::default()
        }
    }

    #[test]
    fn the_set_defaults_row_matches_the_csharp() {
        // `NPC.cs:13701-13718`.
        assert_eq!(FISHRON_WIDTH, 150.0);
        assert_eq!(FISHRON_HEIGHT, 100.0);
        assert_eq!(FISHRON_DEF_DAMAGE, 100);
        assert_eq!(FISHRON_DEF_DEFENSE, 50);
        assert_eq!(FISHRON_LIFE_MAX, 60000);
        let s = FishronState::default();
        assert_eq!(s.damage, 100);
        assert_eq!(s.defense, 50);
    }

    #[test]
    fn the_first_tick_latches_the_materialize() {
        // `:48877-48887`: localAI[0] 0 → 1, alpha 255, rotation 0,
        // ai[0] = -1 (single-player).
        let mut s = state_at(0.0, 0.0);
        let env = env_at(400.0, 0.0);
        let out = fishron_ai_step(&mut s, &env);
        assert_eq!(s.local_ai0, 1.0);
        assert_eq!(s.ai0, -1.0);
        // The rise branch needs ai[2] > 20 (`:49005`) — tick 0 keeps
        // the latched 255 (the solid-alpha block skips state -1).
        assert_eq!(s.alpha, 255);
        assert!(out.net_update);
        assert!(out.dont_take_damage); // flag7 = false in state -1
    }

    #[test]
    fn the_materialize_hands_off_at_tick_75() {
        // `:49036-49043`: ai[2] >= 75 → ai[0] = 0.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = -1.0;
        s.ai2 = 74.0;
        let env = env_at(400.0, 0.0);
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai2, 0.0);
        // The rise runs past tick 20 (`:49005-49007`).
        assert_eq!(s.vy, -2.0);
    }

    #[test]
    fn the_phase_stat_bands_match_the_csharp() {
        // `:48783-48797`: phase 1 100/50; phase 2 (ai0 > 4)
        // 120/40 classic; phase 3 (ai0 > 9) 110/0 classic; expert
        // ×1.2 → 144 / 132.
        let env = env_at(400.0, 0.0);
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!((s.damage, s.defense), (100, 50));
        s.ai0 = 5.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!((s.damage, s.defense), (120, 40));
        s.ai0 = 10.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!((s.damage, s.defense), (110, 0));
        let mut e = env;
        e.expert = true;
        s.ai0 = 5.0;
        fishron_ai_step(&mut s, &e);
        assert_eq!((s.damage, s.defense), (144, 40));
        s.ai0 = 10.0;
        fishron_ai_step(&mut s, &e);
        assert_eq!((s.damage, s.defense), (132, 0));
    }

    #[test]
    fn the_enrage_gate_doubles_the_stats_and_shortens_the_cycle() {
        // `:48868-48875`: the player above y=800 → damage 200,
        // defense 100, num2 = 10, num6 = 22.
        let mut s = state_at(0.0, 2000.0);
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        let mut env = env_at(400.0, 2000.0);
        env.player_y = 100.0; // above 800 — enraged
        // 10 hover ticks reach the picker (num2 = 10).
        s.ai2 = 9.0;
        s.ai3 = 0.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.damage, 200);
        assert_eq!(s.defense, 100);
        assert_eq!(s.ai0, 1.0); // the charge launched on tick 10
        // The charge speed is num6 = 16 + 6 = 22.
        let speed = (s.vx * s.vx + s.vy * s.vy).sqrt();
        assert!((speed - 22.0).abs() < 1e-3, "speed {speed}");
    }

    #[test]
    fn the_phase_one_cycle_runs_five_charges_then_bubbles_then_sharknado() {
        // `:49101-49132` + `:49187-49195`: ai[3] steps 0,2,4,6,8 →
        // charge; 10 → bubbles (ai[3] = 1); 11 → sharknado (ai[3] = 0).
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        let env = env_at(400.0, 0.0);
        // Drive the picker directly at each counter value. num2:
        // ai[3] < 10 → flag5 → 30 (`:48813-48816`); ai[3] >= 10 →
        // the base 60 (`:48798`).
        for (counter, next) in [
            (0.0f32, 1.0f32),
            (2.0, 1.0),
            (4.0, 1.0),
            (6.0, 1.0),
            (8.0, 1.0),
            (10.0, 2.0),
            (11.0, 3.0),
        ] {
            s.ai0 = 0.0;
            s.ai2 = if counter < 10.0 { 29.0 } else { 59.0 };
            s.ai3 = counter;
            fishron_ai_step(&mut s, &env);
            assert_eq!(s.ai0, next, "counter {counter}");
        }
        // The bubble pick resets ai[3] to 1, the sharknado pick to 0.
        s.ai0 = 0.0;
        s.ai2 = 59.0;
        s.ai3 = 10.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai3, 1.0);
        s.ai0 = 0.0;
        s.ai2 = 59.0;
        s.ai3 = 11.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai3, 0.0);
    }

    #[test]
    fn the_half_hp_band_forces_the_transform() {
        // `:48778` + `:49129-49132`: life <= 50 % → num27 = 4 on any
        // cycle boundary, whatever the counter says.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        s.ai2 = 29.0;
        s.ai3 = 0.0;
        s.life = 30000; // exactly half
        let env = env_at(400.0, 0.0);
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 4.0);
        // The transform runs 180 ticks then phase 2 (`:49300-49308`).
        s.ai2 = 179.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 5.0);
        assert_eq!(s.ai3, 0.0);
    }

    #[test]
    fn the_charge_launch_snaps_the_velocity_and_the_rotation() {
        // `:49135-49150`: velocity = normalize(player − center) × 16,
        // rotation = atan2(vy, vx) (+π when the sprite faced right).
        let mut s = state_at(0.0, 0.0); // centre (75, 50)
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        s.ai2 = 29.0;
        s.ai3 = 0.0;
        let env = env_at(475.0, 50.0); // dead right
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 1.0);
        assert!((s.vx - 16.0).abs() < 1e-4, "vx {}", s.vx);
        assert!(s.vy.abs() < 1e-4, "vy {}", s.vy);
        // The charge holds num5 = 30 ticks then ai[3] += 2 (`:49188`).
        s.ai2 = 29.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai3, 2.0);
    }

    #[test]
    fn the_bubble_volley_spawns_every_four_ticks() {
        // `:49240-49248`: ai[2] % 4 == 0 → one 371 at the aim muzzle,
        // y + 45; 80 ticks total (`:49260`).
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 2.0;
        s.ai2 = 0.0;
        let env = env_at(475.0, 50.0);
        let out = fishron_ai_step(&mut s, &env);
        let bubbles: Vec<_> = out
            .requests
            .iter()
            .filter(|r| matches!(r, FishronRequest::Bubble { .. }))
            .collect();
        assert_eq!(bubbles.len(), 1);
        if let FishronRequest::Bubble { x, y } = bubbles[0] {
            // muzzle = normalize × (150+20)/2 + centre = (75+85, 50).
            assert_eq!(*x, 160);
            assert_eq!(*y, 50 + 45);
        }
        // Tick 1 draws nothing.
        let out = fishron_ai_step(&mut s, &env);
        assert!(!out
            .requests
            .iter()
            .any(|r| matches!(r, FishronRequest::Bubble { .. })));
    }

    #[test]
    fn the_sharknado_bolt_fires_two_bolts_at_tick_60() {
        // `:49276-49281`: at ai[2] == 60, two 385s at ±direction·2 / 8.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 3.0;
        s.ai2 = 60.0;
        s.direction = 1;
        let env = env_at(475.0, 50.0);
        let out = fishron_ai_step(&mut s, &env);
        let bolts: Vec<_> = out
            .requests
            .iter()
            .filter_map(|r| match r {
                FishronRequest::Sharknado { vx, vy, .. } => Some((*vx, *vy)),
                _ => None,
            })
            .collect();
        assert_eq!(bolts, vec![(2.0, 8.0), (-2.0, 8.0)]);
        // 90 ticks then back to the hover (`:49283`).
        s.ai2 = 89.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 0.0);
    }

    #[test]
    fn the_enrage_swaps_the_bubble_pick_for_the_sharknado() {
        // `:49125-49128`: flag6 && num27 == 2 → 3, and the enraged
        // sharknado entry starts at tick 50 (`:49160-49163`).
        let mut s = state_at(0.0, 2000.0);
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        s.ai2 = 9.0; // num2 = 10 enraged
        s.ai3 = 10.0;
        let mut env = env_at(400.0, 2000.0);
        env.player_y = 100.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 3.0);
        assert_eq!(s.ai2, 50.0); // num11 − 40
    }

    #[test]
    fn the_phase_two_cycle_runs_three_charges_then_cthulunado_then_sharknado() {
        // `:49366-49393`: ai[3] 0..5 → charge (6), 6 → spin (7, reset
        // 1), 7 → aimed sharknado (8, reset 0).
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        let env = env_at(400.0, 0.0);
        for (counter, next) in [
            (0.0f32, 6.0f32),
            (2.0, 6.0),
            (4.0, 6.0),
            (6.0, 7.0),
            (7.0, 8.0),
        ] {
            s.ai0 = 5.0;
            s.ai2 = 59.0; // num2 = 60 (flag5 false at ai3 >= 6 → base 60; ai3 < 6 → 20)
            if counter < 6.0 {
                s.ai2 = 19.0; // flag5 && flag3 → num2 = 20 classic
            }
            s.ai3 = counter;
            fishron_ai_step(&mut s, &env);
            assert_eq!(s.ai0, next, "counter {counter}");
        }
        // The Cthulunado launch snaps 20 (`:49413`).
        s.ai0 = 5.0;
        s.ai2 = 59.0;
        s.ai3 = 6.0;
        let env2 = env_at(475.0, 50.0);
        fishron_ai_step(&mut s, &env2);
        assert_eq!(s.ai0, 7.0);
        assert!((s.vx - 20.0).abs() < 1e-3, "vx {}", s.vx);
    }

    #[test]
    fn the_expert_fifteen_percent_band_forces_the_vanish() {
        // `:48779` + `:49386-49389`: expert && life <= 15 % → 9.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 5.0;
        s.ai2 = 39.0; // expert flag3&flag5 → num2 = 40 (`:48811`)
        s.ai3 = 0.0;
        s.life = 9000; // 15 % of 60000
        let mut env = env_at(400.0, 0.0);
        env.expert = true;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 9.0);
        // Classic at the same hp does NOT vanish (num2 = 20 classic).
        env.expert = false;
        s.ai0 = 5.0;
        s.ai2 = 19.0;
        s.ai3 = 0.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 6.0);
        // The vanish runs 180 ticks then phase 3 (`:49553-49559`).
        s.ai0 = 9.0;
        s.ai2 = 179.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 10.0);
        assert_eq!(s.ai3, 0.0);
    }

    #[test]
    fn the_cthulunado_spins_and_trails_bubbles() {
        // `:49471-49485`: a trail bubble every 4 ticks, the velocity
        // rotates by −num19·direction (num19 = 2π/60), 120 ticks.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 7.0;
        s.ai2 = 0.0;
        s.vx = 20.0;
        s.vy = 0.0;
        s.direction = 1;
        let env = env_at(475.0, 50.0);
        let out = fishron_ai_step(&mut s, &env);
        let trail: Vec<_> = out
            .requests
            .iter()
            .filter(|r| matches!(r, FishronRequest::TrailBubble { .. }))
            .collect();
        assert_eq!(trail.len(), 1);
        if let FishronRequest::TrailBubble { vx, vy, target, .. } = trail[0] {
            // normalize(20,0) rotated +π/2 → (0,1) × 6.
            assert!(vx.abs() < 1e-4);
            assert!((vy - 6.0).abs() < 1e-4);
            assert_eq!(*target, 0);
        }
        // The spin step: rotation −= 2π/60 (`:49485`).
        let step = PI * 2.0 / 60.0;
        assert!((s.rotation + step).abs() < 1e-5, "rot {}", s.rotation);
        // The velocity rotated by −step: (20cos, −20sin).
        assert!((s.vx - 20.0 * step.cos()).abs() < 1e-3);
        assert!((s.vy + 20.0 * step.sin()).abs() < 1e-3);
    }

    #[test]
    fn the_aimed_sharknado_carries_the_target_and_the_enrage() {
        // `:49503-49506`: one still 385 at the centre, ai[0] = 1,
        // ai[1] = target + 1, localAI[0] = flag6.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 8.0;
        s.ai2 = 60.0;
        let mut env = env_at(475.0, 50.0);
        env.target = 3;
        let out = fishron_ai_step(&mut s, &env);
        let bolts: Vec<_> = out
            .requests
            .iter()
            .filter_map(|r| match r {
                FishronRequest::Sharknado {
                    x,
                    y,
                    vx,
                    vy,
                    ai0,
                    ai1,
                    local_ai0,
                } => Some((*x, *y, *vx, *vy, *ai0, *ai1, *local_ai0)),
                _ => None,
            })
            .collect();
        assert_eq!(bolts, vec![(75.0, 50.0, 0.0, 0.0, 1.0, 4.0, 0.0)]);
        // Enraged: the third arg flips to 1.
        env.player_y = 100.0;
        s.ai2 = 60.0;
        let out = fishron_ai_step(&mut s, &env);
        assert!(out.requests.iter().any(|r| matches!(
            r,
            FishronRequest::Sharknado { local_ai0, .. } if *local_ai0 == 1.0
        )));
    }

    #[test]
    fn the_phase_three_cycle_picks_charge_and_teleport() {
        // `:49599-49615`: 0/2/3/5/6/7 → charge (11), 1/4/8 → teleport
        // (12); num2 = 30 in phase 3 (`:48801-48806`).
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        let env = env_at(400.0, 0.0);
        for (counter, next) in [
            (0.0f32, 11.0f32),
            (1.0, 12.0),
            (2.0, 11.0),
            (3.0, 11.0),
            (4.0, 12.0),
            (5.0, 11.0),
            (6.0, 11.0),
            (7.0, 11.0),
            (8.0, 12.0),
        ] {
            s.ai0 = 10.0;
            s.ai2 = 29.0;
            s.ai3 = counter;
            fishron_ai_step(&mut s, &env);
            assert_eq!(s.ai0, next, "counter {counter}");
        }
        // The phase-3 hover is unchaseable and fades (+25, `:49564-49572`).
        s.ai0 = 10.0;
        s.ai2 = 0.0;
        s.alpha = 0;
        s.chaseable = true;
        fishron_ai_step(&mut s, &env);
        assert!(!s.chaseable);
        assert_eq!(s.alpha, 25);
        // The phase-3 side offset is ±360 (`:49575`).
        assert_eq!(s.ai1.abs(), 360.0);
    }

    #[test]
    fn the_phase_three_charge_restores_chaseable_and_counts() {
        // `:49648-49675`: chaseable = true, alpha −25, num5 = 25 ticks
        // then ai[3]++ (no wrap).
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 11.0;
        s.ai2 = 24.0;
        s.ai3 = 8.0;
        s.alpha = 100;
        s.chaseable = false;
        let env = env_at(400.0, 0.0);
        fishron_ai_step(&mut s, &env);
        assert!(s.chaseable);
        assert_eq!(s.alpha, 75);
        assert_eq!(s.ai0, 10.0);
        assert_eq!(s.ai3, 9.0); // NOT wrapped here
    }

    #[test]
    fn the_teleport_recentres_over_the_player_and_wraps_at_nine() {
        // `:49691-49733`: at tick 15 the centre snaps to
        // player + (−ai[1], −200); at tick 30 ai[3]++ wraps 9 → 0.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 12.0;
        s.ai2 = 15.0; // the C# checks ai[2] == 15 BEFORE the increment
        s.ai1 = 300.0;
        let env = env_at(475.0, 50.0);
        fishron_ai_step(&mut s, &env);
        // centre = (475 − 300, 50 − 200) = (175, −150).
        assert_eq!(s.x, 175.0 - 75.0);
        assert_eq!(s.y, -150.0 - 50.0);
        assert!(out_dont_take_damage(&s, &env));
        // The wrap: run to tick 30 with ai[3] = 8.
        s.ai2 = 29.0;
        s.ai3 = 8.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 10.0);
        assert_eq!(s.ai3, 0.0);
    }

    fn out_dont_take_damage(s: &FishronState, env: &FishronEnv) -> bool {
        let mut c = *s;
        fishron_ai_step(&mut c, env).dont_take_damage
    }

    #[test]
    fn the_dead_player_arms_the_despawn_and_resets_the_phase() {
        // `:48854-48867`: vy −= 0.4, EncourageDespawn(10), ai[0] > 4 →
        // 5 else 0, ai[2] = 0.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 7.0;
        s.ai2 = 40.0;
        let mut env = env_at(400.0, 0.0);
        env.player_dead = true;
        let out = fishron_ai_step(&mut s, &env);
        assert!(out
            .requests
            .contains(&FishronRequest::EncourageDespawn(10)));
        assert_eq!(s.ai0, 5.0);
        assert_eq!(s.ai2, 0.0);
        assert!((s.vy + 0.4).abs() < 1e-5);
        // Phase 1 resets to 0 instead.
        s.ai0 = 2.0;
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.ai0, 0.0);
    }

    #[test]
    fn the_simple_fly_movement_double_adds_on_the_sign_reversal() {
        // `NPC.cs:77397-77431`: crossing zero adds the accel twice —
        // but only when the FIRST add leaves the velocity still past
        // zero (−0.6 + 0.45 = −0.15 < 0 → the second add fires).
        let (mut vx, mut vy) = (-0.6f32, 0.6f32);
        simple_fly_movement(&mut vx, &mut vy, 5.0, -5.0, 0.45);
        assert!((vx - 0.3).abs() < 1e-5, "vx {vx}"); // −0.6 + 0.45 + 0.45
        assert!((vy + 0.3).abs() < 1e-5, "vy {vy}"); // 0.6 − 0.45 − 0.45
        // −0.2 + 0.45 = +0.25 crosses zero → a SINGLE add.
        let (mut vx3, mut vy3) = (-0.2f32, 0.2f32);
        simple_fly_movement(&mut vx3, &mut vy3, 5.0, -5.0, 0.45);
        assert!((vx3 - 0.25).abs() < 1e-5, "vx3 {vx3}");
        assert!((vy3 + 0.25).abs() < 1e-5, "vy3 {vy3}");
        // No reversal: a single add.
        let (mut vx2, mut vy2) = (1.0f32, -1.0f32);
        simple_fly_movement(&mut vx2, &mut vy2, 5.0, -5.0, 0.45);
        assert!((vx2 - 1.45).abs() < 1e-5);
        assert!((vy2 + 1.45).abs() < 1e-5);
    }

    #[test]
    fn the_rotation_tracking_snaps_inside_the_step_band() {
        // `:48938-48975`: the 0.04 step, the ±step snap band.
        let mut s = state_at(0.0, 0.0);
        s.local_ai0 = 1.0;
        s.ai0 = 0.0;
        s.rotation = 0.0;
        s.sprite_direction = -1;
        // Player dead right → aim 0 → rotation stays 0 (snapped).
        let env = env_at(475.0, 50.0);
        fishron_ai_step(&mut s, &env);
        assert_eq!(s.rotation, 0.0);
        // Player straight up → aim −π/2 → +2π = 3π/2; one 0.04 step
        // toward it takes the SHORT way (down through 0 → wrap).
        let env_up = env_at(75.0, -400.0);
        s.rotation = 0.0;
        fishron_ai_step(&mut s, &env_up);
        // rotation < aim and (aim − rotation) > π → rotation −= 0.04,
        // then the < 0 wrap adds 2π (`:48964-48967`).
        assert!(
            (s.rotation - (PI * 2.0 - 0.04)).abs() < 1e-4,
            "rot {}",
            s.rotation
        );
    }

    #[test]
    fn the_seam_gate_is_kind_370_only() {
        assert!(fishron_seam_applies(370));
        assert!(!fishron_seam_applies(371)); // the bubble
        assert!(!fishron_seam_applies(113)); // the WoF
        assert!(!fishron_seam_applies(4)); // the EoC
    }

    #[test]
    fn the_machine_draws_no_rng() {
        // By construction: AI_069's only Main.rand draws are the dust
        // loops (`:49180`, `:49448`, `:49660`), which the port skips;
        // the pure core takes no RNG parameter at all. A full phase-1
        // charge cycle therefore runs deterministically with zero
        // stream consumption — this test pins the API shape (no rng
        // argument) and the repeatability.
        let mut a = state_at(0.0, 0.0);
        let mut b = state_at(0.0, 0.0);
        let env = env_at(400.0, 0.0);
        for _ in 0..600 {
            fishron_ai_step(&mut a, &env);
            fishron_ai_step(&mut b, &env);
        }
        assert_eq!(a.ai0, b.ai0);
        assert_eq!(a.ai3, b.ai3);
        assert_eq!(a.vx, b.vx);
        assert_eq!(a.vy, b.vy);
        assert_eq!(a.rotation, b.rotation);
    }
}
