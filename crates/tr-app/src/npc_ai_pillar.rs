//! S1-D-3 (B-6a) — the Lunar Pillar machine, `NPC.AI()`'s
//! `else if (this.aiStyle == 94)` block (`NPC.cs:38968-39383`), ported as
//! pure fns + a Ctx/Effects seam. WIRED by W4-c2 (`npc.rs::ai_094_pillar_family`
//! on the style axis + the `world_sim/npcs.rs` death-pass intercept); the
//! original B-6a landing was code-only (the witness tests WRITTEN, NOT RUN),
//! the wiring wave is code-only too.
//!
//! # Identity census (verified against the C#, not the audit's guesses)
//!
//! | Pillar | NPCID | NPCID.cs | SetDefaults (`aiStyle = 94`) |
//! | --- | --- | --- | --- |
//! | Solar | **517** `LunarTowerSolar` | `:11711` | `NPC.cs:15256` (130×270, 20000 hp, dmg 0, def 20) |
//! | Vortex | **422** `LunarTowerVortex` | `:11689` | `NPC.cs:15179` |
//! | Nebula | **507** `LunarTowerNebula` | `:11687` | `NPC.cs:15108` |
//! | Stardust | **493** `LunarTowerStardust` | `:11659` | `NPC.cs:14995` |
//!
//! The audit's id table (`53-audit-pillars.md` §0) matches. The port's
//! `NpcKind::{Solar,Vortex,Nebula,Stardust}Tower` already claim aiStyle 94
//! on the style axis (`npc/data.rs:2017`) and carry the 130×270 box
//! (`npc/data.rs:2312`); their NET ids are the pre-existing 113/114/115/146
//! collision (audit §0, the S1-F-1 family — NOT fixed here).
//!
//! **SolarGoop 519** (`NPCID.cs:11715`): the audit's port-spec guessed an
//! "aiStyle-3 fighter row" — WRONG. `SetDefaults` case 519
//! (`NPC.cs:15409-15423`) is **aiStyle 99** (26×26, dmg 120, def 0,
//! lifeMax 1, noGravity, noTileCollide, no hit/death sound, kbResist 0);
//! the style-99 arm (`NPC.cs:39927-39990`) is the goop's whole machine:
//! the landing latch (`vy == 0 && ai[0] == 0` → `ai[0] = 1`, `:39929-39935`),
//! the 5-tick pinned dissolve (`position = oldPosition`, `HitEffect(0,9999)`
//! + `active = false` at `ai[1] >= 5`, `:39936-39947`), the 0.2 gravity /
//! 12 cap (`:39948-39952`), the `velocity.ToRotation() - π/2` spin
//! (`:39953`) and the dust trail (`:39958-39989`). The goop KIND + machine
//! are a follow-up census row; this module emits the spawn REQUEST
//! (vanilla id 519, bottom-anchored, with the C#'s launch velocity) so the
//! wiring wave needs no re-derivation.
//!
//! # What the live stand-in does wrong (audit §2.1, evidence)
//!
//! `npc.rs:12553-12606` (`update_ancient_cultist`, the towers' dispatch at
//! `npc.rs:3936-3940`) is an invented ~50-line body: it counts **ai[2] as
//! a 180-tick self-kill timer** (ai[2] is the C#'s dissolve FLAG, set only
//! by `checkDead` `NPC.cs:78080-78086`), never reads the shield state
//! (`world.tower_shield_*` — pillars killable from tick 1), fires the 579
//! ritual at the player centre EVERY tick (no ai[1] cooldown, no count
//! gates, no tile scan, no 578 branch) and discards `rng` (`:12605`).
//! This module is the faithful replacement; W4-c2 retired the stand-in:
//! the style axis claims 94 (`npc.rs::ai_094_pillar_family`) and the sim's
//! death sweep runs `check_dead_pillar_intercept` before the retain.
//!
//! # Coverage (this module) vs deferral
//!
//! | C# block | Range | Status here |
//! | --- | --- | --- |
//! | `checkDead` dissolve intercept + `PrepareForDeathAnimation` | `NPC.cs:78080-78086`, `:78032-78038` | PORTED (`check_dead_pillar_intercept`) |
//! | ai[2]==1 dissolve arm (velocity clamp, opacity, 180 death) | `:38970-39067` | PORTED FULLY (W4-e2: body ring 39007 + plume 39022 + the 6-burst 39039 with its SelectRandom(1,18) sound + the 241 tail 39052 all armed) |
//! | ai[3] shield-break flash cycle | `:39069-39100` | PORTED (sounds as effect flags) |
//! | per-type `dontTakeDamage = Shield != 0` gate | `:39101-39115` | PORTED (`pillar_shield`) |
//! | TargetClosest + localAI[0] regen + sine bob | `:39116-39131`, `:39190-39195` | PORTED (the regen's inert-counter quirk kept, see below) |
//! | ground hover Down(10)/Down(20)/Down(30) | `:39132-39153` | PORTED (`find_down_solid`, `Conditions.IsSolid`) |
//! | edge clamps incl. the `maxTilesX*16` Y-axis quirk + surface floor | `:39154-39189` | PORTED (quirk copied, not "fixed") |
//! | 517 Solar arm → SolarGoop 519 spawn | `:39347-39382` | PORTED FULLY (the one complete spawn arm) |
//! | 493 Stardust arm → `SpawnStardustMark_StardustTower` | `:39196-39224`, `:43821-43907` | DEFERRED body; W4-e2 ambient dust 39200/39209 armed + ai[1] cooldown kept |
//! | 507 Nebula arm (dust only — no attack in C# either) | `:39226-39247` | W4-e2 ambient dust 39230/39239 armed; nothing else exists to port |
//! | 422 Vortex arms (3240-px no-LOS + 1080-px window pick) | `:39249-39345` | DEFERRED bodies; W4-e2 ambient dust 39253/39262 armed + ai[1] cooldown kept |
//! | 629 shield-chip loop (spawn sites + −1 spend) | `:79279-79320`, `Projectile.cs:72944-72979` | SEAMS ONLY (`bolt_host_pillar`, `bolt_shield_spend`) — R72 wires |
//! | death → apocalypse chain | `:79321-79345` | already FAITHFUL (`world_sim/npcs.rs:3360-3406`) — untouched |
//!
//! Deferred-arm exact ranges for the wiring wave:
//! * **Stardust mark tree** `SpawnStardustMark_StardustTower`
//!   (`NPC.cs:43821-43907`): the count-gated candidate list [405 if
//!   `CountNPCS(405)+CountNPCS(406) < 2`, 402 if `CountNPCS(402) < 2`, 407
//!   if `CountNPCS(407) < 1`], `SelectRandom` pick, `ai[1] = 30*Next(5,16)`,
//!   `Next(3,6)` depth + `Next(0,4)` split budget, the BFS over `list2`
//!   (root `Top - UnitY*120`), per-node `Next(3)` split gate + `Next(max(1,
//!   depth))`, the `SelectRandom(-1,1)` root side, the angle/distance
//!   floats, the 540 marks with `ai0 = -depth*10, ai1 = 0.5+NextFloat*0.5`,
//!   the LAST mark's `localAI[0] = carried kind`, the empty-list
//!   `ai[1] = 30` fallback. The port's `wants_stardust_mark`
//!   (`npc.rs:1425`) is the WORM's mark (`:43909+`), a different fn.
//! * **Vortex 3240-px arm** (`:39275-39300`): `ai[1] = 60 + Next(120)`,
//!   count gate `CountNPCS(428)+CountNPCS(427)+CountNPCS(426) < 14`, the
//!   player-top tile up-scan (≤10 rows, stop at solid or Y≤10), 579 at
//!   `(X*16+8, Y*16+24)` when the gate holds else 578 at `+17` with ai1=1.
//! * **Vortex 1080-px arm** (`:39301-39345`): `ai[1] = 420 + Next(360)`,
//!   count gate `CountNPCS(427)+3*CountNPCS(426)+CountNPCS(428) < 20`, the
//!   100-try `Next(playerTile±20)` X-then-Y window pick with the 3-tile
//!   tower / 8-tile player exclusion boxes, `!nactive()` + no-lava +
//!   `SolidTiles(±2)` + LOS gates → 579 at `(X*16+8, Y*16+8)`.
//! * **Nebula mobs** arrive via the celestial ZONE spawn pool (no direct
//!   arm in C#) — `biome.rs` has no tower zone; a separate spawn-system
//!   slice (audit §2.3).
//!
//! # RNG by construction
//!
//! Every `Main.rand` draw the ported blocks make is threaded in C# order:
//! * dissolve arm: `Next(5)` (`:39003`, drawn even at ai[1]≥120 — left
//!   operand first), 3×`Next(4)` only while ai[1]<150 (`:39016-39030`),
//!   `Next(5)` (`:39031`), `Next(3)` (`:39050`);
//! * per-type ambient gates: `Next(5)` + 3×`Next(5)` (`:39198/:39207`,
//!   `:39228/:39237`, `:39251/:39260`, `:39351/:39360`);
//! * solar spawn: `RandomVector2` X-then-Y (`Utils.cs:2431-2434`, two
//!   `NextDouble`), `RotatedByRandom(π/4)` two `NextDouble`
//!   (`Utils.cs:1697-1700`), the speed `NextFloat` (`:39377`).
//!
//! W4-e2 (the dust-sites batch): the ambient per-type bodies (sites
//! 39200/39209/39230/39239/39253/39262/39353/39362) and the dissolve's
//! body ring + plume (39007/39022) now ride the live `DustPool`
//! (`new_dust`) on the SAME stream — the gate draws stay in C# order and
//! a passed gate rolls the pool internals + the explicit
//! `NextFloat`/`RotatedByRandom` jitter. The witnesses below replay them
//! through mirrored pool calls. W4-e2 batch 5 arms the LAST two dissolve
//! sites verbatim: the 6-dust burst (39039, `:39031-39049` — per outer
//! iteration the RotatedByRandom pair + the NextFloat radius + the
//! num1294 NextFloat, then 6 × (pool internals + the scale NextFloat),
//! then the `PlaySound(3, vector196, SelectRandom(1,18))` — the pick's
//! ONE `Next(2)` draw (`Utils.cs:2459-2462`) rides the stream HERE, the
//! case-3 pitch draw rides the audio drain's attenuation gate, the
//! `NpcSoundCue::NpcHit` seam) and the 241 tail (39052, `:39050-39056` —
//! the ambient body shape WITHOUT the 0.8 outer factor and at 4π/3).
//! STILL deferred (their draws stay skipped with the bodies): the
//! 493/422 attack bodies' rearm/window/BFS draws until the wiring wave
//! ports them.
//!
//! # Faithful quirks kept (do not "fix")
//!
//! * The Y edge clamp uses `num1302 = Main.maxTilesX * 16` — the X extent
//!   on BOTH axes (`:39154-39155`).
//! * The regen counter is INERT in vanilla: `localAI[0]++` when far
//!   (`:39117-39120`), then the SECOND if's else resets it to 0 whenever
//!   it is < 60 (`:39121-39130`) — the counter can never accumulate past
//!   1 through this arm alone, so the +200 heal only fires from an
//!   externally seeded ≥59. Ported exactly; the stand-in's
//!   `caster_counter` accumulation was a different (wrong) machine.
//! * The dissolve clamp is two SEQUENTIAL ifs (`:38973-38980`): vy=0.24
//!   → +0.02 → 0.26 → −0.02 → 0.24 in one tick.
//! * `Opacity` is a view over `alpha` (`NPC.cs:6820-6829`): the setter is
//!   `(int)Clamp((1-value)*255, 0, 255)` — the two-step f32 order kept.
//! * `Conditions.IsSolid` (`Conditions.cs:84-98`) is `InWorld(x,y,10) &&
//!   active() && tileSolid[type]` — platforms COUNT, actuated tiles are
//!   NOT excluded (unlike `WorldGen.SolidTile`).

