//! The dust layer: the `Dust` record and the 6000-slot pressure-gated pool of
//! `Main.dust`, with the verbatim `NewDust` spawn family
//! (`decompiled/Terraria/Dust.cs:10-407`).
//!
//! The C# keeps the pool as the static `Main.dust[6001]` array plus three
//! static pressure counters (`Dust.dCount`, `lavaBubbles`, `SandStormCount`,
//! `Dust.cs:12-16`); the port carries them on [`DustPool`], which the App owns
//! (Phase 2 of S1-E-1). Slot 6000 is the sentinel the C# returns when a spawn
//! is refused — it is never scanned, never drawn ([`DUST_SENTINEL`]).
//!
//! Everything player-visible here is a 1:1 transcription: the guard chain
//! (`:77-101`), the off-screen cull with the `400*(1-dCount)` margin
//! (`:89-97`), the first-inactive-slot scan and the five pressure bands rolled
//! on the caller's `UnifiedRandom` (`:103-152`), the default block with the
//! per-100 frame shift (`:153-193`), the `ChildSafety` re-type arm
//! (`:194-213`) and the per-type tweak chain (`:214-252`).
//!
//! Documented drops (each at its site below): the `Main.rand == null`
//! re-seed, the `Main.netMode == 2` server guard, the dead `Main.NoPooling`
//! realloc arm (`Main.cs:67208` sets it `false`), and the `shader` /
//! `customData` record fields (the port has no draw-shader chain; the
//! `customData` followers live in `UpdateDust`, task S1-E-3).
//!
//! The pool draws from the **caller-passed** [`UnifiedRandom`] — the C#'s
//! global `Main.rand` interleaving across call sites is the S1-A-11 residual,
//! and per-consumer streams stay untouched here (the audit seed's "no
//! determinism change").

use crate::unified_random::UnifiedRandom;
use crate::world::World;
use glam::Vec2;

/// The refuse sentinel: `NewDust` returns 6000 when no slot is taken
/// (`Dust.cs:79` and passim); `Main.dust[6000]` exists but is never scanned
/// (`Main.cs:66985`, scan `Dust.cs:103`).
pub const DUST_SENTINEL: usize = 6000;

/// XNA `Microsoft.Xna.Framework.Color` as sRGB bytes — the C# `Dust.color`
/// carrier. `default(Color)` and `Color.Transparent` are both all-zero
/// (`Dust.cs:204`); the draw side converts to linear `[f32; 4]` when this
/// record is rendered (E-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// `Color.Transparent` (MonoGame): `(0, 0, 0, 0)` — the ChildSafety
    /// re-type colour (`Dust.cs:204`).
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    /// `Color.White`.
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// One dust record — `Dust.cs:10-52` with the three statics moved to
/// [`DustPool`]. `shader` (`ArmorShaderData`, `Dust.cs:48`) and `customData`
/// (`Dust.cs:50`) are dropped: the port has no draw-time shader chain and the
/// `customData` follower arms live in `UpdateDust` (S1-E-3).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dust {
    /// The slot's own index, written once at pool init (`Main.cs:6686-6689`).
    /// `CloneDust`'s early return compares it against `maxDustToDraw`
    /// (`Dust.cs:265`).
    pub dust_index: usize,
    pub position: Vec2,
    pub velocity: Vec2,
    pub fade_in: f32,
    pub no_gravity: bool,
    pub scale: f32,
    pub rotation: f32,
    pub no_light: bool,
    pub no_light_emittance: bool,
    pub full_bright: bool,
    pub active: bool,
    /// C# `Dust.type` (`Dust.cs:40`).
    pub dust_type: i32,
    pub color: Color,
    pub alpha: i32,
    /// C# `Rectangle frame` (`Dust.cs:46`) — the 8×8 sheet cell
    /// (`frame.X = 10*Type`, per-100 shift, `Dust.cs:174-188`).
    pub frame_x: i32,
    pub frame_y: i32,
    pub frame_w: i32,
    pub frame_h: i32,
    pub first_frame: bool,
}

impl Dust {
    /// `Dust.HackFrame` (`Dust.cs:298-309`): recompute the sheet cell for a
    /// new type, drawing the row on the passed stream (`Main.rand.Next(3)`).
    pub fn hack_frame(&mut self, rng: &mut UnifiedRandom, dust_type: i32) {
        self.frame_x = 10 * dust_type;
        self.frame_y = 10 * rng.next_max(3);
        let mut num = dust_type;
        while num >= 100 {
            num -= 100;
            self.frame_x -= 1000;
            self.frame_y += 30;
        }
    }
}

/// `Terraria.GameContent.ChildSafety` — the blood gate. The C# `SafeDust`
/// table is `CreateBoolSet(true, 5, 227, 273, 237, 248, 249, 250, 251)`
/// (`ChildSafety.cs:32`); `SetFactory.CreateBoolSet` fills the default state
/// and flips the listed ids (`SetFactory.cs:97-109`), so `DangerousDust(id)`
/// is true exactly for the listed ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChildSafety {
    /// `ChildSafety.Disabled` — `true` by default (`ChildSafety.cs:33`), i.e.
    /// the gate is off until the App arms it (`Settings::blood` off → armed).
    pub disabled: bool,
}

impl Default for ChildSafety {
    fn default() -> Self {
        Self { disabled: true }
    }
}

/// The dust ids `DangerousDust` re-types to 16 (`ChildSafety.cs:32` — the
/// blood family).
const DANGEROUS_DUST: &[i32] = &[5, 227, 273, 237, 248, 249, 250, 251];

impl ChildSafety {
    /// `ChildSafety.DangerousDust` (`ChildSafety.cs:22-25`):
    /// `!SafeDust[id]`, true exactly for [`DANGEROUS_DUST`].
    pub fn dangerous_dust(id: i32) -> bool {
        DANGEROUS_DUST.contains(&id)
    }
}

/// The per-call inputs the C# reads as `Main` statics during `NewDust`
/// (`Dust.cs:77-97`). There is no `netMode` field: the port is single-player
/// and the `netMode == 2` server guard (`Dust.cs:93-96`) is a documented drop.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpawnCtx {
    /// `Main.gameMenu` (`Dust.cs:77`).
    pub game_menu: bool,
    /// `Main.gamePaused` (`Dust.cs:85`).
    pub game_paused: bool,
    /// `WorldGen.isGeneratingOrLoadingWorld` (`Dust.cs:89`).
    pub generating: bool,
    /// `Main.screenPosition` (`Dust.cs:98`).
    pub screen_position: Vec2,
    /// `Main.screenWidth` / `Main.screenHeight` (`Dust.cs:98`), as exact
    /// whole floats.
    pub screen_size: Vec2,
    /// `Main.maxDustToDraw` — the live per-frame value (`Dust.cs:114` passim).
    pub max_dust_to_draw: i32,
}

/// The wet probe seam for the type-34 arm: `Collision.WetCollision(pos, 4, 4)`
/// (`Dust.cs:248`). The port's probe lives in `tr-app` (`player.rs::
/// wet_collision`, the `Collision.cs:1572-1651` port), which tr-core cannot
/// depend on — the App injects it once at pool init. `None` = always dry (a
/// world with no liquid never reports wet), so type 34 then deactivates.
type WetProbe = Box<dyn Fn(&World, f32, f32, f32, f32) -> bool>;

