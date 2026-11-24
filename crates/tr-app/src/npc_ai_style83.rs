//! The aiStyle-83 family — the cultist ritual (`NPC.AI()`'s
//! `else if (this.aiStyle == 83)` arm, `NPC.cs:36757-36925`). W4-d31
//! (research/25 row 83, the 75-83 census band's only MISSING family),
//! CODE-ONLY MODE (no compiler invoked; the witness tests below are
//! written, not run).
//!
//! ## Census
//!
//! `this.aiStyle = 83;` across `NPC.cs` returns exactly TWO hits — the
//! family is:
//! * CultistTablet **437** (`ID/NPCID.cs:11549`, `SetDefaults`
//!   `NPC.cs:14176-14194`: 54×54, lifeMax 400, damage/defense 0, value 0,
//!   npcSlots 0, kb 0, noGravity + noTileCollide, dontTakeDamage,
//!   netAlways, chaseable false; HitSound NPCHit4, DeathSound NPCDeath14),
//! * CultistDevote **438** (`ID/NPCID.cs:11551`, `:14195-14211`: 22×40,
//!   lifeMax 400, damage/defense 0, value 0, npcSlots 0, kb 0, lavaImmune,
//!   netAlways, chaseable false; HitSound NPCHit1, DeathSound NPCDeath1).
//!
//! No dynamic `aiStyle` reassignment to 83 exists. The spawned 379
//! (CultistArcherBlue) rides aiStyle 3 (`:13860`) and the 439
//! (CultistBoss) rides aiStyle 84 (`:14216`, `npc_ai_cultist.rs`'s
//! census) — both are REQUESTS here, not this arm's kinds.
//!
//! ## Wiring (the W4-d31 wiring wave, the d10 activation contract)
//!
//! The arm is WIRED on the style axis: `npc.rs::update_rng_dust`'s
//! `83 =>` arm (the solo path) + the pooled branch in
//! `world_sim/npcs.rs`'s tick closure (the live route, the style75
//! precedent). The 437/438 KINDS ride the roster wave (W4-d40
//! `7fec14aa` rowed the SetDefaults kinds); until they merge no kind has
//! `ai_style() == 83` and the dispatch is dormant. The sim's
//! `ritual_link_prepass` re-derives the slot-aligned ai fields from the
//! stable uids each tick; `resolve_ritual_requests` allocates the spawns
//! + links + `TabletDestroyed`; the 526 shots spawn owner-re-pinned to
//! the local id (the C# call sites pass `Main.myPlayer`, the fishron
//! `spawn_hostile_ai_owned` precedent — the generic drain's 255 would
//! leave the channel-track tail inert). The death volley rides a side
//! channel (the dying tablet is culled by the death retain before the
//! generic shot drain). The 379 archer has no ported kind yet — its
//! spawn folds to None (the recorded roster-wave gap, ANALYSIS).
//!
//! ## The machine
//!
//! **The tablet 437** (`:36759-36888` + the light tail `:36921-36924`):
//! * the `localAI[3]` first-tick latch (`:36761-36792`):
//!   `CultistRitual.CheckFloor(Center)` finds the four ritual spots
//!   (the ±10/±6-tile columns, `CultistRitual.cs:102-131`); a failed
//!   floor kills the tablet (`life = 0` + HitEffect + deactivate,
//!   `:36768-36771`). The four spawns: spots 0/3 → 379 archers (their
//!   slots ride `ai[0]`/`ai[1]`), spots 1/2 → 438 devotees (their slots
//!   ride `localAI[1]`/`localAI[2]`); every spawn's `ai[3] =
//!   -(whoAmI + 1)` (the owner link) + netUpdate. `NewNPC` places by
//!   `Bottom = (X, Y)` (`NPC.cs:80770`); the spawn Y is
//!   `spot.Y * 16 - 48` (three tiles up, `:36779`).
//! * the devotee-release block (`:36793-36804`): a struck devotee sets
//!   the tablet's `localAI[0] = 1` (`:36911-36914`); the tablet then
//!   latches it to 2 and releases BOTH devotees (`ai[1] = 1`, the
//!   devotee's own despawn counter) and BOTH archers (`ai[3] = 0` — the
//!   owner link drops — then `TargetClosest()`).
//! * the boss-summon block (`:36805-36846`): while `ai[0] != -1`, when
//!   all four ritual NPCs are gone (the active+type scan), the tablet
//!   arms `ai[0] = -1`, rolls ONE `Next(2)` for the ±11-tile side
//!   (`ToDirectionInt`: 0 → -1, `Utils.cs:2010-2017`), scans the
//!   -5..11 row window for the first `SolidTile` with a clear 3×3 body
//!   (the `:36835-36838` last-row fallback), spawns the 439 boss at
//!   `(col * 16 + 10, row * 16 - 2)` facing the tablet
//!   (`Math.Sign(Center.X - col * 16 - 10)`), stores its slot in
//!   `ai[2]`, and calls `CultistRitual.TabletDestroyed()` (the 43200
//!   respawn delay, `CultistRitual.cs:54-57` — the port's
//!   `world_events::tablet_destroyed`).
//! * the dissolve (`:36847-36887`): `ai[3]` counts up; at >300 the
//!   tablet dies (`HitEffect(0, 9999.0)`) and fires the death volley —
//!   SIX 526 particles on the full random ring (3 draws each: speed
//!   `NextFloat`, then `RotatedByRandom(TAU)`'s two `NextDouble`s),
//!   then TWENTY gated rolls (`Next(2) != 0` → 3 more draws + one
//!   `NextFloat` radius, the -20 Y launch); every 10th tick past 120
//!   (`ai[3] % 10 == 1`) fires ONE ring particle at the 25-px radius.
//!   All 526s aim at the boss's centre (`Main.npc[ai[2]].Center` rides
//!   the trailing `ai0`/`ai1` payload).
//! * the light tail (`:36921-36924`): the constant
//!   `AddLight(Center, 0.8, 0.75, 0.55)` — it runs even on the
//!   spawn/dissolve ticks (only the failed-floor and devotee-orphan
//!   paths `return` before it).
//!
//! **The devotee 438** (`:36889-36920`): the ×0.93 X decay with the
//! ±0.1 snap (the double-compare, `:36891-36895`); the owner slot
//! `(int)(0 - ai[3] - 1)` — a -1 result (the link was never set) kills
//! the devotee (`:36897-36903`); the face-the-owner flip
//! (`Math.Sign(owner.Center.X - Center.X)`, zero keeps the old
//! direction, `:36904-36910`); a `justHit` while the owner's
//! `localAI[0] == 0` sets it to 1 (the release request,
//! `:36911-36914`); the `ai[0]` 300-tick pose cycle (`:36915-36919`).
//!
//! ## RNG order (by construction)
//!
//! Every `Main.rand` draw the arm makes is consumed here, in C# textual
//! order, on the caller's stream:
//! 1. the boss-summon side (`:36824`): ONE `Next(2)` — only on the
//!    all-four-gone tick.
//! 2. the dissolve death volley (`:36857-36876`, the `ai[3] > 300`
//!    tick): per ring particle `NextFloat()` (speed) THEN
//!    `RotatedByRandom(6.2831854820251465)`'s TWO `NextDouble()`s
//!    (`Utils.cs:1697-1700`, left operand first) — 3 draws × 6; then
//!    20 × `Next(2)`, each non-zero gate drawing 3 more (speed + the
//!    rotation pair) + ONE `NextFloat()` (the 45..90 radius,
//!    `:36873`) — the gate draw is consumed even when it rejects.
//! 3. the dissolve trickle (`:36879-36886`): the same 3-draw shape,
//!    ONE particle per `ai[3] % 10 == 1 && ai[3] > 120` tick.
//!
//! The 438 devotee block draws NOTHING. `TargetClosest` is the
//! deterministic Manhattan scan (`npc_target.rs`, S1-D-3p) — no draws.
//! `NewNPC`/`CheckFloor`/`SolidTile`/`SolidTiles`/`TabletDestroyed`/
//! `Lighting.AddLight` draw nothing. `HitEffect()`'s dust/gore is the
//! cosmetic skip (the style73 precedent — the death burst's draws ride
//! the death pass when it lands, NOT this arm).
//!
//! ## Wiring seams (WIRED — the W4-d31 wiring wave, the d10 contract)
//!
//! * The pool is slot-aligned (`npcs[me]` is `Main.npc[whoAmI]`, the
//!   `npc_ai_style37.rs` convention); `ai[0..1]`/`localAI[1..2]` hold
//!   slot + 1 and `ai[2]` the boss slot + 1 (the port's carrier-link
//!   convention; the C#'s `ai[2]` holds the RAW slot, `:36842` — the
//!   sim's `ritual_link_prepass` owns the uid→slot+1 translation).
//!   `active` ≈ `hp > 0`; a 0 link is an inactive slot (the wiring
//!   guards below skip it — behaviour-identical to the C#'s
//!   inactive-slot reads/writes).
//! * `Main.netMode` is the single-player 0: every `netMode != 1` arm
//!   is active.
//! * The four ritual spawns + the boss spawn ride
//!   [`Style83Out::spawns`] IN ORDER — the C# writes the returned slot
//!   into the link fields synchronously; the port's caller allocates
//!   after the tick, so the links ride [`Style83Out::links`] as
//!   spawn-index references the caller resolves (the style75
//!   carrier-spawn one-tick fold precedent). The archer
//!   `TargetClosest()` on release rides [`Style83Out::retargets`].
//! * The 526 particle requests carry the boss-centre aim in
//!   `ai0`/`ai1` (the `HostileShotAi` shape); the 526 `SetDefaults`
//!   row + its projectile arm are the projectile family's obligation
//!   (`ProjectileID.CultistBossParticle`, `ID/ProjectileID.cs`).
//! * `HitEffect` + `active = false` ride the `died` flag (the port's
//!   `hp = 0` death convention); the caller owes the deactivation.
//! * `CultistRitual.TabletDestroyed()` rides
//!   [`Style83Out::tablet_destroyed`] — the caller runs
//!   `world_events::tablet_destroyed` (the delay machine is already
//!   ported, `world_events.rs:297-301`).
//!
//! See docs/ANALYSIS.md "The cultist ritual (W4-d31 aiStyle-83,
//! code-only)" + research/281 (the RETIRE/FINAL + the cultist-ritual
//! family declaration).
#![allow(dead_code)] // the SetDefaults row consts are the census witnesses' pins; the dispatch activates with the roster-wave kinds

