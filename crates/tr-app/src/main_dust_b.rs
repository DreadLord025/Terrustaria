//! W4-g12 — the Main.cs `Dust.NewDust*` sites, batch 2 of 3 (sites 9-16 of
//! the research/64 census; g11 owns 1-8, g13 owns 17-23 — the line-order
//! thirds split in `docs/research/64-dust-main-sites-claims.md`).
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 13390 — `TeleportEffect` case 10: the 267 sparkle lane
//!   (`Main.cs:13381-13418`, `NewDustDirect` + `CloneDust`).
//! - 13438 — `TeleportEffect` case 11: the 50× `NewDustPerfect` 267 burst
//!   (`Main.cs:13419-13446`).
//! - 13452 — `TeleportEffect` case 13: the 21× type-27 smoke
//!   (`Main.cs:13447-13462`).
//! - 22744 — `DrawNPCDirect`'s `type == 36` bone-arm chain, the j == 1
//!   hand-tip puff (type 5, `Main.cs:22701-22748`).
//! - 22820 — `DrawNPCDirect`'s `aiStyle >= 33 && <= 36` bone-arm chain,
//!   the k == 1 hand-tip puff (type 6, `Main.cs:22786-22824`).
//! - 24736 / 24748 / 24752 — `DrawNPCDirect`'s `type == 398` Moon Lord
//!   core arm, three of the four `flag8`-gated type-6 puffs
//!   (`Main.cs:24701-24764`; the fourth, `:24756`, is g13's site 17).
//!
//! ## Staged, not wired (the e2-batch-1 / d50 lane pattern)
//!
//! None of the eight sites has a live ported machine yet:
//! * `Main.TeleportEffect` (`Main.cs:13233-13464`) is unported — the
//!   recall doc's "separate follow-up" (`recall_teleport.rs` header,
//!   ANALYSIS "The recall / mirror teleport (S1-P-i38)"). The port's
//!   `Player::teleport_style` (`player.rs`) is the movement-relevant core
//!   only; its live callers are the Queen Slime hook
//!   (`projectile.rs`'s 935 arm → `p.teleport`, `Projectile.cs:52775` →
//!   `DoQueenSlimeHookTeleport` → `Teleport(vector, 10)`,
//!   `Player.cs:22742-22755`) and the wiring teleporter pad
//!   (`world_sim/wiring.rs`, `Wiring.cs:3154` — style 0, g11's cases).
//!   The wiring wave ports `Player.Teleport`/`NPC.Teleport`
//!   (`Player.cs:37414-37496`, `NPC.cs:81035-81058`) and calls these
//!   case arms at the Entry/Exit `TeleportEffect` sites with
//!   `dustCountMult = MathHelper.Clamp(1f - teleportTime * 0.99f,
//!   0.01f, 1f)` (`Player.cs:37442`). Cases 10/11/13 read neither `side`
//!   nor `extraInfo`; case 10 reads `otherPosition` (the Exit leg's
//!   pre-teleport position, `Player.cs:37443`/`:37489`).
//! * The two bone-arm chains' SPRITE draws (`TextureAssets.BoneArm`/
//!   `BoneArm2`/`BoneArm3`, the `localAI[3]` texture pick) are ported as
//!   the staged geometry plan `bone_arm_sprites` (W4-g12 render-sweep-2:
//!   per-leg anchor + `atan2 - 1.57` rotation over the SAME
//!   `bone_arm_leg` walk the dust rides). `render/entities.rs` draws the
//!   hand/limb sheets only; the wiring wave resolves the head snapshot
//!   (`Main.npc[(int)ai[1]]` — the limb spawn's ctx-seam stand-in,
//!   `npc_ai_prime_limbs.rs`) + the sheet pick and pushes the quads at
//!   the j/k == 1 leg, beside the dust call below.
//! * The Moon Lord core arm is parked (`render/entities.rs::
//!   ml_extra_parts`, ANALYSIS M4-NPC-3v). **Dead-gate census finding:**
//!   `flag8` (`Main.cs:24703`) is initialised `false` and NEVER assigned
//!   `true` anywhere in the type-398 block (the only `flag8` reads are
//!   the four `if (flag8)` dust gates `:24734/:24746/:24750/:24754`) —
//!   all four puffs are dead code in 1.4.5.8. The port keeps the gate
//!   verbatim (the caller passes `false`); the witness pins zero spawns
//!   AND zero draws on the dead path.
//!
//! ## RNG contract (research/64: one slot per C# pass, S1-A-10)
//!
//! * The `TeleportEffect` arms take the caller's `UnifiedRandom`: the
//!   C# executes them inside `Player.Teleport`/`NPC.Teleport`, i.e.
//!   under the `Main.SwapRandom("UpdatePlayers")` (`Main.cs:17754`) or
//!   `("UpdateNPCs")` (`:17767`) block of whichever pass teleports —
//!   never a new stream. Argument draws run BEFORE the pool internals
//!   (C# evaluates arguments first; case 10's named arguments evaluate
//!   in written order — `newColor`'s hue float first), the post-spawn
//!   writes after (the sentinel slot takes them, `Dust.cs:79`).
//! * The two bone-arm sites are DRAW-path (`DrawNPCs`, `Main.cs:63390`/
//!   `:63406` — outside every `SwapRandom`). They follow the port's
//!   draw-dust precedent (ANALYSIS "Mushroom glow RNG" / the
//!   `leaf_rng`/`TrySparkling` frame-pinned streams, `frame/draw.rs`):
//!   the wiring wave feeds a frame-derived stream, NEVER a simulation
//!   stream — a draw-time draw on a stage stream would desync the
//!   fixed-step order. The emitters are generic over `DustRng` so the
//!   draw thread can pass its own adapter. The C# gate
//!   `FocusHelper.AllowMiscDustEffects` (`FocusHelper.cs:39` =
//!   `!WorldUpdateStepper.DrawnThisUpdate`) is the `allow_misc_dust`
//!   parameter — the port tracks the flag 1:1 (`stepper.
//!   drawn_this_update`, `frame/world.rs:90/:681`).
//! * The Moon Lord puffs are draw-path too (same precedent); the dead
//!   gate means no stream ever moves on the live path.
//!
//! Sounds: case 10 plays `SoundID.Item8` at `effectRect.Center` AFTER
//! the loop (`:13416`), case 13 BEFORE it (`:13449`); both are the
//! style façade + case-2 pitch draw (`play_npc_item_cue`, the d52
//! precedent — the façade's `GetRandomPitch` NextFloat rides the
//! audio-only stream, the case-2 `Next(-6,7)` the caller's). Case 11
//! plays nothing. WRITTEN NOT RUN (code-only mode): the witnesses below
//! are compiled by the merge-verify wave, not executed here.
#![allow(dead_code)] // STAGED (the e2-batch-1/d50 pattern): the wiring wave ports TeleportEffect's callers + the bone-arm draws; the witnesses stay as the audit cross-check

