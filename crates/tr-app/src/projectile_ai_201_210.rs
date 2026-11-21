//! W4-d21 — projectile aiStyles 201-210, the missing machines (census:
//! `docs/research/26d-projectile-ai-201-210-census.md`). On `a725acb0` the
//! dispatch (`projectile.rs`'s `match data.ai_style`) carried ONLY 210
//! (`ai_yoyo_shot`, M4-PROJ-4b); 206 landed on the sibling a9 branch
//! (`8c4fded8`, `forbidden_minion_step` — verify-only here, NOT re-ported);
//! 207 is the C#'s EMPTY `AI_207_SnappingStone` (`Projectile.cs:37492-37494`)
//! and NO `SetDefaults` arm ever writes `aiStyle = 207` (the Snapping Stone
//! accessory's strike effects 1121/1131/1132 carry aiStyle 0 — the accessory
//! proc itself, `TryMakingSnappingStoneStrike` `:14283-14395`, is the
//! on-hit seam's follow-up, not an AI machine). This module ports the seven
//! MISSING styles: 201, 202, 203, 204, 205, 208, 209.
//!
//! C# sources read WHOLE (the golden rule):
//! - aiStyle 201: `AI_201_ThrownMelee` (`Projectile.cs:39954-39983`) + the
//!   `HandleMovement`-201 collision arm (`:19798-19802`: `ai[1] = 1`,
//!   velocity restored — the bounce-back that arms the fade) + the rows
//!   1082/1083 (`:10095-10114`).
//! - aiStyle 202: `AI_202_TorchGodHelper` (`:39912-39952`) + the 1084 row
//!   (`:10115-10124`) + `TorchID.Dust` (`ID/TorchID.cs:179-184`).
//! - aiStyle 203: `AI_203_StormLightning` (`:39641-39855`) +
//!   `AI_203_GetLightningColor`/`GetLightningColor` (`:39857-39878`) +
//!   `AI_203_TooFar` (`:39880-39883`) + `PackTargetIndex`/`UnpackTargetIndex`
//!   (`:39885-39910`) + `Colliding`'s 203 arm (`:15863-15877`) +
//!   `StormLightningCollisionCheck` (`:16112-16151`) + the `Kill()` bolt
//!   arms (`:70771-70854`) + the rows 1091/1117/1122 (`:10175-10185`,
//!   `:10427-10449`).
//! - aiStyle 204: `AI_204_Digtoise` (`:38463-38801`) + `DoPalAppearEffect`
//!   (`:38803-38837`) + the rows 1098/1123 (`:10241-10250`, `:10504-10513`)
//!   + `Collision.WalkDownSlope` is NOT this arm's (the 205's); the digtoise
//!   rides `correctSlopeCollision` (`:20447-20474`).
//! - aiStyle 205: `AI_205_RemoteControlCar` (`:38076-38461`) + the 1105 row
//!   (`:10294-10302`) + `Collision.WalkDownSlope` (`Collision.cs:1686-1789`)
//!   + `Tile.blockType` (`Tile.cs:202-214`) + the `Kill()` 1105 arm
//!   (`:70855-70901`).
//! - aiStyle 208: `AI_208_GlacierFangShot` (`:36903-37049`) + the 1115 row
//!   (`:10401-10412`) + `Colliding`'s 1115 arm (`:15397-15405`).
//! - aiStyle 209: `AI_209_ChlorophyteClaymoreBladeSlam` (`:69622-69631`) +
//!   the 1129 row (`:10580-10596`).
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the dust/gore BODIES are skipped (no draw layer wired here) and
//! the `Dust.NewDust*` internal rolls go with the dust pool (ANALYSIS
//! "Dust & gore", the `ai_style2`/`projectile_ai_misc` precedent). The
//! `Gore.NewGore` internal kicks DO stay — `Next(10, 31)` on Y first, then
//! `Next(-20, 21)` on X (`Gore.cs:1327-1328`, the G3 standard). Per style:
//! - 201: ONE `NextFloat` on the first tick (`:39958`, the random spin).
//! - 202: the 4-dust loop's per-dust `Next(3)` noGravity pick (`:39938`).
//! - 203: the acquire scans draw nothing; the windup/dust gates draw per
//!   the cited lines (kept inline in [`ai_storm_lightning`]).
//! - 204: the appear-effect dust loop draws nothing explicit (the
//!   `NextVector2Circular` pair per particle at `num3 >= 0.9`, `:38834`);
//!   the dig dust's per-particle `Next(3)` scale pick (`:38713`).
//! - 205: the jump's 2 dust (no explicit draws), the drive dust's two
//!   `Next(num6)` gates + `NextVector2Circular` pairs (`:38239-38246`),
//!   the landing dust's `NextFloat` gate + `NextVector2CircularEdge`
//!   (`:38439-38447`).
//! - 208: the release jitter `NextVector2Circular(1,1)` (`:36968`), the
//!   release burst's 15×(`Next(2)` + `NextVector2Circular(4,2)` +
//!   `NextFloat`) (`:36984-36994`), the trail's `Next(num11)` gate +
//!   `NextVector2Circular(4,2)` (`:37030-37048`).
//! - 209: ZERO draws (the whole arm is the box resize + the counter).
//!
//! Movement is the dispatch's per-style seam, run AFTER the AI exactly as
//! the C# runs `HandleMovement` after `AI()` (`:17015`):
//! - 201/209: `tileCollide` gates the `kill_step` default (`:19849-19853`);
//!   the 201's own collision arm (`:19798-19802`) restores the velocity and
//!   arms the fade instead of killing.
//! - 202/203/208(fly)/209: `tileCollide = false` — the free integrate
//!   (`UpdatePosition` `:20445`).
//! - 204/205: `correctSlopeCollision` — the `TileCollision` slide then the
//!   `UpdatePosition` slope pass (`:20447-20474`); the 205's AI moves the
//!   position itself (`WalkDownSlope` `:38416-38418`).
//!
//! Deliberate folds (ANALYSIS "The projectile aiStyles 201-210 (W4-d21)"):
//! - `gravDir == -1` (the 205's flipped-gravity input XOR, `:38098`): the
//!   port is upright-only (`gravDir = 1`), so `flag4` is false.
//! - `player.petting` (the 204's state-5 joy, `:38508-38526`): no ported
//!   petting layer — the state is unreachable, recorded not ported as dead
//!   code (the golf `gravDir` precedent).
//! - `Main.Difficulty >= Legendary` (the 203's `num3`/`num4`, `:39646-
//!   39647`): the port is single-difficulty (below Legendary), so 14/5.
//! - the `LightningGenerator` bolt paths + `MultiPointHitbox` (the 203's
//!   `Kill()` damage, `:70790-70840`): UNFOLDED (W4-d21 folds 3+4) — the
//!   census proved the generator is NOT render-only (the tile truncation
//!   re-centers the bolt and picks `localAI[0]`; the positions ARE the
//!   damage hitbox). Ported verbatim in `lightning_generator.rs`; the
//!   `CalcRotations` draw-angle tail stays the render-only skip.
//! - `SoundEngine.PlayTrackedLoopedSound` (the 205's engine loop,
//!   `:38256-38267`/`:38288-38299`): no tracked-loop layer — the
//!   `localAI[1]`/`localAI[2]` state machine runs, the sound body is the
//!   skip (the volume/pitch math feeds only the skipped sound).
//! - `Main.DroneCameraTracker.Track` (`:38125`), `CheckSectionsInCaseOwner
//!   IsWatching` (`:38460`), `RemoteClient.CheckSection`/`ActiveSections`
//!   (`:38497-38505`): MP/camera bookkeeping, no ported layer.
//! - `NetMessage.SendTileSquare`/`SendData` — the `net_update` flag rides
//!   the pool's convention.
//! - the dust/gore bodies, `Lighting.AddLight`, the `ParticleOrchestrator`
//!   requests — no ported layers; their EXPLICIT `Main.rand` gates stay.
//!
//! CODE-ONLY MODE (W4-d21): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); verified by re-reading the
//! cited C# blocks and hand arithmetic.

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

use crate::npc::Npc;
use crate::projectile::{
    move_towards, npc_center, remap, safe_normalize, wrap_angle, Projectile,
};

/// `AI_209_ChlorophyteClaymoreBladeSlam` (`Projectile.cs:69622-69631`) —
/// the Chlorophyte Claymore's blade slam (1129). The whole machine: the
/// box resize from `ai[1]`/`ai[2]` (the swing's reach the shoot path
/// stamps), the `ai[0]++` age and the >20 kill. ZERO `Main.rand` draws.
/// `tileCollide = false` (the row), so the dispatch's free integrate runs.
pub(crate) fn ai_claymore_blade_slam(p: &mut Projectile) {
    // `:69624-69625` — `width = (int)ai[1]`, `height = (int)ai[2]`.
    p.w = p.ai[1] as i32 as f32;
    p.h = p.ai[2] as i32 as f32;
    // `:69626` — the age counter.
    p.ai[0] += 1.0;
    // `:69627-69630` — the >20 kill (the port's death convention zeroes
    // time_left too, the `kill()` shape).
    if p.ai[0] > 20.0 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
}

