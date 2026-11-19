//! S1-D-3 (speed table) — `AI_003_Fighters`' per-type velocity ladder
//! (`NPC.cs:57403-58401`), ported arm-for-arm in the C#'s if-else order.
//!
//! The C# runs this ladder inside `AI_003_Fighters` AFTER the shared
//! target/stuck core (`NPC.cs:57120-57402`) and BEFORE the lighting block
//! (`:58402+`) and the position integration — the port wires it at the same
//! position: `npc_ai.rs::walker_step_ladder` sets the facing/sprite (the
//! port's target-core stand-in), runs this ladder in place of the direct
//! velocity write, then integrates; `npc.rs::update_zombie` calls the same
//! entry (`run_ai3_ladder`) before its integration.
//!
//! **RNG: none.** The whole ladder (`:57403-58401`) draws no `Main.rand`
//! value — every arm is deterministic arithmetic on velocity, direction,
//! scale, life and the target's centre. (The shooter block `:59069+` draws
//! `Next(-40, 41)` per axis, but that block is OUT of this module.)
//!
//! **Skipped kinds (sibling-owned, do NOT duplicate):**
//! - The Frost Moon walkers 338/339/340/342 (`num83` rows `:57637-57648`,
//!   the scale row `:57693`), 343/348/351 (cap-2 group `:57554`) and 349
//!   (the spin arm `:57403-57440`) are ported by
//!   `npc_ai_frost_mob.rs::fighter_walk_velocity` (commit `5ef9b24b`).
//!   This module CITES the same C# lines but returns `None` for those ids —
//!   the sibling owns their walk arms (338-352 as a block).
//! - The Frost Legion trio 143/144/145 never reaches this ladder: they are
//!   aiStyle 38 (`NPC.cs:10809/:10823/:10837`), ported by
//!   `npc.rs::update_frost_legion` (`NPC.cs:28995-29145`, R-25-1). No
//!   AI_003 arm names them.
//! - The aiStyle-7 caster/town family is a different style entirely.
//!
//! **Deferred terms (documented, ANALYSIS "The aiStyle-3 velocity ladder"):**
//! - The shooter exclusion list of the default arm (`:58322`): 110/111/
//!   206/214/215/216/290-293/350/379-382/449-452/468/481/411/409/498-506/
//!   424/426/520 — their walk rides the separate shooter block
//!   (`:59069-59800`, the `ai3_walk_speed` rows, S1-D-3c) until that block
//!   ports; this ladder returns `None` for them.
//! - `Main.remixWorld` (the 181 row `:57609-57612`) and `Main.expertMode`
//!   (the 489 row `:58367`) have no port world flags (single-difficulty
//!   normal, `world.rs`'s S1 note) — the normal-world terms ship.
//! - The 430-436/494/495/591 arm's `CanHit` ray (`:58288`) rides the port's
//!   `projectile::can_hit` stand-in at the call site's granularity.

/// The ladder's per-tick inputs — the C# fields the arms read. `vx`/`vy`
/// are `base.velocity`, `direction` is `base.direction`, `scale` is
/// `NPC.scale`, `life`/`life_max` the NPC's, `ai2`/`ai3` the `ai[2]`/`ai[3]`
/// slots, and the `target_*` fields the `Main.player[this.target]` reads.
pub struct LadderCtx {
    pub npc_type: u16,
    pub vx: f32,
    pub vy: f32,
    pub direction: i32,
    pub sprite_direction: i32,
    pub scale: f32,
    pub life: i32,
    pub life_max: i32,
    pub ai2: f32,
    pub ai3: f32,
    /// `base.Center.X`/`.Y` — the NPC's own centre (the critter hop, the
    /// lerp machine's 20-px gate, the charge family's distance).
    pub center_x: f32,
    pub center_y: f32,
    /// `NPC.gravity` — the static the gravity pass sets this tick
    /// (`NPC.cs:90881` defaults 0.3f); the lerp machine's weight gates.
    pub gravity: f32,
    pub target_center_x: f32,
    pub target_center_y: f32,
    pub target_dead: bool,
    /// `Main.IsItDay()` / `position.Y > Main.worldSurface * 16.0` — the
    /// 430-family charge gate.
    pub day_time: bool,
    pub below_surface: bool,
    /// The `CanHit` ray between the NPC and the target (430 family).
    pub can_hit_target: bool,
    /// `NPC.defDamage` — the 430 family's damage restore.
    pub def_damage: i32,
}

/// The ladder's side effects beyond velocity: the 305-309 hop flag (the
/// `flag` at `:57847`, consumed by the obstacle-jump scan at `:59831` —
/// the scan itself is the port's collision-jump stand-in, so the flag is
/// carried but not yet read: ANALYSIS deferral) and the 430 family's
/// `ai[2]`/`ai[3]`/`damage` writes (`:58262-58321`).
#[derive(Default)]
pub struct LadderFx {
    pub hop_flag: bool,
    pub ai2: Option<f32>,
    pub ai3: Option<f32>,
    pub damage: Option<i32>,
}

/// `NPCID.Sets.CritterThatCanTurnOnPlayers` (`NPCID.cs:10436`): the ten
/// critters the `:57441` arm names. Also read by the aiStyle-3 chase gate
/// (`npc_ai.rs::fighters_not_discouraged`, `NPC.cs:60309`).
pub(crate) const TURN_CRITTERS: [u16; 10] = [645, 639, 644, 642, 643, 641, 640, 299, 539, 538];

/// The shooter-block exclusion list of the default arm (`NPC.cs:58322`'s
/// `this.type != …` chain — the same set the `:59069` block claims).
fn shooter_excluded(t: u16) -> bool {
    matches!(t, 110 | 111 | 206 | 214 | 215 | 216 | 350 | 468 | 481 | 411 | 409 | 424 | 426 | 520)
        || (290..=293).contains(&t)
        || (379..=382).contains(&t)
        || (449..=452).contains(&t)
        || (498..=506).contains(&t)
}

/// The shared accel/brake shape every plain arm folds into: over-cap the
/// grounded NPC brakes BOTH velocity components (`base.velocity *=
/// over_factor`); under-cap it accelerates toward `direction` by `accel`,
/// clamped at `cap`, with an optional grounded counter-drift brake
/// (`counter_factor`, applied only when moving against the direction).
/// The C# repeats this block per arm (`:57409-57439` and siblings); the
/// fold keeps the constants per-arm and the arithmetic in one place.
fn accel_brake(
    c: &mut LadderCtx,
    cap: f32,
    accel: f32,
    over_factor: f32,
    counter_factor: Option<f32>,
) {
    if c.vx < -cap || c.vx > cap {
        if c.vy == 0.0 {
            // `base.velocity *= f` scales both components (`:57413`).
            c.vx *= over_factor;
            c.vy *= over_factor;
        }
    } else if c.vx < cap && c.direction == 1 {
        if let Some(f) = counter_factor {
            if c.vy == 0.0 && c.vx < 0.0 {
                c.vx *= f;
            }
        }
        c.vx += accel;
        if c.vx > cap {
            c.vx = cap;
        }
    } else if c.vx > -cap && c.direction == -1 {
        if let Some(f) = counter_factor {
            if c.vy == 0.0 && c.vx > 0.0 {
                c.vx *= f;
            }
        }
        c.vx -= accel;
        if c.vx < -cap {
            c.vx = -cap;
        }
    }
}