#![allow(dead_code)] // the machine is WIRED (W4-c2); the 629 seams (`bolt_host_pillar`/`bolt_shield_spend`) stay R72's — the live B-6b loop owns the sites.

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{session_candidate, target_closest};
use crate::player::Player;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The four pillar kinds' vanilla type ids (`ID/NPCID.cs:11659-11711`).
/// The port's net ids (113/114/115/146) are the pre-existing collision —
/// consumers of THIS table must use the vanilla ids (audit §0).
pub fn pillar_vanilla_id(kind: NpcKind) -> Option<u16> {
    match kind {
        NpcKind::SolarTower => Some(517),
        NpcKind::VortexTower => Some(422),
        NpcKind::NebulaTower => Some(507),
        NpcKind::StardustTower => Some(493),
        _ => None,
    }
}

/// The per-type shield read (`NPC.cs:39101-39115`'s switch): the C#
/// statics `NPC.ShieldStrengthTower<Kind>` ride the port's World fields
/// (`world.rs:347-350`, the faithful state fns at
/// `world_sim/npcs.rs:1570-1608`).
pub fn pillar_shield(world: &World, kind: NpcKind) -> i32 {
    match kind {
        NpcKind::SolarTower => world.tower_shield_solar,
        NpcKind::VortexTower => world.tower_shield_vortex,
        NpcKind::NebulaTower => world.tower_shield_nebula,
        NpcKind::StardustTower => world.tower_shield_stardust,
        _ => 0,
    }
}

/// `NPC.checkDead`'s pillar intercept (`NPC.cs:78080-78086`): on death
/// with `ai[2] != 1` the tower does NOT die — it enters the dissolve.
/// Returns true when the death was intercepted (the caller keeps the NPC
/// in the roster). `PrepareForDeathAnimation` (`:78032-78038`):
/// `life = lifeMax`, `dontTakeDamage = true`, `netUpdate = true`;
/// `SetStrikePhaseChange` (`:90695-90701`) is a `netMode == 1`-only write
/// — a no-op in the single-player port (documented fold).
///
/// The sim's death sweep (`world_sim/npcs.rs:3352+`) calls this BEFORE
/// the retain, the same shape as the crystal's intercept already landed
/// at `npcs.rs:2094-2106` (`NPC.cs:78087-78095`). WIRED by W4-c2 at the
/// death sweep's head (`world_sim/npcs.rs`, before the `hp > 0` retain).
pub fn check_dead_pillar_intercept(npc: &mut Npc) -> bool {
    if pillar_vanilla_id(npc.kind).is_none() || npc.ai[2] == 1.0 {
        return false;
    }
    // :78082-78083
    npc.ai[2] = 1.0;
    npc.ai[1] = 0.0;
    // PrepareForDeathAnimation :78034-78036
    npc.hp = npc.max_hp;
    npc.dont_take_damage = true;
    npc.net_update = true;
    true
}

/// The 629 bolt's mob→host census (`NPC.cs:79279-79320`, inside
/// `checkDead`'s pre-death switch): a dying celestial mob of one of the
/// four id sets sends ONE 629 at its centre with `ai0 = FindFirstNPC(<host
/// type>)` while the matching shield stands. NOTE: SolarGoop **519 is
/// NOT in the solar set** (412-419 + 518 only, `:79279-79287`) — goop
/// deaths never chip the shield. R72 wires the spawn sites; this table is
/// the seam (keyed on VANILLA ids — the port's tower net ids are wrong).
pub fn bolt_host_pillar(dead_type: u16) -> Option<NpcKind> {
    match dead_type {
        // :79279-79287 — solar: 412-419 + 518 (SolarSpearman).
        412..=419 | 518 => Some(NpcKind::SolarTower),
        // :79293-79296 — vortex: 425/426/427/429.
        425 | 426 | 427 | 429 => Some(NpcKind::VortexTower),
        // :79302-79305 — nebula: 420/421/423/424.
        420 | 421 | 423 | 424 => Some(NpcKind::NebulaTower),
        // :79311-79315 — stardust: 402/405/407/409/411.
        402 | 405 | 407 | 409 | 411 => Some(NpcKind::StardustTower),
        _ => None,
    }
}

/// The 629 contact spend (`Projectile.cs:72944-72979`): the shield drops
/// by **−1 per bolt** regardless of damage (`:72953`'s
/// `Clamp(Shield - 1, 0, ShieldStrengthTowerMax)`), and the host's
/// `ai[3] = 1` flash trigger fires only when the shield was non-zero
/// (`:72949-72951`). Returns that flash condition; the caller (R72's
/// contact resolver) sets `host.ai[3] = 1.0` when true and
/// `host.net_update = true` unconditionally (`:72977`). The C# selects
/// the shield by `Main.npc[(int)ai[0]].type` — the HOST NPC's type, not
/// a tower ordinal (the live `projectile.rs:13502-13521` spend's defect
/// 2, audit §3.1); this fn takes the resolved host kind.
///
/// The `downed_tower_*` flags are NOT written here — they belong to the
/// tower's DEATH (`NPC.cs:79321-79345`, already faithful at
/// `world_sim/npcs.rs:3360-3391`). `shield_max` is
/// `world_sim/npcs.rs::tower_shield_max` (`NPC.cs:6795-6806`).
pub fn bolt_shield_spend(world: &mut World, host: NpcKind, shield_max: i32) -> bool {
    let strength = match host {
        NpcKind::SolarTower => &mut world.tower_shield_solar,
        NpcKind::VortexTower => &mut world.tower_shield_vortex,
        NpcKind::NebulaTower => &mut world.tower_shield_nebula,
        NpcKind::StardustTower => &mut world.tower_shield_stardust,
        _ => return false,
    };
    let flash = *strength != 0;
    *strength = (*strength - 1).clamp(0, shield_max);
    flash
}

/// The machine's environment inputs (the C# statics the port folds).
pub struct PillarCtx {
    /// `Main.isThereAWorldSurface` (`:39178`) — true in the port's worlds.
    pub has_world_surface: bool,
    /// `Main.remixWorld` (`:39186`) — unmodelled, the wiring passes false.
    pub remix_world: bool,
    /// `Main.getGoodWorld` (`:39186`) — unmodelled, the wiring passes false.
    pub get_good_world: bool,
    /// `Main.EverLastingTicker` for `TargetClosest`'s frame stamp.
    pub ticker: u64,
}

impl PillarCtx {
    /// The classic single-player fold the wiring wave passes.
    pub fn classic(ticker: u64) -> Self {
        Self {
            has_world_surface: true,
            remix_world: false,
            get_good_world: false,
            ticker,
        }
    }
}

/// The SolarGoop spawn request (`NPC.cs:39376-39380`): `NewNPC(..., 519,
/// base.whoAmI)` — the trailing `whoAmI` is the slot-search START hint
/// (`NPC.cs:80750`'s `Start` parameter), NOT ai0; the goop spawns with
/// ai[0..3] = 0, target 255, and `Bottom = ((int)v198.X, (int)v198.Y)`
/// (`NPC.cs:80770`). The resolver sets the velocity + netUpdate.
#[derive(Clone, Copy, Debug)]
pub struct SolarGoopSpawn {
    /// The bottom anchor, already `(int)`-cast as `NewNPC` receives it.
    pub bottom_x: i32,
    pub bottom_y: i32,
    /// `-UnitY.RotatedByRandom(π/4) * (7 + NextFloat*5)` (`:39377`).
    pub vx: f32,
    pub vy: f32,
}

