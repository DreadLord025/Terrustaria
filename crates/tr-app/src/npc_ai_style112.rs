//! The aiStyle-112 family — the Fairy Critters
//! (`NPC.AI_112_FairyCritter`, `NPC.cs:48088-48650`, dispatched at
//! `:41068-41071`). W4-d33, research/25 row 112, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has THREE vanilla members: the FairyCritterPink 583 /
//! Green 584 / Blue 585 (`ID/NPCID.cs:11843-11847`; the shared
//! `SetDefaults` row `NPC.cs:16583-16607`). The census of
//! `this.aiStyle = 112;` returns exactly that one row (`:16587`) — no
//! projectile-space user. WIRED by W4-d33 (the 583-585 kinds + the
//! axis' 112 arm): `Npc::ai_112_fairy` feeds the session player + the
//! sim-stamped `fairy_neighbors` pool + the `time_for_visuals` parity
//! stamp; the vanilla `npc_type` rides `cs_id()` (the AXIS dispatch —
//! not a kind match).
//!
//! ## The machine
//!
//! The treasure-hunting critter: a state machine on `ai[2]` —
//! 0 the spawn hop, 1 the fly-to-player, 2 the follow, 3 the treasure
//! circle, 4 the fly-to-treasure, 5 the sparkle-despawn, 6 the idle
//! hover, 7 the leave. The 18000-tick timeout (`:48092-48109`,
//! non-client, `ai[2] > 1`) flips to 7 facing AWAY from the player
//! (the `playerCX < centerX → direction 1` write is verbatim — it
//! faces away, not toward).
//!
//! State 0 (`:48112-48152`): lavaImmune/noTileCollide false; the
//! ai[0]/ai[1] centre latch; the `localAI[0] == 0` launch —
//! `(Lerp(2,4,NextFloat) * (Next(2)*2-1), Lerp(1,2,NextFloat) *
//! (Next(2)*2-1)) * 0.7` (X first: NextFloat, Next(2), NextFloat,
//! Next(2)); the >20-px anchor pull ±0.04 with the |vy|>2 ×0.95 damp;
//! TargetClosest; the alive player within 250 px flips to 1 facing
//! away, the vx sign-flip to `direction*2`, ai[3]=0.
//!
//! State 1 (`:48153-48236`): the collideX facing flip + vx re-arm, the
//! collideY `oldVelocity.Y > 0 ? 1 : -1` snap; the 4.5 cruise (the
//! Sign-mismatch OR under-speed gate, the 0.04 accel, the against-sign
//! 0.4/0.2 double-accel, the over-speed clamp); the 21-column × 8-row
//! ground scan (the direction<0 col -= 20 shift, the <row+5 near
//! flag), the 0.05 fall / -0.2 lift / -0.3 near-lift, caps 3/-5.
//!
//! State 2 (`:48237-48322`): noTileCollide true; the dead player
//! drops to 1; the non-client `CenteredRectangle(target.Center,
//! (w+60, h/2))` hitbox intersect runs the treasure search — found →
//! state 3 at the coords, else state 6; the closest-point chase
//! (DirectionTo ×2, the >150 ×2 / >80 ×1.5 bands, the 0.07 Lerp); the
//! ai[3]<300 bird recommendation (6 down, 3 up: +0.05 / -0.02, caps
//! 2/-4); the InWorld-gated SolidTile stuck counter (+2 to 400 / -1
//! to 0).
//!
//! State 3 (`:48323-48369`): the ai[3]==15 chime (`PlaySound(27)`),
//! the ≤15 ×0.9 damp, then the treasure circle — the phase 0 / +π/8
//! (≤65) / -π/8 (≤130) / 0, the height 22/14/18/22, the
//! `GetFairyCircleOffset(t+1/65) - GetFairyCircleOffset(t)` velocity;
//! ai[3] ≥ 210 → state 4 + TargetClosest.
//!
//! State 6 (`:48370-48412`): the >100-px drop to 2; the
//! !SolidCollision noTileCollide=false + collide flips; the >20-px
//! ±0.04 pull + |vy|>2 damp; the non-client treasure search → 3.
//!
//! State 4 (`:48413-48532`): the dead-player drop to 1; the 5×5
//! treasure-rect intersect → 5; the >300-px leash (the <400
//! !SolidCollision collide window, the DirectionFrom ×-0.1 push past
//! 360 with the `Next(30)==0` chime, the <330 closest-point ×0.1 pull,
//! the speed-1 normalize); the closest-point chase (×3 / ×2 bands,
//! the 0.07 Lerp, the ai[3]<300 bird recommendation (4,2) ±0.05 caps
//! ±1); the UNGATED SolidTile stuck counter (the C# catch returns
//! false out-of-bounds — the port's bounds check).
//!
//! State 5 (`:48533-48588`): localAI[1]=0; the dead `flag2` ore-tile
//! validity read (`:48537-48542` — computed, never read, documented);
//! the ai[3]==15 chime + ≤15 damp; the cos-modulated circle (the
//! `(int)(t/50)` step, `cos(step)*2π/16` phase, `cos(step*2)*10+8`
//! height, the 0.02 offset step); ai[3] > 200 non-client →
//! `active = false` + the single-player [`fairy_effects`] burst (the
//! server's life=0 + SendData 23/112 ride the net seam, documented).
//!
//! State 7 (`:48589-48596`): the 0.05×direction X accel clamped ±10,
//! the -0.025 Y drift clamped ±5, EncourageDespawn(10).
//!
//! The tail (`:48598-48649`): `dontTakeDamage =
//! dontTakeDamageFromHostiles = ai[2] > 1`; the same-style Manhattan
//! `width*1.5` separation (±0.05 Y only); the !flag velocity facing
//! (`spriteDirection = -direction`); the `(int)timeForVisualEffects %
//! 2 == 0` glow dust (type 278, the per-type HotPink/LightPink,
//! LimeGreen/LightSeaGreen, RoyalBlue/LightBlue `Color.Lerp` on ONE
//! NextFloat, alpha 200 scale 0.65, velocity zeroed then
//! `+= velocity*0.3`, noGravity noLight, under the netOffset shift);
//! the `AddLight(Center, color.ToVector3()*0.7)` contribution; the
//! non-server bestiary-nearby achievement check (the local player's
//! +300/+200 inflated hitbox, `HandleSpecialEvent(22)` — the outcome
//! seam).
//!
//! [`fairy_effects`] (`NPC.cs:48652-48683`): the 40-dust type-278
//! burst — each `Color.Lerp` on ONE NextFloat, velocity ×1.5 then the
//! i≥30 ×3.5 / i≥20 ×2 bands, `fadeIn = Next(0,17)*0.1`, noGravity;
//! the `SoundID.Item4` chime.
//!
//! ## RNG order (by construction)
//!
//! State 0's launch: NextFloat, Next(2), NextFloat, Next(2). State 4's
//! leash chime: Next(30). The glow dust: ONE NextFloat (the Lerp) then
//! the NewDust internals. [`fairy_effects`]: per dust the Lerp
//! NextFloat BEFORE the internals, then Next(0,17). The treasure
//! search, the bird recommendation, the circles and the separation
//! draw NOTHING. TargetClosest is RNG-free.
//!
//! ## FindFrame cases 583-585
//!
//! `NPC.cs:69835-69848`: `rotation = vx * 0.1`; `++frameCounter >= 4`
//! advances the row, wrapping at `Main.npcFrameCount[583..585] = 4`
//! (`Main.cs:67142`, the 580-589 row). [`fairy_find_frame`] ports the
//! counter pair (the style73 convention).
#![allow(dead_code)] // WIRED by W4-d33 (axis 112 → style112_step, the cs_id feed); the defaults row + witnesses stay as the audit cross-check

