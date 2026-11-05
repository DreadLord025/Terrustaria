"""M4-NPC-5 part A: the caster/sphere framework + the seven kinds.

The vanilla sources (all verified against the 1.4.5 decompile):
- AI_008 caster cycle: NPC.cs:20817-21278
- AI_AttemptToFindTeleportSpot: NPC.cs:18937-19007
- AI_009 sphere core: NPC.cs:21377-21430
- Generic gravity: NPC.cs:90465-90470 (NPC.gravity -> port GRAVITY 0.4)
- NewNPC Bottom semantics: NPC.cs:80771
- SetDefaults rows: 24 (NPC.cs:9078), 25 (:9093), 32 (:9121), 33 (:9135),
  45 (:9322), 665 (ChaosBallTim fold, :17398)
- FindFrame: casters NPC.cs:77019-77054, Fire Imp :76979-77017
- npcFrameCount: Main.cs:67082 (24=10, 29=3, 32=6, 45=3, spheres=1)
"""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) npc_ai.rs: the framework ----------
p = 'crates/tr-app/src/npc_ai.rs'
s = io.open(p, encoding='utf-8').read()

old = 'use crate::npc::Npc;'
assert old in s, 'ai imports'
s = s.replace(old, 'use crate::npc::{Npc, NpcKind};')

