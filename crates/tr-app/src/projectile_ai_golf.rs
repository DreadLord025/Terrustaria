//! S1-P-i55 golf port — the two golf projectile AIs and the club strike
//! (`crates/tr-core/src/golf.rs` + `golf_state.rs` REUSED, never duplicated).
//!
//! C# sources read WHOLE (the golden rule):
//! - aiStyle 149: `Projectile.AI_149_GolfBall` (`Projectile.cs:20331-20374`)
//!   + `CheckSectionsInCaseOwnerIsWatching` (`:20376-20389`) + the
//!   `SetDefaults` arm for 721/739-752 (`:7626-7635`) + `UpdatePosition`'s
//!   golf-ball skip (`:20435-20438`) + `NewProjectile`'s >10-ball cull
//!   (`:10932-10955`).
//! - aiStyle 150: `Projectile.AI_150_GolfClubHelper`
//!   (`Projectile.cs:52417-52548`) + the 722 `SetDefaults` arm
//!   (`:7636-7645`).
//! - the strike: `GolfHelper.HitGolfBall` (`GolfHelper.cs:434-457`) — the
//!   velocity half is `tr_core::golf::hit_golf_ball_velocity` (reused), the
//!   projectile writes + `RecordSwing` land here;
//!   `GolfHelper.CalculateShotStrength(Projectile, Entity)` (`:305-324`) is
//!   `tr_core::golf::calculate_shot_strength_from_helper` (reused), the club
//!   table `GetClubPropertiesFromGolfHelper` (`:326-329`) resolves the
//!   owner's held item through `tr_core::golf::get_club_properties`.
//! - the tee placement: `Player.TryPlacingAGolfBallNearANearbyTee`
//!   (`Player.cs:49677-49737`) + `GetPreferredGolfBallToUse`
//!   (`:49739-49770`) + the ball-item shoot arm
//!   (`ItemCheck_Shoot`, `:47516-47537`) + the tee right-click
//!   (`TileInteractionsUse`'s 494 arm, `:32342-32354`).
//! - the cup: `GolfHelper.ContactListener.PutBallInCup`
//!   (`GolfHelper.cs:84-113`) + `PutBallInCup_TextAndEffects` (`:115-123`)
//!   + `EmitGolfballExplosion` (`:154-214`) — the contact DECISIONS are
//!   `tr_core::golf::GolfContactListener` (reused); this module performs the
//!   effects the listener reports.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! - 149: ZERO `Main.rand` draws in the AI itself. The bounce dust's one
//!   `NextVector2CircularEdge(0.5, 0.4)` (`GolfHelper.cs:80`, ONE
//!   `NextFloat` — `Utils.cs:1271-1274` → `NextVector2Unit` `:1261-1264`)
//!   fires per collision whose bounce gate held
//!   (`-2 < vy < -0.3 && |v| > 1`, `:75`); the core listener reports a
//!   `GolfContact::BounceDust` in collision order, so the port consumes one
//!   draw per contact on the same stream at the same decision point, and a
//!   `BreaksToy`/`InCup` kill earlier in the same step suppresses the later
//!   dust draws exactly as the C#'s `projectile.active` re-read does (the
//!   dust BODY is the layer skip).
//! - 150: ONE `Main.rand.NextFloatDirection()` per struck ball
//!   (`Projectile.cs:52457`), BEFORE the dust ring (the ring itself rolls
//!   nothing — every `Dust.NewDustPerfect` there passes a fixed velocity),
//!   then `CalculateShotStrength`/`HitGolfBall` (RNG-free). The
//!   `NextFloatDirection` draw is `next_float() * 2 - 1` on the SAME
//!   `DropRng` stream (`Utils.cs:1203-1206`; the ai067_b convention maps
//!   NextDouble onto `next_float` — one Sample() per draw either way).
//! - the cup: `EmitGolfballExplosion`'s draws ride the strike's stream in
//!   the C#'s order (`GolfHelper.cs:154-214`): 12 `NextFloat` colour draws
//!   (the 0.085 ring), one `NextFloat` hue seed, one `Next(5, 10)` spoke
//!   count, per spoke one `Next(5, 22)` dust count + one `NextFloat`
//!   radius wobble + per dust one `NextVector2Circular(0.05, 0.05)` (TWO
//!   `NextFloat`s, `Utils.cs:1266-1269`), 5 `NextFloat` colour draws (the
//!   0.2 ring), one `NextFloatDirection`, then 7 × two `NextFloat` colour
//!   draws (the 0.15 ring). The f32 step loops keep the C#'s accumulation
//!   (`num += 0.085f` — the iteration count is the source's own, drift
//!   included, the `celeb2_explosion` precedent).
//!
//! CODE-ONLY MODE (S1-P-i55): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); they were verified by
//! re-reading the cited C# blocks and hand arithmetic.
//!
//! Deliberate folds (ANALYSIS "The golf projectile AIs (S1-P-i55)"):
//! - `gravDir == -1` arms (`:52425-52430`, `Player.cs:52229-52232`): the
//!   port is upright-only (`gravDir = 1`), so the flipped-gravity kill/use
//!   refusal is unreachable — recorded, not ported as dead code.
//! - `player.SetDummyItemTime` / `SetItemTime` (`Player.cs:4432-4441`,
//!   `:4406-4414`) drive `itemAnimation`/`itemTime`/`itemTimeMax` — the
//!   port's swing cadence is the app's `swing`/`item_time` seconds pair, so
//!   the helper's writes ride back as [`GolfClubTick::dummy_item_time`] /
//!   [`GolfClubTick::cancel_use`] and the dispatch applies them.
//! - `player.heldProj` (`Player.cs:1516`) — the port's player carries the
//!   field now; the writes are the C#'s (`:52490`/`:52535`).
//! - `CheckSectionsInCaseOwnerIsWatching` (`:20372`) — MP section loading,
//!   no ported layer (the pool doc's netdeviation).
//! - `Collision.SwitchTiles(..., 4)` (`:20367`) — the ball's pressure-pad
//!   trip runs pool-side (the arm cannot borrow the wiring); the gate
//!   `ForcePlateDetection[135]` is null in the set (`ProjectileID.cs:434-
//!   520` lists no 135) so `!flag.HasValue` holds — the pad call fires for
//!   every moving owned ball with `localAI[1] != 0`.
//! - `NetMessage.SendData(27/59/128)` — the `27` rides the pool's
//!   `net_update` flag; the cup's `59` rides `App::send_hit_switch`; the
//!   `128` golf-score message is client-only bookkeeping, unported.
//! - the dust/gore bodies, the chat colour's byte-255 alpha, the
//!   `Main.blockMouse`/`mouseInterface` UI flags (`:52513-52514`) — no
//!   ported layers; the cancel's item-timer writes DO land.

use glam::Vec2;
use tr_core::ball::{BallBody, BallState};
use tr_core::golf::{
    self, DefaultMaterials, GolfContact, GolfContactListener, GolfMaterials,
};
use tr_core::golf_state::{GolfBall, GolfState, is_a_golf_ball};
use tr_core::loot::DropRng;
use tr_core::World;

use crate::projectile::{Projectile, ProjectilePool, GOLF_BALL};

/// `TileID.GolfTee` (`TileID.cs:1617`) — the tee the -1 sentinel watches.
pub(crate) const TILE_GOLF_TEE: u16 = 494;

/// The per-tick golf context the world-sim pass builds from the App (the
/// pool cannot borrow it): the `Main.LocalGolfState` clone (the C# reads
/// the live state mid-walk; the port's arms run on the clone and the
/// effects drain writes the resolved state back — the strikes'
/// `RecordSwing` and the cup's scoring run IN the arms, so the clone is
/// authoritative for the tick), the mouse world point, the mouse-right
/// pair (`Main.mouseRight`/`Main.mouseRightRelease` — the release flag
/// lags one frame, `Main.cs:62458-62465` runs after Draw, so the pair's
/// `true && true` is the PRESS tick), and the player snapshot the
/// aiStyle-150 arm reads.
#[derive(Clone, Debug)]
pub(crate) struct GolfContext {
    pub golf_state: GolfState,
    pub mouse_world: Vec2,
    pub mouse_right: bool,
    pub mouse_right_release: bool,
    pub player: GolfClubPlayer,
    /// `Main.player[owner].name` — the cup chat line's first argument.
    pub player_name: String,
    /// `Main.player[owner].golferScore` — the accumulator the cup's
    /// `AccumulateGolfingScore` writes (`Player.cs` via
    /// `GolfHelper.cs:101`).
    pub golfer_score: i32,
}