use crate::npc::Npc;
use crate::npc_ai_style37::rotated_by;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::scene::is_valid_for_ore_finder;
use tr_core::tiledata::{
    ORE_FINDER_PRIORITY, SET_FRIENDLY_FAIRY_CAN_LURE_TO, SET_ORE, TILE_SOLID, TILE_SOLID_TOP,
};
use tr_core::{UnifiedRandom, World};

/// The shared type-583/584/585 `SetDefaults` row
/// (`NPC.cs:16583-16607`), verbatim — only `catchItem` differs.
pub struct FairyCritterDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1`.
    pub death_sound: (i32, i32),
    /// The per-type catch item: 4068 / 4069 / 4070.
    pub catch_items: [i32; 3],
    pub no_gravity: bool,
    pub rarity: i32,
}

/// `NPC.cs:16583-16607` — `else if (this.type == 583 || 584 || 585)`.
pub const FAIRY_CRITTER: FairyCritterDefaults = FairyCritterDefaults {
    width: 18,
    height: 20,
    ai_style: 112,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    catch_items: [4068, 4069, 4070],
    no_gravity: true,
    rarity: 2,
};

/// `Main.npcFrameCount[583..=585]` = 4 (`Main.cs:67142`, the 580-589
/// row: `6, 9, 6, 4, 4, 4, 14, 4, 25, 13`).
pub const FAIRY_CRITTER_FRAME_COUNT: u32 = 4;

/// The per-type glow colours (`:48618-48630`): (primary, secondary).
/// XNA `Color.HotPink`/`LightPink`, `LimeGreen`/`LightSeaGreen`,
/// `RoyalBlue`/`LightBlue`.
pub const FAIRY_COLORS: [(Color, Color); 3] = [
    (
        Color::new(255, 105, 180, 255),
        Color::new(255, 182, 193, 255),
    ),
    (Color::new(50, 205, 50, 255), Color::new(32, 178, 170, 255)),
    (
        Color::new(65, 105, 225, 255),
        Color::new(173, 216, 230, 255),
    ),
];

/// One `Main.npc[k]` slot as the separation scan sees it (`:48601`).
#[derive(Clone, Copy, Debug)]
pub struct FairyNeighbor {
    /// `k != whoAmI` — the self skip, precomputed by the wiring.
    pub is_self: bool,
    pub active: bool,
    /// `Main.npc[k].aiStyle`.
    pub ai_style: i32,
    pub x: f32,
    pub y: f32,
}

/// The per-tick environment (the `Style111Env` shape).
pub struct Style112Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// The NPC slots in `Main.npc` order, self included (`is_self`).
    pub neighbors: &'a [FairyNeighbor],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.timeForVisualEffects` — the glow dust's parity gate
    /// (`:48631`, the `(int)` cast then `% 2`).
    pub time_for_visuals: f64,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
    /// `Main.LocalPlayer` for the bestiary check (`:48644-48648`):
    /// `(position.X, position.Y, width, height, dead)`. `None` when
    /// the port has no session player.
    pub local_player: Option<(f32, f32, i32, i32, bool)>,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style112Outcome {
    /// The state-5 expiry (`:48575`): `active = false` — the caller
    /// owes the pool removal. The single-player FairyEffects burst
    /// already ran INSIDE the step; the server's life=0 + SendData
    /// 23/112 ride the net seam (documented, unported).
    pub died: bool,
    /// The `SoundEngine.PlaySound` cues in C# order —
    /// `(type, style, x, y)`: the state-3/5/4 chimes `(27, 1)` at the
    /// truncated position, the FairyEffects `Item4 = (2, 4)`.
    pub sounds: Vec<(i32, i32, f32, f32)>,
    /// The last `noTileCollide` write (the per-state flips; the `Npc`
    /// slot has no field — the style24 `no_gravity` seam).
    pub no_tile_collide: Option<bool>,
    /// The last `lavaImmune` write (`:48091` true, the state-0/1
    /// false).
    pub lava_immune: Option<bool>,
    /// `dontTakeDamageFromHostiles` (`:48598`) — the `Npc` slot has
    /// no field; `dontTakeDamage` rides the existing `Npc` field.
    pub dont_take_damage_from_hostiles: bool,
    /// `Lighting.AddLight(Center, color.ToVector3() * 0.7)`
    /// (`:48641`) — `(x, y, [r, g, b])` for the caller's light feed.
    pub light: Option<(f32, f32, [f32; 3])>,
    /// `AchievementsHelper.HandleSpecialEvent(localPlayer, 22)`
    /// (`:48647`) — the bestiary-nearby request.
    pub bestiary_event: bool,
}

/// XNA `Vector2.Distance` — the f32 sum widened to f64 for the root.
fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32
}

