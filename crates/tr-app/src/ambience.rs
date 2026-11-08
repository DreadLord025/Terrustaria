//! The ambient sky creatures: `GameContent/Skies/AmbientSky.cs` (the entity
//! classes and the `AmbientSky : CustomSky` manager) and
//! `GameContent/Ambience/AmbienceServer.cs` (the spawn scheduler).
//!
//! The C# split exists for multiplayer: `AmbienceServer.SpawnForPlayer`
//! broadcasts a `NetAmbienceModule` packet whose seed is one `Main.rand`
//! draw (`NetAmbienceModule.cs:13`), and the loopback constructs every entity
//! attribute from `FastRandom(seed)` (`AmbientSky.cs:1391-1393`) — in single
//! player the port inlines the loopback: one draw from this module's
//! `Main.rand` stand-in becomes the seed, then `FastRandom(seed)` (the exact
//! C# stream) drives the constructor, exactly like the original. Per-tick
//! updates consume no RNG.
//!
//! Draw-side state is consumed by `render::sky` (the depth sandwich); sounds
//! are `M1.4-BG-20`'s scope. `SkyEntityType.BoneSerpent` is carried in the
//! enum but `AmbientSky.Spawn` has no case for it in 1.4.5.8
//! (`AmbientSky.cs:1394-1480`) — the dead path is preserved, not "fixed".

use glam::Vec2;
use tr_core::tile_light::FastRandom;

use crate::app::next_rand;

/// `SkyEntityType` (`SkyEntityType.cs:3-24`), C# declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkyEntityType {
    BirdsV,
    Wyvern,
    Airship,
    AirBalloon,
    Eyeball,
    Meteor,
    BoneSerpent,
    Bats,
    Butterflies,
    LostKite,
    Vulture,
    PixiePosse,
    Seagulls,
    SlimeBalloons,
    Gastropods,
    Pegasus,
    EaterOfSouls,
    Crimera,
    Hellbats,
}

use SkyEntityType::*;

/// Per-tick inputs the scheduler and entities read off `Main`/`Player`
/// (`AmbienceServer.Update`, the `FadingSkyEntity` velocity overrides).
/// `player_center` is `Main.LocalPlayer.Center`; `player_y` its pixel Y for
/// the height gates.
pub(crate) struct AmbienceCtx<'a> {
    pub world: &'a tr_core::World,
    pub day_time: bool,
    pub eclipse: bool,
    pub blood_moon: bool,
    pub pumpkin_moon: bool,
    pub snow_moon: bool,
    pub hard_mode: bool,
    pub is_raining: bool,
    pub is_storming: bool,
    pub happy_windy_day: bool,
    pub too_windy_for_butterflies: bool,
    pub butterfly_chance: i32,
    /// `Main.WindForVisuals` = `windSpeedCurrent` (`Main.cs:2823`).
    pub wind: f32,
    /// The scene-metrics zone flags the secondary spawn conditions read
    /// (`AmbienceServer.cs:77-84`).
    pub zone_desert: bool,
    pub zone_hallow: bool,
    pub zone_beach: bool,
    pub zone_corrupt: bool,
    pub zone_crimson: bool,
    pub zone_jungle: bool,
    pub player_center: Vec2,
    pub player_y: f32,
    /// The entity-update repetition per 60 Hz step (`SkyManager.Update`'s
    /// `max(1, dayRate)` loop, `SkyManager.cs:24-46`); 1 until the sundial
    /// (BG-21) lands.
    pub day_rate: i32,
}