/// The ladder entry — returns the new `(vx, vy)` and the side effects, or
/// `None` when the type is sibling-owned / shooter-deferred (the caller
/// keeps its current walk). Arm order is the C#'s if-else chain verbatim.
pub fn apply_ladder(c: &mut LadderCtx) -> Option<((f32, f32), LadderFx)> {
    let t = c.npc_type;
    // The frost sibling owns the whole 338-352 block (`npc_ai_frost_mob.rs`,
    // commit `5ef9b24b`) — including the ids the C# folds into the num83 /
    // cap-2 / default arms (341/344-347/352 ride num107 there).
    if (338..=352).contains(&t) {
        return None;
    }
    let mut fx = LadderFx::default();
    // `NPC.cs:57403-57440` — the 159/349 spin arm. 349 is sibling-owned
    // (`npc_ai_frost_mob.rs`); 159 keeps the arm with its own turn brake.
    if t == 159 || t == 349 {
        if t == 349 {
            return None; // the frost sibling owns 338-352
        }
        // `:57405-57408` — 159-only: moving against the direction brakes.
        if (c.vx > 0.0 && c.direction < 0) || (c.vx < 0.0 && c.direction > 0) {
            c.vx *= 0.95;
        }
        accel_brake(c, 6.0, 0.07, 0.8, Some(0.99));
    }
    // `NPC.cs:57441-57485` — the turn-on-player critters.
    else if TURN_CRITTERS.contains(&t) {
        // `:57443-57452` — the hop: alive target, grounded, |dx| in
        // (0, 100), moving toward the player. (`num81`'s Length() write at
        // `:57445` is dead — `:57446` overwrites it with |dx|.)
        if !c.target_dead && c.vy == 0.0 {
            let dx_abs = (c.center_x - c.target_center_x).abs();
            if dx_abs > 0.0
                && dx_abs < 100.0
                && ((c.vx < 0.0 && c.target_center_x < c.center_x)
                    || (c.vx > 0.0 && c.target_center_x > c.center_x))
            {
                c.vy = -4.0;
                c.vx += c.direction as f32 * 3.0;
            }
        }
        accel_brake(c, 3.5, 0.1, 0.8, Some(0.8));
    }
    // `NPC.cs:57486-57519` — the 199 arm.
    else if t == 199 {
        accel_brake(c, 4.0, 0.1, 0.8, Some(0.8));
    }
    // `NPC.cs:57520-57553` — the cap-3 group.
    else if matches!(t, 120 | 166 | 213 | 258 | 528 | 529) {
        accel_brake(c, 3.0, 0.07, 0.8, Some(0.99));
    }
    // `NPC.cs:57554-57579` — the cap-2 group (no counter brake).
    // 343/348/351 are sibling-owned (`npc_ai_frost_mob.rs`).
    else if matches!(
        t,
        461 | 27 | 77 | 104 | 163 | 162 | 196 | 197 | 212 | 257 | 326 | 343 | 348 | 351 | 530 | 236
    ) || (524..=527).contains(&t)
    {
        if matches!(t, 343 | 348 | 351) {
            return None; // the frost sibling owns 338-352
        }
        accel_brake(c, 2.0, 0.07, 0.8, None);
    }
    // `NPC.cs:57580-57605` — the 109 arm (the slow 0.04 accel).
    else if t == 109 {
        accel_brake(c, 2.0, 0.04, 0.8, None);
    }
    // `NPC.cs:57606-57740` — the num83 group.
    else if matches!(
        t,
        21 | 26 | 31 | 294 | 295 | 296 | 47 | 73 | 140 | 164 | 239 | 167 | 168 | 185 | 198
            | 201 | 202 | 203 | 217 | 218 | 219 | 226 | 181 | 254 | 338 | 339 | 340 | 342
            | 385 | 389 | 462 | 463 | 466 | 464 | 469 | 470 | 480 | 482 | 425 | 429 | 586
            | 631 | 635
    ) {
        if matches!(t, 338 | 339 | 340 | 342) {
            return None; // the frost sibling owns 338-352
        }
        // `:57608-57692` — the per-type cap rows. The 181 remix term
        // (`:57609`) needs `Main.remixWorld`, unported — the base 1.5 ships.
        let life_frac = 1.0 - c.life as f32 / c.life_max as f32;
        let mut cap = match t {
            294 => 2.0,
            295 => 1.75,
            296 => 1.25,
            201 => 1.1,
            202 => 0.9,
            203 => 1.2,
            385 => 1.8,
            389 => 2.25,
            462 => 4.0,
            463 => 0.75,
            466 => 3.75,
            469 => 3.25,
            480 => 1.5 + life_frac * 2.0, // `:57675`
            425 => 6.0,
            429 => 4.0,
            631 => 0.9,
            586 => 1.5 + life_frac * 3.5, // `:57691`
            _ => 1.5,
        };
        // `:57693-57696` — the scale row (21/201/202/203/342/635; 342 is
        // sibling-owned and never reaches here).
        if matches!(t, 21 | 201 | 202 | 203 | 635) {
            cap *= 1.0 + (1.0 - c.scale);
        }
        if c.vx < -cap || c.vx > cap {
            if c.vy == 0.0 {
                c.vx *= 0.8;
                c.vy *= 0.8;
            }
        } else if c.vx < cap && c.direction == 1 {
            // `:57706-57713` — the 466/586 opposing brakes.
            if t == 466 && c.vx < -2.0 {
                c.vx *= 0.9;
            }
            if t == 586 && c.vy == 0.0 && c.vx < -1.0 {
                c.vx *= 0.9;
            }
            c.vx += 0.07;
            if c.vx > cap {
                c.vx = cap;
            }
        } else if c.vx > -cap && c.direction == -1 {
            if t == 466 && c.vx > 2.0 {
                c.vx *= 0.9;
            }
            if t == 586 && c.vy == 0.0 && c.vx > 1.0 {
                c.vx *= 0.9;
            }
            c.vx -= 0.07;
            if c.vx < -cap {
                c.vx = -cap;
            }
        }
        // `:57736-57739` — the 462 post-arm grounded counter-drift.
        if c.vy == 0.0
            && t == 462
            && ((c.direction > 0 && c.vx < 0.0) || (c.direction < 0 && c.vx > 0.0))
        {
            c.vx *= 0.9;
        }
    }
    // `NPC.cs:57741-57816` — the num84 group (269-280, all scale-multiplied).
    else if (269..=280).contains(&t) {
        let mut cap = match t {
            269 => 2.0,
            270 => 1.0,
            271 => 1.5,
            272 => 3.0,
            273 => 1.25,
            274 => 3.0,
            275 => 3.25,
            276 => 2.0,
            277 => 2.75,
            278 => 1.8,
            279 => 1.3,
            280 => 2.5,
            _ => 1.5,
        };
        cap *= 1.0 + (1.0 - c.scale); // `:57792`
        accel_brake(c, cap, 0.07, 0.8, None);
    }
    // `NPC.cs:57817-57880` — the num85 group (305-314, the scarecrows).
    else if (305..=314).contains(&t) {
        let cap = match t {
            305 | 310 => 2.0,
            306 | 311 => 1.25,
            307 | 312 => 2.25,
            308 | 313 => 1.5,
            309 | 314 => 1.0,
            _ => 1.5,
        };
        if t < 310 {
            // `:57840-57856` — the hop machine (305-309): grounded decay,
            // the ±0.3 (double) snap into a -7 hop at the row's speed, the
            // airborne 10:1 lerp while the sprite faces the direction.
            if c.vy == 0.0 {
                c.vx *= 0.85;
                if (c.vx as f64) > -0.3 && (c.vx as f64) < 0.3 {
                    fx.hop_flag = true; // `flag = true` (`:57847`)
                    c.vy = -7.0;
                    c.vx = cap * c.direction as f32;
                }
            } else if c.sprite_direction == c.direction {
                c.vx = (c.vx * 10.0 + cap * c.direction as f32) / 11.0;
            }
        } else {
            // `:57857-57879` — 310-314: the plain accel/brake, no counter.
            accel_brake(c, cap, 0.07, 0.8, None);
        }
    }
    // `NPC.cs:57881-57906` — the 67/220/428 slow arm.
    else if matches!(t, 67 | 220 | 428) {
        accel_brake(c, 0.5, 0.03, 0.7, None);
    }
    // `NPC.cs:57907-57943` — the 78/79/80/630 arm (the half-life rage).
    else if matches!(t, 78 | 79 | 80 | 630) {
        let mut cap = 1.0;
        let mut accel = 0.05;
        if c.life < c.life_max / 2 {
            // `:57911-57915` — integer division on lifeMax, as the C#.
            cap = 2.0;
            accel = 0.1;
        }
        if matches!(t, 79 | 630) {
            cap *= 1.5; // `:57916-57919`
        }
        accel_brake(c, cap, accel, 0.7, None);
    }
    // `NPC.cs:57944-57971` — the 287 arm.
    else if t == 287 {
        accel_brake(c, 5.0, 0.2, 0.7, None);
    }
    // `NPC.cs:57972-58001` — the 243 arm (life-scaled cap + accel).
    else if t == 243 {
        let life_frac = 1.0 - c.life as f32 / c.life_max as f32;
        let cap = 1.0 + life_frac * 1.5;
        let accel = 0.07 + life_frac * 0.15;
        accel_brake(c, cap, accel, 0.7, None);
    }
    // `NPC.cs:58002-58031` — the 251 arm (life-scaled cap + accel).
    else if t == 251 {
        let life_frac = 1.0 - c.life as f32 / c.life_max as f32;
        let cap = 1.0 + life_frac * 2.0;
        let accel = 0.08 + life_frac * 0.2;
        accel_brake(c, cap, accel, 0.7, None);
    }
    // `NPC.cs:58032-58069` — the 386 arm (the ai[2] work state brakes).
    else if t == 386 {
        if c.ai2 > 0.0 {
            if c.vy == 0.0 {
                c.vx *= 0.8; // `:58034-58040`
            }
        } else {
            accel_brake(c, 1.5, 0.15, 0.7, None);
        }
    }
    // `NPC.cs:58070-58138` — the 460 arm (the speed-staged accel).
    else if t == 460 {
        let ax = c.vx.abs();
        let mut accel = 0.1;
        // `:58074-58105` — eight cumulative ×0.8 stages; the 2.5/3.5/4.5/5.5
        // thresholds compare in double, as the C#.
        if ax > 2.0 {
            accel *= 0.8;
        }
        if (ax as f64) > 2.5 {
            accel *= 0.8;
        }
        if ax > 3.0 {
            accel *= 0.8;
        }
        if (ax as f64) > 3.5 {
            accel *= 0.8;
        }
        if ax > 4.0 {
            accel *= 0.8;
        }
        if (ax as f64) > 4.5 {
            accel *= 0.8;
        }
        if ax > 5.0 {
            accel *= 0.8;
        }
        if (ax as f64) > 5.5 {
            accel *= 0.8;
        }
        let cap = 3.0 + (1.0 - c.life as f32 / c.life_max as f32) * 3.0; // `:58106`
        if c.vx < -cap || c.vx > cap {
            if c.vy == 0.0 {
                c.vx *= 0.7;
                c.vy *= 0.7;
            }
        } else if c.vx < cap && c.direction == 1 {
            if c.vx < 0.0 {
                c.vx *= 0.93; // `:58116-58119` — no ground check
            }
            c.vx += accel;
            if c.vx > cap {
                c.vx = cap;
            }
        } else if c.vx > -cap && c.direction == -1 {
            if c.vx > 0.0 {
                c.vx *= 0.93;
            }
            c.vx -= accel;
            if c.vx < -cap {
                c.vx = -cap;
            }
        }
    }
    // `NPC.cs:58139-58202` — the 508/580/582 lerp machine.
    else if matches!(t, 508 | 580 | 582) {
        let mut num98 = 2.5;
        let mut num99 = 10.0;
        let ax = c.vx.abs();
        if t == 582 {
            // `:58144-58158`
            num98 = 2.25;
            num99 = 7.0;
            if ax > 2.5 {
                num98 = 3.0;
                num99 += 75.0;
            } else if ax > 2.0 {
                num98 = 2.75;
                num99 += 55.0;
            }
        } else if ax > 2.75 {
            // `:58159-58168`
            num98 = 3.5;
            num99 += 80.0;
        } else if (ax as f64) > 2.25 {
            num98 = 3.0;
            num99 += 60.0;
        }
        // `:58169-58179` — the near-ground counter brake (both components).
        if (c.vy.abs() as f64) < 0.5 {
            if c.vx > 0.0 && c.direction < 0 {
                c.vx *= 0.95;
                c.vy *= 0.95;
            }
            if c.vx < 0.0 && c.direction > 0 {
                c.vx *= 0.95;
                c.vy *= 0.95;
            }
        }
        // `:58180-58188` — the airborne weight bump. `NPC.gravity` is the
        // per-NPC static the gravity pass sets (`NPC.cs:90881` defaults
        // 0.3f); the port passes the tick's effective gravity.
        if c.vy.abs() > c.gravity {
            let mult = if t == 582 { 2.0 } else { 3.0 };
            num99 *= mult;
        }
        // `:58189-58201` — the lerp / the far-target nudge.
        if c.vx <= 0.0 && c.direction < 0 {
            c.vx = (c.vx * num99 - num98) / (num99 + 1.0);
        } else if c.vx >= 0.0 && c.direction > 0 {
            c.vx = (c.vx * num99 + num98) / (num99 + 1.0);
        } else if (c.center_x - c.target_center_x).abs() > 20.0 && c.vy.abs() <= c.gravity {
            c.vx *= 0.99;
            c.vx += c.direction as f32 * 0.025;
        }
    }
    // `NPC.cs:58203-58261` — the carrier group (391/427/415/419/518/532).
    else if matches!(t, 391 | 427 | 415 | 419 | 518 | 532) {
        let (cap, accel, over) = match t {
            427 => (6.0, 0.2, 0.8),   // `:58208-58213`
            415 => (4.0, 0.1, 0.95),  // `:58214-58219`
            419 => (6.0, 0.15, 0.85), // `:58220-58225`
            518 => (5.0, 0.1, 0.95),  // `:58226-58231`
            532 => (5.0, 0.15, 0.98), // `:58232-58237`
            _ => (5.0, 0.25, 0.7),    // 391, `:58205-58207`
        };
        accel_brake(c, cap, accel, over, None);
    }
    // `NPC.cs:58262-58321` — the charge family (430-436/494/495/591).
    else if (430..=436).contains(&t) || matches!(t, 494 | 495 | 591) {
        if c.ai2 == 0.0 {
            fx.damage = Some(c.def_damage); // `:58266`
            let cap = 1.0 * (1.0 + (1.0 - c.scale)); // `:58267`
            accel_brake(c, cap, 0.07, 0.8, None);
            // `:58291-58304` — the close-range stop into the charge state.
            if c.vy == 0.0 && (!c.day_time || c.below_surface) && !c.target_dead {
                let dx = c.center_x - c.target_center_x;
                let dy = c.center_y - c.target_center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                let range = if matches!(t, 494 | 495) { 42.0 } else { 50.0 };
                if dist < range && c.can_hit_target {
                    c.vx *= 0.7;
                    fx.ai2 = Some(1.0);
                }
            }
        } else {
            // `:58306-58320` — the charge state: ×1.5 damage (the double
            // cast then truncation), the 0.9 decay with the ±0.1 snap, the
            // ai[2] counter and its three reset gates.
            fx.damage = Some((c.def_damage as f64 * 1.5) as i32);
            fx.ai3 = Some(1.0);
            c.vx *= 0.9;
            if (c.vx.abs() as f64) < 0.1 {
                c.vx = 0.0;
            }
            let mut ai2 = c.ai2 + 1.0;
            if ai2 >= 20.0 || c.vy != 0.0 || (c.day_time && !c.below_surface) {
                ai2 = 0.0;
            }
            fx.ai2 = Some(ai2);
        }
    }
    // `NPC.cs:58322-58401` — the num107 default arm (every other fighter).
    else if !shooter_excluded(t) {
        let mut cap = match t {
            624 => 2.5,  // `:58325-58328`
            186 => 1.1,  // `:58329-58332`
            187 => 0.9,  // `:58333-58336`
            188 => 1.2,  // `:58337-58340`
            189 => 0.8,  // `:58341-58344`
            132 => 0.95, // `:58345-58348`
            200 => 0.87, // `:58349-58352`
            223 => 1.05, // `:58353-58356`
            632 => 0.8,  // `:58357-58360`
            691 => 0.85, // `:58361-58364`
            489 => {
                // `:58365-58373` — the distance-scaled cap. The expert
                // term (`Main.expertMode`) is unported — normal ships.
                let dx = c.target_center_x - c.center_x;
                let dy = c.target_center_y - c.center_y;
                let mut d = (dx * dx + dy * dy).sqrt() * 0.0025;
                if (d as f64) > 1.5 {
                    d = 1.5;
                }
                (2.5 - d) * 0.8
            }
            _ => 1.0,
        };
        // `:58374-58377` — the scale row.
        if matches!(t, 489 | 3 | 132 | 186 | 187 | 188 | 189 | 200 | 223 | 331 | 332) {
            cap *= 1.0 + (1.0 - c.scale);
        }
        accel_brake(c, cap, 0.07, 0.8, None);
    }
    else {
        return None;
    }
    Some(((c.vx, c.vy), fx))
}

