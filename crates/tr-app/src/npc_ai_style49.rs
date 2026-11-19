//! S1-D-3 (aiStyle-49) — the Angry Nimbus' machine: `NPC.AI()`'s
//! `else if (this.aiStyle == 49)` arm (`NPC.cs:31587-31650`), the ONLY
//! user of the style in the C# (`SetDefaults` case 250, `NPC.cs:12225`;
//! `NPCID.AngryNimbus = 250`, `NPCID.cs:11177`). The cloud hovers 200 px
//! ABOVE the player's centre (speed 4, accel 0.25 with the sign-reversal
//! double-add), and while it horizontally spans the player, floats
//! strictly above them and has a clear sight line, `ai[0]` counts up;
//! every 9th such tick (`ai[0] > 8f`) it drops a Rain projectile (264,
//! speed (0, 5), damage 20) from a random x inside its own body.
//!
//! **UNWIRED module (code-only mode, 2026-11-18).** The port's `NpcKind`
//! has no AngryNimbus variant yet and `npc/data.rs`'s id/ai_style/stat
//! tables are contested by the in-flight S1-D-3 sibling branches, so this
//! slice lands as pure functions with the kind-table rows + dispatch
//! wiring deferred to the family's merge step (the `npc_ai_style42.rs`
//! precedent, `render/special_tiles.rs` pattern). Intended call sites:
//! * [`angry_nimbus_ai`] — `update_rng`'s `match self.kind.ai_style()`
//!   axis (`npc.rs`), a `49 =>` arm once `NpcKind::AngryNimbus` carries
//!   `ai_style() == 49` (`NPC.cs:12225`).
//! * [`nimbus_step`] — the arm plus the generic movement tail the wiring
//!   needs: `Collision_MoveNormal` (`NPC.cs:92723-92726`, type 250 is not
//!   in `ApplyTileCollision`'s special list `:92612-92627`) +
//!   `Collision_MoveWhileDry` (`:92591-92607`). The 250 defaults set
//!   `noGravity` ONLY — `noTileCollide` stays false, so the cloud DOES
//!   collide with tiles (it rests on ceilings; it never falls).
//! * [`nimbus_find_frame`] — `FindFrame`'s `case 250` (`NPC.cs:72884-
//!   72897`, shared with 264/268): the frame advances every 6 ticks over
//!   the 4-frame sheet (`Main.npcFrameCount[250] = 4`, the initializer
//!   `Main.cs:67082`, row `:67109` index 250).
//!
//! ## RNG order (by construction)
//!
//! The arm's ONLY `Main.rand` draw is the shot's x pick
//! (`Main.rand.Next(base.width - 20)`, `NPC.cs:31646`) — ONE `Next(14)`
//! per 9th qualifying tick, drawn AFTER `ai[0]` resets to 0 and BEFORE
//! the `Projectile.NewProjectile` call. `TargetClosest`
//! (`NPC.cs:77652-77781`) is a deterministic scan with no `Next`. So the
//! machine takes the caller's stream and consumes exactly one `next_max`
//! per shot — witnessed by
//! `s1_d_3_s49_the_stream_advances_once_per_shot`.
//!
//! Deliberate stand-ins (recorded in docs/ANALYSIS.md "The Angry Nimbus'
//! aiStyle-49 machine"):
//! * `Main.netMode != 1` (`NPC.cs:31640`) folds to the host path (the
//!   port is single-player/host) — the rain gate always passes.
//! * The shot rides the plain `Npc::wants_hostile_shot` request list
//!   (the C# call passes no `ai0`/`ai1`, `NPC.cs:31647`); the drain
//!   (`world_sim/npcs.rs`) spawns it with knockback 0. The 264
//!   `SetDefaults` row (`Projectile.cs:3491-3502`: 4×40, aiStyle 45,
//!   hostile, penetrate −1, timeLeft 120, scale 1.1, extraUpdates 1,
//!   ignoreWater) and the projectile aiStyle-45 arm are M4-PROJ work.
//! * The wet/slope/conveyor collision halves (`NPC.cs:92545-92562`,
//!   `:92574-92578`) are unported engine layers; `nimbus_step` ports the
//!   dry, no-slope path only. The platform fall-through decision
//!   (`Collision_DecideFallThroughPlatforms`, `:92537`) DOES include the
//!   aiStyle-49 row (`:92952-92955` — `fall = true`): the cloud drops
//!   through platforms. The stand-in `collides_rect` walk does not model
//!   platforms, so the flag has no effect here — the wiring step passes
//!   `fall = true` into the shared `TileCollision` port.
//! * The thunder ambient roll (`NPC.cs:90111-90113`: `Next(700) == 0`
//!   then `Next(41, 44)`) is `UpdateNPC`'s sound layer — unported; when
//!   it lands it draws BEFORE the AI arm each tick. The `HitEffect`
//!   dust/gore rows (`:88959-88979`, cloud puff 54) are the dust-layer
//!   skip (the S1-D-3e/f precedent).
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::npc_target::TargetCandidate;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `SetDefaults` case 250 (`NPC.cs:12221-12233`): 34×26, aiStyle 49,
/// dmg 50, def 24, 300 HP, kb 0.3, value 300 copper (3 silver —
/// `this.value = 300f`, `:12232`); HitSound NPCHit30 / DeathSound
/// NPCDeath33 (`:12229-12230`). `noGravity` is set by the AI arm every
/// tick (`:31589`), NOT by the defaults row.
pub const ANGRY_NIMBUS: NimbusDefaults = NimbusDefaults {
    width: 34.0,
    height: 26.0,
    ai_style: 49,
    damage: 50,
    defense: 24,
    life_max: 300,
    knock_back_resist: 0.3,
    value: 300.0,
};