/// One ambient sky entity: the `SkyEntity` base fields (`AmbientSky.cs:19-33`)
/// plus the `FadingSkyEntity` envelope (`:84-102`) and the per-kind state the
/// C# subclasses carry.
pub(crate) struct SkyEntity {
    pub kind: SkyKind,
    /// The texture row count of the 1×N `SpriteFrame`.
    pub frames: u32,
    /// Which sheet of the kind's variants this entity uses: the pixie's
    /// day/night sheet (1/2, `:398-401`), the bat sheets (1..3, `:924`),
    /// hellbat sheets (1..3, `:814`), the EOC depth tiers (1..3,
    /// `:1158-1167`/`:1182-1190`) and the balloon sizes (0 = Large, 1 =
    /// Small, `:1113`). 0 where the kind has one sheet.
    pub variant: u8,
    /// `FadingSkyEntity.Position`.
    pub position: Vec2,
    pub velocity: Vec2,
    /// `SkyEntity.Depth`: the sandwich stratum (boundary table, P3).
    pub depth: f32,
    /// `SpriteEffects.FlipHorizontally`.
    pub flip: bool,
    pub rotation: f32,
    /// `SpriteFrame.CurrentRow`.
    pub frame: u32,
    /// `FadingSkyEntity.FramingSpeed` (`i32::MAX` = no animation).
    pub framing_speed: i32,
    pub frame_offset: i32,
    pub life_time: i32,
    /// `FadingSkyEntity.TimeEntitySpawnedIn` (-1 until the first update).
    pub spawned_at: i32,
    pub opacity: f32,
    pub brightness_lerper: f32,
    pub final_opacity_multiplier: f32,
    /// `OpacityNormalizedTimeToFadeIn` / `...FadeOut`.
    pub fade_in: f32,
    pub fade_out: f32,
    pub is_active: bool,
    /// Extra per-kind state (`EOCSkyEntity`'s machine, magnet groups).
    pub state: EntityState,
    /// The spawn seed, relayed to clients by the `82` ambience module
    /// (`NetAmbienceModule`).
    pub net_seed: i32,
    /// Whether this spawn has been broadcast to the clients already.
    pub net_sent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkyKind {
    Butterflies,
    LostKite,
    Pegasus,
    Vulture,
    PixiePosse,
    BirdsPack,
    Seagulls,
    Gastropod,
    SlimeBalloons,
    HellBat,
    Bats,
    Wyvern,
    Airship,
    AirBalloon,
    /// `EOCSkyEntity` and its texture variants (`Crimera`, `EOS`).
    Eoc(EocVariant),
    Meteor,
    /// `NormalizedBackgroundLayerSpaceSkyEntity` — `BoneSerpentSkyEntity` is
    /// never constructed (no `Spawn` case); kept for the enum mapping only.
    BoneSerpent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EocVariant {
    /// `EOCSkyEntity` — `Images/Backgrounds/Ambience/EOC`, 3 frames.
    Eyeball,
    /// `EOSSkyEntity` — `EOS{1|2|3}` by depth, colour ×`bgAlphaFrontLayer[1]`.
    Eos,
    /// `CrimeraSkyEntity` — `Crimera{1|2|3}` by depth, colour ×`bgAlphaFrontLayer[8]`.
    Crimera,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum EntityState {
    #[default]
    Plain,
    /// `EOCSkyEntity`'s `_state`/`_direction`/`_waviness`
    /// (`STATE_ZIGZAG = 1`, `STATE_GOOVERPLAYER = 2`).
    Eoc {
        state: i32,
        direction: i32,
        waviness: f32,
    },
    /// The four magnet-group fields (`_magnetAccelerations`,
    /// `_magnetPointTarget`, `_positionVsMagnet`, `_velocityVsMagnet`).
    Magnet {
        accel: Vec2,
        target: Vec2,
        pos_vs_magnet: Vec2,
        vel_vs_magnet: Vec2,
    },
}

/// `Main.maxScreenW/H` on the HiDef profile (`Main.cs:3709-3710`) — the
/// `VirtualCamera` size (`Graphics/VirtualCamera.cs:7-16`); the Reach fallback
/// (1920×1200, `:3715-3716`) is not used by modern builds.
const VIRTUAL_CAMERA_SIZE: Vec2 = Vec2::new(4096.0, 4096.0);

/// `VirtualCamera.Position` = `Center - Size * 0.5`.
fn virtual_camera_position(player_center: Vec2) -> Vec2 {
    player_center - VIRTUAL_CAMERA_SIZE * 0.5
}

/// `Utils.GetLerpValue(from, to, t, clamped)` (`Utils.cs:1855-1860`).
fn lerp_clamped(from: f32, to: f32, t: f32) -> f32 {
    if to == from {
        return if t >= to { 1.0 } else { 0.0 };
    }
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

impl SkyEntity {
    /// `FadingSkyEntity.IsMovementDone` (`AmbientSky.cs:145-157`).
    fn is_movement_done(&mut self, frame_count: i32) -> bool {
        if self.spawned_at == -1 {
            self.spawned_at = frame_count;
        }
        if frame_count - self.spawned_at >= self.life_time {
            self.is_active = false;
            return true;
        }
        false
    }

    /// `FadingSkyEntity.UpdateOpacity` (`AmbientSky.cs:132-143`).
    fn update_opacity(&mut self, frame_count: i32) {
        let age = (frame_count - self.spawned_at) as f32;
        if age >= self.life_time as f32 * self.fade_out {
            self.opacity = lerp_clamped(
                self.life_time as f32,
                self.life_time as f32 * self.fade_out,
                age,
            );
        } else {
            self.opacity = lerp_clamped(0.0, self.life_time as f32 * self.fade_in, age);
        }
    }

    /// `FadingSkyEntity.StartFadingOut` (`AmbientSky.cs:164-171`).
    fn start_fading_out(&mut self, current_frame_count: i32) {
        let shifted = current_frame_count - (self.life_time as f32 * self.fade_out) as i32;
        if shifted < self.spawned_at {
            self.spawned_at = shifted;
        }
    }

    /// `SkyEntity.NextFrame` (`AmbientSky.cs:36-39`).
    fn next_frame(&mut self) {
        self.frame = (self.frame + 1) % self.frames;
    }

    /// `FadingSkyEntity.Update` (`AmbientSky.cs:114-126`): the envelope, the
    /// frame advance and the per-kind velocity, then the position integrate.
    fn update(&mut self, frame_count: i32, ctx: &AmbienceCtx) {
        if self.is_movement_done(frame_count) {
            return;
        }
        self.update_opacity(frame_count);
        if (frame_count + self.frame_offset) % self.framing_speed == 0 {
            self.next_frame();
        }
        self.update_velocity(frame_count, ctx);
        self.position += self.velocity;
    }

    /// The `UpdateVelocity` overrides, one arm per C# subclass.
    fn update_velocity(&mut self, frame_count: i32, ctx: &AmbienceCtx) {
        let wind = ctx.wind;
        let dir: f32 = if self.flip { -1.0 } else { 1.0 };
        match self.kind {
            // `AmbientSky.cs:208-211`
            SkyKind::Butterflies => {
                self.velocity = Vec2::new((0.1 + wind.abs() * 0.05) * dir, 0.0);
            }
            // `:256-264` — the storm ×1.5; the storm frame rate is applied in
            // [`Ambience::step_once`].
            SkyKind::LostKite => {
                let mut speed = 1.2 + wind.abs() * 3.0;
                if ctx.is_storming {
                    speed *= 1.5;
                }
                self.velocity = Vec2::new(speed * dir, 0.0);
            }
            // `:309-312`
            SkyKind::Pegasus => {
                self.velocity = Vec2::new((1.5 + wind.abs() * 0.6) * dir, 0.0);
            }
            // `:357-360`
            SkyKind::Vulture => {
                self.velocity = Vec2::new((3.0 + wind.abs() * 0.8) * dir, 0.0);
            }
            // `:412-415`
            SkyKind::PixiePosse => {
                self.velocity = Vec2::new((0.12 + wind.abs() * 0.08) * dir, 0.0);
            }
            // `:460-463`
            SkyKind::BirdsPack => {
                self.velocity = Vec2::new((3.0 + wind.abs() * 0.8) * dir, 0.0);
            }
            // `:517-522`
            SkyKind::Seagulls => {
                self.magnet_step();
                self.velocity = Vec2::new(4.0 * dir, 0.0) + self.magnet_velocity();
            }
            // `:622-628`
            SkyKind::Gastropod => {
                self.magnet_step();
                self.velocity =
                    Vec2::new((1.5 + wind.abs() * 0.2) * dir, 0.0) + self.magnet_velocity();
                self.rotation = self.velocity.x * 0.1;
            }
            // `:728-734`
            SkyKind::SlimeBalloons => {
                self.magnet_step();
                self.velocity = Vec2::new((1.0 + wind.abs()) * dir, -0.01) + self.magnet_velocity();
                self.rotation = self.velocity.x * 0.1;
            }
            // `:830-835` / `:940-945`
            SkyKind::HellBat | SkyKind::Bats => {
                self.magnet_step();
                self.velocity =
                    Vec2::new((3.0 + wind.abs() * 0.8) * dir, 0.0) + self.magnet_velocity();
            }
            // `:1031-1034`
            SkyKind::Wyvern => {
                self.velocity = Vec2::new((3.0 + wind.abs() * 0.8) * dir, 0.0);
            }
            // `:1086-1089`
            SkyKind::Airship => {
                self.velocity = Vec2::new((6.0 + wind.abs() * 1.6) * dir, 0.0);
            }
            // `:1124-1141`: the ascent halves three times above the
            // `worldSurface * {12, 8, 4}` pixel bands; the drift is the signed
            // wind (not its magnitude).
            SkyKind::AirBalloon => {
                let mut climb = 3.0 + wind.abs();
                let surface = ctx.world.world_surface as f32;
                for band in [12.0, 8.0, 4.0] {
                    if (self.position.y as f64) < (surface * band) as f64 {
                        climb *= 0.5;
                    }
                }
                self.velocity = Vec2::new(wind * 4.0, -climb);
            }
            // `:1260-1288`: the two `EOCSkyEntity` states; rotation follows
            // the velocity (`:1271`).
            SkyKind::Eoc(_) => {
                let EntityState::Eoc {
                    state,
                    direction,
                    waviness,
                } = self.state
                else {
                    return;
                };
                if state == 1 {
                    // `ZigzagMove` (`:1274-1277`).
                    self.velocity = Vec2::new(
                        direction as f32 * 3.0,
                        (frame_count as f32 / 1200.0 * std::f32::consts::TAU).cos() * waviness,
                    );
                } else {
                    // `ChasePlayerTop` (`:1279-1288`).
                    let to_player = ctx.player_center + Vec2::new(0.0, -500.0) - self.position;
                    if to_player.length() >= 100.0 {
                        self.velocity.x += 0.1 * to_player.x.signum();
                        self.velocity.y += 0.1 * to_player.y.signum();
                        self.velocity = self.velocity.clamp(Vec2::splat(-18.0), Vec2::splat(18.0));
                    }
                }
                self.rotation = self.velocity.to_angle();
            }
            // `MeteorSkyEntity` never changes velocity after the constructor.
            SkyKind::Meteor | SkyKind::BoneSerpent => {}
        }
    }

    /// The magnet integration shared by the group kinds
    /// (`AmbientSky.cs:519-520,624-625,730-731,832-833,942-943`):
    /// `_velocityVsMagnet += accel * Sign(target - posVsMagnet)` per axis,
    /// then `_positionVsMagnet += _velocityVsMagnet`.
    fn magnet_step(&mut self) {
        if let EntityState::Magnet {
            accel,
            target,
            pos_vs_magnet,
            vel_vs_magnet,
        } = &mut self.state
        {
            let sign = Vec2::new(
                (target.x - pos_vs_magnet.x).signum(),
                (target.y - pos_vs_magnet.y).signum(),
            );
            *vel_vs_magnet += *accel * sign;
            *pos_vs_magnet += *vel_vs_magnet;
        }
    }

    fn magnet_velocity(&self) -> Vec2 {
        match self.state {
            EntityState::Magnet { vel_vs_magnet, .. } => vel_vs_magnet,
            _ => Vec2::ZERO,
        }
    }

    /// The `Update` overrides' abort conditions: `StartFadingOut` when the
    /// weather/night no longer permits the kind.
    fn aborts(&self, ctx: &AmbienceCtx) -> bool {
        let (rain, day, eclipse) = (ctx.is_raining, ctx.day_time, ctx.eclipse);
        match self.kind {
            // `:216-219` butterflies, `:317-320` pegasus, `:365-368` vulture,
            // `:467-471` birds, `:527-530` seagulls, `:1094-1097` airship,
            // `:1146-1149` air balloon:
            SkyKind::Butterflies
            | SkyKind::Pegasus
            | SkyKind::Vulture
            | SkyKind::BirdsPack
            | SkyKind::Seagulls
            | SkyKind::Airship
            | SkyKind::AirBalloon => rain || !day || eclipse,
            // `:274-277` lost kite — rain does NOT abort it.
            SkyKind::LostKite => !day || eclipse,
            // `:420-423` pixie posse — the variant must match the daytime.
            SkyKind::PixiePosse => {
                let day_variant = self.variant == 1;
                (day_variant && !day)
                    || (!day_variant && day)
                    || rain
                    || eclipse
                    || ctx.blood_moon
                    || ctx.pumpkin_moon
                    || ctx.snow_moon
            }
            // `:633-636` gastropods — the calm-night family.
            SkyKind::Gastropod => {
                rain || day || ctx.blood_moon || ctx.pumpkin_moon || ctx.snow_moon
            }
            // `:740-743` slime balloons — the wind gate too.
            SkyKind::SlimeBalloons => !ctx.happy_windy_day || rain || !day || eclipse,
            // `:947-953` bats — raining or day aborts them. The hellbat's
            // `Update` is `base.Update` only (`:836-838`): no abort.
            SkyKind::Bats => rain || !day,
            // Wyvern (`:1003-1035`), the EOC family (`:1201-1289`) and the
            // meteor (`:1291-1314`) have no abort override — they fly their
            // lifetime out.
            SkyKind::HellBat
            | SkyKind::Wyvern
            | SkyKind::Eoc(_)
            | SkyKind::Meteor
            | SkyKind::BoneSerpent => false,
        }
    }
}

/// The `AmbientSky : CustomSky` manager (`AmbientSky.cs:1316-1486`) plus the
/// `AmbienceServer` scheduler (`AmbienceServer.cs:9-203`). The C# "Ambience"
/// sky activation flag collapses into "the entity list is non-empty".
pub(crate) struct Ambience {
    /// The seed of the last `spawn` call — stamped onto the entities it
    /// creates so the net tick can relay them (`NetAmbienceModule`).
    spawn_seed: i32,
    /// `AmbientSky._entities` (`:1320`, a `SlotVector<SkyEntity>(500)` — the
    /// live set is what matters; spawn order is kept).
    pub(crate) entities: Vec<SkyEntity>,
    /// `AmbientSky._frameCounter` (`:1322`): the global clock entities measure
    /// their lifetime against.
    frame_counter: i32,
    /// `AmbienceServer._updatesUntilNextAttempt` (`:26`).
    updates_until_next_attempt: i32,
    /// `AmbienceServer._forcedSpawns` (`:28`): queued
    /// `(SkyEntityType, targetPlayer)` pairs, -1 = any eligible player.
    forced_spawns: Vec<(SkyEntityType, i32)>,
    /// The 60 Hz accumulator (`Main.Update` runs the scheduler once per tick,
    /// `Main.cs:17214`).
    accumulator: f32,
    /// The `TR_AMBIENCE` test pin: the next forced spawn uses this seed
    /// instead of drawing the scheduler stream, keeping the entity's
    /// attributes reproducible for screenshots.
    pinned_seed: Option<i32>,
}

impl Default for Ambience {
    fn default() -> Self {
        Self::new()
    }
}

impl Ambience {
    pub(crate) fn new() -> Self {
        Self {
            entities: Vec::new(),
        spawn_seed: 0,
            frame_counter: 0,
            updates_until_next_attempt: 0,
            forced_spawns: Vec::new(),
            accumulator: 0.0,
            pinned_seed: None,
        }
    }

    /// `AmbienceServer.ForceEntitySpawn` (`:139-142`).
    pub(crate) fn force_spawn(&mut self, ty: SkyEntityType, target_player: i32) {
        self.forced_spawns.push((ty, target_player));
    }

    /// The `TR_AMBIENCE` test harness: queue a forced spawn whose entity
    /// attributes derive from the pinned seed.
    pub(crate) fn forced_spawn_for_tests(
        &mut self,
        ty: SkyEntityType,
        target_player: i32,
        seed: i32,
    ) {
        self.forced_spawns.push((ty, target_player));
        self.pinned_seed = Some(seed);
    }

    /// The per-frame entry point: fixed-step 60 Hz like the original's update
    /// cadence (the rain accumulator precedent), so a variable display rate
    /// does not drift the frame counter.
    pub(crate) fn update(&mut self, dt: f32, ctx: &AmbienceCtx, rng: &mut u64) {
        self.accumulator += dt;
        const STEP: f32 = 1.0 / 60.0;
        while self.accumulator >= STEP {
            self.accumulator -= STEP;
            self.step_once(ctx, rng);
        }
    }

    /// One 60 Hz tick: `Main.AmbienceServer.Update()` (`Main.cs:17214-17216`)
    /// then `AmbientSky.Update` (`:1343-1367`) ×`max(1, dayRate)`.
    fn step_once(&mut self, ctx: &AmbienceCtx, rng: &mut u64) {
        // The `MoonLord`/`MonolithMoonLord` conflict (`:1350-1353`) has no
        // ported counterpart sky yet — those skies cannot run in M1.
        self.update_scheduler(ctx, rng);
        for _ in 0..ctx.day_rate.max(1) {
            self.frame_counter += 1;
            let frame_count = self.frame_counter;
            for entity in &mut self.entities {
                // The Lost Kite's storm frame rate (`:268-271`).
                if matches!(entity.kind, SkyKind::LostKite) && ctx.is_storming {
                    entity.framing_speed = 4;
                }
                // The kite/balloon flip tracks the wind every tick
                // (`:272`, `:739`).
                if matches!(entity.kind, SkyKind::LostKite | SkyKind::SlimeBalloons) {
                    entity.flip = !(ctx.wind > 0.0);
                }
                entity.update(frame_count, ctx);
                if entity.aborts(ctx) {
                    entity.start_fading_out(frame_count);
                }
            }
            self.entities.retain(|e| e.is_active);
            // `:1361-1364`: an empty entity list deactivates the sky — the
            // port's update/draw simply sees no entities.
        }
    }

    /// `AmbienceServer.Update` (`:96-128`).
    fn update_scheduler(&mut self, ctx: &AmbienceCtx, rng: &mut u64) {
        self.spawn_forced_entities(ctx, rng);
        if self.updates_until_next_attempt > 0 {
            self.updates_until_next_attempt -= ctx.day_rate.max(1);
            return;
        }
        self.reset_spawn_time(rng);
        let eligible: Vec<SkyEntityType> = PRIMARY_CONDITIONS
            .iter()
            .copied()
            .filter(|ty| self.primary_condition(*ty, ctx))
            .collect();
        if eligible.is_empty() {
            return;
        }
        // `FindPlayerThatCanSeeBackgroundAmbience` (`:170-178`): the random
        // pick always consumes a draw, even with a single eligible player.
        if !self.player_sees_ambience(ctx) {
            return;
        }
        let _ = next_rand(rng);
        // The priority trick (`:117-123`): the first pass only considers
        // types that have a secondary condition AND satisfy it; `num == 0`
        // short-circuits the `Next(5)` draw.
        let secondary: Vec<SkyEntityType> = eligible
            .iter()
            .copied()
            .filter(|ty| {
                secondary_zone(*ty).is_some_and(|zone| zone(ctx)) && self.height_gate(*ty, ctx)
            })
            .collect();
        let pool = if !secondary.is_empty() && next_rand(rng) % 5 >= 3 {
            secondary
        } else {
            eligible
                .iter()
                .copied()
                .filter(|ty| {
                    self.height_gate(*ty, ctx) && secondary_zone(*ty).map_or(true, |zone| zone(ctx))
                })
                .collect()
        };
        if pool.is_empty() {
            return;
        }
        // `source2.ElementAt(Main.rand.Next(num))` — the C# enumerates a
        // `Dictionary`, whose order is implementation-defined; the port pins
        // the declaration order of `_spawnConditions` (documented).
        let ty = pool[(next_rand(rng) % pool.len() as u64) as usize];
        // `SpawnForPlayer` → the `NetAmbienceModule` loopback
        // (`AmbienceServer.cs:199-202`): the seed is one `Main.rand.Next()`
        // draw (`NetAmbienceModule.cs:13`).
        let seed = next_rand(rng) as i32;
        self.spawn(ctx, ty, seed);
    }

    /// `AmbienceServer.ResetSpawnTime` (`:130-137`): `Next(600, 7200)`, halved
    /// on tenth-anniversary worlds (no such worlds in the port).
    fn reset_spawn_time(&mut self, rng: &mut u64) {
        self.updates_until_next_attempt = 600 + (next_rand(rng) % (7200 - 600)) as i32;
    }

    /// `SpawnForcedEntities` (`:144-168`): every queued request spawns or dies
    /// in the tick it was processed.
    fn spawn_forced_entities(&mut self, ctx: &AmbienceCtx, rng: &mut u64) {
        if self.forced_spawns.is_empty() {
            return;
        }
        // The C# walks the queue backwards (`:150-167`).
        for (ty, _target) in std::mem::take(&mut self.forced_spawns).into_iter().rev() {
            // The `targetPlayer` routing (`:153-161`) collapses to the local
            // player in single player; the height gate still applies.
            if self.height_gate(ty, ctx) && self.player_sees_ambience(ctx) {
                let seed = self
                    .pinned_seed
                    .take()
                    .unwrap_or_else(|| next_rand(rng) as i32);
                self.spawn(ctx, ty, seed);
            }
        }
    }

    /// `IsPlayerInAPlaceWhereTheyCanSeeAmbience` (`:180-187`): the sky band or
    /// the hell band.
    fn player_sees_ambience(&self, ctx: &AmbienceCtx) -> bool {
        let sky = ctx.player_y <= ctx.world.world_surface as f32 * 16.0 + 1600.0;
        let hell = ctx.player_y >= (ctx.world.height - 200 - 100) as f32 * 16.0;
        sky || hell
    }

    /// `IsPlayerAtRightHeightForType` (`:87-94`): Hellbats need the hell band,
    /// everything else the sky band.
    fn height_gate(&self, ty: SkyEntityType, ctx: &AmbienceCtx) -> bool {
        if ty == Hellbats {
            ctx.player_y >= (ctx.world.height - 200 - 100) as f32 * 16.0
        } else {
            ctx.player_y <= ctx.world.world_surface as f32 * 16.0 + 1600.0
        }
    }

    /// `_spawnConditions` (`:60-76`).
    fn primary_condition(&self, ty: SkyEntityType, ctx: &AmbienceCtx) -> bool {
        let (rain, day, eclipse) = (ctx.is_raining, ctx.day_time, ctx.eclipse);
        let sunny = !rain && day && !eclipse;
        let sunset = day && ctx.world.time > 40500.0;
        let calm_night = !rain && !day && !ctx.blood_moon && !ctx.pumpkin_moon && !ctx.snow_moon;
        match ty {
            BirdsV | Vulture | Seagulls | Pegasus => sunny,
            Wyvern => sunny && ctx.hard_mode,
            Airship | SlimeBalloons => sunny && ctx.happy_windy_day,
            AirBalloon => sunny && !ctx.happy_windy_day,
            Eyeball => !day,
            Butterflies => {
                sunny
                    && !ctx.happy_windy_day
                    && !ctx.too_windy_for_butterflies
                    && ctx.butterfly_chance < 6
            }
            LostKite => day && !eclipse && ctx.happy_windy_day,
            Bats => (sunset && sunny) || calm_night,
            PixiePosse => sunny || calm_night,
            Gastropods => calm_night,
            EaterOfSouls | Crimera => sunny || calm_night,
            Hellbats => true,
            // `SkyEntityType.Meteor` has no entry in `_spawnConditions`
            // (`AmbienceServer.cs:60-76`) — the meteor sky is only reachable
            // through `ForceEntitySpawn` during a landing (`Main.cs:65481`).
            // `BoneSerpent` has no condition either (and no `Spawn` case).
            Meteor | BoneSerpent => false,
        }
    }

    /// `AmbientSky.Spawn` (`:1391-1485`): the factory switch. Each
    /// constructor's RNG draw order is normative — keep it in sync with the
    /// C# lines cited per arm.
    pub(crate) fn spawn(&mut self, ctx: &AmbienceCtx, ty: SkyEntityType, seed: i32) {
        if ty == BoneSerpent {
            // No case in `AmbientSky.Spawn` (`:1394-1480`): the packet is
            // silently ignored, exactly as the original does.
            return;
        }
        // The relayed seed: the host stamps every entity it creates and the
        // net tick broadcasts the unsent ones.
        self.spawn_seed = seed;
        let mut random = FastRandom::new(seed as u64);
        match ty {
            AirBalloon => self.entities.push(spawn_air_balloon(&mut random, ctx)),
            Airship => self.entities.push(spawn_airship(&mut random, ctx)),
            BirdsV => self.entities.push(spawn_birds_pack(&mut random, ctx)),
            Eyeball => self
                .entities
                .push(spawn_eoc(&mut random, ctx, EocVariant::Eyeball)),
            Meteor => self.entities.push(spawn_meteor(&mut random, ctx)),
            Wyvern => self.entities.push(spawn_wyvern(&mut random, ctx)),
            Bats => append_group(&mut self.entities, &mut random, ctx, GroupKind::Bats),
            Butterflies => self.entities.push(spawn_butterflies(&mut random, ctx)),
            LostKite => self.entities.push(spawn_lost_kite(&mut random, ctx)),
            Vulture => self.entities.push(spawn_vulture(&mut random, ctx)),
            PixiePosse => self.entities.push(spawn_pixie_posse(&mut random, ctx)),
            Seagulls => append_group(&mut self.entities, &mut random, ctx, GroupKind::Seagulls),
            SlimeBalloons => append_group(
                &mut self.entities,
                &mut random,
                ctx,
                GroupKind::SlimeBalloons,
            ),
            Gastropods => append_group(&mut self.entities, &mut random, ctx, GroupKind::Gastropods),
            Pegasus => self.entities.push(spawn_pegasus(&mut random, ctx)),
            EaterOfSouls => self
                .entities
                .push(spawn_eoc(&mut random, ctx, EocVariant::Eos)),
            Crimera => self
                .entities
                .push(spawn_eoc(&mut random, ctx, EocVariant::Crimera)),
            Hellbats => append_group(&mut self.entities, &mut random, ctx, GroupKind::HellBats),
            BoneSerpent => {}
        }
    }
}

/// The `SkyEntityType` C# name parser for the `TR_AMBIENCE` pin.
pub(crate) fn parse_sky_entity_type(name: &str) -> Option<SkyEntityType> {
    Some(match name {
        "BirdsV" => BirdsV,
        "Wyvern" => Wyvern,
        "Airship" => Airship,
        "AirBalloon" => AirBalloon,
        "Eyeball" => Eyeball,
        "Meteor" => Meteor,
        "BoneSerpent" => BoneSerpent,
        "Bats" => Bats,
        "Butterflies" => Butterflies,
        "LostKite" => LostKite,
        "Vulture" => Vulture,
        "PixiePosse" => PixiePosse,
        "Seagulls" => Seagulls,
        "SlimeBalloons" => SlimeBalloons,
        "Gastropods" => Gastropods,
        "Pegasus" => Pegasus,
        "EaterOfSouls" => EaterOfSouls,
        "Crimera" => Crimera,
        "Hellbats" => Hellbats,
        _ => return None,
    })
}

/// The `_secondarySpawnConditionsPerPlayer` table (`:77-84`) as zone lookups.
fn secondary_zone(ty: SkyEntityType) -> Option<fn(&AmbienceCtx) -> bool> {
    match ty {
        Vulture => Some(|c: &AmbienceCtx| c.zone_desert),
        PixiePosse | Gastropods | Pegasus => Some(|c: &AmbienceCtx| c.zone_hallow),
        Seagulls => Some(|c: &AmbienceCtx| c.zone_beach),
        EaterOfSouls => Some(|c: &AmbienceCtx| c.zone_corrupt),
        Crimera => Some(|c: &AmbienceCtx| c.zone_crimson),
        Bats => Some(|c: &AmbienceCtx| c.zone_jungle),
        _ => None,
    }
}

/// The `_spawnConditions` table in declaration order (`:60-76`) — the pool the
/// scheduler picks from (the C# enumerates a `Dictionary`; the port pins this
/// order).
const PRIMARY_CONDITIONS: [SkyEntityType; 17] = [
    BirdsV,
    Wyvern,
    Airship,
    AirBalloon,
    Eyeball,
    Butterflies,
    LostKite,
    Vulture,
    Bats,
    PixiePosse,
    Seagulls,
    SlimeBalloons,
    Gastropods,
    Pegasus,
    EaterOfSouls,
    Crimera,
    Hellbats,
];

enum GroupKind {
    Bats,
    Seagulls,
    SlimeBalloons,
    Gastropods,
    HellBats,
}

/// `SetPositionInWorldBasedOnScreenSpace` (`AmbientSky.cs:45-49`).
fn set_position_in_world(position: Vec2, depth: f32, player_center: Vec2) -> Vec2 {
    player_center + (position - player_center) * (depth / 3.0)
}

/// The side/flip rule every screen-space entity shares (`:184`, `:228`, …):
/// `Effects = wind <= 0 ? FlipHorizontally : None` — the flipped variant
/// spawns at the camera's right edge and flies left.
fn wind_flip(wind: f32) -> bool {
    !(wind > 0.0)
}

fn base_fading(fade_in: f32, fade_out: f32, brightness: f32, framing_speed: i32) -> SkyEntity {
    SkyEntity {
        kind: SkyKind::Butterflies, // overwritten by the caller
        frames: 1,
        variant: 0,
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        depth: 0.0,
        flip: false,
        rotation: 0.0,
        frame: 0,
        framing_speed,
        frame_offset: 0,
        life_time: 0,
        spawned_at: -1,
        opacity: 0.0,
        brightness_lerper: brightness,
        final_opacity_multiplier: 1.0,
        fade_in,
        fade_out,
        is_active: true,
        state: EntityState::Plain,
        net_seed: 0,
        net_sent: false,
    }
}

/// The shared screen-space spawn prologue for the single-flier family
/// (`:184-196` and its siblings): the flip picks the edge, then Y, depth and
/// the screen-space mapping draw in order. The lifetime draw is left to the
/// caller — the sheet draws sit between the depth and lifetime draws in the
/// C# ctors, and the order is normative.
fn spawn_flier_prologue(
    random: &mut FastRandom,
    ctx: &AmbienceCtx,
    kind: SkyKind,
    frames: u32,
    // The C# `+ band` term (`:194`, `:238`, ...).
    y_band: f32,
    // The C# multiplier's subtrahend: `worldSurface*16 - band_span`. The
    // butterflies/pixie family subtracts the band itself (4000); the other
    // six fliers subtract `1600 + 2400` (`:238`, `:296`, `:344`, `:447`,
    // `:1018`, `:1073`).
    band_span: f32,
    fade_in: f32,
    fade_out: f32,
    framing_speed: i32,
    depth_scale: f32,
) -> SkyEntity {
    let cam = virtual_camera_position(ctx.player_center);
    let mut e = base_fading(fade_in, fade_out, 0.2, framing_speed);
    e.kind = kind;
    e.frames = frames;
    e.flip = wind_flip(ctx.wind);
    const EDGE_MARGIN: f32 = 100.0;
    if e.flip {
        e.position.x = cam.x + VIRTUAL_CAMERA_SIZE.x + EDGE_MARGIN;
    } else {
        e.position.x = cam.x - EDGE_MARGIN;
    }
    // `Position.Y = NextFloat() * (worldSurface*16 - band_span) + y_band`.
    let surface_px = ctx.world.world_surface as f32 * 16.0;
    e.position.y = random.next_float() * (surface_px - band_span) + y_band;
    e.depth = random.next_float() * 3.0 + depth_scale;
    e.position = set_position_in_world(e.position, e.depth, ctx.player_center);
    e
}

/// `ButterfliesSkyEntity` (`:179-221`): sheet draw between depth and lifetime.
fn spawn_butterflies(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::Butterflies,
        19,
        4000.0,
        4000.0,
        0.15,
        0.85,
        5,
        3.0,
    );
    // `num2 = Next(2) + 1`: sheet 1 has 17 rows, sheet 2 has 19.
    let sheet = random.next(2) + 1;
    e.variant = sheet as u8;
    if sheet == 1 {
        e.frames = 17;
    }
    e.life_time = random.next_range(60, 121) * 60;
    e
}

/// `LostKiteSkyEntity` (`:223-279`): lifetime, then a random start frame.
fn spawn_lost_kite(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::LostKite,
        42,
        2400.0,
        4000.0,
        0.15,
        0.85,
        6,
        3.0,
    );
    e.life_time = random.next_range(60, 121) * 60;
    let start = random.next(e.frames as i32);
    for _ in 0..start {
        e.next_frame();
    }
    e
}

/// `PegasusSkyEntity` (`:281-327`).
fn spawn_pegasus(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::Pegasus,
        11,
        2400.0,
        4000.0,
        0.15,
        0.85,
        5,
        3.0,
    );
    e.life_time = random.next_range(60, 121) * 60;
    e
}

/// `VultureSkyEntity` (`:329-376`).
fn spawn_vulture(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::Vulture,
        10,
        2400.0,
        4000.0,
        0.15,
        0.85,
        5,
        3.0,
    );
    e.life_time = random.next_range(60, 121) * 60;
    e
}

