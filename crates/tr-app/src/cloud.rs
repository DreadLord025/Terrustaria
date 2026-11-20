//! The live cloud field — `Terraria.Cloud` (`Cloud.cs:9-451`) ported 1:1:
//! the 200-slot pool, `addCloud`'s spawn recipe in its exact RNG order, the
//! per-tick `Update` (wind drift, alpha ramps, rotation jitter, retire bounds),
//! `UpdateClouds`' bubble pass + spawn/kill balance, `UpdateCloudParallax`'s
//! camera-delta shift and `resetClouds`. Also the two `Main` walkers that feed
//! it: `updateCloudLayer` (`Main.cs:13174-13231`, the `cloudBGActive` state
//! machine) and `UpdateWeather`'s `numCloudsTemp` walk (`Main.cs:58952-59036`).
//!
//! RNG: `Cloud.rand` is a PRIVATE static `UnifiedRandom` (`Cloud.cs:35`,
//! `:447-450`) — separate from `Main.rand`. The C# seeds it with OS entropy
//! (`new UnifiedRandom()`); the port pins it to the world seed (the
//! `make_stars` precedent) so sessions stay deterministic. The `Main.rand`
//! draws of `updateCloudLayer`/the `numClouds` walk run outside every
//! `SwapRandom` block (`Main.DoUpdate` `:17226`/`:17228-17231`, before
//! `DoUpdateInWorld` `:17280`), so the caller passes the bare-stream stand-in
//! (`NamedRngs::global`).
//!
//! Draw side: `render/sky.rs`'s three `DrawClouds_*` passes consume this pool
//! (`Main.cs:59644-59703` + `NextHorizonRenderer.DrawCloud` `:241-252`);
//! W4-g69 wired them plus the tick glue — `updateCloudLayer`/the numClouds
//! tail/`UpdateClouds` ride `world_sim::weather::update_world_tick`,
//! `UpdateCloudParallax` + the `DrawBG` `resetClouds` latch ride the draw head
//! (`app/frame/draw.rs`).

use glam::Vec2;
use tr_core::{UnifiedRandom, World};

use crate::assets;

/// `Main.cloud[200]` (`Main.cs:66843`).
pub(crate) const CLOUD_SLOTS: usize = 200;

/// `CloudID.Sets.RareClouds_Normal` (`CloudID.cs:18-22`).
const RARE_NORMAL: [i32; 20] = [
    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
];
/// `CloudID.Sets.RareClouds_Celebration` (`CloudID.cs:23-27`).
const RARE_CELEBRATION: [i32; 16] = [
    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 41,
];

/// One `Cloud` (`Cloud.cs:11-33`). `width`/`height` are the `(int)(tex * scale)`
/// caches `addCloud`/`Update` maintain — the spawn-time overlap rejection reads
/// the OLD-type sizes (`Cloud.cs:84-85` runs before the type roll), kept verbatim.
#[derive(Clone, Copy, Default)]
pub(crate) struct Cloud {
    pub position: Vec2,
    pub scale: f32,
    pub rotation: f32,
    pub r_speed: f32,
    /// `Cloud.sSpeed` — set to 0 at spawn, never read elsewhere in the C#.
    pub s_speed: f32,
    pub active: bool,
    /// `SpriteEffects.FlipHorizontally` (`Cloud.cs:88-91`).
    pub flip: bool,
    pub kind: i32,
    pub width: i32,
    pub height: i32,
    pub alpha: f32,
    pub kill: bool,
}

/// The cross-system inputs the C# reads off `Main`/`NPC`/`WorldGen` statics.
#[derive(Clone, Copy)]
pub(crate) struct CloudEnv {
    /// `Main.screenWidth`/`screenHeight`.
    pub screen: Vec2,
    /// `Main.windSpeedCurrent`.
    pub wind_speed_current: f32,
    /// `Main.player[Main.myPlayer].velocity.X` (`Cloud.cs:95`).
    pub player_vx: f32,
    /// `Main.gameMenu`.
    pub game_menu: bool,
    /// `Main.dayRate`.
    pub day_rate: i32,
    /// `Main.cloudAlpha` — the RAW intensity (can exceed 1), not the clamped
    /// visual value.
    pub cloud_alpha: f32,
    /// `Main.bgAlphaFrontLayer[4]` — the ocean front layer (`Cloud.cs:393`).
    pub bg_alpha_front_4: f32,
    /// `Main.dontStarveWorld` / `Main.tenthAnniversaryWorld` (`Cloud.cs:140`).
    pub dont_starve: bool,
    pub tenth_anniversary: bool,
    /// `NPC.downedBoss1/2/3`, `WorldGen.crimson`, `Main.hardMode` — the
    /// `RollRareCloud` gates (`Cloud.cs:192-221`).
    pub downed_boss1: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    pub crimson: bool,
    pub hard_mode: bool,
    /// The drunk-menu type override's flags (`Cloud.cs:342-382`):
    /// `WorldGen.drunkWorldGenText && Main.gameMenu` gates the whole arm.
    pub drunk_world_text: bool,
    pub dont_starve_gen: bool,
    pub get_good_gen: bool,
    pub only_shimmer_ocean: bool,
}

impl CloudEnv {
    /// Gather the world-side flags; the caller supplies the live per-tick
    /// values. `WorldGen.crimson` is not persisted at world level (the
    /// `lang.rs:222` documented default — corruption), and `NPC.downedBoss2`
    /// folds to `downed_eow` (the Brain's downed flag has no world field yet).
    /// The drunk-menu arm's gen flags stay false in-world (`gameMenu` false).
    pub(crate) fn gather(
        world: &World,
        screen: Vec2,
        wind_speed_current: f32,
        player_vx: f32,
        game_menu: bool,
        day_rate: i32,
        cloud_alpha: f32,
        bg_alpha_front_4: f32,
    ) -> Self {
        use tr_core::seeds::SeedFlags;
        Self {
            screen,
            wind_speed_current,
            player_vx,
            game_menu,
            day_rate,
            cloud_alpha,
            bg_alpha_front_4,
            dont_starve: world.seeds.contains(SeedFlags::DONT_STARVE),
            tenth_anniversary: world.seeds.contains(SeedFlags::TENTH_ANNIVERSARY),
            downed_boss1: world.downed_eoc,
            downed_boss2: world.downed_eow,
            downed_boss3: world.downed_skeletron,
            crimson: false,
            hard_mode: world.hard_mode,
            drunk_world_text: false,
            dont_starve_gen: false,
            get_good_gen: false,
            only_shimmer_ocean: false,
        }
    }
}