old = 'fn npc_size_w(npc: &Npc) -> f32 {'
assert old in s, 'ai helper anchor'
s = s.replace(old, '''/// `AI_008`'s per-kind constants (`NPC.cs`'s aiStyle 8 block).
#[derive(Clone, Copy, Debug)]
pub struct CasterParams {
    /// `num88` — the teleport search radius in tiles (20; the Fire Imp 5,
    /// `NPC.cs:21089-21093`).
    pub teleport_radius: i32,
    /// The post-teleport windup (`ai[1] = num88`, `:21101`).
    pub post_teleport_windup: i32,
    /// The shoot tick — the windup countdown fires the sphere when it
    /// reaches it (`num89` = 25 for the standard casters, `:21115`; the
    /// Fire Imp checks `ai[1] == 10`, `:21166`).
    pub shoot_tick: i32,
}

impl CasterParams {
    /// The Goblin Sorcerer/Tim/Dark Caster group (`:21069`'s triggers with
    /// the `num88 = 20` / `num89 = 25` defaults).
    pub const STANDARD: Self = Self {
        teleport_radius: 20,
        post_teleport_windup: 20,
        shoot_tick: 25,
    };
    /// The Fire Imp (`:21090-21093`'s `num88 = 5`, `:21166`'s shoot tick).
    pub const FIRE_IMP: Self = Self {
        teleport_radius: 5,
        post_teleport_windup: 5,
        shoot_tick: 10,
    };
}

/// `AI_008`'s `ai[0]`/`ai[1]` pair folded into the caller's fields
/// (`npc.caster_counter`/`npc.cast_windup`).
pub struct CasterPhase {
    /// `ai[0]` — the cycle counter (the 500 init, the 100/200/300 shot
    /// triggers, the 650 teleport threshold).
    pub counter: i32,
    /// `ai[1]` — the windup countdown (30 from a trigger, `num88` after a
    /// teleport).
    pub windup: i32,
}

/// `AI_AttemptToFindTeleportSpot` (`NPC.cs:18937-19007`): 100 random
/// attempts, each a random x column scanned downward from a random y in the
/// radius window; the floor must be active+solid, the 3×3 body space above
/// it free (`Collision.SolidTiles(x±1, i-4..i-2)` must be false, fluff 1),
/// the tile above not lava (`:19053`), and the ±5-tile telefrag rect clear
/// of the player's hitbox (`:19056-19071`; the port drops the velocity
/// extrapolation). The caster's own ±1 tile never counts (`:18962`'s skip).
/// Deviations: the Dark Caster's dungeon-wall gate (`:19045-19050`) is
/// constants-off (the dungeon sets land with the dungeon work); the
/// velocity-prediction rect is dropped.
fn find_teleport_spot(
    world: &World,
    rng: &mut u64,
    target_tile: (i32, i32),
    self_tile: (i32, i32),
    radius: i32,
    player_x: f32,
    player_y: f32,
) -> Option<(i32, i32)> {
    for _ in 0..100 {
        let x = target_tile.0
            - radius
            + (crate::app::frame::rand01(rng) * (2 * radius + 1) as f32) as i32;
        let y_start = target_tile.1
            - radius
            + (crate::app::frame::rand01(rng) * (2 * radius + 1) as f32) as i32;
        for y in y_start..target_tile.1 + radius {
            if (x - self_tile.0).abs() <= 1 && (y - self_tile.1).abs() <= 1 {
                continue;
            }
            // The floor: active + solid (`:18966`'s `nactive` skip plus
            // `:18968`'s `tileSolid` reject — the `teleportInAir: false`
            // default).
            if !world
                .tile(x, y)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                continue;
            }
            // The lava gate (`:19053`'s `else if (lava())` — the tile above
            // the floor must not be lava).
            if world.tile(x, y - 1).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            }) {
                continue;
            }
            // The 3×3 body space above the floor (`:18971-18975`):
            // `SolidTiles(x - 1..x + 1, i - 4..i - 2)` must be false.
            let mut blocked = false;
            'outer: for bx in (x - 1)..=(x + 1) {
                for by in (y - 4)..=(y - 2) {
                    if world
                        .tile(bx, by)
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    {
                        blocked = true;
                        break 'outer;
                    }
                }
            }
            if blocked {
                continue;
            }
            // The telefrag gate (`:19056-19071`): the spot's 16×16 rect
            // inflated by 5 tiles (176 px) must not intersect the player's
            // 20x42 hitbox.
            let sx = (x * 16 - 80) as f32;
            let sy = (y * 16 - 80) as f32;
            if sx < player_x + 10.0
                && sx + 176.0 > player_x - 10.0
                && sy < player_y + 21.0
                && sy + 176.0 > player_y - 21.0
            {
                continue;
            }
            return Some((x, y));
        }
    }
    None
}

/// `AI_008`'s caster cycle (`NPC.cs:20817-21278`). The order is vanilla:
/// the stand snap, the 500 counter init (`:20836-20843`), `ai[0]++`
/// (`:20993`), the shot triggers at 100/200/300 (`:21069-21073` — `flag5`'s
/// firing-distance gate is constant-on in the port, the
/// `Main.MaxWorldViewSize` rectangle lands with the camera work), the
/// teleport attempt at 650 (`:21083-21112`), the generic gravity + floor
/// stop (`:90465-90470`'s `NPC.gravity` block + `UpdateCollision`), then
/// the windup countdown + the shoot (`:21113+`). Returns true on the shoot
/// tick — the caller spawns the sphere through `wants_sphere`.
pub fn caster_cycle(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    rng: &mut u64,
    params: CasterParams,
    phase: &mut CasterPhase,
) -> bool {
    // The stand: `velocity.X *= 0.93`, snapped to 0 inside ±0.1
    // (`:20831-20835`).
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    npc.facing = if target_x >= npc.x + npc.size().0 * 0.5 {
        1
    } else {
        -1
    };
    if phase.counter == 0 {
        phase.counter = 500;
    }
    phase.counter += 1;
    if phase.counter == 100 || phase.counter == 200 || phase.counter == 300 {
        phase.windup = 30;
    }
    if phase.counter >= 650 {
        phase.counter = 1;
        let (w, h) = npc.size();
        let self_tile = (
            ((npc.x + w * 0.5) / TILE_PX) as i32,
            ((npc.y + h * 0.5) / TILE_PX) as i32,
        );
        if let Some((tx, ty)) = find_teleport_spot(
            world,
            rng,
            ((target_x / TILE_PX) as i32, (target_y / TILE_PX) as i32),
            self_tile,
            params.teleport_radius,
            target_x,
            target_y,
        ) {
            phase.windup = params.post_teleport_windup;
            // The teleport placement (`:20924-20930`): the feet land on the
            // spot's floor tile, the X centred on the tile, velocity zeroed.
            npc.x = tx as f32 * TILE_PX - w * 0.5 + 8.0;
            npc.y = ty as f32 * TILE_PX - h;
            npc.vx = 0.0;
            npc.vy = 0.0;
        }
    }
    // The X stand decay meets the tiles (`UpdateCollision`'s X stop).
    let (w, h) = npc.size();
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
    }
    // The generic gravity + floor stop (`:90465-90470` + `UpdateCollision`).
    npc.vy = (npc.vy + crate::npc::GRAVITY).min(crate::npc::MAX_FALL);
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
    } else {
        npc.vy = 0.0;
    }
    // The windup countdown + the shoot tick (`:21113-21175`).
    if phase.windup > 0 {
        phase.windup -= 1;
        if phase.windup == params.shoot_tick {
            return true;
        }
    }
    false
}

/// `AI_009`'s sphere core (`NPC.cs:21404-21429`): the velocity points at
/// the player ONCE (`target == 255`'s launch arm — `TargetClosest` + the
/// normalized delta × `num120`), then the ball flies straight
/// (`noGravity` + `noTileCollide`: the generic `position += velocity`
/// integration, `:90495-90498`). The port launches while the velocity is
/// still zero. `num120` is 6, the Burning Sphere 5 (`:21395-21399`).
pub fn sphere_step(npc: &mut Npc, target_x: f32, target_y: f32, world: &World) {
    let (w, h) = npc.size();
    if npc.vx == 0.0 && npc.vy == 0.0 {
        let dx = target_x - (npc.x + w * 0.5);
        let dy = target_y - (npc.y + h * 0.5);
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        let speed = match npc.kind {
            NpcKind::BurningSphere => 5.0,
            _ => 6.0,
        };
        npc.vx = dx / d * speed;
        npc.vy = dy / d * speed;
    }
    npc.x += npc.vx;
    npc.y += npc.vy;
    npc.facing = if npc.vx >= 0.0 { 1 } else { -1 };
    // The off-world guard (the flyer_step shape): the balls fly through
    // everything, so the only stop is the world edge.
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

fn npc_size_w(npc: &Npc) -> f32 {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc_ai.rs ok')

# ---------- 2) npc.rs: GRAVITY/MAX_FALL pub(crate) ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = 'const GRAVITY: f32 = 0.4;\nconst MAX_FALL: f32 = 8.0;'
assert old in s, 'gravity consts'
s = s.replace(old, 'pub(crate) const GRAVITY: f32 = 0.4;\npub(crate) const MAX_FALL: f32 = 8.0;')

# ---------- the enum ----------
old = '''    /// `NPCID.MeteorHead` (23) — the meteor hover (`aiStyle 5`).
    MeteorHead,
}'''
assert old in s, 'enum tail'
s = s.replace(old, '''    /// `NPCID.MeteorHead` (23) — the meteor hover (`aiStyle 5`).
    MeteorHead,
    /// `NPCID.GoblinSorcerer` (29) — the goblin-invasion caster (`aiStyle 8`:
    /// 18x38, 40 HP).
    GoblinSorcerer,
    /// `NPCID.Tim` (45) — the deep-cavern caster (`aiStyle 8`, `NPC.cs:9322`:
    /// 18x40, 200 HP, value 5000).
    Tim,
    /// `NPCID.DarkCaster` (32) — the dungeon caster (`aiStyle 8`,
    /// `NPC.cs:9121`: 18x40, 50 HP).
    DarkCaster,
    /// `NPCID.FireImp` (24) — the underworld caster (`aiStyle 8`,
    /// `NPC.cs:9078`: 18x40, 70 HP, lava-immune).
    FireImp,
    /// `NPCID.ChaosBall` (30) — the caster's sphere (`aiStyle 9`, 16x16,
    /// 1 HP, `noGravity`/`noTileCollide`). Tim's `ChaosBallTim` clone 665
    /// folds here (identical stats and name, `NPC.cs:17398`).
    ChaosBall,
    /// `NPCID.BurningSphere` (25) — the Fire Imp's sphere (`aiStyle 9`,
    /// `NPC.cs:9093`).
    BurningSphere,
    /// `NPCID.WaterSphere` (33) — the Dark Caster's sphere (`aiStyle 9`,
    /// `NPC.cs:9135`).
    WaterSphere,
}''')

# ---------- to_u16 / from_u16 ----------
old = '''            NpcKind::MeteorHead => 14,
        }
    }'''
assert old in s, 'to_u16 tail'
s = s.replace(old, '''            NpcKind::MeteorHead => 14,
            NpcKind::GoblinSorcerer => 15,
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
            NpcKind::ChaosBall => 19,
            NpcKind::BurningSphere => 20,
            NpcKind::WaterSphere => 21,
        }
    }''')

old = '''            14 => NpcKind::MeteorHead,
            _ => return None,'''
assert old in s, 'from_u16 tail'
s = s.replace(old, '''            14 => NpcKind::MeteorHead,
            15 => NpcKind::GoblinSorcerer,
            16 => NpcKind::Tim,
            17 => NpcKind::DarkCaster,
            18 => NpcKind::FireImp,
            19 => NpcKind::ChaosBall,
            20 => NpcKind::BurningSphere,
            21 => NpcKind::WaterSphere,
            _ => return None,''')

# ---------- size ----------
old = '''            NpcKind::MeteorHead => (22.0, 22.0),
            NpcKind::CaveBat => (22.0, 18.0),'''
assert old in s, 'size anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => (22.0, 22.0),
            // The casters: `SetDefaults` case 29 is 18x38, the cases
            // 45/32/24 18x40 (`NPC.cs:8951`/`:9322`/`:9121`/`:9078`).
            NpcKind::GoblinSorcerer => (18.0, 38.0),
            NpcKind::Tim => (18.0, 40.0),
            NpcKind::DarkCaster => (18.0, 40.0),
            NpcKind::FireImp => (18.0, 40.0),
            // The spheres: the cases 30/25/33 are 16x16.
            NpcKind::ChaosBall => (16.0, 16.0),
            NpcKind::BurningSphere => (16.0, 16.0),
            NpcKind::WaterSphere => (16.0, 16.0),
            NpcKind::CaveBat => (22.0, 18.0),''')