use crate::npc::{HostileShotAi, Npc};
use crate::npc_ai_style37::rotated_by;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPC.SetDefaults` type 437 (`NPC.cs:14176-14194`), verbatim, with
/// the head defaults the branch leaves untouched (`:8507`
/// `dontTakeDamage = false` — overridden true at `:14191`, `:8525`
/// `alpha = 0`, `:8536` `scale = 1`, `:8553` `canGhostHeal = true`).
pub struct CultistTabletDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit4` (`ID/SoundID.cs`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath14`.
    pub death_sound: (i32, i32),
    pub value: f32,
    pub npc_slots: f32,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub dont_take_damage: bool,
    pub net_always: bool,
    pub chaseable: bool,
}

/// `NPC.cs:14176-14194` — `else if (this.type == 437)`.
pub const CULTIST_TABLET: CultistTabletDefaults = CultistTabletDefaults {
    width: 54,
    height: 54,
    ai_style: 83,
    damage: 0,
    defense: 0,
    life_max: 400,
    hit_sound: (3, 4),
    death_sound: (4, 14),
    value: 0.0,
    npc_slots: 0.0,
    knock_back_resist: 0.0,
    no_gravity: true,
    no_tile_collide: true,
    dont_take_damage: true,
    net_always: true,
    chaseable: false,
};

/// `NPC.SetDefaults` type 438 (`NPC.cs:14195-14211`), verbatim.
pub struct CultistDevoteDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1`.
    pub death_sound: (i32, i32),
    pub value: f32,
    pub npc_slots: f32,
    pub knock_back_resist: f32,
    pub lava_immune: bool,
    pub net_always: bool,
    pub chaseable: bool,
}

/// `NPC.cs:14195-14211` — `else if (this.type == 438)`.
pub const CULTIST_DEVOTE: CultistDevoteDefaults = CultistDevoteDefaults {
    width: 22,
    height: 40,
    ai_style: 83,
    damage: 0,
    defense: 0,
    life_max: 400,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    value: 0.0,
    npc_slots: 0.0,
    knock_back_resist: 0.0,
    lava_immune: true,
    net_always: true,
    chaseable: false,
};