/// `TextureAssets.Cloud[type].Width()/Height()` — the port's decoded sheet
/// sizes; a missing sheet (headless lookups) reads 0×0.
fn tex_size(lookups: &assets::Lookups, kind: i32) -> (f32, f32) {
    lookups
        .clouds
        .get(kind.max(0) as usize)
        .and_then(|t| t.as_ref())
        .map(|t| (t.w, t.h))
        .unwrap_or((0.0, 0.0))
}

/// `Cloud.GetParallax` (`Cloud.cs:318-338`) — verbatim, including the DOUBLE
/// compare on the 1.15 bucket edge (`(double)scale <= 1.15`): a float 1.15f
/// promotes to 1.14999997… which is `<= 1.15`, so scale 1.15f lands in the
/// middle bucket here while `DrawClouds_Closest`'s FLOAT `>= 1.15f` also
/// claims it — the C# draws such a cloud twice, kept.
pub(crate) fn cloud_parallax(scale: f32) -> f32 {
    let (num, num2) = if scale < 1.0 {
        (0.07, (scale + 0.15 + 1.0) / 2.0)
    } else if (scale as f64) <= 1.15 {
        (0.19, scale - 0.075)
    } else {
        (0.23, scale - 0.15 - 0.075)
    };
    num * (num2 * num2)
}

/// The three `DrawClouds_*` bucket gates, verbatim:
/// distant `scale < 1f` (`Main.cs:59697`), closer `(double)scale < 1.15 &&
/// scale >= 1f` (`:59674`), closest `scale >= 1.15f` (`:59655`).
pub(crate) fn cloud_in_bucket(scale: f32, pass: u8) -> bool {
    match pass {
        1 => scale < 1.0,
        2 => (scale as f64) < 1.15 && scale >= 1.0,
        _ => scale >= 1.15,
    }
}

/// `Cloud.cloudColor` (`Cloud.cs:227-235`): every channel (alpha included) is
/// `(byte)(float)(int)((float)(int)bg * min(scale*Alpha, 1))` — the XNA byte
/// truncation, the `render/gore.rs` convention.
pub(crate) fn cloud_color(scale: f32, alpha: f32, bg: [u8; 4]) -> [u8; 4] {
    let mut num = scale * alpha;
    if num > 1.0 {
        num = 1.0;
    }
    let ch = |c: u8| ((c as f32 * num) as i32) as u8;
    [ch(bg[0]), ch(bg[1]), ch(bg[2]), ch(bg[3])]
}

/// `NextHorizonRenderer.OriginalColorsForCloud` (`NextHorizonRenderer.cs:254-263`):
/// the distant pass pulls red/green down through the same byte cast.
pub(crate) fn original_colors_for_cloud(color: &mut [u8; 4], scale: f32, pass: u8) {
    if pass == 1 {
        let num = scale * 0.8;
        let num2 = (scale + 1.0) / 2.0 * 0.9;
        color[0] = ((color[0] as f32 * num) as i32) as u8;
        color[1] = ((color[1] as f32 * num2) as i32) as u8;
    }
}

/// The live pool + the `Main` cloud-count statics that drive it.
pub(crate) struct CloudField {
    /// `Main.cloud[200]`.
    pub slots: [Cloud; CLOUD_SLOTS],
    /// `Cloud.rand` (`Cloud.cs:35`) — the private static stream, pinned to the
    /// world seed (module doc).
    pub rand: UnifiedRandom,
    /// `Cloud.lastCameraCenter` (`Cloud.cs:37`).
    last_camera_center: Option<Vec2>,
    /// `Main.numClouds` (`Main.cs:1259`) — init 200 (`Main.cs:66840`).
    pub num_clouds: i32,
    /// `Main.numCloudsTemp` (`Main.cs:1261`).
    pub num_clouds_temp: i32,
    /// `Main.weatherCounter` (`Main.cs:1257`) — the jump cadence.
    pub weather_counter: i32,
    /// `Main.cloudBGActive` (`Main.cs:551`) — the overcast state machine's
    /// value: >0 active (≥1 while the deck shows), <0 the cooldown climb.
    pub cloud_bg_active: f32,
    /// `Main.resetClouds` (`Main.cs:1281`, set at `:66845`, consumed by
    /// `DrawBG` `:63980-63984`).
    pub reset_pending: bool,
}

impl CloudField {
    pub(crate) fn new(world_seed: u64) -> Self {
        Self {
            slots: [Cloud::default(); CLOUD_SLOTS],
            rand: UnifiedRandom::new((world_seed ^ 0x434C_4F55) as i32),
            last_camera_center: None,
            num_clouds: 200,
            num_clouds_temp: 200,
            weather_counter: 0,
            cloud_bg_active: 0.0,
            reset_pending: true,
        }
    }

    /// `Cloud.resetClouds` (`Cloud.cs:39-58`). The caller applies the
    /// `Main.windSpeedCurrent = Main.windSpeedTarget` head (`:43`) on the
    /// `Wind` before calling. The `cloud[j].Alpha = 1` loop indexes `j`, NOT
    /// the slot `addCloud` picked — verbatim quirk; the final loop sets every
    /// slot's Alpha anyway.
    pub(crate) fn reset_clouds(&mut self, env: &CloudEnv, lookups: &assets::Lookups) {
        for c in self.slots.iter_mut() {
            c.active = false;
        }
        for j in 0..(self.num_clouds.max(0) as usize).min(CLOUD_SLOTS) {
            self.add_cloud(env, lookups);
            self.slots[j].alpha = 1.0;
        }
        for c in self.slots.iter_mut() {
            c.alpha = 1.0;
        }
    }