use crate::audio::Audio;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx, DUST_SENTINEL};
use tr_core::unified_random::UnifiedRandom;
use tr_core::whip::Rect;
use tr_core::World;

/// Site 9 — `TeleportEffect` case 10 (`Main.cs:13381-13418`): the
/// Rod-of-Discord-style 267 sparkle lane. `rect` is the caller's
/// `getRect()` (the C# inflates it 15×15 here); `mult` is
/// `dustCountMult`; `other_position` is the `otherPosition` argument
/// (the Exit leg's pre-teleport position). Per dust the C# draws:
/// fadeIn `NextFloat`, scale `NextFloat` (the two locals, `:13388-13389`),
/// the hue `NextFloat` inside the `newColor` named argument (`:13390`,
/// written first → evaluated first), then the `NewDustDirect` internals
/// (type 267, alpha 127, zero speeds), then the post chain: the
/// `Next(20,70)` scale roll + the n<10/n<5 bonuses (all DEAD — the
/// `scale = scale` local write at `:13406` overwrites them, the draws
/// still ride the stream), the gated `vector * 0.1 * NextFloat`
/// velocity pull, `noGravity`/`noLight`, the local scale/fadeIn writes,
/// and the `CloneDust` (scale/fadeIn ×0.65, white) when the slot is not
/// the sentinel. The Item8 cue plays AFTER the loop (`:13416`).
pub fn teleport_effect_case10(
    rect: Rect,
    mult: f32,
    other_position: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    audio: &mut Audio,
) {
    let mut rect = rect;
    rect.inflate(15, 15); // `:13383`
    let num7 = (60.0f32 * mult) as i32; // `:13384`
    let top_left = Vec2::new(rect.x as f32, rect.y as f32);
    let vector = other_position - top_left; // `:13385`
    for n in 0..num7 {
        let fade_in = 0.4 + rng.next_float(); // `:13388`
        let scale = 0.4 + rng.next_float(); // `:13389`
        // `:13390` — the newColor argument's hue draw, then the pool.
        let hue = 0.66f32 + rng.next_float() * 0.24;
        let [r, g, b, a] = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.5, 255);
        let idx = dust.new_dust_direct(
            rng,
            ctx,
            world,
            top_left,
            rect.w,
            rect.h,
            267,
            0.0,
            0.0,
            127,
            Color::new(r, g, b, a),
            1.0,
        );
        // `:13391-13399` — the dead scale ladder (the draws stay).
        {
            let d = dust.slot_mut(idx);
            d.scale = rng.next_range(20, 70) as f32 * 0.01;
            if n < 10 {
                d.scale += 0.25;
            }
            if n < 5 {
                d.scale += 0.25;
            }
        }
        // `:13400-13402` — the gated velocity pull toward otherPosition.
        if (n as f32) < (num7 as f32) * 0.8 {
            let pull = rng.next_float();
            dust.slot_mut(idx).velocity += vector * 0.1 * pull;
        }
        // `:13404-13407` — the flags, then the local overwrites.
        {
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.no_light = true;
            d.scale = scale;
            d.fade_in = fade_in;
        }
        // `:13408-13414` — the white 0.65× clone (sentinel excluded).
        if idx != DUST_SENTINEL {
            let c = dust.clone_dust(rng, ctx, world, idx);
            let d = dust.slot_mut(c);
            d.scale *= 0.65;
            d.fade_in *= 0.65;
            d.color = Color::new(255, 255, 255, 255);
        }
    }
    // `:13416` — Item8 at the inflated rect's Centre (integer halves).
    let cx = (rect.x + rect.w / 2) as f32;
    let cy = (rect.y + rect.h / 2) as f32;
    audio.play_npc_item_cue(8, cx, cy, rng);
}