/// `AI_201_ThrownMelee` (`Projectile.cs:39954-39983`) — the thrown melee
/// weapons (1082 Antlion Claw, 1083 Stylist scissors). The first-tick
/// random spin, the 15-tick straight flight then the 0.95 X drag + 0.4 Y
/// gravity; the `ai[1] == 1` return state (armed by the HandleMovement-201
/// collision, [`thrown_melee_collision`]) fades the weapon out.
/// ONE `NextFloat` on the first tick (`:39958`).
pub(crate) fn ai_thrown_melee(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:39956-39960` — the first-tick random rotation latch.
    if p.local_ai[0] == 0.0 {
        // `:39958` — `(float)((double)NextFloat() * Math.PI * 2.0)`: the
        // float is widened to double, multiplied by the double 2π, then
        // narrowed back (the C#'s own cast order).
        p.rot = (rng.next_float() as f64 * std::f64::consts::PI * 2.0) as f32;
        p.local_ai[0] = 1.0;
    }
    // `:39961-39974` — the flying state.
    if p.ai[1] == 0.0 {
        // `:39963` — `rotation += direction * 0.45`.
        p.rot += p.direction as f32 * 0.45;
        // `:39964-39972` — past 15 ticks the drag + gravity, else the count.
        if p.ai[0] > 15.0 {
            p.vel.x *= 0.95;
            p.vel.y += 0.4;
        } else {
            p.ai[0] += 1.0;
        }
        return;
    }
    // `:39975-39982` — the return state: no collision, no damage, the 0.6
    // velocity damp and the +7 alpha fade that kills at 255.
    p.tile_collide = false;
    p.damage = 0;
    p.vel *= 0.6;
    p.alpha += 7;
    if p.alpha >= 255 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
}

/// `HandleMovement`'s aiStyle-201 collision arm (`Projectile.cs:19798-
/// 19802`): on a blocked move `ai[1] = 1` and the velocity is RESTORED to
/// `lastVelocity` (the weapon bounces back, then the AI's return state
/// fades it). The port's blocked-move probe is the per-axis destination-box
/// test the `kill_step`/boomerang arms use (the swept-`TileCollision`
/// stand-in, ANALYSIS). Returns true when the move was blocked (the arm
/// fired); the caller then integrates the restored velocity.
pub(crate) fn thrown_melee_collision(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last_velocity = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x || hit_y {
        // `:19800-19801` — arm the return, restore the velocity.
        p.ai[1] = 1.0;
        p.vel = last_velocity;
        // `UpdatePosition` (`:19858` → `:20445`) integrates the restored
        // velocity on the bounce tick (the boomerang precedent).
        p.pos += p.vel;
    } else {
        p.pos.x = nx;
        p.pos.y = ny;
    }
}

/// `AI_202_TorchGodHelper` (`Projectile.cs:39912-39952`) — the Torch God's
/// favour helper (1084): flies to the torch tile `ai[1]`/`ai[2]`, snaps the
/// torch's frame to the favour style `ai[0]` on arrival and dies. The
/// 4-dust trail's per-dust `Next(3)` noGravity pick stays on the stream
/// (`:39938`); the dust bodies + the Item8 cue are the layer skips.
/// `tileCollide = false` + `extraUpdates = 2` (the row) — the free integrate
/// runs per sub-step.
pub(crate) fn ai_torch_god_helper(p: &mut Projectile, world: &mut World, rng: &mut dyn DropRng) {
    // `:39914-39916` — the target torch tile → world coordinates (the
    // tile's centre, `Point.ToWorldCoordinates()` = `(x*16+8, y*16+8)`).
    let tx = p.ai[1] as i32;
    let ty = p.ai[2] as i32;
    let target = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:39917-39927` — the arrival: snap, reframe the torch, kill.
    if target.distance(center) < 16.0 {
        p.pos = target - Vec2::new(p.w * 0.5, p.h * 0.5);
        p.vel = Vec2::ZERO;
        // `:39921-39925` — the owner-gated torch reframe (tile type 4, the
        // torch): `frameY = (short)(ai[0] * 22)`. The `SendTileSquare` is
        // the net_update convention.
        if p.owner == 0 && world.in_bounds(tx, ty) {
            if let Some(t) = world.tile_mut(tx, ty) {
                if t.active() && t.kind == 4 {
                    t.frame_y = (p.ai[0] * 22.0) as i16;
                    p.net_update = true;
                }
            }
        }
        // `:39926` — `Kill()` (no return in the C# — the trail below runs
        // on the dead slot, a nothing-reads write, the flamethrower shape).
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:39928-39932` — the one-shot Item8 cue latch (the sound body is the
    // skip; the localAI[0] latch state stays).
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
    }
    // `:39933-39951` — the 4-dust trail: `TorchID.Dust[ai[0]]` picks the
    // dust type (the body is the skip), each dust rolls one `Next(3)`
    // noGravity pick (`:39938`). The `num3 == 66` disco arm writes only the
    // skipped dust colour.
    let _dust_type = TORCH_DUST[(p.ai[0] as usize).min(TORCH_DUST.len() - 1)];
    for _ in 0..4 {
        let _ = rng.next_range(0, 3); // the noGravity pick (`:39938`)
    }
}

/// `TorchID.Dust` (`ID/TorchID.cs:179-184`) — the 24 torch styles' dust ids.
const TORCH_DUST: [i32; 24] = [
    6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294, 295, 296, 297,
    298, 307, 310,
];

/// `AI_207_SnappingStone` (`Projectile.cs:37492-37494`) — the C# body is
/// EMPTY and no `SetDefaults` arm writes aiStyle 207 (the Snapping Stone
/// accessory's strike effects carry aiStyle 0). The dispatch arm is the
/// faithful no-op; this function documents the census verdict so a future
/// row cannot silently mis-route. ZERO draws, ZERO state.
pub(crate) fn ai_snapping_stone(_p: &mut Projectile) {
    // `:37492-37494` — `{ }`.
}

/// `AI_208_GlacierFangShot` (`Projectile.cs:36903-37049`) — the Glacier
/// Fang's shot (1115): a held-channel charge (the `ai[1] == 0` state) that
/// hangs off the owner's hand and aims at the cursor, then a release
/// (`ai[1] = 1`) that scales the damage/knockback/penetration by the charge
/// and flies with a charge-scaled gravity. The owner reads ride the mutable
/// player channel (the `ai_last_prism`/`charged_blaster_step` seam); the aim
/// is the latched `channel_target` (the port's `Main.MouseWorld` stand-in).
///
/// RNG: the release jitter `NextVector2Circular(1,1)` (`:36968`, two
/// floats), the release burst's 15×(`Next(2)` + taken-branch
/// `NextVector2Circular(4,2)` pair + `NextFloat`) (`:36984-36994`), the
/// trail's per-tick `Next(num11)` gate + taken-branch
/// `NextVector2Circular(4,2)` pair (`:37030-37048`). The dust bodies are the
/// skip; the explicit gates stay.
pub(crate) fn ai_glacier_fang_shot(
    p: &mut Projectile,
    who_am_i: usize,
    world: &World,
    owner: Option<(f32, f32)>,
    mut player: Option<&mut crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:36906` — the pre-increment charge snapshot (the trail gate reads it).
    let num = p.ai[0];
    let channel = player.as_deref().is_some_and(|pl| pl.channel);
    // `:36907-36955` — the charge state.
    if channel && p.ai[1] == 0.0 {
        let num2 = std::f32::consts::FRAC_PI_2;
        // `:36910-36914` — the hand anchor + the position snap.
        let vector = match player.as_deref() {
            Some(pl) => {
                let (mx, my) = pl.mounted_center();
                let mc = Vec2::new(mx, my);
                pl.rotated_relative_point(mc, 0)
            }
            None => owner.map(|(x, y)| Vec2::new(x, y)).unwrap_or(Vec2::ZERO),
        };
        // `GetSlowMagicUseTime(10)` (`Player.cs:47019-47026`): 25 while the
        // slow cast is armed, else 10.
        let slow_magic_use_time = match player.as_deref() {
            Some(pl) if pl.slow_magic_use => {
                (crate::regen::SLOW_MAGIC_MULTIPLIER * 10.0) as i32
            }
            _ => 10,
        };
        let num3 = 0.0f32;
        p.pos = vector - Vec2::new(p.w * 0.5, p.h * 0.5);
        p.rot = p.vel.y.atan2(p.vel.x) + num2;
        p.direction = if p.vel.x >= 0.0 { 1 } else { -1 };
        p.sprite_direction = p.direction;
        p.time_left = 300;
        p.tile_collide = false;
        // `:36920-36935` — the owner-gated aim refresh (every 3rd charge
        // tick): the cursor direction at the charge-scaled speed.
        let no_items = player.as_deref().is_some_and(|pl| pl.no_items);
        let cced = player.as_deref().is_some_and(|pl| pl.cced());
        if p.owner == 0 && channel && !no_items && !cced && (p.ai[0] as i32) % 3 == 0 {
            let num4 = remap(p.ai[0], 0.0, 60.0, 4.0, 15.0);
            let target = player
                .as_deref()
                .map(|pl| pl.channel_target)
                .unwrap_or(Vec2::ZERO);
            let mut vec = target - vector;
            vec = safe_normalize(vec);
            if vec.is_nan() || vec == Vec2::ZERO {
                let dir = player.as_deref().map_or(1, |pl| pl.facing);
                vec = Vec2::new(dir as f32, 0.0);
            }
            vec *= num4;
            if vec.x != p.vel.x || vec.y != p.vel.y {
                p.net_update = true;
            }
            p.vel = vec;
        }
        // `:36936` — the hang offset: back along the velocity then out along
        // its heading by `34 + ai[0]/10`.
        let heading = if p.vel == Vec2::ZERO || p.vel.is_nan() {
            Vec2::new(0.0, 1.0)
        } else {
            p.vel.normalize()
        };
        p.pos += -p.vel + heading * (34.0 + p.ai[0] / 10.0);
        // `:36937-36945` — the every-20-tick charge cue + the mana pay
        // (<=60). The sound body is the skip; the pay rides check_mana.
        if (p.ai[0] as i32) % 20 == 0 && p.sound_delay == 0 && p.ai[0] != 0.0 {
            p.sound_delay = 2;
            if (p.ai[0] as i32) <= 60 {
                if let Some(pl) = player.as_deref_mut() {
                    let mana = crate::registry::item(pl.channel_item).map_or(0, |it| it.mana);
                    let _ = crate::regen::check_mana(
                        &mut pl.mana,
                        &mut pl.slow_magic_use,
                        mana,
                        pl.mana_cost,
                        true,
                    );
                }
            }
        }
        // `:36946-36949` — the owner writes: ChangeDir, heldProj,
        // SetDummyItemTime, itemRotation.
        if let Some(pl) = player.as_deref_mut() {
            pl.facing = p.direction;
            pl.held_proj = who_am_i as i32;
            pl.item_rotation = wrap_angle(
                (p.vel.y * p.direction as f32).atan2(p.vel.x * p.direction as f32) + num3,
            );
        }
        // `SetDummyItemTime(slowMagicUseTime)` rides the app's swing timers
        // in the live pass; the slot arm records the intent on the player's
        // held_proj (the golf PlayerWrites precedent — the dispatch applies
        // the timer writes). Here the charge keeps the channel alive.
        let _ = slow_magic_use_time;
        // `:36950` — `ai[0] += GetSlowMagicUseRate()` (0.4 slow, else 1).
        let rate = match player.as_deref() {
            Some(pl) if pl.slow_magic_use => 1.0 / crate::regen::SLOW_MAGIC_MULTIPLIER,
            _ => 1.0,
        };
        p.ai[0] += rate;
        // `:36951-36954` — full charge drops the channel (owner only).
        if p.ai[0] >= 60.0 && p.owner == 0 {
            if let Some(pl) = player.as_deref_mut() {
                pl.channel = false;
            }
        }
    } else {
        // `:36956-37008` — the release state.
        if p.ai[1] != 1.0 {
            p.ai[1] = 1.0;
            p.net_update = true;
            // `:36963-36968` — the charge fraction + the release jitter.
            let mut num5 = remap(p.ai[0], 0.0, 60.0, 0.0, 1.0);
            if p.ai[0] < 10.0 {
                num5 = 0.0;
            }
            // `NextVector2Circular(1,1)` — two floats (the unit angle + the
            // radius, `Utils.cs:1266-1269`).
            let jitter = next_vector2_circular(rng, 1.0, 1.0);
            p.vel += jitter * (1.0 - num5);
            // `:36969-36979` — the owner-gated damage/knockback/penetrate
            // scale + the release Damage(). The Damage() rides the pool's
            // post-AI walk (the dispatch runs damage_pve every sub-step);
            // the penetrate is opened to num6 here so the walk lands.
            if p.owner == 0 {
                let num6 = remap(num5, 0.0, 1.0, 1.0, 3.0) as i32;
                p.damage = (p.damage as f32 * remap(num5, 0.0, 1.0, 1.0, 4.0)) as i32;
                p.knock_back *= remap(num5, 0.0, 1.0, 1.0, 4.0);
                p.max_penetrate = num6;
                p.penetrate = num6;
            }
            // `:36980-36983` — the blocked line to the owner kills the shot.
            let player_center = player
                .as_deref()
                .map(|pl| {
                    let (x, y) = pl.center();
                    Vec2::new(x, y)
                })
                .unwrap_or(Vec2::ZERO);
            let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            if !crate::projectile::can_hit(
                world,
                player_center.x,
                player_center.y,
                0.0,
                0.0,
                center.x,
                center.y,
                0.0,
                0.0,
            ) {
                p.kill_time_left = p.time_left;
                p.active = false;
                p.time_left = 0;
            }
            // `:36984-36994` — the 15-dust release burst: each rolls
            // `Next(2)`; the taken branch rolls the `NextVector2Circular(4,2)`
            // pair + one `NextFloat` (the velocity blend).
            for _ in 0..15 {
                if rng.next_range(0, 2) == 0 {
                    let _ = next_vector2_circular(rng, 4.0, 2.0);
                    let _ = rng.next_float();
                }
            }
        }
        // `:36996-37007` — the flight: extraUpdates 1 (2 past full charge),
        // tileCollide on, the charge-scaled gravity + the 16 clamp.
        p.extra_updates = 1;
        if p.ai[0] >= 60.0 {
            p.extra_updates = 2;
        }
        p.tile_collide = true;
        let num8 = remap(p.ai[0], 0.0, 45.0, 0.15, 0.025);
        p.vel.y += num8;
        if p.vel.y > 16.0 {
            p.vel.y = 16.0;
        }
    }
    // `:37009-37021` — the frame ladder + the scale/rotation tail.
    if p.ai[0] >= 20.0 {
        p.frame = 1;
    }
    if p.ai[0] >= 40.0 {
        p.frame = 2;
    }
    p.scale = 1.0;
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    // `:37022` — the light cast is the skip.
    // `:37023-37048` — the trail dust: the charge-tick gate widens the
    // count to 20 / the odds to 1; each dust rolls `Next(num11)` and the
    // taken branch rolls the `NextVector2Circular(4,2)` pair.
    let (mut num10, mut num11) = (1, 3);
    if (num as i32) != (p.ai[0] as i32)
        && (p.ai[0] as i32) % 20 == 0
        && p.ai[1] == 0.0
        && p.ai[0] < 50.0
    {
        num10 = 20;
        num11 = 1;
    }
    for _ in 0..num10 {
        if rng.next_range(0, num11) != 0 {
            continue;
        }
        if p.ai[1] == 0.0 {
            let _ = next_vector2_circular(rng, 4.0, 2.0);
        }
    }
}

/// `Utils.NextVector2Circular(hw, hh)` (`Utils.cs:1266-1269`):
/// `NextVector2Unit() * (hw, hh) * NextFloat()` — the unit-angle float
/// (`:1261-1264`, one `NextFloat` → `rotationRange * NextFloat()` →
/// `ToRotationVector2`) then the radius float. TWO draws, in order.
fn next_vector2_circular(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::TAU * rng.next_float();
    let unit = Vec2::new(angle.cos(), angle.sin());
    unit * Vec2::new(hw, hh) * rng.next_float()
}

/// `Projectile.GetLightningColor(type)` (`Projectile.cs:39862-39878`) — the
/// storm-lightning family's colour: 1117 blue (60,120,255), 1122 red
/// (255,60,60), 1091 green (80,220,80) ONLY under both lightning micro-seeds
/// (`moreLightning && noLightning`), else the default teal (80,220,220).
/// The colour feeds the skipped light/dust/particles; the port keeps the
/// helper for the census + the future draw seam.
pub(crate) fn get_lightning_color(kind: u16, more_lightning: bool, no_lightning: bool) -> [u8; 3] {
    match kind {
        1117 => [60, 120, 255],
        1122 => [255, 60, 60],
        1091 if more_lightning && no_lightning => [80, 220, 80],
        _ => [80, 220, 220],
    }
}

/// `AI_203_TooFar` (`Projectile.cs:39880-39883`): the target's centre is
/// more than 600 px from the bolt's centre.
fn storm_lightning_too_far(center: Vec2, target_center: Vec2) -> bool {
    center.distance(target_center) > 600.0
}

/// `Projectile.PackTargetIndex` (`Projectile.cs:39885-39896`): the NPC index
/// rides as-is, the player index as `-2 - playerIndex`, none as -1.
fn pack_target_index(npc_index: i32, player_index: i32) -> f32 {
    if npc_index != -1 {
        return npc_index as f32;
    }
    if player_index != -1 {
        return (-2 - player_index) as f32;
    }
    -1.0
}

/// `Projectile.UnpackTargetIndex` (`Projectile.cs:39898-39910`): `ai[0] >= 0`
/// is an NPC index, `<= -2` is `-ai[0] - 2` as a player index, else none.
fn unpack_target_index(ai_value: f32) -> (i32, i32) {
    let mut npc_index = -1;
    let mut player_index = -1;
    if ai_value >= 0.0 {
        npc_index = ai_value as i32;
    }
    if ai_value <= -2.0 {
        player_index = -(ai_value as i32) - 2;
    }
    (npc_index, player_index)
}

/// `AI_203_StormLightning` (`Projectile.cs:39641-39855`) — the storm
/// lightning (1091 trap, 1117 Lightning Strike weapon, 1122 Arc Surge).
/// The bolt locks a target (NPC or player) packed in `ai[0]`, flies its
/// centre toward the target at 32 px/tick and dies at the age cap (240, or
/// 30 for the 1117). The acquire scans, the validity re-checks and the
/// MoveTowards are the state; the windup/dust/particle gates' explicit
/// `Main.rand` draws stay (the bodies are the skip).
///
/// The single-player port maps `Main.player[0]` to the local player; the
/// 255-player scan folds to that one slot (the empress precedent). The
/// `Kill()` bolt-path damage (`:70771-70854`) rides the pool's death hook
/// ([`storm_lightning_kill`]) — the `LightningGenerator`/`MultiPointHitbox`
/// are ported for real (W4-d21 folds 3+4, `lightning_generator.rs`).
pub(crate) fn ai_storm_lightning(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:39643-39653` — the per-kind constants. The difficulty is below
    // Legendary in the port (single-difficulty), so num3 = 14, num4 = 5.
    // `flag` (the 1117's chaseable-only gate) is `p.kind == 1117`.
    let (mut num, mut num2) = (240.0f32, 32.0f32);
    let num3 = 14.0f32;
    let num4 = 5.0f32;
    if p.kind == 1117 {
        num = 30.0;
        num2 = 12.0;
    }
    let flag = p.kind == 1117;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:39654-39676` — unpack + re-validate the locked target.
    let (mut npc_index, mut player_index) = unpack_target_index(p.ai[0]);
    let had_target = npc_index != -1 || player_index != -1;
    if npc_index != -1 {
        let valid = npcs.get(npc_index as usize).is_some_and(|n| {
            n.hp > 0 && !n.dont_take_damage && !storm_lightning_too_far(center, npc_center(n))
        });
        if !valid {
            npc_index = -1;
        }
    }
    if player_index != -1 {
        // The single-player fold: only index 0 maps to the local player.
        let valid = player_index == 0
            && player.is_some_and(|pl| {
                !pl.dead
                    && !storm_lightning_too_far(center, Vec2::from(pl.center()))
                    && pl.dash_delay >= 0
            });
        if !valid {
            player_index = -1;
        }
    }
    if had_target && npc_index == -1 && player_index == -1 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:39678-39726` — the acquire scan (no lock yet).
    if npc_index == -1 && player_index == -1 {
        // `:39681-39693` — the first NPC within num2 of the bolt's centre
        // (the hitbox-closest-point distance), chaseable for the 1117.
        let mut found = false;
        for (i, n) in npcs.iter().enumerate() {
            if found {
                break;
            }
            if n.hp <= 0 || n.dont_take_damage {
                continue;
            }
            let (nw, nh) = n.size();
            let closest = closest_point_in_rect(center, Vec2::new(n.x, n.y), nw, nh);
            if center.distance(closest) > num2 {
                continue;
            }
            if flag && !n.can_be_chased_by() {
                continue;
            }
            found = true;
            npc_index = i as i32;
        }
        // `:39694-39720` — the player scan (not the 1117): the first live
        // player within num3, gated by the localAI[2] skip counter.
        let flag2 = p.kind != 1117;
        let flag3 = p.local_ai[2] >= num4;
        if flag2 && !found {
            if let Some(pl) = player {
                if !pl.dead {
                    let pc = Vec2::from(pl.center());
                    let closest = closest_point_in_rect(
                        center,
                        Vec2::new(pl.x, pl.y),
                        crate::player::WIDTH,
                        pl.height,
                    );
                    if center.distance(closest) <= num3 {
                        if !flag3 {
                            p.local_ai[2] += 1.0;
                        } else {
                            player_index = 0;
                            found = true;
                        }
                    }
                }
            }
        }
        if found {
            p.ai[0] = pack_target_index(npc_index, player_index);
            p.net_update = true;
        }
    }
    // `:39727-39738` — the target centre/velocity the MoveTowards seeks.
    let mut target_center = center;
    if npc_index != -1 {
        if let Some(n) = npcs.get(npc_index as usize) {
            target_center = npc_center(n);
        }
    }
    if player_index != -1 {
        if let Some(pl) = player {
            target_center = Vec2::from(pl.center());
        }
    }
    // `:39740-39773` — the 1117's no-target cursor steer (owner only): the
    // velocity points at the cursor, clamped to 16, zeroed into a solid
    // tile. The dust pair is the skip (no explicit draws — NewDustPerfect).
    if p.kind == 1117 && npc_index == -1 && player_index == -1 && p.owner == 0 {
        if let Some(pl) = player {
            let mut v = pl.channel_target - center;
            if v.length() > 16.0 {
                v *= 16.0 / v.length();
            }
            let probe = center + v;
            let tx = (probe.x / 16.0) as i32;
            let ty = (probe.y / 16.0) as i32;
            if solid_tile_no_platforms(world, tx, ty) {
                v = Vec2::ZERO;
            }
            p.vel = v;
            p.net_update = true;
        }
    }
    // `:39774-39779` — the MoveTowards + the age kill.
    let new_center = move_towards(center, target_center, 32.0);
    p.pos = new_center - Vec2::new(p.w * 0.5, p.h * 0.5);
    p.ai[1] += 1.0;
    if p.ai[1] >= num {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:39780-39783` — the light ramp (the skip). num7/num8 feed the gates.
    let num7 = p.ai[1] / num;
    let num8 = remap(num7, 0.0, 0.5, 0.0, 1.0);
    // `:39784-39796` — the windup sound + 5 particles at the 15% tick (the
    // sound/particle bodies are the skip; no explicit rand draws).
    // `:39797-39805` — the per-tick windup particle gate: ONE NextFloat.
    if rng.next_float() <= num7 * 0.2 {
        // the particle body is the skip.
    }
    // `:39806-39820` — the 3-dust windup loop: each rolls Next(4); the taken
    // branch rolls NextFloat (the num8² gate), then the dust's
    // RotatedByRandom pair + NextFloatDirection + Next(3) noGravity pick.
    for _ in 0..3 {
        if rng.next_range(0, 4) == 0 && rng.next_float() <= num8 * num8 * 0.13 {
            let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
            let _ = rng.next_float(); // NextFloatDirection
            let _ = rng.next_range(0, 3); // noGravity pick
        }
    }
    // `:39821-39836` — the 2-dust release loop: each rolls Next(4) +
    // NextFloat (the num10 gate); the taken branch rolls Next(3).
    let num10 = remap(num7, 0.7, 1.0, 0.0, 1.0);
    for _ in 0..2 {
        if rng.next_range(0, 4) == 0 && rng.next_float() <= num10 * 0.2 {
            let _ = rng.next_range(0, 3); // noGravity pick
        }
    }
    // `:39837-39854` — the 2-dust spark loop: each rolls Next(4) + NextFloat
    // (the 0.2 gate); the taken branch rolls NextFloat (the velocity scale)
    // + the NextVector2Circular(3,2) pair.
    for _ in 0..2 {
        if rng.next_range(0, 4) == 0 && rng.next_float() <= 0.2 {
            let _ = rng.next_float(); // the velocity scale
            let _ = next_vector2_circular(rng, 3.0, 2.0);
        }
    }
}

/// `Rectangle.ClosestPointInRect` (XNA) — the point clamped into the rect.
fn closest_point_in_rect(point: Vec2, rect_pos: Vec2, w: f32, h: f32) -> Vec2 {
    Vec2::new(
        point.x.clamp(rect_pos.x, rect_pos.x + w),
        point.y.clamp(rect_pos.y, rect_pos.y + h),
    )
}

/// `WorldGen.SolidTileNoPlatforms` (`WorldGen.cs:70196-70218`): an
/// out-of-world / missing tile reads solid; an active, non-platform,
/// solid-or-solidTop, non-actuated tile reads solid.
fn solid_tile_no_platforms(world: &World, x: i32, y: i32) -> bool {
    if !world.in_bounds(x, y) {
        return true;
    }
    match world.tile(x, y) {
        None => true,
        Some(t) => {
            t.active()
                && !tr_core::tiledata::SET_PLATFORMS.has(t.kind)
                && (tr_core::tiledata::TILE_SOLID.has(t.kind)
                    || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind))
                && !t.inactive()
        }
    }
}

/// `Tile.blockType()` (`Tile.cs:202-214`): 0 = full block, 1 = half brick,
/// 2-5 = the four slopes. The 205's ground-tilt reads it.
fn block_type(t: &tr_core::tile::Tile) -> u8 {
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

/// `AI_205_RemoteControlCar` (`Projectile.cs:38076-38461`) — the RC car
/// (1105): the owner's input bits pack into `ai[0]`, the car drives on the
/// ground (the slope-tilt + the left/right accel + the jump) or flies on a
/// wall (the up/down thrust + the left/right turn) while the "fly" bit is
/// set. The `localAI[0]` fly-time counter, the `localAI[1]` engine-speed
/// lerp and the `localAI[2]` tracked-sound slot are the state; the sound
/// bodies are the skip (no tracked-loop layer, ANALYSIS).
///
/// The single-player port folds `Main.myPlayer == owner` to `owner == 0`;
/// `gravDir == -1` is unreachable (upright-only). The input bits read the
/// live player's control flags (the `LocalInputCache` stand-in).
pub(crate) fn ai_remote_control_car(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:38078-38083` — the dead-owner kill flag.
    let mut kill = player.is_some_and(|pl| pl.dead);
    // `:38084-38126` — the owner-gated input pack.
    if p.owner == 0 {
        // `:38086-38090` — the space-height kill.
        if p.pos.y - p.h <= 640.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `:38091-38094` — the held-item gate (5738 = the RC Car item). The
        // C# reads `Main.player[owner]` unconditionally; an absent player is
        // the inactive slot (held type 0 → the gate fails → kill).
        let held = player.map_or(0, |pl| pl.held_item_kind);
        if held != 5738 {
            kill = true;
        }
        // `:38095-38124` — the input bit pack. gravDir is 1 (flag4 false).
        // An absent player's control flags are all false (the inactive slot).
        let bits = p.ai[0] as u8;
        let flag2 = bits & (1 << 4) != 0; // the jump latch
        let mut flag3 = bits & (1 << 5) != 0; // the fly mode
        let mut out: u8 = 0;
        let (cl, cr, cu, cd, cj) = match player {
            Some(pl) => (
                pl.control_left,
                pl.control_right,
                pl.control_up,
                pl.control_down,
                pl.control_jump,
            ),
            None => (false, false, false, false, false),
        };
        if cl {
            out |= 1 << 0;
        }
        if cr {
            out |= 1 << 1;
        }
        if cu {
            out |= 1 << 2;
        }
        if cd {
            out |= 1 << 3;
        }
        if cj {
            out |= 1 << 4;
        }
        // `:38106-38117` — the jump press toggles fly (the sound + 2 dust
        // are the skip; no explicit draws — NewDustDirect).
        if !flag2 && cj && p.vel.y != 0.0 {
            flag3 = !flag3;
        }
        if flag3 {
            out |= 1 << 5;
        }
        let num = out as f32;
        if p.ai[0] != num {
            p.ai[0] = num;
            p.net_update = true;
        }
        // `:38125` — DroneCameraTracker.Track is the camera skip.
    }
    // `:38127-38131` — the dead-owner / wrong-item kill.
    if kill {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:38132-38140` — unpack the input bits.
    let bits = p.ai[0] as u8;
    p.rot = p.ai[1];
    let flag5 = bits & (1 << 0) != 0; // left
    let flag6 = bits & (1 << 1) != 0; // right
    let flag7 = bits & (1 << 2) != 0; // up
    let flag8 = bits & (1 << 3) != 0; // down
    let flag9 = bits & (1 << 4) != 0; // jump
    let flag10 = bits & (1 << 5) != 0; // fly
    let flag11 = flag5 | flag6 | flag7 | flag8; // any direction
    // `:38141-38145` — the fly-time counter.
    p.local_ai[0] += 1.0;
    if !flag7 && !flag8 {
        p.local_ai[0] = 0.0;
    }
    // `:38146-38148` — the wall probes (oldPosition + centre).
    let old_center = p.old_position + Vec2::new(p.w * 0.5, p.h * 0.5);
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let wall_at = |v: Vec2| -> bool {
        let tx = (v.x / 16.0) as i32;
        let ty = (v.y / 16.0) as i32;
        world.tile(tx, ty).is_some_and(|t| t.wall > 0)
    };
    let flag12 = wall_at(old_center) & flag10;
    let flag13 = wall_at(center) & flag10;
    let flag14 = !flag13;
    // `:38149-38272` — the wall-flight branch.
    if flag13 {
        let num2 = if flag7 | flag8 { 0.037699115 } else { 0.07539823 };
        let num3 = (flag7 as i32 - flag8 as i32) as f32 * 0.275;
        p.rot += (flag6 as i32 - flag5 as i32) as f32 * num2;
        p.vel += Vec2::new(p.rot.cos(), p.rot.sin()) * num3;
        p.vel *= 0.96;
        if p.frame < 4 {
            p.vel *= 0.5;
            p.frame = 4;
            if p.direction == -1 {
                p.rot += std::f32::consts::PI;
            }
            p.direction = 1;
            p.sprite_direction = 1;
        }
        // `:38170-38215` — the speed-scaled frame machine.
        let num5 = p.vel.length();
        let mut num4 = frame_delay(num5);
        if flag11 {
            num4 = 1;
        }
        p.frame_counter += 1;
        if p.frame_counter >= num4 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 7 {
                p.frame = 4;
            }
        }
        p.sprite_direction = 1;
        p.direction = 1;
        // `:38217-38247` — the exhaust dust gates.
        let num6 = if num3 != 0.0 {
            3
        } else if num5 > 0.2 {
            6
        } else if flag11 {
            2
        } else {
            0
        };
        if num6 != 0 {
            if rng.next_range(0, num6) == 0 {
                let _ = next_vector2_circular(rng, 0.3, 0.3);
            }
            if rng.next_range(0, num6) == 0 {
                let _ = next_vector2_circular(rng, 0.3, 0.3);
            }
        }
        // `:38248-38267` — the engine-speed lerp + the tracked sound (the
        // sound body is the skip; the localAI[1] state stays).
        let num9 = if flag11 { 100.0 } else { 0.0 };
        p.local_ai[1] = math_helper_lerp(p.local_ai[1], num9, 0.2);
        // `:38268-38271` — the zero-Y nudge.
        if p.vel.y == 0.0 {
            p.vel.y = 0.0001;
        }
    }
    // `:38273-38452` — the ground branch.
    if flag14 {
        let flag17 = p.vel.y == 0.0; // on the ground
        // `:38275-38299` — the engine-speed lerp + the tracked sound (skip).
        let num12 = if flag11 {
            if flag17 {
                100.0
            } else {
                10.0
            }
        } else {
            0.0
        };
        p.local_ai[1] = math_helper_lerp(p.local_ai[1], num12, 0.2);
        // `:38300-38344` — the ground tilt / air turn.
        let mut num15 = 0.3f32;
        if flag17 {
            // `:38303-38314` — the slope tilt from the two foot probes.
            let mut value3 = 0.0f32;
            let foot_l = p.pos + Vec2::new(0.0, p.h + 0.02);
            let foot_r = p.pos + Vec2::new(p.w, p.h + 0.02);
            let tl = world.tile((foot_l.x / 16.0) as i32, (foot_l.y / 16.0) as i32);
            let tr = world.tile((foot_r.x / 16.0) as i32, (foot_r.y / 16.0) as i32);
            if tl.is_some_and(|t| block_type(t) == 2) {
                value3 = std::f32::consts::FRAC_PI_4;
            }
            if tr.is_some_and(|t| block_type(t) == 3) {
                value3 = -std::f32::consts::FRAC_PI_4;
            }
            p.rot = math_helper_lerp(wrap_angle(p.rot), value3, 0.3);
            p.vel.x *= 0.97;
            // `:38316-38332` — the facing from the input.
            let num16 = if flag5 {
                -1
            } else if flag6 {
                1
            } else {
                0
            };
            if num16 != 0 {
                if p.sprite_direction != num16 {
                    p.local_ai[1] = 0.0;
                }
                p.sprite_direction = num16;
                p.direction = num16;
            }
            // `:38333-38337` — the jump.
            if flag9 {
                p.vel += Vec2::new((p.rot - std::f32::consts::FRAC_PI_2).cos(), (p.rot - std::f32::consts::FRAC_PI_2).sin()) * 6.0;
                num15 = 0.0;
            }
        } else if !flag10 {
            // `:38339-38344` — the air turn.
            let num17 = std::f32::consts::PI / 15.0;
            let num17 = remap(p.local_ai[0], 0.0, 10.0, num17 * 0.1, num17);
            p.rot += (flag7 as i32 - flag8 as i32) as f32 * num17 * p.sprite_direction as f32;
        }
        // `:38345-38349` — the wall-exit boost.
        if flag12 && !flag13 {
            p.vel *= 1.5;
            p.rot = wrap_angle(p.rot);
        }
        // `:38350-38359` — the frame reset on landing.
        let flag18 = (!flag17 && p.frame >= 4) & flag10;
        if (flag17 || !flag10) && p.frame >= 4 {
            if !flag17 && p.vel.x < 0.0 {
                p.direction = -1;
                p.sprite_direction = -1;
                p.rot += std::f32::consts::PI;
            }
            p.frame = 0;
        }
        // `:38360-38405` — the |X|-speed frame machine.
        let num18 = p.vel.x.abs();
        let mut num19 = frame_delay(num18);
        if flag11 {
            num19 = 1;
        }
        if !flag18 {
            p.frame_counter += 1;
            if p.frame_counter >= num19 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
        }
        // `:38406-38415` — the gravity + the left/right accel.
        p.vel.y += num15;
        let num20 = if flag17 { 0.3 } else { 0.1 };
        if flag5 {
            p.vel.x -= num20;
        }
        if flag6 {
            p.vel.x += num20;
        }
        // `:38416-38418` — WalkDownSlope moves the position + velocity.
        let wd = walk_down_slope(world, p.pos, p.vel, p.w as i32, p.h as i32, num15);
        p.pos = wd.0;
        p.vel = wd.1;
        // `:38419-38451` — the landing dust (the ground + any-direction
        // gate): the 3-tile drop probe, then 2×(NextFloat gate + the
        // NextVector2CircularEdge draw).
        if flag17 & flag11 {
            let mut num21 = 0.0f32;
            let num23 = 16.0 * 3.0;
            for j in 0..3 {
                let step = Vec2::new(0.0, 16.0);
                let tc = crate::collision::tile_collision(
                    world,
                    [p.pos.x + step.x * j as f32, p.pos.y + step.y * j as f32],
                    [step.x, step.y],
                    p.w,
                    p.h,
                    false,
                    false,
                );
                num21 += tc.vel[1];
                if tc.vel != [step.x, step.y] {
                    break;
                }
            }
            if num21 != num23 {
                for _ in 0..2 {
                    if rng.next_float() < 0.66 {
                        continue;
                    }
                    let _ = next_vector2_circular_edge(rng, 2.5, 1.0);
                }
            }
        }
    }
    // `:38453-38460` — the rotation latch, the 16-speed clamp, the light
    // (skip), the timeLeft refresh and the section check (skip).
    p.ai[1] = p.rot;
    if p.vel.length() > 16.0 {
        p.vel *= 16.0 / p.vel.length();
    }
    p.time_left = 2;
}

/// The 205's speed→frame-delay ladder (`:38170-38207` / `:38360-38397`):
/// 999999 below 0.02, then 16/12/10/8/6/3/2/1 at the cited thresholds.
fn frame_delay(speed: f32) -> i32 {
    let mut d = 999999;
    if speed >= 0.02 {
        d = 16;
    }
    if speed >= 0.2 {
        d = 12;
    }
    if speed >= 0.5 {
        d = 10;
    }
    if speed >= 0.8 {
        d = 8;
    }
    if speed >= 1.5 {
        d = 6;
    }
    if speed >= 3.0 {
        d = 3;
    }
    if speed >= 6.0 {
        d = 2;
    }
    if speed >= 9.0 {
        d = 1;
    }
    d
}

/// `MathHelper.Lerp` (XNA): `value1 + (value2 - value1) * amount`. The
/// 205's engine-speed lerp + the ground tilt read it (NOT `Utils.GetLerpValue`,
/// which is the clamped 0..1 ramp the `remap` helper wraps).
fn math_helper_lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 + (value2 - value1) * amount
}

/// `Utils.NextVector2CircularEdge(hw, hh)` (`Utils.cs:1271-1274`):
/// `NextVector2Unit() * (hw, hh)` — ONE float (the unit angle), no radius.
fn next_vector2_circular_edge(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::TAU * rng.next_float();
    Vec2::new(angle.cos(), angle.sin()) * Vec2::new(hw, hh)
}

/// `Collision.WalkDownSlope` (`Collision.cs:1686-1789`): when the velocity's
/// Y equals the gravity, scan the two rows below the box for the nearest
/// slope tile and add |X| to Y when the box rides it (the down-slope walk).
/// Returns the (position, velocity) pair — the position is unchanged (the
/// C# returns `Position` verbatim); only Y of the velocity may grow.
fn walk_down_slope(
    world: &World,
    position: Vec2,
    velocity: Vec2,
    width: i32,
    height: i32,
    gravity: f32,
) -> (Vec2, Vec2) {
    if velocity.y != gravity {
        return (position, velocity);
    }
    let mut vel = velocity;
    let value = (position.x / 16.0) as i32;
    let value2 = ((position.x + width as f32) / 16.0) as i32;
    let value3 = ((position.y + height as f32 + 4.0) / 16.0) as i32;
    let value = value.clamp(0, world.width - 1);
    let value2 = value2.clamp(0, world.width - 1);
    let value3 = value3.clamp(0, (world.height - 2 - 40).max(0));
    let mut num = (value3 + 3) as f32 * 16.0;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let num4 = if velocity.x < 0.0 { 2 } else { 1 };
    for i in value..=value2 {
        for j in value3..=value3 + 1 {
            let Some(t) = world.tile(i, j) else { continue };
            if !t.active()
                || (!tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind))
            {
                continue;
            }
            let mut num5 = j as f32 * 16.0;
            if t.half_brick() {
                num5 += 8.0;
            }
            // The 16×16 probe rect at (i*16, j*16-17) vs the box.
            let probe = tr_core::whip::Rect::new(i * 16, j * 16 - 17, 16, 16);
            let box_r = tr_core::whip::Rect::new(
                position.x as i32,
                position.y as i32,
                width,
                height,
            );
            if !probe.intersects(box_r) || !(num5 <= num) {
                continue;
            }
            if num == num5 {
                if t.slope() == 0 {
                    continue;
                }
                if num2 != -1 && num3 != -1 {
                    if let Some(prev) = world.tile(num2, num3) {
                        if prev.slope() != 0 {
                            if t.slope() == num4 {
                                num = num5;
                                num2 = i;
                                num3 = j;
                            }
                            continue;
                        }
                    }
                }
                num = num5;
                num2 = i;
                num3 = j;
            } else {
                num = num5;
                num2 = i;
                num3 = j;
            }
        }
    }
    if num2 != -1 && num3 != -1 {
        if let Some(t) = world.tile(num2, num3) {
            let num8 = t.slope();
            if num8 > 0 {
                let vx = num2 as f32 * 16.0;
                let vy = num3 as f32 * 16.0;
                match num8 {
                    2 => {
                        let num9 = vx + 16.0 - (position.x + width as f32);
                        if position.y + height as f32 >= vy + num9 && velocity.x < 0.0 {
                            vel.y += velocity.x.abs();
                        }
                    }
                    1 => {
                        let num9 = position.x - vx;
                        if position.y + height as f32 >= vy + num9 && velocity.x > 0.0 {
                            vel.y += velocity.x.abs();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    (position, vel)
}

/// `AI_204_Digtoise` (`Projectile.cs:38463-38801`) — the Digtoise / Trusty
/// Digtoise (1098/1123): the mining turtle pet. The `ai[0]` states: 0 the
/// idle appear (180 ticks), 1 the dig-down, 2/3 the dig-left/right, 4 the
/// walk, 5 the petted joy (unreachable — no petting layer, ANALYSIS). The
/// dig breaks the tiles ahead via the owner's pick (`PickTile`, the
/// `pick_tile_hit` seam rides the App drain); the appear effect fades the
/// alpha in. The `correctSlopeCollision` row rides the dispatch's slope
/// pass.
///
/// The single-player port folds `Main.myPlayer == owner` to `owner == 0`.
/// The petting state-5 (`:38508-38526`) is unreachable (no `player.petting`
/// field) — recorded, not ported as dead code. The dig's `PickTile` calls
/// ride the [`DigtoiseDig`] intent out to the App drain (the pool cannot
/// borrow the hit-tile book).
pub(crate) fn ai_digtoise(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) -> Vec<(i32, i32)> {
    let mut digs = Vec::new();
    // `:38465-38469` — the out-of-world kill (20-tile fluff).
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let cx = (center.x / 16.0) as i32;
    let cy = (center.y / 16.0) as i32;
    if !in_world_fluff(world, cx, cy, 20) {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return digs;
    }
    // `:38470-38475` — the state flags.
    let mut flag = p.ai[0] == 0.0; // idle
    let flag2 = p.ai[0] == 1.0; // dig down
    let flag3 = p.ai[0] == 2.0; // dig right
    let flag4 = p.ai[0] == 3.0; // dig left
    let flag5 = p.ai[0] == 4.0; // walk
    let mut flag6 = p.ai[0] == 5.0; // joy
    // `:38477-38481` — the gravity + clamp.
    p.vel.y += 2.0 / 15.0;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    p.time_left = 2;
    // `:38483-38487` — the dead / no-building owner kill.
    let pl = match player {
        Some(pl) => pl,
        None => {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return digs;
        }
    };
    if pl.dead {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return digs;
    }
    // `:38488-38491` — the 60-tick net cadence (the update_count stand-in
    // is the pool's; the slot arm uses ai[1]-independent whoAmI — the
    // dispatch passes the cadence through net_update on the pool tick).
    // `:38492-38507` — the section-load probe (the MP skip; the single-
    // player IsTileLoaded folds true, so no velocity zero).
    let point = {
        let bottom = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h);
        let probe = bottom + Vec2::new(0.0, -15.0);
        ((probe.x / 16.0) as i32, (probe.y / 16.0) as i32)
    };
    // `:38508-38526` — the petting joy (unreachable: no petting layer).
    // The else-if flag6 reset (`:38520-38526`) keeps a stray state-5 from
    // sticking: it returns to idle.
    if flag6 {
        flag = true;
        flag6 = false;
        p.ai[0] = 0.0;
        p.ai[1] = 0.0;
    }
    // `:38527-38537` — the idle sound cues (the sound bodies are the skip;
    // the soundDelay counter state stays).
    if flag {
        p.sound_delay -= 1;
        // the -20 / -1 cue gates fire the (skipped) sounds.
    }
    // `:38538-38584` — the appear / walk / joy branch.
    if flag | flag5 | flag6 {
        // `DoPalAppearEffect(player, withParticles: true)` (`:38803-38837`).
        do_pal_appear_effect(p, pl, rng);
        // `:38541-38553` — the held-item gate: the matching staff keeps the
        // pet alive; a mismatch (owner only) kills it.
        let mut flag7 = false;
        if p.kind == 1123 && pl.held_item_kind == 6174 {
            flag7 = true;
        }
        if p.kind == 1098 && pl.held_item_kind == 5667 {
            flag7 = true;
        }
        if !flag7 && p.owner == 0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
        p.vel.x *= 0.7;
        // `:38555-38558` — the idle facing (first tick only).
        let pc = Vec2::from(pl.center());
        if p.ai[1] == 0.0 && flag {
            let dir = if center.x > pc.x { 1 } else { -1 };
            p.sprite_direction = dir;
            p.direction = dir;
        }
        p.ai[1] += 1.0;
        // `:38560-38565` — idle → dig-down at 180.
        if flag && p.ai[1] >= 180.0 {
            p.ai[1] = 0.0;
            p.ai[0] = 1.0;
            p.net_update = true;
        }
        // `:38566-38569` — walk → kill at 600 (owner only).
        if flag5 && p.ai[1] >= 600.0 && p.owner == 0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
        // `:38570-38583` — the appear frame machine (count down to 0).
        if p.frame > 2 {
            p.frame = 2;
        }
        let num = if p.frame == 1 { 60 } else { 4 };
        p.frame_counter += 1;
        if p.frame_counter >= num {
            p.frame_counter = 0;
            p.frame -= 1;
            if p.frame < 0 {
                p.frame = 0;
            }
        }
    }
    // `:38585-38589` — the dig/walk gate.
    let mut flag8 = flag2 | flag4 | flag3;
    if !flag8 {
        return digs;
    }
    p.alpha = 0;
    // `:38591-38608` — the dig direction.
    let mut num2 = 0i32; // horizontal
    let mut num3 = 0i32; // vertical (down)
    if flag2 {
        num3 = 1;
    }
    if flag4 {
        num2 = -1;
    }
    if flag3 {
        num2 = 1;
    }
    if num2 != 0 {
        p.sprite_direction = num2;
        p.direction = num2;
    }
    // `:38609-38618` — the accel past the 40-tick windup.
    let num4 = 40.0f32;
    if p.ai[1] > num4 {
        p.vel.x += (num2 * 8) as f32 / 60.0;
        let num5 = 8.0;
        if p.vel.x.abs() >= num5 {
            p.vel.x = if p.vel.x >= 0.0 { num5 } else { -num5 };
        }
    }
    // `:38619-38632` — the edge probe: no solid ahead → walk state.
    if num2 != 0 && p.ai[1] > num4 {
        let bottom = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h);
        let probe = bottom
            + Vec2::new((p.w * num2 as f32) / 2.0 + (num2 * 2) as f32, 2.0);
        let point2 = ((probe.x / 16.0) as i32, (probe.y / 16.0) as i32);
        let point3 = (point2.0 + num2, point2.1);
        let point4 = (point2.0 + num2, point2.1);
        if !solid_tile3(world, point2.0, point2.1)
            && !solid_tile3(world, point3.0, point3.1)
            && !solid_tile3(world, point4.0, point4.1)
        {
            flag8 = false;
            p.ai[1] = 0.0;
            p.ai[0] = 4.0;
            p.vel.x *= 0.5;
            p.net_update = true;
        }
    }
    // `:38633-38655` — the wall clamp.
    if flag8 && num2 != 0 {
        let num6 = point.0 + num2 * 2;
        let mut flag9 = false;
        for k in point.1 - 2..=point.1 {
            if solid_tile3(world, num6, k) {
                flag9 = true;
            }
        }
        if flag9 {
            if num2 > 0 {
                p.vel.x = p.vel.x.min(num6 as f32 * 16.0 - (p.pos.x + p.w));
            } else {
                p.vel.x = p.vel.x.max(num6 as f32 * 16.0 + 16.0 - p.pos.x);
            }
        }
    }
    // `:38656-38724` — the dig animation + dust past the windup.
    if flag8 && p.ai[1] > num4 {
        if p.frame < 2 {
            p.frame = 2;
            p.local_ai[1] = 0.0;
            p.frame_counter = 0;
        }
        let mut num7 = 4 - (p.local_ai[1] / 4.0) as i32;
        if num7 < 1 {
            num7 = 1;
        }
        let num8 = if p.frame == 2 { 5 } else { num7 };
        p.frame_counter += 1;
        if p.frame_counter >= num8 {
            p.local_ai[1] += 1.0;
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 10 {
                p.frame = 3;
            }
        }
        // `:38679-38683` — the dig sound cooldown (the body is the skip).
        if p.sound_delay <= 0 {
            p.sound_delay = 10;
        }
        // `:38684-38706` — the solid-ahead probe (down or horizontal).
        let mut flag10 = false;
        if num3 != 0 {
            let j2 = point.1 + 1;
            for l in point.0 - 1..=point.0 + 1 {
                if solid_tile3(world, l, j2) {
                    flag10 = true;
                }
            }
        }
        if num2 != 0 {
            let i2 = point.0 + num2 * 2;
            for m in point.1 - 2..=point.1 {
                if solid_tile3(world, i2, m) {
                    flag10 = true;
                }
            }
        }
        // `:38707` — the light cast (skip).
        // `:38708-38723` — the 3-dust dig burst: each rolls Next(3) (the
        // scale/fadeIn pick).
        if flag10 {
            for _ in 0..3 {
                let _ = rng.next_range(0, 3);
            }
        }
    }
    // `:38725-38729` — the windup count.
    if flag8 && p.ai[1] <= num4 {
        p.ai[1] += 1.0;
        p.frame = 0;
    }
    // `:38730-38733` — the dig-tick gate.
    if !flag8 || !(p.ai[1] > num4) {
        return digs;
    }
    p.ai[1] += 1.0;
    if !(p.ai[1] >= num4 + 30.0) {
        return digs;
    }
    // `:38739-38743` — the dig cadence reset (the 1123 digs slower).
    p.ai[1] = num4 + 10.0;
    if p.kind == 1123 {
        p.ai[1] = num4 + 20.0;
    }
    if p.owner != 0 {
        return digs;
    }
    // `:38748-38758` — the pick power + the damage-base (the 1123 reads the
    // best pickaxe and uses base 36). The slot arm cannot see the inventory;
    // the App drain resolves `GetBestPickaxe` and applies the
    // `PickTile_DetermineDamage` gate + the `dealDamageAsIfBaseNumberIs`
    // clamp (the `pick_tile_hit` seam). The cells ride out in the C#'s scan
    // order (down 3-wide first, then horizontal 3-tall).
    // `:38759-38794` — the dig cells: down (3-wide) or horizontal (3-tall).
    if num3 != 0 {
        let num12 = point.1 + 1;
        for num13 in point.0 - 1..=point.0 + 1 {
            if solid_tile3(world, num13, num12) {
                digs.push((num13, num12));
            }
        }
    }
    if num2 != 0 {
        let num15 = point.0 + num2 * 2;
        for num16 in point.1 - 2..=point.1 {
            if solid_tile3(world, num15, num16) {
                digs.push((num15, num16));
            }
        }
    }
    // `:38795-38800` — a partially-blocked dig (num10 < num11) returns to
    // walk. The drain counts the actual breakable cells (the
    // PickTile_DetermineDamage > 0 gate) and applies the walk transition
    // (ai[1] = 0, ai[0] = 4, netUpdate) when not every solid cell broke.
    digs
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world_fluff(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.SolidTile3` (`WorldGen.cs:70465-70485`): in-world (1-tile
/// fluff), active, solid, non-solidTop.
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    if !in_world_fluff(world, x, y, 1) {
        return false;
    }
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
    })
}

/// `DoPalAppearEffect` (`Projectile.cs:38803-38837`): the alpha fade-in
/// (−15/tick, floor 0) + the Hermite-arc dust trail (the dust bodies are the
/// skip; the per-particle `NextVector2Circular(3,3)` at `num3 >= 0.9` stays).
fn do_pal_appear_effect(p: &mut Projectile, _master: &crate::player::Player, rng: &mut dyn DropRng) {
    if p.alpha <= 0 {
        return;
    }
    let num = p.alpha;
    p.alpha -= 15;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    let mut num2 = 1.0 - num as f32 / 255.0;
    let mut num3 = 1.0 - p.alpha as f32 / 255.0;
    num2 *= 2.0;
    num3 *= 2.0;
    if !(num2 < 1.0) {
        return;
    }
    // `:38828-38836` — the 0..1 step-0.1 dust loop (11 particles, the f32
    // accumulation is the C#'s own iteration count). Each particle at
    // num3 >= 0.9 rolls the NextVector2Circular(3,3) pair.
    let mut num4 = 0.0f32;
    while num4 <= 1.0 {
        if num3 >= 0.9 {
            let _ = next_vector2_circular(rng, 3.0, 3.0);
        }
        num4 += 0.1;
    }
}

/// `Kill()`'s storm-lightning bolt arms (`Projectile.cs:70771-70854`) —
/// the 1122 Arc Surge (`:70771-70798`) and the 1091/1117 (`:70799-70854`).
/// W4-d21 folds 3+4 UNFOLDED: the `LightningGenerator` bolt path is
/// generated for real ([`crate::lightning_generator`]) and its positions
/// ride out as the `MultiPointHitbox` the pool's boxed `Damage()` reads
/// (the census proved the generator is NOT render-only — the tile
/// truncation re-centers the 1091/1117 and picks `localAI[0]`).
///
/// `arc_surge_start` is the 1122's `player.RotatedRelativePoint(pos)`
/// hand anchor (`:70775-70788` — the caller resolves it off the owner
/// snapshot; `None` = no owner link, the bolt starts at the centre).
/// The particle/orchestrator bodies are the skip. The 1091/1117's leafy-
/// tree-top greek-fire spawn (`:70846-70853`) rides the returned spawn
/// list.
///
/// Returns `(hitbox, spawns)`: the bolt's `MultiPointHitbox` (the boxed
/// `Damage()` seam reads it through `damage_pve_boxed`'s 203 arm) and the
/// greek-fire spawns `(kind, x, y, vx, vy)` (the 400-402 trio,
/// `timeLeft /= 3` applied by the caller). The liquid self-hurt
/// (`:70836-70839`, the 1117's `localAI[0] == 3`) rides the pool's
/// `SelfHurtPlayers` seam at the caller.
pub(crate) fn storm_lightning_kill(
    p: &mut Projectile,
    world: &World,
    arc_surge_start: Option<Vec2>,
    rng: &mut dyn DropRng,
) -> (crate::lightning_generator::MultiPointHitbox, Vec<(u16, f32, f32, f32, f32)>) {
    use crate::lightning_generator::{LightningGenerator, MultiPointHitbox};
    let mut spawns = Vec::new();
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    if p.kind == 1122 {
        // `:70788-70797` — the Arc Surge: the hand→centre bolt, then
        // localAI[0] = 2, the penetration open, the bolt-path Damage().
        // `GetArcSurgeWeaponGenerator(vector2, center)` seeds the config
        // off the hand distance; `GenerateMainBoltPath((uint)ai[2],
        // center, center - vector2)` walks it. No owner link → the hand
        // IS the centre (the zero-length bolt, the documented fold).
        let start = arc_surge_start.unwrap_or(center);
        let bolt = LightningGenerator::arc_surge_weapon(start, center)
            .generate_main_bolt_path(p.ai[2] as u32, center, Some(center - start), world);
        p.local_ai[0] = 2.0;
        p.max_penetrate = -1;
        p.penetrate = -1;
        // `:70794` — `_lightningLastHitChainPos = null`: the chain anchor
        // resets; the 1122 never reads it (the liquid arm is 1091-only),
        // so the reset is the caller's local-chain clear (no field).
        return (MultiPointHitbox::new((16, 16), bolt.positions), spawns);
    }
    // `:70813-70835` — the 1091/1117 bolt: StormLightning vs
    // LightningStrikeWeapon, the tile truncation re-centers the projectile
    // (`flag6 = !collidedWithTile`; a collided bolt moves `base.Center` to
    // its LAST position, `:70819-70823`), then localAI[0] = 1 (no tile hit)
    // / 2 (tile hit) / 3 (the centre tile carries liquid).
    let generator = if p.kind == 1117 {
        LightningGenerator::lightning_strike_weapon()
    } else {
        LightningGenerator::storm_lightning()
    };
    let bolt = generator.generate_main_bolt_path(p.ai[2] as u32, center, None, world);
    let flag6 = !bolt.collided_with_tile;
    if !flag6 {
        // `:70822` — the re-centre on the bolt's last position.
        let last = *bolt.positions.last().expect("the bolt has >= 1 position");
        p.pos = last - Vec2::new(p.w * 0.5, p.h * 0.5);
    }
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let tx = (center.x / 16.0) as i32;
    let ty = (center.y / 16.0) as i32;
    let tile = world.tile(tx, ty);
    let liquid = tile.is_some_and(|t| t.liquid != 0);
    p.local_ai[0] = if liquid {
        3.0
    } else if flag6 {
        1.0
    } else {
        2.0
    };
    if liquid {
        // `:70829` — SpawnLightningLiquidElectrificationParticles
        // (`:78535-78559`): the 39 radial PlotLine walks; every liquid
        // cell plots `Next(4)` (the particle/light bodies are the skip,
        // the draw stays — the G3 explicit-gate rule).
        spawn_lightning_liquid_electrification_draws(p, world, rng);
    }
    p.max_penetrate = -1;
    p.penetrate = -1;
    // `:70836-70839` — the 1117's liquid self-hurt rides the pool's
    // SelfHurtPlayers seam (the caller reads localAI[0] == 3).
    // `:70841-70853` — the leafy-tree-top greek-fire spawn (NOT the
    // 1117): Next(2,6) fires, each NextVector2Circular(5,3) + Next(3).
    let flag7 = p.kind != 1117;
    if flag7 && p.owner == 0 && tile.is_some_and(is_tile_a_leafy_tree_top) {
        let count = rng.next_range(2, 6);
        for _ in 0..count {
            let v = next_vector2_circular(rng, 5.0, 3.0);
            let kind = 400 + rng.next_range(0, 3) as u16;
            spawns.push((kind, center.x, center.y, v.x, v.y));
        }
    }
    (MultiPointHitbox::new((16, 16), bolt.positions), spawns)
}

/// `SpawnLightningLiquidElectrificationParticles` (`Projectile.cs:78535-
/// 78559`): 39 radial `Utils.PlotLine` walks (i = 1..39, the endpoint at
/// `centre + UnitX.RotatedBy(PI * i / 20) * 500`); each plotted cell that
/// is in-world AND carries liquid rolls `Next(4)` (the particle request +
/// `Lighting.AddLight` bodies are the skip — no ported layers; the
/// explicit draw stays on the `UpdateProjectiles` stream). The plot
/// returns false on a dry/out-of-world cell, aborting that walk.
fn spawn_lightning_liquid_electrification_draws(p: &Projectile, world: &World, rng: &mut dyn DropRng) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let (x0, y0) = ((center.x as i32) / 16, (center.y as i32) / 16);
    for i in 1..40 {
        let angle = std::f32::consts::PI * i as f32 / 20.0;
        let end = center + crate::bow_launch::rotated_by(Vec2::X, angle as f64) * 500.0;
        let (x1, y1) = ((end.x as i32) / 16, (end.y as i32) / 16);
        // `Utils.PlotLine(p0, p1, plot)` — jump defaults true (`:2233`).
        plot_line(x0, y0, x1, y1, true, &mut |x, y| {
            // `:78545-78548` — the in-world + liquid gate.
            if !world.in_bounds(x, y) {
                return false;
            }
            let Some(t) = world.tile(x, y) else {
                return false;
            };
            if t.liquid == 0 {
                return false;
            }
            // `:78549` — the Next(4) gate (the bodies are the skip).
            let _ = rng.next_range(0, 4);
            true
        });
    }
}

/// `Utils.PlotLine` (`Utils.cs:2238-2292`) — the Bresenham walk with the
/// `jump: false` double-plot on the minor-axis step; `plot` returns false
/// to abort (the `TileActionAttempt` convention). The `npc_ai_town` twin
/// is private; this copy serves the lightning walks (the third-copy
/// precedent: the town module's own is byte-identical). `pub(crate)`:
/// the pool's `storm_lightning_collision_check` shares it.
pub(crate) fn plot_line(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    jump: bool,
    plot: &mut dyn FnMut(i32, i32) -> bool,
) -> bool {
    if x0 == x1 && y0 == y1 {
        return plot(x0, y0);
    }
    let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
    let flag = (y1 - y0).abs() > (x1 - x0).abs();
    if flag {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    let num = (x1 - x0).abs();
    let num2 = (y1 - y0).abs();
    let mut num3 = num / 2;
    let mut num4 = y0;
    let num5 = if x0 < x1 { 1 } else { -1 };
    let num6 = if y0 < y1 { 1 } else { -1 };
    let mut i = x0;
    while i != x1 {
        if flag {
            if !plot(num4, i) {
                return false;
            }
        } else if !plot(i, num4) {
            return false;
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        if !jump {
            if flag {
                if !plot(num4, i) {
                    return false;
                }
            } else if !plot(i, num4) {
                return false;
            }
        }
        num3 += num;
        i += num5;
    }
    true
}

/// `WorldGen.IsTileALeafyTreeTop` (`WorldGen.cs:29679-29693`): an active
/// `GetsCheckedForLeaves` tile whose frame marks a leafy top (323 frameX>=88,
/// or frameX 22 + frameY 198..242).
fn is_tile_a_leafy_tree_top(t: &tr_core::tile::Tile) -> bool {
    if t.active() && tr_core::tiledata::SET_GETS_CHECKED_FOR_LEAVES.has(t.kind) {
        if t.kind == 323 && t.frame_x >= 88 {
            return true;
        }
        if t.frame_x == 22 && (198..=242).contains(&t.frame_y) {
            return true;
        }
    }
    false
}

/// `Kill()`'s digtoise dust arm (`Projectile.cs:70749-70757`): the 1098/1123
/// death spawns 50 dust (the bodies are the skip; no explicit rand draws —
/// `NewDustDirect`). Draw-free; the census keeps the arm for the seam.
pub(crate) fn digtoise_kill_draws(_p: &Projectile, _rng: &mut dyn DropRng) {
    // `:70751-70756` — 50× NewDustDirect (no explicit Main.rand draws).
}

/// `Kill()`'s RC-car explosion arm (`Projectile.cs:70855-70901`) — the 1105
/// (and the 1020 Terrarian-style car, unrowed). The Item62 cue + the dust
/// bursts are the skip; the 4-corner gore loop's explicit draws stay:
/// each corner rolls `Next(5)`; the taken branch rolls `Next(61,64)` (the
/// gore type) + the `NewGore` kicks `Next(10,31)`/`Next(-20,21)`
/// (`Gore.cs:1327-1328`).
pub(crate) fn remote_control_car_kill_draws(p: &Projectile, rng: &mut dyn DropRng) {
    if p.kind != 1105 {
        return;
    }
    // `:70865-70887` — the Item62 cue + the 15/5/3 dust bursts (no explicit
    // draws — NewDustDirect).
    // `:70888-70900` — the 4-corner gore loop.
    for _n in [-1i32, 1] {
        for _num10 in [-1i32, 1] {
            if rng.next_range(0, 5) == 0 {
                let _ = rng.next_range(61, 64); // the gore type
                let _ = rng.next_range(10, 31); // NewGore's Y kick
                let _ = rng.next_range(-20, 21); // NewGore's X kick
            }
        }
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, ANTLION_CLAW, STYLIST_SCISSORS, TORCH_GOD_HELPER, STORM_LIGHTNING, LIGHTNING_STRIKE_SHOT, ARC_SURGE, DIGTOISE, TRUSTY_DIGTOISE, REMOTE_CONTROL_CAR, GLACIER_FANG_SHOT, CHLOROPHYTE_CLAYMORE_BLADE_SLAM};
    use tr_core::tile_light::FastRandom;
    use tr_core::World;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// A bare slot carrying its SetDefaults row slice (the special-bullets
    /// `proj` helper shape — the struct has private bookkeeping fields).
    fn slot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the W4-d21 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(160.0, 160.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p.uses_local_immunity = d.uses_local_immunity;
        p.local_hit_cooldown = d.local_hit_cooldown;
        p.uses_id_static_immunity = d.uses_id_static_immunity;
        p.id_static_hit_cooldown = d.id_static_hit_cooldown;
        p.melee = d.melee;
        p.magic = d.magic;
        p
    }

    /// The rows are the verbatim SetDefaults slices (the cited arms).
    #[test]
    fn w4_d21_the_rows_are_verbatim() {
        let claw = proj_data(ANTLION_CLAW).unwrap();
        assert_eq!((claw.w, claw.h, claw.ai_style, claw.penetrate), (26.0, 26.0, 201, 1));
        assert!(claw.melee, ":10102");
        let scissors = proj_data(STYLIST_SCISSORS).unwrap();
        assert_eq!((scissors.w, scissors.h, scissors.ai_style, scissors.penetrate), (28.0, 28.0, 201, 2));
        let torch = proj_data(TORCH_GOD_HELPER).unwrap();
        assert_eq!((torch.w, torch.h, torch.ai_style, torch.extra_updates, torch.time_left), (4.0, 4.0, 202, 2, 300));
        assert!(!torch.tile_collide, ":10123");
        let storm = proj_data(STORM_LIGHTNING).unwrap();
        assert_eq!((storm.w, storm.h, storm.ai_style), (16.0, 16.0, 203));
        assert!(storm.trap && storm.ignore_water && storm.net_important, ":10178-10182");
        let strike = proj_data(LIGHTNING_STRIKE_SHOT).unwrap();
        assert!(!strike.tile_collide && strike.magic && strike.ignore_water, ":10431-10435");
        let surge = proj_data(ARC_SURGE).unwrap();
        assert_eq!(surge.time_left, 1, ":10445");
        assert!(surge.uses_id_static_immunity && surge.id_static_hit_cooldown == 10, ":10447-10448");
        let dig = proj_data(DIGTOISE).unwrap();
        assert_eq!((dig.w, dig.h, dig.ai_style, dig.alpha), (34.0, 34.0, 204, 255));
        assert!(dig.net_important, ":10248");
        let car = proj_data(REMOTE_CONTROL_CAR).unwrap();
        assert_eq!((car.w, car.h, car.ai_style), (26.0, 26.0, 205));
        assert!(car.tile_collide, ":10300");
        let fang = proj_data(GLACIER_FANG_SHOT).unwrap();
        assert_eq!((fang.w, fang.h, fang.ai_style), (10.0, 10.0, 208));
        assert!(fang.magic && fang.ignore_water && fang.uses_local_immunity, ":10407-10411");
        assert_eq!(fang.local_hit_cooldown, -1, ":10411");
        let slam = proj_data(CHLOROPHYTE_CLAYMORE_BLADE_SLAM).unwrap();
        assert_eq!((slam.w, slam.h, slam.ai_style, slam.penetrate, slam.time_left), (16.0, 16.0, 209, -1, 5));
        assert!(!slam.tile_collide && slam.melee && slam.alpha == 255, ":10585-10593");
    }

    /// AI_209: the box resize from ai[1]/ai[2], the age counter, the >20 kill.
    #[test]
    fn w4_d21_the_blade_slam_resizes_and_ages() {
        let mut p = slot(CHLOROPHYTE_CLAYMORE_BLADE_SLAM);
        p.ai[1] = 40.0;
        p.ai[2] = 24.0;
        ai_claymore_blade_slam(&mut p);
        assert_eq!((p.w, p.h), (40.0, 24.0), ":69624-69625");
        assert_eq!(p.ai[0], 1.0, ":69626");
        assert!(p.active);
        // 20 more ticks: ai[0] reaches 21 → the kill.
        for _ in 0..20 {
            ai_claymore_blade_slam(&mut p);
        }
        assert_eq!(p.ai[0], 21.0);
        assert!(!p.active, ":69627-69630");
        assert_eq!(p.time_left, 0);
    }

    /// AI_201: the first-tick spin latch, the 15-tick straight flight then
    /// the drag/gravity, the return-state fade.
    #[test]
    fn w4_d21_the_thrown_melee_spins_flies_and_fades() {
        let mut p = slot(ANTLION_CLAW);
        p.vel = Vec2::new(6.0, -2.0);
        p.direction = 1;
        let mut rng = FastRandom::new(7);
        // Tick 1: the spin latch + the rotation += 0.45, ai[0] → 1.
        ai_thrown_melee(&mut p, &mut rng);
        assert_eq!(p.local_ai[0], 1.0, ":39959");
        assert_eq!(p.ai[0], 1.0, ":39971");
        // The spin: rot = NextFloat*2π + 0.45 (the latch set it first).
        // 15 more ticks: ai[0] reaches 16, still flying (no drag yet at 15).
        for _ in 0..15 {
            ai_thrown_melee(&mut p, &mut rng);
        }
        assert_eq!(p.ai[0], 16.0);
        // Tick 17: ai[0] > 15 → the drag + gravity, ai[0] stays 16.
        let vx_before = p.vel.x;
        ai_thrown_melee(&mut p, &mut rng);
        assert!((p.vel.x - vx_before * 0.95).abs() < 1e-5, ":39966");
        assert_eq!(p.ai[0], 16.0, "the count freezes past 15");
        // The return state: ai[1] = 1 → the fade.
        p.ai[1] = 1.0;
        p.alpha = 250;
        ai_thrown_melee(&mut p, &mut rng);
        assert!(!p.tile_collide && p.damage == 0, ":39975-39976");
        assert_eq!(p.alpha, 257, ":39978");
        assert!(!p.active, ":39979-39982");
    }

    /// AI_202: the torch-tile seek + reframe + kill on arrival.
    #[test]
    fn w4_d21_the_torch_helper_seeks_and_reframes() {
        let mut w = air();
        // A torch (type 4) at tile (20, 20).
        let i = w.index(20, 20);
        w.tiles[i].kind = 4;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = slot(TORCH_GOD_HELPER);
        p.ai[0] = 3.0; // the favour style
        p.ai[1] = 20.0;
        p.ai[2] = 20.0;
        // Place the helper within 16 px of the torch centre (328, 328).
        p.pos = Vec2::new(326.0, 326.0);
        let mut rng = FastRandom::new(7);
        ai_torch_god_helper(&mut p, &mut w, &mut rng);
        assert!(!p.active, ":39926");
        assert_eq!(p.vel, Vec2::ZERO, ":39920");
        assert_eq!(w.tiles[i].frame_y, 66, "ai[0]*22 = 3*22 (:39923)");
    }

    /// AI_203: the target lock + the MoveTowards + the age kill.
    #[test]
    fn w4_d21_the_storm_lightning_locks_and_seeks() {
        let mut p = slot(LIGHTNING_STRIKE_SHOT);
        p.pos = Vec2::new(160.0, 160.0); // centre (168, 168)
        // An NPC whose box contains the bolt centre → closest-point dist 0.
        let mut n = crate::npc::Npc::spawn_slime_sky(168.0, 168.0, crate::npc::SlimeVariant::Blue);
        n.max_hp = 40;
        n.hp = 40;
        n.x = 160.0;
        n.y = 160.0;
        let mut rng = FastRandom::new(7);
        ai_storm_lightning(&mut p, &air(), &[n], None, &mut rng);
        // The acquire locked the NPC (ai[0] = its index 0).
        assert_eq!(p.ai[0], 0.0, ":39723");
        assert_eq!(p.ai[1], 1.0, ":39775");
        assert!(p.active);
    }

    /// AI_203: the age cap kills the bolt (240 for the trap, 30 for 1117).
    #[test]
    fn w4_d21_the_storm_lightning_dies_at_the_age_cap() {
        let mut p = slot(LIGHTNING_STRIKE_SHOT);
        p.ai[1] = 29.0; // one tick from the 30 cap
        let mut rng = FastRandom::new(7);
        ai_storm_lightning(&mut p, &air(), &[], None, &mut rng);
        assert_eq!(p.ai[1], 30.0);
        assert!(!p.active, ":39776-39779");
    }

    /// AI_205: the input-bit pack + the ground drive.
    #[test]
    fn w4_d21_the_rc_car_packs_input_and_drives() {
        let mut p = slot(REMOTE_CONTROL_CAR);
        p.pos = Vec2::new(160.0, 800.0); // below the 640 space-kill
        p.vel = Vec2::new(0.0, 0.0);
        let mut pl = crate::player::Player::spawn(&air());
        pl.held_item_kind = 5738;
        pl.control_right = true;
        let mut rng = FastRandom::new(7);
        ai_remote_control_car(&mut p, &air(), Some(&pl), &mut rng);
        // The right bit (1<<1) packed into ai[0].
        assert_eq!(p.ai[0], 2.0, ":38102");
        assert!(p.active);
        assert_eq!(p.time_left, 2, ":38459");
    }

    /// AI_205: the space-height kill + the wrong-item kill.
    #[test]
    fn w4_d21_the_rc_car_dies_in_space_and_on_wrong_item() {
        let mut p = slot(REMOTE_CONTROL_CAR);
        p.pos = Vec2::new(160.0, 100.0); // above 640
        let mut rng = FastRandom::new(7);
        ai_remote_control_car(&mut p, &air(), None, &mut rng);
        assert!(!p.active, ":38086-38090");
        // The wrong-item kill.
        let mut p2 = slot(REMOTE_CONTROL_CAR);
        p2.pos = Vec2::new(160.0, 800.0);
        let mut pl = crate::player::Player::spawn(&air());
        pl.held_item_kind = 1; // not the RC Car item
        ai_remote_control_car(&mut p2, &air(), Some(&pl), &mut rng);
        assert!(!p2.active, ":38091-38094 + :38127-38131");
    }

    /// AI_208: the release scales the damage/knockback/penetrate.
    #[test]
    fn w4_d21_the_glacier_fang_release_scales() {
        let w = air();
        let mut p = slot(GLACIER_FANG_SHOT);
        p.ai[0] = 60.0; // full charge
        p.ai[1] = 0.0;
        p.damage = 10;
        p.knock_back = 2.0;
        p.vel = Vec2::new(8.0, 0.0);
        let mut pl = crate::player::Player::spawn(&w);
        // The owner centre coincides with the shot centre → can_hit true.
        pl.x = p.pos.x + p.w * 0.5 - crate::player::WIDTH * 0.5;
        pl.y = p.pos.y + p.h * 0.5 - pl.height * 0.5;
        let mut rng = FastRandom::new(7);
        // No channel → the release state. The owner-0 scale fires.
        ai_glacier_fang_shot(&mut p, 0, &w, Some((160.0, 160.0)), Some(&mut pl), &mut rng);
        assert_eq!(p.ai[1], 1.0, ":36960");
        // num5 = Remap(60, 0, 60, 0, 1) = 1 → damage ×4, kb ×4, penetrate 3.
        assert_eq!(p.damage, 40, ":36974");
        assert!((p.knock_back - 8.0).abs() < 1e-4, ":36975");
        assert_eq!(p.penetrate, 3, ":36973-36977");
        assert_eq!(p.extra_updates, 2, ":36997-37000");
        assert!(p.tile_collide, ":37001");
        assert!(p.active, "the clear line to the owner keeps the shot");
    }

    /// AI_204: the idle appear fades the alpha in and counts to the dig.
    #[test]
    fn w4_d21_the_digtoise_appears_then_digs() {
        let mut p = slot(DIGTOISE);
        p.pos = Vec2::new(160.0, 160.0);
        p.alpha = 255;
        let mut pl = crate::player::Player::spawn(&air());
        pl.held_item_kind = 5667; // the matching staff keeps it alive
        let mut rng = FastRandom::new(7);
        let digs = ai_digtoise(&mut p, &air(), Some(&pl), &mut rng);
        assert!(digs.is_empty(), "the idle state digs nothing");
        assert_eq!(p.alpha, 240, "DoPalAppearEffect -15 (:38810)");
        assert_eq!(p.ai[1], 1.0, ":38559");
        assert_eq!(p.time_left, 2, ":38482");
        // 180 ticks: the idle → dig-down transition.
        for _ in 0..179 {
            ai_digtoise(&mut p, &air(), Some(&pl), &mut rng);
        }
        assert_eq!(p.ai[0], 1.0, ":38560-38565");
    }

    /// Folds 3+4 — the 1091 kill in AIR: the storm bolt reaches the
    /// centre untruncated (flag6 true → localAI[0] = 1, `:70825`), the
    /// penetration opens, and the hitbox's points close on the centre.
    /// The slot does NOT resize (the C# never resizes in this arm).
    #[test]
    fn w4_d21_the_storm_kill_generates_the_bolt_and_opens_penetration() {
        let w = air();
        let mut p = slot(STORM_LIGHTNING);
        p.pos = Vec2::new(160.0, 160.0); // centre (168, 168)
        p.ai[2] = 42.0; // the bolt seed
        let mut rng = FastRandom::new(7);
        let (hb, spawns) = storm_lightning_kill(&mut p, &w, None, &mut rng);
        assert_eq!(p.local_ai[0], 1.0, ":70825 — no tile hit, no liquid");
        assert_eq!((p.max_penetrate, p.penetrate), (-1, -1), ":70831-70832");
        assert_eq!((p.w, p.h), (16.0, 16.0), "no resize in the C# arm");
        assert!(spawns.is_empty(), "no leafy tree top under the centre");
        assert_eq!(hb.point_size, (16, 16), ":70834");
        assert!(!hb.points.is_empty());
        let last = *hb.points.last().unwrap();
        assert!(last.distance(Vec2::new(168.0, 168.0)) < 16.0);
        // The same seed re-generates the same path (the LCG determinism).
        let mut p2 = slot(STORM_LIGHTNING);
        p2.pos = Vec2::new(160.0, 160.0);
        p2.ai[2] = 42.0;
        let (hb2, _) = storm_lightning_kill(&mut p2, &w, None, &mut rng);
        assert_eq!(hb.points, hb2.points);
    }

    /// Folds 3+4 — the 1091 kill through a SOLID row: the bolt truncates
    /// (flag6 false), the slot RE-CENTERS on the bolt's last position
    /// (`:70822`), localAI[0] = 2 (`:70825`). The 1117's generator phases
    /// through the same row (SolidTileCollision false) — localAI[0] = 1.
    #[test]
    fn w4_d21_the_storm_kill_recenters_on_the_truncated_bolt() {
        let mut w = air();
        for x in 0..w.width {
            let i = w.index(x, 5);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = slot(STORM_LIGHTNING);
        p.pos = Vec2::new(160.0, 160.0);
        p.ai[2] = 42.0;
        let mut rng = FastRandom::new(7);
        let (hb, _) = storm_lightning_kill(&mut p, &w, None, &mut rng);
        assert_eq!(p.local_ai[0], 2.0, ":70825 — the tile hit");
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let last = *hb.points.last().unwrap();
        assert_eq!(center, last, ":70822 — base.Center = the last position");
        assert!((64.0..112.0).contains(&center.y), "the re-centre sits at/above the row");
        // The 1117 phases through (its generator has no tile collision).
        let mut q = slot(LIGHTNING_STRIKE_SHOT);
        q.pos = Vec2::new(160.0, 160.0);
        q.ai[2] = 42.0;
        let (hb2, _) = storm_lightning_kill(&mut q, &w, None, &mut rng);
        assert_eq!(q.local_ai[0], 1.0, ":323 — the 1117 keeps the no-hit state");
        let qc = Vec2::new(q.pos.x + q.w * 0.5, q.pos.y + q.h * 0.5);
        assert_eq!(qc, Vec2::new(168.0, 168.0), "no re-centre for the 1117");
        assert!(hb2.points.len() > hb.points.len(), "the phased bolt walks farther");
    }

    /// Folds 3+4 — the liquid centre: localAI[0] = 3 (`:70826-70830`)
    /// and the electrification walks consume their Next(4) draws on the
    /// stream (one per liquid cell plotted, `:78549`). The witness
    /// mirrors the 39 radial walks over the same single-liquid-cell
    /// world and replays one Next(4) per plotted liquid cell.
    #[test]
    fn w4_d21_the_liquid_kill_sets_state_three_and_draws_per_cell() {
        let mut w = air();
        let i = w.index(10, 10); // the centre cell (168/16 = 10)
        w.tiles[i].liquid = 255;
        let mut p = slot(STORM_LIGHTNING);
        p.pos = Vec2::new(160.0, 160.0);
        p.ai[2] = 42.0;
        let mut rng = FastRandom::new(7);
        let mut witness = FastRandom::new(7);
        let (_, _) = storm_lightning_kill(&mut p, &w, None, &mut rng);
        assert_eq!(p.local_ai[0], 3.0, ":70828");
        // Mirror the walks (`:78537-78540`): the centre is (168, 168),
        // each endpoint at centre + UnitX.RotatedBy(PI*i/20)*500.
        let center = Vec2::new(168.0, 168.0);
        let (x0, y0) = (10, 10);
        let mut draws = 0;
        for i in 1..40 {
            let angle = std::f32::consts::PI * i as f32 / 20.0;
            let end = center
                + crate::bow_launch::rotated_by(Vec2::X, angle as f64) * 500.0;
            let (x1, y1) = ((end.x as i32) / 16, (end.y as i32) / 16);
            plot_line(x0, y0, x1, y1, true, &mut |x, y| {
                let wet = w.in_bounds(x, y) && w.tile(x, y).is_some_and(|t| t.liquid != 0);
                if wet {
                    draws += 1;
                    let _ = witness.next_range(0, 4);
                }
                wet
            });
        }
        assert!(draws >= 39, "every walk plots the liquid centre cell");
        assert_eq!(rng.seed, witness.seed, "one Next(4) per liquid cell, in walk order");
    }

    /// Folds 3+4 — the 1122 Arc Surge kill: localAI[0] = 2 (`:70791`),
    /// the bolt walks from the hand anchor to the centre. The hand at
    /// (168, 100) → the bolt's first position sits near the hand, the
    /// last near the centre.
    #[test]
    fn w4_d21_the_arc_surge_kill_walks_hand_to_centre() {
        let w = air();
        let mut p = slot(ARC_SURGE);
        p.pos = Vec2::new(160.0, 160.0);
        p.ai[2] = 42.0;
        let mut rng = FastRandom::new(7);
        let hand = Vec2::new(168.0, 100.0);
        let (hb, spawns) = storm_lightning_kill(&mut p, &w, Some(hand), &mut rng);
        assert_eq!(p.local_ai[0], 2.0, ":70791");
        assert_eq!((p.max_penetrate, p.penetrate), (-1, -1), ":70792-70793");
        assert!(spawns.is_empty(), "the 1122 has no greek-fire arm");
        let first = hb.points[0];
        let last = *hb.points.last().unwrap();
        assert!(first.distance(hand) < 16.0, "the bolt starts at the hand");
        assert!(last.distance(Vec2::new(168.0, 168.0)) < 16.0);
    }

    /// The dispatch census: every new rowed style has exactly one live arm.
    #[test]
    fn w4_d21_every_new_style_has_one_live_arm() {
        let src = include_str!("projectile.rs");
        let start = src.find("match data.ai_style {").expect("the dispatch");
        let body = &src[start..];
        let mut counts = std::collections::BTreeMap::new();
        for line in body.lines().skip(1) {
            if line == "                        _ => {}" {
                break;
            }
            if let Some(rest) = line.strip_prefix("                        ") {
                if rest.starts_with(' ') {
                    continue;
                }
                let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !digits.is_empty() && rest[digits.len()..].trim_start().starts_with("=>") {
                    *counts.entry(digits.parse::<u8>().unwrap()).or_insert(0u32) += 1;
                }
            }
        }
        for style in [201u8, 202, 203, 204, 205, 208, 209] {
            assert_eq!(counts.get(&style), Some(&1), "aiStyle {style} has one live arm");
        }
        // 210 was already ported; 206 lands a9 (not on this base); 207 has
        // no rowed kinds (the empty AI).
        assert_eq!(counts.get(&210), Some(&1), "aiStyle 210 keeps its arm");
    }
}
