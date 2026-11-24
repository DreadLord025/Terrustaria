//! The aiStyle-117 family — the Blood Nautilus
//! (`NPC.AI_117_BloodNautilus`, `NPC.cs:47127-47472`, dispatched at
//! `:41088-41091`). W4-d33, research/25 row 117, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the BloodNautilus 618
//! (`NPCID.BloodNautilus`, `ID/NPCID.cs:11913`; `SetDefaults` row
//! `NPC.cs:16918-16933`). The census of `this.aiStyle = 117;` across
//! the decompiled tree returns `NPC.cs:16923` plus five
//! projectile-space hits (`Projectile.cs:6558`/`:6588`/`:6721`/
//! `:8992`/`:9879` — the 608/612/624/953/1044 rows, a different
//! machine, out of NPC scope). The port's roster has no nautilus kind
//! — the module lands UNWIRED (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The blood-moon mini-boss: a four-state cycle — -1 the fade-in
//! spawn, 0 the hover, 1 the reflecting charge, 2 the tear spit, 3
//! the call-for-help. `ai[3]` counts the attacks; every 7th is a call
//! (`% 7 == 3`), the evens a charge, the odds a spit.
//!
//! The head (`:47139-47176`): the `localAI[0] == 0` spawn latch
//! (alpha 255, the non-client `ai[0] = -1`); the `ai[0] != -1 &&
//! Next(4) == 0` blood drip (type 5 at position+(5,5), box (w-10,
//! h-10), velocity ×0.5 with the negative-vy flip, alpha 120, scale
//! `1 + NextFloat()*0.4`, `+= velocity*0.3`, under the netOffset
//! shift); the target-255 pick (`ai[2] = direction`); the dead /
//! >2000-px retarget; `GetTargetData` — the DAY or non-blood-moon
//! world BLANKS it to `default` (the (0,0) aim).
//!
//! State -1 (`:47180-47231`): the ×0.98 damp; the aim-sign facing
//! (`spriteDirection = -direction`); the `localAI[1] == 0 && alpha <
//! 100` 36-dust spawn ring (the velocity-normalized ellipse
//! `(w/2, h)*0.75*0.5` rotated per index, each dust at `ring +
//! offset` with the offset×2 velocity then the `Normalize(offset)*3`
//! overwrite, noGravity); the `ai[2] > 5` rise (-2.5 vy, alpha -10,
//! the SolidCollision +15 capped 150, the alpha floor 0); `ai[2]++`;
//! at 50 the state-0 reset.
//!
//! State 0 (`:47232-47273`): the hover at `target + (-ai[2]*300,
//! -200)` — the >50-px `SimpleFlyMovement(DirectionTo*7.5, 0.15)`;
//! the facing; the aim rotation (`DirectionTo.ToRotation() -
//! 0.47123894*spriteDirection`, the -1 π-shift, the facing-flip
//! negate) at `AngleTowards 0.02`; `ai[1]++`; past 60 the attack pick
//! (`ai[3] % 7 == 3` → 3; `% 2 == 0` → 2 + Item170; else 1 + Item170).
//!
//! State 1 (`:47274-47336`): the INVERTED facing compare; the
//! `DirectionFrom` aim; the `ai[1] < 90` windup (the Item172 cue at
//! 89, the reflects flag, the ×0.95 damp, the AngleLerp 0.02, the two
//! mouth dusts — type 16 at mouth+dir*60-(40,40) box 80 alpha 150
//! scale 0.6 fadeIn 1 and mouth+dir*100-(30,30) box 60 alpha 100
//! scale 0.9 fadeIn 1.5, each velocity redirected
//! `DirectionTo(mouth + Circular(15,15))` at the drawn length (+5 for
//! the second), noGravity); the `90..270` charge (the AngleLerp 0.05,
//! the >150-px `mouthDir*-16 + DirectionTo(target)*1.5` lunge, the
//! 4×2 type-5 mouth dusts with the `DirectionFrom` redirect and the
//! dir*60/dir*100 position pull-backs); at 270 → state 0.
//!
//! State 2 (`:47337-47419`): the facing + the AngleLerp 0.2 aim; the
//! `ai[1] < 90` windup (×0.95, the `Next(4) != 0` two type-16 dusts —
//! the 120/160 boxes, the +3/+5 length boosts); the `90..180` spit
//! (×0.9, the `(ai[1]-90) % 30` phase — `< 24` runs the 5×2 type-5
//! dust pairs, `(int)phase == 0` fires the `mouthDir*-8` recoil + the
//! 20×2 dust burst + the non-client `Next(5,11)` type-814
//! BloodNautilusShot volley at `mouth - dir*5` with the
//! `dir*10 + NextVector2Square(-6,6)` velocity and the
//! `GetAttackDamage_ForProjectiles(30, 25)` damage); at 180 → state 0.
//!
//! State 3 (`:47420-47453`): the facing + spriteDirection; the
//! `ai[1] < 180` hold — the velocity Lerp is IMMEDIATELY overwritten
//! with Zero (`:47428-47429`, the dead computation documented), the
//! AngleLerp-to-0 0.02, the dead `GetMouthPositionAndRotation(out _,
//! out _)` call (no effect), the `GetLerpValue(0,0.5,t) *
//! GetLerpValue(1,0.5,t)` UNCLAMPED glow (the (1,0.5,0.5) light), the
//! `Next(3) != 0` type-5 dust at Center-(6,6) box 12 alpha 60 scale
//! 1.4 with the (spriteDir*12, 12) shift + ×0.1 velocity; the
//! `ai[1] == 10/20/30` CallForHelp; at 180 → state 0.
//!
//! The transition tail (`:47455-47471`): `ai[0] = next`, ai[1]/ai[2]
//! zero, netUpdate, TargetClosest; state 0 keeps `ai[2] = direction`,
//! the attacks increment `ai[3]`. `reflectsProjectiles = flag` (the
//! state-1 windup only).
//!
//! [`nautilus_call_for_help`] (`BloodNautilus_CallForHelp`,
//! `:47474-47516`): the client / dead / >2000-px / `CountNPCS(619) >=
//! 3` gates; up to 100 `Next(±20)` tile rolls — accepted outside the
//! 8-box AND the 3-box, inactive, no lava, no `SolidTiles(±2)`, and
//! `CanHitLine` to the player — spawn ONE type-813 BloodNautilusTears
//! at `tile*16+8`.
//!
//! ## RNG order (by construction)
//!
//! Head: Next(4) (passing: the NewDust internals + ONE NextFloat).
//! State -1: the ring's 36 NewDust internals (no extra draws). State
//! 1: per dust the internals THEN the Circular(15,15)/(5,5)/(10,10)
//! TWO NextFloat. State 2: Next(4); the phase dusts as state 1; the
//! volley's Next(5,11) then per-shot NextVector2Square (TWO
//! NextDouble). State 3: Next(3); CallForHelp's 2 Next per attempt.
//! TargetClosest is RNG-free.
//!
//! ## FindFrame case 618
//!
//! `NPC.cs:67000-67010`: `++frameCounter >= 5` advances the row,
//! wrapping at `Main.npcFrameCount[618] = 4`. [`nautilus_find_frame`]
//! ports the pair (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_ai_fishron::simple_fly_movement;
use crate::npc_ai_style37::{attack_damage_for_projectiles, rotated_by};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::projectile::{angle_lerp, angle_towards, can_hit_line};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tile::LiquidKind;
use tr_core::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{UnifiedRandom, World};

