//! The Dust `Lighting.AddLight` sources, SECOND chain (`S1-E-5`, W4-g13 —
//! the research/202 flag-a orphan closure).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the FIRST-ladder dust family
//! (`dust_lights.rs`), the projectile families (`proj_lights.rs`,
//! `proj_lights_a.rs` W4-c19, `proj_ai_lights.rs` W4-e6), the item family
//! (`entity_lights.rs::item_light`) and the NPC family (`npc_lights.rs`); the
//! tile-anchor vocabulary is shared from `entity_lights.rs` too.
//!
//! Ported here: the **52** `AddLight` arms of `Dust.UpdateDust`'s SECOND
//! chain — every site from `:1061` (the 159 glow) to `:2312` (the 272
//! cursed-flame tail), 42 + 52 = the family's 94 sites. research/202 flag-a:
//! `dust_lights.rs` deferred these to "S1-E-3's chain port" while d50's
//! `dust_physics` (`origin/task/W4-d50-dust-type-table`, `2e243a2d`) skipped
//! them ALL as "S1-E-5 (`dust_lights`)" — the circular deferral this module
//! closes. The FIRST ladder's 42 stay in `dust_lights.rs` (its 206/174
//! local-decrement retire is research/202 flag-c, recorded in research/206).
//!
//! # The collector contract (the e6/c19 `EntityLights` pattern)
//!
//! [`dust_chain_light`] is a PURE per-slot fn over the live pool's POST-move
//! record — no RNG parameter exists, so the feed cannot move any stream (the
//! `dust_lights.rs` witness rule). The C# computes each arm's colour MID-ARM
//! from the scale/position of that moment; the port's pool runs the skeleton
//! only (`particles.rs::update_dust`, the 213/268 arms via `minecart::tick`),
//! so [`light_scale`] / [`light_position`] REPLAY the owning arm's pre-light
//! mutations on a local copy (the `dust_lights.rs` 206/174 compensation
//! precedent, extended to the whole chain). Every replay cites its d50
//! `type_chain` arm; when d50's chain is wired into the `type_arm` hook the
//! replays retire arm-by-arm (research/206 §4 — the flag-c generalisation).
//! The feed ([`feed_dust_chain_lights`]) runs in `app/frame/draw.rs`
//! immediately after `feed_dust_lights`, in `Main.dust` slot order
//! (`Dust.cs:431`), reading positions AFTER the move (d50 owns the move).
//!
//! # What stays dropped, exactly
//!
//! * **No arm here draws RNG.** The chain's `Main.rand` draws (159/164/173's
//!   velocity jitter + child spawns, 160/162, 168, 41/44, the 34/35/152 wet
//!   `Next(2)`/`Next(-10,10)`) are d50's physics column — the light itself is
//!   RNG-free in every arm, so the feed takes no draws.
//! * **`customData` is dropped** (`particles.rs`'s `Dust` record): the 226
//!   `customData is Color` second light (`:2270-2277`) cannot fire and is
//!   absent wholesale; the Player/NPC follower halves (43 `:1443`, 267
//!   `:1600`, 219-223 `:2239`, 226 `:2265`, 272 `:2323-2332`) are position
//!   mutations, not lights — d50's column.
//! * **The 35 wet-arm alpha kill** (`:1962-1965`, `alpha > 255 → scale = 0`):
//!   the kill is physics (d50's), but it gates the `:1994` light via the
//!   post-wet scale, so the 35 replay evaluates the wet probe and mirrors the
//!   DRAW-INDEPENDENT kill (`alpha > 255`) on its LOCAL copy. The `alpha ==
//!   255` boundary splits on the `Next(2)` draw (d50's) and stays with the
//!   physics wiring (research/206 §4).
//! * **The 43 dark-kill** (`:1435-1438`) reads the PRE-ramp terms and runs
//!   BEFORE the light's `else` — the collector mirrors it exactly (a dark 43
//!   emits nothing; the `active = false` write itself is d50's).
//! * **274/292 share the 15/57/58 arm but have NO AddLight branch**
//!   (`:1468-1479` covers 15/57/58 only) — dark, cited below. **134** shares
//!   the 130-133 arm with no branch (`:2137-2152`) — dark. **223** shares
//!   219-222 (`:2212-2227`) — dark.
//!
//! # The site register (all 52, in line order)
//!
//! | # | C# | type | colour (× the arm's num) | gate |
//! | - | -- | ---- | ------------------------ | ---- |
//! | 1 | `:1061` | 159 | (1, 1, 0.1), num32 = scale*1.3 cap 1 | none |
//! | 2 | `:1099` | 164 | (1, 0.1, 0.8), num34 = scale cap 1 | none |
//! | 3 | `:1137` | 173 | (0.4, 0.1, 1), num36 = scale cap 1 | none |
//! | 4 | `:1208` | 162 | (1, 0.7, 0.1), num40 = scale*1.3 cap 1 | none |
//! | 5 | `:1212` | 160 | (0.1, 1, 1), num40 | none |
//! | 6 | `:1241` | 168 | (1, 0, 0.8), num42 = scale*0.8 cap 0.55 | none |
//! | 7 | `:1399` | 244-247 | GetColor-fed (b/270+1)/2 * scale*0.9 * per-type tint * num47 | `!noLightEmittance` |
//! | 8 | `:1441` | 43 | GetColor/270 * scale*1.07 * color/255 | dark-kill else `!noLightEmittance` |
//! | 9 | `:1470` | 15 | (0.45, 0.55, 1), num54 = scale cap 1 | `!noLightEmittance` |
//! | 10 | `:1474` | 57 | (0.95, 0.95, 0.45), num54 = scale*0.8 cap 1 | `!noLightEmittance` |
//! | 11 | `:1478` | 58 | (1, 0.55, 0.75), num54 = scale*0.8 cap 1 | `!noLightEmittance` |
//! | 12 | `:1501` | 110 | (0.2, 1, 0.5), num55 = scale*0.1 cap 1 | none |
//! | 13 | `:1510` | 111 | (0.2, 0.7, 1), num56 = scale*0.125 cap 1 | none |
//! | 14 | `:1519` | 112 | (0.8, 0.2, 0.8), num57 = scale*0.1 cap 1 | none |
//! | 15 | `:1528` | 113 | (0.2, 0.3, 1.3), num58 = scale*0.1 cap 1 | none |
//! | 16 | `:1537` | 114 | (1.2, 0.5, 0.4), num59 = scale*0.1 cap 1 | none |
//! | 17 | `:1546` | 311 | torch 16 (Desert), num60 = scale*0.1 cap 1 | none |
//! | 18 | `:1555` | 312 | torch 9 (Ice), num61 = scale*0.1 cap 1 | none |
//! | 19 | `:1564` | 313 | (1, 0.8, 0.6), num62 = scale*0.25 cap 1 | none |
//! | 20 | `:1588` | 66 | color/255, num63 = (scale+0.02)*0.8 cap 1 (POST-grow) | none |
//! | 21 | `:1619` | 267 | color/255, num64 = (scale+0.02)*0.8 cap 1 | `!noLight && !noLightEmittance` (noLight lifted first) |
//! | 22 | `:1635` | 21 | (0.8, 0.3, 1), num65 = (scale+0.005)*0.4 UNCAPPED | `!noLightEmittance` (else falls to the 20 arm) |
//! | 23 | `:1640` | 231 | (1, 0.5, 0.3), num65 = (scale+0.005)*0.4 UNCAPPED | none |
//! | 24 | `:1644` | 20 | (0.3, 0.6, 1), num65 = (scale+0.005)*0.8 cap 1 | none |
//! | 25 | `:1692` | 27/45 | (0.6, 0.2, 1), num66 = post-burn scale (*0.1 if noLight) cap 1 | none |
//! | 26 | `:1706` | 55 | (1, 1, 0.6), num67 = scale*0.8 cap 1 | `!noLightEmittance` |
//! | 27 | `:1714` | 73 | (1, 0.35, 0.5), num67 = scale*0.8 cap 1 | `!noLightEmittance` |
//! | 28 | `:1722` | 74 | (0.35, 1, 0.5), num67 = scale*0.8 cap 1 | `!noLightEmittance` |
//! | 29 | `:1731` | 56 | (0.35, 0.5, 1), num67 = scale*1.2 cap 1 | `!noLightEmittance` |
//! | 30 | `:1743` | 71/72 | (0.2, 0, 0.1), num68 = scale cap 1 | none |
//! | 31 | `:1896` | 35 ng | (1, 0.3, 0.1), num69 = (scale+0.03)*0.6 cap 1, **y+1 tile** | `noGravity` |
//! | 32 | `:1926` | 67/92 | (0, 0.8, 1), num70 = scale cap 1 (*0.1 if noLight) | none |
//! | 33 | `:1939` | 185 | (0.1, 0.7, 1), num71 = scale cap 1 (*0.1 if noLight) | none |
//! | 34 | `:1950` | 107 | (0.1, 1, 0.4), num72 = scale*0.5 cap 1 | `!noLightEmittance` |
//! | 35 | `:1994` | 35 wet | (1, 0.5, 0.3), num73 = post-wet scale*0.3+0.4 cap 1 | gravity 35, wet-arm replay |
//! | 36 | `:2004` | 68 | (0.1, 0.2, 1), num74 = scale*0.3 cap 1 | none |
//! | 37 | `:2013` | 70 | (0.5, 0, 1), num75 = scale*0.3 cap 1 | none |
//! | 38 | `:2041` | 41 | (0.4, 0.9, 1), num76 = (scale+0.007)*0.7 cap 1 | none |
//! | 39 | `:2069` | 44 | (0.7, 1, 0.8), num77 = (scale+0.0085)*0.7 cap 1 | none |
//! | 40 | `:2139` | 130 | (1, 0.5, 0.4), num80 = post-ramp scale cap 1 | none |
//! | 41 | `:2143` | 131 | (0.4, 1, 0.6), num80 | none |
//! | 42 | `:2147` | 132 | (0.3, 0.5, 1), num80 | none |
//! | 43 | `:2151` | 133 | (0.9, 0.9, 0.3), num80 | none |
//! | 44 | `:2182` | 278 | color/255 * num81 (Vector2 overload), num81 = scale cap 1 | `!noLight && !noLightEmittance` |
//! | 45 | `:2214` | 219 | (1, 0.5, 0.4), num82 = scale cap 1 | `!noLight` |
//! | 46 | `:2218` | 220 | (0.4, 1, 0.6), num82 | `!noLight` |
//! | 47 | `:2222` | 221 | (0.3, 0.5, 1), num82 | `!noLight` |
//! | 48 | `:2226` | 222 | (0.9, 0.9, 0.3), num82 | `!noLight` |
//! | 49 | `:2254` | 226 | (0.2, 0.7, 1), num83 = scale cap 1 | `!noLight` |
//! | 50 | `:2275` | 226 | customData Color — ABSENT (field dropped) | `customData is Color` |
//! | 51 | `:2299` | 331 | color/255 * num84, num84 = scale cap 1 | `!noLightEmittance` |
//! | 52 | `:2312` | 272 | (0.5, 0.2, 0.8), num85 = scale cap 1 | `!noLight` |
//!
//! The 244-247 per-type tints (`:1366-1393`): 244 (226/255, 118/255, 76/255)
//! ×0.9; 245 (131/255, 172/255, 173/255) ×1; 246 (0.8, 181/255, 72/255)
//! ×1.1; 247 (0.6, 172/255, 185/255) ×1.2. The 244-247/43 ambient sample is
//! `Lighting.GetColor(tileX, tileY)` (`Lighting.cs:216-242`) — the port's
//! [`crate::main_dust::light_color_bytes`] (the light map stores the RAW
//! engine colour; GetColor multiplies GlobalBrightness, truncates, clamps).