/// XNA `Vector2.Length()` — the same root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// `Math.Sign(float)` — -1/0/+1.
fn math_sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): `MathHelper.Lerp` per
/// channel, then the ctor's truncating cast (the dust_table shape).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| ((x as f32 + (y as f32 - x as f32) * t) as i32) as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Utils.CenteredRectangle(Vector2, Vector2)` (`Utils.cs:947-950`):
/// the `(int)` truncation on all four fields.
fn centered_rectangle(center: Vec2, size: Vec2) -> (i32, i32, i32, i32) {
    (
        (center.x - size.x / 2.0) as i32,
        (center.y - size.y / 2.0) as i32,
        size.x as i32,
        size.y as i32,
    )
}

/// XNA `Rectangle.Intersects` — the strict `<`/`>` on both axes.
fn rect_intersects(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && a.0 + a.2 > b.0 && a.1 < b.1 + b.3 && a.1 + a.3 > b.1
}

/// `Utils.ClosestPointInRect` (`Utils.cs:1581-1601`): the per-axis
/// clamp to the rect's inclusive edges.
fn closest_point_in_rect(r: (i32, i32, i32, i32), p: Vec2) -> Vec2 {
    Vec2::new(
        p.x.clamp(r.0 as f32, (r.0 + r.2) as f32),
        p.y.clamp(r.1 as f32, (r.1 + r.3) as f32),
    )
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`): the
/// strict test; a null tile is SOLID, an out-of-bounds index throws
/// into the catch → FALSE (the port's bounds check).
fn solid_tile_ij(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && !t.inactive()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `NPC.GetBirdFlightRecommendation` (`NPC.cs:48685-48708`): the
/// direction-shifted column scan — solid-or-liquid within `up_range`
/// rows flags up; any hit clears down. Draw-free.
fn bird_flight_recommendation(
    world: &World,
    down_scan_range: i32,
    up_range: i32,
    mut tx: i32,
    ty: i32,
    direction: i32,
) -> (bool, bool) {
    tx += direction;
    let mut go_downwards = true;
    let mut go_upwards = false;
    let mut i = ty;
    while i < ty + down_scan_range && tx >= 0 && tx < world.width && i >= 0 && i < world.height {
        let Some(t) = world.tile(tx, i) else {
            break; // the null-tile break (`:48694-48697`)
        };
        if (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0 {
            if i < ty + up_range {
                go_upwards = true;
            }
            go_downwards = false;
            break;
        }
        i += 1;
    }
    (go_downwards, go_upwards)
}

/// `NPC.GetFairyCircleOffset` (`NPC.cs:48710-48713`):
/// `((2π·t + π/2).ToRotationVector2() + (0,-1)) * (6·-direction,
/// height)` then `RotatedBy(phase)`.
fn fairy_circle_offset(elapsed: f32, phase: f32, height: f32, direction: i32) -> Vec2 {
    let a = std::f32::consts::PI * 2.0 * elapsed + std::f32::consts::FRAC_PI_2;
    let v = Vec2::new(a.cos(), a.sin() - 1.0) * Vec2::new(6.0 * -direction as f32, height);
    let (x, y) = rotated_by(v.x, v.y, phase as f64, 0.0, 0.0);
    Vec2::new(x, y)
}

/// `NPC.GetFairyTreasureCoords` (`NPC.cs:48715-48772`): the ±75/±50
/// tile window (clamped 40 from the world edges), the
/// FriendlyFairyCanLureTo + IsValidForOreFinder filter, the ore
/// priority (the 7×7 same-type count < 40 demotes to -1), the
/// `priority <= best` + nearest-tile selection. Draw-free.
fn fairy_treasure_coords(npc: &Npc, world: &World) -> Option<(i32, i32)> {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let px = center.x as i32 >> 4;
    let py = center.y as i32 >> 4;
    // `value.Inflate(75, 50)` on the 1×1 rect, intersected with the
    // world rect inflated -40 (`:48719-48724`).
    let left = (px - 75).max(40);
    let top = (py - 50).max(40);
    let right = (px + 75).min(world.width - 40 - 1);
    let bottom = (py + 50).min(world.height - 40 - 1);
    let mut best_priority: i32 = -1;
    let mut best_dist = -1.0f32;
    let mut coords: Option<(i32, i32)> = None;
    for i in left..=right {
        for j in top..=bottom {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            if !t.active()
                || !SET_FRIENDLY_FAIRY_CAN_LURE_TO.has(t.kind)
                || !is_valid_for_ore_finder(t)
            {
                continue;
            }
            let mut priority = ORE_FINDER_PRIORITY[t.kind as usize] as i32;
            if SET_ORE.has(t.kind) {
                // `:48739-48756` — the 7×7 same-type count.
                let mut count = 0;
                for k in i - 3..=i + 3 {
                    for l in j - 3..=j + 3 {
                        if let Some(n) = world.tile(k, l) {
                            if n.active() && n.kind == t.kind {
                                count += 1;
                            }
                        }
                    }
                }
                if count < 40 {
                    priority = -1;
                }
            }
            if best_priority <= priority {
                let d = distance(
                    center,
                    Vec2::new(i as f32 * 16.0 + 8.0, j as f32 * 16.0 + 8.0),
                );
                if best_priority != priority || d < best_dist {
                    best_priority = priority;
                    best_dist = d;
                    coords = Some((i, j));
                }
            }
        }
    }
    coords
}

/// The target slot's candidate — the style16 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `NPC.GetTargetData` for the fairy (NOT in `UsesNewTargeting` —
/// player-only): the target player's centre/box, or the `default`
/// (0,0,0,0) + `None` type when invalid (`NPC.cs:6901-6912`).
fn fairy_target(npc: &Npc, env: &Style112Env<'_>) -> (Vec2, Vec2, bool) {
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
            return (
                Vec2::new(p.x, p.y),
                Vec2::new(p.width as f32, p.height as f32),
                true,
            );
        }
    }
    (Vec2::ZERO, Vec2::ZERO, false)
}

/// The shared state-0/2/4 dead-player drop to state 1
/// (`:48143-48149` / `:48249-48255` / `:48425-48431`).
fn drop_to_fly(npc: &mut Npc, target_cx: f32, center: Vec2) {
    npc.ai[2] = 1.0;
    npc.facing = if !(target_cx > center.x) { 1 } else { -1 };
    if npc.vx * (npc.facing as f32) < 0.0 {
        npc.vx = npc.facing as f32 * 2.0;
    }
    npc.ai[3] = 0.0;
    npc.net_update = true;
}