/// `PixiePosseSkyEntity` (`:378-430`): the depth base is 2, brightness 0.6 and
/// the sheet variant is the daytime at spawn.
fn spawn_pixie_posse(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::PixiePosse,
        25,
        4000.0,
        4000.0,
        0.15,
        0.85,
        5,
        2.0,
    );
    e.variant = if ctx.day_time { 1 } else { 2 };
    e.brightness_lerper = 0.6;
    e.life_time = random.next_range(60, 121) * 60;
    e
}

/// `BirdsPackSkyEntity` (`:432-473`).
fn spawn_birds_pack(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::BirdsPack,
        4,
        2400.0,
        4000.0,
        0.15,
        0.85,
        5,
        3.0,
    );
    e.life_time = random.next_range(60, 121) * 60;
    e
}

/// `WyvernSkyEntity` (`:1003-1035`): a shorter lifetime.
fn spawn_wyvern(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::Wyvern,
        5,
        2400.0,
        4000.0,
        0.15,
        0.85,
        4,
        3.0,
    );
    e.life_time = random.next_range(40, 71) * 60;
    e
}

/// `AirshipSkyEntity` (`:1058-1099`): the flip is a coin, not the wind; the
/// fade window is 0.05/0.95 and the lifetime is shorter.
fn spawn_airship(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = spawn_flier_prologue(
        random,
        ctx,
        SkyKind::Airship,
        4,
        2400.0,
        4000.0,
        0.05,
        0.95,
        4,
        3.0,
    );
    e.flip = random.next(2) != 0;
    // The flip decides the edge (`:1063-1072`), so re-place after the draw.
    let cam = virtual_camera_position(ctx.player_center);
    e.position.x = if e.flip {
        cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
    } else {
        cam.x - 100.0
    };
    e.life_time = random.next_range(40, 71) * 60;
    e
}