# ---------- max_hp ----------
old = '''            NpcKind::MeteorHead => 26,
            NpcKind::KingSlime => 2000,'''
assert old in s, 'max_hp anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 26,
            // The casters: `SetDefaults` cases 29/45/32/24.
            NpcKind::GoblinSorcerer => 40,
            NpcKind::Tim => 200,
            NpcKind::DarkCaster => 50,
            NpcKind::FireImp => 70,
            // The spheres: 1 HP.
            NpcKind::ChaosBall => 1,
            NpcKind::BurningSphere => 1,
            NpcKind::WaterSphere => 1,
            NpcKind::KingSlime => 2000,''')

# ---------- contact_damage ----------
old = '''            NpcKind::MeteorHead => 40,
            NpcKind::Guide => 0,'''
assert old in s, 'contact anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 40,
            // The casters: `SetDefaults` damage 20/20/20/30.
            NpcKind::GoblinSorcerer => 20,
            NpcKind::Tim => 20,
            NpcKind::DarkCaster => 20,
            NpcKind::FireImp => 30,
            // The spheres: damage 20/30/20.
            NpcKind::ChaosBall => 20,
            NpcKind::BurningSphere => 30,
            NpcKind::WaterSphere => 20,
            NpcKind::Guide => 0,''')

# ---------- knock_back_resist / defense / value ----------
old = '            NpcKind::MeteorHead => 0.4,'
assert old in s, 'kb anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 0.4,
            // The casters: `SetDefaults` knockBackResist 0.6/0.6/0.6/0.5.
            NpcKind::GoblinSorcerer => 0.6,
            NpcKind::Tim => 0.6,
            NpcKind::DarkCaster => 0.6,
            NpcKind::FireImp => 0.5,
            // The spheres: 0 (the knockback passes through).
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,''')

old = '            NpcKind::MeteorHead => 6,'
assert old in s, 'defense anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 6,
            // The casters: `SetDefaults` defense 2/4/2/16.
            NpcKind::GoblinSorcerer => 2,
            NpcKind::Tim => 4,
            NpcKind::DarkCaster => 2,
            NpcKind::FireImp => 16,
            // The spheres: defense 0.
            NpcKind::ChaosBall => 0,
            NpcKind::BurningSphere => 0,
            NpcKind::WaterSphere => 0,''')