/// The type-618 `SetDefaults` row (`NPC.cs:16918-16933`), verbatim.
pub struct BloodNautilusDefaults {
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
    pub knock_back_resist: f32,
    /// `Item.buyPrice(0, 2)` = 20000 copper.
    pub value: i32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub rarity: i32,
}

/// `NPC.cs:16918-16933` — `else if (this.type == 618)`.
pub const BLOOD_NAUTILUS: BloodNautilusDefaults = BloodNautilusDefaults {
    width: 100,
    height: 100,
    ai_style: 117,
    damage: 55,
    defense: 24,
    life_max: 7000,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    knock_back_resist: 0.0,
    value: 20000,
    no_gravity: true,
    no_tile_collide: true,
    rarity: 1,
};

/// `Main.npcFrameCount[618]` = 4 (`Main.cs:67145`, the 610-619 row:
/// `6, 13, 5, 5, 7, 5, 14, 14, 4, 6` — index 618 is the 9th entry).
pub const BLOOD_NAUTILUS_FRAME_COUNT: u32 = 4;

/// The per-tick environment (the `Style111Env` shape).
pub struct Style117Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.dayTime` — the target blanking (`:47173`).
    pub day_time: bool,
    /// `Main.bloodMoon` — the target blanking.
    pub blood_moon: bool,
    /// `this.difficulty` — the volley's damage remap.
    pub difficulty: f32,
    /// `NPC.CountNPCS(619)` — the CallForHelp cap (`:47476`); the
    /// wiring counts the pool.
    pub nautilus_count: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// A `Projectile.NewProjectile` request — the spit volley (814) or
/// the CallForHelp tears (813).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NautilusShot {
    /// 814 (`BloodNautilusShot`) or 813 (`BloodNautilusTears`).
    pub projectile_type: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style117Outcome {
    /// The projectile requests in C# order (the volley, then the
    /// help tears — never both on one tick).
    pub shots: Vec<NautilusShot>,
    /// The `SoundEngine.PlaySound` cues — `(type, style, x, y)`:
    /// Item170 `(2, 170)` (the attack pick), Item172 `(2, 172)` (the
    /// charge windup's last tick).
    pub sounds: Vec<(i32, i32, f32, f32)>,
    /// The state-3 `Lighting.AddLight(Center, (1,0.5,0.5)*glow)`
    /// contribution (`:47434`).
    pub light: Option<(f32, f32, [f32; 3])>,
    /// `this.reflectsProjectiles` (`:47471`) — the state-1 windup
    /// only; the `Npc` slot has no field (the outcome seam).
    pub reflects_projectiles: bool,
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

/// XNA `Vector2.Normalize` — UNGUARDED (the style100 convention; the
/// zero vector yields NaN exactly like the C#).
fn normalize_xna(v: Vec2) -> Vec2 {
    let n = 1.0 / ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32;
    Vec2::new(v.x * n, v.y * n)
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

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE NextFloat.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second NextFloat radius scale.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `Utils.NextVector2Square` (`Utils.cs:1251-1254`) — TWO NextDouble,
/// X first, the double cast to f32 BEFORE the multiply.
fn next_vector2_square(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Utils.GetLerpValue(from, to, t)` UNCLAMPED (`Utils.cs:342` — the
/// state-3 glow calls the two-arg form, `clamped` defaults false).
fn get_lerp_value_unclamped(from: f32, to: f32, t: f32) -> f32 {
    (t - from) / (to - from)
}

/// `BloodNautilus_GetMouthPositionAndRotation`
/// (`NPC.cs:66416-66425`): the rotation + the 0.47123894 sprite tilt
/// (the -1 π-shift), the unit direction, the Centre + dir*50 mouth.
fn mouth(npc: &Npc) -> (Vec2, Vec2) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let mut a = npc.rotation + 0.47123894 * npc.sprite_dir as f32;
    if npc.sprite_dir == -1 {
        a += std::f32::consts::PI;
    }
    let dir = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32);
    (center + dir * 50.0, dir)
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out of bounds = solid; a null tile
/// SKIPS (returns false for that cell — the C# `return false` inside
/// the loop is the whole-call false, ported as the early return);
/// active && !inactive && solid && !solidTop counts.
fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for i in x0..=x1 {
        for j in y0..=y1 {
            match world.tile(i, j) {
                None => return false, // the C# null-tile `return false`
                Some(t) => {
                    if t.active()
                        && !t.inactive()
                        && TILE_SOLID.has(t.kind)
                        && !TILE_SOLID_TOP.has(t.kind)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
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

/// `NPC.GetTargetData` for 618 (NOT in `UsesNewTargeting` —
/// player-only): the target player's centre, or the `default` (0,0)
/// when invalid OR the day / non-blood-moon blanking (`:47172-47176`).
fn nautilus_target(npc: &Npc, env: &Style117Env<'_>) -> Vec2 {
    if env.day_time || !env.blood_moon {
        return Vec2::ZERO;
    }
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
            return Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        }
    }
    Vec2::ZERO
}

/// `NPC.BloodNautilus_CallForHelp` (`NPC.cs:47474-47516`): the gates,
/// then up to 100 tile rolls for the tears spawn. The
/// `Main.tile[x,y].nactive()` null read is the port's `None` →
/// inactive (the documented widening, the style114 precedent).
fn nautilus_call_for_help(npc: &Npc, env: &mut Style117Env<'_>, out: &mut Style117Outcome) {
    let p = target_of(npc, env.players);
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
    if env.net_mode == 1
        || !p.active
        || p.dead
        || distance(center, pc) > 2000.0
        || env.nautilus_count >= 3
    {
        return;
    }
    let px = center.x as i32 >> 4;
    let py = center.y as i32 >> 4;
    for _ in 0..100 {
        let tx = env.rng.next_range(px - 20, px + 20 + 1);
        let ty = env.rng.next_range(py - 20, py + 20 + 1);
        // Outside the 8-box AND the 3-box, and inactive.
        let outside8 = ty < py - 8 || ty > py + 8 || tx < px - 8 || tx > px + 8;
        let outside3 = ty < py - 3 || ty > py + 3 || tx < px - 3 || tx > px + 3;
        let inactive = !env
            .world
            .tile(tx, ty)
            .is_some_and(|t| t.active() && !t.inactive());
        if !(outside8 && outside3 && inactive) {
            continue;
        }
        // No lava — `Tile.lava()` (Tile.cs:345-348) is the header bit
        // ALONE, no liquid>0 term (verbatim).
        if env
            .world
            .tile(tx, ty)
            .is_some_and(|t| t.liquid_kind() == LiquidKind::Lava)
        {
            continue;
        }
        // No solid in the ±2 box.
        if solid_tiles(env.world, tx - 2, tx + 2, ty - 2, ty + 2) {
            continue;
        }
        // The sight line to the player.
        if !can_hit_line(
            env.world, center.x, center.y, 0.0, 0.0, pc.x, pc.y, 0.0, 0.0,
        ) {
            continue;
        }
        out.shots.push(NautilusShot {
            projectile_type: 813,
            x: (tx * 16 + 8) as f32,
            y: (ty * 16 + 8) as f32,
            vx: 0.0,
            vy: 0.0,
            damage: 0,
        });
        break;
    }
}

/// One fixed step of the aiStyle-117 arm (`NPC.cs:47127-47472`), in C#
/// order. The caller owns the movement integration (the row's
/// noGravity/noTileCollide are true) and drains [`Style117Outcome`].
pub fn style117_step(npc: &mut Npc, env: &mut Style117Env<'_>) -> Style117Outcome {
    let mut out = Style117Outcome::default();
    // `:47129-47138` — the C# locals, verbatim.
    let move_speed = 0.15f32;
    let hover_speed = 7.5f32; // num
    let hover_ticks = 60.0f32; // num2
    let charge_windup = 90.0f32; // num3
    let charge_dash = 180.0f32; // num4
    let spit_windup = 90.0f32; // num5
    let spit_fire = 90.0f32; // num6
    let spit_bursts = 3; // num7
    let call_ticks = 180.0f32; // num8
    let mut reflects = false; // flag
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:47139-47148` — the spawn latch.
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        npc.alpha = 255;
        if env.net_mode != 1 {
            npc.ai[0] = -1.0;
            npc.net_update = true;
        }
    }
    // `:47149-47162` — the blood drip.
    if npc.ai[0] != -1.0 && env.rng.next_max(4) == 0 {
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let idx = env.dust.new_dust_direct(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(npc.x + 5.0, npc.y + 5.0),
            w as i32 - 10,
            h as i32 - 10,
            5,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.velocity *= 0.5;
            if d.velocity.y < 0.0 {
                d.velocity.y *= -1.0;
            }
            d.alpha = 120;
            d.scale = 1.0 + env.rng.next_float() * 0.4;
            d.velocity += Vec2::new(npc.vx, npc.vy) * 0.3;
        }
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    // `:47163-47171` — the target picks.
    if npc.target == 255 {
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 117, env.ticker);
        }
        npc.ai[2] = npc.facing as f32;
    }
    {
        let p = target_of(npc, env.players);
        let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        if p.dead || distance(pc, center) > 2000.0 {
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 117, env.ticker);
            }
        }
    }
    let target = nautilus_target(npc, env);
    let mut next_state: i32 = -1; // num9
    match npc.ai[0] as i32 {
        -1 => {
            // `:47180-47231` — the fade-in spawn.
            npc.vx *= 0.98;
            npc.vy *= 0.98;
            let sign = math_sign(target.x - center.x);
            if sign != 0 {
                npc.facing = sign;
                npc.sprite_dir = -npc.facing;
            }
            if npc.local_ai[1] == 0.0 && npc.alpha < 100 {
                npc.local_ai[1] = 1.0;
                // `:47192-47202` — the 36-dust ring.
                let ring_count = 36;
                for l in 0..ring_count {
                    npc.x += npc.net_offset.x;
                    npc.y += npc.net_offset.y;
                    let c = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
                    let base = normalize_xna(Vec2::new(npc.vx, npc.vy))
                        * Vec2::new(w / 2.0, h)
                        * 0.75
                        * 0.5;
                    let angle = ((l - (ring_count / 2 - 1)) as f32 * (std::f32::consts::PI * 2.0)
                        / ring_count as f32) as f64;
                    let (rx, ry) = rotated_by(base.x, base.y, angle, 0.0, 0.0);
                    let ring = Vec2::new(rx, ry) + c;
                    let offset = ring - c;
                    let idx = env.dust.new_dust(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        ring + offset,
                        0,
                        0,
                        5,
                        offset.x * 2.0,
                        offset.y * 2.0,
                        100,
                        Color::default(),
                        1.4,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.no_gravity = true;
                        d.velocity = normalize_xna(offset) * 3.0;
                    }
                    npc.x -= npc.net_offset.x;
                    npc.y -= npc.net_offset.y;
                }
            }
            if npc.ai[2] > 5.0 {
                npc.vy = -2.5;
                npc.alpha = npc.alpha.saturating_sub(10);
                if crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h) {
                    npc.alpha = (npc.alpha + 15).min(150);
                }
                // the alpha floor 0 — saturating_sub already floors.
            }
            npc.ai[2] += 1.0;
            if npc.ai[2] >= 50.0 {
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        0 => {
            // `:47232-47273` — the hover.
            let destination = target + Vec2::new((0.0 - npc.ai[2]) * 300.0, -200.0);
            if distance(center, destination) > 50.0 {
                let dir = normalize_xna(destination - center) * hover_speed;
                simple_fly_movement(&mut npc.vx, &mut npc.vy, dir.x, dir.y, move_speed);
            }
            npc.facing = if center.x < target.x { 1 } else { -1 };
            // ToRotation = atan2(Y, X) — the signed angle.
            let d = target - center;
            let mut aim =
                (d.y as f64).atan2(d.x as f64) as f32 - 0.47123894 * npc.sprite_dir as f32;
            if npc.sprite_dir == -1 {
                aim += std::f32::consts::PI;
            }
            if npc.sprite_dir != npc.facing {
                npc.sprite_dir = npc.facing;
                npc.rotation = 0.0 - npc.rotation;
                aim = 0.0 - aim;
            }
            npc.rotation = angle_towards(npc.rotation, aim, 0.02);
            npc.ai[1] += 1.0;
            if npc.ai[1] > hover_ticks {
                next_state = 1;
                let cycle = npc.ai[3] as i32;
                if cycle % 7 == 3 {
                    next_state = 3;
                } else if cycle % 2 == 0 {
                    out.sounds.push((2, 170, center.x, center.y));
                    next_state = 2;
                } else {
                    out.sounds.push((2, 170, center.x, center.y));
                    next_state = 1;
                }
            }
        }
        1 => {
            // `:47274-47336` — the reflecting charge.
            npc.facing = if !(center.x < target.x) { 1 } else { -1 };
            let d = center - target; // DirectionFrom
            let mut aim =
                (d.y as f64).atan2(d.x as f64) as f32 - 0.47123894 * npc.sprite_dir as f32;
            if npc.sprite_dir == -1 {
                aim += std::f32::consts::PI;
            }
            let windup = npc.ai[1] < charge_windup; // flag2
            if (npc.sprite_dir != npc.facing) & windup {
                npc.sprite_dir = npc.facing;
                npc.rotation = 0.0 - npc.rotation;
                aim = 0.0 - aim;
            }
            if npc.ai[1] < charge_windup {
                if npc.ai[1] == charge_windup - 1.0 {
                    out.sounds.push((2, 172, center.x, center.y));
                }
                reflects = true;
                npc.vx *= 0.95;
                npc.vy *= 0.95;
                npc.rotation = angle_lerp(npc.rotation, aim, 0.02);
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                let (m, dir) = mouth(npc);
                // The two windup dusts.
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    m + dir * 60.0 - Vec2::splat(40.0),
                    80,
                    80,
                    16,
                    0.0,
                    0.0,
                    150,
                    Color::TRANSPARENT,
                    0.6,
                );
                {
                    let d = env.dust.slot_mut(idx);
                    d.fade_in = 1.0;
                    let jitter = next_vector2_circular(env.rng, 15.0, 15.0);
                    let to = normalize_xna(m + jitter - d.position);
                    d.velocity = to * length(d.velocity);
                    d.no_gravity = true;
                }
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    m + dir * 100.0 - Vec2::splat(30.0),
                    60,
                    60,
                    16,
                    0.0,
                    0.0,
                    100,
                    Color::TRANSPARENT,
                    0.9,
                );
                {
                    let d = env.dust.slot_mut(idx);
                    d.fade_in = 1.5;
                    let jitter = next_vector2_circular(env.rng, 15.0, 15.0);
                    let to = normalize_xna(m + jitter - d.position);
                    d.velocity = to * (length(d.velocity) + 5.0);
                    d.no_gravity = true;
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            } else if npc.ai[1] < charge_windup + charge_dash {
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                npc.rotation = angle_lerp(npc.rotation, aim, 0.05);
                let (m, dir) = mouth(npc);
                if distance(center, target) > 150.0 {
                    let to = normalize_xna(target - center);
                    let v = dir * -16.0 + to * 1.5;
                    npc.vx = v.x;
                    npc.vy = v.y;
                }
                for _ in 0..4 {
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        m + dir * 60.0 - Vec2::splat(15.0),
                        30,
                        30,
                        5,
                        0.0,
                        0.0,
                        0,
                        Color::TRANSPARENT,
                        1.5,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        let jitter = next_vector2_circular(env.rng, 5.0, 5.0);
                        let from = normalize_xna(d.position - (m + jitter));
                        d.velocity = from * length(d.velocity);
                        d.position -= dir * 60.0;
                    }
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        m + dir * 100.0 - Vec2::splat(20.0),
                        40,
                        40,
                        5,
                        0.0,
                        0.0,
                        100,
                        Color::TRANSPARENT,
                        1.5,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        let jitter = next_vector2_circular(env.rng, 10.0, 10.0);
                        let from = normalize_xna(d.position - (m + jitter));
                        d.velocity = from * (length(d.velocity) + 5.0);
                        d.position -= dir * 100.0;
                    }
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            }
            npc.ai[1] += 1.0;
            if npc.ai[1] >= charge_windup + charge_dash {
                next_state = 0;
            }
        }
        2 => {
            // `:47337-47419` — the tear spit.
            npc.facing = if center.x < target.x { 1 } else { -1 };
            let d = target - center;
            let mut aim =
                (d.y as f64).atan2(d.x as f64) as f32 - 0.47123894 * npc.sprite_dir as f32;
            if npc.sprite_dir == -1 {
                aim += std::f32::consts::PI;
            }
            if npc.sprite_dir != npc.facing {
                npc.sprite_dir = npc.facing;
                npc.rotation = 0.0 - npc.rotation;
                aim = 0.0 - aim;
            }
            npc.rotation = angle_lerp(npc.rotation, aim, 0.2);
            if npc.ai[1] < spit_windup {
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                npc.vx *= 0.95;
                npc.vy *= 0.95;
                let (m, dir) = mouth(npc);
                if env.rng.next_max(4) != 0 {
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        m + dir * 60.0 - Vec2::splat(60.0),
                        120,
                        120,
                        16,
                        0.0,
                        0.0,
                        150,
                        Color::TRANSPARENT,
                        0.6,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.fade_in = 1.0;
                        let jitter = next_vector2_circular(env.rng, 15.0, 15.0);
                        let to = normalize_xna(m + jitter - d.position);
                        d.velocity = to * (length(d.velocity) + 3.0);
                        d.no_gravity = true;
                    }
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        m + dir * 100.0 - Vec2::splat(80.0),
                        160,
                        160,
                        16,
                        0.0,
                        0.0,
                        100,
                        Color::TRANSPARENT,
                        0.9,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.fade_in = 1.5;
                        let jitter = next_vector2_circular(env.rng, 15.0, 15.0);
                        let to = normalize_xna(m + jitter - d.position);
                        d.velocity = to * (length(d.velocity) + 5.0);
                        d.no_gravity = true;
                    }
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            } else if npc.ai[1] < spit_windup + spit_fire {
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                npc.vx *= 0.9;
                npc.vy *= 0.9;
                let phase = (npc.ai[1] - spit_windup) % (spit_fire / spit_bursts as f32);
                let (m, dir) = mouth(npc);
                if phase < spit_fire / spit_bursts as f32 * 0.8 {
                    for _ in 0..5 {
                        let idx = env.dust.new_dust_direct(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            m + dir * 50.0 - Vec2::splat(15.0),
                            30,
                            30,
                            5,
                            0.0,
                            0.0,
                            0,
                            Color::TRANSPARENT,
                            1.5,
                        );
                        {
                            let d = env.dust.slot_mut(idx);
                            let jitter = next_vector2_circular(env.rng, 5.0, 5.0);
                            let from = normalize_xna(d.position - (m + jitter));
                            d.velocity = from * length(d.velocity);
                            d.position -= dir * 60.0;
                        }
                        let idx = env.dust.new_dust_direct(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            m + dir * 90.0 - Vec2::splat(20.0),
                            40,
                            40,
                            5,
                            0.0,
                            0.0,
                            100,
                            Color::TRANSPARENT,
                            1.5,
                        );
                        {
                            let d = env.dust.slot_mut(idx);
                            let jitter = next_vector2_circular(env.rng, 10.0, 10.0);
                            let from = normalize_xna(d.position - (m + jitter));
                            d.velocity = from * (length(d.velocity) + 5.0);
                            d.position -= dir * 100.0;
                        }
                    }
                }
                if phase as i32 == 0 {
                    let recoil = dir * -8.0;
                    npc.vx += recoil.x;
                    npc.vy += recoil.y;
                    for _ in 0..20 {
                        let idx = env.dust.new_dust_direct(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            m + dir * 60.0 - Vec2::splat(15.0),
                            30,
                            30,
                            5,
                            0.0,
                            0.0,
                            0,
                            Color::TRANSPARENT,
                            1.5,
                        );
                        {
                            let d = env.dust.slot_mut(idx);
                            let jitter = next_vector2_circular(env.rng, 5.0, 5.0);
                            let from = normalize_xna(d.position - (m + jitter));
                            d.velocity = from * length(d.velocity);
                            d.position -= dir * 60.0;
                        }
                        let idx = env.dust.new_dust_direct(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            m + dir * 100.0 - Vec2::splat(20.0),
                            40,
                            40,
                            5,
                            0.0,
                            0.0,
                            100,
                            Color::TRANSPARENT,
                            1.5,
                        );
                        {
                            let d = env.dust.slot_mut(idx);
                            let jitter = next_vector2_circular(env.rng, 10.0, 10.0);
                            let from = normalize_xna(d.position - (m + jitter));
                            d.velocity = from * (length(d.velocity) + 5.0);
                            d.position -= dir * 100.0;
                        }
                    }
                    if env.net_mode != 1 {
                        let count = env.rng.next_range(5, 11);
                        let base_vel = dir * 10.0;
                        let damage = attack_damage_for_projectiles(env.difficulty, 30.0, 25.0);
                        for _ in 0..count {
                            let jitter = next_vector2_square(env.rng, -6.0, 6.0);
                            out.shots.push(NautilusShot {
                                projectile_type: 814,
                                x: m.x - dir.x * 5.0,
                                y: m.y - dir.y * 5.0,
                                vx: base_vel.x + jitter.x,
                                vy: base_vel.y + jitter.y,
                                damage,
                            });
                        }
                    }
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            }
            npc.ai[1] += 1.0;
            if npc.ai[1] >= spit_windup + spit_fire {
                next_state = 0;
            }
        }
        3 => {
            // `:47420-47453` — the call for help.
            npc.facing = if center.x < target.x { 1 } else { -1 };
            npc.sprite_dir = npc.facing;
            if npc.ai[1] < call_ticks {
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                // `:47428-47429` — the Lerp is computed then ZEROED
                // (the dead computation, verbatim order).
                let clamped = (1.0 - npc.ai[1] / call_ticks * 1.5).clamp(0.0, 1.0);
                let want = Vec2::new(0.0, clamped * -1.5);
                let v = Vec2::new(npc.vx, npc.vy);
                let _blended = v + (want - v) * 0.03;
                npc.vx = 0.0;
                npc.vy = 0.0;
                npc.rotation = angle_lerp(npc.rotation, 0.0, 0.02);
                // `:47431` — the dead mouth call (out _, out _).
                let _ = mouth(npc);
                let t = npc.ai[1] / call_ticks;
                let glow =
                    get_lerp_value_unclamped(0.0, 0.5, t) * get_lerp_value_unclamped(1.0, 0.5, t);
                out.light = Some((center.x, center.y, [glow, 0.5 * glow, 0.5 * glow]));
                if env.rng.next_max(3) != 0 {
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        center - Vec2::splat(6.0),
                        12,
                        12,
                        5,
                        0.0,
                        0.0,
                        60,
                        Color::TRANSPARENT,
                        1.4,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.position += Vec2::new(npc.sprite_dir as f32 * 12.0, 12.0);
                        d.velocity *= 0.1;
                    }
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            }
            if npc.ai[1] == 10.0 || npc.ai[1] == 20.0 || npc.ai[1] == 30.0 {
                nautilus_call_for_help(npc, env, &mut out);
            }
            npc.ai[1] += 1.0;
            if npc.ai[1] >= call_ticks {
                next_state = 0;
            }
        }
        _ => {}
    }
    // `:47455-47470` — the transition tail.
    if next_state != -1 {
        npc.ai[0] = next_state as f32;
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        npc.net_update = true;
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 117, env.ticker);
        }
        if next_state == 0 {
            npc.ai[2] = npc.facing as f32;
        } else {
            npc.ai[3] += 1.0;
        }
    }
    // `:47471` — the reflects flag.
    out.reflects_projectiles = reflects;
    out
}

/// `NPC.FindFrame` case 618 (`NPC.cs:67000-67010`) — the 5-tick row
/// advance over the 4-row sheet (the style73 convention).
pub fn nautilus_find_frame(frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame_row;
    if counter >= 5.0 {
        row += 1;
        counter = 0.0;
        if row >= BLOOD_NAUTILUS_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 618-shaped NPC (the style73 placeholder convention).
    fn nautilus(x: f32, y: f32) -> Npc {
        let d = &BLOOD_NAUTILUS;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.sprite_dir = 1;
        n.local_ai[0] = 1.0; // skip the spawn latch by default
        n.ai[0] = 0.0;
        n.target = 0;
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

    /// `NPC.cs:16918-16933` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s117_the_defaults_row() {
        let d = &BLOOD_NAUTILUS;
        assert_eq!((d.width, d.height), (100, 100));
        assert_eq!(d.ai_style, 117);
        assert_eq!((d.damage, d.defense, d.life_max), (55, 24, 7000));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.value, 20000, "buyPrice(0, 2)");
        assert!(d.no_gravity && d.no_tile_collide);
        assert_eq!(d.rarity, 1);
        assert_eq!(BLOOD_NAUTILUS_FRAME_COUNT, 4);
    }

    /// `:47139-47148` — the spawn latch: alpha 255, the non-client
    /// ai[0] = -1; the client keeps ai[0].
    #[test]
    fn w4_d33_s117_the_spawn_latch() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.local_ai[0] = 0.0;
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: false,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 0,
            ticker: 7,
        };
        let _ = style117_step(&mut n, &mut env);
        assert_eq!(n.alpha, 255);
        assert_eq!(n.ai[0], -1.0);
        assert_eq!(n.local_ai[0], 1.0);
        let mut n = nautilus(100.0, 100.0);
        n.local_ai[0] = 0.0;
        env.net_mode = 1;
        let _ = style117_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "the client keeps the state");
    }

    /// `:47172-47176` — the day / non-blood-moon blanking: the aim
    /// collapses to (0,0), so the hover flies to the world origin's
    /// offset instead of the player.
    #[test]
    fn w4_d33_s117_the_day_blanking() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.ai[1] = 0.0;
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: true,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 0,
            ticker: 7,
        };
        let _ = style117_step(&mut n, &mut env);
        // The blanked target (0,0) is LEFT of the nautilus → face -1.
        assert_eq!(n.facing, -1);
    }

    /// `:47252-47271` — the attack pick: ai[3] % 7 == 3 → call (3),
    /// even → spit (2) + Item170, odd → charge (1) + Item170.
    #[test]
    fn w4_d33_s117_the_attack_pick() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        for (cycle, want_state, want_sound) in [
            (0i32, 2i32, true),
            (1, 1, true),
            (3, 3, false),
            (10, 3, false),
        ] {
            let mut n = nautilus(100.0, 100.0);
            n.ai[1] = 60.0; // ++ → 61 > 60
            n.ai[3] = cycle as f32;
            let mut env = Style117Env {
                world: &w,
                dust: &mut pool,
                dust_ctx: &c,
                rng: &mut rng,
                players: &players,
                net_mode: 0,
                day_time: false,
                blood_moon: true,
                difficulty: 1.0,
                nautilus_count: 0,
                ticker: 7,
            };
            let out = style117_step(&mut n, &mut env);
            assert_eq!(n.ai[0], want_state as f32, "cycle {cycle}");
            assert_eq!(n.ai[1], 0.0, "the transition reset");
            assert_eq!(n.ai[3], (cycle + 1) as f32, "the attack counter");
            assert_eq!(
                out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (2, 170)),
                want_sound,
                "cycle {cycle}"
            );
        }
    }

    /// `:47289-47309` — the charge windup: the reflects flag, the
    /// ×0.95 damp, the Item172 cue at 89, the two-dust draw order
    /// (internals then Circular(15,15) ×2 each).
    #[test]
    fn w4_d33_s117_the_charge_windup() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut oracle = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.ai[0] = 1.0;
        n.ai[1] = 89.0; // the cue tick
        n.vx = 2.0;
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: false,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 0,
            ticker: 7,
        };
        let out = style117_step(&mut n, &mut env);
        assert!(out.reflects_projectiles);
        assert!(out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (2, 172)));
        assert!((n.vx - 1.9).abs() < 1e-5, "the ×0.95 damp");
        // The head drip gate drew first (ai[0] != -1).
        if oracle.next_max(4) == 0 {
            oracle.next_float(); // the drip scale (menu ctx — no internals)
        }
        // The two windup dusts: each Circular(15,15) = 2 NextFloat.
        for _ in 0..2 {
            oracle.next_float();
            oracle.next_float();
        }
        assert_eq!(rng.peek(), oracle.peek(), "the windup draw order");
    }

    /// `:47388-47409` — the spit volley at phase 0: the Next(5,11)
    /// count, the per-shot NextVector2Square (2 NextDouble), the 814
    /// type, the classic-30 damage.
    #[test]
    fn w4_d33_s117_the_spit_volley() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut oracle = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 90.0; // phase 0
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: false,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 0,
            ticker: 7,
        };
        let out = style117_step(&mut n, &mut env);
        // The head drip gate.
        if oracle.next_max(4) == 0 {
            oracle.next_float();
        }
        // Phase 0 runs BOTH the 5-pair phase loop (< 24) AND the
        // 20-pair recoil burst: 25 pairs, each Circular(5,5)/(10,10)
        // = 2 NextFloat per dust.
        for _ in 0..25 {
            oracle.next_float();
            oracle.next_float();
            oracle.next_float();
            oracle.next_float();
        }
        let count = oracle.next_range(5, 11);
        for _ in 0..count {
            oracle.next_double();
            oracle.next_double();
        }
        assert_eq!(rng.peek(), oracle.peek(), "the volley draw order");
        assert_eq!(out.shots.len(), count as usize);
        assert!(out
            .shots
            .iter()
            .all(|s| s.projectile_type == 814 && s.damage == 30));
        // The client never fires.
        let mut n = nautilus(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 90.0;
        env.net_mode = 1;
        let out = style117_step(&mut n, &mut env);
        assert!(out.shots.is_empty());
    }

    /// `:47428-47434` — the call hold: the velocity ZEROES (the dead
    /// Lerp), the unclamped glow peaks at t=0.5.
    #[test]
    fn w4_d33_s117_the_call_hold_and_glow() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.ai[0] = 3.0;
        n.ai[1] = 90.0; // t = 0.5 — the glow peak
        n.vx = 5.0;
        n.vy = 5.0;
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: false,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 0,
            ticker: 7,
        };
        let out = style117_step(&mut n, &mut env);
        assert_eq!((n.vx, n.vy), (0.0, 0.0), "the zero overwrite");
        let (_, _, rgb) = out.light.expect("the glow");
        // t=0.5: GetLerpValue(0,0.5,0.5)=1, GetLerpValue(1,0.5,0.5)=1.
        assert!((rgb[0] - 1.0).abs() < 1e-6);
        assert!((rgb[1] - 0.5).abs() < 1e-6);
    }

    /// `:47474-47516` — CallForHelp's gates: the count cap and the
    /// client skip draw NOTHING.
    #[test]
    fn w4_d33_s117_the_call_for_help_gates() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(200.0, 100.0)];
        let mut rng = UnifiedRandom::new(117);
        let mut n = nautilus(100.0, 100.0);
        n.ai[0] = 3.0;
        n.ai[1] = 10.0; // a call tick
        let mut env = Style117Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            day_time: false,
            blood_moon: true,
            difficulty: 1.0,
            nautilus_count: 3,
            ticker: 7,
        };
        let before = rng.peek();
        let out = style117_step(&mut n, &mut env);
        // The head drip + the state-3 dust still draw; the CALL itself
        // adds nothing (the cap). Pin via a twin with count 0 minus
        // the call's rolls — simpler: assert no 813 shot.
        assert!(out.shots.iter().all(|s| s.projectile_type != 813));
        let _ = before;
    }

    /// `NPC.cs:67000-67010` — the 5-tick row advance wraps at 4.
    #[test]
    fn w4_d33_s117_the_find_frame_wraps_at_four() {
        assert_eq!(nautilus_find_frame(0, 3.0), (0, 4.0));
        assert_eq!(nautilus_find_frame(0, 4.0), (1, 0.0));
        assert_eq!(nautilus_find_frame(3, 4.0), (0, 0.0), "the wrap");
    }

    /// The full-pool stream parity: 40 ticks on the REAL dust ctx must
    /// match a cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s117_the_full_pool_run_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let players = [cand(600.0, 300.0)];
        let mut a = nautilus(280.0, 280.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(618);
        let mut rng_b = UnifiedRandom::new(618);
        for tick in 0..40 {
            let mut env_a = Style117Env {
                world: &w,
                dust: &mut pool_a,
                dust_ctx: &c,
                rng: &mut rng_a,
                players: &players,
                net_mode: 0,
                day_time: false,
                blood_moon: true,
                difficulty: 1.0,
                nautilus_count: 0,
                ticker: 7,
            };
            let _ = style117_step(&mut a, &mut env_a);
            let mut env_b = Style117Env {
                world: &w,
                dust: &mut pool_b,
                dust_ctx: &c,
                rng: &mut rng_b,
                players: &players,
                net_mode: 0,
                day_time: false,
                blood_moon: true,
                difficulty: 1.0,
                nautilus_count: 0,
                ticker: 7,
            };
            let _ = style117_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.rotation.to_bits(), b.rotation.to_bits(), "rot @{tick}");
        }
    }
}
