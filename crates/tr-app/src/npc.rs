//! First enemy pass: a hopping blue slime, a flying demon eye and a walking zombie,
//! modelled on the simplest `NPC.cs` behaviours.

use tr_core::{TILE_PX, World};

use crate::npc_buffs::NpcBuffs;
use crate::player::{Player, collides_rect};

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 8.0;

/// `Collision.WetCollision` (simplified): any water tile overlapping the body.
fn is_wet(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if world.tile(tx, ty).is_some_and(|t| t.liquid > 0) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NpcKind {
    Slime,
    DemonEye,
    /// `NPCID.WanderingEye` (133) — the same `aiStyle 2` machine, night
    /// hardmode spawn (`NPC.cs:4555-4558`).
    WanderingEye,
    Zombie,
    KingSlime,
}

/// The slime colour variants. Blue is `NPCID.BlueSlime` (1); the rest are the
/// negative net ids sharing its `aiStyle 1` machine
/// (`Lang.cs:520-533`, `NPC.SetDefaultsFromNetId`, `NPC.cs:7653-8000`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SlimeVariant {
    #[default]
    Blue,
    Green,
    Pinky,
    Baby,
    Black,
    Purple,
    Red,
    Yellow,
}

/// One slime variant's `SetDefaults` line (`NPC.cs:7653-8000` for the net ids,
/// the `if (this.type == 1)` chain at `NPC.cs:8567` for blue): `life`, melee
/// `damage`, `defense`, the `SetDefaults_ForNetId` size scale, the sprite
/// tint `NPC.color` and the gel drop `min..=max` (0 = none;
/// `ItemDropDatabase.cs:1074-1082`).
#[derive(Clone, Copy)]
pub struct SlimeStats {
    pub life: i32,
    pub damage: i32,
    pub defense: i32,
    pub scale: f32,
    pub color: [u8; 4],
    pub gel: (u32, u32),
    /// `NPC.value` in copper (`NPCLoot_DropMoney`'s base).
    pub value: f32,
}

impl SlimeVariant {
    /// The per-variant `SetDefaults` values. Base 24x18 hitbox, `aiStyle 1`
    /// (blue: `NPC.cs:8567-8580`).
    pub fn stats(self) -> SlimeStats {
        match self {
            SlimeVariant::Blue => SlimeStats {
                life: 25,
                damage: 7,
                defense: 2,
                scale: 1.0,
                color: [0, 80, 255, 100],
                gel: (1, 2),
                value: 25.0,
            },
            SlimeVariant::Green => SlimeStats {
                life: 14,
                damage: 6,
                defense: 0,
                scale: 0.9,
                color: [0, 220, 40, 100],
                gel: (0, 0),
                value: 3.0,
            },
            SlimeVariant::Pinky => SlimeStats {
                life: 150,
                damage: 5,
                defense: 5,
                scale: 0.6,
                color: [250, 30, 90, 90],
                gel: (0, 0),
                value: 10000.0,
            },
            SlimeVariant::Baby => SlimeStats {
                life: 30,
                damage: 13,
                defense: 4,
                scale: 0.9,
                color: [0, 0, 0, 50],
                gel: (0, 0),
                value: 10.0,
            },
            SlimeVariant::Black => SlimeStats {
                life: 45,
                damage: 15,
                defense: 4,
                scale: 1.05,
                color: [0, 0, 0, 50],
                gel: (2, 5),
                value: 20.0,
            },
            SlimeVariant::Purple => SlimeStats {
                life: 40,
                damage: 12,
                defense: 6,
                scale: 1.2,
                color: [200, 0, 255, 150],
                gel: (2, 5),
                value: 10.0,
            },
            SlimeVariant::Red => SlimeStats {
                life: 35,
                damage: 12,
                defense: 4,
                scale: 1.025,
                color: [255, 30, 0, 100],
                gel: (2, 5),
                value: 8.0,
            },
            SlimeVariant::Yellow => SlimeStats {
                life: 45,
                damage: 15,
                defense: 7,
                scale: 1.2,
                color: [255, 255, 0, 100],
                gel: (2, 5),
                value: 10.0,
            },
        }
    }

    /// The `Lang._negativeNpcNameCache` names (localization keys are the C#
    /// strings; the port keeps the English base names).
    pub fn name(self) -> &'static str {
        match self {
            SlimeVariant::Blue => "Blue Slime",
            SlimeVariant::Green => "Green Slime",
            SlimeVariant::Pinky => "Pinky",
            SlimeVariant::Baby => "Baby Slime",
            SlimeVariant::Black => "Black Slime",
            SlimeVariant::Purple => "Purple Slime",
            SlimeVariant::Red => "Red Slime",
            SlimeVariant::Yellow => "Yellow Slime",
        }
    }
}