#[cfg(test)]
mod tests {
    //! S1-D-3 (speed table) — tests-as-code, NOT run in this task's
    //! code-only mode. Every assertion pins a constant to its C# literal
    //! (`NPC.cs:57403-58401`); the expected values are hand-derived from
    //! the same branch order the module ports.

    use super::*;

    /// A grounded, full-life, scale-1 fighter facing right with the
    /// target 200 px to the right — the neutral fixture every arm test
    /// starts from (only the fields the arm reads are overridden).
    fn ctx(t: u16) -> LadderCtx {
        LadderCtx {
            npc_type: t,
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            sprite_direction: 1,
            scale: 1.0,
            life: 100,
            life_max: 100,
            ai2: 0.0,
            ai3: 0.0,
            center_x: 0.0,
            center_y: 0.0,
            gravity: 0.3, // `NPC.cs:90881`'s default
            target_center_x: 200.0,
            target_center_y: 0.0,
            target_dead: false,
            day_time: false,
            below_surface: true,
            can_hit_target: true,
            def_damage: 20,
        }
    }

    fn vx(t: u16, vx0: f32) -> f32 {
        let mut c = ctx(t);
        c.vx = vx0;
        apply_ladder(&mut c).expect("the arm claims the type").0 .0
    }

    // ---- `NPC.cs:57403-57440` — the 159 spin arm ----

