//! W4-d25 (aiStyle-26) — the unicorn/galloper family, `NPC.AI`'s
//! `else if (this.aiStyle == 26) { this.AI_026_Unicorns(); }` block
//! (`NPC.cs:25631-25634` → `AI_026_Unicorns` `:62366-62924`), ported
//! 1:1 for the THREE unported members (code-only mode: no
//! compiler/tests were run on this branch; the witness tests below
//! are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 26` rows, whole-file grep)
//!
//! SEVEN vanilla members:
//! * Unicorn 86 (`NPCID.cs` `Unicorn = 86`, `SetDefaults`
//!   `NPC.cs:9926-9938`: 46×42, dmg 65, def 30, 400 HP, kb 0.3,
//!   NPCHit12/NPCDeath18, value 1000) — NO type branch inside the
//!   machine: it rides the GENERIC core (the 6-cap/0.07-accel walk,
//!   no turnaround damping, no hop-over);
//! * Wolf 155 (`Wolf = 155`, `NPC.cs:10961-10973`: 46×30, dmg 65,
//!   def 30, 300 HP, kb 0.3, NPCHit1/NPCDeath1, value 1000) — the
//!   155/329 hop-over (`:62558-62564`, INTEGER half-widths) + the
//!   155-only 0.95 turnaround damping (`:62621-62631`);
//! * Tumbleweed 546 (`Tumbleweed = 546`, `NPC.cs:15972-15984`: 30×30,
//!   dmg 30, def 6, 50 HP, kb 0.8, NPCHit11/NPCDeath15, value 100) —
//!   the num2=4 stuck bound + the same-type separation
//!   (`:62378-62408`), the FLOAT-centre hop + the (3, 11) chirp
//!   (`:62565-62569`), the ZoneDesert despawn gate (`:62570-62580`),
//!   the wind-driven walk (`:62703-62716`), the spriteDirection
//!   inversion in the jump probe (`:62865-62868`) and the rotation
//!   tail (`:62919-62923`);
//! * HeadlessHorseman 315 (`:12978-12990`) + Hellhound 329
//!   (`:13175-13187`) — ALREADY PORTED, faithful:
//!   `npc_ai_unicorn.rs::unicorn_step` (S1-F-13b2 slice A + S1-D-4
//!   W-10). VERIFY-ONLY: unchanged by this census;
//! * StardustSpiderSmall 410 (`:15074-15086`) + NebulaBeast 423
//!   (`:15117-15129`) — LANDED as documented stand-ins
//!   (`npc.rs::update_stardust_spider_small` /
//!   `update_nebula_beast`, M4-EVENT-9; their deviation notes ride
//!   those fns). Census-noted, NOT re-ported here.
//! The port has no `NpcKind::Unicorn`/`Wolf`/`Tumbleweed` yet, so this
//! module lands UNWIRED (the `npc_ai_style17.rs` pre-wiring
//! precedent): a pure fn keyed on the vanilla type. The wiring step
//! adds the kinds + the axis routing.
//!
//! ## The machine (the shared core, verbatim)
//!
//! * the stuck arm (`:62373-62377`): grounded + walking against the
//!   facing arms flag2 and `ai[3]++`.
//! * the 546 separation (`:62378-62408`): num2 = 4; the grounded
//!   latch BEFORE the sibling loop; each active same-type sibling
//!   inside the Manhattan `< width` window nudges ∓0.05 per axis;
//!   the latch re-zeroes vy after.
//! * the stuck counter (`:62441-62461`): frozen-x (the post-AI
//!   `oldPosition` — the port's `unicorn_stuck_x` end-of-tick latch,
//!   the `npc_ai_unicorn.rs` precedent) OR the window OR flag2 →
//!   `ai[3]++` + flag3; else decay; the `num * num2` wrap (300, or
//!   120 for 546); justHit resets; netUpdate at exactly 30.
//! * the target vector (`:62462-62469`): the player's FLOAT-half
//!   centre-X, the player's TOP y; inside 200 px a non-stuck NPC
//!   clears the counter.
//! * the hop-over (`:62558-62569`): 155 — grounded, <100 px, |vx|>3,
//!   rolling toward the player, INTEGER half-width centres → vy −4;
//!   546 — the same bar the distance gate, FLOAT centres, plus the
//!   (3, 11) chirp.
//! * the 546 desert gate (`:62570-62580`): outside the desert,
//!   re-target, and if still outside: EncourageDespawn(10),
//!   `ai[3] = 30`, the direction restored.
//! * the chase/wander split (`:62581-62616`): inside the window
//!   `TargetClosest()` (the 329/315 pumpkin-moon gate is not ours);
//!   past it the standstill 2-tick facing flip, `directionY = -1`,
//!   the zero-direction pin.
//! * the walk core (`:62617-62740`): the grounded/wet/with-facing
//!   gate; 155's 0.95 damping; 546's 0.92 sign damping + the wind
//!   cap `4 + Lerp(0.6, 1, |windSpeedTarget|) * sign(windSpeedTarget)
//!   * direction * 3` (0 outside the sandstorm) + 0.05 accel; 86 the
//!   generic 6/0.07; then the shared cap/approach block.
//! * the inline step-up (`:62741-62805`) + the jump probe
//!   (`:62806-62903`): verbatim (the `npc_ai_unicorn.rs`
//!   transcription); 546 inverts num23; flag7 (410/423) is FALSE for
//!   all three members — the below-1 conjunct folds true.
//! * the 546 rotation tail (`:62919-62923`): `rotation += vx * 0.05`,
//!   `spriteDirection = -direction`.
//!
//! Seam: writes velocity, position.Y/gfxOffY/stepSpeed (the step-up),
//! `ai[0]`/`ai[3]`, direction/spriteDirection/directionY, rotation,
//! netUpdate, despawn_timer (+ the target fields through
//! `target_closest`). Does NOT integrate the walk: the wiring caller
//! owns the gravity + tile-collision tail (the `npc_ai_unicorn.rs`
//! convention). Sounds leave as `(type, style, x, y)` requests (the
//! 546 chirp); the sibling scan rides [`Ai26Ctx::siblings`] (the
//! pooled-family precedent — `Npc::update` cannot see its siblings).
//!
//! ## RNG by construction
//!
//! The 86/155/546 paths draw NO `Main.rand` — the machine's only
//! draws are the 315 lob/dust and the 423 charge/dust arms (the other
//! modules'). The fn takes no RNG parameter;
//! `the_style26_arm_draws_no_rng` pins a `UnifiedRandom` state across
//! every member's tick (written, not run).
#![allow(dead_code)] // UNWIRED (W4-d25): the three kinds + the axis routing are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata as td;
use tr_core::{Tile, World};

