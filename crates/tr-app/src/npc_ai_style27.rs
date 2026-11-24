//! The aiStyle-27 arm — the Wall of Flesh (`NPC.AI()`'s
//! `else if (this.aiStyle == 27)` block, `NPC.cs:25635-26052`, 418 lines).
//!
//! S1-D-3 (aiStyle-27 round): this module is the faithful 1:1 port of the
//! arm as a pure function over `Npc` + a small context seam.
//! **WIRED (S1-D-3 W-1)**: `Npc::update_rng`'s style axis claims 27 →
//! `npc.rs::ai_027_wof_family` → `style27_step`; the defective
//! `update_wof` stand-in is retired and the sim's catastrophic
//! every-tick eye/Hungry respawn loop is replaced by the one-shot init
//! drain (`world_sim/npcs.rs`, the `wof_spawns` resolve). The wiring
//! passes the `UpdateNPCs` stream as `ctx.rng` (the roar cadence's draws
//! ride the C#'s `Main.rand` swap, `Main.cs:17767-17769`), the World's
//! INT band statics through the in-out seam and the sim pre-pass's live
//! counts; the spawn requests resolve with the `NewNPC` Bottom placement.
//!
//! The only type on this style is `NPCID.WallOfFlesh` (113); its
//! `SetDefaults` row (`NPC.cs:10342-10360`): 100×100 base × scale 1.2 →
//! the scaled 120×120 hitbox (`ScaleStats`, `NPC.cs:17885-17889` +
//! `:17926-17927` — the port's static table already carries (120, 120)),
//! aiStyle 27, damage 50, defense 12, lifeMax 8000,
//! `SoundID.NPCHit8`/`NPCDeath10`, noGravity, noTileCollide, behindTiles,
//! knockBackResist 0, boss. There are no per-type branches inside the
//! arm — the whole block below is the family.
//!
//! ## RNG — every draw, cited, in stream order
//!
//! Classic single-player (the port's difficulty, `getGoodWorld` false,
//! `netMode` 0, `expertMode` false) draws `Main.rand` at exactly TWO
//! sites per tick, in this order:
//! 1. `Main.rand.Next(1000)` (`:25717`) — the roar threshold, drawn EVERY
//!    tick: the C# evaluates `600 + Main.rand.Next(1000)` on each
//!    `localAI[3] >= ...` compare, no short-circuit.
//! 2. `Main.rand.Next(200)` (`:25719`) — ONLY when the roar fires
//!    (`localAI[3] >= 600 + draw1`), the negative re-arm.
//! The `getGoodWorld` gate `Main.rand.Next(180)` (`:25648`) is the FIRST
//! operand of the `&&` chain, so on a getGood world it draws EVERY tick
//! before the tongue block; the per-attempt `Next(-50, 51)` pair
//! (`:25663-25664`) draws only inside the `< 4` demon-count gate.
//! The expert respawn roll (`:25983-26039`) draws, in order:
//! 3. `Main.rand.Next((int)((double)num365 * 0.7))` (`:25999`) — the
//!    hp-squared cadence gate, expert-only, EVERY tick;
//! 4. `Main.rand.Next(1 + num366 * 2)` (`:26012`) — only when roll 3
//!    hits 0 AND fewer than 10 Hungries live;
//! 5. `Main.rand.Next(10)` (`:26017`) — the free-slot pick loop, up to
//!    1000 attempts, only when roll 4 lands `<= 1`; it breaks on the
//!    first slot whose `i * 0.1f - 0.05f` is not already taken.
//! Nothing else in the arm draws: the tongue cycle (`ai[1]`/`ai[2]`),
//! the band scans, the speed ladder, the target picks and the init
//! spawns are deterministic counters/scans. `UnifiedRandom` is the
//! exact C# `Main.rand` stream (`tr_core::unified_random`), so passing
//! it through in this order preserves the global draw sequence; the
//! stream-clone witnesses below are WRITTEN NOT RUN (code-only mode).
//!
//! ## Defect catalogue vs the live arm (the audit verdict)
//!
//! The f1701eb0 (aiStyle-29) defect classes, checked against
//! `npc.rs::update_wof` (M4-BOSS-6b, claimed "the classic path verbatim")
//! and the sim resolve `world_sim/npcs.rs:3557-3652`:
//! * **Respawn loop (catastrophic)** — the sim spawns the TWO mirror
//!   eyes (114) EVERY tick the wall lives (`:3594-3604` sits outside
//!   the `wants_hungry` gate), and `wants_hungry`/`wants_leech` are
//!   never cleared after the resolve, so 11 Hungries (and a leech per
//!   tongue tick) re-spawn every tick too. The C# init runs ONCE
//!   (`localAI[0] == 1f` → `2f`, `:26041-26043`).
//! * **Missing band clamps** — `MathHelper.Clamp` to
//!   `[num346*16, num347*16]` and the 160-px band enforcement
//!   (`:25833-25842`) exist nowhere in the port; the sim comment even
//!   cites `:25834-25840` while implementing only the ease.
//! * **Bottom scan misses the liquid term** — the C# counts
//!   `SolidTile || liquid > 0` in BOTH walks (`:25746`/`:25801`); the
//!   live bottom walk counts solids only (`npc.rs:7347-7351`).
//! * **Missing InWorld fluff gate** — `WorldGen.InWorld(x, y, 2)` gates
//!   both inner tests (`:25746`/`:25801`); the live scans have neither
//!   it nor the defensive `catch` (`:25751-25754`, effectively
//!   unreachable once InWorld passes — documented, not emulated).
//! * **Missing TargetClosest + dead-player nearest-active pick** — the
//!   C# calls `TargetClosest()` and, when the target is dead, scans the
//!   active players by Euclidean `Distance` for the direction sign
//!   (`:25899-25921`); the live arm faces by the session player's x.
//! * **Missing TargetClosest_WOF** — `dead || !gross` re-picks among
//!   living horrified players (`:25933-25936`, `NPC.cs:77668-77682`);
//!   the live arm never re-targets (single-player port: the abandon
//!   outcome matches, but the pick's facing/rect writes are missing).
//! * **INT-division quirk** — `num355 = (Bottom + Top) / 2 - height / 2`
//!   (`:25843`) divides two INTs (the statics are `int`,
//!   `Main.cs:757`/`:759`); the live arm uses f32 halves
//!   (`npc.rs:7386`) — odd sums pin 0.5 px off.
//! * **Rotation drops the num363 scale** — the C# multiplies BOTH aim
//!   deltas by the distance `num363` BEFORE the Atan2 (`:25960-25963`);
//!   the live arm atan2's the raw deltas (`npc.rs:7437-7446`). Same
//!   angle in exact maths for positive scales, but the f32 rounding of
//!   the scaled operands differs. The player-side halves are INTEGER
//!   (`(float)(width/2)`, `:25958-25959`/`:25966`/`:25975`), the wall
//!   centre FLOAT (`:25957`) but the comparison halves INTEGER
//!   (`:25966`/`:25975`); the offset is the literal `3.14f`.
//! * **Missing roar RNG cadence** — `localAI[3]++`, the
//!   `600 + Next(1000)` threshold and the `-Next(200)` re-arm
//!   (`:25716-25721`) are skipped as "cosmetic" — but they DRAW from
//!   `Main.rand` every tick, so skipping them desynchronises every
//!   later draw in the shared stream.
//! * **Missing localAI[0] == 0 band reset** — the first tick sets
//!   `wofDrawAreaBottom/Top = -1` (`:25642-25647`); the live arm relies
//!   on the World's initial −1 and never re-arms it for a second wall.
//! * **Missing expert arms** — the five expert speed bonuses + the
//!   `*1.35f + 0.35f` scale (`:25871-25894`) and the respawn roll
//!   (`:25983-26039`) are absent (documented as out, but the arm claims
//!   "verbatim classic" while also missing the classic pieces above).
//! * **Missing getGoodWorld arms** — the demon spawn (`:25648-25679`)
//!   and the `*1.1f + 0.2f` speed scale (`:25895-25898`).
//! * **Check order** — the C# order is: edge kill (NO return — the tick
//!   continues, `:25637-25640`) → NetUpdateLowPriority → localAI[0]
//!   init → getGood demon → tongue → roar → scans/ease/clamp → pin →
//!   ladder → direction → WOF re-target → abandonment (return) →
//!   sprite/rotation → expert respawn → init spawns. The live arm
//!   returns on the edge kill and folds the abandonment timer into
//!   `ai[0]` instead of `localAI[1]` (`:25939`/`:25954`).
//! * **NewNPC Bottom placement** — `NewNPC` sets `Bottom = (X, Y)`
//!   (`NPC.cs:80770`, `Entity.Bottom` setter: `position = (X - width/2,
//!   Y - height)`); the sim spawns the eyes/Hungries/leeches at raw
//!   `(X, Y)` top-left (`world_sim/npcs.rs:3576-3611`) — every minion
//!   sits half a box right and a full box low.
//!
//! Faithfulness notes (documented deviations, same conventions as the
//! sibling style-28/29 rounds):
//! * `this.active = false` (`:25639`/`:25944`) → the port's hp-0 death
//!   convention. The edge kill does NOT return in the C# — the dead
//!   wall keeps running the tick (its writes land on a corpse); kept.
//! * `NetUpdateLowPriority` (`:25641`) sets
//!   `netUpdatePendingFullSpamCooldown` — the port's `net_update` flag
//!   stands in (the net-spam cooldown system is unported).
//! * `SoundEngine.PlaySound(4, ...)` (`:25720`/`:25942`) — the roar /
//!   death cues queue on `wants_sound` (S1-D-3 W-1d, the S1-E-21g
//!   drain): the int killed overload style 10, one case-4 pitch draw
//!   at the slot's own turn on the `UpdateNPCs` stream. The RNG draws
//!   around them were always kept.
//! * `Main.wofNPCIndex = whoAmI` (`:25724`) → `ctx.wof_npc_index` is an
//!   out-field the caller writes back to the World presence seam.
//! * The band statics are INTs in the C# (`Main.cs:757`/`:759`); the
//!   context carries them as `i32` and the caller owns the write-back
//!   (S1-D-3 W-1 converted `World.wof_draw_top/bottom` f32 → i32; the
//!   sim copies the arm's staged values right after the wall's update).
//! * `NPC.NewNPC` spawns are REQUESTS (`Style27Spawn`), resolved by the
//!   caller in C# order: leech (`:25712`), respawn Hungry (`:26036`),
//!   then the init eyes + 11 Hungries (`:26044-26049`). Each carries
//!   the exact `Bottom` payload + `Start`/`ai0` arguments; the
//!   `CountNPCS` gates (`:25710` < 10 leeches, `:25648` < 4 demons,
//!   `:26005`/`:26012` < 10 Hungries) ride the caller's live counts.
//! * `getGoodWorld` demon spawn (`:25648-25679`) is ported verbatim
//!   behind `ctx.get_good_world` incl. the 1000-attempt column walk;
//!   the wiring passes false (single-difficulty classic port).
//! * `Player.gross` (Horrified) has no `TargetCandidate` field yet —
//!   `ctx.player_gross` is the parallel per-slot slice the wiring fills
//!   from `Player.gross` (`player.rs:1270`, S1-E-7c).
//! * The arm does NOT integrate `position += velocity` — the wall is
//!   `noTileCollide` (`:10354`), `UpdateNPC` owns the step (the live
//!   arm integrates inline; the wiring moves it out).
#![allow(dead_code)] // WIRED (S1-D-3 W-1); the expert/getGood arms stay parked behind the classic flags

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// One `NPC.NewNPC` request the arm emits, in emission order. The
/// caller resolves it with the `Bottom = (x, y)` placement
/// (`NPC.cs:80770`): `position = (x - width/2, y - height)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style27Spawn {
    /// The vanilla type id (117 leech head, 114 eye, 115 Hungry,
    /// 24 demon).
    pub npc_type: i32,
    /// The `X` argument (already `(int)`-truncated by the arm).
    pub x: i32,
    /// The `Y` argument (the Bottom payload).
    pub y: i32,
    /// The `Start` argument (`base.whoAmI` for the wall's parts).
    pub start: i32,
    /// The `ai0` argument.
    pub ai0: f32,
    /// `velocity.X = direction * 8` (`:25713`) — leech heads only.
    pub vx: Option<f32>,
}