/// The cited defaults row's fields (the wiring step folds them into
/// `npc/data.rs`'s kind tables — the `npc_ai_style42.rs` `KindDefaults`
/// convention).
#[derive(Clone, Copy, Debug)]
pub struct NimbusDefaults {
    pub width: f32,
    pub height: f32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    /// `NPC.value` in copper — the literal `300f` (`NPC.cs:12232`).
    pub value: f32,
}

/// `Main.npcFrameCount[250] = 4` — the initializer `Main.cs:67082`:
/// ten entries per row starting at `:67084` (ids 0-9), so `:67109`
/// covers ids 250-259 (`4, 3, 5, 3, 3, 3, 4, 5, 6, 4`) — index 250 is
/// its first entry, 4.
pub const NIMBUS_FRAME_COUNT: u32 = 4;

/// `NPC.TargetClosest()` (`NPC.cs:31590`) through the shared selection
/// (`npc_target.rs`, `NPC.cs:77652-77781`) — the single session player
/// as slot 0, the stock aggro/no-aggro/tank values (the S1-D-3p
/// convention; the zombie's inline candidate, `npc.rs::update_zombie`).
/// RNG-free (`:77652-77781` has no `Next`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = TargetCandidate {
        active: true,
        dead: player.dead,
        // The port has no hardcore ghosts (`Player.ghost`).
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 49, 0);
}