/// One golf arm's queued effect — the world-sim pass resolves them after
/// `update` (the pool cannot borrow the App's wiring/chat/audio/player).
/// The strikes and the cup scoring already ran against the context's
/// `GolfState` clone inside the arm; the pass writes the clone back.
#[derive(Clone, Debug)]
pub(crate) enum GolfEffect {
    /// `TryPlacingAGolfBallNearANearbyTee(player.Center)`
    /// (`Projectile.cs:52442-52445`) — the pass resolves it (it needs the
    /// player + the inventory + the pool).
    TeePlace { place_position: Vec2 },
    /// The cup's `Wiring.HitSwitch` + `SendData(59)` (`GolfHelper.cs:105-
    /// 106`) and the chat line (`:103` → `:121`).
    CupScored {
        hit_location: (i32, i32),
        chat: Option<String>,
        /// The explosion's `SoundID.Item129` cue (`GolfHelper.cs:156`).
        cue: Option<(u16, i32, f32, f32, f32, f32)>,
    },
    /// `Collision.SwitchTiles(..., 4)` (`Projectile.cs:20367`) — the
    /// moving owned ball's pressure-pad probe: `(pos, old_pos, w, h)`.
    PadProbe { pos: Vec2, old_pos: Vec2, w: f32, h: f32 },
    /// The aiStyle-150 player-side writes: `ChangeDir` (`:52489`/
    /// `:52534`), `heldProj` (`:52490`/`:52535`), `SetDummyItemTime`
    /// (`:52496`/`:52547`) and the right-click cancel's timer zeroes
    /// (`:52515-52518`). `held_proj` is `None` on the paths that never
    /// reach a `heldProj` write (the orphan kill, the follow-through's
    /// death tick, the cancel).
    PlayerWrites {
        facing: Option<i32>,
        held_proj: Option<i32>,
        dummy_item_time: Option<i32>,
        cancel_use: bool,
    },
}

/// What one aiStyle-149 tick hands back to the pool: the cup capture the
/// listener reported (the scoring/effects run pool-side where the
/// `GolfState` lives) and the pressure-pad probe request.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GolfBallTick {
    /// `GolfContact::InCup` — `(tile_x, tile_y, impact_point, normal)`; the
    /// pool resolves `PutBallInCup` (`GolfHelper.cs:84-113`).
    pub in_cup: Option<(i32, i32, Vec2, Vec2)>,
    /// `Collision.SwitchTiles(this, position, width, height, oldPosition,
    /// 4)` (`Projectile.cs:20364-20368`) — the moving owned ball's pad
    /// probe: `(pos, old_pos, w, h)`.
    pub wants_pad_probe: Option<(Vec2, Vec2, f32, f32)>,
}

/// `AI_149_GolfBall` (`Projectile.cs:20331-20374`) for one slot.
///
/// `materials` is the `GolfMaterials` seam (the embedded `Materials.json`
/// gap — `DefaultMaterials` until the retail table lands, ANALYSIS
/// "Golf (S1-E-9)"). `local_owner` is `Main.myPlayer`.
pub(crate) fn ai_golf_ball(
    p: &mut Projectile,
    world: &World,
    materials: &dyn GolfMaterials,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> GolfBallTick {
    let mut tick = GolfBallTick::default();
    // `:20333-20336` — the keep-alive floor. `npcProj` is stock false for
    // every player-spawned ball (`:797`); the port has no npcProj field.
    if p.time_left < 10 {
        p.time_left = 10;
    }
    // `:20337-20346` — the tee sentinel: while `ai[1] == -1` the ball sits
    // on its tee; the moment the tile under its Bottom is no longer an
    // active 494 the sentinel clears (netUpdate) and the ball falls. The
    // arm RETURNS either way — no physics while teed.
    if p.ai[1] == -1.0 {
        let bottom = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h);
        let tx = (bottom.x as i32) >> 4;
        let ty = (bottom.y as i32) >> 4;
        let on_tee = world
            .tile(tx, ty)
            .is_some_and(|t| t.active() && t.kind == TILE_GOLF_TEE);
        if !on_tee {
            p.ai[1] = 0.0;
            p.net_update = true;
        }
        return tick;
    }
    // `:20347` — `GolfHelper.StepGolfBall(this, ref localAI[0])`: the
    // wrapper (`tr_core::golf::step_golf_ball`, REUSED) pins the golf
    // physics on the generic engine + the golf listener (both REUSED).
    // `localAI[0]` is the angular velocity.
    let listener = GolfContactListener::new(materials);
    let mut body = BallBody {
        position: p.pos,
        velocity: p.vel,
        size: Vec2::new(p.w, p.h),
    };
    let mut angular = p.local_ai[0];
    let result = golf::step_golf_ball(&mut body, &mut angular, (), world, &listener);
    // The engine writes the entity back only on the non-OutOfBounds paths
    // (`BallCollision.cs:85-87`) — `step` already keeps that contract, so
    // the body fields are copied unconditionally (an OutOfBounds body is
    // byte-equal to the input).
    p.pos = body.position;
    p.vel = body.velocity;
    p.local_ai[0] = angular;
    // `:20348-20355` — Resting zeroes the damage; the client netUpdate on
    // the state CHANGE is the `netMode == 1` arm (single-player port: the
    // flag write is kept, the send cadence is the pool's).
    if result.state == BallState::Resting {
        p.damage = 0;
        if p.owner == local_owner && p.local_ai[1] != ball_state_f32(BallState::Resting) {
            p.net_update = true;
        }
    }
    // `:20356-20361` — the state latch, then the spin: the angular velocity
    // every tick, plus the velocity term while airborne and moving.
    p.local_ai[1] = ball_state_f32(result.state);
    p.rot += p.local_ai[0];
    if p.vel.y != 0.0 && result.state == BallState::Moving {
        p.rot += p.vel.x * 0.1 + p.vel.y * 0.1;
    }
    // `:20362-20369` — the pressure-pad probe (the pool runs the wiring
    // call). `ForcePlateDetection[135]` is null (`ProjectileID.cs:434-520`
    // — 135 is unlisted), so `!flag.HasValue` holds. `localAI[1]` was JUST
    // written to the state, so `!= 0` is `state != Resting` — and the outer
    // gate already requires Moving. The C# reads `base.position` AFTER the
    // engine wrote it back (`BallCollision.cs:85-87` runs before Step
    // returns), so the probe takes the POST-step position.
    if result.state == BallState::Moving && p.owner == local_owner && p.local_ai[1] != 0.0 {
        tick.wants_pad_probe = Some((p.pos, p.old_position, p.w, p.h));
    }
    // `:20370-20373` — CheckSectionsInCaseOwnerIsWatching: MP section
    // loading, no ported layer (module doc).
    //
    // The listener's reported contacts, in COLLISION order (the engine's
    // sub-step order — `GolfContact`'s doc). The cup capture rides out (the
    // pool scores it); the toy-breaker kill (`GolfHelper.cs:71-74`) is
    // `BreaksFromToyBreaker[type] && tile.nactive() && BreaksToys[kind]` —
    // every golf ball is in the set (`ProjectileID.cs:189`) and the only
    // BreaksToys tile is 740 (`TileID.cs:479`), so the core listener's
    // `BreaksToy` contact is the kill. Both kills flip the C#'s
    // `projectile.active`, which gates the later dust draws (`:63`/`:75`)
    // — replaying in order keeps the stream exact across the engine's
    // `ceil(speed/2)` sub-steps. The CUP kill is the pool's
    // (`PutBallInCup`'s `:112`, AFTER the scoring reads the live ball), so
    // the capture only flips the replay-local `alive` flag here; the
    // toy-breaker kill is immediate (`:73`).
    let mut alive = p.active;
    for contact in listener.take_contacts() {
        match contact {
            GolfContact::InCup {
                tile_x,
                tile_y,
                impact_point,
                normal,
            } => {
                // The FIRST capture wins: the C#'s `PutBallInCup` kills the
                // ball (`:112`), so a second capture's `projectile.active`
                // gate (`:63`) refuses it — but the velocity zeroing still
                // ran (the engine already did it).
                if alive && tick.in_cup.is_none() {
                    tick.in_cup = Some((tile_x, tile_y, impact_point, normal));
                    alive = false;
                }
            }
            GolfContact::BreaksToy => {
                p.active = false;
                p.time_left = 0;
                alive = false;
            }
            // `GolfHelper.cs:75-81` — the bounce dust's ONE NextFloat
            // (`NextVector2CircularEdge(0.5, 0.4)` = one `NextVector2Unit`
            // angle, `Utils.cs:1261-1264`), gated on the ball still being
            // active (the kill contacts above are the C#'s `projectile
            // .active` re-read). The dust body is the layer skip.
            GolfContact::BounceDust => {
                if alive {
                    let _ = rng.next_float();
                }
            }
        }
    }
    tick
}