    /// `Cloud.addCloud` (`Cloud.cs:60-178`) — the spawn recipe in its exact
    /// RNG order, short-circuits included. The `netMode == 2` head is the
    /// dedicated server (never the port's single-player session).
    pub(crate) fn add_cloud(&mut self, env: &CloudEnv, lookups: &assets::Lookups) {
        // `:66-78`: first inactive slot.
        let Some(num) = self.slots.iter().position(|c| !c.active) else {
            return;
        };
        let c = &mut self.slots[num];
        c.kill = false;
        c.r_speed = 0.0;
        c.s_speed = 0.0;
        // `:82-83`.
        c.scale = self.rand.next_range(70, 131) as f32 * 0.01;
        c.rotation = self.rand.next_range(-10, 11) as f32 * 0.01;
        // `:84-85`: the OLD type's sizes (the type roll comes later).
        let (w0, h0) = tex_size(lookups, c.kind);
        c.width = (w0 * c.scale) as i32;
        c.height = (h0 * c.scale) as i32;
        c.alpha = 0.0;
        c.flip = false;
        // `:88-91`.
        if self.rand.next_max(2) == 0 {
            c.flip = true;
        }
        // `:92-106`: the spawn-side margins from the effective wind.
        let mut num2 = env.wind_speed_current;
        if !env.game_menu {
            num2 = env.wind_speed_current - env.player_vx * 0.1;
        }
        let mut num3 = 0;
        let mut num4 = 0;
        if num2 > 0.0 {
            num3 -= 200;
        }
        if num2 < 0.0 {
            num4 += 200;
        }
        // `:107-108`.
        let num5 = 300;
        let x = self
            .rand
            .next_range(num3 - num5, env.screen.x as i32 + num4 + num5) as f32;
        c.alpha = 0.0;
        // `:110-114`.
        c.position.y = self.rand.next_range(
            (-env.screen.y * 0.25) as i32,
            (env.screen.y * 0.15) as i32,
        ) as f32;
        if self.rand.next_max(3) == 0 {
            c.position.y -= self.rand.next_max((env.screen.y * 0.1) as i32) as f32;
        }
        // `:115`.
        c.kind = self.rand.next_max(4);
        // `:116-127`: the rain-cloud branch — dark storm types 18..21 while
        // raining (`cloudAlpha > 0`, 3-in-4) or overcast (`cloudBGActive >= 1`,
        // coin flip). The second clause's draw only happens when the first
        // short-circuits false — verbatim.
        if (env.cloud_alpha > 0.0 && self.rand.next_max(4) != 0)
            || (self.cloud_bg_active >= 1.0 && self.rand.next_max(2) == 0)
        {
            c.kind = self.rand.next_range(18, 22);
            if (c.scale as f64) >= 1.15 {
                c.position.y -= 150.0;
            }
            if c.scale >= 1.0 {
                c.position.y -= 150.0;
            }
        } else if self.cloud_bg_active <= 0.0
            && env.cloud_alpha == 0.0
            && c.scale < 1.0
            && c.position.y < -env.screen.y * 0.15
            && (self.num_clouds as f64) <= 80.0
        {
            // `:128-131`: high wisps on a clear sky.
            c.kind = self.rand.next_range(9, 14);
        } else if (((c.scale as f64) < 1.15 && c.position.y < -env.screen.y * 0.3)
            || ((c.scale as f64) < 0.85 && c.position.y < env.screen.y * 0.15))
            && ((self.num_clouds as f64) > 70.0 || self.cloud_bg_active >= 1.0)
        {
            // `:132-135`: the whole condition (count gate included) is one
            // C# `else if` — a failed count gate FALLS THROUGH to branch 4.
            // No draws in the condition, so the grouping is RNG-equivalent.
            c.kind = self.rand.next_range(4, 9);
        } else if c.position.y > -env.screen.y * 0.15
            && self.rand.next_max(2) == 0
            && (self.num_clouds as f64) > 20.0
        {
            // `:136-139`: low cumulus; the coin is drawn BEFORE the count gate.
            c.kind = self.rand.next_range(14, 18);
        }
        // `:140-147`: the rare-cloud rolls (the anniversary extra roll only
        // when the first missed).
        let rare_odds = if env.dont_starve || env.tenth_anniversary {
            25
        } else {
            150
        };
        if self.rand.next_max(rare_odds) == 0 {
            c.kind = roll_rare_cloud(&mut self.rand, env);
        } else if env.tenth_anniversary && self.rand.next_max(3) == 0 {
            c.kind = roll_rare_cloud(&mut self.rand, env);
        }
        // `:148-159`.
        if (c.scale as f64) > 1.2 {
            c.position.y += 100.0;
        }
        if (c.scale as f64) > 1.3 {
            c.scale = 1.3;
        }
        if (c.scale as f64) < 0.7 {
            c.scale = 0.7;
        }
        // `:160-169`.
        c.active = true;
        c.position.x = x;
        if c.position.x > env.screen.x + 400.0 {
            c.alpha = 1.0;
        }
        let (tw, _) = tex_size(lookups, c.kind);
        if c.position.x + tw * c.scale < -400.0 {
            c.alpha = 1.0;
        }
        // `:170-177`: the spawn-time overlap rejection against the OTHER
        // active slots, using the old-type width/height caches. XNA
        // `Rectangle.Intersects` is strict overlap on both axes.
        let rect = (c.position.x as i32, c.position.y as i32, c.width, c.height);
        for j in 0..CLOUD_SLOTS {
            if num != j && self.slots[j].active {
                let o = &self.slots[j];
                let intersects = rect.0 < o.position.x as i32 + o.width
                    && o.position.x as i32 < rect.0 + rect.2
                    && rect.1 < o.position.y as i32 + o.height
                    && o.position.y as i32 < rect.1 + rect.3;
                if intersects {
                    self.slots[num].active = false;
                }
            }
        }
    }