old = '            NpcKind::MeteorHead => 250.0,'
assert old in s, 'value anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 250.0,
            // The casters: `SetDefaults` value 200/5000/140/350.
            NpcKind::GoblinSorcerer => 200.0,
            NpcKind::Tim => 5000.0,
            NpcKind::DarkCaster => 140.0,
            NpcKind::FireImp => 350.0,
            // The spheres carry no value.
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,''')

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs kinds ok')

# ---------- 3) the state fields + init ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    pub home: Option<(i32, i32)>,
    /// `ai[3]`: x position while descending through water, for the bounce flip.'''
assert old in s, 'fields anchor'
s = s.replace(old, '''    pub home: Option<(i32, i32)>,
    /// `ai[0]`: the caster's cycle counter (`AI_008`'s 500 init, the
    /// 100/200/300 shot triggers, the 650 teleport threshold).
    pub caster_counter: i32,
    /// `ai[1]`: the caster's windup countdown (30 from a shot trigger,
    /// `num88` after a teleport; the shoot fires at the params' tick).
    pub cast_windup: i32,
    /// The caster's sphere spawn request — the vanilla `NPC.NewNPC` at the
    /// shoot tick (`AI_008`), resolved by the world sim after the update
    /// pass (the borrow-checker-safe `wants_minion` shape). The payload is
    /// the (kind, bottom-centre x, bottom-centre y) spawn point.
    pub wants_sphere: Option<(NpcKind, f32, f32)>,
    /// `ai[3]`: x position while descending through water, for the bounce flip.''')

