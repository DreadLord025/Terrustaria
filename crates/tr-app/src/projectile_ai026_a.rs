//! AI_026 slice A (S1-D-4b-10) — the 14 early-return flyer pets of the
//! aiStyle-26 mega-family (`Projectile.cs:58102-65024`, 6 922 lines over
//! 68 types).
//!
//! # The slice boundary
//!
//! `AI_026`'s head (`:58102-59119`) runs the owner-inactive kill
//! (`:58104-58108`), the per-type pet keep-alive blocks (death clears the
//! player flag, a live flag pins `timeLeft = 2`), the `flag11`
//! (`ai[0] == -1 || -2`) pin (`:58944-58947`) and the head direction flags
//! `flag2`/`flag3` (`:59008-59026`). Then five flight arms return early:
//!
//! * the 175 BabyEater (`:59120-59243`),
//! * the 197 BabySkeletronHead (`:59244-59407`),
//! * the ten-type flyer group 198/380/774/815/817/886/892/894/901/1046
//!   (`:59408-59781`),
//! * the 211 Wisp (`:59782-59948`),
//! * the 199 TikiSpirit (`:59949-60091`).
//!
//! Every one of the 14 returns before the idle/walk/jump/attack machine
//! (`:60092+`), so slice A needs none of it — and the head's `flag2`/
//! `flag3`/`flag9` (petting) / `flag12`/`flag13` direction forcing are
//! consumed only by that machine: they have **no observable effect** on
//! these types and are not ported (the `flag11` pin IS observable — it
//! writes `timeLeft`). Everything else in `AI_026` (the ground pets, the
//! pygmies/spiders, the Pal summons, the 266 slime minion's hop machine)
//! is slice B (`task/S1-D-4b-11-melee-b`) — the two slices are disjoint by
//! construction.
//!
//! # RNG order (the golden rule)
//!
//! Per tick, in the C#'s evaluation order:
//!
//! * flyer group except 892/894: `Next(-10, 21)` for the Y jitter, THEN
//!   the X jitter (`:59446-59447`), before the `60 * -direction` / `-60`
//!   offsets;
//! * 197: `localAI[0] += Next(10)` (`:59357`) every tick, after the
//!   steering, before the spin/ease branch;
//! * teleports (175 `:59182`, 197 `:59314`, flyer group `:59655`, 199
//!   `:59996`): 12 dusts × (`NextFloat` for SpeedX, then `NextFloat` for
//!   SpeedY — the named arguments evaluate left-to-right). The dust
//!   VISUALS are unported (no `Main.dust` layer), but these 24 explicit
//!   `Main.rand` draws are CONSUMED so the pass's `UpdateProjectiles`
//!   stream stays aligned with the C# (the task's RNG-order constraint;
//!   the `ai_flying_minion` dust-skip precedent is deliberately NOT
//!   followed here — ANALYSIS "AI_026 slice A").
//! * The wisp's flight dust (`:59864`, `Dust.NewDust(..., 156)` with
//!   default speeds) draws no explicit `Main.rand` in `AI_026` — its rolls
//!   are all `Dust.NewDust` internals gated on the unported dust-pool
//!   occupancy (`Dust.cs:114-152`), so they are skipped with the layer
//!   (the same precedent as the teleport dusts' INTERNAL rolls
//!   `Dust.cs:170-190`).
//! * No head rand fires for these types (the pygmy `Next(2, 10)`
//!   `:58919` is slice B).
//!
//! # Documented stand-ins
//!
//! * `Collision.SolidCollision` → [`crate::projectile::box_solid`] (the
//!   5-point approximation, ANALYSIS-documented since M4-PROJ-8).
//! * `Main.myPlayer == owner → netUpdate = true` → no-op (single-player
//!   port, the pool's standing convention).
//! * `base.wet` → the slot carries no liquid-wet flags (the `:5473`
//!   precedent), so the 774/1046 frame offsets read the `!wet` branch
//!   always — ANALYSIS.
//! * `Player.mount.Active && mount.CanFly` → `mount.is_active() &&
//!   mount.can_fly()` (the port's `CanFly(player)` overload folds to the
//!   table check, `mount.rs:2450`).
//! * The pet keep-alive flags' GRANT side (the pet buff/item pass) is
//!   unported — the flags stay false until it lands; the AI-side
//!   clear/pin behaviour is exact.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{
    BABY_EATER, BABY_HORNET, BABY_SKELETRON_HEAD, DD2_BETSY_PET, DUKE_FISHRON_PET,
    GLITTERY_BUTTERFLY, LIL_HARPY, MOON_LORD_PET, PUFFERFISH, Projectile, QUEEN_BEE_PET,
    SHARK_PUP, TIKI_SPIRIT, WISP, ZEPHYR_FISH, box_solid, proj_frames,
};

/// The slice-A type set — the 14 early-return flyers. The dispatch gate
/// (`projectile.rs`'s `26 =>` arm) routes ONLY these; every other
/// aiStyle-26 kind (the 266 slime minion today, slice B's types tomorrow)
/// keeps the pre-existing no-arm fall-through, byte-identical.
pub(crate) const AI_026_A_KINDS: [u16; 14] = [
    BABY_EATER,
    BABY_SKELETRON_HEAD,
    BABY_HORNET,
    TIKI_SPIRIT,
    WISP,
    ZEPHYR_FISH,
    SHARK_PUP,
    LIL_HARPY,
    GLITTERY_BUTTERFLY,
    QUEEN_BEE_PET,
    DUKE_FISHRON_PET,
    MOON_LORD_PET,
    DD2_BETSY_PET,
    PUFFERFISH,
];

/// The owner reads the five arms share — a snapshot of
/// `Main.player[this.owner]` taken once per tick (the C# re-reads the
/// fields, but nothing in the slice writes them mid-arm except the
/// keep-alive flag clear, which no arm reads back).
struct OwnerView {
    center: Vec2,
    /// `position.Y + height` — the grounded-reset bottom comparison.
    bottom: f32,
    vel: Vec2,
    /// `Player.direction` (the port's `facing`).
    dir: f32,
    /// `Player.miscCounter` — the 774/1046 sine hover phase.
    misc_counter: u32,
    gfx_off_y: f32,
    control_left: bool,
    control_right: bool,
    control_up: bool,
    control_down: bool,
    /// `mount.Active && mount.CanFly(player)` — the wisp's flight-exit
    /// alternative (`:59821`).
    mount_flying: bool,
}

impl OwnerView {
    fn capture(p: &Player) -> Self {
        let (cx, cy) = p.center();
        Self {
            center: Vec2::new(cx, cy),
            bottom: p.y + p.height,
            vel: Vec2::new(p.vx, p.vy),
            dir: p.facing as f32,
            misc_counter: p.misc_counter,
            gfx_off_y: p.gfx_off_y,
            control_left: p.control_left,
            control_right: p.control_right,
            control_up: p.control_jump, // the port's merged jump binding carries controlUp (W/ArrowUp feed control_jump — player.rs:2260-2264, the sleeping/sitting exit-mask precedent)
            control_down: p.control_down,
            mount_flying: p.mount.is_active() && p.mount.can_fly(),
        }
    }
}

/// The dispatch seam: `AI()`'s `else if (this.aiStyle == 26) this.AI_026();`
/// (`Projectile.cs:26416-26419`), gated to the slice-A kinds. Returns
/// `false` for every other kind — the caller keeps its `_ => {}`
/// fall-through, so non-slice routing is byte-identical.
///
/// On a slice-A tick this runs the observable head (the owner-inactive
/// kill, the keep-alive pin, the `flag11` pin), the type's flight arm,
/// and `UpdatePosition`'s `position += velocity` (`:20433-20446` — the
/// aiStyle-26 kinds are not in the skip list and the arms set
/// `tileCollide = false`, so `HandleMovement`'s collision half never
/// runs; the `wet` branch folds to the dry one, module doc).
pub(crate) fn ai026_a_tick(
    p: &mut Projectile,
    world: &World,
    player: Option<&mut Player>,
    rng: &mut dyn DropRng,
) -> bool {
    if !AI_026_A_KINDS.contains(&p.kind) {
        return false;
    }
    // `:58104-58108` — `if (!Main.player[this.owner].active) { this.active
    // = false; return; }`. The port's `None` player is the unresolved /
    // inactive owner (the `ai_hook` dead-owner precedent).
    let Some(player) = player else {
        p.active = false;
        return true;
    };
    keep_alive(p, player);
    // `:58200-58202` + `:58944-58947` — `flag11 = ai[0] == -1 || -2` pins
    // `timeLeft = 2` (the despawn request the buff pass writes).
    if p.ai[0] == -1.0 || p.ai[0] == -2.0 {
        p.time_left = 2;
    }
    // The head's `flag2`/`flag3` direction band (`:59008-59026`, num = 85
    // for every slice-A type — none of the `num` overrides `:58120-58199`
    // matches) and the `flag12`/`flag13` forcing (`:59016-59026`) feed
    // only the idle machine the 14 types never reach — not ported.
    let owner = OwnerView::capture(player);
    match p.kind {
        BABY_EATER => eater_arm(p, world, &owner, rng),
        BABY_SKELETRON_HEAD => skeletron_head_arm(p, world, &owner, rng),
        WISP => wisp_arm(p, &owner),
        TIKI_SPIRIT => tiki_arm(p, world, &owner, rng),
        _ => flyer_group_arm(p, world, &owner, rng),
    }
    // `UpdatePosition` (`:19858` → `:20445`): the dry `position +=
    // velocity` rides every tick — the arms' early returns leave
    // `AI_026`, not `Update`, so the integration still runs (the tiki's
    // `frameCounter <= 6` return `:60047` included).
    p.pos += p.vel;
    true
}