/// One `NPC.NewNPC` request (`:36779` / `:36840`) — the caller
/// allocates the slot, places by `Bottom = (x, y)` (`NPC.cs:80770`)
/// and presets the fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RitualSpawn {
    /// 379 (archer), 438 (devotee) or 439 (the boss).
    pub npc_type: i32,
    pub x: f32,
    pub y: f32,
    /// `ai[3] = -(whoAmI + 1)` on the four ritual spawns (`:36783`/
    /// `:36788`); 0 on the boss.
    pub ai3: f32,
    /// The boss's `direction = spriteDirection = Math.Sign(...)`
    /// (`:36841`); `None` on the ritual four.
    pub facing: Option<i32>,
}

/// The link write-backs the caller resolves AFTER allocating the
/// spawn slots (the C# writes them synchronously — the style75
/// one-tick fold). Indices reference [`Style83Out::spawns`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RitualLinks {
    /// `ai[0] = spawn0 + 1`, `ai[1] = spawn2 + 1` (the archers,
    /// `:36787`).
    pub archers: [Option<usize>; 2],
    /// `localAI[1] = spawn1 + 1`, `localAI[2] = spawn3 + 1` (the
    /// devotees, `:36782`).
    pub devotees: [Option<usize>; 2],
    /// `ai[2] = boss spawn index + 1` (the port's slot+1 link
    /// convention — the C# writes the RAW slot at `:36842`; the
    /// wiring's uid translation owns the +1, the Moon Lord parts'
    /// carrier convention).
    pub boss: Option<usize>,
}

/// The arm's per-tick side effects beyond the NPC pool state.
#[derive(Clone, Debug, Default)]
pub struct Style83Out {
    /// The `NewNPC` requests in C# order (the four ritual spawns on
    /// the latch tick, or the one boss on the summon tick).
    pub spawns: Vec<RitualSpawn>,
    pub links: RitualLinks,
    /// The 526 `CultistBossParticle` requests in C# order; the
    /// boss-centre aim rides `ai0`/`ai1` (the trailing
    /// `NewProjectile` arguments, `:36863`/`:36874`/`:36885`).
    pub shots: Vec<HostileShotAi>,
    /// The release block's archer `TargetClosest()` calls
    /// (`:36801`) — the pool slots to retarget, in order.
    pub retargets: Vec<usize>,
    /// `life = 0` + `HitEffect` + `active = false` ran (`:36768-36770`
    /// / `:36852-36854` / `:36899-36901`); the caller owes the
    /// HitEffect/deactivation pair (the style72 pop precedent).
    pub died: bool,
    /// `CultistRitual.TabletDestroyed()` (`:36844`) — the caller runs
    /// `world_events::tablet_destroyed` (delay = 43200).
    pub tablet_destroyed: bool,
    /// The tablet's constant `AddLight(Center, 0.8, 0.75, 0.55)`
    /// (`:36923`) — `(cx, cy, [r, g, b])` for the `EntityLights` feed.
    /// `None` on the early-return ticks and on the 438.
    pub light: Option<(f32, f32, [f32; 3])>,
}

/// `Math.Sign(float)` — -1/0/+1.
fn sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `Utils.ToDirectionInt` (`Utils.cs:2010-2017`): false → -1, true → 1.
fn to_direction_int(b: bool) -> i32 {
    if b {
        1
    } else {
        -1
    }
}