old = '            home: None,'
assert old in s, 'init anchor'
s = s.replace(old, '''            home: None,
            caster_counter: 0,
            cast_windup: 0,
            wants_sphere: None,''')

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs fields ok')

# ---------- 4) the update split + dispatch ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '    pub fn update(&mut self, world: &World, player: &mut Player, dt: f32) {'
assert old in s, 'update head'
s = s.replace(old, '''    pub fn update(&mut self, world: &World, player: &mut Player, dt: f32) {
        // The deterministic scratch stream — the tests and every
        // non-caster kind never consume it (only the caster teleport
        // search draws `rand01`).
        let mut scratch = 0u64;
        self.update_rng(world, player, dt, &mut scratch);
    }

    /// The real step with the caller's RNG stream — the caster teleport
    /// search draws from it where the C# hits `Main.rand` (`NPC.cs:18948`).
    pub fn update_rng(&mut self, world: &World, player: &mut Player, dt: f32, rng: &mut u64) {''')

old = '''            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
        }'''
assert old in s, 'dispatch tail'
s = s.replace(old, '''            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster
            | NpcKind::FireImp => self.update_caster(world, player, rng),
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                self.update_sphere(world, player)
            }
        }''')

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs update split ok')

# ---------- 5) update_caster / update_sphere ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '    fn contact(&mut self, player: &mut Player) {'
assert old in s, 'contact anchor'
s = s.replace(old, '''    /// `AI_008` through the framework's `caster_cycle` (`NPC.cs`'s aiStyle
    /// 8 block, `:20817-21278`). The shoot tick spawns the sphere through
    /// `wants_sphere`: the Goblin Sorcerer → the Chaos Ball 30 (`:21238`),
    /// Tim → its ChaosBallTim clone 665 (`:21242`, folded into `ChaosBall`
    /// — identical stats and name), the Dark Caster → the Water Sphere 33
    /// (`:21246`), the Fire Imp → the Burning Sphere 25 at `ai[1] == 10`
    /// (`:21164-21173`). The spawn point keeps `NPC.NewNPC`'s `Bottom`
    /// semantics (`:80771`).
    fn update_caster(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (pcx, pcy) = player.center();
        let params = match self.kind {
            NpcKind::FireImp => crate::npc_ai::CasterParams::FIRE_IMP,
            _ => crate::npc_ai::CasterParams::STANDARD,
        };
        let mut phase = crate::npc_ai::CasterPhase {
            counter: self.caster_counter,
            windup: self.cast_windup,
        };
        let shot = crate::npc_ai::caster_cycle(self, world, pcx, pcy, rng, params, &mut phase);
        self.caster_counter = phase.counter;
        self.cast_windup = phase.windup;
        if shot {
            let (w, _) = self.size();
            let (sx, sy) = match self.kind {
                // `:21238`/`:21242`/`:21246`: `(pos.X + width/2, pos.Y - 8)`.
                NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                    (self.x + w * 0.5, self.y - 8.0)
                }
                // `:21171`: `(pos.X + width/2 + direction*8, pos.Y + 20)`.
                NpcKind::FireImp => {
                    (self.x + w * 0.5 + self.facing as f32 * 8.0, self.y + 20.0)
                }
                _ => return,
            };
            let sphere = match self.kind {
                NpcKind::GoblinSorcerer | NpcKind::Tim => NpcKind::ChaosBall,
                NpcKind::DarkCaster => NpcKind::WaterSphere,
                NpcKind::FireImp => NpcKind::BurningSphere,
                _ => return,
            };
            self.wants_sphere = Some((sphere, sx, sy));
        }
    }

    /// `AI_009` through the framework's `sphere_step` (`NPC.cs`'s aiStyle 9
    /// core, `:21404-21429`): the one-shot launch, then the straight fly
    /// through the tiles (`noGravity`/`noTileCollide`).
    fn update_sphere(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::sphere_step(self, pcx, pcy, world);
    }

    fn contact(&mut self, player: &mut Player) {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs caster/sphere fns ok')

# ---------- 6) find_frame arms ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            NpcKind::Guide => {
                if self.vy != 0.0'''
assert old in s, 'find_frame guide anchor'
s = s.replace(old, '''            // The casters (`FindFrame`'s case 29/32/45 group,
            // `NPC.cs:77019-77054`): the pose machine — row 0 stands, row 1
            // is the cast windup (`ai[1] > 0`), row 2 airborne. The Dark
            // Caster's Skeletron pose (`localAI[3]`, `:77050-77053`) rides
            // the skeleton-head work.
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                self.frame = if self.vy != 0.0 {
                    2
                } else if self.cast_windup > 0 {
                    1
                } else {
                    0
                };
            }
            // The Fire Imp (`NPC.cs:76979-77017`): the 10-frame strip —
            // rows 0..3 walk (5 steps each), the cast pose snaps to row 4
            // and advances to the held row 9.
            NpcKind::FireImp => {
                self.frame_counter += 1.0;
                if self.cast_windup > 0 {
                    if self.frame < 4 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    if self.frame_counter > 5.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 9 {
                        self.frame = 9;
                    }
                } else {
                    if self.frame_counter > 5.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 3 {
                        self.frame = 0;
                    }
                }
            }
            // The spheres: `Main.npcFrameCount` 30/25/33 = 1.
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {}
            NpcKind::Guide => {
                if self.vy != 0.0''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs find_frame ok')

# ---------- 7) render_key arms ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            NpcKind::MeteorHead => 14,
        } << 5;'''
assert old in s, 'render_key anchor'
s = s.replace(old, '''            NpcKind::MeteorHead => 14,
            NpcKind::GoblinSorcerer => 15,
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
            NpcKind::ChaosBall => 19,
            NpcKind::BurningSphere => 20,
            NpcKind::WaterSphere => 21,
        } << 5;''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs render_key ok')

# ---------- 8) spawn_caster ctor ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    /// Spawns a cave flyer hanging in the open air of the cave layer (the'''
assert old in s, 'spawn_caster anchor'
s = s.replace(old, '''    /// Spawns a caster standing on the found ground tile (`SpawnNPC`'s
    /// `Bottom = (x * 16 + 8, y * 16)` semantics, `NPC.cs:4934`): the feet
    /// on the ground tile's top, the X centred on the tile.
    pub fn spawn_caster(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = ground_y as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(
            kind,
            x,
            y,
            if (x_tile % 2) as i32 == 0 { 1 } else { -1 },
            0.0,
        ))
    }

    /// Spawns a cave flyer hanging in the open air of the cave layer (the''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs spawn_caster ok')
print('PART A DONE')