/// The per-type pet keep-alive block — death clears the player flag, a
/// live flag pins `timeLeft = 2`. The `held` read precedes the clear, so
/// a flag standing on the death tick still pins once (the C#'s order;
/// the `ai_baby_bird` precedent `projectile.rs:7780-7793`).
fn keep_alive(p: &mut Projectile, player: &mut Player) {
    let dead = player.dead;
    // The `held` read precedes the clear — the C# reads the flag into the
    // `if` before the death branch writes it.
    let held = match p.kind {
        BABY_EATER => player.eater,
        BABY_SKELETRON_HEAD => player.skeletron,
        BABY_HORNET => player.hornet,
        TIKI_SPIRIT => player.tiki,
        WISP => player.wisp,
        ZEPHYR_FISH => player.zephyrfish,
        SHARK_PUP => player.pet_flag_baby_shark,
        LIL_HARPY => player.pet_flag_lil_harpy,
        GLITTERY_BUTTERFLY => player.pet_flag_glittery_butterfly,
        QUEEN_BEE_PET => player.pet_flag_queen_bee_pet,
        DUKE_FISHRON_PET => player.pet_flag_duke_fishron_pet,
        MOON_LORD_PET => player.pet_flag_moon_lord_pet,
        DD2_BETSY_PET => player.pet_flag_dd2_betsy_pet,
        PUFFERFISH => player.pet_flag_pufferfish,
        _ => return,
    };
    if dead {
        match p.kind {
            // `:58284-58294`
            BABY_EATER => player.eater = false,
            // `:58295-58305`
            BABY_SKELETRON_HEAD => player.skeletron = false,
            // `:58306-58316`
            BABY_HORNET => player.hornet = false,
            // `:58317-58327`
            TIKI_SPIRIT => player.tiki = false,
            // `:58405-58415`
            WISP => player.wisp = false,
            // `:58493-58503`
            ZEPHYR_FISH => player.zephyrfish = false,
            // `:58526-58536`
            SHARK_PUP => player.pet_flag_baby_shark = false,
            // `:58537-58547`
            LIL_HARPY => player.pet_flag_lil_harpy = false,
            // `:58559-58569`
            GLITTERY_BUTTERFLY => player.pet_flag_glittery_butterfly = false,
            // `:58680-58690`
            QUEEN_BEE_PET => player.pet_flag_queen_bee_pet = false,
            // `:58724-58734`
            DUKE_FISHRON_PET => player.pet_flag_duke_fishron_pet = false,
            // `:58735-58745`
            MOON_LORD_PET => player.pet_flag_moon_lord_pet = false,
            // `:58779-58789`
            DD2_BETSY_PET => player.pet_flag_dd2_betsy_pet = false,
            // `:58900-58910`
            PUFFERFISH => player.pet_flag_pufferfish = false,
            _ => {}
        }
    }
    if held {
        p.time_left = 2;
    }
}

/// The grounded-reset shared by the 175/197/flyer-group/199 arms
/// (`:59135-59142`, `:59267-59274`, `:59488-59495`, `:59966-59973`):
/// close (`dist < range`), the owner stands (`velocity.Y == 0`), the
/// flyer's bottom is at or above the owner's bottom and its box is clear
/// of solids → `ai[0] = 0` and the upward velocity floors at -6.
fn grounded_reset(p: &mut Projectile, world: &World, owner: &OwnerView, dist: f32, range: f32) {
    if dist < range
        && owner.vel.y == 0.0
        && p.pos.y + p.h <= owner.bottom
        && !box_solid(world, p.pos, p.w, p.h)
    {
        p.ai[0] = 0.0;
        if p.vel.y < -6.0 {
            p.vel.y = -6.0;
        }
    }
}

/// The >2000-px teleport shared by the 175/197/flyer-group/199 arms
/// (`:59177-59192`, `:59309-59324`, `:59638-59665`, `:59991-60006`):
/// 12 dusts whose SpeedX/SpeedY each draw `1 - NextFloat() * 2` — the 24
/// explicit `Main.rand.NextFloat()` calls are consumed in order (X then
/// Y per dust, module doc); the dust visuals and their pool-gated
/// internal rolls are skipped. Then `Center = player.Center`,
/// `velocity = Zero`; the `netUpdate` write is the single-player no-op.
fn teleport_to_owner(p: &mut Projectile, owner: &OwnerView, rng: &mut dyn DropRng) {
    for _ in 0..12 {
        let _speed_x = 1.0 - rng.next_float() * 2.0;
        let _speed_y = 1.0 - rng.next_float() * 2.0;
    }
    p.pos = owner.center - Vec2::new(p.w * 0.5, p.h * 0.5);
    p.vel = Vec2::ZERO;
}

/// The 175 BabyEater arm (`Projectile.cs:59120-59243`): speed 7, accel
/// 0.1 (0.01 inside 150 with the ±2 target clamp, 0.2 beyond 300), the
/// dominant-axis steering gate, `rotation = atan2(vy, vx) - 1.57`, the
/// 7-tick 2-frame cycle.
fn eater_arm(p: &mut Projectile, world: &World, owner: &OwnerView, rng: &mut dyn DropRng) {
    let mut accel = 0.1f32; // num20
    p.tile_collide = false; // :59123
    let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5); // vector4
    let mut dx = owner.center.x - center.x; // num22
    let mut dy = owner.center.y - center.y; // num23
    let mut dist = (dx * dx + dy * dy).sqrt(); // num24
    let speed = 7.0f32; // num25
    let teleport = dist > 2000.0; // flag14
    grounded_reset(p, world, owner, dist, 300.0); // num21
    if dist < 150.0 {
        // `:59143-59166` — the close damp + the ±2 clamp.
        if p.vel.x.abs() > 2.0 || p.vel.y.abs() > 2.0 {
            p.vel *= 0.99;
        }
        accel = 0.01;
        dx = dx.clamp(-2.0, 2.0);
        dy = dy.clamp(-2.0, 2.0);
    } else {
        // `:59167-59176` — the far accel + the speed normalization.
        if dist > 300.0 {
            accel = 0.2;
        }
        let scale = speed / dist;
        dx *= scale;
        dy *= scale;
    }
    if teleport {
        teleport_to_owner(p, owner, rng); // dust type 17 (:59179)
    }
    // `:59193-59230` — the dominant-axis gate. The `num20 == 0.05f`
    // disjunct is dead for this arm (accel ∈ {0.01, 0.1, 0.2}); both
    // branches' double-step fires only when accel > 0.05.
    if dx.abs() > dy.abs() {
        if p.vel.x < dx {
            p.vel.x += accel;
            if accel > 0.05 && p.vel.x < 0.0 {
                p.vel.x += accel;
            }
        }
        if p.vel.x > dx {
            p.vel.x -= accel;
            if accel > 0.05 && p.vel.x > 0.0 {
                p.vel.x -= accel;
            }
        }
    }
    if dx.abs() <= dy.abs() {
        if p.vel.y < dy {
            p.vel.y += accel;
            if accel > 0.05 && p.vel.y < 0.0 {
                p.vel.y += accel;
            }
        }
        if p.vel.y > dy {
            p.vel.y -= accel;
            if accel > 0.05 && p.vel.y > 0.0 {
                p.vel.y -= accel;
            }
        }
    }
    // `:59231-59241` — the rotation + the 2-frame cycle.
    p.rot = p.vel.y.atan2(p.vel.x) - 1.57;
    p.frame_counter += 1;
    if p.frame_counter > 6 {
        p.frame += 1;
        p.frame_counter = 0;
    }
    if p.frame > 1 {
        p.frame = 0;
    }
}