/// The `Main` statics + world flags the arm reads/writes through the
/// caller. The band fields are IN-OUT: the arm eases/clamps them in
/// place exactly like the C# mutates `Main.wofDrawAreaTop/Bottom`.
pub struct Style27Context<'a> {
    /// `Main.wofDrawAreaTop` (`Main.cs:757`) — INT, in-out.
    pub wof_draw_top: &'a mut i32,
    /// `Main.wofDrawAreaBottom` (`Main.cs:759`) — INT, in-out.
    pub wof_draw_bottom: &'a mut i32,
    /// `Main.wofNPCIndex` out-write (`:25724`) — the caller stores the
    /// wall's slot here (the port's presence seam).
    pub wof_npc_index: &'a mut i32,
    /// `Main.expertMode` (`:25871-25894`, `:25983`).
    pub expert: bool,
    /// `Main.getGoodWorld` (`:25648`, `:25895`).
    pub get_good_world: bool,
    /// `Main.netMode != 1` — the port is single-player/server, true.
    pub not_client: bool,
    /// The live `NPC.CountNPCS(117)` count (`:25710`).
    pub leech_head_count: i32,
    /// The live `NPC.CountNPCS(24)` count (`:25648`, getGood-only).
    pub demon_count: i32,
    /// The live Hungry (115) `ai[0]` values in SLOT order
    /// (`:26003-26010` collects up to 10; `:26012`'s `num366 < 10`
    /// gate reads the collected count).
    pub hungry_ai0s: &'a [f32],
    /// The target-scan slice (the C# loops 255 `Main.player` slots;
    /// the port's session player is slot 0).
    pub players: &'a [TargetCandidate],
    /// `Player.gross` per slot (`:25933`, `TargetClosest_WOF`'s
    /// `:77676` gate) — the parallel slice, same indexing as `players`.
    pub player_gross: &'a [bool],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
    pub ticker: u64,
    /// `Main.rand` — the shared stream, drawn in the arm's order.
    pub rng: &'a mut UnifiedRandom,
    /// The spawn requests, appended in C# emission order.
    pub spawns: &'a mut Vec<Style27Spawn>,
}