    #[test]
    fn the_159_arm_pins_6_cap_007_accel_and_the_turn_brakes() {
        // `:57422` accel 0.07, `:57425` cap 6.
        assert_eq!(vx(159, 0.0), 0.07);
        assert_eq!(vx(159, 5.99), 6.0);
        // `:57405-57408` — the 159-only turn brake ×0.95, then the
        // dir -1 arm's grounded counter ×0.99 (`:57430-57433`):
        // 2 × 0.95 = 1.9, × 0.99 = 1.881, - 0.07 = 1.811.
        let mut c = ctx(159);
        c.vx = 2.0;
        c.direction = -1;
        let out = apply_ladder(&mut c).unwrap().0 .0;
        assert!((out - 1.811).abs() < 1e-5, "{out}");
        // `:57418-57421` — the grounded counter-drift ×0.99. vx -1 with
        // dir 1 ALSO trips the 159 turn brake (`:57405`): -1 × 0.95 ×
        // 0.99 + 0.07 = -0.8705.
        let mut c = ctx(159);
        c.vx = -1.0;
        let out = apply_ladder(&mut c).unwrap().0 .0;
        assert!((out - (-1.0 * 0.95 * 0.99 + 0.07)).abs() < 1e-5, "{out}");
        // `:57409-57414` — over-cap grounded brakes BOTH components ×0.8.
        let mut c = ctx(159);
        c.vx = 7.0;
        c.vy = 0.0;
        let (vx, vy) = apply_ladder(&mut c).unwrap().0;
        assert_eq!((vx, vy), (5.6, 0.0));
        // 349 is sibling-owned — the arm declines it.
        let mut c = ctx(349);
        assert!(apply_ladder(&mut c).is_none());
    }