/// The 197 BabySkeletronHead arm (`Projectile.cs:59244-59407`): speed 3
/// (14 once `localAI[0] >= 10000`, latched at dist > 500), both-axis
/// steering, the `localAI[0] += Next(10)` accumulator with the
/// 0.25-rad/tick spin above 10000 (reset above 12000) and the
/// `vx * 0.1` rotation ease below, the ±6.28 wrap.
fn skeletron_head_arm(p: &mut Projectile, world: &World, owner: &OwnerView, rng: &mut dyn DropRng) {
    let mut accel = 0.1f32; // num28
    p.tile_collide = false; // :59247
    let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5); // vector5
    let mut dx = owner.center.x - center.x; // num30
    let mut dy = owner.center.y - center.y; // num31
    let mut dist = (dx * dx + dy * dy).sqrt(); // num32
    let mut speed = 3.0f32; // num33
    if dist > 500.0 {
        p.local_ai[0] = 10000.0; // :59258-59261
    }
    if p.local_ai[0] >= 10000.0 {
        speed = 14.0;
    }
    let teleport = dist > 2000.0; // flag15
    grounded_reset(p, world, owner, dist, 300.0); // num29
    if dist < 150.0 {
        // `:59275-59298` — the close damp + the ±2 clamp.
        if p.vel.x.abs() > 2.0 || p.vel.y.abs() > 2.0 {
            p.vel *= 0.99;
        }
        accel = 0.01;
        dx = dx.clamp(-2.0, 2.0);
        dy = dy.clamp(-2.0, 2.0);
    } else {
        // `:59299-59308`.
        if dist > 300.0 {
            accel = 0.2;
        }
        let scale = speed / dist;
        dx *= scale;
        dy *= scale;
    }
    if teleport {
        teleport_to_owner(p, owner, rng); // dust type 26 (:59311)
    }
    // `:59325-59356` — both axes steer, no dominant-axis gate.
    if p.vel.x < dx {
        p.vel.x += accel;
        if accel > 0.05 && p.vel.x < 0.0 {
            p.vel.x += accel;
        }
    }
    if p.vel.x > dx {
        p.vel.x -= accel;
        if accel > 0.05 && p.vel.x > 0.0 {
            p.vel.x -= accel;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if accel > 0.05 && p.vel.y < 0.0 {
            p.vel.y += accel;
        }
    }
    if p.vel.y > dy {
        p.vel.y -= accel;
        if accel > 0.05 && p.vel.y > 0.0 {
            p.vel.y -= accel;
        }
    }
    // `:59357-59397` — the spin accumulator (the arm's only rand draw).
    p.local_ai[0] += rng.next_range(0, 10) as f32;
    if p.local_ai[0] > 10000.0 {
        if p.local_ai[1] == 0.0 {
            p.local_ai[1] = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
        }
        p.rot += 0.25 * p.local_ai[1];
        if p.local_ai[0] > 12000.0 {
            p.local_ai[0] = 0.0;
        }
    } else {
        p.local_ai[1] = 0.0;
        let target = p.vel.x * 0.1; // num36
        if p.rot > target {
            p.rot -= (p.vel.x.abs() + p.vel.y.abs()) * 0.01;
            if p.rot < target {
                p.rot = target;
            }
        }
        if p.rot < target {
            p.rot += (p.vel.x.abs() + p.vel.y.abs()) * 0.01;
            if p.rot > target {
                p.rot = target;
            }
        }
    }
    // `:59398-59405` — the ±6.28 wrap (single-step, as the C#).
    if p.rot > 6.28 {
        p.rot -= 6.28;
    }
    if p.rot < -6.28 {
        p.rot += 6.28;
    }
}

/// The ten-type flyer group (`Projectile.cs:59408-59781`): the 198
/// BabyHornet, 380 ZephyrFish, 774 SharkPup, 815 LilHarpy, 817
/// GlitteryButterfly, 886 QueenBeePet, 892 DukeFishronPet, 894
/// MoonLordPet, 901 DD2BetsyPet, 1046 Pufferfish.
fn flyer_group_arm(p: &mut Projectile, world: &World, owner: &OwnerView, rng: &mut dyn DropRng) {
    let kind = p.kind;
    // `:59410-59418` — num37.
    let mut accel = if kind == ZEPHYR_FISH || kind == SHARK_PUP || kind == PUFFERFISH {
        0.3
    } else {
        0.4
    };
    p.tile_collide = false; // :59419
    // `:59420-59424` — num38/num39/num40/num41/num42.
    let close_range = if kind == SHARK_PUP || kind == PUFFERFISH { 2.0 } else { 50.0 };
    let far_speed_dist = 400.0f32;
    let half_dist = 200.0f32;
    let teleport_dist = 2000.0f32;
    let mut teleport = false; // flag16
    let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5); // vector6
    let mut dx = owner.center.x - center.x; // num43
    let mut dy = owner.center.y - center.y; // num44
    if kind == SHARK_PUP || kind == PUFFERFISH {
        // `:59429-59441` — the sine hover. `sin(2π * miscCounter / 60)`
        // and its clamped `GetLerpValue(0.5, 1, ·)` (`Utils.cs:315-343`);
        // both zero when the PRE-offset target is more than 50 px away.
        let phase = std::f32::consts::PI * 2.0 * (owner.misc_counter as f32 / 60.0);
        let mut sine = phase.sin(); // num45
        let mut lerp = ((sine - 0.5) / 0.5).clamp(0.0, 1.0); // num46
        if Vec2::new(dx - 70.0 * owner.dir, dy - 60.0).length() > 50.0 {
            sine = 0.0;
            lerp = 0.0;
        }
        dy += -60.0 + lerp * -8.0 + sine * 8.0;
        dx += 70.0 * -owner.dir;
    } else {
        // `:59442-59451` — the jitter (Y first, then X — the C#'s line
        // order) for every group type except the 892/894 pair, then the
        // `60 * -direction` / `-60` offsets.
        if kind != DUKE_FISHRON_PET && kind != MOON_LORD_PET {
            dy += rng.next_range(-10, 21) as f32;
            dx += rng.next_range(-10, 21) as f32;
        }
        dx += 60.0 * -owner.dir;
        dy -= 60.0;
    }
    let target = Vec2::new(dx, dy); // vector7 (the idle-flip length read)
    let mut dist = (dx * dx + dy * dy).sqrt(); // num47
    let original_dist = dist; // num48
    // `:59459-59487` — the speed ladder (num49).
    let mut speed = 14.0f32;
    if kind == ZEPHYR_FISH {
        speed = 6.0;
    }
    if kind == LIL_HARPY || kind == GLITTERY_BUTTERFLY {
        speed = if dist < far_speed_dist { 6.0 } else { 10.0 };
    }
    if kind == DUKE_FISHRON_PET || kind == MOON_LORD_PET || kind == DD2_BETSY_PET {
        if dist < half_dist {
            speed = 6.0;
        }
        // The C# writes `num49 = 6f` above and then UNCONDITIONALLY
        // overwrites it (`:59474`) — the 6 is dead. Ported verbatim (both
        // assignments run; only the second survives).
        speed = if dist < far_speed_dist { 9.0 } else { 12.0 };
    }
    if kind == SHARK_PUP {
        speed = 5.0;
    }
    if kind == PUFFERFISH {
        speed = 5.0;
        if dist >= far_speed_dist {
            speed = 8.0;
        }
    }
    grounded_reset(p, world, owner, dist, 100.0); // num38 (:59488-59495)
    if dist < close_range {
        // `:59496-59510` — the close damp. The C#'s `type == 892 || type
        // == 892` (`:59500`) is a source typo — only the 892 damps 0.95.
        if p.vel.x.abs() > 2.0 || p.vel.y.abs() > 2.0 {
            if kind == DUKE_FISHRON_PET {
                p.vel *= 0.95;
            } else {
                p.vel *= 0.99;
            }
        }
        accel = 0.01;
    } else {
        // `:59511-59595` — the per-type far accel ladder.
        if kind == DUKE_FISHRON_PET || kind == MOON_LORD_PET || kind == DD2_BETSY_PET {
            if dist < 100.0 {
                accel = 0.1;
            }
            if dist > teleport_dist {
                teleport = true;
            } else if dist > far_speed_dist {
                accel = 0.7;
            } else if dist > half_dist {
                accel = 0.5;
            }
        } else if kind == LIL_HARPY || kind == GLITTERY_BUTTERFLY {
            if dist < 100.0 {
                accel = 0.1;
            }
            if dist > teleport_dist {
                teleport = true;
            } else if kind != LIL_HARPY && dist > far_speed_dist {
                accel = 0.5;
            }
        } else if kind == ZEPHYR_FISH {
            if dist < 100.0 {
                accel = 0.1;
            }
            if dist > 300.0 {
                accel = 0.4;
            }
            if dist > teleport_dist {
                teleport = true;
            }
        } else if kind == BABY_HORNET || kind == QUEEN_BEE_PET {
            if dist < 100.0 {
                accel = 0.1;
            }
            if dist > 300.0 {
                accel = 0.6;
            }
            if dist > teleport_dist {
                teleport = true;
            }
        } else if kind == SHARK_PUP || kind == PUFFERFISH {
            if dist < 40.0 {
                accel = 0.1;
            }
            if dist > 300.0 {
                accel = 0.6;
            }
            if dist > teleport_dist {
                teleport = true;
            }
        }
        let scale = speed / dist;
        dx *= scale;
        dy *= scale;
    }
    // `:59596-59627` — both axes steer; the Y double-step is ×2
    // (`:59617`/`:59625`), the X one plain.
    if p.vel.x < dx {
        p.vel.x += accel;
        if accel > 0.05 && p.vel.x < 0.0 {
            p.vel.x += accel;
        }
    }
    if p.vel.x > dx {
        p.vel.x -= accel;
        if accel > 0.05 && p.vel.x > 0.0 {
            p.vel.x -= accel;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if accel > 0.05 && p.vel.y < 0.0 {
            p.vel.y += accel * 2.0;
        }
    }
    if p.vel.y > dy {
        p.vel.y -= accel;
        if accel > 0.05 && p.vel.y > 0.0 {
            p.vel.y -= accel * 2.0;
        }
    }
    // `:59628-59637` — the facing + the base rotation.
    if p.vel.x > 0.25 {
        p.direction = -1;
    } else if p.vel.x < -0.25 {
        p.direction = 1;
    }
    p.sprite_direction = p.direction;
    p.rot = p.vel.x * 0.05;
    if teleport {
        // `:59638-59665` — dust type 33 (147 for the 198/886, 31 for the
        // 815, 21 for the 817 — visual only, the rolls are type-free).
        teleport_to_owner(p, owner, rng);
    }
    // `:59666-59755` — the frame parameters (num52 tick rate, num53/num54
    // the frame range, flag17 the 817's ping-pong).
    let mut rate = 3;
    let mut first = 0;
    let mut last = 3;
    let mut ping_pong = false;
    if kind == ZEPHYR_FISH {
        rate = 6;
    }
    if kind == LIL_HARPY {
        rate = 5;
        first = 0;
        last = 5;
        if original_dist > far_speed_dist {
            first = 6;
            last = 9;
        }
    }
    if kind == GLITTERY_BUTTERFLY {
        rate = 5;
        first = 0;
        last = 2;
        ping_pong = true;
    }
    if kind == DD2_BETSY_PET {
        rate = 4;
        first = 0;
        last = 5;
        if original_dist > far_speed_dist / 2.0 {
            rate = 3;
            first = 6;
            last = 11;
        }
    }
    if kind == DUKE_FISHRON_PET {
        rate = 6;
        first = 0;
        last = proj_frames(kind) as i32 - 1;
    }
    if kind == QUEEN_BEE_PET || kind == MOON_LORD_PET {
        rate = 4;
        first = 0;
        last = proj_frames(kind) as i32 - 1;
    }
    if kind == SHARK_PUP {
        // `:59716-59730` — the idle direction flip + the dry-frame offset.
        if owner.vel.length() < 2.0 && target.length() < 10.0 {
            p.direction = -(owner.dir as i32);
            p.sprite_direction = p.direction;
        }
        rate = 6;
        // `!base.wet` — the port carries no wet flag (module doc), so the
        // +4 offset always applies.
        first += 4;
        last += 4;
        p.rot = p.vel.x * 0.05 + (p.vel.y * -0.05).abs();
    }
    if kind == PUFFERFISH {
        // `:59731-59755`.
        first = 0;
        last = 3;
        if owner.vel.length() < 2.0 && target.length() < 10.0 {
            p.direction = -(owner.dir as i32);
            p.sprite_direction = p.direction;
        }
        rate = 7;
        if original_dist >= far_speed_dist {
            first = 4;
            last = 5;
            rate = 6;
        }
        // `!base.wet` — the +6 offset always applies (module doc).
        first += 6;
        last += 6;
        p.rot = p.vel.x * 0.05 + (p.vel.y * -0.05).abs();
    }
    // `:59756-59779` — the frame machines.
    if ping_pong {
        let cycle = rate * (last - first) * 2;
        p.frame_counter += 1;
        if p.frame_counter >= cycle {
            p.frame_counter = 0;
        }
        p.frame = p.frame_counter / rate;
        if p.frame > last {
            p.frame = last + (last - p.frame);
        }
        p.frame = p.frame.clamp(first, last);
    } else if rate > 0 {
        p.frame_counter += 1;
        if p.frame_counter >= rate {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame < first || p.frame > last {
                p.frame = first;
            }
        }
    }
}