/// `NPC.AI()`'s aiStyle-27 arm (`NPC.cs:25635-26052`), one tick.
pub fn style27_step(npc: &mut Npc, world: &World, ctx: &mut Style27Context) {
    let (w, h) = npc.size();
    let wi = w as i32;
    let hi = h as i32;
    // `:25637-25640` — the world-edge kill. NOTE: no return — the C#
    // deactivates and keeps running the tick.
    if npc.x < 160.0 || npc.x > ((world.width - 10) * 16) as f32 {
        npc.hp = 0; // `this.active = false` — the port's death convention
    }
    // `:25641` — NetUpdateLowPriority (the net-spam flag stand-in).
    npc.net_update = true;
    // `:25642-25647` — the first-tick band reset.
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        *ctx.wof_draw_bottom = -1;
        *ctx.wof_draw_top = -1;
    }
    // `:25648-25679` — the getGoodWorld demon spawn. The Next(180) is
    // the FIRST && operand: it draws every tick on a getGood world.
    if ctx.get_good_world
        && ctx.not_client
        && ctx.rng.next_max(180) == 0
        && ctx.demon_count < 4
    {
        // num336 = 1 (`:25650`) — one spawn, up to 1000 column attempts.
        for _ in 0..1000 {
            // `:25656-25662` — the base column re-read EACH attempt.
            let mut num340 = ((npc.x + w * 0.5) / 16.0) as i32;
            let mut num341 = ((npc.y + h * 0.5) / 16.0) as i32;
            if npc.target >= 0 && (npc.target as usize) < ctx.players.len() {
                let p = &ctx.players[npc.target as usize];
                num340 = ((p.x + p.width as f32 / 2.0) / 16.0) as i32;
                num341 = ((p.y + p.height as f32 / 2.0) / 16.0) as i32;
            }
            // `:25663-25664` — the per-attempt jitter pair, in order.
            let num340 = num340 + ctx.rng.next_range(-50, 51);
            let mut num341 = num341 + ctx.rng.next_range(-50, 51);
            while num341 < world.height - 10 && !solid_tile(world, num340, num341) {
                num341 += 1;
            }
            num341 -= 1;
            if !solid_tile(world, num340, num341) {
                // `:25670` — NewNPC at (col*16+8, row*16), type 24.
                ctx.spawns.push(Style27Spawn {
                    npc_type: 24,
                    x: num340 * 16 + 8,
                    y: num341 * 16,
                    start: 0,
                    ai0: 0.0,
                    vx: None,
                });
                break;
            }
        }
    }
    // `:25680-25695` — the tongue timer with the hp bonuses.
    npc.ai[1] += 1.0;
    if npc.ai[2] == 0.0 {
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
            npc.ai[1] += 1.0;
        }
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.2 {
            npc.ai[1] += 1.0;
        }
        if npc.ai[1] > 2700.0 {
            npc.ai[2] = 1.0;
        }
    }
    // `:25696-25715` — the tongue phase machine + the leech spawn.
    let num343 = 60;
    if npc.ai[2] > 0.0 && npc.ai[1] > num343 as f32 {
        let mut num344 = 3;
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.3 {
            num344 += 1;
        }
        npc.ai[2] += 1.0;
        npc.ai[1] = 0.0;
        if npc.ai[2] > num344 as f32 {
            npc.ai[2] = 0.0;
        }
        if ctx.not_client && ctx.leech_head_count < 10 {
            // `:25712-25713` — the INTEGER half-extents on the spawn
            // payload, Start = 1, velocity.X = direction * 8.
            ctx.spawns.push(Style27Spawn {
                npc_type: 117,
                x: (npc.x + (wi / 2) as f32) as i32,
                y: (npc.y + (hi / 2) as f32 + 20.0) as i32,
                start: 1,
                ai0: 0.0,
                vx: Some(npc.facing as f32 * 8.0),
            });
        }
    }
    // `:25716-25721` — the roar cadence. BOTH draws are stream-visible
    // even though the sound cue itself rides the sound family.
    npc.local_ai[3] += 1.0;
    if npc.local_ai[3] >= (600 + ctx.rng.next_max(1000)) as f32 {
        npc.local_ai[3] = -ctx.rng.next_max(200) as f32;
        // `:25720` — SoundEngine.PlaySound(4, (int)position.X,
        // (int)position.Y, 10): the int killed overload, style 10. The
        // cue queues on `wants_sound` and drains at the slot's own turn
        // on the UpdateNPCs stream (the case-4 pitch draw, S1-E-21g).
        npc.wants_sound.push(crate::npc::NpcSoundRequest {
            cue: crate::npc::NpcSoundCue::NpcDeath(10),
            x: npc.x,
            y: npc.y,
        });
    }
    // `:25722-25727` — the scan bounds. `Main.UnderworldLayer =
    // maxTilesY - 200` (`Main.cs:2790`).
    let underworld_layer = world.height - 200;
    let num346 = underworld_layer + 10;
    let num347 = num346 + 70;
    *ctx.wof_npc_index = npc.slot as i32; // `:25724`
    let num348 = (npc.x / 16.0) as i32;
    let num349 = ((npc.x + wi as f32) / 16.0) as i32;
    // `:25727` — the INTEGER half-height.
    let num350 = ((npc.y + (hi / 2) as f32) / 16.0) as i32;
    // `:25728-25756` — the bottom walk: 15 solid-or-liquid tiles.
    let mut num351 = 0;
    let mut num352 = num350 + 7;
    while num351 < 15 && num352 > underworld_layer {
        num352 += 1;
        if num352 > world.height - 10 {
            num352 = world.height - 10;
            break;
        }
        if num352 < num346 {
            continue;
        }
        for num353 in num348..=num349 {
            // `:25746` — InWorld(x, y, 2) gates the test; an
            // out-of-fluff column counts NOTHING (the `catch` at
            // `:25751-25754` is defensive — InWorld already bounds the
            // tile access, so it is effectively unreachable).
            if in_world_fluff(world, num353, num352, 2)
                && (solid_tile(world, num353, num352) || liquid_at(world, num353, num352))
            {
                num351 += 1;
            }
        }
    }
    num352 += 4;
    // `:25758-25777` — the bottom ease, 1 px per tick.
    ease_band(ctx.wof_draw_bottom, num352 * 16);
    // `:25778-25811` — the top walk mirrors it.
    num351 = 0;
    num352 = num350 - 7;
    while num351 < 15 && num352 < world.height - 10 {
        num352 -= 1;
        if num352 <= 10 {
            num352 = 10;
            break;
        }
        if num352 > num347 {
            continue;
        }
        if num352 < num346 {
            num352 = num346;
            break;
        }
        for num354 in num348..=num349 {
            // `:25801` — the same InWorld + solid-or-liquid test.
            if in_world_fluff(world, num354, num352, 2)
                && (solid_tile(world, num354, num352) || liquid_at(world, num354, num352))
            {
                num351 += 1;
            }
        }
    }
    num352 -= 4;
    // `:25813-25832` — the top ease.
    ease_band(ctx.wof_draw_top, num352 * 16);
    // `:25833-25842` — the band clamps + the 160-px enforcement.
    *ctx.wof_draw_top = (*ctx.wof_draw_top).clamp(num346 * 16, num347 * 16);
    *ctx.wof_draw_bottom = (*ctx.wof_draw_bottom).clamp(num346 * 16, num347 * 16);
    if *ctx.wof_draw_top > *ctx.wof_draw_bottom - 160 {
        *ctx.wof_draw_top = *ctx.wof_draw_bottom - 160;
    } else if *ctx.wof_draw_bottom < *ctx.wof_draw_top + 160 {
        *ctx.wof_draw_bottom = *ctx.wof_draw_top + 160;
    }
    // `:25843-25853` — the Y pinning. `(Bottom + Top) / 2` and
    // `height / 2` are INT divisions (`:25843`); the ±1 velocity
    // writes are dead stores (the C# zeroes Y right after) — kept.
    let num355 = ((*ctx.wof_draw_bottom + *ctx.wof_draw_top) / 2 - hi / 2) as f32;
    if npc.y > num355 + 1.0 {
        npc.vy = -1.0;
    } else if npc.y < num355 - 1.0 {
        npc.vy = 1.0;
    }
    npc.vy = 0.0;
    npc.y = num355;
    // `:25854-25898` — the slide-speed ladder.
    let mut num356 = 1.5f32;
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
        num356 += 0.25;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
        num356 += 0.4;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
        num356 += 0.5;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
        num356 += 0.6;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.66 && ctx.expert {
        num356 += 0.3;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.33 && ctx.expert {
        num356 += 0.3;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.05 && ctx.expert {
        num356 += 0.6;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.035 && ctx.expert {
        num356 += 0.6;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.025 && ctx.expert {
        num356 += 0.6;
    }
    if ctx.expert {
        num356 = num356 * 1.35 + 0.35;
    }
    if ctx.get_good_world {
        num356 = num356 * 1.1 + 0.2;
    }
    // `:25899-25922` — the direction pick: TargetClosest, then the
    // dead-target nearest-ACTIVE-player Euclidean scan for the sign.
    if npc.vx == 0.0 {
        target_closest(npc, world, ctx.players, true, 27, ctx.ticker);
        let tp_dead = target_player(ctx.players, npc.target)
            .map(|p| p.dead)
            .unwrap_or(false);
        if tp_dead {
            let mut num357 = f32::INFINITY;
            let mut num358 = 0;
            let (cx, cy) = npc.center();
            // `:25906-25918` — the C# loops 255 slots but re-reads
            // `Main.player[this.target]` each iteration (a decompiler
            // artifact of the inlined scan); the effective behaviour
            // is the nearest active player by `Distance` (Euclidean,
            // `Entity.Distance` → `Vector2.Distance`). The port scans
            // the slice in slot order with the same strict `>` pick.
            for p in ctx.players.iter() {
                if p.active {
                    let pcx = p.x + p.width as f32 / 2.0;
                    let pcy = p.y + p.height as f32 / 2.0;
                    let num360 = ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
                    if num357 > num360 {
                        num357 = num360;
                        num358 = if cx < pcx { 1 } else { -1 };
                    }
                }
            }
            npc.facing = num358;
        }
        npc.vx = npc.facing as f32;
    }
    // `:25923-25932` — the one-sided speed lock.
    if npc.vx < 0.0 {
        npc.vx = 0.0 - num356;
        npc.facing = -1;
    } else {
        npc.vx = num356;
        npc.facing = 1;
    }
    // `:25933-25936` — TargetClosest_WOF when the target is dead or
    // not gross (Horrified). The shared scan with the gross gate.
    {
        let idx = target_slot(npc.target);
        let tp = ctx.players.get(idx);
        let gross = ctx.player_gross.get(idx).copied().unwrap_or(false);
        if tp.map(|p| p.dead).unwrap_or(false) || !gross {
            target_closest_wof(npc, world, ctx.players, ctx.player_gross, ctx.ticker);
        }
    }
    // `:25937-25955` — the dead-player abandonment on localAI[1].
    let tp_dead = target_player(ctx.players, npc.target)
        .map(|p| p.dead)
        .unwrap_or(false);
    if tp_dead {
        npc.local_ai[1] += 1.0 / 180.0;
        if npc.local_ai[1] >= 1.0 {
            // `:25942` — SoundEngine.PlaySound(4, (int)position.X,
            // (int)position.Y, 10): the abandonment death cry, queued
            // BEFORE the life=0 kill (the C#'s order). The corpse's
            // `wants_sound` still drains at the slot's tail — the sim's
            // drain loop runs for every slot, dead included.
            npc.wants_sound.push(crate::npc::NpcSoundRequest {
                cue: crate::npc::NpcSoundCue::NpcDeath(10),
                x: npc.x,
                y: npc.y,
            });
            npc.hp = 0; // `life = 0; active = false`
            return;
        }
    } else {
        npc.local_ai[1] = (npc.local_ai[1] - 1.0 / 30.0).clamp(0.0, 1.0);
    }
    // `:25956-25982` — the sprite flip + the eye rotation. The aim
    // deltas scale by num363 (the distance) BEFORE the Atan2
    // (`:25960-25963`); the player halves are INTEGER, the wall
    // centre FLOAT (`:25957`), the comparison halves INTEGER
    // (`:25966`/`:25975`); the offset is the literal 3.14f.
    npc.sprite_dir = npc.facing;
    let vector33_x = npc.x + w * 0.5;
    let vector33_y = npc.y + h * 0.5;
    let (px, py, pw, ph) = match target_player(ctx.players, npc.target) {
        Some(p) => (p.x, p.y, p.width, p.height),
        None => (0.0, 0.0, 0, 0),
    };
    let mut num361 = px + (pw / 2) as f32 - vector33_x;
    let mut num362 = py + (ph / 2) as f32 - vector33_y;
    let num363 = (num361 * num361 + num362 * num362).sqrt();
    num361 *= num363;
    num362 *= num363;
    let player_half_x = px + (pw / 2) as f32;
    let wall_half_x = npc.x + (wi / 2) as f32;
    if npc.facing > 0 {
        if player_half_x > wall_half_x {
            npc.rotation = (-num362).atan2(-num361) + 3.14;
        } else {
            npc.rotation = 0.0;
        }
    } else if player_half_x < wall_half_x {
        npc.rotation = num362.atan2(num361) + 3.14;
    } else {
        npc.rotation = 0.0;
    }
    // `:25983-26040` — the expert Hungry respawn roll.
    if ctx.expert && ctx.not_client {
        // `:25985-25998` — the hp-squared cadence with the three
        // INT-division floors. `(float)life / (float)lifeMax` is a
        // FLOAT ratio here (unlike the Hungry arm's :26341 INT quirk).
        let mut num365 = (1.0 + npc.hp as f32 / npc.max_hp as f32 * 10.0) as i32;
        num365 *= num365;
        if num365 < 400 {
            num365 = (num365 * 19 + 400) / 20;
        }
        if num365 < 60 {
            num365 = (num365 * 3 + 60) / 4;
        }
        if num365 < 20 {
            num365 = (num365 + 20) / 2;
        }
        // `:25999` — the cadence gate draw (every expert tick).
        if ctx.rng.next_max((num365 as f64 * 0.7) as i32) == 0 {
            // `:26001-26010` — collect up to 10 live Hungry ai[0]s in
            // slot order (the caller pre-collects; the C# caps at 10).
            let mut array = [0.0f32; 10];
            let mut num366 = 0usize;
            for &a in ctx.hungry_ai0s.iter() {
                if num366 < 10 {
                    array[num366] = a;
                    num366 += 1;
                }
            }
            let max_value = 1 + num366 as i32 * 2;
            // `:26012` — the second draw, only under the < 10 gate.
            if num366 < 10 && ctx.rng.next_max(max_value) <= 1 {
                let mut num368 = -1i32;
                for _ in 0..1000 {
                    // `:26017-26032` — the free-slot pick loop.
                    let num370 = ctx.rng.next_max(10);
                    let num371 = num370 as f32 * 0.1 - 0.05;
                    let mut flag41 = true;
                    for &taken in array.iter().take(num366) {
                        if num371 == taken {
                            flag41 = false;
                            break;
                        }
                    }
                    if flag41 {
                        num368 = num370;
                        break;
                    }
                }
                if num368 >= 0 {
                    // `:26036` — the respawn at (position.X, num355).
                    ctx.spawns.push(Style27Spawn {
                        npc_type: 115,
                        x: npc.x as i32,
                        y: num355 as i32,
                        start: npc.slot as i32,
                        ai0: num368 as f32 * 0.1 - 0.05,
                        vx: None,
                    });
                }
            }
        }
    }
    // `:26041-26051` — the ONE-TIME init: two mirror eyes + 11 Hungries.
    if ctx.not_client && npc.local_ai[0] == 1.0 {
        npc.local_ai[0] = 2.0;
        let center_y = npc.y + h * 0.5;
        // `:26044` — the top eye, ai0 = 1.
        ctx.spawns.push(Style27Spawn {
            npc_type: 114,
            x: npc.x as i32,
            y: ((center_y + *ctx.wof_draw_top as f32) / 2.0) as i32,
            start: npc.slot as i32,
            ai0: 1.0,
            vx: None,
        });
        // `:26045` — the bottom eye, ai0 = -1.
        ctx.spawns.push(Style27Spawn {
            npc_type: 114,
            x: npc.x as i32,
            y: ((center_y + *ctx.wof_draw_bottom as f32) / 2.0) as i32,
            start: npc.slot as i32,
            ai0: -1.0,
            vx: None,
        });
        // `:26046-26050` — the 11 Hungries at the bottom-half row.
        let num374 = (center_y + *ctx.wof_draw_bottom as f32) / 2.0;
        for num375 in 0..11 {
            ctx.spawns.push(Style27Spawn {
                npc_type: 115,
                x: npc.x as i32,
                y: num374 as i32,
                start: npc.slot as i32,
                ai0: num375 as f32 * 0.1 - 0.05,
                vx: None,
            });
        }
    }
}

/// `NPC.TargetClosest_WOF` (`NPC.cs:77668-77682`): the shared tracking
/// scan gated on `active && !dead && !ghost && gross` — the Horrified
/// filter. Rides `npc_target`'s `TryTrackingTarget`/
/// `SetTargetTrackingValues` through a filtered candidate view (a
/// non-gross slot is marked inactive so the shared scan skips it,
/// exactly the C# gate).
fn target_closest_wof(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    gross: &[bool],
    ticker: u64,
) {
    let filtered: Vec<TargetCandidate> = players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut q = *p;
            if !(p.active && !p.dead && !p.ghost && gross.get(i).copied().unwrap_or(false)) {
                q.active = false;
            }
            q
        })
        .collect();
    target_closest(npc, world, &filtered, true, 27, ticker);
}

/// The C#'s slot-0 fallback for an unset target
/// (`SetTargetTrackingValues`, `NPC.cs:77731-77734`).
fn target_slot(target: i32) -> usize {
    if target < 0 || target >= 255 {
        0
    } else {
        target as usize
    }
}

/// The `Main.player[this.target]` read with the slot-0 fallback.
fn target_player<'a>(players: &'a [TargetCandidate], target: i32) -> Option<&'a TargetCandidate> {
    players.get(target_slot(target))
}