/// `NPC.FairyEffects` (`NPC.cs:48652-48683`): the 40-dust type-278
/// burst + the Item4 chime. `color_index` is `type - 583` (0/1/2).
pub fn fairy_effects(
    position: Vec2,
    color_index: usize,
    env: &mut Style112Env<'_>,
    out: &mut Style112Outcome,
) {
    let (c1, c2) = FAIRY_COLORS[color_index.min(2)];
    let num = 4.0f32;
    for i in 0..40 {
        // The Lerp NextFloat draws BEFORE the NewDust internals.
        let t = env.rng.next_float();
        let idx = env.dust.new_dust_direct(
            env.rng,
            env.dust_ctx,
            env.world,
            position - Vec2::splat(num * 0.5),
            (num + 4.0) as i32,
            (num + 4.0) as i32,
            278,
            0.0,
            0.0,
            200,
            lerp_color(c1, c2, t),
            0.65,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.velocity *= 1.5;
            if i >= 30 {
                d.velocity *= 3.5;
            } else if i >= 20 {
                d.velocity *= 2.0;
            }
            d.fade_in = env.rng.next_range(0, 17) as f32 * 0.1;
            d.no_gravity = true;
        }
    }
    // `SoundID.Item4 = (2, 4)` — the audio seam.
    out.sounds
        .push((2, 4, position.x as i32 as f32, position.y as i32 as f32));
}