/// The 211 Wisp arm (`Projectile.cs:59782-59948`): the control-offset
/// target (±120 horizontal, +120 down / -120-60 up), the >1000-px
/// position jump (no dust, no rand), the `localAI[0]` flight latch
/// (enter at dist > 200, exit at dist < 10 with the owner slow and
/// grounded-or-flying), flight speed 12 / idle speed 5, the idle snap
/// and the 0.99 cross-zero damp.
fn wisp_arm(p: &mut Projectile, owner: &OwnerView) {
    let mut accel = 0.2f32; // num56
    let mut speed = 5.0f32; // num57
    p.tile_collide = false; // :59786
    let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5); // vector8
    // `:59788-59789` — the target rides the owner's gfxOffY.
    let mut dx = owner.center.x - center.x; // num58
    let mut dy = owner.center.y + owner.gfx_off_y - center.y; // num59
    // `:59790-59809` — the control offsets.
    if owner.control_left {
        dx -= 120.0;
    } else if owner.control_right {
        dx += 120.0;
    }
    if owner.control_down {
        dy += 120.0;
    } else {
        if owner.control_up {
            dy -= 120.0;
        }
        dy -= 60.0;
    }
    let mut dist = (dx * dx + dy * dy).sqrt(); // num60
    // `:59811-59815` — the long-range jump (no dust in the C#).
    if dist > 1000.0 {
        p.pos.x += dx;
        p.pos.y += dy;
    }
    if p.local_ai[0] == 1.0 {
        // `:59816-59871` — the flight state.
        if dist < 10.0
            && owner.vel.x.abs() + owner.vel.y.abs() < speed
            && (owner.vel.y == 0.0 || owner.mount_flying)
        {
            p.local_ai[0] = 0.0;
        }
        speed = 12.0;
        if dist < speed {
            p.vel.x = dx;
            p.vel.y = dy;
        } else {
            let scale = speed / dist;
            p.vel.x = dx * scale;
            p.vel.y = dy * scale;
        }
        if p.vel.x > 0.5 {
            p.direction = -1;
        } else if p.vel.x < -0.5 {
            p.direction = 1;
        }
        p.sprite_direction = p.direction;
        p.rot -= (0.2 + p.vel.x.abs() * 0.025) * p.direction as f32;
        p.frame_counter += 1;
        if p.frame_counter > 3 {
            p.frame += 1;
            p.frame_counter = 0;
        }
        if p.frame < 5 {
            p.frame = 5;
        }
        if p.frame > 9 {
            p.frame = 5;
        }
        // `:59862-59869` — the two flight dusts (type 156, default
        // speeds): no explicit `Main.rand` in AI_026, the internal rolls
        // ride the unported dust pool — skipped (module doc).
        return;
    }
    // `:59872-59947` — the idle state.
    if dist > 200.0 {
        p.local_ai[0] = 1.0;
    }
    if p.vel.x > 0.5 {
        p.direction = -1;
    } else if p.vel.x < -0.5 {
        p.direction = 1;
    }
    p.sprite_direction = p.direction;
    if dist < 10.0 {
        // `:59885-59897` — the snap.
        p.vel.x = dx;
        p.vel.y = dy;
        p.rot = p.vel.x * 0.05;
        if dist < speed {
            p.pos += p.vel;
            p.vel *= 0.0;
            accel = 0.0;
        }
        p.direction = -(owner.dir as i32);
    }
    // `:59898-59900` — the normalization runs UNCONDITIONALLY, even at
    // dist == 0 (the C# divides by zero → IEEE NaN/inf; the port keeps
    // the same f32 semantics, golden rule).
    let scale = speed / dist;
    dx *= scale;
    dy *= scale;
    // `:59901-59932` — the steering with the 0.99 cross-zero damp.
    if p.vel.x < dx {
        p.vel.x += accel;
        if p.vel.x < 0.0 {
            p.vel.x *= 0.99;
        }
    }
    if p.vel.x > dx {
        p.vel.x -= accel;
        if p.vel.x > 0.0 {
            p.vel.x *= 0.99;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if p.vel.y < 0.0 {
            p.vel.y *= 0.99;
        }
    }
    if p.vel.y > dy {
        p.vel.y -= accel;
        if p.vel.y > 0.0 {
            p.vel.y *= 0.99;
        }
    }
    if p.vel.x != 0.0 || p.vel.y != 0.0 {
        p.rot = p.vel.x * 0.05;
    }
    p.frame_counter += 1;
    if p.frame_counter > 3 {
        p.frame += 1;
        p.frame_counter = 0;
    }
    if p.frame > 4 {
        p.frame = 0;
    }
}

