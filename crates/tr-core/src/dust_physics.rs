//! The per-type **physics** half of `Dust.UpdateDust` — the if-chain between
//! the `:477` move and the `:2343` screen-bottom kill (`Dust.cs:467-2334`).
//!
//! This is the sibling of [`crate::dust_table`] (the colour/fade half, S1-E-2)
//! and the body the [`crate::particles::DustPool::update_dust`] `type_arm` hook
//! dispatches (S1-E-3's skeleton). The pool owns the generic head/move/kills;
//! this module owns the per-type velocity / scale / alpha / rotation / active /
//! position / `noGravity` / `noLight` mutations, in the C#'s exact statement
//! order, with every `Main.rand` draw preserved on the caller's stream.
//!
//! # Scope (W4-d50, the dust foundation)
//!
//! The ROADMAP S1-E-2/E-3 remainder is "E-2's if-chain arms / light arms at
//! `:467-2334`". This module ports the **physics** column. The other columns
//! are not duplicated here:
//!
//! - **light** — every `Lighting.AddLight` in the chain is the S1-E-5 column
//!   (`tr-app::dust_lights`, which already feeds the 42 RNG-free FIRST-ladder
//!   arms). Each skipped `AddLight` is cited at its arm below so the light feed
//!   can reconcile timing. Where the C# computes a light from a POST-mutation
//!   scale (206 `:766`, 174 `:807`, 230 `:639`, 309 `:486`, 261 `:695-704`), the
//!   mutation lands HERE and the feed must read the post-update record (the
//!   feed's current local-decrement compensation for 206/174/230 is retired at
//!   the wiring wave — see ANALYSIS "The dust physics chain (W4-d50)").
//! - **`customData`** — dropped with the field (`particles.rs`'s `Dust` record):
//!   the Player/Projectile/NPC/Vector2/Color/int/float follower halves never
//!   fire. Each is cited; the 31 `else if (noGravity)` arm (`:1297`) becomes
//!   reachable BECAUSE the NPC `customData` branch is dropped (noted there).
//! - **`Main.snowDust`** (`:1747`) — a `Main` static (`Main.cs:621`), not a
//!   `Dust` one; the pool does not carry it (the skeleton's documented drop).
//!   The 76 arm's `snowDust++` is cited and skipped.
//!
//! # Wiring (the d51-54 / e1-e4 sites' wave)
//!
//! UNWIRED pure port (the `npc_ai_*` code-only precedent). The generic loop
//! calls [`pre_move`] BEFORE `position += velocity` (`Dust.cs:468-476` precedes
//! `:477`) and [`type_chain`] AFTER it (the `type_arm` hook, `:478-2334`); the
//! hook's `bool` is [`type_chain`]'s return — `true` exactly when a SECOND-chain
//! arm matched (`:2130-2334`), so the skeleton's final `else if (type != 304 &&
//! noGravity)` gravity tail (`:2335`) is suppressed, exactly as the C#'s else-if
//! structure does. The FIRST ladder's standalone arms (213/268 included) return
//! `false` and take the tail alongside, the corrected S1-E-3 chain reading.
//! Child `NewDust` spawns (159/164/173/160/162) are recorded in
//! [`DustPhysicsEnv::spawns`] and drained by the caller after the pass (the
//! `wants_*` sink precedent); the deferral is stream-neutral for every current
//! consumer (only 213/268 are live, neither spawns children) — see the census.
//!
//! # RNG census — every `Main.rand` draw in the chain, in per-slot C# order
//!
//! The chain rolls the caller's stream (the C#'s single `Main.rand`). One draw
//! = one stream step; no new streams. In source order, per active slot:
//!
//! | arm | C# | draws (in order) | gate |
//! | --- | --- | --- | --- |
//! | 235 | `:817` | `Next(-100,101)` ×2 | always (the `vector3` is DEAD — computed, never used; the draws are kept) |
//! | 159 noGravity | `:1068` | `Next(2)` | `scale >= 0.7` (the `< 0.7` branch draws nothing) |
//! | 159 gravity | `:1082-1084` | `Next(-10,11)` ×2, `Next(5)` | always; `Next(5)==0` → child `NewDust` (`:1086`) |
//! | 164 noGravity | `:1106` | `Next(2)` | `scale >= 0.7` |
//! | 164 gravity | `:1120-1122` | `Next(-10,11)` ×2, `Next(5)` | always; `==0` → child (`:1124`) |
//! | 173 noGravity | `:1141-1142` | `Next(-20,21)` ×2 | always |
//! | 173 gravity | `:1149-1151` | `Next(-10,11)` ×2, `Next(10)` | always; `==10` → child (`:1153`) |
//! | 160/162 noGravity | `:1217-1218` | `Next(-20,21)` ×2 | always |
//! | 160/162 gravity | `:1224-1226` | `Next(-10,11)` ×2, `Next(50)` | always; `scale>0.3 && ==0` → child (`:1228`) |
//! | 168 | `:1243-1244` | `Next(-10,11)` ×2 | always |
//! | 34/35/152 wet | `:1961`,`:1977` | `Next(2)`, `Next(-10,10)` | only the WET else-branch (dry sets `scale=0`, draws nothing) |
//! | 41 | `:2017-2018` | `Next(-10,11)` ×2 | always |
//! | 44 | `:2045-2046` | `Next(-10,11)` ×2 | always |
//!
//! Each child `NewDust` then rolls the spawn family's OWN draws (the pressure
//! bands + the default block, `particles.rs::new_dust`, `Dust.cs:103-193`) on
//! the SAME stream, in the C#'s mid-pass position. The skeleton's ChildSafety
//! head re-type (`:447-454`, `Next(2)`/`NextFloat`/`Next(3)`) precedes all of
//! these and is already ported in `update_dust`. No OTHER arm draws: the torch
//! family, the gem dusts, 304, 139-142, the bubble family's dry path, 27/45,
//! 76/268/270/271, the second chain and the generic tails are RNG-free.
//!
//! # Numeric conventions
//!
//! `MathHelper.Lerp(a,b,t) = a + (b-a)*t` (`:1752`,`:1802`). XNA
//! `Vector2.Normalize` divides by the length with NO zero guard (`:2097`); the
//! port mirrors that (`v / v.length()`), not glam's `normalize_or_zero`.
//! `rotation++`/`--` are `+= 1.0`/`-= 1.0` (the C# `float` field). The
//! `1.0050251f` / `0.0043982295f` literals are transcribed verbatim.

use crate::edit::solid_tile;
use crate::particles::{Color, Dust, DustRng};
use crate::world::World;
use glam::Vec2;

/// A child `NewDust` the chain requests mid-pass (159/164/173/160/162). The C#
/// calls `Dust.NewDust(pos, w, h, type)` (default Speed/Alpha/Color, Scale=1)
/// then writes `noGravity = true` and a new `scale` (`Dust.cs:1086-1088`,
/// `:1124-1126`, `:1153-1155`, `:1228-1230`). The caller drains the sink after
/// the pass: `new_dust(..., scale = 1.0)` then `slot.no_gravity = true;
/// slot.scale = spawn.scale` (the sentinel slot is written too, the C#'s shape).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DustChildSpawn {
    pub position: Vec2,
    pub width: i32,
    pub height: i32,
    pub dust_type: i32,
    /// The post-write `scale` (already folded: `dust.scale * 2.5` / `* 1.5` /
    /// `dust.scale`).
    pub scale: f32,
}

/// The per-tick inputs the chain reads as `Main`/`Collision`/`Lighting` statics.
/// The probes are injected (tr-core cannot depend on tr-app's `Player`
/// collision port — the same seam shape as the pool's `WetProbe`).
pub struct DustPhysicsEnv<'a> {
    /// `WorldGen.SolidTile(Framing.GetTileSafely(pos))` for the 278 arm
    /// (`:2197`); the indexed `solid_tile` port (`edit/kill.rs`).
    pub world: &'a World,
    /// `Sandstorm.ShowSandstormVisuals()` (`Dust.cs:430`, read by 268 `:1794`).
    pub show_sandstorm_visuals: bool,
    /// `Main.player[Main.myPlayer].velocity.X` (27/45 `:1681`,`:1685`).
    pub player_vx: f32,
    /// `Main.player[Main.myPlayer].velocity.Y` (76 `:1749`, 268 `:1799`).
    pub player_vy: f32,
    /// `Main.player[Main.myPlayer].wet` (27/45 `:1679`).
    pub player_wet: bool,
    /// `Main.player[num78].Center` for the 235 fadeIn homing (`:2091`). The
    /// port has ONE player: index 0 reads this; any other index reads the
    /// default-constructed `Player.Center` `(10, 21)` (width 20 / height 42 at
    /// the origin, `Player.cs` ctor) — single-player never spawns a 235 with
    /// another player's `fadeIn` encoding (the spawn sites use `myPlayer+1`).
    pub player_center: Vec2,
    /// `Collision.SolidCollision(position, width, height)` (`Collision.cs`).
    pub solid_collision: &'a dyn Fn(Vec2, f32, f32) -> bool,
    /// `Collision.WetCollision(position, width, height)` (`Collision.cs:1572`).
    pub wet_collision: &'a dyn Fn(Vec2, f32, f32) -> bool,
    /// `Lighting.GetColor(tileX, tileY)` as sRGB bytes (43 `:1405`, 244-247
    /// `:1337` — the latter only feeds the skipped light, so the probe is read
    /// solely by 43's death check). The real sample rides the light pass
    /// (S1-E-5); until that seam is wired the caller passes the bright default.
    pub light_color: &'a dyn Fn(i32, i32) -> Color,
    /// The child-spawn sink (the census's `NewDust` requests), drained after
    /// the pass.
    pub spawns: &'a mut Vec<DustChildSpawn>,
    /// `Dust.SandStormCount++` (268 `:1791`) accumulated as a delta; the caller
    /// adds it to the pool after the pass (the skeleton resets the counter to 0
    /// at the head, `:429`, so the post-pass add lands the exact count).
    pub sand_storm_count: &'a mut i32,
}

/// `MathHelper.Lerp(value1, value2, amount)` = `value1 + (value2 - value1) *
/// amount` (the 76/268 fall-lerp, `:1752`/`:1802`).
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// XNA `Vector2.Normalize` — divide by the length, NO zero guard (`:2097`).
#[inline]
fn xna_normalize(v: Vec2) -> Vec2 {
    v / v.length()
}