/// Site 10 — `TeleportEffect` case 11 (`Main.cs:13419-13446`): the
/// 50× type-267 `NewDustPerfect` burst (the Shellphone/no-space style).
/// Per dust: the `Next(4)` colour pick FIRST (`:13423`), then the
/// `NextVector2FromRectangle` position pair (two `NextFloat`s,
/// `Utils.cs:1256-1259`), then the Perfect internals (0×0 box), then
/// `noGravity`, the picked colour (0/1 → (100,255,100), 2 → Yellow,
/// 3 → White — the initial `Color.Green` is always overwritten),
/// `velocity *= 2`, `scale = 0.8 + NextFloat() * 0.6`, `fadeIn = 0.5`.
/// NO sound in this case.
pub fn teleport_effect_case11(
    rect: Rect,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..50 {
        let num4 = rng.next_max(4); // `:13423`
        let color = match num4 {
            0 | 1 => Color::new(100, 255, 100, 255), // `:13429`
            2 => Color::new(255, 255, 0, 255),       // `:13432`
            _ => Color::WHITE,                       // `:13435`
        };
        // `:13438` — NextVector2FromRectangle: X float then Y float.
        let px = rect.x as f32 + rng.next_float() * rect.w as f32;
        let py = rect.y as f32 + rng.next_float() * rect.h as f32;
        let idx = dust.new_dust_perfect(
            rng,
            ctx,
            world,
            Vec2::new(px, py),
            267,
            None,
            0,
            Color::default(),
            1.0,
        );
        // `:13439-13443` — the post chain, sentinel included.
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.color = color;
        d.velocity *= 2.0;
        d.scale = 0.8 + rng.next_float() * 0.6;
        d.fade_in = 0.5;
    }
}

/// Site 11 — `TeleportEffect` case 13 (`Main.cs:13447-13462`): the
/// Mystic-Frog-style 21× type-27 smoke. The Item8 cue plays FIRST
/// (`:13449`, at the UN-inflated rect's Centre), then per dust: the
/// `NextVector2FromRectangle` position pair, the `Next(1,3)` scale
/// ARGUMENT (`:13452`, evaluated after the position), the `NewDust`
/// internals (2×2 box → the pool's <5 clamp, alpha 100), then
/// `velocity *= 1.75`, the `i % 3 == 0` extra `*= 1.5`,
/// `noLightEmittance` and `noGravity`.
pub fn teleport_effect_case13(
    rect: Rect,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    audio: &mut Audio,
) {
    // `:13449` — the Vector2 façade truncates; the cue precedes the loop.
    let cx = (rect.x + rect.w / 2) as f32;
    let cy = (rect.y + rect.h / 2) as f32;
    audio.play_npc_item_cue(8, cx, cy, rng);
    for i in 0..21 {
        let px = rect.x as f32 + rng.next_float() * rect.w as f32;
        let py = rect.y as f32 + rng.next_float() * rect.h as f32;
        let scale_arg = rng.next_range(1, 3) as f32; // `:13452`
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(px, py),
            2,
            2,
            27,
            0.0,
            0.0,
            100,
            Color::default(),
            scale_arg,
        );
        // `:13453-13459` — the velocity ladder + flags, sentinel included.
        let d = dust.slot_mut(idx);
        d.velocity *= 1.75;
        if i % 3 == 0 {
            d.velocity *= 1.5;
        }
        d.no_light_emittance = true;
        d.no_gravity = true;
    }
}

/// One leg of the shared two-hop bone-arm walk
/// (`Main.cs:22706-22724` / `:22791-22809`, identical in both arms):
/// advances `v` by the normalised hop toward the head and returns
/// `(draw_pos, dx, dy, norm)`. `draw_pos` is the arm sprite's anchor
/// (the post-hop `vector5`/`vector7`); `dx`/`dy` are the PRE-normalisation
/// deltas (the puff's speeds are these × 0.02) and `norm` the reach
/// factor. The HEAD halves are INTEGER (`(float)(width / 2)`); the sqrt
/// runs in f64 over the f32 sum of squares, the divide in f32.
fn bone_arm_leg(
    v: &mut Vec2,
    j: usize,
    side: f32,
    head_x: f32,
    head_y: f32,
    head_width: i32,
    head_height: i32,
) -> (Vec2, f32, f32, f32) {
    let mut dx = head_x + (head_width / 2) as f32 - v.x;
    let mut dy = head_y + (head_height / 2) as f32 - v.y;
    // `:22708-22724`/`:22793-22809` — the per-leg pull + reach.
    let reach = if j == 0 {
        dx -= 200.0 * side;
        dy += 130.0;
        92.0f32
    } else {
        dx -= 50.0 * side;
        dy += 80.0;
        60.0f32
    };
    let norm = reach / ((dx * dx + dy * dy) as f64).sqrt() as f32;
    v.x += dx * norm;
    v.y += dy * norm;
    (*v, dx, dy, norm)
}