/// One member's `SetDefaults` row (the census above).
pub struct GallopDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    /// `(SoundType, Style)` — the HitSound.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — the DeathSound.
    pub death_sound: (i32, i32),
    pub value: f32,
}

/// `NPC.cs:9926-9938` — `else if (this.type == 86)` (Unicorn).
pub const UNICORN: GallopDefaults = GallopDefaults {
    ty: 86,
    width: 46,
    height: 42,
    ai_style: 26,
    damage: 65,
    defense: 30,
    life_max: 400,
    knock_back_resist: 0.3,
    hit_sound: (3, 12),
    death_sound: (4, 18),
    value: 1000.0,
};

/// `NPC.cs:10961-10973` — `else if (this.type == 155)` (Wolf).
pub const WOLF: GallopDefaults = GallopDefaults {
    ty: 155,
    width: 46,
    height: 30,
    ai_style: 26,
    damage: 65,
    defense: 30,
    life_max: 300,
    knock_back_resist: 0.3,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    value: 1000.0,
};

/// `NPC.cs:15972-15984` — `else if (this.type == 546)` (Tumbleweed).
pub const TUMBLEWEED: GallopDefaults = GallopDefaults {
    ty: 546,
    width: 30,
    height: 30,
    ai_style: 26,
    damage: 30,
    defense: 6,
    life_max: 50,
    knock_back_resist: 0.8,
    hit_sound: (3, 11),
    death_sound: (4, 15),
    value: 100.0,
};