/// `AirBalloonSkyEntity` (`:1101-1151`): raw world coordinates (no screen
/// mapping), a tile-anchored X, sheet + row draws before the lifetime and no
/// frame animation.
fn spawn_air_balloon(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = base_fading(0.05, 0.95, 0.2, i32::MAX);
    e.kind = SkyKind::AirBalloon;
    e.frames = 5;
    e.flip = random.next(2) != 0;
    let tile_x = (ctx.player_center.x / 16.0) as i32;
    e.position.x = (tile_x as f32 + 100.0 * (random.next_float() * 2.0 - 1.0)) * 16.0;
    e.position.y = ctx.world.world_surface as f32 * 16.0 - random.next_range(50, 81) as f32 * 16.0;
    e.depth = random.next_float() * 3.0 + 3.0;
    // `AirBalloons_Large` when `Next(2) == 0`, else `AirBalloons_Small`
    // (`:1113`); then the random current row (`:1115`).
    e.variant = if random.next(2) == 0 { 0 } else { 1 };
    e.frame = random.next(5) as u32;
    e.life_time = random.next_range(20, 51) * 60;
    e
}

/// `EOCSkyEntity` (`:1201-1289`) — also the base of `Crimera`/`EOS`.
fn spawn_eoc(random: &mut FastRandom, ctx: &AmbienceCtx, variant: EocVariant) -> SkyEntity {
    let mut e = base_fading(0.1, 0.9, 0.2, 5);
    e.kind = SkyKind::Eoc(variant);
    e.frames = match variant {
        EocVariant::Eyeball | EocVariant::Crimera => 3,
        EocVariant::Eos => 4,
    };
    e.depth = random.next_float() * 3.0 + 4.5;
    e.state = if random.next(4) != 0 {
        // `BeginZigZag` (`:1235-1251`): the direction draw is the argument,
        // then the lifetime and the waviness.
        let direction = if random.next(2) == 1 { 1 } else { -1 };
        e.life_time = random.next_range(18, 31) * 60;
        let waviness = random.next_float() + 1.0;
        let cam = virtual_camera_position(ctx.player_center);
        e.position.y = cam.y;
        e.position.x = if direction == 1 {
            cam.x - 100.0
        } else {
            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
        };
        EntityState::Eoc {
            state: 1,
            direction,
            waviness,
        }
    } else {
        // `BeginChasingPlayer` (`:1253-1258`).
        e.life_time = random.next_range(18, 31) * 60;
        let cam = virtual_camera_position(ctx.player_center);
        e.position =
            cam + VIRTUAL_CAMERA_SIZE * Vec2::new(random.next_float(), random.next_float());
        EntityState::Eoc {
            state: 2,
            direction: 0,
            waviness: 0.0,
        }
    };
    e.position = set_position_in_world(e.position, e.depth, ctx.player_center);
    e
}

/// `MeteorSkyEntity` (`:1291-1314`): crosses the surface column in 1200 ticks,
/// spawned mid-trajectory around the player.
fn spawn_meteor(random: &mut FastRandom, ctx: &AmbienceCtx) -> SkyEntity {
    let mut e = base_fading(0.05, 0.95, 0.5, 5);
    e.kind = SkyKind::Meteor;
    e.frames = 4;
    e.flip = random.next(2) != 0;
    e.depth = random.next_float() * 3.0 + 3.0;
    // `angle = PI/4 + NextFloat() * PI/2`; `num = (worldSurface*16) / dir.y`
    // and the velocity is `dir * (num / 1200)` (`:1300-1303`) — the vertical
    // leg always covers the surface column in 1200 ticks.
    let angle = std::f32::consts::FRAC_PI_4 + random.next_float() * std::f32::consts::FRAC_PI_2;
    let surface_px = ctx.world.world_surface as f32 * 16.0;
    let dir = Vec2::from_angle(angle);
    e.velocity = dir * (surface_px / dir.y / 1200.0);
    const RANGE: i32 = 100;
    e.position = ctx.player_center
        + Vec2::new(
            random.next_range(-RANGE, RANGE + 1) as f32,
            random.next_range(-RANGE, RANGE + 1) as f32,
        )
        - e.velocity * 1200.0 * 0.5;
    e.life_time = 1200;
    e.rotation = e.velocity.to_angle() + std::f32::consts::FRAC_PI_2;
    e
}