/// One fixed step of the aiStyle-49 arm (`NPC.cs:31587-31650`). The
/// caller's stream is `Main.rand`: exactly one `Next(width - 20)` draw
/// per fired shot (`:31646`), zero otherwise. The shot is pushed onto
/// `Npc::wants_hostile_shot` as `(264, x, y, 0f, 5f, 20)` — the C#'s
/// `Projectile.NewProjectile` int-truncated spawn point (`:31646-31647`).
pub fn angry_nimbus_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    // `:31589` — noGravity is latched EVERY tick inside the arm. The
    // port carries no noGravity field (the integration choice is the
    // wiring's `nimbus_step`); nothing to write here.
    // `:31590` — TargetClosest every tick, before the aim.
    target_closest(npc, world, player);
    // `:31591-31592` — num735 (the speed) 4f, num736 (the accel) 0.25f.
    let num735 = 4.0f32;
    let num736 = 0.25f32;
    // `:31593-31595` — the aim: the player's CENTRE x, the player's
    // centre y MINUS 200f (the cloud hovers above, `vector88` = own
    // centre).
    let (cx, cy) = npc.center();
    let (pcx, pcy) = player.center();
    let mut num737 = pcx - cx;
    let mut num738 = pcy - cy - 200.0;
    // `:31596-31607` — normalize to the speed; a distance under 20f
    // keeps the CURRENT velocity as the aim (the strict `<`).
    let num739 = (num737 * num737 + num738 * num738).sqrt();
    if num739 < 20.0 {
        num737 = npc.vx;
        num738 = npc.vy;
    } else {
        let scale = num735 / num739;
        num737 *= scale;
        num738 *= scale;
    }
    // `:31608-31623` — the x approach: ±0.25 toward the aim, plus the
    // extra ±0.5 while the velocity still points the OTHER way (the
    // sign-reversal double-add — the comparisons are strict).
    if npc.vx < num737 {
        npc.vx += num736;
        if npc.vx < 0.0 && num737 > 0.0 {
            npc.vx += num736 * 2.0;
        }
    } else if npc.vx > num737 {
        npc.vx -= num736;
        if npc.vx > 0.0 && num737 < 0.0 {
            npc.vx -= num736 * 2.0;
        }
    }
    // `:31624-31639` — the same approach on y.
    if npc.vy < num738 {
        npc.vy += num736;
        if npc.vy < 0.0 && num738 > 0.0 {
            npc.vy += num736 * 2.0;
        }
    } else if npc.vy > num738 {
        npc.vy -= num736;
        if npc.vy > 0.0 && num738 < 0.0 {
            npc.vy -= num736 * 2.0;
        }
    }
    // `:31640-31649` — the rain window: the cloud's box horizontally
    // spans the player's box (`x + width > player.x && x < player.x +
    // player.width`), its BOTTOM edge is strictly ABOVE the player's
    // top (`y + height < player.y`), a clear sight line, and the host
    // path (`Main.netMode != 1` folds true — see the module doc).
    let (w, h) = npc.size();
    if npc.x + w > player.x
        && npc.x < player.x + crate::player::WIDTH
        && npc.y + h < player.y
        && crate::projectile::can_hit(
            world,
            npc.x,
            npc.y,
            w,
            h,
            player.x,
            player.y,
            crate::player::WIDTH,
            crate::player::HEIGHT,
        )
    {
        npc.ai[0] += 1.0;
        // `:31643-31648` — every 9th qualifying tick (1..8 count, 9
        // fires: `> 8f` strict), reset then the ONE rand draw.
        if npc.ai[0] > 8.0 {
            npc.ai[0] = 0.0;
            // `:31646` — `(int)(position.X + 10f + (float)Next(width - 20))`.
            // width 34 → Next(14), 0..=13. C# `(int)` truncates toward
            // zero; the port's positions are non-negative world pixels,
            // so `as i32` matches.
            let num740 = (npc.x + 10.0 + rng.next_max(w as i32 - 20) as f32) as i32;
            // `:31647` — Y: `(int)(position.Y + height + 4f)`, SpeedX 0,
            // SpeedY 5, Type 264, Damage 20, KnockBack 0.
            let y = (npc.y + h + 4.0) as i32;
            npc.wants_hostile_shot.push((264, num740 as f32, y as f32, 0.0, 5.0, 20));
        }
    }
}