/// The `Main.dust` pool: 6001 slots (0..6000 scanned, 6000 the sentinel) plus
/// the three C# statics (`Dust.cs:12-16`).
pub struct DustPool {
    slots: Vec<Dust>,
    /// `Dust.dCount` — pool pressure for the cull margin (`Dust.cs:97`); the
    /// low band resets it (`:151`), `UpdateDust` increments it (`:440`, E-3).
    pub d_count: f32,
    /// `Dust.lavaBubbles` (`Dust.cs:14`); its consumers are E-3.
    pub lava_bubbles: i32,
    /// `Dust.SandStormCount` (`Dust.cs:15`); its consumers are E-3.
    pub sand_storm_count: i32,
    /// The blood gate; `disabled` defaults `true` (`ChildSafety.cs:33`).
    pub child_safety: ChildSafety,
    wet_probe: Option<WetProbe>,
}

impl Default for DustPool {
    fn default() -> Self {
        Self::new()
    }
}

impl DustPool {
    /// `Main.dust = new Dust[6001]` with `dustIndex = j` per slot
    /// (`Main.cs:66985`, `:6686-6689`).
    pub fn new() -> Self {
        let mut slots = vec![Dust::default(); DUST_SENTINEL + 1];
        for (i, slot) in slots.iter_mut().enumerate() {
            slot.dust_index = i;
        }
        Self {
            slots,
            d_count: 0.0,
            lava_bubbles: 0,
            sand_storm_count: 0,
            child_safety: ChildSafety::default(),
            wet_probe: None,
        }
    }

    /// The whole slot array, sentinel included (`Main.dust`).
    pub fn slots(&self) -> &[Dust] {
        &self.slots
    }

    /// Mutable slot array (the per-type AI arms iterate it, Phase 2).
    pub fn slots_mut(&mut self) -> &mut [Dust] {
        &mut self.slots
    }

    /// `Main.dust[i]`.
    pub fn slot(&self, i: usize) -> &Dust {
        &self.slots[i]
    }

    /// `Main.dust[i]` mutable.
    pub fn slot_mut(&mut self, i: usize) -> &mut Dust {
        &mut self.slots[i]
    }

    /// Inject the type-34 wet probe (`Dust.cs:248`); see [`WetProbe`].
    pub fn set_wet_probe(&mut self, probe: WetProbe) {
        self.wet_probe = Some(probe);
    }

    /// `Dust.NewDust` (`Dust.cs:75-256`) — the only allocator, verbatim:
    /// guards → cull → scan + pressure bands → default block → ChildSafety →
    /// per-type tweaks. Returns the slot index, or [`DUST_SENTINEL`].
    #[allow(clippy::too_many_arguments)]
    pub fn new_dust(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        position: Vec2,
        width: i32,
        height: i32,
        dust_type: i32,
        speed_x: f32,
        speed_y: f32,
        alpha: i32,
        color: Color,
        scale: f32,
    ) -> usize {
        // Guards (`Dust.cs:77-96`). The `Main.rand == null` re-seed
        // (`:81-84`) has no port equivalent: streams are caller-owned and
        // never null.
        if ctx.game_menu {
            return DUST_SENTINEL;
        }
        if ctx.game_paused {
            return DUST_SENTINEL;
        }
        if ctx.generating {
            return DUST_SENTINEL;
        }
        // `Main.netMode == 2` (`:93-96`): server-only, dropped (single-player
        // port).
        // Off-screen cull (`:97-101`): margin `num = (int)(400f*(1f-dCount))`.
        let margin = (400.0f32 * (1.0f32 - self.d_count)) as i32;
        let rx = (ctx.screen_position.x - margin as f32) as i32;
        let ry = (ctx.screen_position.y - margin as f32) as i32;
        let rw = ctx.screen_size.x as i32 + margin * 2;
        let rh = ctx.screen_size.y as i32 + margin * 2;
        let sx = position.x as i32;
        let sy = position.y as i32;
        // XNA `Rectangle.Intersects`: strict `<` on both axes.
        if !(rx < sx + 10 && sx < rx + rw && ry < sy + 10 && sy < ry + rh) {
            return DUST_SENTINEL;
        }
        let armed = !self.child_safety.disabled;
        let mut result = DUST_SENTINEL;
        for i in 0..DUST_SENTINEL {
            if self.slots[i].active {
                continue;
            }
            // `Main.NoPooling` realloc arm (`:110-113`) is dead —
            // `Main.NoPooling = false` (`Main.cs:67208`) — dropped.
            // Pressure bands (`:114-152`), each on the caller's stream. A
            // refusal returns without touching `d_count`.
            let fi = i as f64;
            let max = ctx.max_dust_to_draw as f64;
            if fi > max * 0.9 {
                if rng.next_max(4) != 0 {
                    return DUST_SENTINEL;
                }
            } else if fi > max * 0.8 {
                if rng.next_max(3) != 0 {
                    return DUST_SENTINEL;
                }
            } else if fi > max * 0.7 {
                if rng.next_max(2) == 0 {
                    return DUST_SENTINEL;
                }
            } else if fi > max * 0.6 {
                if rng.next_max(4) == 0 {
                    return DUST_SENTINEL;
                }
            } else if fi > max * 0.5 {
                if rng.next_max(5) == 0 {
                    return DUST_SENTINEL;
                }
            } else {
                self.d_count = 0.0;
            }
            // Default block (`:153-193`).
            let mut num2 = width;
            let mut num3 = height;
            if num2 < 5 {
                num2 = 5;
            }
            if num3 < 5 {
                num3 = 5;
            }
            result = i;
            {
                let d = &mut self.slots[i];
                d.fade_in = 0.0;
                d.active = true;
                d.dust_type = dust_type;
                d.no_gravity = false;
                d.color = color;
                d.alpha = alpha;
                d.position.x = position.x + rng.next_max(num2 - 4) as f32 + 4.0;
                d.position.y = position.y + rng.next_max(num3 - 4) as f32 + 4.0;
                d.velocity.x = rng.next_range(-20, 21) as f32 * 0.1 + speed_x;
                d.velocity.y = rng.next_range(-20, 21) as f32 * 0.1 + speed_y;
                d.frame_x = 10 * dust_type;
                d.frame_y = 10 * rng.next_max(3);
                // `dust.shader = null; dust.customData = null;` (`:176-177`) —
                // dropped fields.
                d.no_light_emittance = false;
                d.full_bright = false;
                let mut num4 = dust_type;
                while num4 >= 100 {
                    num4 -= 100;
                    d.frame_x -= 1000;
                    d.frame_y += 30;
                }
                d.frame_w = 8;
                d.frame_h = 8;
                d.rotation = 0.0;
                d.scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
                d.scale *= scale;
                d.no_light = false;
                d.first_frame = true;
            }
            // ChildSafety arm (`:194-213`). A 50 % deactivate leaves the
            // written slot inactive and returns the sentinel — the C#'s own
            // shape.
            {
                let d = &mut self.slots[i];
                if armed && ChildSafety::dangerous_dust(d.dust_type) {
                    if rng.next_max(2) != 0 {
                        d.active = false;
                        return DUST_SENTINEL;
                    }
                    d.first_frame = false;
                    d.dust_type = 16;
                    d.scale = rng.next_float() * 1.6 + 0.3;
                    d.color = Color::TRANSPARENT;
                    d.frame_x = 10 * d.dust_type;
                    d.frame_y = 10 * rng.next_max(3);
                    let num5 = d.dust_type / 100;
                    d.frame_x -= 1000 * num5;
                    d.frame_y += 30 * num5;
                    d.no_gravity = true;
                }
            }
            // Per-type tweak chain (`:214-252`); the type-34 wet probe is
            // taken after the slot borrow (the probe reads the pool).
            let mut type_34_at: Option<(f32, f32)> = None;
            {
                let d = &mut self.slots[i];
                if matches!(
                    d.dust_type,
                    228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                        | 293..=298 | 302 | 307 | 310
                ) {
                    d.velocity.y = rng.next_range(-10, 6) as f32 * 0.1;
                    d.velocity.x *= 0.3;
                    d.scale *= 0.7;
                }
                if d.dust_type == 127 || d.dust_type == 187 {
                    d.velocity *= 0.3;
                    d.scale *= 0.7;
                }
                if d.dust_type == 308 {
                    d.velocity *= 0.5;
                    d.velocity.y += 1.0;
                }
                if matches!(d.dust_type, 33 | 52 | 266 | 98..=105) {
                    d.alpha = 170;
                    d.velocity *= 0.5;
                    d.velocity.y += 1.0;
                }
                if d.dust_type == 41 {
                    d.velocity *= 0.0;
                }
                if d.dust_type == 80 {
                    d.alpha = 50;
                }
                if d.dust_type == 34 || d.dust_type == 35 || d.dust_type == 152 {
                    d.velocity *= 0.1;
                    d.velocity.y = -0.5;
                    if d.dust_type == 34 {
                        type_34_at = Some((d.position.x, d.position.y - 8.0));
                    }
                }
            }
            if let Some((px, py)) = type_34_at {
                let wet = self
                    .wet_probe
                    .as_ref()
                    .is_some_and(|probe| probe(world, px, py, 4.0, 4.0));
                if !wet {
                    // `Dust.cs:250`: deactivate but still return the slot.
                    self.slots[i].active = false;
                }
            }
            break;
        }
        result
    }
}