/// The machine's effect requests (the seam the wiring wave drains).
#[derive(Default)]
pub struct PillarEffects {
    /// The 517 arm's SolarGoop 519 spawn (`:39374-39382`).
    pub solar_goop: Option<SolarGoopSpawn>,
    /// The dissolve reached ai[1] ≥ 180 (`:39061-39066`): `life = 0` was
    /// written; the caller runs `HitEffect(0, 1337)` + the death sweep
    /// (the intercept passes through — ai[2] is already 1).
    pub dissolve_finished: bool,
    /// `SoundID.NPCDeath58` = (4, 58) on the shield-state flip
    /// (`:39087-39090`, `SoundID.cs:1372`).
    pub shield_break_sound: bool,
    /// `SoundID.NPCDeath3` = (4, 3) at ai[3] == 1 (`:39091-39094`,
    /// `SoundID.cs:1317`).
    pub shield_hit_sound: bool,
    /// The dissolve's 60-tick drone, `PlaySound(4, Center, 22)`
    /// (`:39057-39060`).
    pub dissolve_drone_sound: bool,
    /// W4-e2 site 39039 — the burst's per-iteration
    /// `PlaySound(3, vector196, Utils.SelectRandom(Main.rand, 1, 18))`
    /// (`:39047`): (position.x, position.y, style). The SelectRandom
    /// pick's ONE `Next(2)` draw (`Utils.cs:2459-2462`,
    /// `choices[Next(len)]` → 1 or 18) rides the machine's stream at the
    /// C# call position (AFTER the iteration's 6 dusts); the case-3
    /// pitch draw rides the audio drain (`NpcSoundCue::NpcHit(style)`,
    /// the S1-E-21g queue convention — attenuation-gated before the
    /// draw). Up to 3 per tick, in outer-loop order.
    pub dissolve_burst_sounds: Vec<(f32, f32, i32)>,
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Searches.Chain(new Searches.Down(max), new Conditions.IsSolid())`
/// (`Searches.cs:51-71` + `Conditions.cs:84-98`): scan rows origin.Y + i
/// for i in 0..max while < maxTilesY; the condition is `InWorld(x,y,10)
/// && active() && tileSolid[type]` — platforms count, actuated tiles are
/// not excluded (NOT `WorldGen.SolidTile`). Returns the found row.
fn find_down_solid(world: &World, ox: i32, oy: i32, max_distance: i32) -> Option<i32> {
    for i in 0..max_distance {
        let y = oy + i;
        if y >= world.height {
            break;
        }
        if in_world(world, ox, y, 10) {
            if let Some(t) = world.tile(ox, y) {
                if t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind) {
                    return Some(y);
                }
            }
        }
    }
    None
}