/// The generic movement tail the arm rides in `UpdateNPC` — the dry,
/// no-slope path for type 250: `Collision_MoveNormal`
/// (`NPC.cs:92723-92726` — plain `Collision.TileCollision`, 250 has no
/// special arm in `ApplyTileCollision` `:92612-92627`) +
/// `Collision_MoveWhileDry` (`:92591-92607`: the `Collision.up` 0.01f
/// nudge, the `collideX`/`collideY` latches off `oldVelocity`, the
/// `oldPosition` stamp, `position += velocity`). The C#'s `fall` is the
/// `Collision_DecideFallThroughPlatforms` result — TRUE for aiStyle 49
/// (`:92952-92955`), so the cloud drops through platforms; the stand-in
/// walk does not model platforms (the wiring passes the flag into the
/// shared `TileCollision` port).
///
/// The port's stand-in `TileCollision` is the axis-separated
/// `collides_rect` walk (the zombie convention, `npc.rs::update_zombie`):
/// a blocked axis zeroes that velocity component instead of the C#'s
/// pixel-snap — the snap lands with the shared Collision port. The
/// slope/wet/conveyor layers (`:92545-92578`) are unported engine work.
pub fn nimbus_step(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    angry_nimbus_ai(npc, world, player, rng);
    let (w, h) = npc.size();
    let old_vx = npc.vx;
    let old_vy = npc.vy;
    // `Collision_MoveNormal` — the stand-in tile collision: a blocked
    // axis zeroes its velocity (no bounce — 250 rides the NORMAL arm,
    // not the Stardust Cell's 0.8 rebound `:92759-92773`).
    if crate::player::collides_rect(world, npc.x + npc.vx, npc.y, w, h) {
        npc.vx = 0.0;
    }
    if crate::player::collides_rect(world, npc.x, npc.y + npc.vy, w, h) {
        npc.vy = 0.0;
    }
    // `Collision_MoveWhileDry` (`:92593-92596`) — the ceiling nudge:
    // `Collision.up` means the tile walk found a solid directly above;
    // the stand-in tests the row above the head. The 0.01f keeps the
    // cloud pressed to the ceiling instead of stalling at vy 0.
    if crate::player::collides_rect(world, npc.x, npc.y - 1.0, w, h) {
        npc.vy = 0.01;
    }
    // `:92597-92604` — the collide latches compare oldVelocity.
    npc.collide_x = old_vx != npc.vx;
    npc.collide_y = old_vy != npc.vy;
    // `:92605-92607` — the stamps + the integration.
    npc.old_x = npc.x;
    npc.old_y = npc.y;
    npc.x += npc.vx;
    npc.y += npc.vy;
}