    /// `Cloud.UpdateClouds` (`Cloud.cs:242-293`): the per-slot update + alive
    /// count, the two-neighbour bubble pass (note `j > 1`, not `j > 0` —
    /// verbatim), then ONE add or ONE kill-mark per tick to track `numClouds`.
    pub(crate) fn update_clouds(&mut self, env: &CloudEnv, lookups: &assets::Lookups) {
        let mut num = 0;
        for i in 0..CLOUD_SLOTS {
            if self.slots[i].active {
                self.update_one(i, env, lookups);
                if !self.slots[i].kill {
                    num += 1;
                }
            }
        }
        // `:260-277`: the scale-ordering bubble pass.
        for j in 0..CLOUD_SLOTS {
            if self.slots[j].active {
                if j > 1
                    && (!self.slots[j - 1].active
                        || (self.slots[j - 1].scale as f64)
                            > (self.slots[j].scale as f64) + 0.02)
                {
                    self.slots.swap(j - 1, j);
                }
                if j < CLOUD_SLOTS - 1
                    && (!self.slots[j].active
                        || (self.slots[j + 1].scale as f64)
                            < (self.slots[j].scale as f64) - 0.02)
                {
                    self.slots.swap(j + 1, j);
                }
            }
        }
        // `:278-292`.
        if num < self.num_clouds {
            self.add_cloud(env, lookups);
        } else if num > self.num_clouds {
            let mut num2 = self.rand.next_max(num) as usize;
            let mut num3 = 0;
            while self.slots[num2].kill && num3 < 100 {
                num3 += 1;
                num2 = self.rand.next_max(num) as usize;
            }
            self.slots[num2].kill = true;
        }
    }

    /// `Cloud.UpdateCloudParallax` (`Cloud.cs:295-316`): each active cloud
    /// shifts against the camera's X delta by its own `GetParallax` fraction.
    /// Runs per draw, right after the camera update (`Main.cs:61832-61833`).
    pub(crate) fn update_parallax(&mut self, center: Vec2) {
        let mut num = 0.0;
        if let Some(last) = self.last_camera_center {
            num = center.x - last.x;
        }
        self.last_camera_center = Some(center);
        if num == 0.0 {
            return;
        }
        for c in self.slots.iter_mut() {
            if c.active {
                c.position.x -= num * cloud_parallax(c.scale);
            }
        }
    }

    /// `Cloud.Update` (`Cloud.cs:340-445`) for one slot — verbatim order:
    /// the drunk-menu type override, the bucket-edge nudges, the wind drift,
    /// the ocean kill, the alpha ramps, the retire bounds, the rotation
    /// jitter draw, the clamps, the size caches, the cirrus storm kill.
    fn update_one(&mut self, i: usize, env: &CloudEnv, lookups: &assets::Lookups) {
        let c = &mut self.slots[i];
        // `:342-382`: the drunk-world menu override (inert in-world).
        if env.drunk_world_text && env.game_menu {
            if env.only_shimmer_ocean {
                c.kind = 29;
            } else if env.dont_starve_gen {
                c.kind = if (c.scale as f64) < 0.7 {
                    40
                } else if (c.scale as f64) < 0.8 {
                    39
                } else if (c.scale as f64) < 0.9 {
                    38
                } else {
                    37
                };
            } else if env.get_good_gen {
                c.kind = if (c.scale as f64) < 0.8 { 25 } else { 26 };
            } else {
                c.kind = 28;
            }
        }
        // `:383-390`: the float `== 1f` nudge fires; the DOUBLE `== 1.15`
        // compare never matches a float 1.15f (1.14999997… != 1.15) — dead in
        // the C#, kept verbatim.
        if c.scale == 1.0 {
            c.scale -= 0.0001;
        }
        if (c.scale as f64) == 1.15 {
            c.scale -= 0.0001;
        }
        // `:391`: the wind drift — `windSpeedCurrent * 9 * GetParallax * dayRate`.
        c.position.x +=
            env.wind_speed_current * 9.0 * cloud_parallax(c.scale) * env.day_rate as f32;
        // `:392-397`: the ocean front layer retires low clouds faster.
        let num = 600.0;
        if env.bg_alpha_front_4 == 1.0 && c.position.y > 200.0 {
            c.kill = true;
            c.alpha -= 0.005 * env.day_rate as f32;
        }
        // `:398-416`: the fade-in / fade-out ramps.
        if !c.kill {
            if c.alpha < 1.0 {
                c.alpha += 0.001 * env.day_rate as f32;
                if c.alpha > 1.0 {
                    c.alpha = 1.0;
                }
            }
        } else {
            c.alpha -= 0.001 * env.day_rate as f32;
            if c.alpha <= 0.0 {
                c.active = false;
            }
        }
        // `:417-420`: retire 600 px off either side.
        let (tw, th) = tex_size(lookups, c.kind);
        if c.position.x + tw * c.scale < 0.0 - num || c.position.x > env.screen.x + num {
            c.active = false;
        }
        // `:421-429`: one rotation-jitter draw per active cloud per tick.
        c.r_speed += self.rand.next_range(-10, 11) as f32 * 2E-05;
        if (c.r_speed as f64) > 0.0002 {
            c.r_speed = 0.0002;
        }
        if (c.r_speed as f64) < -0.0002 {
            c.r_speed = -0.0002;
        }
        // `:430-438`: clamp BEFORE integrating (verbatim order).
        if (c.rotation as f64) > 0.02 {
            c.rotation = 0.02;
        }
        if (c.rotation as f64) < -0.02 {
            c.rotation = -0.02;
        }
        c.rotation += c.r_speed;
        // `:439-440`.
        c.width = (tw * c.scale) as i32;
        c.height = (th * c.scale) as i32;
        // `:441-444`: the clear-weather wisps die when rain/overcast arrives.
        if (9..=13).contains(&c.kind) && (env.cloud_alpha > 0.0 || self.cloud_bg_active >= 1.0)
        {
            c.kill = true;
        }
    }