/// The 199 TikiSpirit arm (`Projectile.cs:59949-60091`): the (-2, -60)
/// target offset, speed 4 → 10 / accel 0.1 → 0.2 beyond 350, the <4-px
/// snap, the UNCONDITIONAL double-step (no accel gate), the facing pin
/// (`direction = -owner.direction`, `spriteDirection = 1`), the
/// dist ≥ 50 frame convergence (2 left / 6 right) vs the 0-7 direction
/// cycle.
fn tiki_arm(p: &mut Projectile, world: &World, owner: &OwnerView, rng: &mut dyn DropRng) {
    let mut accel = 0.1f32; // num62
    p.tile_collide = false; // :59952
    let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5); // vector9
    let mut dx = owner.center.x - center.x; // num64
    let mut dy = owner.center.y - center.y - 60.0; // num65
    dx -= 2.0; // :59957
    let mut dist = (dx * dx + dy * dy).sqrt(); // num66
    let mut speed = 4.0f32; // num67
    let original_dist = dist; // num68
    let teleport = dist > 2000.0; // flag20
    grounded_reset(p, world, owner, dist, 200.0); // num63
    if dist < 4.0 {
        // `:59974-59979` — the snap.
        p.vel.x = dx;
        p.vel.y = dy;
        accel = 0.0;
    } else {
        // `:59980-59990`.
        if dist > 350.0 {
            accel = 0.2;
            speed = 10.0;
        }
        let scale = speed / dist;
        dx *= scale;
        dy *= scale;
    }
    if teleport {
        teleport_to_owner(p, owner, rng); // dust type 2 (:59993)
    }
    // `:60007-60038` — the double-step is unconditional (no `num62 >
    // 0.05` gate — the tiki doubles even at the 0.1 accel).
    if p.vel.x < dx {
        p.vel.x += accel;
        if p.vel.x < 0.0 {
            p.vel.x += accel;
        }
    }
    if p.vel.x > dx {
        p.vel.x -= accel;
        if p.vel.x > 0.0 {
            p.vel.x -= accel;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if p.vel.y < 0.0 {
            p.vel.y += accel;
        }
    }
    if p.vel.y > dy {
        p.vel.y -= accel;
        if p.vel.y > 0.0 {
            p.vel.y -= accel;
        }
    }
    // `:60039-60041` — the facing pin + the rotation.
    p.direction = -(owner.dir as i32);
    p.sprite_direction = 1;
    p.rot = p.vel.y * 0.05 * -(p.direction as f32);
    // `:60042-60089` — the two frame machines.
    if original_dist >= 50.0 {
        p.frame_counter += 1;
        if p.frame_counter <= 6 {
            return;
        }
        p.frame_counter = 0;
        if p.vel.x < 0.0 {
            if p.frame < 2 {
                p.frame += 1;
            }
            if p.frame > 2 {
                p.frame -= 1;
            }
        } else {
            if p.frame < 6 {
                p.frame += 1;
            }
            if p.frame > 6 {
                p.frame -= 1;
            }
        }
    } else {
        p.frame_counter += 1;
        if p.frame_counter > 6 {
            p.frame += p.direction;
            p.frame_counter = 0;
        }
        if p.frame > 7 {
            p.frame = 0;
        }
        if p.frame < 0 {
            p.frame = 7;
        }
    }
}

#[cfg(test)]
mod tests {
    //! Witness tests-as-code (S1-D-4b-10, code-only mode — written, NOT
    //! run this round). Each pins a C# behaviour with its citation; the
    //! RNG-order witnesses record the draw sequence on a counting stream.

    use super::*;
    use crate::projectile::{ProjectilePool, SLIME_MINION, proj_data};
    use tr_core::tile_light::FastRandom;

    /// A recording `DropRng` — the draw log pins the C#'s `Main.rand`
    /// order (the golden rule's RNG witness).
    #[derive(Default)]
    struct RecordRng {
        draws: Vec<String>,
    }
    impl DropRng for RecordRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn owner(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 500.0;
        p.y = 500.0;
        p.vx = 0.0;
        p.vy = 0.0;
        p.facing = 1;
        p
    }

    fn pet(kind: u16, x: f32, y: f32) -> Projectile {
        let data = proj_data(kind).expect("slice-A row");
        // Field-by-field on `Default` (the struct has private bookkeeping
        // fields — a struct literal with `..Default::default()` cannot be
        // written outside projectile.rs, E0451).
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = data.w;
        p.h = data.h;
        p.friendly = true;
        p.time_left = data.time_left;
        p.penetrate = data.penetrate;
        p.max_penetrate = data.penetrate;
        p.tile_collide = data.tile_collide;
        p
    }

    /// The 14 rows carry the C# `SetDefaults` slice (the box, aiStyle 26,
    /// penetrate -1, timeLeft 18000, the stock tileCollide, the
    /// netImportant/ignoreWater overlays).
    #[test]
    fn the_fourteen_rows_carry_the_csharp_set_defaults() {
        let cases = [
            (BABY_EATER, 34.0, false, false),
            (BABY_SKELETRON_HEAD, 42.0, true, false),
            (BABY_HORNET, 26.0, true, false),
            (TIKI_SPIRIT, 28.0, true, true),
            (WISP, 24.0, true, true),
            (ZEPHYR_FISH, 26.0, true, false),
            (SHARK_PUP, 26.0, true, false),
            (LIL_HARPY, 26.0, true, false),
            (GLITTERY_BUTTERFLY, 26.0, true, false),
            (QUEEN_BEE_PET, 20.0, true, false),
            (DUKE_FISHRON_PET, 20.0, true, false),
            (MOON_LORD_PET, 20.0, true, false),
            (DD2_BETSY_PET, 20.0, true, false),
            (PUFFERFISH, 26.0, true, false),
        ];
        for (kind, size, net_important, ignore_water) in cases {
            let d = proj_data(kind).expect("row");
            assert_eq!((d.w, d.h), (size, size), "kind {kind} box");
            assert_eq!(d.ai_style, 26, "kind {kind} aiStyle");
            assert_eq!(d.penetrate, -1, "kind {kind} penetrate");
            assert_eq!(d.time_left, 18000, "kind {kind} timeLeft *= 5");
            assert!(d.tile_collide, "kind {kind} stock tileCollide");
            assert!(!d.minion && !d.sentry, "kind {kind} is a pet");
            assert_eq!(d.net_important, net_important, "kind {kind} netImportant");
            assert_eq!(d.ignore_water, ignore_water, "kind {kind} ignoreWater");
        }
        // The frame table (`Main.cs` citations in `proj_frames`); the 197
        // keeps the init loop's 1.
        let frames = [
            (BABY_EATER, 2),
            (BABY_SKELETRON_HEAD, 1),
            (BABY_HORNET, 4),
            (TIKI_SPIRIT, 8),
            (WISP, 10),
            (ZEPHYR_FISH, 4),
            (SHARK_PUP, 8),
            (LIL_HARPY, 10),
            (GLITTERY_BUTTERFLY, 18),
            (QUEEN_BEE_PET, 8),
            (DUKE_FISHRON_PET, 6),
            (MOON_LORD_PET, 8),
            (DD2_BETSY_PET, 12),
            (PUFFERFISH, 12),
        ];
        for (kind, n) in frames {
            assert_eq!(proj_frames(kind), n, "kind {kind} projFrames");
        }
    }

    /// The gate routes ONLY the 14 — the 266 slime minion (slice B)
    /// returns false and nothing mutates, RNG untouched.
    #[test]
    fn the_gate_leaves_non_slice_kinds_byte_identical() {
        let w = world();
        let mut p = owner(&w);
        let mut slime = pet(SLIME_MINION, 480.0, 480.0);
        let before = slime.clone();
        let mut rng = RecordRng::default();
        assert!(!ai026_a_tick(&mut slime, &w, Some(&mut p), &mut rng));
        assert_eq!(slime.pos, before.pos);
        assert_eq!(slime.vel, before.vel);
        assert_eq!(slime.time_left, before.time_left);
        assert!(rng.draws.is_empty(), "no rand for a non-slice kind");
    }

    /// `:58104-58108` — the unresolved owner kills the slot.
    #[test]
    fn the_ownerless_pet_dies() {
        let w = world();
        let mut p = pet(BABY_EATER, 480.0, 480.0);
        let mut rng = RecordRng::default();
        assert!(ai026_a_tick(&mut p, &w, None, &mut rng));
        assert!(!p.active);
    }

