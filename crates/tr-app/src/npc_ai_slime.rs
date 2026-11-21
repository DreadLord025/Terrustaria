//! S1-D-3 (aiStyle-1) — the slime family, `NPC.AI_001_Slimes`
//! (`NPC.cs:60341-61723`), re-ported 1:1 as a PURE velocity machine
//! (code-only mode: no compiler/tests were run on this branch; the witness
//! tests below are written, not executed). The A-1 audit
//! (`docs/research/54-audit-slime-eye.md` §1, branch `docs/audit-slime-eye`)
//! found the live `npc.rs::update_slime` (`:6652-6731`) is a **sin-timer hop
//! stand-in**: an invented `hop_timer` cadence (1.0 + 0.5·|sin(anim·7)|),
//! invented 1.4 / -4.2 hop constants, an invented 60-tile aggro gate, an
//! invented 0.4/8 gravity pair, an invented `vx = -vx*0.4` wall bounce and
//! `vx *= 0.6` landing brake. NONE of that exists in the C# arm.
//!
//! ## The real machine (audit §1a — ZERO-RNG cadence)
//! The hop cadence is the `ai[0]` tick counter (`:61511-61703`): init
//! `ai[0] = -100` + `TargetClosest` when `ai[2] == 0` (`:61512-61517`),
//! grounded `ai[0]++` (doubled by `flag3`, `:61550-61554`), per-type bonuses
//! (`:61555-61602`), the three tiers `num51` (`:61612-61624`), hop
//! `vy = -6/-8`, `vx += 2/3·direction`, resets `ai[0] = -120 + num50` /
//! `-120 + 2·num50` / `-200` (`:61632-61664`). The cadence draws **zero**
//! `Main.rand` values — the audit confirmed the census's "jump rolls" worry
//! is unfounded for the cadence itself. There is NO distance gate: slimes
//! hop toward `base.direction` from `TargetClosest` regardless of range.
//!
//! ## Family census (`SetDefaults` `aiStyle = 1` rows, whole-tree grep)
//! 27 rows / 28 types: 1 (`:8576`, + the negative net ids -1..-10 via
//! `SetDefaultsFromNetId` `:7653-7755`), 16 (`:8968`), 59 (`:9520`),
//! 71 (`:9702`), 81 (`:9842`), 138 (`:10752`), 141 (`:10780`),
//! 147 (`:10851`), 183 (`:11336`), 184 (`:11350`), 204 (`:11615`),
//! 225 (`:11902`), 244 (`:12152`), 302 (`:12906`), 304 (`:12918`),
//! 333/334/335/336 (`:13234-13272`), 377 (`:13815`), 446 (`:14307`),
//! 535 (`:15833`), 537 (`:15862`), 658+659 (`:17330`, one row),
//! 667 (`:17438`), 676 (`:17570`), 685 (`:17613`).
//! The port's 10 live kinds (`npc.rs:3744-3753`): Slime 1, MotherSlime 16,
//! LavaSlime 59, DungeonSlime 71, CorruptSlime 81, IlluminantSlime 138,
//! IceSlime 147, Crimslime 183, SpikedIceSlime 184, UmbrellaSlime 225
//! (bestiary net ids verified == `NPCID.cs`). The machine below carries
//! EVERY per-type row (the unported kinds' rows are constants — they cost
//! nothing and keep the port whole-arm); only the per-type SHOOTER/light
//! blocks of unported kinds are deferred (documented below).
//!
//! ## Scope this round (the brief: core + gravity + 2 merges fully)
//! PORTED: the direction-0 latch (`:61041-61045`), the `ai[0] == -999`
//! balloon freeze (`:61046-61052`), `flag3` (`:61060-61099` + the 184/535/
//! 658/659/204 rows), the 81/147/184 dust GATE draws (dust bodies skipped —
//! the `DustRng` seam, S1-E-1, the caster precedent), the 667 speed-gated
//! dust draw, the WHOLE SpikedIceSlime 184 spike-shooter (`:61151-61210`,
//! shots ride [`SlimeEffects`]), the `ai[2]` countdown (`:61459-61462`),
//! the wet tail incl. the LavaSlime rows (`:61463-61510`), the init latch,
//! the grounded machine with the stuck-flip, the conveyor row, the f64
//! ±0.1 landing deadband and every cadence/tier/hop row
//! (`:61511-61702`), the air steer with the collideX 1.4-px nudge
//! (`:61704-61722`). Plus [`slime_gravity`] =
//! `UpdateNPC_UpdateGravity` (`:90878-90955`) for the slime kinds: the
//! 0.3 depth-scaled gravity / 10 cap with the **honey 0.1/4 and shimmer
//! 0.15/5.5 merges** the live arm lacks (audit §1b), and
//! [`wet_collision`] = `Collision.WetCollision` (`Collision.cs:1572-1651`)
//! which produces the `wet`/`honeyWet`/`shimmerWet` flags the C# carries
//! on the Entity (`NPC.cs:93063-93075`).
//!
//! DEFERRED (documented, the A1-5 slice + content waves):
//! * The item-content init (`:60356-60576`) + `GenerateItemInsideBody`
//!   (`:61734-61848`) — the real RNG deficit at spawn. Full draw census
//!   below; the port lands it as its own slice (audit seed A1-5).
//! * The type-1 paint tint (`:60577-60693`) and the `ai[1] > 0` content
//!   rows (`:60694-61040`: the ore-slime stat block + hover `flag2`, the
//!   dart/web/herb/hive/balloon/confetti arms). `flag2` folds to false
//!   (no content yet) — the direction-0 latch keeps its `!flag2` shape.
//! * The 75-lantern glow (`:60343-60355`, type 1 `ai[1] == 75` — only
//!   reachable through the deferred remix content init).
//! * The 244 rainbow colour (`:61053-61059`, `AI_001_SetRainbowSlimeColor`
//!   `:61725-61732`), the 535/658/659/204 shooter blocks
//!   (`:61212-61439`), the 377/446 friendly latch (`:61440-61450`), the
//!   59/667 lights (`:61451-61458`, `:61109-61127` — the
//!   `npc_lights.rs` seam) and every `Dust.NewDust` INTERNAL draw (the
//!   `DustRng` seam). The 676 light/dust arm (`:61128-61150`) is PORTED
//!   (W4-f47 keep-one 144): the gate + the 4-NextFloat body roll in-arm,
//!   the `NewDustPerfect(306)` spawn rides `wants_shimmer_slime_dust` to
//!   the sim drain; the torch-23 light is the draw-side
//!   `npc_lights::shimmer_slime_light` feed.
//! * The generic `UpdateNPC` tail stays the caller's: gravity application
//!   `vy += NPC.gravity` + cap (`:90465-90472`), the ±0.005 vx snap
//!   (`:90473-90476`), `UpdateCollision`/integration (`:90490-90503`).
//!   The C# reads LAST tick's `base.wet` flags at AI time (the Entity
//!   flags are written by `UpdateCollision` → `Collision_WaterCollision`,
//!   `NPC.cs:92526`/`:93063-93075`); the pure module takes them as
//!   [`SlimeCtx`] inputs — the wiring step feeds them from the carried
//!   Entity flags (or [`wet_collision`] at the same site).
//!
//! ## RNG by construction — the item-content init census (A1-5)
//! Gate (`:60356`): `SlimeCanContainItems[type]` (the set {1, 59, 147, 184,
//! 537}, `NPCID.cs:10443`) && `ai[1] == 0` && `netMode != 1` && `value > 0`.
//! The `ai[1] = -1` latch draws nothing. Attempts `num2`: 1 base; skyblock
//! `lowTiles` → `maxValue2 = 20`, `maxValue = 15`, +4 (netID -6: +5 more);
//! else netID -6 (Black Slime) → +4; `Main.slimeRain` → +2 (`:60362-60379`).
//! Per attempt (breaks once `ai[1] != -1`), in source order:
//! * **59** (`:60386-60397`): `remixWorld && Next(maxValue) == 0` →
//!   `GenerateItemInsideBody(ai[0] == -999)`; ELSE-IF
//!   `Skyblock.noHellstone && downedBoss3 && Next(15) == 0` → 174. The
//!   `&&` short-circuits: a non-remix world draws NOTHING for the first
//!   roll; the `Next(15)` is drawn only when the first branch failed.
//! * **147/184 below surface** (`:60398-60425`): `Next(maxValue2) == 0` →
//!   `Next(2)` → 1103 (slush) / 593 (snow). 1 draw on a failed gate, 2 on
//!   a hit. Above surface: the chain falls to the type-1 else and
//!   `continue`s — ZERO draws.
//! * **537** (`:60426-60432`): `Skyblock.noFossils && Next(5) == 0` → 3347.
//! * **1, netID ∉ {-5, -4}** (`:60435-60572`; -5 Baby / -4 Pinky
//!   `continue` — zero draws): `num4` = 500 (noTraps 20 / getGood 100 /
//!   above surface -1, `:60439-60451`). The else-if chain, each roll drawn
//!   only when every earlier gate failed:
//!   1. rock-layer && (noLifeCrystals || lowTiles) && !AnyLifeCrystalSlimes
//!      → `Next(200)` → 29 (`:60452`).
//!   2. lowTiles && below surface → `Next(1000)` → `GetRandomVoiceItem`
//!      (`Item.cs` — **`WorldGen.genRand.Next(14)`**, a DIFFERENT stream;
//!      `:60456-60459`).
//!   3. GenuineParty && above surface → `Next(2)`; 0 → `Next(3736, 3739)`
//!      (balloon), else 1345 (`:60460-60470`).
//!   4. netID -10 (Jungle) → `Next(20)`; hit → `Next(3)` (`Next(4)` on
//!      notTheBees) → 1124/1125/314/5395 (`:60471-60481`).
//!   5. `Next(maxValue)` → `GenerateItemInsideBody` (`:60482-60486`).
//!   6. `Next(maxValue2)` (`:60487-60556`): surface → lowTiles &&
//!      (moonPhase == 0 || `Next(2)`) → `Next(2)` → (`Next(50)` → 194 /
//!      `Next(10)` → 195 / 62) or 27; non-lowTiles → 751 (no draw).
//!      Underground-not-rock → `Next(3)` → 2/3/9. Rock → `Next(10)` →
//!      3609; else lowTiles && hardMode && `Next(2)` → `Next(6)` (ore);
//!      else `Next(4)` → 3/150/3086/3081.
//!   7. `num4 > 0` → `Next(num4)` → 539 (`:60557`).
//!   8. getGood && below surface → `Next(num4)` → 147 (`:60561`).
//!   9. `i == 0` && remix && `ai[0] != -999` → `Next(3)` → 75 (`:60565`).
//!   10. vampireSeed && !remix → `Next(13)` **drawn BEFORE the depth
//!       check** → 9 (`:60569` — the audit's draw-before-gate class).
//!   Then `netUpdate = true` (`:60575`).
//! `GenerateItemInsideBody` (`:61734-61848`): `Next(4)` ALWAYS; lowTiles →
//! `Next(3) != 0` → `Next(1, 3)` re-roll, then `Next(3) != 0` → unballoon.
//! Ballooned → `Next(13)` (the balloon table). Else by the first roll:
//! 0 → `Next(7)` (potions; default arm: `netMode != 0 && Next(2)` → 2997
//! else 2350); 1 → `Next(4)` (torch/rope/bomb/heart; lowTiles: moonPhase 0
//! → `Next(2)` re-roll, then roll == 2 → `Next(4)` again); 2 → rock &&
//! lowTiles && hardMode && `Next(2)` → `Next(6)` (ore), else `Next(2)` →
//! `Next(11, 15)` (bars) / `Next(699, 703)` (gems); 3 → `Next(3)`
//! (coins; lowTiles && `Next(5) != 0` → copper).
//!
//! ## RNG by construction — the PORTED arm's draws (per tick, in order)
//! 1. **81** (`:61072`): `Next(30)` EVERY tick (the dust body is the
//!    DustRng skip; the gate draw is the arm's).
//! 2. **147** (`:61101`): `Next(10)` EVERY tick (same skip).
//! 3. **667** (`:61113`): `velocity.Length() > 1 || Next(4) != 0` — the
//!    `||` short-circuits: speed > 1 draws NOTHING, else one `Next(4)`.
//! 4. **184** (`:61153`): `Next(8)` EVERY tick; the shooter
//!    (`:61179-61208`, `netMode != 1 && localAI[0] == 0` only): expert
//!    5-spike fan = 3 draws per spike (`Next(-50,51)` X-jitter,
//!    `Next(-50,51)` Y-jitter, `Next(-50,51)` speed) = **15 draws**;
//!    classic lob = **1 draw** (`Next(0, 200)` Y-jitter, `:61202`).
//! 5. **685** hop (`:61674`): `Next(2)` per hop (direction flip).
//! 6. **676** (`:61131-61146`, W4-f47 keep-one 144 — PORTED):
//!    `(speed > 1 && Next(3) == 0) || Next(5) == 0` — speed <= 1 draws
//!    ONLY the `Next(5)`; speed > 1 draws `Next(3)` first and the `||`
//!    short-circuits on its hit. On a gate hit:
//!    `NextVector2FromRectangle` (2 NextFloat over the integer-truncated
//!    Hitbox) + `NextFloat` hue jitter + `NextFloat` scale = 4 draws; the
//!    allocator's own 6 ride the sim drain (the gallop-dust seam).
//! Everything else — the cadence, tiers, wet tail, gravity, stuck-flips,
//! air steer, `TargetClosest` (`NPC.cs:77652-77781`) — draws NOTHING.
//! Unported-kind gates censused for their waves:
//! 535/658/659/204 shooters mirror the 184 shape with
//! their own jitter ranges (`:61232-61435`); the 75-lantern
//! (`:60347-60349`): `Next(12)` gate, on a hit `NextVector2CircularEdge`
//! (1 NextFloat via `NextVector2Unit`, `Utils.cs:1261-1274`) + `NextFloat`
//! radius + `NextFloat` velocity.
//! All draws ride the caller's `UnifiedRandom` (the `Main.rand` analogue);
//! the witnesses mirror the cited draws on a cloned stream and compare
//! the full state (`UnifiedRandom: PartialEq`).