/// `BallState`'s C# enum values (`BallState.cs:3-8`): Resting 0, Moving 1,
/// OutOfBounds 2 — the `localAI[1]` latch stores the raw int as a float.
fn ball_state_f32(state: BallState) -> f32 {
    match state {
        BallState::Resting => 0.0,
        BallState::Moving => 1.0,
        BallState::OutOfBounds => 2.0,
    }
}

/// What one aiStyle-150 tick hands back to the pool: the tee-placement
/// request, the struck balls (the pool owns the `GolfState.RecordSwing`),
/// the player-side timer writes and the queued cues.
#[derive(Clone, Debug, Default)]
pub(crate) struct GolfClubTick {
    /// `TryPlacingAGolfBallNearANearbyTee(player.Center)`
    /// (`Projectile.cs:52442-52445`) — the pool resolves it (it needs the
    /// player + the inventory + the pool itself).
    pub wants_tee_place: bool,
    /// The struck balls, in the C#'s scan order: `(slot, velocity,
    /// rough_land_resistance)` — the pool applies `HitGolfBall`'s
    /// projectile writes + `RecordSwing` per entry (the velocity blend
    /// needs the world + the materials, which the slot arm cannot see).
    pub strikes: Vec<(usize, Vec2, f32)>,
    /// `player.SetDummyItemTime(frames)` (`Player.cs:4432-4441`) — the
    /// dispatch converts to the app's swing/item-time pair.
    pub dummy_item_time: Option<i32>,
    /// The right-click cancel's `SetItemTime(0)` + the three counter
    /// zeroes (`Projectile.cs:52515-52518`).
    pub cancel_use: bool,
    /// `player.ChangeDir(direction)` (`:52489`/`:52534`) — the flail
    /// precedent: the facing rides out, the dispatch applies it.
    pub facing: Option<i32>,
    /// `player.heldProj = whoAmI` (`:52490`/`:52535`) — true on the paths
    /// that reach a write (the follow-through's live ticks, the charge
    /// hold); false on the early returns (the orphan kill, the death
    /// tick, the cancel).
    pub writes_held_proj: bool,
    /// The strike arm's cue list, in the C#'s play order: Item1 at the
    /// player (`:52448`), then one Item126 per struck ball (`:52467`). The
    /// slot's `wants_sound` is a single Option — a multi-ball strike would
    /// lose every cue but the last — so the golf arm queues through this
    /// list and the dispatch drains it into the pool's sound queue.
    pub sounds: Vec<(u16, i32, f32, f32, f32, f32)>,
}

/// The per-tick player snapshot the helper reads (`Main.player[owner]`):
/// the port's pool cannot borrow the player mid-slot, so the dispatch
/// builds this once (the `FlailInputs` pattern).
#[derive(Clone, Copy, Debug)]
pub(crate) struct GolfClubPlayer {
    /// `player.active` — the port's player presence (`:52420`).
    pub active: bool,
    /// `player.Center`.
    pub center: Vec2,
    /// `player.Bottom` — `ValidateShot`'s legality box anchor.
    pub bottom: Vec2,
    /// `player.direction`.
    pub direction: i32,
    /// `player.channel` (`:52446`).
    pub channel: bool,
    /// `player.HeldItem.useAnimation` (`:52482`/`:52491`/`:52537`).
    pub use_animation: i32,
    /// `player.HeldItem.type` — `GetClubPropertiesFromGolfHelper`
    /// (`GolfHelper.cs:326-329`).
    pub held_item_type: i32,
}