/// `Vector2.Distance` (`Vector2.cs`): `(float)Math.Sqrt` of the f32
/// squared-delta sum — the sqrt runs in DOUBLE on the f32 sum, NOT
/// `hypot` (a different rounding path).
fn distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4` —
/// truncation toward zero, then the arithmetic shift.
fn tile_coord(v: f32) -> i32 {
    (v as i32) >> 4
}

/// `Vector2.UnitY.RotatedByRandom(max)` (`Utils.cs:1697-1700` +
/// `:1675-1684`): TWO `NextDouble` draws (+max then −max), then the
/// double cos/sin rotation of (0, 1) → (−sin, cos).
fn rotated_unit_y_random(rng: &mut UnifiedRandom, max_radians: f64) -> (f32, f32) {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    (-(angle.sin() as f32), angle.cos() as f32)
}

/// The ambient per-type dust, shared shape — W4-e2 sites 39200/39209
/// (493, plume 135), 39230/39239 (507, plume 242, Color.Black),
/// 39253/39262 (422, plume 229, Color.Black), 39353/39362 (517, plume 6):
/// one `Next(5)` gate (the 241 body dust) then three `Next(5)` gates (the
/// top plume). The bodies now ride the live pool; the gate draws stay in
/// C# order. `plume_black` is the Nebula/Vortex `color = Color.Black`
/// write (`:39244`/`:39267`); Stardust/Solar leave the spawn colour.
fn ambient_dust(
    npc: &Npc,
    world: &World,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    plume_type: i32,
    plume_black: bool,
) {
    let (w, h) = npc.size();
    let (cx, cy) = npc.center();
    // `:39198-39204` etc — the 241 body dust at Left, box (width,
    // INTEGER height/2), default speeds/alpha/colour/scale. Post-writes:
    // position = Center + UnitY.RotatedByRandom(2π/3) * (width/2,
    // height/2) * (0.8 + NextFloat·0.2) — INTEGER halves; velocity.X = 0,
    // velocity.Y = |velocity.Y|·0.25. NO noGravity/fadeIn/scale writes.
    if rng.next_max(5) == 0 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, Vec2::new(npc.x, npc.y + h / 2.0),
            w as i32, h as i32 / 2, 241, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let (rx, ry) = rotated_unit_y_random(rng, 2.094395160675049);
        let k = 0.8 + rng.next_float() * 0.2;
        let d = dust.slot_mut(idx);
        d.position = Vec2::new(
            cx + rx * (w as i32 / 2) as f32 * k,
            cy + ry * (h as i32 / 2) as f32 * k,
        );
        d.velocity.x = 0.0;
        d.velocity.y = d.velocity.y.abs() * 0.25;
    }
    // `:39205-39216` etc — the three top-plume gates. Per i: the spawn at
    // Top + (−width·(0.33 − 0.11·i), −20), box ((int)(width·(0.66 −
    // 0.22·i)), 20), plume type, defaults. Post-writes: velocity.X = 0,
    // velocity.Y = −|velocity.Y − i + vy − 4|·1, noGravity, fadeIn 1,
    // (Nebula/Vortex: color Black), scale = 1 + NextFloat + i·0.3.
    for i in 0..3 {
        if rng.next_max(5) == 0 {
            let fi = i as f32;
            let px = npc.x + w * 0.5 + (-w) * (0.33 - 0.11 * fi);
            let py = npc.y - 20.0;
            let bw = (w * (0.66 - 0.22 * fi)) as i32;
            let idx = dust.new_dust(
                rng, dust_ctx, world, Vec2::new(px, py), bw, 20,
                plume_type, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.velocity.x = 0.0;
            d.velocity.y = (0.0 - (d.velocity.y - fi + npc.vy - 4.0).abs()) * 1.0;
            d.no_gravity = true;
            d.fade_in = 1.0;
            if plume_black {
                d.color = Color::new(0, 0, 0, 255); // Color.Black (alpha 255)
            }
            d.scale = 1.0 + rng.next_float() + fi * 0.3;
        }
    }
}

/// The aiStyle-94 step, one tick, in the C#'s block order
/// (`NPC.cs:38968-39383`). The caller integrates position afterwards
/// (the C# arm ends before `UpdateCollision`; the towers are
/// `noGravity`/`noTileCollide` — the generic pass integrates straight
/// through). `Main.netMode != 1` folds to true (single-player port).
pub fn pillar_ai_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    ctx: &PillarCtx,
    fx: &mut PillarEffects,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    let (w, h) = npc.size();
    // ---- the ai[2] == 1 dissolve arm (:38970-39067) ----
    if npc.ai[2] == 1.0 {
        // :38972 — velocity = UnitY * velocity.Length(); Length() is
        // (float)Math.Sqrt of the f32 sum (double sqrt, Vector2.cs).
        let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
        npc.vx = 0.0;
        npc.vy = len;
        // :38973-38980 — two SEQUENTIAL ifs (the 0.24 → 0.26 → 0.24 quirk).
        if npc.vy < 0.25 {
            npc.vy += 0.02;
        }
        if npc.vy > 0.25 {
            npc.vy -= 0.02;
        }
        // :38981-38982
        npc.dont_take_damage = true;
        npc.ai[1] += 1.0;
        // :38983-38986 — Opacity = 1 - (ai[1]-120)/60; the setter
        // (NPC.cs:6826-6829) is (int)Clamp((1-value)*255, 0, 255).
        if npc.ai[1] > 120.0 {
            let opacity = 1.0 - (npc.ai[1] - 120.0) / 60.0;
            npc.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as u8;
        }
        // :38987-39002 — the per-type dissolve dust id (517→127, 422→229,
        // 507→242, 493→135, default 6). W4-e2 arms the bodies.
        let num1290 = match npc.kind {
            NpcKind::SolarTower => 127,
            NpcKind::VortexTower => 229,
            NpcKind::NebulaTower => 242,
            NpcKind::StardustTower => 135,
            _ => 6,
        };
        let (dw, dh) = npc.size();
        let (dcx, dcy) = npc.center();
        // :39003 — the gate draw happens even at ai[1] >= 120 (left
        // operand first). W4-e2 site 39007: the 3-dust body ring.
        if rng.next_max(5) == 0 && npc.ai[1] < 120.0 {
            for num1291 in 0..3 {
                let fi = num1291 as f32;
                // NewDust(Left, width, INTEGER height/2, num1290), defaults.
                let idx = dust.new_dust(
                    rng, dust_ctx, world, Vec2::new(npc.x, npc.y + dh / 2.0),
                    dw as i32, dh as i32 / 2, num1290,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                // position = Center + UnitY.RotatedByRandom(4π/3) *
                // (width·1.5, height·1.1) · 0.8 · (0.8 + NextFloat·0.2).
                let (rx, ry) = rotated_unit_y_random(rng, 4.188790321350098);
                let k = 0.8 * (0.8 + rng.next_float() * 0.2);
                let d = dust.slot_mut(idx);
                d.position = Vec2::new(
                    dcx + rx * (dw * 1.5) * k,
                    dcy + ry * (dh * 1.1) * k,
                );
                d.velocity.x = 0.0;
                d.velocity.y =
                    (0.0 - (d.velocity.y - fi + npc.vy - 4.0).abs()) * 3.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.scale = 1.0 + rng.next_float() + fi * 0.3;
            }
        }
        // :39016-39030 — three Next(4) gates, only while ai[1] < 150.
        // W4-e2 site 39022: the top plume (same shape as the ambient one,
        // but the velocity multiplier is (1 + ai[1]/180·0.5) and the
        // colour stays the spawn default).
        if npc.ai[1] < 150.0 {
            for num1292 in 0..3 {
                if rng.next_max(4) == 0 {
                    let fi = num1292 as f32;
                    let px = npc.x + dw * 0.5 + (-dw) * (0.33 - 0.11 * fi);
                    let py = npc.y - 20.0;
                    let bw = (dw * (0.66 - 0.22 * fi)) as i32;
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, Vec2::new(px, py), bw, 20,
                        num1290, 0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = dust.slot_mut(idx);
                    d.velocity.x = 0.0;
                    d.velocity.y = (0.0
                        - (d.velocity.y - fi + npc.vy - 4.0).abs())
                        * (1.0 + npc.ai[1] / 180.0 * 0.5);
                    d.no_gravity = true;
                    d.fade_in = 1.0;
                    d.scale = 1.0 + rng.next_float() + fi * 0.3;
                }
            }
        }
        // :39031-39049 — drawn even at ai[1] >= 150. W4-e2 site 39039:
        // the 3×(6-dust burst + SelectRandom(1,18) sound). Per outer
        // iteration: vector196 = Center + UnitY.RotatedByRandom(4π/3) *
        // (width, height) * 0.7 * NextFloat (the rotation pair FIRST,
        // then the radius float — the C# expression order), num1294 =
        // 1 + NextFloat*2 + ai[1]/180*4; then 6 × NewDust(vector196, 4,
        // 4, num1290) with position = vector196, velocity.X *= num1294,
        // velocity.Y = -|velocity.Y| * num1294, noGravity, fadeIn 1,
        // scale = 1.5 + NextFloat + i*0.13; then the sound (:39047).
        if rng.next_max(5) == 0 && npc.ai[1] < 150.0 {
            for _ in 0..3 {
                let (rx, ry) = rotated_unit_y_random(rng, 4.188790321350098);
                // `rot * (width, height) * 0.7f * NextFloat()` — the
                // left-to-right f32 order: ((rot·extent)·0.7)·nf.
                let nf = rng.next_float();
                let vx196 = dcx + (rx * dw) * 0.7 * nf;
                let vy196 = dcy + (ry * dh) * 0.7 * nf;
                let num1294 = 1.0 + rng.next_float() * 2.0 + npc.ai[1] / 180.0 * 4.0;
                for num1295 in 0..6 {
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, Vec2::new(vx196, vy196), 4, 4,
                        num1290, 0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = dust.slot_mut(idx);
                    d.position = Vec2::new(vx196, vy196);
                    d.velocity.x *= num1294;
                    d.velocity.y = (0.0 - d.velocity.y.abs()) * num1294;
                    d.no_gravity = true;
                    d.fade_in = 1.0;
                    d.scale = 1.5 + rng.next_float() + num1295 as f32 * 0.13;
                }
                // :39047 — SelectRandom(1, 18): ONE Next(2) draw here
                // (the pick), the play rides the queue (the case-3 pitch
                // draw is the drain's, attenuation-gated).
                let style = if rng.next_max(2) == 0 { 1 } else { 18 };
                fx.dissolve_burst_sounds.push((vx196, vy196, style));
            }
        }
        // :39050-39056 — drawn even at ai[1] >= 150. W4-e2 site 39052:
        // the 241 tail — NewDust(Left, width, INTEGER height/2, 241),
        // position = Center + UnitY.RotatedByRandom(4π/3) * (INTEGER
        // width/2, INTEGER height/2) * (0.8 + NextFloat*0.2) — NO 0.8
        // outer factor (unlike the 39007 ring), velocity.X = 0,
        // velocity.Y = |velocity.Y| * 0.25. NO noGravity/fadeIn/scale.
        if rng.next_max(3) != 0 && npc.ai[1] < 150.0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, Vec2::new(npc.x, npc.y + dh / 2.0),
                dw as i32, dh as i32 / 2, 241, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let (rx, ry) = rotated_unit_y_random(rng, 4.188790321350098);
            let k = 0.8 + rng.next_float() * 0.2;
            let d = dust.slot_mut(idx);
            d.position = Vec2::new(
                dcx + rx * (dw as i32 / 2) as f32 * k,
                dcy + ry * (dh as i32 / 2) as f32 * k,
            );
            d.velocity.x = 0.0;
            d.velocity.y = d.velocity.y.abs() * 0.25;
        }
        // :39057-39060 — the 60-tick drone.
        if npc.ai[1] % 60.0 == 1.0 {
            fx.dissolve_drone_sound = true;
        }
        // :39061-39066 — death at 180: HitEffect(0, 1337) + checkDead ride
        // the sim's death sweep (the intercept passes through, ai[2]==1).
        if npc.ai[1] >= 180.0 {
            npc.hp = 0;
            fx.dissolve_finished = true;
        }
        return;
    }
    // ---- the ai[3] shield-break flash cycle (:39069-39100) ----
    if npc.ai[3] > 0.0 {
        let flag100 = pillar_shield(world, npc.kind) != 0;
        if flag100 != npc.dont_take_damage {
            // :39089 — NPCDeath58 on the state flip.
            fx.shield_break_sound = true;
        } else if npc.ai[3] == 1.0 {
            // :39093 — NPCDeath3 on the first flash tick.
            fx.shield_hit_sound = true;
        }
        npc.ai[3] += 1.0;
        if npc.ai[3] > 120.0 {
            npc.ai[3] = 0.0;
        }
    }
    // ---- the per-type shield gate (:39101-39115) ----
    npc.dont_take_damage = pillar_shield(world, npc.kind) != 0;
    // ---- TargetClosest + the regen (:39116-39130) ----
    let cand = session_candidate(player);
    target_closest(npc, world, &[cand], false, 94, ctx.ticker);
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let (pcx, pcy) = player.center();
    // :39117-39120 — the far-player increment.
    if distance(pcx, pcy, cx, cy) > 2000.0 {
        npc.local_ai[0] += 1.0;
    }
    // :39121-39130 — the SECOND if's else resets the counter whenever it
    // is < 60: the vanilla counter is inert (never accumulates past 1
    // through this arm alone). Kept exactly — see the module doc.
    if npc.local_ai[0] >= 60.0 {
        npc.local_ai[0] = 0.0;
        npc.net_update = true;
        npc.hp = (npc.hp + 200).clamp(0, npc.max_hp);
    } else {
        npc.local_ai[0] = 0.0;
    }
    // ---- the sine bob (:39131) ----
    // Math.Sin runs in double on the f32 argument, cast back to float.
    npc.vx = 0.0;
    npc.vy = ((std::f32::consts::PI * 2.0 * npc.ai[0] / 300.0) as f64).sin() as f32 * 0.5;
    // ---- the ground hover (:39132-39153) ----
    let origin_x = tile_coord(npc.x + w * 0.5);
    let origin_y = tile_coord(npc.y + h);
    // :39137 — InWorld(origin, 60) && tile != null.
    if in_world(world, origin_x, origin_y, 60) && world.tile(origin_x, origin_y).is_some() {
        if let Some(found_y) = find_down_solid(world, origin_x, origin_y, 10) {
            // :39141-39142 — 1 - |ΔY|/10, lift 1.5·that.
            let num1299 = 1.0 - (origin_y - found_y).abs() as f32 / 10.0;
            npc.y -= 1.5 * num1299;
        } else if find_down_solid(world, origin_x, origin_y, 20).is_none() {
            // :39146-39151 — no solid within 20: sink 1.5·lerp, the
            // Down(30) hit ramping 20→30 (clamped GetLerpValue).
            let mut num1300 = 1.0;
            if let Some(found_y) = find_down_solid(world, origin_x, origin_y, 30) {
                num1300 = crate::projectile::get_lerp_value(
                    20.0,
                    30.0,
                    (origin_y - found_y).abs() as f32,
                );
            }
            npc.y += 1.5 * num1300;
        }
    }
    // ---- the edge clamps (:39154-39177) ----
    let num1301 = world.width * 16;
    // :39155 — the C# QUIRK: the Y extent is maxTilesX*16 too. Copied.
    let num1302 = world.width * 16;
    let num1303 = 60 * 16;
    let num1304 = 80;
    let (mut ccx, mut ccy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    if ccx < num1303 as f32 {
        ccx = (num1303 + num1304) as f32;
        npc.net_update = true;
    }
    if ccx > (num1301 - num1303) as f32 {
        ccx = (num1301 - num1303 - num1304) as f32;
        npc.net_update = true;
    }
    if ccy < num1303 as f32 {
        ccy = (num1303 + num1304) as f32;
        npc.net_update = true;
    }
    if ccy > (num1302 - num1303) as f32 {
        ccy = (num1302 - num1303 - num1304) as f32;
        npc.net_update = true;
    }
    // the Center setter (Entity.cs:56-59)
    npc.x = ccx - w * 0.5;
    npc.y = ccy - h * 0.5;
    // ---- the surface arms (:39178-39189) ----
    if !ctx.has_world_surface {
        if npc.y < 640.0 {
            // :39182 — (float)Main.rockLayer * 16f.
            npc.y = world.rock_layer as f32 * 16.0;
            npc.net_update = true;
        }
    } else if !ctx.remix_world
        && !ctx.get_good_world
        // :39186 — the DOUBLE comparison on Bottom.Y.
        && (npc.y + h) as f64 > world.world_surface * 16.0 - 100.0
    {
        npc.y = world.world_surface as f32 * 16.0 - h - 100.0;
    }
    // ---- the bob cycle (:39190-39195) ----
    npc.ai[0] += 1.0;
    if npc.ai[0] >= 300.0 {
        npc.ai[0] = 0.0;
        npc.net_update = true;
    }
    // ---- the per-type arms ----
    match npc.kind {
        NpcKind::StardustTower => {
            // :39196-39224 — the ambient dust (W4-e2 sites 39200/39209,
            // plume 135, spawn colour) + the ai[1] cooldown.
            ambient_dust(npc, world, rng, dust, dust_ctx, 135, false);
            if npc.ai[1] > 0.0 {
                npc.ai[1] -= 1.0;
            }
            // :39221-39224 — the mark-tree call
            // (SpawnStardustMark_StardustTower, :43821-43907) is the
            // DEFERRED body: gate ai[1]<=0 && player active/!dead &&
            // Distance < 1080 && player.Y - tower.Y < 400. Its BFS +
            // SelectRandom + rearm draws arrive with the wiring wave
            // (the module doc's exact-range census).
        }
        NpcKind::NebulaTower => {
            // :39226-39247 — dust only (W4-e2 sites 39230/39239, plume
            // 242, Color.Black); the C# has NO attack arm here (nebula
            // mobs come from the zone spawn pool).
            ambient_dust(npc, world, rng, dust, dust_ctx, 242, true);
        }
        NpcKind::VortexTower => {
            // :39249-39274 — the ambient dust (W4-e2 sites 39253/39262,
            // plume 229, Color.Black) + the ai[1] cooldown.
            ambient_dust(npc, world, rng, dust, dust_ctx, 229, true);
            if npc.ai[1] > 0.0 {
                npc.ai[1] -= 1.0;
            }
            // :39275-39345 — the two attack arms (3240-px no-LOS ritual +
            // 1080-px window pick) are DEFERRED bodies; their
            // 60+Next(120) / 420+Next(360) rearms, count gates, tile scan
            // and 100-try window draws arrive with the wiring wave (the
            // module doc's exact-range census). The live stand-in's
            // every-tick 579 spam is NOT reproduced here.
        }
        NpcKind::SolarTower => {
            // :39347-39382 — the FULL solar arm. W4-e2 sites 39353/39362:
            // the ambient dust, plume 6, spawn colour.
            ambient_dust(npc, world, rng, dust, dust_ctx, 6, false);
            if npc.ai[1] > 0.0 {
                npc.ai[1] -= 1.0;
            }
            // :39374 — netMode != 1 folds true; the target gates. The
            // C# reads base.Distance / base.position HERE, after the
            // hover + clamps above — recompute the centre.
            let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            if npc.ai[1] <= 0.0
                && !player.dead
                && distance(pcx, pcy, cx, cy) < 1080.0
                && player.y - npc.y < 700.0
            {
                // :39376 — vector198 = Top + (-w*0.33, -20) +
                // (w*0.66, 20) * RandomVector2(0, 1). The X draw precedes
                // the Y draw (Utils.cs:2431-2434).
                let rx = rng.next_float();
                let ry = rng.next_float();
                let spawn_x = npc.x + w * 0.5 + (-w * 0.33) + w * 0.66 * rx;
                let spawn_y = npc.y - 20.0 + 20.0 * ry;
                // :39377 — -UnitY.RotatedByRandom(0.7853981852531433) *
                // (7 + NextFloat*5). RotatedByRandom draws NextDouble()*max
                // - NextDouble()*max (Utils.cs:1697-1700); RotatedBy on
                // (0,1) gives (-sin, cos) (Utils.cs:1675-1684), negated →
                // (sin, -cos).
                let max_radians = 0.7853981852531433f64;
                let angle =
                    rng.next_double() * max_radians - rng.next_double() * max_radians;
                let cos = angle.cos() as f32;
                let sin = angle.sin() as f32;
                let speed = 7.0 + rng.next_float() * 5.0;
                // :39378-39381 — NewNPC bottom-anchors at the (int) cast
                // (NPC.cs:80770); the velocity + netUpdate ride the
                // resolver; ai[1] = 60.
                fx.solar_goop = Some(SolarGoopSpawn {
                    bottom_x: spawn_x as i32,
                    bottom_y: spawn_y as i32,
                    vx: sin * speed,
                    vy: -cos * speed,
                });
                npc.ai[1] = 60.0;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// An air world, 200×300 tiles (the Y-clamp quirk is visible: the C#
    /// bounds Y by width*16 = 3200, not height*16 = 4800).
    fn air_world() -> World {
        World::new("pillar".into(), 200, 300)
    }

    fn tower(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    fn far_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        // Far outside every gate; the surface/edge clamps own the tower.
        p.x = 1600.0;
        p.y = 1400.0;
        p
    }

    /// W4-e2: the permissive dust ctx for the witnesses — the whole world
    /// on-screen, the full 6000 budget (the `npc::permissive_dust_ctx`
    /// shape, local so the module keeps no npc.rs test dependency).
    fn permissive_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// W4-e2 mirror: replay ONE ambient body-dust hit's draws
    /// (`NPC.cs:39198-39204` etc) — the NewDust(Left, width, height/2,
    /// 241) internals, then RotatedByRandom(2π/3)'s two NextDouble, then
    /// the (0.8 + NextFloat·0.2) scale float. The permissive empty pool
    /// never culls, so the spawn always lands.
    fn mirror_ambient_body(
        mpool: &mut DustPool, mirror: &mut UnifiedRandom, dc: &SpawnCtx,
        w: &World, x: f32, y: f32, wi: i32, hi: i32,
    ) {
        mpool.new_dust(
            mirror, dc, w, Vec2::new(x, y + hi as f32 / 2.0), wi, hi / 2,
            241, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        mirror.next_double(); // RotatedByRandom +
        mirror.next_double(); // RotatedByRandom −
        mirror.next_float(); // the 0.8 + ·0.2 scale
    }

    /// W4-e2 mirror: replay ONE ambient top-plume hit's draws
    /// (`NPC.cs:39205-39216` etc) — the NewDust(Top + offset, box, 20,
    /// plume) internals, then the (1 + NextFloat + i·0.3) scale float.
    fn mirror_ambient_plume(
        mpool: &mut DustPool, mirror: &mut UnifiedRandom, dc: &SpawnCtx,
        w: &World, x: f32, y: f32, wi: f32, i: usize, plume: i32,
    ) {
        let fi = i as f32;
        let px = x + wi * 0.5 + (-wi) * (0.33 - 0.11 * fi);
        let bw = (wi * (0.66 - 0.22 * fi)) as i32;
        mpool.new_dust(
            mirror, dc, w, Vec2::new(px, y - 20.0), bw, 20,
            plume, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        mirror.next_float(); // the 1 + NextFloat + i·0.3 scale
    }

    #[test]
    fn b6a_the_intercept_keeps_the_tower_alive_and_arms_the_dissolve() {
        // NPC.cs:78080-78086 + PrepareForDeathAnimation :78032-78038.
        let mut n = tower(NpcKind::SolarTower, 1535.0, 1330.0);
        n.hp = 0;
        n.ai[1] = 42.0;
        assert!(check_dead_pillar_intercept(&mut n));
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(n.hp, n.max_hp, "life = lifeMax");
        assert!(n.dont_take_damage);
        assert!(n.net_update);
        // The second death (ai[2] already 1) passes through to the real
        // death chain.
        n.hp = 0;
        assert!(!check_dead_pillar_intercept(&mut n));
        // Non-pillars never intercept.
        let mut other = tower(NpcKind::Zombie, 0.0, 0.0);
        other.hp = 0;
        assert!(!check_dead_pillar_intercept(&mut other));
    }

    #[test]
    fn b6a_the_dissolve_clamps_velocity_toward_quarter_with_two_sequential_ifs() {
        // :38972-38980 — vy = 0.24 → +0.02 → 0.26 → −0.02 → 0.24.
        let w = air_world();
        let p = far_player(&w);
        let mut n = tower(NpcKind::VortexTower, 1535.0, 1330.0);
        n.ai[2] = 1.0;
        n.vx = 0.0;
        n.vy = 0.24;
        let mut rng = UnifiedRandom::new(7);
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n.vy - 0.24).abs() < 1e-5, "the double-if quirk: {}", n.vy);
        assert_eq!(n.vx, 0.0);
        assert_eq!(n.ai[1], 1.0);
        assert!(n.dont_take_damage);
        // A fast fall decays 0.02/tick toward 0.25 (:38977-38980).
        n.vy = 3.0;
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n.vy - 2.98).abs() < 1e-5);
    }

    #[test]
    fn b6a_the_dissolve_fades_after_120_and_dies_at_180() {
        // :38983-38986 + :39061-39066; the Opacity setter :6826-6829.
        let w = air_world();
        let p = far_player(&w);
        let mut n = tower(NpcKind::NebulaTower, 1535.0, 1330.0);
        n.ai[2] = 1.0;
        n.ai[1] = 149.0; // the step makes it 150
        let mut rng = UnifiedRandom::new(7);
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        // opacity = 1 - 30/60 = 0.5 → alpha = (int)(0.5*255) = 127.
        assert_eq!(n.alpha, 127);
        assert!(!fx.dissolve_finished);
        n.ai[1] = 179.0;
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[1], 180.0);
        assert_eq!(n.hp, 0);
        assert!(fx.dissolve_finished);
        // alpha clamps at 255 (opacity 0 at ai[1] = 180).
        assert_eq!(n.alpha, 255);
    }

    #[test]
    fn b6a_the_dissolve_gate_draws_ride_in_csharp_order() {
        // W4-e2: :39003 Next(5) (+ the 3-dust body ring on a pass while
        // ai[1]<120); :39020 3×Next(4) (+ plume bodies, ai[1]<150);
        // :39031 Next(5) (+ 3×(6-dust burst + SelectRandom Next(2)));
        // :39050 Next(3) (+ the 241 tail). The mirror replays gate +
        // ALL FOUR armed bodies through the same pool calls and
        // compares full stream state.
        let w = air_world();
        let dc = permissive_ctx(&w);
        let p = far_player(&w);
        let mut n = tower(NpcKind::StardustTower, 1535.0, 1330.0);
        n.ai[2] = 1.0;
        let (wi, hi) = (130i32, 270i32);
        let mut rng = UnifiedRandom::new(99);
        let mut mirror = UnifiedRandom::new(99);
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut pool, &dc);
        // The dissolve clamps vy to 0.25 before the dust (:38972-38980);
        // the position is unchanged (the arm returns before integration).
        // The post-writes (position/velocity/scale) draw NOTHING beyond the
        // RotatedByRandom pair + the two NextFloat scales mirrored below.
        let (dx, dy) = (1535.0f32, 1330.0f32);
        // :39003 — the body gate; on a pass + ai[1]<120 the 3-dust ring.
        let ai1_after = 1.0f32; // the step incremented ai[1] 0 → 1
        if mirror.next_max(5) == 0 && ai1_after < 120.0 {
            for _ in 0..3 {
                mpool.new_dust(
                    &mut mirror, &dc, &w, Vec2::new(dx, dy + hi as f32 / 2.0),
                    wi, hi / 2, 135, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                mirror.next_double(); // RotatedByRandom +
                mirror.next_double(); // RotatedByRandom −
                mirror.next_float(); // the 0.8·(0.8 + ·0.2) position scale
                mirror.next_float(); // the 1 + NextFloat + i·0.3 scale
            }
        }
        // :39016-39030 — the three Next(4) plume gates (ai[1]<150).
        if ai1_after < 150.0 {
            for num1292 in 0..3 {
                if mirror.next_max(4) == 0 {
                    let fi = num1292 as f32;
                    let px = dx + wi as f32 * 0.5 + (-wi as f32) * (0.33 - 0.11 * fi);
                    let bw = (wi as f32 * (0.66 - 0.22 * fi)) as i32;
                    mpool.new_dust(
                        &mut mirror, &dc, &w, Vec2::new(px, dy - 20.0), bw, 20,
                        135, 0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    mirror.next_float(); // the scale
                }
            }
        }
        // :39031 — the burst gate; on a pass + ai[1]<150 the 3×(6-dust
        // burst + SelectRandom pick). Per outer iteration: the
        // RotatedByRandom pair, the 0.7·NextFloat radius, the num1294
        // NextFloat, then 6 × (NewDust(vector196, 4, 4) internals + the
        // scale NextFloat), then the pick's Next(2).
        if mirror.next_max(5) == 0 && ai1_after < 150.0 {
            for _ in 0..3 {
                mirror.next_double(); // RotatedByRandom +
                mirror.next_double(); // RotatedByRandom −
                mirror.next_float(); // the 0.7·NextFloat radius
                mirror.next_float(); // num1294 = 1 + NextFloat·2 + …
                for _ in 0..6 {
                    mpool.new_dust(
                        &mut mirror, &dc, &w, Vec2::ZERO, 4, 4,
                        135, 0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    mirror.next_float(); // 1.5 + NextFloat + i·0.13
                }
                mirror.next_max(2); // SelectRandom(1, 18)
            }
        }
        // :39050 — the tail gate; on a pass + ai[1]<150 the 241 tail:
        // NewDust(Left, width, INTEGER height/2, 241) internals, the
        // RotatedByRandom(4π/3) pair, the (0.8 + NextFloat·0.2) float.
        if mirror.next_max(3) != 0 && ai1_after < 150.0 {
            mpool.new_dust(
                &mut mirror, &dc, &w, Vec2::new(dx, dy + hi as f32 / 2.0),
                wi, hi / 2, 241, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            mirror.next_double(); // RotatedByRandom +
            mirror.next_double(); // RotatedByRandom −
            mirror.next_float(); // the 0.8 + ·0.2 position scale
        }
        assert_eq!(rng, mirror, "the dissolve tick's gates + armed bodies");
        // At ai[1] >= 150 the body ring AND the three Next(4) gates stop;
        // the two outer gates still draw (left operand first): 3 draws.
        n.ai[1] = 160.0;
        let mut rng2 = UnifiedRandom::new(5);
        let mut mirror2 = UnifiedRandom::new(5);
        pillar_ai_step(&mut n, &w, &p, &mut rng2, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &dc);
        mirror2.next_max(5);
        mirror2.next_max(5);
        mirror2.next_max(3);
        assert_eq!(rng2, mirror2);
    }

    /// W4-e2 site 39039 — the burst's shape: 6 dusts per outer iteration
    /// at the SAME point (position = vector196 exactly), velocity.X is
    /// the spawn jitter × num1294, velocity.Y = -|jitter| × num1294,
    /// noGravity + fadeIn 1, scale = 1.5 + NextFloat + i·0.13 (the
    /// per-i ladder), and one SelectRandom(1,18) sound per iteration at
    /// vector196. The mirror replays the whole tick through the same
    /// pool calls and compares every spawned slot field-for-field.
    #[test]
    fn w4_e2_the_dissolve_burst_six_per_iteration_with_the_pick_sound() {
        let w = air_world();
        let dc = permissive_ctx(&w);
        let p = far_player(&w);
        // Seed scan: find a stream whose FIRST Next(5) passes (the ring
        // gate) AND whose :39031 Next(5) passes — the burst is rare
        // (1/25 per tick); the witness pins the passed shape.
        let mut seed = 0u64;
        let mut found = None;
        while seed < 4096 {
            let mut probe = UnifiedRandom::new(seed as i32);
            let g1 = probe.next_max(5);
            // the ring body (3 × (internals + 2 doubles + 2 floats))
            if g1 == 0 {
                for _ in 0..3 {
                    let mut mp = DustPool::new();
                    mp.new_dust(&mut probe, &dc, &w, Vec2::ZERO, 130, 135, 135, 0.0, 0.0, 0, Color::default(), 1.0);
                    probe.next_double(); probe.next_double();
                    probe.next_float(); probe.next_float();
                }
            }
            for _ in 0..3 {
                if probe.next_max(4) == 0 {
                    let mut mp = DustPool::new();
                    mp.new_dust(&mut probe, &dc, &w, Vec2::ZERO, 40, 20, 135, 0.0, 0.0, 0, Color::default(), 1.0);
                    probe.next_float();
                }
            }
            if probe.next_max(5) == 0 {
                found = Some(seed);
                break;
            }
            seed += 1;
        }
        let seed = found.expect("a seed with both gates passed") as i32;
        let mut n = tower(NpcKind::StardustTower, 1535.0, 1330.0);
        n.ai[2] = 1.0;
        let mut rng = UnifiedRandom::new(seed);
        let mut pool = DustPool::new();
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut pool, &dc);
        // The burst fired: 3 sounds, styles in {1, 18}, all at the same
        // per-iteration point as that iteration's 6 dusts.
        assert_eq!(fx.dissolve_burst_sounds.len(), 3, "3 outer iterations");
        for &(sx, sy, style) in &fx.dissolve_burst_sounds {
            assert!(style == 1 || style == 18, "SelectRandom(1,18): {style}");
            let at_point = (0..6000)
                .filter(|&i| pool.slot(i).active && pool.slot(i).position == Vec2::new(sx, sy))
                .count();
            assert_eq!(at_point, 6, "6 burst dusts at vector196");
        }
        // The scale ladder: within one iteration the six scales ascend
        // by ~0.13 plus the NextFloat jitter (each in [1.5+i·0.13,
        // 2.5+i·0.13)).
        let (sx, sy, _) = fx.dissolve_burst_sounds[0];
        let scales: Vec<f32> = (0..6000)
            .filter(|&i| pool.slot(i).active && pool.slot(i).position == Vec2::new(sx, sy))
            .map(|i| pool.slot(i).scale)
            .collect();
        for (i, s) in scales.iter().enumerate() {
            let lo = 1.5 + i as f32 * 0.13;
            assert!(*s >= lo && *s < lo + 1.0, "scale ladder {i}: {s}");
        }
        // Every burst dust: noGravity, fadeIn 1, velocity.Y <= 0.
        for i in 0..6000 {
            let d = pool.slot(i);
            if d.active && d.position == Vec2::new(sx, sy) {
                assert!(d.no_gravity);
                assert_eq!(d.fade_in, 1.0);
                assert!(d.velocity.y <= 0.0, "the -|vY|·num1294 write");
            }
        }
    }

    /// W4-e2 site 39052 — the 241 tail: the ambient body shape at 4π/3
    /// WITHOUT the 0.8 outer factor; velocity.X = 0, velocity.Y =
    /// |jitter|·0.25 (POSITIVE — the fall), NO noGravity/fadeIn/scale
    /// writes. The mirror replays the tick and compares the tail slot.
    #[test]
    fn w4_e2_the_dissolve_tail_241_falls_without_the_outer_factor() {
        let w = air_world();
        let dc = permissive_ctx(&w);
        let p = far_player(&w);
        // Seed scan: the :39050 Next(3) != 0 gate passes on ~2/3 of
        // streams; take the first seed where the whole tick's tail
        // fires (the earlier gates' bodies replay through the mirror).
        let mut n = tower(NpcKind::VortexTower, 1535.0, 1330.0);
        n.ai[2] = 1.0;
        let mut seed = 1i32;
        let mut hit = false;
        while seed < 4096 && !hit {
            let mut probe_n = tower(NpcKind::VortexTower, 1535.0, 1330.0);
            probe_n.ai[2] = 1.0;
            let mut probe_rng = UnifiedRandom::new(seed);
            let mut probe_pool = DustPool::new();
            let mut probe_fx = PillarEffects::default();
            pillar_ai_step(&mut probe_n, &w, &p, &mut probe_rng, &PillarCtx::classic(0), &mut probe_fx, &mut probe_pool, &dc);
            hit = (0..6000).any(|i| probe_pool.slot(i).active && probe_pool.slot(i).dust_type == 241);
            if !hit { seed += 1; }
        }
        assert!(hit, "a seed with the tail gate passed");
        n.ai[1] = 0.0;
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = UnifiedRandom::new(seed);
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut pool, &dc);
        // Replay the whole tick (the same census as the gate witness).
        let (dx, dy) = (1535.0f32, 1330.0f32);
        let (wi, hi) = (130i32, 270i32);
        let ai1 = 1.0f32;
        if mirror.next_max(5) == 0 && ai1 < 120.0 {
            for _ in 0..3 {
                mpool.new_dust(&mut mirror, &dc, &w, Vec2::new(dx, dy + hi as f32 / 2.0), wi, hi / 2, 229, 0.0, 0.0, 0, Color::default(), 1.0);
                mirror.next_double(); mirror.next_double();
                mirror.next_float(); mirror.next_float();
            }
        }
        if ai1 < 150.0 {
            for num1292 in 0..3 {
                if mirror.next_max(4) == 0 {
                    let fi = num1292 as f32;
                    let px = dx + wi as f32 * 0.5 + (-wi as f32) * (0.33 - 0.11 * fi);
                    let bw = (wi as f32 * (0.66 - 0.22 * fi)) as i32;
                    mpool.new_dust(&mut mirror, &dc, &w, Vec2::new(px, dy - 20.0), bw, 20, 229, 0.0, 0.0, 0, Color::default(), 1.0);
                    mirror.next_float();
                }
            }
        }
        if mirror.next_max(5) == 0 && ai1 < 150.0 {
            for _ in 0..3 {
                mirror.next_double(); mirror.next_double();
                mirror.next_float(); mirror.next_float();
                for _ in 0..6 {
                    mpool.new_dust(&mut mirror, &dc, &w, Vec2::ZERO, 4, 4, 229, 0.0, 0.0, 0, Color::default(), 1.0);
                    mirror.next_float();
                }
                mirror.next_max(2);
            }
        }
        let tail_idx = if mirror.next_max(3) != 0 && ai1 < 150.0 {
            let idx = mpool.new_dust(&mut mirror, &dc, &w, Vec2::new(dx, dy + hi as f32 / 2.0), wi, hi / 2, 241, 0.0, 0.0, 0, Color::default(), 1.0);
            // `:39052-39055` — the custom position/velocity overwrite (the
            // 4π/3 ellipse, NO 0.8 outer factor). The mirror must apply it
            // too, or it compares the raw NewDust position.
            let (rx, ry) = rotated_unit_y_random(&mut mirror, 4.188790321350098);
            let k = 0.8 + mirror.next_float() * 0.2;
            let md = mpool.slot_mut(idx);
            md.position = Vec2::new(
                dx + wi as f32 * 0.5 + rx * (wi / 2) as f32 * k,
                dy + hi as f32 * 0.5 + ry * (hi / 2) as f32 * k,
            );
            md.velocity.x = 0.0;
            md.velocity.y = md.velocity.y.abs() * 0.25;
            Some(idx)
        } else { None };
        assert_eq!(rng, mirror, "the tail tick's full census");
        let ti = tail_idx.expect("the tail fired on the scanned seed");
        let d = pool.slot(ti);
        let m = mpool.slot(ti);
        assert_eq!(d.dust_type, 241);
        assert_eq!(d.position, m.position, "the 4π/3 no-0.8 position");
        assert_eq!(d.velocity.x, 0.0);
        assert!(d.velocity.y >= 0.0, "the |vY|·0.25 fall");
        assert!(!d.no_gravity, "NO noGravity write on the tail");
        assert_eq!(d.fade_in, 0.0, "NO fadeIn write");
        // The position radius: |pos - Center| <= (w/2, h/2) ellipse
        // bound × 1.0 (the k ∈ [0.8, 1.0) factor, NO 0.8 outer).
        let (cx, cy) = (dx + wi as f32 * 0.5, dy + hi as f32 * 0.5);
        let rx = (d.position.x - cx).abs() / (wi / 2) as f32;
        let ry = (d.position.y - cy).abs() / (hi / 2) as f32;
        assert!(rx * rx + ry * ry <= 1.0 + 1e-4, "the unit-ellipse bound");
    }

    #[test]
    fn b6a_the_shield_gate_reads_the_world_state_per_type() {
        // :39101-39115 — dontTakeDamage = ShieldStrengthTower<Kind> != 0.
        let mut w = air_world();
        let p = far_player(&w);
        let mut n = tower(NpcKind::SolarTower, 1535.0, 1330.0);
        w.tower_shield_solar = 100;
        let mut rng = UnifiedRandom::new(1);
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(n.dont_take_damage, "the shield protects");
        w.tower_shield_solar = 0;
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(!n.dont_take_damage, "the broken shield exposes the tower");
        // The other pillars' shields do not protect the solar tower.
        w.tower_shield_vortex = 100;
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(!n.dont_take_damage);
    }

    #[test]
    fn b6a_the_flash_cycle_flips_sounds_and_wraps_at_120() {
        // :39069-39100 — the flip plays NPCDeath58; ai[3]==1 without a
        // flip plays NPCDeath3; ai[3] wraps past 120.
        let mut w = air_world();
        let p = far_player(&w);
        let mut n = tower(NpcKind::VortexTower, 1535.0, 1330.0);
        n.ai[3] = 1.0;
        n.dont_take_damage = true; // the shield stood last tick
        w.tower_shield_vortex = 0; // and just broke
        let mut rng = UnifiedRandom::new(1);
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(fx.shield_break_sound);
        assert!(!fx.shield_hit_sound);
        assert_eq!(n.ai[3], 2.0);
        // No flip + ai[3] == 1 → the hit sound.
        let mut n2 = tower(NpcKind::VortexTower, 1535.0, 1330.0);
        n2.ai[3] = 1.0;
        n2.dont_take_damage = false;
        let mut fx2 = PillarEffects::default();
        pillar_ai_step(&mut n2, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx2, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(fx2.shield_hit_sound);
        assert!(!fx2.shield_break_sound);
        // The wrap: 120 → 121 → 0.
        let mut n3 = tower(NpcKind::VortexTower, 1535.0, 1330.0);
        n3.ai[3] = 120.0;
        let mut fx3 = PillarEffects::default();
        pillar_ai_step(&mut n3, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx3, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n3.ai[3], 0.0);
    }

    #[test]
    fn b6a_the_regen_counter_is_inert_like_vanilla() {
        // :39117-39130 — the second if's else zeroes the counter every
        // tick it is < 60: a far player never accumulates past 1.
        let w = air_world();
        let mut p = Player::spawn(&w);
        p.x = 1600.0;
        p.y = 1400.0;
        let mut n = tower(NpcKind::SolarTower, 1535.0, 1330.0);
        // Move the tower far from the player (> 2000 px) without hitting
        // the clamps: the player goes far instead.
        p.x = 1600.0 + 2500.0;
        let mut rng = UnifiedRandom::new(1);
        let mut fx = PillarEffects::default();
        for _ in 0..120 {
            pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
            assert_eq!(n.local_ai[0], 0.0, "the vanilla counter never accumulates");
        }
        // Externally seeded 59 → the increment reaches 60 → the +200 heal.
        n.hp = 1000;
        n.local_ai[0] = 59.0;
        pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.hp, 1200);
        assert_eq!(n.local_ai[0], 0.0);
    }

    #[test]
    fn b6a_the_hover_lifts_over_near_ground() {
        // :39139-39143 — solid 5 rows under the bottom → Y -= 1.5·0.5.
        let mut w = air_world();
        {
            let t = w.tile_mut(100, 105).unwrap();
            t.kind = 1; // stone
            t.flags |= FLAG_ACTIVE;
        }
        let p = far_player(&w);
        // Bottom tile = (100, 100): x + 65 = 1600, y + 270 = 1600.
        let mut n = tower(NpcKind::StardustTower, 1535.0, 1330.0);
        let mut rng = UnifiedRandom::new(1);
        let mut fx = PillarEffects::default();
        pillar_ai_step(
            &mut n,
            &w,
            &p,
            &mut rng,
            &PillarCtx { has_world_surface: false, remix_world: false, get_good_world: false, ticker: 0 },
            &mut fx,
            &mut DustPool::new(),
            &permissive_ctx(&w),
        );
        assert!((n.y - 1329.25).abs() < 1e-3, "the lift: {}", n.y);
    }

    #[test]
    fn b6a_the_y_edge_clamp_uses_the_x_extent_quirk() {
        // :39154-39155/:39173-39177 — num1302 = maxTilesX*16 = 3200 on a
        // 200×300 world; center.Y > 2240 → 2160 (NOT the 4800 extent).
        let w = air_world();
        let p = far_player(&w);
        let mut n = tower(NpcKind::NebulaTower, 1535.0, 2400.0);
        let mut rng = UnifiedRandom::new(1);
        let mut fx = PillarEffects::default();
        pillar_ai_step(
            &mut n,
            &w,
            &p,
            &mut rng,
            &PillarCtx { has_world_surface: false, remix_world: false, get_good_world: false, ticker: 0 },
            &mut fx,
            &mut DustPool::new(),
            &permissive_ctx(&w),
        );
        // center.Y clamps to 3200-960-80 = 2160 → y = 2160-135 = 2025.
        assert!((n.y - 2025.0).abs() < 1e-3, "the quirk clamp: {}", n.y);
        assert!(n.net_update);
    }

    /// The no-surface fold: keeps the tower at its spawn height (the
    /// classic ctx's worldSurface floor would clamp a 200×300 test world).
    fn no_surface_ctx() -> PillarCtx {
        PillarCtx { has_world_surface: false, remix_world: false, get_good_world: false, ticker: 0 }
    }

    #[test]
    fn b6a_the_solar_arm_spawns_the_goop_with_the_csharp_launch() {
        // :39374-39382 — the full arm: the spawn request, the 60 rearm,
        // the speed band [7, 12), the upward launch.
        let w = air_world();
        let mut p = Player::spawn(&w);
        p.x = 1700.0;
        p.y = 1400.0; // ΔY = 70 < 700, distance < 1080
        let mut n = tower(NpcKind::SolarTower, 1535.0, 1330.0);
        let mut rng = UnifiedRandom::new(42);
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &no_surface_ctx(), &mut fx, &mut DustPool::new(), &permissive_ctx(&w));
        let goop = fx.solar_goop.expect("the goop spawn request");
        let speed = (goop.vx * goop.vx + goop.vy * goop.vy).sqrt();
        assert!((7.0..12.0).contains(&speed), "the launch speed: {speed}");
        assert!(goop.vy < 0.0, "the launch is upward (±π/4 of -UnitY)");
        assert_eq!(n.ai[1], 60.0, "the rearm");
        // The spawn point sits in the tower-top box (± the 1.5-px hover
        // sink of the air world): x ∈ Top.X + [-42.9, +42.9], y ∈ Top.Y-20..Top.Y.
        let top_x = 1535.0 + 65.0;
        assert!((goop.bottom_x as f32 - top_x).abs() <= 44.0);
        // The tower sank 1.5 px (the air world's Down(20)-miss hover), so
        // Top.Y = 1331.5 and the spawn y ∈ [1311.5, 1331.5].
        assert!(goop.bottom_y as f32 >= 1311.0 && goop.bottom_y as f32 <= 1332.0);
        // The cooldown blocks the next tick.
        let mut fx2 = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &no_surface_ctx(), &mut fx2, &mut DustPool::new(), &permissive_ctx(&w));
        assert!(fx2.solar_goop.is_none());
        assert_eq!(n.ai[1], 59.0);
    }

    #[test]
    fn b6a_the_solar_tick_draws_nine_rolls_in_order() {
        // W4-e2: the 4 ambient Next(5) gates (:39351/:39360) + (on a pass)
        // the armed body/plume internals, then the spawn's RandomVector2
        // pair (:39376) + RotatedByRandom pair (:39377) + the speed float.
        // The mirror replays every draw through the same pool calls.
        let w = air_world();
        let dc = permissive_ctx(&w);
        let mut p = Player::spawn(&w);
        p.x = 1700.0;
        p.y = 1400.0;
        let mut n = tower(NpcKind::SolarTower, 1535.0, 1330.0);
        let (wi, hi) = (130i32, 270i32);
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = UnifiedRandom::new(42);
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut fx = PillarEffects::default();
        pillar_ai_step(&mut n, &w, &p, &mut rng, &no_surface_ctx(), &mut fx, &mut pool, &dc);
        // The ambient dust reads the POST-hover position (the no-surface
        // fold keeps the spawn height; the air-world hover sinks 1.5 px).
        let (ax, ay) = (1535.0f32, n.y);
        if mirror.next_max(5) == 0 {
            mirror_ambient_body(&mut mpool, &mut mirror, &dc, &w, ax, ay, wi, hi);
        }
        for i in 0..3 {
            if mirror.next_max(5) == 0 {
                mirror_ambient_plume(&mut mpool, &mut mirror, &dc, &w, ax, ay, wi as f32, i, 6);
            }
        }
        mirror.next_float(); // RandomVector2 X
        mirror.next_float(); // RandomVector2 Y
        mirror.next_double(); // RotatedByRandom +
        mirror.next_double(); // RotatedByRandom −
        mirror.next_float(); // the speed
        assert_eq!(rng, mirror, "the solar spawn tick's gates + bodies + launch");
    }

    #[test]
    fn b6a_the_idle_towers_draw_only_the_four_ambient_gates() {
        // W4-e2: 493/507/422 with the attack bodies deferred — the four
        // ambient Next(5) gates, plus (on a pass) the armed dust bodies'
        // pool internals + jitter. The mirror replays gate + body through
        // the same pool calls; the deferred rearms take their draws with
        // them (the documented deviation).
        let w = air_world();
        let dc = permissive_ctx(&w);
        let p = far_player(&w);
        for (kind, plume) in [
            (NpcKind::StardustTower, 135),
            (NpcKind::NebulaTower, 242),
            (NpcKind::VortexTower, 229),
        ] {
            let mut n = tower(kind, 1535.0, 1330.0);
            let (wi, hi) = (130i32, 270i32);
            let mut rng = UnifiedRandom::new(3);
            let mut mirror = UnifiedRandom::new(3);
            let mut pool = DustPool::new();
            let mut mpool = DustPool::new();
            let mut fx = PillarEffects::default();
            pillar_ai_step(&mut n, &w, &p, &mut rng, &PillarCtx::classic(0), &mut fx, &mut pool, &dc);
            // The body gate (+ internals on a pass). The tower's hover
            // sink does not move X; the body reads position (1535, y).
            if mirror.next_max(5) == 0 {
                mirror_ambient_body(&mut mpool, &mut mirror, &dc, &w, 1535.0, n.y, wi, hi);
            }
            for i in 0..3 {
                if mirror.next_max(5) == 0 {
                    mirror_ambient_plume(&mut mpool, &mut mirror, &dc, &w, 1535.0, n.y, wi as f32, i, plume);
                }
            }
            assert_eq!(rng, mirror, "{kind:?} draws the ambient gates + bodies");
        }
    }

    #[test]
    fn b6a_the_bolt_spend_chips_one_and_flashes_only_while_standing() {
        // Projectile.cs:72949-72953 — −1 per bolt, clamp 0..max, the
        // ai[3]=1 trigger only when the shield was non-zero.
        let mut w = air_world();
        w.tower_shield_solar = 100;
        assert!(bolt_shield_spend(&mut w, NpcKind::SolarTower, 100));
        assert_eq!(w.tower_shield_solar, 99);
        w.tower_shield_solar = 1;
        assert!(bolt_shield_spend(&mut w, NpcKind::SolarTower, 100));
        assert_eq!(w.tower_shield_solar, 0);
        // The broken shield: no flash, the clamp holds at 0.
        assert!(!bolt_shield_spend(&mut w, NpcKind::SolarTower, 100));
        assert_eq!(w.tower_shield_solar, 0);
        // The downed flags are NOT touched (the death arms own them).
        assert!(!w.downed_tower_solar);
        // The max clamp (the post-Moon-Lord halving case).
        w.tower_shield_vortex = 60;
        assert!(bolt_shield_spend(&mut w, NpcKind::VortexTower, 50));
        assert_eq!(w.tower_shield_vortex, 50, "Clamp(59, 0, 50)");
    }

    #[test]
    fn b6a_the_bolt_host_census_matches_the_checkdead_switch() {
        // NPC.cs:79279-79320 — the four id sets; 519 (SolarGoop) is NOT
        // in the solar set; the pillars themselves are not mobs.
        for id in [412u16, 413, 414, 415, 416, 417, 418, 419, 518] {
            assert_eq!(bolt_host_pillar(id), Some(NpcKind::SolarTower), "{id}");
        }
        for id in [425u16, 426, 427, 429] {
            assert_eq!(bolt_host_pillar(id), Some(NpcKind::VortexTower), "{id}");
        }
        for id in [420u16, 421, 423, 424] {
            assert_eq!(bolt_host_pillar(id), Some(NpcKind::NebulaTower), "{id}");
        }
        for id in [402u16, 405, 407, 409, 411] {
            assert_eq!(bolt_host_pillar(id), Some(NpcKind::StardustTower), "{id}");
        }
        assert_eq!(bolt_host_pillar(519), None, "the goop never chips");
        assert_eq!(bolt_host_pillar(406), None, "CellSmall is not in the set");
        assert_eq!(bolt_host_pillar(428), None, "the larva is not in the set");
        assert_eq!(bolt_host_pillar(517), None);
        // The vanilla-id table (the port's net ids are the known collision).
        assert_eq!(pillar_vanilla_id(NpcKind::SolarTower), Some(517));
        assert_eq!(pillar_vanilla_id(NpcKind::VortexTower), Some(422));
        assert_eq!(pillar_vanilla_id(NpcKind::NebulaTower), Some(507));
        assert_eq!(pillar_vanilla_id(NpcKind::StardustTower), Some(493));
    }
}