    /// `Main.updateCloudLayer` (`Main.cs:13174-13231`) — the `cloudBGActive`
    /// overcast state machine, on the bare `Main.rand` stream. `netMode == 1`
    /// (client) and the single-player main menu skip it; the port's session
    /// is netMode 0 in-world.
    pub(crate) fn update_cloud_layer(
        &mut self,
        rng: &mut UnifiedRandom,
        cloud_alpha: f32,
        day_rate: i32,
        raining: bool,
        storming: bool,
        game_menu: bool,
    ) {
        if game_menu {
            return;
        }
        let num = 86400;
        let num2 = num / 24;
        let num3 = (1.0 + 4.0 * cloud_alpha).max(1.0);
        if self.cloud_bg_active > 0.0 {
            if self.cloud_bg_active > 1.0 {
                self.cloud_bg_active -= day_rate as f32 / num3;
            }
            if self.cloud_bg_active < 1.0 {
                self.cloud_bg_active = 1.0;
            }
            if self.cloud_bg_active == 1.0
                && rng.next_max(
                    ((num2 * 2 / day_rate.max(1)) as f32 * num3) as i32,
                ) == 0
            {
                self.cloud_bg_active = -rng.next_range(num2 * 4, num * 4) as f32;
            }
            return;
        }
        if self.cloud_bg_active < 0.0 {
            self.cloud_bg_active += day_rate as f32 * num3;
            if raining {
                self.cloud_bg_active += (2 * day_rate) as f32 * num3;
            }
        }
        if self.cloud_bg_active > 0.0 {
            self.cloud_bg_active = 0.0;
        }
        if self.cloud_bg_active == 0.0
            && rng.next_max(
                ((num2 * 12 / if day_rate == 0 { 1 } else { day_rate }) as f32 / num3) as i32,
            ) == 0
        {
            self.cloud_bg_active = rng.next_range(num2 * 3, num * 2) as f32;
        } else if storming {
            self.cloud_bg_active = rng.next_range(num2, num2 * 4) as f32;
        }
    }

    /// `UpdateWeather`'s cloud-count tail (`Main.cs:58952-59036`) — runs once
    /// per dayRate iteration, on the bare `Main.rand` stream.
    /// `cloud_bg_alpha` is `Main.cloudBGAlpha` (the port's `Rain::cloud_bg`).
    pub(crate) fn update_num_clouds(
        &mut self,
        rng: &mut UnifiedRandom,
        cloud_alpha: f32,
        cloud_bg_alpha: f32,
    ) {
        if rng.next_max(60) == 0 {
            self.num_clouds_temp += rng.next_range(-1, 2);
        }
        if (rng.next_max(1000) as f32) < 50.0 * cloud_bg_alpha {
            self.num_clouds_temp += 1;
        }
        if (rng.next_max(1300) as f32) < 25.0 * (1.0 - cloud_bg_alpha) {
            self.num_clouds_temp -= 1;
        }
        if (rng.next_max(1000) as f32) < 200.0 * cloud_alpha && self.num_clouds_temp < 100 {
            self.num_clouds_temp += 1;
        }
        if (rng.next_max(1000) as f32) < 50.0 * cloud_alpha {
            self.num_clouds_temp += 1;
        }
        if self.num_clouds_temp > 66 && rng.next_max(100) == 0 {
            self.num_clouds_temp -= rng.next_range(1, 3);
        }
        if self.num_clouds_temp < 50 && rng.next_max(100) == 0 {
            self.num_clouds_temp += rng.next_range(1, 3);
        }
        if self.cloud_bg_active <= 0.0 && self.num_clouds_temp > 100 && cloud_alpha == 0.0 {
            self.num_clouds_temp = 100;
        }
        if self.num_clouds_temp < -20 {
            self.num_clouds_temp = -20;
        }
        if cloud_alpha > 0.0 && (self.num_clouds as f32) < 200.0 * cloud_alpha {
            while (self.num_clouds as f32) < 200.0 * cloud_alpha {
                self.num_clouds += rng.next_max(30);
                if self.num_clouds > 200 {
                    self.num_clouds = 200;
                }
                if self.num_clouds_temp < self.num_clouds {
                    self.num_clouds_temp = self.num_clouds;
                }
            }
        }
        self.weather_counter -= 1;
        if self.weather_counter > 0 {
            return;
        }
        if rng.next_max(2) == 0 {
            if rng.next_max(2) == 0 {
                self.num_clouds_temp += rng.next_max(250);
            } else {
                self.num_clouds_temp += rng.next_max(100);
            }
        }
        if self.num_clouds_temp < 0 {
            self.num_clouds_temp = 0;
        }
        if self.num_clouds_temp > 200 {
            self.num_clouds_temp = 200;
        }
        self.num_clouds = self.num_clouds_temp;
        self.weather_counter = rng.next_range(3600, 10800);
    }
}