impl DustPool {
    /// `Dust.NewDustPerfect` (`Dust.cs:54-63`): a `NewDust` with a 0×0 box,
    /// then the position (and velocity) are overwritten exactly. Note the C#
    /// writes into whatever slot `NewDust` returned — the sentinel included.
    pub fn new_dust_perfect(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        position: Vec2,
        dust_type: i32,
        velocity: Option<Vec2>,
        alpha: i32,
        color: Color,
        scale: f32,
    ) -> usize {
        let index = self.new_dust(
            rng, ctx, world, position, 0, 0, dust_type, 0.0, 0.0, alpha, color, scale,
        );
        let d = &mut self.slots[index];
        d.position = position;
        if let Some(vel) = velocity {
            d.velocity = vel;
        }
        index
    }

    /// `Dust.NewDustDirect` (`Dust.cs:65-73`): a `NewDust` whose NaN velocity
    /// is zeroed (`Vector2.HasNaNs`).
    #[allow(clippy::too_many_arguments)]
    pub fn new_dust_direct(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        position: Vec2,
        width: i32,
        height: i32,
        dust_type: i32,
        speed_x: f32,
        speed_y: f32,
        alpha: i32,
        color: Color,
        scale: f32,
    ) -> usize {
        let index = self.new_dust(
            rng, ctx, world, position, width, height, dust_type, speed_x, speed_y, alpha, color,
            scale,
        );
        let d = &mut self.slots[index];
        if d.velocity.x.is_nan() || d.velocity.y.is_nan() {
            d.velocity = Vec2::ZERO;
        }
        index
    }

    /// `Dust.CloneDust(Dust)` (`Dust.cs:263-286`): a fresh spawn of the
    /// source's type, overwritten with the copied field list. The
    /// `dustIndex == maxDustToDraw` early return (`:265-268`) is the C#'s own
    /// guard (it fires for the sentinel slot, whose index is 6000).
    pub fn clone_dust(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        dust_index: usize,
    ) -> usize {
        let rf = self.slots[dust_index];
        if rf.dust_index as i32 == ctx.max_dust_to_draw {
            return dust_index;
        }
        let num = self.new_dust(
            rng,
            ctx,
            world,
            rf.position,
            0,
            0,
            rf.dust_type,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let obj = &mut self.slots[num];
            obj.position = rf.position;
            obj.velocity = rf.velocity;
            obj.fade_in = rf.fade_in;
            obj.no_gravity = rf.no_gravity;
            obj.scale = rf.scale;
            obj.rotation = rf.rotation;
            obj.no_light = rf.no_light;
            obj.active = rf.active;
            obj.dust_type = rf.dust_type;
            obj.color = rf.color;
            obj.alpha = rf.alpha;
            obj.frame_x = rf.frame_x;
            obj.frame_y = rf.frame_y;
            obj.frame_w = rf.frame_w;
            obj.frame_h = rf.frame_h;
            // `obj.shader = rf.shader; obj.customData = rf.customData;`
            // (`:283-284`) — dropped fields. `noLightEmittance`, `fullBright`
            // and `firstFrame` are not in the C# copy list either.
        }
        num
    }

    /// `Dust.QuickDust(Vector2, Color)` (`Dust.cs:367-377`): a type-267 spawn
    /// overwritten into the debug-dust shape. Returns the slot for callers
    /// that tweak it (the C# returns the record reference).
    pub fn quick_dust(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        pos: Vec2,
        color: Color,
    ) -> usize {
        let index = self.new_dust(
            rng,
            ctx,
            world,
            pos,
            0,
            0,
            267,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let obj = &mut self.slots[index];
            obj.position = pos;
            obj.velocity = Vec2::ZERO;
            obj.fade_in = 1.0;
            obj.no_light = true;
            obj.no_gravity = true;
            obj.color = color;
        }
        index
    }

    /// `Dust.QuickDust(int, int, Color)` + `QuickDust(Point, Color)`
    /// (`Dust.cs:288-296`): tile coords → world coords via
    /// `Utils.ToWorldCoordinates` (`Utils.cs:1762-1765`, `p*16 + (8, 8)`).
    pub fn quick_dust_tile(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        tile_x: i32,
        tile_y: i32,
        color: Color,
    ) -> usize {
        let pos = Vec2::new(tile_x as f32 * 16.0 + 8.0, tile_y as f32 * 16.0 + 8.0);
        self.quick_dust(rng, ctx, world, pos, color)
    }

    /// `Dust.QuickDustSmall` (`Dust.cs:379-389`).
    pub fn quick_dust_small(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        pos: Vec2,
        color: Color,
        floor_position_values: bool,
    ) -> usize {
        let index = self.quick_dust(rng, ctx, world, pos, color);
        {
            let d = &mut self.slots[index];
            d.fade_in = 0.0;
            d.scale = 0.35;
            if floor_position_values {
                d.position = d.position.floor();
            }
        }
        index
    }