/// `AI_150_GolfClubHelper` (`Projectile.cs:52417-52548`) for one slot.
///
/// `slots` is the pool's full array (the two 1000-slot scans read it);
/// `mouse_world` is `Main.MouseWorld`, `mouse_right`/`mouse_right_release`
/// the `Main.mouseRight`/`Main.mouseRightRelease` pair (`:52510`).
/// `local_owner` is `Main.myPlayer`.
pub(crate) fn ai_golf_club_helper(
    p: &mut Projectile,
    slots: &[Projectile],
    player: GolfClubPlayer,
    mouse_world: Vec2,
    mouse_right: bool,
    mouse_right_release: bool,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> GolfClubTick {
    let mut tick = GolfClubTick::default();
    // `:52419-52424` — the orphan kill.
    if !player.active {
        p.active = false;
        p.time_left = 0;
        return tick;
    }
    // `:52425-52430` — the flipped-gravity kill: gravDir is 1 in the port
    // (module doc), unreachable.
    //
    // `:52431-52445` — the nearby-ball scan: a 500x500 rect centred on the
    // player (`Utils.CenteredRectangle`, `Utils.cs:947-950` — the (int)
    // truncation of `center - size/2`), any ACTIVE local-owned golf ball
    // whose truncated centre point is inside. With none, the helper tees
    // a fresh ball (ai[0] == 0 only).
    let rx = (player.center.x - 250.0) as i32;
    let ry = (player.center.y - 250.0) as i32;
    let mut near_ball = false;
    for q in slots.iter() {
        if q.active && q.owner == local_owner && is_a_golf_ball(q.kind as i32) {
            let (cx, cy) = q.center();
            let (px, py) = (cx as i32, cy as i32);
            // `Rectangle.Contains(Point)`: x in [X, X+Width), y likewise.
            if px >= rx && px < rx + 500 && py >= ry && py < ry + 500 {
                near_ball = true;
                break;
            }
        }
    }
    if !near_ball && p.owner == local_owner && p.ai[0] == 0.0 {
        tick.wants_tee_place = true;
    }
    // `:52446-52478` — the release strike: the channel let go while still
    // charging. SoundID.Item1 plays (the cue rides the tick's `sounds`
    // list — the S1-E-21h queue), then EVERY active owned golf ball that
    // passes `ValidateShot` is struck — the scan does NOT break on the
    // first.
    if !player.channel && p.ai[0] == 0.0 {
        let (cx, cy) = p.center();
        // `:52448` — SoundID.Item1 at PLAYER.Center (not the helper's own
        // center — the C# reads `player.Center`; S1-E-21h d76 fix).
        tick.sounds.push((2, 1, player.center.x, player.center.y, 0.0, 1.0));
        for (j, q) in slots.iter().enumerate() {
            // `:52452` — the shot vector is computed for EVERY slot (even
            // inactive ones) but only consumed by the survivors.
            let (qcx, qcy) = q.center();
            let shot_vector = mouse_world - Vec2::new(qcx, qcy);
            if !q.active
                || !is_a_golf_ball(q.kind as i32)
                || q.owner != p.owner
            {
                continue;
            }
            let (valid, shot_vector) = golf::validate_shot(
                Vec2::new(qcx, qcy),
                player.bottom,
                player.direction,
                shot_vector,
            );
            if !valid {
                continue;
            }
            // `:52457` — the ONE NextFloatDirection per struck ball,
            // BEFORE the dust ring (the ring rolls nothing — fixed
            // velocities). `Utils.cs:1203-1206`: NextDouble()*2-1; the
            // DropRng seam's next_float IS NextDouble (one Sample()).
            let _num = rng.next_float() * 2.0 - 1.0;
            // `:52458-52466` — the 10-dust ring: no draws (the
            // `num2 % 0.2f == 0f` velocity scale is dust-body only).
            // `:52467` — SoundID.Item126 at the ball.
            tick.sounds.push((2, 126, qcx, qcy, 0.0, 1.0));
            // `:52468-52473` — the local-owner strike: the strength off
            // the HELPER's centre and the owner's held club, the velocity
            // along the clamped shot vector. The projectile writes +
            // RecordSwing run pool-side (the strike list).
            if p.owner == local_owner {
                let strength = golf::calculate_shot_strength_from_helper(
                    p.ai[0],
                    Vec2::new(cx, cy),
                    Vec2::new(qcx, qcy),
                    golf::get_club_properties(player.held_item_type),
                );
                // `Vector2.Normalize(shotVector) * AbsoluteStrength`
                // (`:52471`) — the unguarded XNA normalize (a zero vector
                // yields NaN, exactly as the C#).
                let v = shot_vector / shot_vector.length() * strength.absolute_strength;
                tick.strikes.push((j, v, strength.rough_land_resistance));
            }
        }
        p.ai[0] = 1.0;
        p.ai[1] = 0.0;
        p.net_update = true;
    }
    // `:52479-52497` — the follow-through: the counter ticks, the helper
    // dies at useAnimation + 30, the player's dummy item time counts the
    // swing down (floor 2).
    if p.ai[0] == 1.0 {
        p.ai[1] += 1.0;
        let num3 = (player.use_animation + 30) as f32;
        if p.ai[1] >= num3 {
            p.active = false;
            p.time_left = 0;
            return tick;
        }
        p.vel = Vec2::ZERO;
        tick.facing = Some(p.direction);
        // `player.heldProj = whoAmI` — the dispatch writes the slot index.
        tick.writes_held_proj = true;
        let mut num4 = player.use_animation - p.ai[1] as i32;
        if num4 < 2 {
            num4 = 2;
        }
        tick.dummy_item_time = Some(num4);
    }
    // `:52498-52501` — the charge tail only runs while ai[0] == 0.
    if p.ai[0] != 0.0 {
        return tick;
    }
    // `:52502-52521` — the local-owner aim: the helper's centre tracks the
    // mouse (netUpdate on the move); the right-click PRESS edge cancels
    // the swing (the C#'s `mouseRight && mouseRightRelease` pair — the
    // release flag is false while held, so this is the press tick).
    if p.owner == local_owner {
        let (cx, cy) = p.center();
        if mouse_world != Vec2::new(cx, cy) {
            p.net_update = true;
            // `base.Center = mouseWorld` — the position re-centres.
            p.pos = mouse_world - Vec2::new(p.w * 0.5, p.h * 0.5);
        }
        if mouse_right && mouse_right_release {
            p.active = false;
            p.time_left = 0;
            // `player.mouseInterface`/`Main.blockMouse` — UI flags, no
            // ported layer (module doc); the item-timer writes land.
            tick.cancel_use = true;
            return tick;
        }
    }
    // `:52522-52532` — the facing latch: only when the helper MOVED this
    // tick (position != oldPosition), face away from the player.
    if p.pos != p.old_position {
        let (cx, _) = p.center();
        p.direction = if cx - player.center.x > 0.0 { 1 } else { -1 };
    }
    // `:52533-52547` — the charge hold: zero velocity, the forced facing,
    // the heldProj write, the wind-up counter (wraps at useAnimation*4*3
    // back to useAnimation*4) and the dummy item time (floor 2).
    p.vel = Vec2::ZERO;
    tick.facing = Some(p.direction);
    tick.writes_held_proj = true;
    p.ai[1] += 1.0;
    let num5 = player.use_animation * 4;
    if p.ai[1] > (num5 * 3) as f32 {
        p.ai[1] = num5 as f32;
    }
    let mut num6 = player.use_animation + (num5 as f32).min(p.ai[1]) as i32;
    if num6 < 2 {
        num6 = 2;
    }
    tick.dummy_item_time = Some(num6);
    tick
}

/// `GolfHelper.HitGolfBall` (`GolfHelper.cs:434-457`) — the projectile
/// half. The velocity blend is `tr_core::golf::hit_golf_ball_velocity`
/// (REUSED); this applies it plus the four projectile writes and the
/// `GolfState.RecordSwing` (`:448-455`). `who_am_i` is the slot index (the
/// C# `Projectile.whoAmI`).
pub(crate) fn strike_golf_ball(
    ball: &mut Projectile,
    who_am_i: usize,
    velocity: Vec2,
    rough_land_resistance: f32,
    world: &World,
    materials: &dyn GolfMaterials,
    golf_state: &mut GolfState,
) {
    // `:436-439` — the tile under the ball's Bottom + 1 px.
    let bottom = Vec2::new(ball.pos.x + ball.w * 0.5, ball.pos.y + ball.h) + Vec2::new(0.0, 1.0);
    let tx = (bottom.x as i32) >> 4;
    let ty = (bottom.y as i32) >> 4;
    let tile_below = world.tile(tx, ty).copied();
    ball.vel = golf::hit_golf_ball_velocity(velocity, rough_land_resistance, tile_below, materials);
    // `:448-454` — the projectile writes, in the C#'s order.
    ball.time_left = 18000;
    if ball.ai[1] < 0.0 {
        ball.ai[1] = 0.0;
    }
    ball.ai[1] += 1.0;
    ball.local_ai[1] = 1.0;
    // `:455` — RecordSwing reads the POST-write state (the C# passes the
    // live projectile after the ai[1]++/localAI[1] writes).
    let (cx, cy) = ball.center();
    golf_state.record_swing(&GolfBall {
        who_am_i,
        kind: ball.kind as i32,
        owner: ball.owner as i32,
        active: ball.active,
        position: ball.pos,
        center: Vec2::new(cx, cy),
        ai1: ball.ai[1],
        local_ai1: ball.local_ai[1],
    });
}

/// `Player.GetPreferredGolfBallToUse` (`Player.cs:49739-49770`): the held
/// item's golf-ball shoot wins, then the armour slots 19..0 (the port's
/// `equipped` kinds — `GetEffectiveArmor`'s loadout fold is the port's
/// direct slot), then the first golf-ball shoot in the 50 inventory slots.
/// The default is 721 (the dirt ball).
///
/// `held_shoot` is the selected item's `shoot` (0 = air/none);
/// `armor_shoots` the 20 equipped kinds' shoots; `inventory_shoots` the 50
/// slots' shoots. `IsItemSlotUnlockedAndUsable` is the journey-mode slot
/// lock — the port has no locked slots, so every armour slot is usable.
pub(crate) fn preferred_golf_ball(
    held_shoot: i32,
    armor_shoots: &[i32; 20],
    inventory_shoots: &[i32],
) -> u16 {
    let mut proj_type: u16 = GOLF_BALL;
    if held_shoot > 0 && is_a_golf_ball(held_shoot) {
        return held_shoot as u16;
    }
    for num in (0..20).rev() {
        let shoot = armor_shoots[num];
        if shoot > 0 && is_a_golf_ball(shoot) {
            return shoot as u16;
        }
    }
    for &shoot in inventory_shoots.iter().take(50) {
        if shoot > 0 && is_a_golf_ball(shoot) {
            proj_type = shoot as u16;
            break;
        }
    }
    proj_type
}

/// The tee-placement decision — the RNG-free half of
/// `Player.TryPlacingAGolfBallNearANearbyTee` (`Player.cs:49677-49737`):
/// the ±2-tile tee scan around `place_position` (the INNER loop breaks on
/// the first tee per column — at most one candidate per i), the nearest
/// candidate to `mouse_world` (the `num3 == -1f || num4 < num3` pick — the
/// FIRST candidate wins ties), then the reach-box test. Returns the chosen
/// tee cell when the placement should fire.
///
/// `reach` is the `TileReachCheckSettings.Simple` region
/// (`TileReachCheckSettings.cs:45-54`, `:81-85`: multiplier 1, limit 20)
/// clamped to [10, maxTiles-10] by the caller (`:49680-49683`).
pub(crate) fn tee_placement_target(
    world: &World,
    place_position: Vec2,
    mouse_world: Vec2,
    reach: (i32, i32, i32, i32),
) -> Option<(i32, i32)> {
    // `:49689` — ToTileCoordinates: truncate toward zero, then >> 4.
    let px = (place_position.x as i32) >> 4;
    let py = (place_position.y as i32) >> 4;
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for i in -2..=2 {
        for j in -2..=2 {
            let (x, y) = (px + i, py + j);
            // `WorldGen.InWorld(x, y, 1)` (`WorldGen.cs:9688-9695`).
            if x < 1 || x >= world.width - 1 || y < 1 || y >= world.height - 1 {
                continue;
            }
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && t.kind == TILE_GOLF_TEE)
            {
                candidates.push((x, y));
                break; // the inner-loop break (`:49703`)
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    // `:49711-49721` — the nearest-to-mouse pick; the candidate centre is
    // `(x, y) * 16 + (8, 8)`.
    let mut best_dist = -1.0f32;
    let mut best = candidates[0];
    for &(x, y) in &candidates {
        let c = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::splat(8.0);
        let d = (c - mouse_world).length();
        if best_dist == -1.0 || d < best_dist {
            best_dist = d;
            best = (x, y);
        }
    }
    // `:49722` — Collision.InTileBounds (`Collision.cs:3776-3783`): the
    // inclusive box test.
    let (lx, ly, hx, hy) = reach;
    if best.0 < lx || best.0 > hx || best.1 < ly || best.1 > hy {
        return None;
    }
    Some(best)
}

/// The full tee placement — the pool-side resolver: the decision above,
/// then the C#'s effects (`Player.cs:49724-49733`): EVERY owned golf ball
/// dies (no `active` gate — the C# loop tests type+owner only, and
/// `Kill()` on an inactive slot is a no-op), then the preferred ball
/// spawns at `(x*16 + 8.5, y*16 + 6)` with zero velocity, damage 0,
/// `ai0 = 0, ai1 = -1` (the tee sentinel). Returns whether the placement
/// fired (the C# `result`).
///
/// The spawn is centre-anchored (`NewProjectile` subtracts half the box,
/// `Projectile.cs:10876-10877`), so the port's top-left spawn subtracts
/// the row's half-box.
pub(crate) fn try_place_golf_ball_near_tee(
    pool: &mut ProjectilePool,
    world: &World,
    place_position: Vec2,
    mouse_world: Vec2,
    reach: (i32, i32, i32, i32),
    owner: u8,
    preferred: u16,
) -> bool {
    let Some((x, y)) = tee_placement_target(world, place_position, mouse_world, reach) else {
        return false;
    };
    // `:49725-49731` — the owned-ball cull (type+owner, no active gate).
    for q in pool.slots_mut() {
        if is_a_golf_ball(q.kind as i32) && q.owner == owner {
            q.active = false;
            q.time_left = 0;
        }
    }
    // `:49733` — the fresh ball on the tee.
    let row = crate::projectile::proj_data(preferred);
    let (hw, hh) = row.map_or((3.5, 3.5), |d| (d.w * 0.5, d.h * 0.5));
    pool.spawn_with_data_ai_owned(
        preferred,
        owner,
        x as f32 * 16.0 + 8.5 - hw,
        (y * 16) as f32 + 6.0 - hh,
        0.0,
        0.0,
        0,
        0.0,
        0.0,
        -1.0,
    );
    true
}

/// `TileReachCheckSettings.Simple.GetTileRegion`
/// (`TileReachCheckSettings.cs:20-54`, `:81-85`) + the caller's clamp
/// (`Player.cs:49680-49683`): the player's tile box widened by the reach
/// (min(tileRange, 20)), clamped to [10, maxTiles-10]. `tile_range_x/y`
/// are `Player.tileRangeX/Y` (the port's accessors).
pub(crate) fn simple_tee_reach(
    player_pos: Vec2,
    player_size: Vec2,
    tile_range_x: f32,
    tile_range_y: f32,
    world_w: i32,
    world_h: i32,
) -> (i32, i32, i32, i32) {
    let x = (tile_range_x as i32).min(20);
    let y = (tile_range_y as i32).min(20);
    let lx = ((player_pos.x / 16.0) as i32 - x).clamp(10, world_w - 10);
    let hx = (((player_pos.x + player_size.x) / 16.0).ceil() as i32 - 1 + x).clamp(10, world_w - 10);
    let ly = ((player_pos.y / 16.0) as i32 - y).clamp(10, world_h - 10);
    let hy = (((player_pos.y + player_size.y) / 16.0).ceil() as i32 - 1 + y).clamp(10, world_h - 10);
    (lx, ly, hx, hy)
}

/// `GolfHelper.ContactListener.EmitGolfballExplosion`
/// (`GolfHelper.cs:154-214`) — the DRAW half: the dust bodies are the
/// layer skip, but every explicit `Main.rand` draw stays on the stream in
/// the C#'s order (the module doc's RNG contract). The sound
/// (`SoundID.Item129`, `:156`) rides the returned cue.
///
/// Returns the cue `(sound id 2, style 129, x, y, pitch 0)`.
pub(crate) fn emit_golf_ball_explosion_draws(
    center: Vec2,
    rng: &mut dyn DropRng,
) -> (u16, i32, f32, f32, f32, f32) {
    // `:157-167` — the 0.085 ring: one NextFloat colour per dust.
    let mut num = 0.0f32;
    while num < 1.0 {
        let _ = rng.next_float();
        num += 0.085;
    }
    // `:168-169` — the hue seed + the spoke count.
    let _num2 = rng.next_float();
    let num3 = rng.next_range(5, 10);
    // `:170-188` — per spoke: the dust count, the radius wobble, then per
    // dust the NextVector2Circular pair (TWO NextFloats).
    for _i in 0..num3 {
        let num4 = rng.next_range(5, 22);
        let _ = rng.next_float();
        for _j in 0..num4 {
            let _ = rng.next_float();
            let _ = rng.next_float();
        }
    }
    // `:189-199` — the 0.2 ring: one NextFloat colour per dust.
    let mut num5 = 0.0f32;
    while num5 < 1.0 {
        let _ = rng.next_float();
        num5 += 0.2;
    }
    // `:200-213` — the NextFloatDirection, then the 0.15 ring's TWO
    // NextFloat colour draws per dust (`hslToRgb(NextFloat(), 1, 0.6 +
    // NextFloat()*0.2)`).
    let _num6 = rng.next_float();
    let mut num7 = 0.0f32;
    while num7 < 1.0 {
        let _ = rng.next_float();
        let _ = rng.next_float();
        num7 += 0.15;
    }
    (2, 129, center.x, center.y, 0.0, 1.0)
}

/// `PutBallInCup_TextAndEffects` (`GolfHelper.cs:115-123`): the hole-in-one
/// / stroke-count chat line. `numberOfHits == 0` prints nothing (the tee
/// ball dropped in untouched). The explosion's draws run only when the
/// line prints. Returns the chat text (`None` = silent) and the cue.
pub(crate) fn put_ball_in_cup_text(
    player_name: &str,
    number_of_hits: i32,
    proj_kind: u16,
    hit_location: (i32, i32),
    rng: &mut dyn DropRng,
) -> (Option<String>, Option<(u16, i32, f32, f32, f32, f32)>) {
    if number_of_hits == 0 {
        return (None, None);
    }
    // `:119` — the explosion at `hitLocation.ToWorldCoordinates(8, 0)`.
    let center = Vec2::new(hit_location.0 as f32 * 16.0 + 8.0, hit_location.1 as f32 * 16.0);
    let cue = emit_golf_ball_explosion_draws(center, rng);
    let ball_name = crate::lang::projectile_name(proj_kind as i32);
    let text = if number_of_hits == 1 {
        crate::lang::text_args(
            "Game.BallBounceResultGolf_Single",
            &[player_name.to_string(), ball_name],
        )
    } else {
        crate::lang::text_args(
            "Game.BallBounceResultGolf_Plural",
            &[player_name.to_string(), ball_name, number_of_hits.to_string()],
        )
    };
    (Some(text), Some(cue))
}

/// `GolfHelper.ContactListener.PutBallInCup` (`GolfHelper.cs:84-113`) —
/// the pool-side scoring half. `ball` is the captured slot (still active
/// at entry — the C# `projectile.active` gate at `:63`), `hit_location`
/// the `(ImpactPoint - Normal*0.5).ToTileCoordinates()` cell (`:88`).
///
/// Returns `(chat text, cue, hit_switch cell)` — the caller owns the chat
/// monitor, the audio queue and the wiring (`Wiring.HitSwitch`, `:105`).
/// The ball ALWAYS dies (`:112`), scored or not.
pub(crate) fn put_ball_in_cup(
    ball: &mut Projectile,
    who_am_i: usize,
    hit_location: (i32, i32),
    golf_state: &mut GolfState,
    golfer_score: &mut i32,
    player_name: &str,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> (Option<String>, Option<(u16, i32, f32, f32, f32, f32)>, Option<(i32, i32)>) {
    let mut out = (None, None, None);
    // `:86` — the scoring gate: the local owner AND the 90-tick delay.
    if ball.owner == local_owner && golf_state.should_score_hole() {
        let num = ball.ai[1] as i32;
        // `:92-95` — a multi-stroke hole pins the score timer to max.
        if num > 1 {
            golf_state.set_score_time();
        }
        // `:96-97` — RecordBallInfo + LandBall. The port's GolfState takes
        // the live-ball snapshot for RecordBallInfo's GetLastHitBall
        // re-validation; the ball is still live here (the Kill is `:112`).
        let (cx, cy) = ball.center();
        let snap = GolfBall {
            who_am_i,
            kind: ball.kind as i32,
            owner: ball.owner as i32,
            active: ball.active,
            position: ball.pos,
            center: Vec2::new(cx, cy),
            ai1: ball.ai[1],
            local_ai1: ball.local_ai[1],
        };
        // `Main.gameTimeCache.TotalGameTime.TotalSeconds` — the port has
        // no monotonic game-time clock at this seam; 0.0 keeps the
        // recorded-time field written (the 2 s camera window is the
        // unported consumer, `GolfState.cs:63`).
        golf_state.record_ball_info(&snap, Some(&snap), local_owner as i32, 0.0);
        golf_state.land_ball(&snap);
        let score = golf_state.get_golf_ball_score(&snap);
        // `:99-102` — the score accumulates only for a struck ball.
        if num > 0 {
            *golfer_score = crate::angler::accumulate_golfing_score(*golfer_score, score);
        }
        // `:103` — the text + effects.
        let (text, cue) = put_ball_in_cup_text(player_name, num, ball.kind, hit_location, rng);
        // `:104-105` — the timer reset + the wiring trip.
        golf_state.reset_score_time();
        out = (text, cue, Some(hit_location));
    }
    // `:112` — the unconditional kill.
    ball.active = false;
    ball.time_left = 0;
    out
}

/// `NewProjectile`'s golf-ball cull (`Projectile.cs:10932-10955`): when a
/// fresh damage-0 golf ball spawns for the local owner, the 11th+ active
/// damage-0 owned ball kills the OLDEST (lowest timeLeft). Runs AFTER the
/// spawn (the C# scans the pool with the new ball already in it).
pub(crate) fn cull_extra_golf_balls(pool: &mut ProjectilePool, owner: u8, spawned_kind: u16, damage: i32) {
    if !is_a_golf_ball(spawned_kind as i32) || damage > 0 {
        return;
    }
    let mut count = 0;
    let mut victim = 0usize;
    let mut oldest = 99999999i32;
    for (i, q) in pool.slots().iter().enumerate() {
        if q.active && is_a_golf_ball(q.kind as i32) && q.owner == owner && q.damage <= 0 {
            count += 1;
            if oldest > q.time_left {
                victim = i;
                oldest = q.time_left;
            }
        }
    }
    if count > 10 {
        let q = &mut pool.slots_mut()[victim];
        q.active = false;
        q.time_left = 0;
    }
}

/// The materials seam the pool passes until the retail `Materials.json`
/// lands (ANALYSIS "Golf (S1-E-9)") — re-exported so the dispatch cites
/// one name.
pub(crate) fn default_materials() -> DefaultMaterials {
    DefaultMaterials
}

/// The App-side tee placement — `Player.TryPlacingAGolfBallNearANearbyTee`
/// with the player/inventory reads resolved (`Player.cs:49677-49737`):
/// the reach region off the player's tile ranges, the preferred ball off
/// the held item / armour / inventory shoots, the spawn owned by the local
/// player. `place_position` is the caller's argument (the helper passes
/// `player.Center`, `Projectile.cs:52444`; the ball-item use passes the
/// mouse world point, `Player.cs:47523`).
pub(crate) fn try_place_golf_ball_near_tee_app(app: &mut crate::app::App, place_position: Vec2) -> bool {
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let reach = simple_tee_reach(
        Vec2::new(p.x, p.y),
        Vec2::new(crate::player::WIDTH, p.height),
        p.tile_range_x(),
        p.tile_range_y(),
        app.world.width,
        app.world.height,
    );
    let preferred = preferred_golf_ball_from_inventory(app.selected, &app.inventory, &p.equipped);
    let owner = app.local_id as u8;
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    let placed = try_place_golf_ball_near_tee(
        &mut app.projectiles,
        &app.world,
        place_position,
        mouse_world,
        reach,
        owner,
        preferred,
    );
    if placed {
        // `NewProjectile`'s >10-ball cull (`Projectile.cs:10932-10955`):
        // the tee spawn is damage 0 and owned by the local player, so the
        // `Owner == Main.myPlayer` gate holds.
        cull_extra_golf_balls(&mut app.projectiles, owner, preferred, 0);
    }
    placed
}

/// The disjoint-parts tee placement — the item-use path's borrow shape:
/// the caller holds `&mut Player` (the shoot block's `p`), so the App
/// cannot ride whole. The player reads (position/size/tile ranges) are
/// snapshotted by the caller; the pool/world/inventory are the App's
/// other fields.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_place_golf_ball_near_tee_parts(
    pool: &mut ProjectilePool,
    world: &World,
    place_position: Vec2,
    mouse_world: Vec2,
    player_pos: Vec2,
    player_size: Vec2,
    tile_range_x: f32,
    tile_range_y: f32,
    selected: usize,
    inventory: &[crate::item::ItemSlot],
    equipped: &[u16; 20],
    owner: u8,
) -> bool {
    let reach = simple_tee_reach(
        player_pos,
        player_size,
        tile_range_x,
        tile_range_y,
        world.width,
        world.height,
    );
    let preferred = preferred_golf_ball_from_inventory(selected, inventory, equipped);
    let placed = try_place_golf_ball_near_tee(
        pool,
        world,
        place_position,
        mouse_world,
        reach,
        owner,
        preferred,
    );
    if placed {
        cull_extra_golf_balls(pool, owner, preferred, 0);
    }
    placed
}

/// `GetPreferredGolfBallToUse`'s inventory reads (`Player.cs:49739-49770`):
/// the selected slot's shoot, the 20 equipped kinds' shoots, the 50
/// inventory slots' shoots — all resolved through the registry (an empty
/// slot reads shoot 0, the C#'s `IsAir` refusal).
pub(crate) fn preferred_golf_ball_from_inventory(
    selected: usize,
    inventory: &[crate::item::ItemSlot],
    equipped: &[u16; 20],
) -> u16 {
    let held = inventory
        .get(selected)
        .filter(|s| s.count > 0)
        .map(|s| s.kind)
        .unwrap_or(0);
    let held_shoot = crate::registry::projectile(held).unwrap_or(0) as i32;
    let mut armor_shoots = [0i32; 20];
    for (i, &kind) in equipped.iter().enumerate() {
        armor_shoots[i] = crate::registry::projectile(kind).unwrap_or(0) as i32;
    }
    let inventory_shoots: Vec<i32> = inventory
        .iter()
        .take(50)
        .map(|s| {
            if s.count > 0 {
                crate::registry::projectile(s.kind).unwrap_or(0) as i32
            } else {
                0
            }
        })
        .collect();
    preferred_golf_ball(held_shoot, &armor_shoots, &inventory_shoots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::GOLF_CLUB_HELPER;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::UnifiedRandom;

    fn world(w: i32, h: i32) -> World {
        World::new("golf-test".into(), w, h)
    }

    fn solid(w: &mut World, x: i32, y: i32, kind: u16) {
        let t = w.tile_mut(x, y).expect("in bounds");
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
    }

    fn ball(kind: u16, x: f32, y: f32) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = 7.0;
        p.h = 7.0;
        p
    }

    /// `:20333-20336` — the timeLeft floor fires below 10 and only then.
    #[test]
    fn the_golf_ball_keep_alive_floor_is_ten() {
        let w = world(64, 64);
        let mut p = ball(GOLF_BALL, 500.0, 500.0);
        p.time_left = 3;
        let mats = default_materials();
        let mut rng = UnifiedRandom::new(0);
        let _ = ai_golf_ball(&mut p, &w, &mats, 0, &mut rng);
        assert!(p.time_left >= 10, "the floor lifted it: {}", p.time_left);
        // The floor is a floor, not a reset: a long-lived ball keeps its
        // time (minus nothing — the AI itself never decrements).
        let mut q = ball(GOLF_BALL, 500.0, 500.0);
        q.time_left = 18000;
        let _ = ai_golf_ball(&mut q, &w, &mats, 0, &mut rng);
        assert_eq!(q.time_left, 18000);
    }

    /// `:20337-20346` — the tee sentinel: on an active 494 the ball holds
    /// (ai[1] stays -1, no physics); off it the sentinel clears with a
    /// netUpdate and the arm returns WITHOUT stepping.
    #[test]
    fn the_tee_sentinel_holds_until_the_tee_gone() {
        let mut w = world(64, 64);
        // The ball's Bottom is (503.5, 507) → tile (31, 31).
        solid(&mut w, 31, 31, TILE_GOLF_TEE);
        let mut p = ball(GOLF_BALL, 500.0, 500.0);
        p.ai[1] = -1.0;
        let before = p.pos;
        let mats = default_materials();
        let mut rng = UnifiedRandom::new(0);
        let _ = ai_golf_ball(&mut p, &w, &mats, 0, &mut rng);
        assert_eq!(p.ai[1], -1.0, "still teed");
        assert!(!p.net_update);
        assert_eq!(p.pos, before, "no physics while teed");
        // Remove the tee: the sentinel clears, netUpdate fires, and the
        // arm still returns before the step (the C#'s `return` at :20345).
        w.tile_mut(31, 31).unwrap().flags &= !FLAG_ACTIVE;
        let _ = ai_golf_ball(&mut p, &w, &mats, 0, &mut rng);
        assert_eq!(p.ai[1], 0.0);
        assert!(p.net_update);
        assert_eq!(p.pos, before, "the clear tick does not step either");
    }

    /// `:20348-20356` — a resting ball zeroes its damage and latches
    /// localAI[1] = 0 (BallState.Resting).
    #[test]
    fn a_resting_ball_zeroes_damage_and_latches_state_zero() {
        let mut w = world(64, 200);
        for x in 0..64 {
            solid(&mut w, x, 100, 1);
        }
        // The ball sits ON the floor row (tile y=100 spans 1600..1616):
        // its bottom edge at 1599.9 is 0.1 px above the surface, so the
        // step's gravity sub-step lands it flush and the Resting test
        // (`|vx| < 0.01 && -gravity < vy <= 0`) holds.
        let mut p = ball(GOLF_BALL, 500.0, 1592.9);
        p.vel = Vec2::new(0.0, -0.1);
        p.damage = 10;
        let mats = default_materials();
        let mut rng = UnifiedRandom::new(0);
        let _ = ai_golf_ball(&mut p, &w, &mats, 0, &mut rng);
        assert_eq!(p.damage, 0, "Resting zeroes the damage");
        assert_eq!(p.local_ai[1], 0.0, "the Resting latch");
    }

    /// `GolfHelper.cs:434-457` — the strike writes: timeLeft 18000, the
    /// negative ai[1] reset then ++, localAI[1] = 1, and RecordSwing's
    /// fresh-record-on-first-hit.
    #[test]
    fn the_strike_writes_the_four_fields_and_records_the_swing() {
        let w = world(64, 64);
        let mut p = ball(GOLF_BALL, 500.0, 500.0);
        p.ai[1] = -1.0; // the tee sentinel
        let mut state = GolfState::new();
        let mats = default_materials();
        strike_golf_ball(&mut p, 7, Vec2::new(10.0, 0.0), 0.0, &w, &mats, &mut state);
        assert_eq!(p.time_left, 18000);
        assert_eq!(p.ai[1], 1.0, "-1 → 0 → 1");
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.vel, Vec2::new(10.0, 0.0), "the identity material keeps the velocity");
        // The swing recorded: a second strike from 160 px away scores
        // (160/16)/(1+2) = 3 at the full timer.
        p.pos = Vec2::new(660.0, 500.0);
        strike_golf_ball(&mut p, 7, Vec2::new(10.0, 0.0), 0.0, &w, &mats, &mut state);
        assert_eq!(p.ai[1], 2.0);
        state.set_score_time();
        let (cx, cy) = p.center();
        let snap = GolfBall {
            who_am_i: 7,
            kind: p.kind as i32,
            owner: 0,
            active: true,
            position: p.pos,
            center: Vec2::new(cx, cy),
            ai1: p.ai[1],
            local_ai1: p.local_ai[1],
        };
        assert_eq!(state.get_golf_ball_score(&snap), 3);
    }

    /// `Player.cs:49739-49770` — the preference order: held > armour
    /// (19..0) > inventory (0..49), default 721.
    #[test]
    fn the_preferred_ball_follows_the_csharp_scan_order() {
        let armor = [0i32; 20];
        let inv = [0i32; 50];
        assert_eq!(preferred_golf_ball(0, &armor, &inv), 721);
        // The held item wins over everything.
        let mut a = armor;
        a[3] = 740;
        let mut i = inv;
        i[0] = 741;
        assert_eq!(preferred_golf_ball(739, &a, &i), 739);
        // The armour scan runs 19 DOWN to 0 — the higher slot wins.
        a[19] = 742;
        assert_eq!(preferred_golf_ball(0, &a, &i), 742);
        // The inventory scan runs 0 UP — the first slot wins.
        i[5] = 743;
        assert_eq!(preferred_golf_ball(0, &armor, &i), 741);
        // A non-golf shoot is skipped everywhere.
        assert_eq!(preferred_golf_ball(10, &armor, &inv), 721);
    }

    /// `Player.cs:49691-49707` — the tee scan's inner break: one candidate
    /// per column; the nearest-to-mouse pick; the reach box refusal.
    #[test]
    fn the_tee_scan_breaks_per_column_and_picks_the_nearest() {
        let mut w = world(64, 64);
        // The place position (500, 500) → tile (31, 31). Two tees in the
        // SAME column i = -2 (x = 29): only the first (j = -2, y = 29)
        // survives the inner break.
        solid(&mut w, 29, 29, TILE_GOLF_TEE);
        solid(&mut w, 29, 30, TILE_GOLF_TEE);
        // A second column's tee (x = 33, i = +2) closer to the mouse.
        solid(&mut w, 33, 31, TILE_GOLF_TEE);
        let reach = (10, 10, 54, 54);
        // The mouse near (33*16+8, 31*16+8) = (536, 504): the x=33 tee wins.
        let t = tee_placement_target(&w, Vec2::new(500.0, 500.0), Vec2::new(536.0, 504.0), reach);
        assert_eq!(t, Some((33, 31)));
        // The mouse near the x=29 column: the (29, 29) candidate wins —
        // the (29, 30) tee never entered the list (the inner break).
        let t = tee_placement_target(&w, Vec2::new(500.0, 500.0), Vec2::new(472.0, 472.0), reach);
        assert_eq!(t, Some((29, 29)));
        // Outside the reach box: no placement.
        let t = tee_placement_target(&w, Vec2::new(500.0, 500.0), Vec2::new(536.0, 504.0), (40, 40, 50, 50));
        assert_eq!(t, None);
    }

    /// `Projectile.cs:10932-10955` — the 11th damage-0 owned ball kills
    /// the lowest-timeLeft one; damage > 0 balls never cull.
    #[test]
    fn the_eleventh_ball_culls_the_oldest() {
        let mut pool = ProjectilePool::new();
        for i in 0..10 {
            let s = pool
                .spawn_with_data_ai_owned(GOLF_BALL, 0, 100.0, 100.0, 0.0, 0.0, 0, 0.0, 0.0, -1.0)
                .unwrap();
            pool.slots_mut()[s].time_left = 100 + i as i32;
        }
        // The 11th spawn: the cull kills the timeLeft-100 ball (slot 0).
        let s = pool
            .spawn_with_data_ai_owned(GOLF_BALL, 0, 100.0, 100.0, 0.0, 0.0, 0, 0.0, 0.0, -1.0)
            .unwrap();
        cull_extra_golf_balls(&mut pool, 0, GOLF_BALL, 0);
        assert!(!pool.slots()[0].active, "the oldest died");
        assert!(pool.slots()[s].active, "the fresh ball lives");
        assert_eq!(
            pool.slots().iter().filter(|q| q.active).count(),
            10,
            "back to ten"
        );
        // A damage-carrying spawn never culls.
        let s2 = pool
            .spawn_with_data_ai_owned(GOLF_BALL, 0, 100.0, 100.0, 0.0, 0.0, 5, 0.0, 0.0, 0.0)
            .unwrap();
        cull_extra_golf_balls(&mut pool, 0, GOLF_BALL, 5);
        assert!(pool.slots()[s2].active);
        assert_eq!(pool.slots().iter().filter(|q| q.active).count(), 11);
    }

    /// `GolfHelper.cs:154-214` — the explosion's draw count, pinned by a
    /// mirrored stream: 12 + 1 + 1 + per-spoke(1 + 1 + 2·count) + 5 + 1 +
    /// 14 draws. The f32 ring steps are the C#'s own accumulation.
    #[test]
    fn the_cup_explosion_draws_mirror_the_csharp_stream() {
        let mut live = UnifiedRandom::new(42);
        let _ = emit_golf_ball_explosion_draws(Vec2::new(100.0, 100.0), &mut live);
        let mut witness = UnifiedRandom::new(42);
        // The 0.085 ring: f32 accumulation — 12 iterations (0.085·11 =
        // 0.935 < 1, 0.085·12 = 1.02 ≥ 1).
        let mut n = 0.0f32;
        while n < 1.0 {
            let _ = witness.next_float();
            n += 0.085;
        }
        let _ = witness.next_float(); // the hue seed
        let spokes = witness.next_range(5, 10);
        for _ in 0..spokes {
            let count = witness.next_range(5, 22);
            let _ = witness.next_float();
            for _ in 0..count {
                let _ = witness.next_float();
                let _ = witness.next_float();
            }
        }
        let mut n = 0.0f32;
        while n < 1.0 {
            let _ = witness.next_float();
            n += 0.2;
        }
        let _ = witness.next_float(); // NextFloatDirection
        let mut n = 0.0f32;
        while n < 1.0 {
            let _ = witness.next_float();
            let _ = witness.next_float();
            n += 0.15;
        }
        assert_eq!(live, witness, "the explosion's draws replay identically");
    }

    /// `Projectile.cs:52446-52478` — the release strike: one
    /// NextFloatDirection per struck ball, the ai[0]/ai[1] transition, and
    /// the out-of-box ball is NOT struck (ValidateShot's legality box).
    #[test]
    fn the_release_strikes_every_valid_ball_and_draws_once_per_strike() {
        let mut helper = ball(GOLF_CLUB_HELPER, 500.0, 500.0);
        helper.w = 2.0;
        helper.h = 2.0;
        // Two owned balls: one in the box (player bottom (500, 542), the
        // ball centre (510, 540) → x +10, y -2: inside), one far away.
        let near = ball(GOLF_BALL, 506.5, 536.5);
        let far = ball(GOLF_BALL, 900.0, 900.0);
        let slots = vec![near, far, helper.clone()];
        let player = GolfClubPlayer {
            active: true,
            center: Vec2::new(500.0, 521.0),
            bottom: Vec2::new(500.0, 542.0),
            direction: 1,
            channel: false,
            use_animation: 12,
            held_item_type: 4039,
        };
        let mut rng = UnifiedRandom::new(7);
        let tick = ai_golf_club_helper(
            &mut helper,
            &slots,
            player,
            Vec2::new(700.0, 400.0),
            false,
            false,
            0,
            &mut rng,
        );
        assert_eq!(tick.strikes.len(), 1, "only the in-box ball struck");
        assert_eq!(tick.strikes[0].0, 0);
        assert_eq!(helper.ai[0], 1.0);
        assert_eq!(helper.ai[1], 1.0, "the follow-through tick counted");
        assert!(helper.net_update);
        // S1-E-21h d76 (`:52448`/`:52467`): the cue list — Item1 at the
        // PLAYER's center (500, 521), then one Item126 per struck ball at
        // its center (510, 540). The far ball is not struck → no cue.
        assert_eq!(tick.sounds.len(), 2, "Item1 + one Item126");
        assert_eq!(tick.sounds[0], (2, 1, 500.0, 521.0, 0.0, 1.0), "Item1 at player.Center");
        assert_eq!(tick.sounds[1], (2, 126, 510.0, 540.0, 0.0, 1.0), "Item126 at the ball");
        // Exactly ONE NextFloatDirection on the stream (the near ball).
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_float();
        assert_eq!(rng, witness, "one draw per struck ball");
    }

    /// `Projectile.cs:52479-52497` — the follow-through dies at
    /// useAnimation + 30 and the dummy time floors at 2.
    #[test]
    fn the_follow_through_dies_at_use_animation_plus_thirty() {
        let mut helper = ball(GOLF_CLUB_HELPER, 500.0, 500.0);
        helper.ai[0] = 1.0;
        helper.ai[1] = 40.0; // useAnimation 12 + 30 = 42; the tick makes 41
        let player = GolfClubPlayer {
            active: true,
            center: Vec2::new(500.0, 521.0),
            bottom: Vec2::new(500.0, 542.0),
            direction: 1,
            channel: true,
            use_animation: 12,
            held_item_type: 4039,
        };
        let mut rng = UnifiedRandom::new(0);
        let tick = ai_golf_club_helper(
            &mut helper,
            &[],
            player,
            Vec2::new(500.0, 500.0),
            false,
            false,
            0,
            &mut rng,
        );
        assert!(helper.active, "41 < 42: still alive");
        assert_eq!(tick.dummy_item_time, Some(2), "12 - 41 floors at 2");
        // One more tick: 42 >= 42 → dead.
        let _ = ai_golf_club_helper(
            &mut helper,
            &[],
            player,
            Vec2::new(500.0, 500.0),
            false,
            false,
            0,
            &mut rng,
        );
        assert!(!helper.active);
        assert_eq!(helper.time_left, 0);
    }

    /// `Projectile.cs:52536-52547` — the charge counter wraps at
    /// useAnimation*4*3 back to useAnimation*4, and the dummy time is
    /// useAnimation + min(useAnimation*4, ai[1]).
    #[test]
    fn the_charge_counter_wraps_and_the_dummy_time_tracks() {
        let mut helper = ball(GOLF_CLUB_HELPER, 500.0, 500.0);
        helper.ai[1] = 143.0; // useAnimation 12: num5 = 48, wrap at > 144
        let player = GolfClubPlayer {
            active: true,
            center: Vec2::new(500.0, 521.0),
            bottom: Vec2::new(500.0, 542.0),
            direction: 1,
            channel: true,
            use_animation: 12,
            held_item_type: 4039,
        };
        let mut rng = UnifiedRandom::new(0);
        // The helper's centre equals the mouse: no move, no facing flip;
        // pos == old_position (both ZERO-default... set them equal).
        helper.old_position = helper.pos;
        let mouse = helper.center().into();
        let tick = ai_golf_club_helper(
            &mut helper,
            &[],
            player,
            mouse,
            false,
            false,
            0,
            &mut rng,
        );
        assert_eq!(helper.ai[1], 144.0, "144 is not > 144: no wrap yet");
        assert_eq!(tick.dummy_item_time, Some(12 + 48), "12 + min(48, 144)");
        let mouse = helper.center().into();
        let _ = ai_golf_club_helper(
            &mut helper,
            &[],
            player,
            mouse,
            false,
            false,
            0,
            &mut rng,
        );
        assert_eq!(helper.ai[1], 48.0, "145 > 144: wrapped to 48");
    }

    /// `GolfHelper.cs:84-113` — the cup scores only past the 90-tick delay
    /// and for the local owner; the ball always dies; the multi-stroke
    /// hole pins the timer; the zero-hit ball prints nothing.
    #[test]
    fn the_cup_scores_past_the_delay_and_always_kills() {
        let mut state = GolfState::new();
        let mut p = ball(GOLF_BALL, 500.0, 500.0);
        p.ai[1] = 2.0;
        let mut score = 0;
        let mut rng = UnifiedRandom::new(0);
        // Before the delay: no score, no text, but the kill lands.
        let (text, cue, wire) = put_ball_in_cup(
            &mut p, 0, (31, 31), &mut state, &mut score, "Tester", 0, &mut rng,
        );
        assert!(text.is_none() && cue.is_none() && wire.is_none());
        assert!(!p.active, "the unconditional kill");
        assert_eq!(rng, UnifiedRandom::new(0), "no draws before the delay");
        // Past the delay, a fresh ball with a recorded swing history.
        let mut q = ball(GOLF_BALL, 500.0, 500.0);
        let mats = default_materials();
        let w = world(64, 64);
        strike_golf_ball(&mut q, 1, Vec2::new(10.0, 0.0), 0.0, &w, &mats, &mut state);
        q.pos = Vec2::new(660.0, 500.0);
        strike_golf_ball(&mut q, 1, Vec2::new(10.0, 0.0), 0.0, &w, &mats, &mut state);
        // The delay: 90 update ticks.
        for _ in 0..90 {
            state.update(4039, 0, 0, None);
        }
        // The ball rolls on to the cup at (820, 500): LandBall appends it,
        // so the record is [500, 660, 820] — total 320 px, 2 hit pairs.
        q.pos = Vec2::new(820.0, 500.0);
        let (text, cue, wire) = put_ball_in_cup(
            &mut q, 1, (31, 31), &mut state, &mut score, "Tester", 0, &mut rng,
        );
        assert!(text.is_some(), "the plural line prints");
        assert!(cue.is_some(), "the explosion cue");
        assert_eq!(wire, Some((31, 31)), "the wiring trip");
        assert!(!q.active);
        // The score accumulated: (320/16)/(2+2) = 5 at the pinned timer.
        assert_eq!(score, 5);
        // The timer reset: the next cup inside the delay scores nothing.
        assert!(!state.should_score_hole());
    }
}