/// The per-tick context (the `npc_ai_style22.rs::Ai22Ctx` shape).
pub struct Ai26Ctx<'a> {
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
    pub ticker: u64,
    /// `Main.windSpeedTarget` (`Main.cs`) — the 546 walk's wind cap.
    pub wind_speed_target: f32,
    /// `Main.player[target].ZoneDesert` — the 546 despawn gate.
    pub zone_desert: bool,
    /// `Main.player[target].ZoneSandstorm` — the 546 wind gate.
    pub zone_sandstorm: bool,
    /// The OTHER active same-type NPCs' positions (`:62382-62384`'s
    /// `Main.npc` scan; only the 546 separation reads it). The sim's
    /// pre-pass collects them in slot order (the style75 pooled
    /// precedent).
    pub siblings: &'a [(f32, f32)],
}

/// The side effects beyond the NPC state.
#[derive(Clone, Debug, Default)]
pub struct Ai26Effects {
    /// `(SoundType, Style, x, y)` — the 546 hop chirp
    /// `SoundEngine.PlaySound(3, base.Center, 11)` (`:62568`).
    pub sounds: Vec<(i32, i32, f32, f32)>,
}

/// The target slot's candidate — the `npc_ai_style17.rs::target_of`
/// fallback (an unset/stale slot reads slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Tile.nactive()` (`Tile.cs:240-248`) — the `npc_ai_unicorn.rs`
/// transcription (active and NOT actuated).
fn nactive(t: &Tile) -> bool {
    t.active() && t.flags & tr_core::tile::FLAG_INACTIVE == 0
}

/// `Main.tileSolid[t.type]` — the generated table.
fn solid_kind(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind)
}

/// `Main.tileSolidTop[t.type]`.
fn solid_top(kind: u16) -> bool {
    td::TILE_SOLID_TOP.has(kind)
}

/// `Tile.topSlope()` (`Tile.cs:290-298`): slope 1 or 2.
fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