    /// `Dust.QuickDustLine` (`Dust.cs:391-400`): both endpoints plus the
    /// lerped walk, each scaled 0.3.
    pub fn quick_dust_line(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        start: Vec2,
        end: Vec2,
        splits: f32,
        color: Color,
    ) {
        let index = self.quick_dust(rng, ctx, world, start, color);
        self.slots[index].scale = 0.3;
        let index = self.quick_dust(rng, ctx, world, end, color);
        self.slots[index].scale = 0.3;
        let num = 1.0 / splits;
        let mut num2 = 0.0f32;
        while num2 < 1.0 {
            let pos = Vec2::new(
                start.x + (end.x - start.x) * num2,
                start.y + (end.y - start.y) * num2,
            );
            let index = self.quick_dust(rng, ctx, world, pos, color);
            self.slots[index].scale = 0.3;
            num2 += num;
        }
    }

    /// `Dust.QuickBox` (`Dust.cs:311-333`): the four box edges walked
    /// `divisions + 3` times, scale/fadein set per dust. The C#'s optional
    /// `Action<Dust>` manipulator is the trailing closure.
    #[allow(clippy::too_many_arguments)]
    pub fn quick_box(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        top_left: Vec2,
        bottom_right: Vec2,
        divisions: i32,
        color: Color,
        fadein: f32,
        dust_scale: f32,
        manipulator: Option<&mut dyn FnMut(&mut Dust)>,
    ) {
        let num = (divisions + 2) as f32;
        let mut manip = manipulator;
        let mut num2 = 0.0f32;
        while num2 <= num {
            let t = num2 / num;
            let x = top_left.x + (bottom_right.x - top_left.x) * t;
            let y = top_left.y + (bottom_right.y - top_left.y) * t;
            for pos in [
                Vec2::new(x, top_left.y),
                Vec2::new(x, bottom_right.y),
                Vec2::new(top_left.x, y),
                Vec2::new(bottom_right.x, y),
            ] {
                let index = self.quick_dust(rng, ctx, world, pos, color);
                let d = &mut self.slots[index];
                d.scale = dust_scale;
                d.fade_in = fadein;
                if let Some(manip) = &mut manip {
                    manip(d);
                }
            }
            num2 += 1.0;
        }
    }

    /// `Dust.QuickCircle` (`Dust.cs:335-343`): `(radius, 0)` walked a full
    /// turn in `divisions` steps through `Utils.RotatedBy`
    /// (`Utils.cs:1675-1685` — `(float)Math.Cos/Sin` of the double-widened
    /// angle), the required manipulator applied per dust.
    pub fn quick_circle(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        center: Vec2,
        radius: f32,
        divisions: i32,
        color: Color,
        manipulator: &mut dyn FnMut(&mut Dust),
    ) {
        let num = 1.0 / 1f32.max(divisions as f32);
        let mut num2 = 0.0f32;
        while num2 < 1.0 {
            let radians = num2 * (std::f32::consts::PI * 2.0);
            let cos = (radians as f64).cos() as f32;
            let sin = (radians as f64).sin() as f32;
            // `RotatedBy` of `(radius, 0)` about `Vector2.Zero`.
            let pos = Vec2::new(center.x + radius * cos, center.y + radius * sin);
            let index = self.quick_dust(rng, ctx, world, pos, color);
            manipulator(&mut self.slots[index]);
            num2 += num;
        }
    }

    /// `Dust.DrawDebugBox` (`Dust.cs:345-365`): the rectangle's perimeter
    /// cells (the C#'s edge test pins `Width - 1` / `Height - 1` while the
    /// loops run to `<=` — kept verbatim), scale 0.31, fadein 0, white when
    /// the caller passed `default(Color)`.
    pub fn draw_debug_box(
        &mut self,
        rng: &mut UnifiedRandom,
        ctx: &SpawnCtx,
        world: &World,
        rect: (i32, i32, i32, i32),
        color: Color,
    ) {
        let (rx, ry, rw, rh) = rect;
        let color = if color == Color::default() {
            Color::WHITE
        } else {
            color
        };
        let base = Vec2::new(rx as f32, ry as f32);
        for i in 0..=rw {
            for j in 0..=rh {
                if i == 0 || j == 0 || i == rw - 1 || j == rh - 1 {
                    let index = self.quick_dust(
                        rng,
                        ctx,
                        world,
                        base + Vec2::new(i as f32, j as f32),
                        color,
                    );
                    let d = &mut self.slots[index];
                    d.scale = 0.31;
                    d.fade_in = 0.0;
                }
            }
        }
    }
}