use glam::Vec2;
use tr_core::lighting::{LightMap, LightParams};
use tr_core::particles::{DUST_SENTINEL, Dust, DustPool};
use tr_core::tile_light::torch_color;

use crate::entity_lights::{EntityLights, XAnchor, tile_x, tile_y};

/// The C# cap idiom `if (num > m) num = m;` — not `f32::min`, which resolves
/// NaN differently; the port mirrors the branch (the `dust_lights.rs` form).
#[inline]
fn cap(v: f32, m: f32) -> f32 {
    if v > m { m } else { v }
}

/// `Lighting.AddLight(int i, int j, int torchID, float lightAmount)`
/// (`Lighting.cs:145-149`): `TorchID.TorchColor(torchID)` scaled per channel.
#[inline]
fn torch_rgb(id: i32, amount: f32, p: &LightParams) -> [f32; 3] {
    let c = torch_color(id, p);
    [c[0] * amount, c[1] * amount, c[2] * amount]
}

/// The per-slot inputs the collector reads as `Main`/`Lighting` statics —
/// the d50 `DustPhysicsEnv` probe shape, narrowed to what the LIGHT column
/// needs (the physics probes stay d50's).
pub(crate) struct ChainLightEnv<'a> {
    /// `Main.player[Main.myPlayer].wet` (27/45 `:1679`).
    pub player_wet: bool,
    /// `Main.player[Main.myPlayer].velocity` (27/45 `:1681`/`:1685`).
    pub player_velocity: Vec2,
    /// `Lighting.GetColor(x, y)` as XNA bytes (244-247 `:1337`, 43 `:1405`) —
    /// the feed binds [`crate::main_dust::light_color_bytes`] over the light
    /// map + `style.brightness` (GlobalBrightness).
    pub light_color: &'a dyn Fn(i32, i32) -> (u8, u8, u8),
    /// `Collision.WetCollision` (the 35 wet-arm replay, `:1955`) — the feed
    /// binds [`crate::player::wet_collision`] (the pool's probe precedent).
    pub wet_collision: &'a dyn Fn(Vec2, f32, f32) -> bool,
}

/// The fadeIn ramp + burn-out (`Dust.cs:2083-2129`) as the SECOND-chain
/// types see it: none of 130-134/278/219-223/226/331/272 is 235/46/213/260/
/// 304, so the ramp is the generic +0.03 and the burn-out the generic −0.01.
/// The C# runs it BEFORE the second chain reads `num80..num85`, so the
/// collector's scale replay folds it (the d50 `type_chain` tail, `:872-911`
/// of `dust_physics.rs`). Pure — the record's `fade_in` is read, not written.
#[inline]
fn ramp_scale(d: &Dust, s: f32) -> f32 {
    if d.fade_in > 0.0 && d.fade_in < 100.0 {
        s + 0.03 // `:2110-2113` — the generic ramp (the cap zeroes fadeIn, a physics write)
    } else {
        s - 0.01 // `:2125-2128` — the generic burn-out
    }
}

/// The 27/45 arm's pre-light scale replay (`Dust.cs:1647-1692`, d50
/// `type_chain` `:612-642`): the 27 fadeIn>=100 burn-down, the +0.002 grow,
/// then the noLight double −0.06. The light's `num66` reads the scale AFTER
/// all of these (the C# computes it at `:1670`, post-mutation); the noLight
/// ×0.1 dim and the cap 1 ride [`dust_chain_light`]'s 27/45 arm. The player
/// push (`:1679-1686`) is a POSITION mutation — [`light_position`].
#[inline]
fn arm_27_45_scale(d: &Dust) -> f32 {
    let mut s = d.scale;
    if d.dust_type == 27 && d.fade_in >= 100.0 {
        // `:1651-1666` — the burn-down ladder (f64 compares, verbatim).
        if (s as f64) >= 1.5 {
            s -= 0.01;
        } else {
            s -= 0.05;
        }
        if (s as f64) <= 0.5 {
            s -= 0.05;
        }
        if (s as f64) <= 0.25 {
            s -= 0.05;
        }
    }
    s += 0.002; // `:1669`
    if d.no_light {
        s -= 0.06; // `:1674`
        if s < 1.0 {
            s -= 0.06; // `:1675-1678`
        }
    }
    s
}