/// A missing tile reads as the C#'s `new Tile()` (`:62756-62775`).
fn tile_or_blank(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

/// `Math.Sign(float)` — −1 / 0 / +1.
fn sign_f(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `AI_026_Unicorns` (`NPC.cs:62366-62924`) for Unicorn 86 / Wolf 155
/// / Tumbleweed 546, one fixed step. RNG-free on these paths; the
/// caller owns the walk integration tail (the module writes the
/// step-up's position.Y lift only, verbatim).
pub fn ai_style26(ty: i32, npc: &mut Npc, world: &World, ctx: &Ai26Ctx) -> Ai26Effects {
    debug_assert!(
        ty == 86 || ty == 155 || ty == 546,
        "315/329 ride npc_ai_unicorn; 410/423 their landed stand-ins"
    );
    let mut fx = Ai26Effects::default();
    let (w, h) = npc.size();
    // `:62368-62372` — num = 30, num2 = 10 (546 → 4, `:62380`);
    // `flag` is only set by the unported-in-this-module 423 charge.
    let num = 30.0f32;
    let mut num2 = 10.0f32;
    let flag = false;
    let mut flag2 = false;
    let mut flag3 = false;
    // `:62373-62377` — the stuck arm.
    if npc.vy == 0.0 && ((npc.vx > 0.0 && npc.facing < 0) || (npc.vx < 0.0 && npc.facing > 0)) {
        flag2 = true;
        npc.ai[3] += 1.0;
    }
    // `:62378-62408` — the 546 separation.
    if ty == 546 {
        num2 = 4.0;
        let flag4 = npc.vy == 0.0;
        for &(sx, sy) in ctx.siblings {
            if (npc.x - sx).abs() + (npc.y - sy).abs() < w {
                if npc.x < sx {
                    npc.vx -= 0.05;
                } else {
                    npc.vx += 0.05;
                }
                if npc.y < sy {
                    npc.vy -= 0.05;
                } else {
                    npc.vy += 0.05;
                }
            }
        }
        if flag4 {
            npc.vy = 0.0;
        }
    }
    // `:62409-62440` — the 315 block is the other module's.
    // `:62441-62449` — the stuck counter (the C#'s bitwise `|` kept).
    if (npc.x == npc.unicorn_stuck_x || npc.ai[3] >= num) | flag2 {
        npc.ai[3] += 1.0;
        flag3 = true;
    } else if npc.ai[3] > 0.0 {
        npc.ai[3] -= 1.0;
    }
    // `:62450-62461` — the wrap, the justHit reset, the 30 netUpdate.
    if npc.ai[3] > num * num2 {
        npc.ai[3] = 0.0;
    }
    if npc.just_hit {
        npc.ai[3] = 0.0;
    }
    if npc.ai[3] == num {
        npc.net_update = true;
    }
    // `:62462-62469` — the target vector (the player's FLOAT-half
    // centre-X, the TOP y), the <200 clear.
    let p = target_of(npc, ctx.players);
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    let num6 = p.x + p.width as f32 * 0.5 - cx;
    let num7 = p.y - cy;
    let num8 = (num6 * num6 + num7 * num7).sqrt();
    if num8 < 200.0 && !flag3 {
        npc.ai[3] = 0.0;
    }
    // `:62470-62557` — the 410/423 arms are the landed stand-ins'.
    // `:62558-62564` — the 155 hop-over (INTEGER half-widths).
    if ty == 155
        && npc.vy == 0.0
        && num8 < 100.0
        && npc.vx.abs() > 3.0
    {
        let ncx = npc.x + (w as i32 / 2) as f32;
        let pcx_int = p.x + (p.width / 2) as f32;
        if (ncx < pcx_int && npc.vx > 0.0) || (ncx > pcx_int && npc.vx < 0.0) {
            npc.vy -= 4.0;
        }
    } else if ty == 546
        && npc.vy == 0.0
        && npc.vx.abs() > 3.0
    {
        // `:62565-62569` — the 546 hop: FLOAT centres, NO distance
        // gate, plus the chirp.
        let pcx = p.x + p.width as f32 * 0.5;
        if (cx < pcx && npc.vx > 0.0) || (cx > pcx && npc.vx < 0.0) {
            npc.vy -= 4.0;
            fx.sounds.push((3, 11, cx, cy));
        }
    }
    // `:62570-62580` — the 546 desert gate.
    if ty == 546 && !ctx.zone_desert {
        let num9 = npc.facing;
        target_closest(npc, world, ctx.players, true, 26, ctx.ticker);
        if !ctx.zone_desert {
            npc.encourage_despawn(10);
            npc.ai[3] = num;
            npc.facing = num9;
        }
    }
    // `:62581-62616` — the chase/wander split (the 329/315
    // pumpkin-moon gate is not these types').
    if npc.ai[3] < num {
        target_closest(npc, world, ctx.players, true, 26, ctx.ticker);
    } else {
        // `:62594-62615` — the wander tail.
        if npc.vx == 0.0 {
            if npc.vy == 0.0 {
                npc.ai[0] += 1.0;
                if npc.ai[0] >= 2.0 {
                    npc.facing *= -1;
                    npc.sprite_dir = npc.facing;
                    npc.ai[0] = 0.0;
                }
            }
        } else {
            npc.ai[0] = 0.0;
        }
        npc.direction_y = -1;
        if npc.facing == 0 {
            npc.facing = 1;
        }
    }
    // `:62617-62740` — the walk core.
    let mut num10 = 6.0f32;
    let mut num11 = 0.07f32;
    if !flag
        && (npc.vy == 0.0
            || npc.wet
            || (npc.vx <= 0.0 && npc.facing < 0)
            || (npc.vx >= 0.0 && npc.facing > 0))
    {
        if ty == 155 {
            // `:62621-62631` — the 155-only 0.95 turnaround damping.
            if npc.vx > 0.0 && npc.facing < 0 {
                npc.vx *= 0.95;
            }
            if npc.vx < 0.0 && npc.facing > 0 {
                npc.vx *= 0.95;
            }
        } else if ty == 546 {
            // `:62703-62716` — the wind-driven walk.
            if sign_f(npc.vx) != npc.facing {
                npc.vx *= 0.92;
            }
            // `MathHelper.Lerp(0.6f, 1f, |windSpeedTarget|)` —
            // UNCLAMPED (`MathHelper.cs`: value1 + (value2-value1)*amount).
            let mut num12 =
                (0.6 + (1.0 - 0.6) * ctx.wind_speed_target.abs()) * sign_f(ctx.wind_speed_target) as f32;
            if !ctx.zone_sandstorm {
                num12 = 0.0;
            }
            num10 = 4.0 + num12 * npc.facing as f32 * 3.0;
            num11 = 0.05;
        }
        // 86: no per-type arm — the generic 6/0.07.
        // `:62717-62739` — the shared cap/approach block.
        if npc.vx < 0.0 - num10 || npc.vx > num10 {
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                npc.vy *= 0.8;
            }
        } else if npc.vx < num10 && npc.facing == 1 {
            npc.vx += num11;
            if npc.vx > num10 {
                npc.vx = num10;
            }
        } else if npc.vx > 0.0 - num10 && npc.facing == -1 {
            npc.vx -= num11;
            if npc.vx < 0.0 - num10 {
                npc.vx = 0.0 - num10;
            }
        }
    }
    // `:62741-62805` — the inline step-up (the npc_ai_unicorn.rs
    // transcription; only while falling or level).
    if npc.vy >= 0.0 {
        let num13 = sign_f(npc.vx);
        let vx_pos = npc.x + npc.vx;
        let num14 = ((vx_pos + (w as i32 / 2) as f32 + ((w as i32 / 2 + 1) * num13) as f32)
            / 16.0) as i32;
        let num15 = ((npc.y + h - 1.0) / 16.0) as i32;
        let t0 = tile_or_blank(world, num14, num15);
        let t1 = tile_or_blank(world, num14, num15 - 1);
        let t2 = tile_or_blank(world, num14, num15 - 2);
        let t3 = tile_or_blank(world, num14, num15 - 3);
        let t4 = tile_or_blank(world, num14 - num13, num15 - 3);
        let tm4 = tile_or_blank(world, num14, num15 - 4);
        // `:62776` — the ledge gate, verbatim conjunct order.
        let base_ok = (nactive(&t0)
            && !top_slope(&t0)
            && !top_slope(&t1)
            && solid_kind(t0.kind)
            && !solid_top(t0.kind))
            || (t1.half_brick() && nactive(&t1));
        let head_clear = !nactive(&t1)
            || !solid_kind(t1.kind)
            || solid_top(t1.kind)
            || (t1.half_brick() && (!nactive(&tm4) || !solid_kind(tm4.kind) || solid_top(tm4.kind)));
        let body_clear = (!nactive(&t2) || !solid_kind(t2.kind) || solid_top(t2.kind))
            && (!nactive(&t3) || !solid_kind(t3.kind) || solid_top(t3.kind))
            && (!nactive(&t4) || !solid_kind(t4.kind));
        if (num14 as f32 * 16.0) < vx_pos + w
            && (num14 as f32 * 16.0 + 16.0) > vx_pos
            && base_ok
            && head_clear
            && body_clear
        {
            // `:62778-62803` — the lift.
            let mut num16 = num15 as f32 * 16.0;
            if t0.half_brick() {
                num16 += 8.0;
            }
            if t1.half_brick() {
                num16 -= 8.0;
            }
            if num16 < npc.y + h {
                let num17 = npc.y + h - num16;
                if (num17 as f64) <= 16.1 {
                    npc.gfx_off_y += npc.y + h - num16;
                    npc.y = num16 - h;
                    npc.step_speed = if num17 < 9.0 { 1.0 } else { 2.0 };
                }
            }
        }
    }
    // `:62806-62903` — the jump probe.
    if npc.vy == 0.0 {
        let mut flag6 = true;
        let num18 = (npc.y - 7.0) as i32 / 16;
        let num19 = (npc.x - 7.0) as i32 / 16;
        let num20 = (npc.x + w + 7.0) as i32 / 16;
        for m in num19..=num20 {
            let t = tile_or_blank(world, m, num18);
            if nactive(&t) && solid_kind(t.kind) {
                flag6 = false;
                break;
            }
        }
        if flag6 {
            let num21 = ((npc.x + (w as i32 / 2) as f32
                + ((w as i32 / 2 + 2) * npc.facing) as f32
                + npc.vx * 5.0)
                / 16.0) as i32;
            let num22 = ((npc.y + h - 15.0) / 16.0) as i32;
            let j0 = tile_or_blank(world, num21, num22);
            let j1 = tile_or_blank(world, num21, num22 - 1);
            let j2 = tile_or_blank(world, num21, num22 - 2);
            let j3 = tile_or_blank(world, num21, num22 - 3);
            // The `num22 + 1` probe (`:62840-62843`) is only read
            // behind flag7 (410/423) — FALSE for all three members.
            let below2 = tile_or_blank(world, num21, num22 + 2);
            let ahead3 = tile_or_blank(world, num21 + npc.facing, num22 + 3);
            // `:62864-62868` — num23 = spriteDirection; 546 inverts.
            let mut num23 = npc.sprite_dir;
            if ty == 546 {
                num23 *= -1;
            }
            if (npc.vx < 0.0 && num23 == -1) || (npc.vx > 0.0 && num23 == 1) {
                let num24 = 3.0f32;
                if nactive(&j2) && solid_kind(j2.kind) {
                    if nactive(&j3) && solid_kind(j3.kind) {
                        npc.vy = -8.5;
                        npc.net_update = true;
                    } else {
                        npc.vy = -7.5;
                        npc.net_update = true;
                    }
                } else if nactive(&j1) && !top_slope(&j1) && solid_kind(j1.kind) {
                    npc.vy = -7.0;
                    npc.net_update = true;
                } else if npc.y + h - (num22 as f32 * 16.0) > 20.0
                    && nactive(&j0)
                    && !top_slope(&j0)
                    && solid_kind(j0.kind)
                {
                    npc.vy = -6.0;
                    npc.net_update = true;
                } else if (npc.direction_y < 0 || npc.vx.abs() > num24)
                    && (!nactive(&below2) || !solid_kind(below2.kind))
                    && (!nactive(&ahead3) || !solid_kind(ahead3.kind))
                {
                    npc.vy = -8.0;
                    npc.net_update = true;
                }
            }
        }
    }
    // `:62904-62918` — the 423 dust tail is the stand-in's.
    // `:62919-62923` — the 546 rotation tail.
    if ty == 546 {
        npc.rotation += npc.vx * 0.05;
        npc.sprite_dir = -npc.facing;
    }
    fx
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins
    //! one C# branch against its citation (`NPC.cs:62366-62924`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::{UnifiedRandom, TILE_PX};

    /// A flat stone floor at row 100 (the npc_ai_unicorn tests' shape).
    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 120);
        for x in 0..200 {
            for y in 100..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A stand-in body with the member's box (the kinds are unported).
    fn beast(row: &GallopDefaults, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((row.width as f32, row.height as f32));
        n.hp = row.life_max;
        n.max_hp = row.life_max;
        n.target = 0;
        n.sprite_dir = 1;
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], siblings: &'a [(f32, f32)]) -> Ai26Ctx<'a> {
        Ai26Ctx {
            players,
            ticker: 0,
            wind_speed_target: 0.0,
            zone_desert: true,
            zone_sandstorm: false,
            siblings,
        }
    }

    /// The `SetDefaults` rows (`NPC.cs:9926-9938` / `:10961-10973` /
    /// `:15972-15984`).
    #[test]
    fn the_defaults_rows_match_the_census() {
        assert_eq!((UNICORN.width, UNICORN.height), (46, 42));
        assert_eq!((UNICORN.damage, UNICORN.defense, UNICORN.life_max), (65, 30, 400));
        assert_eq!(UNICORN.knock_back_resist, 0.3);
        assert_eq!(UNICORN.hit_sound, (3, 12));
        assert_eq!(UNICORN.death_sound, (4, 18));
        assert_eq!(UNICORN.value, 1000.0);
        assert_eq!((WOLF.width, WOLF.height), (46, 30));
        assert_eq!((WOLF.damage, WOLF.defense, WOLF.life_max), (65, 30, 300));
        assert_eq!(WOLF.hit_sound, (3, 1));
        assert_eq!(WOLF.death_sound, (4, 1));
        assert_eq!((TUMBLEWEED.width, TUMBLEWEED.height), (30, 30));
        assert_eq!((TUMBLEWEED.damage, TUMBLEWEED.defense, TUMBLEWEED.life_max), (30, 6, 50));
        assert_eq!(TUMBLEWEED.knock_back_resist, 0.8);
        assert_eq!(TUMBLEWEED.hit_sound, (3, 11));
        assert_eq!(TUMBLEWEED.death_sound, (4, 15));
        assert_eq!(TUMBLEWEED.value, 100.0);
        for row in [UNICORN, WOLF, TUMBLEWEED] {
            assert_eq!(row.ai_style, 26);
        }
    }

    /// `:62617-62739` — the Unicorn rides the GENERIC walk: no
    /// turnaround damping, the 6 cap / 0.07 accel.
    #[test]
    fn the_unicorn_walks_the_generic_six_cap() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&UNICORN, 60.0 * TILE_PX, 100.0 * TILE_PX - 42.0);
        n.unicorn_stuck_x = n.x - 1.0; // moved last tick: no stuck arm
        let fx = ai_style26(86, &mut n, &w, &ctx(&players, &[]));
        assert_eq!(n.vx, 0.07, "the generic accel, no damping");
        assert!(fx.sounds.is_empty());
        // The cap: from 5.98 the accel clamps at exactly 6.
        n.vx = 5.98;
        ai_style26(86, &mut n, &w, &ctx(&players, &[]));
        assert_eq!(n.vx, 6.0);
    }

    /// `:62621-62631` — the Wolf's 0.95 turnaround damping runs BEFORE
    /// the shared accel.
    #[test]
    fn the_wolf_damps_the_turnaround() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&WOLF, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        n.vx = -2.0; // rolling LEFT while facing RIGHT
        ai_style26(155, &mut n, &w, &ctx(&players, &[]));
        assert_eq!(n.vx, -2.0f32 * 0.95 + 0.07);
    }

    /// `:62558-62564` — the Wolf hop-over: grounded, <100 px, |vx|>3,
    /// rolling toward the player's INTEGER-half centre → vy −4.
    #[test]
    fn the_wolf_hops_over_the_near_player() {
        let w = ground();
        // The player 60 px right of the wolf's centre.
        let players = [cand(60.0 * TILE_PX + 60.0, 100.0 * TILE_PX - 42.0)];
        let mut n = beast(&WOLF, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        n.vx = 4.0;
        ai_style26(155, &mut n, &w, &ctx(&players, &[]));
        // The hop fired (-4); the walk gate still runs (the
        // with-facing conjunct) but only touches vx — vy keeps the
        // hop's -4 (the jump probe is grounded-only).
        assert_eq!(n.vy, -4.0, "the hop armed");
    }

    /// `:62378-62408` — the Tumbleweed separation: the Manhattan
    /// window, the ∓0.05 nudges, the grounded vy re-zero.
    #[test]
    fn the_tumbleweed_separates_from_siblings() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        // A sibling 10 px right + 5 px down: Manhattan 15 < width 30.
        let siblings = [(n.x + 10.0, n.y + 5.0)];
        ai_style26(546, &mut n, &w, &ctx(&players, &siblings));
        // vx: -0.05 (sibling right), then the walk's 0.92 sign damp
        // (sign(-0.05) != facing) and the +0.05 accel.
        assert_eq!(n.vx, -0.05f32 * 0.92 + 0.05);
        assert_eq!(n.vy, 0.0, "the grounded latch re-zeroed the nudge");
        // A far sibling (Manhattan >= 30) nudges nothing.
        let mut n2 = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n2.unicorn_stuck_x = n2.x - 1.0;
        let far = [(n2.x + 40.0, n2.y)];
        ai_style26(546, &mut n2, &w, &ctx(&players, &far));
        assert_eq!(n2.vx, 0.05, "only the walk accel");
    }

    /// `:62703-62716` — the Tumbleweed wind cap: Lerp(0.6,1,|wind|) *
    /// sign(wind) * direction * 3 + 4, zeroed outside the sandstorm;
    /// the 0.92 sign damping + the 0.05 accel.
    #[test]
    fn the_tumbleweed_walks_the_wind_cap() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        let mut c = ctx(&players, &[]);
        c.wind_speed_target = 0.5;
        c.zone_sandstorm = true;
        ai_style26(546, &mut n, &w, &c);
        // num12 = (0.6 + 0.4*0.5) * 1 = 0.8; cap = 4 + 0.8*1*3 = 6.4.
        assert_eq!(n.vx, 0.05);
        n.vx = 6.39;
        ai_style26(546, &mut n, &w, &c);
        assert_eq!(n.vx, 6.4, "the wind cap 4 + 0.8*3");
        // Outside the sandstorm the wind term zeroes: the cap is 4.
        let mut n2 = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n2.unicorn_stuck_x = n2.x - 1.0;
        n2.vx = 3.99;
        let mut c2 = ctx(&players, &[]);
        c2.wind_speed_target = 0.5;
        ai_style26(546, &mut n2, &w, &c2);
        assert_eq!(n2.vx, 4.0);
    }

    /// `:62570-62580` — outside the desert the Tumbleweed discourages
    /// despawn, pins ai[3] = 30 and restores the direction.
    #[test]
    fn the_tumbleweed_despawns_outside_the_desert() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        n.despawn_timer = 600;
        let mut c = ctx(&players, &[]);
        c.zone_desert = false;
        ai_style26(546, &mut n, &w, &c);
        assert_eq!(n.despawn_timer, 10);
        assert_eq!(n.ai[3], 30.0, "the wander window pinned");
        assert_eq!(n.facing, 1, "the pre-retarget direction restored");
    }

    /// `:62919-62923` — the rotation tail: rotation += vx*0.05 and
    /// spriteDirection = −direction, every 546 tick.
    #[test]
    fn the_tumbleweed_spins_with_its_velocity() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.unicorn_stuck_x = n.x - 1.0;
        ai_style26(546, &mut n, &w, &ctx(&players, &[]));
        assert_eq!(n.rotation, n.vx * 0.05);
        assert_eq!(n.sprite_dir, -n.facing);
    }

    /// `:62441-62461` — the stuck counter: the frozen-x climb, the
    /// netUpdate at exactly 30, the 546 wrap at 30*4 = 120.
    #[test]
    fn the_stuck_counter_wraps_at_num_times_num2() {
        let w = ground();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        let mut n = beast(&TUMBLEWEED, 60.0 * TILE_PX, 100.0 * TILE_PX - 30.0);
        n.ai[3] = 120.0;
        n.unicorn_stuck_x = n.x; // frozen
        n.vx = 0.0;
        let mut saw_net = false;
        n.ai[3] = 29.0;
        n.net_update = false;
        ai_style26(546, &mut n, &w, &ctx(&players, &[]));
        if n.net_update {
            saw_net = true;
        }
        assert!(saw_net, "netUpdate at exactly 30");
        n.ai[3] = 120.0;
        n.x = 60.0 * TILE_PX;
        n.unicorn_stuck_x = n.x;
        n.vx = 0.0;
        ai_style26(546, &mut n, &w, &ctx(&players, &[]));
        // 120 → 121 > 120 → wrap 0 (then the wander tail runs).
        assert_eq!(n.ai[3], 0.0);
    }

    /// The RNG census: the three members' paths take no stream and
    /// the C# draws only on the 315/423 arms — a probe is untouched.
    #[test]
    fn the_style26_arm_draws_no_rng() {
        let w = ground();
        let mut rng = UnifiedRandom::new(26);
        let probe = rng.clone();
        let players = [cand(150.0 * TILE_PX, 98.0 * TILE_PX)];
        for (ty, row) in [(86, UNICORN), (155, WOLF), (546, TUMBLEWEED)] {
            let mut n = beast(&row, 60.0 * TILE_PX, 100.0 * TILE_PX - row.height as f32);
            for _ in 0..60 {
                ai_style26(ty, &mut n, &w, &ctx(&players, &[]));
                n.unicorn_stuck_x = n.x;
            }
        }
        assert_eq!(rng, probe, "the arm never touches the stream");
    }
}