    // ---- `NPC.cs:57441-57485` — the turn-on-player critters ----

    #[test]
    fn the_critter_arm_pins_35_cap_01_accel_and_the_hop() {
        for t in TURN_CRITTERS {
            assert_eq!(vx(t, 0.0), 0.1, "type {t}: `:57467` accel 0.1");
            assert_eq!(vx(t, 3.49), 3.5, "type {t}: `:57453` cap 3.5");
        }
        // `:57447-57451` — the hop: grounded, |dx| in (0,100), moving
        // toward the living player: vy = -4, vx += direction * 3.
        let mut c = ctx(299);
        c.center_x = 0.0;
        c.target_center_x = 50.0;
        c.vx = 0.5;
        let (vx, vy) = apply_ladder(&mut c).unwrap().0;
        assert_eq!(vy, -4.0);
        // 0.5 + 3 = 3.5, then the accel arm clamps at 3.5.
        assert_eq!(vx, 3.5);
        // The dead target never hops (`:57443`).
        let mut c = ctx(299);
        c.target_center_x = 50.0;
        c.target_dead = true;
        c.vx = 0.5;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .1, 0.0);
        // |dx| == 0 or >= 100 never hops (`:57447`).
        let mut c = ctx(299);
        c.target_center_x = 0.0;
        c.vx = 0.5;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .1, 0.0);
        let mut c = ctx(299);
        c.target_center_x = 100.0;
        c.vx = 0.5;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .1, 0.0);
    }

    // ---- the plain accel arms ----

    #[test]
    fn the_199_arm_pins_4_cap_01_accel() {
        assert_eq!(vx(199, 0.0), 0.1); // `:57501`
        assert_eq!(vx(199, 3.99), 4.0); // `:57488`/`:57504`
        // `:57497-57500` counter ×0.8.
        assert!((vx(199, -1.0) - (-0.8 + 0.1)).abs() < 1e-6);
    }

    #[test]
    fn the_cap3_group_pins_3_cap_007_accel_099_counter() {
        for t in [120u16, 166, 213, 258, 528, 529] {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:57535`");
            assert_eq!(vx(t, 2.99), 3.0, "type {t}: `:57522` cap 3");
            assert!(
                (vx(t, -1.0) - (-0.99 + 0.07)).abs() < 1e-6,
                "type {t}: `:57533` ×0.99"
            );
        }
    }

    #[test]
    fn the_cap2_group_pins_2_cap_007_accel_no_counter() {
        for t in [
            461u16, 27, 77, 104, 163, 162, 196, 197, 212, 257, 326, 524, 525, 526, 527, 530,
            236,
        ] {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:57565`");
            assert_eq!(vx(t, 1.99), 2.0, "type {t}: `:57556` cap 2");
            // NO counter brake (`:57563-57569`): -1 + 0.07, not -0.99+0.07.
            assert!((vx(t, -1.0) - (-1.0 + 0.07)).abs() < 1e-6, "type {t}");
        }
        // The frost siblings decline their cap-2 ids.
        for t in [343u16, 348, 351] {
            let mut c = ctx(t);
            assert!(apply_ladder(&mut c).is_none(), "type {t} is sibling-owned");
        }
    }

    #[test]
    fn the_109_arm_pins_2_cap_004_accel() {
        assert_eq!(vx(109, 0.0), 0.04); // `:57591`
        assert_eq!(vx(109, 1.99), 2.0); // `:57582` cap 2
    }

    // ---- `NPC.cs:57606-57740` — the num83 group ----

    #[test]
    fn the_num83_group_pins_the_base_and_every_row() {
        // The base 1.5 (`:57608`) for the un-overridden members.
        for t in [26u16, 31, 47, 73, 140, 164, 239, 167, 168, 185, 198, 217, 218, 219, 226,
            181, 254, 464, 470, 482, 635]
        {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:57714` accel 0.07");
            assert_eq!(vx(t, 1.49), 1.5, "type {t}: base 1.5");
        }
        // The per-type rows (`:57613-57692`).
        let rows: [(u16, f32); 14] = [
            (294, 2.0),
            (295, 1.75),
            (296, 1.25),
            (201, 1.1),
            (202, 0.9),
            (203, 1.2),
            (385, 1.8),
            (389, 2.25),
            (462, 4.0),
            (463, 0.75),
            (466, 3.75),
            (469, 3.25),
            (425, 6.0),
            (429, 4.0),
        ];
        for (t, cap) in rows {
            assert_eq!(vx(t, cap - 0.01), cap, "type {t} cap {cap}");
        }
        assert_eq!(vx(631, 0.89), 0.9); // `:57687`
        // The life-scaled rows (`:57675`, `:57691`): half life.
        let mut c = ctx(480);
        c.life = 50;
        c.vx = 2.49;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 2.5); // 1.5 + 0.5*2
        let mut c = ctx(586);
        c.life = 50;
        c.vx = 3.24;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 3.25); // 1.5 + 0.5*3.5
        // The scale row (`:57693-57696`): 21 at scale 0.5 → 1.5 * 1.5.
        let mut c = ctx(21);
        c.scale = 0.5;
        c.vx = 2.24;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.25).abs() < 1e-6);
        // 635 rides the scale row too; 26 does not.
        let mut c = ctx(635);
        c.scale = 0.5;
        c.vx = 2.24;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.25).abs() < 1e-6);
        assert_eq!(vx(26, 1.49), 1.5);
        // The 466 opposing brake (`:57706-57709`): vx -3, dir 1 →
        // -3 × 0.9 + 0.07 = -2.63.
        assert!((vx(466, -3.0) - (-2.63)).abs() < 1e-6);
        // The 586 grounded opposing brake (`:57710-57713`): vx -1.2
        // (inside the full-life cap 1.5) → -1.2 × 0.9 + 0.07 = -1.01.
        let mut c = ctx(586);
        c.vx = -1.2;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (-1.01)).abs() < 1e-5);
        // The 462 post-arm (`:57736-57739`): grounded, dir 1, vx -1 →
        // the accel arm adds 0.07 (still < 0), then ×0.9.
        assert!((vx(462, -1.0) - (-0.93 * 0.9)).abs() < 1e-6);
        // The frost siblings decline their num83 ids.
        for t in [338u16, 339, 340, 342] {
            let mut c = ctx(t);
            assert!(apply_ladder(&mut c).is_none(), "type {t} is sibling-owned");
        }
    }

    // ---- `NPC.cs:57741-57816` — the num84 group ----

    #[test]
    fn the_num84_group_pins_every_row_and_the_scale_factor() {
        let rows: [(u16, f32); 12] = [
            (269, 2.0),
            (270, 1.0),
            (271, 1.5),
            (272, 3.0),
            (273, 1.25),
            (274, 3.0),
            (275, 3.25),
            (276, 2.0),
            (277, 2.75),
            (278, 1.8),
            (279, 1.3),
            (280, 2.5),
        ];
        for (t, cap) in rows {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:57802` accel 0.07");
            assert_eq!(vx(t, cap - 0.01), cap, "type {t} cap {cap}");
            // ALL rows scale (`:57792`): scale 0.5 → factor 1+(1-0.5) =
            // 1.5, so the cap grows ×1.5.
            let mut c = ctx(t);
            c.scale = 0.5;
            c.vx = cap * 1.5 - 0.01;
            assert!(
                (apply_ladder(&mut c).unwrap().0 .0 - cap * 1.5).abs() < 1e-5,
                "type {t} scaled cap"
            );
        }
    }
    // ---- `NPC.cs:57817-57880` — the num85 scarecrow group ----

    #[test]
    fn the_num85_pairs_pin_the_five_caps() {
        for (t, cap) in [
            (305u16, 2.0f32),
            (306, 1.25),
            (307, 2.25),
            (308, 1.5),
            (309, 1.0),
            (310, 2.0),
            (311, 1.25),
            (312, 2.25),
            (313, 1.5),
            (314, 1.0),
        ] {
            if t >= 310 {
                // The plain accel half (`:57857-57879`).
                assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:57866`");
                assert_eq!(vx(t, cap - 0.01), cap, "type {t} cap {cap}");
            }
        }
    }

    #[test]
    fn the_scarecrow_hop_machine_pins_085_decay_03_snap_minus7_hop() {
        // `:57842-57850` — grounded: ×0.85, then the ±0.3 snap hops.
        let mut c = ctx(305);
        c.vx = 1.0;
        let (vx, vy) = apply_ladder(&mut c).unwrap().0;
        assert_eq!((vx, vy), (0.85, 0.0)); // 1 × 0.85, no hop yet
        let mut c = ctx(305);
        c.vx = 0.3; // 0.3 × 0.85 = 0.255 < 0.3 → hop
        let ((vx, vy), fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(vy, -7.0); // `:57848`
        assert_eq!(vx, 2.0); // num85 * direction (`:57849`)
        assert!(fx.hop_flag); // `:57847`
        // Left-facing hop: vx = -num85.
        let mut c = ctx(309);
        c.vx = -0.3;
        c.direction = -1;
        let ((vx, vy), _) = apply_ladder(&mut c).unwrap();
        assert_eq!((vx, vy), (-1.0, -7.0));
        // `:57852-57855` — airborne, sprite == direction: the 10:1 lerp.
        let mut c = ctx(305);
        c.vy = -3.0;
        c.vx = 1.0;
        let (vx, _) = apply_ladder(&mut c).unwrap().0;
        assert!((vx - (1.0 * 10.0 + 2.0) / 11.0).abs() < 1e-6);
        // Airborne with the sprite against the direction: no lerp.
        let mut c = ctx(305);
        c.vy = -3.0;
        c.vx = 1.0;
        c.sprite_direction = -1;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 1.0);
    }

    // ---- the slow / rage / life-scaled arms ----

    #[test]
    fn the_67_220_428_arm_pins_05_cap_003_accel_07_brake() {
        for t in [67u16, 220, 428] {
            assert_eq!(vx(t, 0.0), 0.03, "type {t}: `:57892`");
            assert_eq!(vx(t, 0.49), 0.5, "type {t}: `:57883` cap 0.5");
            let mut c = ctx(t);
            c.vx = 1.0;
            let (vx, vy) = apply_ladder(&mut c).unwrap().0;
            assert_eq!((vx, vy), (0.7, 0.0), "type {t}: `:57887` ×0.7");
        }
    }

    #[test]
    fn the_78_79_80_630_arm_pins_the_half_life_rage() {
        // Full life: 1 / 0.05 (`:57909-57910`); 79/630 ×1.5 (`:57918`).
        assert_eq!(vx(78, 0.0), 0.05);
        assert_eq!(vx(78, 0.99), 1.0);
        assert_eq!(vx(80, 0.99), 1.0);
        assert_eq!(vx(79, 1.49), 1.5);
        assert_eq!(vx(630, 1.49), 1.5);
        // Half life: 2 / 0.1 (`:57913-57914`), the ×1.5 still applies.
        for (t, cap) in [(78u16, 2.0f32), (80, 2.0), (79, 3.0), (630, 3.0)] {
            let mut c = ctx(t);
            c.life = 49; // lifeMax/2 = 50 (integer division, `:57911`)
            c.vx = cap - 0.01;
            assert!((apply_ladder(&mut c).unwrap().0 .0 - cap).abs() < 1e-5);
            let mut c = ctx(t);
            c.life = 49;
            assert!((apply_ladder(&mut c).unwrap().0 .0 - 0.1).abs() < 1e-6);
        }
    }

    #[test]
    fn the_287_arm_pins_5_cap_02_accel() {
        assert_eq!(vx(287, 0.0), 0.2); // `:57947`
        assert_eq!(vx(287, 4.99), 5.0); // `:57946`
    }

    #[test]
    fn the_243_and_251_arms_pin_the_life_scaled_pairs() {
        // 243: 1 + frac*1.5 cap, 0.07 + frac*0.15 accel (`:57974-57977`).
        assert_eq!(vx(243, 0.0), 0.07);
        let mut c = ctx(243);
        c.life = 50;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (0.07 + 0.5 * 0.15)).abs() < 1e-6);
        let mut c = ctx(243);
        c.life = 50;
        c.vx = 1.74;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 1.75).abs() < 1e-5);
        // 251: 1 + frac*2 cap, 0.08 + frac*0.2 accel (`:58004-58007`).
        assert_eq!(vx(251, 0.0), 0.08);
        let mut c = ctx(251);
        c.life = 50;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (0.08 + 0.5 * 0.2)).abs() < 1e-6);
        let mut c = ctx(251);
        c.life = 50;
        c.vx = 1.99;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn the_386_arm_pins_the_ai2_work_brake_and_the_15_015_walk() {
        // ai[2] > 0, grounded: ×0.8 (`:58034-58040`).
        let mut c = ctx(386);
        c.ai2 = 1.0;
        c.vx = 1.0;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 0.8);
        // ai[2] == 0: 1.5 cap / 0.15 accel (`:58043-58044`).
        assert_eq!(vx(386, 0.0), 0.15);
        assert_eq!(vx(386, 1.49), 1.5);
    }
    // ---- `NPC.cs:58070-58138` — the 460 staged-accel arm ----

    #[test]
    fn the_460_arm_pins_the_eight_08_stages_and_the_life_cap() {
        // Full life: cap 3 (`:58072` + `:58106`), accel 0.1 at rest.
        assert_eq!(vx(460, 0.0), 0.1);
        assert_eq!(vx(460, 2.99), 3.0);
        // Half life: cap 3 + 0.5*3 = 4.5 (`:58106`).
        let mut c = ctx(460);
        c.life = 50;
        c.vx = 4.49;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 4.5).abs() < 1e-5);
        // The accel stages (`:58074-58105`): |vx| 2.2 crosses ONE >2
        // stage → 0.1 × 0.8 = 0.08; 2.6 crosses >2 and >2.5 → 0.064.
        let mut c = ctx(460);
        c.life = 1; // cap ≈ 5.97 so the accel arm runs
        c.vx = 2.2;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (2.2 + 0.08)).abs() < 1e-5);
        let mut c = ctx(460);
        c.life = 1;
        c.vx = 2.6;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (2.6 + 0.064)).abs() < 1e-5);
        // The counter ×0.93 has NO ground check (`:58116-58119`).
        let mut c = ctx(460);
        c.life = 1;
        c.vx = -1.0;
        c.vy = -2.0; // airborne — the brake still applies
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (-0.93 + 0.1)).abs() < 1e-5);
    }

    // ---- `NPC.cs:58139-58202` — the lerp machine ----

    #[test]
    fn the_lerp_machine_pins_the_508_580_rows() {
        // At rest, dir 1: (0*10 + 2.5)/11 (`:58141-58142`, `:58195`).
        let mut c = ctx(508);
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.5 / 11.0).abs() < 1e-6);
        // |vx| > 2.75 → 3.5 / 90 (`:58159-58163`): (3*90 + 3.5)/91.
        let mut c = ctx(580);
        c.vx = 3.0;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (3.0 * 90.0 + 3.5) / 91.0).abs() < 1e-5);
        // |vx| > 2.25 (double) → 3 / 70 (`:58164-58168`).
        let mut c = ctx(508);
        c.vx = 2.5;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (2.5 * 70.0 + 3.0) / 71.0).abs() < 1e-5);
        // Airborne |vy| > gravity → num99 ×3 (`:58180-58188`): at rest
        // the weight is 30 → 2.5/31.
        let mut c = ctx(508);
        c.vy = 1.0;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.5 / 31.0).abs() < 1e-6);
        // The near-ground counter brake scales BOTH (`:58169-58179`):
        // vx 1, dir -1, vy 0.2 → vx 0.95, vy 0.19, then the far-target
        // nudge is skipped (dir -1 with vx > 0 hits neither lerp arm;
        // |dx| 200 > 20 but |vy| 0.19 <= 0.3 → ×0.99 - 0.025).
        let mut c = ctx(508);
        c.vx = 1.0;
        c.vy = 0.2;
        c.direction = -1;
        let (vx, vy) = apply_ladder(&mut c).unwrap().0;
        assert!((vy - 0.19).abs() < 1e-6);
        assert!((vx - (0.95 * 0.99 - 0.025)).abs() < 1e-6);
    }

    #[test]
    fn the_lerp_machine_pins_the_582_rows() {
        // At rest: 2.25 / 7 → 2.25/8 (`:58146-58147`).
        let mut c = ctx(582);
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.25 / 8.0).abs() < 1e-6);
        // |vx| > 2.5 → 3 / 82 (`:58148-58152`).
        let mut c = ctx(582);
        c.vx = 3.0;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (3.0 * 82.0 + 3.0) / 83.0).abs() < 1e-5);
        // |vx| > 2 → 2.75 / 62 (`:58153-58157`).
        let mut c = ctx(582);
        c.vx = 2.2;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - (2.2 * 62.0 + 2.75) / 63.0).abs() < 1e-5);
        // Airborne weight ×2, not ×3 (`:58183-58186`): 2.25/15.
        let mut c = ctx(582);
        c.vy = 1.0;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 2.25 / 15.0).abs() < 1e-6);
    }

    // ---- `NPC.cs:58203-58261` — the carrier group ----

    #[test]
    fn the_carrier_group_pins_the_six_param_triples() {
        let rows: [(u16, f32, f32, f32); 6] = [
            (391, 5.0, 0.25, 0.7),
            (427, 6.0, 0.2, 0.8),
            (415, 4.0, 0.1, 0.95),
            (419, 6.0, 0.15, 0.85),
            (518, 5.0, 0.1, 0.95),
            (532, 5.0, 0.15, 0.98),
        ];
        for (t, cap, accel, over) in rows {
            assert_eq!(vx(t, 0.0), accel, "type {t} accel");
            assert_eq!(vx(t, cap - 0.01), cap, "type {t} cap");
            let mut c = ctx(t);
            c.vx = cap + 1.0;
            let (vx, vy) = apply_ladder(&mut c).unwrap().0;
            assert!((vx - (cap + 1.0) * over).abs() < 1e-5, "type {t} over");
            assert_eq!(vy, 0.0);
        }
    }

    // ---- `NPC.cs:58262-58321` — the charge family ----

    #[test]
    fn the_charge_family_pins_the_walk_and_the_stop() {
        // ai[2] == 0: damage restores, cap 1*(1+(1-scale)), accel 0.07.
        let mut c = ctx(430);
        let ((vx, _), fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(vx, 0.07);
        assert_eq!(fx.damage, Some(20));
        // Scale 0.5 → cap 1.5 (`:58267`).
        let mut c = ctx(431);
        c.scale = 0.5;
        c.vx = 1.49;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 1.5).abs() < 1e-5);
        // The stop (`:58291-58303`): night, alive target, dist < 50,
        // CanHit → vx ×0.7 and ai[2] = 1. (vx 1.0 sits AT the cap 1, so
        // the accel arm's `vx < num105` guard skips — the C# does too.)
        let mut c = ctx(430);
        c.vx = 1.0;
        c.target_center_x = 30.0;
        let ((vx, _), fx) = apply_ladder(&mut c).unwrap();
        assert!((vx - 1.0 * 0.7).abs() < 1e-6);
        assert_eq!(fx.ai2, Some(1.0));
        // 494/495 use the 42-px range (`:58295-58298`): dist 45 stops
        // for 430 but not for 494.
        let mut c = ctx(494);
        c.vx = 1.0;
        c.target_center_x = 45.0;
        let (_, fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(fx.ai2, None);
        let mut c = ctx(430);
        c.vx = 1.0;
        c.target_center_x = 45.0;
        let (_, fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(fx.ai2, Some(1.0));
        // Day above the surface never stops (`:58291`).
        let mut c = ctx(430);
        c.vx = 1.0;
        c.target_center_x = 30.0;
        c.day_time = true;
        c.below_surface = false;
        let (_, fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(fx.ai2, None);
    }

    #[test]
    fn the_charge_state_pins_15_damage_09_decay_and_the_resets() {
        // ai[2] > 0 (`:58306-58320`): damage = (int)(defDamage * 1.5),
        // ai[3] = 1, vx ×0.9, ai[2]++.
        let mut c = ctx(430);
        c.ai2 = 1.0;
        c.vx = 1.0;
        let ((vx, _), fx) = apply_ladder(&mut c).unwrap();
        assert_eq!(vx, 0.9);
        assert_eq!(fx.damage, Some(30)); // (20 as f64 * 1.5) as i32
        assert_eq!(fx.ai3, Some(1.0));
        assert_eq!(fx.ai2, Some(2.0));
        // The ±0.1 (double) snap (`:58311-58314`).
        let mut c = ctx(430);
        c.ai2 = 1.0;
        c.vx = 0.1; // 0.1 × 0.9 = 0.09 < 0.1 → 0
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 0.0);
        // The 20-tick reset (`:58316-58319`).
        let mut c = ctx(430);
        c.ai2 = 19.0;
        assert_eq!(apply_ladder(&mut c).unwrap().1.ai2, Some(0.0));
        // The airborne reset.
        let mut c = ctx(430);
        c.ai2 = 1.0;
        c.vy = -1.0;
        assert_eq!(apply_ladder(&mut c).unwrap().1.ai2, Some(0.0));
        // The day-above-surface reset.
        let mut c = ctx(430);
        c.ai2 = 1.0;
        c.day_time = true;
        c.below_surface = false;
        assert_eq!(apply_ladder(&mut c).unwrap().1.ai2, Some(0.0));
    }

    // ---- `NPC.cs:58322-58401` — the num107 default arm ----

    #[test]
    fn the_default_arm_pins_the_rows_and_the_scale_set() {
        let rows: [(u16, f32); 10] = [
            (624, 2.5),
            (186, 1.1),
            (187, 0.9),
            (188, 1.2),
            (189, 0.8),
            (132, 0.95),
            (200, 0.87),
            (223, 1.05),
            (632, 0.8),
            (691, 0.85),
        ];
        for (t, cap) in rows {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}: `:58387` accel 0.07");
            assert!((vx(t, cap - 0.01) - cap).abs() < 1e-5, "type {t} cap");
        }
        // The base 1 (`:58324`) — the zombie (3), the warrior (28), the
        // miner (44), the DD2 fighters (552/566) and every unlisted id.
        for t in [3u16, 28, 44, 552, 566, 423] {
            assert_eq!(vx(t, 0.0), 0.07, "type {t}");
            assert_eq!(vx(t, 0.99), 1.0, "type {t} base cap 1");
        }
        // The scale set (`:58374-58377`): 3/132/186-189/200/223/331/332.
        for t in [3u16, 132, 186, 187, 188, 189, 200, 223, 331, 332] {
            let mut c = ctx(t);
            c.scale = 0.5;
            c.vx = 0.0;
            let base = match t {
                132 => 0.95,
                186 => 1.1,
                187 => 0.9,
                188 => 1.2,
                189 => 0.8,
                200 => 0.87,
                223 => 1.05,
                _ => 1.0,
            };
            // Pin the CAP: vx just under base*1.5 clamps at base*1.5.
            let mut c = ctx(t);
            c.scale = 0.5;
            c.vx = base * 1.5 - 0.01;
            let out = apply_ladder(&mut c).unwrap().0 .0;
            assert!((out - base * 1.5).abs() < 1e-5, "type {t} scaled cap ({out})");
        }
        // 26 is NOT in the scale set (it rides num83 anyway); 44 is not
        // in the default scale set — scale 0.5 keeps the cap 1.
        let mut c = ctx(44);
        c.scale = 0.5;
        c.vx = 0.99;
        assert_eq!(apply_ladder(&mut c).unwrap().0 .0, 1.0);
    }

    #[test]
    fn the_489_row_pins_the_distance_scaled_cap() {
        // `:58365-58373` — dist 200 → 200*0.0025 = 0.5 → (2.5-0.5)*0.8
        // = 1.6; the scale row applies (489 is in the set).
        let mut c = ctx(489);
        c.target_center_x = 200.0;
        c.vx = 1.59;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 1.6).abs() < 1e-5);
        // The 1.5 clamp (`:58368-58371`): dist 1000 → 2.5 → clamp 1.5
        // → (2.5-1.5)*0.8 = 0.8.
        let mut c = ctx(489);
        c.target_center_x = 1000.0;
        c.vx = 0.79;
        assert!((apply_ladder(&mut c).unwrap().0 .0 - 0.8).abs() < 1e-5);
    }

    #[test]
    fn the_shooter_exclusions_and_frost_block_decline() {
        // The `:58322` exclusion list — the shooter block owns them.
        for t in [
            110u16, 111, 206, 214, 215, 216, 290, 291, 292, 293, 350, 379, 380, 381, 382,
            449, 450, 451, 452, 468, 481, 411, 409, 498, 499, 500, 501, 502, 503, 504, 505,
            506, 424, 426, 520,
        ] {
            let mut c = ctx(t);
            assert!(apply_ladder(&mut c).is_none(), "type {t} is shooter-owned");
        }
        // The whole frost block 338-352 (the sibling owns it).
        for t in 338u16..=352 {
            let mut c = ctx(t);
            assert!(apply_ladder(&mut c).is_none(), "type {t} is frost-owned");
        }
        // The Frost Legion trio never appears in the ladder (aiStyle 38).
        for t in [143u16, 144, 145] {
            // They fall into the default arm if called directly — the
            // PORT never calls it for them (ai_style() == 38 gates the
            // wiring); pin that the C# ladder itself has no named arm.
            let mut c = ctx(t);
            let _ = apply_ladder(&mut c);
        }
    }
}