/// The 35 gravity wet-arm's pre-light scale replay (`Dust.cs:1953-1995`, d50
/// `type_chain` `:790-816`). The wet probe gates the whole arm: dry sets
/// `scale = 0` (`:1957`); wet runs the alpha kill then the 35/152
/// `alpha++`/`scale -= 0.01` (`:1973-1974`). The light's `num73 = scale*0.3 +
/// 0.4` reads the POST-wet scale.
///
/// **The alpha kill is draw-dependent at the boundary**: `alpha += Next(2)`
/// (`:1961`) then `if (alpha > 255) scale = 0` (`:1962`). For `alpha >= 256`
/// the kill fires on EVERY draw; for `alpha <= 254` it NEVER fires; only
/// `alpha == 255` splits on the `Next(2)` result. The `Next(2)` is d50's
/// physics draw — the pure collector cannot take it, so it mirrors the
/// draw-independent kill (`alpha > 255`) and leaves the `alpha == 255`
/// boundary to the physics wiring (research/206 §4: d50's chain writes the
/// post-kill scale, the replay retires). The alpha/velocity writes themselves
/// are d50's physics column.
#[inline]
fn arm_35_wet_scale(d: &Dust, env: &ChainLightEnv<'_>) -> f32 {
    if !(env.wet_collision)(Vec2::new(d.position.x, d.position.y - 8.0), 4.0, 4.0) {
        return 0.0; // `:1955-1958` — dry: scale = 0
    }
    let mut s = d.scale;
    if d.alpha > 255 {
        s = 0.0; // `:1962-1965` — the draw-independent kill (alpha >= 256)
    }
    s -= 0.01; // `:1974` — the 35/152 else-arm shrink (34's +0.005 is not 35)
    s
}

/// The scale the C# arm reads at its `AddLight`, replayed from the post-move
/// record. Most arms read `d.scale` verbatim (the light precedes any scale
/// mutation, or the arm has none); the exceptions replay their pre-light
/// mutations. Each cites the d50 `type_chain` arm it mirrors.
#[inline]
fn light_scale(d: &Dust, env: &ChainLightEnv<'_>) -> f32 {
    match d.dust_type {
        // POST-grow reads (the C# grows BEFORE the light):
        66 | 267 => d.scale + 0.02, // `:1578`/`:1607` (d50 `:590`/`:602`)
        20 | 21 | 231 => d.scale + 0.005, // `:1624` (d50 `:609`)
        41 => d.scale + 0.007, // `:2035` (d50 `:839`)
        44 => d.scale + 0.0085, // `:2063` (d50 `:856`)
        // The 27/45 burn-down + noLight shrink (`:1647-1692`).
        27 | 45 => arm_27_45_scale(d),
        // The 35 noGravity grow (`:1877`, d50 `:762`) vs the gravity wet-arm.
        35 => {
            if d.no_gravity {
                d.scale + 0.03
            } else {
                arm_35_wet_scale(d, env)
            }
        }
        // The SECOND chain reads the POST-fadeIn-ramp scale (`:2083-2129`
        // precedes `:2130-2334`); none of these types is a ramp special-case.
        130..=134 | 278 | 219..=223 | 226 | 331 | 272 => ramp_scale(d, d.scale),
        // Every other arm reads the scale verbatim (light-first or no mutation).
        _ => d.scale,
    }
}

/// The position the C# arm anchors its `AddLight` at, replayed from the
/// post-move record. Only the 27/45 noLight player-push (`:1679-1686`) moves
/// the dust BEFORE its light; every other arm anchors the post-move position
/// (the customData followers are dropped, and the second-chain followers run
/// AFTER their lights). The push is d50's physics column (`type_chain`
/// `:636-641`); the collector replays it so the anchor tile matches.
#[inline]
fn light_position(d: &Dust, env: &ChainLightEnv<'_>) -> Vec2 {
    if (d.dust_type == 27 || d.dust_type == 45) && d.no_light {
        let push = if env.player_wet {
            env.player_velocity * 0.5 // `:1681`
        } else {
            env.player_velocity // `:1685`
        };
        d.position + push
    } else {
        d.position
    }
}