/// `Dust.dustWater()` (`Dust.cs:402-418`): the per-water-style splash dust id.
pub fn dust_water(water_style: i32) -> i32 {
    match water_style {
        2 => 98,
        3 => 99,
        4 => 100,
        5 => 101,
        6 => 102,
        7 => 103,
        8 => 104,
        9 => 105,
        10 => 123,
        12 => 288,
        _ => 33,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_random::UnifiedRandom;

    /// `GOLDEN_42[0..10]` — the C#-pinned seed-42 `Next()` stream
    /// (`unified_random.rs`); every expected draw below is derived from it by
    /// hand-tracing the C# (`Sample = next * 2^-31`, `Next(k) = (int)(Sample*k)`,
    /// `Next(min,max) = (int)(Sample*(max-min)) + min`, f32 literals rounded).
    const S42: [i32; 10] = [
        1434747710, 302596119, 269548474, 1122627734, 361709742, 563913476, 1555655117, 1101493307,
        372913049, 1634773126,
    ];
    /// `GOLDEN_1[0..10]`.
    const S1: [i32; 10] = [
        534011718, 237820880, 1002897798, 1657007234, 1412011072, 929393559, 760389092, 2026928803,
        217468053, 1379662799,
    ];

    fn new_pool() -> DustPool {
        DustPool::new()
    }

    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    /// A 1920x1080 screen at the origin with the given dust cap.
    fn ctx(max: i32) -> SpawnCtx {
        SpawnCtx {
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: max,
            ..SpawnCtx::default()
        }
    }

    /// Activate slots `0..n` so the scan's first inactive slot is `n`
    /// (the tests control the found index directly).
    fn fill_active(pool: &mut DustPool, n: usize) {
        for slot in &mut pool.slots_mut()[..n] {
            slot.active = true;
        }
    }

    fn spawn(
        pool: &mut DustPool,
        rng: &mut UnifiedRandom,
        world: &World,
        ctx: &SpawnCtx,
        dust_type: i32,
    ) -> usize {
        pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(100.0, 200.0),
            10,
            10,
            dust_type,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        )
    }

    /// DW-1.1 — the guard chain refuses before any RNG draw for
    /// menu / paused / generating (`Dust.cs:77-96`; the `netMode == 2` arm is
    /// a documented single-player drop).
    #[test]
    fn s1_e_1_the_guard_chain_returns_the_sentinel_without_a_draw() {
        let mut pool = new_pool();
        let world = world();
        for ctx in [
            SpawnCtx {
                game_menu: true,
                ..ctx(6000)
            },
            SpawnCtx {
                game_paused: true,
                ..ctx(6000)
            },
            SpawnCtx {
                generating: true,
                ..ctx(6000)
            },
        ] {
            let mut rng = UnifiedRandom::new(42);
            let r = spawn(&mut pool, &mut rng, &world, &ctx, 213);
            assert_eq!(r, DUST_SENTINEL);
            // The stream is untouched: the next raw draw is still the first.
            assert_eq!(rng.next(), S42[0], "a refused guard consumed a draw");
        }
    }

    /// DW-1.2 — the pressure bands refuse per the C# odds at each boundary
    /// (`Dust.cs:114-148`), with seeded accept/refuse vectors; the roll
    /// happens before the refuse branch (a refusal consumes exactly one
    /// draw).
    #[test]
    fn s1_e_1_the_pressure_bands_refuse_per_the_csharp_odds() {
        let world = world();
        // Slot 2001 vs max 4000: the 0.5 band, refuse iff Next(5) == 0.
        // Seed 42 draw 1 → Next(5) = 3 → accept (1 band + 6 default draws).
        let mut pool = new_pool();
        fill_active(&mut pool, 2001);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 2001);
        assert!(pool.slot(2001).active);
        assert_eq!(rng.next(), S42[7], "0.5-band accept consumed 7 draws");
        // Same band, seed 42 with one pre-burn: draw 2 → Next(5) = 0 → refuse.
        let mut pool = new_pool();
        fill_active(&mut pool, 2001);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(rng.next(), S42[0]);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert!(!pool.slot(2001).active);
        assert_eq!(rng.next(), S42[2], "0.5-band refuse consumed 1 draw");
        // Slot 2401: the 0.6 band, refuse iff Next(4) == 0.
        // Seed 42 → 2 → accept; seed 1 → 0 → refuse.
        let mut pool = new_pool();
        fill_active(&mut pool, 2401);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 2401);
        assert_eq!(rng.next(), S42[7]);
        let mut pool = new_pool();
        fill_active(&mut pool, 2401);
        let mut rng = UnifiedRandom::new(1);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S1[1], "0.6-band refuse consumed 1 draw");
        // Slot 2801: the 0.7 band, refuse iff Next(2) == 0.
        // Seed 42 → 1 → accept; seed 1 → 0 → refuse.
        let mut pool = new_pool();
        fill_active(&mut pool, 2801);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 2801);
        assert_eq!(rng.next(), S42[7]);
        let mut pool = new_pool();
        fill_active(&mut pool, 2801);
        let mut rng = UnifiedRandom::new(1);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S1[1], "0.7-band refuse consumed 1 draw");
        // Slot 3201: the 0.8 band, refuse unless Next(3) != 0.
        // Seed 42 → 2 → refuse; seed 1 → 0 → accept.
        let mut pool = new_pool();
        fill_active(&mut pool, 3201);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[1], "0.8-band refuse consumed 1 draw");
        let mut pool = new_pool();
        fill_active(&mut pool, 3201);
        let mut rng = UnifiedRandom::new(1);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 3201);
        assert_eq!(rng.next(), S1[7], "0.8-band accept consumed 7 draws");
        // Slot 3601: the 0.9 band, refuse unless Next(4) != 0.
        // Seed 42 → 2 → refuse; seed 1 → 0 → accept.
        let mut pool = new_pool();
        fill_active(&mut pool, 3601);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[1], "0.9-band refuse consumed 1 draw");
        let mut pool = new_pool();
        fill_active(&mut pool, 3601);
        let mut rng = UnifiedRandom::new(1);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 3601);
        assert_eq!(rng.next(), S1[7], "0.9-band accept consumed 7 draws");
        // The f64 boundary artifact: `4000 * 0.9` is exactly 3600.0 in f64,
        // so slot 3600 falls into the 0.8 band (Next(3)), not the 0.9 band.
        // Seed 42 with draws 1..5 pre-burned: draw 6 → Next(3) = 0 → accept
        // (a 0.9-band Next(4) would be 1 → refuse).
        let mut pool = new_pool();
        fill_active(&mut pool, 3600);
        let mut rng = UnifiedRandom::new(42);
        for expected in S42.iter().take(5) {
            assert_eq!(rng.next(), *expected);
        }
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 3600);
        assert!(pool.slot(3600).active);
    }

    /// DW-1.2 — a band refusal leaves `d_count` untouched; a low-band spawn
    /// resets it (`Dust.cs:151`).
    #[test]
    fn s1_e_1_band_refusals_leave_d_count_and_the_low_band_resets_it() {
        let world = world();
        let mut pool = new_pool();
        pool.d_count = 0.7;
        fill_active(&mut pool, 3601);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(4000), 213),
            DUST_SENTINEL
        );
        assert_eq!(pool.d_count, 0.7, "a band refusal touched d_count");
        // An accept inside a band leaves it alone too.
        let mut pool = new_pool();
        pool.d_count = 0.7;
        fill_active(&mut pool, 2401);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(4000), 213), 2401);
        assert_eq!(pool.d_count, 0.7);
        // The low band (`i <= 0.5*max`) resets it.
        let mut pool = new_pool();
        pool.d_count = 0.7;
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(6000), 213), 0);
        assert_eq!(pool.d_count, 0.0);
    }

    /// DW-1.3 — the cull margin is `400*(1-dCount)` (`Dust.cs:97-101`) and
    /// off-screen spawns refuse with no RNG draw.
    #[test]
    fn s1_e_1_the_cull_margin_scales_with_d_count_and_refuses_off_screen() {
        let world = world();
        let at = |x: f32| Vec2::new(x, 100.0);
        // d_count 0 → margin 400: the cull box covers x < 2320.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            pool.new_dust(
                &mut rng,
                &ctx(6000),
                &world,
                at(2330.7),
                10,
                10,
                213,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0
            ),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[0], "a culled spawn consumed a draw");
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(6000), 213),
            0,
            "on-screen spawns pass the cull"
        );
        // d_count 0.5 → margin 200: the box covers x < 2120.
        let mut pool = new_pool();
        pool.d_count = 0.5;
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            pool.new_dust(
                &mut rng,
                &ctx(6000),
                &world,
                at(2200.0),
                10,
                10,
                213,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0
            ),
            DUST_SENTINEL
        );
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(6000), 213),
            0,
            "x = 2100 still inside the 200-margin box"
        );
        // d_count 2.0 → margin -400: the box shrinks to (400, 400, 1120, 280).
        let mut pool = new_pool();
        pool.d_count = 2.0;
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            pool.new_dust(
                &mut rng,
                &ctx(6000),
                &world,
                at(100.0),
                10,
                10,
                213,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0
            ),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[0]);
    }

    /// DW-1.4 — the default block (`Dust.cs:153-193`) against a hand-computed
    /// C# trace (seed 42, slot 0, W=H=10, type 213, speed (0.5, -1.0), alpha
    /// 7, scale 1.25): jitter 4/0 → position (108, 204); velocity jitter
    /// -15/+1 → (-1.0, 0.1f-1.0); frame (130, 60) via the per-100 shift;
    /// scale (1 - 10*0.01f)*1.25 = 1.125. Floats compared by bit pattern.
    #[test]
    fn s1_e_1_the_default_block_matches_a_hand_computed_csharp_trace() {
        let mut pool = new_pool();
        let world = world();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.new_dust(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(100.0, 200.0),
            10,
            10,
            213,
            0.5,
            -1.0,
            7,
            Color::new(10, 20, 30, 255),
            1.25,
        );
        assert_eq!(r, 0);
        let d = pool.slot(0);
        assert_eq!(d.position, Vec2::new(108.0, 204.0));
        assert_eq!(d.velocity.x.to_bits(), (-1.0f32).to_bits());
        assert_eq!(
            d.velocity.y.to_bits(),
            (-0.899_999_976_158_142_1f32).to_bits()
        );
        assert_eq!(
            (d.frame_x, d.frame_y, d.frame_w, d.frame_h),
            (130, 60, 8, 8)
        );
        assert_eq!(d.scale.to_bits(), 1.125f32.to_bits());
        assert_eq!(d.dust_type, 213);
        assert_eq!(d.alpha, 7);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert_eq!(d.rotation, 0.0);
        assert_eq!(d.fade_in, 0.0);
        assert!(d.active && d.first_frame);
        assert!(!d.no_gravity && !d.no_light && !d.no_light_emittance && !d.full_bright);
        // The width/height floor: W=H=2 behaves as 5 (Next(1) jitter).
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.new_dust(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(100.0, 200.0),
            2,
            2,
            213,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        assert_eq!(pool.slot(0).position, Vec2::new(104.0, 204.0));
        // 6 draws consumed (slot 0 is the low band: no band roll).
        assert_eq!(rng.next(), S42[6]);
    }

    /// DW-1.5 — the ChildSafety arm (`Dust.cs:194-213`): blood gate armed →
    /// dangerous types draw Next(2); 0 re-types to 16 (scale, transparent
    /// colour, frame, noGravity, firstFrame), non-0 deactivates the written
    /// slot and returns the sentinel; safe types and the disabled gate are
    /// untouched.
    #[test]
    fn s1_e_1_child_safety_retypes_dangerous_dust_and_can_deactivate() {
        let world = world();
        // Armed, type 5 (dangerous), seed 1: the 7th draw → Next(2) = 0 →
        // re-type. Trace: 6 default draws then Next(2), NextFloat, Next(3).
        let mut pool = new_pool();
        pool.child_safety.disabled = false;
        let mut rng = UnifiedRandom::new(1);
        let r = pool.new_dust(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::ZERO,
            10,
            10,
            5,
            0.0,
            0.0,
            0,
            Color::new(200, 0, 0, 255),
            1.0,
        );
        assert_eq!(r, 0);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 16);
        assert_eq!(d.scale.to_bits(), 1.810_179_710_388_183_6f32.to_bits());
        assert_eq!(d.color, Color::TRANSPARENT);
        assert_eq!((d.frame_x, d.frame_y, d.frame_w, d.frame_h), (160, 0, 8, 8));
        assert!(d.no_gravity && d.active && !d.first_frame);
        assert_eq!(
            d.position,
            Vec2::new(5.0, 4.0),
            "the re-type keeps position"
        );
        assert_eq!(rng.next(), S1[9], "re-type consumed 9 draws");
        // Armed, seed 42: the 7th draw → Next(2) = 1 → deactivate; the slot
        // was already written and is left inactive — the C#'s own shape.
        let mut pool = new_pool();
        pool.child_safety.disabled = false;
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            pool.new_dust(
                &mut rng,
                &ctx(6000),
                &world,
                Vec2::ZERO,
                10,
                10,
                5,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0
            ),
            DUST_SENTINEL
        );
        let d = pool.slot(0);
        assert!(!d.active);
        assert_eq!(d.dust_type, 5, "the deactivate arm leaves the write");
        assert_eq!(rng.next(), S42[7], "deactivate consumed 7 draws");
        // Armed but a safe type (16 is not in the dangerous set): no arm draws.
        let mut pool = new_pool();
        pool.child_safety.disabled = false;
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 16);
        assert_eq!(pool.slot(0).dust_type, 16);
        assert_eq!(rng.next(), S42[6], "safe type consumed 6 draws");
        // Disabled (blood on, the default): dangerous type untouched.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 5);
        assert_eq!(pool.slot(0).dust_type, 5);
        assert!(pool.slot(0).active);
        assert_eq!(rng.next(), S42[6], "disabled gate consumed 6 draws");
    }

    /// DW-1.6 (type 34) — the wet probe seam: type 34 dies unless the
    /// injected probe (`Collision.WetCollision`, `Dust.cs:248`) reports water
    /// at `(pos.x, pos.y - 8, 4, 4)`; the slot stays written and the spawn
    /// still returns its index. No probe = always dry.
    #[test]
    fn s1_e_1_type_34_dies_unless_the_wet_probe_reports_water() {
        let world = world();
        // Dry probe → inactive.
        let mut pool = new_pool();
        pool.set_wet_probe(Box::new(|_, _, _, _, _| false));
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(6000), 34), 0);
        assert!(!pool.slot(0).active);
        // Wet probe → active; the probe sees the jittered position minus
        // (0, 8) and the fixed 4x4 box. Seed 42 position trace: (108, 204).
        let mut pool = new_pool();
        let probed = std::rc::Rc::new(core::cell::Cell::new((0.0f32, 0.0f32, 0.0f32, 0.0f32)));
        let probe_cell = probed.clone();
        pool.set_wet_probe(Box::new(move |_, x, y, w, h| {
            probe_cell.set((x, y, w, h));
            true
        }));
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(6000), 34), 0);
        assert!(pool.slot(0).active);
        assert_eq!(probed.get(), (108.0, 196.0, 4.0, 4.0));
        // Types 35/152 take the velocity shape without any probe.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 35);
        assert!(pool.slot(0).active);
        assert_eq!(pool.slot(0).velocity.y, -0.5);
        // No probe injected = always dry.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 34);
        assert!(!pool.slot(0).active);
    }

    /// DW-1.6 — each per-type tweak family (`Dust.cs:214-252`) pinned against
    /// the seed-42 trace: pre-tweak velocity (-1.5, 0.1f), pre-tweak scale
    /// 0.8999999761581421 (speed 0, scale param 1.0).
    #[test]
    fn s1_e_1_per_type_tweak_families_match_the_csharp_deltas() {
        let world = world();
        // Velocity-damp family (type 6): velY = Next(-10,6)*0.1 (one extra
        // draw — seed 42 draw 7 → 1*0.1f, coincidentally the pre-tweak value),
        // velX *= 0.3, scale *= 0.7. The 7-draw count and the velX/scale
        // products pin the family; the pre-tweak velocity was (-1.5, 0.1f).
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 6);
        let d = pool.slot(0);
        assert_eq!(
            d.velocity.y.to_bits(),
            0.100_000_001_490_116_12f32.to_bits()
        );
        assert_eq!(
            d.velocity.x.to_bits(),
            (-0.450_000_017_881_393_43f32).to_bits()
        );
        assert_eq!(d.scale.to_bits(), 0.629_999_995_231_628_4f32.to_bits());
        assert_eq!(rng.next(), S42[7], "the damp family consumed 7 draws");
        // {127, 187}: velocity *= 0.3 on both axes, scale *= 0.7, no draw.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 127);
        let d = pool.slot(0);
        assert_eq!(
            d.velocity.x.to_bits(),
            (-0.450_000_017_881_393_43f32).to_bits()
        );
        assert_eq!(
            d.velocity.y.to_bits(),
            0.030_000_001_192_092_896f32.to_bits()
        );
        assert_eq!(d.scale.to_bits(), 0.629_999_995_231_628_4f32.to_bits());
        assert_eq!(rng.next(), S42[6]);
        // 308: velocity *= 0.5, velY += 1.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 308);
        let d = pool.slot(0);
        assert_eq!(d.velocity.x, -0.75);
        assert_eq!(d.velocity.y.to_bits(), 1.049_999_952_316_284_2f32.to_bits());
        // Alpha-170 family (type 33): alpha = 170, velocity *= 0.5, velY += 1.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 33);
        let d = pool.slot(0);
        assert_eq!(d.alpha, 170);
        assert_eq!(d.velocity.x, -0.75);
        assert_eq!(d.velocity.y.to_bits(), 1.049_999_952_316_284_2f32.to_bits());
        // 41: velocity zeroed.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 41);
        assert_eq!(pool.slot(0).velocity, Vec2::ZERO);
        // 80: alpha = 50, velocity kept.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 80);
        assert_eq!(pool.slot(0).alpha, 50);
        assert_eq!(pool.slot(0).velocity.x, -1.5);
        // {35}: velocity *= 0.1, velY forced to -0.5.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        spawn(&mut pool, &mut rng, &world, &ctx(6000), 35);
        let d = pool.slot(0);
        assert_eq!(
            d.velocity.x.to_bits(),
            (-0.150_000_005_960_464_48f32).to_bits()
        );
        assert_eq!(d.velocity.y, -0.5);
        assert!(d.active);
    }

    /// DW-1.7 — a full pool refuses with zero RNG draws; the first inactive
    /// slot wins the scan (`Dust.cs:103-109`).
    #[test]
    fn s1_e_1_pool_full_refuses_without_a_draw_and_the_first_inactive_slot_wins() {
        let world = world();
        let mut pool = new_pool();
        fill_active(&mut pool, DUST_SENTINEL);
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            spawn(&mut pool, &mut rng, &world, &ctx(6000), 213),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[0], "a pool-full scan consumed a draw");
        // Free slot 1234: the scan takes it.
        let mut pool = new_pool();
        fill_active(&mut pool, 3000);
        pool.slot_mut(1234).active = false;
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(spawn(&mut pool, &mut rng, &world, &ctx(6000), 213), 1234);
        assert!(pool.slot(1234).active);
    }

    /// DW-1.7 — `NewDustPerfect` (`Dust.cs:54-63`) overwrites position and
    /// velocity exactly (jitter replaced), including into the sentinel slot
    /// on a refused spawn — the C#'s own shape.
    #[test]
    fn s1_e_1_new_dust_perfect_overwrites_position_and_velocity() {
        let world = world();
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.new_dust_perfect(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(7.25, -3.5),
            6,
            Some(Vec2::new(9.0, -9.0)),
            0,
            Color::default(),
            1.0,
        );
        assert_eq!(r, 0);
        let d = pool.slot(0);
        assert_eq!(d.position, Vec2::new(7.25, -3.5));
        assert_eq!(d.velocity, Vec2::new(9.0, -9.0));
        // The spawn's own work stays: type 6's damp family scaled it.
        assert_eq!(d.scale.to_bits(), 0.629_999_995_231_628_4f32.to_bits());
        assert_eq!(d.dust_type, 6);
        assert_eq!(rng.next(), S42[7], "7 draws: 6 default + the damp roll");
        // Refused spawn: the sentinel record still receives the overwrite.
        let mut pool = new_pool();
        fill_active(&mut pool, DUST_SENTINEL);
        let mut rng = UnifiedRandom::new(42);
        let r = pool.new_dust_perfect(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(7.25, -3.5),
            6,
            Some(Vec2::new(9.0, -9.0)),
            0,
            Color::default(),
            1.0,
        );
        assert_eq!(r, DUST_SENTINEL);
        assert_eq!(pool.slot(DUST_SENTINEL).position, Vec2::new(7.25, -3.5));
        assert_eq!(pool.slot(DUST_SENTINEL).velocity, Vec2::new(9.0, -9.0));
        assert!(!pool.slot(DUST_SENTINEL).active);
        assert_eq!(rng.next(), S42[0], "no draws on a refused spawn");
    }

    /// DW-1.7 — `NewDustDirect` (`Dust.cs:65-73`) zeroes a NaN velocity.
    #[test]
    fn s1_e_1_new_dust_direct_fixes_nan_velocity() {
        let world = world();
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.new_dust_direct(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(100.0, 200.0),
            10,
            10,
            213,
            f32::NAN,
            2.0,
            0,
            Color::default(),
            1.0,
        );
        assert_eq!(r, 0);
        let d = pool.slot(0);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert_eq!(d.velocity.y.to_bits(), 0.0f32.to_bits());
        // A clean spawn keeps its jittered velocity.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.new_dust_direct(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(100.0, 200.0),
            10,
            10,
            213,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        assert_eq!(pool.slot(0).velocity, Vec2::new(-1.5, 0.1));
    }

    /// DW-1.7 — `CloneDust` (`Dust.cs:263-286`): the early return for
    /// `dustIndex == maxDustToDraw` (the sentinel), then the exact copy list.
    #[test]
    fn s1_e_1_clone_dust_copies_the_source_record() {
        let world = world();
        let mut pool = new_pool();
        {
            let rf = pool.slot_mut(7);
            rf.active = true;
            rf.position = Vec2::new(1.0, 2.0);
            rf.velocity = Vec2::new(3.0, 4.0);
            rf.fade_in = 5.5;
            rf.no_gravity = true;
            rf.scale = 0.5;
            rf.rotation = 0.25;
            rf.no_light = true;
            rf.no_light_emittance = true;
            rf.full_bright = true;
            rf.first_frame = true;
            rf.dust_type = 6;
            rf.color = Color::new(9, 8, 7, 6);
            rf.alpha = 3;
            (rf.frame_x, rf.frame_y, rf.frame_w, rf.frame_h) = (13, 17, 8, 8);
        }
        let mut rng = UnifiedRandom::new(42);
        let num = pool.clone_dust(&mut rng, &ctx(6000), &world, 7);
        assert_eq!(num, 0, "the scan's first inactive slot takes the clone");
        let obj = pool.slot(0);
        assert_eq!(obj.position, Vec2::new(1.0, 2.0));
        assert_eq!(obj.velocity, Vec2::new(3.0, 4.0));
        assert_eq!(obj.fade_in, 5.5);
        assert!(obj.no_gravity && obj.no_light && obj.active);
        assert_eq!(obj.scale, 0.5);
        assert_eq!(obj.rotation, 0.25);
        assert_eq!(obj.dust_type, 6);
        assert_eq!(obj.color, Color::new(9, 8, 7, 6));
        assert_eq!(obj.alpha, 3);
        assert_eq!(
            (obj.frame_x, obj.frame_y, obj.frame_w, obj.frame_h),
            (13, 17, 8, 8)
        );
        // Not in the C# copy list: these keep the fresh spawn's values.
        assert!(!obj.no_light_emittance && !obj.full_bright && obj.first_frame);
        assert_eq!(obj.dust_index, 0);
        // The sentinel early return: slot 6000's index equals the cap.
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(
            pool.clone_dust(&mut rng, &ctx(6000), &world, DUST_SENTINEL),
            DUST_SENTINEL
        );
        assert_eq!(rng.next(), S42[0], "the guarded clone consumed no draws");
    }

    /// The QuickDust family (`Dust.cs:288-400`): the overwrite shape, the
    /// tile-coord lift, the small/floor variants, the line walk, the box and
    /// the circle with their manipulators.
    #[test]
    fn s1_e_1_quick_dust_family_pins_the_overwrite_shape() {
        let world = world();
        // QuickDust: type 267, exact position, zero velocity, the debug shape.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.quick_dust(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(10.5, 20.5),
            Color::new(1, 2, 3, 4),
        );
        assert_eq!(r, 0);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 267);
        assert_eq!(d.position, Vec2::new(10.5, 20.5));
        assert_eq!(d.velocity, Vec2::ZERO);
        assert_eq!(d.fade_in, 1.0);
        assert!(d.no_light && d.no_gravity);
        assert_eq!(d.color, Color::new(1, 2, 3, 4));
        assert_eq!(
            (d.frame_x, d.frame_y, d.frame_w, d.frame_h),
            (670, 60, 8, 8)
        );
        // QuickDust(Point): tile coords lift to world coords (`p*16 + 8`).
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.quick_dust_tile(&mut rng, &ctx(6000), &world, 3, 4, Color::WHITE);
        assert_eq!(pool.slot(r).position, Vec2::new(56.0, 72.0));
        // QuickDustSmall: fadein 0, scale 0.35, optional floor.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        let r = pool.quick_dust_small(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(10.7, 20.9),
            Color::WHITE,
            true,
        );
        let d = pool.slot(r);
        assert_eq!(d.position, Vec2::new(10.0, 20.0));
        assert_eq!(d.fade_in, 0.0);
        assert_eq!(d.scale, 0.35);
        // QuickDustLine: both endpoints then the lerped walk, scale 0.3.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.quick_dust_line(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 50.0),
            2.0,
            Color::WHITE,
        );
        let at = [0, 1, 2, 3].map(|i| pool.slot(i).position);
        assert_eq!(
            at,
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 50.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 25.0)
            ]
        );
        assert!([0, 1, 2, 3].iter().all(|&i| pool.slot(i).scale == 0.3));
        // QuickBox: (divisions+3) rings of four, scale/fadein set, the
        // manipulator runs per dust.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.quick_box(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            1,
            Color::WHITE,
            2.0,
            3.0,
            Some(&mut |d: &mut Dust| d.rotation = 0.5),
        );
        assert!(pool.slot(15).active, "16 box spawns");
        assert_eq!(pool.slot(4).position.x.to_bits(), 3.333_333_5f32.to_bits());
        assert_eq!(pool.slot(4).position.y, 0.0);
        assert_eq!(pool.slot(4).scale, 3.0);
        assert_eq!(pool.slot(4).fade_in, 2.0);
        assert_eq!(pool.slot(4).rotation, 0.5);
        assert_eq!(pool.slot(12).position, Vec2::new(10.0, 0.0));
        // QuickCircle: `divisions` steps around the turn, manipulator applied.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.quick_circle(
            &mut rng,
            &ctx(6000),
            &world,
            Vec2::new(50.0, 50.0),
            10.0,
            4,
            Color::WHITE,
            &mut |d: &mut Dust| d.fade_in = 9.0,
        );
        let at = [0, 1, 2, 3].map(|i| pool.slot(i).position);
        assert_eq!(
            at,
            [
                Vec2::new(60.0, 50.0),
                Vec2::new(50.0, 60.0),
                Vec2::new(40.0, 50.0),
                Vec2::new(50.0, 40.0)
            ]
        );
        assert!([0, 1, 2, 3].iter().all(|&i| pool.slot(i).fade_in == 9.0));
    }

    /// `dustWater` (`Dust.cs:402-418`): every arm plus the default.
    #[test]
    fn s1_e_1_dust_water_matches_the_csharp_switch() {
        assert_eq!(dust_water(2), 98);
        assert_eq!(dust_water(3), 99);
        assert_eq!(dust_water(4), 100);
        assert_eq!(dust_water(5), 101);
        assert_eq!(dust_water(6), 102);
        assert_eq!(dust_water(7), 103);
        assert_eq!(dust_water(8), 104);
        assert_eq!(dust_water(9), 105);
        assert_eq!(dust_water(10), 123);
        assert_eq!(dust_water(12), 288);
        assert_eq!(dust_water(1), 33);
        assert_eq!(dust_water(11), 33);
        assert_eq!(dust_water(0), 33);
    }

    /// `HackFrame` (`Dust.cs:298-309`) recomputes the sheet cell; the debug
    /// box (`Dust.cs:345-365`) walks its perimeter with the C#'s own edge
    /// test, white on a default colour, scale 0.31, fadein 0.
    #[test]
    fn s1_e_1_hack_frame_and_debug_box_recompute_frames() {
        let mut rng = UnifiedRandom::new(42);
        let mut d = Dust::default();
        d.hack_frame(&mut rng, 213);
        // Draw 1 row = 2, then two per-100 shifts: 20 + 60 = 80.
        assert_eq!((d.frame_x, d.frame_y), (130, 80));
        d.hack_frame(&mut rng, 5);
        assert_eq!((d.frame_x, d.frame_y), (50, 0), "seed 42 draw 2 row = 0");
        assert_eq!(rng.next(), S42[2], "two rows drawn");
        // DrawDebugBox on a 3x2 rect: 10 perimeter cells — (1,2) and (3,2)
        // fall outside the C#'s edge test.
        let world = world();
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.draw_debug_box(&mut rng, &ctx(6000), &world, (0, 0, 3, 2), Color::default());
        let active: Vec<usize> = (0..DUST_SENTINEL)
            .filter(|&i| pool.slot(i).active)
            .collect();
        assert_eq!(active.len(), 10, "the edge test skips (1,2) and (3,2)");
        assert_eq!(pool.slot(0).position, Vec2::new(0.0, 0.0));
        assert_eq!(pool.slot(9).position, Vec2::new(3.0, 1.0));
        assert_eq!(pool.slot(0).color, Color::WHITE);
        assert_eq!(pool.slot(0).scale, 0.31);
        assert_eq!(pool.slot(0).fade_in, 0.0);
        assert_eq!(pool.slot(0).dust_type, 267);
        // An explicit colour is kept.
        let mut pool = new_pool();
        let mut rng = UnifiedRandom::new(42);
        pool.draw_debug_box(
            &mut rng,
            &ctx(6000),
            &world,
            (0, 0, 1, 1),
            Color::new(7, 7, 7, 7),
        );
        assert_eq!(pool.slot(0).color, Color::new(7, 7, 7, 7));
    }
}