/// The five group spawners: `CreateGroup` draws the shared parameters, then
/// constructs each member whose own ctor draws (and then overrides) its
/// attributes — the discarded draws are part of the stream.
fn append_group(
    entities: &mut Vec<SkyEntity>,
    random: &mut FastRandom,
    ctx: &AmbienceCtx,
    which: GroupKind,
) {
    let cam = virtual_camera_position(ctx.player_center);
    let surface_px = ctx.world.world_surface as f32 * 16.0;
    let underworld_px = (ctx.world.height - 200) as f32 * 16.0;
    match which {
        GroupKind::Seagulls => {
            // `SeagullsGroupSkyEntity.CreateGroup` (`:549-583`).
            let count = random.next_range(5, 9);
            let flip = wind_flip(ctx.wind);
            let base_y = random.next_float() * (surface_px - 1600.0 - 2400.0) + 2400.0;
            let depth_base = random.next_float() * 2.0 + 1.0;
            let life_base = random.next_range(30, 61) * 60;
            let accel_base = Vec2::new(
                random.next_float() * 0.5 + 0.5,
                random.next_float() * 0.5 + 0.5,
            );
            let target = Vec2::new(
                random.next_float() * 2.0 - 1.0,
                random.next_float() * 2.0 - 1.0,
            ) * 100.0;
            for _ in 0..count {
                // The member ctor (`:485-515`) draws Y/depth/lifetime/offset/
                // start frame before the group overrides them.
                let mut member = base_fading(0.1, 0.9, 0.2, 4);
                member.kind = SkyKind::Seagulls;
                member.frames = 9;
                member.flip = flip;
                let _ = random.next_float() * (surface_px - 1600.0 - 2400.0) + 2400.0; // ctor Y
                let _ = random.next_float() * 3.0 + 3.0; // ctor depth
                let _ = random.next_range(60, 121) * 60; // ctor lifetime
                member.frame_offset = random.next(9);
                member.frame = random.next(9) as u32;
                member.depth = depth_base + random.next_float() * 0.5;
                member.position =
                    Vec2::new(
                        if flip {
                            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
                        } else {
                            cam.x - 100.0
                        },
                        base_y,
                    ) + Vec2::new(random.next_float() * 20.0 - 10.0, random.next_float() * 3.0)
                        * 50.0;
                member.life_time = life_base + random.next(301);
                let accel = accel_base * (random.next_float() * 0.3 + 0.85) * 0.05;
                member.position =
                    set_position_in_world(member.position, member.depth, ctx.player_center);
                member.state = EntityState::Magnet {
                    accel,
                    target,
                    pos_vs_magnet: Vec2::ZERO,
                    vel_vs_magnet: Vec2::ZERO,
                };
                entities.push(member);
            }
        }
        GroupKind::Gastropods => {
            // `GastropodGroupSkyEntity.CreateGroup` (`:655-688`).
            let count = random.next_range(3, 8);
            let flip = wind_flip(ctx.wind);
            let base_y = random.next_float() * (surface_px - 1600.0 - 3200.0) + 3200.0;
            let depth_base = random.next_float() * 3.0 + 2.0;
            let life_base = random.next_range(30, 61) * 60;
            let accel_base = Vec2::new(
                random.next_float() * 0.1 + 0.1,
                random.next_float() * 0.3 + 0.3,
            );
            let target = Vec2::new(
                random.next_float() * 2.0 - 1.0,
                random.next_float() * 2.0 - 1.0,
            ) * 120.0;
            for _ in 0..count {
                // The member ctor (`:596-620`) draws Y/depth/lifetime (no
                // frames — `FramingSpeed` is `int.MaxValue`).
                let mut member = base_fading(0.1, 0.9, 0.75, i32::MAX);
                member.kind = SkyKind::Gastropod;
                member.frames = 1;
                member.flip = flip;
                let _ = random.next_float() * (surface_px - 1600.0 - 3200.0) + 3200.0; // ctor Y
                let _ = random.next_float() * 3.0 + 2.0; // ctor depth
                let _ = random.next_range(60, 121) * 60; // ctor lifetime
                member.depth = depth_base + random.next_float() * 0.5;
                member.position =
                    Vec2::new(
                        if flip {
                            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
                        } else {
                            cam.x - 100.0
                        },
                        base_y,
                    ) + Vec2::new(random.next_float() * 20.0 - 10.0, random.next_float() * 3.0)
                        * 60.0;
                member.life_time = life_base + random.next(301);
                let accel = accel_base * (random.next_float() * 0.5) * 0.05;
                member.position =
                    set_position_in_world(member.position, member.depth, ctx.player_center);
                member.state = EntityState::Magnet {
                    accel,
                    target,
                    pos_vs_magnet: Vec2::ZERO,
                    vel_vs_magnet: Vec2::ZERO,
                };
                entities.push(member);
            }
        }
        GroupKind::SlimeBalloons => {
            // `SlimeBalloonGroupSkyEntity.CreateGroup` (`:752-785`).
            let count = random.next_range(5, 10);
            let flip = wind_flip(ctx.wind);
            let base_y = random.next_float() * (surface_px - 1600.0 - 2400.0) + 2400.0;
            let depth_base = random.next_float() * 3.0 + 3.0;
            let life_base = random.next_range(80, 121) * 60;
            let accel_base = Vec2::new(
                random.next_float() * 0.1 + 0.1,
                random.next_float() * 0.1 + 0.1,
            );
            let target = Vec2::new(
                random.next_float() * 2.0 - 1.0,
                random.next_float() * 2.0 - 1.0,
            ) * 150.0;
            for _ in 0..count {
                // The member ctor (`:701-726`) draws Y/depth/row/lifetime.
                let mut member = base_fading(0.025, 0.975, 0.2, i32::MAX);
                member.kind = SkyKind::SlimeBalloons;
                member.frames = 7;
                member.flip = flip;
                let _ = random.next_float() * (surface_px - 1600.0 - 4000.0) + 4000.0; // ctor Y
                let _ = random.next_float() * 3.0 + 3.0; // ctor depth
                member.frame = random.next(7) as u32; // ctor current row
                let _ = random.next_range(60, 121) * 60; // ctor lifetime
                member.depth = depth_base + random.next_float() * 0.5;
                member.position =
                    Vec2::new(
                        if flip {
                            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
                        } else {
                            cam.x - 100.0
                        },
                        base_y,
                    ) + Vec2::new(random.next_float() * 20.0 - 10.0, random.next_float() * 3.0)
                        * 80.0;
                member.life_time = life_base + random.next(301);
                let accel = accel_base * (random.next_float() * 0.2) * 0.05;
                member.position =
                    set_position_in_world(member.position, member.depth, ctx.player_center);
                member.state = EntityState::Magnet {
                    accel,
                    target,
                    pos_vs_magnet: Vec2::ZERO,
                    vel_vs_magnet: Vec2::ZERO,
                };
                entities.push(member);
            }
        }
        GroupKind::HellBats | GroupKind::Bats => {
            let hell = matches!(which, GroupKind::HellBats);
            // `HellBatsGoupSkyEntity.CreateGroup` (`:853-886`) /
            // `BatsGroupSkyEntity.CreateGroup` (`:967-1000`).
            let count = random.next_range(20, 40);
            let flip = wind_flip(ctx.wind);
            let base_y = if hell {
                random.next_float() * 800.0 + underworld_px
            } else {
                random.next_float() * (surface_px - 1600.0 - 2400.0) + 2400.0
            };
            let depth_base = if hell {
                random.next_float() * 5.0 + 3.0
            } else {
                random.next_float() * 3.0 + 3.0
            };
            let life_base = random.next_range(30, 61) * 60;
            let accel_base = Vec2::new(
                random.next_float() * 0.5 + 0.5,
                random.next_float() * 0.5 + 0.5,
            );
            let target = Vec2::new(
                random.next_float() * 2.0 - 1.0,
                random.next_float() * 2.0 - 1.0,
            ) * 100.0;
            for _ in 0..count {
                // The member ctors (`:798-828` hell / `:908-938` bats) draw
                // Y/depth/sheet/lifetime/offset/start frame in that order.
                let mut member = base_fading(0.1, 0.9, 0.2, 4);
                member.kind = if hell {
                    SkyKind::HellBat
                } else {
                    SkyKind::Bats
                };
                member.frames = 10;
                member.flip = flip;
                let _ = if hell {
                    random.next_float() * 400.0 + underworld_px // ctor Y
                } else {
                    random.next_float() * (surface_px - 1600.0 - 2400.0) + 2400.0
                };
                let _ = if hell {
                    random.next_float() * 5.0 + 3.0 // ctor depth
                } else {
                    random.next_float() * 3.0 + 3.0
                };
                // `HellBat{Next(1,3)}` (`:814`) / `Bat{Next(1,4)}` (`:924`).
                member.variant = random.next_range(1, if hell { 3 } else { 4 }) as u8;
                let _ = random.next_range(60, 121) * 60; // ctor lifetime
                member.frame_offset = random.next(10);
                member.frame = random.next(10) as u32;
                member.depth = depth_base + random.next_float() * 0.5;
                member.position =
                    Vec2::new(
                        if flip {
                            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
                        } else {
                            cam.x - 100.0
                        },
                        base_y,
                    ) + Vec2::new(random.next_float() * 20.0 - 10.0, random.next_float() * 3.0)
                        * 50.0;
                member.life_time = life_base + random.next(301);
                let accel = accel_base * (random.next_float() * 0.3 + 0.85) * 0.05;
                member.position =
                    set_position_in_world(member.position, member.depth, ctx.player_center);
                member.state = EntityState::Magnet {
                    accel,
                    target,
                    pos_vs_magnet: Vec2::ZERO,
                    vel_vs_magnet: Vec2::ZERO,
                };
                entities.push(member);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small world: `world_surface = 300` rows (4800 px), underworld at row
    /// 1000 (16000 px).
    fn test_world() -> tr_core::World {
        tr_core::World::new("ambience".into(), 4200, 1200)
    }

    fn ctx<'a>(world: &'a tr_core::World, day: bool, player_y: f32) -> AmbienceCtx<'a> {
        AmbienceCtx {
            world,
            day_time: day,
            eclipse: false,
            blood_moon: false,
            pumpkin_moon: false,
            snow_moon: false,
            hard_mode: false,
            is_raining: false,
            is_storming: false,
            happy_windy_day: false,
            too_windy_for_butterflies: false,
            butterfly_chance: 0,
            wind: 0.0,
            zone_desert: false,
            zone_hallow: false,
            zone_beach: false,
            zone_corrupt: false,
            zone_crimson: false,
            zone_jungle: false,
            player_center: Vec2::new(1000.0 * 16.0, player_y),
            player_y,
            day_rate: 1,
        }
    }

    const SURFACE_Y: f32 = 4800.0; // world_surface 300 * 16
    const HELL_Y: f32 = (1000.0 - 100.0) * 16.0;

    // --- Per-entity ctor characterization: the RNG draw order is normative ---

    #[test]
    fn butterflies_ctor_replays_the_csharp_draw_order() {
        // `AmbientSky.cs:184-206`: Y, depth, sheet, lifetime — in that order.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let seed = 42u64;
        let e = spawn_butterflies(&mut FastRandom::new(seed), &ctx);
        let mut reference = FastRandom::new(seed);

        let y = reference.next_float() * (4800.0 - 4000.0) + 4000.0;
        let depth = reference.next_float() * 3.0 + 3.0;
        let sheet = reference.next(2) + 1;
        let life = reference.next_range(60, 121) * 60;
        assert_eq!(e.depth, depth);
        assert_eq!(e.variant, sheet as u8);
        assert_eq!(e.frames, if sheet == 1 { 17 } else { 19 });
        assert_eq!(e.life_time, life);
        // `SetPositionInWorldBasedOnScreenSpace` (`:196`) from the wind-flipped
        // edge 100 px outside the virtual camera (`:188-193`).
        let cam = virtual_camera_position(ctx.player_center);
        let expected_x = if e.flip {
            cam.x + VIRTUAL_CAMERA_SIZE.x + 100.0
        } else {
            cam.x - 100.0
        };
        let raw = Vec2::new(expected_x, y);
        assert_eq!(
            e.position,
            ctx.player_center + (raw - ctx.player_center) * (e.depth / 3.0)
        );
    }

    #[test]
    fn the_six_fliers_spawn_in_the_minus_1600_band() {
        // `:238/:296/:344/:447/:1018/:1073`: the multiplier is
        // `worldSurface*16 - 1600 - 2400`, the base 2400 — the spawn band is
        // [2400, surface*16 - 1600], never inside the top 1600 px.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let cases = [
            SkyKind::LostKite,
            SkyKind::Pegasus,
            SkyKind::Vulture,
            SkyKind::BirdsPack,
            SkyKind::Wyvern,
            SkyKind::Airship,
        ];
        for kind in cases {
            let mut e = spawn_flier_prologue(
                &mut FastRandom::new(31),
                &ctx,
                kind,
                10,
                2400.0,
                4000.0,
                0.15,
                0.85,
                5,
                3.0,
            );
            let mut r = FastRandom::new(31);
            let expect_y = r.next_float() * (4800.0 - 4000.0) + 2400.0;
            let _ = r.next_float(); // depth
            // The raw Y before the screen-space mapping: recover it from the
            // mapping (pos = center + (raw - center) * depth/3).
            let raw_y =
                ctx.player_center.y + (e.position.y - ctx.player_center.y) / (e.depth / 3.0);
            assert!(
                (raw_y - expect_y).abs() < 1e-3,
                "{kind:?}: {raw_y} vs {expect_y}"
            );
            assert!(
                raw_y <= 4800.0 - 1600.0 + 1e-3,
                "{kind:?} too deep: {raw_y}"
            );
            let _ = &mut e;
        }
    }

    #[test]
    fn pixie_posse_depth_uses_the_two_base_and_night_variant() {
        // `:396-401`: `Depth = NextFloat()*3 + 2`; a night spawn takes sheet 2.
        let world = test_world();
        let night = ctx(&world, false, SURFACE_Y);
        let e = spawn_pixie_posse(&mut FastRandom::new(7), &night);
        let mut reference = FastRandom::new(7);
        let _ = reference.next_float(); // Y
        assert_eq!(e.depth, reference.next_float() * 3.0 + 2.0);
        assert_eq!(e.variant, 2);
        assert_eq!(e.brightness_lerper, 0.6);
    }

    #[test]
    fn air_balloon_uses_raw_world_coordinates_and_sheet_draws() {
        // `:1105-1122`: flip coin, tile-anchored X, Y above the surface,
        // depth, sheet, row, lifetime — in that order, no screen mapping.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let seed = 9u64;
        let e = spawn_air_balloon(&mut FastRandom::new(seed), &ctx);
        let mut r = FastRandom::new(seed);
        let flip = r.next(2) != 0;
        let x = (1000.0 + 100.0 * (r.next_float() * 2.0 - 1.0)) * 16.0;
        let y = 4800.0 - r.next_range(50, 81) as f32 * 16.0;
        let depth = r.next_float() * 3.0 + 3.0;
        let variant = r.next(2);
        let row = r.next(5);
        let life = r.next_range(20, 51) * 60;
        assert_eq!(e.flip, flip);
        assert_eq!(e.position.x, x);
        assert_eq!(e.depth, depth);
        assert_eq!(e.variant, variant as u8);
        assert_eq!(e.frame, row as u32);
        assert_eq!(e.life_time, life);
        assert_eq!(e.framing_speed, i32::MAX);
    }

    #[test]
    fn eoc_branches_split_three_to_one_and_the_chase_clamps() {
        // `:1219-1258`: `Next(4) != 0` -> zigzag (3/4), else the chase.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let mut zigzag = 0;
        for seed in 0..400u64 {
            let e = spawn_eoc(&mut FastRandom::new(seed), &ctx, EocVariant::Eyeball);
            if let EntityState::Eoc { state, .. } = e.state {
                if state == 1 {
                    zigzag += 1;
                }
            } else {
                panic!("EOC entity without the state machine");
            }
            assert!((4.5..=7.5).contains(&e.depth));
        }
        assert_eq!(zigzag, 300); // exactly 3/4 over the exact C# stream
        // The chase accelerates toward the player's head and clamps at 18
        // (`:1279-1288`): take the first chase-state seed and run it far away.
        let mut e = None;
        for seed in 0..400u64 {
            let candidate = spawn_eoc(&mut FastRandom::new(seed), &ctx, EocVariant::Eyeball);
            if matches!(candidate.state, EntityState::Eoc { state: 2, .. }) {
                e = Some(candidate);
                break;
            }
        }
        let mut e = e.expect("no chase-state seed in 400 draws");
        e.position = Vec2::new(0.0, 0.0);
        for i in 0..400 {
            e.update_velocity(i, &ctx);
        }
        // `Vector2.Clamp(velocity, (-18, -18), (18, 18))` clamps per axis
        // (`:1286`) — the diagonal may reach 18*sqrt(2).
        assert!(e.velocity.x.abs() <= 18.0 + 1e-3);
        assert!(e.velocity.y.abs() <= 18.0 + 1e-3);
    }

    #[test]
    fn meteor_lifetime_is_1200_ticks() {
        // `:1306`: `LifeTime = (int)num2` = 1200; framing 5, brightness 0.5.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let e = spawn_meteor(&mut FastRandom::new(3), &ctx);
        assert_eq!(e.life_time, 1200);
        assert_eq!(e.framing_speed, 5);
        assert_eq!(e.brightness_lerper, 0.5);
        // The velocity's Y leg crosses the surface column in 1200 ticks:
        // `velocity = dir(angle) * (surfacePx / 1200)` with the angle in
        // `[pi/4, 3pi/4]` — `velocity.y >= 0` and `velocity.y * 1200` is at
        // least `surfacePx / sqrt(2)`.
        // `num = surfacePx / dir.y` (`:1301`): the vertical leg covers the
        // surface column exactly in 1200 ticks.
        assert!(
            (e.velocity.y * 1200.0 - 4800.0).abs() < 1e-3,
            "{:?}",
            e.velocity
        );
    }

    #[test]
    fn group_sizes_follow_the_csharp_ranges() {
        // Seagulls `Next(5,9)` = 5-8; Gastropods `Next(3,8)` = 3-7;
        // SlimeBalloons `Next(5,10)` = 5-9; Bats/Hellbats `Next(20,40)` = 20-39.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..300u64 {
            // Spread the seeds over the 32-bit range the real `Main.rand`
            // seed draw covers; tiny seeds all land in the LCG's first bucket.
            let seed = 1_000_003 + i * 0x9E37_79B9;
            let mut entities = Vec::new();
            append_group(
                &mut entities,
                &mut FastRandom::new(seed),
                &ctx,
                GroupKind::Seagulls,
            );
            assert!((5..=8).contains(&entities.len()));
            seen.insert(entities.len());
        }
        assert!(seen.len() >= 3, "seagull counts never varied: {seen:?}");
        for i in 0..50u64 {
            let seed = 7_000_001 + i * 0x9E37_79B9;
            let mut entities = Vec::new();
            append_group(
                &mut entities,
                &mut FastRandom::new(seed),
                &ctx,
                GroupKind::Bats,
            );
            assert!((20..=39).contains(&entities.len()));
            let mut entities = Vec::new();
            append_group(
                &mut entities,
                &mut FastRandom::new(seed),
                &ctx,
                GroupKind::HellBats,
            );
            assert!((20..=39).contains(&entities.len()));
        }
        for i in 0..50u64 {
            let seed = 13_000_009 + i * 0x9E37_79B9;
            let mut entities = Vec::new();
            append_group(
                &mut entities,
                &mut FastRandom::new(seed),
                &ctx,
                GroupKind::Gastropods,
            );
            assert!((3..=7).contains(&entities.len()));
            let mut entities = Vec::new();
            append_group(
                &mut entities,
                &mut FastRandom::new(seed),
                &ctx,
                GroupKind::SlimeBalloons,
            );
            assert!((5..=9).contains(&entities.len()));
        }
    }

    #[test]
    fn seagull_members_share_the_group_depth_base() {
        // `:574`: member depth = group base + NextFloat()*0.5 — the spread is
        // at most 0.5 around the shared base, and the flip is shared.
        let world = test_world();
        let ctx = ctx(&world, true, SURFACE_Y);
        let mut entities = Vec::new();
        append_group(
            &mut entities,
            &mut FastRandom::new(11),
            &ctx,
            GroupKind::Seagulls,
        );
        let min = entities.iter().map(|e| e.depth).fold(f32::MAX, f32::min);
        let max = entities.iter().map(|e| e.depth).fold(f32::MIN, f32::max);
        assert!(max - min <= 0.5 + 1e-4);
        assert!(entities.iter().all(|e| e.flip == entities[0].flip));
    }

    #[test]
    fn bone_serpent_spawns_nothing() {
        // No `Spawn` case (`AmbientSky.cs:1394-1480`): the dead path is silent.
        let world = test_world();
        let mut amb = Ambience::new();
        amb.spawn(&ctx(&world, true, SURFACE_Y), BoneSerpent, 1);
        assert!(amb.entities.is_empty());
    }

    // --- Life cycle (FadingSkyEntity) ---

    #[test]
    fn fade_envelope_fades_in_then_out_and_expires() {
        // `:114-157`: opacity ramps to 1 by `life*fade_in`, falls from
        // `life*fade_out`, and the entity dies at `life`.
        let world = test_world();
        let mut amb = Ambience::new();
        amb.spawn(&ctx(&world, true, SURFACE_Y), BirdsV, 5);
        let e = &amb.entities[0];
        let life = e.life_time;
        let ramp_end = (life as f32 * e.fade_in) as i32;
        let fade_start = (life as f32 * e.fade_out) as i32;
        // Block the scheduler so the test drives exactly one entity.
        amb.updates_until_next_attempt = i32::MAX;
        let mut rng = 1u64;
        let c = ctx(&world, true, SURFACE_Y);
        amb.update(ramp_end as f32 / 60.0, &c, &mut rng);
        let e = &amb.entities[0];
        assert!((e.opacity - 1.0).abs() < 0.02, "opacity {}", e.opacity);
        amb.update((fade_start - ramp_end) as f32 / 60.0, &c, &mut rng);
        let mid = amb.entities[0].opacity;
        amb.update(10.0 / 60.0, &c, &mut rng);
        let later = amb.entities[0].opacity;
        assert!(later < mid, "{later} !< {mid}");
        amb.update(((life - fade_start) + 5) as f32 / 60.0, &c, &mut rng);
        assert!(amb.entities.is_empty());
    }

    #[test]
    fn abort_pulls_the_fade_forward() {
        // `StartFadingOut` (`:164-171`) shifts the spawn time so the fade-out
        // begins now: rain aborts the birds (`:467-471`).
        let world = test_world();
        let mut amb = Ambience::new();
        amb.spawn(&ctx(&world, true, SURFACE_Y), BirdsV, 5);
        let life = amb.entities[0].life_time;
        let fade_out_ticks = (life as f32 * amb.entities[0].fade_out) as i32;
        // Block the scheduler; ramp the fade-in to full.
        amb.updates_until_next_attempt = i32::MAX;
        let mut rng = 1u64;
        let ramp = (life as f32 * amb.entities[0].fade_in) as i32;
        amb.update(ramp as f32 / 60.0, &ctx(&world, true, SURFACE_Y), &mut rng);
        assert!((amb.entities[0].opacity - 1.0).abs() < 0.02);
        // Rain starts: the fade-out begins now (`:164-171`), i.e. the spawn
        // time moves back to `now - life*fadeOut`.
        let mut rainy = ctx(&world, true, SURFACE_Y);
        rainy.is_raining = true;
        amb.update(60.0 / 60.0, &rainy, &mut rng);
        let e = &amb.entities[0];
        // `StartFadingOut` fires on the FIRST rainy frame (ramp+1); the float
        // accumulator can round the stepped frame count by one.
        let expected = ramp + 1 - fade_out_ticks;
        assert!(
            (e.spawned_at - expected).abs() <= 2,
            "spawned_at {} != {expected}",
            e.spawned_at
        );
        assert!(e.opacity < 1.0);
    }

    // --- Scheduler ---

    #[test]
    fn spawn_timer_stays_inside_the_csharp_bounds() {
        // `Next(600, 7200)` (`:132`).
        let world = test_world();
        let mut amb = Ambience::new();
        let mut rng = 7u64;
        for _ in 0..200 {
            amb.reset_spawn_time(&mut rng);
            assert!((600..7200).contains(&amb.updates_until_next_attempt));
        }
    }

    #[test]
    fn biome_gated_types_win_forty_percent_then_the_rest_splits() {
        // `:117-123`: with a desert player the first pass is [Vulture] and
        // wins on `Next(5) >= 3` (40%); the broad pass keeps only types whose
        // secondary condition is absent or satisfied — here
        // [BirdsV, AirBalloon, Butterflies, Vulture] — so Vulture ends at
        // 0.4 + 0.6/4 = 0.55.
        let world = test_world();
        let mut amb = Ambience::new();
        let mut rng = 0xDEB_u64;
        let mut vulture = 0;
        let runs = 4000;
        for _ in 0..runs {
            amb.entities.clear();
            amb.updates_until_next_attempt = 0;
            let mut c = ctx(&world, true, SURFACE_Y);
            c.zone_desert = true;
            amb.update(1.0 / 60.0, &c, &mut rng);
            assert_eq!(amb.entities.len(), 1);
            if matches!(amb.entities[0].kind, SkyKind::Vulture) {
                vulture += 1;
            }
        }
        let share = vulture as f64 / runs as f64;
        // 0.4 (the priority trick) + 0.6 * 1/4 (the broad pool excludes the
        // gated types whose zone is off: `!ContainsKey || secondary`,
        // `AmbienceServer.cs:121`).
        assert!((0.50..=0.60).contains(&share), "vulture share {share}");
    }

    #[test]
    fn night_brings_the_eyeball_and_day_brings_the_birds() {
        let world = test_world();
        let mut amb = Ambience::new();
        let mut rng = 3u64;
        // Day, no zones: the broad pool keeps only types whose secondary
        // condition is absent or satisfied (`:121`) — here
        // [BirdsV, AirBalloon, Butterflies]; Hellbats fail the height gate and
        // the gated types fail their zones.
        let mut day_kinds = std::collections::BTreeSet::new();
        for _ in 0..50 {
            amb.entities.clear();
            amb.updates_until_next_attempt = 0;
            amb.update(1.0 / 60.0, &ctx(&world, true, SURFACE_Y), &mut rng);
            for e in &amb.entities {
                day_kinds.insert(format!("{:?}", e.kind));
            }
        }
        assert_eq!(
            day_kinds,
            ["Butterflies", "AirBalloon", "BirdsPack"]
                .into_iter()
                .map(str::to_string)
                .collect()
        );
        // Night: only the Eyeball survives the gates.
        let mut night_kinds = std::collections::BTreeSet::new();
        for _ in 0..50 {
            amb.entities.clear();
            amb.updates_until_next_attempt = 0;
            amb.update(1.0 / 60.0, &ctx(&world, false, SURFACE_Y), &mut rng);
            for e in &amb.entities {
                night_kinds.insert(format!("{:?}", e.kind));
            }
        }
        assert_eq!(
            night_kinds,
            ["Eoc(Eyeball)"].into_iter().map(str::to_string).collect()
        );
    }

    #[test]
    fn height_gates_admit_the_hell_band_only_for_hellbats() {
        // `:87-94`: Hellbats need the hell band, the rest the sky band. A
        // player deep in the underworld only ever gets hellbats.
        let world = test_world();
        let mut amb = Ambience::new();
        let mut rng = 5u64;
        for _ in 0..50 {
            amb.entities.clear();
            amb.updates_until_next_attempt = 0;
            amb.update(1.0 / 60.0, &ctx(&world, true, HELL_Y), &mut rng);
            assert!(
                amb.entities
                    .iter()
                    .all(|e| matches!(e.kind, SkyKind::HellBat))
            );
        }
        // A surface player's forced hellbat spawn is rejected (`:162`). Rain
        // blocks the regular scheduler (no sunny condition holds and the
        // hellbats fail the height gate), so the entity list stays empty.
        let mut amb = Ambience::new();
        let mut rng = 6u64;
        let mut rainy = ctx(&world, true, SURFACE_Y);
        rainy.is_raining = true;
        amb.force_spawn(Hellbats, -1);
        amb.update(1.0 / 60.0, &rainy, &mut rng);
        assert!(amb.entities.is_empty());
        // A forced meteor (a sky-band type) spawns through the same gate.
        amb.force_spawn(Meteor, -1);
        amb.update(1.0 / 60.0, &rainy, &mut rng);
        assert!(matches!(amb.entities[0].kind, SkyKind::Meteor));
    }

    #[test]
    fn scheduler_without_an_eligible_player_never_spawns() {
        // `:112-116`: a player between the bands sees no ambience.
        let world = test_world();
        let mut amb = Ambience::new();
        let mut rng = 9u64;
        let between = (SURFACE_Y + 1600.0 + 100.0).min(HELL_Y - 100.0);
        for _ in 0..30 {
            amb.updates_until_next_attempt = 0;
            amb.update(1.0 / 60.0, &ctx(&world, true, between), &mut rng);
            assert!(amb.entities.is_empty());
        }
    }

    // --- Draw colour chain (GetColor overrides, DW-3.4) ---

    fn draw_ctx<'a>(bg: &'a [f32; 16], backgrounds: bool) -> SkyDrawCtx<'a> {
        SkyDrawCtx {
            camera: Vec2::ZERO,
            zoom: 1.0,
            screen: Vec2::new(1280.0, 720.0),
            color_of_the_skies: [0.5, 0.6, 0.7],
            atmo: 1.0,
            bg_alpha_front: bg,
            background_enabled: backgrounds,
        }
    }

    #[test]
    fn pegasus_multiplies_by_bg_alpha_six() {
        // `:323-326`: `base.GetColor * bgAlphaFrontLayer[6]`.
        let mut bg = [0.0f32; 16];
        bg[6] = 0.25;
        let world = test_world();
        let c = ctx(&world, true, SURFACE_Y);
        let mut e = spawn_pegasus(&mut FastRandom::new(1), &c);
        e.opacity = 1.0;
        let rgba = e.draw_color(&draw_ctx(&bg, true));
        // The base lerp: sky (0.5,0.6,0.7) toward white by 0.2, times alpha 1.
        assert!(
            (rgba[0] - (0.5 + 0.5 * 0.2) * 0.25).abs() < 1e-5,
            "{rgba:?}"
        );
        assert!((rgba[3] - 0.25).abs() < 1e-5, "{rgba:?}");
    }

    #[test]
    fn vulture_takes_the_max_of_its_layer_set() {
        // `:371-375`: `max(bg[2], max(bg[5], bg[14], bg[13]))`.
        let mut bg = [0.0f32; 16];
        bg[2] = 0.1;
        bg[14] = 0.9;
        let world = test_world();
        let c = ctx(&world, true, SURFACE_Y);
        let mut e = spawn_vulture(&mut FastRandom::new(1), &c);
        e.opacity = 1.0;
        let rgba = e.draw_color(&draw_ctx(&bg, true));
        assert!((rgba[0] - (0.5 + 0.5 * 0.2) * 0.9).abs() < 1e-5, "{rgba:?}");
    }

    #[test]
    fn hellbat_vanishes_with_backgrounds_off_and_grays_with_depth() {
        // `:848-895`: `Lerp(White, Gray, Depth/15) * ... * BackgroundsOff`.
        let world = test_world();
        let c = ctx(&world, true, HELL_Y);
        let bg = [0.0f32; 16];
        let mut entities = Vec::new();
        append_group(
            &mut entities,
            &mut FastRandom::new(4),
            &c,
            GroupKind::HellBats,
        );
        let e = &entities[0];
        let off = e.draw_color(&draw_ctx(&bg, false));
        assert_eq!(off[3], 0.0);
        let on = e.draw_color(&draw_ctx(&bg, true));
        let t = e.depth / 15.0;
        let expect = (1.0 + (128.0 / 255.0 - 1.0) * t) * e.opacity;
        assert!((on[0] - expect).abs() < 1e-4, "{on:?} vs {expect}");
    }

    #[test]
    fn gastropod_lerps_toward_the_glow_colour_without_the_bg_alpha() {
        // `:639-642`: `Lerp(bg, (102, 0, 63), 0.75) * opacity`.
        let world = test_world();
        let c = ctx(&world, false, SURFACE_Y);
        let bg = [0.0f32; 16];
        let mut entities = Vec::new();
        append_group(
            &mut entities,
            &mut FastRandom::new(4),
            &c,
            GroupKind::Gastropods,
        );
        let e = &mut entities[0];
        e.opacity = 1.0;
        let rgba = e.draw_color(&draw_ctx(&bg, true));
        let t = e.brightness_lerper;
        let expect_r = 0.5 + (102.0 / 255.0 - 0.5) * t;
        assert!((rgba[0] - expect_r).abs() < 1e-5, "{rgba:?}");
        assert!((rgba[2] - (0.7 + (63.0 / 255.0 - 0.7) * t)).abs() < 1e-5);
    }

    #[test]
    fn seagull_bats_crimera_eos_colour_multipliers() {
        // `:539-542` seagulls [4]; `:962-965` bats max(3,0,10,11,12);
        // `:1171-1174` crimera [8]; `:1195-1198` EOS [1].
        let world = test_world();
        let c = ctx(&world, true, SURFACE_Y);
        let mut bg = [0.0f32; 16];
        bg[4] = 0.4;
        bg[3] = 0.9;
        bg[8] = 0.5;
        bg[1] = 0.6;
        let dctx = draw_ctx(&bg, true);
        let mut entities = Vec::new();
        append_group(
            &mut entities,
            &mut FastRandom::new(2),
            &c,
            GroupKind::Seagulls,
        );
        let e = &mut entities[0];
        e.opacity = 1.0;
        let base = 0.5 + 0.5 * 0.2;
        assert!((e.draw_color(&dctx)[0] - base * 0.4).abs() < 1e-5);
        let mut entities = Vec::new();
        append_group(&mut entities, &mut FastRandom::new(2), &c, GroupKind::Bats);
        let e = &mut entities[0];
        e.opacity = 1.0;
        assert!((e.draw_color(&dctx)[0] - base * 0.9).abs() < 1e-5);
        let mut e = spawn_eoc(&mut FastRandom::new(2), &c, EocVariant::Crimera);
        e.opacity = 1.0;
        assert!((e.draw_color(&dctx)[0] - base * 0.5).abs() < 1e-5);
        let mut e = spawn_eoc(&mut FastRandom::new(2), &c, EocVariant::Eos);
        e.opacity = 1.0;
        assert!((e.draw_color(&dctx)[0] - base * 0.6).abs() < 1e-5);
    }

    #[test]
    fn atmo_scales_the_drawn_colour() {
        // `CommonDraw` (`:63`): `GetColor(...) * Main.atmo` — the shared draw
        // applies the atmosphere density on top of every kind's colour.
        let world = test_world();
        let c = ctx(&world, true, SURFACE_Y);
        let mut e = spawn_birds_pack(&mut FastRandom::new(1), &c);
        e.opacity = 1.0;
        let bg = [0.0f32; 16];
        let mut ctx_draw = draw_ctx(&bg, true);
        ctx_draw.atmo = 0.5;
        let rgba = e.draw_color(&ctx_draw);
        // draw_color itself does not include atmo; draw_depth_range multiplies.
        assert!((rgba[3] - 1.0).abs() < 1e-5);
        let full = 0.5 + 0.5 * 0.2;
        assert!((rgba[0] * ctx_draw.atmo - full * 0.5).abs() < 1e-5);
    }

    // --- Determinism ---

    #[test]
    fn two_runs_with_the_same_seed_are_identical() {
        let world = test_world();
        let run = |rng_seed: u64| {
            let mut amb = Ambience::new();
            let mut rng = rng_seed;
            for _ in 0..600 {
                amb.updates_until_next_attempt = 0;
                let mut c = ctx(&world, true, SURFACE_Y);
                c.zone_desert = true;
                c.zone_hallow = true;
                amb.update(1.0 / 60.0, &c, &mut rng);
            }
            amb.entities
                .iter()
                .map(|e| {
                    format!(
                        "{:?}/{:?}/{}/{}/{}/{}/{}",
                        e.kind,
                        e.variant,
                        e.flip,
                        e.depth.to_bits(),
                        e.position.x.to_bits(),
                        e.position.y.to_bits(),
                        e.life_time
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(run(123), run(123));
        assert_ne!(run(123), run(124));
    }
}

// --- Draw side (the CommonDraw port; consumed by render::sky) ---

/// The draw-time inputs `CommonDraw` reads off `Main`
/// (`AmbientSky.cs:58-65`): the view centre, the backbuffer size, the sky
/// colour, the atmosphere density and the per-layer cross-fade alphas.
pub(crate) struct SkyDrawCtx<'a> {
    /// The view centre in world pixels (`Main.Camera.Center`).
    pub camera: Vec2,
    pub zoom: f32,
    /// The backbuffer size (`Main.screenWidth/Height`).
    pub screen: Vec2,
    /// `Main.ColorOfTheSkies`.
    pub color_of_the_skies: [f32; 3],
    /// `Main.atmo` (`Main.cs:2503`).
    pub atmo: f32,
    /// `Main.bgAlphaFrontLayer`.
    pub bg_alpha_front: &'a [f32; 16],
    /// `Main.BackgroundEnabled` — the hellbat visibility gate
    /// (`AmbientSky.cs:888-895`).
    pub background_enabled: bool,
}

impl SkyEntity {
    /// The texture key this entity draws from — the `Main.Assets.Request`
    /// argument of its class (`AmbientSky.cs`).
    pub(crate) fn sheet_name(&self) -> &'static str {
        match self.kind {
            SkyKind::Butterflies => match self.variant {
                1 => "Backgrounds/Ambience/ButterflySwarm1",
                _ => "Backgrounds/Ambience/ButterflySwarm2",
            },
            SkyKind::LostKite => "Backgrounds/Ambience/LostKite",
            SkyKind::Pegasus => "Backgrounds/Ambience/Pegasus",
            SkyKind::Vulture => "Backgrounds/Ambience/Vulture",
            SkyKind::PixiePosse => match self.variant {
                1 => "Backgrounds/Ambience/PixiePosse1",
                _ => "Backgrounds/Ambience/PixiePosse2",
            },
            SkyKind::BirdsPack => "Backgrounds/Ambience/BirdsVShape",
            SkyKind::Seagulls => "Backgrounds/Ambience/Seagull",
            SkyKind::Gastropod => "Backgrounds/Ambience/Gastropod",
            SkyKind::SlimeBalloons => "Backgrounds/Ambience/SlimeBalloons",
            SkyKind::HellBat => match self.variant {
                1 => "Backgrounds/Ambience/HellBat1",
                _ => "Backgrounds/Ambience/HellBat2",
            },
            SkyKind::Bats => match self.variant {
                1 => "Backgrounds/Ambience/Bat1",
                2 => "Backgrounds/Ambience/Bat2",
                _ => "Backgrounds/Ambience/Bat3",
            },
            SkyKind::Wyvern => "Backgrounds/Ambience/Wyvern",
            SkyKind::Airship => "Backgrounds/Ambience/FlyingShip",
            SkyKind::AirBalloon => match self.variant {
                0 => "Backgrounds/Ambience/AirBalloons_Large",
                _ => "Backgrounds/Ambience/AirBalloons_Small",
            },
            SkyKind::Eoc(EocVariant::Eyeball) => "Backgrounds/Ambience/EOC",
            SkyKind::Eoc(EocVariant::Eos) => match self.texture_tier() {
                1 => "Backgrounds/Ambience/EOS1",
                2 => "Backgrounds/Ambience/EOS2",
                _ => "Backgrounds/Ambience/EOS3",
            },
            SkyKind::Eoc(EocVariant::Crimera) => match self.texture_tier() {
                1 => "Backgrounds/Ambience/Crimera1",
                2 => "Backgrounds/Ambience/Crimera2",
                _ => "Backgrounds/Ambience/Crimera3",
            },
            SkyKind::Meteor => "Backgrounds/Ambience/Meteor",
            SkyKind::BoneSerpent => "Backgrounds/Ambience/BoneSerpant",
        }
    }

    /// The EOC depth tier (`:1158-1166`/`:1182-1190`):
    /// `3`, demoted at depth <= 6 and <= 5.
    fn texture_tier(&self) -> u8 {
        if self.depth <= 5.0 {
            1
        } else if self.depth <= 6.0 {
            2
        } else {
            3
        }
    }

    /// The `GetColor` overrides (`AmbientSky.cs`): the returned RGBA is the
    /// lerped/opacity-scaled colour *before* the `x Main.atmo` the shared
    /// draw applies. `ctx.color_of_the_skies` is `Main.ColorOfTheSkies`.
    fn draw_color(&self, ctx: &SkyDrawCtx) -> [f32; 4] {
        let sky = ctx.color_of_the_skies;
        let opacity = self.opacity * self.final_opacity_multiplier;
        match self.kind {
            // `:848-851`: `Lerp(White, Gray, Depth/15) * opacity * mult *
            // Helper_GetOpacityWithAccountingForBackgroundsOff` — the hellbat
            // is invisible with backgrounds off. (`Color.Gray` = 128.)
            SkyKind::HellBat => {
                let t = (self.depth / 15.0).clamp(0.0, 1.0);
                let on = if ctx.background_enabled { 1.0 } else { 0.0 };
                let gray = 128.0 / 255.0;
                let ch = (1.0 + (gray - 1.0) * t) * opacity * on;
                [ch, ch, ch, opacity * on]
            }
            // `:639-642`: the gastropod glows — `Lerp(bg,
            // Colors.AmbientNPCGastropodLight (102, 0, 63), BrightnessLerper)
            // * opacity * mult`, with no ocean-line term.
            SkyKind::Gastropod => {
                let t = self.brightness_lerper;
                let light = [102.0 / 255.0, 0.0, 63.0 / 255.0];
                [
                    (sky[0] + (light[0] - sky[0]) * t) * opacity,
                    (sky[1] + (light[1] - sky[1]) * t) * opacity,
                    (sky[2] + (light[2] - sky[2]) * t) * opacity,
                    opacity,
                ]
            }
            // The `FadingSkyEntity` base (`:159-162`): `Lerp(bg, White,
            // BrightnessLerper) * opacity * mult * oceanLine` — the ocean line
            // is the Q4 deviation (constant 1). The per-type overrides that
            // only scale (`:323-326` pegasus [6], `:371-375` vulture,
            // `:539-542` seagulls [4], `:962-965` bats, `:1171-1174` crimera
            // [8], `:1195-1198` EOS [1]) fold their `bgAlphaFrontLayer`
            // multiplier in here.
            kind => {
                let t = self.brightness_lerper;
                let mut rgba = [
                    (sky[0] + (1.0 - sky[0]) * t) * opacity,
                    (sky[1] + (1.0 - sky[1]) * t) * opacity,
                    (sky[2] + (1.0 - sky[2]) * t) * opacity,
                    opacity,
                ];
                let bg = ctx.bg_alpha_front;
                let multiplier = match kind {
                    SkyKind::Pegasus => bg[6],
                    SkyKind::Vulture => bg[2].max(bg[5].max(bg[14].max(bg[13]))),
                    SkyKind::Seagulls => bg[4],
                    SkyKind::Bats => bg[3].max(bg[0].max(bg[10].max(bg[11].max(bg[12])))),
                    SkyKind::Eoc(EocVariant::Crimera) => bg[8],
                    SkyKind::Eoc(EocVariant::Eos) => bg[1],
                    _ => 1.0,
                };
                for c in &mut rgba {
                    *c *= multiplier;
                }
                rgba
            }
        }
    }
}

/// `SkyManager.DrawDepthRange`'s semantics on the port's surface pass: draw
/// every entity with `Depth in (min_depth, last]`, then move the tracker down
/// (`SkyManager.cs:53-60`). A call with `min_depth >= *last` no-ops — exactly
/// the stale-parallax no-op of `Main.cs:59890`.
pub(crate) fn draw_range_min(
    r: &mut tr_render::SpriteRenderer,
    entities: &[SkyEntity],
    lookups: &crate::assets::Lookups,
    ctx: &SkyDrawCtx,
    min_depth: f32,
    last: &mut f32,
) {
    if *last <= min_depth {
        return;
    }
    draw_depth_range(r, entities, lookups, ctx, min_depth, *last);
    *last = min_depth;
}

/// `SkyManager.DrawRemainingDepth` (`SkyManager.cs:70-75`): flush everything
/// below the tracker (`Main.cs:59626`).
pub(crate) fn draw_range_remaining(
    r: &mut tr_render::SpriteRenderer,
    entities: &[SkyEntity],
    lookups: &crate::assets::Lookups,
    ctx: &SkyDrawCtx,
    last: &mut f32,
) {
    draw_depth_range(r, entities, lookups, ctx, f32::NEG_INFINITY, *last);
    *last = f32::NEG_INFINITY;
}

/// `AmbientSky.Draw` (`:1369-1380`) then `CommonDraw` (`:58-65`): the depth
/// cull, the `(1/Depth, 0.9/Depth)` parallax about the view centre, the
/// `depthScale / Depth` scale with the manager's `depthScale = 3` (`:1378`),
/// the centred origin and the `GetColor * Main.atmo` colour.
fn draw_depth_range(
    r: &mut tr_render::SpriteRenderer,
    entities: &[SkyEntity],
    lookups: &crate::assets::Lookups,
    ctx: &SkyDrawCtx,
    min_depth: f32,
    max_depth: f32,
) {
    for e in entities {
        if e.depth <= min_depth || e.depth > max_depth {
            continue;
        }
        let Some(sheet) = lookups.ambience.get(e.sheet_name()) else {
            continue;
        };
        // The `Draw` overrides' `depthScale - x` arguments (`:428` pixie and
        // `:646` gastropod at -0.1, `:546` seagull at -1.5; the rest use the
        // manager's 3, `:1378`).
        let depth_scale = match e.kind {
            SkyKind::Seagulls => 1.5,
            SkyKind::PixiePosse | SkyKind::Gastropod => 2.9,
            _ => 3.0,
        };
        // `GetDrawPositionByDepth` (`:67-70`).
        let depth = e.depth;
        let parallax = Vec2::new(1.0 / depth, 0.9 / depth);
        let draw_pos = ctx.camera + (e.position - ctx.camera) * parallax;
        // The sprite's screen-space centre: the batch position is
        // `drawPos - Main.Camera.UnscaledPosition` (`:63`) — the unscaled
        // top-left of the view is `camera - screen*0.5/zoom`.
        let top_left = ctx.camera - ctx.screen * 0.5 / ctx.zoom;
        let centre = (draw_pos - top_left) * ctx.zoom;
        // `scale: depthScale / Depth` on the source frame; the frame is row
        // `frame` of the 1xN grid (`SourceRectangle`, `:34`).
        let scale = depth_scale / depth;
        let frame_h = sheet.h / sheet.frames as f32;
        let rows = sheet.frames as f32;
        let frame_uv = [
            sheet.uv[0],
            sheet.uv[1] + (sheet.uv[3] - sheet.uv[1]) * (e.frame as f32 / rows),
            sheet.uv[2],
            sheet.uv[1] + (sheet.uv[3] - sheet.uv[1]) * ((e.frame + 1) as f32 / rows),
        ];
        // The sprite-batch transform carries the zoom, so the on-screen size
        // is `source * scale * zoom` (the batch position mapping above is the
        // same transform).
        let size = [sheet.w * scale * ctx.zoom, frame_h * scale * ctx.zoom];
        // `color: GetColor(Main.ColorOfTheSkies) * Main.atmo` (`:63`).
        let mut color = e.draw_color(ctx);
        for c in &mut color {
            *c *= ctx.atmo;
        }
        // The flip swaps the UV horizontally (the `SpriteEffects` port the
        // cloud pass uses).
        let uv = if e.flip {
            [frame_uv[2], frame_uv[1], frame_uv[0], frame_uv[3]]
        } else {
            frame_uv
        };
        crate::render::bg::push_screen_rot(
            r,
            ctx.camera,
            ctx.zoom,
            ctx.screen,
            centre.to_array(),
            size,
            uv,
            color,
            e.rotation,
        );
    }
}