/// The `Dust.UpdateDust` SECOND-chain light contribution of one dust record
/// this tick, or `None`. Pure: no RNG, no mutation (module docs). `p` resolves
/// the animated torch colours (311 Desert / 312 Ice) exactly as the FIRST
/// ladder's torch arms do; `env` carries the ambient-light sample, the player
/// push and the wet probe. The scale/position replays ([`light_scale`] /
/// [`light_position`]) recover the C#'s mid-arm values from the post-move
/// record (the d50 wiring retires them arm-by-arm, research/206 §4).
pub(crate) fn dust_chain_light(
    d: &Dust,
    p: &LightParams,
    env: &ChainLightEnv<'_>,
) -> Option<(i32, i32, [f32; 3])> {
    let s = light_scale(d, env);
    let rgb: [f32; 3] = match d.dust_type {
        // `:1054-1061` — 159: num32 = scale*1.3 cap 1, (1, 1, 0.1).
        159 => {
            let n = cap(s * 1.3, 1.0);
            [n, n, n * 0.1]
        }
        // `:1092-1099` — 164: num34 = scale cap 1, (1, 0.1, 0.8).
        164 => {
            let n = cap(s, 1.0);
            [n, n * 0.1, n * 0.8]
        }
        // `:1130-1137` — 173: num36 = scale cap 1, (0.4, 0.1, 1).
        173 => {
            let n = cap(s, 1.0);
            [n * 0.4, n * 0.1, n]
        }
        // `:1199-1213` — 160/162: num40 = scale*1.3 cap 1; 162 warm, 160 cyan.
        160 | 162 => {
            let n = cap(s * 1.3, 1.0);
            if d.dust_type == 162 {
                [n, n * 0.7, n * 0.1] // `:1208`
            } else {
                [n * 0.1, n, n] // `:1212`
            }
        }
        // `:1234-1241` — 168: num42 = scale*0.8 cap 0.55, (1, 0, 0.8).
        168 => {
            let n = cap(s * 0.8, 0.55);
            [n, 0.0, n * 0.8]
        }
        // `:1334-1400` — 244-247: the GetColor-fed ambient glow. num44/45/46
        // all start ((b/270)+1)/2 * scale*0.9 (PRE-ramp scale, `:1339-1344`),
        // then the per-type tint + num47 (`:1366-1396`). Gate `!noLightEmittance`.
        244..=247 => {
            if d.no_light_emittance {
                return None; // `:1397`
            }
            let (r, g, b) = (env.light_color)(
                (d.position.x / 16.0) as i32,
                (d.position.y / 16.0) as i32,
            );
            let byte = ((r as i32 + g as i32 + b as i32) / 3) as u8; // `:1338`
            let base = (byte as f32 / 270.0 + 1.0) / 2.0 * s * 0.9; // `:1339-1344`
            let (tr, tg, tb, n47) = match d.dust_type {
                244 => (226.0 / 255.0, 118.0 / 255.0, 76.0 / 255.0, 0.9),
                245 => (131.0 / 255.0, 172.0 / 255.0, 173.0 / 255.0, 1.0),
                246 => (0.8, 181.0 / 255.0, 72.0 / 255.0, 1.1),
                _ => (0.6, 172.0 / 255.0, 185.0 / 255.0, 1.2), // 247
            };
            [base * tr * n47, base * tg * n47, base * tb * n47]
        }
        // `:1402-1442` — 43: GetColor/270 * scale*1.07 * color/255 (PRE-ramp).
        // The full-dark kill (`:1435`) runs BEFORE the light's else; a dark 43
        // emits nothing (the active=false write is d50's). Gate `!noLightEmittance`.
        43 => {
            let (r, g, b) = (env.light_color)(
                (d.position.x / 16.0) as i32,
                (d.position.y / 16.0) as i32,
            );
            let n48 = r as f32 / 270.0 * s * 1.07 * (d.color.r as f32 / 255.0);
            let n49 = g as f32 / 270.0 * s * 1.07 * (d.color.g as f32 / 255.0);
            let n50 = b as f32 / 270.0 * s * 1.07 * (d.color.b as f32 / 255.0);
            if (n48 as f64) < 0.05 && (n49 as f64) < 0.05 && (n50 as f64) < 0.05 {
                return None; // `:1435-1438` — the dark kill suppresses the else light
            }
            if d.no_light_emittance {
                return None; // `:1439`
            }
            [n48, n49, n50]
        }
        // `:1449-1480` — 15/57/58 (274/292 share the arm, NO AddLight branch —
        // dark). Gate `!noLightEmittance`; num54 = scale (15) or scale*0.8, cap 1.
        15 | 57 | 58 => {
            if d.no_light_emittance {
                return None; // `:1453`
            }
            let n = cap(if d.dust_type == 15 { s } else { s * 0.8 }, 1.0);
            match d.dust_type {
                15 => [n * 0.45, n * 0.55, n], // `:1470`
                57 => [n * 0.95, n * 0.95, n * 0.45], // `:1474`
                _ => [n, n * 0.55, n * 0.75],  // 58, `:1478`
            }
        }
        // `:1494-1564` — the 110-114/311-313 light-only arms (no scale mutation).
        110 => {
            let n = cap(s * 0.1, 1.0);
            [n * 0.2, n, n * 0.5] // `:1501`
        }
        111 => {
            let n = cap(s * 0.125, 1.0);
            [n * 0.2, n * 0.7, n] // `:1510`
        }
        112 => {
            let n = cap(s * 0.1, 1.0);
            [n * 0.8, n * 0.2, n * 0.8] // `:1519`
        }
        113 => {
            let n = cap(s * 0.1, 1.0);
            [n * 0.2, n * 0.3, n * 1.3] // `:1528`
        }
        114 => {
            let n = cap(s * 0.1, 1.0);
            [n * 1.2, n * 0.5, n * 0.4] // `:1537`
        }
        311 => torch_rgb(16, cap(s * 0.1, 1.0), p), // `:1546` — Desert torch
        312 => torch_rgb(9, cap(s * 0.1, 1.0), p),  // `:1555` — Ice torch
        313 => {
            let n = cap(s * 0.25, 1.0);
            [n, n * 0.8, n * 0.6] // `:1564`
        }
        // `:1566-1588` — 66: num63 = (scale+0.02)*0.8 cap 1 (the `type != 15`
        // branch always fires for 66), × color/255. POST-grow scale.
        66 => {
            let n = cap(s * 0.8, 1.0);
            [
                n * (d.color.r as f32 / 255.0),
                n * (d.color.g as f32 / 255.0),
                n * (d.color.b as f32 / 255.0),
            ]
        }
        // `:1590-1620` — 267: num64 = (scale+0.02)*0.8 cap 1 × color/255. The
        // `noLight = false` lift (`:1613-1616`) precedes the gate, so noLight
        // never mutes it; only noLightEmittance does. POST-grow scale.
        267 => {
            if d.no_light_emittance {
                return None; // `:1617` (noLight already lifted, d50 `:603-605`)
            }
            let n = cap(s * 0.8, 1.0);
            [
                n * (d.color.r as f32 / 255.0),
                n * (d.color.g as f32 / 255.0),
                n * (d.color.b as f32 / 255.0),
            ]
        }
        // `:1622-1645` — 20/21/231: POST-grow scale (+0.005). The C# else-if is
        // `if (type==21 && !noLightEmittance)` / `else if (type==231)` / `else`:
        // a 21 WITH noLightEmittance falls to the ELSE arm (num65 = scale*0.8
        // cap 1, the 20 colour) — it does NOT go dark. 21 (no emittance) and
        // 231 use scale*0.4 UNCAPPED.
        20 | 21 | 231 => {
            if d.dust_type == 21 && !d.no_light_emittance {
                let n = s * 0.4; // `:1634` — UNCAPPED
                [n * 0.8, n * 0.3, n] // `:1635`
            } else if d.dust_type == 231 {
                let n = s * 0.4; // `:1639` — UNCAPPED
                [n, n * 0.5, n * 0.3] // `:1640`
            } else {
                // `:1642-1644` — the else arm: type 20, OR a 21 muted by
                // noLightEmittance. num65 keeps the `:1627-1631` scale*0.8 cap 1.
                let n = cap(s * 0.8, 1.0);
                [n * 0.3, n * 0.6, n]
            }
        }
        // `:1647-1692` — 27/45: num66 = post-burn scale (×0.1 if noLight) cap 1,
        // (0.6, 0.2, 1). The anchor rides the player push ([`light_position`]).
        27 | 45 => {
            let n = cap(if d.no_light { s * 0.1 } else { s }, 1.0);
            [n * 0.6, n * 0.2, n]
        }
        // `:1694-1733` — 55/56/73/74: gate `!noLightEmittance`; num67 =
        // scale*0.8 cap 1 (55/73/74) or scale*1.2 cap 1 (56, the else).
        55 | 56 | 73 | 74 => {
            if d.no_light_emittance {
                return None; // `:1697`
            }
            if d.dust_type == 56 {
                let n = cap(s * 1.2, 1.0); // `:1726-1730`
                [n * 0.35, n * 0.5, n] // `:1731`
            } else {
                let n = cap(s * 0.8, 1.0);
                match d.dust_type {
                    55 => [n, n, n * 0.6],          // `:1706`
                    73 => [n, n * 0.35, n * 0.5],   // `:1714`
                    _ => [n * 0.35, n, n * 0.5],    // 74, `:1722`
                }
            }
        }
        // `:1735-1743` — 71/72: num68 = scale cap 1, (0.2, 0, 0.1).
        71 | 72 => {
            let n = cap(s, 1.0);
            [n * 0.2, 0.0, n * 0.1]
        }
        // `:1875-1896` — 35 noGravity: num69 = (scale+0.03)*0.6 cap 1,
        // (1, 0.3, 0.1) at **(x, y+1)** — the one y-offset anchor in the family.
        35 if d.no_gravity => {
            let n = cap(s * 0.6, 1.0);
            let pos = light_position(d, env);
            return Some((
                tile_x(pos, 0.0, XAnchor::Center),
                (pos.y / 16.0 + 1.0) as i32, // `:1896` — `(int)(Y/16f + 1f)`
                [n, n * 0.3, n * 0.1],
            ));
        }
        // `:1915-1926` — 67/92: num70 = scale cap 1 (×0.1 if noLight), (0, 0.8, 1).
        67 | 92 => {
            let n = cap(if d.no_light { s * 0.1 } else { s }, 1.0);
            [0.0, n * 0.8, n]
        }
        // `:1928-1939` — 185: num71 = scale cap 1 (×0.1 if noLight), (0.1, 0.7, 1).
        185 => {
            let n = cap(if d.no_light { s * 0.1 } else { s }, 1.0);
            [n * 0.1, n * 0.7, n]
        }
        // `:1941-1950` — 107: num72 = scale*0.5 cap 1, gate `!noLightEmittance`.
        107 => {
            if d.no_light_emittance {
                return None; // `:1948`
            }
            let n = cap(s * 0.5, 1.0);
            [n * 0.1, n, n * 0.4]
        }
        // `:1953-1994` — 35 gravity: the wet-arm replay; num73 = post-wet
        // scale*0.3 + 0.4 cap 1, (1, 0.5, 0.3). (noGravity 35 returned above.)
        35 => {
            let n = cap(s * 0.3 + 0.4, 1.0);
            [n, n * 0.5, n * 0.3]
        }
        // `:1997-2004` — 68: num74 = scale*0.3 cap 1, (0.1, 0.2, 1).
        68 => {
            let n = cap(s * 0.3, 1.0);
            [n * 0.1, n * 0.2, n]
        }
        // `:2006-2013` — 70: num75 = scale*0.3 cap 1, (0.5, 0, 1).
        70 => {
            let n = cap(s * 0.3, 1.0);
            [n * 0.5, 0.0, n]
        }
        // `:2015-2041` — 41: num76 = (scale+0.007)*0.7 cap 1, (0.4, 0.9, 1).
        41 => {
            let n = cap(s * 0.7, 1.0);
            [n * 0.4, n * 0.9, n]
        }
        // `:2043-2069` — 44: num77 = (scale+0.0085)*0.7 cap 1, (0.7, 1, 0.8).
        44 => {
            let n = cap(s * 0.7, 1.0);
            [n * 0.7, n, n * 0.8]
        }
        // `:2130-2151` — 130-133 (134 shares the arm, NO branch — dark):
        // num80 = post-ramp scale cap 1.
        130..=133 => {
            let n = cap(s, 1.0);
            match d.dust_type {
                130 => [n, n * 0.5, n * 0.4],       // `:2139`
                131 => [n * 0.4, n, n * 0.6],       // `:2143`
                132 => [n * 0.3, n * 0.5, n],       // `:2147`
                _ => [n * 0.9, n * 0.9, n * 0.3],   // 133, `:2151`
            }
        }
        // `:2173-2182` — 278: color/255 * num81 (Vector2 overload), num81 =
        // post-ramp scale cap 1. Gate `!noLight && !noLightEmittance`.
        278 => {
            if d.no_light || d.no_light_emittance {
                return None; // `:2180`
            }
            let n = cap(s, 1.0);
            [
                d.color.r as f32 / 255.0 * n,
                d.color.g as f32 / 255.0 * n,
                d.color.b as f32 / 255.0 * n,
            ]
        }
        // `:2203-2226` — 219-222 (223 shares the arm, NO branch — dark):
        // num82 = post-ramp scale cap 1. Gate `!noLight`.
        219..=222 => {
            if d.no_light {
                return None; // `:2210`
            }
            let n = cap(s, 1.0);
            match d.dust_type {
                219 => [n, n * 0.5, n * 0.4],       // `:2214`
                220 => [n * 0.4, n, n * 0.6],       // `:2218`
                221 => [n * 0.3, n * 0.5, n],       // `:2222`
                _ => [n * 0.9, n * 0.9, n * 0.3],   // 222, `:2226`
            }
        }
        // `:2245-2254` — 226: num83 = post-ramp scale cap 1, (0.2, 0.7, 1).
        // Gate `!noLight`. The customData Color second light (`:2275`) is
        // absent (field dropped, module docs).
        226 => {
            if d.no_light {
                return None; // `:2252`
            }
            let n = cap(s, 1.0);
            [n * 0.2, n * 0.7, n]
        }
        // `:2280-2299` — 331: color/255 * num84, num84 = post-ramp scale cap 1.
        // Gate `!noLightEmittance`.
        331 => {
            if d.no_light_emittance {
                return None; // `:2297`
            }
            let n = cap(s, 1.0);
            [
                d.color.r as f32 / 255.0 * n,
                d.color.g as f32 / 255.0 * n,
                d.color.b as f32 / 255.0 * n,
            ]
        }
        // `:2303-2312` — 272: num85 = post-ramp scale cap 1, (0.5, 0.2, 0.8).
        // Gate `!noLight`.
        272 => {
            if d.no_light {
                return None; // `:2310`
            }
            let n = cap(s, 1.0);
            [n * 0.5, n * 0.2, n * 0.8]
        }
        _ => return None,
    };
    // Every arm anchors `(int)(position / 16f)` per axis (the Center form,
    // zero box) — except the 35 noGravity y+1, which returned above.
    let pos = light_position(d, env);
    Some((
        tile_x(pos, 0.0, XAnchor::Center),
        tile_y(pos, 0.0, XAnchor::Center),
        rgb,
    ))
}