/// The bone-arm chain's SPRITE half (sites 12/13's render twin,
/// `Main.cs:22725-22734` / `:22810`): one rotated arm quad per leg at
/// the post-hop anchor, `rotation = atan2(dy, dx) - 1.57`, origin the
/// BoneArm frame centre, scale 1, no flip. STAGED like the dust half —
/// the wiring wave resolves the head snapshot (`Main.npc[(int)ai[1]]`)
/// and the sheet, then pushes these. Returns `(anchor, rotation)` per
/// leg; the caller maps the sheet + tint (`Lighting.GetColor` at the
/// anchor's tile, `:22726`/`:22810`) and the sheet pick — the type-36
/// arm reads `localAI[3] == 1` → BoneArm3, else BoneArm
/// (`:22727-22733`); the Prime-limb arm always draws BoneArm2
/// (`:22810`), both at BoneArm's frame size/origin. Draw-only, no RNG
/// — the geometry is the SAME walk the dust rides ([`bone_arm_leg`]),
/// so the two halves never diverge.
pub fn bone_arm_sprites(
    hand_x: f32,
    hand_y: f32,
    hand_width: f32,
    side: f32,
    head_x: f32,
    head_y: f32,
    head_width: i32,
    head_height: i32,
) -> [(Vec2, f32); 2] {
    let mut v = Vec2::new(hand_x + hand_width * 0.5 - 5.0 * side, hand_y + 20.0);
    let mut out = [(Vec2::ZERO, 0.0f32); 2];
    for j in 0..2 {
        let (pos, dx, dy, norm) =
            bone_arm_leg(&mut v, j, side, head_x, head_y, head_width, head_height);
        // `:22725`/`:22810` — the arm's rotation about its centre.
        out[j] = (pos, dy.atan2(dx) - 1.57);
        // `:22735-22738`/`:22811-22814` — the j == 0 midpoint advance
        // (the second leg's anchor starts half a hop further along).
        if j == 0 {
            v.x += dx * norm / 2.0;
            v.y += dy * norm / 2.0;
        }
    }
    out
}