/// One fixed step of the aiStyle-112 arm (`NPC.cs:48088-48650`), in C#
/// order. `npc_type` is the vanilla id (583/584/585) until the roster
/// kinds land. The caller owns the movement integration and drains
/// [`Style112Outcome`].
pub fn style112_step(npc: &mut Npc, npc_type: i32, env: &mut Style112Env<'_>) -> Style112Outcome {
    let mut out = Style112Outcome::default();
    let mut circle_facing = false; // flag — the circle states keep the facing
    out.lava_immune = Some(true); // `:48091`
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:48092-48109` — the 18000-tick timeout.
    if env.net_mode != 1 && npc.ai[2] > 1.0 {
        let timeout = 18000;
        npc.local_ai[1] += 1.0;
        if npc.local_ai[1] >= timeout as f32 {
            npc.ai[2] = 7.0;
            let p = target_of(npc, env.players);
            let pcx = p.x + (p.width / 2) as f32;
            // VERBATIM: the player LEFT of centre faces +1 (away).
            npc.facing = if pcx < center.x { 1 } else { -1 };
            npc.net_update = true;
        }
    }
    match npc.ai[2] as i32 {
        0 => {
            // `:48112-48152` — the spawn hop.
            out.lava_immune = Some(false);
            out.no_tile_collide = Some(false);
            if npc.ai[0] == 0.0 && npc.ai[1] == 0.0 {
                npc.ai[0] = center.x;
                npc.ai[1] = center.y;
            }
            if npc.local_ai[0] == 0.0 {
                npc.local_ai[0] = 1.0;
                // X first: NextFloat, Next(2), then Y's pair.
                let fx = env.rng.next_float();
                let sx = env.rng.next_max(2) * 2 - 1;
                let fy = env.rng.next_float();
                let sy = env.rng.next_max(2) * 2 - 1;
                npc.vx = (2.0 + (4.0 - 2.0) * fx) * sx as f32;
                npc.vy = (1.0 + (2.0 - 1.0) * fy) * sy as f32;
                npc.vx *= 0.7;
                npc.vy *= 0.7;
                npc.net_update = true;
            }
            let anchor = Vec2::new(npc.ai[0], npc.ai[1]) - center;
            if length(anchor) > 20.0 {
                let pull = Vec2::new(
                    if anchor.x > 0.0 { 1.0 } else { -1.0 },
                    if anchor.y > 0.0 { 1.0 } else { -1.0 },
                );
                npc.vx += pull.x * 0.04;
                npc.vy += pull.y * 0.04;
                if npc.vy.abs() > 2.0 {
                    npc.vy *= 0.95;
                }
            }
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 112, env.ticker);
            }
            let p = target_of(npc, env.players);
            let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
            if !p.dead && distance(pc, center) < 250.0 {
                drop_to_fly(npc, pc.x, center);
            }
        }
        1 => {
            // `:48153-48236` — the fly-to-player.
            out.lava_immune = Some(false);
            out.no_tile_collide = Some(false);
            if npc.collide_x {
                npc.facing *= -1;
                npc.vx = npc.facing as f32 * 2.0;
            }
            if npc.collide_y {
                npc.vy = if npc.old_vy > 0.0 { 1.0 } else { -1.0 };
            }
            let cruise = 4.5f32; // num12
            if math_sign(npc.vx) != npc.facing || npc.vx.abs() < cruise {
                npc.vx += npc.facing as f32 * 0.04;
                if npc.vx * (npc.facing as f32) < 0.0 {
                    if npc.vx.abs() > cruise {
                        npc.vx += npc.facing as f32 * 0.4;
                    } else {
                        npc.vx += npc.facing as f32 * 0.2;
                    }
                } else if npc.vx.abs() > cruise {
                    npc.vx = npc.facing as f32 * cruise;
                }
            }
            // `:48186-48214` — the 21×8 ground scan.
            let mut col = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32; // num13
            let span = 20; // num14
            if npc.facing < 0 {
                col -= span;
            }
            let row = ((npc.y + h) / 16.0) as i32; // num15
            let mut no_ground = true; // flag5
            let depth = 8; // num16
            let mut ground_near = false; // flag6
            for i in col..=col + span {
                for j in row..row + depth {
                    let blocked = env.world.tile(i, j).is_some_and(|t| {
                        (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
                    });
                    if blocked {
                        if j < row + 5 {
                            ground_near = true;
                        }
                        no_ground = false;
                        break;
                    }
                }
            }
            if no_ground {
                npc.vy += 0.05;
            } else {
                npc.vy -= 0.2;
            }
            if ground_near {
                npc.vy -= 0.3;
            }
            if npc.vy > 3.0 {
                npc.vy = 3.0;
            }
            if npc.vy < -5.0 {
                npc.vy = -5.0;
            }
        }
        2 => {
            // `:48237-48322` — the follow.
            out.no_tile_collide = Some(true);
            let (t_pos, t_size, is_player) = fairy_target(npc, env);
            let t_center = t_pos + t_size / 2.0;
            let dead = is_player && target_of(npc, env.players).dead;
            if dead {
                drop_to_fly(npc, t_center.x, center);
            } else {
                let r = centered_rectangle(
                    t_center,
                    Vec2::new(t_size.x + 60.0, (t_size.y as i32 / 2) as f32),
                );
                let hitbox = (npc.x as i32, npc.y as i32, w as i32, h as i32);
                if env.net_mode != 1 && rect_intersects(hitbox, r) {
                    if let Some((tx, ty)) = fairy_treasure_coords(npc, env.world) {
                        npc.ai[0] = tx as f32;
                        npc.ai[1] = ty as f32;
                        npc.ai[2] = 3.0;
                        npc.ai[3] = 0.0;
                        npc.net_update = true;
                    } else {
                        npc.ai[2] = 6.0;
                        npc.ai[3] = 0.0;
                        npc.net_update = true;
                    }
                } else {
                    let near = closest_point_in_rect(r, center);
                    let d = center - near;
                    let n = 1.0 / ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32;
                    let mut value = Vec2::new(-d.x * n, -d.y * n) * 2.0;
                    let dist = distance(center, near);
                    if dist > 150.0 {
                        value *= 2.0;
                    } else if dist > 80.0 {
                        value *= 1.5;
                    }
                    let v = Vec2::new(npc.vx, npc.vy);
                    let blended = v + (value - v) * 0.07;
                    npc.vx = blended.x;
                    npc.vy = blended.y;
                    let px = center.x as i32 >> 4;
                    let py = center.y as i32 >> 4;
                    if npc.ai[3] < 300.0 {
                        let (down, up) =
                            bird_flight_recommendation(env.world, 6, 3, px, py, npc.facing);
                        if down {
                            npc.vy += 0.05;
                        }
                        if up {
                            npc.vy -= 0.02;
                        }
                        if npc.vy > 2.0 {
                            npc.vy = 2.0;
                        }
                        if npc.vy < -4.0 {
                            npc.vy = -4.0;
                        }
                    }
                    if px >= 0 && px < env.world.width && py >= 0 && py < env.world.height {
                        if solid_tile_ij(env.world, px, py) {
                            npc.ai[3] = (npc.ai[3] + 2.0).min(400.0);
                        } else {
                            npc.ai[3] = (npc.ai[3] - 1.0).max(0.0);
                        }
                    }
                }
            }
        }
        3 => {
            // `:48323-48369` — the treasure circle.
            out.no_tile_collide = Some(true);
            if npc.ai[3] == 15.0 {
                out.sounds
                    .push((27, 1, npc.x as i32 as f32, npc.y as i32 as f32));
            }
            if npc.ai[3] <= 15.0 {
                npc.vx *= 0.9;
                npc.vy *= 0.9;
            } else {
                let p = target_of(npc, env.players);
                let pcx = p.x + (p.width / 2) as f32;
                npc.sprite_dir = if pcx > center.x { -1 } else { 1 };
                circle_facing = true;
                let mut phase = 0.0f32; // num6
                let t = npc.ai[3] - 15.0; // num7
                let mut height = 22.0f32;
                if t <= 65.0 {
                    phase = std::f32::consts::PI / 8.0;
                    height = 14.0;
                } else if t <= 130.0 {
                    phase = -std::f32::consts::PI / 8.0;
                    height = 18.0;
                }
                phase *= npc.facing as f32;
                let here = fairy_circle_offset(t / 65.0, phase, height, npc.facing);
                let next = fairy_circle_offset(t / 65.0 + 1.0 / 65.0, phase, height, npc.facing);
                let v = next - here;
                npc.vx = v.x;
                npc.vy = v.y;
            }
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 210.0 {
                npc.ai[2] = 4.0;
                if !env.players.is_empty() {
                    target_closest(npc, env.world, env.players, true, 112, env.ticker);
                }
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        6 => {
            // `:48370-48412` — the idle hover.
            out.no_tile_collide = Some(true);
            let p = target_of(npc, env.players);
            let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
            let to_player = pc - center;
            if length(to_player) > 100.0 {
                npc.ai[2] = 2.0;
                if !env.players.is_empty() {
                    target_closest(npc, env.world, env.players, true, 112, env.ticker);
                }
                npc.ai[3] = 0.0;
                npc.net_update = true;
            } else {
                if !crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h) {
                    out.no_tile_collide = Some(false);
                    if npc.collide_x {
                        npc.vx *= -1.0;
                    }
                    if npc.collide_y {
                        npc.vy *= -1.0;
                    }
                }
                if length(to_player) > 20.0 {
                    let pull = Vec2::new(
                        if to_player.x > 0.0 { 1.0 } else { -1.0 },
                        if to_player.y > 0.0 { 1.0 } else { -1.0 },
                    );
                    npc.vx += pull.x * 0.04;
                    npc.vy += pull.y * 0.04;
                    if npc.vy.abs() > 2.0 {
                        npc.vy *= 0.95;
                    }
                }
                if env.net_mode != 1 {
                    if let Some((tx, ty)) = fairy_treasure_coords(npc, env.world) {
                        npc.ai[0] = tx as f32;
                        npc.ai[1] = ty as f32;
                        npc.ai[2] = 3.0;
                        npc.ai[3] = 0.0;
                        npc.net_update = true;
                    }
                }
            }
        }
        4 => {
            // `:48413-48532` — the fly-to-treasure.
            out.no_tile_collide = Some(true);
            let (t_pos, t_size, is_player) = fairy_target(npc, env);
            let t_center = t_pos + t_size / 2.0;
            let dead = is_player && target_of(npc, env.players).dead;
            if dead {
                drop_to_fly(npc, t_center.x, center);
            } else {
                let r2 = centered_rectangle(
                    Vec2::new(npc.ai[0] * 16.0 + 8.0, npc.ai[1] * 16.0 + 8.0),
                    Vec2::splat(5.0),
                );
                let hitbox = (npc.x as i32, npc.y as i32, w as i32, h as i32);
                if rect_intersects(hitbox, r2) {
                    npc.ai[2] = 5.0;
                    npc.ai[3] = 0.0;
                    npc.net_update = true;
                } else {
                    let dist = distance(center, t_center);
                    let leash = 300.0f32; // num10
                    if dist > leash {
                        if dist < leash + 100.0
                            && !crate::player::Player::solid_collision(
                                env.world, npc.x, npc.y, w, h,
                            )
                        {
                            out.no_tile_collide = Some(false);
                            if npc.collide_x {
                                npc.vx *= -1.0;
                            }
                            if npc.collide_y {
                                npc.vy *= -1.0;
                            }
                        }
                        circle_facing = true;
                        let p = target_of(npc, env.players);
                        let pcx = p.x + (p.width / 2) as f32;
                        npc.sprite_dir = if pcx > center.x { -1 } else { 1 };
                        // DirectionFrom(t_center) = normalize(center - t).
                        let d = center - t_center;
                        let n = 1.0 / ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32;
                        let away = Vec2::new(d.x * n, d.y * n);
                        if dist > leash + 60.0 {
                            npc.vx += away.x * -0.1;
                            npc.vy += away.y * -0.1;
                            if env.rng.next_max(30) == 0 {
                                out.sounds
                                    .push((27, 1, npc.x as i32 as f32, npc.y as i32 as f32));
                            }
                        } else if dist < leash + 30.0 {
                            let near = closest_point_in_rect(r2, center);
                            let d2 = near - center;
                            let n2 = 1.0 / ((d2.x * d2.x + d2.y * d2.y) as f64).sqrt() as f32;
                            npc.vx += d2.x * n2 * 0.1;
                            npc.vy += d2.y * n2 * 0.1;
                        }
                        let v = Vec2::new(npc.vx, npc.vy);
                        let len = length(v);
                        if len > 1.0 {
                            npc.vx *= 1.0 / len;
                            npc.vy *= 1.0 / len;
                        }
                    } else {
                        let near = closest_point_in_rect(r2, center);
                        let d = near - center;
                        let n = 1.0 / ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32;
                        let mut value = Vec2::new(d.x * n, d.y * n);
                        let nd = distance(center, near);
                        if nd > 150.0 {
                            value *= 3.0;
                        } else if nd > 80.0 {
                            value *= 2.0;
                        }
                        let px = center.x as i32 >> 4;
                        let py = center.y as i32 >> 4;
                        let v = Vec2::new(npc.vx, npc.vy);
                        let blended = v + (value - v) * 0.07;
                        npc.vx = blended.x;
                        npc.vy = blended.y;
                        if npc.ai[3] < 300.0 {
                            let (down, up) =
                                bird_flight_recommendation(env.world, 4, 2, px, py, npc.facing);
                            if down {
                                npc.vy += 0.05;
                            }
                            if up {
                                npc.vy -= 0.05;
                            }
                            if npc.vy > 1.0 {
                                npc.vy = 1.0;
                            }
                            if npc.vy < -1.0 {
                                npc.vy = -1.0;
                            }
                        }
                        // `:48523` — the UNGATED SolidTile (the catch
                        // returns false out-of-bounds).
                        if solid_tile_ij(env.world, px, py) {
                            npc.ai[3] = (npc.ai[3] + 2.0).min(400.0);
                        } else {
                            npc.ai[3] = (npc.ai[3] - 1.0).max(0.0);
                        }
                    }
                }
            }
        }
        5 => {
            // `:48533-48588` — the sparkle-despawn.
            npc.local_ai[1] = 0.0;
            out.no_tile_collide = Some(true);
            // `:48537-48542` — flag2 (the ore-tile validity read) is
            // computed and NEVER read — the documented dead local.
            if npc.ai[3] == 15.0 {
                out.sounds
                    .push((27, 1, npc.x as i32 as f32, npc.y as i32 as f32));
            }
            if npc.ai[3] <= 15.0 {
                npc.vx *= 0.9;
                npc.vy *= 0.9;
            } else {
                circle_facing = true;
                let t = npc.ai[3] - 15.0; // num3
                let step = (t / 50.0) as i32; // num5
                let mut phase = (step as f32).cos() * (std::f32::consts::PI * 2.0) / 16.0; // num2
                let height = (step as f32 * 2.0).cos() * 10.0 + 8.0; // num4
                phase *= npc.facing as f32;
                let here = fairy_circle_offset(t / 50.0, phase, height, npc.facing);
                let next = fairy_circle_offset(t / 50.0 + 0.02, phase, height, npc.facing);
                let v = next - here;
                npc.vx = v.x;
                npc.vy = v.y;
                let p = target_of(npc, env.players);
                let pcx = p.x + (p.width / 2) as f32;
                npc.sprite_dir = if pcx > center.x { -1 } else { 1 };
            }
            npc.ai[3] += 1.0;
            if env.net_mode != 1 && npc.ai[3] > 200.0 {
                out.died = true; // `active = false`
                if env.net_mode == 0 {
                    fairy_effects(center, (npc_type - 583) as usize, env, &mut out);
                }
                // netMode 2: life = 0 + SendData 23/112 — the net
                // seam (documented, unported).
            }
        }
        7 => {
            // `:48589-48596` — the leave.
            out.no_tile_collide = Some(true);
            npc.vx += 0.05 * npc.facing as f32;
            npc.vx = npc.vx.clamp(-10.0, 10.0);
            npc.vy -= 0.025;
            npc.vy = npc.vy.clamp(-5.0, 5.0);
            npc.encourage_despawn(10);
        }
        _ => {}
    }
    // `:48598` — the invulnerability pair.
    let shielded = npc.ai[2] > 1.0;
    npc.dont_take_damage = shielded;
    out.dont_take_damage_from_hostiles = shielded;
    // `:48599-48612` — the same-style separation (Y only).
    for n in env.neighbors {
        if !n.is_self
            && n.active
            && n.ai_style == 112
            && (npc.x - n.x).abs() + (npc.y - n.y).abs() < w * 1.5
        {
            if npc.y < n.y {
                npc.vy -= 0.05;
            } else {
                npc.vy += 0.05;
            }
        }
    }
    // `:48613-48617` — the velocity facing (the circle states keep it).
    if !circle_facing {
        npc.facing = if npc.vx >= 0.0 { 1 } else { -1 };
        npc.sprite_dir = -npc.facing;
    }
    // `:48618-48640` — the glow dust.
    let (c1, c2) = FAIRY_COLORS[((npc_type - 583).max(0) as usize).min(2)];
    let num17 = 4.0f32;
    if (env.time_for_visuals as i32) % 2 == 0 {
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let c = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
        // The Lerp NextFloat draws BEFORE the NewDust internals.
        let t = env.rng.next_float();
        let idx = env.dust.new_dust_direct(
            env.rng,
            env.dust_ctx,
            env.world,
            c - Vec2::splat(num17 * 0.5),
            (num17 + 4.0) as i32,
            (num17 + 4.0) as i32,
            278,
            0.0,
            0.0,
            200,
            lerp_color(c1, c2, t),
            0.65,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.velocity *= 0.0;
            d.velocity += Vec2::new(npc.vx, npc.vy) * 0.3;
            d.no_gravity = true;
            d.no_light = true;
        }
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    // `:48641` — the light contribution (ToVector3 = /255, ×0.7).
    out.light = Some((
        center.x,
        center.y,
        [
            c1.r as f32 / 255.0 * 0.7,
            c1.g as f32 / 255.0 * 0.7,
            c1.b as f32 / 255.0 * 0.7,
        ],
    ));
    // `:48642-48649` — the bestiary-nearby achievement check.
    if env.net_mode != 2 {
        if let Some((lx, ly, lw, lh, dead)) = env.local_player {
            if !dead {
                // HitboxForBestiaryNearbyCheck (Player.cs:3313-3321):
                // the hitbox inflated +300/+200.
                let near = (lx as i32 - 300, ly as i32 - 200, lw + 600, lh + 400);
                let hitbox = (npc.x as i32, npc.y as i32, w as i32, h as i32);
                if rect_intersects(near, hitbox) {
                    out.bestiary_event = true;
                }
            }
        }
    }
    out
}

/// `NPC.FindFrame` cases 583-585 (`NPC.cs:69835-69848`) — the counter
/// pair (the style73 convention). The frame pass ALSO writes
/// `rotation = vx * 0.1` (`:69838`) — the wiring caller applies it.
pub fn fairy_find_frame(frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame_row;
    if counter >= 4.0 {
        row += 1;
        counter = 0.0;
        if row >= FAIRY_CRITTER_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 583-shaped NPC (the style73 placeholder convention).
    fn fairy(x: f32, y: f32) -> Npc {
        let d = &FAIRY_CRITTER;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
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

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    /// `NPC.cs:16583-16607` — the shared row, field by field.
    #[test]
    fn w4_d33_s112_the_defaults_row() {
        let d = &FAIRY_CRITTER;
        assert_eq!((d.width, d.height), (18, 20));
        assert_eq!(d.ai_style, 112);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.catch_items, [4068, 4069, 4070]);
        assert!(d.no_gravity);
        assert_eq!(d.rarity, 2);
        assert_eq!(FAIRY_CRITTER_FRAME_COUNT, 4);
        assert_eq!(FAIRY_COLORS[0].0, Color::new(255, 105, 180, 255), "HotPink");
        assert_eq!(
            FAIRY_COLORS[2].1,
            Color::new(173, 216, 230, 255),
            "LightBlue"
        );
    }

    /// `:48121-48127` — the launch draws NextFloat, Next(2),
    /// NextFloat, Next(2) in X-then-Y order; the ×0.7 scale applies.
    #[test]
    fn w4_d33_s112_the_launch_draw_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut oracle = UnifiedRandom::new(112);
        let mut n = fairy(100.0, 100.0);
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let _ = style112_step(&mut n, 583, &mut env);
        let fx = oracle.next_float();
        let sx = oracle.next_max(2) * 2 - 1;
        let fy = oracle.next_float();
        let sy = oracle.next_max(2) * 2 - 1;
        assert_eq!(rng.peek(), oracle.peek(), "the launch draw order");
        assert_eq!(n.vx, (2.0 + 2.0 * fx) * sx as f32 * 0.7);
        assert_eq!(n.vy, (1.0 + fy) * sy as f32 * 0.7);
        assert_eq!(n.local_ai[0], 1.0);
        assert!(n.net_update);
        // The centre latch.
        assert_eq!(n.ai[0], 109.0);
        assert_eq!(n.ai[1], 110.0);
    }

    /// `:48140-48150` — the alive player within 250 px flips to state
    /// 1 facing AWAY; the against-sign vx re-arms to direction*2.
    #[test]
    fn w4_d33_s112_the_near_player_flip() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(200.0, 100.0)]; // right of the fairy
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut n = fairy(100.0, 100.0);
        n.local_ai[0] = 1.0; // skip the launch
        n.vx = 1.0; // toward the player — against the away-facing
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let _ = style112_step(&mut n, 583, &mut env);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.facing, -1, "faces away from the right-side player");
        assert_eq!(n.vx, -2.0, "the sign-flip re-arm");
        assert_eq!(n.ai[3], 0.0);
    }

    /// `:48166-48185` — the 4.5 cruise: the under-speed accel, the
    /// against-sign 0.2 double-accel, the over-speed clamp.
    #[test]
    fn w4_d33_s112_the_cruise_arms() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        // Under speed, with the facing: 0 + 0.04.
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 1.0;
        let _ = style112_step(&mut n, 583, &mut env);
        assert!((n.vx - 0.04).abs() < 1e-6);
        // Against the facing under 4.5: -1 + 0.04 + 0.2.
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 1.0;
        n.vx = -1.0;
        let _ = style112_step(&mut n, 583, &mut env);
        assert!((n.vx - (-0.76)).abs() < 1e-6, "vx {}", n.vx);
        // Over speed with the facing: the clamp to 4.5.
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 1.0;
        n.vx = 5.0;
        let _ = style112_step(&mut n, 583, &mut env);
        // Sign matches and |5| >= 4.5 — the gate is FALSE, no change
        // (the ground scan's vy only). vx stays 5.
        assert_eq!(n.vx, 5.0, "the gate skips the matched over-speed");
    }

    /// `:48715-48772` — the treasure search finds the lure-able tile
    /// (the chest 21) and returns its coords; the empty world finds
    /// nothing.
    #[test]
    fn w4_d33_s112_the_treasure_search() {
        let mut w = air();
        // A chest (tile 21 — FriendlyFairyCanLureTo) at (100, 100).
        let i = w.index(100, 100);
        w.tiles[i] = Tile {
            kind: 21,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let n = fairy(100.0 * 16.0, 100.0 * 16.0);
        assert_eq!(fairy_treasure_coords(&n, &w), Some((100, 100)));
        let w = air();
        assert_eq!(fairy_treasure_coords(&n, &w), None);
    }

    /// `:48325-48368` — the circle: the ai[3]==15 chime, the ≤15
    /// damp, the 210-tick expiry to state 4.
    #[test]
    fn w4_d33_s112_the_circle_chime_and_expiry() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 3.0;
        n.ai[3] = 15.0;
        n.vx = 2.0;
        let out = style112_step(&mut n, 583, &mut env);
        assert!(
            out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (27, 1)),
            "the chime"
        );
        assert_eq!(n.vx, 1.8, "the ≤15 damp");
        assert_eq!(n.ai[3], 16.0);
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 3.0;
        n.ai[3] = 209.0;
        let _ = style112_step(&mut n, 583, &mut env);
        assert_eq!(n.ai[2], 4.0);
        assert_eq!(n.ai[3], 0.0);
        assert!(n.net_update);
    }

    /// `:48573-48585` + `:48652-48683` — the state-5 expiry: the
    /// death flag + the 40-dust FairyEffects burst (each dust draws
    /// the Lerp NextFloat then Next(0,17)) + the Item4 chime.
    #[test]
    fn w4_d33_s112_the_sparkle_despawn() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut oracle = UnifiedRandom::new(112);
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 5.0;
        n.ai[3] = 200.0;
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let out = style112_step(&mut n, 583, &mut env);
        assert!(out.died);
        assert!(
            out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (2, 4)),
            "Item4"
        );
        for _ in 0..40 {
            oracle.next_float(); // the Lerp
            oracle.next_range(0, 17); // the fadeIn
        }
        assert_eq!(rng.peek(), oracle.peek(), "the burst draw order");
        // The client never expires.
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 5.0;
        n.ai[3] = 200.0;
        env.net_mode = 1;
        let out = style112_step(&mut n, 583, &mut env);
        assert!(!out.died);
    }

    /// `:48092-48109` — the 18000-tick timeout flips to 7 facing away.
    #[test]
    fn w4_d33_s112_the_timeout_leave() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(200.0, 100.0)]; // right of the fairy
        let neighbors: [FairyNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(112);
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 2.0;
        n.local_ai[1] = 17999.0;
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let _ = style112_step(&mut n, 583, &mut env);
        assert_eq!(n.ai[2], 7.0);
        assert_eq!(n.facing, -1, "the player right → face -1 (away)");
        assert!(n.net_update);
        // State 7 ran the same tick: the -0.025 drift.
        assert!(n.vy < 0.0);
    }

    /// `:48598-48617` — the shield pair, the Y-only separation, the
    /// velocity facing with spriteDirection = -direction.
    #[test]
    fn w4_d33_s112_the_tail_shield_and_separation() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let mut sib = FairyNeighbor {
            is_self: false,
            active: true,
            ai_style: 112,
            x: 105.0,
            y: 105.0,
        };
        let neighbors = [
            FairyNeighbor {
                is_self: true,
                active: true,
                ai_style: 112,
                x: 100.0,
                y: 100.0,
            },
            sib,
        ];
        let mut rng = UnifiedRandom::new(112);
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 7.0; // the leave — no circle facing
        n.vx = -1.0;
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let out = style112_step(&mut n, 583, &mut env);
        assert!(n.dont_take_damage);
        assert!(out.dont_take_damage_from_hostiles);
        // The sibling below: vy += 0.05 on top of the -0.025 drift.
        assert!((n.vy - 0.025).abs() < 1e-6, "vy {}", n.vy);
        assert_eq!(n.facing, -1, "vx < 0");
        assert_eq!(n.sprite_dir, 1, "-direction");
        sib.ai_style = 3;
        let neighbors = [
            FairyNeighbor {
                is_self: true,
                active: true,
                ai_style: 112,
                x: 100.0,
                y: 100.0,
            },
            sib,
        ];
        let mut n = fairy(100.0, 100.0);
        n.ai[2] = 7.0;
        let mut env = Style112Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
            time_for_visuals: 1.0,
            ticker: 7,
            local_player: None,
        };
        let _ = style112_step(&mut n, 583, &mut env);
        assert!(
            (n.vy - (-0.025)).abs() < 1e-6,
            "the non-112 sibling is skipped"
        );
    }

    /// `NPC.cs:69835-69848` — the 4-tick row advance wraps at 4.
    #[test]
    fn w4_d33_s112_the_find_frame_wraps_at_four() {
        assert_eq!(fairy_find_frame(0, 2.0), (0, 3.0));
        assert_eq!(fairy_find_frame(0, 3.0), (1, 0.0));
        assert_eq!(fairy_find_frame(3, 3.0), (0, 0.0), "the wrap");
    }

    /// The full-pool stream parity: 40 ticks on the REAL dust ctx must
    /// match a cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s112_the_full_pool_run_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let players = [cand(300.0, 300.0)];
        let neighbors: [FairyNeighbor; 0] = [];
        let mut a = fairy(280.0, 280.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(583);
        let mut rng_b = UnifiedRandom::new(583);
        for tick in 0..40 {
            let mut env_a = Style112Env {
                world: &w,
                dust: &mut pool_a,
                dust_ctx: &c,
                rng: &mut rng_a,
                players: &players,
                neighbors: &neighbors,
                net_mode: 0,
                time_for_visuals: tick as f64,
                ticker: 7,
                local_player: None,
            };
            let _ = style112_step(&mut a, 583, &mut env_a);
            let mut env_b = Style112Env {
                world: &w,
                dust: &mut pool_b,
                dust_ctx: &c,
                rng: &mut rng_b,
                players: &players,
                neighbors: &neighbors,
                net_mode: 0,
                time_for_visuals: tick as f64,
                ticker: 7,
                local_player: None,
            };
            let _ = style112_step(&mut b, 583, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[2].to_bits(), b.ai[2].to_bits(), "state @{tick}");
        }
    }
}