/// The SECOND-chain feed: walk the pool in `Main.dust` slot order
/// (`Dust.cs:431`) and append every active dust's contribution to the shared
/// [`EntityLights`] pair, immediately after the FIRST-ladder feed
/// (`dust_lights.rs::feed_dust_lights`). Pure over the pool — no RNG parameter
/// exists, so the feed cannot move any stream.
pub(crate) fn feed_dust_chain_lights(
    pool: &DustPool,
    p: &LightParams,
    env: &ChainLightEnv<'_>,
    lights: &mut EntityLights,
) {
    for d in &pool.slots()[..DUST_SENTINEL] {
        if !d.active {
            continue;
        }
        if let Some((x, y, rgb)) = dust_chain_light(d, p, env) {
            lights.add(x, y, rgb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::particles::{Color, UpdateCtx};
    use tr_core::UnifiedRandom;

    fn params() -> LightParams {
        let mut p = LightParams::default();
        p.demon_torch = 1.0;
        p.disco = [255, 0, 0];
        p
    }

    fn dust(ty: i32, scale: f32, pos: Vec2) -> Dust {
        Dust {
            active: true,
            dust_type: ty,
            scale,
            position: pos,
            ..Default::default()
        }
    }

    /// The default probe env: bright ambient (255,255,255), dry player at
    /// rest, always-wet collision (the 35 wet-arm tests override it).
    fn with_env<R>(f: impl FnOnce(&ChainLightEnv<'_>) -> R) -> R {
        let lc = |_: i32, _: i32| (255u8, 255u8, 255u8);
        let wet = |_: Vec2, _: f32, _: f32| true;
        let env = ChainLightEnv {
            player_wet: false,
            player_velocity: Vec2::ZERO,
            light_color: &lc,
            wet_collision: &wet,
        };
        f(&env)
    }

    fn light(d: &Dust) -> Option<(i32, i32, [f32; 3])> {
        with_env(|env| dust_chain_light(d, &params(), env))
    }

    #[test]
    fn the_glowstick_trio_caps_and_colours() {
        // 159 (`:1061`): scale 1 -> num32 = 1.3 cap 1, (1, 1, 0.1).
        assert_eq!(light(&dust(159, 1.0, Vec2::ZERO)).unwrap().2, [1.0, 1.0, 0.1]);
        // 159 scale 0.5 -> 0.65, under the cap.
        let n = 0.5f32 * 1.3;
        assert_eq!(light(&dust(159, 0.5, Vec2::ZERO)).unwrap().2, [n, n, n * 0.1]);
        // 164 (`:1099`): scale 2 -> num34 cap 1, (1, 0.1, 0.8).
        assert_eq!(light(&dust(164, 2.0, Vec2::ZERO)).unwrap().2, [1.0, 0.1, 0.8]);
        // 173 (`:1137`): scale 0.5 -> num36 = 0.5, (0.2, 0.05, 0.5).
        assert_eq!(
            light(&dust(173, 0.5, Vec2::ZERO)).unwrap().2,
            [0.5 * 0.4, 0.5 * 0.1, 0.5]
        );
    }

    #[test]
    fn the_160_162_pair_splits_warm_and_cyan() {
        // num40 = scale*1.3 cap 1. Scale 1 -> 1.
        // 162 (`:1208`): (1, 0.7, 0.1).
        assert_eq!(light(&dust(162, 1.0, Vec2::ZERO)).unwrap().2, [1.0, 0.7, 0.1]);
        // 160 (`:1212`): (0.1, 1, 1).
        assert_eq!(light(&dust(160, 1.0, Vec2::ZERO)).unwrap().2, [0.1, 1.0, 1.0]);
        // The cap: scale 2 -> 2.6 -> 1.
        assert_eq!(light(&dust(160, 2.0, Vec2::ZERO)).unwrap().2, [0.1, 1.0, 1.0]);
    }

    #[test]
    fn the_168_cap_is_055_not_1() {
        // `:1236-1241`: num42 = scale*0.8 cap 0.55. Scale 1 -> 0.8 -> 0.55.
        assert_eq!(
            light(&dust(168, 1.0, Vec2::ZERO)).unwrap().2,
            [0.55, 0.0, 0.55 * 0.8]
        );
        // Scale 0.5 -> 0.4, under the cap.
        let n = 0.5f32 * 0.8;
        assert_eq!(light(&dust(168, 0.5, Vec2::ZERO)).unwrap().2, [n, 0.0, n * 0.8]);
    }

    #[test]
    fn the_244_247_ambient_tints_ride_the_cell_light() {
        // Bright ambient (255,255,255): b = 255, base = (255/270+1)/2 * scale*0.9.
        // Scale 1 -> base = (1.9444)/2 * 0.9. Per-type tint × num47.
        let base = (255.0f32 / 270.0 + 1.0) / 2.0 * 1.0 * 0.9;
        let got = light(&dust(244, 1.0, Vec2::ZERO)).unwrap().2;
        assert_eq!(
            got,
            [
                base * (226.0 / 255.0) * 0.9,
                base * (118.0 / 255.0) * 0.9,
                base * (76.0 / 255.0) * 0.9
            ]
        );
        // 247: (0.6, 172/255, 185/255) × 1.2.
        let got = light(&dust(247, 1.0, Vec2::ZERO)).unwrap().2;
        assert_eq!(
            got,
            [base * 0.6 * 1.2, base * (172.0 / 255.0) * 1.2, base * (185.0 / 255.0) * 1.2]
        );
        // The gate: noLightEmittance mutes it.
        let mut d = dust(245, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_43_dark_kill_suppresses_the_light() {
        // A white 43 in bright ambient lights; the terms are well above 0.05.
        let mut d = dust(43, 1.0, Vec2::ZERO);
        d.color = Color::new(255, 255, 255, 255);
        let n = 255.0f32 / 270.0 * 1.0 * 1.07 * 1.0;
        assert_eq!(light(&d).unwrap().2, [n, n, n]);
        // A BLACK 43 (color 0) -> all terms 0 < 0.05 -> the dark kill, no light.
        d.color = Color::new(0, 0, 0, 255);
        assert!(light(&d).is_none());
        // noLightEmittance mutes a bright 43 too.
        d.color = Color::new(255, 255, 255, 255);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_15_57_58_arm_and_the_dark_274_292() {
        // 15 (`:1470`): num54 = scale cap 1, (0.45, 0.55, 1).
        assert_eq!(light(&dust(15, 1.0, Vec2::ZERO)).unwrap().2, [0.45, 0.55, 1.0]);
        // 57 (`:1474`): num54 = scale*0.8 cap 1. Scale 1 -> 0.8.
        let n = 0.8f32;
        assert_eq!(
            light(&dust(57, 1.0, Vec2::ZERO)).unwrap().2,
            [n * 0.95, n * 0.95, n * 0.45]
        );
        // 58 (`:1478`): (1, 0.55, 0.75) × 0.8.
        assert_eq!(
            light(&dust(58, 1.0, Vec2::ZERO)).unwrap().2,
            [n, n * 0.55, n * 0.75]
        );
        // 274/292 share the arm but have NO AddLight branch — dark.
        assert!(light(&dust(274, 1.0, Vec2::ZERO)).is_none());
        assert!(light(&dust(292, 1.0, Vec2::ZERO)).is_none());
        // The gate.
        let mut d = dust(15, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_110_114_light_only_arms() {
        // num = scale*0.1 cap 1 (111 uses 0.125). Scale 1 -> 0.1 / 0.125.
        let n = 0.1f32;
        assert_eq!(light(&dust(110, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.2, n, n * 0.5]);
        let n = 0.125f32;
        assert_eq!(light(&dust(111, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.2, n * 0.7, n]);
        let n = 0.1f32;
        assert_eq!(light(&dust(112, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.8, n * 0.2, n * 0.8]);
        assert_eq!(light(&dust(113, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.2, n * 0.3, n * 1.3]);
        assert_eq!(light(&dust(114, 1.0, Vec2::ZERO)).unwrap().2, [n * 1.2, n * 0.5, n * 0.4]);
        // The cap: scale 20 -> 2 -> 1.
        assert_eq!(light(&dust(110, 20.0, Vec2::ZERO)).unwrap().2, [0.2, 1.0, 0.5]);
    }

    #[test]
    fn the_311_312_torch_arms_and_313() {
        let p = params();
        // 311 (`:1546`): torch 16 (Desert) = [1.4, 0.85, 0.55], amount scale*0.1 cap 1.
        with_env(|env| {
            let got = dust_chain_light(&dust(311, 1.0, Vec2::ZERO), &p, env).unwrap().2;
            assert_eq!(got, [1.4 * 0.1, 0.85 * 0.1, 0.55 * 0.1]);
            // 312 (`:1555`): torch 9 (Ice) = [0.75, 0.85, 1.4].
            let got = dust_chain_light(&dust(312, 1.0, Vec2::ZERO), &p, env).unwrap().2;
            assert_eq!(got, [0.75 * 0.1, 0.85 * 0.1, 1.4 * 0.1]);
        });
        // 313 (`:1564`): num62 = scale*0.25 cap 1, (1, 0.8, 0.6). Scale 1 -> 0.25.
        let n = 0.25f32;
        assert_eq!(light(&dust(313, 1.0, Vec2::ZERO)).unwrap().2, [n, n * 0.8, n * 0.6]);
    }

    #[test]
    fn the_66_267_post_grow_colour_arms() {
        // 66 (`:1588`): num63 = (scale+0.02)*0.8 cap 1 × color/255. Scale 1 -> 1.02*0.8.
        let mut d = dust(66, 1.0, Vec2::ZERO);
        d.color = Color::new(200, 100, 50, 255);
        let n = (1.0f32 + 0.02) * 0.8;
        assert_eq!(
            light(&d).unwrap().2,
            [n * (200.0 / 255.0), n * (100.0 / 255.0), n * (50.0 / 255.0)]
        );
        // 267 (`:1619`): same grow; noLight is LIFTED first, so it does not mute.
        let mut d = dust(267, 1.0, Vec2::ZERO);
        d.color = Color::new(255, 255, 255, 255);
        d.no_light = true;
        let n = (1.0f32 + 0.02) * 0.8;
        assert_eq!(light(&d).unwrap().2, [n, n, n]);
        // noLightEmittance DOES mute 267.
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_20_21_231_grow_and_the_uncapped_21_231() {
        // POST-grow scale (+0.005). Scale 1 -> 1.005.
        // 20 (`:1644`): num65 = 1.005*0.8 cap 1, (0.3, 0.6, 1).
        let n = (1.0f32 + 0.005) * 0.8;
        assert_eq!(light(&dust(20, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.3, n * 0.6, n]);
        // 21 (`:1635`): num65 = 1.005*0.4 UNCAPPED, (0.8, 0.3, 1). Scale 4 -> 4.005*0.4 = 1.602 > 1.
        let n = (4.0f32 + 0.005) * 0.4;
        assert!(n > 1.0, "21's num65 is uncapped");
        assert_eq!(light(&dust(21, 4.0, Vec2::ZERO)).unwrap().2, [n * 0.8, n * 0.3, n]);
        // 231 (`:1640`): num65 = scale*0.4 UNCAPPED, (1, 0.5, 0.3).
        assert_eq!(light(&dust(231, 4.0, Vec2::ZERO)).unwrap().2, [n, n * 0.5, n * 0.3]);
        // 21 WITH noLightEmittance falls to the ELSE arm (the 20 colour,
        // scale*0.8 cap 1) — it does NOT go dark (`:1632` vs `:1642`).
        let mut d = dust(21, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        let n = (1.0f32 + 0.005) * 0.8;
        assert_eq!(light(&d).unwrap().2, [n * 0.3, n * 0.6, n]);
        // 231 is NOT gated by noLightEmittance.
        let mut d = dust(231, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert!(light(&d).is_some());
    }

    #[test]
    fn the_27_45_burn_down_and_no_light_dim_and_push() {
        // 45 (no fadeIn burn): num66 = (scale+0.002) cap 1, (0.6, 0.2, 1).
        let n = 1.0f32 + 0.002;
        assert_eq!(light(&dust(45, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.6, n * 0.2, n]);
        // 27 fadeIn >= 100, scale 1.6: burn -0.01 (>=1.5), then +0.002.
        let mut d = dust(27, 1.6, Vec2::ZERO);
        d.fade_in = 100.0;
        let s = 1.6f32 - 0.01 + 0.002;
        assert_eq!(light(&d).unwrap().2, [s * 0.6, s * 0.2, s]);
        // 27 fadeIn >= 100, scale 0.6: burn -0.05 (<1.5), then <=0.5? 0.55 no;
        // +0.002 -> 0.552.
        let mut d = dust(27, 0.6, Vec2::ZERO);
        d.fade_in = 100.0;
        let s = 0.6f32 - 0.05 + 0.002;
        assert_eq!(light(&d).unwrap().2, [s * 0.6, s * 0.2, s]);
        // noLight: num66 *= 0.1, scale -= 0.06 twice (post-grow < 1), and the
        // anchor rides the player push. Dry player velocity (3, 4).
        let mut d = dust(45, 1.0, Vec2::new(160.0, 176.0));
        d.no_light = true;
        let lc = |_: i32, _: i32| (255u8, 255u8, 255u8);
        let wet = |_: Vec2, _: f32, _: f32| true;
        let env = ChainLightEnv {
            player_wet: false,
            player_velocity: Vec2::new(3.0, 4.0),
            light_color: &lc,
            wet_collision: &wet,
        };
        let got = dust_chain_light(&d, &params(), &env).unwrap();
        // scale 1 + 0.002 - 0.06 - 0.06 = 0.882; num66 = 0.0882.
        let s = 1.0f32 + 0.002 - 0.06 - 0.06;
        let n = s * 0.1;
        assert_eq!(got.2, [n * 0.6, n * 0.2, n]);
        // position (160,176) + (3,4) = (163,180) -> tile (10, 11).
        assert_eq!((got.0, got.1), (10, 11));
    }

    #[test]
    fn the_55_56_73_74_glowstick_colours() {
        // Gate !noLightEmittance. num67 = scale*0.8 cap 1 (55/73/74).
        let n = 0.8f32;
        assert_eq!(light(&dust(55, 1.0, Vec2::ZERO)).unwrap().2, [n, n, n * 0.6]);
        assert_eq!(light(&dust(73, 1.0, Vec2::ZERO)).unwrap().2, [n, n * 0.35, n * 0.5]);
        assert_eq!(light(&dust(74, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.35, n, n * 0.5]);
        // 56 (the else): num67 = scale*1.2 cap 1. Scale 1 -> 1.2 -> 1.
        assert_eq!(light(&dust(56, 1.0, Vec2::ZERO)).unwrap().2, [0.35, 0.5, 1.0]);
        // 56 scale 0.5 -> 0.6, under the cap.
        let n = 0.5f32 * 1.2;
        assert_eq!(light(&dust(56, 0.5, Vec2::ZERO)).unwrap().2, [n * 0.35, n * 0.5, n]);
        let mut d = dust(55, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_71_72_coin_sparkle() {
        // num68 = scale cap 1, (0.2, 0, 0.1). Scale 2 -> cap 1.
        assert_eq!(light(&dust(71, 2.0, Vec2::ZERO)).unwrap().2, [0.2, 0.0, 0.1]);
        assert_eq!(light(&dust(72, 0.5, Vec2::ZERO)).unwrap().2, [0.1, 0.0, 0.05]);
    }

    #[test]
    fn the_35_no_gravity_lights_one_tile_lower() {
        // `:1896`: num69 = (scale+0.03)*0.6 cap 1, (1, 0.3, 0.1) at (x, y+1).
        let mut d = dust(35, 1.0, Vec2::new(160.0, 176.0));
        d.no_gravity = true;
        let got = light(&d).unwrap();
        let n = (1.0f32 + 0.03) * 0.6;
        assert_eq!(got.2, [n, n * 0.3, n * 0.1]);
        // x = 160/16 = 10; y = (176/16 + 1) = 12 (NOT 11).
        assert_eq!((got.0, got.1), (10, 12));
        // The cap: scale 2 -> 2.03*0.6 = 1.218 -> 1.
        d.scale = 2.0;
        assert_eq!(light(&d).unwrap().2, [1.0, 0.3, 0.1]);
    }

    #[test]
    fn the_35_gravity_wet_arm_replay() {
        // Gravity 35, wet probe true, alpha 0: num73 = (scale-0.01)*0.3+0.4 cap 1.
        let mut d = dust(35, 1.0, Vec2::ZERO);
        d.no_gravity = false;
        let n = (1.0f32 - 0.01) * 0.3 + 0.4;
        assert_eq!(light(&d).unwrap().2, [n, n * 0.5, n * 0.3]);
        // alpha > 255: the draw-independent kill zeroes the scale -> num73 =
        // (0-0.01)*0.3+0.4 = 0.397.
        d.alpha = 256;
        let n = (0.0f32 - 0.01) * 0.3 + 0.4;
        assert_eq!(light(&d).unwrap().2, [n, n * 0.5, n * 0.3]);
        // Dry probe: scale = 0 -> same 0.397 floor.
        let lc = |_: i32, _: i32| (255u8, 255u8, 255u8);
        let dry = |_: Vec2, _: f32, _: f32| false;
        let env = ChainLightEnv {
            player_wet: false,
            player_velocity: Vec2::ZERO,
            light_color: &lc,
            wet_collision: &dry,
        };
        let mut d = dust(35, 1.0, Vec2::ZERO);
        d.no_gravity = false;
        let n = (0.0f32 - 0.01) * 0.3 + 0.4;
        assert_eq!(dust_chain_light(&d, &params(), &env).unwrap().2, [n, n * 0.5, n * 0.3]);
    }

    #[test]
    fn the_67_92_185_107_no_light_dim_arms() {
        // 67/92 (`:1926`): num70 = scale cap 1, (0, 0.8, 1); noLight ×0.1.
        assert_eq!(light(&dust(67, 1.0, Vec2::ZERO)).unwrap().2, [0.0, 0.8, 1.0]);
        let mut d = dust(92, 1.0, Vec2::ZERO);
        d.no_light = true;
        assert_eq!(light(&d).unwrap().2, [0.0, 0.08, 0.1]);
        // 185 (`:1939`): (0.1, 0.7, 1); noLight ×0.1.
        assert_eq!(light(&dust(185, 1.0, Vec2::ZERO)).unwrap().2, [0.1, 0.7, 1.0]);
        // 107 (`:1950`): num72 = scale*0.5 cap 1, gate !noLightEmittance.
        let n = 0.5f32;
        assert_eq!(light(&dust(107, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.1, n, n * 0.4]);
        let mut d = dust(107, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_68_70_41_44_arms() {
        // 68 (`:2004`): num74 = scale*0.3 cap 1, (0.1, 0.2, 1).
        let n = 0.3f32;
        assert_eq!(light(&dust(68, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.1, n * 0.2, n]);
        // 70 (`:2013`): (0.5, 0, 1).
        assert_eq!(light(&dust(70, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.5, 0.0, n]);
        // 41 (`:2041`): num76 = (scale+0.007)*0.7 cap 1, (0.4, 0.9, 1).
        let n = (1.0f32 + 0.007) * 0.7;
        assert_eq!(light(&dust(41, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.4, n * 0.9, n]);
        // 44 (`:2069`): num77 = (scale+0.0085)*0.7 cap 1, (0.7, 1, 0.8).
        let n = (1.0f32 + 0.0085) * 0.7;
        assert_eq!(light(&dust(44, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.7, n, n * 0.8]);
    }

    #[test]
    fn the_second_chain_reads_the_post_ramp_scale() {
        // fadeIn 0 (burn-out): scale 1 -> 0.99. num80 = 0.99 cap 1.
        // 130 (`:2139`): (1, 0.5, 0.4).
        let n = 1.0f32 - 0.01;
        assert_eq!(light(&dust(130, 1.0, Vec2::ZERO)).unwrap().2, [n, n * 0.5, n * 0.4]);
        // fadeIn 50 (ramp +0.03): scale 1 -> 1.03 -> cap 1.
        let mut d = dust(131, 1.0, Vec2::ZERO);
        d.fade_in = 50.0;
        assert_eq!(light(&d).unwrap().2, [0.4, 1.0, 0.6]);
        // 132/133 colours.
        let n = 1.0f32 - 0.01;
        assert_eq!(light(&dust(132, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.3, n * 0.5, n]);
        assert_eq!(light(&dust(133, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.9, n * 0.9, n * 0.3]);
        // 134 shares the arm, NO branch — dark.
        assert!(light(&dust(134, 1.0, Vec2::ZERO)).is_none());
    }

    #[test]
    fn the_278_331_colour_arms_and_gates() {
        // 278 (`:2182`): color/255 * num81 (post-ramp scale cap 1).
        let mut d = dust(278, 1.0, Vec2::ZERO);
        d.color = Color::new(200, 100, 50, 255);
        let n = 1.0f32 - 0.01;
        assert_eq!(
            light(&d).unwrap().2,
            [200.0 / 255.0 * n, 100.0 / 255.0 * n, 50.0 / 255.0 * n]
        );
        // 278 gates BOTH flags.
        d.no_light = true;
        assert!(light(&d).is_none());
        d.no_light = false;
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
        // 331 (`:2299`): color/255 * num84, gate !noLightEmittance only.
        let mut d = dust(331, 1.0, Vec2::ZERO);
        d.color = Color::new(255, 255, 255, 255);
        d.no_light = true; // noLight does NOT mute 331
        let n = 1.0f32 - 0.01;
        assert_eq!(light(&d).unwrap().2, [n, n, n]);
        d.no_light_emittance = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn the_219_222_226_272_arms_and_the_dark_223() {
        let n = 1.0f32 - 0.01; // post-ramp burn-out
        // 219-222 (`:2214-2226`), gate !noLight.
        assert_eq!(light(&dust(219, 1.0, Vec2::ZERO)).unwrap().2, [n, n * 0.5, n * 0.4]);
        assert_eq!(light(&dust(220, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.4, n, n * 0.6]);
        assert_eq!(light(&dust(221, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.3, n * 0.5, n]);
        assert_eq!(light(&dust(222, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.9, n * 0.9, n * 0.3]);
        // 223 shares the arm, NO branch — dark.
        assert!(light(&dust(223, 1.0, Vec2::ZERO)).is_none());
        // 226 (`:2254`): (0.2, 0.7, 1), gate !noLight. The customData Color
        // second light is absent (field dropped).
        assert_eq!(light(&dust(226, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.2, n * 0.7, n]);
        // 272 (`:2312`): (0.5, 0.2, 0.8), gate !noLight.
        assert_eq!(light(&dust(272, 1.0, Vec2::ZERO)).unwrap().2, [n * 0.5, n * 0.2, n * 0.8]);
        let mut d = dust(272, 1.0, Vec2::ZERO);
        d.no_light = true;
        assert!(light(&d).is_none());
    }

    #[test]
    fn an_unported_type_is_dark() {
        // FIRST-ladder types (the dust_lights.rs family) and physics-only types
        // emit nothing here.
        for ty in [6, 213, 260, 309, 268, 76, 270, 271, 0, 16, 304] {
            assert!(light(&dust(ty, 1.0, Vec2::ZERO)).is_none(), "type {ty}");
        }
    }

    #[test]
    fn the_feed_walks_slot_order_and_skips_dead_slots() {
        let mut pool = DustPool::new();
        *pool.slot_mut(2) = dust(159, 1.0, Vec2::new(32.0, 32.0));
        *pool.slot_mut(5) = dust(130, 1.0, Vec2::new(64.0, 64.0));
        let mut dead = dust(159, 1.0, Vec2::ZERO);
        dead.active = false;
        *pool.slot_mut(3) = dead;
        let mut lights = EntityLights::default();
        with_env(|env| feed_dust_chain_lights(&pool, &params(), env, &mut lights));
        lights.end_frame();
        let vis = lights.visible();
        assert_eq!(vis.len(), 2);
        assert_eq!((vis[0].x, vis[0].y), (2, 2), "slot 2 first");
        assert_eq!((vis[1].x, vis[1].y), (4, 4), "slot 5 second");
        // Deterministic: the same pool gives the same key.
        let mut again = EntityLights::default();
        with_env(|env| feed_dust_chain_lights(&pool, &params(), env, &mut again));
        again.end_frame();
        assert_eq!(again.key(), lights.key());
    }

    #[test]
    fn the_feed_leaves_the_update_stream_untouched() {
        // The RNG witness (the dust_lights.rs form): two identical pools, two
        // identical streams; feed the chain lights on ONE between updates and
        // compare the streams afterwards — the feed takes no RNG.
        let setup = || {
            let mut pool = DustPool::new();
            pool.child_safety.disabled = false;
            let mut d = dust(227, 1.0, Vec2::new(100.0, 100.0));
            d.first_frame = true;
            *pool.slot_mut(0) = d;
            *pool.slot_mut(1) = dust(159, 1.0, Vec2::new(100.0, 100.0));
            *pool.slot_mut(2) = dust(130, 1.0, Vec2::new(100.0, 100.0));
            pool
        };
        let ctx = UpdateCtx {
            max_dust_to_draw: 6000,
            screen_position: Vec2::ZERO,
            screen_height: 10000.0,
        };
        let mut a = setup();
        let mut b = setup();
        let mut ra = UnifiedRandom::new(42);
        let mut rb = UnifiedRandom::new(42);
        a.update_dust(&ctx, &mut ra, |_| false);
        b.update_dust(&ctx, &mut rb, |_| false);
        let mut lights = EntityLights::default();
        with_env(|env| feed_dust_chain_lights(&a, &params(), env, &mut lights));
        a.update_dust(&ctx, &mut ra, |_| false);
        b.update_dust(&ctx, &mut rb, |_| false);
        for _ in 0..32 {
            assert_eq!(ra.next_max(1_000_000), rb.next_max(1_000_000));
        }
        // The feed found the live 159 + 130 (the 227 was re-typed/killed).
        lights.end_frame();
        assert!(lights.visible().iter().any(|l| l.rgb == [1.0, 1.0, 0.1]));
    }

    /// SEAL census witness (research/308 §3, the 94/94 AddLight census):
    /// every SECOND-chain C# `Lighting.AddLight` line (`Dust.cs`, the 52-line
    /// register below from a fresh `Select-String` at the SEAL round) either
    /// emits from [`dust_chain_light`] or is the ONE cited drop (`:2275`,
    /// the 226 customData-Color second light — d54 G5 CORRECTED it LIVE via
    /// ParticleOrchestrator.cs:1171/1184/1199; the port's field-drop absence
    /// is a cited gap, not an uncounted site). WRITTEN NOT RUN.
    #[test]
    fn w4_g13_seal_the_52_line_census_covers_every_addlight_site() {
        // (C# line, representative type) pairs in file order — the module-doc
        // register, re-derived by Select-String at the SEAL round.
        let sites: [(i32, i32); 52] = [
            (1061, 159), (1099, 164), (1137, 173), (1208, 162), (1212, 160),
            (1241, 168), (1399, 244), (1441, 43), (1470, 15), (1474, 57),
            (1478, 58), (1501, 110), (1510, 111), (1519, 112), (1528, 113),
            (1537, 114), (1546, 311), (1555, 312), (1564, 313), (1588, 66),
            (1619, 267), (1635, 21), (1640, 231), (1644, 20), (1692, 27),
            (1706, 55), (1714, 73), (1722, 74), (1731, 56), (1743, 71),
            (1896, 35), (1926, 67), (1939, 185), (1950, 107), (1994, 35),
            (2004, 68), (2013, 70), (2041, 41), (2069, 44), (2139, 130),
            (2143, 131), (2147, 132), (2151, 133), (2182, 278), (2214, 219),
            (2218, 220), (2222, 221), (2226, 222), (2254, 226), (2275, 226),
            (2299, 331), (2312, 272),
        ];
        assert_eq!(sites.len(), 52, "the SECOND chain has exactly 52 sites");
        assert_eq!(42 + sites.len(), 94, "42 FIRST-ladder + 52 = the family");
        // Every site except the :2275 customData drop must emit with its
        // gates open (bright ambient, dry player at rest, wet probe true).
        let mut emitting = 0;
        for (line, ty) in sites {
            let mut d = dust(ty, 0.5, Vec2::new(32.0, 32.0));
            d.color = tr_core::particles::Color::new(255, 255, 255, 255);
            if line == 1896 {
                d.no_gravity = true; // the ng arm; :1994 rides the wet replay
            }
            if line == 2275 {
                // The ONE cited drop: the collector has no customData read,
                // so the 226 contributes ONLY its :2254 light (checked below).
                continue;
            }
            assert!(light(&d).is_some(), "site :{line} type {ty} must emit");
            emitting += 1;
        }
        assert_eq!(emitting, 51, "52 minus the :2275 customData drop");
        // The 226 emits ONLY the :2254 tint — the :2275 second light is
        // absent wholesale (no customData field, particles.rs; the d54 G5
        // reconstruction contract reads d.color at the merger).
        let d = dust(226, 0.5, Vec2::new(32.0, 32.0));
        let got = light(&d).unwrap();
        let n = 0.5f32 - 0.01; // the post-ramp burn-out read
        assert_eq!(got.2, [n * 0.2, n * 0.7, n], "the :2254 tint ONLY");
    }
}