/// Sites 12/13 — the shared bone-arm chain geometry
/// (`Main.cs:22701-22748` / `:22786-22824`, identical walks): the
/// two-hop tether from the hand's shoulder toward the head, then the
/// j/k == 1 tip puff. `side` is `ai[0]` (±1), the head snapshot is
/// `Main.npc[(int)ai[1]]` (the caller resolves the slot). The C#'s
/// `width / 2` / `height / 2` on the HEAD are INTEGER halves
/// (`(float)(Main.npc[..].width / 2)`); the hand's `width * 0.5f` is
/// float. The normaliser is `reach / (float)Math.Sqrt(dx*dx + dy*dy)`
/// — the sum of squares in f32, the sqrt in f64, the divide in f32.
/// The j == 0 leg advances the anchor by HALF the hop (the draw rides
/// the midpoint); the j == 1 leg advances by the FULL hop minus
/// (16, 6) and spawns — gated by `FocusHelper.AllowMiscDustEffects`.
/// The speeds are the PRE-normalisation deltas × 0.02. The sprite
/// draws are ported as the staged geometry plan [`bone_arm_sprites`]
/// (W4-g12 render-sweep-2) — the wiring wave pushes the quads.
fn bone_arm_chain<R: DustRng>(
    hand_x: f32,
    hand_y: f32,
    hand_width: f32,
    side: f32,
    head_x: f32,
    head_y: f32,
    head_width: i32,
    head_height: i32,
    allow_misc_dust: bool,
    dust_type: i32,
    scale: f32,
    rng: &mut R,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:22703`/`:22788` — the shoulder anchor.
    let mut v = Vec2::new(hand_x + hand_width * 0.5 - 5.0 * side, hand_y + 20.0);
    for j in 0..2 {
        let (_, dx, dy, norm) = bone_arm_leg(&mut v, j, side, head_x, head_y, head_width, head_height);
        // The arm sprite draw sits here ([`bone_arm_sprites`], no RNG).
        if j == 0 {
            // `:22735-22738`/`:22811-22814` — the midpoint advance.
            v.x += dx * norm / 2.0;
            v.y += dy * norm / 2.0;
        } else if allow_misc_dust {
            // `:22740-22745`/`:22816-22821` — the tip puff.
            v.x += dx * norm - 16.0;
            v.y += dy * norm - 6.0;
            let idx = dust.new_dust(
                rng,
                ctx,
                world,
                v,
                30,
                10,
                dust_type,
                dx * 0.02,
                dy * 0.02,
                0,
                Color::default(),
                scale,
            );
            dust.slot_mut(idx).no_gravity = true;
        }
    }
}

/// Site 12 — `DrawNPCDirect`'s `type == 36` arm (`Main.cs:22701-22748`):
/// the Skeletron hand's BoneArm chain, the j == 1 type-5 puff (30×10
/// box, scale 2). Draw-path: the wiring feeds the draw thread's stream
/// (the leaf/TrySparkling precedent), never a stage stream. Type 5 is
/// `DangerousDust` — the pool's ChildSafety arm rides verbatim.
pub fn skeletron_hand_bone_dust<R: DustRng>(
    hand_x: f32,
    hand_y: f32,
    hand_width: f32,
    side: f32,
    head_x: f32,
    head_y: f32,
    head_width: i32,
    head_height: i32,
    allow_misc_dust: bool,
    rng: &mut R,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    bone_arm_chain(
        hand_x, hand_y, hand_width, side, head_x, head_y, head_width, head_height,
        allow_misc_dust, 5, 2.0, rng, dust, ctx, world,
    );
}

/// Site 13 — `DrawNPCDirect`'s `aiStyle >= 33 && <= 36` arm
/// (`Main.cs:22786-22824`): the four Prime limbs' BoneArm2 chain, the
/// k == 1 type-6 puff (30×10 box, scale 2.5). Same geometry as site 12;
/// no `localAI[3]` texture branch (BoneArm2 always). The caller gates
/// on the limb's aiStyle (the port's `NpcKind::ai_style()`,
/// `npc/data.rs:2334-2340`); the type-36 hand rides aiStyle 12
/// (`npc/data.rs:2305`), so the two arms never double-spawn for one NPC.
pub fn prime_limb_bone_dust<R: DustRng>(
    hand_x: f32,
    hand_y: f32,
    hand_width: f32,
    side: f32,
    head_x: f32,
    head_y: f32,
    head_width: i32,
    head_height: i32,
    allow_misc_dust: bool,
    rng: &mut R,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    bone_arm_chain(
        hand_x, hand_y, hand_width, side, head_x, head_y, head_width, head_height,
        allow_misc_dust, 6, 2.5, rng, dust, ctx, world,
    );
}

/// Sites 14/15/16 — `DrawNPCDirect`'s `type == 398` Moon Lord core arm
/// (`Main.cs:24701-24764`), the first three of the four `flag8`-gated
/// type-6 puffs (the fourth, `:24756`, is g13's). The caller resolves
/// the arm geometry (the 397-hand scan + `vector32`/`vector33`, the
/// shared enclosing-arm context cited in both batches): `anchor` =
/// `vector32` (`center2 + (220,-60) * side`), `pull` = `vector33`
/// (`(hand.Center + (0,76) - vector32) * 0.5`), `center` = `center2`.
/// C# order: site 14 at `anchor + pull` BEFORE the arm sprite draw,
/// sites 15 (`anchor`) and 16 (`center`) AFTER it — no RNG between
/// them but the pool internals, so the dust-stream order is 14/15/16.
/// **`flag8` is dead** (`Main.cs:24703`, never set true): the live
/// caller passes `false` and nothing spawns or draws.
pub fn moon_lord_core_dust(
    flag8: bool,
    anchor: Vec2,
    pull: Vec2,
    center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !flag8 {
        return;
    }
    // `:24736` — before the arm draw.
    let idx = dust.new_dust(rng, ctx, world, anchor + pull, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0);
    dust.slot_mut(idx).no_gravity = true;
    // The arm sprite draw sits here (unported, no RNG).
    // `:24748` — after the arm draw.
    let idx = dust.new_dust(rng, ctx, world, anchor, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0);
    dust.slot_mut(idx).no_gravity = true;
    // `:24752`.
    let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0);
    dust.slot_mut(idx).no_gravity = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("g12".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget —
    /// the pool never culls (the e2 scratch convention).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// A headless Audio with the listener parked on the rect centre so
    /// the Item8 attenuation gate passes (the case-2 pitch draw rides).
    fn audio_at(x: f32, y: f32) -> Audio {
        let mut a = Audio::new(None);
        a.set_listener(crate::sound::Listener {
            camera_center: (x, y),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        a
    }

    /// The six `NewDust` default-block draws on a mirror stream
    /// (`Dust.cs:153-193`; the <5 clamp makes a 0/2-box roll `Next(1)`).
    fn mirror_internals(rng: &mut UnifiedRandom, w: i32, h: i32) {
        let _ = rng.next_max((w.max(5)) - 4);
        let _ = rng.next_max((h.max(5)) - 4);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
    }

    /// W4-g12 site 9: the case-10 draw sequence mirrors exactly — per
    /// dust the three argument floats, the internals, the dead
    /// `Next(20,70)`, the gated pull float (n < num7*0.8), and the
    /// clone's six internals; the final stream states agree and the
    /// records pin the local-overwrite + white-0.65×-clone shape.
    #[test]
    fn w4_g12_case10_mirrors_the_sparkle_lane_and_clone() {
        let w = world();
        let c = ctx();
        let rect = Rect::new(400, 400, 20, 42);
        let other = Vec2::new(900.0, 300.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut audio = audio_at(425.0, 436.0);
        teleport_effect_case10(rect, 1.0, other, &mut rng, &mut pool, &c, &w, &mut audio);
        // The mirror: 60 iterations + the trailing case-2 pitch draw.
        let mut mirror = UnifiedRandom::new(7);
        let mut locals = Vec::new();
        let num7 = 60;
        for n in 0..num7 {
            let fi = 0.4 + mirror.next_float();
            let sc = 0.4 + mirror.next_float();
            let _hue = mirror.next_float();
            mirror_internals(&mut mirror, 50, 72); // inflated 20+30 × 42+30
            let _dead = mirror.next_range(20, 70);
            if (n as f32) < (num7 as f32) * 0.8 {
                let _pull = mirror.next_float();
            }
            mirror_internals(&mut mirror, 0, 0); // CloneDust's NewDust
            locals.push((fi, sc));
        }
        let _pitch = mirror.next_range(-6, 7); // the Item8 case-2 draw
        assert_eq!(rng, mirror, "the case-10 draw sequence is exact");
        let act = active(&pool);
        assert_eq!(act.len(), 120, "60 originals + 60 clones");
        for (k, &(fi, sc)) in locals.iter().enumerate() {
            let o = pool.slot(act[2 * k]);
            let cl = pool.slot(act[2 * k + 1]);
            assert_eq!(o.dust_type, 267);
            assert_eq!(o.alpha, 127);
            assert!(o.no_gravity && o.no_light);
            assert_eq!(o.fade_in, fi, "the local fadeIn overwrites");
            assert_eq!(o.scale, sc, "the dead Next(20,70) ladder is overwritten");
            assert_eq!(cl.scale, sc * 0.65);
            assert_eq!(cl.fade_in, fi * 0.65);
            assert_eq!(cl.color, Color::new(255, 255, 255, 255));
            assert_eq!(cl.position, o.position, "CloneDust copies the position");
        }
    }

    /// W4-g12 site 9: the count truncates `(int)(60f * mult)` and the
    /// menu-guard refusal still burns the argument draws — the sentinel
    /// takes the post writes and NO clone fires (idx == 6000).
    #[test]
    fn w4_g12_case10_mult_truncates_and_refusal_burns_arguments() {
        let w = world();
        let mut c = ctx();
        c.game_menu = true;
        let rect = Rect::new(400, 400, 20, 42);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let mut audio = audio_at(425.0, 436.0);
        teleport_effect_case10(rect, 0.3, Vec2::ZERO, &mut rng, &mut pool, &c, &w, &mut audio);
        assert!(active(&pool).is_empty(), "the menu guard refuses every slot");
        // 18 iterations × (3 argument floats + Next(20,70) + the gated
        // pull float for n < 14.4) — no internals, no clone draws.
        let mut mirror = UnifiedRandom::new(9);
        for n in 0..18 {
            let _ = mirror.next_float();
            let _ = mirror.next_float();
            let _ = mirror.next_float();
            let _ = mirror.next_range(20, 70);
            if (n as f32) < 18.0 * 0.8 {
                let _ = mirror.next_float();
            }
        }
        let _pitch = mirror.next_range(-6, 7);
        assert_eq!(rng, mirror, "refused spawns still burn the argument draws");
        // The sentinel took the last local writes (the C#'s Main.dust[6000]).
        let s = pool.slot(DUST_SENTINEL);
        assert!(s.no_gravity && s.no_light);
    }

    /// W4-g12 site 10: the case-11 sequence — Next(4) colour pick, the
    /// position pair, the Perfect internals, the scale float — ×50, no
    /// sound; the colours/scale/fadeIn/velocity-doubling pin.
    #[test]
    fn w4_g12_case11_mirrors_the_perfect_burst() {
        let w = world();
        let c = ctx();
        let rect = Rect::new(300, 300, 40, 60);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        teleport_effect_case11(rect, &mut rng, &mut pool, &c, &w);
        let mut mirror = UnifiedRandom::new(13);
        let mut picks = Vec::new();
        for _ in 0..50 {
            let pick = mirror.next_max(4);
            let _px = mirror.next_float();
            let _py = mirror.next_float();
            mirror_internals(&mut mirror, 0, 0);
            let sc = 0.8 + mirror.next_float() * 0.6;
            picks.push((pick, sc));
        }
        assert_eq!(rng, mirror, "the case-11 draw sequence is exact");
        let act = active(&pool);
        assert_eq!(act.len(), 50);
        for (k, &(pick, sc)) in picks.iter().enumerate() {
            let d = pool.slot(act[k]);
            assert_eq!(d.dust_type, 267);
            assert!(d.no_gravity);
            assert_eq!(d.fade_in, 0.5);
            assert_eq!(d.scale, sc);
            let want = match pick {
                0 | 1 => Color::new(100, 255, 100, 255),
                2 => Color::new(255, 255, 0, 255),
                _ => Color::WHITE,
            };
            assert_eq!(d.color, want);
            assert!(d.position.x >= 300.0 && d.position.x < 340.0);
            assert!(d.position.y >= 300.0 && d.position.y < 360.0);
        }
    }

    /// W4-g12 site 11: the case-13 Item8 pitch draw precedes the 21
    /// smoke dusts; each dust draws the position pair + the Next(1,3)
    /// scale argument BEFORE the internals; the velocity ladder pins
    /// (×1.75, extra ×1.5 on i%3==0) and the two flags.
    #[test]
    fn w4_g12_case13_sound_first_then_the_smoke_ladder() {
        let w = world();
        let c = ctx();
        let rect = Rect::new(500, 500, 30, 50);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        let mut audio = audio_at(515.0, 525.0);
        teleport_effect_case13(rect, &mut rng, &mut pool, &c, &w, &mut audio);
        let mut mirror = UnifiedRandom::new(17);
        let _pitch = mirror.next_range(-6, 7); // the Item8 cue FIRST
        let mut scales = Vec::new();
        for _ in 0..21 {
            let _px = mirror.next_float();
            let _py = mirror.next_float();
            let sc = mirror.next_range(1, 3);
            mirror_internals(&mut mirror, 2, 2);
            scales.push(sc);
        }
        assert_eq!(rng, mirror, "the case-13 draw sequence is exact");
        let act = active(&pool);
        assert_eq!(act.len(), 21);
        for (i, &sc) in scales.iter().enumerate() {
            let d = pool.slot(act[i]);
            assert_eq!(d.dust_type, 27);
            assert_eq!(d.alpha, 100);
            assert!(d.no_gravity && d.no_light_emittance);
            // The rolled velocity jitter is in [-2,2]; the ladder is
            // ×1.75 (×1.5 more on i%3==0) — re-derive via a mirror pool.
            let mut mp = DustPool::new();
            let mut mr = UnifiedRandom::new(17);
            let _ = mr.next_range(-6, 7);
            for _ in 0..i {
                let _ = mr.next_float();
                let _ = mr.next_float();
                let _ = mr.next_range(1, 3);
                mirror_internals(&mut mr, 2, 2);
            }
            let px = 500.0 + mr.next_float() * 30.0;
            let py = 500.0 + mr.next_float() * 50.0;
            let sarg = mr.next_range(1, 3) as f32;
            let mi = mp.new_dust(&mut mr, &c, &w, Vec2::new(px, py), 2, 2, 27, 0.0, 0.0, 100, Color::default(), sarg);
            let mut want = mp.slot(mi).velocity * 1.75;
            if i % 3 == 0 {
                want *= 1.5;
            }
            assert_eq!(d.velocity, want);
            assert_eq!(d.scale, mp.slot(mi).scale, "the Next(1,3) scale argument");
            assert_eq!(sc as f32, sarg, "the two mirrors agree on the scale roll");
        }
    }

    /// W4-g12 site 12: the bone-arm chain's two-hop geometry pins the
    /// tip puff's position/speeds; the AllowMiscDustEffects gate blocks
    /// the spawn AND every draw when false.
    #[test]
    fn w4_g12_site12_bone_arm_geometry_and_gate() {
        let w = world();
        let c = ctx();
        // Hand at (600,500), 52 wide, side +1; head at (800,400), 60×60.
        let (hx, hy, hw, side) = (600.0f32, 500.0f32, 52.0f32, 1.0f32);
        let (tx, ty, tw, th) = (800.0f32, 400.0f32, 60i32, 60i32);
        // Re-derive the C# chain by hand (f64 sqrt, integer head halves).
        let mut v = Vec2::new(hx + hw * 0.5 - 5.0 * side, hy + 20.0);
        let mut tip = Vec2::ZERO;
        let (mut ddx, mut ddy) = (0.0f32, 0.0f32);
        for j in 0..2 {
            let mut dx = tx + (tw / 2) as f32 - v.x;
            let mut dy = ty + (th / 2) as f32 - v.y;
            let reach = if j == 0 { dx -= 200.0 * side; dy += 130.0; 92.0f32 } else { dx -= 50.0 * side; dy += 80.0; 60.0f32 };
            let norm = reach / ((dx * dx + dy * dy) as f64).sqrt() as f32;
            v.x += dx * norm;
            v.y += dy * norm;
            if j == 0 {
                v.x += dx * norm / 2.0;
                v.y += dy * norm / 2.0;
            } else {
                v.x += dx * norm - 16.0;
                v.y += dy * norm - 6.0;
                tip = v;
                ddx = dx;
                ddy = dy;
            }
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        skeletron_hand_bone_dust(hx, hy, hw, side, tx, ty, tw, th, true, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert_eq!(act.len(), 1, "only the j==1 leg spawns");
        // Mirror the single spawn's internals at the pinned position.
        let mut mp = DustPool::new();
        let mut mr = UnifiedRandom::new(23);
        let mi = mp.new_dust(&mut mr, &c, &w, tip, 30, 10, 5, ddx * 0.02, ddy * 0.02, 0, Color::default(), 2.0);
        let d = pool.slot(act[0]);
        let m = mp.slot(mi);
        // Type 5 is DangerousDust: the pool's ChildSafety arm may re-type
        // to 16 — both pools ride the same stream, so the records agree.
        assert_eq!(d.dust_type, m.dust_type);
        assert_eq!(d.position, m.position);
        assert_eq!(d.velocity, m.velocity);
        assert_eq!(d.scale, m.scale);
        assert!(d.no_gravity);
        assert_eq!(rng, mr, "one spawn's internals, nothing else");
        // The closed gate: zero spawns, zero draws.
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(23);
        skeletron_hand_bone_dust(hx, hy, hw, side, tx, ty, tw, th, false, &mut rng2, &mut pool2, &c, &w);
        assert!(active(&pool2).is_empty());
        assert_eq!(rng2, UnifiedRandom::new(23), "the gate blocks every draw");
    }

    /// W4-g12 site 13: the Prime-limb twin — type 6, scale 2.5, the same
    /// geometry; type 6 rides the pool's per-type tweak arm (the extra
    /// Next(-10,6) + the ×0.3/×0.7 folds) on the same stream.
    #[test]
    fn w4_g12_site13_prime_limb_type6_scale25() {
        let w = world();
        let c = ctx();
        let (hx, hy, hw, side) = (700.0f32, 450.0f32, 52.0f32, -1.0f32);
        let (tx, ty, tw, th) = (600.0f32, 350.0f32, 70i32, 80i32);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        prime_limb_bone_dust(hx, hy, hw, side, tx, ty, tw, th, true, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert_eq!(act.len(), 1);
        let d = pool.slot(act[0]);
        assert_eq!(d.dust_type, 6, "type 6 is not DangerousDust");
        assert!(d.no_gravity);
        // The tweak arm's scale fold: rolled scale × 2.5 × 0.7.
        assert!(d.scale > 0.0 && d.scale <= 1.21 * 2.5 * 0.7 + 1e-4);
        // Mirror: re-derive the tip, then the internals + the type-6 arm.
        let mut v = Vec2::new(hx + hw * 0.5 - 5.0 * side, hy + 20.0);
        let mut tip = Vec2::ZERO;
        let (mut ddx, mut ddy) = (0.0f32, 0.0f32);
        for j in 0..2 {
            let mut dx = tx + (tw / 2) as f32 - v.x;
            let mut dy = ty + (th / 2) as f32 - v.y;
            let reach = if j == 0 { dx -= 200.0 * side; dy += 130.0; 92.0f32 } else { dx -= 50.0 * side; dy += 80.0; 60.0f32 };
            let norm = reach / ((dx * dx + dy * dy) as f64).sqrt() as f32;
            v.x += dx * norm;
            v.y += dy * norm;
            if j == 0 { v.x += dx * norm / 2.0; v.y += dy * norm / 2.0; } else {
                v.x += dx * norm - 16.0; v.y += dy * norm - 6.0; tip = v; ddx = dx; ddy = dy;
            }
        }
        let mut mp = DustPool::new();
        let mut mr = UnifiedRandom::new(29);
        let mi = mp.new_dust(&mut mr, &c, &w, tip, 30, 10, 6, ddx * 0.02, ddy * 0.02, 0, Color::default(), 2.5);
        assert_eq!(d.position, mp.slot(mi).position);
        assert_eq!(d.velocity, mp.slot(mi).velocity);
        assert_eq!(d.scale, mp.slot(mi).scale);
        assert_eq!(rng, mr);
    }

    /// W4-g12 sites 14/15/16: the dead flag8 gate spawns nothing and
    /// draws nothing; the open gate (never live in 1.4.5.8) spawns the
    /// three type-6 noGravity puffs at the cited positions in order.
    #[test]
    fn w4_g12_ml_core_dead_gate_and_open_shape() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(1000.0, 600.0);
        let pull = Vec2::new(40.0, -30.0);
        let center = Vec2::new(900.0, 650.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        moon_lord_core_dust(false, anchor, pull, center, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty(), "flag8 is dead — no spawns");
        assert_eq!(rng, UnifiedRandom::new(31), "flag8 is dead — no draws");
        moon_lord_core_dust(true, anchor, pull, center, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert_eq!(act.len(), 3);
        for &i in &act {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 6);
            assert!(d.no_gravity);
        }
        // The 0×0 box still rolls the default block: position = arg +
        // Next(1) + 4 (`Dust.cs:170-171`) — mirror each spawn exactly.
        let mut mp = DustPool::new();
        let mut mr = UnifiedRandom::new(31);
        for (k, pos) in [anchor + pull, anchor, center].iter().enumerate() {
            let mi = mp.new_dust(&mut mr, &c, &w, *pos, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0);
            assert_eq!(pool.slot(act[k]).position, mp.slot(mi).position);
            assert_eq!(pool.slot(act[k]).velocity, mp.slot(mi).velocity);
            assert_eq!(pool.slot(act[k]).scale, mp.slot(mi).scale);
        }
        assert_eq!(rng, mr, "three spawns' internals, nothing else");
    }

    /// W4-g12 render-sweep-2: the bone-arm SPRITE half's anchors and
    /// rotations match the C# walk (`Main.cs:22725-22738`), and the
    /// second leg's anchor is the dust tip's anchor BEFORE the (16,6)
    /// nudge — the two halves ride the same `bone_arm_leg` geometry.
    #[test]
    fn w4_g12_sweep2_bone_arm_sprite_anchors_and_rotations() {
        let (hx, hy, hw, side) = (600.0f32, 500.0f32, 52.0f32, 1.0f32);
        let (tx, ty, tw, th) = (800.0f32, 400.0f32, 60i32, 60i32);
        let sprites = bone_arm_sprites(hx, hy, hw, side, tx, ty, tw, th);
        // Hand-re-derive the C# chain (f64 sqrt, integer head halves).
        let mut v = Vec2::new(hx + hw * 0.5 - 5.0 * side, hy + 20.0);
        let mut want = [(Vec2::ZERO, 0.0f32); 2];
        for j in 0..2 {
            let mut dx = tx + (tw / 2) as f32 - v.x;
            let mut dy = ty + (th / 2) as f32 - v.y;
            let reach = if j == 0 { dx -= 200.0 * side; dy += 130.0; 92.0f32 } else { dx -= 50.0 * side; dy += 80.0; 60.0f32 };
            let norm = reach / ((dx * dx + dy * dy) as f64).sqrt() as f32;
            v.x += dx * norm;
            v.y += dy * norm;
            want[j] = (v, dy.atan2(dx) - 1.57);
            if j == 0 {
                v.x += dx * norm / 2.0;
                v.y += dy * norm / 2.0;
            }
        }
        for j in 0..2 {
            assert_eq!(sprites[j].0, want[j].0, "leg {j} anchor");
            assert_eq!(sprites[j].1, want[j].1, "leg {j} rotation");
        }
        // The second sprite anchor + the (16,6) nudge IS the dust tip
        // position the site-12 witness pins (the shared walk).
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        skeletron_hand_bone_dust(hx, hy, hw, side, tx, ty, tw, th, true, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert_eq!(act.len(), 1);
        let tip_base = sprites[1].0 - Vec2::new(16.0, 6.0);
        // The pool's NewDust jitters the position by Next(30-4)+4 /
        // Next(10-4)+4 (`Dust.cs:170-171`) — the tip is within the box.
        let p = pool.slot(act[0]).position;
        assert!(p.x >= tip_base.x && p.x <= tip_base.x + 30.0, "tip x in the 30-box");
        assert!(p.y >= tip_base.y && p.y <= tip_base.y + 10.0, "tip y in the 10-box");
    }
}