/// `Vector2.ToTileCoordinates` (`Utils.cs:1807-1810`): the `(int)`
/// truncation THEN the arithmetic `>> 4` (the floor for negatives).
fn to_tile(v: f32) -> i32 {
    (v as i32) >> 4
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) — the active +
/// solid + !solidTop + !halfBrick + slopeless + !actuated test (the
/// `npc_ai_style52.rs` precedent).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out-of-bounds counts as SOLID (the four
/// early returns), the per-tile test is
/// `active && !inActive && tileSolid && !tileSolidTop` (the
/// `npc_ai_caster.rs` precedent).
fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for x in x0..=x1 {
        for y in y0..=y1 {
            if let Some(t) = world.tile(x, y) {
                if t.active()
                    && !t.inactive()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `CultistRitual.CheckFloor` (`CultistRitual.cs:102-131`): the four
/// ritual spots at the ±10/±6-tile columns (`i` walks -5..5 step 2,
/// the ±1 skips are dead — the step never lands on them), each the
/// first floor tile in the -5..11 row window whose 3×3 body space is
/// clear (or clear under the two-column fallback). `None` unless all
/// four columns find a spot.
pub fn check_floor(world: &World, center: (f32, f32)) -> Option<[(i32, i32); 4]> {
    let mut array = [(0i32, 0i32); 4];
    let mut num = 0usize;
    let px = to_tile(center.0);
    let py = to_tile(center.1);
    let mut i = -5;
    while i <= 5 {
        // `:109-112` — the dead ±1 skips, kept verbatim.
        if i != -1 && i != 1 {
            let mut j = -5;
            while j < 12 {
                let x = px + i * 2;
                let y = py + j;
                // `:117` — the floor (solid OR platform) + the body
                // gates. `Framing.GetTileSafely` reads an inactive
                // tile out of bounds (type 0 — not a platform).
                let floor = solid_tile(world, x, y)
                    || world
                        .tile(x, y)
                        .is_some_and(|t| tr_core::tiledata::SET_PLATFORMS.has(t.kind));
                let body_clear = !solid_tiles(world, x - 1, x + 1, y - 3, y - 1)
                    || (!solid_tiles(world, x, x, y - 3, y - 1)
                        && !solid_tiles(world, x + 1, x + 1, y - 3, y - 2)
                        && !solid_tiles(world, x - 1, x - 1, y - 3, y - 2));
                if floor && body_clear {
                    array[num] = (x, y);
                    num += 1;
                    break;
                }
                j += 1;
            }
        }
        i += 2;
    }
    if num != 4 {
        return None;
    }
    Some(array)
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`):
/// `RotatedBy(NextDouble() * max - NextDouble() * max)` — TWO draws,
/// left operand first (the `npc_ai_cultist.rs` precedent).
fn rotated_by_random(px: f32, py: f32, max_radians: f64, rng: &mut UnifiedRandom) -> (f32, f32) {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    rotated_by(px, py, angle, 0.0, 0.0)
}

/// One 526 ring particle (`:36859-36863` / `:36881-36885`): the speed
/// `3 + NextFloat() * 6`, the `UnitY.RotatedByRandom(TAU)` spin (the
/// float-widened `6.2831854820251465` literal the decompiler shows),
/// the `radius`-px offset from the centre, the velocity along the
/// spin, the boss-centre aim on `ai0`/`ai1`. THREE draws.
fn ring_particle(
    out: &mut Style83Out,
    center: (f32, f32),
    radius: f32,
    boss_center: (f32, f32),
    rng: &mut UnifiedRandom,
) {
    let speed = 3.0f32 + rng.next_float() * 6.0f32;
    let (sx, sy) = rotated_by_random(0.0, 1.0, 6.2831854820251465, rng);
    out.shots.push(HostileShotAi {
        kind: 526,
        x: center.0 + sx * radius,
        y: center.1 + sy * radius,
        vx: sx * speed,
        vy: sy * speed,
        damage: 0,
        ai0: boss_center.0,
        ai1: boss_center.1,
        scale: None,
    });
}

/// The pool view the arm reads: `npcs[me]` is the tablet/devotee;
/// `me` is its slot (`base.whoAmI`). The ritual links
/// (`ai[0..1]`/`localAI[1..2]`/`ai[2]`, all slot + 1 — the port's
/// carrier convention, the C#'s `ai[2]` raw slot translated by the
/// wiring) index the same slice. `type_of` resolves a pool NPC's
/// vanilla `NPC.type` (the `:36810`/`:36814` active+type scans read
/// `Main.npc[slot].type` — the `npc_ai_style52.rs` seam): the wiring
/// passes the kind→id table; no ported kind resolves to 379/438 yet,
/// so the summon gate reads all-gone until the ritual kinds land (the
/// witnesses pass a stand-in mapping). `players` feeds the release
/// block's archer `TargetClosest` replays, executed on the pool slots
/// in C# order.
pub fn style83_step(
    npcs: &mut [Npc],
    me: usize,
    world: &World,
    players: &[TargetCandidate],
    npc_type: i32,
    type_of: &dyn Fn(&Npc) -> i32,
    just_hit: bool,
    ticker: u64,
    rng: &mut UnifiedRandom,
) -> Style83Out {
    let mut out = Style83Out::default();
    let (cx, cy) = npcs[me].center();
    if npc_type == 437 {
        // The latch tick's summon-scan fold: the C#'s NewNPC is
        // synchronous, so the `:36805` scan on the SAME tick reads the
        // four fresh (living) ritual NPCs and flag94 stays false. The
        // port's links resolve after the tick — the scan is skipped
        // on the spawn tick (behaviour-identical: the ritual four are
        // alive; the style75 carrier-spawn one-tick fold precedent).
        let mut spawned_this_tick = false;
        // `:36761-36792` — the first-tick latch (netMode != 1 folds true).
        if npcs[me].local_ai[3] == 0.0 {
            npcs[me].local_ai[3] = 1.0;
            npcs[me].net_update = true;
            let Some(spots) = check_floor(world, (cx, cy)) else {
                // `:36768-36771` — the failed floor kills the tablet.
                npcs[me].hp = 0;
                out.died = true;
                return out;
            };
            // `:36773-36791` — the four spawns: 0/3 archers, 1/2 devotees.
            spawned_this_tick = true;
            let mut ai_idx = 0usize; // num1166
            let mut local_idx = 1usize; // num1167
            for (k, &(sx, sy)) in spots.iter().enumerate() {
                let flag93 = k == 1 || k == 2;
                let spawn = RitualSpawn {
                    npc_type: if flag93 { 438 } else { 379 },
                    x: (sx * 16 + 8) as f32,
                    y: (sy * 16 - 48) as f32,
                    ai3: -((me as i32) + 1) as f32,
                    facing: None,
                };
                let idx = out.spawns.len();
                out.spawns.push(spawn);
                if flag93 {
                    out.links.devotees[local_idx - 1] = Some(idx);
                    local_idx += 1;
                } else {
                    out.links.archers[ai_idx] = Some(idx);
                    ai_idx += 1;
                }
            }
        }
        // `:36793-36804` — the devotee release. WIRING GUARD (the d31
        // wiring wave): a 0 link is an INACTIVE slot in the port (the
        // vec culls the dead; the uid pre-pass writes 0 for a missing
        // ritual NPC). The C#'s writes to an inactive slot are
        // unobservable (`SetDefaults` resets on reuse, `NPC.cs:80768`)
        // — skipping them is behaviour-identical; `TargetClosest` on
        // an inactive NPC never runs in the C# either (the slot's
        // `active` gate is the update loop's, the release writes land
        // on a corpse). Valid links keep the verbatim body.
        if npcs[me].local_ai[0] == 1.0 {
            npcs[me].local_ai[0] = 2.0;
            for k in 0..2 {
                let d = npcs[me].local_ai[k + 1] as usize;
                if d > 0 {
                    let d = d - 1;
                    npcs[d].ai[1] = 1.0;
                    npcs[d].net_update = true;
                }
                let a = npcs[me].ai[k] as usize;
                if a > 0 {
                    let a = a - 1;
                    npcs[a].ai[3] = 0.0;
                    // `:36801` — TargetClosest() (faceTarget default true).
                    target_closest(&mut npcs[a], world, players, true, 3, ticker);
                    out.retargets.push(a);
                    npcs[a].net_update = true;
                }
            }
        }
        // `:36805-36846` — the boss summon.
        if npcs[me].ai[0] != -1.0 && !spawned_this_tick {
            let mut flag94 = true;
            for k in 0..2 {
                // WIRING GUARD (the release block's note): a 0 link is
                // an inactive slot — the C#'s `active && type` test
                // reads false there, exactly the skip below.
                let d = npcs[me].local_ai[k + 1] as usize;
                if d > 0 {
                    let d = d - 1;
                    // `:36810-36813` — active && type == 438.
                    if npcs[d].hp > 0 && type_of(&npcs[d]) == 438 {
                        flag94 = false;
                    }
                }
                let a = npcs[me].ai[k] as usize;
                if a > 0 {
                    let a = a - 1;
                    // `:36814-36817` — active && type == 379.
                    if npcs[a].hp > 0 && type_of(&npcs[a]) == 379 {
                        flag94 = false;
                    }
                }
            }
            if flag94 {
                npcs[me].ai[0] = -1.0;
                npcs[me].ai[1] = 0.0;
                npcs[me].ai[3] = 0.0;
                // `:36824` — the ONE side roll.
                let col =
                    cx as i32 / 16 + 11 * to_direction_int(rng.next_max(2) == 0);
                // `:36825-36839` — the floor scan with the last-row
                // fallback.
                let mut row = 0;
                for j in -5..12 {
                    let y = cy as i32 / 16 + j;
                    if solid_tile(world, col, y)
                        && !solid_tiles(world, col - 1, col + 1, y - 3, y - 1)
                    {
                        row = y;
                        break;
                    }
                    if j == 11 {
                        row = y;
                    }
                }
                // `:36840-36844` — the boss spawn + the facing + the
                // ritual-delay reset.
                let bx = (col * 16 + 10) as f32;
                let by = (row * 16 - 2) as f32;
                out.spawns.push(RitualSpawn {
                    npc_type: 439,
                    x: bx,
                    y: by,
                    ai3: 0.0,
                    facing: Some(sign(cx - (col * 16) as f32 - 10.0)),
                });
                out.links.boss = Some(out.spawns.len() - 1);
                npcs[me].net_update = true;
                out.tablet_destroyed = true;
            }
        }
        // `:36847-36887` — the dissolve.
        if npcs[me].ai[0] == -1.0 {
            npcs[me].ai[3] += 1.0;
            // The boss-centre aim: the C# reads
            // `Main.npc[(int)ai[2]].Center` live (the RAW slot, 0
            // valid). The port's link convention is slot + 1 (the
            // Moon Lord parts' carrier convention — 0 = unresolved),
            // so the caller writes `ai[2] = boss slot + 1` when it
            // resolves [`RitualLinks::boss`]; an unresolved link reads
            // the tablet's own centre (the summon tick itself fires no
            // particles: ai[3] restarts at 1).
            let boss_center = if npcs[me].ai[2] > 0.0 {
                npcs[npcs[me].ai[2] as usize - 1].center()
            } else {
                (cx, cy)
            };
            if npcs[me].ai[3] > 300.0 {
                // `:36852-36854` — the death.
                npcs[me].hp = 0;
                out.died = true;
                // `:36857-36864` — the six ring particles.
                for _ in 0..6 {
                    ring_particle(&mut out, (cx, cy), 30.0, boss_center, rng);
                }
                // `:36865-36876` — the twenty gated launches.
                for _ in 0..20 {
                    if rng.next_max(2) != 0 {
                        let speed = 3.0f32 + rng.next_float() * 6.0f32;
                        let (sx, sy) =
                            rotated_by_random(0.0, 1.0, 6.2831854820251465, rng);
                        // `:36873` — the radius draw AFTER the spin.
                        let radius = rng.next_float() * 45.0f32 + 45.0f32;
                        let tx = cx + sx * radius;
                        let ty = cy + sy * radius + 20.0;
                        out.shots.push(HostileShotAi {
                            kind: 526,
                            x: cx + sx * 30.0,
                            y: cy + sy * 30.0,
                            vx: sx * speed,
                            vy: -20.0,
                            damage: 0,
                            ai0: tx,
                            ai1: ty,
                            scale: None,
                        });
                    }
                }
            } else if npcs[me].ai[3] % 10.0 == 1.0 && npcs[me].ai[3] > 120.0 {
                // `:36879-36886` — the trickle.
                ring_particle(&mut out, (cx, cy), 25.0, boss_center, rng);
            }
        }
    }
    if npc_type == 438 {
        // `:36891-36895` — the X decay + the double-compare snap.
        npcs[me].vx *= 0.93;
        if (npcs[me].vx as f64) > -0.1 && (npcs[me].vx as f64) < 0.1 {
            npcs[me].vx = 0.0;
        }
        // `:36896-36903` — the owner slot; -1 kills the devotee.
        let owner = (0.0 - npcs[me].ai[3] - 1.0) as i32;
        if owner == -1 {
            npcs[me].hp = 0;
            out.died = true;
            return out;
        }
        let owner = owner as usize;
        // `:36904-36910` — the face-the-owner flip (zero keeps).
        let dir = sign(npcs[owner].center().0 - cx);
        if dir != npcs[me].facing {
            npcs[me].vx = 0.0;
            npcs[me].facing = dir;
            npcs[me].sprite_dir = dir;
            npcs[me].net_update = true;
        }
        // `:36911-36914` — the release request.
        if just_hit && npcs[owner].local_ai[0] == 0.0 {
            npcs[owner].local_ai[0] = 1.0;
        }
        // `:36915-36919` — the 300-tick pose cycle.
        npcs[me].ai[0] += 1.0;
        if npcs[me].ai[0] >= 300.0 {
            npcs[me].ai[0] = 0.0;
            npcs[me].net_update = true;
        }
    }
    if npc_type == 437 {
        // `:36921-36924` — the constant light (the early-return paths
        // above skipped it, exactly as the C#'s returns skip the tail).
        out.light = Some((cx, cy, [0.8, 0.75, 0.55]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A tablet-shaped NPC (the roster has no 437 kind — the style73
    /// placeholder convention): the 54×54 box, 400 HP.
    fn tablet(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((54.0, 54.0));
        n.hp = 400;
        n.max_hp = 400;
        n
    }

    /// A devotee-shaped NPC: the 22×40 box, 400 HP. `tag` overrides the
    /// `max_hp` type tag the stand-in resolver reads (438 default).
    fn devotee(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((22.0, 40.0));
        n.hp = 400;
        n.max_hp = 400;
        n
    }

    /// Tag a fixture with its vanilla type for `tagged_type` (the hp
    /// stays the C# 400 — only the resolver reads the tag).
    fn tag(n: &mut Npc, ty: i32) {
        n.max_hp = ty;
    }

    /// A ritual arena: flat dirt at y = 40 in a 200×120 world — every
    /// CheckFloor column finds its floor at the same row.
    fn arena() -> World {
        let mut w = World::new("t".into(), 200, 120);
        for x in 0..200 {
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

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

    /// The stand-in `type_of` resolver (the style52 witness
    /// convention): the fixtures ride placeholder Zombie kinds and tag
    /// `max_hp` with the vanilla type, so the resolver reads the tag.
    /// The wiring passes the real kind→id table instead.
    fn tagged_type(n: &Npc) -> i32 {
        n.max_hp
    }

    /// `NPC.cs:14176-14194` / `:14195-14211` — the SetDefaults rows.
    #[test]
    fn the_defaults_rows_match_the_csharp() {
        let t = &CULTIST_TABLET;
        assert_eq!((t.width, t.height), (54, 54));
        assert_eq!(t.ai_style, 83);
        assert_eq!(t.life_max, 400);
        assert_eq!((t.damage, t.defense), (0, 0));
        assert_eq!(t.hit_sound, (3, 4), "NPCHit4 :14184");
        assert_eq!(t.death_sound, (4, 14), "NPCDeath14 :14185");
        assert!(t.no_gravity && t.no_tile_collide && t.dont_take_damage);
        assert!(t.net_always && !t.chaseable);
        let d = &CULTIST_DEVOTE;
        assert_eq!((d.width, d.height), (22, 40));
        assert_eq!(d.ai_style, 83);
        assert_eq!(d.life_max, 400);
        assert_eq!(d.hit_sound, (3, 1), "NPCHit1 :14203");
        assert_eq!(d.death_sound, (4, 1), "NPCDeath1 :14204");
        assert!(d.lava_immune && d.net_always && !d.chaseable);
    }

    /// `CultistRitual.cs:102-131` — the four spots at ±10/±6 tiles,
    /// each the first floor row in the window; a wall over one column
    /// fails the whole floor.
    #[test]
    fn the_floor_check_finds_the_four_columns() {
        let w = arena();
        // Centre at tile (100, 30): the columns 90/94/106/110, floor 40.
        let spots = check_floor(&w, (100.0 * 16.0 + 8.0, 30.0 * 16.0)).expect("the floor");
        assert_eq!(spots, [(90, 40), (94, 40), (106, 40), (110, 40)]);
        // A 3-tile pillar over column 106 blocks its body space.
        let mut w2 = arena();
        for y in 36..40 {
            let i = w2.index(106, y);
            w2.tiles[i] = Tile { kind: 1, ..Default::default() };
            w2.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The pillar's own top (36) is a floor candidate but its body
        // (33..35) is clear — the scan finds the HIGHER floor first.
        let spots2 = check_floor(&w2, (100.0 * 16.0 + 8.0, 30.0 * 16.0)).expect("the raised floor");
        assert_eq!(spots2[2], (106, 36), "the pillar top is the first floor");
        // No floor at all (the world's bottom band is solid, so use a
        // centre whose window ends above the dirt): rows -5..11 around
        // tile y = 20 end at 31 < 40 — no column finds a floor.
        assert!(check_floor(&w, (100.0 * 16.0 + 8.0, 20.0 * 16.0)).is_none());
    }

    /// `:36761-36792` — the latch spawns 379/438/438/379 in order with
    /// the owner links; the failed floor kills the tablet.
    #[test]
    fn the_latch_spawns_the_ritual_four() {
        let w = arena();
        let mut pool = [tablet(100.0 * 16.0 + 8.0 - 27.0, 30.0 * 16.0 - 27.0)];
        let mut rng = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert_eq!(pool[0].local_ai[3], 1.0, "the latch spent");
        assert!(pool[0].net_update);
        assert_eq!(out.spawns.len(), 4);
        let types: Vec<i32> = out.spawns.iter().map(|s| s.npc_type).collect();
        assert_eq!(types, vec![379, 438, 438, 379]);
        for s in &out.spawns {
            assert_eq!(s.ai3, -1.0, "ai[3] = -(whoAmI + 1), whoAmI 0");
        }
        // The spots (90/94/106/110, 40): X = spot*16+8, Y = 40*16-48.
        assert_eq!(out.spawns[0].x, (90 * 16 + 8) as f32);
        assert_eq!(out.spawns[0].y, (40 * 16 - 48) as f32);
        assert_eq!(out.links.archers, [Some(0), Some(3)]);
        assert_eq!(out.links.devotees, [Some(1), Some(2)]);
        assert_eq!(rng.peek(), UnifiedRandom::new(83).peek(), "the latch draws nothing");
        assert!(out.light.is_some(), "the tail light runs on the latch tick");
        // The failed floor: a centre whose window finds no dirt.
        let mut pool2 = [tablet(100.0 * 16.0 + 8.0 - 27.0, 20.0 * 16.0 - 27.0)];
        let out2 = style83_step(&mut pool2, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert!(out2.died);
        assert_eq!(pool2[0].hp, 0);
        assert!(out2.light.is_none(), "the early return skips the tail (:36771)");
    }

    /// `:36793-36804` + `:36911-36914` — a struck devotee requests the
    /// release; the tablet releases both devotees (ai[1] = 1) and both
    /// archers (ai[3] = 0 + TargetClosest).
    #[test]
    fn the_struck_devotee_releases_the_ritual() {
        let w = arena();
        // Pool: tablet 0, archers 1/2, devotees 3/4 (the links +1).
        let mut pool = [
            tablet(1600.0, 480.0),
            devotee(1440.0, 592.0),
            devotee(1760.0, 592.0),
            devotee(1504.0, 592.0),
            devotee(1696.0, 592.0),
        ];
        pool[0].local_ai[3] = 1.0; // the latch spent
        pool[0].ai[0] = 2.0; // archer slot 1 + 1
        pool[0].ai[1] = 3.0; // archer slot 2 + 1
        pool[0].local_ai[1] = 4.0; // devotee slot 3 + 1
        pool[0].local_ai[2] = 5.0; // devotee slot 4 + 1
        pool[1].ai[3] = -1.0;
        pool[2].ai[3] = -1.0;
        pool[3].ai[3] = -1.0; // owner = tablet slot 0 (:36896)
        pool[4].ai[3] = -1.0;
        // The type tags for the summon gate's active+type scan.
        tag(&mut pool[0], 437);
        tag(&mut pool[1], 379);
        tag(&mut pool[2], 379);
        tag(&mut pool[3], 438);
        tag(&mut pool[4], 438);
        let mut rng = UnifiedRandom::new(83);
        // The devotee's justHit sets the tablet's localAI[0] = 1.
        let out = style83_step(&mut pool, 3, &w, &[cand(1600.0, 480.0)], 438, &tagged_type, true, 0, &mut rng);
        assert_eq!(pool[0].local_ai[0], 1.0, "the release request (:36913)");
        assert!(!out.died);
        // The tablet's next tick runs the release.
        let out2 = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert_eq!(pool[0].local_ai[0], 2.0, "the release latch (:36795)");
        assert_eq!(pool[3].ai[1], 1.0, "devotee 0 released (:36798)");
        assert_eq!(pool[4].ai[1], 1.0, "devotee 1 released");
        assert_eq!(pool[1].ai[3], 0.0, "archer 0 link dropped (:36800)");
        assert_eq!(pool[2].ai[3], 0.0, "archer 1 link dropped");
        assert_eq!(out2.retargets, vec![1, 2], "the TargetClosest order");
        assert!(out2.spawns.is_empty(), "the living ritual blocks the summon (:36810-36817)");
        assert_eq!(rng.peek(), UnifiedRandom::new(83).peek(), "the release draws nothing");
    }

    /// `:36805-36846` — all four gone: the ONE Next(2) side roll, the
    /// floor scan, the 439 spawn with the facing, the TabletDestroyed
    /// request. The mirrored stream pins the draw.
    #[test]
    fn the_boss_summon_rolls_one_side_draw() {
        let w = arena();
        let mut pool = [
            tablet(100.0 * 16.0 + 8.0 - 27.0, 30.0 * 16.0 - 27.0),
            devotee(0.0, 0.0),
            devotee(0.0, 0.0),
            devotee(0.0, 0.0),
            devotee(0.0, 0.0),
        ];
        for p in pool.iter_mut().skip(1) {
            p.hp = 0; // all four ritual NPCs gone
        }
        tag(&mut pool[0], 437);
        tag(&mut pool[1], 379);
        tag(&mut pool[2], 379);
        tag(&mut pool[3], 438);
        tag(&mut pool[4], 438);
        pool[0].local_ai[3] = 1.0;
        pool[0].ai[0] = 2.0;
        pool[0].ai[1] = 3.0;
        pool[0].local_ai[1] = 4.0;
        pool[0].local_ai[2] = 5.0;
        let mut rng = UnifiedRandom::new(83);
        let mut mirror = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        let side = to_direction_int(mirror.next_max(2) == 0);
        assert_eq!(rng.peek(), mirror.peek(), "exactly ONE Next(2) (:36824)");
        assert_eq!(pool[0].ai[0], -1.0);
        assert_eq!(pool[0].ai[3], 1.0, "the dissolve counter restarts and the SAME tick's :36849 increments it");
        assert_eq!(out.spawns.len(), 1);
        let boss = &out.spawns[0];
        assert_eq!(boss.npc_type, 439);
        let col = 100 + 11 * side;
        assert_eq!(boss.x, (col * 16 + 10) as f32);
        assert_eq!(boss.y, (40 * 16 - 2) as f32, "the floor row 40");
        assert_eq!(boss.facing, Some(sign(100.0 * 16.0 + 8.0 - (col * 16) as f32 - 10.0)));
        assert_eq!(out.links.boss, Some(0));
        assert!(out.tablet_destroyed, "CultistRitual.TabletDestroyed (:36844)");
    }

    /// `:36879-36886` — the trickle: ONE 3-draw particle per
    /// `ai[3] % 10 == 1 && ai[3] > 120` tick, at the 25-px radius.
    #[test]
    fn the_dissolve_trickle_draws_three_per_tenth_tick() {
        let w = arena();
        let mut pool = [tablet(1600.0, 480.0), devotee(1600.0, 200.0)];
        pool[0].local_ai[3] = 1.0;
        pool[0].ai[0] = -1.0;
        pool[0].ai[2] = 2.0; // the boss slot 1 + 1 (the port's link convention)
        pool[0].ai[3] = 130.0; // ++ → 131: %10 == 1, > 120
        let mut rng = UnifiedRandom::new(83);
        let mut mirror = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert_eq!(out.shots.len(), 1);
        let s = &out.shots[0];
        assert_eq!(s.kind, 526);
        assert_eq!(s.damage, 0);
        // The mirrored 3-draw shape: speed, then the rotation pair.
        let speed = 3.0f32 + mirror.next_float() * 6.0f32;
        let angle = mirror.next_double() * 6.2831854820251465
            - mirror.next_double() * 6.2831854820251465;
        let (sx, sy) = rotated_by(0.0, 1.0, angle, 0.0, 0.0);
        assert_eq!(rng.peek(), mirror.peek(), "the trickle census");
        assert_eq!(s.vx.to_bits(), (sx * speed).to_bits());
        assert_eq!(s.vy.to_bits(), (sy * speed).to_bits());
        assert_eq!(s.x.to_bits(), (1627.0 + sx * 25.0).to_bits());
        // The aim: the boss (slot 1) centre — the 22×40 box halves.
        assert_eq!(s.ai0, 1600.0 + 11.0);
        assert_eq!(s.ai1, 200.0 + 20.0);
        // A non-trickle tick draws nothing.
        pool[0].ai[3] = 131.0; // ++ → 132
        let before = rng.peek();
        let out2 = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert!(out2.shots.is_empty());
        assert_eq!(rng.peek(), before);
    }

    /// `:36852-36876` — the death volley census: 6 × 3 draws, then
    /// 20 × (Next(2) + gated 4). The mirrored replay pins the whole
    /// stream and the shot count.
    #[test]
    fn the_death_volley_replays_the_full_census() {
        let w = arena();
        let mut pool = [tablet(1600.0, 480.0), devotee(1600.0, 200.0)];
        pool[0].local_ai[3] = 1.0;
        pool[0].ai[0] = -1.0;
        pool[0].ai[2] = 2.0; // the boss slot 1 + 1
        pool[0].ai[3] = 300.0; // ++ → 301 > 300: the death tick
        let mut rng = UnifiedRandom::new(83);
        let mut mirror = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert!(out.died);
        assert_eq!(pool[0].hp, 0);
        // The six ring particles.
        let mut gated = 0;
        for _ in 0..6 {
            let _ = mirror.next_float();
            let _ = mirror.next_double();
            let _ = mirror.next_double();
        }
        // The twenty gates.
        for _ in 0..20 {
            if mirror.next_max(2) != 0 {
                gated += 1;
                let _ = mirror.next_float();
                let _ = mirror.next_double();
                let _ = mirror.next_double();
                let _ = mirror.next_float();
            }
        }
        assert_eq!(rng.peek(), mirror.peek(), "the death-volley census");
        assert_eq!(out.shots.len(), 6 + gated);
        // The gated launches ride vy = -20 (:36874).
        for s in &out.shots[6..] {
            assert_eq!(s.vy, -20.0);
        }
        assert!(out.light.is_some(), "the tail light runs on the death tick");
    }

    /// `:36889-36920` — the devotee: the ×0.93 decay + snap, the
    /// orphan kill, the facing flip, the 300-tick cycle. ZERO draws.
    #[test]
    fn the_devotee_decays_faces_and_cycles() {
        let w = arena();
        let mut pool = [tablet(1600.0, 480.0), devotee(1500.0, 480.0)];
        pool[1].ai[3] = -1.0; // owner = (0 - (-1) - 1) = 0
        pool[1].vx = 2.0;
        pool[1].facing = -1; // facing away from the owner at 1627
        let mut rng = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 1, &w, &[cand(1600.0, 480.0)], 438, &tagged_type, false, 0, &mut rng);
        // The decay ran, then the flip zeroed it (:36907).
        assert_eq!(pool[1].vx, 0.0);
        assert_eq!(pool[1].facing, 1, "faces the owner (:36904-36908)");
        assert_eq!(pool[1].sprite_dir, 1);
        assert_eq!(pool[1].ai[0], 1.0, "the pose counter");
        assert!(!out.died);
        assert!(out.light.is_none(), "the 438 has no light tail");
        // The snap: a small velocity zeroes without a flip.
        pool[1].vx = 0.05;
        style83_step(&mut pool, 1, &w, &[cand(1600.0, 480.0)], 438, &tagged_type, false, 0, &mut rng);
        assert_eq!(pool[1].vx, 0.0, "the ±0.1 snap (:36892-36895)");
        // The 300-wrap.
        pool[1].ai[0] = 299.0;
        style83_step(&mut pool, 1, &w, &[cand(1600.0, 480.0)], 438, &tagged_type, false, 0, &mut rng);
        assert_eq!(pool[1].ai[0], 0.0, "the 300-tick wrap (:36915-36918)");
        // The orphan: ai[3] = 0 → owner -1 → death + early return.
        pool[1].ai[3] = 0.0;
        let out2 = style83_step(&mut pool, 1, &w, &[cand(1600.0, 480.0)], 438, &tagged_type, false, 0, &mut rng);
        assert!(out2.died);
        assert_eq!(pool[1].hp, 0);
        assert_eq!(rng.peek(), UnifiedRandom::new(83).peek(), "the 438 draws nothing");
    }

    /// W4-d31 wiring wave — the 0-link WIRING GUARDS (the d10 activation
    /// contract). The port's vec culls the dead, so an unresolved ritual
    /// link reads 0 (the C#'s inactive slot). The release block and the
    /// summon scan must SKIP a 0 link (no underflow panic, no write to a
    /// stranger slot) — behaviour-identical to the C#'s inactive-slot
    /// semantics. WRITTEN NOT RUN (code-only).
    #[test]
    fn the_zero_link_guards_skip_the_inactive_slots() {
        let w = arena();
        // A tablet whose ritual links are ALL unresolved (0) — the ritual
        // four were culled / never spawned. The release request is armed.
        let mut pool = [tablet(100.0 * 16.0 + 8.0 - 27.0, 30.0 * 16.0 - 27.0)];
        pool[0].local_ai[3] = 1.0; // the latch spent
        pool[0].local_ai[0] = 1.0; // the release request armed
        // ai[0..1] / local_ai[1..2] stay 0 (unresolved).
        let mut rng = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        // The release latch advanced but NO retarget ran (all links 0).
        assert_eq!(pool[0].local_ai[0], 2.0, "the release latch (:36795)");
        assert!(out.retargets.is_empty(), "the 0 links skip TargetClosest");
        // The summon scan reads all-gone (every link inactive) → the boss
        // summon arms (ONE Next(2) side roll). The 0-link guard must not
        // panic on the `as usize - 1` underflow.
        assert_eq!(pool[0].ai[0], -1.0, "the all-gone summon arms");
        assert_eq!(out.spawns.len(), 1, "the boss spawn");
        assert_eq!(out.spawns[0].npc_type, 439);
        assert!(out.tablet_destroyed);
        // The stream: exactly ONE Next(2) (the side roll), nothing else.
        let mut mirror = UnifiedRandom::new(83);
        let _ = mirror.next_max(2);
        assert_eq!(rng.peek(), mirror.peek(), "the summon's ONE side draw");
    }

    /// W4-d31 wiring wave — a PARTIALLY resolved link set: one archer link
    /// alive, the other 0. The release writes only the living archer; the
    /// summon gate reads the living archer (type 379) and blocks. WRITTEN
    /// NOT RUN (code-only).
    #[test]
    fn the_partial_links_release_only_the_living_slots() {
        let w = arena();
        // Pool: tablet 0, one living archer 1 (tagged 379). The second
        // archer link + both devotee links are 0 (unresolved).
        let mut pool = [
            tablet(1600.0, 480.0),
            devotee(1440.0, 592.0),
        ];
        tag(&mut pool[0], 437);
        tag(&mut pool[1], 379);
        pool[0].local_ai[3] = 1.0;
        pool[0].local_ai[0] = 1.0; // the release request
        pool[0].ai[0] = 2.0; // archer slot 1 + 1 (living)
        pool[0].ai[1] = 0.0; // the second archer unresolved
        pool[0].local_ai[1] = 0.0; // devotee 0 unresolved
        pool[0].local_ai[2] = 0.0; // devotee 1 unresolved
        pool[1].ai[3] = -1.0;
        let mut rng = UnifiedRandom::new(83);
        let out = style83_step(&mut pool, 0, &w, &[cand(1600.0, 480.0)], 437, &tagged_type, false, 0, &mut rng);
        assert_eq!(pool[0].local_ai[0], 2.0);
        // Only the living archer (slot 1) retargets; its link drops.
        assert_eq!(out.retargets, vec![1], "only the living archer");
        assert_eq!(pool[1].ai[3], 0.0, "the living archer's link dropped");
        // The living archer (379) blocks the summon.
        assert!(out.spawns.is_empty(), "the living archer blocks the summon");
        assert_ne!(pool[0].ai[0], -1.0, "no summon while an archer lives");
        assert_eq!(rng.peek(), UnifiedRandom::new(83).peek(), "the release draws nothing");
    }
}