impl NpcKind {
    pub fn to_u16(self) -> u16 {
        match self {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::WanderingEye => 2,
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 4,
        }
    }

    pub fn from_u16(v: u16) -> Option<Self> {
        Some(match v {
            0 => NpcKind::Slime,
            1 => NpcKind::DemonEye,
            2 => NpcKind::WanderingEye,
            3 => NpcKind::Zombie,
            4 => NpcKind::KingSlime,
            _ => return None,
        })
    }

    /// The vanilla hitboxes: blue slime 24x18 (`NPC.cs:8568-8569`), demon eye
    /// and Wandering Eye 30x32 (`NPC.cs` cases 2/133), zombie 18x40.
    pub fn size(self) -> (f32, f32) {
        match self {
            NpcKind::Slime => (24.0, 18.0),
            NpcKind::DemonEye | NpcKind::WanderingEye => (30.0, 32.0),
            NpcKind::Zombie => (18.0, 40.0),
            NpcKind::KingSlime => (98.0, 92.0),
        }
    }

    fn max_hp(self) -> i32 {
        match self {
            NpcKind::Slime => 25,         // NPC.cs:8572
            NpcKind::DemonEye => 60,      // case 2
            NpcKind::WanderingEye => 300, // case 133
            NpcKind::Zombie => 45,
            NpcKind::KingSlime => 2000,
        }
    }