/// The pre-move per-type statements (`Dust.cs:468-476`), which the C# runs
/// BEFORE `position += velocity` (`:477`). The generic loop calls this ahead of
/// the move; `lava_bubbles` is `Dust.lavaBubbles` (`:475`, the 35 arm).
pub fn pre_move(d: &mut Dust, lava_bubbles: &mut i32) {
    // `:468-472` — the 299/300/301 (`(uint)(type-299) <= 2`) + 305 family.
    let num3 = d.dust_type;
    if ((num3 - 299) as u32) <= 2 || num3 == 305 {
        d.scale *= 0.96;
        d.velocity.y -= 0.01;
    }
    // `:473-476` — the lava-bubble counter.
    if d.dust_type == 35 {
        *lava_bubbles += 1;
    }
}

/// The per-type physics chain (`Dust.cs:478-2334`), post-move. Returns `true`
/// exactly when a SECOND-chain arm matched (`:2130-2334`), so the caller's
/// generic gravity tail (`:2335`, the skeleton's final `else if`) is suppressed.
/// Every `Lighting.AddLight` is the S1-E-5 column (cited, skipped); every
/// `customData` follower is dropped with the field (cited). RNG draws ride
/// `rng` in the census order.
pub fn type_chain<R: DustRng>(d: &mut Dust, rng: &mut R, env: &mut DustPhysicsEnv<'_>) -> bool {
    let ty = d.dust_type;

    // ---- FIRST ladder: the standalone ifs `:478-814` (post-move) ----

    // `:478-482` — type 258.
    if ty == 258 {
        d.no_gravity = true;
        d.scale += 0.015;
    }
    // `:483-487` — ShimmerSpark 309: AddLight (`:485`, S1-E-5) then the shrink.
    if ty == 309 {
        d.scale *= 0.97;
    }
    // `:488-512` — PinkPhaseblade 325: AddLight (`:497`) + the Player/Projectile
    // customData follower (`:499-511`) — both dropped (light = S1-E-5, field gone).
    // `:513-558` — the gem dusts 86-92/286: AddLight (`:557`) only — S1-E-5.
    // `:559-574` — the gem dusts' Player/Projectile customData follower — dropped.
    // `:575-578` — AmberBolt 262: AddLight (`:577`) only — S1-E-5.
    // `:579-586` — type 240's Projectile customData follower — dropped.
    // `:587-590` — type 329: the solid-contact shrink.
    if ty == 329 && (env.solid_collision)(d.position, 4.0, 4.0) {
        d.scale *= 0.8;
    }
    // `:591-611` — 259/6/158/135's `customData is int` arm — dropped (field gone).
    // `:612-629` — PortalBolt/Trail 263/264: AddLight (`:616`) + the
    // Player/Projectile customData follower (`:618-628`) — dropped.
    // `:630-650` — type 230: AddLight (`:640`) reads the PRE-increment scale,
    // then `scale += 0.02f` (`:639`); the Player customData homing (`:641-649`)
    // is dropped. The light feed must recover the pre-increment scale.
    if ty == 230 {
        d.scale += 0.02;
    }
    // `:651-655` — 154/218: the spin + shrink.
    if ty == 154 || ty == 218 {
        d.rotation += d.velocity.x * 0.3;
        d.scale -= 0.03;
    }
    // `:656-664` — type 172: AddLight (`:663`) only — S1-E-5.
    // `:665-683` — type 182: `rotation++` (`:667`), AddLight (`:675`), the
    // Player customData follower (`:677-682`) dropped.
    if ty == 182 {
        d.rotation += 1.0;
    }
    // `:684-710` — AncientLight 261: AddLight (`:693`), the noGravity drift,
    // the per-axis damp + shrink, the Player customData follower (`:705-709`)
    // dropped.
    if ty == 261 {
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        }
        d.velocity *= Vec2::new(0.97, 0.99);
        d.scale -= 0.0025;
    }
    // `:711-719` — CrystalPulse 254: AddLight (`:718`) only — S1-E-5.
    // `:720-728` — CrystalPulse2 255: AddLight (`:727`) only — S1-E-5.
    // `:729-732` — type 211: the noLight solid-contact kill.
    if ty == 211 && d.no_light && (env.solid_collision)(d.position, 4.0, 4.0) {
        d.active = false;
    }
    // `:733-736` — type 284: the solid-contact damp (fadeIn-gated).
    if ty == 284
        && (env.solid_collision)(d.position - Vec2::ONE * 4.0, 8.0, 8.0)
        && d.fade_in == 0.0
    {
        d.velocity *= 0.25;
    }
    // `:737-758` — the Sparks 213/260: `rotation = 0f` (`:739`) + AddLight
    // (`:757`, S1-E-5). The rotation rebuild at `:2081` follows.
    if ty == 213 || ty == 260 {
        d.rotation = 0.0;
    }
    // `:759-763` — Chlorophyte 157: AddLight (`:762`) only — S1-E-5.
    // `:764-769` — type 206: `scale -= 0.1f` (`:766`) THEN AddLight (`:768`)
    // from the new scale — the feed reads the post-decrement record.
    if ty == 206 {
        d.scale -= 0.1;
    }
    // `:770-774` — type 163: AddLight (`:773`) only — S1-E-5.
    // `:775-779` — type 205: AddLight (`:778`) only — S1-E-5.
    // `:780-784` — IchorSplash 170: AddLight (`:783`) only — S1-E-5.
    // `:785-794` — UltraBrightTorch 156: AddLight (`:793`) only — S1-E-5.
    // `:795-800` — BoneTorch 234: AddLight (`:799`) only — S1-E-5.
    // `:801-804` — type 175: the shrink.
    if ty == 175 {
        d.scale -= 0.05;
    }
    // `:805-814` — InfernoFlame 174: `scale -= 0.01f` (`:807`) THEN AddLight
    // (`:813`) from the new scale — the feed reads the post-decrement record.
    if ty == 174 {
        d.scale -= 0.01;
    }

    // ---- FIRST ladder: the big else-if chain `:815-1829` ----

    if ty == 235 {
        // `:817-819` — `vector3` is computed (two Next(-100,101), Normalize,
        // *15) and NEVER used — dead. The two draws are kept (stream fidelity).
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_range(-100, 101);
        // `:820`.
        d.scale -= 0.01;
    } else if matches!(
        ty,
        228 | 279
            | 229
            | 6
            | 242
            | 135
            | 127
            | 187
            | 75
            | 169
            | 29
            | 59..=65
            | 158
            | 293
            | 294
            | 295
            | 296
            | 297
            | 298
            | 302
            | 307
            | 310
    ) {
        // `:822-1023` — the torch/fire family. `:824-827` the gravity half.
        if !d.no_gravity {
            d.velocity.y += 0.05;
        }
        // `:828-849` — the 229/228/279 NPC/Player/Vector2 customData follower —
        // dropped (field gone).
        // `:850-1022` — the family AddLight chain (the per-type caps + torch
        // ids) — S1-E-5 (`dust_lights::torch_family_light`).
    } else if ty == 306 {
        // `:1024-1036`.
        if !d.no_gravity {
            d.velocity.y += 0.05;
        }
        d.scale -= 0.04;
        if (env.solid_collision)(d.position - Vec2::ONE * 5.0, 10.0, 10.0) && d.fade_in == 0.0 {
            d.scale *= 0.9;
            d.velocity *= 0.25;
        }
    } else if ty == 269 {
        // `:1037-1053` — Sandnado: AddLight (`:1046`, S1-E-5) + the Vector2
        // customData velocity half (`:1048-1052`) — dropped.
    } else if ty == 159 {
        // `:1054-1091` — AddLight (`:1061`, S1-E-5) then the motion split.
        if d.no_gravity {
            if d.scale < 0.7 {
                d.velocity *= 1.075;
            } else if rng.next_max(2) == 0 {
                d.velocity *= -0.95;
            } else {
                d.velocity *= 1.05;
            }
            d.scale -= 0.03;
        } else {
            d.scale += 0.005;
            d.velocity *= 0.9;
            d.velocity.x += rng.next_range(-10, 11) as f32 * 0.02;
            d.velocity.y += rng.next_range(-10, 11) as f32 * 0.02;
            if rng.next_max(5) == 0 {
                // `:1086-1088` — the child NewDust (noGravity, scale*2.5).
                env.spawns.push(DustChildSpawn {
                    position: d.position,
                    width: 4,
                    height: 4,
                    dust_type: ty,
                    scale: d.scale * 2.5,
                });
            }
        }
    } else if ty == 164 {
        // `:1092-1129` — AddLight (`:1099`, S1-E-5); the 159 shape with the
        // gravity scale sign flipped (`-= 0.005f`, `:1118`).
        if d.no_gravity {
            if d.scale < 0.7 {
                d.velocity *= 1.075;
            } else if rng.next_max(2) == 0 {
                d.velocity *= -0.95;
            } else {
                d.velocity *= 1.05;
            }
            d.scale -= 0.03;
        } else {
            d.scale -= 0.005;
            d.velocity *= 0.9;
            d.velocity.x += rng.next_range(-10, 11) as f32 * 0.02;
            d.velocity.y += rng.next_range(-10, 11) as f32 * 0.02;
            if rng.next_max(5) == 0 {
                // `:1124-1126`.
                env.spawns.push(DustChildSpawn {
                    position: d.position,
                    width: 4,
                    height: 4,
                    dust_type: ty,
                    scale: d.scale * 2.5,
                });
            }
        }
    } else if ty == 173 {
        // `:1130-1158` — AddLight (`:1137`, S1-E-5).
        if d.no_gravity {
            d.velocity *= 0.8;
            d.velocity.x += rng.next_range(-20, 21) as f32 * 0.01;
            d.velocity.y += rng.next_range(-20, 21) as f32 * 0.01;
            d.scale -= 0.01;
        } else {
            d.scale -= 0.015;
            d.velocity *= 0.8;
            d.velocity.x += rng.next_range(-10, 11) as f32 * 0.005;
            d.velocity.y += rng.next_range(-10, 11) as f32 * 0.005;
            if rng.next_max(10) == 10 {
                // `:1153-1155` — the child carries the parent's scale (no fold).
                env.spawns.push(DustChildSpawn {
                    position: d.position,
                    width: 4,
                    height: 4,
                    dust_type: ty,
                    scale: d.scale,
                });
            }
        }
    } else if ty == 304 {
        // `:1159-1190` — the shimmer sine + the alpha ladder; the Player
        // customData homing (`:1176-1189`) is dropped.
        d.velocity.y = d.rotation.sin() / 5.0;
        d.rotation += 0.015;
        if d.scale < 1.15 {
            d.alpha = (d.alpha - 20).max(0);
            d.scale += 0.0015;
        } else {
            d.alpha += 6;
            if d.alpha >= 255 {
                d.active = false;
            }
        }
    } else if ty == 184 {
        // `:1191-1198`.
        if !d.no_gravity {
            d.velocity *= 0.0;
            d.scale -= 0.01;
        }
    } else if ty == 160 || ty == 162 {
        // `:1199-1233` — AddLight (`:1208`/`:1212`, S1-E-5).
        if d.no_gravity {
            d.velocity *= 0.8;
            d.velocity.x += rng.next_range(-20, 21) as f32 * 0.04;
            d.velocity.y += rng.next_range(-20, 21) as f32 * 0.04;
            d.scale -= 0.1;
        } else {
            d.scale -= 0.1;
            d.velocity.x += rng.next_range(-10, 11) as f32 * 0.02;
            d.velocity.y += rng.next_range(-10, 11) as f32 * 0.02;
            if d.scale as f64 > 0.3 && rng.next_max(50) == 0 {
                // `:1228-1230` — the child at pos-4, 1x1, scale*1.5.
                env.spawns.push(DustChildSpawn {
                    position: Vec2::new(d.position.x - 4.0, d.position.y - 4.0),
                    width: 1,
                    height: 1,
                    dust_type: ty,
                    scale: d.scale * 1.5,
                });
            }
        }
    } else if ty == 168 {
        // `:1234-1246` — AddLight (`:1241`, S1-E-5).
        d.scale += 0.03;
        d.velocity.x += rng.next_range(-10, 11) as f32 * 0.02;
        d.velocity.y += rng.next_range(-10, 11) as f32 * 0.02;
        d.velocity *= 0.99;
    } else if (139..143).contains(&ty) {
        // `:1247-1265`.
        d.velocity.x *= 0.98;
        d.velocity.y *= 0.98;
        if d.velocity.y < 1.0 {
            d.velocity.y += 0.05;
        }
        d.scale += 0.009;
        d.rotation -= d.velocity.x * 0.4;
        if d.velocity.x > 0.0 {
            d.rotation += 0.005;
        } else {
            d.rotation -= 0.005;
        }
    } else if matches!(ty, 326 | 327 | 328 | 14 | 16 | 31 | 46 | 124 | 186 | 188 | 303) {
        // `:1266-1324`.
        d.velocity.y *= 0.98;
        d.velocity.x *= 0.98;
        if ty == 31 {
            // `:1272-1276` the float customData velocity add — dropped.
            // `:1277-1296` the NPC customData follower — dropped (field gone),
            // so the `else if (noGravity)` (`:1297`) IS reachable.
            if d.no_gravity {
                d.velocity *= 1.02;
                d.scale += 0.02;
                d.alpha += 4;
                if d.alpha > 255 {
                    d.scale = 0.0001;
                    d.alpha = 255;
                }
            }
        }
        if ty == 303 && d.no_gravity {
            // `:1309-1323`.
            d.velocity *= 1.02;
            d.scale += 0.03;
            if d.alpha < 90 {
                d.alpha = 90;
            }
            d.alpha += 4;
            if d.alpha > 255 {
                d.scale = 0.0001;
                d.alpha = 255;
            }
        }
    } else if ty == 32 {
        // `:1325-1333`.
        d.scale -= 0.01;
        d.velocity.x *= 0.96;
        if !d.no_gravity {
            d.velocity.y += 0.1;
        }
    } else if (244..=247).contains(&ty) {
        // `:1334-1401` — the spin + the alpha/scale ramp; the GetColor-fed
        // AddLight (`:1337`,`:1399`) is S1-E-5 (the colour multipliers feed only
        // the light, so no physics reads them).
        d.rotation += 0.1 * d.scale;
        if d.alpha < 255 {
            d.scale += 0.09;
            if d.scale >= 1.0 {
                d.scale = 1.0;
                d.alpha = 255;
            }
        } else {
            if (d.scale as f64) < 0.8 {
                d.scale -= 0.01;
            }
            if (d.scale as f64) < 0.5 {
                d.scale -= 0.01;
            }
        }
    } else if ty == 43 {
        // `:1402-1448` — the spin, the GetColor-fed light terms, the ramp, the
        // death check, the AddLight (`:1441`, S1-E-5), the Player customData
        // follower (`:1443`) dropped.
        d.rotation += 0.1 * d.scale;
        // `:1405-1414` — num48/49/50 = light/270 * scale*1.07 * dust.color/255,
        // computed from the PRE-ramp scale (the C# reads them before `:1415`).
        let c = (env.light_color)(
            (d.position.x / 16.0) as i32,
            (d.position.y / 16.0) as i32,
        );
        let num48 = c.r as f32 / 270.0 * d.scale * 1.07 * (d.color.r as f32 / 255.0);
        let num49 = c.g as f32 / 270.0 * d.scale * 1.07 * (d.color.g as f32 / 255.0);
        let num50 = c.b as f32 / 270.0 * d.scale * 1.07 * (d.color.b as f32 / 255.0);
        // `:1415-1434` — the alpha/scale ramp.
        if d.alpha < 255 {
            d.scale += 0.09;
            if d.scale >= 1.0 {
                d.scale = 1.0;
                d.alpha = 255;
            }
        } else {
            if (d.scale as f64) < 0.8 {
                d.scale -= 0.01;
            }
            if (d.scale as f64) < 0.5 {
                d.scale -= 0.01;
            }
        }
        // `:1435-1438` — the full-dark kill reads the PRE-ramp terms (the else
        // AddLight is S1-E-5).
        if (num48 as f64) < 0.05 && (num49 as f64) < 0.05 && (num50 as f64) < 0.05 {
            d.active = false;
        }
    } else if matches!(ty, 15 | 57 | 58 | 274 | 292) {
        // `:1449-1481` — the per-axis damp; inside `!noLightEmittance` the
        // noLight damp (`:1460-1463`) then AddLight (`:1470`/`:1474`/`:1478`,
        // S1-E-5).
        d.velocity.y *= 0.98;
        d.velocity.x *= 0.98;
        if !d.no_light_emittance && d.no_light {
            d.velocity *= 0.95;
        }
    } else if ty == 204 {
        // `:1482-1493`.
        if d.fade_in > d.scale {
            d.scale += 0.02;
        } else {
            d.scale -= 0.02;
        }
        d.velocity *= 0.95;
    } else if matches!(ty, 110 | 111 | 112 | 113 | 114 | 311 | 312 | 313) {
        // `:1494-1565` — AddLight only (the per-type torch/colour arms) — S1-E-5.
    } else if ty == 66 {
        // `:1566-1589` — the spin, the damp, the grow; AddLight (`:1588`, S1-E-5).
        if d.velocity.x < 0.0 {
            d.rotation -= 1.0;
        } else {
            d.rotation += 1.0;
        }
        d.velocity.y *= 0.98;
        d.velocity.x *= 0.98;
        d.scale += 0.02;
    } else if ty == 267 {
        // `:1590-1621` — the spin, the Player customData follower (`:1600-1604`)
        // dropped, the damp/grow, the `noLight = false` lift (`:1613-1616`),
        // AddLight (`:1619`, S1-E-5).
        if d.velocity.x < 0.0 {
            d.rotation -= 1.0;
        } else {
            d.rotation += 1.0;
        }
        d.velocity.y *= 0.98;
        d.velocity.x *= 0.98;
        d.scale += 0.02;
        if d.no_light {
            d.no_light = false;
        }
    } else if ty == 20 || ty == 21 || ty == 231 {
        // `:1622-1646` — the grow + damp; AddLight (`:1635`/`:1640`/`:1644`,
        // S1-E-5).
        d.scale += 0.005;
        d.velocity.y *= 0.94;
        d.velocity.x *= 0.94;
    } else if ty == 27 || ty == 45 {
        // `:1647-1693` — the 27 fadeIn burn-down, the damp/grow, the noLight
        // player-push, AddLight (`:1692`, S1-E-5).
        if ty == 27 && d.fade_in >= 100.0 {
            if (d.scale as f64) >= 1.5 {
                d.scale -= 0.01;
            } else {
                d.scale -= 0.05;
            }
            if (d.scale as f64) <= 0.5 {
                d.scale -= 0.05;
            }
            if (d.scale as f64) <= 0.25 {
                d.scale -= 0.05;
            }
        }
        d.velocity *= 0.94;
        d.scale += 0.002;
        if d.no_light {
            // `:1673-1674` num66 *= 0.1 feeds only the light (S1-E-5).
            d.scale -= 0.06;
            if d.scale < 1.0 {
                d.scale -= 0.06;
            }
            // `:1679-1686` — the wet half-push vs the full push.
            if env.player_wet {
                d.position += Vec2::new(env.player_vx, env.player_vy) * 0.5;
            } else {
                d.position += Vec2::new(env.player_vx, env.player_vy);
            }
        }
    } else if matches!(ty, 55 | 56 | 73 | 74) {
        // `:1694-1734` — the damp; AddLight (`:1706`/`:1714`/`:1722`/`:1731`,
        // S1-E-5).
        d.velocity *= 0.98;
    } else if ty == 71 || ty == 72 {
        // `:1735-1744` — the damp; AddLight (`:1743`, S1-E-5).
        d.velocity *= 0.98;
    } else if ty == 76 {
        // `:1745-1763` — `Main.snowDust++` (`:1747`) is a Main static, dropped
        // (module docs). The grow, the player-fall lerp + push, the solid damp.
        d.scale += 0.009;
        let y = env.player_vy;
        if y > 0.0 && d.fade_in == 0.0 && d.velocity.y < y {
            d.velocity.y = lerp(d.velocity.y, y, 0.04);
        }
        if !d.no_light && y > 0.0 {
            d.position.y += env.player_vy * 0.2;
        }
        if (env.solid_collision)(d.position - Vec2::ONE * 5.0, 10.0, 10.0) && d.fade_in == 0.0 {
            d.scale *= 0.9;
            d.velocity *= 0.25;
        }
    } else if ty == 270 {
        // `:1764-1780`.
        d.velocity *= 1.0050251;
        d.scale += 0.01;
        d.rotation = 0.0;
        if (env.solid_collision)(d.position - Vec2::ONE * 5.0, 10.0, 10.0) && d.fade_in == 0.0 {
            d.scale *= 0.95;
            d.velocity *= 0.25;
        } else {
            d.velocity.y = (d.position.x * 0.0043982295).sin() * 2.0;
            d.velocity.y -= 3.0;
            d.velocity.y /= 20.0;
        }
    } else if ty == 271 {
        // `:1781-1788`.
        d.velocity *= 1.0050251;
        d.scale += 0.003;
        d.rotation = 0.0;
        d.velocity.y -= 4.0;
        d.velocity.y /= 6.0;
    } else if ty == 268 {
        // `:1789-1818` — the flying sand. `SandStormCount++` (`:1791`).
        *env.sand_storm_count += 1;
        d.velocity *= 1.0050251;
        d.scale += 0.01;
        if !env.show_sandstorm_visuals {
            d.scale -= 0.05;
        }
        d.rotation = 0.0;
        let y2 = env.player_vy;
        if y2 > 0.0 && d.fade_in == 0.0 && d.velocity.y < y2 {
            d.velocity.y = lerp(d.velocity.y, y2, 0.04);
        }
        if !d.no_light && y2 > 0.0 {
            d.position.y += y2 * 0.2;
        }
        if (env.solid_collision)(d.position - Vec2::ONE * 5.0, 10.0, 10.0) && d.fade_in == 0.0 {
            d.scale *= 0.9;
            d.velocity *= 0.25;
        } else {
            d.velocity.y = (d.position.x * 0.0043982295).sin() * 2.0;
            d.velocity.y += 3.0;
        }
    } else if !d.no_gravity && ty != 41 && ty != 44 && ty != 309 {
        // `:1819-1829` — the generic gravity fall (the chain's final else-if).
        if ty == 107 {
            d.velocity *= 0.9;
        } else {
            d.velocity.y += 0.1;
        }
    }

    // ---- Standalone ifs `:1830-1996` ----

    // `:1830-1833` — the blood family 5/273 noGravity shrink.
    if (ty == 5 || ty == 273) && d.no_gravity {
        d.scale -= 0.04;
    }
    // `:1834-1874` — the bubble/drip family.
    if matches!(
        ty,
        308 | 33 | 52 | 266 | 98 | 99 | 100 | 101 | 102 | 103 | 104 | 105 | 123 | 288
    ) {
        if d.velocity.x == 0.0 {
            if (env.solid_collision)(d.position, 2.0, 2.0) {
                d.scale = 0.0;
            }
            d.rotation += 0.5;
            d.scale -= 0.01;
        }
        if (env.wet_collision)(Vec2::new(d.position.x, d.position.y), 4.0, 4.0) {
            d.alpha += 20;
            d.scale -= 0.1;
        }
        d.alpha += 2;
        d.scale -= 0.005;
        if d.alpha > 255 {
            d.scale = 0.0;
        }
        if d.velocity.y > 4.0 {
            d.velocity.y = 4.0;
        }
        if d.no_gravity {
            if d.velocity.x < 0.0 {
                d.rotation -= 0.2;
            } else {
                d.rotation += 0.2;
            }
            d.scale += 0.03;
            d.velocity.x *= 1.05;
            d.velocity.y += 0.15;
        }
    }
    // `:1875-1996` — the else-if sub-chain (35/152 noGravity, then the light
    // arms, then the 34/35/152 wet arm).
    if ty == 35 && d.no_gravity {
        // `:1875-1897` — AddLight (`:1896`, S1-E-5).
        d.scale += 0.03;
        if d.scale < 1.0 {
            d.velocity.y += 0.075;
        }
        d.velocity.x *= 1.08;
        if d.velocity.x > 0.0 {
            d.rotation += 0.01;
        } else {
            d.rotation -= 0.01;
        }
    } else if ty == 152 && d.no_gravity {
        // `:1898-1914`.
        d.scale += 0.03;
        if d.scale < 1.0 {
            d.velocity.y += 0.075;
        }
        d.velocity.x *= 1.08;
        if d.velocity.x > 0.0 {
            d.rotation += 0.01;
        } else {
            d.rotation -= 0.01;
        }
    } else if ty == 67 || ty == 92 {
        // `:1915-1927` — AddLight (`:1926`, S1-E-5) only.
    } else if ty == 185 {
        // `:1928-1940` — AddLight (`:1939`, S1-E-5) only.
    } else if ty == 107 {
        // `:1941-1952` — AddLight (`:1950`, S1-E-5) only.
    } else if ty == 34 || ty == 35 || ty == 152 {
        // `:1953-1996` — the wet bubble arm.
        if !(env.wet_collision)(Vec2::new(d.position.x, d.position.y - 8.0), 4.0, 4.0) {
            d.scale = 0.0;
        } else {
            d.alpha += rng.next_max(2);
            if d.alpha > 255 {
                d.scale = 0.0;
            }
            d.velocity.y = -0.5;
            if ty == 34 {
                d.scale += 0.005;
            } else {
                d.alpha += 1;
                d.scale -= 0.01;
                d.velocity.y = -0.2;
            }
            d.velocity.x += rng.next_range(-10, 10) as f32 * 0.002;
            if (d.velocity.x as f64) < -0.25 {
                d.velocity.x = -0.25;
            }
            if (d.velocity.x as f64) > 0.25 {
                d.velocity.x = 0.25;
            }
        }
        // `:1987-1995` — the 35 AddLight (`:1994`, S1-E-5).
    }

    // ---- Standalone ifs `:1997-2074` ----

    // `:1997-2005` — type 68: AddLight (`:2004`, S1-E-5) only.
    // `:2006-2014` — type 70: AddLight (`:2013`, S1-E-5) only.
    // `:2015-2074` — the 41/44/else else-if chain.
    if ty == 41 {
        // `:2015-2042` — the jitter + clamp + grow; AddLight (`:2041`, S1-E-5).
        d.velocity.x += rng.next_range(-10, 11) as f32 * 0.01;
        d.velocity.y += rng.next_range(-10, 11) as f32 * 0.01;
        if (d.velocity.x as f64) > 0.75 {
            d.velocity.x = 0.75;
        }
        if (d.velocity.x as f64) < -0.75 {
            d.velocity.x = -0.75;
        }
        if (d.velocity.y as f64) > 0.75 {
            d.velocity.y = 0.75;
        }
        if (d.velocity.y as f64) < -0.75 {
            d.velocity.y = -0.75;
        }
        d.scale += 0.007;
    } else if ty == 44 {
        // `:2043-2070` — the jitter + clamp + grow; AddLight (`:2069`, S1-E-5).
        d.velocity.x += rng.next_range(-10, 11) as f32 * 0.003;
        d.velocity.y += rng.next_range(-10, 11) as f32 * 0.003;
        if (d.velocity.x as f64) > 0.35 {
            d.velocity.x = 0.35;
        }
        if (d.velocity.x as f64) < -0.35 {
            d.velocity.x = -0.35;
        }
        if (d.velocity.y as f64) > 0.35 {
            d.velocity.y = 0.35;
        }
        if (d.velocity.y as f64) < -0.35 {
            d.velocity.y = -0.35;
        }
        d.scale += 0.0085;
    } else if ty != 304 {
        // `:2071-2074` — the generic air drag.
        d.velocity.x *= 0.99;
    }

    // ---- `:2075-2129` ----

    // `:2075-2078` — type 322 gravity shrink.
    if ty == 322 && !d.no_gravity {
        d.scale *= 0.98;
    }
    // `:2079-2082` — the generic rotation rebuild (79/268/304 exempt).
    if ty != 79 && ty != 268 && ty != 304 {
        d.rotation += d.velocity.x * 0.5;
    }
    // `:2083-2129` — the fadeIn ramp.
    if d.fade_in > 0.0 && d.fade_in < 100.0 {
        if ty == 235 {
            // `:2085-2101` — the homing ramp.
            d.scale += 0.007;
            let num78 = d.fade_in as i32 - 1;
            if (0..=255).contains(&num78) {
                let center = if num78 == 0 {
                    env.player_center
                } else {
                    // The default-constructed Player.Center (module docs).
                    Vec2::new(10.0, 21.0)
                };
                let mut vector6 = d.position - center;
                let num79 = 100.0 - vector6.length();
                if num79 > 0.0 {
                    d.scale -= num79 * 0.0015;
                }
                vector6 = xna_normalize(vector6);
                vector6 *= 0.0 - (1.0 - d.scale) * 20.0;
                d.velocity = (d.velocity * 4.0 + vector6) / 5.0;
            }
        } else if ty == 46 {
            d.scale += 0.1;
        } else if ty == 213 || ty == 260 {
            d.scale += 0.1;
        } else {
            d.scale += 0.03;
        }
        if d.scale > d.fade_in {
            d.fade_in = 0.0;
        }
    } else if ty != 304 {
        // `:2119-2129` — the burn-out tail.
        if ty == 213 || ty == 260 {
            d.scale -= 0.2;
        } else {
            d.scale -= 0.01;
        }
    }

    // ---- SECOND chain `:2130-2334` (the matched arms suppress the tail) ----

    if (130..=134).contains(&ty) {
        // `:2130-2172` — AddLight (`:2139`/`:2143`/`:2147`/`:2151`, S1-E-5).
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        } else if ty == 131 {
            d.velocity *= 0.98;
            d.velocity.y -= 0.1;
            d.scale += 0.0025;
        } else {
            d.velocity *= 0.95;
            d.scale -= 0.0025;
        }
        true
    } else if ty == 278 {
        // `:2173-2202` — AddLight (`:2182`, S1-E-5).
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        } else {
            d.velocity *= 0.95;
            d.scale -= 0.0025;
        }
        // `:2197-2201` — WorldGen.SolidTile(Framing.GetTileSafely(pos)).
        if solid_tile(env.world, (d.position.x / 16.0) as i32, (d.position.y / 16.0) as i32)
            && d.fade_in == 0.0
            && !d.no_gravity
        {
            d.scale *= 0.9;
            d.velocity *= 0.25;
        }
        true
    } else if (219..=223).contains(&ty) {
        // `:2203-2244` — AddLight (`:2214`/`:2218`/`:2222`/`:2226`, S1-E-5);
        // the Player customData follower (`:2239-2243`) dropped.
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        }
        d.velocity *= Vec2::new(0.97, 0.99);
        d.scale -= 0.0025;
        true
    } else if ty == 226 {
        // `:2245-2279` — AddLight (`:2254`, S1-E-5); the Player (`:2265-2269`)
        // and Color (`:2270-2277`) customData halves dropped.
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        }
        d.velocity *= Vec2::new(0.97, 0.99);
        d.scale -= 0.01;
        true
    } else if ty == 331 {
        // `:2280-2302` — AddLight (`:2299`, S1-E-5).
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        }
        d.velocity *= Vec2::new(0.97, 0.99);
        d.scale -= 0.01;
        true
    } else if ty == 272 {
        // `:2303-2334` — AddLight (`:2312`, S1-E-5); the Player (`:2323-2327`)
        // and NPC (`:2328-2332`) customData followers dropped.
        if d.no_gravity {
            d.velocity *= 0.93;
            if d.fade_in == 0.0 {
                d.scale += 0.0025;
            }
        }
        d.velocity *= Vec2::new(0.97, 0.99);
        d.scale -= 0.01;
        true
    } else {
        // `:2335-2342` — the final `else if (type != 304 && noGravity)` gravity
        // tail is the SKELETON's (the hook returns false → the caller applies
        // it). No arm matched here.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particles::{DustPool, SpawnCtx, UpdateCtx};
    use crate::tile::{FLAG_ACTIVE, Tile};
    use crate::UnifiedRandom;

    /// A scripted `DustRng`: replays a fixed draw script and records every call
    /// (kind + args) so the census order is pinned exactly. `next_float` pulls
    /// the next script entry too (the ChildSafety head's `NextFloat`).
    struct ScriptRng {
        script: std::collections::VecDeque<i32>,
        calls: Vec<String>,
    }

    impl ScriptRng {
        fn new(script: &[i32]) -> Self {
            Self {
                script: script.iter().copied().collect(),
                calls: Vec::new(),
            }
        }
        fn pull(&mut self, kind: &str) -> i32 {
            let v = self.script.pop_front().expect("script exhausted");
            self.calls.push(format!("{kind}={v}"));
            v
        }
    }

    impl DustRng for ScriptRng {
        fn next_max(&mut self, max: i32) -> i32 {
            let v = self.pull(&format!("max{max}"));
            v.rem_euclid(max.max(1))
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            let v = self.pull(&format!("rng{min}..{max}"));
            min + v.rem_euclid((max - min).max(1))
        }
        fn next_float(&mut self) -> f32 {
            self.pull("float") as f32 / 255.0
        }
    }

    /// An empty world (no tiles) — the collision probes are injected, so the
    /// world only matters for the 278 `solid_tile` arm.
    fn empty_world() -> World {
        World::new("dust".into(), 16, 16)
    }

    /// A world with one solid stone tile at (5, 5) for the 278 arm.
    fn solid_world() -> World {
        let mut w = World::new("dust".into(), 16, 16);
        let i = w.index(5, 5);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    /// A dust record with the given type/scale at the origin, active.
    fn dust(ty: i32, scale: f32) -> Dust {
        Dust {
            active: true,
            dust_type: ty,
            scale,
            position: Vec2::new(80.0, 80.0),
            ..Default::default()
        }
    }

    /// Run `type_chain` with default (dry, non-solid, bright) probes and a
    /// counting stream; returns the matched flag, the draws and the spawns.
    fn run(
        d: &mut Dust,
        rng: &mut impl DustRng,
    ) -> (bool, Vec<DustChildSpawn>, i32) {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mut env = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        let matched = type_chain(d, rng, &mut env);
        (matched, spawns, sand)
    }

    // ---- pre-move (`:468-476`) ----

    #[test]
    fn w4_d50_pre_move_the_299_family_and_the_lava_counter() {
        let mut lava = 0;
        for ty in [299, 300, 301, 305] {
            let mut d = dust(ty, 1.0);
            d.velocity = Vec2::new(2.0, 3.0);
            pre_move(&mut d, &mut lava);
            assert_eq!(d.scale, 0.96, "type {ty} shrinks");
            assert_eq!(d.velocity.y, 3.0 - 0.01, "type {ty} lifts");
        }
        // 302/304 are NOT in the `(uint)(type-299) <= 2` band nor 305.
        for ty in [298, 302, 304] {
            let mut d = dust(ty, 1.0);
            pre_move(&mut d, &mut lava);
            assert_eq!(d.scale, 1.0, "type {ty} untouched");
        }
        let mut d = dust(35, 1.0);
        pre_move(&mut d, &mut lava);
        assert_eq!(lava, 1, "the 35 arm counts a lava bubble");
    }

    // ---- the standalone FIRST-ladder physics ----

    #[test]
    fn w4_d50_the_standalone_arms_mutate_in_source_order() {
        let mut rng = UnifiedRandom::new(7);
        // 258 (`:478`): noGravity lift + grow.
        let mut d = dust(258, 1.0);
        run(&mut d, &mut rng);
        assert!(d.no_gravity);
        assert_eq!(d.scale, 1.0 + 0.015);
        // 309 (`:483`): the shrink (light skipped).
        let mut d = dust(309, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.97);
        // 230 (`:630`): the grow AFTER the (skipped) light.
        let mut d = dust(230, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 1.0 + 0.02);
        // 154/218 (`:651`): spin + shrink.
        let mut d = dust(154, 1.0);
        d.velocity = Vec2::new(2.0, 0.0);
        run(&mut d, &mut rng);
        assert_eq!(d.rotation, 2.0 * 0.3);
        assert_eq!(d.scale, 1.0 - 0.03);
        // 182 (`:665`): rotation++.
        let mut d = dust(182, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.rotation, 1.0);
        // 206 (`:764`): the decrement.
        let mut d = dust(206, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.9);
        // 174 (`:805`): the decrement.
        let mut d = dust(174, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.99);
        // 175 (`:801`): the shrink.
        let mut d = dust(175, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.95);
        // 213/260 (`:737`): rotation reset (the `:2081` rebuild follows).
        let mut d = dust(213, 1.0);
        d.rotation = 5.0;
        d.velocity = Vec2::new(2.0, 0.0);
        run(&mut d, &mut rng);
        assert_eq!(d.rotation, 2.0 * 0.5, "reset to 0 then rebuilt from vx");
    }

    #[test]
    fn w4_d50_the_261_ancient_light_drift_and_damp() {
        let mut rng = UnifiedRandom::new(7);
        // noGravity, fadeIn 0 (`:695-704`).
        let mut d = dust(261, 1.0);
        d.no_gravity = true;
        d.fade_in = 0.0;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        // velocity *= 0.93 then *= (0.97, 0.99).
        assert_eq!(d.velocity, Vec2::new(0.93 * 0.97, 0.93 * 0.99));
        // scale += 0.0025 (fadeIn 0) then -= 0.0025 → net 0.
        assert_eq!(d.scale, 1.0);
        // fadeIn != 0: no grow, only the shrink.
        let mut d = dust(261, 1.0);
        d.no_gravity = true;
        d.fade_in = 5.0;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 1.0 - 0.0025);
    }

    #[test]
    fn w4_d50_the_solid_contact_arms_329_211_284() {
        let mut rng = UnifiedRandom::new(7);
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| true;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mk = || DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        // 329 (`:587`): scale *= 0.8.
        let mut d = dust(329, 1.0);
        let mut e = mk();
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(d.scale, 0.8);
        // 211 (`:729`): noLight + solid → kill.
        let mut d = dust(211, 1.0);
        d.no_light = true;
        let mut e = mk();
        type_chain(&mut d, &mut rng, &mut e);
        assert!(!d.active);
        // 284 (`:733`): fadeIn 0 + solid → velocity *= 0.25.
        let mut d = dust(284, 1.0);
        d.fade_in = 0.0;
        d.velocity = Vec2::new(4.0, 4.0);
        let mut e = mk();
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(d.velocity, Vec2::new(1.0, 1.0));
    }

    // ---- the big else-if chain ----

    #[test]
    fn w4_d50_the_235_dead_vector_keeps_its_two_draws() {
        // `:817-820`: two Next(-100,101) (the dead vector3) then scale -= 0.01.
        let mut d = dust(235, 1.0);
        let mut rng = ScriptRng::new(&[50, -50]);
        let (matched, spawns, _) = run(&mut d, &mut rng);
        assert!(!matched);
        assert!(spawns.is_empty());
        assert_eq!(d.scale, 0.99);
        assert_eq!(
            rng.calls,
            vec!["rng-100..101=50", "rng-100..101=-50"],
            "the dead vector's two draws are kept, in order"
        );
    }

    #[test]
    fn w4_d50_the_torch_family_gravity_half() {
        let mut rng = UnifiedRandom::new(7);
        // gravity torch dust (`:824-827`): velocity.Y += 0.05.
        let mut d = dust(6, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        // The family is in the big chain; the generic rotation rebuild
        // (`:2081`) adds vx*0.5 and the burn-out (`:2127`) -= 0.01.
        assert_eq!(d.velocity.y, 1.05);
        // noGravity torch: no lift.
        let mut d = dust(6, 1.0);
        d.no_gravity = true;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.velocity.y, 1.0);
    }

    #[test]
    fn w4_d50_the_304_shimmer_sine_and_alpha_ladder() {
        let mut rng = UnifiedRandom::new(7);
        // scale < 1.15 (`:1163`): alpha -= 20 (floor 0), scale += 0.0015.
        let mut d = dust(304, 1.0);
        d.rotation = 0.0;
        d.alpha = 100;
        run(&mut d, &mut rng);
        assert_eq!(d.velocity.y, 0.0f32.sin() / 5.0);
        assert_eq!(d.rotation, 0.015);
        assert_eq!(d.alpha, 80);
        assert_eq!(d.scale, 1.0015);
        // scale >= 1.15 (`:1168`): alpha += 6; >= 255 kills.
        let mut d = dust(304, 1.2);
        d.alpha = 250;
        run(&mut d, &mut rng);
        assert_eq!(d.alpha, 256);
        assert!(!d.active, "alpha >= 255 deactivates");
        // 304 is exempt from the rotation rebuild and the burn-out tail.
        let mut d = dust(304, 1.0);
        d.fade_in = 0.0;
        let s0 = d.scale;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, s0 + 0.0015, "no generic -0.01 burn-out for 304");
    }

    #[test]
    fn w4_d50_the_139_family_spin_and_lift() {
        let mut rng = UnifiedRandom::new(7);
        let mut d = dust(140, 1.0);
        d.velocity = Vec2::new(2.0, 0.5);
        run(&mut d, &mut rng);
        // vx *= 0.98 → 1.96; vy *= 0.98 → 0.49 then += 0.05 (vy < 1) → 0.54.
        assert_eq!(d.velocity.x, 2.0 * 0.98);
        assert_eq!(d.velocity.y, 0.5 * 0.98 + 0.05);
        assert_eq!(d.scale, 1.009);
        // rotation -= vx*0.4 then += 0.005 (vx > 0), then the generic
        // rebuild += vx*0.5 (`:2081`).
        let vx = 2.0 * 0.98;
        assert_eq!(d.rotation, -vx * 0.4 + 0.005 + vx * 0.5);
    }

    #[test]
    fn w4_d50_the_31_no_gravity_arm_is_reachable_without_custom_data() {
        // `:1297` — with the NPC customData branch dropped, the else-if fires.
        let mut rng = UnifiedRandom::new(7);
        let mut d = dust(31, 1.0);
        d.no_gravity = true;
        d.velocity = Vec2::new(1.0, 1.0);
        d.alpha = 0;
        run(&mut d, &mut rng);
        // vy *= 0.98, vx *= 0.98 (`:1268-1269`) then *= 1.02 (`:1299`).
        assert_eq!(d.velocity, Vec2::new(0.98 * 1.02, 0.98 * 1.02));
        assert_eq!(d.scale, 1.02);
        assert_eq!(d.alpha, 4);
        // alpha > 255 collapses the scale.
        let mut d = dust(31, 1.0);
        d.no_gravity = true;
        d.alpha = 254;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.0001);
        assert_eq!(d.alpha, 255);
    }

    #[test]
    fn w4_d50_the_303_no_gravity_alpha_floor() {
        let mut rng = UnifiedRandom::new(7);
        let mut d = dust(303, 1.0);
        d.no_gravity = true;
        d.alpha = 10;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        // alpha floor 90 (`:1313`) then += 4 → 94.
        assert_eq!(d.alpha, 94);
        assert_eq!(d.scale, 1.03);
        assert_eq!(d.velocity, Vec2::new(0.98 * 1.02, 0.98 * 1.02));
    }

    #[test]
    fn w4_d50_the_27_no_light_player_push_wet_half() {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mut rng = UnifiedRandom::new(7);
        // noLight + wet → half push (`:1679-1682`).
        let mut d = dust(27, 1.0);
        d.no_light = true;
        d.position = Vec2::new(80.0, 80.0);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 2.0,
            player_vy: 4.0,
            player_wet: true,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(d.position, Vec2::new(81.0, 82.0), "the wet half-push");
        // noLight + dry → full push (`:1685`).
        let mut d = dust(45, 1.0);
        d.no_light = true;
        d.position = Vec2::new(80.0, 80.0);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 2.0,
            player_vy: 4.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(d.position, Vec2::new(82.0, 84.0), "the dry full push");
    }

    #[test]
    fn w4_d50_the_27_fade_in_burn_down_ladder() {
        let mut rng = UnifiedRandom::new(7);
        // 27, fadeIn >= 100, scale 1.6 (`:1649-1667`): >= 1.5 → -= 0.01.
        let mut d = dust(27, 1.6);
        d.fade_in = 100.0;
        run(&mut d, &mut rng);
        // 1.6 - 0.01 = 1.59; not <= 0.5/0.25; then *= 0.94 vel, += 0.002.
        assert!((d.scale - (1.59 + 0.002)).abs() < 1e-5);
        // scale 0.3: < 1.5 → -= 0.05 → 0.25; <= 0.5 → -= 0.05 → 0.20;
        // <= 0.25 → -= 0.05 → 0.15; then += 0.002.
        let mut d = dust(27, 0.3);
        d.fade_in = 100.0;
        run(&mut d, &mut rng);
        assert!((d.scale - (0.15 + 0.002)).abs() < 1e-5);
    }

    #[test]
    fn w4_d50_the_76_snow_fall_lerp_and_push() {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mut rng = UnifiedRandom::new(7);
        let mut d = dust(76, 1.0);
        d.fade_in = 0.0;
        d.velocity = Vec2::new(0.0, 1.0);
        d.position = Vec2::new(80.0, 80.0);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 5.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        // lerp(1, 5, 0.04) = 1.16; push y += 5*0.2 = 1.
        assert_eq!(d.velocity.y, 1.0 + (5.0 - 1.0) * 0.04);
        assert_eq!(d.position.y, 81.0);
        assert_eq!(d.scale, 1.009);
    }

    #[test]
    fn w4_d50_the_268_sand_count_and_sine_and_no_visuals_fade() {
        let mut rng = UnifiedRandom::new(7);
        let mut d = dust(268, 1.0);
        d.velocity = Vec2::new(2.0, 0.0);
        d.fade_in = 0.0;
        let (matched, _, sand) = run(&mut d, &mut rng);
        assert!(!matched, "268 is a FIRST-ladder arm");
        assert_eq!(sand, 1, "SandStormCount++");
        // velocity *= 1.0050251 then the sine overwrites vy.
        assert_eq!(d.velocity.x, 2.0 * 1.0050251);
        assert_eq!(d.velocity.y, (80.0f32 * 0.0043982295).sin() * 2.0 + 3.0);
        // 268 is exempt from the rotation rebuild (`:2079`).
        assert_eq!(d.rotation, 0.0);
        // Without visuals the scale fades (`:1794-1797`).
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand2 = 0;
        let mut d = dust(268, 1.0);
        d.fade_in = 0.0;
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: false,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand2,
        };
        type_chain(&mut d, &mut rng, &mut e);
        // +0.01 - 0.05 (no visuals) - 0.01 (burn-out) = -0.05.
        assert!((d.scale - 0.95).abs() < 1e-5);
    }

    #[test]
    fn w4_d50_the_270_271_sine_rise_and_fall() {
        let mut rng = UnifiedRandom::new(7);
        // 270 (`:1764`): sine - 3 then / 20.
        let mut d = dust(270, 1.0);
        d.velocity = Vec2::new(2.0, 9.0);
        d.fade_in = 0.0;
        run(&mut d, &mut rng);
        let sine = (80.0f32 * 0.0043982295).sin() * 2.0;
        assert_eq!(d.velocity.y, (sine - 3.0) / 20.0);
        assert_eq!(d.velocity.x, 2.0 * 1.0050251);
        // 271 (`:1781`): (vy*1.0050251 - 4) / 6.
        let mut d = dust(271, 1.0);
        d.velocity = Vec2::new(2.0, 9.0);
        run(&mut d, &mut rng);
        assert_eq!(d.velocity.y, (9.0 * 1.0050251 - 4.0) / 6.0);
    }

    #[test]
    fn w4_d50_the_generic_gravity_fall_and_the_107_exception() {
        let mut rng = UnifiedRandom::new(7);
        // A plain gravity dust (`:1827`): vy += 0.1.
        let mut d = dust(60, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        // vy += 0.1 then the generic drag vx *= 0.99 (`:2073`).
        assert_eq!(d.velocity.y, 1.1);
        assert_eq!(d.velocity.x, 0.99);
        // 107 (`:1821`): velocity *= 0.9 instead.
        let mut d = dust(107, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(1.0, 1.0);
        run(&mut d, &mut rng);
        assert_eq!(d.velocity, Vec2::new(0.9 * 0.99, 0.9));
        // 41/44/309 are exempt from the generic fall (their own arms run).
        let mut d = dust(41, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(0.0, 0.0);
        run(&mut d, &mut rng);
        // 41 jitters (rng) but no +0.1 fall; scale += 0.007.
        assert_eq!(d.scale, 1.007);
    }

    // ---- the standalone bubble / wet arms ----

    #[test]
    fn w4_d50_the_bubble_family_alpha_and_cap() {
        let mut rng = UnifiedRandom::new(7);
        // 308 (`:1834`): vx == 0 → rotation += 0.5, scale -= 0.01; then the
        // dry tail alpha += 2, scale -= 0.005.
        let mut d = dust(308, 1.0);
        d.velocity = Vec2::new(0.0, 5.0);
        d.alpha = 0;
        run(&mut d, &mut rng);
        assert_eq!(d.rotation, 0.5);
        assert_eq!(d.alpha, 2);
        // scale 1 - 0.01 - 0.005 = 0.985; vy capped at 4.
        assert!((d.scale - 0.985).abs() < 1e-5);
        assert_eq!(d.velocity.y, 4.0);
        // alpha > 255 → scale 0.
        let mut d = dust(33, 1.0);
        d.velocity = Vec2::new(1.0, 0.0);
        d.alpha = 254;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.0);
    }

    #[test]
    fn w4_d50_the_34_wet_arm_draws_and_clamps() {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| true;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        // 34 wet (`:1959-1986`): Next(2) then Next(-10,10), vx clamp ±0.25.
        let mut d = dust(34, 1.0);
        d.velocity = Vec2::new(0.0, 0.0);
        d.alpha = 0;
        let mut rng = ScriptRng::new(&[1, 100]);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(rng.calls, vec!["max2=1", "rng-10..10=100"]);
        assert_eq!(d.alpha, 1, "Next(2)=1");
        assert_eq!(d.scale, 1.005, "the 34 grow");
        assert_eq!(d.velocity.y, -0.5);
        // Next(-10,10)=100 → 100 % 20 - 10 = 0 → vx += 0; clamp leaves 0.
        assert_eq!(d.velocity.x, 0.0);
        // Dry 34 (`:1955`): scale 0, NO draws.
        let dry = |_: Vec2, _: f32, _: f32| false;
        let mut d = dust(34, 1.0);
        let mut rng = ScriptRng::new(&[]);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &dry,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert_eq!(d.scale, 0.0);
        assert!(rng.calls.is_empty(), "the dry path draws nothing");
    }

    #[test]
    fn w4_d50_the_41_44_jitter_clamps() {
        // 41 (`:2015`): two Next(-10,11) * 0.01, clamp ±0.75, scale += 0.007.
        let mut d = dust(41, 1.0);
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[100, -100]);
        run(&mut d, &mut rng);
        assert_eq!(rng.calls, vec!["rng-10..11=100", "rng-10..11=-100"]);
        // 100 % 21 - 10 = 16 - 10 = 6 → vx += 0.06; -100 % 21 = 5 → -5 → vy -= 0.05.
        assert!((d.velocity.x - 0.06).abs() < 1e-6);
        assert!((d.velocity.y - (-0.05)).abs() < 1e-6);
        assert_eq!(d.scale, 1.007);
        // 44 (`:2043`): * 0.003, clamp ±0.35, scale += 0.0085.
        let mut d = dust(44, 1.0);
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0]);
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 1.0085);
    }

    // ---- the fadeIn ramp + the generic tails ----

    #[test]
    fn w4_d50_the_fade_in_ramp_per_type_and_burn_out() {
        let mut rng = UnifiedRandom::new(7);
        // 46 fadeIn ramp (`:2102`): scale += 0.1.
        let mut d = dust(46, 1.0);
        d.fade_in = 5.0;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 1.1);
        assert_eq!(d.fade_in, 0.0, "scale > fadeIn zeroes it");
        // generic fadeIn ramp (`:2110`): scale += 0.03.
        let mut d = dust(60, 1.0);
        d.fade_in = 5.0;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 1.03);
        // 213 burn-out (`:2121`): scale -= 0.2 when fadeIn stands at 0.
        let mut d = dust(213, 1.0);
        d.fade_in = 0.0;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.8);
        // generic burn-out (`:2127`): scale -= 0.01.
        let mut d = dust(60, 1.0);
        d.fade_in = 0.0;
        run(&mut d, &mut rng);
        assert_eq!(d.scale, 0.99);
    }

    #[test]
    fn w4_d50_the_235_fade_in_homing_uses_the_player_center() {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        // fadeIn = 1 → num78 = 0 → the player center (`:2091`).
        let mut d = dust(235, 1.0);
        d.fade_in = 1.0;
        d.position = Vec2::new(110.0, 21.0);
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = UnifiedRandom::new(7);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        // The FIRST-ladder 235 arm (`:820`) runs scale -= 0.01 → 0.99, then
        // the fadeIn ramp (`:2087`) += 0.007 → 0.997. vector6 = (100, 0),
        // length 100 → num79 = 0 (no scale cut); normalize → (1, 0);
        // *= -(1-0.997)*20 = -0.06 → vector6 = (-0.06, 0);
        // velocity = (0*4 + (-0.06))/5 = -0.012.
        assert!((d.velocity.x - (-0.012)).abs() < 1e-5);
        assert_eq!(d.fade_in, 1.0, "scale (0.997) < fadeIn (1) — NOT zeroed");
    }

    // ---- the SECOND chain (matched → tail suppressed) ----

    #[test]
    fn w4_d50_the_second_chain_arms_match_and_suppress_the_tail() {
        let mut rng = UnifiedRandom::new(7);
        // 130 noGravity (`:2153`): velocity *= 0.93, fadeIn 0 → scale += 0.0025.
        let mut d = dust(130, 1.0);
        d.no_gravity = true;
        d.fade_in = 0.0;
        d.velocity = Vec2::new(1.0, 1.0);
        let (matched, _, _) = run(&mut d, &mut rng);
        assert!(matched, "130 is a SECOND-chain arm");
        assert_eq!(d.velocity, Vec2::new(0.93, 0.93));
        assert_eq!(d.scale, 1.0025);
        // 131 gravity (`:2161`): velocity *= 0.98, vy -= 0.1, scale += 0.0025.
        let mut d = dust(131, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(1.0, 1.0);
        let (matched, _, _) = run(&mut d, &mut rng);
        assert!(matched);
        assert_eq!(d.velocity, Vec2::new(0.98, 0.98 - 0.1));
        // 132 gravity (`:2167`): velocity *= 0.95, scale -= 0.0025.
        let mut d = dust(132, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(1.0, 1.0);
        let (matched, _, _) = run(&mut d, &mut rng);
        assert!(matched);
        assert_eq!(d.velocity, Vec2::new(0.95, 0.95));
        assert_eq!(d.scale, 0.9975);
        // 219 noGravity (`:2229`): *= 0.93 then *= (0.97, 0.99), scale -= 0.0025.
        let mut d = dust(219, 1.0);
        d.no_gravity = true;
        d.fade_in = 0.0;
        d.velocity = Vec2::new(1.0, 1.0);
        let (matched, _, _) = run(&mut d, &mut rng);
        assert!(matched);
        assert_eq!(d.velocity, Vec2::new(0.93 * 0.97, 0.93 * 0.99));
        // 226/331/272 (`:2264`/`:2295`/`:2322`): *= (0.97,0.99), scale -= 0.01.
        for ty in [226, 331, 272] {
            let mut d = dust(ty, 1.0);
            d.no_gravity = false;
            d.velocity = Vec2::new(1.0, 1.0);
            let (matched, _, _) = run(&mut d, &mut rng);
            assert!(matched, "type {ty} matches");
            assert_eq!(d.velocity, Vec2::new(0.97, 0.99));
            assert_eq!(d.scale, 0.99);
        }
    }

    #[test]
    fn w4_d50_the_278_solid_tile_damp() {
        let w = solid_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mut rng = UnifiedRandom::new(7);
        // 278 gravity over the solid tile (5,5) = px (80,80) (`:2197`).
        let mut d = dust(278, 1.0);
        d.no_gravity = false;
        d.fade_in = 0.0;
        d.position = Vec2::new(80.0, 80.0);
        d.velocity = Vec2::new(1.0, 1.0);
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &light,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        let matched = type_chain(&mut d, &mut rng, &mut e);
        assert!(matched);
        // *= 0.95 then scale -= 0.0025, then *= 0.9 (solid).
        assert!((d.scale - (1.0 * 0.95 - 0.0025) * 0.9).abs() < 1e-5);
        assert_eq!(d.velocity, Vec2::new(0.95 * 0.25, 0.95 * 0.25));
    }

    #[test]
    fn w4_d50_the_43_dark_kill_reads_the_pre_ramp_scale() {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let mut spawns = Vec::new();
        let mut sand = 0;
        let mut rng = UnifiedRandom::new(7);
        // Black light + black dust colour → num48/49/50 = 0 → kill (`:1435`).
        let dark = |_: i32, _: i32| Color::new(0, 0, 0, 255);
        let mut d = dust(43, 1.0);
        d.color = Color::new(0, 0, 0, 255);
        d.alpha = 255;
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &dark,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert!(!d.active, "the full-dark 43 dies");
        // Bright light + bright colour → survives.
        let bright = |_: i32, _: i32| Color::WHITE;
        let mut d = dust(43, 1.0);
        d.color = Color::WHITE;
        d.alpha = 255;
        let mut e = DustPhysicsEnv {
            world: &w,
            show_sandstorm_visuals: true,
            player_vx: 0.0,
            player_vy: 0.0,
            player_wet: false,
            player_center: Vec2::new(10.0, 21.0),
            solid_collision: &solid,
            wet_collision: &wet,
            light_color: &bright,
            spawns: &mut spawns,
            sand_storm_count: &mut sand,
        };
        type_chain(&mut d, &mut rng, &mut e);
        assert!(d.active);
    }

    // ---- the RNG NewDust census: the child-spawn arms ----

    #[test]
    fn w4_d50_census_the_159_gravity_child_spawn_draw_order() {
        // `:1080-1089`: scale += 0.005, vel *= 0.9, Next(-10,11) x2, Next(5);
        // ==0 → child NewDust(pos, 4, 4, 159), noGravity, scale*2.5.
        let mut d = dust(159, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(0.0, 0.0);
        // Script: the two jitters then Next(5)=0 (the child gate).
        let mut rng = ScriptRng::new(&[0, 0, 0]);
        let (matched, spawns, _) = run(&mut d, &mut rng);
        assert!(!matched);
        assert_eq!(
            rng.calls,
            vec!["rng-10..11=0", "rng-10..11=0", "max5=0"],
            "the 159 gravity census: two jitters then the Next(5) gate"
        );
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].dust_type, 159);
        assert_eq!((spawns[0].width, spawns[0].height), (4, 4));
        assert_eq!(spawns[0].position, Vec2::new(80.0, 80.0));
        // scale after += 0.005 = 1.005; child scale = 1.005 * 2.5.
        assert_eq!(spawns[0].scale, 1.005 * 2.5);
    }

    #[test]
    fn w4_d50_census_the_159_no_gravity_scale_gate() {
        // scale < 0.7 (`:1064`): vel *= 1.075, NO Next(2) draw.
        let mut d = dust(159, 0.5);
        d.no_gravity = true;
        d.velocity = Vec2::new(1.0, 1.0);
        let mut rng = ScriptRng::new(&[]);
        run(&mut d, &mut rng);
        assert!(rng.calls.is_empty(), "the < 0.7 branch draws nothing");
        assert_eq!(d.velocity, Vec2::new(1.075, 1.075));
        assert_eq!(d.scale, 0.5 - 0.03);
        // scale >= 0.7 (`:1068`): Next(2) decides -0.95 vs 1.05.
        let mut d = dust(159, 0.8);
        d.no_gravity = true;
        d.velocity = Vec2::new(1.0, 1.0);
        let mut rng = ScriptRng::new(&[0]);
        run(&mut d, &mut rng);
        assert_eq!(rng.calls, vec!["max2=0"]);
        assert_eq!(d.velocity, Vec2::new(-0.95, -0.95));
        let mut d = dust(159, 0.8);
        d.no_gravity = true;
        d.velocity = Vec2::new(1.0, 1.0);
        let mut rng = ScriptRng::new(&[1]);
        run(&mut d, &mut rng);
        assert_eq!(d.velocity, Vec2::new(1.05, 1.05));
    }

    #[test]
    fn w4_d50_census_the_173_gravity_next10_gate() {
        // `:1147-1156`: scale -= 0.015, vel *= 0.8, Next(-10,11) x2 * 0.005,
        // Next(10); ==10 → child scale = dust.scale (no fold).
        let mut d = dust(173, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0, 10]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert_eq!(
            rng.calls,
            vec!["rng-10..11=0", "rng-10..11=0", "max10=10"]
        );
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].scale, 1.0 - 0.015, "the 173 child carries the parent scale");
        // Next(10) != 10 → no child.
        let mut d = dust(173, 1.0);
        d.no_gravity = false;
        let mut rng = ScriptRng::new(&[0, 0, 3]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert!(spawns.is_empty());
    }

    #[test]
    fn w4_d50_census_the_160_gravity_next50_scale_gate() {
        // `:1223-1231`: scale -= 0.1, Next(-10,11) x2 * 0.02, then
        // scale > 0.3 && Next(50)==0 → child at pos-4, 1x1, scale*1.5.
        let mut d = dust(160, 1.0);
        d.no_gravity = false;
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0, 0]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert_eq!(
            rng.calls,
            vec!["rng-10..11=0", "rng-10..11=0", "max50=0"]
        );
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].position, Vec2::new(76.0, 76.0), "pos - 4");
        assert_eq!((spawns[0].width, spawns[0].height), (1, 1));
        assert_eq!(spawns[0].scale, 0.9 * 1.5);
        // scale <= 0.3 (`:1226`): the Next(50) is NOT drawn (short-circuit).
        let mut d = dust(162, 0.35);
        d.no_gravity = false;
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        // scale 0.35 - 0.1 = 0.25 <= 0.3 → no Next(50).
        assert_eq!(rng.calls, vec!["rng-10..11=0", "rng-10..11=0"]);
        assert!(spawns.is_empty());
    }

    #[test]
    fn w4_d50_census_the_168_two_jitters_no_child() {
        // `:1242-1245`: scale += 0.03, Next(-10,11) x2 * 0.02, vel *= 0.99.
        let mut d = dust(168, 1.0);
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert_eq!(rng.calls, vec!["rng-10..11=0", "rng-10..11=0"]);
        assert!(spawns.is_empty());
        assert_eq!(d.scale, 1.03);
    }

    #[test]
    fn w4_d50_census_the_173_no_gravity_two_jitters() {
        // `:1140-1143`: vel *= 0.8, Next(-20,21) x2 * 0.01, scale -= 0.01.
        let mut d = dust(173, 1.0);
        d.no_gravity = true;
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert_eq!(rng.calls, vec!["rng-20..21=0", "rng-20..21=0"]);
        assert!(spawns.is_empty(), "the noGravity 173 spawns no child");
        assert_eq!(d.scale, 0.99);
    }

    #[test]
    fn w4_d50_census_the_160_no_gravity_two_jitters() {
        // `:1216-1219`: vel *= 0.8, Next(-20,21) x2 * 0.04, scale -= 0.1.
        let mut d = dust(160, 1.0);
        d.no_gravity = true;
        d.velocity = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[0, 0]);
        let (_, spawns, _) = run(&mut d, &mut rng);
        assert_eq!(rng.calls, vec!["rng-20..21=0", "rng-20..21=0"]);
        assert!(spawns.is_empty());
        assert_eq!(d.scale, 0.9);
    }

    // ---- the no-draw witness: the RNG-free arms leave the stream untouched ----

    #[test]
    fn w4_d50_the_rng_free_arms_draw_nothing() {
        // The torch family, the gem dusts, 304, 139-142, the second chain and
        // the generic tails draw NO rand. A scripted rng with an empty script
        // must survive them (any draw would panic on exhaustion).
        let mut rng = ScriptRng::new(&[]);
        for ty in [6, 29, 75, 86, 92, 286, 304, 140, 130, 219, 226, 278, 331, 272, 60] {
            let mut d = dust(ty, 1.0);
            d.no_gravity = false;
            let _ = run(&mut d, &mut rng);
        }
        assert!(rng.calls.is_empty(), "no RNG-free arm drew");
    }

    // ---- the full-pool integration: pre_move + move + type_chain + tail ----

    /// Drive the generic loop with the physics hook, mirroring the wiring:
    /// `pre_move` before the move, `type_chain` as the hook, the skeleton's
    /// gravity tail when the hook returns false.
    fn tick_physics(pool: &mut DustPool, rng: &mut UnifiedRandom, ctx: &UpdateCtx) {
        let w = empty_world();
        let solid = |_: Vec2, _: f32, _: f32| false;
        let wet = |_: Vec2, _: f32, _: f32| false;
        let light = |_: i32, _: i32| Color::WHITE;
        let mut spawns = Vec::new();
        let mut sand = 0;
        // pre_move (`:468-476`) runs BEFORE the skeleton's move (`:477`); the
        // skeleton owns the move and the kills, the hook is type_chain.
        for i in 0..crate::particles::DUST_SENTINEL {
            if pool.slot(i).active {
                let mut lava = pool.lava_bubbles;
                let mut d = *pool.slot(i);
                pre_move(&mut d, &mut lava);
                *pool.slot_mut(i) = d;
                pool.lava_bubbles = lava;
            }
        }
        pool.update_dust(ctx, rng, |d| {
            let mut env = DustPhysicsEnv {
                world: &w,
                show_sandstorm_visuals: true,
                player_vx: 0.0,
                player_vy: 0.0,
                player_wet: false,
                player_center: Vec2::new(10.0, 21.0),
                solid_collision: &solid,
                wet_collision: &wet,
                light_color: &light,
                spawns: &mut spawns,
                sand_storm_count: &mut sand,
            };
            type_chain(d, rng, &mut env)
        });
        pool.sand_storm_count += sand;
        // Drain the child spawns (the census's NewDust requests).
        let sc = SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(10000.0, 10000.0),
            max_dust_to_draw: 6000,
        };
        for s in spawns {
            let idx = pool.new_dust(
                rng, &sc, &w, s.position, s.width, s.height, s.dust_type, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = pool.slot_mut(idx);
            d.no_gravity = true;
            d.scale = s.scale;
        }
    }

    #[test]
    fn w4_d50_the_full_pool_tick_integrates_the_physics_chain() {
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(42);
        let ctx = UpdateCtx {
            max_dust_to_draw: 6000,
            screen_position: Vec2::ZERO,
            screen_height: 10000.0,
        };
        // A gravity torch dust (6): pre_move no-op, move, family lift +0.05,
        // generic drag, burn-out -0.01, NO tail (gravity dust).
        {
            let d = pool.slot_mut(0);
            d.active = true;
            d.dust_type = 6;
            d.no_gravity = false;
            d.fade_in = 0.0;
            d.scale = 1.0;
            d.position = Vec2::new(100.0, 100.0);
            d.velocity = Vec2::new(2.0, 1.0);
        }
        // A no-gravity 130 (SECOND chain): tail suppressed.
        {
            let d = pool.slot_mut(1);
            d.active = true;
            d.dust_type = 130;
            d.no_gravity = true;
            d.fade_in = 0.0;
            d.scale = 1.0;
            d.position = Vec2::new(100.0, 100.0);
            d.velocity = Vec2::new(2.0, 1.0);
        }
        // A no-gravity plain dust (60): no arm matches → the skeleton tail.
        {
            let d = pool.slot_mut(2);
            d.active = true;
            d.dust_type = 60;
            d.no_gravity = true;
            d.fade_in = 0.0;
            d.scale = 1.0;
            d.position = Vec2::new(100.0, 100.0);
            d.velocity = Vec2::new(2.0, 1.0);
        }
        tick_physics(&mut pool, &mut rng, &ctx);
        // Slot 0 (6): move by (2,1) → (102,101); family lift vy 1→1.05; drag
        // vx 2→1.98; burn-out scale 1→0.99.
        let d0 = pool.slot(0);
        assert_eq!(d0.position, Vec2::new(102.0, 101.0));
        assert_eq!(d0.velocity, Vec2::new(1.98, 1.05));
        assert_eq!(d0.scale, 0.99);
        // Slot 1 (130): move → (102,101); SECOND chain *= 0.93, fadeIn 0 →
        // scale += 0.0025; the tail is SUPPRESSED (matched).
        let d1 = pool.slot(1);
        assert_eq!(d1.velocity, Vec2::new(2.0 * 0.93, 1.0 * 0.93));
        assert_eq!(d1.scale, 1.0025);
        // Slot 2 (60): move → (102,101); no arm → the skeleton tail *= 0.92,
        // fadeIn 0 → scale -= 0.04; the generic burn-out -= 0.01 also runs.
        let d2 = pool.slot(2);
        assert_eq!(d2.velocity, Vec2::new(2.0 * 0.92, 1.0 * 0.92));
        assert_eq!(d2.scale, 1.0 - 0.01 - 0.04);
    }

    #[test]
    fn w4_d50_the_child_spawn_drains_through_the_real_allocator() {
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(42);
        let ctx = UpdateCtx {
            max_dust_to_draw: 6000,
            screen_position: Vec2::ZERO,
            screen_height: 10000.0,
        };
        // A gravity 159 at scale 1: the Next(5) gate fires a child on most
        // seeds; drive several ticks and confirm a noGravity 159 child lands.
        {
            let d = pool.slot_mut(0);
            d.active = true;
            d.dust_type = 159;
            d.no_gravity = false;
            d.fade_in = 0.0;
            d.scale = 1.0;
            d.position = Vec2::new(100.0, 100.0);
            d.velocity = Vec2::new(0.0, 0.0);
        }
        let mut saw_child = false;
        for _ in 0..40 {
            tick_physics(&mut pool, &mut rng, &ctx);
            for i in 1..crate::particles::DUST_SENTINEL {
                let d = pool.slot(i);
                if d.active && d.dust_type == 159 && d.no_gravity {
                    saw_child = true;
                }
            }
            // Keep slot 0 alive for the next tick.
            let d = pool.slot_mut(0);
            d.active = true;
            d.scale = 1.0;
            d.fade_in = 0.0;
        }
        assert!(saw_child, "a 159 gravity child spawned and drained noGravity");
    }
}