#![allow(dead_code)] // the update_slime wiring swap is the merge step (S1-D-3)

use crate::npc::{Npc, NpcKind, ShimmerSlimeDust};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tile::LiquidKind;
use tr_core::{UnifiedRandom, World};

/// The per-tick context the arm reads besides the NPC and the world.
pub struct SlimeCtx<'a> {
    /// The `Main.player` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam, S1-D-3p).
    pub players: &'a [TargetCandidate],
    /// `Main.expertMode` (`:61172` — the 184 five-spike fan).
    pub expert_mode: bool,
    /// `Main.netMode` — 1 (client) skips the 184 shot (`:61179`/`:61200`);
    /// the port is single-player (0).
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`).
    pub ticker: u64,
    /// `base.wet` (`:61463`, `:61166`) — the Entity flag the C# carries
    /// from `UpdateCollision`'s `WetCollision` (`NPC.cs:93063`); the
    /// wiring feeds the carried flag (or [`wet_collision`] at the site).
    pub wet: bool,
    /// `base.honeyWet` (`NPC.cs:93066`) — [`slime_gravity`]'s 0.1/4 merge.
    pub honey_wet: bool,
    /// `base.shimmerWet` (`NPC.cs:93070`) — the 0.15/5.5 merge.
    pub shimmer_wet: bool,
}

/// One `Projectile.NewProjectile` request from the 184 spike-shooter
/// (`:61188`/`:61207`). The damage is `GetAttackDamage_ForProjectiles(9f,
/// 9f)` (`NPC.cs:7052-7055`) — classic and expert both 9, so the remap is
/// the constant 9; knockback 0f.
#[derive(Debug, PartialEq)]
pub struct SlimeShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `ProjectileID.IceSpike` 174 (`:61188`/`:61207`).
    pub kind: i32,
    pub damage: i32,
    pub knockback: f32,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct SlimeEffects {
    /// The 184 spike volleys, in fire order.
    pub shots: Vec<SlimeShot>,
    /// The `ai[0] == -999` freeze writes `frameCounter = 0` (`:61049`);
    /// the port's `Npc::frame_counter` is private to `npc.rs` — the
    /// wiring step resets it there (the `npc_ai_style42.rs` precedent).
    /// `frame.Y = 0` and `rotation = 0` land on the NPC directly.
    pub reset_frame_counter: bool,
}

/// The C# `this.type` for the port's kinds: every `SlimeVariant` is type 1
/// (the negative net ids share the base type, `NPC.cs:7653-7755`); the
/// other nine kinds ride their bestiary-verified `NPCID` (`cs_id`).
fn cs_type(npc: &Npc) -> i32 {
    if npc.kind == NpcKind::Slime {
        1
    } else {
        npc.kind.cs_id() as i32
    }
}

/// The target slot's candidate — the C# indexes `Main.player[this.target]`
/// (255 slots always exist); the port's slice holds the session player, so
/// an unset/stale slot falls back to slot 0 (the style55 precedent,
/// `NPC.cs:77731-77734`).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `this.TargetClosest()` — `faceTarget = true` default (`NPC.cs:77652`),
/// aiStyle 1. The empty-slice guard is the style24 precedent (the C#
/// always has 255 slots).
fn aim(npc: &mut Npc, world: &World, ctx: &SlimeCtx) {
    if !ctx.players.is_empty() {
        target_closest(npc, world, ctx.players, true, 1, ctx.ticker);
    }
}

/// `(float)Math.Sqrt(x*x + y*y)` — f32 mul/add, f64 root, f32 cast
/// (`:61171`, `Vector2.Distance`/`Length`).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA/FNA `Vector2.Normalize()` — `num = 1f / (float)Math.Sqrt(X*X+Y*Y);
/// X *= num; Y *= num`. UNGUARDED, exactly like the C# at `:61186` (the
/// 184 fan); the style55 precedent.
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70133`) — the
/// `npc_ai_pumpkin_boss.rs` shape: active, solid, not top-only, no
/// half-brick, no slope, not framed off; out-of-bounds → false.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `Collision.SolidCollision(position, width, height)`
/// (`Collision.cs:2700-2733`): the padded ±1/+2 tile scan (clamped to the
/// world, Y to `maxTilesY - 40`), solid-not-top tiles with the half-brick
/// 8-px lift, strict overlap. NOTE: unlike `WorldGen.SolidTile` this does
/// NOT exclude slopes — port verbatim.
fn solid_collision(world: &World, x: f32, y: f32, w: i32, h: i32) -> bool {
    let x0 = ((x / 16.0) as i32 - 1).clamp(0, world.width - 1);
    let x1 = (((x + w as f32) / 16.0) as i32 + 2).clamp(0, world.width - 1);
    let y0 = ((y / 16.0) as i32 - 1).clamp(0, world.height - 40);
    let y1 = (((y + h as f32) / 16.0) as i32 + 2).clamp(0, world.height - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let Some(t) = world.tile(i, j) else { continue };
            if !t.inactive()
                && t.active()
                && tr_core::tiledata::TILE_SOLID.has(t.kind)
                && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            {
                let tx = (i * 16) as f32;
                let mut ty = (j * 16) as f32;
                let mut th = 16;
                if t.half_brick() {
                    ty += 8.0;
                    th -= 8;
                }
                if x + w as f32 > tx && x < tx + 16.0 && y + h as f32 > ty && y < ty + th as f32 {
                    return true;
                }
            }
        }
    }
    false
}

/// `Collision.WetCollision(Position, Width, Height)`
/// (`Collision.cs:1572-1651`) — returns `(wet, honey, shimmer)`, the
/// Entity flags `NPC.cs:93063-93075` copies. The probe box is the centre
/// shrunk to `min(10, width) × min(height/2, height)` (INTEGER halves);
/// the scan is the padded ±1/+2 range clamped to `maxTilesY - 40`; the
/// FIRST overlapping liquid cell wins (early return) and sets the kind
/// flags from that cell (`tile.honey()`/`tile.shimmer()`,
/// `Tile.cs` — the port's `liquid_kind`); a liquid-less SLOPED tile under
/// liquid counts via the tile above (the `:1628-1646` else arm, honey
/// ELSE-IF shimmer there). The liquid surface drops by
/// `(256 - liquid)/32 * 2` px (`:1610-1612`).
pub fn wet_collision(world: &World, x: f32, y: f32, w: i32, h: i32) -> (bool, bool, bool) {
    let mut honey = false;
    let mut shimmer = false;
    let mut num = 10;
    let mut num2 = h / 2;
    if num > w {
        num = w;
    }
    if num2 > h {
        num2 = h;
    }
    let px = x + (w / 2) as f32 - (num / 2) as f32;
    let py = y + (h / 2) as f32 - (num2 / 2) as f32;
    let x0 = ((x / 16.0) as i32 - 1).clamp(0, world.width - 1);
    let x1 = (((x + w as f32) / 16.0) as i32 + 2).clamp(0, world.width - 1);
    let y0 = ((y / 16.0) as i32 - 1).clamp(0, world.height - 40);
    let y1 = (((y + h as f32) / 16.0) as i32 + 2).clamp(0, world.height - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let Some(t) = world.tile(i, j) else { continue };
            if t.liquid > 0 {
                let tx = (i * 16) as f32;
                let mut ty = (j * 16) as f32;
                let mut th = 16;
                let num5 = (256 - t.liquid as i32) as f32 / 32.0;
                ty += num5 * 2.0;
                th -= (num5 * 2.0) as i32;
                if px + num as f32 > tx
                    && px < tx + 16.0
                    && py + num2 as f32 > ty
                    && py < ty + th as f32
                {
                    if t.liquid_kind() == LiquidKind::Honey {
                        honey = true;
                    }
                    if t.liquid_kind() == LiquidKind::Shimmer {
                        shimmer = true;
                    }
                    return (true, honey, shimmer);
                }
            } else {
                if !t.active() || t.slope() == 0 || j <= 0 {
                    continue;
                }
                let Some(above) = world.tile(i, j - 1) else {
                    continue;
                };
                if above.liquid <= 0 {
                    continue;
                }
                let tx = (i * 16) as f32;
                let ty = (j * 16) as f32;
                if px + num as f32 > tx
                    && px < tx + 16.0
                    && py + num2 as f32 > ty
                    && py < ty + 16.0
                {
                    if above.liquid_kind() == LiquidKind::Honey {
                        honey = true;
                    } else if above.liquid_kind() == LiquidKind::Shimmer {
                        shimmer = true;
                    }
                    return (true, honey, shimmer);
                }
            }
        }
    }
    (false, honey, shimmer)
}

/// `NPC.UpdateNPC_UpdateGravity(out maxFallSpeed)` (`NPC.cs:90878-90955`)
/// for the aiStyle-1 kinds: base `NPC.gravity = 0.3`, `maxFallSpeed = 10`
/// (`:90880-90881`); NONE of the per-type rows (`:90882-90925`: 258/425/
/// 576/577/427/426/541/aiStyle-7) matches a slime kind. The depth factor
/// `num2 = (y/16 - (60 + 10·(maxTilesX/4200)²)) / (worldSurface/6)` clamped
/// [0.25, 1] (`:90926-90936`, the `npc_ai_golem.rs::body_gravity` shape —
/// f32 numerator, f64 divide), then the wet merges: **shimmer 0.15/5.5**
/// (`:90939-90943`), **honey 0.1/4** (`:90944-90948`), water 0.2/7
/// (`:90949-90953`). The live stand-in's dry 0.4/8 pair is invented
/// (audit §1b). The caller applies `vy += gravity` + the cap AFTER the AI
/// (`NPC.cs:90465-90472`).
pub fn slime_gravity(
    y: f32,
    max_tiles_x: i32,
    world_surface: f64,
    wet: bool,
    honey_wet: bool,
    shimmer_wet: bool,
) -> (f32, f32) {
    let mut max_fall = 10.0f32;
    let mut gravity = 0.3f32;
    let num = max_tiles_x as f32 / 4200.0;
    let mut num2 =
        ((y / 16.0 - (60.0 + 10.0 * (num * num))) as f64 / (world_surface / 6.0)) as f32;
    if num2 < 0.25 {
        num2 = 0.25;
    }
    if num2 > 1.0 {
        num2 = 1.0;
    }
    gravity *= num2;
    if wet {
        if shimmer_wet {
            gravity = 0.15;
            max_fall = 5.5;
        } else if honey_wet {
            gravity = 0.1;
            max_fall = 4.0;
        } else {
            gravity = 0.2;
            max_fall = 7.0;
        }
    }
    (gravity, max_fall)
}

/// The family entry — `NPC.AI_001_Slimes` (`NPC.cs:60341-61723`), the
/// ported scope (module header). Writes only the fields the C# arm
/// writes: velocity, position nudges, `ai[0..3]`, `localAI[0]`,
/// `aiAction`, `direction`, `netUpdate`, `frame`/`rotation` (the -999
/// freeze) and the `TargetClosest` tracking fields. It does NOT integrate
/// position or apply gravity — the C# arm ends before the generic
/// `UpdateNPC` tail (`:90465-90503`).
pub fn ai_001_slimes(
    npc: &mut Npc,
    world: &World,
    ctx: &SlimeCtx,
    rng: &mut UnifiedRandom,
) -> SlimeEffects {
    let mut fx = SlimeEffects::default();
    let t = cs_type(npc);
    let (w, h) = npc.size();
    let wi = w as i32;
    let hi = h as i32;
    let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);

    // :60343-61040 — the 75-lantern, the item-content init, the paint tint
    // and the ai[1] > 0 content rows are the A1-5/content slices (module
    // header census). `flag2` (the ore-slime hover, :60740) folds to false.

    // :61041-61045 — the type-1 direction-0 latch.
    if t == 1 && npc.facing == 0 {
        npc.facing = 1;
        npc.net_update = true;
    }
    // :61046-61052 — the ballooned-slime freeze: frame.Y = 0,
    // frameCounter = 0, rotation = 0, RETURN (zero draws).
    if npc.ai[0] == -999.0 {
        npc.frame = 0;
        npc.rotation = 0.0;
        fx.reset_frame_counter = true;
        return fx;
    }
    // :61053-61059 — the 244 rainbow light/colour: unported kind, the
    // npc_lights seam (no arm-level draw; the colour is disco-driven).

    // :61060-61068 — flag3, the "aggressive" gate.
    let mut flag3 = !world.day_time
        || npc.hp != npc.max_hp
        || (npc.y as f64) > world.world_surface * 16.0
        || world.slime_rain;
    if remix && t == 59 && npc.hp == npc.max_hp {
        flag3 = false;
    }
    // :61069-61079 — CorruptSlime: always aggressive; Next(30) EVERY tick
    // (the dust body is the DustRng skip; the gate draw is the arm's).
    if t == 81 {
        flag3 = true;
        let _dust_gate = rng.next_max(30) == 0;
    }
    // :61080-61083 — the Slimer/ToxicSludge proximity aggro (unported
    // kinds; the row is kept whole-arm). Vector2.Distance <= 200.
    if (t == 377 || t == 446) && npc.target != 255 {
        let p = target_of(npc, ctx.players);
        if !p.dead {
            let (cx, cy) = npc.center();
            let pcx = p.x + p.width as f32 * 0.5;
            let pcy = p.y + p.height as f32 * 0.5;
            if sqrt_f32(pcx - cx, pcy - cy) <= 200.0 && !ctx.wet {
                flag3 = true;
            }
        }
    }
    // :61084-61095 — Crimslime / HoppinJack / GoldenSlime always aggro.
    if t == 183 || t == 304 || t == 667 {
        flag3 = true;
    }
    // :61096-61100 — the RainbowSlime double-step (unported kind).
    if t == 244 {
        flag3 = true;
        npc.ai[0] += 2.0;
    }
    // :61101-61108 — IceSlime: Next(10) EVERY tick (dust body skipped).
    if t == 147 {
        let _dust_gate = rng.next_max(10) == 0;
    }
    // :61109-61127 — GoldenSlime light (npc_lights seam) + the
    // speed-short-circuited dust gate: `Length() > 1 || Next(4) != 0` —
    // speed > 1 draws NOTHING.
    if t == 667 && (sqrt_f32(npc.vx, npc.vy) > 1.0 || rng.next_max(4) != 0) {
        // dust body — the DustRng skip.
    }
    // :61128-61150 — the 676 ShimmerSlime light/dust (W4-f47 keep-one 144:
    // the kind landed, the censused skip retired). The LIGHT half is the
    // RNG-free draw-side feed (`npc_lights.rs`, torch 23 = the shimmer
    // torch's mouseTextColor pulse). The dust gate keeps the EXACT C#
    // short-circuit: `(speed > 1 && Next(3) == 0) || Next(5) == 0` —
    // speed > 1 draws Next(3) first; only on its failure (or speed <= 1,
    // which draws NOTHING for the first term) does Next(5) run.
    if t == 676 {
        let gate = (sqrt_f32(npc.vx, npc.vy) > 1.0 && rng.next_max(3) == 0)
            || rng.next_max(5) == 0;
        if gate {
            // :61133-61148 — the roll census (4 NextFloat + the allocator's
            // own 6): NextVector2FromRectangle(Hitbox) = 2 NextFloat
            // (Utils.cs:1256-1258, the INTEGER-truncated top-left), the
            // hue NextFloat, the scale NextFloat. The velocity derives
            // from the PRE-netOffset position vs base.Center (`:61139-
            // 61145`); the colour needs `Main.timeForVisualEffects` and
            // the pool — the request rides `wants_shimmer_slime_dust` to
            // the sim drain (the gallop-dust seam; the allocator draws
            // land post-tick, the recorded pool-pass deviation).
            let px = npc.x as i32 as f32 + rng.next_float() * wi as f32;
            let py = npc.y as i32 as f32 + rng.next_float() * hi as f32;
            let hue_jitter = rng.next_float();
            let scale = rng.next_float() * 0.3 + 0.5;
            let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let mut vx = (px - cx) * 0.1;
            let mut vy = (py - cy) * 0.1;
            vx *= 0.25;
            if vy > 0.0 {
                vy *= -1.0;
            }
            npc.wants_shimmer_slime_dust.push(ShimmerSlimeDust {
                x: px,
                y: py,
                vx,
                vy,
                hue_jitter,
                scale,
            });
        }
    }

    // :61151-61211 — the WHOLE SpikedIceSlime block.
    if t == 184 {
        // :61153-61160 — Next(8) EVERY tick (dust body skipped).
        let _dust_gate = rng.next_max(8) == 0;
        flag3 = true;
        // :61162-61165 — the shot cooldown.
        if npc.local_ai[0] > 0.0 {
            npc.local_ai[0] -= 1.0;
        }
        // :61166-61210 — the lob gate: !wet && the target alive/aggro.
        let p = target_of(npc, ctx.players);
        if !ctx.wet && p.active && !p.dead && !p.npc_type_no_aggro {
            // :61168-61171 — the aim: own centre (FLOAT half), the
            // player's centre X but TOP Y, f64 sqrt.
            let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let mut num30 = p.x + p.width as f32 * 0.5 - cx;
            let mut num31 = p.y - cy;
            let num32 = sqrt_f32(num30, num31);
            let can_hit = crate::projectile::can_hit(
                world,
                npc.x,
                npc.y,
                w,
                h,
                p.x,
                p.y,
                p.width as f32,
                p.height as f32,
            );
            if ctx.expert_mode && num32 < 120.0 && can_hit && npc.vy == 0.0 {
                // :61172-61192 — the expert five-spike fan.
                npc.ai[0] = -40.0;
                if npc.vy == 0.0 {
                    npc.vx *= 0.9;
                }
                if ctx.net_mode != 1 && npc.local_ai[0] == 0.0 {
                    for k in 0..5 {
                        let mut sx = (k - 2) as f32;
                        let mut sy = -4.0f32;
                        sx *= 1.0 + rng.next_range(-50, 51) as f32 * 0.005;
                        sy *= 1.0 + rng.next_range(-50, 51) as f32 * 0.005;
                        let (nx, ny) = normalize_xna(sx, sy);
                        let speed = 4.0 + rng.next_range(-50, 51) as f32 * 0.01;
                        fx.shots.push(SlimeShot {
                            x: cx,
                            y: cy,
                            vx: nx * speed,
                            vy: ny * speed,
                            kind: 174,
                            damage: 9,
                            knockback: 0.0,
                        });
                        npc.local_ai[0] = 30.0;
                    }
                }
            } else if num32 < 200.0 && can_hit && npc.vy == 0.0 {
                // :61193-61209 — the classic lob: ONE Next(0, 200)
                // Y-jitter draw, 4.5-speed normalise.
                npc.ai[0] = -40.0;
                if npc.vy == 0.0 {
                    npc.vx *= 0.9;
                }
                if ctx.net_mode != 1 && npc.local_ai[0] == 0.0 {
                    num31 = p.y - cy - rng.next_range(0, 200) as f32;
                    let scale = 4.5 / sqrt_f32(num30, num31);
                    num30 *= scale;
                    num31 *= scale;
                    npc.local_ai[0] = 50.0;
                    fx.shots.push(SlimeShot {
                        x: cx,
                        y: cy,
                        vx: num30,
                        vy: num31,
                        kind: 174,
                        damage: 9,
                        knockback: 0.0,
                    });
                }
            }
        }
    }
    // :61212-61439 — the 535/658/659/204 shooter blocks: unported kinds
    // (the 535 SpikedSlime is the King Slime's expert minion — its block
    // lands with that kind; census in the module header). Their flag3
    // rows keep the machine whole for the future kinds:
    if t == 535 || t == 658 || t == 659 || t == 204 {
        flag3 = true;
    }
    // :61440-61450 — the 377/446 friendly latch (localAI[2] < 90):
    // unported kinds, deferred with their kinds.
    // :61451-61458 — the 59 light + torch dust: the npc_lights/DustRng
    // seams (no arm-level draw).

    // :61459-61462 — the ai[2] countdown (the stuck-flip's 200-tick gate).
    if npc.ai[2] > 1.0 {
        npc.ai[2] -= 1.0;
    }
    // :61463-61510 — the wet tail.
    if ctx.wet {
        if npc.collide_y {
            npc.vy = -2.0;
        }
        // :61469-61473 — the wet stuck-flip: ai[3] == position.X while
        // rising → flip + ai[2] = 200 (TICKS, not the stand-in's seconds).
        if npc.vy < 0.0 && npc.ai[3] == npc.x {
            npc.facing *= -1;
            npc.ai[2] = 200.0;
        }
        if npc.vy > 0.0 {
            npc.ai[3] = npc.x;
        }
        if t == 59 && !remix {
            // :61478-61493 — the LavaSlime buoyancy: the directionY lift
            // and the -10 cap.
            if npc.vy > 2.0 {
                npc.vy *= 0.9;
            } else if npc.direction_y < 0 {
                npc.vy -= 0.8;
            }
            npc.vy -= 0.5;
            if npc.vy < -10.0 {
                npc.vy = -10.0;
            }
        } else {
            // :61494-61505 — the generic buoyancy.
            if npc.vy > 2.0 {
                npc.vy *= 0.9;
            }
            npc.vy -= 0.5;
            if npc.vy < -4.0 {
                npc.vy = -4.0;
            }
        }
        // :61506-61509 — the re-aim at the countdown's last tick
        // (the C#'s non-short-circuit `&` on two pure reads).
        if npc.ai[2] == 1.0 && flag3 {
            aim(npc, world, ctx);
        }
    }
    // :61511-61517 — aiAction reset + the init latch.
    npc.ai_action = 0;
    if npc.ai[2] == 0.0 {
        npc.ai[0] = -100.0;
        npc.ai[2] = 1.0;
        aim(npc, world, ctx);
    }
    // :61518-61703 — the grounded machine.
    if npc.vy == 0.0 {
        // :61520-61523 — the landing unstick nudge.
        if npc.collide_y && npc.old_vy != 0.0 && solid_collision(world, npc.x, npc.y, wi, hi) {
            npc.x -= npc.vx + npc.facing as f32;
        }
        // :61524-61529 — the grounded stuck-flip + the anchor clear.
        if npc.ai[3] == npc.x {
            npc.facing *= -1;
            npc.ai[2] = 200.0;
        }
        npc.ai[3] = 0.0;
        if npc.ai[1] == 3609.0 {
            // :61530-61541 — the ConveyorSlime row (content-gated; the
            // branch is kept verbatim for the A1-5 wave).
            if npc.facing < 0 {
                npc.vx -= 0.1;
            } else {
                npc.vx += 0.1;
            }
            npc.vx = npc.vx.clamp(-2.5, 2.5);
        } else {
            // :61542-61549 — the landing brake: ×0.8 with the f64 ±0.1
            // deadband (the stand-in's ×0.6 is invented).
            npc.vx *= 0.8;
            if (npc.vx as f64) > -0.1 && (npc.vx as f64) < 0.1 {
                npc.vx = 0.0;
            }
        }
        // :61550-61554 — the cadence tick (flag3 doubles it).
        if flag3 {
            npc.ai[0] += 1.0;
        }
        npc.ai[0] += 1.0;
        // :61555-61602 — the per-type cadence bonuses.
        if t == 59 && !remix {
            npc.ai[0] += 2.0;
        }
        if t == 71 {
            npc.ai[0] += 3.0;
        }
        if t == 667 {
            npc.ai[0] += 3.0;
        }
        if t == 138 {
            npc.ai[0] += 2.0;
        }
        if t == 183 {
            npc.ai[0] += 1.0;
        }
        if t == 658 {
            npc.ai[0] += 5.0;
        }
        if t == 659 {
            npc.ai[0] += 3.0;
        }
        if t == 304 {
            // :61585 — INTEGER life/lifeMax division (0 damaged, 1 full)
            // — the int-vs-float class, port verbatim.
            let num49 = ((1 - npc.hp / npc.max_hp) * 10) as f32;
            npc.ai[0] += num49;
        }
        if t == 377 || t == 446 {
            npc.ai[0] += 3.0;
        }
        if t == 81 {
            // :61592-61602 — the C#'s `scale >= 0f` is true for every
            // live scale; port verbatim.
            if npc.scale >= 0.0 {
                npc.ai[0] += 4.0;
            } else {
                npc.ai[0] += 1.0;
            }
        }
        // :61603-61624 — the tier windows.
        let mut num50 = -1000.0f32;
        if t == 659 {
            num50 = -500.0;
        }
        if t == 667 {
            num50 = -400.0;
        }
        let mut num51 = 0;
        if npc.ai[0] >= 0.0 {
            num51 = 1;
        }
        if npc.ai[0] >= num50 && npc.ai[0] <= num50 * 0.5 {
            num51 = 2;
        }
        if npc.ai[0] >= num50 * 2.0 && npc.ai[0] <= num50 * 1.5 {
            num51 = 3;
        }
        if num51 > 0 {
            // :61625-61698 — the hop.
            npc.net_update = true;
            if flag3 && npc.ai[2] == 1.0 {
                aim(npc, world, ctx);
            }
            if num51 == 3 {
                npc.vy = -8.0;
                if t == 59 && !remix {
                    npc.vy -= 2.0;
                }
                npc.vx += 3.0 * npc.facing as f32;
                if t == 59 && !remix {
                    npc.vx += 0.5 * npc.facing as f32;
                }
                npc.ai[0] = -200.0;
                npc.ai[3] = npc.x;
            } else {
                npc.vy = -6.0;
                npc.vx += 2.0 * npc.facing as f32;
                if t == 59 && !remix {
                    npc.vx += 2.0 * npc.facing as f32;
                }
                npc.ai[0] = -120.0;
                if num51 == 1 {
                    npc.ai[0] += num50;
                } else {
                    npc.ai[0] += num50 * 2.0;
                }
            }
            if t == 659 {
                npc.vy *= 1.6;
                npc.vx *= 1.2;
            }
            if t == 685 {
                // :61670-61678 — the BoundTownSlime hop: half height,
                // fifth speed, ONE Next(2) direction-flip draw per hop.
                npc.vy *= 0.5;
                npc.vx *= 0.2;
                if rng.next_max(2) == 0 {
                    npc.facing *= -1;
                }
            }
            if t == 141 {
                npc.vy *= 1.3;
                npc.vx *= 1.2;
            }
            if t == 377 || t == 446 {
                // :61684-61697 — the Slimer rows + the ceiling check.
                npc.vy *= 0.9;
                npc.vx *= 0.6;
                if flag3 {
                    npc.facing = -npc.facing;
                    npc.vx *= -1.0;
                }
                let (cx, cy) = npc.center();
                if solid_tile(world, (cx / 16.0) as i32, (cy / 16.0) as i32 - 1)
                    && 0.0 - npc.vy + h > 16.0
                {
                    npc.vy = -(16.0 - h);
                }
            }
        } else if npc.ai[0] >= -30.0 {
            // :61699-61702 — the wind-up frame flag (FindFrame reads it).
            npc.ai_action = 1;
        }
    } else if npc.target < 255
        && ((npc.facing == 1 && npc.vx < 3.0) || (npc.facing == -1 && npc.vx > -3.0))
    {
        // :61704-61722 — the air steer.
        if npc.collide_x && npc.vx.abs() == 0.2 {
            // :61706-61709 — the 1.4-px wall nudge.
            npc.x -= 1.4 * npc.facing as f32;
        }
        if npc.collide_y && npc.old_vy != 0.0 && solid_collision(world, npc.x, npc.y, wi, hi) {
            npc.x -= npc.vx + npc.facing as f32;
        }
        // :61714-61721 — accelerate toward the facing under the 3 cap,
        // else damp ×0.93 (the f64 ±0.01 comparisons are the C#'s).
        if (npc.facing == -1 && (npc.vx as f64) < 0.01)
            || (npc.facing == 1 && (npc.vx as f64) > -0.01)
        {
            npc.vx += 0.2 * npc.facing as f32;
        } else {
            npc.vx *= 0.93;
        }
    }
    fx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

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
        World::new("t".into(), 200, 120)
    }

    /// A blue slime (24×18, `NPC.cs:8574-8575`) at the surface in daylight
    /// at full hp — `flag3 == false` (`:61061`).
    fn slime(kind: NpcKind) -> Npc {
        let mut n = Npc::new(kind, 160.0, 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[2] = 1.0; // post-init
        n.target = 255; // no air-steer unless a test sets it
        n
    }

    fn ctx_classic<'a>(players: &'a [TargetCandidate]) -> SlimeCtx<'a> {
        SlimeCtx {
            players,
            expert_mode: false,
            net_mode: 0,
            ticker: 7,
            wet: false,
            honey_wet: false,
            shimmer_wet: false,
        }
    }

    fn day_world() -> World {
        let mut w = air();
        w.day_time = true;
        w.world_surface = 60.0; // the slime at y=16 (tile 1) is above it
        w.slime_rain = false;
        w
    }

    /// `NPC.cs:61511-61664` — the cadence is the ai[0] tick counter and
    /// draws ZERO RNG: a daylight full-hp surface slime ticks +1 per
    /// grounded tick and hops at 0 with vy=-6, vx += 2·direction,
    /// ai[0] = -120 + (-1000) = -1120. No sine, no distance gate.
    #[test]
    fn s1_d_3_slime_the_cadence_is_zero_rng_and_hops_at_tier_one() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)]; // far away — NO distance gate
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -1.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        let fx = ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(fx, SlimeEffects::default());
        assert_eq!(n.vy, -6.0, "tier-1 hop (:61649)");
        assert_eq!(n.vx, 2.0, "vx += 2·direction (:61650)");
        assert_eq!(n.ai[0], -1120.0, "ai[0] = -120 + num50 (:61655-61658)");
        assert!(n.net_update, "(:61627)");
        assert_eq!(rng, mirror, "the cadence draws NOTHING");
    }

    /// `NPC.cs:61550-61554` — flag3 (night) doubles the tick; the tier-2
    /// window [-1000, -500] resets ai[0] = -120 + 2·num50 = -2120; the
    /// tier-3 window [-2000, -1500] hops vy=-8, vx += 3·dir, ai[0] = -200
    /// and stamps ai[3] = position.X (`:61632-61646`).
    #[test]
    fn s1_d_3_slime_the_tier_windows_and_resets_match() {
        let mut w = day_world();
        w.day_time = false; // flag3
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        // tier 2: -1002 + 2 (flag3 double) = -1000 → the window's edge.
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -1002.0;
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vy, -6.0);
        assert_eq!(n.ai[0], -2120.0, "tier 2: -120 + 2·num50 (:61662)");
        // tier 3: -2002 + 2 = -2000 → the window's edge.
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -2002.0;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vy, -8.0, "(:61634)");
        assert_eq!(n.vx, 3.0, "(:61639)");
        assert_eq!(n.ai[0], -200.0, "(:61644)");
        assert_eq!(n.ai[3], n.x, "(:61645)");
        // tier 1 via ai[0] >= 0: -2 + 2 = 0.
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -2.0;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.ai[0], -1120.0, "tier 1: -120 + num50");
    }

    /// `NPC.cs:61555-61558`/`:61635-61654` — the LavaSlime rows: +2/tick
    /// cadence, tier-1 vx += 4·direction, tier-3 vy = -10 and
    /// vx += 3.5·direction.
    #[test]
    fn s1_d_3_slime_the_lava_slime_rows_match() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::LavaSlime);
        n.ai[0] = -3.0; // +1 +2 = 0 → tier 1
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vy, -6.0);
        assert_eq!(n.vx, 4.0, "2·dir + 2·dir (:61650-61654)");
        let mut n = slime(NpcKind::LavaSlime);
        n.ai[0] = -2003.0; // +3 = -2000 → tier 3
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vy, -10.0, "-8 - 2 (:61634-61638)");
        assert_eq!(n.vx, 3.5, "3·dir + 0.5·dir (:61639-61643)");
    }

    /// `NPC.cs:61544-61548` — the landing brake ×0.8 with the f64 ±0.1
    /// deadband: 0.0625 → 0.05 → snap 0; 0.25 → 0.2 stays.
    #[test]
    fn s1_d_3_slime_the_landing_brake_deadband_is_f64() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -50.0; // no hop this tick
        n.vx = 0.0625;
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vx, 0.0, "0.0625·0.8 = 0.05 < 0.1 → snap");
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -50.0;
        n.vx = 0.25;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vx, 0.25f32 * 0.8, "0.2 stays (the ×0.6 is gone)");
    }

    /// `NPC.cs:61524-61529` — the grounded stuck-flip: ai[3] == position.X
    /// → direction flips and ai[2] = 200 TICKS; ai[3] then clears to 0.
    #[test]
    fn s1_d_3_slime_the_grounded_stuck_flip_writes_ticks() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -50.0;
        n.ai[3] = n.x;
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.facing, -1, "direction *= -1 (:61526)");
        assert_eq!(n.ai[3], 0.0, "the anchor clears (:61529)");
        // ai[2] was 200, the countdown does not run again this tick.
        assert_eq!(n.ai[2], 200.0, "ai[2] = 200 ticks (:61527)");
    }

    /// `NPC.cs:61463-61505` — the wet tail: collideY → vy = -2, then the
    /// -0.5/tick buoyancy (cap -4); the LavaSlime arm lifts -0.8 more on
    /// directionY < 0 and caps at -10 (`:61478-61493`).
    #[test]
    fn s1_d_3_slime_the_wet_tail_and_lava_buoyancy_match() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let mut ctx = ctx_classic(&players);
        ctx.wet = true;
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -50.0;
        n.collide_y = true;
        n.ai[3] = 12345.0; // not stuck
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vy, -2.5, "-2 then -0.5 (:61467/:61500)");
        let mut n = slime(NpcKind::LavaSlime);
        n.ai[0] = -50.0;
        n.collide_y = true;
        n.direction_y = -1;
        n.ai[3] = 12345.0;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        // f32 order: (-2 - 0.8) - 0.5 — the intermediate -2.8 rounds to
        // -2.79999995, so the result is NOT bit-equal to the literal -3.3.
        assert_eq!(n.vy, (-2.0f32 - 0.8) - 0.5, "-2 - 0.8 - 0.5 (:61486-61488)");
    }

    /// `NPC.cs:61704-61721` — the air steer: +0.2·direction under the 3
    /// cap, ×0.93 against it, and the collideX |vx| == 0.2 → 1.4-px nudge.
    #[test]
    fn s1_d_3_slime_the_air_steer_and_wall_nudge_match() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.target = 0;
        n.vy = -1.0; // airborne
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(42);
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vx, 0.2, "+0.2·direction (:61716)");
        // against the facing: damp.
        let mut n = slime(NpcKind::Slime);
        n.target = 0;
        n.facing = -1;
        n.vy = -1.0;
        n.vx = 1.0;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vx, 1.0f32 * 0.93, "(:61720)");
        // the nudge: collideX && |vx| == 0.2.
        let mut n = slime(NpcKind::Slime);
        n.target = 0;
        n.vy = -1.0;
        n.vx = 0.2;
        n.collide_x = true;
        let x0 = n.x;
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.x, x0 - 1.4, "(:61708)");
        assert_eq!(n.vx, 0.2f32 + 0.2, "then the steer accelerates");
    }

    /// `NPC.cs:90880-90953` — gravity: 0.3 × the depth factor clamped
    /// [0.25, 1], cap 10; wet water 0.2/7, honey 0.1/4, shimmer 0.15/5.5
    /// (the two merges the live arm lacks). maxTilesX 4200 → num = 1,
    /// the offset row 70; worldSurface 300 → the factor's denominator 50.
    #[test]
    fn s1_d_3_slime_gravity_is_depth_scaled_with_the_two_merges() {
        // y = 16·70 = 1120 → num2 = 0 → clamp 0.25 → 0.075.
        assert_eq!(slime_gravity(1120.0, 4200, 300.0, false, false, false), (0.075, 10.0));
        // y = 16·120 = 1920 → num2 = 1 → 0.3.
        assert_eq!(slime_gravity(1920.0, 4200, 300.0, false, false, false), (0.3, 10.0));
        // the wet merges override the depth factor entirely.
        assert_eq!(slime_gravity(1120.0, 4200, 300.0, true, false, false), (0.2, 7.0));
        assert_eq!(slime_gravity(1120.0, 4200, 300.0, true, true, false), (0.1, 4.0));
        assert_eq!(slime_gravity(1120.0, 4200, 300.0, true, false, true), (0.15, 5.5));
        // shimmer wins over honey (the else-if order, :90939-90948).
        assert_eq!(slime_gravity(1120.0, 4200, 300.0, true, true, true), (0.15, 5.5));
    }

    /// `NPC.cs:61512-61517` — the init latch: ai[2] == 0 → ai[0] = -100,
    /// ai[2] = 1, TargetClosest (the target slot fills).
    #[test]
    fn s1_d_3_slime_the_init_latch_aims_and_seeds() {
        let w = day_world();
        let players = [cand(400.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.ai[2] = 0.0;
        n.ai[0] = 0.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.target, 0, "TargetClosest (:61516)");
        assert_eq!(n.facing, 1, "the player is to the right");
        // ai[0] = -100 then the grounded +1 → -99.
        assert_eq!(n.ai[0], -99.0);
        assert_eq!(rng, mirror, "TargetClosest is RNG-free");
    }

    /// `NPC.cs:61046-61052` — the ballooned freeze: frame/rotation zero,
    /// the frameCounter reset request, EARLY RETURN, zero draws.
    #[test]
    fn s1_d_3_slime_the_ballooned_freeze_returns_without_draws() {
        let w = day_world();
        let players = [cand(400.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::Slime);
        n.ai[0] = -999.0;
        n.frame = 2;
        n.rotation = 0.5;
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        let fx = ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert!(fx.reset_frame_counter);
        assert_eq!(n.frame, 0);
        assert_eq!(n.rotation, 0.0);
        assert_eq!(n.vx, 1.0, "no machine tick");
        assert_eq!(rng, mirror);
    }

    /// `NPC.cs:61101` — the IceSlime dust gate draws Next(10) EVERY tick
    /// (even airborne, even with no target) — the RNG-order contract
    /// S1-A-10: the draw is the gate, the dust body is the DustRng skip.
    #[test]
    fn s1_d_3_slime_the_ice_slime_dust_gate_draws_every_tick() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::IceSlime);
        n.vy = -1.0; // airborne — the gate still draws
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        let _ = mirror.next_max(10); // the ONE cited draw (:61101)
        assert_eq!(rng, mirror, "exactly one Next(10)");
    }

    /// W4-f47 keep-one 144 — `NPC.cs:61131`: the 676 gate is
    /// `(speed > 1 && Next(3) == 0) || Next(5) == 0`. A SLOW slime
    /// (speed <= 1) draws NOTHING for the first term — exactly ONE
    /// Next(5); on a hit the body rolls 4 NextFloat (the hitbox pair, the
    /// hue jitter, the scale) and queues the request. The mirror replays
    /// the cited draws in order and compares the full stream state.
    #[test]
    fn w4_f47_the_shimmer_slime_slow_gate_draws_one_next5_then_the_quad() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        // Find a seed whose Next(5) hits 0 (a 1-in-5 gate) with the slime
        // at rest (speed 0 — the first term draws nothing).
        let mut seed = 0u64;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(5) == 0 {
                break;
            }
            seed += 1;
        }
        let mut n = slime(NpcKind::ShimmerSlime);
        n.ai[0] = -50.0; // mid-cadence: no hop this tick
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(mirror.next_max(5), 0, "the gate hit");
        // The body: NextVector2FromRectangle(Hitbox) = 2 NextFloat over
        // the INTEGER-truncated top-left (160, 16) 24x18 (`Utils.cs:1258`),
        // then the hue jitter + the scale NextFloat.
        let fx = mirror.next_float();
        let fy = mirror.next_float();
        let hue = mirror.next_float();
        let sc = mirror.next_float();
        assert_eq!(rng, mirror, "Next(5) + 4 NextFloat, nothing else");
        assert_eq!(n.wants_shimmer_slime_dust.len(), 1);
        let d = n.wants_shimmer_slime_dust[0];
        assert_eq!(d.x, 160.0 + fx * 24.0);
        assert_eq!(d.y, 16.0 + fy * 18.0);
        assert_eq!(d.hue_jitter, hue);
        assert_eq!(d.scale, sc * 0.3 + 0.5);
        // The velocity: (pos - Center) * 0.1, X ×0.25, positive Y flipped
        // (`:61139-61145`); Center = (172, 25) for the 24x18 box at (160,16).
        let vx = (d.x - 172.0) * 0.1 * 0.25;
        let mut vy = (d.y - 25.0) * 0.1;
        if vy > 0.0 {
            vy *= -1.0;
        }
        assert_eq!(d.vx, vx);
        assert_eq!(d.vy, vy);
    }

    /// `NPC.cs:61131` — a FAST slime (speed > 1) draws Next(3) FIRST; when
    /// it hits 0 the `||` short-circuits — NO Next(5). The stream twin
    /// pins the order: Next(3) then the 4-NextFloat body.
    #[test]
    fn w4_f47_the_shimmer_slime_fast_gate_short_circuits_the_next5() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut seed = 0u64;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(3) == 0 {
                break;
            }
            seed += 1;
        }
        let mut n = slime(NpcKind::ShimmerSlime);
        n.ai[0] = -50.0;
        n.vx = 2.0; // speed 2 > 1 — the first term draws
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(mirror.next_max(3), 0, "the speed gate hit");
        for _ in 0..4 {
            let _ = mirror.next_float();
        }
        assert_eq!(rng, mirror, "Next(3) + 4 NextFloat — the Next(5) never ran");
        assert_eq!(n.wants_shimmer_slime_dust.len(), 1);
    }

    /// `NPC.cs:61131` — both gates failing: the fast slime draws Next(3)
    /// (non-zero) THEN Next(5) (non-zero) — two draws, no body, no request.
    #[test]
    fn w4_f47_the_shimmer_slime_double_miss_draws_both_gates_only() {
        let w = day_world();
        let players = [cand(10000.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut seed = 0u64;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(3) != 0 && probe.next_max(5) != 0 {
                break;
            }
            seed += 1;
        }
        let mut n = slime(NpcKind::ShimmerSlime);
        n.ai[0] = -50.0;
        n.vx = 2.0;
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_ne!(mirror.next_max(3), 0);
        assert_ne!(mirror.next_max(5), 0);
        assert_eq!(rng, mirror, "exactly the two gate draws");
        assert!(n.wants_shimmer_slime_dust.is_empty());
    }

    /// `NPC.cs:61193-61208` — the classic 184 lob: ai[0] = -40, vx ×0.9,
    /// ONE Next(0, 200) Y-jitter draw (after the every-tick Next(8)),
    /// localAI[0] = 50, one 174 shot at 4.5 speed, damage 9.
    #[test]
    fn s1_d_3_slime_the_spiked_ice_lob_draws_one_jitter() {
        let w = day_world();
        // the player 100 px right, same row → dist < 200, CanHit in air.
        let players = [cand(260.0, 16.0)];
        let ctx = ctx_classic(&players);
        let mut n = slime(NpcKind::SpikedIceSlime);
        n.ai[0] = -50.0;
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let fx = ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        // the shooter writes -40 (:61195); 184 sets flag3 = true
        // (:61161), so the grounded cadence double-ticks +2 (:61550-61554).
        assert_eq!(n.ai[0], -38.0);
        // ×0.9 (:61198) then the landing brake ×0.8 (:61544).
        assert_eq!(n.vx, 1.0f32 * 0.9 * 0.8);
        assert_eq!(n.local_ai[0], 50.0, "(:61206)");
        assert_eq!(fx.shots.len(), 1);
        assert_eq!(fx.shots[0].kind, 174);
        assert_eq!(fx.shots[0].damage, 9);
        let jitter = mirror.next_max(8); // the dust gate (:61153)
        let _ = jitter;
        let y_jitter = mirror.next_range(0, 200); // (:61202)
        assert_eq!(rng, mirror, "exactly Next(8) + Next(0,200)");
        // the shot's speed is the 4.5 normalise of the jittered aim.
        let s = &fx.shots[0];
        let speed = sqrt_f32(s.vx, s.vy);
        assert!((speed - 4.5).abs() < 1e-4, "4.5-speed normalise");
        let _ = y_jitter;
    }

    /// `NPC.cs:61172-61191` — the expert fan: 5 spikes × 3 draws = 15
    /// draws after the Next(8) gate; localAI[0] = 30; the cooldown blocks
    /// the next volley (zero shooter draws while it counts down).
    #[test]
    fn s1_d_3_slime_the_expert_fan_draws_fifteen_and_cools_down() {
        let w = day_world();
        let players = [cand(220.0, 16.0)]; // ~60 px → < 120
        let mut ctx = ctx_classic(&players);
        ctx.expert_mode = true;
        let mut n = slime(NpcKind::SpikedIceSlime);
        n.ai[0] = -50.0;
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let fx = ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert_eq!(fx.shots.len(), 5, "the five-spike fan (:61181)");
        assert_eq!(n.local_ai[0], 30.0, "(:61189)");
        let _ = mirror.next_max(8);
        for _ in 0..15 {
            let _ = mirror.next_range(-50, 51);
        }
        assert_eq!(rng, mirror, "Next(8) + 15 fan draws");
        // next tick: the cooldown counts down, NO shooter draws.
        let mirror2 = rng.clone();
        let fx2 = ai_001_slimes(&mut n, &w, &ctx, &mut rng);
        assert!(fx2.shots.is_empty());
        assert_eq!(n.local_ai[0], 29.0);
        let mut m2 = mirror2;
        let _ = m2.next_max(8); // only the dust gate
        assert_eq!(rng, m2);
    }

    /// `Collision.cs:1572-1651` — WetCollision: a honey cell under the
    /// body sets (true, true, false); a shimmer cell (true, false, true);
    /// plain water (true, false, false); dry (false, false, false).
    #[test]
    fn s1_d_3_slime_wet_collision_reports_the_liquid_kind() {
        let mut w = air();
        // the slime box 24×18 at (160, 16): centre probe around tile
        // (10, 1). Fill tile (10, 1) with liquid.
        let idx = w.index(10, 1);
        w.tiles[idx].liquid = 255;
        assert_eq!(wet_collision(&w, 160.0, 16.0, 24, 18), (true, false, false));
        w.tiles[idx].set_liquid_kind(LiquidKind::Honey);
        assert_eq!(wet_collision(&w, 160.0, 16.0, 24, 18), (true, true, false));
        w.tiles[idx].set_liquid_kind(LiquidKind::Shimmer);
        assert_eq!(wet_collision(&w, 160.0, 16.0, 24, 18), (true, false, true));
        w.tiles[idx].liquid = 0;
        assert_eq!(wet_collision(&w, 160.0, 16.0, 24, 18), (false, false, false));
    }
}