    /// The keep-alive blocks (`:58284-58294` etc.): a live flag pins
    /// timeLeft = 2; death clears the flag (and the standing flag still
    /// pins on the death tick — the C# reads before it clears).
    #[test]
    fn the_keep_alive_flag_pins_and_death_clears() {
        let w = world();
        let mut p = owner(&w);
        p.eater = true;
        let mut pet = pet(BABY_EATER, 480.0, 480.0);
        pet.time_left = 500;
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.time_left, 2, "the live flag pins");
        p.dead = true;
        pet.time_left = 500;
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert!(!p.eater, "death clears the flag");
        assert_eq!(pet.time_left, 2, "the death tick still pins once");
        pet.time_left = 500;
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.time_left, 500, "the cleared flag no longer pins");
    }

    /// `:58944-58947` — `ai[0] == -1/-2` pins timeLeft = 2.
    #[test]
    fn the_despawn_ai_pins_the_lifetime() {
        let w = world();
        let mut p = owner(&w);
        for ai0 in [-1.0f32, -2.0] {
            let mut pet = pet(WISP, 480.0, 480.0);
            pet.ai[0] = ai0;
            pet.time_left = 900;
            let mut rng = RecordRng::default();
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
            assert_eq!(pet.time_left, 2, "ai[0] = {ai0}");
        }
    }

    /// The flyer group's jitter order (`:59446-59447`): Y first, then X,
    /// both `Next(-10, 21)`; the 892/894 pair draws NOTHING.
    #[test]
    fn the_flyer_jitter_draws_y_then_x_and_the_892_894_skip() {
        let w = world();
        let mut p = owner(&w);
        // Far enough that no teleport fires (dist < 2000).
        let mut hornet = pet(BABY_HORNET, 800.0, 500.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut hornet, &w, Some(&mut p), &mut rng);
        assert_eq!(
            rng.draws,
            vec!["range(-10,21)".to_string(), "range(-10,21)".to_string()],
            "Y then X, nothing else"
        );
        for kind in [DUKE_FISHRON_PET, MOON_LORD_PET] {
            let mut pet = pet(kind, 800.0, 500.0);
            let mut rng = RecordRng::default();
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
            assert!(rng.draws.is_empty(), "kind {kind} draws no jitter");
        }
    }

    /// The teleport consumes 24 `NextFloat` draws (12 dusts × X then Y,
    /// `:59182`/`:59314`/`:59655`/`:59996`) and snaps the center onto the
    /// owner with zero velocity. The 175/197/199 teleport BEFORE their
    /// steering (`:59177`/`:59309`/`:59991` precede `:59193`/`:59325`/
    /// `:60007`), so the snapped-zero velocity takes one steering step
    /// from the PRE-teleport normalized target; the flyer group teleports
    /// AFTER its steering (`:59638`), so its velocity stays zero.
    #[test]
    fn the_teleport_consumes_24_floats_and_snaps() {
        let w = world();
        let mut p = owner(&w);
        for kind in [BABY_EATER, BABY_SKELETRON_HEAD, BABY_HORNET, TIKI_SPIRIT] {
            // 2500 px away — past every arm's 2000 gate.
            let mut pet = pet(kind, 3000.0, 500.0);
            let mut rng = RecordRng::default();
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
            // The flyer group draws its Y-then-X jitter BEFORE the
            // teleport; the 197 draws its Next(10) AFTER its steering
            // (`:59357`); the 175/199 draw only the 24 floats.
            if kind == BABY_HORNET {
                assert_eq!(rng.draws.len(), 26, "2 jitter + 24 floats");
                assert_eq!(rng.draws[0], "range(-10,21)");
                assert_eq!(rng.draws[1], "range(-10,21)");
                assert!(rng.draws[2..].iter().all(|d| d == "float"));
            } else if kind == BABY_SKELETRON_HEAD {
                assert_eq!(rng.draws.len(), 25, "24 floats + Next(10)");
                assert!(rng.draws[..24].iter().all(|d| d == "float"));
                assert_eq!(rng.draws[24], "range(0,10)");
            } else {
                assert_eq!(rng.draws.len(), 24, "kind {kind}: 12 dusts × 2 floats");
                assert!(rng.draws.iter().all(|d| d == "float"), "kind {kind}");
            }
            let (cx, cy) = p.center();
            let snapped = Vec2::new(cx - pet.w * 0.5, cy - pet.h * 0.5);
            let stepped = match kind {
                // The owner is LEFT of the 3000-px spawn: the pre-teleport
                // targets are (-7, ~0) / (-14, 0) / (-10, -0.21), accel
                // 0.2 / 0.2 / 0.2 (the tiki's dist > 350 raises it). The
                // decrease branch's double-step only fires while the
                // stepped velocity is still > 0 — off zero it goes
                // negative at once, so one step each.
                BABY_EATER => Vec2::new(-0.2, 0.0),
                BABY_SKELETRON_HEAD => Vec2::new(-0.2, 0.0),
                TIKI_SPIRIT => Vec2::new(-0.2, -0.2),
                _ => Vec2::ZERO,
            };
            assert_eq!(pet.pos, snapped + stepped, "kind {kind} snap + step");
            assert_eq!(pet.vel, stepped, "kind {kind} post-snap velocity");
        }
    }

    /// The 175's dominant-axis gate (`:59193-59230`): at (300, 500) the
    /// owner delta is (200, 0) — normalized to (7, 0), accel 0.1, only X
    /// steers; `rotation = atan2(vy, vx) - 1.57` (`:59231`).
    #[test]
    fn the_eater_steers_the_dominant_axis() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(BABY_EATER, 300.0, 500.0);
        pet.vel = Vec2::new(0.0, 5.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert!((pet.vel.x - 0.1).abs() < 1e-6, "X accelerates");
        assert_eq!(pet.vel.y, 5.0, "Y untouched (|dx| > |dy|)");
        assert!((pet.rot - (5.0f32.atan2(0.1) - 1.57)).abs() < 1e-6);
        assert!(rng.draws.is_empty(), "the 175 draws no jitter");
    }

    /// The 175's close band (`:59143-59166`): inside 150 the target
    /// clamps to ±2 and the accel drops to 0.01; a fast velocity damps
    /// 0.99 first.
    #[test]
    fn the_eater_close_band_clamps_and_damps() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // The eater's center 100 px left of the owner's → dist ≈ 100.
        let mut pet = pet(BABY_EATER, cx - 17.0 - 100.0, cy - 17.0);
        pet.vel = Vec2::new(3.0, 0.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        // dist 100 < 150: damp 3 → 2.97, the target clamps 100 → 2,
        // vel 2.97 > 2 → -0.01 (the double-step is off at accel 0.01).
        assert!((pet.vel.x - 2.96).abs() < 1e-4, "damp then steer: {}", pet.vel.x);
    }

    /// The 197's spin latch (`:59258-59261`, `:59357-59376`): dist > 500
    /// sets localAI[0] = 10000, the per-tick `Next(10)` pushes it past
    /// 10000, localAI[1] latches the velocity sign and the rotation spins
    /// 0.25/tick. RecordRng's `Next(10)` returns the min (0), so the
    /// accumulator stays exactly 10000 + 0 — the > 10000 spin branch does
    /// NOT fire on the latch tick itself (the C#'s strict >).
    #[test]
    fn the_skeletron_head_latches_the_spin() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(BABY_SKELETRON_HEAD, 1100.0, 500.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.local_ai[0], 10000.0, "the dist > 500 latch + Next(10)=0");
        assert_eq!(pet.local_ai[1], 0.0, "10000 is not > 10000 — no spin yet");
        assert_eq!(rng.draws, vec!["range(0,10)".to_string()], "one Next(10)");
        // Seed past the latch: 10001 > 10000 → the sign latch reads the
        // steered velocity (negative — the owner is to the left) and the
        // rotation spins -0.25. The pet is still > 500 away, so the head
        // re-latches 10000 first; seed 10001 via a close placement instead.
        let (cx, cy) = p.center();
        pet.pos = Vec2::new(cx - 21.0 - 100.0, cy - 21.0);
        pet.local_ai[0] = 10001.0;
        pet.local_ai[1] = 0.0;
        pet.rot = 0.0;
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.local_ai[1], -1.0, "the velocity sign latch");
        assert!((pet.rot - -0.25).abs() < 1e-6, "the spin step");
        // The accumulator resets past 12000 (`:59372-59375`): 12001 + 0 >
        // 12000 → reset to 0 (the close placement keeps the re-latch off).
        pet.local_ai[0] = 12001.0;
        pet.local_ai[1] = 1.0;
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.local_ai[0], 0.0, "the >12000 reset");
    }

    /// The 197's rotation ease (`:59377-59397`): below the spin latch the
    /// rotation eases toward `vx * 0.1` at `(|vx| + |vy|) * 0.01`/tick.
    #[test]
    fn the_skeletron_head_eases_toward_the_velocity_tilt() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // The head's center 100 px left of the owner's → dist ≈ 100 <
        // 150 (the close band: damp off at |v| ≤ 2, accel 0.01).
        let mut pet = pet(BABY_SKELETRON_HEAD, cx - 21.0 - 100.0, cy - 21.0);
        pet.vel = Vec2::new(1.0, 2.0);
        pet.rot = 0.5;
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        // vx steers 1 → 1.01 (target clamps to 2); vy 2 > dy 0 → 1.99
        // (accel 0.01, no double-step). Ease target 0.101, step
        // (1.01 + 1.99) * 0.01 = 0.03 → rot 0.47; the second `rot <
        // target` check is false (0.47 > 0.101).
        assert!((pet.rot - 0.47).abs() < 1e-3, "the ease step: {}", pet.rot);
        assert_eq!(pet.local_ai[1], 0.0, "the spin sign clears");
    }

    /// The 892/894/901 speed quirk (`:59468-59475`): the `6 if < 200`
    /// write is dead — the unconditional overwrite gives 9 below 400 and
    /// 12 at/above. The steering witness pins the far-accel ladder (0.4
    /// in the 100-200 band, 0.7 beyond 400); the speed itself is pinned
    /// by the frame-band tests (the 901's > 200 band reads the same
    /// pre-normalization dist).
    #[test]
    fn the_boss_pets_speed_quirk_overwrites_the_six() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // Center 80 px right of the owner: dx = -80 - 60 = -140 (the 892
        // draws no jitter), dy = -60 → dist ≈ 152.3. The 100-200 band
        // keeps accel 0.4; both axes steer one step off zero.
        let mut duke = pet(DUKE_FISHRON_PET, cx - 10.0 + 80.0, cy - 10.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut duke, &w, Some(&mut p), &mut rng);
        assert!((duke.vel.x - -0.4).abs() < 1e-6, "accel 0.4: {}", duke.vel.x);
        assert!((duke.vel.y - -0.4).abs() < 1e-6);
        assert!(rng.draws.is_empty(), "the 892 draws no jitter");
        // Beyond 400: dx = -560, dy = -60 → dist ≈ 563 → accel 0.7.
        let mut far = pet(DUKE_FISHRON_PET, cx - 10.0 + 500.0, cy - 10.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut far, &w, Some(&mut p), &mut rng);
        assert!((far.vel.x - -0.7).abs() < 1e-6, "the > 400 accel: {}", far.vel.x);
    }

    /// The grounded reset (`:59488-59495`): dist < 100, owner vy == 0,
    /// the flyer's bottom ≤ the owner's bottom and a clear box → ai[0]
    /// resets and vy floors at -6.
    #[test]
    fn the_grounded_reset_floors_the_rise() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // Center 25 px up-left of the owner: the pre-offset delta (25, 25)
        // + RecordRng's min jitter (-10, -10) + the (-60, -60) offset →
        // (-45, -45), dist ≈ 63.6 — inside the 100 reset, outside the 50
        // close band. Bottom: cy - 25 - 13 + 26 = cy - 12 ≤ owner bottom.
        let mut hornet = pet(BABY_HORNET, cx - 13.0 - 25.0, cy - 13.0 - 25.0);
        hornet.ai[0] = 3.0;
        hornet.vel = Vec2::new(0.0, -10.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut hornet, &w, Some(&mut p), &mut rng);
        assert_eq!(hornet.ai[0], 0.0, "the reset fires");
        // vy floors at -6; the far branch (dist ≥ 50) keeps accel 0.1
        // (dist < 100) and normalizes to 14 → target (-9.9, -9.9). Both
        // axes sit ABOVE their targets → the decrease branches: Y
        // -6 - 0.1 = -6.1 (not > 0, no double-step); X 0 - 0.1 = -0.1.
        assert!((hornet.vel.y - -6.1).abs() < 1e-4, "floor then steer: {}", hornet.vel.y);
        assert!((hornet.vel.x - -0.1).abs() < 1e-4, "the X step: {}", hornet.vel.x);
        // A falling owner blocks the reset.
        let mut p2 = owner(&w);
        p2.vy = 1.0;
        let mut pet2 = pet(BABY_HORNET, cx - 13.0 - 25.0, cy - 13.0 - 25.0);
        pet2.ai[0] = 3.0;
        ai026_a_tick(&mut pet2, &w, Some(&mut p2), &mut rng);
        assert_eq!(pet2.ai[0], 3.0, "the owner must stand");
    }

    /// The 892's close damp is 0.95 (the C# typo `892 || 892`,
    /// `:59500-59507`); every other group type damps 0.99.
    #[test]
    fn the_duke_fishron_pet_damps_095() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // The 892 draws no jitter: center at (cx - 60, cy - 60) → the
        // pre-offset delta (60, 60) + the (-60, -60) offset → target
        // (0, 0), dist 0 < 50 — the close band.
        let mut pet = pet(DUKE_FISHRON_PET, cx - 10.0 - 60.0, cy - 10.0 - 60.0);
        pet.vel = Vec2::new(4.0, 0.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        // Damp 4 → 3.8 first; the target (0, 0) steers X down 0.01 → 3.79
        // (accel 0.01, no double-step off a positive velocity... 3.79 > 0
        // and accel 0.01 ≤ 0.05 → no double). The witness is the 0.95.
        assert!((pet.vel.x - 3.79).abs() < 0.02, "0.95 damp: {}", pet.vel.x);
    }

    /// The wisp's flight latch (`:59872-59875`, `:59826-59837`): idle at
    /// dist > 200 sets localAI[0] = 1; the next tick flies at speed 12
    /// with frames clamped into 5-9.
    #[test]
    fn the_wisp_latches_flight_at_200_and_flies_at_12() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(WISP, 800.0, 500.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.local_ai[0], 1.0, "the latch");
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert!(pet.vel.length() <= 12.0 + 1e-3, "speed 12 cap");
        assert!(pet.vel.length() > 11.0, "still far — full speed");
        assert!((5..=9).contains(&pet.frame), "flight frames: {}", pet.frame);
        assert!(rng.draws.is_empty(), "the wisp draws no explicit rand");
    }

    /// The wisp's flight exit (`:59818-59825`): dist < 10 with the owner
    /// slow (|vx| + |vy| < 5) and grounded clears the latch.
    #[test]
    fn the_wisp_exits_flight_when_the_owner_stands() {
        let w = world();
        let mut p = owner(&w);
        // Target = owner center + (0, -60) - wisp center; place the wisp
        // so the target delta is (0, -5) → dist 5 < 10.
        let (cx, cy) = p.center();
        let mut pet = pet(WISP, cx - 12.0, cy - 60.0 + 5.0 - 12.0);
        pet.local_ai[0] = 1.0;
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.local_ai[0], 0.0, "the exit");
    }

    /// The wisp's idle snap (`:59885-59897`): dist < 10 sets the velocity
    /// to the delta, and dist < 5 integrates it immediately and zeroes
    /// the velocity (the accel drops to 0).
    #[test]
    fn the_wisp_idle_snap_integrates_and_zeroes() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // Delta (0, -3): dist 3 < 5.
        let mut pet = pet(WISP, cx - 12.0, cy - 60.0 + 3.0 - 12.0);
        let start = pet.pos;
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        // The snap integrates (0, -3) inside the arm, zeroes vel, then
        // UpdatePosition adds the zero velocity.
        assert_eq!(pet.pos, start + Vec2::new(0.0, -3.0));
        assert_eq!(pet.vel, Vec2::ZERO);
        assert_eq!(pet.direction, -1, "the idle facing flip");
    }

    /// The wisp's control offsets (`:59790-59809`): controlRight shifts
    /// the target +120 X, controlDown +120 Y (else up -120 and the -60).
    #[test]
    fn the_wisp_target_rides_the_controls() {
        let w = world();
        let mut p = owner(&w);
        p.control_right = true;
        p.control_down = true;
        let (cx, cy) = p.center();
        // Wisp center at the owner center → delta (120, 120), dist ≈ 170
        // (< 200, no latch; > 10, no snap).
        let mut pet = pet(WISP, cx - 12.0, cy - 12.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        // Steering toward (120, 120) * 5/170: both axes accelerate +0.2.
        assert!(pet.vel.x > 0.0 && pet.vel.y > 0.0, "down-right target");
    }

    /// The tiki's frame convergence (`:60042-60072`): at dist ≥ 50 the
    /// counter gates 7 ticks, then the frame steps toward 2 (moving
    /// left) or 6 (moving right).
    #[test]
    fn the_tiki_converges_its_frames_when_far() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(TIKI_SPIRIT, 300.0, 500.0);
        pet.frame = 0;
        let mut rng = RecordRng::default();
        for _ in 0..7 {
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        }
        assert_eq!(pet.frame, 1, "the 7th tick steps once");
        // Moving right (the owner is to the right) → converge to 6: five
        // more 7-tick gates (the pet keeps closing, but stays ≥ 50 px for
        // all 42 ticks — the snap at < 4 never fires inside the window).
        for _ in 0..35 {
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        }
        assert_eq!(pet.frame, 6, "the right convergence");
        assert_eq!(pet.sprite_direction, 1, "the tiki's sprite pin");
        assert_eq!(pet.direction, -1, "direction = -owner.direction");
    }

    /// The tiki's idle cycle (`:60073-60089`): below 50 the frame steps
    /// by `direction` every 7 ticks and wraps 0-7.
    #[test]
    fn the_tiki_idle_cycle_wraps_by_direction() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        // Delta ≈ (0, 0): the tiki center at (cx - 2, cy - 60).
        let mut pet = pet(TIKI_SPIRIT, cx - 2.0 - 14.0, cy - 60.0 - 14.0);
        pet.frame = 0;
        let mut rng = RecordRng::default();
        for _ in 0..7 {
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        }
        // direction = -1 → frame 0 - 1 = -1 → wraps to 7.
        assert_eq!(pet.frame, 7, "the negative wrap");
    }

    /// The 774's sine hover (`:59429-59441`): at miscCounter = 15 the
    /// sine is 1, the lerp 1. The zero-gate reads the PRE-offset delta
    /// MINUS the (70·dir, 60) offset — a pet at the owner center sees
    /// |(-70, -60)| ≈ 92 > 50 → sine zeroed; a pet 70 left / 60 up sees
    /// |(0, 0)| ≤ 50 → the sine survives and its -8/+8 Y terms cancel.
    #[test]
    fn the_shark_pup_hovers_on_the_sine() {
        let w = world();
        let mut p = owner(&w);
        p.misc_counter = 15; // sin(2π * 15/60) = sin(π/2) = 1
        let (cx, cy) = p.center();
        // Pet center AT the owner center: gate |(-70, -60)| > 50 → sine
        // zeroed → target (-70, -60): the pup steers up and left.
        let mut far = pet(SHARK_PUP, cx - 13.0, cy - 13.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut far, &w, Some(&mut p), &mut rng);
        assert!(far.vel.y < 0.0, "the zeroed-sine target");
        assert!(far.vel.x < 0.0);
        // Pet center 70 left / 60 up: gate |(0, 0)| ≤ 50 → sine = lerp =
        // 1 survives; the Y offset -60 + 1*-8 + 1*8 = -60 cancels the 60
        // delta → target (-10, 0): X steers left, Y untouched.
        let mut near = pet(SHARK_PUP, cx - 60.0 - 13.0, cy - 60.0 - 13.0);
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut near, &w, Some(&mut p), &mut rng);
        assert!(near.vel.x < 0.0, "the -70 X offset");
        assert_eq!(near.vel.y, 0.0, "the -8/+8 cancellation");
    }

    /// The 774/1046 idle direction flip (`:59718-59722`, `:59735-59739`):
    /// owner speed < 2 and target length < 10 → direction =
    /// -owner.direction. The sine hover's target is (dx - 70·dir, dy - 60
    /// + lerp·-8 + sine·8); at miscCounter = 15 (sine = lerp = 1) the Y
    /// offset cancels to -60, so the pet center at (cx - 70, cy - 60)
    /// gives the post-offset target (0, 0) — length 0 < 10.
    #[test]
    fn the_fish_pets_flip_when_the_owner_idles() {
        let w = world();
        let mut p = owner(&w);
        p.misc_counter = 15; // sine = 1, lerp = 1 → the Y offset cancels
        let (cx, cy) = p.center();
        for kind in [SHARK_PUP, PUFFERFISH] {
            // Pet center 70 left / 60 up: the pre-offset delta (70, 60)
            // minus the (70, 60) gate offset → length 0 ≤ 50 (sine
            // survives, but its -8/+8 Y terms cancel either way); the
            // post-offset target is (0, 0) → length 0 < 10, the owner
            // stands still → the flip fires.
            let mut pet = pet(kind, cx - 70.0 - 13.0, cy - 60.0 - 13.0);
            let mut rng = RecordRng::default();
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
            assert_eq!(pet.direction, -1, "kind {kind} flips to -owner.dir");
            assert_eq!(pet.sprite_direction, -1);
        }
    }

    /// The 817's ping-pong frame machine (`:59756-59770`): rate 5, range
    /// 0-2, cycle 20 — the counter sawtooths and the frame reflects.
    #[test]
    fn the_butterfly_ping_pongs_its_frames() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(GLITTERY_BUTTERFLY, 800.0, 500.0);
        let mut rng = RecordRng::default();
        let mut frames = Vec::new();
        for _ in 0..20 {
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
            frames.push(pet.frame);
        }
        // The counter sawtooths 1..19 then wraps to 0 at tick 20; frame =
        // counter / 5, reflected past 2 (frame = 2 + (2 - frame)):
        // ticks 1-4 → 0, 5-9 → 1, 10-14 → 2, 15-19 → 3 → 1, tick 20 → 0.
        assert_eq!(frames[3], 0);
        assert_eq!(frames[4], 1);
        assert_eq!(frames[8], 1);
        assert_eq!(frames[9], 2);
        assert_eq!(frames[13], 2);
        assert_eq!(frames[14], 1, "the reflection");
        assert_eq!(frames[18], 1);
        assert_eq!(frames[19], 0, "the cycle wrap");
    }

    /// The 815's far frames (`:59674-59684`): original dist > 400 →
    /// frames 6-9; the 901's (`:59692-59703`): > 200 → rate 3, frames
    /// 6-11. The band switch resets the frame to the band's FIRST on the
    /// entry tick (the `frame < num53` reset, `:59775-59778`).
    #[test]
    fn the_harpy_and_betsy_switch_frame_bands_when_far() {
        let w = world();
        let mut p = owner(&w);
        let mut harpy = pet(LIL_HARPY, 1000.0, 500.0);
        let mut rng = RecordRng::default();
        for _ in 0..5 {
            ai026_a_tick(&mut harpy, &w, Some(&mut p), &mut rng);
        }
        // The 5th tick is the band entry: frame 0 + 1 = 1 < 6 → reset 6.
        assert_eq!(harpy.frame, 6, "harpy far band entry");
        for _ in 0..5 {
            ai026_a_tick(&mut harpy, &w, Some(&mut p), &mut rng);
        }
        assert_eq!(harpy.frame, 7, "harpy far band step");
        let mut betsy = pet(DD2_BETSY_PET, 800.0, 500.0);
        for _ in 0..3 {
            ai026_a_tick(&mut betsy, &w, Some(&mut p), &mut rng);
        }
        assert_eq!(betsy.frame, 6, "betsy far band entry");
    }

    /// The 1046's far band (`:59741-59753`): dist ≥ 400 → frames 4-5
    /// (+6 dry offset → 10-11), rate 6.
    #[test]
    fn the_pufferfish_far_band_rides_the_dry_offset() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(PUFFERFISH, 1000.0, 500.0);
        let mut rng = RecordRng::default();
        for _ in 0..6 {
            ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        }
        assert!((10..=11).contains(&pet.frame), "far + dry: {}", pet.frame);
    }

    /// `UpdatePosition` (`:20445`): the dispatch integrates the velocity
    /// after the arm — the 175 at rest far from the owner moves by its
    /// steered velocity.
    #[test]
    fn the_dispatch_integrates_the_position() {
        let w = world();
        let mut p = owner(&w);
        let mut pet = pet(BABY_EATER, 300.0, 500.0);
        let start = pet.pos;
        let mut rng = RecordRng::default();
        ai026_a_tick(&mut pet, &w, Some(&mut p), &mut rng);
        assert_eq!(pet.pos, start + pet.vel, "position += velocity");
        assert!(!pet.tile_collide, "the arm cleared tileCollide");
    }

    /// The pool seam: the `26 =>` gate routes the 14 through
    /// `ai026_a_tick`; the 266 slime minion falls through A's gate to
    /// slice C's ground machine (`ai026_c_tick`, S1-D-4b-16 — it walks
    /// now; A's own arm still never touches it). The 64×64 test world's
    /// bounds are 1024 px — the owner stands at (400, 400) so both spawns
    /// stay inside (the out-of-world guard kills non-minions there).
    #[test]
    fn the_pool_dispatch_gates_slice_a_only() {
        let mut w = world();
        let mut p = owner(&w);
        p.x = 400.0;
        p.y = 400.0;
        let (cx, cy) = p.center();
        let mut pool = ProjectilePool::new();
        let eater = pool
            .spawn_with_data_ai_owned(BABY_EATER, 0, cx + 200.0, cy, 0.0, 0.0, 0, 0.0, 0.0, 0.0)
            .expect("eater slot");
        let slime = pool
            .spawn_with_data_ai_owned(SLIME_MINION, 0, cx + 50.0, cy, 0.0, 0.0, 0, 0.0, 0.0, 0.0)
            .expect("slime slot");
        pool.update(
            &mut w,
            &mut [],
            Some((cx, cy)),
            None,
            Some(&mut p),
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut FastRandom::new(7),
            0,
            true,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(pool.slots()[eater].vel.x < 0.0, "the eater steers home");
        // Slice C arms the 266: the band walks it left toward the
        // formation slot behind the owner (the pre-C fall-through froze
        // it — the slice C module's own pool witness pins the move).
        assert!(
            pool.slots()[slime].vel.x < 0.0,
            "slice C's ground machine walks the slime"
        );
        // A's arm never ran on it: no flyer keep-alive pin, no flight
        // frame (the ground machine's frames stay in 0-7).
        assert_eq!(pool.slots()[slime].time_left, 18000 - 1);
        assert!(pool.slots()[slime].frame < 8, "no flyer frame");
    }
}