/// `WorldGen.SolidTile` — the active + solid-set test (the port's
/// `TILE_SOLID` analog, same as the live arm's closure).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
}

/// `Main.tile[x, y].liquid > 0` (`:25746`/`:25801`).
fn liquid_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.liquid > 0)
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs`): the fluff-margin
/// bounds test.
fn in_world_fluff(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// The 1-px-per-tick band ease (`:25758-25777`/`:25813-25832`), shared
/// by the top and bottom statics.
fn ease_band(band: &mut i32, target: i32) {
    if *band == -1 {
        *band = target;
    } else if *band > target {
        *band -= 1;
        if *band < target {
            *band = target;
        }
    } else if *band < target {
        *band += 1;
        if *band > target {
            *band = target;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::FLAG_ACTIVE;

    const GROSS: &[bool] = &[true];

    fn wall(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::WallOfFlesh, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n.slot = 7;
        n
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

    fn underworld() -> World {
        // 200×400: UnderworldLayer = 200, the band rows 210..280.
        World::new("t".into(), 200, 400)
    }

    struct Harness {
        top: i32,
        bottom: i32,
        index: i32,
        rng: UnifiedRandom,
        spawns: Vec<Style27Spawn>,
    }

    impl Harness {
        fn new(seed: i32) -> Self {
            Self {
                top: -1,
                bottom: -1,
                index: -1,
                rng: UnifiedRandom::new(seed),
                spawns: Vec::new(),
            }
        }

        fn step(&mut self, npc: &mut Npc, world: &World, players: &[TargetCandidate]) {
            let mut ctx = Style27Context {
                wof_draw_top: &mut self.top,
                wof_draw_bottom: &mut self.bottom,
                wof_npc_index: &mut self.index,
                expert: false,
                get_good_world: false,
                not_client: true,
                leech_head_count: 0,
                demon_count: 0,
                hungry_ai0s: &[],
                players,
                player_gross: GROSS,
                ticker: 0,
                rng: &mut self.rng,
                spawns: &mut self.spawns,
            };
            style27_step(npc, world, &mut ctx);
        }
    }

    /// The init fires ONCE: tick 1 emits 2 eyes + 11 Hungries in C#
    /// order, tick 2 emits nothing (`localAI[0]` 1 → 2, `:26041-26043`).
    /// The live sim re-spawns every tick — the catastrophic defect.
    #[test]
    fn the_init_spawns_fire_once_in_csharp_order() {
        let w = underworld();
        let mut n = wall(1000.0, 220.0 * 16.0);
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(42);
        h.step(&mut n, &w, &players);
        assert_eq!(n.local_ai[0], 2.0);
        assert_eq!(h.spawns.len(), 13);
        assert_eq!(h.spawns[0].npc_type, 114);
        assert_eq!(h.spawns[0].ai0, 1.0);
        assert_eq!(h.spawns[1].npc_type, 114);
        assert_eq!(h.spawns[1].ai0, -1.0);
        for i in 0..11 {
            assert_eq!(h.spawns[2 + i].npc_type, 115);
            assert_eq!(h.spawns[2 + i].ai0, i as f32 * 0.1 - 0.05);
            assert_eq!(h.spawns[2 + i].start, 7);
        }
        h.spawns.clear();
        h.step(&mut n, &w, &players);
        assert!(h.spawns.is_empty(), "the init must not re-fire");
    }

    /// The first tick resets both band statics to −1 (`:25645-25646`)
    /// then the ease snaps them to the scan targets (`:25758`/`:25813`)
    /// and the clamps hold (`:25833-25834`).
    #[test]
    fn the_first_tick_resets_and_snaps_the_band() {
        let w = underworld();
        let mut n = wall(1000.0, 220.0 * 16.0);
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(1);
        h.top = 9999;
        h.bottom = 9999;
        h.step(&mut n, &w, &players);
        // Air world: the bottom walk runs to maxTilesY-10 = 390, +4 →
        // 394*16 = 6304, clamped to num347*16 = 280*16 = 4480. The top
        // walk breaks at num346 = 210, −4 → 206*16 = 3296, clamped up
        // to 210*16 = 3360. The 160-px rule does not fire (4480 − 3360
        // = 1120 > 160).
        assert_eq!(h.top, 3360);
        assert_eq!(h.bottom, 4480);
        assert_eq!(h.index, 7);
    }

    /// The band clamps (`:25833-25842`): both statics stay inside
    /// [num346*16, num347*16] and the band is at least 160 px — two
    /// close statics get pushed apart by the `:25835-25838` arm.
    #[test]
    fn the_band_clamps_hold_the_160px_minimum() {
        let w = underworld();
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0; // skip the reset
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(1);
        h.top = 4470;
        h.bottom = 4471;
        h.step(&mut n, &w, &players);
        assert!(h.top >= 3360 && h.top <= 4480);
        assert!(h.bottom >= 3360 && h.bottom <= 4480);
        assert!(h.bottom - h.top >= 160);
    }

    /// The Y pin uses INT halves (`:25843`): an odd band sum truncates
    /// (a float port would pin 0.5 px lower).
    #[test]
    fn the_y_pin_divides_ints() {
        let w = underworld();
        let mut n = wall(1000.0, 0.0);
        n.local_ai[0] = 2.0;
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(1);
        // y = 0 → num350 = 3: the bottom walk never enters (10 > 200
        // false) → target (3+7+4)*16 = 224; the top walk breaks at 10
        // → target (10−4)*16 = 96. The eases step top 3361 → 3360
        // (clamped) and bottom 3522 → 3521; the 160-px rule does not
        // fire (3521 − 3360 = 161). Sum 6881 / 2 = 3440 (INT), − 60.
        h.top = 3361;
        h.bottom = 3522;
        h.step(&mut n, &w, &players);
        assert_eq!(h.top, 3360);
        assert_eq!(h.bottom, 3521);
        assert_eq!(n.y, 3380.0); // float halves would give 3380.5
        assert_eq!(n.vy, 0.0);
    }

    /// The speed ladder (`:25854-25870`): the classic rows sum
    /// 1.5/1.75/2.15/2.65/3.25 at the 75/50/25/10% thresholds, f64
    /// compares against the raw ints.
    #[test]
    fn the_classic_speed_ladder_matches_the_thresholds() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        for (hp, want) in [
            (8000, 1.5f32),
            (5999, 1.75),
            (3999, 2.15),
            (1999, 2.65),
            (799, 3.25),
        ] {
            let mut n = wall(1000.0, 220.0 * 16.0);
            n.hp = hp;
            n.vx = 1.0; // skip the target pick
            let mut h = Harness::new(1);
            h.step(&mut n, &w, &players);
            assert!((n.vx - want).abs() < 1e-5, "hp {hp}: {} != {want}", n.vx);
            assert_eq!(n.facing, 1);
            assert_eq!(n.sprite_dir, 1);
        }
    }

    /// The expert ladder (`:25871-25894`): at full hp the expert scale
    /// alone gives 1.5*1.35+0.35 = 2.375; below 2.5% hp all five
    /// bonuses fire on top of the classic rows.
    #[test]
    fn the_expert_ladder_scales_and_bonuses() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.vx = 1.0;
        let mut h = Harness::new(1);
        let mut ctx = Style27Context {
            wof_draw_top: &mut h.top,
            wof_draw_bottom: &mut h.bottom,
            wof_npc_index: &mut h.index,
            expert: true,
            get_good_world: false,
            not_client: true,
            leech_head_count: 0,
            demon_count: 0,
            hungry_ai0s: &[],
            players: &players,
            player_gross: GROSS,
            ticker: 0,
            rng: &mut h.rng,
            spawns: &mut h.spawns,
        };
        style27_step(&mut n, &w, &mut ctx);
        assert!((n.vx - 2.375).abs() < 1e-5);
        let mut n2 = wall(1000.0, 220.0 * 16.0);
        n2.hp = 100; // < 2.5% of 8000
        n2.vx = 1.0;
        let mut h2 = Harness::new(1);
        let mut ctx2 = Style27Context {
            wof_draw_top: &mut h2.top,
            wof_draw_bottom: &mut h2.bottom,
            wof_npc_index: &mut h2.index,
            expert: true,
            get_good_world: false,
            not_client: true,
            leech_head_count: 0,
            demon_count: 0,
            hungry_ai0s: &[],
            players: &players,
            player_gross: GROSS,
            ticker: 0,
            rng: &mut h2.rng,
            spawns: &mut h2.spawns,
        };
        style27_step(&mut n2, &w, &mut ctx2);
        let want = (1.5 + 0.25 + 0.4 + 0.5 + 0.6 + 0.3 + 0.3 + 0.6 + 0.6 + 0.6) * 1.35 + 0.35;
        assert!((n2.vx - want).abs() < 1e-4);
    }

    /// The tongue cycle (`:25680-25715`): ai[1] crosses 2700 → ai[2]=1,
    /// then every 61 ticks a leech request with Start=1 and vx=dir*8;
    /// the spawn payload uses the PRE-pin y and INTEGER half-extents.
    #[test]
    fn the_tongue_cycle_emits_leech_requests() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.ai[1] = 2700.0;
        n.vx = 1.0;
        let mut h = Harness::new(1);
        // ai[1] 2701 > 2700 → ai[2] = 1, and the SAME tick's spawn
        // block (`:25697` follows `:25691`) fires: ai[1] 2701 > 60 →
        // ai[2] = 2, ai[1] = 0, one leech request.
        h.step(&mut n, &w, &players);
        assert_eq!(n.ai[2], 2.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(h.spawns.len(), 1);
        n.ai[1] = 61.0;
        h.spawns.clear();
        let pre_y = n.y;
        h.step(&mut n, &w, &players);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(n.ai[2], 3.0);
        assert_eq!(h.spawns.len(), 1);
        let s = h.spawns[0];
        assert_eq!(s.npc_type, 117);
        assert_eq!(s.start, 1);
        assert_eq!(s.vx, Some(8.0));
        assert_eq!(s.x, (1000.0 + 60.0) as i32);
        assert_eq!(s.y, (pre_y + 60.0 + 20.0) as i32);
    }

    /// The leech gate: CountNPCS(117) >= 10 suppresses the spawn but
    /// the phase machine still advances (`:25710`).
    #[test]
    fn the_leech_count_gate_suppresses_only_the_spawn() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.ai[2] = 1.0;
        n.ai[1] = 61.0;
        n.vx = 1.0;
        let mut h = Harness::new(1);
        let mut ctx = Style27Context {
            wof_draw_top: &mut h.top,
            wof_draw_bottom: &mut h.bottom,
            wof_npc_index: &mut h.index,
            expert: false,
            get_good_world: false,
            not_client: true,
            leech_head_count: 10,
            demon_count: 0,
            hungry_ai0s: &[],
            players: &players,
            player_gross: GROSS,
            ticker: 0,
            rng: &mut h.rng,
            spawns: &mut h.spawns,
        };
        style27_step(&mut n, &w, &mut ctx);
        assert!(h.spawns.is_empty());
        assert_eq!(n.ai[2], 2.0);
    }

    /// The edge kill (`:25637-25640`) deactivates WITHOUT returning —
    /// the tick continues (the init still fires on the corpse tick).
    #[test]
    fn the_edge_kill_does_not_return() {
        let w = underworld();
        let mut n = wall(100.0, 220.0 * 16.0); // x < 160
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(1);
        h.step(&mut n, &w, &players);
        assert_eq!(n.hp, 0);
        assert_eq!(n.local_ai[0], 2.0, "the tick ran past the kill");
        assert_eq!(h.spawns.len(), 13);
    }

    /// The abandonment (`:25937-25955`): a dead target ramps
    /// localAI[1] by 1/180 per tick, kills at >= 1 and RETURNS; a
    /// living target recovers by 1/30 clamped to [0, 1].
    #[test]
    fn the_abandonment_timer_ramps_and_recovers() {
        let w = underworld();
        let mut dead = cand(1200.0, 220.0 * 16.0);
        dead.dead = true;
        let players = [dead];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.local_ai[1] = 1.0 - 1.0 / 360.0;
        n.vx = 1.0;
        let mut h = Harness::new(1);
        h.step(&mut n, &w, &players);
        assert_eq!(n.hp, 0);
        // Recovery: two living ticks off 1.0 → 1 − 2/30.
        let live = [cand(1200.0, 220.0 * 16.0)];
        let mut n2 = wall(1000.0, 220.0 * 16.0);
        n2.local_ai[0] = 2.0;
        n2.local_ai[1] = 1.0;
        n2.vx = 1.0;
        let mut h2 = Harness::new(1);
        h2.step(&mut n2, &w, &live);
        h2.step(&mut n2, &w, &live);
        assert!((n2.local_ai[1] - (1.0 - 2.0 / 30.0)).abs() < 1e-6);
        assert!(n2.hp > 0);
    }

    /// The rotation (`:25956-25982`): the scaled-delta Atan2 + the
    /// literal 3.14f; a dead-centre aim (equal half-x) zeroes it.
    #[test]
    fn the_rotation_aims_with_the_literal_314() {
        let w = underworld();
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.vx = 1.0;
        // Player right of the wall's integer half (1000+60=1060).
        let players = [cand(1200.0, 220.0 * 16.0 - 100.0)];
        let mut h = Harness::new(1);
        h.step(&mut n, &w, &players);
        let dx = 1200.0 + 10.0 - (n.x + 60.0);
        let dy = (220.0 * 16.0 - 100.0) + 21.0 - (n.y + 60.0);
        let d = (dx * dx + dy * dy).sqrt();
        let want = (-(dy * d)).atan2(-(dx * d)) + 3.14;
        assert!((n.rotation - want).abs() < 1e-4);
        // Dead centre: player half-x == wall half-x → rotation 0.
        let mut n2 = wall(1000.0, 220.0 * 16.0);
        n2.local_ai[0] = 2.0;
        n2.vx = 1.0;
        let centred = [cand(1050.0, 220.0 * 16.0)]; // 1050+10 = 1060
        let mut h2 = Harness::new(1);
        h2.step(&mut n2, &w, &centred);
        assert_eq!(n2.rotation, 0.0);
    }

    /// The expert respawn cadence (`:25985-25999`): at full hp
    /// num365 = 11² = 121 → (121*19+400)/20 = 134 → Next(93); the
    /// gate draw happens EVERY expert tick after the roar draw.
    #[test]
    fn the_expert_respawn_cadence_draws_every_tick() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.vx = 1.0;
        let mut h = Harness::new(1);
        let before = h.rng.clone();
        let mut ctx = Style27Context {
            wof_draw_top: &mut h.top,
            wof_draw_bottom: &mut h.bottom,
            wof_npc_index: &mut h.index,
            expert: true,
            get_good_world: false,
            not_client: true,
            leech_head_count: 0,
            demon_count: 0,
            hungry_ai0s: &[],
            players: &players,
            player_gross: GROSS,
            ticker: 0,
            rng: &mut h.rng,
            spawns: &mut h.spawns,
        };
        style27_step(&mut n, &w, &mut ctx);
        assert_ne!(h.rng, before);
        // Replay the draw order on the clone: tick 1's roar threshold
        // (localAI[3] = 1 < 600 + anything → no re-arm), then the
        // full-hp cadence Next(93); a cadence hit of 0 draws Next(1)
        // (0 live Hungries → maxValue 1, always <= 1) then exactly one
        // Next(10) (the first pick is always free with 0 taken slots).
        let mut probe = before.clone();
        let _roar = probe.next_max(1000);
        let cadence = probe.next_max(93);
        assert!((0..93).contains(&cadence));
        if cadence == 0 {
            let second = probe.next_max(1);
            assert!(second <= 1);
            let pick = probe.next_max(10);
            assert_eq!(h.spawns.len(), 1);
            assert_eq!(h.spawns[0].npc_type, 115);
            assert_eq!(h.spawns[0].ai0, pick as f32 * 0.1 - 0.05);
        } else {
            assert!(h.spawns.is_empty());
        }
        assert_eq!(probe, h.rng, "the stream advanced in the cited order");
    }

    /// The respawn slot pick (`:26012-26037`): with 10 live Hungries
    /// the second draw never happens and no respawn spawns, even
    /// across many cadence hits.
    #[test]
    fn the_respawn_pick_respects_the_ten_cap() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let ten: Vec<f32> = (0..10).map(|i| i as f32 * 0.1 - 0.05).collect();
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.vx = 1.0;
        n.hp = 1; // cadence 30 → Next(21): ~19 gate hits over 400 ticks
        let mut h = Harness::new(1);
        for _ in 0..400 {
            let mut ctx = Style27Context {
                wof_draw_top: &mut h.top,
                wof_draw_bottom: &mut h.bottom,
                wof_npc_index: &mut h.index,
                expert: true,
                get_good_world: false,
                not_client: true,
                leech_head_count: 0,
                demon_count: 0,
                hungry_ai0s: &ten,
                players: &players,
                player_gross: GROSS,
                ticker: 0,
                rng: &mut h.rng,
                spawns: &mut h.spawns,
            };
            style27_step(&mut n, &w, &mut ctx);
        }
        assert!(
            h.spawns.iter().all(|s| s.npc_type != 115),
            "10 live Hungries must block every respawn"
        );
    }

    /// Stream-clone witness (WRITTEN NOT RUN in code-only mode): two
    /// identical inputs + cloned streams produce identical NPC state,
    /// identical band statics and identical spawn lists over 300 ticks.
    #[test]
    fn the_arm_is_deterministic_under_a_cloned_stream() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut a = wall(1000.0, 220.0 * 16.0);
        let mut b = a.clone();
        let mut ha = Harness::new(7);
        let mut hb = Harness {
            top: ha.top,
            bottom: ha.bottom,
            index: ha.index,
            rng: ha.rng.clone(),
            spawns: Vec::new(),
        };
        for _ in 0..300 {
            ha.step(&mut a, &w, &players);
            hb.step(&mut b, &w, &players);
        }
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.ai, b.ai);
        assert_eq!(a.local_ai, b.local_ai);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(ha.top, hb.top);
        assert_eq!(ha.bottom, hb.bottom);
        assert_eq!(ha.spawns, hb.spawns);
        assert_eq!(ha.rng, hb.rng);
    }

    /// S1-D-3 W-1d — the roar cue (`:25720`): when the cadence fires,
    /// the arm queues `PlaySound(4, (int)position.X, (int)position.Y,
    /// 10)` on `wants_sound` — the int killed overload, style 10. The
    /// two cadence draws still ride the stream in the cited order.
    #[test]
    fn the_roar_queues_the_style_10_death_cue() {
        let w = underworld();
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.local_ai[3] = 1599.0; // 1600 >= 600 + any Next(1000)
        n.vx = 1.0;
        let mut h = Harness::new(1);
        let before = h.rng.clone();
        h.step(&mut n, &w, &players);
        assert_eq!(n.wants_sound.len(), 1, "the roar fired");
        let req = n.wants_sound[0];
        assert!(matches!(req.cue, crate::npc::NpcSoundCue::NpcDeath(10)));
        assert_eq!(req.x, n.x);
        assert_eq!(req.y, n.y);
        // The stream order: Next(1000) then the re-arm Next(200).
        let mut probe = before.clone();
        let _threshold = probe.next_max(1000);
        let rearm = probe.next_max(200);
        assert_eq!(n.local_ai[3], -(rearm as f32));
        assert_eq!(probe, h.rng, "exactly the two cited draws");
        // A non-firing tick queues nothing.
        let mut n2 = wall(1000.0, 220.0 * 16.0);
        n2.local_ai[0] = 2.0;
        n2.vx = 1.0;
        let mut h2 = Harness::new(1);
        h2.step(&mut n2, &w, &players);
        assert!(n2.wants_sound.is_empty(), "localAI[3] = 1 < 600");
    }

    /// S1-D-3 W-1d — the abandonment cry (`:25942`): the kill tick
    /// queues the same style-10 cue BEFORE `life = 0` (the C#'s order)
    /// and returns; the corpse's queue drains at the slot's tail.
    #[test]
    fn the_abandonment_kill_queues_the_cue_before_dying() {
        let w = underworld();
        let mut dead = cand(1200.0, 220.0 * 16.0);
        dead.dead = true;
        let players = [dead];
        let mut n = wall(1000.0, 220.0 * 16.0);
        n.local_ai[0] = 2.0;
        n.local_ai[1] = 1.0 - 1.0 / 360.0;
        n.vx = 1.0;
        let mut h = Harness::new(1);
        h.step(&mut n, &w, &players);
        assert_eq!(n.hp, 0, "the abandonment kill");
        assert_eq!(n.wants_sound.len(), 1);
        assert!(matches!(
            n.wants_sound[0].cue,
            crate::npc::NpcSoundCue::NpcDeath(10)
        ));
        assert_eq!(n.wants_sound[0].x, n.x);
        assert_eq!(n.wants_sound[0].y, n.y);
    }

    /// The liquid term counts in the BOTTOM walk too (`:25746`): two
    /// liquid rows (8 columns each = 16 >= 15) stop the walk exactly
    /// like two solid rows — the live arm's solids-only bottom walk
    /// would run to the world floor instead.
    #[test]
    fn the_bottom_walk_counts_liquid() {
        let mut w = underworld();
        // The wall at x=1000 spans columns 62..=70; the shelf covers
        // 60..70 → 8 liquid columns per row. The walk starts testing
        // at num350+8 = 231, so the shelf sits at rows 235/236.
        for x in 60..70 {
            for y in [235, 236] {
                if let Some(t) = w.tile_mut(x, y) {
                    t.liquid = 255;
                }
            }
        }
        let mut n = wall(1000.0, 220.0 * 16.0);
        let players = [cand(1200.0, 220.0 * 16.0)];
        let mut h = Harness::new(1);
        h.step(&mut n, &w, &players);
        // 8 counts at row 235, 8 more at 236 → 16 >= 15 → the walk
        // stops at 236, +4 → 240*16 = 3840 (inside the band clamp).
        assert_eq!(h.bottom, 3840);
        // The solid-shelf twin must produce the identical band.
        let mut w2 = underworld();
        for x in 60..70 {
            for y in [235, 236] {
                if let Some(t) = w2.tile_mut(x, y) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 1; // stone — TILE_SOLID
                }
            }
        }
        let mut n2 = wall(1000.0, 220.0 * 16.0);
        let mut h2 = Harness::new(1);
        h2.step(&mut n2, &w2, &players);
        assert_eq!(h.bottom, h2.bottom, "liquid must count like solid");
        assert_eq!(h.top, h2.top);
    }
}