/// `Cloud.RollRareCloud` (`Cloud.cs:180-225`): re-roll until the picked rare
/// type's progression gate passes. The 28/37-40 arms draw their own 1-in-10
/// (the dontStarve arm short-circuits the draw). Free fn so `add_cloud` can
/// call it while a slot borrow is live (disjoint `rand` field).
fn roll_rare_cloud(rand: &mut UnifiedRandom, env: &CloudEnv) -> i32 {
    loop {
        let num = if !env.tenth_anniversary {
            RARE_NORMAL[rand.next_max(RARE_NORMAL.len() as i32) as usize]
        } else {
            RARE_CELEBRATION[rand.next_max(RARE_CELEBRATION.len() as i32) as usize]
        };
        let ok = match num {
            31 => env.downed_boss3,
            36 => env.downed_boss2 && env.crimson,
            25 | 26 => env.downed_boss1,
            30 | 35 | 41 => env.hard_mode,
            28 => rand.next_max(10) == 0,
            37 | 38 | 39 | 40 => env.dont_starve || rand.next_max(10) == 0,
            _ => true,
        };
        if ok {
            return num;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookups() -> assets::Lookups {
        let mut l = assets::Lookups::default();
        l.clouds = (0..42)
            .map(|_| {
                Some(assets::CloudTex {
                    uv: [0.0; 4],
                    w: 100.0,
                    h: 50.0,
                })
            })
            .collect();
        l
    }

    fn env() -> CloudEnv {
        CloudEnv {
            screen: Vec2::new(1280.0, 720.0),
            wind_speed_current: 0.3,
            player_vx: 0.0,
            game_menu: false,
            day_rate: 1,
            cloud_alpha: 0.0,
            bg_alpha_front_4: 0.0,
            dont_starve: false,
            tenth_anniversary: false,
            downed_boss1: false,
            downed_boss2: false,
            downed_boss3: false,
            crimson: false,
            hard_mode: false,
            drunk_world_text: false,
            dont_starve_gen: false,
            get_good_gen: false,
            only_shimmer_ocean: false,
        }
    }

    fn field() -> CloudField {
        CloudField::new(42)
    }

    #[test]
    fn parallax_buckets_are_the_csharp_formula() {
        // `Cloud.cs:318-338`: 0.07*((s+1.15)/2)^2 / 0.19*(s-0.075)^2 /
        // 0.23*(s-0.225)^2.
        let p = cloud_parallax(0.8);
        assert!((p - 0.07 * (0.975f32 * 0.975)).abs() < 1e-7);
        let p = cloud_parallax(1.0);
        assert!((p - 0.19 * (0.925f32 * 0.925)).abs() < 1e-7);
        let p = cloud_parallax(1.3);
        assert!((p - 0.23 * (1.075f32 * 1.075)).abs() < 1e-7);
        // The DOUBLE `<= 1.15` edge: a float 1.15f promotes BELOW the double
        // literal, so it stays in the middle bucket.
        let p = cloud_parallax(1.15);
        assert!((p - 0.19 * (1.075f32 * 1.075)).abs() < 1e-7);
    }

    #[test]
    fn bucket_gates_keep_the_double_float_edge_quirk() {
        // `Main.cs:59655/59674/59697`: scale 1.15f passes BOTH the closer
        // (double < 1.15) and closest (float >= 1.15f) gates — drawn twice.
        assert!(cloud_in_bucket(1.15, 2));
        assert!(cloud_in_bucket(1.15, 3));
        assert!(!cloud_in_bucket(1.15, 1));
        assert!(cloud_in_bucket(0.9999, 1));
        assert!(!cloud_in_bucket(0.9999, 2));
        assert!(cloud_in_bucket(1.0, 2));
    }

    #[test]
    fn cloud_color_truncates_every_channel_through_the_byte_cast() {
        // `Cloud.cs:227-235`: (byte)(float)(int)(bg * min(scale*Alpha, 1)).
        let c = cloud_color(0.5, 1.0, [200, 100, 50, 255]);
        assert_eq!(c, [100, 50, 25, 127]);
        // scale*Alpha > 1 clamps at 1 (no wrap).
        let c = cloud_color(1.3, 1.0, [200, 100, 50, 255]);
        assert_eq!(c, [200, 100, 50, 255]);
        // The distant pull (`NextHorizonRenderer.cs:254-263`), byte-truncated.
        let mut c = [200u8, 100, 50, 255];
        original_colors_for_cloud(&mut c, 0.8, 1);
        assert_eq!(c[0], (200.0f32 * (0.8 * 0.8)) as i32 as u8);
        assert_eq!(c[1], (100.0f32 * ((0.8 + 1.0) / 2.0 * 0.9)) as i32 as u8);
        assert_eq!(c[2], 50);
        // Passes 2/3 leave the colour alone.
        let mut c2 = [200u8, 100, 50, 255];
        original_colors_for_cloud(&mut c2, 0.8, 2);
        assert_eq!(c2, [200, 100, 50, 255]);
    }

    #[test]
    fn add_cloud_replays_the_csharp_rng_order() {
        // State-pin witness: an independent stream replays `Cloud.cs:82-147`
        // draw-for-draw (clear sky, numClouds 200 — branch 2's count gate is
        // false) and must land on the same fields AND post-spawn stream state.
        let l = lookups();
        let e = env();
        let mut f = field();
        let mut r = f.rand.clone();
        let scale_raw = r.next_range(70, 131) as f32 * 0.01;
        let rotation = r.next_range(-10, 11) as f32 * 0.01;
        let flip = r.next_max(2) == 0;
        // wind 0.3 - vx*0.1 > 0 -> num3 = -200, num4 = 0.
        let x = r.next_range(-500, 1280 + 300) as f32;
        let mut y =
            r.next_range((-720.0f32 * 0.25) as i32, (720.0f32 * 0.15) as i32) as f32;
        if r.next_max(3) == 0 {
            y -= r.next_max((720.0f32 * 0.1) as i32) as f32;
        }
        let mut kind = r.next_max(4);
        // Branch 1 false without draws (cloudAlpha 0, cloudBGActive 0).
        // Branch 2: numClouds 200 > 80 -> false, no draws.
        if ((scale_raw as f64) < 1.15 && y < -720.0 * 0.3)
            || ((scale_raw as f64) < 0.85 && y < 720.0 * 0.15)
        {
            // numClouds 200 > 70 -> the cirro branch draws.
            kind = r.next_range(4, 9);
        } else if y > -720.0 * 0.15 && r.next_max(2) == 0 {
            // numClouds 200 > 20.
            kind = r.next_range(14, 18);
        }
        if r.next_max(150) == 0 {
            kind = roll_rare_cloud(&mut r, &e);
        }
        let mut scale = scale_raw;
        if (scale as f64) > 1.3 {
            scale = 1.3;
        }
        if (scale as f64) < 0.7 {
            scale = 0.7;
        }
        if (scale as f64) > 1.2 {
            y += 100.0;
        }
        f.add_cloud(&e, &l);
        let c = f.slots[0];
        assert!(c.active);
        assert_eq!(c.scale, scale);
        assert_eq!(c.rotation, rotation);
        assert_eq!(c.flip, flip);
        assert_eq!(c.position.x, x);
        assert_eq!(c.position.y, y);
        assert_eq!(c.kind, kind);
        assert_eq!(
            f.rand.peek(),
            r.peek(),
            "the spawn consumed the replayed draws"
        );
    }

    #[test]
    fn rain_spawn_takes_the_dark_type_branch() {
        // `Cloud.cs:116-127`: cloudAlpha > 0 with Next(4) != 0 -> type 18..21,
        // with the scale >= 1 / >= 1.15 lifts.
        let l = lookups();
        let mut e = env();
        e.cloud_alpha = 0.9;
        let mut dark = 0;
        let mut f = field();
        for _ in 0..60 {
            f.slots.iter_mut().for_each(|c| c.active = false);
            f.add_cloud(&e, &l);
            if let Some(c) = f.slots.iter().find(|c| c.active).copied() {
                if (18..22).contains(&c.kind) {
                    dark += 1;
                    if c.scale >= 1.15 {
                        // Spawn band floor -180-72, minus 300, plus the <=100
                        // lift for scale > 1.2.
                        assert!(c.position.y <= 108.0 - 300.0 + 100.0);
                    }
                }
            }
        }
        assert!(dark > 30, "3-in-4 rain spawns are dark clouds, got {dark}/60");
    }

    #[test]
    fn spawn_overlap_rejection_deactivates_the_new_cloud() {
        // `Cloud.cs:170-177`: a covering active neighbour kills the spawn.
        let l = lookups();
        let mut f = field();
        f.slots[0] = Cloud {
            active: true,
            position: Vec2::new(-50000.0, -50000.0),
            width: 100000,
            height: 100000,
            ..Default::default()
        };
        f.add_cloud(&env(), &l);
        assert!(!f.slots[1].active, "the overlapping spawn is rejected");
        assert!(f.slots[0].active, "the blocker survives");
    }

    #[test]
    fn reset_clouds_spawns_num_clouds_and_sets_all_alphas() {
        // `Cloud.cs:39-58` with 0x0 sheets (no overlap possible): exactly
        // numClouds active, every slot Alpha 1.
        let mut f = field();
        f.num_clouds = 5;
        f.reset_clouds(&env(), &assets::Lookups::default());
        assert_eq!(f.slots.iter().filter(|c| c.active).count(), 5);
        assert!(f.slots.iter().all(|c| c.alpha == 1.0));
    }

    #[test]
    fn update_drifts_by_wind_times_parallax_and_draws_one_jitter() {
        // `Cloud.cs:391` + `:421`: x += wind*9*GetParallax*dayRate; exactly
        // one Next(-10,11) per active cloud per tick. numClouds 0 keeps the
        // balance arm from spawning (one jitter draw only).
        let l = lookups();
        let e = env();
        let mut f = field();
        f.num_clouds = 1;
        f.slots[0] = Cloud {
            active: true,
            scale: 1.0,
            position: Vec2::new(100.0, 0.0),
            alpha: 1.0,
            ..Default::default()
        };
        let mut r = f.rand.clone();
        let jitter = r.next_range(-10, 11) as f32 * 2E-05;
        // scale == 1f nudges to 0.9999 BEFORE the drift (`:383-386`).
        let expected_scale = 1.0f32 - 0.0001;
        let expected_x = 100.0 + e.wind_speed_current * 9.0 * cloud_parallax(expected_scale);
        f.update_clouds(&e, &l);
        let c = f.slots.iter().find(|c| c.active).copied().unwrap();
        assert_eq!(c.scale, expected_scale);
        assert!((c.position.x - expected_x).abs() < 1e-4);
        assert_eq!(c.r_speed, jitter);
        assert_eq!(c.rotation, jitter);
        assert_eq!(f.rand.peek(), r.peek(), "one jitter draw per tick");
    }

    #[test]
    fn update_alpha_ramps_and_kill_fade() {
        // `Cloud.cs:398-416`: +0.001*dayRate capped at 1; kill -0.001*dayRate
        // deactivates at <= 0. numClouds 2 = the alive count: no balance draw.
        let l = lookups();
        let mut e = env();
        e.day_rate = 2;
        let mut f = field();
        f.num_clouds = 2;
        f.slots[0] = Cloud {
            active: true,
            scale: 0.9,
            alpha: 0.9995,
            ..Default::default()
        };
        f.slots[1] = Cloud {
            active: true,
            scale: 0.9,
            alpha: 0.001,
            kill: true,
            ..Default::default()
        };
        f.update_clouds(&e, &l);
        let live = f.slots.iter().find(|c| c.active && !c.kill).copied().unwrap();
        assert_eq!(live.alpha, 1.0, "the ramp caps at 1");
        assert!(
            !f.slots.iter().any(|c| c.active && c.kill),
            "the faded kill deactivated"
        );
    }

    #[test]
    fn update_retires_past_the_600px_bounds_and_ocean_kills_low_clouds() {
        // `Cloud.cs:417-420` + `:393-397`.
        let l = lookups();
        let mut e = env();
        e.bg_alpha_front_4 = 1.0;
        let mut f = field();
        f.num_clouds = 1;
        f.slots[0] = Cloud {
            active: true,
            scale: 0.9,
            position: Vec2::new(1280.0 + 601.0, 0.0),
            alpha: 1.0,
            ..Default::default()
        };
        f.slots[1] = Cloud {
            active: true,
            scale: 0.9,
            position: Vec2::new(100.0, 201.0),
            alpha: 1.0,
            ..Default::default()
        };
        f.update_clouds(&e, &l);
        assert!(
            !f.slots.iter().any(|c| c.active && c.position.x > 1880.0),
            "the off-screen cloud retired"
        );
        let low = f.slots.iter().find(|c| c.active).copied().unwrap();
        assert!(low.kill, "the ocean layer kills low clouds");
        assert!((low.alpha - (1.0 - 0.005 - 0.001)).abs() < 1e-6);
    }

    #[test]
    fn cirrus_wisps_die_when_rain_arrives() {
        // `Cloud.cs:441-444`: types 9..13 kill on cloudAlpha > 0 or overcast.
        let l = lookups();
        let mut e = env();
        e.cloud_alpha = 0.2;
        let mut f = field();
        f.num_clouds = 1;
        f.slots[0] = Cloud {
            active: true,
            scale: 0.9,
            kind: 11,
            alpha: 1.0,
            ..Default::default()
        };
        f.update_clouds(&e, &l);
        assert!(f.slots.iter().find(|c| c.active).unwrap().kill);
    }

    #[test]
    fn bubble_pass_skips_slot_zero_and_orders_by_scale() {
        // `Cloud.cs:264`: `j > 1` — slot 1 never pulls into slot 0; slot 2
        // swaps down when slot 1 is bigger by > 0.02.
        let l = lookups();
        let mut f = field();
        f.num_clouds = 2;
        f.slots[1] = Cloud {
            active: true,
            scale: 1.3,
            alpha: 1.0,
            position: Vec2::new(100.0, 0.0),
            ..Default::default()
        };
        f.slots[2] = Cloud {
            active: true,
            scale: 0.7,
            alpha: 1.0,
            position: Vec2::new(200.0, 0.0),
            ..Default::default()
        };
        f.update_clouds(&env(), &l);
        assert!(!f.slots[0].active, "the j > 1 quirk keeps slot 0 empty");
        let small_first = f
            .slots
            .iter()
            .filter(|c| c.active)
            .map(|c| c.scale)
            .collect::<Vec<_>>();
        assert_eq!(small_first, vec![0.7, 1.3], "small scales bubble forward");
    }

    #[test]
    fn kill_balance_marks_one_cloud_when_over_target() {
        // `Cloud.cs:282-292`: num > numClouds -> one Next(num) pick (re-rolled
        // while already killed), kill = true.
        let l = lookups();
        let mut f = field();
        f.num_clouds = 1;
        for i in 0..3 {
            f.slots[i] = Cloud {
                active: true,
                scale: 0.9,
                alpha: 1.0,
                position: Vec2::new(100.0 + i as f32 * 10.0, 0.0),
                ..Default::default()
            };
        }
        f.update_clouds(&env(), &l);
        assert_eq!(
            f.slots.iter().filter(|c| c.active && c.kill).count(),
            1,
            "exactly one kill mark per tick"
        );
    }

    #[test]
    fn camera_parallax_shifts_each_cloud_by_its_own_fraction() {
        // `Cloud.cs:295-316`: first call only latches; the delta call shifts
        // x -= dx * GetParallax(scale); inactive slots untouched.
        let mut f = field();
        f.slots[0] = Cloud {
            active: true,
            scale: 0.8,
            position: Vec2::new(500.0, 0.0),
            ..Default::default()
        };
        f.slots[1] = Cloud {
            active: true,
            scale: 1.3,
            position: Vec2::new(500.0, 0.0),
            ..Default::default()
        };
        f.slots[2] = Cloud {
            scale: 0.9,
            position: Vec2::new(500.0, 0.0),
            ..Default::default()
        };
        f.update_parallax(Vec2::new(1000.0, 0.0));
        assert_eq!(f.slots[0].position.x, 500.0, "the latch call does not move");
        f.update_parallax(Vec2::new(1100.0, 0.0));
        assert!((f.slots[0].position.x - (500.0 - 100.0 * cloud_parallax(0.8))).abs() < 1e-4);
        assert!((f.slots[1].position.x - (500.0 - 100.0 * cloud_parallax(1.3))).abs() < 1e-4);
        assert_eq!(f.slots[2].position.x, 500.0, "inactive slots never move");
    }

    #[test]
    fn clear_sky_clamps_the_count_walk_at_100() {
        // `Main.cs:58980-58982`: cloudBGActive <= 0 && temp > 100 &&
        // cloudAlpha == 0 -> temp = 100 (before the weatherCounter jump).
        let mut f = field();
        f.num_clouds_temp = 150;
        f.weather_counter = 5; // no jump this tick
        let mut rng = UnifiedRandom::new(7);
        f.update_num_clouds(&mut rng, 0.0, 0.0);
        assert_eq!(f.num_clouds_temp, 100);
        assert_eq!(f.weather_counter, 4);
    }

    #[test]
    fn rain_surge_lifts_num_clouds_to_the_200_alpha_band() {
        // `Main.cs:58988-59001`: while numClouds < 200*cloudAlpha, += Next(30)
        // capped at 200, temp follows.
        let mut f = field();
        f.num_clouds = 0;
        f.num_clouds_temp = 0;
        f.weather_counter = 5;
        let mut rng = UnifiedRandom::new(7);
        f.update_num_clouds(&mut rng, 1.0, 0.0);
        assert_eq!(f.num_clouds, 200);
        assert!(f.num_clouds_temp >= 200);
    }

    #[test]
    fn overcast_active_value_decays_to_one_and_holds() {
        // `Main.cs:13183-13201`: >1 decays by dayRate/num3, floors at 1; the
        // end-roll replays on an independent stream (state pin).
        let mut f = field();
        f.cloud_bg_active = 1.5;
        let mut rng = UnifiedRandom::new(9);
        let mut r = rng.clone();
        f.update_cloud_layer(&mut rng, 0.0, 1, false, false, false);
        let roll = r.next_max((3600 * 2) as f32 as i32);
        let expected = if roll == 0 {
            -(r.next_range(3600 * 4, 86400 * 4) as f32)
        } else {
            1.0
        };
        assert_eq!(f.cloud_bg_active, expected);
        assert_eq!(rng.peek(), r.peek());
    }

    #[test]
    fn overcast_cooldown_climbs_faster_while_raining() {
        // `Main.cs:13203-13214`: += dayRate*num3, +2*dayRate*num3 more while
        // raining, clamped at 0. num3 = 1 + 4*cloudAlpha.
        let mut f = field();
        f.cloud_bg_active = -100.0;
        let mut rng = UnifiedRandom::new(9);
        f.update_cloud_layer(&mut rng, 0.5, 2, true, false, false);
        assert_eq!(f.cloud_bg_active, -100.0 + 6.0 + 12.0);
    }

    #[test]
    fn same_seed_same_field() {
        let l = lookups();
        let e = env();
        let mut a = field();
        let mut b = field();
        a.num_clouds = 30;
        b.num_clouds = 30;
        a.reset_clouds(&e, &l);
        b.reset_clouds(&e, &l);
        for _ in 0..120 {
            a.update_clouds(&e, &l);
            b.update_clouds(&e, &l);
        }
        for (x, y) in a.slots.iter().zip(b.slots.iter()) {
            assert_eq!(x.active, y.active);
            assert_eq!(x.position, y.position);
            assert_eq!(x.kind, y.kind);
            assert_eq!(x.alpha, y.alpha);
        }
        assert_eq!(a.rand.peek(), b.rand.peek());
    }
}