/// `FindFrame`'s `case 250` (`NPC.cs:72884-72897`, shared with the Rain
/// 264 / Sharknado 268): `frameCounter++`, at `>= 6.0` advance the frame
/// row and reset; wrap at `num * npcFrameCount[type]` (num = the frame
/// height 26 px + the sheet padding). The port's `frame` is the frame
/// INDEX (`npc.rs`'s convention), so the wrap is modulo the count.
/// Returns the new (frame index, counter); the wiring step drives
/// `Npc::frame` and the private `frame_counter` from `find_frame`
/// inside `npc.rs` — the pure twin here takes both by value.
pub fn nimbus_find_frame(frame: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame;
    if counter >= 6.0 {
        row += 1;
        counter = 0.0;
    }
    if row >= NIMBUS_FRAME_COUNT {
        row = 0;
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    // The kind-table rows are the wiring step; the tests ride a stand-in
    // kind with the Nimbus' 34x26 hitbox via size_override (the
    // npc_ai_style42 tests' convention).
    fn nimbus(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((ANGRY_NIMBUS.width, ANGRY_NIMBUS.height));
        n.max_hp = ANGRY_NIMBUS.life_max;
        n.hp = n.max_hp;
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// `NPC.cs:31591-31607` — the aim is the player's centre minus 200f
    /// on y, normalized to the 4f speed: a cloud far right of and level
    /// with the player accelerates LEFT and UP toward the hover point.
    #[test]
    fn s1_d_3_s49_the_aim_hovers_200_above_the_player() {
        let w = air();
        let p = player_at(100.0 * TILE_PX, 60.0 * TILE_PX);
        let mut n = nimbus(100.0 * TILE_PX + 400.0, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(0);
        angry_nimbus_ai(&mut n, &w, &p, &mut rng);
        assert!(n.vx < 0.0, "the player is left: vx {}", n.vx);
        assert!(n.vy < 0.0, "the hover point is above: vy {}", n.vy);
        // One plain-accel tick from rest: |v| = 0.25 per axis (the aim
        // points left-up, no sign reversal from 0).
        assert_eq!(n.vx, -0.25);
        assert_eq!(n.vy, -0.25);
        assert_eq!(n.target, 0, "TargetClosest picked the session player");
    }

    /// `NPC.cs:31596-31601` — a distance under 20f keeps the CURRENT
    /// velocity as the aim: at the hover point with velocity (1, -1),
    /// the approach drives both components toward themselves (no
    /// change); the bound is STRICT (`< 20f`).
    #[test]
    fn s1_d_3_s49_the_close_aim_keeps_the_current_velocity() {
        let w = air();
        // The hover point: player centre (px + 10, py + 21); the
        // cloud's centre must sit within 20 of (pcx, pcy - 200).
        let px = 100.0 * TILE_PX;
        let py = 60.0 * TILE_PX;
        // Centre exactly ON the hover point: distance 0 < 20.
        let mut n = nimbus(px + 10.0 - 17.0, py + 21.0 - 200.0 - 13.0);
        n.vx = 1.0;
        n.vy = -1.0;
        let p = player_at(px, py);
        let mut rng = UnifiedRandom::new(0);
        angry_nimbus_ai(&mut n, &w, &p, &mut rng);
        // aim == velocity → neither `<` nor `>` fires: unchanged.
        assert_eq!((n.vx, n.vy), (1.0, -1.0));
        // Exactly 20 away (dx +20, dy 0 — the cloud LEFT of the hover
        // point) is NOT < 20: the normalized aim is (4, 0) — vx
        // approaches 4, vy approaches 0.
        let mut n2 = nimbus(px + 10.0 - 17.0 - 20.0, py + 21.0 - 200.0 - 13.0);
        n2.vx = 1.0;
        n2.vy = -1.0;
        angry_nimbus_ai(&mut n2, &w, &p, &mut rng);
        assert_eq!(n2.vx, 1.25, "the aim is +4: plain accel");
        assert_eq!(n2.vy, -0.75, "the aim y is 0: +0.25 toward it");
    }

    /// `NPC.cs:31608-31639` — the sign-reversal double-add: moving
    /// RIGHT (+2) while the aim points LEFT adds 0.25 + 0.5 = 0.75 per
    /// tick; once the velocity crosses zero the extra add stops.
    #[test]
    fn s1_d_3_s49_the_sign_reversal_double_adds() {
        let w = air();
        let px = 100.0 * TILE_PX;
        let py = 60.0 * TILE_PX;
        // Far LEFT of the hover point → the aim x is +4... use far
        // RIGHT so the aim x is negative while vx starts positive.
        let mut n = nimbus(px + 600.0, py - 200.0);
        n.vx = 2.0;
        n.vy = 0.0;
        let p = player_at(px, py);
        let mut rng = UnifiedRandom::new(0);
        angry_nimbus_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.vx, 2.0 - 0.75, "0.25 + the 0.5 double-add");
        // From vx -0.1 with a negative aim: plain -0.25 (already
        // negative — the `vx > 0f` gate is false).
        let mut n2 = nimbus(px + 600.0, py - 200.0);
        n2.vx = -0.1;
        angry_nimbus_ai(&mut n2, &w, &p, &mut rng);
        assert_eq!(n2.vx, -0.35);
    }

    /// `NPC.cs:31640-31648` — the rain window: spanning + strictly
    /// above + sight → ai[0] counts; the 9th tick fires ONE shot with
    /// the cited payload and resets the counter.
    #[test]
    fn s1_d_3_s49_the_rain_fires_on_the_ninth_spanning_tick() {
        let w = air();
        let px = 100.0 * TILE_PX;
        let py = 60.0 * TILE_PX;
        // The cloud spans the player's x (both boxes overlap) and its
        // bottom (y + 26) is strictly above the player's top.
        let mut n = nimbus(px - 7.0, py - 100.0);
        let p = player_at(px, py);
        let mut rng = UnifiedRandom::new(0);
        for tick in 1..=8 {
            angry_nimbus_ai(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[0], tick as f32);
            assert!(n.wants_hostile_shot.is_empty(), "tick {tick}: no shot yet");
        }
        angry_nimbus_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 0.0, "the > 8f reset");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, sx, sy, svx, svy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, 264, "the Rain projectile");
        assert_eq!((svx, svy, dmg), (0.0, 5.0, 20));
        // The spawn x is (int)(x + 10 + Next(14)) — inside [x+10, x+23].
        assert!(sx >= n.x + 10.0 && sx <= n.x + 23.0, "spawn x {sx}");
        assert_eq!(sy, (n.y + 26.0 + 4.0).trunc(), "the bottom + 4f");
    }

    /// `NPC.cs:31640` — each window gate alone blocks the count: not
    /// spanning (left/right), NOT strictly above (bottom == player top
    /// is `<`-false), and a solid wall between the boxes (CanHit).
    #[test]
    fn s1_d_3_s49_the_window_gates_are_strict() {
        let px = 100.0 * TILE_PX;
        let py = 60.0 * TILE_PX;
        let p = player_at(px, py);
        let mut rng = UnifiedRandom::new(0);
        // Fully left: x + 34 == px → `>` false.
        let w = air();
        let mut n = nimbus(px - 34.0, py - 100.0);
        angry_nimbus_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 0.0, "the touching left edge does not span");
        // Fully right: x == px + 20 → `<` false.
        let mut n2 = nimbus(px + 20.0, py - 100.0);
        angry_nimbus_ai(&mut n2, &w, &p, &mut rng);
        assert_eq!(n2.ai[0], 0.0, "the touching right edge does not span");
        // Bottom edge level with the player's top: y + 26 == py → `<`
        // false.
        let mut n3 = nimbus(px - 7.0, py - 26.0);
        angry_nimbus_ai(&mut n3, &w, &p, &mut rng);
        assert_eq!(n3.ai[0], 0.0, "level bottoms are not strictly above");
        // One pixel higher counts.
        let mut n4 = nimbus(px - 7.0, py - 27.0);
        angry_nimbus_ai(&mut n4, &w, &p, &mut rng);
        assert_eq!(n4.ai[0], 1.0);
        // A solid slab between the cloud and the player blocks the
        // sight walk: the boxes overlap in x, so the Manhattan walk
        // (`projectile::can_hit`, `Collision.cs:381-479`) steps
        // VERTICALLY and tests the left/right neighbours — a 3-wide
        // solid row between the two centre rows fails the walk.
        let mut w2 = air();
        let centre_x = ((px + 10.0) / TILE_PX) as i32; // both centre columns
        let slab_row = ((py - 50.0) / TILE_PX) as i32; // between the centres
        for x in centre_x - 1..=centre_x + 1 {
            let i = w2.index(x, slab_row);
            w2.tiles[i] = Tile { kind: 1, ..Default::default() };
            w2.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n5 = nimbus(px - 7.0, py - 100.0);
        angry_nimbus_ai(&mut n5, &w2, &p, &mut rng);
        assert_eq!(n5.ai[0], 0.0, "the slab blocks the rain window");
    }

    /// RNG witness (written, NOT run — code-only mode): the ONLY draw
    /// is the shot's `Next(14)` (`NPC.cs:31646`). A run of 9 qualifying
    /// ticks consumes exactly ONE `next_max`; a control stream advanced
    /// once must match the carried stream draw-for-draw. The
    /// non-qualifying ticks consume ZERO.
    #[test]
    fn s1_d_3_s49_the_stream_advances_once_per_shot() {
        let w = air();
        let px = 100.0 * TILE_PX;
        let py = 60.0 * TILE_PX;
        let p = player_at(px, py);
        let mut carried = UnifiedRandom::new(123);
        let mut control = UnifiedRandom::new(123);
        let mut n = nimbus(px - 7.0, py - 100.0);
        // 8 non-firing ticks: the streams stay identical.
        for _ in 0..8 {
            angry_nimbus_ai(&mut n, &w, &p, &mut carried);
        }
        for _ in 0..8 {
            assert_eq!(carried.next_max(1000), control.next_max(1000));
        }
        // The 9th tick fires: exactly one Next(14) draw.
        let mut carried2 = UnifiedRandom::new(77);
        let mut control2 = UnifiedRandom::new(77);
        let mut n2 = nimbus(px - 7.0, py - 100.0);
        n2.ai[0] = 8.0;
        angry_nimbus_ai(&mut n2, &w, &p, &mut carried2);
        assert_eq!(n2.wants_hostile_shot.len(), 1);
        let expected_x = (n2.x + 10.0 + control2.next_max(14) as f32) as i32;
        assert_eq!(n2.wants_hostile_shot[0].1, expected_x as f32);
        for _ in 0..8 {
            assert_eq!(carried2.next_max(1000), control2.next_max(1000));
        }
    }

    /// `NPC.cs:92723-92726` + `:92591-92607` — the movement tail: free
    /// air integrates the velocity and stamps oldPosition; a blocked
    /// axis zeroes its velocity and latches the collide flag; the
    /// ceiling nudge writes vy 0.01.
    #[test]
    fn s1_d_3_s49_the_step_integrates_and_latches_collisions() {
        let w = air();
        let p = player_at(100.0 * TILE_PX, 60.0 * TILE_PX);
        let mut n = nimbus(100.0 * TILE_PX + 400.0, 40.0 * TILE_PX);
        n.vx = 1.5;
        n.vy = -0.5;
        let (x0, y0) = (n.x, n.y);
        let mut rng = UnifiedRandom::new(0);
        nimbus_step(&mut n, &w, &p, &mut rng);
        // The AI arm adjusts the velocity first (the aim is left-down
        // from up-right... vx 1.5 > aim < 0 → -0.75 double-add).
        assert_eq!(n.old_x, x0);
        assert_eq!(n.old_y, y0);
        assert!(!n.collide_x && !n.collide_y, "free air: no latches");
        // A ceiling directly above: the row above the head is solid →
        // the vy 0.01 nudge + the collideY latch (vy changed).
        let mut w2 = air();
        let head_row = ((40.0 * TILE_PX - 1.0) / TILE_PX) as i32;
        for x in 90..130 {
            let i = w2.index(x, head_row);
            w2.tiles[i] = Tile { kind: 1, ..Default::default() };
            w2.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n2 = nimbus(100.0 * TILE_PX + 400.0, 40.0 * TILE_PX);
        n2.vy = -0.5;
        nimbus_step(&mut n2, &w2, &p, &mut rng);
        assert!(n2.collide_y, "the ceiling changed vy");
        assert_eq!(n2.vy, 0.01, "the Collision.up nudge (:92595)");
    }

    /// `NPC.cs:72884-72897` — the frame clock: 6 ticks per frame, the
    /// wrap at the frame count (the `Main.npcFrameCount[250]` row).
    #[test]
    fn s1_d_3_s49_the_frame_advances_every_six_ticks_and_wraps() {
        let mut frame = 0u32;
        let mut counter = 0.0f32;
        for tick in 1..=6 * NIMBUS_FRAME_COUNT {
            (frame, counter) = nimbus_find_frame(frame, counter);
            if tick % 6 == 0 {
                assert_eq!(counter, 0.0);
                assert_eq!(frame, (tick / 6) % NIMBUS_FRAME_COUNT);
            } else {
                assert_eq!(counter, (tick % 6) as f32);
            }
        }
        assert_eq!(frame, 0, "the full cycle wraps to row 0");
    }

    /// The cited `SetDefaults` row (`NPC.cs:12221-12233`) — the wiring
    /// step's kind-table pin.
    #[test]
    fn s1_d_3_s49_the_defaults_row_matches_the_csharp_table() {
        assert_eq!((ANGRY_NIMBUS.width, ANGRY_NIMBUS.height), (34.0, 26.0));
        assert_eq!(ANGRY_NIMBUS.ai_style, 49);
        assert_eq!(ANGRY_NIMBUS.damage, 50);
        assert_eq!(ANGRY_NIMBUS.defense, 24);
        assert_eq!(ANGRY_NIMBUS.life_max, 300);
        assert_eq!(ANGRY_NIMBUS.knock_back_resist, 0.3);
        assert_eq!(ANGRY_NIMBUS.value, 300.0, "the literal 300f (:12232)");
    }
}