    fn contact_damage(self) -> i32 {
        match self {
            NpcKind::Slime => 7,         // NPC.cs:8571
            NpcKind::DemonEye => 18,     // case 2
            NpcKind::WanderingEye => 40, // case 133
            NpcKind::Zombie => 14,
            NpcKind::KingSlime => 40,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Npc {
    pub kind: NpcKind,
    /// The slime colour variant (`SlimeVariant`); only meaningful while
    /// `kind == NpcKind::Slime`.
    pub slime: SlimeVariant,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub on_ground: bool,
    pub facing: i32,
    pub hop_timer: f32,
    pub anim: f32,
    pub hurt_cd: f32,
    /// `NPC.immune[256]`: per-owner i-frame ticks, applied by projectile hits
    /// (`Projectile.cs:14087-14119`) and decremented once per NPC update
    /// (`NPC.cs:90454-90459`).
    pub immune: [i32; 256],
    /// `NPC.buffType[20]` / `buffTime[20]` / `buffImmune` — the debuff engine
    /// (`npc_buffs.rs`, `NPC.cs:6193-6197`); the DoT tick runs from
    /// `Npc::update` like the C# buff block of `UpdateNPC`.
    pub buffs: NpcBuffs,
    /// `NPC.frame`: the rendered frame row, advanced by [`Npc::find_frame`] in
    /// the fixed step (the original advances animation in the sim too —
    /// `FindFrame` runs from `NPC.Update`, `NPC.cs:66684`).
    pub frame: u32,
    /// `NPC.frameCounter`: the per-kind accumulator `FindFrame` checks.
    frame_counter: f32,
    /// `NPC.rotation`: the aim tilt. Only the demon eye sets it
    /// (`NPC.cs:74019-74031`: `atan2(velocity.Y, velocity.X)`, `+ π` when
    /// `velocity.X < 0`).
    pub rotation: f32,
    /// Accumulated daylight burn time (zombies only).
    burn: f32,
    /// A boss that just landed wants the app to spawn a minion (`AI_015`).
    pub wants_minion: bool,
    /// `ai[3]`: x position while descending through water, for the bounce flip.
    wet_x: f32,
}

impl Npc {
    pub fn spawn_slime(world: &World, x_tile: i32) -> Option<Self> {
        Self::spawn_slime_variant(world, x_tile, SlimeVariant::Blue)
    }

    /// Spawns one slime variant: the hitbox is the base 24x18 scaled by the
    /// variant's `SetDefaults_ForNetId` size scale, at the surface.
    pub fn spawn_slime_variant(world: &World, x_tile: i32, variant: SlimeVariant) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let stats = variant.stats();
        let scale = stats.scale;
        let (w, h) = (
            NpcKind::Slime.size().0 * scale,
            NpcKind::Slime.size().1 * scale,
        );
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        let mut seed = (x_tile as u32).wrapping_mul(2_654_435_761);
        seed ^= seed >> 15;
        let mut npc = Self::new(
            NpcKind::Slime,
            x,
            y,
            if seed & 1 == 0 { 1 } else { -1 },
            0.4 + (seed % 80) as f32 * 0.01,
        );
        npc.slime = variant;
        npc.max_hp = stats.life;
        npc.hp = stats.life;
        Some(npc)
    }

    /// `NPCID.KingSlime`: the first boss, spawned by the `B` debug key or `TR_BOSS`.
    pub fn spawn_king_slime(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::KingSlime.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        let mut npc = Self::new(NpcKind::KingSlime, x, y, -1, 1.2);
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// Zombies walk on the surface and burn away at dawn.
    pub fn spawn_zombie(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Zombie.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        Some(Self::new(NpcKind::Zombie, x, y, -1, 0.0))
    }

    /// Demon eyes spawn a few tiles above the surface and ignore gravity.
    pub fn spawn_eye(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::DemonEye.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h - 4.0 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(NpcKind::DemonEye, x, y, 1, 0.0))
    }

    fn new(kind: NpcKind, x: f32, y: f32, facing: i32, hop_timer: f32) -> Self {
        Self {
            kind,
            slime: SlimeVariant::Blue,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp: kind.max_hp(),
            max_hp: kind.max_hp(),
            on_ground: false,
            facing,
            hop_timer,
            anim: 0.0,
            hurt_cd: 0.0,
            immune: [0; 256],
            buffs: NpcBuffs::for_kind(kind),
            frame: 0,
            frame_counter: 0.0,
            rotation: 0.0,
            burn: 0.0,
            wants_minion: false,
            wet_x: 0.0,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        let (w, h) = self.size();
        (self.x + w * 0.5, self.y + h * 0.5)
    }

    /// `NPC.knockBackResist` — `StrikeNPC_Inner`'s soft-cap chain input
    /// (blue `NPC.cs:8595` 0.8; demon eye 0.8, Wandering Eye 0.8,
    /// zombie 0.5, King Slime 0; the slime variants multiply the blue
    /// base per `SetDefaultsFromNetId`: green x1.2, pinky x1.4, baby
    /// x0.95, purple x0.9, black/red/yellow x1).
    pub fn knock_back_resist(&self) -> f32 {
        if self.kind == NpcKind::Slime {
            let mult = match self.slime {
                SlimeVariant::Blue
                | SlimeVariant::Black
                | SlimeVariant::Red
                | SlimeVariant::Yellow => 1.0,
                SlimeVariant::Green => 1.2,
                SlimeVariant::Pinky => 1.4,
                SlimeVariant::Baby => 0.95,
                SlimeVariant::Purple => 0.9,
            };
            return 0.8 * mult;
        }
        match self.kind {
            NpcKind::DemonEye => 0.8,
            NpcKind::WanderingEye => 0.8,
            NpcKind::Zombie => 0.5,
            NpcKind::KingSlime => 0.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// `NPC.defense` — `Main.CalculateDamageNPCsTake`'s input
    /// (blue `NPC.cs:8573` 2, demon eye 2, Wandering Eye 20, zombie 6,
    /// King Slime 10; the slime variants ride their `SetDefaultsFromNetId`
    /// lines).
    pub fn defense(&self) -> i32 {
        if self.kind == NpcKind::Slime {
            return self.slime.stats().defense;
        }
        match self.kind {
            NpcKind::DemonEye => 2,
            NpcKind::WanderingEye => 20,
            NpcKind::Zombie => 6,
            NpcKind::KingSlime => 10,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// `NPC.value` in copper — `NPCLoot_DropMoney`'s base
    /// (blue `NPC.cs:8576` 25, demon eye 75, Wandering Eye 500, zombie 60,
    /// King Slime 10000).
    pub fn value(&self) -> f32 {
        if self.kind == NpcKind::Slime {
            return self.slime.stats().value;
        }
        match self.kind {
            NpcKind::DemonEye => 75.0,
            NpcKind::WanderingEye => 500.0,
            NpcKind::Zombie => 60.0,
            NpcKind::KingSlime => 10_000.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// The scaled size for slime variants (the hitbox rides
    /// `SetDefaults_ForNetId`'s size scale).
    pub fn size(&self) -> (f32, f32) {
        let (w, h) = self.kind.size();
        if self.kind == NpcKind::Slime {
            let s = self.slime.stats().scale;
            (w * s, h * s)
        } else {
            (w, h)
        }
    }

    pub fn contains(&self, wx: f32, wy: f32) -> bool {
        let (w, h) = self.size();
        wx >= self.x - 4.0 && wx <= self.x + w + 4.0 && wy >= self.y - 4.0 && wy <= self.y + h + 4.0
    }

    /// `NPC.StrikeNPC`'s knockback half (`NPC.cs:81389-81458`): the
    /// soft-capped impulse applied by `tr_core::damage::knock_back_npc`.
    /// `hit_direction` is the knockback side (+1 = pushed right) and
    /// `knock_back` the weapon's `Item.knockBack`.
    pub fn hurt(&mut self, amount: i32, from_x: f32, knock_back: f32, crit: bool) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp -= amount;
        self.hurt_cd = 0.18;
        let (w, _) = self.size();
        let dir = if self.x + w * 0.5 >= from_x { 1 } else { -1 };
        let (vx, vy) = tr_core::damage::knock_back_npc(
            knock_back,
            self.knock_back_resist(),
            crit,
            amount,
            self.max_hp,
            (self.vx, self.vy),
            dir,
            false,
            false,
        );
        self.vx = vx;
        self.vy = vy;
        self.on_ground = false;
        true
    }

    /// The projectile-hit path: identical to [`Npc::hurt`] but without the
    /// shared `hurt_cd` gate — a projectile's right to hit is decided by the
    /// three C# immunity systems (`Projectile.cs:12577-12624`), not by the
    /// melee cooldown stand-in. The hit still arms `hurt_cd`, so melee is
    /// gated afterward, matching `StrikeNPC`'s 10-tick `NPC.immune` window.
    pub fn hurt_from_projectile(
        &mut self,
        amount: i32,
        from_x: f32,
        knock_back: f32,
        crit: bool,
    ) -> bool {
        if self.hp <= 0 {
            return false;
        }
        self.hp -= amount;
        self.hurt_cd = 0.18;
        let (w, _) = self.size();
        let dir = if self.x + w * 0.5 >= from_x { 1 } else { -1 };
        let (vx, vy) = tr_core::damage::knock_back_npc(
            knock_back,
            self.knock_back_resist(),
            crit,
            amount,
            self.max_hp,
            (self.vx, self.vy),
            dir,
            false,
            false,
        );
        self.vx = vx;
        self.vy = vy;
        self.on_ground = false;
        true
    }

    pub fn update(&mut self, world: &World, player: &mut Player, dt: f32) {
        // `NPC.Update`'s buff block (`NPC.cs:90373-90377`): timer decrement →
        // clear expired → ApplyDOTs, before the AI — DoT damage lands ahead of
        // the frame's movement, exactly like the original.
        self.buffs.update_buffs_tick(&mut self.hp, self.max_hp);
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        // `NPC.Update`'s per-owner i-frame decay (`NPC.cs:90454-90459`).
        for v in self.immune.iter_mut() {
            *v = v.saturating_sub(1);
        }
        self.anim += dt;
        match self.kind {
            NpcKind::Slime => self.update_slime(world, player, dt),
            NpcKind::DemonEye | NpcKind::WanderingEye => self.update_eye(world, player, dt),
            NpcKind::Zombie => self.update_zombie(world, player, dt),
            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
        }
        self.find_frame();
        self.contact(player);
    }

    /// `NPC.FindFrame` (`NPC.cs:66684`) for the four ported kinds. `num` (the
    /// frame height unit) cancels out because frames are tracked as row indices;
    /// the per-kind frame counts are `Main.npcFrameCount` (`Main.cs:67082`):
    /// slime 2, demon eye 2, zombie 3, King Slime 6. `num2` is the motion class
    /// (`NPC.cs:66698-66703`): 2 jumping up, 3 falling, 1 moving, 0 idle —
    /// `aiAction == 4` (attacking) never happens for these kinds in the port.
    fn find_frame(&mut self) {
        let num2 = if self.vy < 0.0 {
            2
        } else if self.vy > 0.0 {
            3
        } else if self.vx != 0.0 {
            1
        } else {
            0
        };
        match self.kind {
            // Case 1 — blue slime (`NPC.cs:70801-70826`): the counter ticks once
            // per step, again when moving and a third time while attacking; at
            // 8.0 the frame advances and wraps over the 2-frame strip.
            NpcKind::Slime => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // Case 50 — King Slime (`NPC.cs:73380-73414`): airborne snaps to
            // frame 4, holds it 4 steps, then frame 5; landing returns to 4 and
            // the grounded cycle runs frames 0..=3 (never reaching 4).
            NpcKind::KingSlime => {
                if self.vy != 0.0 {
                    if self.frame < 4 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 4.0 {
                        self.frame = 5;
                    }
                } else {
                    if self.frame >= 5 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter +=
                        1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                    if self.frame_counter >= 8.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                        if self.frame >= 4 {
                            self.frame = 0;
                        }
                    }
                }
            }
            // Case 2 — demon eye and Wandering Eye (`NPC.cs:73994-74057`): a
            // plain 8-step frame cycle (2 frames for the eye, 4 for the
            // Wandering Eye, `Main.cs:67082`), plus the aim: type 2 is in the
            // first rotation branch (`NPC.cs:74019-74031`) —
            // `rotation = atan2(vy, vx)`, `+ 3.14f` when moving left, and the
            // sprite mirrors (`spriteDirection`) so its pupil follows the
            // flight direction. `rotation` freezes while `velocity.X == 0`
            // exactly like the original (neither `if` fires).
            NpcKind::DemonEye | NpcKind::WanderingEye => {
                // Wandering Eye wraps over Main.npcFrameCount[133] = 4 frames
                // (Main.cs:67082); the demon eye over its 2.
                let frame_count = match self.kind {
                    NpcKind::WanderingEye => 4,
                    _ => 2,
                };
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= frame_count {
                        0
                    } else {
                        self.frame + 1
                    };
                }
                if self.vx > 0.0 {
                    self.rotation = self.vy.atan2(self.vx);
                } else if self.vx < 0.0 {
                    // The original adds the literal `3.14f`, not PI.
                    self.rotation = self.vy.atan2(self.vx) + 3.14;
                }
            }
            // Case 3 — zombie (`NPC.cs:76324-76389`): falling or moving against
            // the facing direction holds the lurch frame 2; standing still is
            // frame 0; walking accumulates `|velocity.X|` through the band
            // 0 → 1 → 2 → 1 at thresholds 8/16/24/32 and restarts.
            NpcKind::Zombie => {
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        // The original leaves `frame.Y` on its previous value
                        // here — the restart tick keeps frame 1.
                        self.frame_counter = 0.0;
                    }
                }
            }
        }
    }

    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Slime.size();
        let (pcx, _) = player.center();
        let (cx, _) = self.center();
        self.hop_timer -= 1.0 / 60.0;
        let near = (pcx - cx).abs() < 60.0 * TILE_PX;
        if self.on_ground && self.hop_timer <= 0.0 && near {
            self.facing = if pcx >= cx { 1 } else { -1 };
            self.vx = self.facing as f32 * 1.4;
            self.vy = -4.2;
            self.on_ground = false;
            self.hop_timer = 1.0 + (self.anim * 7.0).sin().abs() * 0.5;
        }
        // `NPC.UpdateNPC_Gravity`: water lowers gravity and the fall cap.
        let wet = is_wet(world, self.x, self.y, w, h);
        if wet {
            self.vy = (self.vy + 0.2).min(7.0);
        } else {
            self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        }

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let mut collide_y = false;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.6;
            collide_y = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
            collide_y = true;
        }
        // `AI_001_Slimes` water tail: pushed up every tick, with a small bounce when
        // it touches the floor or the surface, so slimes bob at the water line.
        if wet {
            if collide_y {
                self.vy = -2.0;
            }
            if self.vy < 0.0 && self.wet_x == self.x {
                self.facing = -self.facing;
                self.hop_timer = 200.0 / 60.0;
            }
            if self.vy > 0.0 {
                self.wet_x = self.x;
            }
            if self.vy > 2.0 {
                self.vy *= 0.9;
            }
            self.vy -= 0.5;
            if self.vy < -4.0 {
                self.vy = -4.0;
            }
        }
        if self.vx != 0.0 {
            self.facing = if self.vx > 0.0 { 1 } else { -1 };
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    fn update_eye(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::DemonEye.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        self.vx = (self.vx + dx / d * 0.12).clamp(-3.0, 3.0);
        self.vy = (self.vy + dy / d * 0.12).clamp(-3.0, 3.0);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.5;
        }
        let bob = (self.anim * 3.0).sin() * 0.4;
        let ny = self.y + self.vy + bob;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
        } else {
            self.vy = -self.vy * 0.5;
        }
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// `AI_003_Fighters`: walk toward the player and jump when the path is blocked.
    fn update_zombie(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Zombie.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let near = dx.abs() < 80.0 * TILE_PX && dy.abs() < 40.0 * TILE_PX;
        if near {
            self.facing = if dx >= 0.0 { 1 } else { -1 };
            self.vx = self.facing as f32 * 0.6;
        } else {
            self.vx *= 0.9;
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else if self.on_ground {
            // Jump a one-tile ledge or wall instead of grinding into it.
            self.vy = -6.2;
            self.on_ground = false;
        } else {
            self.vx = 0.0;
        }

        let ny = self.y + self.vy;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }

        if world.day_time {
            self.burn += 1.0 / 60.0;
            if self.burn >= 1.0 {
                self.burn -= 1.0;
                self.hp -= 6;
            }
        } else {
            self.burn = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// Simplified `AI_015_KingSlime`: big hops toward the player, a teleport when the
    /// player escapes, and a burst of blue slimes when it lands. Minion spawning is
    /// reported through `spawn_minion` so the app can add them to its NPC list.
    fn update_king_slime(&mut self, world: &World, player: &mut Player, dt: f32) {
        let (w, h) = NpcKind::KingSlime.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        self.hop_timer -= dt;
        if self.on_ground && self.hop_timer <= 0.0 {
            self.vx = self.facing as f32 * 2.4;
            self.vy = -8.0;
            self.on_ground = false;
            self.hop_timer = 1.4 + (self.anim * 2.7).sin().abs() * 0.8;
        }
        // Teleport next to the player when they outrun the boss.
        if (dx.abs() > 70.0 * TILE_PX || (pcy - cy).abs() > 50.0 * TILE_PX) && self.on_ground {
            let side = if dx >= 0.0 { -1 } else { 1 };
            let tx = ((pcx / TILE_PX) as i32 + side * 10).clamp(4, world.width - 5);
            let top = world.surface_of(tx);
            if top < world.height - 2 {
                let (w2, h2) = NpcKind::KingSlime.size();
                let nx = tx as f32 * TILE_PX + 8.0 - w2 * 0.5;
                let ny = top as f32 * TILE_PX - h2;
                if !collides_rect(world, nx, ny, w2, h2) {
                    self.x = nx;
                    self.y = ny;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.on_ground = true;
                }
            }
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let was_air = !self.on_ground;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.5;
            if was_air {
                self.wants_minion = true;
            }
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    fn contact(&self, player: &mut Player) {
        let (w, h) = self.size();
        if self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
        {
            let (cx, _) = self.center();
            let dmg = if self.kind == NpcKind::Slime {
                self.slime.stats().damage
            } else {
                self.kind.contact_damage()
            };
            player.hurt(dmg, cx);
        }
    }

    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.hp.max(0) as u32) << 4;
        h ^= (self.facing as u32) << 1;
        h ^= (self.hurt_cd > 0.0) as u32;
        // The rendered frame and the eye's aim tilt are the animation state the
        // world pass bakes in (`frame_counter`/`anim` only matter through them).
        h ^= self.frame << 2;
        h ^= self.rotation.to_bits().wrapping_mul(0x27D4_EB2F);
        h ^= match self.kind {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::WanderingEye => 2,
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 4,
        } << 5;
        if self.kind == NpcKind::Slime {
            h ^= (self.slime as u32) << 9;
        }
        h
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }
}

/// Hash of all NPCs for the G6 world-range key.
pub fn render_key(npcs: &[Npc]) -> u32 {
    let mut h = npcs.len() as u32;
    for n in npcs {
        h = h.wrapping_mul(31).wrapping_add(n.render_key());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn slime_falls_onto_the_ground_and_hurts_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut slime = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..120 {
            slime.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(slime.on_ground);
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn demon_eye_reaches_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye(&w, 20).unwrap();
        assert_eq!(eye.kind, NpcKind::DemonEye);
        for _ in 0..240 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn zombie_walks_to_the_player_and_burns_at_dawn() {
        let mut w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut zombie = Npc::spawn_zombie(&w, 30).unwrap();
        assert_eq!(zombie.kind, NpcKind::Zombie);
        let start_x = zombie.x;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.x < start_x, "zombie walked the wrong way");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Daylight burns the zombie away.
        w.day_time = true;
        let hp = zombie.hp;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.hp < hp, "hp {} stayed at {hp}", zombie.hp);
    }

    #[test]
    fn king_slime_hops_toward_the_player_and_teleports() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut king = Npc::spawn_king_slime(&w, 34).unwrap();
        assert_eq!(king.hp, 2000);
        let start_x = king.x;
        for _ in 0..1200 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(king.x < start_x, "boss did not approach");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Outrun the boss: it should teleport back next to the player.
        player.x = 4.0 * TILE_PX;
        player.y = 30.0 * TILE_PX - crate::player::HEIGHT;
        for _ in 0..120 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        let dx = ((king.x + 49.0) - (player.x + crate::player::WIDTH * 0.5)).abs();
        assert!(dx < 20.0 * TILE_PX, "boss still {dx} px away");
    }

    #[test]
    fn attacking_kills_a_slime() {
        let w = ground();
        let mut s = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..3 {
            assert!(s.hurt(10, 0.0, 5.5, false));
            s.hurt_cd = 0.0;
        }
        // Vanilla keeps `life` as the raw remainder and dies on
        // `life <= 0` (`NPC.checkDead`); the blue slime now carries the
        // vanilla 25 HP
        // (NPC.cs:8572), so three 10-damage hits leave -5.
        assert!(s.hp <= 0);
    }
    #[test]
    fn npc_update_runs_the_buff_tick_before_the_ai() {
        // NPC.cs:90373-90377 — the buff block runs at the top of UpdateNPC;
        // an OnFire slime dies from DoT alone (25 hp, 1 damage per 15 ticks —
        // slimes are immune to Poisoned 20, so the fire row drives the tick).
        let w = ground();
        let mut player = Player::spawn(&w);
        let mut s = Npc::spawn_slime(&w, 34).unwrap();
        s.buffs.add_buff(24, 2000); // OnFire — 8 regenLoss/tick
        for _ in 0..380 {
            s.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(s.hp, 0, "hp {}", s.hp);
    }

    #[test]
    fn spawned_npcs_carry_their_debuff_immunity_table() {
        // SetDefaults' immunity application (NPC.cs:17818-17837) rides
        // Npc::new → NpcBuffs::for_kind: slimes (type 1) are immune to
        // Poisoned 20 (NPCID.cs:6079-6084) with the derived 30/375; zombies
        // (null entry) have no immunity.
        let s = Npc::spawn_slime(&ground(), 20).unwrap();
        assert!(s.buffs.buff_immune[20]);
        assert!(
            s.buffs.buff_immune[30] && s.buffs.buff_immune[375],
            "derived from 20"
        );
        let z = Npc::spawn_zombie(&ground(), 20).unwrap();
        assert!(!z.buffs.buff_immune[20]);
    }

    #[test]
    fn slime_variants_match_the_setdefaults_lines() {
        // NPC.cs:8567-8580 (blue) and the SetDefaultsFromNetId cases
        // (-3 green, -4 pinky, -5 baby, -6 black, -7 purple, -8 red,
        // -9 yellow).
        let cases: [(SlimeVariant, i32, i32, i32, f32, [u8; 4]); 8] = [
            (SlimeVariant::Blue, 25, 7, 2, 1.0, [0, 80, 255, 100]),
            (SlimeVariant::Green, 14, 6, 0, 0.9, [0, 220, 40, 100]),
            (SlimeVariant::Pinky, 150, 5, 5, 0.6, [250, 30, 90, 90]),
            (SlimeVariant::Baby, 30, 13, 4, 0.9, [0, 0, 0, 50]),
            (SlimeVariant::Black, 45, 15, 4, 1.05, [0, 0, 0, 50]),
            (SlimeVariant::Purple, 40, 12, 6, 1.2, [200, 0, 255, 150]),
            (SlimeVariant::Red, 35, 12, 4, 1.025, [255, 30, 0, 100]),
            (SlimeVariant::Yellow, 45, 15, 7, 1.2, [255, 255, 0, 100]),
        ];
        for (v, life, damage, defense, scale, color) in cases {
            let st = v.stats();
            assert_eq!(
                (st.life, st.damage, st.defense, st.scale, st.color),
                (life, damage, defense, scale, color),
                "variant {:?}",
                v
            );
        }
        // ItemDropDatabase.cs:1074-1082: the standard slimes drop 1-2 gel,
        // the red/purple/black family 2-5, green/baby/pinky none.
        assert_eq!(SlimeVariant::Blue.stats().gel, (1, 2));
        assert_eq!(SlimeVariant::Red.stats().gel, (2, 5));
        assert_eq!(SlimeVariant::Green.stats().gel, (0, 0));
        assert_eq!(SlimeVariant::Pinky.stats().gel, (0, 0));
    }

    #[test]
    fn slime_hitboxes_scale_with_the_variant() {
        // SetDefaults_ForNetId multiplies the base 24x18 by the variant scale.
        let mut p = Npc::spawn_slime(&ground(), 20).unwrap();
        p.slime = SlimeVariant::Purple;
        let (w, h) = p.size();
        assert!((w - 24.0 * 1.2).abs() < 1e-4 && (h - 18.0 * 1.2).abs() < 1e-4);
        p.slime = SlimeVariant::Pinky;
        let (w, h) = p.size();
        assert!((w - 24.0 * 0.6).abs() < 1e-4 && (h - 18.0 * 0.6).abs() < 1e-4);
    }

    #[test]
    fn wandering_eye_rides_the_eye_machine_with_four_frames() {
        // aiStyle 2 (`NPC.cs` case 133) with Main.npcFrameCount[133] = 4
        // (Main.cs:67082); stats: 300 life, 40 damage, 30x32.
        let mut e = bare(NpcKind::WanderingEye);
        assert_eq!(e.max_hp, 300);
        for _ in 0..9 {
            e.find_frame();
        }
        assert_eq!(e.frame, 1, "8 steps advance one of the 4 frames");
        // Each further batch of 8 steps advances one frame: 2, 3, then the
        // wrap 4 -> 0 (the strip never shows a fifth frame).
        for expect in [2u32, 3, 0] {
            for _ in 0..8 {
                e.find_frame();
            }
            assert_eq!(e.frame, expect);
        }
    }

    fn bare(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 1, 0.0)
    }

    #[test]
    fn slime_frames_cycle_every_eight_steps_and_speed_up_while_moving() {
        // NPC.cs:70801-70826: one counter tick per step, +1 when num2 > 0.
        let mut s = bare(NpcKind::Slime);
        for _ in 0..7 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
        s.find_frame();
        assert_eq!(s.frame, 1, "8 idle steps advance one frame");
        // Moving (num2 == 1) doubles the tick rate: 4 steps to the next frame.
        s.vx = 1.0;
        for _ in 0..3 {
            s.find_frame();
        }
        assert_eq!(s.frame, 1);
        s.find_frame();
        assert_eq!(s.frame, 0, "moving ticks twice per step: 4 steps advance");
        // The strip wraps over Main.npcFrameCount[1] = 2 frames.
        for _ in 0..16 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
    }

    #[test]
    fn zombie_walk_band_is_zero_one_two_one_by_accumulated_speed() {
        // NPC.cs:76324-76389: the counter accumulates |velocity.X| and the
        // frame follows the bands <8 -> 0, <16 -> 1, <24 -> 2, <32 -> 1; at 32
        // the counter restarts WITHOUT touching the frame (it stays 1 until
        // the next tick re-enters the first band).
        let mut z = bare(NpcKind::Zombie);
        z.vx = 0.9; // aligned with facing 1
        z.frame_counter = 0.0;
        z.frame = 0;
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "9 ticks = 8.1 accumulated: second band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 2, "18 ticks = 16.2: third band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "27 ticks = 24.3: fourth band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "36 ticks = 32.4: restart keeps the frame");
        z.find_frame();
        assert_eq!(z.frame, 0, "the next tick re-enters the first band");
    }

    #[test]
    fn zombie_holds_the_lurch_frame_when_airborne_or_backpedalling() {
        // NPC.cs:76334-76342.
        let mut z = bare(NpcKind::Zombie);
        z.vy = 0.1;
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vy = 0.0;
        z.facing = 1;
        z.vx = -0.5; // moving against the facing
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vx = 0.0;
        z.find_frame();
        assert_eq!(z.frame, 0, "standing still is the idle frame");
    }

    #[test]
    fn king_slime_snaps_to_four_five_in_the_air_and_cycles_zero_to_three_on_ground() {
        // NPC.cs:73380-73414: airborne snaps to frame 4, holds it 4 steps,
        // then frame 5; landing returns to 4 and the grounded cycle advances
        // every 8 steps with the wrap `>= 4 -> 0`, so frames 4/5 never show
        // while grounded.
        let mut k = bare(NpcKind::KingSlime);
        k.vy = -0.1; // leaving the ground
        k.find_frame();
        assert_eq!(k.frame, 4, "airborne snaps straight to frame 4");
        for _ in 0..3 {
            k.find_frame();
        }
        assert_eq!(k.frame, 5, "after 4 airborne steps the squash frame shows");
        k.vy = 0.0; // landed
        k.find_frame();
        assert_eq!(k.frame, 4, "landing returns to frame 4, counter reset");
        // The landing squash (4) is itself displayed while grounded until the
        // first advance; frames 0..=3 then cycle. Only 5 is airborne-only.
        for _ in 0..64 {
            k.find_frame();
            assert!(
                k.frame <= 4,
                "grounded frames never show 5, got {}",
                k.frame
            );
        }
        // The grounded cycle must actually move through 0..=3.
        let mut k2 = bare(NpcKind::KingSlime);
        k2.vy = 0.0;
        let mut seen = [false; 4];
        for _ in 0..32 {
            k2.find_frame();
            seen[k2.frame as usize] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "one 32-step cycle covers frames 0..=3"
        );
    }
}
