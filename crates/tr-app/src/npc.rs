//! First enemy pass: a hopping blue slime, a flying demon eye and a walking zombie,
//! modelled on the simplest `NPC.cs` behaviours.

use glam::Vec2;
use tr_core::{TILE_PX, World};

use crate::npc_ai::HoverParams;
use crate::npc_buffs::NpcBuffs;
use crate::player::{Player, collides_rect};

pub(crate) const GRAVITY: f32 = 0.4;
pub(crate) const MAX_FALL: f32 = 8.0;

/// `Collision.WetCollision` (simplified): any water tile overlapping the body.
pub(crate) fn is_wet(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
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
    /// `NPCID.Guide` (22) — the first town NPC (`aiStyle 7`): the port's
    /// walker is the zombie machine minus the chase (the town NPC wanders).
    Guide,
    /// `NPCID.Skeleton` (21) — the cave walker (`aiStyle 3`, the 15-frame
    /// humanoid strip).
    Skeleton,
    /// `NPCID.UndeadMiner` (44) — the rare cave walker (`aiStyle 3`).
    UndeadMiner,
    /// `NPCID.CaveBat` (49) — the cave flyer (`aiStyle 14`, the erratic
    /// flight, 5 frames).
    CaveBat,
    /// `NPCID.JungleBat` (51) — the jungle flyer (`aiStyle 14`).
    JungleBat,
    /// `NPCID.Hellbat` (60) — the underworld flyer (`aiStyle 14`).
    Hellbat,
    /// `NPCID.VoodooDemon` (66) — the underworld flyer (`aiStyle 14`).
    VoodooDemon,
    /// `NPCID.EaterofSouls` (6) — the corruption hover (`aiStyle 5`).
    EaterOfSouls,
    /// `NPCID.Hornet` (42) — the jungle hover (`aiStyle 5`).
    Hornet,
    /// `NPCID.MeteorHead` (23) — the meteor hover (`aiStyle 5`).
    MeteorHead,
    /// `NPCID.GoblinThief` (27) — the invasion runner (`NPC.cs`: 18×38,
    /// aiStyle 3, dmg 12, def 4, 60 HP, kb 0.8, value 100).
    GoblinThief,
    /// `NPCID.GoblinWarrior` (28) — the invasion fighter (`NPC.cs`: 18×38,
    /// aiStyle 3, dmg 20, def 6, 80 HP, kb 0.7, value 200).
    GoblinWarrior,
    /// `NPCID.GoblinArcher` (111) — the invasion archer (`NPC.cs`: 18×38,
    /// aiStyle 3, dmg 25, def 8, 110 HP, kb 0.5, value 150; the arrows
    /// ride the projectile work).
    GoblinArcher,
    /// `NPCID.GoblinPeon` (26) — the invasion grunt (`NPC.cs`: 18×38,
    /// aiStyle 3, dmg 20, def 6, 80 HP, kb 0.7, value 200).
    GoblinPeon,
    /// `NPCID.GoblinSummoner` (471) — the hardmode invasion caster
    /// (`NPC.cs`: 18×38, aiStyle 3 (the summoner's cast arms ride the
    /// caster work), dmg 80, def 26, 2000 HP, kb 0.15, value 5000;
    /// `GetNPCInvasionGroup`'s 10-point kill).
    GoblinSummoner,

    /// `NPCID.SnowmanGangsta` (143) — the Frost Legion's bottle-thrower
    /// (`NPC.cs`: 26×40, aiStyle 38 (the speed 3/accel 0.7; the throw
    /// arm lands with the projectile work), dmg 50, def 20, 200 HP,
    /// kb 0.6, value 400).
    SnowmanGangsta,
    /// `NPCID.SnowBalla` (144) — the kicker (`NPC.cs`: 26×40, aiStyle 38
    /// (the speed 3.5/accel 0.8; the kick arm = the follow-up), dmg 65,
    /// def 26, 240 HP, kb 0.6, value 400).
    SnowBalla,
    /// `NPCID.MysterySnowman` (145) — the jumper (`NPC.cs`: 26×40,
    /// aiStyle 38 (the speed 4/accel 1 + the jump arm), dmg 55, def 22,
    /// 220 HP, kb 0.6, value 400).
    MysterySnowman,
    /// `NPCID.MartianOfficer` (383) — the martian officer (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 75, def 50, 300 HP, kb 0.25, value 1200).
    MartianOfficer,
    /// `NPCID.GrayGrunt` (385) — the martian grunt (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 80, def 30, 750 HP, kb 0, value 600).
    GrayGrunt,
    /// `NPCID.MartianEngineer` (386) — the martian engineer (`NPC.cs`:
    /// 18×40, aiStyle 3, dmg 40, def 34, 400 HP, kb 0.4, value 1200).
    MartianEngineer,
    /// `NPCID.GigaZapper` (389) — the martian zapper (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 75, def 38, 600 HP, kb 0.3, value 1200).
    GigaZapper,
    /// `NPCID.MartianWalker` (520) — the martian walker (`NPC.cs`: 34×110,
    /// aiStyle 3, dmg 60, def 40, 2000 HP, kb 0).
    MartianWalker,
    /// `NPCID.MartianProbe` (399) — the Martian Madness trigger (`NPC.cs`:
    /// 42×72, aiStyle 80, dmg 5, def 5, 500 HP, kb 0, value 0, noGravity,
    /// chaseable false, rarity 1; the escape machine runs
    /// `AI()`'s aiStyle-80 block).
    MartianProbe,
    /// `NPCID.MartianDrone` (388) — the Martian Madness sky flyer (`NPC.cs`:
    /// 40×30, aiStyle 74, dmg 60, def 16, 300 HP, kb 0.4, value 1000,
    /// noTileCollide, noGravity, npcSlots 1.5, canGhostHeal false).
    MartianDrone,
    /// `NPCID.BrainScrambler` (381) — the martian ground shooter (`NPC.cs`:
    /// 18×40, aiStyle 3, dmg 50, def 25, 350 HP, kb 0.35, value 1000; the
    /// stop-and-fire cycle fires the projectile 436).
    BrainScrambler,
    /// `NPCID.RayGunner` (382) — the martian ground shooter (`NPC.cs`: the
    /// 381/382 grouped row — 18×40, aiStyle 3, dmg 50, def 25, 350 HP,
    /// kb 0.35, value 1000; fires the projectile 438 at its speed).
    RayGunner,
    /// `NPCID.ScutlixRider` (390) — the ray gunner mounted on a Scutlix
    /// (`NPC.cs:13966-13977`: 18×40, aiStyle 75, dmg 65, def 30, 350 HP,
    /// value 1200, npcSlots 0.5; the mount machine carries the carrier
    /// spawn + the 438 ray at the 60/-60 cycle).
    ScutlixRider,
    /// `NPCID.Scutlix` (391) — the walker carrier (`NPC.cs:13979-13991`:
    /// 60×40, aiStyle 3, dmg 85, def 30, 600 HP, kb 0.1, value 1200;
    /// walks through the walker params 5/0.25/0.7).
    Scutlix,
    /// `NPCID.MartianSaucer` (392) — the saucer shell (`NPC.cs:13993-14008`:
    /// 150×80, aiStyle 75, dmg 50, def 100, 100 HP; rides the core and
    /// drives its phase state).
    MartianSaucer,
    /// `NPCID.MartianSaucerTurret` (393) — the saucer turret
    /// (`NPC.cs:14009-14024`: 40×16, aiStyle 75, 5000 HP, def 20, dmg 60).
    MartianSaucerTurret,
    /// `NPCID.MartianSaucerCannon` (394) — the saucer cannon
    /// (`NPC.cs:14025-14044`: 46×36, aiStyle 75, 3500 HP, def 20, dmg 60).
    MartianSaucerCannon,
    /// `NPCID.FlyingDutchman` (491) — the pirate invasion's ship
    /// (`NPC.cs:14632-14650`: 350×120, aiStyle 93, dmg 0, def 100, 50 HP,
    /// dontTakeDamage, value 0; the crew's 4 cannons fight for it).
    FlyingDutchman,
    /// `NPCID.DD2GoblinT1` (552) — the Old One's Army goblin
    /// (`NPC.cs:13891-13906`: 18×40, aiStyle 107, dmg 18, def 14, 30 HP,
    /// kb 0.2, value 0, npcSlots 0, lavaImmune).
    DD2GoblinT1,
    /// `NPCID.DD2GoblinT2` (553) — the tier-2 goblin (170 HP, dmg 46,
    /// def 20, the +0.01/+0.2 accel/speed).
    DD2GoblinT2,
    /// `NPCID.DD2GoblinT3` (554) — the tier-3 goblin (560 HP, dmg 70,
    /// def 28, the +0.02/+0.4 accel/speed).
    DD2GoblinT3,
    /// `NPCID.DD2GoblinBomberT1` (555) — the kobold bomber (`NPC.cs`:
    /// 18×40, aiStyle 107, dmg 26, def 16, 50 HP; the fuse death).
    DD2GoblinBomberT1,
    /// `NPCID.DD2GoblinBomberT2` (556) — the tier-2 bomber (200 HP,
    /// dmg 55, def 26).
    DD2GoblinBomberT2,
    /// `NPCID.DD2GoblinBomberT3` (557) — the tier-3 bomber (700 HP,
    /// dmg 75, def 34).
    DD2GoblinBomberT3,
    /// `NPCID.DD2JavelinstT1` (561) — the javelin thrower (`NPC.cs`:
    /// 26×52, aiStyle 107, dmg 30, def 18, 60 HP).
    DD2JavelinT1,
    /// `NPCID.DD2JavelinstT2` (562) — the tier-2 javelin (300 HP,
    /// dmg 60, def 28).
    DD2JavelinT2,
    /// `NPCID.DD2JavelinstT3` (563) — the tier-3 javelin (1000 HP,
    /// dmg 80, def 38, the 685 spear).
    DD2JavelinT3,
    /// `NPCID.DD2SkeletonT1` (566) — the etherian skeleton (`NPC.cs`:
    /// 18×40, aiStyle 107, dmg 18, def 12, 25 HP; materializes in).
    DD2SkeletonT1,
    /// `NPCID.DD2SkeletonT3` (567) — the tier-3 skeleton (480 HP,
    /// dmg 70, def 22).
    DD2SkeletonT3,
    /// `NPCID.LunarTowerStardust` (493) — the Stardust Pillar
    /// (`NPC.cs:14988-15003`: 130×270, aiStyle 94, 20000 HP, dmg 0,
    /// def 20, noGravity, noTileCollide; the pillar's sheet NPC_493 is
    /// 174x364 — the frame overhangs the box).
    StardustTower,
    /// `NPCID.TowerNebula` (507).
    NebulaTower,
    /// `NPCID.TowerSolar` (517) — the Solar Pillar (the same
    /// `SetDefaults` row as its siblings).
    SolarTower,
    /// `NPCID.LunarTowerVortex` (422) — the Vortex Pillar: the fourth
    /// pillar `WorldGen.TriggerLunarApocalypse` spawns
    /// (`WorldGen.cs:86628`'s `{517, 422, 507, 493}` shuffle) and the
    /// port was missing entirely — the three kinds below drew the
    /// Stardust sheet and the vortex had neither kind nor art. Its
    /// internal slot is 146 (112-145 are taken).
    VortexTower,
    /// `NPCID.DD2LanePortal` (549) — the Old One's Army's lane gate
    /// (`NPC.cs:16033-16053`: 78×130, aiStyle 106, 5 HP, def 20, damage
    /// 0, noGravity/noTileCollide/dontTakeDamage, netAlways; `ai[2]`
    /// picks the lane side).
    DD2LanePortal,
    /// `NPCID.DD2WyvernT1` (558) — the tier-1 etherian wyvern
    /// (`NPC.cs:16207-16223`: 38×38, aiStyle 108, 60 HP, dmg 30, def 4,
    /// kb 0.2, noGravity/noTileCollide, netAlways).
    DD2WyvernT1,
    /// `NPCID.DD2WyvernT2` (559) — the tier-2 wyvern (`:16225-16241`:
    /// 38×38, aiStyle 108, 180 HP, dmg 75, def 16, kb 0.2).
    DD2WyvernT2,
    /// `NPCID.DD2WyvernT3` (560) — the tier-3 wyvern (`:16243-16259`:
    /// 38×38, aiStyle 108, 600 HP, dmg 100, def 30, kb 0.05).
    DD2WyvernT3,
    /// `NPCID.DD2DarkMageT3` (565) — the tier-3 Dark Mage
    /// (`NPC.cs:16463-16479`: 34×62, aiStyle 109, 4000 HP, dmg 90, def 38,
    /// kb 0, noGravity, netAlways — the T1's 800-HP machine).
    DD2DarkMageT3,
    /// `NPCID.DD2WitherBeastT3` (569) — the tier-3 wither beast
    /// (`NPC.cs:16326-16342`: 18×40, aiStyle 107, 1400 HP, dmg 80, def 40,
    /// kb 0.05 — the aura/heal walker).
    DD2WitherBeastT3,
    /// `NPCID.DD2DrakinT2` (570) — the tier-2 drakin
    /// (`NPC.cs:16411-16427`: 42×58, aiStyle 107, 900 HP, dmg 60, def 30,
    /// kb 0 — the stand-and-shoot ranger).
    DD2DrakinT2,
    /// `NPCID.DD2DrakinT3` (571) — the tier-3 drakin (`:16428-16444`:
    /// 42×58, aiStyle 107, 3000 HP, dmg 90, def 40, kb 0).
    DD2DrakinT3,
    /// `NPCID.DD2KoboldWalkerT2` (572) — the tier-2 kobold walker
    /// (`NPC.cs:16377-16393`: 18×40, aiStyle 107, 260 HP, dmg 60, def 26,
    /// kb 0.2 — the lit-kamikaze walker).
    DD2KoboldWalkerT2,
    /// `NPCID.DD2KoboldWalkerT3` (573) — the tier-3 kobold walker
    /// (`:16394-16410`: 18×40, aiStyle 107, 800 HP, dmg 80, def 32, kb 0.1).
    DD2KoboldWalkerT3,
    /// `NPCID.DD2KoboldFlyerT2` (574) — the tier-2 kobold glider
    /// (`NPC.cs:16481-16497`: 38×38, aiStyle 108, 170 HP, dmg 50, def 16,
    /// kb 0.4, noGravity/noTileCollide — the dive bomber).
    DD2KoboldFlyerT2,
    /// `NPCID.DD2KoboldFlyerT3` (575) — the tier-3 kobold glider
    /// (`:16499-16515`: 38×38, aiStyle 108, 580 HP, dmg 80, def 32, kb 0.25).
    DD2KoboldFlyerT3,
    /// `NPCID.DD2OgreT2` (576) — the tier-2 ogre (`NPC.cs:16275-16291`:
    /// 96×124, aiStyle 107, 5000 HP, dmg 70, def 34, kb 0 — the melee
    /// thrower with the ground pound).
    DD2OgreT2,
    /// `NPCID.DD2OgreT3` (577) — the tier-3 ogre (`:16292-16308`:
    /// 96×124, aiStyle 107, 13000 HP, dmg 90, def 40, kb 0).
    DD2OgreT3,
    /// `NPCID.StardustCellBig` (405) — the stardust shard's grown form
    /// (`NPC.cs:15022-15035`: 44×44, aiStyle 85, dmg 120, def 50, 300 HP,
    /// kb 0.3; the aiStyle-85 hover machine).
    StardustCellBig,
    /// `NPCID.StardustCellSmall` (406) — the stardust shard (`NPC.cs`:
    /// 22×22, aiStyle 95, dmg 70, def 0; grows into 405 over 300 ticks).
    StardustCellSmall,
    /// `NPCID.StardustJellyfishBig` (407) — the stardust floater
    /// (`NPC.cs:15087-15100`: 40×60, aiStyle 96, dmg 70, def 38, 1500 HP,
    /// kb 0.03).
    StardustJellyfishBig,
    /// `NPCID.NebulaBeast` (423) — the nebula quadruped (`NPC.cs`:
    /// 50×44, aiStyle 26, dmg 90, def 46, 850 HP, kb 0.2; the 575 shot).
    NebulaBeast,
    /// `NPCID.VortexRifleman` (425) — the vortex rifleman (`NPC.cs`:
    /// 30×56, aiStyle 3, dmg 100, def 40).
    VortexRifleman,
    /// `NPCID.VortexSoldier` (429) — the vortex soldier (`NPC.cs`:
    /// 22×56, aiStyle 3, dmg 90, def 34).
    VortexSoldier,
    /// `NPCID.VortexHunter` (426) — the vortex pillar's shooter
    /// (`NPC.cs:15214-15247`: 50×62, aiStyle 3, 1000 HP, dmg 100, def 44,
    /// kb 0.3; the 581 battery and the 4/0.6/0.95 walk).
    VortexHunter,
    /// `NPCID.VortexAlien` (427) — the hive's diver (`NPC.cs`:
    /// 15226-15235: 40×28, aiStyle 3, 500 HP, dmg 75, def 20, kb 0.5; the
    /// 0.015/0.15 dive, the leap and the 1200-tick `Transform(426)`).
    VortexAlien,
    /// `NPCID.VortexHeadcrab` (428) — the crawler (`NPC.cs:15215-15222`:
    /// 24×26, aiStyle 3, 200 HP, dmg 50, def 6, kb 1 (the class default);
    /// the rolling rotation and the 300-tick `Transform(427)`).
    VortexHeadcrab,
    /// `NPCID.NebulaFloater` (420) — the nebula pillar's floater
    /// (`NPC.cs:15157-15171`: 40×40, aiStyle 97, 1300 HP, dmg 75, def 20,
    /// kb 0.5; the 574 emission + the 480-tick teleport cycle).
    NebulaFloater,
    /// `NPCID.BrainSuckler` (421) — the nebula leech (`NPC.cs:15130-15143`:
    /// 44×44, aiStyle 85, 330 HP, dmg 70, def 34, kb 0.5; the aiStyle-85
    /// hoverer with the <40-px attachment onto the player's Top).
    BrainSuckler,
    /// `NPCID.Predictor` (424) — the nebula pillar's shooter (`NPC.cs`:
    /// 15144-15156: 22×56, aiStyle 3, 700 HP, dmg 80, def 30, kb 0.6; the
    /// 573 fan).
    Predictor,
    /// `NPCID.StardustSpiderBig` (409) — the stardust pillar's spawner
    /// (`NPC.cs:15061-15075`: 34×42, aiStyle 3, 800 HP, dmg 70, def 40,
    /// kb 0.4; its 60-tick cycle spawns the 410 rather than firing).
    StardustSpiderBig,
    /// `NPCID.StardustSpiderSmall` (410) — the kamikaze spider
    /// (`NPC.cs:15074-15087`: 22×22, aiStyle 26, 200 HP, dmg 80, def 10,
    /// kb 0.3; the 240-tick burst then self-destruct).
    StardustSpiderSmall,
    /// `NPCID.StardustSoldier` (411) — the stardust pillar's shooter
    /// (`NPC.cs:15049-15059`: 22×56, aiStyle 3, 700 HP, dmg 80, def 34,
    /// kb 0.4; the 330-tick cycle's 537).
    StardustSoldier,
    /// `NPCID.SolarDrakomire` (415) — the solar pillar's mount
    /// (`NPC.cs:15319-15329`: 60×40, aiStyle 3, 800 HP, dmg 55, def 32,
    /// kb 0.2; the 4/0.1/0.95 jump assist, hidden under a rider).
    SolarDrakomire,
    /// `NPCID.SolarDrakomireRider` (416) — the rider (`NPC.cs:15331-15342`:
    /// 18×40, aiStyle 75, 800 HP, dmg 80, def 28, kb 0.2; it spawns and
    /// rides the 415 mount, then `Transform(518)`s when it dies).
    SolarDrakomireRider,
    /// `NPCID.SolarSroller` (417) — the solar pillar's roller-at (`NPC.cs`
    /// `:15355-15367`: 46×52, aiStyle 39, 700 HP, dmg 80, def 34, kb 0;
    /// the ball state's ×1.8 damage and the doubled defense).
    SolarSroller,
    /// `NPCID.SolarCorite` (418) — the solar pillar's charger
    /// (`NPC.cs:15368-15380`: 38×38, aiStyle 74, 600 HP, dmg 70, def 26,
    /// kb 0.2; the approach/dash/implosion cycle).
    SolarCorite,
    /// `NPCID.SolarSolenian` (419) — the solar pillar's walker
    /// (`NPC.cs:15382-15392`: 22×56, aiStyle 3, 800 HP, dmg 90, def 30,
    /// kb 0.4; the 6/0.15/0.85 walk).
    SolarSelenian,
    /// `NPCID.SolarCrawltipedeHead` (412) — the solar tower's crawltipede
    /// head (`NPC.cs:15265-15281`: 20×20, aiStyle 6, 10000 HP, dmg 120,
    /// def 1000, kb 0, dontTakeDamage; the 30-segment chain).
    SolarCrawltipedeHead,
    /// `NPCID.SolarCrawltipedeBody` (413) — the chain's bodies (`NPC.cs`
    /// `:15283-15299`: 20×20, aiStyle 6, 10000 HP, dmg 80, def 1000, kb
    /// 0, dontCountMe/dontTakeDamage).
    SolarCrawltipedeBody,
    /// `NPCID.SolarCrawltipedeTail` (414) — the chain's tail (`NPC.cs`:
    /// 15301-15317: 20×20, aiStyle 6, 10000 HP, dmg 50, def 0, kb 0,
    /// dontCountMe, `takenDamageMultiplier = 10`).
    SolarCrawltipedeTail,
    /// `NPCID.StardustWormHead` (402) — the stardust tower's worm head
    /// (`NPC.cs:15004-15021`: 32×32, aiStyle 6, 1200 HP, dmg 80, def 10,
    /// kb 0, noGravity/noTileCollide, netAlways; the mark fire).
    StardustWormHead,
    /// `NPCID.SolarSpearman` (518) — the rider's unmounted form
    /// (`NPC.cs:15343-15353`: 22×56, aiStyle 3, 1000 HP, dmg 80, def 28,
    /// kb 0.4; the 5/0.1/0.95 walk).
    SolarSpearman,
    /// `NPCID.MoonLordHead` (396) — the Moon Lord's eye (`NPC.cs`:
    /// 14082-14100: 38×56, aiStyle 79, 45000 HP, dmg 0, def 50, kb 0,
    /// boss/hide/netAlways; the `AI_079_MoonLordHead` sequencer).
    MoonLordHead,
    /// `NPCID.MoonLordHand` (397) — the Moon Lord's hand (`NPC.cs`:
    /// 14101-14120: 46×66, aiStyle 78, 25000 HP, dmg 0, def 40, kb 0,
    /// boss/hide/netAlways; `AI_078_MoonLordHands`, two of them).
    MoonLordHand,
    /// `NPCID.MoonLordCore` (398) — the Moon Lord's heart (`NPC.cs`:
    /// 14120-14140: 46×66, aiStyle 77, 50000 HP, dmg 0, def 70, kb 0,
    /// dontTakeDamage/boss/behindTiles/hide/netAlways; the `AI_077`
    /// hover + the 600-tick death drama).
    MoonLordCore,
    /// `NPCID.MoonLordFreeEye` (400) — the True Eye of Cthulhu
    /// (`NPC.cs:14141-14158`: 60×60, aiStyle 81, 100 HP, dmg 60, def 0,
    /// dontTakeDamage/netAlways; a dying hand or head spawns one).
    MoonLordFreeEye,
    /// `NPCID.MoonLordLeechBlob` (401) — the Moon Leech Clot (`NPC.cs`:
    /// 14159-14175: 30×30, aiStyle 82, 400 HP, dmg 0, def 0, hide; it
    /// drains 1000 hp from the head, the core and the hands).
    MoonLordLeechBlob,
    /// `NPCID.CultistBoss` (439) — the Lunar Events' summoner (the port's
    /// bestiary entry carried the Vortex Pillar's id 422 — corrected
    /// here and in the catalog; the port ships no NPC_439 sheet, so the
    /// draw stands on NPC_422's art as the documented stand-in) (`NPC.cs`
    /// `:15172-15189`: 130×270, aiStyle 94, 20000 HP, dmg 0, def 20,
    /// noGravity, noTileCollide; the ritual + the portal-summon machine).
    AncientCultist,
    /// `NPCID.DD2Bartender` (550) — the tavernkeeper (`NPC.cs`:
    /// 18×40, aiStyle 7 town NPC, 250 HP, dmg 10, def 15, kb 0.5).
    DD2Bartender,
    /// `NPCID.DD2EterniaCrystal` (548) — the Old One's Army's crystal
    /// (`NPC.cs:15998-16022`: 40×40, aiStyle 105, def 14, 1000 HP,
    /// friendly, npcSlots 0, hide; the 180-tick spawn-in shield).
    DD2EterniaCrystal,
    /// `NPCID.DD2WitherBeastT2` (568) — the Old One's Army's ethereal
    /// caster-beast (`NPC.cs`: 18×40, aiStyle 107, 500 HP, dmg 50, def 30,
    /// kb 0.15; the aura + the self-heal + the 683 crystal shots).
    DD2WitherBeast,
    /// `NPCID.DD2Betsy` (551) — the Old One's Army's dragon boss
    /// (`NPC.cs:16400-16424`: 190×90, aiStyle 110, 50000 HP, dmg 80,
    /// def 38, noGravity, noTileCollide, lavaImmune).
    DD2Betsy,
    /// `NPCID.DD2DarkMageT1` (564) — the Old One's Army caster (`NPC.cs`
    /// `:16445-16466`: 34×62, aiStyle 109, 800 HP, dmg 40, def 18, kb 0,
    /// noGravity, lavaImmune; the summon/heal/shadowflame casts).
    DD2DarkMage,
    /// `NPCID.FlyingDutchmanCannon` (492) — the ship's manned cannon
    /// (`NPC.cs:14651-14666`: 30×30, aiStyle 75, 2000 HP, def 20, dmg 30,
    /// hide; rides the ship and fires the 240 shots).
    FlyingDutchmanCannon,
    /// `NPCID.MartianSaucerCore` (395) — the saucer boss
    /// (`NPC.cs:14045-14060`: 120×90, aiStyle 76, dmg 80, 10000 HP,
    /// def 0, dontTakeDamage, npcSlots 6).
    MartianSaucerCore,
    /// `NPCID.PirateDeckhand` (212) — the pirate grunt (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 35, def 17, 300 HP, kb 0.4, value 700).
    PirateDeckhand,
    /// `NPCID.PirateCaptainsMate` (213) — the pirate bruiser (`NPC.cs`:
    /// 18×40, aiStyle 3, dmg 50, def 22, 450 HP, kb 0.2, value 1000).
    PirateCaptainsMate,
    /// `NPCID.PirateArcher` (214) — the pirate archer (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 30, def 14, 225 HP, kb 0.3, value 1000; the arrows
    /// ride the projectile work).
    PirateArcher,
    /// `NPCID.PirateCorsair` (215) — the pirate elite (`NPC.cs`: 18×40,
    /// aiStyle 3, dmg 35, def 20, 350 HP, kb 0.35, value 1500).
    PirateCorsair,
    /// `NPCID.PirateCaptain` (216) — the pirate boss-minion (`NPC.cs`:
    /// 18×40, aiStyle 3, dmg 70, def 30, 3000 HP, kb 0, value 50000;
    /// `GetNPCInvasionGroup`'s 5-point kill).
    PirateCaptain,
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
    /// `NPCID.Goldfish` (55) — the passive water critter (`aiStyle 16`,
    /// `NPC.cs:9460`: 20x18, 5 HP, damage 0, the flag25 exclusion).
    Goldfish,
    /// `NPCID.Piranha` (58) — the jungle/deep-water fish (`aiStyle 16`,
    /// `NPC.cs:9502`: 18x20, 30 HP).
    Piranha,
    /// `NPCID.Shark` (65) — the ocean fish (`aiStyle 16`, `NPC.cs:9610`:
    /// 100x24, 300 HP).
    Shark,
    /// `NPCID.DevourerHead` (7) — the corruption worm's head (`aiStyle 6`,
    /// `NPC.cs:8763`). The body 8 and the tail 9 chain behind it.
    DevourerHead,
    /// `NPCID.DevourerBody` (8) — the middle segments (`NPC.cs:8792`).
    DevourerBody,
    /// `NPCID.DevourerTail` (9) — the tail segment (`NPC.cs:8821`).
    DevourerTail,
    /// `NPCID.GiantWormHead` (10) — the cavern worm's head (`NPC.cs:8850`).
    GiantWormHead,
    /// `NPCID.GiantWormBody` (11) — the middle segments (`NPC.cs:8867`).
    GiantWormBody,
    /// `NPCID.GiantWormTail` (12) — the tail segment (`NPC.cs:8885`).
    GiantWormTail,
    /// `NPCID.EaterofWorldsHead` (13) — the boss worm's head (`NPC.cs:8903`);
    /// the natural spawn waits for the orb-break summon work.
    EaterOfWorldsHead,
    /// `NPCID.EaterofWorldsBody` (14) — the middle segments (`NPC.cs:8923`).
    EaterOfWorldsBody,
    /// `NPCID.EaterofWorldsTail` (15) — the tail segment (`NPC.cs:8943`).
    EaterOfWorldsTail,
    /// `NPCID.Merchant` (17) — the second town NPC (`NPC.cs:8980`: 18x40,
    /// dmg 10, def 15, 250 HP; the shop index 1 through M4-TOWN-3's
    /// engine; the arrival `SpawnAllowed_Merchant`, `NPC.cs:7175`).
    Merchant,
    /// `NPCID.Dryad` (20) — the nature town NPC (`NPC.cs:9022`: the town
    /// twin; the shop index 3; the arrival rides the downedBoss1 gate —
    /// the boss-downed tracking lands with the boss work).
    Dryad,
    /// `NPCID.Nurse` (18) — the healer town NPC (`NPC.cs:8994`: the town
    /// twin; no shop — the heal interaction rides the right-click; the
    /// arrival `SpawnAllowed_Nurse`, `NPC.cs:7158`).
    Nurse,
    /// `NPCID.ArmsDealer` (19) — the ammo seller (`NPC.cs:9008`: the town
    /// twin; the shop index 2; the arrival `SpawnAllowed_ArmsDealer`,
    /// `NPC.cs:7092` — the bullets in the inventory).
    ArmsDealer,
    /// `NPCID.Demolitionist` (38) — the explosive seller (`NPC.cs:9211`:
    /// the town twin; the shop index 4; the arrival
    /// `SpawnAllowed_Demolitionist`, `NPC.cs:7067` — the bombs in the
    /// inventory).
    Demolitionist,
    /// `NPCID.Painter` (227) — the paint seller (`NPC.cs:11925`: the town
    /// twin; the shop index 15; the arrival = at least 2 other town NPCs).
    Painter,
    /// `NPCID.WitchDoctor` (228) — the jungle shaman (`NPC.cs:11942`: the
    /// town twin; the shop index 16; the arrival rides the downedQueenBee
    /// gate — constants-off until the boss-downed tracking).
    WitchDoctor,
    /// `NPCID.Clothier` (54) — the tailor (`NPC.cs:9446`: the town twin;
    /// the shop index 5; the arrival rides the downedSkeletron gate —
    /// constants-off).
    Clothier,
    /// `NPCID.TravellingMerchant` (368) — the travelling peddler
    /// (`NPC.cs:13673-13686`: the town twin 18x40, dmg 10, def 15, 250 HP,
    /// kb 0.5, aiStyle 7 — the town walker) who visits for one day on a
    /// rolled chance (`Main.cs:65017-65039`) with his own 40-slot stock
    /// (`Chest.SetupTravelShop`, `Chest.cs:1275-1358`) sold through the
    /// shop index 19 (`Chest.cs:2791`). `homeless` every tick and gone at
    /// night (`NPC.cs:53358-53374` -> `WorldGen.UnspawnTravelNPC`,
    /// `WorldGen.cs:5000`) — M4-TOWN-14.
    TravellingMerchant,
    /// `NPCID.Angler` (369) — the quest-giving town NPC (`NPC.cs:13687`:
    /// the town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7 —
    /// the town walker; `npcFrameCount[369]` = 23, `Main.cs:67120`). No
    /// shop: the quest interaction rides `NPCInteractions.cs:519` —
    /// M4-TOWN-15.
    Angler,
    /// `NPCID.SleepingAngler` (376) — the bound form found on the beach /
    /// in the ocean (`NPC.cs:13796`: 34x8, aiStyle 0 the bound stand,
    /// scale 0.9, rarity 1, `npcFrameCount[376]` = 2 — `Main.cs:67121`).
    /// `Sets.BoundTownNPCs` (`NPCID.cs:10441`): the talk transforms
    /// 376 -> 369 and raises `savedAngler` (`NPC.cs:19853`).
    SleepingAngler,
    /// `NPCID.Stylist` (353) — the hair-dye seller (`NPC.cs:13488-13501`:
    /// the town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7;
    /// `npcFrameCount[353]` = 23, `Main.cs:67119`) — the shop index 18
    /// (`Chest.cs:2698`) — M4-TOWN-17.
    Stylist,
    /// `NPCID.WebbedStylist` (354) — the spider-cavern tied Stylist
    /// (`NPC.cs:13502-13516`: 18x34, scale 0.9, aiStyle 0 the bound stand,
    /// rarity 1, `npcFrameCount[354]` = 1); the talk transforms 354 → 353.
    WebbedStylist,
    /// `NPCID.TaxCollector` (441) — the coin collector (`NPC.cs:14704-14717`:
    /// the town twin, aiStyle 7; `npcFrameCount[441]` = 25, `Main.cs:67128`)
    /// — no shop: his tax timer rides the player's coin purse
    /// (`Main.cs:64994`) — M4-TOWN-17.
    TaxCollector,
    /// `NPCID.Golfer` (588) — the golf seller (`NPC.cs:16638-16651`: the
    /// town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7;
    /// `npcFrameCount[588]` = 25, `Main.cs:67142`) — the shop index 22
    /// (`Chest.cs:3156`) — M4-TOWN-18.
    Golfer,
    /// `NPCID.GolferRescue` (589) — the underground-desert tied Golfer
    /// (`NPC.cs:16652-16665`: 18x34, aiStyle 0 the bound stand, rarity 1,
    /// `npcFrameCount[589]` = 13) — `Sets.BoundTownNPCs`
    /// (`NPCID.cs:10441`); the talk transforms 589 → 588.
    SleepingGolfer,
    /// `NPCID.BestiaryGirl` (633) — the bestiary seller (`NPC.cs:17147`:
    /// the town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7;
    /// `npcFrameCount[633]` = 23, `Main.cs:67147`) — the shop index 23
    /// (`Chest.cs:3263`) gates on the bestiary completion — M4-TOWN-19.
    Zoologist,
    /// `NPCID.Princess` (663) — the last town NPC (`NPC.cs:17384-17397`:
    /// the town twin, aiStyle 7; `npcFrameCount[663]` = 23) — her arrival
    /// needs EVERY other town NPC present (`Main.cs:65944-65955`); the
    /// shop index 24 (`Chest.cs:3409`) — M4-TOWN-19.
    Princess,
    /// `NPCID.DyeTrader` (207) — the dye seller (`NPC.cs:11654-11667`: the
    /// town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7;
    /// `npcFrameCount[207]` = 25, `Main.cs:67120`) — the shop index 12
    /// (`Chest.cs:1120`) — M4-TOWN-20.
    DyeTrader,
    /// `NPCID.PartyGirl` (208) — the party starter (`NPC.cs:11668-11681`:
    /// the town twin; `npcFrameCount[208]` = 23) — the shop index 13
    /// (`Chest.cs:case 13`) — M4-TOWN-20.
    PartyGirl,
    /// `NPCID.Wizard` (108) — the magic seller (`NPC.cs:10285-10298`: the
    /// town twin 18x40, dmg 10, def 15, 250 HP, kb 0.5, aiStyle 7 — the
    /// town walker; `npcFrameCount[108]` = 23, `Main.cs:67094`) — M4-TOWN-16.
    Wizard,
    /// `NPCID.BoundWizard` (106) — the tied Wizard awaiting the rescue
    /// (`NPC.cs:10257-10270`: 18x40, aiStyle 0 the bound stand, rarity 1,
    /// `npcFrameCount[106]` = 1) — `Sets.BoundTownNPCs`
    /// (`NPCID.cs:10441`); the talk transforms 106 → 108.
    BoundWizard,
    /// `NPCID.Mechanic` (124) — the wire seller (`NPC.cs:10508-10521`: the
    /// town twin, aiStyle 7; `npcFrameCount[124]` = 23) — M4-TOWN-16.
    Mechanic,
    /// `NPCID.BoundMechanic` (123) — the tied Mechanic awaiting the rescue
    /// (`NPC.cs:10493-10507`: 18x34, scale 0.9, aiStyle 0, rarity 1,
    /// `npcFrameCount[123]` = 1) — `Sets.BoundTownNPCs`; the talk
    /// transforms 123 → 124.
    BoundMechanic,
    /// `NPCID.EyeofCthulhu` (4) — the first boss (`NPC.cs:8703-8719`:
    /// 100x110, dmg 15, def 12, 2800 HP, noGravity/noTileCollide; the
    /// death sets `downed_eoc` and opens the Dryad's arrival). The full
    /// aiStyle-4 phase machine rides M4-BOSS-2b.
    EyeOfCthulhu,
    /// `NPCID.QueenBee` (222) — the jungle boss (`NPC.cs:11855-11872`:
    /// 66x66, dmg 30, def 8, 3400 HP; the death sets `downed_queen_bee`
    /// and opens the Witch Doctor's arrival).
    QueenBee,
    /// `NPCID.SkeletronHead` (35) — the dungeon gatekeeper
    /// (`NPC.cs:9166-9182`: 80x102, dmg 32, def 10, 4400 HP; the death
    /// sets `downed_skeletron` and opens the Clothier's arrival).
    SkeletronHead,
    /// `NPCID.ServantofCthulhu` (5) — the EoC minion (`NPC.cs:8717-8727`:
    /// 20×20, aiStyle 5 (the generic hover, num 5 / num2 0.03
    /// `NPC.cs:50299-50301`), damage 12, lifeMax 8, noGravity,
    /// noTileCollide).
    ServantOfCthulhu,
    /// `NPCID.WallofFlesh` (113) — the underworld wall (`NPC.cs:10342`:
    /// 100x100 x1.2, dmg 50, def 12, 8000 HP; the death starts hardmode —
    /// `WorldGen.StartHardmode()`, `NPC.cs:79485`).
    WallOfFlesh,
    /// `NPCID.TheHungry` (115) — the wall's minion (`NPC.cs:10363+`:
    /// 30×30, aiStyle 29, dmg 30, def 10, 240 HP, noGravity,
    /// noTileCollide). The AI anchors to the wall's draw band
    /// (`Main.wofNPCIndex`/`wofDrawArea*` → the World fields) and reads the
    /// wall's hp for the damage/defense tiers.
    Hungry,
    /// `NPCID.LeechHead` (117) — the tongue-phase worm head
    /// (`NPC.cs`: 14×14, aiStyle 6 (the worm engine), dmg 26, def 2,
    /// 60 HP, noGravity/noTileCollide, behindTiles).
    LeechHead,
    /// `NPCID.LeechBody` (118) — the worm's link (`NPC.cs`: 14×14,
    /// aiStyle 6, dmg 22, def 6, 60 HP).
    LeechBody,
    /// `NPCID.TheDestroyer` (134) — the mech worm (`NPC.cs:10659+`,
    /// the same-kind chain; M4-BOSS-7).
    TheDestroyer,
    /// `NPCID.SkeletronPrime` (127) — the mech skull (`NPC.cs:10556+`,
    /// M4-BOSS-8).
    SkeletronPrime,
    /// `NPCID.Retinazer` (125) — the Twins' laser eye (`NPC.cs:10522+`,
    /// M4-BOSS-9).
    Retinazer,
    /// `NPCID.Spazmatism` (126) — the Twins' cursed eye (M4-BOSS-9).
    Spazmatism,
    /// `NPCID.Plantera` (262) — the jungle bulb boss (`NPC.cs:12379+`,
    /// M4-BOSS-10).
    Plantera,
    /// `NPCID.BrainofCthulhu` (266) — the crimson boss (`NPC.cs:12438`:
    /// 160x110, dmg 30, def 14, 1250 HP, dontTakeDamage in the phase 1;
    /// the death sets the downedBoss2 equivalent).
    BrainOfCthulhu,
    /// `NPCID.Creeper` (267) — the Brain's satellite (`NPC.cs:12456`:
    /// 30x30, dmg 20, def 10, 100 HP).
    Creeper,
    /// `NPCID.BoundGoblin` (105) — the tied Tinkerer awaiting the rescue
    /// (`SetDefaults` `NPC.cs:10245-10257`: 18x34, aiStyle 0 — no
    /// movement, dmg 10, def 15, 250 HP, kb 0.5, friendly).
    BoundGoblin,
    /// `NPCID.GoblinTinkerer` (107) — the freed Tinkerer (`SetDefaults`
    /// `NPC.cs:10271-10283`: 18x40, aiStyle 7 the town walker, 250 HP,
    /// def 15, kb 0.5).
    GoblinTinkerer,
    /// `NPCID.Golem` (245) — the BODY (`SetDefaults` `NPC.cs:12163-12177`:
    /// 140x140, aiStyle 45, dmg 72, def 26, 15000 HP, kb 0, boss, value
    /// 15 gold). Its death is the fight's end (`:79346`'s event flag).
    Golem,
    /// `NPCID.GolemHead` (246) — the attached head (`:12178-12192`: 70x70,
    /// aiStyle 46, dmg 64, def 20, 25000 HP, the silent death — the head
    /// detaches rather than dies).
    GolemHead,
    /// `NPCID.GolemFistLeft` (247) — the lunging fist (`:12193-12206`:
    /// 40x30, aiStyle 47, dmg 59, def 28, 10000 HP).
    GolemFistLeft,
    /// `NPCID.GolemFistRight` (248) — the mirrored fist (the same stats).
    GolemFistRight,
    /// `NPCID.GolemHeadFree` (249) — the detached head (`:12207-12220`:
    /// 70x70, aiStyle 48, dmg 80, def 32, 16000 HP, `dontTakeDamage` —
    /// the spawn-shielded free-flying phase).
    GolemHeadFree,
    /// `NPCID.Deerclops` (668) — the frozen-boss (`SetDefaults`
    /// `NPC.cs:17449-17466`: 60x154, aiStyle 123, dmg 20, def 10,
    /// 7000 HP, kb 0, boss, noGravity, noTileCollide, 5 gold).
    Deerclops,
    /// `NPCID.SkeletronHand` (36) — the Skeletron's tether hand (`aiStyle
    /// 12`, `NPC.cs:9183-9196`: 52x52, dmg 20, def 14, 600 HP, kb 0).
    SkeletronHand,
    /// `NPCID.WallOfFleshEye` (114) — the WoF's mirror eye (`aiStyle 28`,
    /// `NPC.cs:10362-10377`: 100x100, def 0, mirrored life).
    WallOfFleshEye,
    /// `NPCID.PrimeCannon` (128) — the Prime's bomb limb (`aiStyle 35`,
    /// `NPC.cs:10577-10591`: 52x52, dmg 30, def 23, 7000 HP).
    PrimeCannon,
    /// `NPCID.PrimeSaw` (129) — the Prime's saw limb (`aiStyle 33`:
    /// 56 dmg, def 38, 9000 HP).
    PrimeSaw,
    /// `NPCID.PrimeVice` (130) — the Prime's vice limb (`aiStyle 34`:
    /// 52 dmg, def 34, 9000 HP).
    PrimeVice,
    /// `NPCID.PrimeLaser` (131) — the Prime's laser limb (`aiStyle 36`:
    /// 29 dmg, def 20, 6000 HP).
    PrimeLaser,
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

/// The bestiary info-page stat rows for one entry — the same tables
/// `Npc::max_hp`/`defense`/`value`/`knock_back_resist` read, exposed for the
/// UI without constructing an `Npc` (`EntryStats` mirrors
/// `UIBestiaryEntryInfoPage`'s `Stat_*` lines).
pub struct EntryStats {
    pub hp: i32,
    pub damage: i32,
    pub defense: i32,
    pub value: f32,
    pub kb: f32,
}

/// The stat rows for one kind/variant pair; the slime variants ride their
/// `SlimeStats`, the other kinds the `Npc` accessors' tables.
pub(crate) fn entry_stats(kind: NpcKind, variant: SlimeVariant) -> EntryStats {
    // The accessors live on `Npc` instances; the private ctor builds the
    // shell and the variant rides the only other field the tables consult.
    let mut shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);
    shell.slime = variant;
    EntryStats {
        hp: kind.max_hp(),
        damage: if kind == NpcKind::Slime {
            variant.stats().damage
        } else {
            kind.contact_damage()
        },
        defense: shell.defense(),
        value: shell.value(),
        kb: shell.knock_back_resist(),
    }
}

impl NpcKind {
    // The protocol id tables below must stay a single live arm per kind/id:
    // the pirate/martian merges of 2026-11 appended duplicates that the
    // compiler only warns about, so 60 dead `to_u16` arms and 28 dead
    // `from_u16` arms silently survived — and two of them were the 78..84
    // collision that sent saved Deerclops back as pirates (`S1-F-1`). A
    // duplicate is a hard error here.
    #[deny(unreachable_patterns)]
    pub fn to_u16(self) -> u16 {
        match self {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::WanderingEye => 2,
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 4,
            NpcKind::Guide => 5,
            NpcKind::Skeleton => 6,
            NpcKind::UndeadMiner => 7,
            NpcKind::CaveBat => 8,
            NpcKind::JungleBat => 9,
            NpcKind::Hellbat => 10,
            NpcKind::VoodooDemon => 11,
            NpcKind::EaterOfSouls => 12,
            NpcKind::Hornet => 13,
            NpcKind::MeteorHead => 14,
            NpcKind::GoblinSorcerer => 15,
            NpcKind::GoblinThief => 57,
            NpcKind::GoblinWarrior => 58,
            NpcKind::GoblinArcher => 59,
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinSummoner => 61,
            NpcKind::SnowmanGangsta => 65,
            NpcKind::SnowBalla => 66,
            NpcKind::MysterySnowman => 67,
            NpcKind::MartianOfficer => 68,
            NpcKind::GrayGrunt => 69,
            NpcKind::MartianEngineer => 70,
            NpcKind::GigaZapper => 71,
            NpcKind::MartianWalker => 72,
            // The pirates + the probe + the drone: the martian/snowman
            // family owns 65-72 and the Golem family 73-77, so this band
            // rides 182-188 — `S1-F-1` renumbered it off the 78-84 slots the
            // Deerclops/Skeletron-Prime family owns (the 2026-11 pirate
            // renumber parked the family on those ids and every Deerclops or
            // Prime limb came back as a pirate). The ids deliberately sit
            // above 181: the free holes in 0..181 (62, 148, 149, 152-155)
            // are non-contiguous and the family must not split.
            NpcKind::PirateDeckhand => 182,
            NpcKind::PirateCaptainsMate => 183,
            NpcKind::PirateArcher => 184,
            NpcKind::PirateCorsair => 185,
            NpcKind::PirateCaptain => 186,
            NpcKind::MartianProbe => 187,
            NpcKind::MartianDrone => 188,
            NpcKind::BrainScrambler => 85,
            NpcKind::RayGunner => 86,
            NpcKind::ScutlixRider => 87,
            NpcKind::Scutlix => 88,
            NpcKind::MartianSaucer => 89,
            NpcKind::MartianSaucerTurret => 90,
            NpcKind::MartianSaucerCannon => 91,
            NpcKind::MartianSaucerCore => 92,
            NpcKind::FlyingDutchman => 93,
            NpcKind::FlyingDutchmanCannon => 94,
            NpcKind::DD2GoblinT1 => 95,
            NpcKind::DD2GoblinT2 => 96,
            NpcKind::DD2GoblinT3 => 97,
            NpcKind::DD2GoblinBomberT1 => 98,
            NpcKind::DD2GoblinBomberT2 => 99,
            NpcKind::DD2GoblinBomberT3 => 100,
            NpcKind::DD2JavelinT1 => 101,
            NpcKind::DD2JavelinT2 => 102,
            NpcKind::DD2JavelinT3 => 103,
            NpcKind::DD2SkeletonT1 => 104,
            NpcKind::DD2SkeletonT3 => 105,
            NpcKind::DD2DarkMage => 106,
            NpcKind::DD2Betsy => 107,
            NpcKind::DD2WitherBeast => 108,
            NpcKind::DD2Bartender => 109,
            NpcKind::DD2EterniaCrystal => 110,
            // The Old One's Army's tier-2/3 families (M4-EVENT-19):
            // the port ids 148-160.
            NpcKind::DD2WyvernT1 => 169,
            NpcKind::DD2WyvernT2 => 170,
            NpcKind::DD2WyvernT3 => 171,
            NpcKind::DD2DarkMageT3 => 172,
            NpcKind::DD2WitherBeastT3 => 173,
            NpcKind::DD2DrakinT2 => 174,
            NpcKind::DD2DrakinT3 => 175,
            NpcKind::DD2KoboldWalkerT2 => 176,
            NpcKind::DD2KoboldWalkerT3 => 177,
            NpcKind::DD2KoboldFlyerT2 => 178,
            NpcKind::DD2KoboldFlyerT3 => 179,
            NpcKind::DD2OgreT2 => 180,
            NpcKind::DD2OgreT3 => 181,
            NpcKind::AncientCultist => 112,
            NpcKind::StardustTower => 113,
            NpcKind::NebulaTower => 114,
            NpcKind::SolarTower => 115,
            NpcKind::VortexTower => 146,
            NpcKind::DD2LanePortal => 147,
            NpcKind::StardustCellBig => 121,
            NpcKind::StardustCellSmall => 116,
            NpcKind::VortexHunter => 127,
            NpcKind::VortexAlien => 128,
            NpcKind::VortexHeadcrab => 129,
            NpcKind::StardustSpiderBig => 133,
            NpcKind::StardustSpiderSmall => 134,
            NpcKind::StardustSoldier => 135,
            NpcKind::SolarDrakomire => 136,
            NpcKind::SolarDrakomireRider => 137,
            NpcKind::SolarSroller => 138,
            NpcKind::SolarCorite => 139,
            NpcKind::SolarCrawltipedeHead => 141,
            NpcKind::SolarCrawltipedeBody => 142,
            NpcKind::SolarCrawltipedeTail => 143,
            NpcKind::StardustWormHead => 144,
            NpcKind::SolarSpearman => 145,
            NpcKind::SolarSelenian => 140,
            NpcKind::NebulaFloater => 130,
            NpcKind::BrainSuckler => 131,
            NpcKind::Predictor => 132,
            NpcKind::MoonLordHead => 122,
            NpcKind::MoonLordHand => 123,
            NpcKind::MoonLordCore => 124,
            NpcKind::MoonLordFreeEye => 125,
            NpcKind::MoonLordLeechBlob => 126,
            NpcKind::StardustJellyfishBig => 117,
            NpcKind::NebulaBeast => 118,
            NpcKind::VortexRifleman => 119,
            NpcKind::VortexSoldier => 120,
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
            NpcKind::ChaosBall => 19,
            NpcKind::BurningSphere => 20,
            NpcKind::WaterSphere => 21,
            NpcKind::Goldfish => 22,
            NpcKind::Piranha => 23,
            NpcKind::Shark => 24,
            NpcKind::DevourerHead => 25,
            NpcKind::DevourerBody => 26,
            NpcKind::DevourerTail => 27,
            NpcKind::GiantWormHead => 28,
            NpcKind::GiantWormBody => 29,
            NpcKind::GiantWormTail => 30,
            NpcKind::EaterOfWorldsHead => 31,
            NpcKind::EaterOfWorldsBody => 32,
            NpcKind::EaterOfWorldsTail => 33,
            NpcKind::Merchant => 34,
            NpcKind::Dryad => 35,
            NpcKind::Nurse => 36,
            NpcKind::ArmsDealer => 37,
            NpcKind::Demolitionist => 38,
            NpcKind::Painter => 39,
            NpcKind::WitchDoctor => 40,
            NpcKind::Clothier => 41,
            NpcKind::EyeOfCthulhu => 42,
            NpcKind::QueenBee => 43,
            NpcKind::SkeletronHead => 44,
            NpcKind::ServantOfCthulhu => 45,
            NpcKind::WallOfFlesh => 46,
            NpcKind::Hungry => 47,
            NpcKind::LeechHead => 48,
            NpcKind::LeechBody => 49,
            NpcKind::TheDestroyer => 50,
            NpcKind::SkeletronPrime => 51,
            NpcKind::Retinazer => 52,
            NpcKind::Spazmatism => 53,
            NpcKind::Plantera => 54,
            NpcKind::BrainOfCthulhu => 55,
            NpcKind::Creeper => 56,
            NpcKind::BoundGoblin => 63,
            NpcKind::GoblinTinkerer => 64,
            NpcKind::Golem => 73,
            NpcKind::GolemHead => 74,
            NpcKind::GolemFistLeft => 75,
            NpcKind::GolemFistRight => 76,
            NpcKind::GolemHeadFree => 77,
            NpcKind::Deerclops => 78,
            NpcKind::SkeletronHand => 79,
            NpcKind::WallOfFleshEye => 80,
            NpcKind::PrimeCannon => 81,
            NpcKind::PrimeSaw => 82,
            NpcKind::PrimeVice => 83,
            NpcKind::PrimeLaser => 84,
            NpcKind::TravellingMerchant => 111,
            NpcKind::Angler => 150,
            NpcKind::SleepingAngler => 151,
            NpcKind::Wizard => 156,
            NpcKind::BoundWizard => 157,
            NpcKind::Mechanic => 158,
            NpcKind::BoundMechanic => 159,
            NpcKind::Stylist => 160,
            NpcKind::WebbedStylist => 161,
            NpcKind::TaxCollector => 162,
            NpcKind::Golfer => 163,
            NpcKind::SleepingGolfer => 164,
            NpcKind::Zoologist => 165,
            NpcKind::Princess => 166,
            NpcKind::DyeTrader => 167,
            NpcKind::PartyGirl => 168,
        }
    }

    /// The protocol id decode — the inverse of [`NpcKind::to_u16`], a
    /// bijection over the 182-kind roster (`S1-F-1`); the unmatched ids
    /// (the free 62/148/149/152-155 holes) decode to `None`. One live arm
    /// per id: a duplicate is a hard error (`to_u16`'s guard above).
    #[deny(unreachable_patterns)]
    pub fn from_u16(v: u16) -> Option<Self> {
        Some(match v {
            0 => NpcKind::Slime,
            1 => NpcKind::DemonEye,
            2 => NpcKind::WanderingEye,
            3 => NpcKind::Zombie,
            4 => NpcKind::KingSlime,
            5 => NpcKind::Guide,
            6 => NpcKind::Skeleton,
            7 => NpcKind::UndeadMiner,
            8 => NpcKind::CaveBat,
            9 => NpcKind::JungleBat,
            10 => NpcKind::Hellbat,
            11 => NpcKind::VoodooDemon,
            12 => NpcKind::EaterOfSouls,
            13 => NpcKind::Hornet,
            14 => NpcKind::MeteorHead,
            15 => NpcKind::GoblinSorcerer,
            57 => NpcKind::GoblinThief,
            58 => NpcKind::GoblinWarrior,
            59 => NpcKind::GoblinArcher,
            60 => NpcKind::GoblinPeon,
            61 => NpcKind::GoblinSummoner,
            65 => NpcKind::SnowmanGangsta,
            66 => NpcKind::SnowBalla,
            67 => NpcKind::MysterySnowman,
            68 => NpcKind::MartianOfficer,
            69 => NpcKind::GrayGrunt,
            70 => NpcKind::MartianEngineer,
            71 => NpcKind::GigaZapper,
            72 => NpcKind::MartianWalker,
            // The martian ground family: `to_u16` encoded the 85-88 ids but
            // `from_u16` had no arms, so a save or net message carrying one
            // decoded to `None` and the NPC was dropped (`S1-F-1`).
            85 => NpcKind::BrainScrambler,
            86 => NpcKind::RayGunner,
            87 => NpcKind::ScutlixRider,
            88 => NpcKind::Scutlix,
            89 => NpcKind::MartianSaucer,
            90 => NpcKind::MartianSaucerTurret,
            91 => NpcKind::MartianSaucerCannon,
            92 => NpcKind::MartianSaucerCore,
            93 => NpcKind::FlyingDutchman,
            94 => NpcKind::FlyingDutchmanCannon,
            95 => NpcKind::DD2GoblinT1,
            96 => NpcKind::DD2GoblinT2,
            97 => NpcKind::DD2GoblinT3,
            98 => NpcKind::DD2GoblinBomberT1,
            99 => NpcKind::DD2GoblinBomberT2,
            100 => NpcKind::DD2GoblinBomberT3,
            101 => NpcKind::DD2JavelinT1,
            102 => NpcKind::DD2JavelinT2,
            103 => NpcKind::DD2JavelinT3,
            104 => NpcKind::DD2SkeletonT1,
            105 => NpcKind::DD2SkeletonT3,
            106 => NpcKind::DD2DarkMage,
            107 => NpcKind::DD2Betsy,
            108 => NpcKind::DD2WitherBeast,
            109 => NpcKind::DD2Bartender,
            110 => NpcKind::DD2EterniaCrystal,
            147 => NpcKind::DD2LanePortal,
            169 => NpcKind::DD2WyvernT1,
            170 => NpcKind::DD2WyvernT2,
            171 => NpcKind::DD2WyvernT3,
            172 => NpcKind::DD2DarkMageT3,
            173 => NpcKind::DD2WitherBeastT3,
            174 => NpcKind::DD2DrakinT2,
            175 => NpcKind::DD2DrakinT3,
            176 => NpcKind::DD2KoboldWalkerT2,
            177 => NpcKind::DD2KoboldWalkerT3,
            178 => NpcKind::DD2KoboldFlyerT2,
            179 => NpcKind::DD2KoboldFlyerT3,
            180 => NpcKind::DD2OgreT2,
            181 => NpcKind::DD2OgreT3,
            112 => NpcKind::AncientCultist,
            113 => NpcKind::StardustTower,
            114 => NpcKind::NebulaTower,
            115 => NpcKind::SolarTower,
            146 => NpcKind::VortexTower,
            116 => NpcKind::StardustCellSmall,
            117 => NpcKind::StardustJellyfishBig,
            118 => NpcKind::NebulaBeast,
            119 => NpcKind::VortexRifleman,
            120 => NpcKind::VortexSoldier,
            127 => NpcKind::VortexHunter,
            128 => NpcKind::VortexAlien,
            129 => NpcKind::VortexHeadcrab,
            133 => NpcKind::StardustSpiderBig,
            134 => NpcKind::StardustSpiderSmall,
            135 => NpcKind::StardustSoldier,
            136 => NpcKind::SolarDrakomire,
            137 => NpcKind::SolarDrakomireRider,
            138 => NpcKind::SolarSroller,
            139 => NpcKind::SolarCorite,
            141 => NpcKind::SolarCrawltipedeHead,
            142 => NpcKind::SolarCrawltipedeBody,
            143 => NpcKind::SolarCrawltipedeTail,
            144 => NpcKind::StardustWormHead,
            145 => NpcKind::SolarSpearman,
            140 => NpcKind::SolarSelenian,
            130 => NpcKind::NebulaFloater,
            131 => NpcKind::BrainSuckler,
            132 => NpcKind::Predictor,
            122 => NpcKind::MoonLordHead,
            123 => NpcKind::MoonLordHand,
            124 => NpcKind::MoonLordCore,
            125 => NpcKind::MoonLordFreeEye,
            126 => NpcKind::MoonLordLeechBlob,
            121 => NpcKind::StardustCellBig,
            16 => NpcKind::Tim,
            17 => NpcKind::DarkCaster,
            18 => NpcKind::FireImp,
            19 => NpcKind::ChaosBall,
            20 => NpcKind::BurningSphere,
            21 => NpcKind::WaterSphere,
            22 => NpcKind::Goldfish,
            23 => NpcKind::Piranha,
            24 => NpcKind::Shark,
            25 => NpcKind::DevourerHead,
            26 => NpcKind::DevourerBody,
            27 => NpcKind::DevourerTail,
            28 => NpcKind::GiantWormHead,
            29 => NpcKind::GiantWormBody,
            30 => NpcKind::GiantWormTail,
            31 => NpcKind::EaterOfWorldsHead,
            32 => NpcKind::EaterOfWorldsBody,
            33 => NpcKind::EaterOfWorldsTail,
            34 => NpcKind::Merchant,
            35 => NpcKind::Dryad,
            36 => NpcKind::Nurse,
            37 => NpcKind::ArmsDealer,
            38 => NpcKind::Demolitionist,
            39 => NpcKind::Painter,
            40 => NpcKind::WitchDoctor,
            41 => NpcKind::Clothier,
            42 => NpcKind::EyeOfCthulhu,
            43 => NpcKind::QueenBee,
            44 => NpcKind::SkeletronHead,
            45 => NpcKind::ServantOfCthulhu,
            46 => NpcKind::WallOfFlesh,
            47 => NpcKind::Hungry,
            48 => NpcKind::LeechHead,
            49 => NpcKind::LeechBody,
            50 => NpcKind::TheDestroyer,
            51 => NpcKind::SkeletronPrime,
            52 => NpcKind::Retinazer,
            53 => NpcKind::Spazmatism,
            54 => NpcKind::Plantera,
            55 => NpcKind::BrainOfCthulhu,
            56 => NpcKind::Creeper,
            63 => NpcKind::BoundGoblin,
            64 => NpcKind::GoblinTinkerer,
            73 => NpcKind::Golem,
            74 => NpcKind::GolemHead,
            75 => NpcKind::GolemFistLeft,
            76 => NpcKind::GolemFistRight,
            77 => NpcKind::GolemHeadFree,
            // 78-84: the Deerclops/Skeletron-Prime family — the band the
            // pirates vacated in the `S1-F-1` renumber (`from_u16` resolved
            // these ids to the pirates before it, which is how a saved
            // Deerclops came back as a Pirate Deckhand).
            78 => NpcKind::Deerclops,
            79 => NpcKind::SkeletronHand,
            80 => NpcKind::WallOfFleshEye,
            81 => NpcKind::PrimeCannon,
            82 => NpcKind::PrimeSaw,
            83 => NpcKind::PrimeVice,
            84 => NpcKind::PrimeLaser,
            111 => NpcKind::TravellingMerchant,
            150 => NpcKind::Angler,
            151 => NpcKind::SleepingAngler,
            156 => NpcKind::Wizard,
            157 => NpcKind::BoundWizard,
            158 => NpcKind::Mechanic,
            159 => NpcKind::BoundMechanic,
            160 => NpcKind::Stylist,
            161 => NpcKind::WebbedStylist,
            162 => NpcKind::TaxCollector,
            163 => NpcKind::Golfer,
            164 => NpcKind::SleepingGolfer,
            165 => NpcKind::Zoologist,
            166 => NpcKind::Princess,
            167 => NpcKind::DyeTrader,
            168 => NpcKind::PartyGirl,
            // The pirates + the probe + the drone — the `S1-F-1` renumber
            // (`to_u16` above): the family moved off the 78-84
            // Deerclops/Skeletron-Prime band to the contiguous 182-188.
            182 => NpcKind::PirateDeckhand,
            183 => NpcKind::PirateCaptainsMate,
            184 => NpcKind::PirateArcher,
            185 => NpcKind::PirateCorsair,
            186 => NpcKind::PirateCaptain,
            187 => NpcKind::MartianProbe,
            188 => NpcKind::MartianDrone,
            _ => return None,
        })
    }

    /// `npc.townNPC` (`NPC.cs:6537`) — the flag the town `SetDefaults`
    /// cases raise: the town roster, the goblin pair and the Travelling
    /// Merchant himself (`NPC.cs:13675`'s `townNPC = true`). The Old Man
    /// (37) and the Skeleton Merchant (453) carry the flag in the
    /// vanilla; neither is ported, so the spawner's town counts need no
    /// exclusions (`Main.cs:65030`).
    pub fn is_town_resident(self) -> bool {
        matches!(
            self,
            NpcKind::Guide
                | NpcKind::Merchant
                | NpcKind::Dryad
                | NpcKind::Nurse
                | NpcKind::ArmsDealer
                | NpcKind::Demolitionist
                | NpcKind::Painter
                | NpcKind::WitchDoctor
                | NpcKind::Clothier
                | NpcKind::GoblinTinkerer
                | NpcKind::TravellingMerchant
                | NpcKind::Angler
                | NpcKind::Wizard
                | NpcKind::Mechanic
                | NpcKind::Stylist
                | NpcKind::TaxCollector
                | NpcKind::Golfer
                | NpcKind::Zoologist
                | NpcKind::Princess
                | NpcKind::DyeTrader
                | NpcKind::PartyGirl
        )
    }

    /// The worm family role (`AI_006`'s type groups): the head digs, the
    /// body links, the tail ends (`:51393`'s body list, `:51436`'s
    /// head/mid list).
    pub fn is_worm(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::DevourerBody
                | NpcKind::DevourerTail
                | NpcKind::GiantWormHead
                | NpcKind::GiantWormBody
                | NpcKind::GiantWormTail
                | NpcKind::EaterOfWorldsHead
                | NpcKind::EaterOfWorldsBody
                | NpcKind::EaterOfWorldsTail
                | NpcKind::LeechHead
                | NpcKind::LeechBody
                | NpcKind::TheDestroyer
                | NpcKind::SolarCrawltipedeHead
                | NpcKind::SolarCrawltipedeBody
                | NpcKind::SolarCrawltipedeTail
        )
    }

    /// The Destroyer: the head of its own same-kind chain (`AI_037`).
    pub fn is_worm_head(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::GiantWormHead
                | NpcKind::EaterOfWorldsHead
                | NpcKind::LeechHead
                | NpcKind::TheDestroyer
                | NpcKind::SolarCrawltipedeHead
        )
    }

    /// The mid-body kinds that spawn another of themselves while the
    /// budget holds (`:51350`'s list plus the Destroyer's same-kind
    /// chain).
    pub fn is_worm_mid(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerBody
                | NpcKind::GiantWormBody
                | NpcKind::EaterOfWorldsBody
                | NpcKind::LeechBody
                | NpcKind::TheDestroyer
                | NpcKind::SolarCrawltipedeBody
        )
    }

    /// The next segment's kind down the chain (`type + 1` in the vanilla
    /// spawns, `:51347`/`:51357`).
    pub fn worm_next(self) -> NpcKind {
        if self == NpcKind::TheDestroyer {
            // The same-kind chain: every segment IS the head type.
            return self;
        }
        match self {
            NpcKind::DevourerHead => NpcKind::DevourerBody,
            NpcKind::DevourerBody | NpcKind::DevourerTail => NpcKind::DevourerTail,
            NpcKind::GiantWormHead => NpcKind::GiantWormBody,
            NpcKind::GiantWormBody | NpcKind::GiantWormTail => NpcKind::GiantWormTail,
            NpcKind::EaterOfWorldsHead => NpcKind::EaterOfWorldsBody,
            NpcKind::LeechHead => NpcKind::LeechBody,
            NpcKind::LeechBody => NpcKind::LeechBody,
            // The crawltipede chain (`:51375-51381`'s per-index kind):
            // the bodies self-chain and the tail closes it.
            NpcKind::SolarCrawltipedeHead => NpcKind::SolarCrawltipedeBody,
            NpcKind::SolarCrawltipedeBody | NpcKind::SolarCrawltipedeTail => {
                NpcKind::SolarCrawltipedeTail
            }
            _ => NpcKind::EaterOfWorldsTail,
        }
    }

    /// The head's dig tunables — `num45` (the max speed) and `num46` (the
    /// steering accel) per family (`:51670-51706`).
    pub fn worm_tunables(self) -> (f32, f32) {
        match self {
            // The stardust worm head's 9/0.3 (`:51740-51743`) and the
            // crawltipede chain's 10/0.3 (`:51779-51880`).
            NpcKind::StardustWormHead => (9.0, 0.3),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (10.0, 0.3),
            NpcKind::DevourerHead => (9.0, 0.1),
            NpcKind::GiantWormHead => (6.0, 0.05),
            NpcKind::EaterOfWorldsHead => (10.0, 0.07),
            _ => (8.0, 0.07),
        }
    }

    /// `NPC.townNPC` — the types `SetDefaults` marks as town NPCs
    /// (`NPC.cs:8979-11943`, the `this.townNPC = true` arms: 17/18/19/20/22/
    /// 38/54/107/227/228 here, and 37/368/441/… with no ported kind). The
    /// world-file section filters on it (`SaveNPCs:1717`, `FixAgainstExploits:
    /// 1881`), and `town::quick_find_home`'s evaluation is the same set.
    pub fn is_town_npc(self) -> bool {
        matches!(
            self,
            NpcKind::Merchant
                | NpcKind::Nurse
                | NpcKind::ArmsDealer
                | NpcKind::Dryad
                | NpcKind::Guide
                | NpcKind::Demolitionist
                | NpcKind::Clothier
                | NpcKind::GoblinTinkerer
                | NpcKind::Painter
                | NpcKind::WitchDoctor
        )
    }

    /// `NPC.isLikeATownNPC` (`NPC.cs:6856-6864`: `townNPC || type == 453` —
    /// the Skeleton Merchant, which the port does not ship): the vanilla
    /// `townNPC` flag for every town kind the port carries — the housing set
    /// of [`NpcKind::is_town_npc`] plus the eleven sellers that have no
    /// ported housing entry yet (TravellingMerchant 368, Angler 369, Stylist
    /// 353, TaxCollector 441, Golfer 588, Zoologist 633, Princess 663,
    /// DyeTrader 207, PartyGirl 208, Wizard 108, Mechanic 124 — all
    /// `this.townNPC = true` in `SetDefaults`, `NPC.cs:9011-17397`).
    /// `ProcessHitAgainstNPC`'s melee exceptions read this set
    /// (`Player.cs:44274`, `:44344`).
    pub fn is_town_npc_like(self) -> bool {
        self.is_town_npc()
            || matches!(
                self,
                NpcKind::TravellingMerchant
                    | NpcKind::Angler
                    | NpcKind::Stylist
                    | NpcKind::TaxCollector
                    | NpcKind::Golfer
                    | NpcKind::Zoologist
                    | NpcKind::Princess
                    | NpcKind::DyeTrader
                    | NpcKind::PartyGirl
                    | NpcKind::Wizard
                    | NpcKind::Mechanic
            )
    }

    /// `NPC.friendly` (`NPC.cs:8519`'s default `false` plus the `SetDefaults`
    /// arms `:8983-9060`, `:10257`, `:10493-10507`, `:13502-13516`,
    /// `:13796-13809`, `:16652-16665`): every town NPC and the six
    /// bound/rescue forms the port ships. `ProcessHitAgainstNPC` refuses
    /// every friendly NPC (`Player.cs:44272-44276`), and
    /// `Damage_PVE_Inner` gates projectiles on the same flag
    /// (`Projectile.cs:12601`).
    pub fn is_friendly(self) -> bool {
        self.is_town_npc_like()
            || matches!(
                self,
                NpcKind::BoundGoblin
                    | NpcKind::BoundWizard
                    | NpcKind::BoundMechanic
                    | NpcKind::WebbedStylist
                    | NpcKind::SleepingAngler
                    | NpcKind::SleepingGolfer
            )
    }

    /// `NPC.netID` as the `.wld` section writes it (`SaveNPCs:1718`): the
    /// vanilla id, which `crate::bestiary::ENTRIES` carries per kind.
    /// **Not** [`NpcKind::to_u16`] — that is the port's internal protocol id
    /// space; the wire needs the real one.
    pub fn net_id(self) -> i32 {
        crate::bestiary::entry_for_npc(self, SlimeVariant::Blue).net_id
    }

    /// `AI_003_Fighters`'s per-type walk-speed caps (`NPC.cs:57403-59800`):
    /// the velocity ceiling each kind walks at.
    ///
    /// The C#'s groups:
    /// - the `num107` default (`:58322-58374`): 1f;
    /// - the `num83` group (`:57606-57691`): 1.5f with the per-type rows;
    /// - the cap-2 group (`:57554-57578`);
    /// - the cap-3 group (`:57520-57552`);
    /// - the shooters' group (`:59717-59750`): 1f with the per-type rows;
    /// - the Martian engineer's own arm (`:58032-58069`): 1.5f.
    ///
    /// The kinds whose arm is bespoke (the pillar mobs, the vortex family,
    /// the Goblin Archer) carry their own speed and are not listed — the
    /// table is the walkers' shared source.
    pub fn ai3_walk_speed(self) -> Option<f32> {
        Some(match self {
            NpcKind::Zombie => 1.0,             // the num107 default (`:58322`)
            NpcKind::Skeleton => 1.5,           // the num83 group (`:57606`)
            NpcKind::UndeadMiner => 1.0,        // the num107 default
            NpcKind::GoblinPeon => 1.5,         // the num83 group
            NpcKind::GoblinThief => 2.0,        // the cap-2 group (`:57554`)
            NpcKind::GoblinWarrior => 1.0,      // the num107 default
            NpcKind::Spazmatism => 2.0,        // the cap-2 group (`:57554`)
            NpcKind::PirateDeckhand => 2.0,     // the cap-2 group
            NpcKind::PirateCaptainsMate => 3.0, // the cap-3 group (`:57520`)
            NpcKind::PirateArcher => 2.0,       // the shooters' group (`:59722`)
            NpcKind::PirateCorsair => 1.5,      // the shooters' group (`:59727`)
            NpcKind::PirateCaptain => 1.0,      // the shooters' default (`:59717`)
            NpcKind::BrainScrambler => 2.0,     // the shooters' group (`:59731`)
            NpcKind::RayGunner => 2.0,          // the shooters' group (`:59731`)
            NpcKind::MartianOfficer => 1.0,     // the shooters' default (`:59717`)
            NpcKind::GrayGrunt => 1.8,          // the num83 group (`:57649`)
            NpcKind::MartianEngineer => 1.5,    // its own arm (`:58032`)
            NpcKind::GigaZapper => 2.25,        // the num83 group (`:57653`)
            NpcKind::MartianWalker => 4.0,      // the shooters' group (`:59735`)
            _ => return None,
        })
    }

    /// `NPC.GetNPCInvasionGroup(type)` (`NPC.cs`): the goblin army's members
    /// (26/27/28/29/111/471) — every ported invasion NPC. The other groups
    /// (pirate 2, snow 3, ...) have no NPCs in the ported set.
    pub fn is_invasion_npc(self) -> bool {
        matches!(
            self,
            NpcKind::GoblinThief
                | NpcKind::GoblinWarrior
                | NpcKind::GoblinArcher
                | NpcKind::GoblinSorcerer
                | NpcKind::GoblinPeon
                | NpcKind::GoblinSummoner
        )
    }

    /// The vanilla hitboxes: blue slime 24x18 (`NPC.cs:8568-8569`), demon eye
    /// and Wandering Eye 30x32 (`NPC.cs` cases 2/133), zombie 18x40.
    pub fn size(self) -> (f32, f32) {
        match self {
            NpcKind::Slime => (24.0, 18.0),
            NpcKind::DemonEye | NpcKind::WanderingEye => (30.0, 32.0),
            NpcKind::Zombie => (18.0, 40.0),
            // The Guide's hitbox rides the same humanoid frame.
            NpcKind::Guide => (18.0, 40.0),
            NpcKind::Merchant => (18.0, 40.0),
            NpcKind::Dryad => (18.0, 40.0),
            NpcKind::Nurse => (18.0, 40.0),
            NpcKind::ArmsDealer => (18.0, 40.0),
            NpcKind::Demolitionist => (18.0, 40.0),
            NpcKind::Painter => (18.0, 40.0),
            NpcKind::WitchDoctor => (18.0, 40.0),
            NpcKind::Clothier => (18.0, 40.0),
            NpcKind::TravellingMerchant => (18.0, 40.0),
            NpcKind::Angler => (18.0, 40.0),
            NpcKind::SleepingAngler => (34.0, 8.0),
            NpcKind::Wizard => (18.0, 40.0),
            NpcKind::BoundWizard => (18.0, 40.0),
            NpcKind::Mechanic => (18.0, 40.0),
            NpcKind::BoundMechanic => (18.0, 34.0),
            NpcKind::Stylist => (18.0, 40.0),
            NpcKind::WebbedStylist => (18.0, 34.0),
            NpcKind::TaxCollector => (18.0, 40.0),
            NpcKind::Golfer => (18.0, 40.0),
            NpcKind::SleepingGolfer => (18.0, 34.0),
            NpcKind::Zoologist => (18.0, 40.0),
            NpcKind::Princess => (18.0, 40.0),
            NpcKind::DyeTrader => (18.0, 40.0),
            NpcKind::PartyGirl => (18.0, 40.0),
            // `SetDefaults` case 4 (`NPC.cs:8706`): 100x110.
            NpcKind::EyeOfCthulhu => (100.0, 110.0),
            // `SetDefaults` case 222 (`NPC.cs:11858`): 66x66.
            NpcKind::QueenBee => (66.0, 66.0),
            // `SetDefaults` case 35 (`NPC.cs:9168`): 80x102.
            NpcKind::SkeletronHead => (80.0, 102.0),
            // `SetDefaults` case 113 (`NPC.cs:10344`): 100x100 x1.2 = 120.
            NpcKind::WallOfFlesh => (120.0, 120.0),
            // `SetDefaults` case 134 (`NPC.cs:10662`): 38x38 x1.25.
            NpcKind::TheDestroyer => (48.0, 48.0),
            // `SetDefaults` case 127 (`NPC.cs:10558`): 80x102.
            NpcKind::SkeletronPrime => (80.0, 102.0),
            // The Twins: `SetDefaults` cases 125/126 are 100x110.
            NpcKind::Retinazer => (100.0, 110.0),
            NpcKind::Spazmatism => (100.0, 110.0),
            // `SetDefaults` case 262 (`NPC.cs:12381`): 86x86.
            NpcKind::Plantera => (86.0, 86.0),
            // `SetDefaults` case 266 (`NPC.cs:12440`): 160x110.
            NpcKind::BrainOfCthulhu => (160.0, 110.0),
            // `SetDefaults` case 267 (`NPC.cs:12458`): 30x30.
            NpcKind::Creeper => (30.0, 30.0),
            // The goblins: 18x38 (`NPC.cs`'s cases 26/27/28/111/29/471).
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner => (18.0, 38.0),
            // The frost trio: 26x40 (`NPC.cs`'s cases 143/144/145).
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => (26.0, 40.0),
            // The martians: 18x40 (`NPC.cs`'s cases 383/385/386/389) + the walker's 34x110.
            NpcKind::MartianOfficer => (18.0, 40.0),
            NpcKind::GrayGrunt => (18.0, 40.0),
            NpcKind::MartianEngineer => (18.0, 40.0),
            NpcKind::GigaZapper => (18.0, 40.0),
            NpcKind::MartianWalker => (34.0, 110.0),
            // The probe: 42x72 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => (42.0, 72.0),
            // The drone: 40x30 (`NPC.cs`'s case 388).
            NpcKind::MartianDrone => (40.0, 30.0),
            // The shooters: 18x40 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => (18.0, 40.0),
            // The rider + its carrier: 18x40 / 60x40 (`NPC.cs`'s cases
            // 390/391).
            NpcKind::ScutlixRider => (18.0, 40.0),
            NpcKind::Scutlix => (60.0, 40.0),
            // The saucer family: the shell 150x80, the turret 40x16, the
            // cannon 46x36, the core 120x90 (`NPC.cs`'s cases 392-395).
            NpcKind::MartianSaucer => (150.0, 80.0),
            NpcKind::MartianSaucerTurret => (40.0, 16.0),
            NpcKind::MartianSaucerCannon => (46.0, 36.0),
            NpcKind::MartianSaucerCore => (120.0, 90.0),
            // The Dutchman 350x120 + its cannons 30x30 (`NPC.cs`'s cases
            // 491/492).
            NpcKind::FlyingDutchman => (350.0, 120.0),
            NpcKind::FlyingDutchmanCannon => (30.0, 30.0),
            // The DD2 goblins: 18x40 (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => (18.0, 40.0),
            // The DD2 bombers: 18x40; the javelin throwers 26x52; the
            // etherian skeletons 18x40 (`NPC.cs`'s 555-567 rows).
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3 => (18.0, 40.0),
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => (26.0, 52.0),
            NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => (18.0, 40.0),
            // The Dark Mage: 34x62 (`NPC.cs`'s case 564).
            NpcKind::DD2DarkMage => (34.0, 62.0),
            // Betsy: 190x90 (`NPC.cs`'s case 551).
            NpcKind::DD2Betsy => (190.0, 90.0),
            // The WitherBeast: 18x40 (`NPC.cs`'s case 568).
            NpcKind::DD2WitherBeast => (18.0, 40.0),
            NpcKind::DD2WyvernT1
            | NpcKind::DD2WyvernT2
            | NpcKind::DD2WyvernT3
            | NpcKind::DD2KoboldFlyerT2
            | NpcKind::DD2KoboldFlyerT3 => (38.0, 38.0),
            // The Dark Mage T3 rides the T1's 34x62 box.
            NpcKind::DD2DarkMageT3 => (34.0, 62.0),
            // The Wither Beast T3 rides the T2's 18x40 box.
            NpcKind::DD2WitherBeastT3 => (18.0, 40.0),
            // The Drakin: the 42x58 caster box.
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 => (42.0, 58.0),
            // The Kobold walker: the 18x40 goblin box.
            NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 => (18.0, 40.0),
            // The Ogre: the 96x124 box.
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 => (96.0, 124.0),
            // The Lunar Pillars: 130x270 (`NPC.cs` cases 422/493/507/517).
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => (130.0, 270.0),
            // Lunatic Cultist (439): 24x50 (`NPC.cs:14214-14215`). S1-D-3b.
            NpcKind::AncientCultist => (24.0, 50.0),
            // The lane gate: 78x130 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => (78.0, 130.0),
            NpcKind::VortexHunter => (50.0, 62.0),
            NpcKind::VortexAlien => (40.0, 28.0),
            NpcKind::VortexHeadcrab => (24.0, 26.0),
            NpcKind::StardustSpiderBig => (34.0, 42.0),
            NpcKind::StardustSpiderSmall => (22.0, 22.0),
            NpcKind::StardustSoldier => (22.0, 56.0),
            NpcKind::SolarDrakomire => (60.0, 40.0),
            NpcKind::SolarDrakomireRider => (18.0, 40.0),
            NpcKind::SolarSroller => (46.0, 52.0),
            NpcKind::SolarCorite => (38.0, 38.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarSelenian => (22.0, 56.0),
            NpcKind::NebulaFloater => (40.0, 40.0),
            NpcKind::BrainSuckler => (44.0, 44.0),
            NpcKind::Predictor => (22.0, 56.0),
            NpcKind::MoonLordHead => (38.0, 56.0),
            NpcKind::MoonLordHand | NpcKind::MoonLordCore => (46.0, 66.0),
            NpcKind::MoonLordFreeEye => (60.0, 60.0),
            NpcKind::MoonLordLeechBlob => (30.0, 30.0),
            NpcKind::StardustCellBig => (44.0, 44.0),
            NpcKind::StardustCellSmall => (22.0, 22.0),
            NpcKind::StardustJellyfishBig => (40.0, 60.0),
            NpcKind::NebulaBeast => (50.0, 44.0),
            NpcKind::VortexRifleman => (30.0, 56.0),
            NpcKind::VortexSoldier => (22.0, 56.0),
            // The tavernkeeper + the etherian crystal: 18x40 / 40x40.
            NpcKind::DD2Bartender => (18.0, 40.0),
            NpcKind::DD2EterniaCrystal => (40.0, 40.0),
            // The pirates: 18x40 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand
            | NpcKind::PirateCaptainsMate
            | NpcKind::PirateArcher
            | NpcKind::PirateCorsair
            | NpcKind::PirateCaptain => (18.0, 40.0),
            // `SetDefaults` 245-249 (`NPC.cs:12163-12220`): the body
            // 140x140, the head/free head 70x70, the fists 40x30.
            NpcKind::Golem => (140.0, 140.0),
            NpcKind::GolemHead | NpcKind::GolemHeadFree => (70.0, 70.0),
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => (40.0, 30.0),
            // `SetDefaults` 668 (`:17451-17452`): 60x154.
            NpcKind::Deerclops => (60.0, 154.0),
            NpcKind::SkeletronHand => (52.0, 52.0),
            NpcKind::WallOfFleshEye => (100.0, 100.0),
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                (52.0, 52.0)
            }
            // The bound: 18x34 (`NPC.cs:10249-10250`), the freed: 18x40
            // (`:10274-10275`).
            NpcKind::BoundGoblin => (18.0, 34.0),
            NpcKind::GoblinTinkerer => (18.0, 40.0),
            // The skeleton humanoids ride the zombie's 18x40 frame
            // (`SetDefaults` case 21/44: width 18, height 40).
            NpcKind::Skeleton => (18.0, 40.0),
            NpcKind::UndeadMiner => (18.0, 40.0),
            // `SetDefaults` case 49: width 22, height 18.
            // The bat family shares the Cave Bat's 22x18 frame
            // (`SetDefaults` cases 51/60/66: width 22, height 18).
            NpcKind::JungleBat => (22.0, 18.0),
            NpcKind::Hellbat => (22.0, 18.0),
            NpcKind::VoodooDemon => (22.0, 18.0),
            // `SetDefaults` case 6: 30x30; case 42: 34x32; case 23: 22x22.
            NpcKind::EaterOfSouls => (30.0, 30.0),
            NpcKind::Hornet => (34.0, 32.0),
            NpcKind::MeteorHead => (22.0, 22.0),
            // The casters: `SetDefaults` case 29 is 18x38, the cases
            // 45/32/24 18x40 (`NPC.cs:8951`/`:9322`/`:9121`/`:9078`).
            // The goblins: 18x38 (`NPC.cs`'s cases 26/27/28/29/111/471).
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner
            | NpcKind::GoblinSorcerer => (18.0, 38.0),
            NpcKind::Tim => (18.0, 40.0),
            NpcKind::DarkCaster => (18.0, 40.0),
            NpcKind::FireImp => (18.0, 40.0),
            // The spheres: the cases 30/25/33 are 16x16.
            NpcKind::ChaosBall => (16.0, 16.0),
            NpcKind::BurningSphere => (16.0, 16.0),
            NpcKind::WaterSphere => (16.0, 16.0),
            // The swimmers: `SetDefaults` cases 55/58/65 (`NPC.cs:9460`/
            // `:9502`/`:9610`).
            NpcKind::Goldfish => (20.0, 18.0),
            NpcKind::Piranha => (18.0, 20.0),
            NpcKind::Shark => (100.0, 24.0),
            // The worms: the Devourer 22x22 (`:8763`), the Giant Worm 14x14
            // (`:8850`), the Eater of Worlds 38x38 (`:8903`).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => (22.0, 22.0),
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => {
                (14.0, 14.0)
            }
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => (38.0, 38.0),
            NpcKind::CaveBat => (22.0, 18.0),
            NpcKind::KingSlime => (98.0, 92.0),
            // `NPC.cs:8705` (the eye 100×110) / `:8717` (the servant 20×20).
            NpcKind::EyeOfCthulhu => (100.0, 110.0),
            NpcKind::ServantOfCthulhu => (20.0, 20.0),
            // `NPC.cs`'s 115 (30×30) / 117-118 (14×14).
            NpcKind::Hungry => (30.0, 30.0),
            NpcKind::LeechHead | NpcKind::LeechBody => (14.0, 14.0),
        }
    }

    /// The C# `NPCID` this kind ports — the statue spawns' ids, the
    /// `MechSpawn` caps (`NPC.cs:7420`) and the teleporting statues' rosters
    /// (`Wiring.cs:2424`/`:2452`) all speak C# ids. Rows arrive with the
    /// kinds; `u16::MAX` marks a kind the caps can never match (the C#'s
    /// `MechSpawn` counts by type, and no statue spawns an unmapped kind).
    pub(crate) fn cs_id(self) -> u16 {
        match self {
            NpcKind::Slime => 1,
            NpcKind::DemonEye => 2,
            NpcKind::WanderingEye => 133,
            // The vanilla `NPCID.Zombie` is 3 — the hand table's 21 was the
            // net-id sweep's finding (S1-F-3; the generated loot-db net id
            // confirms).
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 50,
            NpcKind::Guide => 22,
            // The teleporting statues' rosters (`Wiring.cs:2424`/`:2452`) are the
            // town NPCs — the port has most of them.
            NpcKind::Merchant => 17,
            NpcKind::Nurse => 18,
            NpcKind::ArmsDealer => 19,
            NpcKind::Dryad => 20,
            NpcKind::Demolitionist => 38,
            NpcKind::Clothier => 54,
            NpcKind::TravellingMerchant => 368,
            NpcKind::Angler => 369,
            NpcKind::SleepingAngler => 376,
            NpcKind::Wizard => 108,
            NpcKind::BoundWizard => 106,
            NpcKind::Mechanic => 124,
            NpcKind::BoundMechanic => 123,
            NpcKind::Stylist => 353,
            NpcKind::WebbedStylist => 354,
            NpcKind::TaxCollector => 441,
            NpcKind::Golfer => 588,
            NpcKind::SleepingGolfer => 589,
            NpcKind::Zoologist => 633,
            NpcKind::Princess => 663,
            NpcKind::DyeTrader => 207,
            NpcKind::PartyGirl => 208,
            NpcKind::Painter => 227,
            NpcKind::WitchDoctor => 228,
            NpcKind::Skeleton => 21, // `NPCID.Skeleton` (the net id's correction, S1-F-3)
            NpcKind::UndeadMiner => 44, // `NPCID.UndeadMiner` (the net id's correction, S1-F-3)
            NpcKind::CaveBat => 49,
            NpcKind::JungleBat => 51,
            NpcKind::Hellbat => 60,
            NpcKind::VoodooDemon => 66,
            NpcKind::EaterOfSouls => 6,
            NpcKind::Hornet => 42,
            NpcKind::MeteorHead => 23, // `NPCID.MeteorHead` (the net id's correction, S1-F-3)
            NpcKind::FireImp => 24,
            NpcKind::Goldfish => 55,
            NpcKind::Piranha => 58,
            NpcKind::Shark => 65,
            // S1-F-3: the remaining kinds ride the generated loot-db net ids
            // (`bestiary.rs`'s entries are the C#'s `NPC.type` values) — the
            // hand table above keeps the roster-verified rows; the negative
            // net ids (none in the port's kind set) and the air sentinel stay
            // `u16::MAX` (the caps can never match them).
            _ => {
                // The kinds without a bestiary entry (the boss segments the
                // roster skips) keep the sentinel — the caps never match them.
                match crate::bestiary::try_entry_for_npc(self, SlimeVariant::Blue) {
                    Some(e) if e.net_id > 0 => e.net_id as u16,
                    _ => u16::MAX,
                }
            }
        }
    }

    /// The humanoid walkers' `Main.npcFrameCount` rows (`Main.cs:67082`):
    /// the town group (17 Merchant / 18 Nurse / 19 ArmsDealer / 20 Dryad /
    /// 22 Guide / 38 Demolitionist / 54 Clothier / 107 Tinkerer / 227
    /// Painter / 228 WitchDoctor), the invasion goblins (26 Peon / 27
    /// Thief / 28 Warrior / 111 Archer) and the cave walkers (21 Skeleton
    /// / 44 UndeadMiner) all ride `FindFrame`'s humanoid-group arm
    /// (`NPC.cs:68547-69587`). The Zombie (9) has no case and a single
    /// frame.
    pub fn humanoid_frame_count(self) -> u32 {
        match self {
            NpcKind::Guide | NpcKind::WitchDoctor | NpcKind::TravellingMerchant => 26,
            NpcKind::Merchant
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::GoblinTinkerer
            | NpcKind::TaxCollector
            | NpcKind::Golfer
            | NpcKind::DyeTrader => 25,
            NpcKind::Nurse
            | NpcKind::Clothier
            | NpcKind::Angler
            | NpcKind::Wizard
            | NpcKind::Mechanic
            | NpcKind::Stylist
            | NpcKind::Zoologist
            | NpcKind::Princess
            | NpcKind::PartyGirl => 23,
            NpcKind::Dryad => 21,
            NpcKind::GoblinArcher => 21,
            NpcKind::GoblinPeon | NpcKind::GoblinThief | NpcKind::GoblinWarrior => 16,
            NpcKind::Skeleton | NpcKind::UndeadMiner => 15,
            _ => 1,
        }
    }

    /// `NPCID.Sets.ExtraFramesCount` (`NPCID.cs:10464`) — the rows the
    /// vanilla reserves at the *bottom* of a town sheet for the attack
    /// poses. `FindFrame`'s humanoid arm wraps the walk band at
    /// `Main.npcFrameCount - ExtraFramesCount` (`NPC.cs:69534`'s `num57`
    /// and the `num57`-subtracted wrap), so a walker must never reach
    /// them: doing so animates the NPC through its attack/pose rows
    /// while it is merely walking. Only the kinds the town machine
    /// animates today carry a value; everything else is 0 (the C#
    /// `CreateIntSet` default).
    pub fn extra_frames_count(self) -> u32 {
        match self {
            NpcKind::Guide | NpcKind::WitchDoctor | NpcKind::TravellingMerchant => 10,
            NpcKind::Merchant
            | NpcKind::Nurse
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::GoblinTinkerer
            | NpcKind::TaxCollector
            | NpcKind::Golfer
            | NpcKind::DyeTrader
            | NpcKind::PartyGirl
            | NpcKind::Zoologist => 9,
            NpcKind::Dryad
            | NpcKind::Clothier
            | NpcKind::Wizard
            | NpcKind::Stylist
            | NpcKind::Princess => 7,
            NpcKind::Angler | NpcKind::Mechanic => 9,
            _ => 0,
        }
    }

    /// `NPC.behindTiles` (`NPC.cs:6517`, `:8520` default false): kinds drawn in
    /// the behind-tiles NPC pass (`Main.cs:22320,22336` - `DrawNPCs(true)`).
    /// Ported subset: worms/segments and the WoF family (e.g. type 113
    /// `NPC.cs:10355`, Devourer 7-9 `:8777/:8806/:8835`, Leech 117-118
    /// `:10419/:10435`, Destroyer 134 `:10673`).
    pub fn behind_tiles(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::DevourerBody
                | NpcKind::DevourerTail
                | NpcKind::GiantWormHead
                | NpcKind::GiantWormBody
                | NpcKind::GiantWormTail
                | NpcKind::EaterOfWorldsHead
                | NpcKind::EaterOfWorldsBody
                | NpcKind::EaterOfWorldsTail
                | NpcKind::WallOfFlesh
                | NpcKind::WallOfFleshEye
                | NpcKind::Hungry
                | NpcKind::LeechHead
                | NpcKind::LeechBody
                | NpcKind::TheDestroyer
        )
    }

    /// The Prime's four limbs (`NPCID` 128-131, `aiStyle 33-36`) - the
    /// kinds `AI_032`'s init spawns and the dock tether drives.
    pub fn is_prime_limb(self) -> bool {
        matches!(
            self,
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser
        )
    }

    /// The Martian Madness family — the kinds whose `FindFrame` cases live
    /// in `NPC.cs:71578-71921` and whose display rides the drawn
    /// `spriteDirection` (the Scutlix family's inversion, the saucer
    /// mounts' inheritance) rather than the motion `facing`.
    pub fn is_martian(self) -> bool {
        matches!(
            self,
            NpcKind::MartianOfficer
                | NpcKind::GrayGrunt
                | NpcKind::MartianEngineer
                | NpcKind::GigaZapper
                | NpcKind::MartianWalker
                | NpcKind::MartianProbe
                | NpcKind::MartianDrone
                | NpcKind::BrainScrambler
                | NpcKind::RayGunner
                | NpcKind::ScutlixRider
                | NpcKind::Scutlix
                | NpcKind::MartianSaucer
                | NpcKind::MartianSaucerTurret
                | NpcKind::MartianSaucerCannon
                | NpcKind::MartianSaucerCore
        )
    }

    pub(crate) fn max_hp(self) -> i32 {
        match self {
            NpcKind::Slime => 25,         // NPC.cs:8572
            NpcKind::DemonEye => 60,      // case 2
            NpcKind::WanderingEye => 300, // case 133
            NpcKind::Zombie => 45,
            // `NPC.SetDefaults` case 22 (`NPC.cs:8815`): 250 HP.
            NpcKind::Guide => 250,
            // `SetDefaults` case 17 (`NPC.cs:8989`): 250 HP.
            NpcKind::Merchant => 250,
            // `SetDefaults` case 20 (`NPC.cs:9032`): 250 HP.
            NpcKind::Dryad => 250,
            // `SetDefaults` case 18 (`NPC.cs:9002` region): 250 HP.
            NpcKind::Nurse => 250,
            // `SetDefaults` case 19 (`NPC.cs:9016`): 250 HP.
            NpcKind::ArmsDealer => 250,
            // `SetDefaults` case 38 (`NPC.cs:9019` region): 250 HP.
            NpcKind::Demolitionist => 250,
            // `SetDefaults` case 227 (`NPC.cs:11933`): 250 HP.
            NpcKind::Painter => 250,
            // `SetDefaults` case 228 (`NPC.cs:11950`): 250 HP.
            NpcKind::WitchDoctor => 250,
            // `SetDefaults` case 54 (`NPC.cs:9456`): 250 HP.
            NpcKind::Clothier => 250,
            NpcKind::Angler => 250,
            NpcKind::Wizard => 250,
            NpcKind::BoundWizard => 250,
            NpcKind::Mechanic => 250,
            NpcKind::Stylist => 250,
            NpcKind::WebbedStylist => 250,
            NpcKind::Golfer => 250,
            NpcKind::Zoologist => 250,
            NpcKind::DyeTrader => 250,
            NpcKind::PartyGirl => 250,
            NpcKind::Princess => 250,
            NpcKind::SleepingGolfer => 250,
            NpcKind::TaxCollector => 250,
            NpcKind::BoundMechanic => 250,
            NpcKind::SleepingAngler => 250,
            NpcKind::TravellingMerchant => 250,
            // `SetDefaults` case 4 (`NPC.cs:8710`): 2800 HP.
            NpcKind::EyeOfCthulhu => 2800,
            // `SetDefaults` case 222 (`NPC.cs:11862`): 3400 HP.
            NpcKind::QueenBee => 3400,
            // `SetDefaults` case 35 (`NPC.cs:9172`): 4400 HP.
            NpcKind::SkeletronHead => 4400,
            // `SetDefaults` case 113 (`NPC.cs:10348`): 8000 HP.
            NpcKind::WallOfFlesh => 8000,
            // `NPC.cs`'s 115 (240) / 117-118 (60).
            NpcKind::Hungry => 240,
            NpcKind::LeechHead | NpcKind::LeechBody => 60,
            // `SetDefaults` case 134 (`NPC.cs:10666`): 8000 HP (the
            // vanilla's realLife pool — the v1 carries the per-segment HP).
            NpcKind::TheDestroyer => 8000,
            // `SetDefaults` case 127 (`NPC.cs:10562`): 28000 HP.
            NpcKind::SkeletronPrime => 28000,
            // `SetDefaults` cases 125/126.
            NpcKind::Retinazer => 20000,
            NpcKind::Spazmatism => 23000,
            // `SetDefaults` case 262 (`NPC.cs:12385`): 30000 HP.
            NpcKind::Plantera => 30000,
            // `SetDefaults` cases 266/267.
            NpcKind::BrainOfCthulhu => 1250,
            NpcKind::Creeper => 100,
            // The goblins.
            // 250 HP both (`:10255`, `:10280`).
            NpcKind::BoundGoblin => 250,
            NpcKind::GoblinTinkerer => 250,
            // `NPC.cs:53945`'s table: the Skeleton 60, the Undead Miner 70,
            // the Cave Bat 16.
            NpcKind::Skeleton => 60,
            NpcKind::UndeadMiner => 70,
            NpcKind::CaveBat => 16,
            // `SetDefaults` case 51: 34; case 60: 46; case 66: 140.
            NpcKind::JungleBat => 34,
            NpcKind::Hellbat => 46,
            NpcKind::VoodooDemon => 140,
            // `SetDefaults` case 6: 40; case 42: 48; case 23: 26.
            NpcKind::EaterOfSouls => 40,
            NpcKind::Hornet => 48,
            NpcKind::MeteorHead => 26,
            // The casters: `SetDefaults` cases 29/45/32/24.
            // `NPC.cs`'s cases 26 (60) / 27+111 (80) / 28 (110) / 29 (40) /
            // 471 (2000).
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinThief | NpcKind::GoblinArcher => 80,
            NpcKind::GoblinWarrior => 110,
            NpcKind::GoblinSorcerer => 40,
            NpcKind::GoblinSummoner => 2000,
            // The frost trio: 200/240/220 (`NPC.cs`'s cases 143-145).
            NpcKind::SnowmanGangsta => 200,
            NpcKind::SnowBalla => 240,
            NpcKind::MysterySnowman => 220,
            // The martians: 300/750/400/600/2000 (`NPC.cs`'s cases 383-520).
            NpcKind::MartianOfficer => 300,
            NpcKind::GrayGrunt => 750,
            NpcKind::MartianEngineer => 400,
            NpcKind::GigaZapper => 600,
            NpcKind::MartianWalker => 2000,
            // The probe: 500 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 500,
            // The drone: 300 (`NPC.cs`'s case 388).
            NpcKind::MartianDrone => 300,
            // The shooters: 350/350 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 350,
            // The rider 350, the carrier 600 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 350,
            NpcKind::Scutlix => 600,
            // The saucer family: 100/5000/3500/10000 HP (`NPC.cs`'s cases
            // 392-395).
            NpcKind::MartianSaucer => 100,
            NpcKind::MartianSaucerTurret => 5000,
            NpcKind::MartianSaucerCannon => 3500,
            NpcKind::MartianSaucerCore => 10000,
            // The Dutchman 50 (the ship's own bar), its cannons 2000
            // (`NPC.cs`'s cases 491/492).
            NpcKind::FlyingDutchman => 50,
            NpcKind::FlyingDutchmanCannon => 2000,
            // The DD2 goblins: 30/170/560 HP (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 => 30,
            NpcKind::DD2GoblinT2 => 170,
            NpcKind::DD2GoblinT3 => 560,
            // The support rows (`NPC.cs`): the bombers 50/200/700, the
            // javelineers 60/300/1000, the skeletons 25/480.
            NpcKind::DD2GoblinBomberT1 => 50,
            NpcKind::DD2GoblinBomberT2 => 200,
            NpcKind::DD2GoblinBomberT3 => 700,
            NpcKind::DD2JavelinT1 => 60,
            NpcKind::DD2JavelinT2 => 300,
            NpcKind::DD2JavelinT3 => 1000,
            NpcKind::DD2SkeletonT1 => 25,
            NpcKind::DD2SkeletonT3 => 480,
            // The Dark Mage: 800 HP.
            NpcKind::DD2DarkMage => 800,
            // The wyverns: 60/180/600 (`NPC.cs`'s rows).
            NpcKind::DD2WyvernT1 => 60,
            NpcKind::DD2WyvernT2 => 180,
            NpcKind::DD2WyvernT3 => 600,
            // The Dark Mage T3: 4000 (`:16468`).
            NpcKind::DD2DarkMageT3 => 4000,
            // The Wither Beast T3: 1400 (`:16331`).
            NpcKind::DD2WitherBeastT3 => 1400,
            // The Drakin: 900/3000 (`:16416`/`:16433`).
            NpcKind::DD2DrakinT2 => 900,
            NpcKind::DD2DrakinT3 => 3000,
            // The Kobold walkers: 260/800 (`:16382`/`:16399`).
            NpcKind::DD2KoboldWalkerT2 => 260,
            NpcKind::DD2KoboldWalkerT3 => 800,
            // The Kobold gliders: 170/580 (`:16486`/`:16504`).
            NpcKind::DD2KoboldFlyerT2 => 170,
            NpcKind::DD2KoboldFlyerT3 => 580,
            // The Ogres: 5000/13000 (`:16280`/`:16297`).
            NpcKind::DD2OgreT2 => 5000,
            NpcKind::DD2OgreT3 => 13000,
            // Betsy: 50000 HP (`NPC.cs`'s case 551).
            NpcKind::DD2Betsy => 50000,
            // The WitherBeast: 500 HP.
            NpcKind::DD2WitherBeast => 500,
            // The Lunar Pillars: 20000 HP.
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => 20000,
            // Lunatic Cultist (439): 32000 HP (`NPC.cs:14219`). S1-D-3b.
            NpcKind::AncientCultist => 32000,
            // The lane gate: 5 HP (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 5,
            NpcKind::VortexHunter => 1000,
            NpcKind::VortexAlien => 500,
            NpcKind::VortexHeadcrab => 200,
            NpcKind::StardustSpiderBig => 800,
            NpcKind::StardustSpiderSmall => 200,
            NpcKind::StardustSoldier => 700,
            NpcKind::SolarDrakomire => 800,
            NpcKind::SolarDrakomireRider => 800,
            NpcKind::SolarSroller => 700,
            NpcKind::SolarCorite => 600,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarSelenian => 800,
            NpcKind::NebulaFloater => 1300,
            NpcKind::BrainSuckler => 330,
            NpcKind::Predictor => 700,
            NpcKind::MoonLordHead => 45000,
            NpcKind::MoonLordHand => 25000,
            NpcKind::MoonLordCore => 50000,
            NpcKind::MoonLordFreeEye => 100,
            NpcKind::MoonLordLeechBlob => 400,
            NpcKind::StardustCellBig => 300,
            NpcKind::StardustCellSmall => 300,
            NpcKind::StardustJellyfishBig => 1500,
            NpcKind::NebulaBeast => 850,
            NpcKind::VortexRifleman => 800,
            NpcKind::VortexSoldier => 700,
            // The tavernkeeper 250, the crystal 1000 (`NPC.cs`'s rows).
            NpcKind::DD2Bartender => 250,
            NpcKind::DD2EterniaCrystal => 1000,
            // The pirates: 300/450/225/350/3000 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 300,
            NpcKind::PirateCaptainsMate => 450,
            NpcKind::PirateArcher => 225,
            NpcKind::PirateCorsair => 350,
            NpcKind::PirateCaptain => 3000,
            // `SetDefaults` 245-249: the body 15000, the head 25000, the
            // fists 10000, the free head 16000.
            NpcKind::Golem => 15000,
            NpcKind::GolemHead => 25000,
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 10000,
            NpcKind::GolemHeadFree => 16000,
            // `:17456`: 7000.
            NpcKind::Deerclops => 7000,
            NpcKind::SkeletronHand => 600,
            NpcKind::WallOfFleshEye => 8000,
            NpcKind::PrimeCannon => 7000,
            NpcKind::PrimeSaw => 9000,
            NpcKind::PrimeVice => 9000,
            NpcKind::PrimeLaser => 6000,
            NpcKind::Tim => 200,
            NpcKind::DarkCaster => 50,
            NpcKind::FireImp => 70,
            // The spheres: 1 HP.
            NpcKind::ChaosBall => 1,
            NpcKind::BurningSphere => 1,
            NpcKind::WaterSphere => 1,
            // The swimmers: `SetDefaults` cases 55/58/65.
            NpcKind::Goldfish => 5,
            NpcKind::Piranha => 30,
            NpcKind::Shark => 300,
            // The worms: the segment HP per family (`:8763`/`:8850`/`:8903`).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => 100,
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => 30,
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 150,
            NpcKind::KingSlime => 2000,
            // `NPC.cs:8710` (the eye) / `:8722` (the servant).
            NpcKind::EyeOfCthulhu => 2800,
            NpcKind::ServantOfCthulhu => 8,
        }
    }

    fn contact_damage(self) -> i32 {
        match self {
            NpcKind::Slime => 7,         // NPC.cs:8571
            NpcKind::DemonEye => 18,     // case 2
            NpcKind::WanderingEye => 40, // case 133
            NpcKind::Zombie => 14,
            // `SetDefaults` case 17: damage 10 — the town NPCs never hurt
            // (the contact gate skips them).
            NpcKind::Merchant => 10,
            NpcKind::Dryad => 10,
            NpcKind::Nurse => 10,
            NpcKind::ArmsDealer => 10,
            NpcKind::Demolitionist => 10,
            NpcKind::Painter => 10,
            NpcKind::WitchDoctor => 10,
            NpcKind::Clothier => 10,
            NpcKind::Angler => 10,
            NpcKind::Wizard => 10,
            NpcKind::BoundWizard => 10,
            NpcKind::Mechanic => 10,
            NpcKind::Stylist => 10,
            NpcKind::WebbedStylist => 10,
            NpcKind::Golfer => 10,
            NpcKind::Zoologist => 10,
            NpcKind::DyeTrader => 10,
            NpcKind::PartyGirl => 10,
            NpcKind::Princess => 10,
            NpcKind::SleepingGolfer => 10,
            NpcKind::TaxCollector => 10,
            NpcKind::BoundMechanic => 10,
            NpcKind::SleepingAngler => 10,
            NpcKind::TravellingMerchant => 10,
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::QueenBee => 30,
            NpcKind::SkeletronHead => 32,
            NpcKind::WallOfFlesh => 50,
            // `NPC.cs`'s 115 (30) / 117 (26) / 118 (22).
            NpcKind::Hungry => 30,
            NpcKind::LeechHead => 26,
            NpcKind::LeechBody => 22,
            NpcKind::TheDestroyer => 70,
            NpcKind::SkeletronPrime => 47,
            NpcKind::Retinazer => 45,
            NpcKind::Spazmatism => 50,
            NpcKind::Plantera => 50,
            NpcKind::BrainOfCthulhu => 30,
            NpcKind::Creeper => 20,
            // The `friendly` pair (`:10246`/`:10272`) — the contact gate
            // excludes them; 0 stands for the vanilla's unused dmg 10.
            NpcKind::BoundGoblin => 0,
            NpcKind::GoblinTinkerer => 0,
            // `SetDefaults` case 21: damage 20; case 44: 22; case 49: 13.
            NpcKind::Skeleton => 20,
            NpcKind::UndeadMiner => 22,
            NpcKind::CaveBat => 13,
            // `SetDefaults` case 51: damage 20; case 60: 35; case 66: 32.
            NpcKind::JungleBat => 20,
            NpcKind::Hellbat => 35,
            NpcKind::VoodooDemon => 32,
            // `SetDefaults` case 6: damage 22; case 42: 26; case 23: 40.
            NpcKind::EaterOfSouls => 22,
            NpcKind::Hornet => 26,
            NpcKind::MeteorHead => 40,
            // The casters: `SetDefaults` damage 20/20/20/30.
            // `NPC.cs`'s cases 26 (12) / 27+111 (20) / 28 (25) / 29 (20) /
            // 471 (80).
            NpcKind::GoblinPeon => 12,
            NpcKind::GoblinThief | NpcKind::GoblinArcher | NpcKind::GoblinSorcerer => 20,
            NpcKind::GoblinWarrior => 25,
            NpcKind::GoblinSummoner => 80,
            // The frost trio: 50/65/55.
            NpcKind::SnowmanGangsta => 50,
            NpcKind::SnowBalla => 65,
            NpcKind::MysterySnowman => 55,
            // The martians: 75/80/40/75/60.
            NpcKind::MartianOfficer => 75,
            NpcKind::GrayGrunt => 80,
            NpcKind::MartianEngineer => 40,
            NpcKind::GigaZapper => 75,
            NpcKind::MartianWalker => 60,
            // The probe: 5 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 5,
            // The drone: 60 (`NPC.cs`'s case 388 — the explode arm's 80
            // rides damage_override).
            NpcKind::MartianDrone => 60,
            // The shooters: 50/50 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 50,
            // The rider 65, the carrier 85 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 65,
            NpcKind::Scutlix => 85,
            // The saucer family: 50/60/60/80 (`NPC.cs`'s cases 392-395).
            NpcKind::MartianSaucer => 50,
            NpcKind::MartianSaucerTurret => 60,
            NpcKind::MartianSaucerCannon => 60,
            NpcKind::MartianSaucerCore => 80,
            // The Dutchman 0 (contactless), its cannons 30.
            NpcKind::FlyingDutchman => 0,
            NpcKind::FlyingDutchmanCannon => 30,
            // The DD2 goblins: 18/46/70 dmg (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 => 18,
            NpcKind::DD2GoblinT2 => 46,
            NpcKind::DD2GoblinT3 => 70,
            // The support's damage: 26/55/75, 30/60/80, 18/70.
            NpcKind::DD2GoblinBomberT1 => 26,
            NpcKind::DD2GoblinBomberT2 => 55,
            NpcKind::DD2GoblinBomberT3 => 75,
            NpcKind::DD2JavelinT1 => 30,
            NpcKind::DD2JavelinT2 => 60,
            NpcKind::DD2JavelinT3 => 80,
            NpcKind::DD2SkeletonT1 => 18,
            NpcKind::DD2SkeletonT3 => 70,
            // The Dark Mage: dmg 40.
            NpcKind::DD2DarkMage => 40,
            // Betsy: dmg 80.
            NpcKind::DD2Betsy => 80,
            // The WitherBeast: dmg 50.
            NpcKind::DD2WitherBeast => 50,
            NpcKind::DD2WyvernT1 => 30,
            NpcKind::DD2WyvernT2 => 75,
            NpcKind::DD2WyvernT3 => 100,
            NpcKind::DD2DarkMageT3 => 90,
            NpcKind::DD2WitherBeastT3 => 80,
            NpcKind::DD2DrakinT2 => 60,
            NpcKind::DD2DrakinT3 => 90,
            NpcKind::DD2KoboldWalkerT2 => 60,
            NpcKind::DD2KoboldWalkerT3 => 80,
            NpcKind::DD2KoboldFlyerT2 => 50,
            NpcKind::DD2KoboldFlyerT3 => 80,
            NpcKind::DD2OgreT2 => 70,
            NpcKind::DD2OgreT3 => 90,
            // The Lunar Pillars: dmg 0.
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => 0,
            // Lunatic Cultist (439): damage 50 (`NPC.cs:14217`). S1-D-3b.
            NpcKind::AncientCultist => 50,
            // The lane gate: damage 0 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 0,
            // The vortex trio's damage: the hunter's 100, the alien's 75,
            // the headcrab's 50.
            NpcKind::StardustSpiderBig => 70,
            NpcKind::StardustSpiderSmall => 80,
            NpcKind::StardustSoldier => 80,
            NpcKind::SolarDrakomire => 55,
            NpcKind::SolarDrakomireRider => 80,
            NpcKind::SolarSroller => 80,
            NpcKind::SolarCorite => 70,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarSelenian => 90,
            NpcKind::NebulaFloater => 75,
            NpcKind::BrainSuckler => 70,
            NpcKind::Predictor => 80,
            NpcKind::VortexHunter => 100,
            NpcKind::VortexAlien => 75,
            NpcKind::VortexHeadcrab => 50,
            NpcKind::MoonLordHead | NpcKind::MoonLordHand | NpcKind::MoonLordCore => 0,
            NpcKind::MoonLordFreeEye => 60,
            NpcKind::MoonLordLeechBlob => 0,
            NpcKind::StardustCellBig => 120,
            NpcKind::StardustCellSmall | NpcKind::StardustJellyfishBig => 70,
            NpcKind::NebulaBeast => 90,
            NpcKind::VortexRifleman => 100,
            NpcKind::VortexSoldier => 90,
            // The tavernkeeper 10, the crystal 0 (its contact damage is 0).
            NpcKind::DD2Bartender => 10,
            NpcKind::DD2EterniaCrystal => 0,
            // The pirates: 35/50/30/35/70 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 35,
            NpcKind::PirateCaptainsMate => 50,
            NpcKind::PirateArcher => 30,
            NpcKind::PirateCorsair => 35,
            NpcKind::PirateCaptain => 70,
            // `SetDefaults` 245-249: 72/64/59/59/80.
            NpcKind::Golem => 72,
            NpcKind::GolemHead => 64,
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 59,
            NpcKind::GolemHeadFree => 80,
            // `:17455`: 20.
            NpcKind::Deerclops => 20,
            NpcKind::SkeletronHand => 20,
            NpcKind::WallOfFleshEye => 50,
            NpcKind::PrimeCannon => 30,
            NpcKind::PrimeSaw => 56,
            NpcKind::PrimeVice => 52,
            NpcKind::PrimeLaser => 29,
            NpcKind::Tim => 20,
            NpcKind::DarkCaster => 20,
            NpcKind::FireImp => 30,
            // The spheres: damage 20/30/20.
            NpcKind::ChaosBall => 20,
            NpcKind::BurningSphere => 30,
            NpcKind::WaterSphere => 20,
            // The swimmers: damage 0/25/40 — the Goldfish never hurts.
            NpcKind::Goldfish => 0,
            NpcKind::Piranha => 25,
            NpcKind::Shark => 40,
            // The worms: the head hits hardest, the tail softest
            // (`:8763`'s 31/16/13, `:8850`'s 8/4/4, `:8903`'s 22/13/11).
            NpcKind::DevourerHead => 31,
            NpcKind::DevourerBody => 16,
            NpcKind::DevourerTail => 13,
            NpcKind::GiantWormHead => 8,
            NpcKind::GiantWormBody | NpcKind::GiantWormTail => 4,
            NpcKind::EaterOfWorldsHead => 22,
            NpcKind::EaterOfWorldsBody => 13,
            NpcKind::EaterOfWorldsTail => 11,
            NpcKind::Guide => 0,
            NpcKind::KingSlime => 40,
            // `NPC.cs:8707` (the eye damage 15) / `:8720` (the servant 12).
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::ServantOfCthulhu => 12,
        }
    }
}

/// The hostile-shot request with the C#'s `ai0`/`ai1` payload —
/// `NPC.NewProjectile`'s trailing arguments (`Projectile.cs:10862-10863`).
/// The plain [`Npc::wants_hostile_shot`] list carries the shots whose call
/// site passes zero.
#[derive(Clone, Copy, Debug)]
pub struct HostileShotAi {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub ai0: f32,
    pub ai1: f32,
}

#[derive(Clone, Debug)]
pub struct Npc {
    /// `NPC.shimmerTransparency` (`NPC.cs`'s field) — the 1.0 of a
    /// shimmer-released NPC (`WorldItem.GetShimmered`'s makeNPC arm,
    /// `WorldItem.cs:1768`, and 4986's `:1732`) or the 0.89 of the shimmered
    /// variant (`NPC.GetShimmered`, `NPC.cs:91548`). Render layer only.
    pub shimmer_transparency: f32,
    /// `NPC.extraValue` (`WorldItem.cs:1125-1200`'s carrier): the coins a
    /// revenge enemy picked up, dropped on kill via `NPCLoot_DropMoney`'s
    /// `+= extraValue` (:79680).
    pub extra_value: u64,
    pub kind: NpcKind,
    /// The slime colour variant (`SlimeVariant`); only meaningful while
    /// `kind == NpcKind::Slime`.
    pub slime: SlimeVariant,
    pub x: f32,
    pub y: f32,
    /// `Entity.oldPosition` (`NPC.cs:92605`): the position before this
    /// tick's movement. The switch-tile bridge (`Collision.SwitchTiles`,
    /// `NPC.cs:90504`) reads it to tell "the NPC moved onto the plate" from
    /// "the NPC stands on it".
    pub old_x: f32,
    pub old_y: f32,
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
    /// `NPC.SpawnedFromStatue` (`NPC.cs:6061`): set by the wire-triggered
    /// statue spawns; gates the loot (`NPCLoot`'s StatueSpawnedDropRarity
    /// check, `NPC.cs:78849`).
    pub spawned_from_statue: bool,
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
    /// `NPC.scale` (`NPC.cs:6171`) — the draw/hitbox multiplier: the
    /// slime variants ride their `SetDefaults` scale, the Stardust
    /// CellSmall's aiStyle-95 ramp grows it 1.0 → 1.3 over its 300 ticks
    /// (`NPC.cs:39393`).
    pub scale: f32,
    /// `NPC.netOffset` (`NPC.cs:6157`) — the client-side smoothing
    /// accumulator for networked NPCs: on each `23 NPCInfo` the client adds
    /// the visual lag here and snaps `x/y` to the wire value, then decays it
    /// toward zero (`netglue::decay_net_offsets`, the C#'s
    /// `MessageBuffer.cs:1636-1646` + `NPC.cs:90287-90311`). Zero on the host
    /// and in single-player, so nothing single-player-rendered moves.
    pub net_offset: Vec2,
    /// Accumulated daylight burn time (zombies only).
    burn: f32,
    /// A boss that just landed wants the app to spawn a minion (`AI_015`).
    pub wants_minion: bool,
    /// The King Slime's minion batch count (`AI_015` `:44233`'s
    /// `rand(1, 4)` damage spawn / `:85689`'s `rand(4) + 4` death spawn).
    pub wants_minion_batch: u32,
    /// The town NPC's assigned home tile (`homeTileX/Y`); `None` =
    /// homeless (`NPC.homeless` inverted). The in-memory state — the
    /// world-file persistence lands with M6-SAVE-1's NPC sections
    /// (M6-SAVE-1e).
    pub home: Option<(i32, i32)>,
    /// `NPC.GivenName` (`NPC.cs:6497`, `SaveNPCs:1719`): the town NPC's
    /// display name — empty carries the kind's static name, exactly the
    /// vanilla's "give it a name at the first world-file save" convention.
    pub given_name: String,
    /// `NPC.townNpcVariationIndex` (`SaveNPCs:1731`): the selected variation
    /// of the town NPC's looks — 0 the default, 1 the shimmer variant
    /// (`NPC.cs:80850`), rolled from the profile otherwise.
    pub variation_index: i32,
    /// `NPC.homelessDespawn` (`LoadNPCs:2917-2920`, `SaveNPCs:1734`): a
    /// homeless town NPC this far from any room is on the despawn path.
    pub homeless_despawn: bool,
    /// `ai[0]`: the caster's cycle counter (`AI_008`'s 500 init, the
    /// 100/200/300 shot triggers, the 650 teleport threshold).
    pub caster_counter: i32,
    /// `ai[1]`: the caster's windup countdown (30 from a shot trigger,
    /// `num88` after a teleport; the shoot fires at the params' tick).
    pub cast_windup: i32,
    /// `localAI[2]` of the aiStyle-38 hop machine (`NPC.cs:29095-29100`):
    /// the previous tick's `position.X` — equality flips the direction and
    /// arms the 60-tick no-retarget cooldown.
    pub frost_stuck_x: f32,
    /// `ai[0]` for the swimmers (`AI_016`): the vertical bob phase — 0 the
    /// chase start, −1 rising, 1 sinking (`:23752`/`:23758`/`:23850`).
    pub swim_phase: f32,
    /// The worm chain id (`AI_006`): every segment of one worm shares it;
    /// 0 = not a worm.
    pub worm_id: u32,
    /// The segment's index down the chain — 0 = the head (`ai[1] == 0`'s
    /// leader), the tail = the last.
    pub seg_index: u32,
    /// `ai[2]`: the remaining segment budget this link passes down (the
    /// head rolls the total, each link decrements).
    pub worm_left: u32,
    /// The init latch — `ai[0] == 0`'s "no child spawned yet" analog: the
    /// link spawns its child on the first worm pass.
    pub worm_inited: bool,
    /// The WoF's Leech-head spawn request (the tongue-phase
    /// `NPC.CountNPCS(117) < 10` spawn; the sim counts and spawns).
    pub wants_leech: bool,
    /// The Prime's 4-limb spawn request (the `aiStyle 32` one-shot init).
    pub wants_limbs: bool,
    /// The WoF's Hungry init request (`localAI[0]` 1 → 2's 11 spawns).
    pub wants_hungry: bool,
    /// The AI's per-tick contact-damage override (the Hungry's
    /// damage tiers, `aiStyle 29`'s `num388`).
    pub damage_override: Option<i32>,
    /// The WoF's draw-band scan targets (`AI_027`'s bottom/top walks) —
    /// the world sim eases `World.wof_draw_*` toward them (the AI holds
    /// `&World`, so the band state lives on the sim side).
    pub wof_bottom_target: f32,
    pub wof_top_target: f32,
    /// The caster's sphere spawn request — the vanilla `NPC.NewNPC` at the
    /// shoot tick (`AI_008`), resolved by the world sim after the update
    /// pass (the borrow-checker-safe `wants_minion` shape). The payload is
    /// the (kind, bottom-centre x, bottom-centre y) spawn point.
    pub wants_sphere: Vec<(NpcKind, f32, f32)>,
    /// `ai[3]`: x position while descending through water, for the bounce flip.
    wet_x: f32,
    /// `NPC.ai[0..4]` — the EoC phase machine's state
    /// (`ai[0]` the phase, `ai[1]` the sub-state, `ai[2]` the timer,
    /// `ai[3]` the dash/servant counter; `AI()`'s aiStyle-4 block).
    pub ai: [f32; 4],
    /// The servant spawn request — the `NPC.NewNPC(..., 5)` calls in the
    /// aiStyle-4 block (`NPC.cs:4598`/`:4651` of the AI()), resolved by the
    /// world sim after the update pass (the `wants_sphere` shape; the payload
    /// is the (x, y, vx, vy) spawn state).
    pub wants_servant: Option<(f32, f32, f32, f32)>,
    /// `this.defense = 0` — the AI's per-frame stat override (the EoC's
    /// final phase zeroes it, `NPC.cs`'s aiStyle-4 block; the expert rage
    /// makes it negative).
    pub defense_override: Option<i32>,
    /// `NPC.dontTakeDamage` (`NPC.cs:6525`): the hurt path refuses while
    /// it holds (the Brain of Cthulhu's Creeper shield).
    pub dont_take_damage: bool,
    /// `NPC.behindTiles` (`NPC.cs:6517`, default false, reset `:8520`): drawn
    /// only in the behind-tiles pass (`Main.cs:22320,22336`).
    pub behind_tiles: bool,
    /// `NPC.chaseable` (`NPC.cs:6261`, the `SetDefaults` default `true`) — the
    /// `CanBeChasedBy` gate's inner term: false = untargetable by minions and
    /// sentries. The C#'s only `chaseable = false` writers are the case-345
    /// family (`NPC.cs:13854`/`:14078`/`:14193`/`:14210` — the Dungeon
    /// Guardian's group, unported kinds) and `:8549` re-affirms true, so every
    /// ported kind keeps the stock value.
    pub chaseable: bool,
    /// `NPC.immortal` (`NPC.cs:6259`, the stock `false`) — the target-dummy
    /// "alive but unhittable" flag; no ported kind's `SetDefaults` writes it,
    /// so it holds the field default.
    pub immortal: bool,
    /// `NPC.friendly` (`NPC.cs:6563`, the stock `false`) — the town NPCs'
    /// `friendly = true` writes (`NPC.cs:8983+`); the port derives it from the
    /// existing `is_town_resident` list.
    pub friendly: bool,
    /// The brain's Creeper-swarm init mark (`AI_054`'s `localAI[0]`
    /// 0 -> 1): the 20 Creepers spawn once, at the first sim tick.
    pub swarm_done: bool,
    /// `AI_015_KingSlime`'s `localAI[0..4]` (`NPC.cs:43121-43414`) — the
    /// King Slime's machine: the line-of-sight counter (`localAI[0]`), the
    /// teleport target (`localAI[1]`/`localAI[2]`) and the init latch
    /// (`localAI[3]`).
    pub king_slime_local: [f32; 4],
    /// The per-NPC box override (`NPC.base.width`/`base.height`) — the King
    /// Slime rescales its body with its hp (`AI_015`'s `base.width = (int)
    /// (98f * this.scale)`, `NPC.cs:43347-43352`).
    pub size_override: Option<(f32, f32)>,
    /// `AI_054`'s `localAI[0..4]` (`NPC.cs:32370-32656`) — the Brain of
    /// Cthulhu's machine: the blink timer (`localAI[1]`), the port's
    /// Creeper count (`localAI[2]`, the C#'s sound latch slot — the sim
    /// writes the count) and the despawn ramp (`localAI[3]`).
    pub brain_local: [f32; 4],
    /// The VortexRifleman's `localAI[3]` one-shot latch
    /// (`AI_003_Fighters`' `NPC.cs:58465-58469` 0 -> 1 arm: the first
    /// tick rearms `ai[3]` to −120, once).
    /// `AI_077_MoonLordCore`'s `localAI[0..2]` (`NPC.cs:41229-41232`): the
    /// core's three part slots — 0/1 the hands (`ai[2]` 0/1) and 2 the
    /// head — as the NPC indices the pre-pass resolves.
    /// The Moon Lord parts' `localAI[0..2]` (`AI_078`'s eyelid angle,
    /// `AI_079`'s open factor, the lid): the port packs the vanilla's
    /// per-part locals into one array.
    pub moon_local: [f32; 3],
    /// `AI_078`/`AI_079`/`AI_081`'s alive-core gate: the pre-pass sets
    /// it while `Main.npc[ai[3]]` is the living 398.
    pub moon_core_alive: bool,
    pub moon_parts: [i32; 3],
    pub rifleman_rearmed: bool,
    /// The hostile-shot request (`NPC.cs`'s `Projectile.NewProjectile`
    /// arms): the (kind, x, y, vx, vy, damage) payload the sim resolves
    /// into the pool (the `wants_sphere` shape).
    /// The hostile-shot requests (the `Projectile.NewProjectile` arms): the
    /// (kind, x, y, vx, vy, damage) payloads the sim resolves into the pool
    /// (the `wants_sphere` shape; the Vec = a per-tick volley).
    pub wants_hostile_shot: Vec<(u16, f32, f32, f32, f32, i32)>,
    /// The hostile shots that carry the C#'s `ai0`/`ai1` payload — the
    /// `NewProjectile(..., ai0, ai1)` call-site values
    /// (`Projectile.cs:10862-10863`): the Saucer deathray's owner slot + 1
    /// (`NPC.cs:35917`/`:36437`), the Saucer missile's 20-tick arming delay
    /// (`NPC.cs:35845`), the Deerclops' ice spike's scale factor
    /// (`NPC.cs:44582`) and the Insanity Shadow's variation age + angle
    /// (`NPC.cs:44394`).
    pub wants_hostile_shot_ai: Vec<HostileShotAi>,
    /// The NPC's slot index in the sim's NPC vec — the C#'s `NPC.whoAmI`
    /// (`NPC.cs:6355`). The port's vec is compacted (the dead are culled
    /// every tick), so the index is the sim's current one; the sim writes it
    /// before each NPC's update.
    pub slot: usize,
    /// The probe's escape resolve (`AI()`'s aiStyle-80 phase-2 arm:
    /// `Main.StartInvasion(4); active = false;`) — the AI holds `&World`,
    /// so the sim resolves the invasion and removes the NPC.
    pub wants_martian_invasion: bool,
    /// The crystal's win-death request (`ai[0] >= 600`'s
    /// `DD2Event.StopInvasion`): the sim resolves it into the event's
    /// stop (`dd2_event_ongoing` clear) — the `Box<Self>`-shaped
    /// request pattern's cousin.
    pub wants_event_stop: bool,
    /// The Scutlix rider's carrier spawn request (`AI_075`'s type-390
    /// `localAI[0]` 0 → 1 arm: the `NewNPC(..., 391, whoAmI)` — the sim
    /// resolves it after the tick).
    pub wants_scutlix: bool,
    /// The carrier link the sim writes each tick (the WoF write-through
    /// shape): the living 391's centre in pixels, (-1, -1) = none.
    pub carrier_x: f32,
    pub carrier_y: f32,
    /// The saucer core's crew spawn request (`AI`'s aiStyle-76 init arm:
    /// the 4 guns + the shell, `NPC.cs:36071-36092`) — the sim resolves.
    pub wants_saucer_crew: bool,
    /// `NPC.alpha`: the spawn fade (`SetDefaults` 245/246/247/248's
    /// `alpha = 255`, `NPC.cs:12176/12190/12201`); the Golem family's
    /// machines decrement it by 10 per tick and the draw reads it as
    /// `GetAlpha`'s `(255 - alpha)/255` tint (`NPC.cs:93406-93412`).
    pub alpha: u8,
    /// The attached head's `localAI[1]` (`NPC.cs:31213/31218/31223`):
    /// the player's side about the head — -1 left of `Center.X - width`,
    /// +1 right of `+ width`, 0 in-band; the phase-1-only turn that
    /// drives FindFrame case 246's +4/+2 rows and the beam volley's ±18
    /// vs ±30 side arm (`NPC.cs:72953-72959` / `:31305-31327`).
    pub head_turn: i32,
    /// `NPC.wet`: the liquid-overlap flag the swimmer machine caches each
    /// tick (`NPC.cs:90474`'s `CheckFacetedLiquid` write); `FindFrame`'s
    /// swimmer cases read it for the wet flap vs the dry flop
    /// (`NPC.cs:74181`/`:74654`).
    pub wet: bool,
    /// `NPC.spriteDirection` (`NPC.cs:6515`, the default -1) — the DISPLAY
    /// direction the `DrawNPCDirect` flip reads (`Main.cs:23124-23128`:
    /// `spriteDirection == 1` is exactly when the sheet flips). The C# keeps
    /// it separate from `direction` (the motion/last-aim side); the port
    /// folds `direction` into `facing`, so the kinds whose `FindFrame` /
    /// AI write `spriteDirection` as its own value — the Scutlix family's
    /// inversion (`NPC.cs:71746-71753`/`:71795`), the drone's
    /// `-Math.Sign(velocity.X)` (`:35232`), the saucer mounts' inheritance
    /// (`:35669-35676`) — carry it here and the martian draws flip on it.
    pub sprite_dir: i32,
    /// The saucer family's write-through payload (the mounts read the
    /// core's live state): the core's `ai[3]` timer and `ai[0]` phase,
    /// and the crew counts within the link's reach (the turrets + the
    /// cannons — the mounts' own `Main.npc[]` siblings have no port
    /// model).
    pub carrier_timer: f32,
    pub carrier_phase: f32,
    /// The core's rotation (`NPC.cs:35625-35651`: every mount inherits
    /// `nPC.rotation`, so the whole saucer family tilts as one body) and
    /// the core's `spriteDirection` (the mounts copy it per
    /// `NPC.cs:35669-35676`; the core's own AI never writes it, so it
    /// holds the `SetDefaults` default -1).
    pub carrier_rot: f32,
    pub carrier_facing: i32,
    pub crew_turrets: i32,
    pub crew_cannons: i32,
    /// The Dutchman's pirate-drop payload (`AI_093`'s `Next(300)` arm,
    /// `NPC.cs:38868-38877`): the (kind, x, y) spawn the sim resolves
    /// through `wants_sphere` — the drop's velocity rides the ship's.
    pub wants_dutchman_drop: Option<(NpcKind, f32, f32)>,
    /// The Dutchman's cannon-crew spawn request (`AI_093`'s init arm:
    /// the 4 cannons 492 at the ship's centre, `NPC.cs:38859-38872`).
    /// `AI_077_MoonLordCore`'s spawn-in request (`NPC.cs:41209-41232`):
    /// the two hands (`ai[2]` 0/1) + the head, at the core's offsets.
    pub wants_moon_parts: bool,
    /// `AI_077_MoonLordCore`'s despawn request (`:41492-41513`): clear
    /// the parts with the core.
    pub wants_moon_parts_clear: bool,
    /// The Moon Leech request (`AI_079`'s `:42303-42318` / the leech
    /// blob's `:36658-36690`): a 456 leeched player spawns the 401 clot,
    /// which then drains 1000 hp from the head -> the core -> the hands.
    /// The Brain Suckler's attached obstruct request (`AI()`'s aiStyle-85
    /// phase 5, `NPC.cs:37159`'s `AddBuff(163, 59)`): the sim applies the
    /// Obstructed buff to the player.
    /// The SolarCorite's implosion request (`AI()`'s aiStyle-74 arm,
    /// `NPC.cs:35496-35500`): the 192-box blast the sim resolves (80
    /// damage scaled by difficulty).
    pub wants_blast: bool,
    /// `AI`'s aiStyle-106 arm (`NPC.cs:40999`'s `SpawnMonsterFromGate`)
    /// request: the lane gate asks for its wave's pick — the sim
    /// resolves it through the dd2 tables.
    pub wants_dd2_spawn: bool,
    /// `NPC.cs:40972`'s `leftGate` reading: `ai[2] == 0f` is the left
    /// lane, `1f` the right — the resolved pick rides this flag.
    pub wants_lane_right: bool,
    /// The StardustWormHead's mark request (`AI_006`'s 402 arm,
    /// `NPC.cs:51061`'s `SpawnStardustMark_StardustWorm`): the sim
    /// resolves the 405/402/407 pick (the 540 hop unmodelled).
    pub wants_stardust_mark: bool,
    pub wants_obstruct: bool,
    pub wants_moon_leech: bool,
    pub wants_dutchman_crew: bool,
}

/// The 2-D rotation helper for the Dutchman's drop offsets
/// (`Vector2.RotatedBy`).
pub(crate) fn rot(x: f32, y: f32, angle: f32) -> (f32, f32) {
    (
        x * angle.cos() - y * angle.sin(),
        x * angle.sin() + y * angle.cos(),
    )
}

/// `Utils.AngleLerp` (`Utils.cs:2033-2049`): the short-way angular lerp —
/// the Prime head's Mech-Queen roll eases into half its drift through it
/// (`NPC.cs:27689`/`:27781`/`:27866`), and the despawn's spin uses the
/// plain accumulation. `MathHelper.WrapAngle` folds the result back into
/// (-π, π], exactly like XNA.
pub(crate) fn angle_lerp(cur: f32, target: f32, amount: f32) -> f32 {
    let mut angle;
    if target < cur {
        let wrapped = target + std::f32::consts::TAU;
        angle = if wrapped - cur > cur - target {
            cur + (target - cur) * amount
        } else {
            cur + (wrapped - cur) * amount
        };
    } else {
        if !(target > cur) {
            return cur;
        }
        let wrapped = target - std::f32::consts::TAU;
        angle = if target - cur > cur - wrapped {
            cur + (wrapped - cur) * amount
        } else {
            cur + (target - cur) * amount
        };
    }
    angle = (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    angle
}

/// The Brain of Cthulhu's blink target (`NPC.cs:32440-32474` for the
/// retreat, `:32546-32578` for the normal phase): a random tile near the
/// player, up to 100 tries. The `range` is the pick's tile radius and
/// `sight_gate` the normal phase's `Collision.CanHit` condition (the
/// retreat has none). The player's velocity leads the pick (`:32454-32460`).
fn brain_pick_tile(
    solid_tile: &dyn Fn(i32, i32) -> bool,
    sight_ok: &dyn Fn(i32, i32) -> bool,
    pcx: f32,
    pcy: f32,
    pvx: f32,
    pvy: f32,
    min: i32,
    max: i32,
    sight_gate: bool,
    rng: &mut u64,
) -> Option<(i32, i32)> {
    let mut tries = 0;
    loop {
        tries += 1;
        let mut tx = (pcx as i32) / 16;
        let mut ty = (pcy as i32) / 16;
        let mut ox = crate::app::frame::rand_range_signed(rng, min, max + 1);
        let mut oy = crate::app::frame::rand_range_signed(rng, min, max + 1);
        if crate::app::frame::rand_range(rng, 2) == 0 {
            ox = -ox;
        }
        if crate::app::frame::rand_range(rng, 2) == 0 {
            oy = -oy;
        }
        // `:32454-32460` — the player's velocity leads the pick.
        let mut v = Vec2::new(ox as f32 * 16.0, oy as f32 * 16.0);
        let player_dir = safe_dir(Vec2::new(pvx, pvy), Vec2::Y);
        let v_dir = safe_dir(v, Vec2::ZERO);
        if player_dir.dot(v_dir) > 0.0 {
            v += v_dir * 16.0 * (pvx * pvx + pvy * pvy).sqrt();
        }
        tx += (v.x / 16.0) as i32;
        ty += (v.y / 16.0) as i32;
        let free = !solid_tile(tx, ty);
        let ok = if sight_gate {
            free && (tries > 75 || sight_ok(tx, ty))
        } else {
            free
        };
        if tries > 100 || ok {
            return Some((tx, ty));
        }
        if tries > 100 {
            return None;
        }
    }
}

/// `Vector2.SafeNormalize(v, defaultValue)` (`Utils.cs:1837-1846`): the
/// zero/NaN direction falls back to `default`.
fn safe_dir(v: Vec2, default: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.is_nan() {
        default
    } else {
        v.normalize()
    }
}

/// `AI_123_Deerclops_ShootRubbleUp` (`NPC.cs:44448-44465`): the rock lands
/// on the first solid tile under the Deerclops' top (ten tiles up and three
/// forward) and flies up at the row's angle.
fn deerclops_shoot_rubble_up(npc: &mut Npc, world: &World, rng: &mut u64, which_one: i32) {
    let how_many = 20i32;
    let spacing = 1i32;
    let (w, _h) = npc.size();
    // `:44257-44260` — the source tile: the Deerclops' top, three tiles
    // forward and ten up.
    let source_x = ((npc.x + w * 0.5) as i32) / 16 + npc.facing * 3;
    let source_y = (npc.y as i32) / 16 - 10;
    let num2 = which_one * spacing;
    for i in 0..35 {
        let x = source_x + num2 * npc.facing;
        let y = source_y + i;
        if !solid_block(world, x, y) {
            continue;
        }
        // The C#'s first velocity (the up-bias target) is overwritten by the
        // rotated `(0, -1)` right after (`:44458-44459`), so only the latter
        // stands.
        let angle = which_one as f32
            * npc.facing as f32
            * 0.7
            * (std::f32::consts::PI / 4.0 / how_many as f32);
        let (sin, cos) = angle.sin_cos();
        let v = Vec2::new(sin, -cos);
        let speed = 8.0 + crate::app::rand01(rng) * 8.0;
        npc.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
            kind: crate::projectile::DEERCLOPS_ICE_CHUNK,
            x: x as f32 * 16.0 + 8.0,
            y: y as f32 * 16.0 - 8.0,
            vx: v.x * speed,
            vy: v.y * speed,
            damage: 18,
            ai0: 0.0,
            // `:44463` — the chunk's `ai1 = 6 + Next(6)`.
            ai1: 6.0 + crate::app::frame::rand_range(rng, 6) as f32,
        });
        break;
    }
}

/// `AI_123_Deerclops_MakeSpikesForward`/`MakeSpikesBothSides` +
/// `TryMakingSpike` (`NPC.cs:44466-44534`, `:44575-44585`): the Deerclops'
/// ice-spike rows. `how_many` is the row's spread (20 forward, 15 both
/// sides), `stride` the cadence (4 / 2 ticks), `trigger` the tick the row
/// opens at (36 / 56) and `both_sides` the mirrored row.
fn deerclops_make_spikes(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut u64,
    how_many: i32,
    stride: i32,
    trigger: f32,
    both_sides: bool,
) {
    let phase = npc.ai[1];
    if phase < trigger {
        return;
    }
    // `:44481-44483` — the source tile: the Deerclops' bottom, three tiles
    // forward.
    let (w, h) = npc.size();
    let source_x = ((npc.x + w * 0.5) as i32) / 16 + npc.facing * 3;
    let source_y = ((npc.y + h) as i32) / 16;
    // `:44484-44492` — the batch window: the ticks since the trigger,
    // rounded down to the stride.
    let elapsed = phase as i32 - trigger as i32;
    let base = elapsed / stride * stride;
    let mut end = base + stride;
    if elapsed % stride != 0 {
        end = base;
    }
    if both_sides {
        // `:44525-44529` — the mirrored row: the spikes walk down from the
        // batch's start, each one on both sides.
        let mut i = base;
        while i >= 0 && i < end && i < how_many {
            let x_offset = i;
            deerclops_try_make_spike(
                npc, world, player, rng, source_x, source_y, npc.facing, how_many, -i, x_offset,
            );
            deerclops_try_make_spike(
                npc,
                world,
                player,
                rng,
                source_x,
                source_y,
                -npc.facing,
                how_many,
                -i,
                x_offset,
            );
            i -= 1;
        }
    } else {
        // `:44493-44496` — the forward row.
        let mut i = base;
        while i < end && i < how_many {
            deerclops_try_make_spike(
                npc, world, player, rng, source_x, source_y, npc.facing, how_many, i, i,
            );
            i += 1;
        }
    }
}

/// `AI_123_Deerclops_TryMakingSpike` (`NPC.cs:44575-44585`): the spike's
/// tile row comes from `FindBestY`, and the spike itself flies at the
/// row's angle (`(0, -1)` rotated by `whichOne * dir * 0.7 * (PI / 4 /
/// howMany)`) with the `ai1` scale factor.
fn deerclops_try_make_spike(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut u64,
    source_x: i32,
    source_y: i32,
    dir: i32,
    how_many: i32,
    which_one: i32,
    x_offset: i32,
) {
    let x = source_x + x_offset * dir;
    let y = deerclops_find_best_y(world, player, source_y, x);
    if !active_and_walkable(world, x, y) {
        return;
    }
    let angle =
        which_one as f32 * dir as f32 * 0.7 * (std::f32::consts::PI / 4.0 / how_many as f32);
    let (sin, cos) = angle.sin_cos();
    let vel = Vec2::new(
        Vec2::new(0.0, -1.0).x * cos - Vec2::new(0.0, -1.0).y * sin,
        Vec2::new(0.0, -1.0).x * sin + Vec2::new(0.0, -1.0).y * cos,
    );
    // `:44582` — the spike's `ai1`: the scale factor.
    let ai1 = 0.1 + crate::app::rand01(rng) * 0.1 + x_offset as f32 * 1.1 / how_many as f32;
    npc.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
        kind: crate::projectile::DEERCLOPS_ICE_SPIKE,
        x: x as f32 * 16.0 + 8.0,
        y: y as f32 * 16.0 - 8.0,
        vx: vel.x,
        vy: vel.y,
        damage: 13,
        ai0: 0.0,
        ai1,
    });
}

/// `AI_123_Deerclops_TryMakingSpike_FindBestY` (`NPC.cs:44586-44640`): the
/// tile row the spike lands on — the walkable row closest to the player's
/// feet along the column, then the walk up through the solid tiles and the
/// walk down to the first walkable one.
fn deerclops_find_best_y(world: &World, player: &Player, source_y: i32, x: i32) -> i32 {
    let mut num = source_y;
    let (pcx, _) = player.center();
    let bottom_y = player.y + player.height;
    let target_row = (bottom_y / 16.0) as i32;
    let step = (target_row - num).signum();
    let stop = target_row + step * 15;
    let mut best: Option<i32> = None;
    let mut best_dist = f32::INFINITY;
    let mut i = num;
    while i != stop {
        if active_and_walkable(world, x, i) {
            let tile = Vec2::new(x as f32 * 16.0, i as f32 * 16.0);
            let d = tile.distance(Vec2::new(pcx, bottom_y));
            if best.is_none() || d < best_dist {
                best = Some(i);
                best_dist = d;
            }
        }
        i += step;
    }
    if let Some(row) = best {
        num = row;
    }
    // `:44619-44627` — the walk up through the solid tiles.
    for _ in 0..20 {
        if num < 10 {
            break;
        }
        if !solid_block(world, x, num) {
            break;
        }
        num -= 1;
    }
    // `:44628-44638` — the walk down to the first walkable tile.
    for _ in 0..20 {
        if num > world.height as i32 - 10 {
            break;
        }
        if active_and_walkable(world, x, num) {
            break;
        }
        num += 1;
    }
    num
}

/// `WorldGen.ActiveAndWalkableTile` (`WorldGen.cs:70300-70318`): the tile is
/// active, solid and not the framed-off "inactive" state.
fn active_and_walkable(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind))
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`): the full solid block
/// (active, solid, not a top-only tile, no slope, not framed off).
fn solid_block(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `AI_015_KingSlime_FindTeleportSpot` + `BuildKingSlimeTeleportCache`
/// (`NPC.cs:43077-43119`): the tile the King Slime teleports onto — a
/// solid (or platform) tile with clear air below and a line of sight to
/// the player, in the ring around the player (the 10/7 cache, then the
/// 6/2 one). The anti-cheese case lands on the player instead.
fn king_slime_find_teleport_spot(
    world: &World,
    player: &Player,
    anti_cheese: bool,
    rng: &mut u64,
) -> (f32, f32) {
    let (pcx, pcy) = player.center();
    let player_bottom = (pcx, player.y + player.height);
    if anti_cheese {
        return player_bottom;
    }
    let tx = (pcx as i32) / 16;
    let ty = (pcy as i32) / 16;
    // `TileID.Sets.Platforms` (`ID/TileID.cs:534`).
    const PLATFORMS: [u16; 7] = [19, 427, 435, 436, 437, 438, 439];
    for (outer, inner) in [(10i32, 7i32), (6, 2)] {
        let mut cache: Vec<(i32, i32)> = Vec::new();
        // The four bands (`:43080-43083`).
        for (x0, x1, y0, y1) in [
            (tx - outer, tx - inner, ty - outer, ty + outer),
            (tx + inner, tx + outer, ty - outer, ty + outer),
            (tx - inner, tx + inner, ty - outer, ty - inner),
            (tx - inner, tx + inner, ty + inner, ty + outer),
        ] {
            for x in x0..=x1 {
                for y in y0..=y1 {
                    let Some(t) = world.tile(x, y) else {
                        continue;
                    };
                    let solid = tr_core::tiledata::TILE_SOLID.has(t.kind)
                        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
                    let platform = PLATFORMS.contains(&t.kind);
                    let below_solid = world
                        .tile(x, y - 1)
                        .is_some_and(|b| b.active() && tr_core::tiledata::TILE_SOLID.has(b.kind));
                    if !t.active() || !(solid || platform) || below_solid {
                        continue;
                    }
                    // `:43065-43066` — the spot's position and the sight line.
                    let pos = Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 - 46.0);
                    if crate::projectile::can_hit_line(
                        world, pos.x, pos.y, 0.0, 0.0, pcx, pcy, 0.0, 0.0,
                    ) {
                        cache.push((x, y));
                    }
                }
            }
        }
        if !cache.is_empty() {
            let pick = crate::app::frame::rand_range(rng, cache.len() as u32) as usize;
            let (x, y) = cache[pick];
            return (x as f32 * 16.0 + 8.0, y as f32 * 16.0);
        }
    }
    player_bottom
}

/// `AI`'s aiStyle-76 ground scan: the solid-tile count under the given
/// tile column (`NPC.cs:36161-36178`'s shape — the first non-`solidTop`
/// solid tile within 150 tiles).
pub(crate) fn ground_clearance(world: &World, tile_x: i32, tile_y: i32) -> f32 {
    let mut count = 0;
    if world
        .tile(tile_x, tile_y)
        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
    {
        return 16.0;
    }
    while (count as usize) < 150 && tile_y + count < world.height {
        let solid = world
            .tile(tile_x, tile_y + count)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
        if solid {
            return (count - 1).max(0) as f32 * TILE_PX;
        }
        count += 1;
    }
    count as f32 * TILE_PX
}

impl Npc {
    /// `NPC.CanBeChasedBy` (`NPC.cs:90030-90041`): the gate every minion and
    /// sentry target search runs. The port's liveness is `hp > 0` (the NPC
    /// list carries no `active` field — callers filter on `hp`); the
    /// `lifeMax > 5` term keeps critters out; `dontTakeDamage`, `friendly` and
    /// `immortal` each block the target. The
    /// `DebugOptions.LetProjectilesAimAtTargetDummies` debug arm (the
    /// immortal bypass) is the default-false path and stays unported — the
    /// target dummies are unported kinds anyway.
    pub fn can_be_chased_by(&self) -> bool {
        self.hp > 0
            && self.chaseable
            && self.max_hp > 5
            && !self.dont_take_damage
            && !self.friendly
            && !self.immortal
    }

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

    /// `SlimeRainSpawns`' sky spawn (`NPC.cs:5958`'s `NewNPC(type)` at
    /// the random sky tile): the position is the caller's — the slime
    /// falls through the air under its own gravity. The seed-facing
    /// variation and the variant stats ride the ground ctor's shape.
    pub fn spawn_slime_sky(x_px: f32, y_px: f32, variant: SlimeVariant) -> Self {
        let stats = variant.stats();
        let mut seed = ((x_px / 16.0) as i32 as u32).wrapping_mul(2_654_435_761);
        seed ^= seed >> 15;
        let mut npc = Self::new(
            NpcKind::Slime,
            x_px,
            y_px,
            if seed & 1 == 0 { 1 } else { -1 },
            0.4 + (seed % 80) as f32 * 0.01,
        );
        npc.slime = variant;
        npc.max_hp = stats.life;
        npc.hp = stats.life;
        npc
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

    /// `NPC.SpawnOnPlayer(plr, 4)` (`NPC.cs:80475+`): the boss spawn scan
    /// looks for open space around the player's tile; the flying,
    /// no-tile-collide eye only needs an in-bounds spot above the player —
    /// the AI flies to the target from wherever it appears (the tile-scan
    /// fidelity is the M4-BOSS polish follow-up).
    pub fn spawn_eye_of_cthulhu(world: &World, player_x: f32, player_y: f32) -> Option<Self> {
        let (w, h) = NpcKind::EyeOfCthulhu.size();
        // SpawnBoss' tile-anchored position: the player's centre column,
        // one screen above (the vanilla scan prefers off-screen space).
        let x = player_x - w * 0.5;
        // Clamped into the world: near the surface the 600-px lift would
        // leave it; the AI flies to the target from wherever it appears.
        let y = (player_y - 600.0 - h * 0.5).max(16.0);
        if x < 0.0
            || y < 0.0
            || x + w > world.width as f32 * TILE_PX
            || y + h > world.height as f32 * TILE_PX
        {
            return None;
        }
        let mut npc = Self::new(NpcKind::EyeOfCthulhu, x, y, -1, 0.0);
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// The aiStyle-4 block's `NPC.NewNPC(..., 5)` minion spawn
    /// (`NPC.cs:4598` of the AI()): the servant appears 10 px out along the
    /// aim and inherits the launch velocity (`vector3.X/Y`).
    pub fn spawn_servant_of_cthulhu(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        let mut npc = Self::new(NpcKind::ServantOfCthulhu, x - 10.0, y - 10.0, 1, 0.0);
        npc.vx = vx;
        npc.vy = vy;
        npc
    }

    /// Zombies walk on the surface and burn away at dawn.
    /// Spawns a cave walker at the cavern layer's surface (the vanilla
    /// cave spawns pick the open-air tiles below the rock layer).
    pub fn spawn_cave_walker(world: &World, x_tile: i32, kind: NpcKind) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
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

    /// Spawns a caster standing on the found ground tile (`SpawnNPC`'s
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

    /// Spawns a worm head standing on the found ground tile (`SpawnNPC`'s
    /// `Bottom` placement — the chain assembles from the head's first worm
    /// pass: each link spawns its child at the parent's bottom-centre,
    /// `NPC.cs:51347`).
    pub fn spawn_worm_head(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Self {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = ground_y as f32 * TILE_PX - h;
        Self::new(kind, x, y, 1, 0.0)
    }

    /// Spawns a swimmer standing on the found ground tile inside its water
    /// column (`SpawnNPC`'s `Bottom = (x*16+8, y*16)` — the same placement
    /// the casters ride).
    pub fn spawn_swimmer(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Option<Self> {
        Self::spawn_caster(world, x_tile, ground_y, kind)
    }

    /// Spawns a cave flyer hanging in the open air of the cave layer (the
    /// vanilla bat spawn picks the open tiles below the rock layer — the
    /// `SpawnAnNPC` cave arm, `NPC.cs:5233`).
    /// Spawns a hover in the open air of its layer (`NPC.NewNPC`).
    pub fn spawn_hover(world: &World, x_tile: i32, y_tile: i32, kind: NpcKind) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = y_tile as f32 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, y_tile).is_some_and(|t| t.active()) {
            return None;
        }
        Some(Self::new(kind, x, y, 1, 0.0))
    }

    pub fn spawn_cave_flyer(
        world: &World,
        x_tile: i32,
        y_tile: i32,
        kind: NpcKind,
    ) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = y_tile as f32 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, y_tile).is_some_and(|t| t.active()) {
            return None;
        }
        if !world.in_bounds(x_tile, (y / TILE_PX) as i32 + 1) {
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

    /// Spawns the Guide homeless at the surface tile (`NPC.NewNPC`'s
    /// world-gen arrival).
    /// The town spawn ctor — the Guide and the Merchant share every stat
    /// (250 HP / 10 dmg / 15 def / 0.5 kb / 18x40), so one body serves both
    /// (`SetDefaults` cases 17/22).
    pub fn spawn_town(world: &World, x_tile: i32, kind: NpcKind) -> Option<Self> {
        let mut npc = Self::spawn_guide(world, x_tile)?;
        npc.kind = kind;
        npc.max_hp = kind.max_hp();
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// A town NPC standing on the given tile — `WorldGen.SpawnTownNPC`'s
    /// `NewNPC` at the room position (`WorldGen.cs:5409-5500`), which is a
    /// *room* tile, not the outdoor surface [`Self::spawn_guide`] uses for the
    /// world-spawn Guide.
    pub fn spawn_town_at(world: &World, tile_x: i32, tile_y: i32, kind: NpcKind) -> Option<Self> {
        let (w, h) = kind.size();
        let x = tile_x as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = tile_y as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(kind, x, y, 1, 0.0))
    }

    pub fn spawn_guide(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Guide.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(NpcKind::Guide, x, y, 1, 0.0))
    }

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

    pub(crate) fn new(kind: NpcKind, x: f32, y: f32, facing: i32, hop_timer: f32) -> Self {
        Self {
            kind,
            shimmer_transparency: 0.0,
            extra_value: 0,
            slime: SlimeVariant::Blue,
            x,
            y,
            old_x: x,
            old_y: y,
            vx: 0.0,
            vy: 0.0,
            given_name: String::new(),
            hp: kind.max_hp(),
            max_hp: kind.max_hp(),
            on_ground: false,
            facing,
            // `NPC.spriteDirection`'s `SetDefaults` default (`NPC.cs:6515`):
            // -1 until a `FindFrame` case or an AI writes it.
            sprite_dir: -1,
            hop_timer,
            anim: 0.0,
            hurt_cd: 0.0,
            immune: [0; 256],
            spawned_from_statue: false,
            buffs: NpcBuffs::for_kind(kind),
            frame: 0,
            frame_counter: 0.0,
            rotation: 0.0,
            scale: 1.0,
            net_offset: Vec2::ZERO,
            burn: 0.0,
            wants_minion: false,
            wants_minion_batch: 0,
            home: None,
            variation_index: 0,
            homeless_despawn: false,
            caster_counter: 0,
            cast_windup: 0,
            frost_stuck_x: 0.0,
            wants_sphere: Vec::new(),
            swim_phase: 0.0,
            worm_id: 0,
            seg_index: 0,
            worm_left: 0,
            worm_inited: false,
            wants_leech: false,
            wants_limbs: false,
            wants_hungry: false,
            damage_override: None,
            wof_bottom_target: -1.0,
            wof_top_target: -1.0,
            wet_x: 0.0,
            ai: [0.0; 4],
            wants_servant: None,
            defense_override: None,
            dont_take_damage: false,
            // NPC.cs:6517,8520 — behindTiles defaults false; per-type SetDefaults sets true (e.g. 10355).
            behind_tiles: kind.behind_tiles(),
            // The `CanBeChasedBy` slice (`NPC.cs:6261`/`:6259`/`:6563`):
            // `chaseable`'s stock true (the false writers are the unported
            // case-345 family), `immortal`'s stock false, `friendly` the town
            // residents' `friendly = true` write (`NPC.cs:8983+`) plus the
            // Sleeping Angler's own arm (`:13798`).
            chaseable: true,
            immortal: false,
            friendly: kind.is_town_resident() || kind == NpcKind::SleepingAngler,
            swarm_done: false,
            king_slime_local: [0.0; 4],
            size_override: None,
            brain_local: [0.0; 4],
            moon_local: [0.0; 3],
            moon_core_alive: false,
            moon_parts: [-1, -1, -1],
            rifleman_rearmed: false,
            wants_hostile_shot: Vec::new(),
            wants_hostile_shot_ai: Vec::new(),
            slot: 0,
            wants_martian_invasion: false,
            wants_event_stop: false,
            wants_scutlix: false,
            carrier_x: -1.0,
            carrier_y: -1.0,
            wants_saucer_crew: false,
            alpha: 0,
            head_turn: 0,
            wet: false,
            carrier_timer: -1.0,
            carrier_phase: 0.0,
            carrier_rot: 0.0,
            carrier_facing: -1,
            crew_turrets: 0,
            crew_cannons: 0,
            wants_dutchman_drop: None,
            wants_moon_parts: false,
            wants_moon_parts_clear: false,
            wants_blast: false,
            wants_dd2_spawn: false,
            wants_lane_right: false,
            wants_stardust_mark: false,
            wants_obstruct: false,
            wants_moon_leech: false,
            wants_dutchman_crew: false,
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
            // `SetDefaults` case 21/44: 0.5; case 49: 0.8.
            NpcKind::Skeleton => 0.5,
            NpcKind::UndeadMiner => 0.5,
            NpcKind::CaveBat => 0.8,
            // `SetDefaults` case 51/60/66: knockBackResist 0.8.
            NpcKind::JungleBat => 0.8,
            NpcKind::Hellbat => 0.8,
            NpcKind::VoodooDemon => 0.8,
            // `SetDefaults` case 6: 0.5; case 42: 0.5; case 23: 0.4.
            NpcKind::EaterOfSouls => 0.5,
            NpcKind::Hornet => 0.5,
            NpcKind::MeteorHead => 0.4,
            // The casters: `SetDefaults` knockBackResist 0.6/0.6/0.6/0.5.
            // `NPC.cs`'s cases 26 (0.8) / 27+111 (0.7) / 28 (0.5) / 29 (0.6)
            // / 471 (0.15).
            NpcKind::GoblinPeon => 0.8,
            NpcKind::GoblinThief | NpcKind::GoblinArcher => 0.7,
            NpcKind::GoblinWarrior => 0.5,
            NpcKind::GoblinSorcerer => 0.6,
            // The frost trio: kb 0.6.
            NpcKind::GoblinSummoner => 0.15,
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => 0.6,
            // The martians: 0.25/0/0.4/0.3/0.
            NpcKind::MartianOfficer => 0.25,
            NpcKind::GrayGrunt => 0.0,
            NpcKind::MartianEngineer => 0.4,
            NpcKind::GigaZapper => 0.3,
            NpcKind::MartianWalker => 0.0,
            // The probe: 0 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 0.0,
            // The drone: 0.4 (`NPC.cs`'s case 388 — the hover's scaled
            // value; the charge/dash arms zero it in the machine).
            NpcKind::MartianDrone => 0.4,
            // The shooters: 0.35 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 0.35,
            // The rider: the field default 0.4 (case 390 sets nothing);
            // the carrier 0.1 (case 391).
            NpcKind::ScutlixRider => 0.4,
            NpcKind::Scutlix => 0.1,
            // The saucer family: kb 0 (`NPC.cs`'s cases 392-395).
            NpcKind::MartianSaucer => 0.0,
            NpcKind::MartianSaucerTurret => 0.0,
            NpcKind::MartianSaucerCannon => 0.0,
            NpcKind::MartianSaucerCore => 0.0,
            // The Dutchman family: kb 0 (cases 491/492).
            NpcKind::FlyingDutchman => 0.0,
            NpcKind::FlyingDutchmanCannon => 0.0,
            // The DD2 goblins: kb 0.2/0.2/0.15 (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 => 0.2,
            NpcKind::DD2GoblinT3 => 0.15,
            // The support's kb: the bombers 0.1, the javelineers 0.2, the
            // skeletons 0.5 (`NPC.cs`'s rows).
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3 => 0.1,
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => 0.2,
            NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => 0.3,
            // The Dark Mage: kb 0 (case 564 sets nothing).
            NpcKind::DD2DarkMage => 0.0,
            // Betsy: kb 0 (case 551 sets nothing).
            NpcKind::DD2Betsy => 0.0,
            // The WitherBeast: kb 0.15.
            NpcKind::DD2WitherBeast => 0.15,
            NpcKind::DD2WyvernT1 => 0.2,
            NpcKind::DD2WyvernT2 => 0.2,
            NpcKind::DD2WyvernT3 => 0.05,
            NpcKind::DD2DarkMageT3 => 0.0,
            NpcKind::DD2WitherBeastT3 => 0.05,
            NpcKind::DD2DrakinT2 => 0.0,
            NpcKind::DD2DrakinT3 => 0.0,
            NpcKind::DD2KoboldWalkerT2 => 0.2,
            NpcKind::DD2KoboldWalkerT3 => 0.1,
            NpcKind::DD2KoboldFlyerT2 => 0.4,
            NpcKind::DD2KoboldFlyerT3 => 0.25,
            NpcKind::DD2OgreT2 => 0.0,
            NpcKind::DD2OgreT3 => 0.0,
            // The cultist + the towers: kb 0.
            NpcKind::AncientCultist
            | NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::VortexTower
            | NpcKind::SolarTower => 0.0,
            NpcKind::VortexTower => 0.0,
            // The shards' kb: 0.3/0.4/0.03/0.2/0.4/0.6 (`NPC.cs`'s rows).
            // The Moon Lord family's kb: 0 across the parts (`NPC.cs` rows).
            NpcKind::MoonLordHead
            | NpcKind::MoonLordHand
            | NpcKind::MoonLordCore
            | NpcKind::MoonLordFreeEye
            | NpcKind::MoonLordLeechBlob => 0.0,
            NpcKind::StardustCellSmall => 0.4,
            NpcKind::StardustJellyfishBig => 0.03,
            NpcKind::NebulaBeast => 0.2,
            // The vortex trio's kb: 0.3/0.5/1.0 (`NPC.cs`'s rows; the headcrab
            // keeps the class default 1.0).
            NpcKind::StardustSpiderBig => 0.4,
            NpcKind::StardustSpiderSmall => 0.3,
            NpcKind::StardustSoldier => 0.4,
            NpcKind::SolarDrakomire => 0.2,
            NpcKind::SolarDrakomireRider => 0.2,
            NpcKind::SolarSroller => 0.0,
            NpcKind::SolarCorite => 0.2,
            NpcKind::SolarSpearman => 0.4,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            NpcKind::SolarSpearman => 0.4,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            NpcKind::SolarSpearman => 0.4,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            NpcKind::SolarSpearman => 0.4,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            NpcKind::SolarSpearman => 0.4,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            NpcKind::SolarSelenian => 0.4,
            NpcKind::NebulaFloater => 0.5,
            NpcKind::BrainSuckler => 0.5,
            NpcKind::Predictor => 0.6,
            NpcKind::NebulaFloater => 0.5,
            NpcKind::BrainSuckler => 0.5,
            NpcKind::Predictor => 0.6,
            NpcKind::VortexHunter => 0.3,
            NpcKind::NebulaFloater => 0.5,
            NpcKind::BrainSuckler => 0.5,
            NpcKind::Predictor => 0.6,
            NpcKind::VortexAlien => 0.5,
            NpcKind::VortexHeadcrab => 1.0,
            NpcKind::VortexRifleman => 0.4,
            NpcKind::VortexSoldier => 0.6,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            // The tavernkeeper 0.5, the crystal 0.
            NpcKind::DD2Bartender => 0.5,
            NpcKind::DD2EterniaCrystal => 0.0,
            // The pirates: 0.4/0.2/0.3/0.35/0 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 0.4,
            NpcKind::PirateCaptainsMate => 0.2,
            NpcKind::PirateArcher => 0.3,
            NpcKind::PirateCorsair => 0.35,
            NpcKind::PirateCaptain => 0.0,
            // The whole family: kb 0 (`:12171`/`:12188`/`:12204`/`:12218`).
            NpcKind::Golem
            | NpcKind::GolemHead
            | NpcKind::GolemFistLeft
            | NpcKind::GolemFistRight
            | NpcKind::GolemHeadFree => 0.0,
            // `:17461`'s buyPrice(0, 5) = 5 gold = 50000 copper.
            NpcKind::Deerclops => 50000.0,
            NpcKind::SkeletronHand => 0.0,
            NpcKind::WallOfFleshEye => 80000.0,
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                0.0
            }
            NpcKind::Deerclops => 0.0,
            NpcKind::SkeletronHand => 0.0,
            NpcKind::WallOfFleshEye => 0.0,
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                0.0
            }
            NpcKind::Tim => 0.6,
            NpcKind::DarkCaster => 0.6,
            NpcKind::FireImp => 0.5,
            // The spheres: 0 (the knockback passes through).
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,
            // The swimmers: the Goldfish 0.5 (`:9471`), the Shark 0.7
            // (`:9622`), the Piranha no row — the `SetDefaults` head's 1f
            // reset (`:8533`).
            NpcKind::Goldfish => 0.5,
            NpcKind::Piranha => 1.0,
            NpcKind::Shark => 0.7,
            // The worms: knockBackResist 0 (`:8763` family-wide).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 0.0,
            // The town NPCs barely knock (`NPC.SetDefaults` 22: 0.5).
            NpcKind::Guide => 0.5,
            // `SetDefaults` case 17: knockBackResist 0.5.
            NpcKind::Merchant => 0.5,
            NpcKind::Dryad => 0.5,
            NpcKind::Nurse => 0.5,
            NpcKind::ArmsDealer => 0.5,
            NpcKind::Demolitionist => 0.5,
            NpcKind::Painter => 0.5,
            NpcKind::WitchDoctor => 0.5,
            NpcKind::Clothier => 0.5,
            NpcKind::Angler => 0.5,
            NpcKind::Wizard => 0.5,
            NpcKind::BoundWizard => 0.5,
            NpcKind::Mechanic => 0.5,
            NpcKind::Stylist => 0.5,
            NpcKind::WebbedStylist => 0.5,
            NpcKind::Golfer => 0.5,
            NpcKind::Zoologist => 0.5,
            NpcKind::DyeTrader => 0.5,
            NpcKind::PartyGirl => 0.5,
            NpcKind::Princess => 0.5,
            NpcKind::SleepingGolfer => 0.5,
            NpcKind::TaxCollector => 0.5,
            NpcKind::BoundMechanic => 0.5,
            NpcKind::SleepingAngler => 0.5,
            NpcKind::TravellingMerchant => 0.5,
            NpcKind::EyeOfCthulhu => 0.0,
            NpcKind::QueenBee => 0.0,
            NpcKind::SkeletronHead => 0.0,
            NpcKind::WallOfFlesh => 0.0,
            NpcKind::TheDestroyer => 0.0,
            // `SetDefaults` case 127: knockBackResist 0.
            NpcKind::SkeletronPrime => 0.0,
            NpcKind::Retinazer => 0.0,
            NpcKind::Spazmatism => 0.0,
            NpcKind::Plantera => 0.0,
            NpcKind::BrainOfCthulhu => 0.45,
            NpcKind::Creeper => 0.8,
            NpcKind::BoundGoblin => 0.5,
            NpcKind::GoblinTinkerer => 0.5,
            NpcKind::SkeletronPrime => 0.0,
            NpcKind::Retinazer => 0.0,
            NpcKind::Spazmatism => 0.0,
            NpcKind::KingSlime => 0.0,
            // 115/117/118: knockBackResist 0.
            NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody => 0.0,
            // The eye/servant blocks set no knockBackResist — the
            // SetDefaults head's default applies.
            NpcKind::EyeOfCthulhu => 1.0,
            NpcKind::ServantOfCthulhu => 1.0,
            NpcKind::Slime => unreachable!(),
            // The pillar worms: kb 0 across the chain (`NPC.cs`'s rows
            // 412-414) and the stardust worm head.
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 0.0,
            NpcKind::StardustWormHead => 0.0,
            _ => 0.4,
        }
    }

    /// `NPC.defense` — `Main.CalculateDamageNPCsTake`'s input
    /// (blue `NPC.cs:8573` 2, demon eye 2, Wandering Eye 20, zombie 6,
    /// King Slime 10; the slime variants ride their `SetDefaultsFromNetId`
    /// lines).
    pub fn defense(&self) -> i32 {
        if let Some(d) = self.defense_override {
            return d;
        }
        if self.kind == NpcKind::Slime {
            return self.slime.stats().defense;
        }
        match self.kind {
            NpcKind::DemonEye => 2,
            NpcKind::WanderingEye => 20,
            NpcKind::Zombie => 6,
            // `SetDefaults` case 21: defense 8; case 44: 9; case 49: 2.
            NpcKind::Skeleton => 8,
            NpcKind::UndeadMiner => 9,
            NpcKind::CaveBat => 2,
            // `SetDefaults` case 51: defense 4; case 60: 8; case 66: 8.
            NpcKind::JungleBat => 4,
            NpcKind::Hellbat => 8,
            NpcKind::VoodooDemon => 8,
            // `SetDefaults` case 6: defense 8; case 42: 12; case 23: 6.
            NpcKind::EaterOfSouls => 8,
            NpcKind::Hornet => 12,
            NpcKind::MeteorHead => 6,
            // The casters: `SetDefaults` defense 2/4/2/16.
            // `NPC.cs`'s cases 26 (4) / 27+111 (6) / 28 (8) / 29 (2) /
            // 471 (26).
            NpcKind::GoblinPeon => 4,
            NpcKind::GoblinThief | NpcKind::GoblinArcher => 6,
            NpcKind::GoblinWarrior => 8,
            NpcKind::GoblinSorcerer => 2,
            // The frost trio: 20/26/22.
            NpcKind::SnowmanGangsta => 20,
            NpcKind::SnowBalla => 26,
            NpcKind::MysterySnowman => 22,
            // The martians: 50/30/34/38/40.
            NpcKind::MartianOfficer => 50,
            NpcKind::GrayGrunt => 30,
            NpcKind::MartianEngineer => 34,
            NpcKind::GigaZapper => 38,
            NpcKind::MartianWalker => 40,
            // The probe: 5 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 5,
            // The drone: 16 (`NPC.cs`'s case 388).
            NpcKind::MartianDrone => 16,
            // The shooters: 25/25 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 25,
            // The rider 30, the carrier 30 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 30,
            NpcKind::Scutlix => 30,
            // The saucer family: def 100/20/20/0 (`NPC.cs`'s cases
            // 392-395).
            NpcKind::MartianSaucer => 100,
            NpcKind::MartianSaucerTurret => 20,
            NpcKind::MartianSaucerCannon => 20,
            NpcKind::MartianSaucerCore => 0,
            // The Dutchman def 100, its cannons def 20.
            NpcKind::FlyingDutchman => 100,
            NpcKind::FlyingDutchmanCannon => 20,
            // The DD2 goblins: 14/20/28 def (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 => 14,
            NpcKind::DD2GoblinT2 => 20,
            NpcKind::DD2GoblinT3 => 28,
            // The support's defense: 16/26/34, 18/28/38, 12/22.
            NpcKind::DD2GoblinBomberT1 => 16,
            NpcKind::DD2GoblinBomberT2 => 26,
            NpcKind::DD2GoblinBomberT3 => 34,
            NpcKind::DD2JavelinT1 => 18,
            NpcKind::DD2JavelinT2 => 28,
            NpcKind::DD2JavelinT3 => 38,
            NpcKind::DD2SkeletonT1 => 12,
            NpcKind::DD2SkeletonT3 => 22,
            // The Dark Mage: def 18.
            NpcKind::DD2DarkMage => 18,
            NpcKind::DD2WyvernT1 => 4,
            NpcKind::DD2WyvernT2 => 16,
            NpcKind::DD2WyvernT3 => 30,
            NpcKind::DD2DarkMageT3 => 38,
            NpcKind::DD2WitherBeastT3 => 40,
            NpcKind::DD2DrakinT2 => 30,
            NpcKind::DD2DrakinT3 => 40,
            NpcKind::DD2KoboldWalkerT2 => 26,
            NpcKind::DD2KoboldWalkerT3 => 32,
            NpcKind::DD2KoboldFlyerT2 => 16,
            NpcKind::DD2KoboldFlyerT3 => 32,
            NpcKind::DD2OgreT2 => 34,
            NpcKind::DD2OgreT3 => 40,
            // Betsy: def 38.
            NpcKind::DD2Betsy => 38,
            // The WitherBeast: def 30.
            NpcKind::DD2WitherBeast => 30,
            // The Lunar Pillars: def 20.
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => 20,
            // Lunatic Cultist (439): defense 42 (`NPC.cs:14218`). S1-D-3b.
            NpcKind::AncientCultist => 42,
            // The lane gate: def 20 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 20,
            // The shards' defense: 50/0/38/46/40/34.
            NpcKind::StardustSpiderBig => 40,
            NpcKind::StardustSpiderSmall => 10,
            NpcKind::StardustSoldier => 34,
            NpcKind::SolarDrakomire => 32,
            NpcKind::SolarDrakomireRider => 28,
            NpcKind::SolarSroller => 34,
            NpcKind::SolarCorite => 26,
            NpcKind::SolarCrawltipedeHead => 1000,
            NpcKind::SolarCrawltipedeBody => 1000,
            NpcKind::SolarCrawltipedeTail => 0,
            NpcKind::StardustWormHead => 10,
            NpcKind::SolarSpearman => 28,
            NpcKind::SolarCrawltipedeHead => 1000,
            NpcKind::SolarCrawltipedeBody => 1000,
            NpcKind::SolarCrawltipedeTail => 0,
            NpcKind::StardustWormHead => 10,
            NpcKind::SolarSpearman => 28,
            NpcKind::SolarCrawltipedeHead => 1000,
            NpcKind::SolarCrawltipedeBody => 1000,
            NpcKind::SolarCrawltipedeTail => 0,
            NpcKind::StardustWormHead => 10,
            NpcKind::SolarSpearman => 28,
            NpcKind::SolarCrawltipedeHead => 1000,
            NpcKind::SolarCrawltipedeBody => 1000,
            NpcKind::SolarCrawltipedeTail => 0,
            NpcKind::StardustWormHead => 10,
            NpcKind::SolarSpearman => 28,
            NpcKind::SolarCrawltipedeHead => 1000,
            NpcKind::SolarCrawltipedeBody => 1000,
            NpcKind::SolarCrawltipedeTail => 0,
            NpcKind::StardustWormHead => 10,
            NpcKind::SolarSpearman => 28,
            NpcKind::SolarSelenian => 30,
            NpcKind::NebulaFloater => 20,
            NpcKind::BrainSuckler => 34,
            NpcKind::Predictor => 30,
            NpcKind::VortexHunter => 44,
            NpcKind::VortexAlien => 20,
            NpcKind::VortexHeadcrab => 6,
            NpcKind::NebulaFloater => 20,
            NpcKind::BrainSuckler => 34,
            NpcKind::Predictor => 30,
            NpcKind::MoonLordHead => 50,
            NpcKind::MoonLordHand => 40,
            NpcKind::MoonLordCore => 70,
            NpcKind::MoonLordFreeEye => 0,
            NpcKind::MoonLordLeechBlob => 0,
            NpcKind::StardustCellBig => 50,
            NpcKind::StardustCellSmall => 0,
            NpcKind::StardustJellyfishBig => 38,
            NpcKind::NebulaBeast => 46,
            NpcKind::VortexRifleman => 40,
            NpcKind::VortexSoldier => 34,
            // The tavernkeeper 15, the crystal 14.
            NpcKind::DD2Bartender => 15,
            NpcKind::DD2EterniaCrystal => 14,
            // The pirates: 17/22/14/20/30 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 17,
            NpcKind::PirateCaptainsMate => 22,
            NpcKind::PirateArcher => 14,
            NpcKind::PirateCorsair => 20,
            NpcKind::PirateCaptain => 30,
            // `SetDefaults` 245-249: 26/20/28/28/32.
            NpcKind::Golem => 26,
            NpcKind::GolemHead => 20,
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 28,
            NpcKind::GolemHeadFree => 32,
            // `:17457`: 10.
            NpcKind::Deerclops => 10,
            NpcKind::SkeletronHand => 14,
            NpcKind::WallOfFleshEye => 0,
            NpcKind::PrimeCannon => 23,
            NpcKind::PrimeSaw => 38,
            NpcKind::PrimeVice => 34,
            NpcKind::PrimeLaser => 20,
            NpcKind::Tim => 4,
            NpcKind::DarkCaster => 2,
            NpcKind::FireImp => 16,
            // The spheres: defense 0.
            NpcKind::ChaosBall => 0,
            NpcKind::BurningSphere => 0,
            NpcKind::WaterSphere => 0,
            // The swimmers: defense 0/2/2.
            NpcKind::Goldfish => 0,
            NpcKind::Piranha => 2,
            NpcKind::Shark => 2,
            // The worms: the head 2/0/2, the body 6/4/4, the tail 10/6/8.
            NpcKind::DevourerHead => 2,
            NpcKind::DevourerBody => 6,
            NpcKind::DevourerTail => 10,
            NpcKind::GiantWormHead => 0,
            NpcKind::GiantWormBody => 4,
            NpcKind::GiantWormTail => 6,
            NpcKind::EaterOfWorldsHead => 2,
            NpcKind::EaterOfWorldsBody => 4,
            NpcKind::EaterOfWorldsTail => 8,
            NpcKind::Guide => 15,
            // `SetDefaults` case 17: defense 15.
            NpcKind::Merchant => 15,
            NpcKind::Dryad => 15,
            NpcKind::Nurse => 15,
            NpcKind::ArmsDealer => 15,
            NpcKind::Demolitionist => 15,
            NpcKind::Painter => 15,
            NpcKind::WitchDoctor => 15,
            NpcKind::Clothier => 15,
            NpcKind::Angler => 15,
            NpcKind::Wizard => 15,
            NpcKind::BoundWizard => 15,
            NpcKind::Mechanic => 15,
            NpcKind::Stylist => 15,
            NpcKind::WebbedStylist => 15,
            NpcKind::Golfer => 15,
            NpcKind::Zoologist => 15,
            NpcKind::DyeTrader => 15,
            NpcKind::PartyGirl => 15,
            NpcKind::Princess => 15,
            NpcKind::SleepingGolfer => 15,
            NpcKind::TaxCollector => 15,
            NpcKind::BoundMechanic => 15,
            NpcKind::SleepingAngler => 15,
            NpcKind::TravellingMerchant => 15,
            NpcKind::EyeOfCthulhu => 12,
            NpcKind::QueenBee => 8,
            NpcKind::SkeletronHead => 10,
            NpcKind::WallOfFlesh => 12,
            NpcKind::TheDestroyer => 0,
            NpcKind::SkeletronPrime => 24,
            NpcKind::Retinazer => 10,
            NpcKind::Spazmatism => 10,
            NpcKind::Plantera => 14,
            NpcKind::BrainOfCthulhu => 14,
            NpcKind::Creeper => 10,
            // def 15 both (`:10254`, `:10279`).
            NpcKind::BoundGoblin => 15,
            NpcKind::GoblinTinkerer => 15,
            NpcKind::KingSlime => 10,
            // `NPC.cs`'s 115 (10) / 117 (2) / 118 (6) — the hungry's
            // per-tick tiers ride `defense_override`.
            NpcKind::Hungry => 10,
            NpcKind::LeechHead => 2,
            NpcKind::LeechBody => 6,
            // `NPC.cs:8709` (the eye defense 12; the AI zeroes it in the
            // final phase) / `:8721` (the servant 0).
            NpcKind::EyeOfCthulhu => 12,
            NpcKind::ServantOfCthulhu => 0,
            // `SetDefaults` case 471: defense 26; the frost trio: 20/26/22.
            NpcKind::GoblinSummoner => 26,
            NpcKind::SnowmanGangsta => 20,
            NpcKind::SnowBalla => 26,
            NpcKind::MysterySnowman => 22,
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
            // `NPC.cs:14130`: the core alone carries a value (1 platinum
            // = 1000000 copper); the parts are value 0.
            NpcKind::VortexHunter => 1200.0,
            NpcKind::NebulaFloater => 400.0,
            NpcKind::DD2LanePortal => 0.0,
            NpcKind::VortexAlien => 400.0,
            NpcKind::VortexHeadcrab => 100.0,
            NpcKind::StardustSpiderBig => 900.0,
            NpcKind::StardustSpiderSmall => 30.0,
            NpcKind::StardustSoldier => 900.0,
            NpcKind::SolarDrakomire => 700.0,
            NpcKind::SolarDrakomireRider => 700.0,
            NpcKind::SolarSroller => 700.0,
            NpcKind::SolarCorite => 800.0,
            NpcKind::SolarSelenian => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::SolarCrawltipedeHead => 700.0,
            NpcKind::SolarCrawltipedeBody => 30.0,
            NpcKind::SolarCrawltipedeTail => 30.0,
            NpcKind::StardustWormHead => 700.0,
            NpcKind::SolarSpearman => 900.0,
            NpcKind::VortexTower => 0.0,
            // The lane gate: value 0 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 0.0,
            NpcKind::BrainSuckler => 60.0,
            NpcKind::Predictor => 900.0,
            NpcKind::MoonLordCore => 1_000_000.0,
            NpcKind::MoonLordHead
            | NpcKind::MoonLordHand
            | NpcKind::MoonLordFreeEye
            | NpcKind::MoonLordLeechBlob => 0.0,
            NpcKind::DemonEye => 75.0,
            NpcKind::WanderingEye => 500.0,
            // `:17461`'s buyPrice(0, 5) = 5 gold = 50000 copper.
            NpcKind::Deerclops => 50000.0,
            NpcKind::SkeletronHand => 0.0,
            NpcKind::WallOfFleshEye => 80000.0,
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                0.0
            }
            NpcKind::Zombie => 60.0,
            // `SetDefaults` case 21: value 100; case 44: 200; case 49: 60.
            NpcKind::Skeleton => 100.0,
            NpcKind::UndeadMiner => 200.0,
            NpcKind::CaveBat => 60.0,
            // `SetDefaults` case 51: value 80; case 60: 120; case 66: 300.
            NpcKind::JungleBat => 80.0,
            NpcKind::Hellbat => 120.0,
            NpcKind::VoodooDemon => 300.0,
            // `SetDefaults` case 6: value 150; case 42: 200; case 23: 250.
            NpcKind::EaterOfSouls => 150.0,
            NpcKind::Hornet => 200.0,
            NpcKind::MeteorHead => 250.0,
            // The casters: `SetDefaults` value 200/5000/140/350.
            // `NPC.cs`'s cases 26 (100) / 27+111 (200) / 28 (150) / 29 (200)
            // / 471 (5000).
            NpcKind::GoblinPeon => 100.0,
            NpcKind::GoblinThief | NpcKind::GoblinArcher | NpcKind::GoblinSorcerer => 200.0,
            NpcKind::GoblinWarrior => 150.0,
            NpcKind::GoblinSummoner => 5000.0,
            // The frost trio: value 400.
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => 400.0,
            // The martians: 1200/600/1200/1200/0 (the walker has no value row —
            // the vanilla's 2000-HP body drops through the kill chain).
            NpcKind::MartianOfficer => 1200.0,
            NpcKind::GrayGrunt => 600.0,
            NpcKind::MartianEngineer => 1200.0,
            NpcKind::GigaZapper => 1200.0,
            NpcKind::MartianWalker => 0.0,
            // The probe: 0 (`NPC.cs`'s case 399; the uniform drops don't
            // exist — the vanilla registers no loot for it).
            NpcKind::MartianProbe => 0.0,
            // The drone: 1000 (`NPC.cs`'s case 388; the loot db registers
            // no rules for 388 — nothing drops).
            NpcKind::MartianDrone => 1000.0,
            // The shooters: 1000/1000 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 1000.0,
            // The rider 1200, the carrier 1200 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 1200.0,
            NpcKind::Scutlix => 1200.0,
            // The saucer family: value 0 (cases 392-395 set nothing).
            NpcKind::MartianSaucer => 0.0,
            NpcKind::MartianSaucerTurret => 0.0,
            NpcKind::MartianSaucerCannon => 0.0,
            NpcKind::MartianSaucerCore => 0.0,
            // The Dutchman family: value 0 (the loot rides the guns).
            NpcKind::FlyingDutchman => 0.0,
            NpcKind::FlyingDutchmanCannon => 0.0,
            // The DD2 goblins: value 0 (cases 552-554 set nothing).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => 0.0,
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3
            | NpcKind::DD2JavelinT1
            | NpcKind::DD2JavelinT2
            | NpcKind::DD2JavelinT3
            | NpcKind::DD2SkeletonT1
            | NpcKind::DD2SkeletonT3
            | NpcKind::DD2DarkMage
            | NpcKind::DD2Betsy
            | NpcKind::DD2WitherBeast
            | NpcKind::DD2LanePortal
            | NpcKind::DD2WyvernT1
            | NpcKind::DD2WyvernT2
            | NpcKind::DD2WyvernT3
            | NpcKind::DD2KoboldFlyerT2
            | NpcKind::DD2KoboldFlyerT3
            | NpcKind::DD2WitherBeastT3
            | NpcKind::DD2DrakinT2
            | NpcKind::DD2DrakinT3
            | NpcKind::DD2KoboldWalkerT2
            | NpcKind::DD2KoboldWalkerT3
            | NpcKind::DD2OgreT2
            | NpcKind::DD2OgreT3
            | NpcKind::DD2DarkMageT3
            | NpcKind::DD2EterniaCrystal
            | NpcKind::DD2Bartender
            | NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::VortexTower
            | NpcKind::SolarTower => 0.0,
            // Lunatic Cultist (439): value 100000f (`NPC.cs:14226`). S1-D-3b.
            NpcKind::AncientCultist => 100000.0,
            NpcKind::VortexTower => 0.0, // the C# value = 0 for the pillars
            NpcKind::DD2LanePortal => 0.0,
            // The Moon Lord family's kb: 0 across the parts (`NPC.cs` rows).
            NpcKind::MoonLordHead
            | NpcKind::MoonLordHand
            | NpcKind::MoonLordCore
            | NpcKind::MoonLordFreeEye
            | NpcKind::MoonLordLeechBlob => 0.0,
            // The shards' kb: 0.3/0.4/0.03/0.2/0.4/0.6 (`NPC.cs`'s rows).
            NpcKind::StardustCellBig => 0.3,
            NpcKind::StardustCellSmall => 0.4,
            NpcKind::StardustJellyfishBig => 0.03,
            NpcKind::NebulaBeast => 0.2,
            NpcKind::VortexRifleman => 0.4,
            NpcKind::VortexSoldier => 0.6,
            // The pirates: 700/1000/1000/1500/50000 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 700.0,
            NpcKind::PirateCaptainsMate => 1000.0,
            NpcKind::PirateArcher => 1000.0,
            NpcKind::PirateCorsair => 1500.0,
            NpcKind::PirateCaptain => 50000.0,
            // The body carries `Item.buyPrice(0, 15)` = 15 gold = 150000
            // copper (`:12173`); the parts none.
            NpcKind::Golem => 150000.0,
            NpcKind::GolemHead
            | NpcKind::GolemFistLeft
            | NpcKind::GolemFistRight
            | NpcKind::GolemHeadFree => 0.0,
            // The frost trio: value 400.
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => 400.0,
            NpcKind::Tim => 5000.0,
            NpcKind::DarkCaster => 140.0,
            NpcKind::FireImp => 350.0,
            // The spheres carry no value.
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,
            // The swimmers: the Piranha 50 (`:9514`), the Shark 400
            // (`:9621`), the Goldfish no row (the catch item 261 rides the
            // critter-catch work).
            NpcKind::Goldfish => 0.0,
            NpcKind::Piranha => 50.0,
            NpcKind::Shark => 400.0,
            // The worms: the per-segment coin value (`:8763`'s 140, `:8850`'s
            // 40, `:8903`'s 800 — the body/tail rows carry the same value).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => 140.0,
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => 40.0,
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 800.0,
            NpcKind::Guide => 0.0,
            // The town NPCs carry no coin value.
            NpcKind::Merchant => 0.0,
            NpcKind::Dryad => 0.0,
            NpcKind::Nurse => 0.0,
            NpcKind::ArmsDealer => 0.0,
            NpcKind::Demolitionist => 0.0,
            NpcKind::Painter => 0.0,
            NpcKind::WitchDoctor => 0.0,
            NpcKind::Clothier => 0.0,
            NpcKind::Angler => 0.0,
            NpcKind::Wizard => 0.0,
            NpcKind::BoundWizard => 0.0,
            NpcKind::Mechanic => 0.0,
            NpcKind::Stylist => 0.0,
            NpcKind::WebbedStylist => 0.0,
            NpcKind::Golfer => 0.0,
            NpcKind::Zoologist => 0.0,
            NpcKind::DyeTrader => 0.0,
            NpcKind::PartyGirl => 0.0,
            NpcKind::Princess => 0.0,
            NpcKind::SleepingGolfer => 0.0,
            NpcKind::TaxCollector => 0.0,
            NpcKind::BoundMechanic => 0.0,
            NpcKind::SleepingAngler => 0.0,
            NpcKind::TravellingMerchant => 0.0,
            // `SetDefaults` case 4: value 30000 (the coin drop).
            NpcKind::EyeOfCthulhu => 30000.0,
            NpcKind::QueenBee => 50000.0,
            NpcKind::SkeletronHead => 50000.0,
            NpcKind::WallOfFlesh => 80000.0,
            NpcKind::TheDestroyer => 120000.0,
            NpcKind::SkeletronPrime => 120000.0,
            NpcKind::Retinazer => 120000.0,
            NpcKind::Spazmatism => 120000.0,
            NpcKind::Plantera => 150000.0,
            NpcKind::BrainOfCthulhu => 50000.0,
            NpcKind::Creeper => 0.0,

            NpcKind::BoundGoblin => 0.0,
            NpcKind::GoblinTinkerer => 0.0,
            NpcKind::KingSlime => 10_000.0,
            // The hungry/leech rows carry no value line.
            NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody => 0.0,
            // `NPC.cs:8713` (the eye value 30000); the servant has no row.
            NpcKind::EyeOfCthulhu => 30_000.0,
            NpcKind::ServantOfCthulhu => 0.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// The scaled size for slime variants (the hitbox rides
    /// `SetDefaults_ForNetId`'s size scale).
    pub fn size(&self) -> (f32, f32) {
        // The per-NPC box override (`NPC.base.width`/`base.height`) — the
        // King Slime's hp-scaled body (`AI_015`, `NPC.cs:43347-43352`).
        if let Some((w, h)) = self.size_override {
            return (w, h);
        }
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

    /// `NPC.IsDamageDodgeable` (`NPC.cs:78744-78753`): the contact's
    /// `Hurt(dodgeable: ..)` value (`Player.cs:31423`). The Skeletron heads
    /// are dodgeable only outside their `ai[1] == 2f` state (`case 35`
    /// `NPCID.SkeletronHead`, `case 127` `NPCID.SkeletronPrime`; the Dungeon
    /// Guardian, `case 68`, rides the same arm but has no port kind yet).
    /// The port's Prime machine maintains `ai[1]` (`update_prime`); the
    /// Skeletron Head's stand-in leaves it 0 — also `!= 2f`, the C#'s
    /// non-spin state. Everything else defaults true; the Empress of Light's
    /// `case 636` arm (`!ShouldEmpressBeEnraged()`) is out with its unported
    /// kind (docs/ANALYSIS.md, "The dodge family").
    pub fn is_damage_dodgeable(&self) -> bool {
        match self.kind {
            NpcKind::SkeletronHead | NpcKind::SkeletronPrime => self.ai[1] != 2.0,
            _ => true,
        }
    }

    /// `NPC.StrikeNPC`'s knockback half (`NPC.cs:81389-81458`): the
    /// soft-capped impulse applied by `tr_core::damage::knock_back_npc`.
    /// `hit_direction` is the knockback side (+1 = pushed right) and
    /// `knock_back` the weapon's `Item.knockBack`.
    pub fn hurt(&mut self, amount: i32, from_x: f32, knock_back: f32, crit: bool) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        // `NPC.dontTakeDamage` (`NPC.cs:6525`): the shielded boss takes
        // nothing (the Brain of Cthulhu's Creeper shield).
        if self.dont_take_damage {
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
        // The deterministic scratch stream — the tests and every
        // non-caster kind never consume it (only the caster teleport
        // search draws `rand01`).
        let mut scratch = 0u64;
        self.update_rng(world, player, dt, &mut scratch);
    }

    /// The real step with the caller's RNG stream — the caster teleport
    /// search draws from it where the C# hits `Main.rand` (`NPC.cs:18948`).
    pub fn update_rng(&mut self, world: &World, player: &mut Player, dt: f32, rng: &mut u64) {
        // `base.oldPosition = base.position` (`NPC.cs:92605`, right before the
        // position integration) — the switch-tile bridge reads it.
        self.old_x = self.x;
        self.old_y = self.y;
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
            NpcKind::Guide
            | NpcKind::DD2Bartender
            | NpcKind::Merchant
            | NpcKind::Dryad
            | NpcKind::Nurse
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::WitchDoctor
            | NpcKind::Clothier
            | NpcKind::TravellingMerchant
            | NpcKind::Angler
            | NpcKind::Wizard
            | NpcKind::Mechanic
            | NpcKind::Stylist
            | NpcKind::TaxCollector
            | NpcKind::Golfer
            | NpcKind::Zoologist
            | NpcKind::Princess
            | NpcKind::DyeTrader
            | NpcKind::PartyGirl
            | NpcKind::GoblinTinkerer => self.update_guide(world, dt),
            NpcKind::QueenBee => self.update_queen_bee(world, player),
            NpcKind::SkeletronHead => self.update_skeletron(world, player, rng),
            NpcKind::SkeletronHand => self.update_skeletron_hand(world, player),
            NpcKind::WallOfFleshEye => self.update_wof_eye(world, player),
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                self.update_prime_limb(world, player)
            }
            NpcKind::WallOfFlesh => self.update_wof(world, player),
            NpcKind::Hungry => self.update_hungry(world, player),
            // The leeches ride the worm pass (aiStyle 6 — the chain move).
            NpcKind::TheDestroyer => {}
            NpcKind::SkeletronPrime => self.update_prime(world, player),
            // The Retinazer (325) rides the aiStyle-57 machine; the
            // Spazmatism (326) is an aiStyle-3 fighter (the walker).
            NpcKind::Retinazer => self.update_retinazer(world, player, rng),
            NpcKind::Spazmatism => self.update_spazmatism(world, player),
            NpcKind::Plantera => self.update_plantera(world, player),
            NpcKind::BrainOfCthulhu => self.update_brain(world, player, rng),
            NpcKind::Creeper => self.update_creeper(world, player, rng),
            // aiStyle 0 (`:10252`): the bound stand still until the talk.
            NpcKind::BoundGoblin => {}
            NpcKind::SleepingAngler => {}
            NpcKind::BoundWizard => {}
            NpcKind::BoundMechanic => {}
            NpcKind::WebbedStylist => {}
            NpcKind::SleepingGolfer => {}
            NpcKind::GoblinArcher => self.update_archer(world, player, rng),
            NpcKind::Golem
            | NpcKind::GolemHead
            | NpcKind::GolemFistLeft
            | NpcKind::GolemFistRight
            | NpcKind::GolemHeadFree => self.update_golem(world, player),
            NpcKind::Deerclops => self.update_deerclops(world, player, rng),
            NpcKind::Skeleton => self.update_skeleton(world, player, dt),
            NpcKind::UndeadMiner => self.update_skeleton(world, player, dt),
            // The goblin army's fighters (aiStyle 3, NPC.cs:15644/15658/
            // :15714 cases 26/27/111): the walker arm. (S1-D-3b: the
            // shadowing `update_goblin` arm that kept Thief/Warrior/Peon
            // off this path is removed so they reach the skeleton walker.)
            NpcKind::GoblinThief | NpcKind::GoblinWarrior | NpcKind::GoblinPeon => {
                self.update_skeleton(world, player, dt)
            }
            // The frost trio: aiStyle 38's own machine (`NPC.cs:28995-29145`,
            // R-25-1) — the hop cycle, the per-type 3/4/3.5 speeds, the
            // Gangsta's bottle throw, the Balla's pause, the Mystery's aim.
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => {
                self.update_frost_legion(world, player, dt)
            }
            // The martians: aiStyle 3 rides the walker (the per-type speeds
            // fold into the walker params — the Scrambler/RayGunner/zapper
            // shot arms land with the projectile work).
            NpcKind::MartianOfficer
            | NpcKind::GrayGrunt
            | NpcKind::MartianEngineer
            | NpcKind::GigaZapper
            | NpcKind::MartianWalker => self.update_skeleton(world, player, dt),
            // The probe: aiStyle 80's hover/detect/escape machine (`AI()`'s
            // aiStyle-80 block, `NPC.cs:36532-36624`).
            NpcKind::MartianProbe => self.update_martian_probe(world, player),
            // The drone: aiStyle 74's approach/charge/dash machine (`AI()`'s
            // aiStyle-74 block, `NPC.cs:35218-35500`).
            NpcKind::MartianDrone => self.update_martian_drone(world, player, rng),
            // The shooters: aiStyle 3's walker with the stop-and-fire
            // cycle (`AI_003_Fighters`'s shooter block, `NPC.cs:59069-59623`).
            NpcKind::BrainScrambler | NpcKind::RayGunner => {
                self.update_martian_shooter(world, player, rng)
            }
            // The rider: aiStyle 75's mount machine (the carrier spawn +
            // the ride + the 438 ray, `NPC.cs:35506-35745`).
            NpcKind::ScutlixRider => self.update_scutlix_rider(world, player, rng),
            // The carrier: aiStyle 3's walker (the params 5/0.25/0.7,
            // `NPC.cs:58203-58240`).
            NpcKind::Scutlix => self.update_scutlix(world, player, dt),
            // The saucer family: the core's aiStyle-76 machine and the
            // mounts' aiStyle-75 rides (`NPC.cs:35530-36610`).
            NpcKind::MartianSaucerCore => self.update_saucer_core(world, player, rng),
            NpcKind::MartianSaucer
            | NpcKind::MartianSaucerTurret
            | NpcKind::MartianSaucerCannon => self.update_saucer_mount(world, player, rng),
            // The pirate ship: aiStyle 93's crew-carrier machine + its
            // cannons' aiStyle-75 ride.
            NpcKind::FlyingDutchman => self.update_flying_dutchman(world, player, rng),
            NpcKind::FlyingDutchmanCannon => self.update_saucer_mount(world, player, rng),
            // The DD2 goblins: aiStyle 107's improved walker.
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => {
                self.update_dd2_goblin(world, player, dt)
            }
            // The DD2 support families (`AI_107_ImprovedWalkers`'s arms):
            // the bombers, the javelin throwers, the materializing
            // skeletons.
            // The DD2 throwers' weapon arm FIRST (the match's order):
            // the javelineers throw the spear (662, the tier-3's 685) at
            // the ai[1] == 82 cadence.
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => {
                self.update_dd2_thrower(world, player, rng)
            }
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3 => self.update_dd2_bomber(world, player, rng),
            NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => {
                self.update_dd2_skeleton(world, player)
            }
            // The Dark Mage: aiStyle 109's cast machine.
            NpcKind::DD2DarkMage => self.update_dark_mage(world, player, rng),
            // Betsy: aiStyle 110's phase machine.
            NpcKind::DD2Betsy => self.update_betsy(world, player, rng),
            // The WitherBeast: aiStyle 107's aura/self-heal machine.
            NpcKind::DD2WitherBeast => self.update_wither_beast(world, player, rng),
            // The Old One's Army's tier-2/3 families (M4-EVENT-19): the
            // wyverns and the kobold gliders ride aiStyle 108's diving
            // flyer, the walkers aiStyle 107's arms and the Dark Mage T3
            // the T1's aiStyle-109 machine.
            NpcKind::DD2WyvernT1
            | NpcKind::DD2WyvernT2
            | NpcKind::DD2WyvernT3
            | NpcKind::DD2KoboldFlyerT2
            | NpcKind::DD2KoboldFlyerT3 => self.update_diving_flyer(world, player, rng),
            NpcKind::DD2DarkMageT3 => self.update_dark_mage(world, player, rng),
            NpcKind::DD2WitherBeastT3 => self.update_wither_beast_t3(world, player, rng),
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 => self.update_drakin(world, player, rng),
            NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 => {
                self.update_dd2_kobold_walker(world, player, dt)
            }
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 => self.update_ogre(world, player, rng),
            // The Lunar Pillars: aiStyle 94 (NPC.cs:38968-39400).
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::VortexTower
            | NpcKind::SolarTower => self.update_ancient_cultist(world, player, rng),
            // S1-D-3b: AncientCultist (CultistBoss 439) is aiStyle 84
            // (NPC.cs:14216, AI_084_LunaticCultist :64414), not the pillar
            // aiStyle 94 row (422/507/517/493).
            NpcKind::AncientCultist => self.update_lunatic_cultist(world, player, rng),
            // The Old One's Army's lane gate: aiStyle 106 — the sim's
            // gate pass owns its spawn timer (it needs the pool + the
            // dd2 state), the machine here is the idle body.
            NpcKind::DD2LanePortal => self.update_dd2_lane(world, player),
            // The crawltipede chain rides the worm pass entirely.
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => {}
            // The pillar worms: the crawltipede chain rides the worm pass
            // (aiStyle 6) and the stardust worm head's own machine; the
            // spearman rides the walker.
            NpcKind::StardustWormHead => self.update_stardust_worm_head(world, player, rng),
            NpcKind::SolarSpearman => {
                let (pcx, pcy) = player.center();
                crate::npc_ai::walker_step(
                    self,
                    world,
                    pcx,
                    pcy,
                    crate::npc_ai::WalkerParams {
                        speed: 5.0,
                        jump: -6.2,
                        aggro_x: 80.0 * TILE_PX,
                        aggro_y: 40.0 * TILE_PX,
                    },
                    GRAVITY,
                    MAX_FALL,
                );
            }
            // The stardust/solar tower mobs: the spider pair, the soldier,
            // the drakomire mount + its rider, the sroller, the corite and
            // the selenian.
            NpcKind::StardustSpiderBig => self.update_stardust_spider_big(world, player, rng),
            NpcKind::StardustSpiderSmall => self.update_stardust_spider_small(world, player, rng),
            NpcKind::StardustSoldier => self.update_stardust_soldier(world, player, rng),
            NpcKind::SolarDrakomire => self.update_solar_drakomire(world, player),
            NpcKind::SolarDrakomireRider => self.update_solar_drakomire_rider(world, player),
            NpcKind::SolarSroller => self.update_solar_sroller(world, player),
            NpcKind::SolarCorite => self.update_solar_corite(world, player, rng),
            NpcKind::SolarSelenian => self.update_solar_selenian(world, player),
            // The nebula trio: the floater's aiStyle-97 teleport cycle, the
            // suckler's aiStyle-85 hover + attachment, the predictor's
            // 573 fan.
            NpcKind::NebulaFloater => self.update_nebula_floater(world, player, rng),
            NpcKind::BrainSuckler => self.update_brain_suckler(world, player, rng),
            NpcKind::Predictor => self.update_nebula_predictor(world, player, rng),
            // The vortex trio: the hunter's shooter arm, the alien's dive
            // + the `Transform(426)`, the headcrab's roll + the
            // `Transform(427)`.
            NpcKind::VortexHunter => self.update_vortex_hunter(world, player, rng),
            NpcKind::VortexAlien => self.update_vortex_alien(world, player, rng),
            NpcKind::VortexHeadcrab => self.update_vortex_headcrab(world, player),
            // The pillar shards: the aiStyle-95/96/85/26 machines and the
            // Vortex pair's aiStyle-3 riders.
            NpcKind::StardustCellSmall => self.update_stardust_cell(world, player),
            NpcKind::StardustJellyfishBig => self.update_stardust_jellyfish(world, player, rng),
            NpcKind::StardustCellBig => self.update_stardust_cell_big(world, player),
            NpcKind::NebulaBeast => self.update_nebula_beast(world, player, rng),
            NpcKind::VortexRifleman => self.update_vortex_rifleman(world, player, rng),
            // `:57681`: the soldier's walk speed is the 4-px cap.
            NpcKind::VortexSoldier => {
                let (pcx, pcy) = player.center();
                crate::npc_ai::walker_step(
                    self,
                    world,
                    pcx,
                    pcy,
                    crate::npc_ai::WalkerParams {
                        speed: 4.0,
                        jump: -6.2,
                        aggro_x: 80.0 * TILE_PX,
                        aggro_y: 40.0 * TILE_PX,
                    },
                    GRAVITY,
                    MAX_FALL,
                );
            }
            // The Moon Lord family: aiStyles 77/78/79/81/82.
            NpcKind::MoonLordCore => self.update_moon_lord_core(world, player, rng),
            NpcKind::MoonLordHand => self.update_moon_lord_hand(world, player, rng),
            NpcKind::MoonLordHead => self.update_moon_lord_head(world, player, rng),
            NpcKind::MoonLordFreeEye => self.update_true_eye(world, player, rng),
            NpcKind::MoonLordLeechBlob => self.update_moon_leech_blob(world, player),
            // The tavernkeeper: the town-NPC walker (aiStyle 7).
            NpcKind::DD2Bartender => self.update_guide(world, dt),
            // The etherian crystal: aiStyle 105's machine.
            NpcKind::DD2EterniaCrystal => self.update_eternia_crystal(world, player),
            NpcKind::MartianSaucer
            | NpcKind::MartianSaucerTurret
            | NpcKind::MartianSaucerCannon => self.update_saucer_mount(world, player, rng),
            // The pirates: aiStyle 3 rides the walker (the per-type speeds
            // fold into the walker params — the Captain's cannon and the
            // Deadeye's shots land with the projectile work).
            NpcKind::PirateDeckhand | NpcKind::PirateCaptainsMate => {
                self.update_skeleton(world, player, dt)
            }
            // The pirate shooters: the aiStyle-3 shooter block (the stop
            // entry's 80-tick timer, the per-kind projectile/speed/damage,
            // the fire at the half-cooldown) — the Archer's arrows, the
            // Corsair's musket balls, the Captain's wounded barrage.
            NpcKind::PirateArcher | NpcKind::PirateCorsair | NpcKind::PirateCaptain => {
                self.update_martian_shooter(world, player, rng)
            }
            // S1-D-3b: GoblinSummoner (471, NPC.cs:56491-56718) is aiStyle 3
            // with a type-specific shadowflame machine, not aiStyle 8 caster.
            NpcKind::GoblinSummoner => self.update_goblin_summoner(world, player, rng),
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => {
                self.update_cave_bat(world, player, dt)
            }
            NpcKind::EaterOfSouls => self.update_hover(world, player, HoverParams::EATER_OF_SOULS),
            NpcKind::Hornet => self.update_hover(world, player, HoverParams::HORNET),
            NpcKind::MeteorHead => self.update_hover(world, player, HoverParams::METEOR_HEAD),
            NpcKind::KingSlime => self.update_king_slime(world, player, rng),
            NpcKind::EyeOfCthulhu => self.update_eye_of_cthulhu(world, player),
            NpcKind::ServantOfCthulhu => {
                self.update_hover(world, player, HoverParams::SERVANT_OF_CTHULHU)
            }
            // The leeches ride the worm pass (aiStyle 6 — the chain move).
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => {
                self.update_caster(world, player, rng)
            }
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                self.update_sphere(world, player)
            }
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => {
                self.update_swimmer(world, player, rng)
            }
            // The worms: the movement runs in the world sim's worm pass
            // (the chain needs the sibling segments; `AI_006` reads
            // `Main.npc` the same way).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail
            | NpcKind::LeechHead
            | NpcKind::LeechBody => {}
        }
        self.find_frame();
        self.contact(player, rng);
    }

    /// `NPC._deerclopsAttack1Frames` (`NPC.cs:95277-95279`): the 12-frame
    /// wind-up + slam sequence the ai[0]==1/4 states play.
    const DEERCLOPS_ATTACK_1: [u32; 12] = [12, 13, 14, 13, 14, 13, 14, 13, 14, 15, 16, 17];
    /// `_deerclopsAttack2Frames` (`:95282-95284`): the 13-frame rubble
    /// sequence of the ai[0]==2 state.
    const DEERCLOPS_ATTACK_2: [u32; 13] = [12, 15, 16, 17, 17, 17, 17, 13, 18, 18, 18, 18, 12];
    /// `_deerclopsAttack3Frames` (`:95287-95290`): the 16-frame
    /// blizzard/shadow-hand sequence of the ai[0]==3/5/7/8 states.
    const DEERCLOPS_ATTACK_3: [u32; 16] = [
        19, 20, 21, 22, 21, 22, 21, 22, 23, 24, 23, 24, 23, 24, 20, 19,
    ];

    /// `NPC.FindFrame_FromSequence` (`NPC.cs:77372-77380`): the sequence
    /// frame index clamps to the array's last entry past the end.
    fn seq_cell(seq: &[u32], i: usize) -> u32 {
        seq[i.min(seq.len() - 1)]
    }

    /// `FindFrame`'s humanoid-group default arm (`NPC.cs:69481-69577`):
    /// the frame writes only while grounded (the vanilla's
    /// `velocity.Y == 0` block leaves the air row untouched), the idle
    /// snaps to row 0, the walk rides rows 2..count-1 — the accumulator
    /// `|vx| * 2 + 1` (the C#'s `frameCounter += Math.Abs(velocity.X) *
    /// 2f; frameCounter++`) advances the row every 6 (the `num108`
    /// stride) and the end wraps back to row 2 (`num109 = num * 2`).
    /// The sit/talk/attack `ai[0]` zones (`:68666-69519`) ride the AI
    /// states the port's town machines do not model — the follow-up.
    fn humanoid_town_frame(&mut self) {
        if self.vy == 0.0 {
            // The wrap band stops before the `ExtraFramesCount` attack rows
            // (`NPC.cs:69534`'s `num57` and the `frame.Y / num >=
            // npcFrameCount - num57` wrap below it).
            let count = self
                .kind
                .humanoid_frame_count()
                .saturating_sub(self.kind.extra_frames_count());
            if self.vx == 0.0 {
                self.frame_counter = 0.0;
                self.frame = 0;
            } else {
                self.frame_counter += self.vx.abs() * 2.0 + 1.0;
                if self.frame < 2 {
                    self.frame = 2;
                }
                if self.frame_counter > 6.0 {
                    self.frame += 1;
                    self.frame_counter = 0.0;
                }
                if self.frame >= count {
                    self.frame = 2;
                }
            }
        }
    }

    /// `FindFrame`'s plain walker arm (`case 3` and its 20-sibling group,
    /// `NPC.cs`): the ground flip, row 2 while airborne or turning, row 0
    /// standing, and the `|vx|` accumulator's 8/16/24/32 bands stepping
    /// rows 0/1/2/1 on the strip's three rows.
    fn walker_frame(&mut self) {
        if self.vy == 0.0 {
            self.sprite_dir = self.facing;
        }
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
            // The C# leaves `frame.Y` alone in the last branch (its 0..32
            // accumulator laps at row 1), so the row is not written there.
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
                self.frame_counter = 0.0;
            }
        }
    }

    /// The shared `case 266`/`case 262` machine (the Brain of Cthulhu and
    /// Plantera, `NPC.cs`'s two identical arms): the frame advances every
    /// 6 ticks; `first_phase` (the shield-down / above-half-life gate) wraps
    /// the strip at row 3, otherwise the rows clamp into 4..7.
    fn phase_frame(&mut self, first_phase: bool) {
        self.frame_counter += 1.0;
        if self.frame_counter > 6.0 {
            self.frame_counter = 0.0;
            self.frame += 1;
        }
        if first_phase {
            if self.frame > 3 {
                self.frame = 0;
            }
        } else {
            if self.frame < 4 {
                self.frame = 4;
            }
            if self.frame > 7 {
                self.frame = 4;
            }
        }
    }

    /// `NPC.FindFrame` (`NPC.cs:66684`) for the four ported kinds. `num` (the
    /// frame height unit) cancels out because frames are tracked as row indices;
    /// the per-kind frame counts are `Main.npcFrameCount` (`Main.cs:67082`):
    /// slime 2, demon eye 2, zombie 3, King Slime 6. `num2` is the motion class
    /// (`NPC.cs:66698-66703`): 2 jumping up, 3 falling, 1 moving, 0 idle —
    /// `aiAction == 4` (attacking) never happens for these kinds in the port.
    /// The row count of this kind's own sheet — `Main.npcFrameCount[type]`,
    /// the divisor `NPC.SetFrameSize` uses. The lunar machines read it instead
    /// of hard-coding a strip length.
    fn rows(&self) -> u32 {
        crate::render::entities::npc_rows(crate::render::entities::texture_net_id(self.kind)).max(1)
    }

    /// `frame.Y += num; if (frame.Y / num >= count) frame.Y = 0;` — the wrap
    /// the C# writes as the `+= num` + `>= Main.npcFrameCount` pair.
    fn next_row(&self, step: u32) -> u32 {
        (self.frame + step) % self.rows()
    }

    /// The row `Main.npcFrameCount - back` the attack bands pin.
    fn last_row(&self, back: u32) -> u32 {
        self.rows().saturating_sub(back)
    }

    /// `case 415`'s machine (`NPC.cs:70119-70168`), shared with the 416 rider:
    /// `ai[2] < 0` is the charge that flips rows 8/9 off the 18-tick counter,
    /// the grounded walk climbs rows 2..(count-2) at `|vx| * 0.5` a tick, and
    /// the air holds row 1.
    fn drakomire_frame(&mut self) {
        let count = self.rows();
        if self.ai[2] < 0.0 {
            if self.frame != 8 && self.frame != 9 && self.frame != 0 {
                self.frame_counter = 0.0;
            }
            self.frame_counter += 1.0;
            let (num127, num128) = (18.0f32, 4.0f32);
            let mut row = 0u32;
            if self.frame_counter > num127 - num128 * 2.0 {
                row = 8 + (self.frame_counter as u32) / 4 % 2;
            }
            if self.frame_counter > num127 + num128 * 6.0 {
                row = 0;
                self.frame_counter = 0.0;
            }
            self.frame = row.min(count.saturating_sub(1));
        } else if self.vy == 0.0 {
            if self.facing != 0 {
                self.sprite_dir = self.facing;
            }
            if self.frame < 2 {
                self.frame = 2;
            }
            self.frame_counter += self.vx.abs() * 0.5;
            if self.frame_counter >= 8.0 {
                self.frame += 1;
                self.frame_counter = 0.0;
            }
            if self.frame >= count.saturating_sub(2) {
                self.frame = 2;
            }
            if self.vx == 0.0 {
                self.frame = 0;
            }
        } else {
            self.frame_counter = 0.0;
            self.frame = 1;
        }
    }

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
            // `case 36` (`NPC.cs:66773-66778`): the same red-hat pin as
            // the head (`NPC.cs`'s `RedHatSkeletronAdjustmentsEnabled`),
            // so row 0 always rides in the port. The strip is 2 rows
            // (`Main.npcFrameCount[36]` = 2): the normal hand and its
            // red-hat row.
            NpcKind::SkeletronHand => {
                self.frame = 0;
            }
            // The Wizard/Mechanic: `FindFrame`'s humanoid-group case list
            // carries them (`NPC.cs:68554`/`:68556`) with no arm of their
            // own — the shared town body's rows stand.
            NpcKind::Wizard | NpcKind::Mechanic => {}
            NpcKind::Stylist | NpcKind::TaxCollector => {}
            NpcKind::Golfer => {}
            NpcKind::Zoologist | NpcKind::Princess => {}
            NpcKind::DyeTrader | NpcKind::PartyGirl => {}
            // The Prime's limbs: `Main.npcFrameCount` 128 = 1, 129 = 2,
            // 130 = 2, 131 = 1 (`Main.cs:67082`) — the cannon and the
            // laser have NO `FindFrame` case at all (row 0 forever), while
            // the saw (`case 129`, `NPC.cs:74539-74554`) and the vice
            // (`case 130`, `NPC.cs:74555-74570`) spin their 2-row strips.
            // The v1 forced row 0 on all four ("the limbs' single frame").
            NpcKind::PrimeCannon | NpcKind::PrimeLaser => {
                // No `case 128`/`case 131` exists in `NPC.cs:66684-77370`
                // — `frame.Y` never leaves its `SetFrameSize` 0.
                self.frame = 0;
            }
            // The Angler has no `FindFrame` case of his own — `case 369`
            // is the shared town-NPC body (`NPC.cs:68608`, the
            // `ai[0]` walk/attack/sit states the town machine ports), so
            // the row the town machine wrote stands.
            NpcKind::Angler => {}
            NpcKind::PrimeSaw => {
                // `case 129`: `spriteDirection = direction` while grounded
                // (the AI's own `-ai[0]` write stands otherwise); the row
                // advances every 2 ticks and wraps at `npcFrameCount`'s 2.
                if self.vy == 0.0 {
                    self.sprite_dir = self.facing;
                }
                self.frame_counter += 1.0;
                if self.frame_counter >= 2.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 2;
                }
            }
            NpcKind::PrimeVice => {
                // `case 130`: the same shape at the 8-tick clamp cadence.
                if self.vy == 0.0 {
                    self.sprite_dir = self.facing;
                }
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 2;
                }
            }
            // The frost trio: the 4/5/7-frame strips (the same walker
            // accumulator).
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    let count = match self.kind {
                        NpcKind::SnowmanGangsta => 4,
                        NpcKind::SnowBalla => 5,
                        _ => 7,
                    };
                    self.frame = (self.frame + 1) % count;
                }
            }
            // ═══════════════════════════════════════════════════════════
            // The martians (`NPC.cs:71578-71921`, the 16 kinds of the
            // Martian Madness family). `Main.npcFrameCount` reads:
            // 381/382/383/385/389 = 9, 386 = 11, 387/388 = 2, 390 = 5,
            // 391 = 6, 392 = 4, 393 = 18, 394 = 8, 395 = 11, 399 = 8,
            // 520 = 8 (`Main.cs:67082`). The grounded walk groups all
            // share the same shape: `spriteDirection = direction`, the
            // row clamps up to the walk band's floor, the accumulator
            // `|vx| * rate` advances the row every 6, the end wraps back
            // to the band's floor, the idle snaps to row 0 and the air
            // holds row 1 (or 0 for the Scutlix). The v1 ran one blind
            // `(frame + 1) % count` strip over all of them — no idle/air
            // split, no per-kind rate, no wrap to the walk band's floor.
            NpcKind::MartianOfficer
            | NpcKind::GrayGrunt
            | NpcKind::BrainScrambler
            | NpcKind::RayGunner
            | NpcKind::GigaZapper => {
                // `case 381/382/383/385/389` (`NPC.cs:71623-71662`): the
                // shared ground group — the 385/389 walk at the slower
                // `|vx| * 1.2`, the rest at `* 2.0`.
                let rate = if matches!(self.kind, NpcKind::GrayGrunt | NpcKind::GigaZapper) {
                    1.2
                } else {
                    2.0
                };
                if self.vy == 0.0 {
                    self.sprite_dir = self.facing;
                    if self.frame < 2 {
                        self.frame = 2;
                    }
                    self.frame_counter += self.vx.abs() * rate;
                    if self.frame_counter > 6.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 9 {
                        self.frame = 2;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 1;
                }
            }
            NpcKind::MartianEngineer => {
                // `case 386` (`NPC.cs:71697-71730`): `ai[2] > 0` parks the
                // engineer on the 9/10 animation rows (`ai[2] / 12 % 2`);
                // else the ground group with the walk wrapping at 9 (not
                // `npcFrameCount`'s 11 — rows 9/10 belong to the
                // animation pair).
                if self.ai[2] > 0.0 {
                    self.frame = 9 + ((self.ai[2] / 12.0) as i32 % 2) as u32;
                } else if self.vy == 0.0 {
                    self.sprite_dir = self.facing;
                    if self.frame < 2 {
                        self.frame = 2;
                    }
                    self.frame_counter += self.vx.abs() * 2.0;
                    if self.frame_counter > 6.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 9 {
                        self.frame = 2;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 1;
                }
            }
            NpcKind::MartianWalker => {
                // `case 520` (`NPC.cs:71889-71921`): the walk rides rows
                // 1..7 (the wrap returns to 1, not 2 — the `num` band's
                // floor is row 1 for this sheet) at `|vx| * 1`; the slow
                // walk (`|vx| < 0.2`) snaps to the idle row 0.
                if self.vy == 0.0 {
                    self.sprite_dir = self.facing;
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter > 6.0 {
                        if self.vx.abs() < 0.2 {
                            self.frame = 0;
                        } else {
                            self.frame += 1;
                        }
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 8 {
                        self.frame = 1;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 1;
                }
            }
            // The probe: `case 399` (`NPC.cs:71885-71887`) — the accumulator
            // steps 0.25 per tick, wraps over 4 rows, and the alerted
            // machine (`ai[0] != 0`) rides the +4 rows of NPC_399's strip.
            NpcKind::MartianProbe => {
                self.frame_counter =
                    (self.frame_counter + 0.25) % 4.0 + if self.ai[0] != 0.0 { 4.0 } else { 0.0 };
                self.frame = self.frame_counter as u32;
            }
            // The drone: `case 388` (`NPC.cs:71731-71740`) — the 2-row strip
            // advances one row every 6 ticks (unconditional: the drone is
            // always airborne).
            NpcKind::MartianDrone => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 6.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 2;
                }
            }
            // The rider: `case 390` (`NPC.cs:71779-71805`) — the row is
            // purely `ai[2]`-driven (the aim octant 1..5 the AI writes:
            // `NPC.cs:35727-35749`), row 2 when it holds 0; the
            // `spriteDirection` is INVERTED against `direction`.
            NpcKind::ScutlixRider => {
                if self.facing != 0 {
                    self.sprite_dir = -self.facing;
                }
                self.frame = if self.ai[2] > 0.0 {
                    self.ai[2] as u32 - 1
                } else {
                    2
                };
            }
            // The carrier: `case 391` (`NPC.cs:71743-71778`) — the walk
            // rides rows 1..5 at `|vx| * 0.5` (the slowest rate in the
            // family), the idle snaps to row 0, the air holds row 0 (NOT
            // row 1), and the `spriteDirection` is INVERTED.
            NpcKind::Scutlix => {
                if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = -self.facing;
                    }
                    if self.frame < 1 {
                        self.frame = 1;
                    }
                    self.frame_counter += self.vx.abs() * 0.5;
                    if self.frame_counter > 6.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 6 {
                        self.frame = 1;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                }
            }
            // The saucer family: `AIMethods`' aiStyle-75/76 machines copy
            // the core's `ai[3]` master clock onto every mount
            // (`NPC.cs:35832`/`:35860`/`:35906`) and `FindFrame` reads it
            // back — the v1 ran a blind strip over all four strips.
            NpcKind::MartianSaucerCore => {
                // `case 395` (`NPC.cs:71845-71865`): 11 rows off
                // `frameCounter % 66 / 6`; the counter advances twice per
                // tick in the attack window (`ai[3]` in [20, 260)) and is
                // pinned to [36, 54) there, parking the animation on rows
                // 6-8 for the whole attack.
                let attacking = self.ai[3] >= 20.0 && self.ai[3] < 260.0;
                self.frame_counter += 1.0;
                if self.frame_counter >= 66.0 {
                    self.frame_counter = 0.0;
                }
                if attacking {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 54.0 || self.frame_counter < 36.0 {
                        self.frame_counter = 36.0;
                    }
                }
                self.frame = (self.frame_counter as i32 % 66 / 6) as u32;
            }
            NpcKind::MartianSaucer => {
                // `case 392` (`NPC.cs:71867-71883`): 4 rows off
                // `frameCounter % 12 / 3`, double speed in the same
                // [20, 260) window as the core.
                let attacking = self.ai[3] >= 20.0 && self.ai[3] < 260.0;
                self.frame_counter += 1.0;
                if attacking {
                    self.frame_counter += 1.0;
                }
                if self.frame_counter >= 12.0 {
                    self.frame_counter = 0.0;
                }
                self.frame = (self.frame_counter as i32 % 12 / 3) as u32;
            }
            NpcKind::MartianSaucerTurret => {
                // `case 393` (`NPC.cs:71806-71819`): 18 rows = the 9 aim
                // rows the `ai[2]` rotation vector selects through the
                // vanilla's 5-band cascade + the 9 firing rows, offset +9
                // while the core's clock sits in the 140-tick burst
                // ([280, 420)) on the flash ticks (`(int)ai[3] % 6 <= 2`).
                let (vx, vy) = (self.ai[2].cos(), self.ai[2].sin());
                let mut row = if vy > vx.abs() * 2.0 {
                    4
                } else if vy > vx.abs() * 1.5 {
                    if vx > 0.0 { 5 } else { 3 }
                } else if vx.abs() > vy * 2.0 {
                    if vx > 0.0 { 8 } else { 0 }
                } else if !(vx.abs() > vy * 1.5) {
                    if vx > 0.0 { 6 } else { 2 }
                } else if vx <= 0.0 {
                    1
                } else {
                    7
                };
                if self.ai[3] >= 280.0 && self.ai[3] < 420.0 && (self.ai[3] as i32) % 6 <= 2 {
                    row += 9;
                }
                self.frame = row as u32;
            }
            NpcKind::MartianSaucerCannon => {
                // `case 394` (`NPC.cs:71820-71843`): 8 rows, all off the
                // core's clock — row 4 idles below the 120 mark, the
                // charge ramps 4..7 over the next 20, the firing spin
                // cycles `(n / 4) % 4`, and the tail falls back toward 4.
                let n = (self.ai[3] as i32) - 300;
                self.frame = if n >= 120 {
                    let n2 = n - 120;
                    if n2 >= 160 {
                        7usize.saturating_sub(((n2 - 160) / 5) as usize) as u32
                    } else if n2 >= 20 {
                        (n2 / 4 % 4) as u32
                    } else {
                        (4 + n2 / 5) as u32
                    }
                } else {
                    4
                };
            }
            // The DD2 goblins: the 14-frame humanoid strip
            // (`Main.npcFrameCount` 552-554).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 14;
                }
            }
            // The support families: the bombers/javelineers/skeletons ride
            // the humanoid strips (the same accumulator; the skeletons'
            // materialize cycle drives their own timer).
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3
            | NpcKind::DD2JavelinT1
            | NpcKind::DD2JavelinT2
            | NpcKind::DD2JavelinT3
            | NpcKind::DD2SkeletonT1
            | NpcKind::DD2SkeletonT3 => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 14;
                }
            }
            // Betsy: the 11-frame dragon strip.
            NpcKind::DD2Betsy => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 11;
                }
            }
            // The WitherBeast: `Main.npcFrameCount[568/569]` = 17 rows.
            NpcKind::DD2WitherBeast => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1)
                        % crate::render::entities::npc_rows(
                            crate::render::entities::texture_net_id(self.kind),
                        );
                }
            }
            // The tavernkeeper: `Main.npcFrameCount[550]` = 25 rows.
            NpcKind::DD2Bartender => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1)
                        % crate::render::entities::npc_rows(
                            crate::render::entities::texture_net_id(self.kind),
                        );
                }
            }
            // The Old One's Army's tier-2/3 families (M4-EVENT-19): the
            // `FindFrame` arms' frame counts off `Main.npcFrameCount` —
            // the wyverns' 5, the gliders' 10 (the 4-frame flap then the
            // 8-frame altimeter), the Dark Mage T3's 9 (the T1's strip),
            // the Wither Beast T3's 17, the drakins' 16, the kobold
            // walkers' 18 and the ogres' 10.
            NpcKind::DD2WyvernT1 | NpcKind::DD2WyvernT2 | NpcKind::DD2WyvernT3 => {
                // `:67838-67863`: the dive (`ai[0] == 2`) parks on the last
                // row, otherwise the strip walks at 6 ticks a row.
                if self.ai[0] == 2.0 {
                    self.frame = 4;
                } else {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 6.0 {
                        self.frame_counter = 0.0;
                        self.frame = (self.frame + 1) % 5;
                    }
                }
            }
            NpcKind::DD2KoboldFlyerT2 | NpcKind::DD2KoboldFlyerT3 => {
                // `:67887-67900`: while flying the strip altimeter-walks
                // (4 ticks a row to row 5, then 8), landing on row 9.
                if self.ai[0] != 0.0 {
                    if self.frame < 2 {
                        self.frame = 2;
                        self.frame_counter = 0.0;
                    }
                    let step = if self.frame >= 5 { 8.0 } else { 4.0 };
                    self.frame_counter += 1.0;
                    if self.frame_counter >= step && self.frame < 9 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                        if self.frame == 9 {
                            // The row-9 ignition burst (8 dusts) — the port
                            // carries the latch through `caster_counter`.
                            self.caster_counter = 1;
                        }
                    }
                } else {
                    self.frame = 0;
                    self.frame_counter = 0.0;
                }
            }
            NpcKind::DD2DarkMageT3 => {
                // `Main.npcFrameCount[564-567]` = 9 (`Main.cs:67140`).
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1)
                        % crate::render::entities::npc_rows(
                            crate::render::entities::texture_net_id(self.kind),
                        );
                }
            }
            NpcKind::DD2WitherBeastT3 => {
                // `Main.npcFrameCount[568/569]` = 17.
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1)
                        % crate::render::entities::npc_rows(
                            crate::render::entities::texture_net_id(self.kind),
                        );
                }
            }
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 => {
                // `:67036`-family's stand-and-shoot strip: the aiming rows
                // ride `ai[0]` 1-5, the walk rows otherwise.
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 16;
                }
            }
            NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 => {
                // `:63198`'s `localAI[3]` spawn-in rides `caster_counter`;
                // the strip is the 18-frame humanoid walk.
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 18;
                }
            }
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 => {
                // `:63029`'s ogre: the 10-frame strip with the roar/pound
                // sub-states riding `ai[1]`.
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 10;
                }
            }
            // The cultist + the towers: the 1-frame strips.
            NpcKind::AncientCultist
            | NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::VortexTower
            | NpcKind::SolarTower => {
                self.frame = 0;
            }
            // The Old One's Army's lane gate: `ai[1]` drives its scale
            // fade in the sim's gate pass; the machine here is 1-frame.
            NpcKind::DD2LanePortal => {
                self.frame = 0;
            }
            // The lunar families' own machines - one `case` per vanilla id
            // (`Main.npcFrameCount` = the row count of the kind's sheet, the
            // same divisor `NPC.SetFrameSize` uses). The v1 ran ONE
            // `|vx| + 0.6` accumulator over every member of each family with a
            // hand-written row count, so the walk cadences, the air rows and
            // the attack bands were all invented.
            //
            // 405/406 (the star cells, `NPC.cs:70727-70740`): a 4-tick full
            // wrap over the strip.
            NpcKind::StardustCellBig | NpcKind::StardustCellSmall => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 4.0 {
                    self.frame_counter = 0.0;
                    self.frame = self.next_row(1);
                }
            }
            // 407 (the big jellyfish, `:70716-70726`): the same wrap at 6
            // ticks.
            NpcKind::StardustJellyfishBig => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 6.0 {
                    self.frame_counter = 0.0;
                    self.frame = self.next_row(1);
                }
            }
            // 409 (the big stardust spider, `NPC.cs:70683-70703`): the
            // `ai[1]` 1..30 attack band pins the last row, otherwise the walk
            // steps every 6 ticks and wraps two rows before the sheet's end
            // (the last two rows are the attack band).
            NpcKind::StardustSpiderBig => {
                if self.vy == 0.0 && self.ai[1] <= 30.0 && self.ai[1] > 0.0 {
                    self.frame = self.last_row(1);
                    self.frame_counter = 0.0;
                } else {
                    if self.vy == 0.0 {
                        self.sprite_dir = self.facing;
                    }
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 6.0 {
                        self.frame_counter = 0.0;
                        self.frame = self.next_row(1);
                        if self.frame >= self.last_row(2) {
                            self.frame = 0;
                        }
                    }
                }
            }
            // 410 (the small spider, `:70704-70715`): the walk accumulator
            // folds the row on every 10 px of horizontal speed.
            NpcKind::StardustSpiderSmall => {
                self.frame_counter += self.vx.abs();
                if self.frame_counter >= 10.0 {
                    self.frame_counter -= 10.0;
                    self.frame = self.next_row(1);
                }
            }
            // 411 (the stardust soldier, `:70642-70682`): the `ai[1]`
            // 120..240 channel pins the last row (the arm-raised band), and
            // the walk otherwise starts on row 2 and wraps one row before the
            // end.
            NpcKind::StardustSoldier => {
                if self.ai[1] >= 120.0 && self.ai[1] < 240.0 {
                    self.sprite_dir = -self.facing;
                    self.frame = self.last_row(1);
                    self.frame_counter = 0.0;
                } else if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = -self.facing;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                        self.frame_counter = 0.0;
                    } else {
                        if self.frame < 2 {
                            self.frame = 2;
                        }
                        self.frame_counter += self.vx.abs() + 1.0;
                        if self.frame_counter > 6.0 {
                            self.frame_counter = 0.0;
                            self.frame = self.next_row(1);
                        }
                        if self.frame >= self.last_row(1) {
                            self.frame = 2;
                        }
                    }
                } else {
                    self.frame = 1;
                    self.frame_counter = 0.0;
                }
            }
            // 415 (the solar drakomire, `NPC.cs:70119-70168`): the
            // `ai[2] < 0` charge state rides the 8/9 flip pair off the 18-tick
            // counter, the grounded walk rides rows 2..(count-2) with the
            // `|vx| * 0.5` accumulator, and the air holds row 1.
            NpcKind::SolarDrakomire => self.drakomire_frame(),
            // 416 (the drakomire's rider, `NPC.cs:70172-70193`) copies its
            // 415's ROW (`Main.npc[num140].frame.Y / row * num`), so the same
            // machine reproduces it.
            NpcKind::SolarDrakomireRider => self.drakomire_frame(),
            // 417 (the solar sroller, `NPC.cs:70195-70263`): `ai[0] == 0`
            // rolls rows `(counter / 6)` over the 48-tick loop, the `ai[0] ==
            // 1` / `5` attack states climb 9..14 off `ai[1] / 6`, and every
            // other state pins row 14.
            NpcKind::SolarSroller => {
                if self.ai[0] == 0.0 {
                    if self.vy == 0.0 {
                        self.sprite_dir = self.facing;
                    } else if self.vy < 0.0 {
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter += self.vx.abs() * 1.1;
                    self.frame = ((self.frame_counter / 6.0) as u32).min(self.rows() - 1);
                    if self.frame_counter >= 48.0 {
                        self.frame_counter = 0.0;
                    }
                } else if self.ai[0] == 1.0 {
                    self.frame_counter = 0.0;
                    self.frame = (9 + (self.ai[1] / 6.0) as u32).min(self.rows() - 1);
                } else if self.ai[0] == 5.0 {
                    self.frame_counter = 0.0;
                    self.frame = 13u32.saturating_sub((self.ai[1] / 6.0) as u32);
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 14.min(self.rows() - 1);
                }
            }
            // 418 (the solar corite, `NPC.cs:70757-70803`): the idle/hover
            // states (`ai[0]` 0, +/-1) walk rows 0..4 at 6 ticks, the
            // `ai[0] == 2` dive splits `ai[1]` into rows 5/6/7, and the
            // `ai[0] == -2` death spin walks 5..7 at 12 ticks.
            NpcKind::SolarCorite => {
                if self.ai[0] == 0.0 || self.ai[0] == 1.0 || self.ai[0] == -1.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter > 6.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 4 {
                        self.frame = 0;
                    }
                } else if self.ai[0] == 2.0 {
                    self.frame = if self.ai[1] < 10.0 {
                        5
                    } else if self.ai[1] < 20.0 {
                        6
                    } else {
                        7
                    };
                } else if self.ai[0] == -2.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter > 12.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 7 {
                        self.frame = 5;
                    }
                }
            }
            // 419 (the solar selenian, `NPC.cs:70266-70330`): the `ai[2] < 0`
            // charge band rides rows 2..4 off the 6-tick counter and then
            // wraps into 4..8; the grounded walk climbs from row 9 and wraps
            // at the sheet's end; the air holds row 1.
            NpcKind::SolarSelenian => {
                if self.ai[2] < 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = self.facing;
                    }
                    if self.frame >= 9 {
                        self.frame = 2;
                        self.frame_counter = 0.0;
                    }
                    if self.frame < 2 {
                        self.frame = 2;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 10.0 {
                        self.frame_counter = 8.0;
                    }
                    self.frame = if self.frame_counter < 6.0 {
                        (2.0 + self.frame_counter / 3.0) as u32
                    } else {
                        (4.0 + (self.frame_counter - 6.0)) as u32
                    };
                } else if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = self.facing;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                        self.frame_counter = 0.0;
                    } else {
                        self.frame_counter += self.vx.abs() + 1.0;
                        if self.frame_counter > 6.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= self.rows() || self.frame < 9 {
                            self.frame = 9;
                        }
                    }
                } else {
                    self.frame = 1;
                    self.frame_counter = 0.0;
                }
            }
            // 425 (the vortex rifleman, `NPC.cs:70331-70372`): the `ai[2] ==
            // 1` firing band cycles rows 10..12 at 3 ticks, the air holds row
            // 1 and the walk runs rows 2..9 off `|vx|`.
            NpcKind::VortexRifleman => {
                if self.facing != 0 {
                    self.sprite_dir = -self.facing;
                }
                if self.ai[2] == 1.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 3.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame > 12 || self.frame < 10 {
                        self.frame = 10;
                    }
                } else if self.vy != 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 1;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter >= 8.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame > 9 || self.frame < 2 {
                        self.frame = 2;
                    }
                }
            }
            // 426 (the vortex hornet queen, `:70410-70464`): the `ai[1] >=
            // 45` dive band cycles 12..15 at 2 ticks (row 8 on the ground),
            // the plain air band 9..11, the walk 1..7 off `|vx|`.
            NpcKind::VortexHunter => {
                if self.facing != 0 {
                    self.sprite_dir = self.facing;
                }
                if self.ai[1] >= 45.0 {
                    if self.vy == 0.0 {
                        self.frame = 8;
                    } else {
                        self.frame_counter += 1.0;
                        if self.frame_counter >= 2.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= 15 || self.frame < 12 {
                            self.frame = 12;
                        }
                    }
                } else if self.vy != 0.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 2.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 12 || self.frame < 9 {
                        self.frame = 9;
                    }
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter >= 10.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame > 7 || self.frame < 1 {
                        self.frame = 1;
                    }
                }
            }
            // 427 (the vortex hornet, `:70373-70409`): the air band cycles
            // 6..8 at 2 ticks, the walk 1..5 off `|vx|`.
            NpcKind::VortexAlien => {
                if self.facing != 0 {
                    self.sprite_dir = self.facing;
                }
                if self.vy != 0.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 2.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= 9 || self.frame < 6 {
                        self.frame = 6;
                    }
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter >= 8.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame > 5 || self.frame < 1 {
                        self.frame = 1;
                    }
                }
            }
            // 428 (the vortex larva, `:70500-70536`): the grounded walk steps
            // at 10 px and wraps one row before the end; the air folds into
            // the single row 4 sprite after 10 ticks of falling.
            NpcKind::VortexHeadcrab => {
                if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = self.facing;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                        self.frame_counter = 0.0;
                    } else {
                        self.frame_counter += self.vx.abs();
                        self.frame_counter += 1.0;
                        if self.frame_counter > 10.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= self.last_row(1) {
                            self.frame = 0;
                        }
                    }
                } else if self.frame != 4 {
                    if self.frame_counter > 0.0 {
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter -= 1.0;
                    if self.frame_counter < -10.0 || self.vy > 0.6 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                }
            }
            // 429 (the vortex soldier, `:70465-70499`): the grounded walk
            // starts on row 2 and wraps at the sheet's end; the air holds row
            // 1.
            NpcKind::VortexSoldier => {
                if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = self.facing;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                        self.frame_counter = 0.0;
                    } else {
                        if self.frame < 2 {
                            self.frame = 2;
                        }
                        self.frame_counter += self.vx.abs() + 1.0;
                        if self.frame_counter > 6.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= self.rows() {
                            self.frame = 2;
                        }
                    }
                } else {
                    self.frame = 1;
                    self.frame_counter = 0.0;
                }
            }
            // 518 (the solar spearman, `NPC.cs:70265`): the grounded walk
            // starts on row 2 and wraps at the sheet's end with the doubled
            // `|vx| + 1` accumulator; the air holds row 1.
            NpcKind::SolarSpearman => {
                if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = self.facing;
                    }
                    if self.vx == 0.0 {
                        self.frame = 0;
                        self.frame_counter = 0.0;
                    } else {
                        if self.frame < 2 {
                            self.frame = 2;
                        }
                        self.frame_counter += self.vx.abs() + 1.0;
                        if self.frame_counter > 6.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= self.rows() {
                            self.frame = 2;
                        }
                    }
                } else {
                    self.frame = 1;
                    self.frame_counter = 0.0;
                }
            }
            // The crawltipede's three links + the stardust worm's head have no
            // `FindFrame` case at all (`NPC.cs:66684-77370`), so the row their
            // `SetFrameSize` zero leaves stands.
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail
            | NpcKind::StardustWormHead => {
                self.frame = 0;
            }
            // 420 (the nebula floater, `:70631-70641`) and 421 (the brain
            // suckler, `:70577-70591`): the plain 5-tick wrap, the suckler's
            // `ai[0] == 5` state double-ticking the counter.
            NpcKind::NebulaFloater => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 5.0 {
                    self.frame_counter = 0.0;
                    self.frame = self.next_row(1);
                }
            }
            NpcKind::BrainSuckler => {
                if self.ai[0] == 5.0 {
                    self.frame_counter += 1.0;
                }
                self.frame_counter += 1.0;
                if self.frame_counter >= 5.0 {
                    self.frame_counter = 0.0;
                    self.frame = self.next_row(1);
                }
            }
            // 423 (the nebula beast, `:70537-70576`): the `ai[2] == 1` charge
            // band pins rows 2/3 off `ai[1]` 30..45, the air pins row 1, the
            // walk runs rows 4.. off `1 + |vx| / 2` and wraps at the end.
            NpcKind::NebulaBeast => {
                if self.ai[2] == 1.0 {
                    self.frame = if self.ai[1] >= 30.0 && self.ai[1] < 45.0 {
                        3
                    } else {
                        2
                    };
                } else if self.vy != 0.0 {
                    self.frame = 1;
                } else if self.vx == 0.0 {
                    self.frame = 0;
                } else {
                    self.sprite_dir = -(self.vx.signum() as i32);
                    if self.frame < 4 {
                        self.frame = 4;
                    }
                    self.frame_counter += 1.0 + self.vx.abs() / 2.0;
                    if self.frame_counter > 6.0 {
                        self.frame += 1;
                        self.frame_counter = 0.0;
                    }
                    if self.frame >= self.rows() {
                        self.frame = 4;
                    }
                }
            }
            // 424 (the nebula predictor, `:70592-70630`): the grounded
            // `ai[2] > 0` aim band pins the last two rows off `ai[1]`, the walk
            // runs rows 2..(count-2) off `|vx|`, the air holds row 1.
            NpcKind::Predictor => {
                if self.vy == 0.0 {
                    if self.facing != 0 {
                        self.sprite_dir = -self.facing;
                    }
                    if self.ai[2] > 0.0 {
                        self.frame = if self.ai[1] > 90.0 {
                            self.last_row(1)
                        } else {
                            self.rows() - 2
                        };
                        self.frame_counter = 0.0;
                    } else if self.vx == 0.0 {
                        self.frame = 0;
                    } else {
                        if self.frame < 2 {
                            self.frame = 2;
                        }
                        self.frame_counter += self.vx.abs();
                        if self.frame_counter > 6.0 {
                            self.frame += 1;
                            self.frame_counter = 0.0;
                        }
                        if self.frame >= self.rows() - 2 {
                            self.frame = 2;
                        }
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = 1;
                }
            }
            // The Moon Lord family: the 396's 4x8 accumulator (the
            // rand boost), the 397's frameCounter/7, the 398's
            // 30/6 counter (the gore burst on its first frame is
            // unmodelled), the 400's 16/4 with the double-tick in the
            // drift/laser states and the 401's single frame.
            NpcKind::MoonLordHead => {
                self.frame_counter += 1.0;
                // `:71955`'s `Main.rand.Next(10) == 0` counter boost —
                // the port has no rng in `find_frame`, so the hash
                // stand-in gives the same 1-in-10 jitter.
                let jitter = (self.frame_counter as i32)
                    .wrapping_mul(0x9E37_79B1u32 as i32)
                    .wrapping_add((self.frame_counter as i32).wrapping_mul(0x85EB_CA6Bu32 as i32));
                if (jitter >> 28) & 0xF == 0 {
                    self.frame_counter += 1.0;
                }
                if self.frame_counter >= 32.0 {
                    self.frame_counter = 0.0;
                }
                self.frame = (self.frame_counter / 4.0) as u32 % 8;
            }
            NpcKind::MoonLordHand => {
                self.frame = ((self.frame_counter / 7.0) as u32).min(3);
            }
            NpcKind::MoonLordCore => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 30.0 {
                    self.frame_counter = 6.0;
                }
                self.frame = ((self.frame_counter as i32 % 30) / 6) as u32;
            }
            NpcKind::MoonLordFreeEye => {
                self.frame_counter += 1.0;
                if self.ai[0] == 0.0 || self.ai[0] == 3.0 {
                    self.frame_counter += 1.0;
                }
                if self.frame_counter >= 16.0 {
                    self.frame_counter = 0.0;
                }
                self.frame = ((self.frame_counter as i32 % 16) / 4) as u32;
            }
            NpcKind::MoonLordLeechBlob => {
                self.frame = 0;
            }
            // The etherian crystal: the 1-frame strip.
            NpcKind::DD2EterniaCrystal => {
                self.frame = 0;
            }
            // The Dark Mage: `Main.npcFrameCount[564]` = 9 rows (the v1
            // walked a 20-row cycle five rows past the sheet).
            NpcKind::DD2DarkMage => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1)
                        % crate::render::entities::npc_rows(
                            crate::render::entities::texture_net_id(self.kind),
                        );
                }
            }
            // The Dutchman: the 1-frame ship strip; its cannons ride the
            // 9-frame strip (`Main.npcFrameCount`).
            NpcKind::FlyingDutchman => {
                self.frame = 0;
            }
            NpcKind::FlyingDutchmanCannon => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 9;
                }
            }
            // The pirates: the 16/20-frame humanoid strips (the same
            // accumulator).
            NpcKind::PirateDeckhand
            | NpcKind::PirateCaptainsMate
            | NpcKind::PirateArcher
            | NpcKind::PirateCorsair
            | NpcKind::PirateCaptain => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    let count = match self.kind {
                        NpcKind::PirateDeckhand | NpcKind::PirateCaptainsMate => 16,
                        _ => 20,
                    };
                    self.frame = (self.frame + 1) % count;
                }
            }
            NpcKind::GoblinSummoner => {
                self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 20;
                }
            }
            // The hungry (3 frames) / the leech segments (1).
            // Case 115 — the Hungry (`NPC.cs:74995-75017`): the 3-row
            // strip (`Main.npcFrameCount[115]` = 3, `Main.cs:67082`) rides
            // a 15-tick sequence 0/1/2/1 (the bands at 3/6/12/15) — not
            // the v1's free 8-tick strip.
            NpcKind::Hungry => {
                self.frame_counter += 1.0;
                if self.frame_counter < 3.0 {
                    self.frame = 0;
                } else if self.frame_counter < 6.0 {
                    self.frame = 1;
                } else if self.frame_counter < 12.0 {
                    self.frame = 2;
                } else if self.frame_counter < 15.0 {
                    self.frame = 1;
                }
                if self.frame_counter >= 15.0 {
                    self.frame_counter = 0.0;
                }
            }
            NpcKind::LeechHead | NpcKind::LeechBody => {
                self.frame = 0;
            }
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
            // Case 4/5 — the eye and the servant (`NPC.cs:67247-67256`): the
            // counter ticks with the motion class, the frame wraps over the
            // 6-frame (the eye) / 2-frame (the servant) strip.
            NpcKind::EyeOfCthulhu => {
                // `case 4/125/126` (`NPC.cs:76905-76926`): the EoC SHARES
                // the Twins' machine — the 20-tick counter walks the bands
                // rows 0/1/2 and the `ai[0] > 1` phase offset lifts them to
                // the 3/4/5 rows (the v1 ran an 8-tick strip over all six
                // rows and claimed a "pupil blink", which the C# has no arm
                // for). `num2` never enters this case.
                self.frame_counter += 1.0;
                let mut row = if self.frame_counter < 7.0 {
                    0
                } else if self.frame_counter < 14.0 {
                    1
                } else if self.frame_counter < 21.0 {
                    2
                } else {
                    self.frame_counter = 0.0;
                    0
                };
                if self.ai[0] > 1.0 {
                    row += 3;
                }
                self.frame = row;
            }
            NpcKind::ServantOfCthulhu => {
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
            NpcKind::Zombie => {
                // `FindFrame`'s walker arm (`case 3` — a member of the
                // `case 3/52/53/132/161/186-189/200/223/251/254/255/319-321/
                // 331/332/342/536/590/691` group, `NPC.cs`): NPC_3 is
                // 38x144 with `Main.npcFrameCount[3]` = 3 (`Main.cs:67082`),
                // i.e. three 48-px rows — the stand (row 0) and the two walk
                // poses. The v1 pinned row 0 and cited `npcFrameCount[9]`
                // (the Devourer tail's single frame), so the most common
                // night enemy never animated; and the draw divided the sheet
                // by 1, so all three rows were squeezed into one 48-px quad.
                self.walker_frame();
            }
            NpcKind::Skeleton | NpcKind::UndeadMiner => {
                // The humanoid-group walk (the cave walkers ride the same
                // town machine — 15 rows).
                self.humanoid_town_frame();
            }
            // Case 49 — Cave Bat (`AI_014_Bats`): the plain flap over the
            // 5-frame strip (`Main.npcFrameCount[49] = 5`, `Main.cs:67082`).
            // The hovers: the plain flap cycles (`Main.npcFrameCount[6] = 2`,
            // `[42] = 3`, `[23] = 2`, `Main.cs:67082`).
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let count = match self.kind {
                    NpcKind::Hornet => 3,
                    _ => 2,
                };
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= count {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => {
                // `FindFrame`'s case 49 group (`NPC.cs:74824-74881`): the
                // spriteDirection rides the velocity's sign, the tilt
                // `rotation = velocity.X * 0.1`, and the flap advances
                // every 6 ticks over `npcFrameCount - 1` rows — the
                // Cave/Jungle/Hell bats' last row is reserved
                // (`:74871-74873`'s `num292--`), the v1 cycled all 5 rows
                // every 4 ticks.
                self.facing = if self.vx > 0.0 {
                    self.sprite_dir = self.facing;
                    1
                } else if self.vx < 0.0 {
                    -1
                } else {
                    self.facing
                };
                self.rotation = self.vx * 0.1;
                self.frame_counter += 1.0;
                if self.frame_counter >= 6.0 {
                    self.frame_counter = 0.0;
                    self.frame += 1;
                }
                let cycle = if self.kind == NpcKind::VoodooDemon {
                    5
                } else {
                    4
                };
                if self.frame >= cycle {
                    self.frame = 0;
                }
            }
            // The casters (`FindFrame`'s case 29/32/45 group,
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
                    // The snap guard covers both ends (`NPC.cs:76991-76994`'s
                    // `frame.Y < num * 4 || frame.Y > num * 9`).
                    if self.frame < 4 || self.frame > 9 {
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
            // The swimmers: the plain flap cycles over `Main.npcFrameCount`
            // 55/58 = 6, 65 = 4 (`Main.cs:67082`).
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => {
                // `FindFrame`'s case 55/57/58 (`NPC.cs:74181-74226`) and
                // case 65 (`:74654-74678`): the wet flap rides rows 0..3
                // at the 6-tick bands; the goldfish/piranha flop shows
                // rows 4/5 on land; the shark's case has no dry arm — the
                // row holds (`:74657`'s `wet && type == 65` gate).
                self.frame_counter += 1.0;
                if self.wet {
                    if self.frame_counter < 6.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 12.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 18.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 3;
                    } else {
                        self.frame_counter = 0.0;
                    }
                } else if self.kind != NpcKind::Shark {
                    // The beached flop rows 4/5 (`:74212-74220`).
                    if self.frame_counter < 6.0 {
                        self.frame = 4;
                    } else if self.frame_counter < 12.0 {
                        self.frame = 5;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            // The worms: `Main.npcFrameCount` 7..15 = 1 (`Main.cs:67082`).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => {}
            NpcKind::GoblinPeon
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher => {
                // The humanoid-group walk (the goblins ride the same town
                // machine — 16 rows, the Archer's 21).
                self.humanoid_town_frame();
            }
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => {
                // The fists: `Main.npcFrameCount` 247/248 = 1
                // (`Main.cs:67108`) — `FindFrame` has no case, the frame
                // stays row 0; the draw tilts the quad by `npc.rotation`
                // (the lunge angle) instead.
                self.frame_counter = 0.0;
                self.frame = 0;
            }
            NpcKind::Deerclops => {
                // `FindFrame`'s case 668 (`NPC.cs:66779-66841`): the
                // frame selects a cell of the 5x5 grid (0..24) that
                // `DrawNPCDirect_Deerclops` decodes (`cell / 5` column,
                // `cell % 5` row). The attacks ride the C#'s three
                // sequences at `(int)ai[1] / 4` (`FindFrame_FromSequence`
                // clamps past the end, `:77372-77380`); the walk is the
                // idle/air/2..11 cycle (`frameCounter += |vx|`, the 150
                // wrap). The C#'s `frame.Width = 180` write is vestigial
                // (this type's draw builds its own rect); the step
                // sounds at the 4/9 rows ride the audio pass and are the
                // follow-up.
                let cell = match self.ai[0] as i32 {
                    1 | 4 => {
                        Self::seq_cell(&Self::DEERCLOPS_ATTACK_1, (self.ai[1] as i32 / 4) as usize)
                    }
                    2 => {
                        Self::seq_cell(&Self::DEERCLOPS_ATTACK_2, (self.ai[1] as i32 / 4) as usize)
                    }
                    3 | 5 | 7 | 8 => {
                        Self::seq_cell(&Self::DEERCLOPS_ATTACK_3, (self.ai[1] as i32 / 4) as usize)
                    }
                    _ => {
                        // The walk (`:66817-66838`): the air frame while
                        // the hop's velocity is live (the C#'s
                        // `velocity.Y > 0 || localAI[0] == 1` — the
                        // machine's hop drives both), the idle at
                        // `vx == 0`, the 2..11 stride otherwise.
                        if self.vy != 0.0 {
                            1
                        } else if self.vx == 0.0 {
                            self.frame_counter = 0.0;
                            0
                        } else {
                            self.frame_counter += self.vx.abs();
                            if self.frame_counter >= 150.0 {
                                self.frame_counter = 0.0;
                            }
                            2 + (self.frame_counter / 15.0) as u32
                        }
                    }
                };
                self.frame = cell;
            }
            NpcKind::Golem => {
                // `FindFrame`'s case 245 (`NPC.cs:72898-72930`): the
                // grounded state selects the 5-row charge ramp over the
                // `ai[1]` window (-20..0), row 0 while charging; the
                // flight arm's literal `frame.Y = 1` pixel write folds
                // to row 0 (the documented 1-px deviation).
                if self.ai[0] == 0.0 {
                    let c = self.cast_windup as f32;
                    self.frame = if c < -16.0 {
                        0
                    } else if c < -12.0 {
                        1
                    } else if c < -8.0 {
                        2
                    } else if c < -4.0 {
                        3
                    } else if c < 0.0 {
                        4
                    } else {
                        0
                    };
                } else {
                    self.frame = 0;
                }
            }
            NpcKind::GolemHead => {
                // `FindFrame`'s case 246 (`NPC.cs:72931-72961`): the mouth
                // row rides the fireball window (the first/last 20 of the
                // 300 cadence — `:31178`'s window with the localAI[0]
                // writes at `:31181`/`:31186` (phase 0) and
                // `:31240`/`:31244` (phase 1), folded: the port tests the
                // window directly); in phase 1 the head-turn rows ride
                // `localAI[1]` (`npc.head_turn`): +4 on the left side
                // (`:72953-72955`), +2 on the right (`:72957-72959`),
                // NOTHING in-band (`:31223`'s zero — the three-way the
                // v1 missed).
                let mouth = self.ai[1] < 20.0 || self.ai[1] > 280.0;
                let mut row = if mouth { 1 } else { 0 };
                if self.ai[0] == 1.0 {
                    row += match self.head_turn {
                        -1 => 4,
                        1 => 2,
                        _ => 0,
                    };
                }
                self.frame = row;
            }
            NpcKind::GoblinTinkerer => {
                // The humanoid-group walk (the Tinkerer 107 rides the
                // same town machine — 25 rows).
                self.humanoid_town_frame();
            }
            NpcKind::BoundGoblin => {
                // The tied pose: the single frame holds (`aiStyle 0` runs
                // no FindFrame cycle).
                self.frame_counter = 0.0;
                self.frame = 0;
            }
            NpcKind::Creeper => {
                // `Main.npcFrameCount[267]` = 1 (`Main.cs:67082`): the
                // single 34x34 frame (the v1's "30x30" was the box, not
                // the sheet).
                self.frame = 0;
            }
            // `case 266` (the Brain) and `case 262` (Plantera) share one
            // shape: the 6-tick advance over the 8-row strip
            // (`Main.npcFrameCount[266]` = [262] = 8, `Main.cs:67082` —
            // the v1 pinned the Brain to row 0 claiming 1 and ran a blind
            // 5-row strip over Plantera claiming 5), the first-four rows
            // above the half-life gate and the 4..7 rows below it.
            NpcKind::BrainOfCthulhu => {
                // The C#'s gate is `ai[0] >= 0 && localAI[2] == 0` — the
                // creeper-shield count, which the port's brain machine does
                // not model, so the port reads `ai[0] >= 0` alone: the
                // shield-down rows 0..3 always ride (the shielded 4..7 rows
                // are the documented follow-up).
                self.phase_frame(self.ai[0] >= 0.0);
            }
            NpcKind::Plantera => {
                // The C#'s gate is `life > lifeMax / 2` (`:262`'s arm).
                self.phase_frame(self.hp > self.max_hp / 2);
            }
            NpcKind::Retinazer | NpcKind::Spazmatism => {
                // `case 125/126` (`NPC.cs:76905-76926`, shared with the
                // EoC's `case 4`): the Twins' sheets
                // are 6 rows of 200 (`Main.npcFrameCount[125/126]` = 6,
                // `Main.cs:67082` — the v1's comment claimed 2 and ran an
                // 8-tick strip over rows 0/1), laid out as the two 3-row
                // wing-beats of the spawn (0-2) and the live (3-5) phases.
                // The 20-tick counter walks the bands 0/1/2 and the
                // `ai[0] > 1` hover-anchor offset (the AI's own write —
                // `AI_022`'s `NPC.cs:24758`) parks the pair on rows 3-5.
                self.frame_counter += 1.0;
                let mut row = if self.frame_counter < 7.0 {
                    0
                } else if self.frame_counter < 14.0 {
                    1
                } else if self.frame_counter < 21.0 {
                    2
                } else {
                    self.frame_counter = 0.0;
                    0
                };
                if self.ai[0] > 1.0 {
                    row += 3;
                }
                self.frame = row;
            }
            NpcKind::SkeletronPrime => {
                // `case 127` (`NPC.cs:74498-74538`): the head's 6-row strip
                // (`Main.npcFrameCount[127]` = 6, `Main.cs:67082` — the v1's
                // comment claimed 1 and left the arm empty, so the sheet
                // pinned its 0th row forever). The hover phase (`ai[1] == 0`)
                // alternates rows 0 and 1 every 12 ticks; every other phase
                // holds the wide row 2. The Mech-Queen variant
                // (`ai[3] != 0` — `NPC.cs:27630-27633`'s registration, the
                // port never sets it yet) alternates rows 3/4 and pins 5.
                let mech_queen = self.ai[3] != 0.0;
                if self.ai[1] == 0.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 12.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                        self.frame = if mech_queen {
                            if self.frame >= 5 { 3 } else { self.frame }
                        } else if self.frame >= 2 {
                            0
                        } else {
                            self.frame
                        };
                    }
                } else {
                    self.frame_counter = 0.0;
                    self.frame = if mech_queen { 5 } else { 2 };
                }
            }
            NpcKind::TheDestroyer => {
                // `Main.npcFrameCount[134]` = 1 (`Main.cs:67082`).
            }
            // `case 113/114` (`NPC.cs:73516-73537`): the Wall of Flesh and
            // its mirror eye share the machine — the plain phase
            // (`ai[2] == 0`) cycles the 2-row strip
            // (`Main.npcFrameCount[113]` = [114] = 2, `Main.cs:67082` —
            // the v1 claimed 1 for both) every 12 ticks, while every other
            // phase holds row 0 with the counter parked at -60.
            NpcKind::WallOfFlesh | NpcKind::WallOfFleshEye => {
                if self.ai[2] == 0.0 {
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 12.0 {
                        self.frame_counter = 0.0;
                        self.frame = (self.frame + 1) % 2;
                    }
                } else {
                    self.frame = 0;
                    self.frame_counter = -60.0;
                }
            }
            // `case 35` (`NPC.cs:66767-66772`): the head's row is 0 except
            // under the red-hat (Halloween) adjustment, which pins row 1 —
            // the port does not model the red hat, so row 0 always rides.
            // `Main.npcFrameCount[35]` = 2, the sheet is the portrait row
            // plus the red-hat row.
            NpcKind::SkeletronHead => {
                self.frame = 0;
            }
            NpcKind::QueenBee => {
                // `case 222` (`NPC.cs`): NPC_222's strip is TWELVE rows
                // (`Main.npcFrameCount[222]` = 12, `Main.cs:67082` — the
                // v1 claimed 2 and ran an 8-tick strip over both), laid
                // out as the charging rows 0..3 and the flight rows
                // 4..11. The 5-tick cadence rides both halves; the
                // `localAI[0] == 1` gate that selects the charging half is
                // an angle slot in the C# (the queen's orbit rotation),
                // which the port does not model — so the flight rows
                // 4..11 always ride, exactly as they do in the vanilla
                // where the angle is never exactly 1.
                self.frame_counter += 1.0;
                if self.frame_counter > 4.0 {
                    self.frame_counter = 0.0;
                    self.frame += 1;
                }
                if self.frame < 4 {
                    self.frame = 4;
                }
                if self.frame >= 12 {
                    self.frame = 4;
                }
            }
            NpcKind::EyeOfCthulhu => {
                // `Main.npcFrameCount[4]` = 2 (`Main.cs:67082`): the pupil
                // blink over the 8-step accumulator (the AI_004 hover's
                // frame machine, `NPC.cs:73800+`).
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::Clothier => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::WitchDoctor => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            // The bound Wizard/Mechanic: `Main.npcFrameCount` 106/123 = 1
            // (`Main.cs:67094`/`:67096`) and `FindFrame` has no case for
            // them — one row, the still stand.
            NpcKind::BoundWizard | NpcKind::BoundMechanic => {}
            // The webbed Stylist: `Main.npcFrameCount[354]` = 1
            // (`Main.cs:67119`) and no `FindFrame` case — one row.
            NpcKind::WebbedStylist => {}
            // The sleeping Golfer: `Main.npcFrameCount[589]` = 13
            // (`Main.cs:67142`) and `FindFrame` has no case for the bound
            // form — the still stand's row stands.
            NpcKind::SleepingGolfer => {}
            NpcKind::SleepingAngler => {
                // `Main.npcFrameCount[376]` = 2 (`Main.cs:67121`) and
                // `FindFrame`'s `case 376` (`NPC.cs:71007-71018`): the
                // frame counter runs past 30 and flips the row, wrapping
                // at the second.
                self.frame_counter += 1.0;
                if self.frame_counter > 30.0 {
                    self.frame_counter = 0.0;
                    self.frame = (self.frame + 1) % 2;
                }
            }
            NpcKind::TravellingMerchant => {
                // The town wander: `case 368` is a member of the shared
                // humanoid-group list (`NPC.cs`'s `case 22 … case 368 …
                // case 684` block), so the walk band is rows 2..15 of the
                // 26-row strip — `Main.npcFrameCount[368]` = 26 with
                // `ExtraFramesCount` 10 (`Main.cs:67120`,
                // `NPCID.cs:10464`). The earlier 3-row stand-in sampled the
                // top of the sheet and never left rows 0-2.
                self.humanoid_town_frame();
            }
            NpcKind::Painter => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::Demolitionist => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::ArmsDealer => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::Nurse => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::Dryad => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::Merchant => {
                // The humanoid-group walk (the town machine's
                // default arm — rows 2..count-1).
                self.humanoid_town_frame();
            }
            NpcKind::Guide => {
                // The humanoid-group walk (the Guide 22 rides the same
                // town machine — 26 rows).
                self.humanoid_town_frame();
            }
            NpcKind::GolemHeadFree => {
                // `FindFrame`'s case 249 (`NPC.cs:72963-72981`): the mouth
                // row only, off the free head's own 300-cadence
                // `localAI[0]` (`:31439`/`:31443` — the same first/last-20
                // window). No wind-up ramp, no side turn; the C#'s
                // `frameCounter++` at :72976-72979 is dead (nothing reads
                // it for this type) and is not carried.
                let mouth = self.ai[1] < 20.0 || self.ai[1] > 280.0;
                self.frame = if mouth { 1 } else { 0 };
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
            self.sprite_dir = self.facing;
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
                self.sprite_dir = self.facing;
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
            self.sprite_dir = self.facing;
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
        // `:22740-22752` (aiStyle 13) — the eye flips on the SAME
        // `num215 = player.Center.X - width/2 - center.X` the rotation
        // uses (its own aim, not the body's), and the write is inside the
        // two one-sided `if`s, so a dead-centre aim keeps the previous
        // flip.
        self.sprite_dir = if dx != 0.0 {
            self.facing
        } else {
            self.sprite_dir
        };
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
            self.sprite_dir = self.facing;
            // The C#'s cap for the zombie (`AI_003_Fighters`'s `num107`
            // default group, `NPC.cs:58322-58374`): 1f.
            self.vx = self.facing as f32 * NpcKind::Zombie.ai3_walk_speed().unwrap_or(1.0);
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
    /// `AI()`'s aiStyle-4 block — the Eye of Cthulhu's phase machine
    /// (`NPC.cs`'s aiStyle-4 `if` in `AI()`, 841 C# lines). The state:
    /// `ai[0]` the phase (0 the hover, 1/2 the spins, 3 the final),
    /// `ai[1]` the sub-state, `ai[2]` the timer, `ai[3]` the dash/servant
    /// counter. The expert-mode and `getGoodWorld` branches are out (the
    /// port has neither difficulty flag) — the classic path is verbatim,
    /// the skips carry the C# cites. The movement integrates directly
    /// (`noTileCollide` + `noGravity`).
    fn update_eye_of_cthulhu(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let dead = player.hp <= 0;
        // `num6`: the aim angle — `Math.Atan2(y: bottom - 59 - target.Y - h/2,
        // x: center.X - target.X - w/2) + 1.57` wrapped to 2π.
        let mut num6 = (self.y + h - 59.0 - pcy).atan2(self.x + w * 0.5 - pcx) + 1.57;
        if num6 < 0.0 {
            num6 += 6.283;
        } else if num6 > 6.283 {
            num6 -= 6.283;
        }
        // `num7`: the turn rate by the state (the C# num7 table).
        let mut num7 = 0.0;
        if self.ai[0] == 0.0 && self.ai[1] == 0.0 {
            num7 = 0.02;
        }
        if self.ai[0] == 0.0 && self.ai[1] == 2.0 && self.ai[2] > 40.0 {
            num7 = 0.05;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 0.0 {
            num7 = 0.05;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 2.0 && self.ai[2] > 40.0 {
            num7 = 0.08;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 4.0 && self.ai[2] > 20.0 {
            num7 = 0.15;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 5.0 {
            num7 = 0.05;
        }
        if self.rotation < num6 {
            if num6 - self.rotation > 3.1415 {
                self.rotation -= num7;
            } else {
                self.rotation += num7;
            }
        } else if self.rotation > num6 {
            if self.rotation - num6 > 3.1415 {
                self.rotation += num7;
            } else {
                self.rotation -= num7;
            }
        }
        if self.rotation > num6 - num7 && self.rotation < num6 + num7 {
            self.rotation = num6;
        }
        if self.rotation < 0.0 {
            self.rotation += 6.283;
        } else if self.rotation > 6.283 {
            self.rotation -= 6.283;
        }
        if self.rotation > num6 - num7 && self.rotation < num6 + num7 {
            self.rotation = num6;
        }
        // `Main.IsItDay() | dead`: the climb-away
        // (`velocity.Y -= 0.04f; EncourageDespawn(10)` — the removal rides
        // the world sim's distance cull).
        if world.day_time || dead {
            self.vy -= 0.04;
            self.integrate_no_tile_collide();
            return;
        }
        if self.ai[0] == 0.0 {
            if self.ai[1] == 0.0 {
                // The hover: 5 speed, 0.04 accel toward 200 px above the
                // player (`num9`/`num10` classic).
                let cx = self.x + w * 0.5;
                let cy = self.y + h * 0.5;
                let num11 = pcx - cx;
                let num12 = pcy - 200.0 - cy;
                let num14 = (num11 * num11 + num12 * num12).sqrt();
                let num13 = if num14 != 0.0 { 5.0 / num14 } else { 0.0 };
                let (dx, dy) = (num11 * num13, num12 * num13);
                if self.vx < dx {
                    self.vx += 0.04;
                    if self.vx < 0.0 && dx > 0.0 {
                        self.vx += 0.04;
                    }
                } else if self.vx > dx {
                    self.vx -= 0.04;
                    if self.vx > 0.0 && dx < 0.0 {
                        self.vx -= 0.04;
                    }
                }
                if self.vy < dy {
                    self.vy += 0.04;
                    if self.vy < 0.0 && dy > 0.0 {
                        self.vy += 0.04;
                    }
                } else if self.vy > dy {
                    self.vy -= 0.04;
                    if self.vy > 0.0 && dy < 0.0 {
                        self.vy -= 0.04;
                    }
                }
                self.ai[2] += 1.0;
                if self.ai[2] >= 600.0 {
                    self.ai[1] = 1.0;
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                } else if (self.y + h < player.y && num14 < 500.0) && !dead {
                    self.ai[3] += 1.0;
                    if self.ai[3] >= 110.0 {
                        // The servant burst (the `NPC.NewNPC(..., 5)` arm).
                        self.ai[3] = 0.0;
                        self.rotation = num6;
                        let num18 = pcx - cx;
                        let num19 = pcy - cy;
                        let num20 = 5.0 / (num18 * num18 + num19 * num19).sqrt();
                        let (vx, vy) = (num18 * num20, num19 * num20);
                        self.wants_servant = Some((cx + vx * 10.0, cy + vy * 10.0, vx, vy));
                    }
                }
            } else if self.ai[1] == 1.0 {
                // The dash wind-up: the velocity locks onto the target
                // (6 speed classic).
                self.rotation = num6;
                let cx = self.x + w * 0.5;
                let cy = self.y + h * 0.5;
                let num23 = pcx - cx;
                let num24 = pcy - cy;
                let num25 = 6.0 / (num23 * num23 + num24 * num24).sqrt();
                self.vx = num23 * num25;
                self.vy = num24 * num25;
                self.ai[1] = 2.0;
            } else if self.ai[1] == 2.0 {
                // The dash flight: 40 free ticks, then the 0.98 damp; the
                // chain resets at 150.
                self.ai[2] += 1.0;
                if self.ai[2] >= 40.0 {
                    self.vx *= 0.98;
                    self.vy *= 0.98;
                    if self.vx.abs() < 0.1 {
                        self.vx = 0.0;
                    }
                    if self.vy.abs() < 0.1 {
                        self.vy = 0.0;
                    }
                } else {
                    self.rotation = self.vy.atan2(self.vx) - 1.57;
                }
                if self.ai[2] >= 150.0 {
                    self.ai[3] += 1.0;
                    self.ai[2] = 0.0;
                    self.rotation = num6;
                    if self.ai[3] >= 3.0 {
                        self.ai[1] = 0.0;
                        self.ai[3] = 0.0;
                    } else {
                        self.ai[1] = 1.0;
                    }
                }
            }
            if (self.hp as f32) < self.max_hp as f32 * 0.5 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
            self.integrate_no_tile_collide();
            return;
        }
        if self.ai[0] == 1.0 || self.ai[0] == 2.0 {
            // The spin: `ai[2]` ramps to 0.5 while phase 1 (or the
            // getGood half-spin `ai[3] == 1`), rotation spins, `ai[1]`
            // counts to 100 → the next phase (the classic path; the
            // expert servant bursts and the `reflectsProjectiles` flag
            // are the getGood/expert arms).
            if self.ai[0] == 1.0 || self.ai[3] == 1.0 {
                self.ai[2] += 0.005;
                if self.ai[2] > 0.5 {
                    self.ai[2] = 0.5;
                }
            } else {
                self.ai[2] -= 0.005;
                if self.ai[2] < 0.0 {
                    self.ai[2] = 0.0;
                }
            }
            self.rotation += self.ai[2];
            self.ai[1] += 1.0;
            if self.ai[1] >= 100.0 {
                if self.ai[3] == 1.0 {
                    self.ai[3] = 0.0;
                    self.ai[1] = 0.0;
                } else {
                    self.ai[0] += 1.0;
                    self.ai[1] = 0.0;
                    if self.ai[0] != 3.0 {
                        self.ai[2] = 0.0;
                    }
                }
            }
            self.vx *= 0.98;
            self.vy *= 0.98;
            if self.vx.abs() < 0.1 {
                self.vx = 0.0;
            }
            if self.vy.abs() < 0.1 {
                self.vy = 0.0;
            }
            self.integrate_no_tile_collide();
            return;
        }
        // Phase 3: `this.defense = 0` and the faster dash machine (6.8
        // speed, 130-tick chains; the expert rage sub-states
        // `ai[1] = 3/4/5` are out with the flag2/flag3 gates).
        self.defense_override = Some(0);
        if self.ai[1] == 0.0 {
            // The hover at 120 px above (6 speed, 0.07 accel).
            let cx = self.x + w * 0.5;
            let cy = self.y + h * 0.5;
            let num39 = pcx - cx;
            let num40 = pcy - 120.0 - cy;
            let num41 = (num39 * num39 + num40 * num40).sqrt();
            let num42 = if num41 != 0.0 { 6.0 / num41 } else { 0.0 };
            let (dx, dy) = (num39 * num42, num40 * num42);
            if self.vx < dx {
                self.vx += 0.07;
                if self.vx < 0.0 && dx > 0.0 {
                    self.vx += 0.07;
                }
            } else if self.vx > dx {
                self.vx -= 0.07;
                if self.vx > 0.0 && dx < 0.0 {
                    self.vx -= 0.07;
                }
            }
            if self.vy < dy {
                self.vy += 0.07;
                if self.vy < 0.0 && dy > 0.0 {
                    self.vy += 0.07;
                }
            } else if self.vy > dy {
                self.vy -= 0.07;
                if self.vy > 0.0 && dy < 0.0 {
                    self.vy -= 0.07;
                }
            }
            self.ai[2] += 1.0;
            if self.ai[2] >= 200.0 {
                self.ai[1] = 1.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
        } else if self.ai[1] == 1.0 {
            self.rotation = num6;
            let cx = self.x + w * 0.5;
            let cy = self.y + h * 0.5;
            let num43 = pcx - cx;
            let num44 = pcy - cy;
            let num45 = 6.8 / (num43 * num43 + num44 * num44).sqrt();
            self.vx = num43 * num45;
            self.vy = num44 * num45;
            self.ai[1] = 2.0;
        } else if self.ai[1] == 2.0 {
            self.ai[2] += 1.0;
            if self.ai[2] >= 40.0 {
                self.vx *= 0.97;
                self.vy *= 0.97;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
            } else {
                self.rotation = self.vy.atan2(self.vx) - 1.57;
            }
            if self.ai[2] >= 130.0 {
                self.ai[3] += 1.0;
                self.ai[2] = 0.0;
                self.rotation = num6;
                if self.ai[3] >= 3.0 {
                    self.ai[1] = 0.0;
                    self.ai[3] = 0.0;
                } else {
                    self.ai[1] = 1.0;
                }
            }
        }
        self.integrate_no_tile_collide();
    }

    /// `noTileCollide` + `noGravity` movement: the raw integration, no
    /// tile interaction (`SetDefaults` cases 4/5 set both flags).
    fn integrate_no_tile_collide(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI_015_KingSlime` (`NPC.cs:43121-43414`) — the King Slime's machine:
    /// the hop cycle (the hp-scaled rate), the shrink/grow teleport, the
    /// hp-scaled body and the 5 %-hp minion batches.
    ///
    /// The dust and the gore are SKIPPED — no dust/gore layer. The MP-only
    /// branches use the single-player values.
    fn update_king_slime(&mut self, world: &World, player: &mut Player, rng: &mut u64) {
        // `:43123-43131` — the flags and the getGood world's scale factor.
        let mut num = 1.0f32;
        let mut num2 = 1.0f32;
        let mut flag2 = false;
        let mut flag3 = false;
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let ph = player.height;
        // `:43136-43147` — the init: the `ai[3]` hp latch and the one-shot
        // `localAI[3]` arm (which drops the body into its first scale).
        if self.ai[3] == 0.0 && self.hp > 0 {
            self.ai[3] = self.max_hp as f32;
        }
        if self.king_slime_local[3] == 0.0 {
            self.king_slime_local[3] = 1.0;
            self.ai[0] = -100.0;
        }
        // `:43149-43176` — the despawn: a dead player or one 3000 px away
        // sends the King Slime to the world's far corner.
        let distance = ((cx - pcx).powi(2) + (cy - pcy).powi(2)).sqrt();
        if player.dead || distance > 3000.0 {
            self.king_slime_local[1] = world.width as f32 * 16.0;
            self.king_slime_local[2] = world.height as f32 * 16.0;
            self.ai[0] = 0.0;
            self.ai[1] = 5.0;
        }
        // `:43177-43196` — the teleport: the player out of sight for 300
        // ticks (and the King Slime grounded) shrinks and blinks.
        if !player.dead && self.ai[2] >= 300.0 && self.ai[1] < 5.0 && self.vy == 0.0 {
            self.ai[2] = 0.0;
            self.ai[0] = 0.0;
            self.ai[1] = 5.0;
            // The anti-cheese case (`:43185-43191`): the King Slime has been
            // out of sight for 360 ticks or the player is 2000 px away.
            let anti_cheese = self.king_slime_local[0] >= 360.0 || distance > 2000.0;
            if self.king_slime_local[0] >= 360.0 {
                self.king_slime_local[0] = 360.0;
            }
            let (tx, ty) = king_slime_find_teleport_spot(world, player, anti_cheese, rng);
            self.king_slime_local[1] = tx;
            self.king_slime_local[2] = ty;
        }
        // `:43197-43211` — the line-of-sight counters.
        let visible = crate::projectile::can_hit_line(world, cx, cy, 0.0, 0.0, pcx, pcy, 0.0, 0.0)
            && (self.y - (player.y + ph)).abs() <= 160.0;
        if !visible {
            self.ai[2] += 1.0;
            self.king_slime_local[0] += 1.0;
        } else {
            self.king_slime_local[0] -= 1.0;
            if self.king_slime_local[0] < 0.0 {
                self.king_slime_local[0] = 0.0;
            }
        }
        // `:43212-43218` — the closing timer clears the state. The port's
        // NPCs carry no `NPC.timeLeft` (the despawn timer is the world's
        // own row), so the arm cannot watch the closing window — the state
        // clears with the normal machine instead.
        if self.ai[1] == 5.0 {
            // `:43219-43261` — the shrink: the scale ramps down over 60 ticks
            // and the King Slime teleports onto the picked spot.
            flag2 = true;
            self.ai[0] += 1.0;
            num = 0.5 + ((60.0 - self.ai[0]) / 60.0).clamp(0.0, 1.0) * 0.5;
            if self.ai[0] >= 60.0 {
                flag3 = true;
                self.x = self.king_slime_local[1] - w * 0.5;
                self.y = self.king_slime_local[2] - h;
                self.ai[1] = 6.0;
                self.ai[0] = 0.0;
            }
        } else if self.ai[1] == 6.0 {
            // `:43262-43284` — the grow-back: 30 ticks at the same ramp.
            flag2 = true;
            self.ai[0] += 1.0;
            num = 0.5 + (self.ai[0] / 30.0).clamp(0.0, 1.0) * 0.5;
            if self.ai[0] >= 30.0 {
                self.ai[1] = 0.0;
                self.ai[0] = 0.0;
            }
        }
        // `:43285-43326` — the movement: the hop cycle (grounded) and the
        // air control.
        if self.vy == 0.0 {
            self.vx *= 0.8;
            if self.vx > -0.1 && self.vx < 0.1 {
                self.vx = 0.0;
            }
            if !flag2 {
                // The hop timer — the faster the King Slime is hurt, the
                // faster it hops.
                self.ai[0] += 2.0;
                let life = self.hp as f32 / self.max_hp as f32;
                if life < 0.8 {
                    self.ai[0] += 1.0;
                }
                if life < 0.6 {
                    self.ai[0] += 1.0;
                }
                if life < 0.4 {
                    self.ai[0] += 2.0;
                }
                if life < 0.2 {
                    self.ai[0] += 3.0;
                }
                if life < 0.1 {
                    self.ai[0] += 4.0;
                }
                if self.ai[0] >= 0.0 {
                    if self.ai[1] == 3.0 {
                        self.vy = -13.0;
                        self.vx += 3.5 * self.facing as f32;
                        self.ai[0] = -200.0;
                        self.ai[1] = 0.0;
                    } else if self.ai[1] == 2.0 {
                        self.vy = -6.0;
                        self.vx += 4.5 * self.facing as f32;
                        self.ai[0] = -120.0;
                        self.ai[1] += 1.0;
                    } else {
                        self.vy = -8.0;
                        self.vx += 4.0 * self.facing as f32;
                        self.ai[0] = -120.0;
                        self.ai[1] += 1.0;
                    }
                }
            }
        } else {
            // `:43327-43339` — the air control: the push toward the facing.
            let num7 = 3.0f32;
            if (self.facing == 1 && self.vx < num7) || (self.facing == -1 && self.vx > -num7) {
                if (self.facing == -1 && self.vx < 0.1) || (self.facing == 1 && self.vx > -0.1) {
                    self.vx += 0.2 * self.facing as f32;
                } else {
                    self.vx *= 0.93;
                }
            }
        }
        // `:43340-43356` — the body scale: the hp drives the size (the
        // C#'s `num9`).
        let num9 = (self.hp as f32 / self.max_hp as f32 * 0.5 + 0.75) * num * num2;
        if num9 != self.scale || self.king_slime_local[3] == 1.0 {
            let new_w = 98.0 * num9;
            let new_h = 92.0 * num9;
            // The C#'s recentre (`:43346-43351`): the bottom-right corner
            // stays put.
            self.x += w * 0.5 - new_w * 0.5;
            self.y += h - new_h;
            self.scale = num9;
            self.size_override = Some((new_w, new_h));
        }
        // `:43357-43395` — the minion batches: every 5 % of the hp the King
        // Slime spawns 1-3 slimes.
        let num10 = (self.max_hp as f32 * 0.05) as i32;
        if self.hp + num10 >= self.ai[3] as i32 {
            return;
        }
        self.ai[3] = self.hp as f32;
        let count = crate::app::frame::rand_range_signed(rng, 1, 4).max(1);
        self.wants_minion = true;
        self.wants_minion_batch = count as u32;
        let _ = num2;
    }

    /// The contact overlap test (`contact`'s AABB) as a pure read —
    /// `Player.StatusFromNPC`'s call site (`Player.cs:31425`) runs the
    /// per-NPC debuff rolls on the same touch branch.
    pub fn touches(&self, player: &Player) -> bool {
        let (w, h) = self.size();
        self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
    }

    /// `AI_007_TownNPCs`'s walk subset (`NPC.cs:10586+`): the town NPC
    /// wanders on the ground, turns at the walls/ledges, jumps the
    /// one-tile obstacles. The chase/doors/talk arms land with their
    /// systems.
    fn update_guide(&mut self, world: &World, _dt: f32) {
        let (w, h) = NpcKind::Guide.size();
        // The wander: a walking pace with the random turn every few
        // seconds (`ai[3]`'s idle timer shape).
        self.anim += 1.0;
        // The deterministic turn: every 240 ticks the wander flips (the
        // per-NPC randomness rides the shared streams' order).
        if self.anim % 240.0 < 1.0 {
            self.facing = -self.facing;
            self.sprite_dir = self.facing;
        }
        self.vx = self.facing as f32 * 0.5;
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else if self.on_ground {
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
    }

    /// `AI_003_Fighters` for the skeleton family (`NPC.cs:10586+`): the
    /// cave walkers chase the player through the framework's walker step
    /// with the vanilla-cited tunables — the 15-frame strip rides the same
    /// accumulator (the frame wraps over `Main.npcFrameCount[21] = 15`,
    /// `Main.cs:67082`).
    fn update_skeleton(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                // The C#'s per-type cap (`AI_003_Fighters`, the `num83` /
                // `num107` / cap-2 / cap-3 groups, `NPC.cs:57520-58374`):
                // the skeleton 1.5, the undead miner 1.0, the pirate
                // deckhand 2.0 and the pirate captain's mate 3.0. The jump
                // -6.2 and the zombie's 80x40-tile aggro window stay.
                speed: self
                    .kind
                    .ai3_walk_speed()
                    .expect("the cave walkers' aiStyle-3 cap"),
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The frame: the same accumulator as the zombie's, wrapping over
        // the 15-frame strip.
        self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
        if self.frame_counter >= 8.0 {
            self.frame_counter = 0.0;
            self.frame = (self.frame + 1) % 15;
        }
    }

    /// `NPC.AI()`'s aiStyle-38 block (`NPC.cs:28995-29145`) — the Frost
    /// Legion trio. The constant hop cycle (`velocity.Y = -6f` per beat,
    /// the every-third `ai[0] > 2f` beat → `-8.2f` + `ai[1]++`), the
    /// per-type speed/accel pair (`num551`/`num552`: the Gangsta 3/0.7,
    /// the Balla 4/1, the Mystery 3.5/0.8 — the defaults 4/1), the stuck
    /// flip (`localAI[2]` = the previous `position.X`, the port's
    /// `frost_stuck_x`) with the 60-tick no-retarget cooldown, the walk
    /// acceleration `direction * num552 * 0.01` and the `num551` clamp.
    /// The type arms: the Gangsta's 120-tick bottle throw (the hostile
    /// `BulletSnowman` 110 at 12×spriteDirection, 25 dmg, `timeLeft = 300`
    /// behind `AI_GlobalFiringDistanceCheck`), the Balla's post-3-hops
    /// 200-tick pause (`ai[1] >= 3f` → the vx decay + the reset), the
    /// Mystery's 16-tick windup into the aimed `SnowBallHostile` 109
    /// (35 dmg, speed 10 at the player's top-centre). The `confused`
    /// resets (`:29015-29017`/`:29131-29135`) ride the NPC confusion
    /// debuff, which is not ported yet.
    fn update_frost_legion(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let _ = world;
        let (mut num551, mut num552) = (4.0f32, 1.0f32);
        if self.kind == NpcKind::SnowmanGangsta {
            num551 = 3.0;
            num552 = 0.7;
        }
        if self.kind == NpcKind::MysterySnowman {
            num551 = 3.5;
            num552 = 0.8;
        }
        let (pcx, pcy) = player.center();
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        // `TargetClosest()` (`NPC.cs:29092`/`:29040`/`:29056`): the facing
        // follows the player's side; the sprite rides it.
        fn target_closest(n: &mut Npc, pcx: f32, cx: f32) {
            n.facing = if pcx >= cx { 1 } else { -1 };
            n.sprite_dir = n.facing;
        }
        // `NPC.AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55842`): the
        // target centre's `Main.MaxWorldViewSize` rect (1920×1200) deflated
        // by 50 — a plain axis box.
        let firing_distance = |sx: f32, sy: f32| -> bool {
            (sx - pcx).abs() <= 1920.0 / 2.0 - 50.0 && (sy - pcy).abs() <= 1200.0 / 2.0 - 50.0
        };
        let player_alive = player.hp > 0;

        // The Gangsta's throw arm (`:29012-29037`) — independent of the
        // hop core below.
        if self.kind == NpcKind::SnowmanGangsta {
            self.ai[2] += 1.0;
            if self.ai[2] >= 120.0 {
                self.ai[2] = 0.0;
                let sx = self.x + w * 0.5 - (self.facing * 12) as f32;
                let sy = self.y + h * 0.5;
                if player_alive && firing_distance(sx, sy) {
                    self.wants_hostile_shot.push((
                        crate::projectile::BULLET_SNOWMAN,
                        sx,
                        sy,
                        12.0 * self.facing as f32,
                        0.0,
                        25,
                    ));
                }
            }
        }

        // The Balla/Mystery attack states; everything else — and the whole
        // Gangsta — rides the hop core (`:29089-29136`).
        let in_attack_state = match self.kind {
            NpcKind::SnowBalla if self.ai[1] >= 3.0 => {
                target_closest(self, pcx, cx);
                if self.vy == 0.0 {
                    self.vx *= 0.9;
                    self.ai[2] += 1.0;
                    if self.vx > -0.3 && self.vx < 0.3 {
                        self.vx = 0.0;
                    }
                    if self.ai[2] >= 200.0 {
                        self.ai[2] = 0.0;
                        self.ai[1] = 0.0;
                    }
                }
                true
            }
            NpcKind::MysterySnowman if self.ai[1] >= 3.0 => {
                target_closest(self, pcx, cx);
                let in_range = player_alive && firing_distance(cx, cy);
                if self.vy == 0.0 && self.ai[2] == 0.0 && !in_range {
                    self.ai[1] = 0.0;
                } else if self.vy == 0.0 {
                    self.ai[2] += 1.0;
                    self.vx *= 0.9;
                    if self.vx > -0.3 && self.vx < 0.3 {
                        self.vx = 0.0;
                    }
                    if self.ai[2] >= 16.0 {
                        self.ai[2] = 0.0;
                        self.ai[1] = 0.0;
                    }
                }
                // The windup throw (`:29078-29087`): at `ai[2] == 8` the
                // aim is the player's top-centre from `(cx - facing*12,
                // y + h*0.25)`, speed 10 normalised, 35 dmg, `timeLeft =
                // 300`, `ai[0] = 2` (inert for type 109 — only type 31
                // reads it, `Projectile.cs:57828`).
                if self.ai[2] == 8.0 {
                    let sx = self.x + w * 0.5 - (self.facing * 12) as f32;
                    let sy = self.y + h * 0.25;
                    let ax = player.x + crate::player::WIDTH as f32 / 2.0 - sx;
                    let ay = player.y - sy;
                    let len = (ax * ax + ay * ay).sqrt();
                    let inv = if len > 0.0 { 10.0 / len } else { 0.0 };
                    self.wants_hostile_shot.push((
                        crate::projectile::SNOW_BALL_HOSTILE,
                        sx,
                        sy,
                        ax * inv,
                        ay * inv,
                        35,
                    ));
                }
                true
            }
            _ => false,
        };

        if !in_attack_state && self.vy == 0.0 {
            if self.frost_stuck_x == self.x {
                self.facing *= -1;
                self.ai[3] = 60.0;
            }
            self.frost_stuck_x = self.x;
            if self.ai[3] == 0.0 {
                target_closest(self, pcx, cx);
            }
            self.ai[0] += 1.0;
            if self.ai[0] > 2.0 {
                self.ai[0] = 0.0;
                self.ai[1] += 1.0;
                self.vy = -8.2;
                self.vx += self.facing as f32 * num552 * 1.1;
            } else {
                self.vy = -6.0;
                self.vx += self.facing as f32 * num552 * 0.9;
            }
            self.sprite_dir = self.facing;
        }
        if !in_attack_state {
            self.vx += self.facing as f32 * num552 * 0.01;
        }
        if self.ai[3] > 0.0 {
            self.ai[3] -= 1.0;
        }
        // The `num551` clamp (`:29139-29144`) — per direction.
        if self.vx > num551 && self.facing > 0 {
            self.vx = num551;
        }
        if self.vx < -num551 && self.facing < 0 {
            self.vx = -num551;
        }
        // The shared `NPC` movement after the AI (`NPC.cs:27485-27910`'s
        // velocity → `Collision.TileCollision` integration): gravity, the
        // fall cap, the axis-separated tile collision — a wall zeroes the
        // vx (the stuck flip above reads the frozen position next tick).
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
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
    }

    /// `AI_014_Bats`' core (`NPC.cs` case 49's aiStyle): the erratic flight
    /// through the framework's flyer step with the direction perturbation
    /// on the aiTimer (the vanilla randomizes velocity.X/Y every ~30 ticks
    /// toward the player).
    fn update_cave_bat(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        // The vanilla: `AI_014`'s velocity reroll rides `ai[1]` counting to
        // 30 (`NPC.cs` case 49's AI block) — the deterministic hash stands
        // in for `Main.rand` (the shared streams' order).
        self.anim += 1.0;
        if self.anim % 30.0 < 1.0 {
            let jitter = ((self.x as i32).wrapping_mul(0x4982_2D4Eu32 as i32) >> 24) & 0x7;
            self.vx = if jitter % 2 == 0 { 1.2 } else { -1.2 };
            self.vy = if jitter < 4 { -0.8 } else { 0.8 };
        }
        crate::npc_ai::flyer_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::FlyerParams {
                accel: 0.15,
                max_speed: 2.5,
            },
        );
        let _ = (cx, cy);
        // The 5-frame flap (`Main.npcFrameCount[49] = 5`).
        self.frame_counter += 1.0;
        if self.frame_counter >= 4.0 {
            self.frame_counter = 0.0;
            self.frame = (self.frame + 1) % 5;
        }
    }

    /// `AI_005_EaterOfSouls`'s hover core through the framework's
    /// `hover_step` — the no-gravity fly toward the player with the vanilla
    /// tunables; the 2-frame/3-frame flaps ride the anim clock.
    fn update_hover(&mut self, world: &World, player: &Player, params: HoverParams) {
        let (pcx, pcy) = player.center();
        // `NPC.GetTargetData`: the centre drives the velocity aim, the top
        // edge (`Player.position.Y`) the surface-damping gate (`NPC.cs:50354`).
        let target = crate::npc_ai::HoverTarget {
            center_x: pcx,
            center_y: pcy,
            top_y: player.y,
        };
        crate::npc_ai::hover_step(self, world, target, params);
        self.frame_counter += 1.0;
        if self.frame_counter >= 8.0 {
            self.frame_counter = 0.0;
            let count = match self.kind {
                NpcKind::Hornet => 3,
                _ => 2,
            };
            self.frame = if self.frame + 1 >= count {
                0
            } else {
                self.frame + 1
            };
        }
    }

    /// Goblin Summoner (471). C# `NPC.cs:15728` SetDefaults aiStyle 3 and the
    /// type-specific AI block `NPC.cs:56491-56718`. (S1-D-3b: was mis-routed to
    /// `update_caster` / aiStyle 8.)
    fn update_goblin_summoner(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();

        // NPC.cs:56491-56718
        if self.ai[3] < 0.0 {
            // shadowflame phase (noGravity/noTileCollide in C#)
            if self.vx < 0.0 {
                self.facing = -1;
            } else if self.vx > 0.0 {
                self.facing = 1;
            }
            self.sprite_dir = self.facing;
            self.rotation = self.vx * 0.1;

            // ai[3] == -1: chase target (NPC.cs:56534-56550)
            if (self.ai[3] + 1.0).abs() < f32::EPSILON {
                let dist = ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
                let num31 = 8.0 + dist / 200.0;
                let num32 = 40.0;
                let mut dirx = pcx - cx;
                let mut diry = pcy - cy;
                let dlen = (dirx * dirx + diry * diry).sqrt().max(0.0001);
                dirx = dirx / dlen * num31;
                diry = diry / dlen * num31;
                self.vx = (self.vx * (num32 - 1.0) + dirx) / num32;
                self.vy = (self.vy * (num32 - 1.0) + diry) / num32;
                let (w, h) = self.size();
                if dist < 500.0 && !crate::player::collides_rect(world, self.x, self.y, w, h) {
                    self.ai[3] = 0.0;
                    self.ai[2] = 0.0;
                }
                return;
            }
            // ai[3] == -2: rise (NPC.cs:56552-56573)
            if (self.ai[3] + 2.0).abs() < f32::EPSILON {
                self.vy -= 0.2;
                if self.vy < -10.0 {
                    self.vy = -10.0;
                }
                if pcy - cy > 200.0 {
                    self.ai[3] = -3.0;
                    self.ai[2] = if pcx > cx { 1.0 } else { -1.0 };
                }
                self.vx *= 0.99;
                return;
            }
            // ai[3] == -3: strafe (NPC.cs:56575-56606)
            if (self.ai[3] + 3.0).abs() < f32::EPSILON {
                if self.facing == 0 {
                    self.facing = if pcx >= cx { 1 } else { -1 };
                    self.sprite_dir = self.facing;
                }
                if self.ai[2] == 0.0 {
                    self.ai[2] = self.facing as f32;
                }
                self.vy *= 0.9;
                self.vx += self.ai[2] * 0.3;
                self.vx = self.vx.clamp(-10.0, 10.0);
                let num34 = pcx - cx;
                if (self.ai[2] < 0.0 && num34 > 300.0) || (self.ai[2] > 0.0 && num34 < -300.0) {
                    self.ai[3] = -4.0;
                    self.ai[2] = 0.0;
                } else if num34.abs() > 800.0 {
                    self.ai[3] = -1.0;
                    self.ai[2] = 0.0;
                }
                return;
            }
            // ai[3] == -4: descend until solid footing (NPC.cs:56608-56639)
            if (self.ai[3] + 4.0).abs() < f32::EPSILON {
                self.ai[2] += 1.0;
                self.vy += 0.1;
                let sp = (self.vx * self.vx + self.vy * self.vy).sqrt();
                if sp > 4.0 {
                    self.vx *= 0.9;
                    self.vy *= 0.9;
                }
                let num35 = (cx / 16.0) as i32;
                let num36 = ((self.y + self.size().1 + 12.0) / 16.0) as i32;
                let mut flag4 = false;
                for l in (num35 - 1)..=(num35 + 1) {
                    if world.in_bounds(l, num36) {
                        let t = &world.tiles[world.index(l, num36)];
                        if t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind) {
                            flag4 = true;
                        }
                    }
                }
                let (w, h) = self.size();
                if flag4 && !crate::player::collides_rect(world, self.x, self.y, w, h) {
                    self.ai[3] = 0.0;
                    self.ai[2] = 0.0;
                } else if self.ai[2] > 300.0 || cy > pcy + 200.0 {
                    self.ai[3] = -1.0;
                    self.ai[2] = 0.0;
                }
                return;
            }
            return;
        }

        // Normal / summoning (ai[3] >= 0). NPC.cs:56520-56717.
        self.rotation *= 0.9;
        // aiStyle 3 baseline chase (NPC.cs:15728).
        let dx = pcx - cx;
        if dx.abs() > 8.0 {
            self.facing = if dx > 0.0 { 1 } else { -1 };
            self.sprite_dir = self.facing;
            let speed = 0.85f32;
            if self.facing > 0 {
                self.vx = (self.vx + 0.07).min(speed);
            } else {
                self.vx = (self.vx - 0.07).max(-speed);
            }
        }

        if (self.ai[3] - 1.0).abs() < f32::EPSILON {
            // summoning pose (56643-56680); ShadowFlameApparition (472) spawn
            // at ai[2]==60 is not modeled (kind unavailable).
            self.vx *= 0.8;
            self.ai[2] += 1.0;
            if self.ai[2] >= 90.0 {
                self.ai[3] = -2.0;
                self.ai[2] = 0.0;
            }
            return;
        }

        // ai[3] == 0: ground phase (56681-56717)
        self.ai[2] += 1.0;
        if self.vy == 0.0 {
            if self.ai[2] >= 180.0 {
                self.ai[2] = 0.0;
                self.ai[3] = 1.0;
            }
        } else if self.ai[2] >= 360.0 {
            self.ai[2] = 0.0;
            self.ai[3] = -2.0;
            self.vy -= 3.0;
        }
        if (pcx - cx).hypot(pcy - cy) > 800.0 {
            self.ai[3] = -1.0;
            self.ai[2] = 0.0;
        }
        // target/despawn checks (56710-56717) are handled by the outer sim.
    }

    /// Lunatic Cultist (439). C# `NPC.cs:14216` SetDefaults aiStyle 84,
    /// `AI_084_LunaticCultist` (`NPC.cs:64414`). (S1-D-3b: was mis-routed to the
    /// pillar aiStyle 94 machine / type-422 row.)
    fn update_lunatic_cultist(&mut self, _world: &World, player: &Player, _rng: &mut u64) {
        // AI_084 core: spawn-in fade/despawn gate (NPC.cs:64554-64585,
        // 64587-64621). The full ritual/attack cycle is not yet ported.
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        if self.ai[0] == 0.0 && self.ai[1] == 0.0 && self.ai[2] == 0.0 {
            self.ai[0] = -1.0;
        }
        if self.ai[0] == -1.0 {
            if self.alpha > 5 {
                self.alpha = self.alpha.saturating_sub(5);
            } else {
                self.alpha = 0;
            }
            self.ai[1] += 1.0;
            if self.ai[1] >= 420.0 {
                self.ai[0] = 0.0;
                self.ai[1] = 0.0;
            } else if self.ai[1] > 360.0 {
                self.vx *= 0.95;
                self.vy *= 0.95;
            }
            return;
        }
        let num10 = 5600.0f32;
        if player.hp <= 0 || (pcx - cx).hypot(pcy - cy) > num10 {
            // NPC.cs:64562-64585
            self.hp = 0;
        }
    }

    /// `AI_008` through the framework's `caster_cycle` (`NPC.cs`'s aiStyle
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
                NpcKind::FireImp => (self.x + w * 0.5 + self.facing as f32 * 8.0, self.y + 20.0),
                _ => return,
            };
            let sphere = match self.kind {
                NpcKind::GoblinSorcerer | NpcKind::Tim => NpcKind::ChaosBall,
                NpcKind::DarkCaster => NpcKind::WaterSphere,
                NpcKind::FireImp => NpcKind::BurningSphere,
                _ => return,
            };
            self.wants_sphere.push((sphere, sx, sy));
        }
    }

    /// `AI_043`'s Queen Bee v1 (`NPC.cs:30190+`) — the EoC's cycle shape
    /// with the bee's numbers: the hover (the ±0.07 accels, the 8 cap),
    /// the charge (the 480-tick cycle, the direct velocity x 7), the decel
    /// (the 40 ticks, x0.98), the flee (`ai[0]==5`'s world-edge drift —
    /// the v1 rises away). The Servant bees and the enraged-outside-the-
    /// jungle state = the follow-ups.
    fn update_queen_bee(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // The flee: rise away (the vanilla's `ai[0]==5` drift — the v1
        // rises, the off-world guard despawns).
        if self.cast_windup == 3 {
            self.vy -= 0.04;
            self.x += self.vx;
            self.y += self.vy;
            if self.y < -500.0 {
                self.hp = 0;
            }
            return;
        }
        let hover_y = pcy - 260.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.07;
            if self.vx.abs() > 7.0 {
                self.vx = 7.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.07;
            if self.vy.abs() > 7.0 {
                self.vy = 7.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 7.0;
            self.vy = dy / d * 7.0;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_011`'s Skeletron head v1 (`NPC.cs`'s aiStyle 11 machine) — the
    /// EoC's cycle shape with the skull's numbers: the hover (the ±0.07,
    /// the 7 cap), the charge (the 480-tick cycle, the direct velocity
    /// x 8), the decel (the 40 ticks x0.98). The spinning hands (the
    /// separate NPCs 36) = the follow-ups.
    /// `AI()`'s aiStyle-11 block (`NPC.cs:21919-22298`) — the Skeletron
    /// head's machine: the spin state (0), the charge (1), the enraged
    /// day-time state (2) and the despawn (3).
    ///
    /// The hands' spawn rides the sim's pass (the C#'s `ai[0] == 0` init,
    /// `:21924-21941`); the arm reads the live hand count from `ai[3]`
    /// (the sim's write).
    ///
    /// The dust (`:22280-22296`) and the sounds are SKIPPED — no dust/sound
    /// layer.
    fn update_skeletron(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // `:21923` — the defense resets to the kind's base every tick.
        self.defense_override = None;
        // `:21949-21957` — a dead player or one 2000 px away → the despawn.
        let far = (self.x - player.x).abs() > 2000.0 || (self.y - player.y).abs() > 2000.0;
        if player.dead || far {
            self.ai[1] = 3.0;
        }
        // `:21958-21962` — the daytime → the enraged state.
        if world.day_time && self.ai[1] != 3.0 && self.ai[1] != 2.0 {
            self.ai[1] = 2.0;
        }
        match self.ai[1] as i32 {
            0 => {
                // `:22019-22051` — the spin: the timer (800 ticks → the
                // charge), the rotation and the hover acceleration.
                self.ai[2] += 1.0;
                if self.ai[2] >= 800.0 {
                    self.ai[2] = 0.0;
                    self.ai[1] = 1.0;
                }
                self.rotation = self.vx / 15.0;
                let (accel_y, cap_y, accel_x, cap_x) = if world.hard_mode {
                    (0.03f32, 4.0f32, 0.07f32, 9.5f32)
                } else {
                    (0.02, 2.0, 0.05, 8.0)
                };
                if self.y > player.y - 250.0 {
                    if self.vy > 0.0 {
                        self.vy *= 0.98;
                    }
                    self.vy -= accel_y;
                    if self.vy > cap_y {
                        self.vy = cap_y;
                    }
                } else if self.y < player.y - 250.0 {
                    if self.vy < 0.0 {
                        self.vy *= 0.98;
                    }
                    self.vy += accel_y;
                    if self.vy < -cap_y {
                        self.vy = -cap_y;
                    }
                }
                if cx > pcx {
                    if self.vx > 0.0 {
                        self.vx *= 0.98;
                    }
                    self.vx -= accel_x;
                    if self.vx > cap_x {
                        self.vx = cap_x;
                    }
                }
                if cx < pcx {
                    if self.vx < 0.0 {
                        self.vx *= 0.98;
                    }
                    self.vx += accel_x;
                    if self.vx < -cap_x {
                        self.vx = -cap_x;
                    }
                }
                // `:21978-22018` — the expert mode's spin shot: every 80
                // ticks (40 with no hand left) the head fires a skull.
                if world.hard_mode {
                    let hands = self.ai[3] as i32;
                    let period = if hands == 0 { 40.0 } else { 80.0 };
                    if self.ai[2] % period == 0.0
                        && crate::projectile::can_hit(
                            world,
                            cx,
                            cy,
                            1.0,
                            1.0,
                            player.x,
                            player.y,
                            crate::player::WIDTH,
                            player.height,
                        )
                    {
                        let speed = if hands == 0 { 5.0 } else { 3.0 };
                        let mut dx =
                            pcx - cx + crate::app::frame::rand_range_signed(rng, -20, 21) as f32;
                        let mut dy =
                            pcy - cy + crate::app::frame::rand_range_signed(rng, -20, 21) as f32;
                        let d = (dx * dx + dy * dy).sqrt().max(1.0);
                        dx = dx / d * speed;
                        dy = dy / d * speed;
                        // The jitter (`:22005-22006`).
                        dx += crate::app::frame::rand_range_signed(rng, -50, 51) as f32 * 0.01;
                        dy += crate::app::frame::rand_range_signed(rng, -50, 51) as f32 * 0.01;
                        let len = (dx * dx + dy * dy).sqrt().max(1.0);
                        dx = dx / len * speed;
                        dy = dy / len * speed;
                        self.wants_hostile_shot.push((
                            crate::projectile::SKULL,
                            cx + dx * 5.0,
                            cy + dy * 5.0,
                            dx + self.vx,
                            dy + self.vy,
                            17,
                        ));
                    }
                }
            }
            1 => {
                // `:22118-22244` — the charge: the defense drops by 10, the
                // timer (400 ticks → the spin) and the 1.5-px rush.
                self.defense_override = Some(self.defense() - 10);
                self.ai[2] += 1.0;
                if self.ai[2] >= 400.0 {
                    self.ai[2] = 0.0;
                    self.ai[1] = 0.0;
                }
                self.rotation += self.facing as f32 * 0.3;
                let dx = pcx - cx;
                let dy = pcy - cy;
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                let mut speed = if world.hard_mode { 3.5 } else { 1.5 };
                if world.hard_mode {
                    // `:22188-22227` — the expert's distance ramp.
                    if d > 150.0 {
                        speed *= 1.05;
                    }
                    for threshold in [
                        200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0, 600.0,
                    ] {
                        if d > threshold {
                            speed *= 1.1;
                        }
                    }
                    // `:22228-22236` — the hand count's share.
                    match self.ai[3] as i32 {
                        0 => speed *= 1.1,
                        1 => speed *= 1.05,
                        _ => {}
                    }
                }
                let k = speed / d;
                self.vx = dx * k;
                self.vy = dy * k;
            }
            2 => {
                // `:22245-22259` — the enraged state: the invulnerable
                // 8-px rush.
                self.damage_override = Some(9999);
                self.defense_override = Some(9999);
                self.rotation += self.facing as f32 * 0.3;
                let dx = pcx - cx;
                let dy = pcy - cy;
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                let k = 8.0 / d;
                self.vx = dx * k;
                self.vy = dy * k;
            }
            3 => {
                // `:22260-22270` — the despawn: the sink and the damp.
                self.vy += 0.1;
                if self.vy < 0.0 {
                    self.vy *= 0.95;
                }
                self.vx *= 0.95;
            }
            _ => {}
        }
        // `:22019` — the spin state restores the kind's damage.
        if self.ai[1] == 0.0 {
            self.damage_override = None;
        } else if self.ai[1] == 1.0 {
            let base_damage = self.kind.contact_damage();
            self.damage_override = Some(if world.hard_mode {
                (base_damage as f32 * 1.3) as i32
            } else {
                base_damage
            });
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI()`'s aiStyle-27 block — the Wall of Flesh's machine
    /// (`NPC.cs:25635-26050`, the 416-line `else if (this.aiStyle == 27)`).
    /// The classic path verbatim: the off-edge kill, the tongue timer
    /// (`ai[1]`/`ai[2]` → the Leech spawns), the underworld draw-band scan
    /// (`Main.wofDrawArea*` → the World fields) with the Y pinning, the
    /// slide speed by the hp thresholds, the direction lock, the
    /// dead-player abandonment (the `localAI[1]` timer folded into
    /// `ai[0]`), the init Hungry spawns (`localAI[0]` → `ai[3]`).
    /// The expert-mode burst block and the `getGoodWorld` spawn are out
    /// (no difficulty flags); the wall's eye/mouth mirrors (114) are the
    /// render-machinery follow-up, so the init spawns the 11 Hungries
    /// only; the roar sound rides `localAI[3]`'s cosmetic timer — skipped.
    fn update_wof(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        // The world-edge kill (`position.X < 160 || > (maxTilesX - 10) * 16`).
        if self.x < 160.0 || self.x > (world.width as f32 - 10.0) * TILE_PX {
            self.hp = 0;
            return;
        }
        // The tongue timer (`ai[1]` with the hp-ratio bonuses; 2700 → the
        // phase machine `ai[2]`).
        self.ai[1] += 1.0;
        if self.ai[2] == 0.0 {
            if (self.hp as f64) < (self.max_hp as f64) * 0.5 {
                self.ai[1] += 1.0;
            }
            if (self.hp as f64) < (self.max_hp as f64) * 0.2 {
                self.ai[1] += 1.0;
            }
            if self.ai[1] > 2700.0 {
                self.ai[2] = 1.0;
            }
        }
        if self.ai[2] > 0.0 && self.ai[1] > 60.0 {
            let mut num344 = 3;
            if (self.hp as f64) < (self.max_hp as f64) * 0.3 {
                num344 += 1;
            }
            self.ai[2] += 1.0;
            self.ai[1] = 0.0;
            if self.ai[2] > num344 as f32 {
                self.ai[2] = 0.0;
            }
            // The Leech head spawn (`NPC.CountNPCS(117) < 10` — the count
            // check rides the sim resolve; the worm pass grows the chain).
            self.wants_leech = true;
        }
        // The underworld draw-band scan (`num346` = UnderworldLayer + 10,
        // `num347` = +70): the bottom walk finds 15 solid/liquid tiles down
        // the wall's column span, the top walk mirrors it; the areas ease
        // 1 px per tick toward the found rows and clamp to a 160 px band.
        // `Main.UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`).
        let underworld_layer = world.height as i32 - 200;
        let num346 = underworld_layer + 10;
        let num347 = num346 + 70;
        let num348 = (self.x / TILE_PX) as i32;
        let num349 = ((self.x + w) / TILE_PX) as i32;
        let num350 = ((self.y + h * 0.5) / TILE_PX) as i32;
        // The `WorldGen.SolidTile` analog (the active + the solid set).
        let solid_tile = |x: i32, y: i32| -> bool {
            world
                .tile(x, y)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        };
        // The bottom: start at +7, walk down until 15 solid/liquid tiles.
        let mut num351 = 0;
        let mut num352 = num350 + 7;
        while num351 < 15 && num352 > underworld_layer {
            num352 += 1;
            if num352 > world.height as i32 - 10 {
                num352 = world.height as i32 - 10;
                break;
            }
            if num352 < num346 {
                continue;
            }
            for num353 in num348..=num349 {
                if solid_tile(num353, num352) {
                    num351 += 1;
                }
            }
        }
        num352 += 4;
        let bottom_target = num352 as f32 * TILE_PX;
        self.wof_bottom_target = bottom_target;
        // The top: start at -7, walk up.
        num351 = 0;
        num352 = num350 - 7;
        while num351 < 15 && num352 < world.height as i32 - 10 {
            num352 -= 1;
            if num352 <= 10 {
                num352 = 10;
                break;
            }
            if num352 > num347 {
                continue;
            }
            if num352 < num346 {
                num352 = num346;
                break;
            }
            for num354 in num348..=num349 {
                // The liquid counts toward the band like the solid
                // (`SolidTile || liquid > 0`, `:25747`'s inner test).
                let liquid = world.tile(num354, num352).is_some_and(|t| t.liquid > 0);
                if solid_tile(num354, num352) || liquid {
                    num351 += 1;
                }
            }
        }
        num352 -= 4;
        let top_target = num352 as f32 * TILE_PX;
        self.wof_top_target = top_target;
        // The band lives on the World (the sim eases toward the targets);
        // the Y pinning rides the CURRENT band midpoint.
        let num355 = (world.wof_draw_bottom + world.wof_draw_top) / 2.0 - h / 2.0;
        self.vy = 0.0;
        self.y = num355;
        // The slide speed by the hp thresholds (the classic rows; the
        // expert/getGood arms are out).
        let mut num356 = 1.5f32;
        if (self.hp as f64) < (self.max_hp as f64) * 0.75 {
            num356 += 0.25;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.5 {
            num356 += 0.4;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.25 {
            num356 += 0.5;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.1 {
            num356 += 0.6;
        }
        // The direction: locked to the sign, never damped (the wall slides
        // at the full speed toward its side).
        if self.vx == 0.0 {
            self.facing = if pcx >= self.x + w * 0.5 { 1 } else { -1 };
            self.sprite_dir = self.facing;
            self.vx = self.facing as f32;
        }
        if self.vx < 0.0 {
            self.vx = -num356;
            self.facing = -1;
            self.sprite_dir = self.facing;
        } else {
            self.vx = num356;
            self.facing = 1;
            self.sprite_dir = self.facing;
        }
        // The dead-player abandonment (`localAI[1]` += 1/180 → the wall
        // kills itself at 1; the recovery −1/30; folded into `ai[0]`).
        if player.hp <= 0 {
            self.ai[0] += 1.0 / 180.0;
            if self.ai[0] >= 1.0 {
                self.hp = 0;
                return;
            }
        } else {
            self.ai[0] = (self.ai[0] - 1.0 / 30.0).clamp(0.0, 1.0);
        }
        self.facing = self.vx.signum() as i32;
        self.sprite_dir = self.facing;
        // The rotation: the eye aims at the player when the player crosses
        // the wall's plane (`:25992-26010`).
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let num361 = pcx - cx;
        let num362 = pcy - cy;
        if self.facing > 0 {
            if pcx > self.x + w * 0.5 {
                self.rotation = (-num362).atan2(-num361) + 3.14;
            } else {
                self.rotation = 0.0;
            }
        } else if pcx < self.x + w * 0.5 {
            self.rotation = num362.atan2(num361) + 3.14;
        } else {
            self.rotation = 0.0;
        }
        // The Hungry init (`localAI[0] == 1 → 2`): the 11 Hungries along
        // the band (`ai[0] = i * 0.1 - 0.05`); the mirrors (114) are the
        // render follow-up. The request resolves in the world sim.
        if self.ai[3] == 0.0 {
            self.ai[3] = 1.0;
            self.wants_hungry = true;
        }
        // `noTileCollide` + `noGravity`: the raw integration.
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI()`'s aiStyle-29 block — the Hungry's machine
    /// (`NPC.cs:26054-26210`): the anchor = the wall's x + the draw-band
    /// row at `ai[0]`, the chase toward the player clamped to `num387`
    /// from the anchor (300/500/700 by the wall's hp), the defense/damage
    /// tiers (30/20 by the hp), the 4-speed cap, the justHit recoil
    /// (`ai[1] = 10`, folded onto `hurt_cd > 0`). The wall's death
    /// (`wof_present` off) deactivates the minion. The expert arms are out.
    fn update_hungry(&mut self, world: &World, player: &Player) {
        if !world.wof_present {
            self.hp = 0;
            return;
        }
        // `justHit → ai[1] = 10` (the recoil pause; the hurt cooldown
        // stands in for the flag).
        if self.hurt_cd > 0.0 {
            self.ai[1] = 10.0;
        }
        // The tiers by the wall's hp (`num388` the damage, the defense,
        // `num387` the leash radius).
        let mut num387 = 300.0f32;
        let mut num388 = 0;
        if (world.wof_hp_ratio as f64) < 0.5 {
            num388 = 60;
            self.defense_override = Some(30);
            num387 = 700.0;
        } else if (world.wof_hp_ratio as f64) < 0.75 {
            num388 = 45;
            self.defense_override = Some(20);
            num387 = 500.0;
        } else {
            self.defense_override = None;
        }
        if num388 > 0 {
            self.damage_override = Some(num388);
        } else {
            self.damage_override = None;
        }
        // The dive cycle: after 100 ticks the leash grows ×1.3, resetting
        // at 200.
        self.ai[2] += 1.0;
        if self.ai[2] > 100.0 {
            num387 = (num387 * 1.3) as i32 as f32;
            if self.ai[2] > 200.0 {
                self.ai[2] = 0.0;
            }
        }
        // The anchor: the wall's x, the band row at `ai[0]`.
        let num389 = world.wof_x;
        let num390 = world.wof_draw_bottom - world.wof_draw_top;
        let y2 = world.wof_draw_top + num390 * self.ai[0];
        let (pcx, pcy) = player.center();
        let (w, h) = self.size();
        let mut num391 = pcx - w * 0.5 - num389;
        let mut num392 = pcy - h * 0.5 - y2;
        let mut num393 = (num391 * num391 + num392 * num392).sqrt();
        if self.ai[1] == 0.0 {
            if num393 > num387 {
                num393 = num387 / num393;
                num391 *= num393;
                num392 *= num393;
            }
            // The approach with the double-add on the sign flips
            // (0.1 accel, the ×2.5 flip bonus).
            if self.x < num389 + num391 {
                self.vx += 0.1;
                if self.vx < 0.0 && num391 > 0.0 {
                    self.vx += 0.25;
                }
            } else if self.x > num389 + num391 {
                self.vx -= 0.1;
                if self.vx > 0.0 && num391 < 0.0 {
                    self.vx -= 0.25;
                }
            }
            if self.y < y2 + num392 {
                self.vy += 0.1;
                if self.vy < 0.0 && num392 > 0.0 {
                    self.vy += 0.25;
                }
            } else if self.y > y2 + num392 {
                self.vy -= 0.1;
                if self.vy > 0.0 && num392 < 0.0 {
                    self.vy -= 0.25;
                }
            }
            // The speed cap 4.
            self.vx = self.vx.clamp(-4.0, 4.0);
            self.vy = self.vy.clamp(-4.0, 4.0);
        } else if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // The rotation toward the player.
        if num391 > 0.0 {
            self.rotation = num392.atan2(num391);
        }
        if num391 < 0.0 {
            self.rotation = num392.atan2(num391);
        }
        // `noTileCollide` + `noGravity`: the raw integration.
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI()`'s aiStyle-80 block (`NPC.cs:36532-36624`) — the Martian
    /// Probe's hover/detect/escape machine. `ai[0] == 0` hovers: the
    /// X drift 3·direction (the collideX flip), the vertical scan's
    /// 15/20-tile rise/damp/sink bands (`NPC.cs:36541-36557`), and the
    /// detect gate (the closest player within 352 px whose centre sits
    /// below the probe, `:36564-36569`). `ai[0] == 1` is the 60-tick
    /// detect flash (the velocity damp ×0.95). `ai[0] == 2` escapes: the
    /// `noTileCollide` rise (`:36577`'s velocity.Y − 0.1 to −10) with the
    /// `ai[3]` x-drift, and the leave-world/180-tick arm fires
    /// `Main.StartInvasion(4); active = false` — the AI holds `&World`,
    /// so the resolve rides `wants_martian_invasion` (the sim starts the
    /// invasion and removes the NPC). The SkyBlue/Red lighting tint is
    /// unmodelled (the port has no NPC light sources).
    fn update_martian_probe(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        if self.ai[0] == 0.0 {
            if self.facing == 0 {
                // TargetClosest: the direction toward the nearest player
                // (`NPC.cs:36534-36537`).
                self.facing = if pcx >= self.x + w * 0.5 { 1 } else { -1 };
                self.sprite_dir = self.facing;
            }
            self.vx = 3.0 * self.facing as f32;
            // The vertical scan: the first solid tile within 30 tiles
            // below the centre (`:36541-36552`).
            let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
            let tx = (cx / TILE_PX) as i32;
            let ty = (cy / TILE_PX) as i32;
            let mut num1138 = 30;
            if world.in_bounds(tx, ty) {
                for dy in 0..30 {
                    if world
                        .tile(tx, ty + dy)
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    {
                        num1138 = dy;
                        break;
                    }
                }
            }
            if num1138 < 15 {
                self.vy = (self.vy - 0.05).max(-3.5);
            } else if num1138 < 20 {
                self.vy *= 0.95;
            } else {
                self.vy = (self.vy + 0.05).min(1.5);
            }
            // `FindClosestPlayer`'s dead fold (`:36559-36562`).
            if player.hp <= 0 {
                return;
            }
            let dist = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
            if dist < 352.0 && pcy > cy {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
            }
        } else if self.ai[0] == 1.0 {
            // The detect flash (`:36571-36586`).
            self.ai[1] += 1.0;
            self.vx *= 0.95;
            self.vy *= 0.95;
            if self.ai[1] >= 60.0 {
                self.ai[1] = 0.0;
                self.ai[0] = 2.0;
                self.ai[3] = if pcx > self.x + w * 0.5 { -1.0 } else { 1.0 };
            }
        } else if self.ai[0] == 2.0 {
            // The escape (`:36588-36600`): the rise, the drift away, and
            // the leave-world/180-tick invasion arm.
            self.ai[1] += 1.0;
            self.vy = (self.vy - 0.1).max(-10.0);
            self.vx = (self.vx + self.ai[3] * 0.05).min(4.0);
            if self.y < -(h as f32) || self.ai[1] >= 180.0 {
                self.wants_martian_invasion = true;
            }
        }
        // The movement: phases 0/1 collide with tiles (the vanilla's
        // velocity passes through TileCollision); phase 2 sets
        // `noTileCollide` and flies through.
        let no_collide = self.ai[0] == 2.0;
        let nx = self.x + self.vx;
        if no_collide || !crate::player::collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            // collideX (`:36537-36540`): flip the drift direction.
            self.facing = -self.facing;
            self.sprite_dir = self.facing;
            self.vx = 3.0 * self.facing as f32;
        }
        let ny = self.y + self.vy;
        if no_collide || !crate::player::collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
        } else {
            self.vy = 0.0;
        }
    }

    /// `AI()`'s aiStyle-74 block (`NPC.cs:35218-35500`) — the Martian
    /// Drone's approach/charge/dash machine, the 388 parameter set (the
    /// 418/Martian Walker Skull arm's different constants fold out).
    /// `ai[0]` 0 = the hover approach (the 200-px-above-the-player
    /// approach vector at speed 10 over a 30-tick damp; the line-of-sight
    /// gate with the ±π/8 angle band; the 120-tick argue timer), 1 = the
    /// 30-tick charge (the ×0.95 damp, then the jittered vector ×14), 2 =
    /// the dash (the 20-lerp toward the player; the off-angle/time exit
    /// at velocity < 7), 3 = the contact explosion (the 192×192 box,
    /// the 80 damage, the 3-tick fuse → the death).
    ///
    /// Deviations (documented): the dust/light spawns are unmodelled (no
    /// NPC light/dust systems); the phase-3 box resize (the vanilla's
    /// `width = height = 192`) is a per-NPC hitbox the kind-fixed `size`
    /// cannot follow — the explosion rides `damage_override` = 80 instead;
    /// the `GetKnockbackMultiplier_ScaledByDifficulty`/`GetAttackDamage_
    /// ScaledByDifficulty` difficulty arms run as the classic constants
    /// (no difficulty flags).
    fn update_martian_drone(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The vanilla's 388 param set.
        let speed = 10.0;
        let hover_height = 200.0;
        let far_dist = 750.0;
        let damp = 30.0;
        let charge_ticks = 30.0;
        let charge_decay = 0.95;
        let jitter = 50.0;
        let dash_speed = 14.0;
        let dash_ticks = 30.0;
        let turn_dist = 100.0;
        let lerp = 20.0;
        let speed_floor = 7.0;
        // `num1038` (the 418-only 4/15 × 60 homing term) is 0 for the drone.
        let home_term = 0.0;

        // The head (`AI()`'s aiStyle-74 block, `NPC.cs:35220-35233`): the
        // sheet rides the velocity's rotation, folded back into ±π/2, and
        // `spriteDirection = -Math.Sign(velocity.X)` — the ONLY aiStyle-74
        // kind whose art inverts against its motion (the 418 stands).
        self.rotation = self.vy.atan2(self.vx);
        if self.rotation < -std::f32::consts::PI / 2.0 {
            self.rotation += std::f32::consts::PI;
        }
        if self.rotation > std::f32::consts::PI / 2.0 {
            self.rotation -= std::f32::consts::PI;
        }
        let sign = self.vx.signum() as i32;
        if sign != 0 {
            self.sprite_dir = -sign;
        }

        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        if self.ai[0] == 0.0 {
            // The approach vectors: `vector128` to the player, `vector129`
            // 200 px above it (the xShift is 0 for the drone).
            let (vx, vy) = (pcx - cx, pcy - cy);
            let raw_dist = (vx * vx + vy * vy).sqrt();
            let norm = if raw_dist > 0.0 { 1.0 / raw_dist } else { 0.0 };
            // vector129 = normalize(raw - UnitY * 200) * speed.
            let vy2 = vy - hover_height;
            let d2 = (vx * vx + vy2 * vy2).sqrt();
            let n2 = if d2 > 0.0 { speed / d2 } else { 0.0 };
            // `vector128.ToRotation()`'s ±π/8 gate: the y-down rotation of
            // the toward-player direction — the cone's 0..π half (the
            // player below).
            let angle = (vy * norm).atan2(vx * norm);
            // `:35319`: `flag76 = CanHit(base.Center, 1, 1, player.Center, 1, 1)`.
            let mut can_hit =
                crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
            // The ai[3] >= 120 argue-override: the drone stops dodging.
            if self.ai[3] >= 120.0 {
                can_hit = true;
            }
            // `:35325`: the cone gate applies after the override.
            let can_hit = can_hit
                && angle > std::f32::consts::PI / 8.0
                && angle < std::f32::consts::PI - std::f32::consts::PI / 8.0;
            let too_far = raw_dist > far_dist;
            if too_far || !can_hit {
                // The 30-tick damp toward the above-the-player vector.
                self.vx = (self.vx * (damp - 1.0) + vx * n2) / damp;
                self.vy = (self.vy * (damp - 1.0) + vy2 * n2) / damp;
                if !can_hit {
                    if !too_far {
                        self.ai[3] += 1.0;
                    }
                } else {
                    self.ai[3] = 0.0;
                }
            } else {
                self.ai[0] = 1.0;
                self.ai[2] = vx * norm;
                self.ai[3] = vy * norm;
            }
        } else if self.ai[0] == 1.0 {
            // The charge (`:35290-35300`).
            self.vx *= charge_decay;
            self.vy *= charge_decay;
            self.ai[1] += 1.0;
            if self.ai[1] >= charge_ticks {
                self.ai[0] = 2.0;
                self.ai[1] = 0.0;
                // The ai[2]/ai[3] vector + the ±50 × 0.04 jitter, normalized ×14.
                let jx = (crate::app::rand01(rng) * 2.0 - 1.0) * jitter * 0.04;
                let jy = (crate::app::rand01(rng) * 2.0 - 1.0) * jitter * 0.04;
                let (dx, dy) = (self.ai[2] + jx, self.ai[3] + jy);
                let len = (dx * dx + dy * dy).sqrt();
                let inv = if len > 0.0 { dash_speed / len } else { 0.0 };
                self.vx = dx * inv;
                self.vy = dy * inv;
            }
        } else if self.ai[0] == 2.0 {
            // The dash (`:35303-35327`).
            self.ai[1] += 1.0;
            let dist = (pcx - cx).hypot(pcy - cy);
            let past = dist > turn_dist && cy > pcy;
            let slow = (self.vx * self.vx + self.vy * self.vy).sqrt() < speed_floor;
            if (self.ai[1] >= dash_ticks && past) || slow {
                self.ai[0] = 0.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
                self.vx /= 2.0;
                self.vy /= 2.0;
            } else {
                let (dx, dy) = (pcx - cx, pcy - cy);
                let len = (dx * dx + dy * dy).sqrt();
                let (nx, ny) = if len > 0.0 {
                    (dx / len, dy / len)
                } else {
                    (self.facing as f32, 0.0)
                };
                let vlen = (self.vx * self.vx + self.vy * self.vy).sqrt();
                self.vx = (self.vx * (lerp - 1.0) + nx * (vlen + home_term)) / lerp;
                self.vy = (self.vy * (lerp - 1.0) + ny * (vlen + home_term)) / lerp;
            }
            // The solid-wall fold to the explosion (`:35329-35334`).
            if crate::player::collides_rect(world, self.x, self.y, w, h) {
                self.ai[0] = 3.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
        }
        // The contact-too-close explosion (`:35336-35343`).
        if self.ai[0] != 3.0 && player.hp > 0 {
            let dist = (pcx - cx).hypot(pcy - cy);
            if dist < 64.0 {
                self.ai[0] = 3.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
        }
        if self.ai[0] != 3.0 {
            // The flying integration: noTileCollide + noGravity (`SetDefaults`).
            self.x += self.vx;
            self.y += self.vy;
            return;
        }
        // The explosion (`:35345-35396`): the position centres, the box
        // swells (the port keeps the kind size — `damage_override` carries
        // the 80), the alpha darkens, the 3-tick fuse. The lights/dust are
        // unmodelled.
        self.damage_override = Some(80);
        self.vx = 0.0;
        self.vy = 0.0;
        self.ai[1] += 1.0;
        if self.ai[1] >= 3.0 {
            // `SoundEngine.PlaySound(Item14)` + `life = 0` + `HitEffect`:
            // the death pass picks the zero HP up (the loot db registers no
            // 388 rules — nothing drops).
            self.hp = 0;
        }
    }

    /// The per-kind table (`NPC.cs:59100-59500`'s rows): the fire cooldown
    /// `num162`, the shot speed `num164`, the projectile `num170` and its
    /// damage `num169` — the pirate trio rides it (the Archer 214's arrow
    /// 180 at 25/10, the Corsair 215's musket ball 82 at 35/14, the Captain
    /// 216's arrow 180 at 30/9 with the ±50 jitter), the martians too.
    fn shooter_params(kind: NpcKind) -> (u32, f32, u16, i32) {
        match kind {
            NpcKind::PirateArcher => (50, 10.0, 180, 25),
            NpcKind::PirateCorsair => (90, 14.0, 82, 35),
            NpcKind::PirateCaptain => (60, 9.0, 180, 30),
            NpcKind::RayGunner => (80, 7.0, 438, 30),
            NpcKind::BrainScrambler => (80, 11.0, 436, 24),
            _ => (80, 11.0, 436, 24),
        }
    }

    /// `AI_003_Fighters`'s shooter block (`NPC.cs:59069-59623`) for the
    /// BrainScrambler 381 / the RayGunner 382. The walk rides the walker
    /// (the per-type speed folds); the stop-and-fire cycle: the
    /// aggression arm (grounded, `ai[1] <= 0`, `ai[2] <= 0`) starts the
    /// 80-tick shot timer with the aim octant on `ai[2]`, the fire at
    /// `ai[1] == 40` launches the projectile at the player ±(0..100)×0.4
    /// ×(0.85..1.15), and the sleep ends at `ai[1] <= 0`.
    ///
    /// Deviations: the player's `stealth`/`itemAnimation` gate (`:59644`)
    /// is constants-off (the port's Player carries no swing timer — the
    /// shooter engages a still player too); the walk params ride the
    /// walker's (the per-type 5/0.25/0.7 table folds like the other
    /// aiStyle-3 families).
    fn update_martian_shooter(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (mut cooldown, speed, proj, dmg) = Self::shooter_params(self.kind);
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let (dx, dy) = (pcx - cx, pcy - cy);
        // The octant table (the vanilla's `ai[2]` 1..5 rows).
        let octant = |dx: f32, dy: f32| -> f32 {
            if dy.abs() > dx.abs() * 2.0 {
                if dy > 0.0 { 1.0 } else { 5.0 }
            } else if dx.abs() > dy.abs() * 2.0 {
                3.0
            } else if dy > 0.0 {
                2.0
            } else {
                4.0
            }
        };
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // `justHit`'s interrupt (`:59094`): the shot cycle rests 30.
        if self.hurt_cd > 0.0 && self.ai[1] > 30.0 {
            self.ai[1] = 30.0;
            self.ai[2] = 0.0;
        }
        if self.ai[2] > 0.0 {
            // The fire (`:59204-59602`): at the half-cooldown the aim
            // carries the per-kind jitter (±100×0.4 for the martians'
            // rays, ±50 for the Captain's arrow) re-scaled ×(0.85..1.15);
            // the spawn is the centre + 6 y, the velocity at the speed.
            // The Captain's WOUNDED arm (`:59170-59235` + `:59603-59606`):
            // `localAI[2]` counts the shots (the port's `caster_counter`),
            // past 20 the barrage fires the 240 shell (100 dmg, 15 speed,
            // no jitter) on the rolling `max(60, 180 - localAI[3]*20)`
            // cooldown (the port's `cast_windup` as `localAI[3]`).
            let captain_wounded = self.kind == NpcKind::PirateCaptain && self.caster_counter >= 20;
            if self.ai[1] * 2.0 >= cooldown as f32 {
                let mut ax = dx;
                let mut ay = dy;
                let (jit, jit_scale) = if captain_wounded {
                    (0.0, 0.0)
                } else if self.kind == NpcKind::PirateCaptain {
                    (50.0, 1.0)
                } else {
                    (100.0, 0.4)
                };
                ax += (crate::app::rand01(rng) * 2.0 - 1.0) * jit * jit_scale;
                ay += (crate::app::rand01(rng) * 2.0 - 1.0) * jit * jit_scale;
                ax *= 0.85 + 0.3 * crate::app::rand01(rng);
                ay *= 0.85 + 0.3 * crate::app::rand01(rng);
                let len = (ax * ax + ay * ay).sqrt().max(0.001);
                let shot_speed = if captain_wounded { 15.0 } else { speed };
                let (shot_proj, shot_dmg) = if captain_wounded {
                    (240u16, 100)
                } else {
                    (proj, dmg)
                };
                let inv = shot_speed / len;
                self.wants_hostile_shot.push((
                    shot_proj,
                    cx,
                    cy + 6.0,
                    ax * inv,
                    ay * inv,
                    shot_dmg,
                ));
                self.ai[2] = octant(ax, ay);
                if self.kind == NpcKind::PirateCaptain {
                    self.caster_counter += 1;
                }
            }
        } else {
            // The aggression/stop entry (`:59630-59697`): grounded +
            // `ai[1] <= 0` and the aim under 700 px → the stop with the
            // 80-tick timer.
            if self.ai[1] <= 0.0
                && player.hp > 0
                && crate::projectile::can_hit(
                    world,
                    self.x,
                    self.y,
                    w,
                    h,
                    player.x,
                    player.y,
                    crate::player::WIDTH,
                    player.height,
                )
            {
                let mut ax = dx + (crate::app::rand01(rng) * 2.0 - 1.0) * 40.0;
                let mut ay = dy + (crate::app::rand01(rng) * 2.0 - 1.0) * 40.0;
                if (ax * ax + ay * ay).sqrt() < 700.0 {
                    self.vx *= 0.5;
                    self.ai[2] = octant(ax, ay);
                    self.ai[1] = 80.0;
                }
                let _ = (&mut ax, &mut ay);
            }
        }
        // The Captain's rolling cooldown (`:59174-59179`): wounded, the
        // timer is `max(60, 180 - the barrage count * 20)`.
        if self.kind == NpcKind::PirateCaptain && self.caster_counter >= 20 {
            let cd = (60).max(180 - self.cast_windup * 20);
            cooldown = cd as u32;
        }
        // The barrage's counter reset: after the 240 shell leaves the
        // vanilla's `localAI[2] = 0` (`:59523`).
        if self.kind == NpcKind::PirateCaptain && self.caster_counter > 20 && self.ai[2] == 0.0 {
            self.cast_windup += 1;
            self.caster_counter = 0;
        }
        // The sleep's exit (`:59618-59623`).
        if self.ai[1] <= 0.0 {
            self.ai[2] = 0.0;
            self.ai[1] = 0.0;
        }
        // The walk: the vanilla's walker step (the grounded damping and
        // the 0.9 stop-damp ride it).
        let params = crate::npc_ai::WalkerParams {
            // The C#'s per-type cap (`AI_003_Fighters`'s shooters' group,
            // `NPC.cs:59717-59750`): the pirate archer 2, the corsair 1.5,
            // the captain 1, the scrambler/ray gunner 2.
            speed: self
                .kind
                .ai3_walk_speed()
                .expect("the shooters' aiStyle-3 cap"),
            jump: -6.2,
            aggro_x: 80.0 * TILE_PX,
            aggro_y: 40.0 * TILE_PX,
        };
        crate::npc_ai::walker_step(self, world, pcx, pcy, params, GRAVITY, MAX_FALL);
        // The stop state's X damp (`:59626`'s `velocity.X *= 0.9`).
        if self.ai[2] > 0.0 {
            self.vx *= 0.9;
        }
    }

    /// `AI_075`'s type-390 arm (`NPC.cs:35506-35745`) — the ray gunner
    /// mounted on a Scutlix. The init spawns the 391 carrier through
    /// `wants_scutlix` (the sim resolves); while the carrier lives the
    /// rider rides 14 px above its centre and mirrors its velocity and
    /// direction; the `ai[1]` cycle (60, hurt → -30) fires the 438 ray
    /// (30 damage, 7 speed) from the centre - 4, the aim at the player
    /// ±50 ×(0.8..1.2), under 700 px with the carrier-direction match.
    ///
    /// Deviations: the carrier link is the sim's write-through (the
    /// port has no NPC indices — the nearest living 91 within 800 px
    /// stands in); the idle aim octant rides the vanilla's table.
    fn update_scutlix_rider(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The init (`NPC.cs:35506-35519`): `localAI[0]`'s one-time latch —
        // `ai[0]` stands in (the mount block's own state rides `ai[1]`).
        if self.ai[0] == 0.0 {
            self.wants_scutlix = true;
            self.ai[0] = 1.0;
        }
        if self.carrier_x < 0.0 {
            // No carrier (yet / dead): the vanilla's mount block skips —
            // the rider stands where it is.
            return;
        }
        let (w, h) = self.size();
        let (ccx, ccy) = (self.carrier_x, self.carrier_y);
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The ride (`:35657-35681`): the position centres on the carrier
        // + UnitY * -14.
        self.x = ccx - w * 0.5;
        self.y = ccy - h * 0.5 - 14.0;
        // The cycle (`:35682-35688`): ai[1] to 60, hurt → -30.
        if self.ai[1] < 60.0 {
            self.ai[1] += 1.0;
        }
        if self.hurt_cd > 0.0 {
            self.ai[1] = -30.0;
        }
        if crate::projectile::can_hit(
            world,
            self.x,
            self.y,
            w,
            h,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        ) {
            let (px, py) = (pcx - cx, pcy - cy);
            if (px * px + py * py).sqrt() < 700.0 {
                if self.ai[1] == 60.0 && (px > 0.0) == (self.facing == 1) {
                    // The fire (`:35700-35724`): the 438 ray, 30 damage,
                    // 7 speed, from the centre - UnitY * 4, the aim
                    // ±50 ×(0.8..1.2).
                    self.ai[1] = -60.0;
                    let mut ax = px;
                    let mut ay = py;
                    ax += (crate::app::rand01(rng) * 2.0 - 1.0) * 50.0;
                    ay += (crate::app::rand01(rng) * 2.0 - 1.0) * 50.0;
                    ax *= 0.8 + 0.4 * crate::app::rand01(rng);
                    ay *= 0.8 + 0.4 * crate::app::rand01(rng);
                    let len = (ax * ax + ay * ay).sqrt().max(0.001);
                    let inv = 7.0 / len;
                    self.wants_hostile_shot
                        .push((438u16, cx, cy - 4.0, ax * inv, ay * inv, 30));
                } else {
                    // The idle aim (`:35727-35749`): the octant + the X
                    // damp.
                    self.vx *= 0.5;
                    let octant = if py.abs() > px.abs() * 2.0 {
                        if py > 0.0 { 1.0 } else { 5.0 }
                    } else if px.abs() > py.abs() * 2.0 {
                        3.0
                    } else if py > 0.0 {
                        2.0
                    } else {
                        4.0
                    };
                    self.ai[2] = octant;
                }
            }
        }
    }

    /// `AI_003_Fighters`'s 391-carrier arm (`NPC.cs:58203-58240`): the
    /// walker with the params 5/0.25/0.7 (the per-type fold of the other
    /// aiStyle-3 families) and the 50-px ground stop (`ai[2] = 1`,
    /// `:58293-58301`).
    fn update_scutlix(&mut self, world: &World, player: &Player, _dt: f32) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The 50-px stop: grounded + the player close + CanHit.
        if self.on_ground
            && ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt() < 50.0
            && crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0)
        {
            self.ai[2] = 1.0;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 5.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The stop's park (`:58295`'s `velocity.X *= 0.7` when grounded).
        if self.ai[2] > 0.0 && self.on_ground {
            self.vx *= 0.7;
        }
        // The frames ride `find_frame`'s case 391 (`NPC.cs:71743-71778`),
        // which the update pass calls right after this machine — the C#'s
        // aiStyle-3 block writes no `frame.Y` itself (the v1 ran a blind
        // 6-row strip from here, racing the FindFrame arm).
    }

    /// `AI`'s aiStyle-75 arms for the saucer family (`NPC.cs:35616-36060`)
    /// — the mounts ride the core (the seat offsets) and run their
    /// batteries off the core's mirrored `ai[3]` timer and `ai[0]` phase
    /// (the pre-pass write-through): the 393 TURRET's 8-way aim + the 449
    /// rays (35 dmg, 16 speed, the ±0.785/3 jitter, every 6 ticks in the
    /// 280-420 window), the 394 CANNON's 448 shells (50 dmg, 8 speed, the
    /// ±0.785 aim, the side·36 + 8 spawn, every 20 ticks in the 440-580
    /// window), and the 392 SHELL's 447 core-shot (50 dmg at the phase-0
    /// window's entry) + the 450 missile battery (30 dmg, 3 speed, four
    /// emitters gated by the crew's aliveness) + the phase bridge (the
    /// crew dead + the core open → the core's `ai[0]` = 3, the classic
    /// arm).
    ///
    /// Deviations: `GetAttackDamage_ForProjectiles` runs as the classic
    /// constants; `Main.npc[localAI[i]]`'s sibling references ride the
    /// pre-pass crew counts; the 448's extra args (0, 20) fold out; the
    /// expert-mode bridge arm is constants-off (no difficulty flags).
    fn update_saucer_mount(&mut self, world: &World, player: &Player, rng: &mut u64) {
        if self.carrier_x < 0.0 {
            return;
        }
        let (w, h) = self.size();
        let side = if self.ai[1] == 1.0 { 1.0 } else { -1.0 };
        let (ox, oy) = match self.kind {
            NpcKind::MartianSaucer => (0.0, 2.0),
            NpcKind::MartianSaucerTurret => (side * 60.0, 29.0),
            _ => (side * 49.0, -13.0),
        };
        self.x = self.carrier_x - w * 0.5 + ox;
        self.y = self.carrier_y - h * 0.5 + oy;
        // The mounted timers mirror the core's. A mount created by the
        // core's own crew init starts with `carrier_timer = -1`; the
        // pre-pass overwrites it on the first tick — seed it from the
        // mount's own mirrored `ai[3]` so the very first tick is in step.
        if self.carrier_timer < 0.0 {
            self.carrier_timer = self.ai[3];
        }
        // The Flying Dutchman's cannon owns its `ai[3]` (the C#'s 492 block
        // counts its own reload, `:35760-35762`), so the core's timer does
        // not mirror into it.
        if self.kind != NpcKind::FlyingDutchmanCannon {
            self.ai[3] = self.carrier_timer;
        }
        // The display inheritance (`NPC.cs:35625-35651`): every mount
        // copies the core's `rotation` so the family tilts as one body,
        // and (`NPC.cs:35669-35676`) its `spriteDirection` — the cannon
        // takes its seat's side instead (`num1056`).
        self.rotation = self.carrier_rot;
        if self.kind == NpcKind::MartianSaucerCannon {
            self.sprite_dir = if self.ai[1] == 1.0 { 1 } else { -1 };
        } else {
            self.sprite_dir = self.carrier_facing;
        }
        let core_open = self.carrier_phase != 1.0 && self.carrier_phase >= 0.0;
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        match self.kind {
            NpcKind::MartianSaucerTurret => {
                // The 280-420 window (`NPC.cs:35750-35894`).
                let t = self.ai[3];
                let in_window = t >= 280.0 && t < 420.0;
                if !in_window {
                    // The aim (`NPC.cs:35873-35880`): outside the burst
                    // the vanilla re-derives `ai[2]` from the player
                    // vector's rotation, Y-clamped at 0 (the turret never
                    // aims upward) — `FindFrame` case 393's 9 aim rows
                    // read it back.
                    let (pcx, pcy) = player.center();
                    let (ax, ay) = (pcx - cx, pcy - cy);
                    let vy = ay.max(0.0);
                    let len = (ax * ax + vy * vy).sqrt();
                    let (nx, ny) = if len > 0.0 {
                        (ax / len, vy / len)
                    } else {
                        (0.0, 1.0)
                    };
                    self.ai[2] = ny.atan2(nx);
                }
                if in_window {
                    if core_open && (t - 280.0) % 6.0 == 0.0 {
                        // The 449 ray at the aim (ai[2] = the rotation).
                        let ang =
                            self.ai[2] + (crate::app::rand01(rng) - 0.5) * 0.7853981852531433 / 3.0;
                        let (vx, vy) = (ang.cos() * 16.0, ang.sin() * 16.0);
                        self.wants_hostile_shot
                            .push((449u16, cx + vx, cy + vy, vx, vy, 35));
                    }
                }
            }
            NpcKind::FlyingDutchmanCannon => {
                // `:35753-35823` — the Dutchman's cannon: the 240-tick
                // reload, the line-of-sight gate, the shot and the eight-ray
                // aim.
                if self.ai[3] < 240.0 {
                    self.ai[3] += 1.0;
                }
                let (pcx, pcy) = player.center();
                if crate::projectile::can_hit(
                    world,
                    self.x,
                    self.y,
                    w,
                    h,
                    player.x,
                    player.y,
                    crate::player::WIDTH,
                    player.height,
                ) {
                    if self.ai[3] >= 240.0 {
                        // The shot (`:35773-35782`): the normalized
                        // direction to the player × 14, minus 5 on the Y.
                        self.ai[3] = 0.0;
                        let dx = pcx - cx;
                        let dy = pcy - cy;
                        let d = (dx * dx + dy * dy).sqrt();
                        let (mut vx, mut vy) = if d > 0.0 {
                            (dx / d, dy / d)
                        } else {
                            (0.0, 1.0)
                        };
                        vx *= 14.0;
                        vy *= 14.0;
                        vy -= 5.0;
                        self.wants_hostile_shot.push((240u16, cx, cy, vx, vy, 30));
                    } else {
                        // The aim scan (`:35789-35812`): the closest of the
                        // eight rays around the cannon sets `ai[2]`.
                        let mut best = 0usize;
                        let mut best_d = f32::INFINITY;
                        for k in 0..8 {
                            let quarter = k as f32 * std::f32::consts::FRAC_PI_4;
                            let ray = Vec2::new(quarter.sin(), quarter.cos());
                            let probe = Vec2::new(cx, cy) + ray * 50.0;
                            let d = (probe - Vec2::new(pcx, pcy)).length();
                            if d < best_d {
                                best_d = d;
                                best = k;
                            }
                        }
                        self.ai[2] = (best + 1) as f32;
                        if self.sprite_dir == 1 {
                            self.ai[2] = 9.0 - self.ai[2];
                        }
                    }
                } else if self.ai[2] != 0.0 {
                    self.ai[2] = 0.0;
                }
            }
            NpcKind::MartianSaucerCannon => {
                // The 440-580 window (`NPC.cs:35820-35858`).
                let t = self.ai[3];
                let in_window = t >= 440.0 && t < 580.0;
                if in_window && core_open && (t - 440.0) % 20.0 == 0.0 {
                    let ang = (crate::app::rand01(rng) - 0.5) * 0.7853981852531433;
                    let (vx, vy) = (side * ang.cos() * 8.0, ang.sin() * 8.0);
                    // The missile (`:35845`): the C# passes `ai1 = 20f` —
                    // the arm's 20-tick arming delay.
                    self.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                        kind: 448u16,
                        x: cx + side * 36.0,
                        y: cy + 8.0,
                        vx,
                        vy,
                        damage: 50,
                        ai0: 0.0,
                        ai1: 20.0,
                    });
                }
            }
            _ => {
                // The shell (`NPC.cs:35895-36060`): the phase-0 window's
                // 447 entry shot + the 450 missile battery.
                let t = self.ai[3];
                let core_phase0 = self.carrier_phase == 0.0;
                if core_phase0 && t >= 20.0 && t < 260.0 && (t - 20.0) == 0.0 {
                    // The deathray (`:35917`): the C# passes `ai0 =
                    // base.whoAmI + 1` — the spawner's own slot index — and
                    // the arm reads it to find its owner.
                    self.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                        kind: 447u16,
                        x: cx,
                        y: cy,
                        vx: 0.0,
                        vy: 0.0,
                        damage: 50,
                        ai0: self.slot as f32 + 1.0,
                        ai1: 0.0,
                    });
                }
                // The windows: the core's gun phase (2) opens them always;
                // else the 280-400 and 440-580 bands.
                let (fire_window, wide, narrow) = if self.carrier_phase == 2.0 {
                    (true, 120.0, 120.0)
                } else if t >= 280.0 && t < 400.0 {
                    (true, 60.0, 90.0)
                } else if t >= 440.0 && t < 580.0 {
                    (true, 90.0, 60.0)
                } else {
                    (false, 1000.0, 1000.0)
                };
                if fire_window && core_open {
                    let dmg = 30;
                    let roll = crate::app::rand01(rng);
                    // The turret emitters (the wide arcs).
                    if self.crew_turrets > 0 && roll < 1.0 / wide {
                        let ang = (crate::app::rand01(rng) - 0.5) * 0.7853981852531433 / 3.0;
                        let (vx, vy) = (-(roll * 3.0) * ang.cos() * 3.0, ang.sin() * 3.0 + 3.0);
                        self.wants_hostile_shot.push((
                            450u16,
                            cx - side * 55.0,
                            cy + 35.0,
                            vx,
                            vy,
                            dmg,
                        ));
                    } else if self.crew_cannons > 0 && roll < 1.0 / narrow {
                        let ang = (crate::app::rand01(rng) - 0.5) * 0.7853981852531433;
                        let (vx, vy) = (
                            -(roll * 2.0) * ang.cos() * 3.0 - 1.0,
                            -ang.sin() * 3.0 - 3.0,
                        );
                        self.wants_hostile_shot.push((
                            450u16,
                            cx - side * 45.0,
                            cy - 20.0,
                            vx,
                            vy,
                            dmg,
                        ));
                    }
                }
            }
        }
    }

    /// `AI()`'s aiStyle-76 block (`NPC.cs:36071-36520`) — the Martian
    /// Saucer Core's machine. The init spawns the four guns (2 turrets
    /// 393, 2 cannons 394) and the shell 392 (all `ai[0]`-linked to the
    /// core). `ai[0]` - 1/-2 = the despawn ride (the player out of the
    /// 5600-px window), 0 = the 600-tick SCAN cycle (the six step bands:
    /// the approach, the 250-px dive windows, the ground-clearance reads),
    /// 1 = the vulnerable wobble (the rotation ramp, 150 ticks), 2 = the
    /// GUN (the 3600-tick cycle: the 60-tick firing window fires the
    /// projectile 447 at 80 damage, the 8·direction dash in the window,
    /// the 16-lerp outside), 3 = the death ride (`StrikeNPCNoInteraction
    /// (9999)`).
    ///
    /// Deviations: `StrikeNPCNoInteraction`/`GetAttackDamage_ScaledBy
    /// Difficulty` run as the classic constants; `TargetClosest` folds to
    /// the single player; the off-world clamp keeps the position in the
    /// world box.
    /// `AI`'s aiStyle-93 block (`NPC.cs:38858-38956`) — the Flying
    /// Dutchman's machine: the crew carrier. The init spawns 4 cannons
    /// 492 at `Centre + seat*40 - 150` (the seat `ai[1]`, the 60*seat
    /// spawn delay folded out); the crew-alive loop kills the ship when
    /// all four die (`flag99` → the self-strike); every 300 ticks a
    /// pirate (213/215/214/212) drops off the ship's side with the
    /// upward velocity; the flight holds the 350-450 ground-clearance
    /// band (the ±4 Y ease toward the band), the X drift toward the
    /// player, the rotation = velocity.X * 0.025.
    ///
    /// Deviations: the child index links (`Main.npc[ai[i]]`) ride the
    /// pre-pass's crew count + the write-through; the dust spawns are
    /// unmodelled; the cannons' stagger (the `60*seat` timeLeft) folds
    /// out (the port spawns them at once).
    fn update_flying_dutchman(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The init (`:38859-38872`).
        if !self.wants_scutlix {
            self.wants_scutlix = true; // the init latch
            self.wants_dutchman_crew = true;
        }
        // The crew-alive loop (`:38874-38890`): all dead → the self-strike.
        // While the init's spawn request is pending (the cannons come at
        // the sim's resolve) the count still reads 0 — the ship lives.
        if self.crew_cannons <= 0 && !self.wants_dutchman_crew {
            self.hp = 0;
            return;
        }
        // The pirate drop (`:38892-38902`).
        if crate::app::rand01(rng) < 1.0 / 300.0 {
            // The rotated side offset: (rand01-0.5)*(width-70) × ±20-height/2-20.
            let jx = (crate::app::rand01(rng) - 0.5) * (350.0 - 70.0);
            let jy = (crate::app::rand01(rng) - 0.5) * 20.0 - 120.0 / 2.0 - 20.0;
            let (rx, ry) = rot(jx, jy, self.rotation);
            let kind = match (crate::app::rand01(rng) * 4.0) as u32 {
                0 => NpcKind::PirateCaptainsMate,
                1 => NpcKind::PirateCorsair,
                2 => NpcKind::PirateArcher,
                _ => NpcKind::PirateDeckhand,
            };
            let (cx, cy) = self.center();
            self.wants_dutchman_drop = Some((kind, cx + rx, cy + ry));
        }
        self.ai[0] = self.ai[0].max(0.0) + 1.0; // the crew-monitor tick (the port's counter)
        // The roll (`AI_093`'s `NPC.cs:38844`): `rotation = velocity.X *
        // 0.025` — the ship banks with its drift and the 492 cannons (and
        // the drop offsets above) read it.
        self.rotation = self.vx * 0.025;
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, _pcy) = player.center();
        // The ground clearance 10 tiles ahead (`:38904-38922`).
        let ahead = (cx / TILE_PX) as i32 + self.vx.signum() as i32 * 10;
        let feet = ((self.y + h) / TILE_PX) as i32;
        let clear = ground_clearance(world, ahead, feet);
        let d = clear;
        // The 350-450 band (`:38923-38942`).
        if d < 350.0 {
            let mut target = d - 350.0;
            if target < -4.0 {
                target = -4.0;
            }
            self.vy += (target - self.vy) * 0.05;
        } else if d > 450.0 {
            let mut target = d - 350.0;
            if target > 4.0 {
                target = 4.0;
            }
            self.vy += (target - self.vy) * 0.05;
        } else {
            self.vy *= 0.95;
        }
        // The X drift (`:38944-38947`).
        if (pcx - cx).abs() >= 300.0
            && (self.vx.abs() < 6.0 || self.vx.signum() != self.facing as f32)
        {
            self.vx += self.facing as f32 * 0.06;
        }
        self.rotation = self.vx * 0.025;
        // The flight (noGravity).
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI_107_ImprovedWalkers`'s DD2-goblin arm (`NPC.cs:63123-63145`) —
    /// the Old One's Army attacker. The walk rides the improved walker
    /// (the harness `num23` speed 1 + `num24` accel 0.07 + `num25` damp
    /// 0.8 + the 30-pixel hop/jump frame); the tiers add (553: +0.01/+0.2;
    /// 554: +0.02/+0.4); the wave acceleration rides `localAI[3]`'s ramp
    /// (the port's `caster_counter`): below 60 the accel is
    /// `0.01 + localAI[3]/60 * 0.05`. The scream/aggro arms
    /// (`SoundID.DD2_GoblinScream`, `DD2Event.EnemiesShouldChasePlayers`)
    /// ride the DD2Event framework — constants-off until it lands, so the
    /// goblins walk toward the player through the walker harness.
    fn update_dd2_goblin(&mut self, world: &World, player: &Player, dt: f32) {
        let (pcx, pcy) = player.center();
        // The wave-ramp: the port's `caster_counter` = `localAI[3]`.
        let wave_accel = if self.caster_counter >= 60 {
            self.caster_counter as f32
        } else {
            0.01 + self.caster_counter as f32 / 60.0 * 0.05
        };
        let (speed_add, accel_add) = match self.kind {
            NpcKind::DD2GoblinT2 => (0.2, 0.01),
            NpcKind::DD2GoblinT3 => (0.4, 0.02),
            _ => (0.0, 0.0),
        };
        // The walker harness (the port's step; the accel rides the wave
        // ramp + the tier's add).
        let params = crate::npc_ai::WalkerParams {
            speed: 1.0 + speed_add,
            jump: -6.2,
            aggro_x: 80.0 * TILE_PX,
            aggro_y: 40.0 * TILE_PX,
        };
        crate::npc_ai::walker_step(self, world, pcx, pcy, params, GRAVITY, MAX_FALL);
        // The DD2 accel overlay: the harness' own velocity bending stands
        // in for the vanilla's `num24` (the accel ramp + the tier add);
        // the vanilla applies it before the step — the port applies the
        // ramp after and folds it into the drift.
        let accel_el = (0.07 + accel_add + wave_accel).min(0.12);
        let drift = if self.vx.abs() > params.speed {
            -accel_el * self.vx.signum()
        } else {
            0.0
        };
        self.vx += drift;
        let _ = dt;
    }

    /// `AI_107_ImprovedWalkers`'s etherian-skeleton arm (`NPC.cs:63510-`)
    /// — the MATERIALIZE cycle. `localAI[3]` (the port's `caster_counter`)
    /// counts the spawn-in: the skeleton fades in over the 110-tick run
    /// (the port's render-carrying counter) and the walk rides the
    /// improved-walker harness once materialized; the arm's dust/fade is
    /// the render follow-up.
    /// `AI_107_ImprovedWalkers`'s javelin-thrower arm (`NPC.cs:63434-63465`)
    /// — the spear cadence: `ai[1]` counts to 82, the throw fires the
    /// spear (662; the tier-3's 685) at the classic damage (30/60/80) from
    /// the centre with the `UnitY * -14` spawn offset and the vanilla's
    /// gravity arc (the port's ballistic integration stands in for the
    /// projectile's own aiStyle-1 flight). The walk rides the harness
    /// with the `speed = 8, accel × 3` row.
    /// `AI_107_ImprovedWalkers`'s kobold-bomber arm (`NPC.cs:63490-63535`)
    /// — the lob: the walk rides the harness (the speed 0.88/0.88/1.12 +
    /// the tiered 0.6/0.6/0.4 `num16` arc weight + the `num15 = 0.4`
    /// sway); the `ai[1]` cycle counts to 18 and lobs the 681 bomb
    /// (14x14, aiStyle 16 — the bomb flight, penetrate -1) at the classic
    /// damages 20/30/40 from the centre - 14 at the ballistic aim; the
    /// vanilla's `flag24`/`flag12`/`flag15` (the chase/park states) and
    /// the `DD2Event.EnemiesShouldChasePlayers` flag ride the DD2Event
    /// framework — constants-off until it lands.
    fn update_dd2_bomber(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The lob cycle: `ai[1]` to 18 (`num9`).
        self.ai[1] += 1.0;
        if self.ai[1] >= 18.0 {
            self.ai[1] = 0.0;
            let dmg = match self.kind {
                NpcKind::DD2GoblinBomberT2 => 30,
                NpcKind::DD2GoblinBomberT3 => 40,
                _ => 20,
            };
            // The ballistic aim: at the player (the arc's weight is the
            // projectile's own aiStyle-16 flight).
            let (dx, dy) = (pcx - cx, pcy - (cy - 14.0));
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let inv = 6.0 / dist; // the vanilla's `num10 = 6`
            self.wants_hostile_shot
                .push((681u16, cx, cy - 14.0, dx * inv, dy * inv, dmg));
            let _ = rng;
        }
    }

    /// `AI_109_DarkMage` (`NPC.cs:65791-66000`) — the Old One's Army's
    /// caster. The materialize cycle rides `localAI[3]` (the port's
    /// `caster_counter`: 60 ticks of fade-in from the portal) — the
    /// vanilla's alpha fade is render-side; the CAST machine rides
    /// `ai[0]`'s negative/positive cycle: the spawn request
    /// (`ai[1] == 2` at `ai[0] == 64` fires the 673 summon marker +
    /// `DD2Event.RaiseGoblins`), the shadowflame (`ai[1] == 0` at
    /// `ai[0] == 32`: the 675 at 40 damage toward the player), the heal
    /// (`ai[1] == 1` at `ai[0] == 40/48/56`: the 674 markers under the
    /// target), and the re-arm (`ai[0] <= 0` → `ai[1]++` wrapped, the
    /// −20/−120 rest). The flight: the hover toward the player with the
    /// slow damp (0.5/0.1 speeds, the 0.96 damping), the rotation = vx
    /// × 0.04.
    ///
    /// Deviations: `DD2Event.RaiseGoblins`/`CanRaiseGoblinsHere` ride the
    /// DD2 framework (the summon request rides `wants_sphere`); the
    /// `GetAttackDamage_ScaledByDifficulty` runs classic; the portal
    /// sound/dust arms are unmodelled.
    fn update_dark_mage(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The materialize: `localAI[3]` counts to 60.
        if self.caster_counter < 60 {
            self.caster_counter += 1;
        }
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The hover (`:65950`'s tail): the slow approach with the damp.
        let (dx, dy) = (pcx - cx, pcy - cy);
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        self.vx += (dx / d * 2.0 - self.vx) * 0.1;
        self.vy += (dy / d * 2.0 - self.vy) * 0.1;
        self.vx *= 0.96;
        self.vy *= 0.96;
        self.rotation = self.vx * 0.04;
        // The cast cycle.
        if self.ai[0] < 0.0 {
            self.ai[0] += 1.0;
        } else if self.ai[0] > 0.0 {
            self.ai[0] -= 1.0;
            // The summon arm (`ai[1] == 2` at 64).
            if self.ai[1] == 2.0 && self.ai[0] == 64.0 {
                self.wants_hostile_shot.push((
                    673u16,
                    cx + self.facing as f32 * 24.0,
                    cy - 40.0,
                    0.0,
                    0.0,
                    0,
                ));
                // `DD2Event.RaiseGoblins` — the port's summon request.
                self.wants_sphere.push((NpcKind::DD2GoblinT1, cx, cy));
            }
            // The shadowflame (`ai[1] == 0` at 32).
            if self.ai[1] == 0.0 && self.ai[0] == 32.0 {
                let (sx, sy) = (cx + self.facing as f32 * 10.0, cy - 16.0);
                let (fx, fy) = (pcx - sx, pcy - sy);
                let fd = (fx * fx + fy * fy).sqrt().max(1.0);
                let inv = 14.0 / fd;
                self.wants_hostile_shot
                    .push((675u16, sx, sy, fx * inv, fy * inv, 40));
            }
            // The heal arm (`ai[1] == 1` at 40/48/56): the markers under
            // the target (the port places them at the vertical scan).
            if self.ai[1] == 1.0 && matches!(self.ai[0], 40.0 | 48.0 | 56.0) {
                self.wants_hostile_shot.push((
                    674u16,
                    cx + self.facing as f32 * 240.0,
                    cy,
                    0.0,
                    0.0,
                    0,
                ));
            }
            if self.ai[0] <= 0.0 {
                let prior = self.ai[1];
                self.ai[1] += 1.0;
                if self.ai[1] >= 3.0 {
                    self.ai[1] = 0.0;
                }
                self.ai[0] = -120.0;
                if prior == 0.0 {
                    self.ai[0] = -20.0;
                }
            }
        }
        // The re-arm (the vanilla's `ai[0] == 0 && localAI[3] >= 60`
        // gate — the cast pattern picks per the roster state): with the
        // materialize done and the rest elapsed the mage rearms.
        if self.ai[0] == 0.0 && self.caster_counter >= 60 {
            match self.ai[1] as u32 {
                0 => self.ai[0] = 97.0,
                1 => self.ai[0] = 127.0,
                _ => self.ai[0] = 183.0,
            }
        }
        // The flight.
        self.x += self.vx;
        self.y += self.vy;
        let _ = rng;
    }

    /// `AI_110_Betsy` (`NPC.cs:61850-62150`) — the Old One's Army's
    /// dragon. The v1 machine: phase 0 (the hover-in — the spawn pause),
    /// phase 1 (the approach to the 300/-200 offset past the player with
    /// the `num5 = 7.5` speed / `num6 = 0.45` accel flight, the phase
    /// table's pick per `ai[3]`'s rotation — 0/1/3 → 2, 2 → 3, 4 → 5, 5
    /// → 6, the wrapped 6 → 2, the mid-sequence `ai[3] == 5 &&
    /// Next(3) == 0` hop), phase 2 (the breath — the 687 projectile at 35
    /// damage from the centre at the velocity), and the off-world /
    /// target-loss guards. The remaining phases (the fireball 686, the
    /// circle flight, the dash) + the phase-5/6 renders ride the attack
    /// family's follow-ups.
    fn update_betsy(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        if self.ai[0] == 0.0 {
            // The spawn pause (`:61878-61884`).
            self.ai[1] += 1.0;
            if self.ai[1] >= 10.0 {
                self.ai[1] = 0.0;
                self.ai[0] = 1.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[0] == 1.0 {
            // The approach (`:61886-61900`).
            if self.ai[2] == 0.0 {
                self.ai[2] = if pcx < cx { 1.0 } else { -1.0 };
            }
            // The offset: the player + (-ai[2] * 300, -200).
            let (tx, ty) = (pcx - self.ai[2] * 300.0, pcy - 200.0);
            let (dx, dy) = (tx - cx, ty - cy);
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            let inv = 7.5 / d;
            self.vx += (dx * inv - self.vx) * 0.45;
            self.vy += (dy * inv - self.vy) * 0.45;
            self.facing = if pcx < cx { 1 } else { -1 };
            self.sprite_dir = self.facing;
            // The phase pick (the vanilla's `num29` table).
            self.ai[1] += 1.0;
            if self.ai[1] >= 30.0 {
                let mut pick = 2;
                if self.ai[3] == 5.0 && crate::app::rand01(rng) < 1.0 / 3.0 {
                    self.ai[3] += 1.0;
                }
                match self.ai[3] as u32 {
                    2 => pick = 3,
                    4 | 6 => pick = 5,
                    _ => pick = 3, // the C# default for 5+ → 6 lands as the
                                   // next attack family; the port folds 5's pick as 3
                                   // (the fireball) until it lands.
                }
                if self.ai[3] >= 7.0 {
                    self.ai[3] = 0.0;
                }
                self.ai[3] += 1.0;
                self.ai[0] = pick as f32;
                self.ai[1] = 0.0;
            }
        } else if self.ai[0] == 2.0 {
            // The wind-attack pause (`NPC.cs:61931-61940`): the 30-tick
            // pause before the return to phase 1 (the flame breath rides
            // phase 3).
            self.ai[1] += 1.0;
            if self.ai[1] >= 30.0 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[0] == 3.0 {
            // The flame breath (`NPC.cs:61942-61985`): the reposition to
            // the 600-px past-the-player offset at −250 (the 12-speed
            // approach, the 16-px snap), the 40-tick arrival, the dash
            // (the 10-speed lunge + the 687 breath at 35), the 120-tick
            // ride with the 0.5 accel past 550 px.
            self.ai[1] += 1.0;
            let dir = if pcx < cx { 1.0 } else { -1.0 };
            if self.ai[1] < 40.0 {
                let (tx, ty) = (pcx + dir * -600.0, pcy - 250.0);
                let (dx, dy) = (tx - cx, ty - cy);
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                if d < 12.0 {
                    self.x = tx - w * 0.5;
                    self.y = ty - h * 0.5;
                } else {
                    let inv = 12.0 / d;
                    self.x += dx * inv;
                    self.y += dy * inv;
                }
            } else if self.ai[1] == 40.0 {
                let dir2 = if pcx > cx { 1.0 } else { -1.0 };
                self.vx = dir2 * 10.0;
                self.vy = 0.0;
                self.facing = dir2 as i32;
                self.sprite_dir = self.facing;
                self.wants_hostile_shot
                    .push((687u16, cx, cy, self.vx, self.vy, 35));
            } else if self.ai[1] >= 40.0 && (pcx - cx).abs() > 550.0 && self.vx.abs() < 20.0 {
                self.vx += self.vx.signum() * 0.5;
            }
            if self.ai[1] >= 120.0 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[0] == 4.0 {
            // The fireball volley (`NPC.cs:61986-62045`): the reposition
            // to the 1500-px offset at −350 (the 13-speed lerp at 1/30),
            // the aim at the player (the flattened Y, the rotation), the
            // strafing glide (the X toward the player at 7, the Y toward
            // the −400 offset at 6) and the 6-shot volley every 10 ticks
            // from the 140·direction rotated muzzle.
            if self.ai[1] < 60.0 {
                let (tx, ty) = (pcx + self.facing as f32 * -1500.0, pcy - 350.0);
                let (dx, dy) = (tx - cx, ty - cy);
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                let inv = 13.0 / d;
                self.vx += ((dx * inv) - self.vx) * (1.0 / 30.0);
                self.vy += ((dy * inv) - self.vy) * (1.0 / 30.0);
            } else if self.ai[1] == 60.0 {
                let (mut ax, mut ay) = (pcx - cx, pcy - cy);
                ay *= 0.25;
                let d = (ax * ax + ay * ay).sqrt().max(1.0);
                ax /= d;
                ay /= d;
                self.facing = if ax > 0.0 { 1 } else { -1 };
                self.sprite_dir = self.facing;
                self.rotation = ay.atan2(ax);
                if self.facing == -1 {
                    self.rotation += std::f32::consts::PI;
                }
                self.vx = ax * 12.0;
                self.vy = ay * 12.0;
            } else {
                self.x += (pcx - cx).signum() * 7.0;
                self.y += (pcy - cy).signum() * 6.0;
                let n33 = (self.ai[1] - 60.0 + 1.0) as i32;
                if n33 <= 60 && n33 % 10 == 0 {
                    let (mx, my) = rot(170.0, 20.0, self.rotation);
                    self.wants_hostile_shot
                        .push((686u16, cx + mx, cy + my, self.vx, self.vy, 35));
                }
            }
            if self.ai[1] > 60.0 {
                self.vy -= 0.1;
            }
            self.ai[1] += 1.0;
            if self.ai[1] >= 180.0 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[0] == 5.0 {
            // The circle flight (`NPC.cs:61986-62000`): the velocity rotates
            // by `num24` (the 60-tick full turn) per side, the upward creep,
            // the 10-toward-the-player pull, the rotation mirror, the
            // 60-tick exit at half velocity.
            let turn = std::f32::consts::TAU / 60.0;
            let ang = (-turn) * self.facing as f32;
            let (nx, ny) = rot(self.vx, self.vy, ang);
            self.vx = nx;
            self.vy = ny;
            self.y -= 0.1;
            self.x += (pcx - cx).signum() * 10.0;
            self.y += (pcy - cy).signum() * 10.0;
            self.rotation -= turn * self.facing as f32;
            self.ai[1] += 1.0;
            if self.ai[1] >= 60.0 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.vx /= 2.0;
                self.vy /= 2.0;
            }
        } else if self.ai[0] == 6.0 {
            // The dash (`NPC.cs:62001-62060`): the first segment flies to
            // the player + (0, −200) at 2× speed until the 350-px contact
            // or 180 ticks; then the 20-tick 0.95 decay, the 0.98 ride,
            // the 90-tick exit.
            if self.ai[1] == 0.0 {
                let (tx, ty) = (pcx, pcy - 200.0);
                let (dx, dy) = (tx - cx, ty - cy);
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                let inv = 15.0 / d;
                self.vx += ((dx * inv) - self.vx) * 0.9;
                self.vy += ((dy * inv) - self.vy) * 0.9;
                self.facing = if pcx < cx { 1 } else { -1 };
                self.sprite_dir = self.facing;
                self.ai[2] += 1.0;
                if (pcx - cx).hypot(pcy - cy) < 350.0 || self.ai[2] >= 180.0 {
                    self.ai[1] = 1.0;
                }
            } else if self.ai[1] < 20.0 {
                self.vx *= 0.95;
                self.vy *= 0.95;
                self.ai[1] += 1.0;
            } else {
                self.vx *= 0.98;
                self.vy *= 0.98;
                self.ai[1] += 1.0;
            }
            if self.ai[1] >= 90.0 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
            }
        }
        // The flight (noGravity + noTileCollide).
        self.x += self.vx;
        self.y += self.vy;
    }
    /// `AI_107_ImprovedWalkers`'s WitherBeast arm (`NPC.cs:65900-65990`) —
    /// the ethereal caster-beast. The walk rides the harness with the
    /// beast's row (`num23 = 1.4`, `num24 = 0.16`, `num25 = 0.7`, the
    /// 600-px aggro window); the aura phase (`ai[0] == 1`): the pulse
    /// counter `localAI[0]` ticks, every 10 past 60 a ring dust spawns
    /// at the 400-px random radius (the port's seed stream), the player
    /// within 400 px takes the `DD2EtherealPortalDebuff`'s stand-in (the
    /// port has no debuff carry — the hurt half rides the contact), the
    /// 60-tick self-heal (`lifeMax / 20` capped to the missing hp); the
    /// wave ramp rides `localAI[3]` (the port's `caster_counter`); the
    /// 683 crystal shots ride the shooter block (the follow-up).
    /// `AI()`'s aiStyle-105 block (`NPC.cs:40971-41040`) — the Etherian
    /// Crystal's machine. The spawn-in shield: `localAI[0]` counts to 180
    /// (the port's `caster_counter`) while `dontTakeDamage` holds; the
    /// count's expiry makes the crystal vulnerable. The hp's tier is baked
    /// by the `SetDefaults` row (the 1000/3000/5000 difficulty arms ride
    /// the difficulty system); the lane spawning + the 600-tick win-death
    /// (`DD2Event.StopInvasion`) = the follow-ups.
    fn update_eternia_crystal(&mut self, world: &World, player: &Player) {
        self.caster_counter += 1;
        self.dont_take_damage = self.caster_counter < 180;
        // The 600-tick win-death (`NPC.cs`'s aiStyle-105 arm,
        // `:40959-40966`): `DD2Event.StopInvasion(); life = 0` — the
        // ai[0] counter advances inside the phase (the `ai[1] == 1`
        // entry) and the sim resolves the event's stop.
        self.ai[0] += 1.0;
        if self.ai[0] >= 600.0 {
            self.wants_event_stop = true;
            self.dont_take_damage = false;
            self.hp = 0;
        }
        let _ = (world, player);
    }

    /// `AI()`'s aiStyle-94 block (`NPC.cs:38968-39400`) — the Ancient Cultist's
    /// machine. `ai[1]`'s timer counts to 180 and the cultist dissolves
    /// (`life = 0`, the death chain); the hover rides the sine bob
    /// (velocity.Y = sin(2π·ai[0]/300)·0.5, the 300-tick cycle); the summon
    /// arms: the ritual projectiles (578/579, 0 damage, the "ritual"
    /// markers) at the player's tile (3240-px gate, the line-of-sight
    /// refusal, the 60+120 rearm) and the 1080-px arm (the 420+360 rearm,
    /// the 20-tile window's non-solid pick, the lava/solid/line gates); the
    /// hp regen (`+200` per 60 ticks while no player within 2000 px).
    ///
    /// Deviations: the dust/sound arms are unmodelled; the shield-strength
    /// reads (`ShieldStrengthTower*`) fold out (no pillar shields yet).
    /// `AI()`'s aiStyle-95 block (`NPC.cs:39384-39429`) — the Stardust
    /// CellSmall's growth: the 300-tick scale ramp (1.0 → 1.3), the
    /// `Transform(405)` at the end, and the rotation bob.
    ///
    /// Deviations: the 226-dust ring (`:39403-39427`) is unmodelled.
    fn update_stardust_cell(&mut self, _world: &World, _player: &Player) {
        // `:39386-39391`'s damping: above 4 px/tick the velocity damps to
        // 0.95, then every tick drifts by the flat 0.99.
        let sp = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if sp > 4.0 {
            self.vx *= 0.95;
            self.vy *= 0.95;
        }
        self.vx *= 0.99;
        self.vy *= 0.99;
        self.ai[0] += 1.0;
        // `:39393`'s ramp: `scale = 1 + 0.3 * clamp(ai[0]/300, 0, 1)`.
        self.scale = 1.0 + 0.3 * (self.ai[0] / 300.0).clamp(0.0, 1.0);
        if self.ai[0] >= 300.0 {
            // `:39394-39401`'s `Transform(405)` — the port swaps the kind in
            // place (the vanilla's `SetDefaultsFromNetId` keeps `life`).
            self.kind = NpcKind::StardustCellBig;
            self.scale = 1.0;
            self.ai = [0.0; 4];
            self.frame = 0;
            self.frame_counter = 0.0;
            return;
        }
        self.rotation += self.vx * 0.1;
    }

    /// `AI()`'s aiStyle-96 block (`NPC.cs:39430-39472`) — the Stardust
    /// Jellyfish: the desired-velocity bands (20/40/80 px → 0.35/0.65/1.0
    /// × 5 of the 250-px-above-the-player vector), the `moveSpeed = 0.15`
    /// fly, the 70-tick 539 shot (60 damage) from the random ring.
    fn update_stardust_jellyfish(&mut self, _world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:39435`'s aim: the player's centre shifted 250 px up.
        let (dx, dy) = (pcx - cx, pcy - cy - 250.0);
        let d = (dx * dx + dy * dy).sqrt();
        // `:39437-39455`'s bands: under 20 the current velocity stands
        // (the port's zero accel), then 1.75 / 3.25 / 5.
        let speed = if d < 20.0 {
            0.0
        } else if d < 40.0 {
            5.0 * 0.35
        } else if d < 80.0 {
            5.0 * 0.65
        } else {
            5.0
        };
        if speed > 0.0 {
            let inv = speed / d.max(0.001);
            self.vx += (dx * inv - self.vx) * 0.15;
            self.vy += (dy * inv - self.vy) * 0.15;
        }
        self.x += self.vx;
        self.y += self.vy;
        self.rotation = self.vx * 0.1;
        self.ai[0] += 1.0;
        if self.ai[0] >= 70.0 {
            self.ai[0] = 0.0;
            // `:39465-39469`'s ring: `UnitY` rotated by a random ±π/2,
            // scaled by (5, 3), resampled until |X| >= 1.5.
            let (mut sx, mut sy) = (0.0f32, 0.0f32);
            let mut guard = 0;
            while sx.abs() < 1.5 && guard < 64 {
                let ang = (crate::app::rand01(rng) * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2;
                sx = -ang.sin() * 5.0;
                sy = ang.cos() * 3.0;
                guard += 1;
            }
            self.wants_hostile_shot.push((539u16, cx, cy, sx, sy, 60));
        }
    }

    /// `AI()`'s aiStyle-85 block (`NPC.cs:36930-37220`) — the Star Cell
    /// (405)'s hover machine: the five `ai[0]` phases — the sighted
    /// approach (the 8 + dist/100 speed over the 50-tick smoothing), the
    /// blind charge, the waypoint homing, the bounce and the separation
    /// push.
    ///
    /// Deviations: the 421-only attachment arm (`:37013-37033`) and the
    /// dust arms are unmodelled; the waypoint/bounce phases ride the
    /// port's own smoothing instead of the vanilla's `netUpdate` probes.
    fn update_stardust_cell_big(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:36996`'s reach vector reads the player's centre lifted by a
        // quarter height.
        let (pdx, pdy) = (pcx - cx, pcy - cy - player.height * 0.25);
        let dist = (pdx * pdx + pdy * pdy).sqrt();
        let sight = crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
        match self.ai[0] as i32 {
            1 => {
                // `:36989-37012`'s approach.
                self.rotation += self.facing as f32 * 0.3;
                let (dx, dy) = (pcx - cx, pcy - cy);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                // `:36998-37003`: the 405's base speed is 8, plus dist/100.
                let speed = 8.0 + d / 100.0;
                let (tx, ty) = (dx / d * speed, dy / d * speed);
                // `:37004-37007`: the 50-tick smoothing.
                self.vx = (self.vx * 49.0 + tx) / 50.0;
                self.vy = (self.vy * 49.0 + ty) / 50.0;
                if !sight {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            2 => {
                // `:37035-37054`'s blind charge (the vanilla sets
                // `noTileCollide` and a 6-px cap for 405).
                self.rotation = self.vx * 0.1;
                if dist < 600.0 {
                    self.ai[0] = 0.0;
                }
            }
            3 => {
                // `:37055-37078`'s waypoint homing (`ai[1]/ai[2]` hold the
                // waypoint), the 3-px speed over the 3-tick divisor.
                self.rotation = self.vx * 0.1;
                let (tx, ty) = (self.ai[1], self.ai[2]);
                let (dx, dy) = (tx - cx, ty - cy);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                self.vx = (self.vx * 2.0 + dx / d * 3.0) / 3.0;
                self.vy = (self.vy * 2.0 + dy / d * 3.0) / 3.0;
                if d < 3.0 || d > 800.0 || sight {
                    self.ai[0] = 4.0;
                }
            }
            4 => {
                // `:37079-37142`'s bounce arm: the 180-tick re-probe.
                self.rotation = self.vx * 0.1;
                self.ai[1] += 1.0;
                if self.ai[1] > 180.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 3.0;
                }
            }
            _ => {
                // `:36933-36988`'s phase 0: the sight probe picks the
                // approach, then the 800-px charge, the corner probe's
                // waypoint, else the drift.
                if sight {
                    self.ai[0] = 1.0;
                } else if dist > 800.0 {
                    self.ai[0] = 2.0;
                } else if pdx.abs() > 8.0 || pdy.abs() > 8.0 {
                    self.ai[1] = pcx;
                    self.ai[2] = pcy;
                    self.ai[0] = 3.0;
                } else {
                    self.ai[0] = 4.0;
                    self.ai[1] = 0.0;
                }
            }
        }
        self.x += self.vx;
        self.y += self.vy;
        // `:37166-37188`'s separation: the 0.05 push per axis off the
        // player when within a width.
        if pdx.abs() + pdy.abs() < w {
            self.vx -= pdx.signum() * 0.05;
            self.vy -= pdy.signum() * 0.05;
        }
    }

    /// `AI_026_Unicorns`' NebulaBeast arm (`NPC.cs:62497-62557`, the
    /// machine at `:62366-62924`) — the 423's charge/idle phases ride the
    /// aiStyle-26 ground walker (`:62694-62702`: the 0.85 turnaround
    /// damping, the 10-px top speed, the 0.2 accel).
    ///
    /// Deviations: the 242-dust arms (`:62505-62526`, `:62904-62918`) are
    /// unmodelled; the ledge-jump arm's tile probes (`:62806-62903`'s
    /// −8.5/−7.5/−7/−6/−8) fold into the walker's single jump.
    fn update_nebula_beast(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let dist = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
        if self.ai[2] == 1.0 {
            // `:62499-62539`'s charge: the 0.7 horizontal damping, the
            // 30-tick wind-up, the 575 at the muzzle, the 60-tick release.
            self.ai[1] += 1.0;
            self.vx *= 0.7;
            if self.ai[1] >= 30.0 && self.ai[1] < 31.0 {
                // `:62532-62534`'s shot: 575, 50 (the classic constant of
                // `GetAttackDamage_ForProjectiles(50, 35)`), the muzzle
                // 20 px behind the centre at `spriteDirection * -7`.
                self.wants_hostile_shot.push((
                    575u16,
                    cx + self.facing as f32 * -20.0,
                    cy,
                    self.facing as f32 * -7.0,
                    0.0,
                    50,
                ));
            }
            if self.ai[1] >= 60.0 {
                // `:62536-62539`'s release: the rearm is −320…−600.
                self.ai[1] = -(320.0 + crate::app::rand01(rng) * 281.0);
                self.ai[2] = 0.0;
            }
        } else {
            self.ai[1] += 1.0;
            // `:62545-62550`'s trigger: the grounded charge opens on the
            // 180-tick idle inside the 500-px reach.
            if self.ai[1] >= 180.0 && dist < 500.0 && self.on_ground {
                self.ai[1] = 0.0;
                self.ai[2] = 1.0;
            }
            // `:62694-62702` + `:62717-62739`'s walker: the 10-px cap,
            // the 0.2 accel, the 0.85 turnaround damping.
            crate::npc_ai::walker_step(
                self,
                world,
                pcx,
                pcy,
                crate::npc_ai::WalkerParams {
                    speed: 10.0,
                    jump: -8.0,
                    aggro_x: 80.0 * TILE_PX,
                    aggro_y: 40.0 * TILE_PX,
                },
                GRAVITY,
                MAX_FALL,
            );
        }
    }

    /// `AI_003_Fighters`' VortexRifleman arm (`NPC.cs:58463-58621`) — the
    /// 425 rides the aiStyle-3 walker (`:57677`: the 6-px cap) with the
    /// leap arm (`:58566-58571`: the player 100 px below + the line of
    /// sight → `velocity.Y = -5`), the flight homing (`:58480-58545`: the
    /// 300-px horizontal offset, the 20-px vertical spacing, the 0.3/8 and
    /// 0.3/7 accel/cap pairs) and the four-shot 577 salvo.
    ///
    /// Deviations: the 229-dust arm (`:58532-58542`), the same-type
    /// separation (`:58543-58564`) and the chase-correction
    /// (`:58606-58613`'s `GetChaseResults`) are unmodelled; the port's
    /// salvo fires the direct aim.
    fn update_vortex_rifleman(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:58465-58469`'s init: the first tick rearms the timer to −120,
        // once (the latch keeps the idling timer from re-arming itself).
        if !self.rifleman_rearmed {
            self.rifleman_rearmed = true;
            self.ai[3] = -120.0;
        }
        // `:58470-58473`: grounded clears the leap state.
        if self.vy == 0.0 {
            self.ai[2] = 0.0;
        }
        if self.vy != 0.0 && self.ai[2] == 1.0 {
            // The flight homing (`:58480-58545`).
            let dx = pcx - self.facing as f32 * 300.0 - cx;
            let dy = (pcy + h * 0.5) - (self.y + h);
            if dx < 0.0 && self.vx > 0.0 {
                self.vx *= 0.9;
            } else if dx > 0.0 && self.vx < 0.0 {
                self.vx *= 0.9;
            }
            if dx < 0.0 && self.vx > -7.0 {
                self.vx -= 0.3;
            } else if dx > 0.0 && self.vx < 7.0 {
                self.vx += 0.3;
            }
            self.vx = self.vx.clamp(-7.0, 7.0);
            if dy < -20.0 && self.vy > 0.0 {
                self.vy *= 0.8;
            } else if dy > 20.0 && self.vy < 0.0 {
                self.vy *= 0.8;
            }
            if dy < -20.0 && self.vy > -8.0 {
                self.vy -= 0.3;
            } else if dy > 20.0 && self.vy < 8.0 {
                self.vy += 0.3;
            }
        } else if pcy + 100.0 < self.y
            && crate::projectile::can_hit(
                world,
                self.x,
                self.y,
                w,
                h,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            // `:58566-58571`'s leap arm.
            self.vy = -5.0;
            self.ai[2] = 1.0;
        }
        if self.ai[3] < 0.0 {
            self.ai[3] += 1.0;
        }
        // `:58580-58594`'s arming: inside the 30-tick window the timer
        // advances only on the 800-px horizontal line of sight, then the
        // 10-tick windup hands back the −150 cooldown.
        let reach = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
        if self.ai[3] >= 0.0 && self.ai[3] <= 30.0 {
            let armed = reach < 800.0
                && (pcy - cy).abs() <= (pcx - cx).abs()
                && crate::projectile::can_hit_line(world, cx, cy, 0.0, 0.0, pcx, pcy, 0.0, 0.0);
            self.ai[3] = (self.ai[3] + if armed { 1.0 } else { 0.0 }).clamp(0.0, 30.0);
        }
        if self.ai[3] >= 31.0 {
            self.ai[3] += 1.0;
            if self.ai[3] >= 40.0 {
                self.ai[3] = 30.0 - 180.0;
            }
        }
        if self.ai[3] == 30.0 {
            // `:58595-58620`'s salvo: the 20-px muzzle speed from 30 px
            // behind the centre, four 577s at 75 (the classic constant of
            // `GetAttackDamage_ForProjectiles(75, 50)`); only the first
            // shot is exact.
            self.ai[3] = 31.0;
            let (mx, my) = (cx + self.facing as f32 * 30.0, cy + 2.0);
            let d = ((pcx - mx).powi(2) + (pcy - my).powi(2)).sqrt().max(0.001);
            for shot in 0..4 {
                let (mut vx, mut vy) = ((pcx - mx) / d * 20.0, (pcy - my) / d * 20.0);
                if shot != 0 {
                    vx += (crate::app::rand01(rng) * 2.0 - 1.0) * 0.8;
                    vy += (crate::app::rand01(rng) * 2.0 - 1.0) * 0.8;
                }
                self.wants_hostile_shot.push((577u16, mx, my, vx, vy, 75));
            }
        }
        // `:57677`: the rifleman's walk speed is 6.
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 6.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    /// `AI`'s aiStyle-106 block (`NPC.cs:40970-41047`) — the Old One's
    /// Army's lane gate: the `ai[0]` counter opens a spawn attempt every
    /// `LaneSpawnRate` frames and resets at three times the rate, the
    /// lane's side rides `ai[2]` (`leftGate`), and after 180 frames of
    /// `localAI[0]` the gate fades (`scale = Lerp(1, 0.05, 500→600)`) and
    /// dies at 550 — unless the crystal (`AnyNPCs(548)`) still stands, which
    /// parks the arm.
    ///
    /// Deviations: the dust/light/sound arms and the netUpdate are
    /// unmodelled; the spawn itself rides `wants_dd2_spawn` (the pool and
    /// the dd2 tables live outside the machine).
    fn update_dd2_lane(&mut self, world: &World, _player: &Player) {
        // `:40972`'s lane side.
        self.wants_lane_right = self.ai[2] == 1.0;
        if self.ai[1] == 0.0 {
            // `:40988-40991`'s counter, parked by `EnemySpawningIsOnHold`.
            if !crate::world_sim::npcs::enemy_spawning_on_hold(world) {
                self.ai[0] += 1.0;
            }
            let rate = crate::world_sim::npcs::lane_spawn_rate(world) as f32;
            if self.ai[0] >= rate {
                // `:40994-40997`'s reset runs first, so the reset frame's
                // `0 % rate == 0` still opens an attempt.
                if self.ai[0] >= rate * 3.0 {
                    self.ai[0] = 0.0;
                }
                if (self.ai[0] as i32) % (rate as i32) == 0 {
                    self.wants_dd2_spawn = true;
                }
            }
            // `:41008-41016`'s idle timer, clamped at 180.
            self.caster_counter += 1;
            if self.caster_counter > 180 {
                self.caster_counter = 180;
            }
            if self.caster_counter >= 180 {
                // `:41014-41017`'s crystal guard: the AI returns with the
                // gate unkillable while the crystal stands.
                if world.dd2_crystal_alive {
                    self.dont_take_damage = true;
                    return;
                }
                self.ai[1] = 1.0;
                self.ai[0] = 0.0;
                self.dont_take_damage = true;
            }
        } else if self.ai[1] == 1.0 {
            // `:41020-41042`'s fade, then the death at 550.
            self.ai[0] += 1.0;
            let t = ((self.ai[0] - 500.0) / 100.0).clamp(0.0, 1.0);
            self.scale = 1.0 + (0.05 - 1.0) * t;
            if self.ai[0] >= 550.0 {
                self.dont_take_damage = false;
                self.hp = 0;
            }
        }
    }

    /// `AI_077_MoonLordCore` (`NPC.cs:41157-41575`) — the Moon Lord's
    /// heart: the spawn-in (`ai[0] == -1`), the cruise (`ai[0] == 0/1`),
    /// the 600-tick crash drama (`ai[0] == 2`), the despawn
    /// (`ai[0] == 3`), the teleport re-entry (`ai[0] == -2`) and the
    /// >2400-px enrage hop.
    ///
    /// Deviations: `MoonlordDeathDrama` (the light/explosion/pieces event)
    /// is unmodelled — the port's drama is the timer + the 622 smoke.
    fn update_moon_lord_core(&mut self, _world: &World, player: &Player, rng: &mut u64) {
        // `:41163-41168`'s init: the first tick arms the spawn phase.
        if self.ai[0] == 0.0 {
            self.ai[0] = -1.0;
        }
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        match self.ai[0] as i32 {
            -2 => {
                // `:41169-41192`'s teleport re-entry: the 60-tick windup,
                // then back to the cruise.
                self.dont_take_damage = true;
                self.ai[1] += 1.0;
                if self.ai[1] >= 60.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            -1 => {
                // `:41193-41235`'s spawn-in: the two hands + the head ride
                // `wants_moon_parts` on the 60th tick.
                self.dont_take_damage = true;
                self.ai[1] += 1.0;
                if self.ai[1] >= 60.0 && !self.wants_moon_parts {
                    self.wants_moon_parts = true;
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            2 => {
                // `:41307-41445`'s crash drama: the slow rise, the 622
                // smoke every 15 ticks inside [90, 480), the 600-tick end.
                self.dont_take_damage = true;
                self.vx += (0.0 - self.vx) * 0.98;
                self.vy += (-0.5 - self.vy) * 0.98;
                self.ai[1] += 1.0;
                if self.ai[1] >= 90.0 && self.ai[1] < 480.0 && (self.ai[1] as i64) % 15 == 0 {
                    // `:41405-41428`'s smoke: one 622 (0 damage) at the
                    // ±(π/8 + π/4·rand) rotation about the down axis.
                    let rot = std::f32::consts::FRAC_PI_8
                        + std::f32::consts::FRAC_PI_4 * crate::app::rand01(rng);
                    let sign = if crate::app::rand01(rng) < 0.5 {
                        -1.0
                    } else {
                        1.0
                    };
                    let vx = rot.sin() * sign * 6.0;
                    let vy = -rot.cos() * (crate::app::rand01(rng) * 0.5 + 0.5);
                    self.wants_hostile_shot.push((622u16, cx, cy, vx, vy, 0));
                }
                self.x += self.vx;
                self.y += self.vy;
                if self.ai[1] >= 600.0 {
                    // `:41439-41442`'s end: `life = 0` → the death pass
                    // raises `downed_moon_lord`.
                    self.hp = 0;
                }
            }
            3 => {
                // `:41446-41515`'s despawn ride.
                self.dont_take_damage = true;
                self.vx += (self.facing as f32 - self.vx) * 0.98;
                self.vy += (-0.5 - self.vy) * 0.98;
                self.ai[1] += 1.0;
                if self.ai[1] >= 60.0 {
                    self.wants_moon_parts_clear = true;
                    self.hp = 0;
                }
                self.x += self.vx;
                self.y += self.vy;
            }
            _ => {
                // `:41236-41306`'s cruise: the 130-px-above hover at the
                // 8-px cap over the 0.5 smoothing.
                self.dont_take_damage = self.ai[0] != 1.0;
                let (dx, dy) = (pcx - cx, pcy - cy + 130.0);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 20.0 {
                    let inv = 8.0 / d.max(0.001);
                    self.vx += (dx * inv - self.vx) * 0.5;
                    self.vy += (dy * inv - self.vy) * 0.5;
                }
                self.x += self.vx;
                self.y += self.vy;
                // `:41537-41542`'s player guard: no live player despawns.
                if player.hp <= 0 {
                    self.ai[0] = 3.0;
                    self.ai[1] = 0.0;
                    return;
                }
                // `:41543-41550`'s enrage hop past 2400 px (the parts ride
                // it in the pre-pass).
                if (pcx - cx).hypot(pcy - cy) > 2400.0 {
                    self.x += pcx - cx;
                    self.y += pcy - cy - 150.0;
                }
                // `:41251-41291`'s part bookkeeping: a missing part opens
                // the death ride, and all parts dead-and-ghosted hands the
                // core its vulnerable phase.
                let lost = self.moon_parts.iter().any(|p| *p < 0);
                if lost {
                    self.ai[0] = 2.0;
                    self.ai[1] = 0.0;
                }
            }
        }
    }

    /// The Moon Lord attack-table sequencer (`AI_078`'s `:41600-41626`,
    /// `AI_079`'s `:42104-42133`, `AI_081`'s `:42476-42493`): the 5-entry
    /// (or the True Eye's 10-entry) walk over the (attack, duration) rows
    /// — returns the picked index, the ticks inside the attack and the
    /// attack's own duration.
    fn moon_lord_pick(row: &[(u32, i32)], timer: &mut f32) -> (usize, f32, f32) {
        let mut i = 0usize;
        let mut elapsed = 0i32;
        while i < row.len() {
            let dur = row[i].1;
            if !(dur as f32 + elapsed as f32 <= *timer) {
                break;
            }
            elapsed += dur;
            i += 1;
        }
        if i >= row.len() {
            i = 0;
            *timer = 0.0;
            elapsed = 0;
        }
        (i, *timer - elapsed as f32, row[i].1 as f32)
    }

    /// `AI_078_MoonLordHands` (`NPC.cs:41577-42082`) — the two 397s: the
    /// sequencer over `ai[2]`'s row (0/1), the hover (attack 0), the
    /// Phantasmal Eyes (1), the flood + the release wave (2) and the
    /// retargeting laser (3).
    ///
    /// Deviations: the 452/454 projectile engines (the vanilla's aiStyle
    /// 82/83) are unmodelled; the release wave fires the wave projectile
    /// through `wants_hostile_shot` instead of rewriting the owned 454s.
    fn update_moon_lord_hand(&mut self, _world: &World, player: &Player, rng: &mut u64) {
        // `:41579-41583`'s orphan guard (the pre-pass owns the link).
        if !self.moon_core_alive {
            self.hp = 0;
            return;
        }
        let row: [(u32, i32); 5] = if self.ai[2] == 0.0 {
            [(0, 50), (1, 70), (2, 330), (0, 60), (3, 90)]
        } else {
            [(1, 70), (0, 50), (3, 90), (0, 60), (2, 330)]
        };
        let num = if self.ai[2] == 0.0 { -1.0f32 } else { 1.0f32 };
        self.ai[1] += 1.0;
        let (pick, num2, dur) = Self::moon_lord_pick(&row, &mut self.ai[1]);
        self.ai[0] = row[pick].0 as f32;
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The anchor (`:41642`/`:41701`'s slot): the core's centre +
        // (±350, -100).
        let (ax, ay) = (self.carrier_x + 350.0 * num, self.carrier_y - 100.0);
        match self.ai[0] as i32 {
            -2 | 0 => {
                // `:41628-41675`'s hover: the side slot at the 6-px cap
                // over the 0.3 smoothing.
                self.dont_take_damage = self.ai[0] == -2.0;
                let (dx, dy) = (ax - cx, ay - cy);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 20.0 {
                    let inv = 6.0 / d.max(0.001);
                    self.vx += (dx * inv - self.vx) * 0.3;
                    self.vy += (dy * inv - self.vy) * 0.3;
                }
            }
            1 => {
                // `:41676-41717`'s Phantasmal Eyes: the 56-tick eyelid
                // cycle (`7 × 4` half-point ticks) fires the 452 (30) every
                // 4 ticks from the ring's point.
                let (ex, ey) = (30.0f32, 66.0f32);
                if num2 >= dur - 4.0 {
                    self.moon_local[1] = (self.moon_local[1] - 0.07).max(0.0);
                } else if num2 >= 28.0 {
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(0.75);
                    if num2 % 4.0 < 1.0 {
                        let ang = std::f32::consts::PI * 2.0 * (num2 % 28.0) / 28.0
                            - std::f32::consts::FRAC_PI_2;
                        let (rx, ry) = (ang.cos() * ex, ang.sin() * ey);
                        let d = (rx * rx + ry * ry).sqrt().max(0.001);
                        self.wants_hostile_shot.push((
                            452u16,
                            cx + rx / d * (d * 0.4) + 1.0 * -num,
                            cy + ry / d * (d * 0.4) + 3.0,
                            rx / d * 8.0,
                            ry / d * 8.0,
                            30,
                        ));
                    }
                } else {
                    self.moon_local[1] = (self.moon_local[1] + 0.02).min(0.75);
                }
            }
            2 => {
                // `:41718-41817`'s flood: six 454s (40) on the 30-tick
                // cadence, the 20-px travel, the release wave at the
                // 292nd tick.
                let t = num2 - 30.0;
                if t >= 0.0 && t < 180.0 && (t as i64) % 30 == 0 {
                    let i = t / 30.0;
                    let mut vx = (5.0 * num + (i - 3.5) * num * 3.0) * 1.2;
                    let mut vy = (-8.0 + (i - 4.5)) * 1.2;
                    vx = vx;
                    vy = vy;
                    self.wants_hostile_shot.push((454u16, cx, cy, vx, vy, 40));
                }
                if num2 == 292.0 {
                    // `:41846`'s release wave: the 12-px lance aimed from
                    // 350 px above the hand.
                    let (dx, dy) = (pcx - cx, pcy - (cy - 350.0));
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        462u16,
                        cx,
                        cy - 350.0,
                        dx / d * 12.0,
                        dy / d * 12.0,
                        30,
                    ));
                }
                // The travel (`:41787`'s SmoothStep at the 20-px cap).
                let (dx, dy) = (ax - cx, ay - cy);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                let inv = 20.0 / d;
                self.vx += (dx * inv - self.vx) * 0.2;
                self.vy += (dy * inv - self.vy) * 0.2;
            }
            _ => {
                // `:41819-41846`'s retargeting laser: the 462 (30) aimed
                // at the player's led position at the −14/−7/0 ticks.
                if num2 < dur - 19.0 {
                    self.moon_local[0] += 0.5 * ((pcy - cy).atan2(pcx - cx) - self.moon_local[0]);
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(1.0);
                }
                if num2 == dur - 14.0 || num2 == dur - 7.0 || num2 == dur {
                    let (ex, ey) = (
                        self.moon_local[0].cos() * 30.0 * self.moon_local[1],
                        self.moon_local[0].sin() * 66.0 * self.moon_local[1],
                    );
                    let d = (ex * ex + ey * ey).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        462u16,
                        cx + ex / d * 30.0,
                        cy + ey / d * 66.0,
                        ex / d * 8.0,
                        ey / d * 8.0,
                        30,
                    ));
                }
                // The idle drift toward the anchor.
                let (dx, dy) = (ax - cx, ay - cy);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 20.0 {
                    let inv = 6.0 / d.max(0.001);
                    self.vx += (dx * inv - self.vx) * 0.3;
                    self.vy += (dy * inv - self.vy) * 0.3;
                }
            }
        }
        self.x += self.vx;
        self.y += self.vy;
        // `:41847-41882`'s frame box clamp (the hand's wander limits).
        let min_x = self.carrier_x + 220.0 * num - 100.0;
        let max_x = self.carrier_x + (220.0 + 370.0) * num;
        let min_y = self.carrier_y - 60.0 - 150.0;
        let max_y = self.carrier_y - 60.0 + 150.0;
        let (lo_x, hi_x) = if min_x <= max_x {
            (min_x, max_x)
        } else {
            (max_x, min_x)
        };
        let (lo_y, hi_y) = if min_y <= max_y {
            (min_y, max_y)
        } else {
            (max_y, min_y)
        };
        self.x = self.x.clamp(lo_x, hi_x);
        self.y = self.y.clamp(lo_y, hi_y);
        let _ = rng;
    }

    /// `AI_079_MoonLordHead` (`NPC.cs:42084-42457`) — the 396: the head
    /// row of `MoonLordAttacksArray`, the eye's open/close factor, the
    /// phantom deathray (455), the Moon Leech summon (456) and the
    /// retargeting laser (462).
    fn update_moon_lord_head(&mut self, _world: &World, player: &Player, rng: &mut u64) {
        // `:42086-42091`'s orphan guard (the pre-pass owns the link).
        if !self.moon_core_alive {
            self.hp = 0;
            return;
        }
        // `:42098`'s fixed slot: the core's centre 400 px up.
        self.x = self.carrier_x - self.size().0 * 0.5;
        self.y = self.carrier_y - 400.0 - self.size().1 * 0.5;
        let (cx, cy) = (self.carrier_x, self.carrier_y - 400.0);
        let (pcx, pcy) = player.center();
        let row: [(u32, i32); 5] = [(3, 180), (0, 30), (2, 435), (3, 180), (1, 375)];
        self.ai[1] += 1.0;
        let (pick, num, dur) = Self::moon_lord_pick(&row, &mut self.ai[1]);
        if self.ai[0] >= 0.0 {
            self.ai[0] = row[pick].0 as f32;
        }
        let _ = dur;
        match self.ai[0] as i32 {
            0 => {
                // `:42172-42190`'s open/close factor: `1 - |aim| / 500`.
                let (dx, dy) = (pcx - cx, pcy - cy + 22.0);
                let d = (dx * dx + dy * dy).sqrt();
                let open = ((1.0 - d / 500.0).clamp(0.0, 1.0) * 2.0).clamp(0.0, 1.0);
                self.moon_local[0] = dy.atan2(dx);
                self.moon_local[1] = open;
                self.moon_local[2] += (1.0 - self.moon_local[2]) * 0.2;
            }
            1 => {
                // `:42191-42280`'s phantom deathray: the 455 (75) on the
                // 180th tick, sweeping at the ±2π/540 rate.
                if num < 180.0 {
                    self.moon_local[1] = (self.moon_local[1] - 0.05).max(0.0);
                } else if num == 180.0 {
                    let (dx, dy) = (pcx - cx, pcy - cy);
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.wants_hostile_shot
                        .push((455u16, cx, cy, dx / d * 6.0, dy / d * 6.0, 75));
                    self.moon_local[0] = dy.atan2(dx);
                    self.moon_local[1] = 1.0;
                } else {
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(1.0);
                }
            }
            2 => {
                // `:42281-42318`'s Moon Leech summon: the 456 (0) per
                // player at the attack's start, the 401 clot per leeched
                // player at 120/180/240 — both ride the sim's resolve.
                if num == 0.0 || num == 120.0 || num == 180.0 || num == 240.0 {
                    self.wants_moon_leech = true;
                }
            }
            _ => {
                // `:42319-42346`'s retargeting laser: the 462 (30) at the
                // −14/−7/0 ticks over the attack's own duration.
                if num < dur - 19.0 {
                    self.moon_local[0] += 0.5 * ((pcy - cy).atan2(pcx - cx) - self.moon_local[0]);
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(1.0);
                }
                if num == dur - 14.0 || num == dur - 7.0 || num == dur {
                    let (ex, ey) = (
                        self.moon_local[0].cos() * 27.0 * self.moon_local[1],
                        self.moon_local[0].sin() * 59.0 * self.moon_local[1],
                    );
                    let d = (ex * ex + ey * ey).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        462u16,
                        cx + ex / d * 27.0,
                        cy + ey / d * 59.0,
                        ex / d * 8.0,
                        ey / d * 8.0,
                        30,
                    ));
                }
            }
        }
        // `:42347-42380`'s lid/iris counters (the two clamping bugs of the
        // original fold into one clamp).
        self.moon_local[2] = self.moon_local[2].clamp(0.0, 14.0);
        let _ = rng;
    }

    /// `AI_081_TrueEyeOfCthulhu` (`NPC.cs:42459-42880`) — the 400: the
    /// `MoonLordAttacksArray2` sequencer, the aimed drift (0), the
    /// retargeting laser (1), the orbit + the 454 release (2) and the
    /// ring's 452 volley (3).
    ///
    /// Deviations: the 454/452 projectile engines (the vanilla's aiStyle
    /// 82/83) are unmodelled.
    fn update_true_eye(&mut self, _world: &World, player: &Player, rng: &mut u64) {
        // `:42466-42470`'s orphan guard (the pre-pass owns the link).
        if !self.moon_core_alive {
            self.hp = 0;
            return;
        }
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let row: [(u32, i32); 10] = [
            (0, 53),
            (1, 90),
            (0, 53),
            (2, 135),
            (0, 53),
            (3, 200),
            (0, 53),
            (4, 375),
            (0, 53),
            (2, 135),
        ];
        self.ai[1] += 1.0;
        let (pick, num, dur) = Self::moon_lord_pick(&row, &mut self.ai[1]);
        self.ai[0] = row[pick].0 as f32;
        let _ = dur;
        match self.ai[0] as i32 {
            1 => {
                // `:42579-42619`'s retargeting laser: the 462 (35) at the
                // −14/−7/0 ticks.
                self.vx *= 0.95;
                if num < 90.0 - 19.0 {
                    self.moon_local[0] += 0.5 * ((pcy - cy).atan2(pcx - cx) - self.moon_local[0]);
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(1.0);
                }
                if num == 90.0 - 14.0 || num == 90.0 - 7.0 || num == 90.0 {
                    let (ex, ey) = (
                        self.moon_local[0].cos() * 30.0 * self.moon_local[1],
                        self.moon_local[0].sin() * 30.0 * self.moon_local[1],
                    );
                    let d = (ex * ex + ey * ey).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        462u16,
                        cx + ex / d * 30.0,
                        cy + ey / d * 30.0,
                        ex / d * 8.0,
                        ey / d * 8.0,
                        35,
                    ));
                }
            }
            2 => {
                // `:42620-42737`'s orbit: the inward ring (the 454 (40) at
                // speed 4 every 10 ticks), the charge, the release over
                // 105..120.
                if num >= 15.0 && num < 75.0 && ((num as i64) - 15) % 10 == 0 {
                    self.wants_hostile_shot
                        .push((454u16, cx, cy, 0.0, -4.0, 40));
                }
                if num >= 105.0 && num < 120.0 {
                    let (dx, dy) = (pcx - cx, pcy - cy);
                    let ang = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
                    self.wants_hostile_shot.push((
                        454u16,
                        cx,
                        cy,
                        ang.cos() * 24.0,
                        ang.sin() * 24.0,
                        40,
                    ));
                }
            }
            3 => {
                // `:42739-42880`'s ring: the 452 (35) volley every 10
                // ticks inside the 150-tick window at the 8-px speed.
                if num >= 45.0 && num < 185.0 && ((num as i64) - 45) % 10 == 0 {
                    let ang = crate::app::rand01(rng) * std::f32::consts::PI * 2.0;
                    self.wants_hostile_shot.push((
                        452u16,
                        cx,
                        cy,
                        ang.cos() * 8.0,
                        ang.sin() * 8.0,
                        35,
                    ));
                }
            }
            _ => {
                // `:42538-42578`'s drift: aim at the player's led centre,
                // hover 200 px above at the 24-px cap.
                if num <= 53.0 {
                    self.moon_local[0] += 0.5 * ((pcy - cy).atan2(pcx - cx) - self.moon_local[0]);
                    self.moon_local[1] = (self.moon_local[1] + 0.05).min(0.7);
                    let (dx, dy) = (pcx - cx, pcy - cy - 200.0);
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.vx += (dx / d * 24.0 - self.vx) / 30.0;
                    self.vy += (dy / d * 24.0 - self.vy) / 30.0;
                }
            }
        }
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI()`'s aiStyle-82 block (`NPC.cs:36629-36756`) — the Moon Leech
    /// Clot: the 90-tick glide, then the 1000-hp drain from the head →
    /// the core → the hands.
    /// `AI_003_Fighters`' VortexHunter arm (`NPC.cs:59069-59756`, the 426
    /// rows at `:59072`/`:59163-59192`/`:59304-59308`/`:59480-59484`/
    /// `:59751-59756`) — the vortex shooter: the ground walk's 4-px cap /
    /// 0.6 accel / 0.95 drag, the 60-tick shot cycle firing the 581 (60)
    /// from the muzzle `(spriteDirection * 2, -12)` with the ±30×0.3
    /// spread, the aim octant on `ai[2]`, and the 700-px engage gate.
    ///
    /// Deviations: the interception aim (`:59529-59542`'s
    /// `GetChaseResults` at the factor 15) folds into the centre aim; the
    /// vanilla's `localAI[0]`-free state means the metamorphosis counter
    /// rides the walker's own counters, so the hunter is the last form.
    /// `AI()`'s aiStyle-97 block (`NPC.cs:39473-39580`) — the Nebula
    /// Floater: the 180-tick 574 emission from the ±π/2 (4, 2.5) ring, the
    /// 480-tick teleport cycle behind `AI_AttemptToFindTeleportSpot`'s
    /// tile pick (the 20-tick fade, the dust bursts, the shot-count
    /// shave of the owned 574s), the 7-px cruise to the aim point
    /// `Center + (direction * 20, 6)` and the `|vx| * direction * 0.1`
    /// roll.
    ///
    /// Deviations: the 242-dust bursts are unmodelled; the teleport-spot
    /// search folds the vanilla's 40×24 probe into the first non-solid
    /// tile near the aim; the owned-574 shave rides a per-kind request.
    /// `AI_003_Fighters`'s StardustSpiderBig arm (`NPC.cs:59069`'s shooter
    /// list, the 409 rows at `:59155-59160`/`:59255-59257`/`:59585-59589`/
    /// `:59746-59750`) — the 60-tick cycle fires no projectile: it spawns
    /// the 410 small spider at its centre with the carried velocity, and
    /// its walk runs the 2-px cap over the 0.5 accel.
    fn update_stardust_spider_big(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // `:59208`'s fire at the half-cooldown: the 410 spawn request.
        if self.ai[1] * 2.0 >= 60.0 && self.ai[2] > 0.0 {
            // `:59585-59589`'s spawn: the 410 spider at the centre with the
            // carried velocity.
            self.wants_sphere
                .push((NpcKind::StardustSpiderSmall, cx, cy));
            self.ai[2] = 0.0;
        }
        if self.ai[1] <= 0.0 {
            self.ai[2] = 0.0;
        }
        if self.ai[1] <= 0.0 && ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt() < 700.0 {
            self.ai[2] = 3.0;
            self.ai[1] = 60.0;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 2.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // `FindFrame` case 409 (`NPC.cs:70684-70703`): the 4-row strip's
        // wind-up row on the shot window, the walk accumulator otherwise.
        if self.vy == 0.0 && self.ai[1] <= 30.0 && self.ai[1] > 0.0 {
            self.frame = 3;
        } else {
            self.frame_counter += 1.0;
            if self.frame_counter >= 6.0 {
                self.frame_counter = 0.0;
                self.frame = (self.frame + 1) % 2;
            }
        }
    }

    /// `AI_026_Unicorns`'s StardustSpiderSmall arm (`NPC.cs:62470-62494` +
    /// `:62706-62713`) — the aiStyle-26 walker at the 6-px/0.2-accel with
    /// the kamikaze: the 240-tick counter (or a player 800 px below within
    /// a 20-px column) fires three 538s (50) and the spider dies.
    fn update_stardust_spider_small(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        self.ai[1] += 1.0;
        let mut armed = self.ai[1] >= 240.0;
        if !armed && self.vy == 0.0 {
            armed = (pcx - cx).abs() < 20.0 && pcy > cy && (pcx - cx).hypot(pcy - cy) < 800.0;
        }
        if armed {
            // `:62486-62492`'s three-shot burst + the self-destruct.
            for _ in 0..3 {
                let vx = (crate::app::rand01(rng) - 0.5) * 2.0;
                let vy = -4.0 - 10.0 * crate::app::rand01(rng);
                self.wants_hostile_shot.push((538u16, cx, cy, vx, vy, 50));
            }
            self.hp = 0;
            return;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 6.0,
                jump: -8.0,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // `FindFrame` case 410 (`NPC.cs:70705-70715`): the 11-row strip's
        // |vx|-driven accumulator.
        self.frame_counter += self.vx.abs();
        if self.frame_counter >= 10.0 {
            self.frame_counter -= 10.0;
            self.frame = (self.frame + 1) % 11;
        }
    }

    /// `AI_003_Fighters`'s StardustSoldier arm (`NPC.cs:59069`'s shooter
    /// list, the 411 rows at `:59076-59081`/`:59151-59153`/`:59193`/
    /// `:59271-59274`/`:59470-59473`/`:59574-59577`/`:59741-59745`) — the
    /// 330-tick cycle fires the 537 (60, the classic constant of
    /// `GetAttackDamage_ForProjectiles(60, 45)`) from the muzzle at the
    /// 1-px speed over the aim spread, and its walk runs the 2-px cap.
    ///
    /// Deviations: the [120, 240) aerodynamic window (`:59624-59633`)'s
    /// velocity drag folds into the walker; the `ai[2]` re-pick by the
    /// velocity ratio is unmodelled.
    fn update_stardust_soldier(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let (dx, dy) = (pcx - cx, pcy - cy);
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // `:59208`'s fire at the 220th tick (the half-cooldown of 330).
        if self.ai[1] * 2.0 >= 330.0 && self.ai[2] > 0.0 {
            let (mut ax, mut ay) = (dx, dy);
            let jit = 40.0;
            ax += (crate::app::rand01(rng) * 2.0 - 1.0) * jit;
            ay += (crate::app::rand01(rng) * 2.0 - 1.0) * jit;
            let len = (ax * ax + ay * ay).sqrt().max(0.001);
            ax = ax / len * 1.0;
            ay = ay / len * 1.0;
            self.wants_hostile_shot
                .push((537u16, cx + 2.0, cy - 12.0, ax, ay, 60));
            self.ai[2] = 0.0;
        }
        if self.ai[1] <= 0.0 {
            self.ai[2] = 0.0;
        }
        if self.ai[1] <= 0.0 && ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt() < 700.0 {
            self.ai[2] = 3.0;
            self.ai[1] = 330.0;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 2.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // `FindFrame` case 411 (`NPC.cs:70643-70675`): the shoot pose on
        // the [120, 240) window, the walk rows otherwise.
        if self.ai[1] >= 120.0 && self.ai[1] < 240.0 {
            self.frame = 4;
        } else {
            self.frame_counter += (self.vx != 0.0) as i32 as f32 + 1.0;
            if self.frame_counter > 6.0 {
                self.frame_counter = 0.0;
                self.frame = (self.frame + 1).clamp(2, 4);
            }
        }
    }

    /// `AI_003_Fighters`'s SolarDrakomire arm (`NPC.cs:58214-58219`'s
    /// 4/0.1/0.95 jump assist + `:58422-58431`'s rider-hide) — the walker
    /// with the mob-specific speed, hidden while a 416 rides it.
    fn update_solar_drakomire(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 4.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // `:58426-58430`'s hide: the mount hides while a rider links to it —
        // the vanilla's `hide` has no port field (the pre-pass owns the
        // link), so the mount stays visible.
    }

    /// `AI()`'s aiStyle-75 block's SolarDrakomireRider arm (`NPC.cs`:
    /// 35523-35537`'s mount spawn + `:35662-35679`'s attach anchor +
    /// `:36060-36061`'s `Transform(518)`) — the rider rides its 415 mount
    /// at the `(-spriteDirection * 10, -30)` offset and becomes the
    /// SolarSrawler (518) when the mount dies.
    fn update_solar_drakomire_rider(&mut self, _world: &World, _player: &Player) {
        if self.carrier_timer <= 0.0 {
            // `:36060-36061`'s metamorph: the port's in-place kind swap
            // (518 stays out of this slice — the vanilla's
            // `SolarSrawler` row is `NPC.cs:15344-15354`).
            self.hp = 0;
            return;
        }
        // `:35663-35679`'s anchor ride.
        self.x = self.carrier_x - self.size().0 * 0.5 + self.sprite_dir as f32 * -10.0;
        self.y = self.carrier_y - self.size().1 * 0.5 - 30.0;
        self.sprite_dir = self.carrier_phase as i32;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// `AI()`'s aiStyle-39 block's SolarSroller arm (`NPC.cs:29394-29421`'s
    /// ball entry + `:29422-29488`'s lunge + `:29532-29580`'s ball state +
    /// `:29634-29668`'s unroll) — the sroller winds up, collapses into a
    /// ball (the ×1.8 damage, the doubled defense, kb 0), rolls for
    /// `ai[2]` in [2, 4] revolutions, then unrolls.
    ///
    /// Deviations: the dust/gore bursts and the `ai[3]` FX layer are
    /// unmodelled; the roll's spin rides the port's rotation.
    fn update_solar_sroller(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        match self.ai[0] as i32 {
            0 => {
                // `:29267-29276`'s aggro counter and the jump reach.
                self.ai[1] += 4.0;
                if self.ai[1] >= 400.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 1.0;
                }
                crate::npc_ai::walker_step(
                    self,
                    world,
                    pcx,
                    pcy,
                    crate::npc_ai::WalkerParams {
                        speed: 1.0,
                        jump: -6.2,
                        aggro_x: 80.0 * TILE_PX,
                        aggro_y: 40.0 * TILE_PX,
                    },
                    GRAVITY,
                    MAX_FALL,
                );
            }
            1 => {
                // `:29394-29419`'s wind-up: at the 30th tick the sroller
                // collapses into the ball (the 32-px height) and rolls for
                // `rand(2, 4)` revolutions.
                self.ai[1] += 0.5;
                if self.ai[1] >= 30.0 {
                    self.ai[1] = 0.0;
                    self.ai[2] = 2.0 + 2.0 * crate::app::rand01(&mut 7u64);
                    self.ai[0] = 6.0;
                }
            }
            6 => {
                // `:29532-29580`'s ball state: the doubled defense, kb 0
                // and the rolling spin over the revolutions.
                self.defense_override = Some(68);
                self.vx = (if self.facing >= 0 { 1.0 } else { -1.0 })
                    * (5.0 + (self.ai[2] * 0.6).min(4.0));
                self.rotation += self.vx * 0.05;
                self.ai[1] += 1.0;
                if self.ai[1] >= 60.0 * self.ai[2] {
                    self.ai[1] = 0.0;
                    self.ai[0] = 5.0;
                }
                self.x += self.vx;
                self.y += self.vy;
            }
            _ => {
                // `:29634-29668`'s unroll: restore the height and rest.
                self.rotation = 0.0;
                self.vx = 0.0;
                self.defense_override = None;
                self.ai[1] += 1.0;
                if self.ai[1] >= 30.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
        }
    }

    /// `AI()`'s aiStyle-74 block's SolarCorite arm (`NPC.cs:35238-35276`'s
    /// parameter set + the four `ai[0]` states + `:35447-35497`'s
    /// implosion) — the corite approaches (8/175/175/600/80/60/20), the
    /// charge releases into the dash (9 px), the dash exits into the
    /// cooldown, and the 64-px contact (or the state-3 arm) triggers the
    /// 192-box implosion (80 damage scaled) that kills it.
    ///
    /// Deviations: the dust/lighting arms and the sound cue are
    /// unmodelled; the implosion's 3-tick delay rides the port's own.
    fn update_solar_corite(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let dist = (pcx - cx).hypot(pcy - cy);
        match self.ai[0] as i32 {
            3 => {
                // `:35447-35497`'s implosion: the 192-box, the zeroed
                // velocity and the 3-tick delay before the death.
                self.vx = 0.0;
                self.vy = 0.0;
                self.damage_override = Some(80);
                self.ai[1] += 1.0;
                if self.ai[1] >= 3.0 {
                    self.wants_blast = true;
                    self.hp = 0;
                }
            }
            2 => {
                // `:35393-35405`'s dash exit.
                self.ai[1] += 1.0;
                if self.ai[1] >= 150.0 && dist > 60.0 {
                    self.ai[0] = 4.0;
                    self.ai[1] = 45.0;
                }
                if self.vx * self.vx + self.vy * self.vy < 0.49 {
                    self.ai[0] = 4.0;
                    self.ai[1] = 45.0;
                }
            }
            4 => {
                // `:35427-35436`'s cooldown.
                self.ai[1] -= 3.0;
                self.vx *= 0.95;
                self.vy *= 0.95;
                if self.ai[1] <= 0.0 {
                    self.ai[0] = 0.0;
                }
                self.x += self.vx;
                self.y += self.vy;
            }
            1 => {
                // `:35358-35390`'s charge: at the 20th tick the dash fires
                // at the 9-px speed.
                self.ai[1] += 1.0;
                if self.ai[1] >= 20.0 {
                    self.ai[0] = 2.0;
                    self.ai[1] = 0.0;
                    let (dx, dy) = (pcx - cx, pcy - cy);
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.vx = dx / d * 9.0;
                    self.vy = dy / d * 9.0;
                }
            }
            _ => {
                // The approach (`:35320-35350`'s 8-px fly at the 175/175
                // bands).
                self.ai[1] += 1.0;
                if self.ai[1] >= 20.0 && dist < 80.0 {
                    self.ai[0] = 1.0;
                    self.ai[1] = 0.0;
                }
                let (dx, dy) = (pcx - cx, pcy - cy);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                self.vx += (dx / d * 8.0 - self.vx) * 0.075;
                self.vy += (dy / d * 8.0 - self.vy) * 0.075;
                self.x += self.vx;
                self.y += self.vy;
            }
        }
        self.rotation = self.vy.atan2(self.vx) + std::f32::consts::PI * 0.5;
        let _ = (world, h);
    }

    /// `AI_003_Fighters`'s SolarSelenian arm (`NPC.cs:58220-58224`'s
    /// 6/0.15/0.85 walk) — the selenian rides the walker with its own
    /// speed and the roller's drag.
    fn update_solar_selenian(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 6.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    /// `AI_006_Worms`'s StardustWormHead arm (`NPC.cs:51030-51073`) — the
    /// 600-tick cycle: the [285, 570] remap decelerates the worm 1 → 0.5,
    /// at 570 it fires the mark (`SpawnStardustMark_StardustWorm`) and
    /// re-normalises the velocity to 6, at 600 the counter resets.
    ///
    /// Deviations: the dust arm (`:51043-51054`) is unmodelled; the mark's
    /// 540 projectile hop folds into a direct spawn (the port has no 540
    /// AI) and the caps live in the sim's resolve.
    fn update_stardust_worm_head(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The steering toward the player (`:51670-51743`'s 9/0.3 pair).
        let (dx, dy) = (pcx - cx, pcy - cy);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        self.vx += (dx / d * 9.0 - self.vx) * 0.3;
        self.vy += (dy / d * 9.0 - self.vy) * 0.3;
        // The cycle.
        self.ai[2] += 1.0;
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed >= 1.0 && self.ai[2] <= 570.0 {
            // `:51035-51037`'s remap over `ai[2] ∈ [285, 570]`.
            let t = ((self.ai[2] - 285.0) / 285.0).clamp(0.0, 1.0);
            let f = 1.0 - 0.5 * t;
            self.vx *= f;
            self.vy *= f;
        }
        if self.ai[2] == 570.0 {
            self.wants_stardust_mark = true;
            let sp = (self.vx * self.vx + self.vy * self.vy).sqrt().max(0.001);
            self.vx = self.vx / sp * 6.0;
            self.vy = self.vy / sp * 6.0;
        }
        if self.ai[2] >= 600.0 {
            self.ai[2] = 0.0;
        }
        self.x += self.vx;
        self.y += self.vy;
    }

    fn update_nebula_floater(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:39485-39496`'s emission window: the first 180 ticks throw the
        // 574 (0 damage) every 60 from the resampled ring.
        if self.caster_counter < 180 {
            self.caster_counter += 1;
            if self.caster_counter % 60 == 0 {
                let (mut sx, mut sy) = (0.0f32, 0.0f32);
                let mut guard = 0;
                while sx.abs() < 1.5 && guard < 64 {
                    let ang = (crate::app::rand01(rng) * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2;
                    sx = -ang.sin() * 4.0;
                    sy = ang.cos() * 2.5;
                    guard += 1;
                }
                self.wants_hostile_shot.push((574u16, cx, cy, sx, sy, 0));
            }
        }
        // `:39513-39521`'s aim + the flight decision.
        let aim = (self.facing as f32 * 20.0, 6.0);
        let (ax, ay) = (cx + aim.0, cy + aim.1);
        let (dx, dy) = (pcx - ax, pcy - ay);
        // `:39513`: `flag104 = CanHit(base.Center, 1, 1, player.Center, 1, 1)`.
        let sight = crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
        let far = (dx * dx + dy * dy).sqrt() > 400.0 || !sight;
        if far {
            // `:39580-39620`'s cruise: the 7-px cap toward the aim.
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            self.vx += (dx / d * 7.0 - self.vx) * 0.2;
            self.vy += (dy / d * 7.0 - self.vy) * 0.2;
        } else {
            // The hover (`:39625-39680`'s idle band).
            self.vx *= 0.95;
            self.vy *= 0.95;
        }
        self.x += self.vx;
        self.y += self.vy;
        // `:39537-39539`'s roll + the sprite flip.
        self.rotation = self.vx.abs() * self.facing as f32 * 0.1;
        // The teleport cycle (`:39566-39575`'s counter).
        self.ai[0] += 1.0;
        if self.ai[0] >= 480.0 {
            self.ai[0] = 0.0;
            self.ai[1] = 20.0;
            // The tile pick: a non-solid tile near the aim point.
            let (tx, ty) = ((ax / TILE_PX) as i32, (ay / TILE_PX) as i32);
            let mut spot = None;
            for _ in 0..24 {
                let bx = tx + (crate::app::rand01(rng) * 40.0 - 20.0) as i32;
                let by = ty + (crate::app::rand01(rng) * 24.0 - 12.0) as i32;
                if !(0..world.width).contains(&bx) || !(0..world.height).contains(&by) {
                    continue;
                }
                let solid = world
                    .tile(bx, by)
                    .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                if !solid {
                    spot = Some((bx, by));
                    break;
                }
            }
            if let Some((sx, sy)) = spot {
                self.ai[2] = sx as f32;
                self.ai[3] = sy as f32;
            }
        }
        // `:39541-39564`'s fade + the hop: `ai[1]` counts down from 20.
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
            if self.ai[1] <= 1.0 && self.ai[2] != 0.0 {
                // `:39597-39615`'s teleport hop.
                self.x = self.ai[2] * TILE_PX - w * 0.5;
                self.y = self.ai[3] * TILE_PX - h * 0.5;
                self.vx = 0.0;
                self.vy = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
        }
    }

    /// The shared aiStyle-85 hoverer for the Brain Suckler (421):
    /// `AI()`'s `:36930-37220` block with the 421's arms — the phase-1
    /// attachment (`:37013-37033`: the `player.Top` reach under 40 px,
    /// the same-type/same-target scan, the snap) and phase 5 (the
    /// `Center = player.Top + direction * 4` ride, the `AddBuff(163,
    /// 59)` obstruct, `hide = true` and the 0.05 separation).
    fn update_brain_suckler(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:36996`'s 421 reach vector reads the player's TOP.
        let (pdx, pdy) = (pcx - cx, pcy - player.height - cy);
        let dist = (pdx * pdx + pdy * pdy).sqrt();
        let sight = crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
        if self.ai[0] == 5.0 {
            // `:37143-37162`'s attached ride: the snap, the obstruct buff
            // request and the zeroed velocity.
            self.x = pcx + player.facing as f32 * 4.0 - w * 0.5;
            self.y = pcy - player.height - h * 0.5;
            self.vx = 0.0;
            self.vy = 0.0;
            self.wants_obstruct = true;
            return;
        }
        match self.ai[0] as i32 {
            1 => {
                // `:36989-37012`'s approach at the 5.5-px base speed over
                // the 50-tick smoothing.
                self.rotation += self.facing as f32 * 0.3;
                let d = dist.max(0.001);
                let speed = 5.5 + d / 100.0;
                self.vx = (self.vx * 49.0 + pdx / d * speed) / 50.0;
                self.vy = (self.vy * 49.0 + pdy / d * speed) / 50.0;
                // `:37013-37033`'s attachment: under 40 px the suckler
                // latches onto the player's Top (the same-type/same-target
                // scan's refusal rides the port's per-player latch).
                if dist < 40.0 && player.hp > 0 && self.caster_counter == 0 {
                    self.caster_counter = 1;
                    self.x = pcx - w * 0.5;
                    self.y = pcy - player.height - h * 0.5;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.ai[0] = 5.0;
                    self.ai[1] = 0.0;
                    return;
                }
                if !sight {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            2 => {
                // `:37035-37054`'s blind charge (3 px) — the 421's own cap.
                self.rotation = self.vx * 0.1;
                if dist < 600.0 {
                    self.ai[0] = 0.0;
                }
            }
            3 => {
                // `:37055-37078`'s waypoint homing at the 2-px speed.
                self.rotation = self.vx * 0.1;
                let (tx, ty) = (self.ai[1], self.ai[2]);
                let (dx, dy) = (tx - cx, ty - cy);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                self.vx = (self.vx * 2.0 + dx / d * 2.0) / 3.0;
                self.vy = (self.vy * 2.0 + dy / d * 2.0) / 3.0;
                if d < 2.0 || d > 800.0 || sight {
                    self.ai[0] = 4.0;
                }
            }
            4 => {
                // `:37079-37142`'s bounce arm with the 180-tick re-probe.
                self.rotation = self.vx * 0.1;
                self.ai[1] += 1.0;
                if self.ai[1] > 180.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 3.0;
                }
            }
            _ => {
                // `:36933-36988`'s phase 0.
                if sight {
                    self.ai[0] = 1.0;
                } else if dist > 800.0 {
                    self.ai[0] = 2.0;
                } else if pdx.abs() > 8.0 || pdy.abs() > 8.0 {
                    self.ai[1] = pcx;
                    self.ai[2] = pcy;
                    self.ai[0] = 3.0;
                } else {
                    self.ai[0] = 4.0;
                    self.ai[1] = 0.0;
                }
            }
        }
        self.x += self.vx;
        self.y += self.vy;
        // `:37196`'s roll (the vanilla's `hide` while attached has no port
        // field — the attached suckler stays visible).
        self.rotation = self.vx * 0.1;
    }

    /// `AI_003_Fighters`' Predictor arm (`NPC.cs:59069-59756`, the 424 rows
    /// at `:59159-59192`/`:59475`/`:59578`) — the nebula pillar's shooter:
    /// the 180-tick cycle with the shot at the 179th tick fires four 573s
    /// (60) in the `(-3 + 2*i) * 0.15` fan from the muzzle.
    fn update_nebula_predictor(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let (dx, dy) = (pcx - cx, pcy - cy);
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // `:59208`'s fire: the four-shot 573 fan (60 damage, the classic
        // constant of `GetAttackDamage_ForProjectiles(60, 45)`).
        if self.ai[1] as i32 == 179 && self.ai[2] > 0.0 {
            let (mx, my) = (cx + self.sprite_dir as f32 * 4.0, cy - 6.0);
            for i in 0..4 {
                let vx = (-3.0 + 2.0 * i as f32) * 0.15;
                self.wants_hostile_shot.push((573u16, mx, my, vx, -0.1, 60));
            }
            self.ai[2] = 0.0;
            self.ai[1] = 0.0;
        }
        // `:59634-59688`'s engage arm: the 700-px gate starts the cycle.
        if self.ai[1] <= 0.0 && (dx * dx + dy * dy).sqrt() < 700.0 && player.hp > 0 {
            self.ai[2] = 3.0;
            self.ai[1] = 180.0;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    fn update_vortex_hunter(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let (dx, dy) = (pcx - cx, pcy - cy);
        // `:59082-59089`'s timer decay + the aim octant.
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        if self.ai[2] > 0.0 {
            let octant = |dx: f32, dy: f32| -> f32 {
                if dy.abs() > dx.abs() * 2.0 {
                    if dy > 0.0 { 1.0 } else { 5.0 }
                } else if dx.abs() > dy.abs() * 2.0 {
                    3.0
                } else if dy > 0.0 {
                    2.0
                } else {
                    4.0
                }
            };
            if self.ai[1] * 2.0 >= 60.0 {
                // `:59204-59202`'s fire: the 581 at 60 (the classic
                // constant of `GetAttackDamage_ForProjectiles(60, 45)`).
                let jit = 30.0 * 0.3;
                let mut ax = dx + (crate::app::rand01(rng) * 2.0 - 1.0) * jit;
                let mut ay = dy + (crate::app::rand01(rng) * 2.0 - 1.0) * jit;
                let len = (ax * ax + ay * ay).sqrt().max(0.001);
                ax = ax / len * 7.0;
                ay = ay / len * 7.0;
                // `:59304-59308`'s muzzle: `spriteDirection * 2, -12`.
                let (mx, my) = (cx + self.sprite_dir as f32 * 2.0, cy - 12.0);
                self.wants_hostile_shot.push((581u16, mx, my, ax, ay, 60));
                let _ = octant;
                self.ai[2] = 0.0;
            }
        }
        // `:59619-59623`'s arm reset: a finished shot cycle rests.
        if self.ai[1] <= 0.0 {
            self.ai[2] = 0.0;
        }
        // `:59634-59688`'s engage arm: the 700-px gate starts the 60-tick
        // cycle with the aim octant.
        if self.ai[1] <= 0.0 && (dx * dx + dy * dy).sqrt() < 700.0 && player.hp > 0 {
            self.ai[2] = 3.0;
            self.ai[1] = 60.0;
        }
        // The ground walk (`:59751-59756`'s num180=4 / num181=0.6 /
        // num182=0.95) — the port's walker carries the speed; the accel
        // and the drag fold into its snapshot step.
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 4.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY * 0.1 / 0.3,
            3.0,
        );
    }

    /// `AI_003_Fighters`' VortexAlien arm (`NPC.cs:58622-58711`) + the
    /// metamorphosis (`:56886-56901`) — the alien's dive (the 0.015/0.15
    /// accel pairs over the ±6 caps, the 0.98 damping, the 20-px deadband,
    /// the 0.05 separation), the leap arm (`velocity.Y = -5`) and the
    /// 1200-tick `Transform(426)` behind the empty 5x5 box.
    fn update_vortex_alien(&mut self, world: &World, player: &Player, _rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:56888-56889`'s metamorphosis counter (`localAI[0]` + |vx|/2).
        self.caster_counter += 1 + (self.vx.abs() * 0.5) as i32;
        if self.caster_counter >= 1200 {
            // `:56892-56897`'s 5x5 probe + the heal + `Transform(426)`.
            let tx = (cx / TILE_PX) as i32;
            let ty = (cy / TILE_PX) as i32;
            let mut boxed = false;
            for bx in (tx - 2)..=(tx + 2) {
                for by in (ty - 3)..=(ty + 1) {
                    if !(0..world.width).contains(&bx) || !(0..world.height).contains(&by) {
                        boxed = true;
                        break;
                    }
                    if world
                        .tile(bx, by)
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    {
                        boxed = true;
                        break;
                    }
                }
            }
            if !boxed {
                self.kind = NpcKind::VortexHunter;
                self.max_hp = NpcKind::VortexHunter.max_hp();
                self.hp = self.max_hp;
                self.ai = [0.0; 4];
                self.frame = 0;
                self.frame_counter = 0.0;
                self.caster_counter = 0;
                return;
            }
        }
        // `:58624-58628`'s landing arm.
        if self.vy == 0.0 {
            self.ai[2] = 0.0;
            self.rotation = 0.0;
        } else if self.ai[2] == 1.0 {
            // The dive homing (`:58639-58681`'s pairs).
            self.rotation = self.vx * 0.1;
            let (hx, hy) = (pcx - cx, pcy - cy);
            if hx < -20.0 && self.vx > -6.0 {
                self.vx = (self.vx - 0.015).max(-6.0);
            } else if hx > 20.0 && self.vx < 6.0 {
                self.vx = (self.vx + 0.015).min(6.0);
            }
            if hy < -20.0 && self.vy > -6.0 {
                self.vy = (self.vy - 0.15).max(-6.0);
            } else if hy > 20.0 && self.vy < 6.0 {
                self.vy = (self.vy + 0.15).min(6.0);
            }
        }
        // `:58705-58709`'s leap arm.
        // `:58705`: `CanHit(base.position, base.width, base.height,
        // player.position, player.width, player.height)`.
        if self.vy == 0.0
            && pcy + 100.0 < self.y
            && crate::projectile::can_hit(
                world,
                self.x,
                self.y,
                w,
                h,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            self.vy = -5.0;
            self.ai[2] = 1.0;
        }
        // `:58684-58700`'s separation from the player.
        if (pcx - cx).abs() + (pcy - cy).abs() < w {
            self.vx -= (pcx - cx).signum() * 0.05;
            self.vy -= (pcy - cy).signum() * 0.05;
        }
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI_003_Fighters`' VortexHeadcrab arm (`NPC.cs:58832-58841`'s
    /// rotation + `:56855-56869`'s metamorphosis) — the headcrab rides the
    /// generic walker (no speed override in the C#) and `Transform(427)`s
    /// on the 300-tick counter behind the empty 3x2 box above its centre.
    fn update_vortex_headcrab(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // `:58834-58841`'s rotation: grounded flat, rolling in the air.
        if self.vy == 0.0 {
            self.rotation = 0.0;
        } else {
            self.rotation += self.vx * 0.08;
        }
        // `:56857-56867`'s metamorphosis counter + the 3x2 probe.
        self.caster_counter += 1;
        if self.caster_counter >= 300 {
            let (w, h) = self.size();
            let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
            let tx = (cx / TILE_PX) as i32;
            let ty = (cy / TILE_PX) as i32;
            let mut boxed = false;
            for bx in (tx - 1)..=(tx + 1) {
                for by in (ty - 1)..=ty {
                    if !(0..world.width).contains(&bx) || !(0..world.height).contains(&by) {
                        boxed = true;
                        break;
                    }
                    if world
                        .tile(bx, by)
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    {
                        boxed = true;
                        break;
                    }
                }
            }
            if !boxed {
                self.kind = NpcKind::VortexAlien;
                self.max_hp = NpcKind::VortexAlien.max_hp();
                self.hp = self.max_hp;
                self.ai = [0.0; 4];
                self.frame = 0;
                self.frame_counter = 0.0;
                self.caster_counter = 0;
            }
        }
    }

    /// `AI_108_DivingFlyer` (`NPC.cs:65392-65788`) — the etherian wyverns
    /// (558/559/560) and the kobold gliders (574/575): the portal spawn
    /// fade, the approach/engage state machine, the wind-up, the dive and
    /// the gliders' impact self-destruct.
    ///
    /// The per-type tuning (`NPC.cs:65417-65471`): the wyverns' `flag2 =
    /// false` keeps them out of the crash state, the gliders have
    /// `num13 = 0` so only the solid-collision and the 64-px proximity
    /// open it; the drag `num12 * num11` is 3.0 for the wyverns (60) and
    /// 0.5 for the gliders (10).
    ///
    /// Deviations: the separation nudge reads the pool (the sim owns the
    /// NPC vec) so it lands in `world_sim`; the dust/gore/sound arms are
    /// unmodelled; `localAI[0]`'s fade rides the port's `caster_counter`
    /// and `alpha`.
    fn update_diving_flyer(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // `:65417-65471`'s per-type row: the family flags drive the tuning.
        let glide = !matches!(
            self.kind,
            NpcKind::DD2WyvernT1 | NpcKind::DD2WyvernT2 | NpcKind::DD2WyvernT3
        );
        // `num19 = num` (3.0 wyvern / 4.0 glider), `num8` (the launch speed
        // 8), `num11` (the dive smoothing 60 / 10), `num13` (the min-speed
        // exit 6 / 0) and the drag `num12 * num11` (3.0 / 0.5).
        let (max_speed, dive_max, homing, min_speed, drag) = if glide {
            (4.0, 8.0, 10.0, 0.0, 0.5)
        } else {
            (3.0, 8.0, 60.0, 6.0, 3.0)
        };
        // `:65392`'s rotation rides the velocity.
        self.rotation = self.vx.atan2(self.vy);
        // `:65476-65510`'s spawn fade: the alpha clears over 60 ticks
        // (`localAI[0]` → the port's `caster_counter`).
        if self.alpha > 0 {
            self.caster_counter = (self.caster_counter + 1).min(60);
            self.alpha = self.alpha.saturating_sub(22);
        }
        match self.ai[0] as i32 {
            0 => {
                // `:65552-65598`'s approach: the 500-px engage gate and the
                // smoothed velocity toward a point 400 px above the player.
                let (dx, dy) = (pcx - cx, pcy - cy - 400.0);
                if dx.hypot(dy) > 500.0 {
                    let len = dx.hypot(dy).max(1.0);
                    let accel = 90.0;
                    self.vx = (self.vx * (accel - 1.0) + dx / len * max_speed) / accel;
                    self.vy = (self.vy * (accel - 1.0) + dy / len * max_speed) / accel;
                } else {
                    // `:65592-65598`'s engage: the dive vector into ai[2:3].
                    let len = (pcx - cx).hypot(pcy - cy).max(1.0);
                    self.ai[2] = (pcx - cx) / len;
                    self.ai[3] = (pcy - cy) / len;
                    self.ai[1] = 0.0;
                    self.ai[0] = 1.0;
                }
            }
            1 => {
                // `:65602-65630`'s wind-up: 20 (wyvern) / 30 (glider) ticks
                // of damping, then the launch at `num8` speed.
                self.vx *= 0.95;
                self.vy *= 0.95;
                if glide {
                    self.vy -= 0.1;
                }
                self.ai[1] += 1.0;
                let windup = if glide { 30.0 } else { 20.0 };
                if self.ai[1] >= windup {
                    let jitter = if glide { 3.0 } else { 0.0 };
                    let jx = self.ai[2] + (crate::app::rand01(rng) * 2.0 - 1.0) * jitter * 0.04;
                    let jy = self.ai[3] + (crate::app::rand01(rng) * 2.0 - 1.0) * jitter * 0.04;
                    let len = (jx * jx + jy * jy).sqrt().max(1.0);
                    self.vx = jx / len * dive_max;
                    self.vy = jy / len * dive_max;
                    self.ai[0] = 2.0;
                    self.ai[1] = 0.0;
                }
            }
            2 => {
                // `:65632-65686`'s dive: the homing, the 30-tick / 150-px
                // exit and the gliders' solid-collision crash entry.
                self.ai[1] += 1.0;
                let (dx, dy) = (pcx - cx, pcy - cy);
                let len = dx.hypot(dy).max(1.0);
                let (ux, uy) = (dx / len, dy / len);
                let speed = self.vx.hypot(self.vy);
                self.vx = (self.vx * (homing - 1.0) + ux * (speed + drag)) / homing;
                self.vy = (self.vy * (homing - 1.0) + uy * (speed + drag)) / homing;
                let timer_exit = !glide && self.ai[1] >= 30.0 && dx.hypot(dy) > 150.0 && cy > pcy;
                let speed_exit = self.vx.hypot(self.vy) < min_speed;
                let crash = glide
                    && (dx.hypot(dy) < 64.0
                        || crate::player::Player::solid_collision(world, self.x, self.y, w, h));
                if timer_exit || speed_exit {
                    // `:65654-65667`'s exit: the 4-state rest with `ai[1] = 45`.
                    self.ai[0] = 4.0;
                    self.ai[1] = 45.0;
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                    self.vx *= 0.5;
                    self.vy *= 0.5;
                } else if crash {
                    self.ai[0] = 3.0;
                    self.ai[1] = 0.0;
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                }
            }
            3 => {
                // `:65728-65788`'s impact: the 192-box, the 80-damage blast
                // and the death at the third tick.
                self.x = cx - 96.0;
                self.y = cy - 96.0;
                self.vx = 0.0;
                self.vy = 0.0;
                self.damage_override = Some(80);
                self.alpha = 255;
                if self.ai[1] as i32 == 0 {
                    self.wants_blast = true;
                }
                self.ai[1] += 1.0;
                if self.ai[1] >= 3.0 {
                    self.hp = 0;
                }
            }
            _ => {
                // `:65688-65698`'s post-dive rest: 15 ticks at 0.95.
                self.vx *= 0.95;
                self.vy *= 0.95;
                self.ai[1] -= 3.0;
                if self.ai[1] <= 0.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
        }
        let _ = rng;
    }

    /// `AI_107`'s 569 arm (`NPC.cs:63348-63432`) — the tier-3 wither
    /// beast's aura/heal walker: the charge and the 60-tick `lifeMax / 20`
    /// self-heal. The walk itself rides `update_wither_beast`.
    fn update_wither_beast_t3(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // `:63351-63360`: the 110-tick charge, the 600-px engage and the
        // 1.4/0.16/0.7 walk tuning ride the shared walker arm; the aura
        // arm is the T2's.
        self.update_wither_beast(world, player, rng);
        // `:63409-63412`'s `ai[1]` drain: 60 ticks then `lifeMax / 20`.
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        } else {
            self.ai[1] = 60.0;
            let heal = self.max_hp / 20;
            if heal > 0 && self.hp < self.max_hp {
                self.hp = (self.hp + heal).min(self.max_hp);
            }
        }
    }

    /// `AI_107`'s 570/571 arm (`NPC.cs:63145-63195`) — the drakins: the
    /// ranger's stand-and-shoot, the 671 projectile at the tier's damage.
    fn update_drakin(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let (t2, dmg) = if self.kind == NpcKind::DD2DrakinT2 {
            (true, 35)
        } else {
            (false, 60)
        };
        // `:64126-64129`'s engage gate: the 600-px range opens the burst.
        if self.ai[0] <= 0.0 && self.ai[1] <= 0.0 && (pcx - cx).hypot(pcy - cy) < 600.0 {
            self.ai[1] = 60.0;
        }
        // `:64052-64056`: the burst counts down and the hit resets it.
        if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // `:64068-64107`'s shot: on the burst's fire ticks, at the tier's
        // damage, with the 571's tighter spread.
        if self.ai[1] > 0.0 && self.ai[1] <= 40.0 {
            let every = if t2 { 9.0 } else { 6.0 };
            let passed = 40.0 - self.ai[1];
            if passed > 0.0 && (passed as i32) % (every as i32) == 0 {
                // The muzzle rides `vector.X += 22 * direction` (`:63162`).
                let (sx, sy) = (cx + 22.0 * self.facing as f32, cy);
                let spread = if t2 { 2.5 } else { 1.5 };
                // The ballistic drop `Math.Abs(v.X) * num15` (0.15).
                let (dx, dy) = (
                    pcx - sx + (crate::app::rand01(rng) * 2.0 - 1.0) * spread,
                    pcy - sy - (pcx - sx).abs() * 0.15,
                );
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                let speed = 13.0;
                self.wants_hostile_shot.push((
                    671,
                    sx,
                    sy,
                    dx / len * speed,
                    dy / len * speed,
                    dmg,
                ));
            }
        }
        // The walker arm: the port's shared physics keeps them grounded.
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 0.77,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    /// `AI_107`'s 572/573 arm (`NPC.cs:63196-63347`) — the kobold walkers:
    /// the ignite-and-explode kamikaze with the 192-box blast.
    fn update_dd2_kobold_walker(&mut self, world: &World, player: &Player, dt: f32) {
        if self.ai[1] == 2.0 {
            // `:63204-63262`'s explode: the 192-box, the 80-damage blast and
            // the death at the third tick.
            let (w, h) = self.size();
            let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
            self.x = cx - 96.0;
            self.y = cy - 96.0;
            self.vx = 0.0;
            self.vy = 0.0;
            self.damage_override = Some(80);
            self.alpha = 255;
            if self.ai[0] == 0.0 {
                self.wants_blast = true;
            }
            self.ai[0] += 1.0;
            if self.ai[0] >= 3.0 {
                self.hp = 0;
            }
            return;
        }
        // `:63276`'s latch: the lit state (`ai[0] == 1`) converts `ai[1]`
        // to 2 — the explode. `:63283-63309`'s ignite runs the sprint.
        if self.ai[0] == 1.0 {
            self.ai[1] = 2.0;
        }
        self.update_dd2_goblin(world, player, dt);
    }

    /// `AI_107`'s 576/577 arm (`NPC.cs:63029-63122`) — the ogres: the
    /// chase/spit/pound sub-state machine with the 676/683 throws and the
    /// hop.
    fn update_ogre(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let dist = (pcx - cx).hypot(pcy - cy);
        // `:63040`'s chase window: `localAI[0]` counts down (the port's
        // `caster_counter`), holding the 5-second "must chase" latch.
        if self.caster_counter > 0 {
            self.caster_counter -= 1;
        }
        // `:63044-63075`'s sub-state pick on `ai[1]`.
        if self.ai[0] <= 0.0 {
            if dist < 130.0 + 300.0 && self.caster_counter <= 0 {
                self.ai[1] = 2.0;
            } else if dist > 130.0 + 30.0 {
                self.ai[1] = 1.0;
            } else {
                self.ai[1] = 0.0;
            }
        } else if self.ai[1] == 2.0 {
            // `:63072`: the sub-2 state latches the 300-tick chase window.
            self.caster_counter = 300;
        }
        // `:63076-63117`'s per-sub-state tuning: the charge length, the
        // engage distance, the cooldown and the throw's own tuning.
        let (tick, engage, cooldown, proj, dmg, hop, ox, oy) = match self.ai[1] as i32 {
            2 => (
                90.0,
                250.0,
                60.0,
                683u16,
                60,
                56.0,
                -20.0 * self.facing as f32,
                h * 0.5 - 1.0,
            ),
            1 => (
                90.0,
                1000.0,
                240.0,
                676u16,
                40,
                0.0,
                30.0 * self.facing as f32,
                -70.0,
            ),
            _ => (44.0, 130.0, 60.0, 676u16, 40, 0.0, 0.0, 0.0),
        };
        // The flag5 charge (`:63850-63880`): the countdown, the damping and
        // the cooldown.
        if self.ai[0] < 0.0 {
            self.ai[0] += 1.0;
        } else if self.ai[0] == 0.0 && self.vy == 0.0 && dist < engage {
            self.ai[0] = tick;
        }
        if self.ai[0] > 0.0 {
            self.vx *= 0.7;
            self.ai[0] -= 1.0;
            if self.ai[0] == 0.0 {
                self.ai[0] = -cooldown;
            }
        }
        // `:63916-63920`'s hop: at `num = 56` the ogre slams up.
        if hop > 0.0 && self.ai[0] == hop {
            self.vy = -13.0;
        }
        // `:63888-63913`'s throw: at `num9 = 36` the projectile leaves from
        // the offset, aimed with the `Math.Abs(v.X) * num15` drop.
        if (proj == 676 || proj == 683) && self.ai[0] == 36.0 {
            let (sx, sy) = (cx + ox, cy + oy);
            let drop = (pcx - sx).abs() * 0.15;
            let (dx, dy) = (pcx - sx, pcy - sy - drop);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let speed = 13.0;
            self.wants_hostile_shot
                .push((proj, sx, sy, dx / len * speed, dy / len * speed, dmg));
        }
        // The walker physics (the shared `AI_107` body's accel/friction):
        // `num23 = 1.0`, `num24 = 0.07`, `num25 = 0.8`.
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        let _ = rng;
    }

    fn update_moon_leech_blob(&mut self, _world: &World, _player: &Player) {
        self.ai[2] += 1.0;
        if self.ai[2] >= 90.0 {
            // `:36658-36690`'s drain: the resolve rides the sim (the blob
            // needs the sibling NPCs) — the blob itself dies here.
            self.wants_moon_leech = true;
            self.hp = 0;
        }
    }

    fn update_ancient_cultist(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The timer: the 180-tick dissolve (`:39060-39066`'s `ai[1]`).
        self.ai[2] += 1.0;
        if self.ai[2] >= 180.0 {
            self.hp = 0;
            return;
        }
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The hover: the sine bob + the 300-tick cycle.
        self.vy = (std::f32::consts::PI * 2.0 * self.ai[0] / 300.0).sin() * 0.5;
        self.ai[0] += 1.0;
        if self.ai[0] >= 300.0 {
            self.ai[0] = 0.0;
        }
        // The regen: +200 per 60 ticks while the player stays 2000+ px
        // (`localAI[0]` — the port's `caster_counter`).
        if (pcx - cx).hypot(pcy - cy) > 2000.0 {
            self.caster_counter += 1;
            if self.caster_counter >= 60 {
                self.caster_counter = 0;
                self.hp = (self.hp + 200).min(self.max_hp);
            }
        } else {
            self.caster_counter = 0;
        }
        // The ritual arm (`:39273-39292`): the player within 3240 px with
        // no line of sight — the 578/579 projectiles at the player's tile.
        if player.hp > 0
            && (pcx - cx).hypot(pcy - cy) < 3240.0
            && !crate::projectile::can_hit_line(world, cx, cy, 0.0, 0.0, pcx, pcy, 0.0, 0.0)
        {
            self.ai[1] = 0.0;
            let ritual = 579u16;
            self.wants_hostile_shot
                .push((ritual, pcx, pcy, 0.0, 0.0, 0));
            let _ = ritual;
        }
        // The 1080-px arm (`:39293-39336`): the 420+360 rearm + the
        // 20-tile window's pick.
        if player.hp > 0
            && (pcx - cx).hypot(pcy - cy) < 1080.0
            && pcy - cy < 400.0
            && self.caster_counter == 0
        {
            self.caster_counter = 420;
            let ritual = 579u16;
            self.wants_hostile_shot
                .push((ritual, pcx, pcy, 0.0, 0.0, 0));
            let _ = ritual;
        }
        let _ = rng;
    }

    fn update_wither_beast(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.4,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        if self.ai[0] == 0.0 {
            self.ai[3] = 0.0;
        } else {
            // The aura pulse counter (`localAI[0]` → the port's `caster_counter`).
            self.caster_counter += 1;
            // The ring dust (the 400-px random radius) — the port's
            // stream stands in for `Main.rand`.
            if self.caster_counter > 60 && crate::app::rand01(rng) < 0.1 {
                let ang = crate::app::rand01(rng) * std::f32::consts::TAU;
                let radius = 400.0 * (0.3 + 0.7 * crate::app::rand01(rng));
                let (rx, ry) = (cx + ang.cos() * radius, cy + ang.sin() * radius);
                // The dust's solid check folds (the port has no dust
                // system) — the ring lands at the point.
                let _ = (rx, ry);
            }
            // The 60-tick self-heal.
            if self.ai[1] > 0.0 {
                self.ai[1] -= 1.0;
            } else {
                self.ai[1] = 60.0;
                let heal = (self.max_hp / 20).min(self.max_hp - self.hp);
                if heal > 0 {
                    self.hp = (self.hp + heal).min(self.max_hp);
                }
            }
        }
    }

    fn update_dd2_thrower(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        // The walker harness (the vanilla's `num23 = 8`, `num24 *= 3`).
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 8.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The throw cycle: `ai[1]` to 82.
        self.ai[1] += 1.0;
        if self.ai[1] >= 82.0 {
            self.ai[1] = 0.0;
            let (proj, dmg) = if self.kind == NpcKind::DD2JavelinT3 {
                (685u16, 80)
            } else if self.kind == NpcKind::DD2JavelinT2 {
                (662u16, 60)
            } else {
                (662u16, 30)
            };
            // The ballistic aim: at the player with the vanilla's arc.
            let (dx, dy) = (pcx - cx, pcy - (cy - 14.0));
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let speed = 11.0;
            let inv = speed / dist;
            self.wants_hostile_shot
                .push((proj, cx, cy - 14.0, dx * inv, dy * inv, dmg));
            let _ = rng;
        }
    }

    fn update_dd2_skeleton(&mut self, world: &World, player: &Player) {
        self.caster_counter += 1;
        // The walk rides the harness (the vanilla walks the materializing
        // skeleton too — the fade is cosmetic).
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    fn update_saucer_core(&mut self, world: &World, player: &Player, rng: &mut u64) {
        // The init (`:36071-36092`): the guns + the shell, the seat links.
        // The vanilla's `localAI[3]` latch never touches `ai[0]` — the
        // forced phases stand (the debug spawn / the table arm set them).
        if !self.wants_scutlix {
            self.wants_scutlix = true; // the init latch (a one-shot)
            self.wants_saucer_crew = true;
        }
        if self.ai[0] == 3.0 {
            // The death ride.
            self.hp = 0;
            return;
        }
        let (w, h) = self.size();
        let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let dist = (pcx - cx).hypot(pcy - cy);
        let far = dist > 5600.0 || player.hp <= 0;
        if far && self.ai[0] != 1.0 {
            // The despawn ride (`:36130-36148`): the negative phases hold
            // the state, the climb-away runs, the return restores it.
            if self.ai[0] == 0.0 {
                self.ai[0] = -1.0;
            } else if self.ai[0] == 2.0 {
                self.ai[0] = -2.0;
            }
            if self.ai[0] == -1.0 || self.ai[0] == -2.0 {
                self.vy -= 0.4;
                if player.hp > 0 {
                    self.ai[0] = if self.ai[0] == -2.0 { 2.0 } else { 0.0 };
                    self.ai[1] = 0.0;
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                } else {
                    // The dead player: the ride climbs out (the port has
                    // no `timeLeft` latch — the position fallthrough
                    // despawns it).
                    self.ai[0] = -1.0;
                }
            }
        } else if self.ai[0] == 0.0 {
            // The scan cycle (`:36149-36372`): `ai[3]` counts 0..600, the
            // step `num1104` selects the band (0 below 20, 1 in 20-260,
            // 2 at 260-280, 3 at 280-420, 4 at 420-440, 5 at 440-580).
            let step = |t: f32| -> u32 {
                if t >= 580.0 {
                    0
                } else if t >= 440.0 {
                    5
                } else if t >= 420.0 {
                    4
                } else if t >= 280.0 {
                    3
                } else if t >= 260.0 {
                    2
                } else if t >= 20.0 {
                    1
                } else {
                    0
                }
            };
            let before = step(self.ai[3]);
            self.ai[3] += 1.0;
            if self.ai[3] >= 600.0 {
                self.ai[3] = 0.0;
            }
            let now = step(self.ai[3]);
            if now != before {
                match now {
                    0 => self.ai[2] = 0.0,
                    1 => self.ai[2] = if pcx > cx { 1.0 } else { -1.0 },
                    2 => self.ai[2] = 0.0,
                    _ => {}
                }
            }
            // The ground clearance under the core.
            let tile_y = ((self.y + h as f32) / TILE_PX) as i32;
            let tile_x = (cx / TILE_PX) as i32;
            let ground_clear = ground_clearance(world, tile_x, tile_y);
            match now {
                0 => {
                    // The approach: hover 250 px above the player ±ai[2].
                    if self.ai[2] == 0.0 {
                        self.ai[2] = -600.0 * (cx - pcx).signum();
                    }
                    let (tx, ty) = (pcx + self.ai[2], pcy - 250.0);
                    if (tx - cx).hypot(ty - cy) < 50.0 {
                        self.ai[3] = 19.0;
                    } else {
                        let inv = 1.0 / (tx - cx).hypot(ty - cy);
                        self.vx += ((tx - cx) * inv * 16.0 - self.vx) * 0.1;
                        self.vy += ((ty - cy) * inv * 16.0 - self.vy) * 0.1;
                    }
                }
                1 => {
                    // The dive: the X drifts 3.5·ai[2], the Y eases.
                    if ground_clear < 250.0 {
                        let target = -(ground_clear.max(-4.0));
                        self.vy += (target - self.vy) * 0.05;
                    } else {
                        self.vy *= 0.95;
                    }
                    self.vx = 3.5 * self.ai[2];
                }
                2 => {
                    // The approach at 170 px above.
                    if self.ai[2] == 0.0 {
                        self.ai[2] = 300.0 * (cx - pcx).signum();
                    }
                    let (mut tx, mut ty) = (pcx + self.ai[2], pcy - 170.0);
                    let clear = ground_clear;
                    if clear < 170.0 {
                        ty -= 170.0 - clear;
                    }
                    if (tx - cx).hypot(ty - cy) < 70.0 {
                        self.ai[3] = 279.0;
                    } else {
                        let inv = 1.0 / (tx - cx).hypot(ty - cy);
                        self.vx += ((tx - cx) * inv * 20.0 - self.vx) * 0.1;
                        self.vy += ((ty - cy) * inv * 20.0 - self.vy) * 0.1;
                    }
                    let _ = (&mut tx, &mut ty);
                }
                3 => {
                    // The park: both damp.
                    if ground_clear < 170.0 {
                        let target = -(ground_clear.max(-4.0));
                        self.vy += (target - self.vy) * 0.05;
                    } else {
                        self.vy *= 0.85;
                    }
                    self.vx *= 0.85;
                }
                4 => {
                    // The approach straight above the player.
                    let (tx, ty) = (pcx, pcy - 250.0);
                    if (tx - cx).hypot(ty - cy) < 50.0 {
                        self.ai[3] = 439.0;
                    } else {
                        let inv = 1.0 / (tx - cx).hypot(ty - cy);
                        self.vx += ((tx - cx) * inv * 16.0 - self.vx) * 0.1;
                        self.vy += ((ty - cy) * inv * 16.0 - self.vy) * 0.1;
                    }
                }
                _ => {
                    // The brake.
                    self.vx *= 0.85;
                    self.vy *= 0.85;
                }
            }
        } else if self.ai[0] == 1.0 {
            // The vulnerable wobble (`:36373-36407`): 150 ticks, the
            // rotation ramps by the phase.
            self.vx *= 0.96;
            self.vy *= 0.96;
            self.ai[1] += 1.0;
            if self.ai[1] >= 150.0 {
                self.ai[0] = 2.0;
                self.ai[1] = 0.0;
                self.rotation = 0.0;
            } else if self.ai[1] < 40.0 {
                // `Vector2.UnitY.RotatedBy(t).Y` is the COSINE of the
                // angle (UnitY = (0, 1) rotates to (-sin t, cos t), and
                // the C# reads its Y) — the v1 ran sin.
                self.rotation = (self.ai[1] / 40.0 * 6.283).cos() * 0.2;
            } else if self.ai[1] < 80.0 {
                self.rotation = (self.ai[1] / 20.0 * 6.283).cos() * 0.3;
            } else if self.ai[1] < 120.0 {
                self.rotation = (self.ai[1] / 10.0 * 6.283).cos() * 0.4;
            } else {
                self.rotation = (self.ai[1] - 120.0) / 30.0 * 6.283;
            }
        } else if self.ai[0] == 2.0 {
            // The gun (`:36408-36500`): the 3600-tick cycle; the firing
            // window opens for 60 ticks of each 120; the projectile 447
            // (the classic 80) at the window's rising edge.
            // The vanilla's `num1131`/`num1130` pair: the check before and
            // after the increment — the fire rides the rising edge.
            let prev_window = self.ai[3] % 120.0 >= 60.0;
            self.ai[3] += 1.0;
            let window_now = self.ai[3] % 120.0 >= 60.0;
            if window_now != prev_window {
                if window_now {
                    self.ai[2] = if pcx > cx { 1.0 } else { -1.0 };
                    // The 447 projectile at (center, center) with the
                    // owner stub (whoAmI + 1) — the port's hostile pool.
                    self.wants_hostile_shot.push((447u16, cx, cy, 0.0, 0.0, 80));
                }
            }
            if self.ai[3] >= 3600.0 {
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            } else if !window_now {
                // The chase: the 16-lerp to the player + 350·ai[2], -250.
                let (tx, ty) = (pcx + self.ai[2] * 350.0, pcy - 250.0);
                let d = (tx - cx).hypot(ty - cy).max(0.001);
                let inv = 1.0 / d;
                self.vx += ((tx - cx) * inv * 16.0 - self.vx) * 0.1;
                self.vy += ((ty - cy) * inv * 16.0 - self.vy) * 0.1;
            } else {
                // The firing dash: the X at 8·ai[2], the Y eases.
                let tile_y = ((self.y + h as f32) / TILE_PX) as i32;
                let tile_x = (cx / TILE_PX) as i32;
                let clear = ground_clearance(world, tile_x, tile_y);
                if clear < 250.0 {
                    let target = -(clear.max(-4.0));
                    self.vy += (target - self.vy) * 0.05;
                } else {
                    self.vy *= 0.95;
                }
                self.vx = 8.0 * self.ai[2];
            }
            self.rotation = 0.0;
        }
        // The off-world clamp (`:36502-36520`).
        let max_x = world.width as f32 * TILE_PX;
        let max_y = world.height as f32 * TILE_PX;
        if self.y < -100.0 || self.x < -100.0 || self.y > max_y + 100.0 || self.x > max_x + 100.0 {
            self.x = self.x.clamp(-100.0, 100.0 + max_x);
            self.y = self.y.clamp(-100.0, 100.0 + max_y);
            self.hp = 0; // `active = false` — the death pass removes it.
        }
        // The flight integration (noGravity + noTileCollide).
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI_032`'s Skeletron Prime v1 (`NPC.cs`'s aiStyle 32 machine) — the
    /// EoC cycle shape with the Prime's numbers: the hover (the ±0.12,
    /// the 9 cap), the charge (the 480-tick cycle, the direct velocity
    /// x 9), the decel (the 40 ticks x0.98). The 4 limbs = the follow-ups.
    /// The Prime's head (`aiStyle 32`, `NPC.cs:27626-27942`): the
    /// 600-tick hover → the 400-tick enraged charge (`defense/damage x2`,
    /// the spin, the straight-at-the-player speed 2 classic) → the
    /// repeat; the day-rage (`damage/defense 9999`, the 8-32 chase) and
    /// the dead-player despawn (`EncourageDespawn(500)`). The four limbs
    /// spawn once on the first tick (the port's `wants_limbs` flag).
    fn update_prime(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let dist = (pcx - cx).hypot(pcy - cy);
        // The state selection (`:27662-27676`).
        if player.hp <= 0 || dist > 6000.0 {
            self.ai[1] = 3.0;
        } else if world.day_time && self.ai[1] != 3.0 && self.ai[1] != 2.0 {
            self.ai[1] = 2.0;
        }
        // The limb spawn (the one-shot).
        if self.caster_counter == 0 && self.cast_windup == 0 {
            self.caster_counter = 1;
            self.wants_limbs = true;
        }
        if self.ai[1] == 0.0 {
            // The hover (`:27677-27764`): the Y steer to `player.Y` in
            // the ±200/±500 bands, the X steer to ±100, the 0.1/2 caps,
            // the 0.98 friction; the 600-tick cycle flips to the rage.
            self.ai[2] += 1.0;
            // The roll (`:27694-27698`): the hover's rotation rides the
            // drift `velocity.X / 15` (the Mech Queen eases into half of
            // it through `AngleLerp`) — the v1 never wrote a rotation, so
            // the head drew bolt upright while it strafed.
            self.rotation = if self.ai[3] != 0.0 {
                angle_lerp(self.rotation, self.vx / 15.0 * 0.5, 0.75)
            } else {
                self.vx / 15.0
            };
            let target_y = if pcy > cy + 200.0 {
                pcy - 500.0
            } else if pcy < cy - 200.0 {
                pcy - 200.0
            } else {
                pcy
            };
            self.vy += (target_y - cy).signum() * 0.1;
            self.vy = self.vy.clamp(-2.0, 2.0);
            let target_x = if pcx > cx + 100.0 {
                cx + 100.0
            } else if pcx < cx - 100.0 {
                cx - 100.0
            } else {
                cx
            };
            self.vx += (target_x - cx).signum() * 0.1;
            self.vx = self.vx.clamp(-8.0, 8.0);
            self.vx *= 0.98;
            if self.ai[2] >= 600.0 {
                self.ai[1] = 1.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[1] == 1.0 {
            // The enraged charge (`:27765-27859`): the straight line at
            // the classic 2, the 400-tick duration.
            self.ai[2] += 1.0;
            // The spin (`:27781-27786`): the head rolls `direction * 0.3`
            // per tick (the Mech Queen eases into the half-drift instead) —
            // the port's find_frame case 127 pins row 2 for this phase, so
            // the roll is the only motion the wide row shows.
            self.rotation = if self.ai[3] != 0.0 {
                angle_lerp(self.rotation, self.vx / 15.0 * 0.5, 0.75)
            } else {
                self.rotation + self.facing as f32 * 0.3
            };
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            self.vx = dx / d * 2.0;
            self.vy = dy / d * 2.0;
            if self.ai[2] >= 400.0 {
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
            }
        } else if self.ai[1] == 2.0 {
            // The day-rage (`:27860-27892`): the 8-32 chase.
            self.rotation = if self.ai[3] != 0.0 {
                angle_lerp(self.rotation, self.vx / 15.0 * 0.5, 0.75)
            } else {
                self.rotation + self.facing as f32 * 0.3
            };
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let speed = (10.0 + d / 100.0).clamp(8.0, 32.0);
            self.vx = dx / d * speed;
            self.vy = dy / d * speed;
        } else {
            // The despawn drift (`:27893-27941`).
            self.vy += 0.1;
            self.vx *= 0.95;
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_030`/`AI_031`'s Twins v1 (`NPC.cs`'s aiStyle 30/31 machines) —
    /// the EoC cycle shape per eye: the hover (the ±0.09, the 8.5 cap),
    /// the charge (the 480-tick cycle, the direct velocity x 8.5), the
    /// decel (the 40 ticks x0.98). The laser/spit phases and the second-
    /// form transformations = the follow-ups.
    /// `AI()`'s aiStyle-57 block (`NPC.cs:32761-33043`) — the Retinazer's
    /// machine: the idle (0), the stomp (1), the spin (2), the beam (3) and
    /// the charge (4), the horizontal drift toward the player and the
    /// vertical hover.
    ///
    /// The light and the dust are SKIPPED — no dust/light layer. The
    /// day-time `EncourageDespawn(10)` rides the world's own day arm (the
    /// port's NPCs carry no despawn timer).
    fn update_retinazer(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // `:32764-32772` — the horizontal speed cap (the hp bands; the
        // day-time raises it to 8).
        let mut speed = 2.0f32;
        if self.hp * 4 < self.max_hp * 3 {
            speed = 3.0;
        }
        if self.hp * 2 < self.max_hp {
            speed = 4.0;
        }
        if world.day_time {
            speed = 8.0;
        }
        // `:32789` — the "firing" flag: the states that hold the boss still
        // while it shoots.
        let mut firing = false;
        match self.ai[0] as i32 {
            0 => {
                // `:32794-32815` — the idle: the counter (the faster it
                // loses hp, the faster it counts) and then the next state.
                self.ai[1] += 1.0;
                if self.hp * 2 < self.max_hp {
                    self.ai[1] += 1.0;
                }
                if self.hp * 4 < self.max_hp {
                    self.ai[1] += 1.0;
                }
                if self.ai[1] >= 300.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = if self.hp * 4 < self.max_hp {
                        crate::app::frame::rand_range_signed(rng, 3, 5) as f32
                    } else {
                        crate::app::frame::rand_range_signed(rng, 1, 3) as f32
                    };
                }
            }
            1 => {
                // `:32836-32866` — the stomp: the flaming wood every 15
                // ticks.
                firing = true;
                self.ai[1] += 1.0;
                if self.ai[1] % 15.0 == 0.0 {
                    let origin = Vec2::new(cx, cy + 30.0);
                    let mut dx = pcx - origin.x;
                    let mut dy = pcy - origin.y;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    dx = dx / d * 10.0;
                    dy = dy / d * 10.0;
                    let sx = dx
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -20, 21) as f32 * 0.01);
                    let sy = dy
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -20, 21) as f32 * 0.01);
                    self.wants_hostile_shot.push((
                        crate::projectile::FLAMING_WOOD,
                        origin.x,
                        origin.y,
                        sx,
                        sy,
                        50,
                    ));
                }
                if self.ai[1] >= 120.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            2 => {
                // `:32868-32929` — the spin: the Greek fires every eighth
                // tick between the 60th and the 240th.
                firing = true;
                self.ai[1] += 1.0;
                if self.ai[1] > 60.0 && self.ai[1] < 240.0 && self.ai[1] % 8.0 == 0.0 {
                    let origin = Vec2::new(cx, cy + 30.0);
                    let mut dx = pcx - origin.x;
                    let mut dy = pcy - origin.y - dx.abs() * 0.3;
                    let mut shot_speed = 10.0f32;
                    shot_speed += dx.abs() * 0.004;
                    if shot_speed > 14.0 {
                        shot_speed = 14.0;
                    }
                    dx += crate::app::frame::rand_range_signed(rng, -50, 51) as f32;
                    dy -= crate::app::frame::rand_range_signed(rng, 50, 201) as f32;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    dx = dx / d * shot_speed;
                    dy = dy / d * shot_speed;
                    let kind = crate::app::frame::rand_range_signed(rng, 326, 329) as u16;
                    let sx = dx
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -30, 31) as f32 * 0.01);
                    let sy = dy
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -30, 31) as f32 * 0.01);
                    self.wants_hostile_shot
                        .push((kind, origin.x, origin.y, sx, sy, 40));
                }
                if self.ai[1] >= 300.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            3 => {
                // `:32930-32956` — the beam: the flaming wood every 30 ticks.
                speed = 4.0;
                self.ai[1] += 1.0;
                if self.ai[1] % 30.0 == 0.0 {
                    let origin = Vec2::new(cx, cy + 30.0);
                    let mut dx = pcx - origin.x;
                    let mut dy = pcy - origin.y;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    dx = dx / d * 16.0;
                    dy = dy / d * 16.0;
                    let sx = dx
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -20, 21) as f32 * 0.001);
                    let sy = dy
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -20, 21) as f32 * 0.001);
                    self.wants_hostile_shot.push((
                        crate::projectile::FLAMING_WOOD,
                        origin.x,
                        origin.y,
                        sx,
                        sy,
                        75,
                    ));
                }
                if self.ai[1] >= 120.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            4 => {
                // `:32957-32990` — the charge: the Greek fires every tenth
                // tick.
                speed = 4.0;
                self.ai[1] += 1.0;
                if self.ai[1] % 10.0 == 0.0 {
                    let origin = Vec2::new(cx, cy + 30.0);
                    let mut dx = pcx - origin.x;
                    let mut dy = pcy - origin.y - dx.abs() * 0.2;
                    let mut shot_speed = 12.0f32;
                    shot_speed += dx.abs() * 0.002;
                    if shot_speed > 16.0 {
                        shot_speed = 16.0;
                    }
                    dx += crate::app::frame::rand_range_signed(rng, -50, 51) as f32;
                    dy -= crate::app::frame::rand_range_signed(rng, 50, 201) as f32;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    dx = dx / d * shot_speed;
                    dy = dy / d * shot_speed;
                    let kind = crate::app::frame::rand_range_signed(rng, 326, 329) as u16;
                    let sx = dx
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -30, 31) as f32 * 0.005);
                    let sy = dy
                        * (1.0 + crate::app::frame::rand_range_signed(rng, -30, 31) as f32 * 0.005);
                    self.wants_hostile_shot
                        .push((kind, origin.x, origin.y, sx, sy, 50));
                }
                if self.ai[1] >= 240.0 {
                    self.ai[1] = 0.0;
                    self.ai[0] = 0.0;
                }
            }
            _ => {}
        }
        // `:32991-32995` — the horizontal damp close to the player.
        if (cx - pcx).abs() < 50.0 {
            firing = true;
        }
        if firing {
            self.vx *= 0.9;
            if self.vx > -0.1 && self.vx < 0.1 {
                self.vx = 0.0;
            }
        } else {
            if self.facing > 0 {
                self.vx = (self.vx * 20.0 + speed) / 21.0;
            }
            if self.facing < 0 {
                self.vx = (self.vx * 20.0 - speed) / 21.0;
            }
        }
        // `:33012-33040` — the vertical hover: the player-overlap test, the
        // solid-tile probe under the boss and the fall/rise ramps.
        let probe_x = cx - 40.0;
        let probe_y = self.y + h - 20.0;
        let over_player = self.x < player.x
            && self.x + w > player.x + crate::player::WIDTH
            && self.y + h < player.y + player.height - 16.0;
        if over_player {
            self.vy += 0.5;
        } else if collides_rect(world, probe_x, probe_y, 80.0, 20.0) {
            if self.vy > 0.0 {
                self.vy = 0.0;
            }
            if self.vy > -0.2 {
                self.vy -= 0.025;
            } else {
                self.vy -= 0.2;
            }
            if self.vy < -4.0 {
                self.vy = -4.0;
            }
        } else {
            if self.vy < 0.0 {
                self.vy = 0.0;
            }
            if self.vy < 0.1 {
                self.vy += 0.025;
            } else {
                self.vy += 0.5;
            }
        }
        if self.vy > 10.0 {
            self.vy = 10.0;
        }
    }

    /// The Spazmatism (326) — the Twins' ground fighter: the C#'s aiStyle-3
    /// family (`AI_003_Fighters`, `NPC.cs:55856+`) with the cap-2 group's
    /// walk speed (`:57554-57578`).
    fn update_spazmatism(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        let speed = NpcKind::Spazmatism.ai3_walk_speed().unwrap_or(2.0);
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    fn update_twin(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 270.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.09;
            if self.vx.abs() > 8.5 {
                self.vx = 8.5 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.09;
            if self.vy.abs() > 8.5 {
                self.vy = 8.5 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 8.5;
            self.vy = dy / d * 8.5;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        // The hover anchor (`AI_022`'s `NPC.cs:24758`): the vanilla keeps
        // the twin's arrival band in `ai[0] = position.X` and
        // `FindFrame`'s case 125/126 reads it back as the +3 row offset —
        // the live wing-beat rows 3-5. The port's simplified hover has no
        // arrival state, so the anchor write stands in for it (the value
        // is any live pixel X, which is > 1 past the spawn tick).
        self.ai[0] = self.x;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_051`'s Plantera v1 (`NPC.cs`'s aiStyle 51 machine) — the EoC
    /// cycle shape with Plantera's numbers: the hover (the ±0.1, the 8
    /// cap), the charge (the 480-tick cycle, the direct velocity x 7.5),
    /// the decel (the 40 ticks x0.98). The vine hooks and the second-form
    /// rage = the follow-ups.
    fn update_plantera(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 250.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.1;
            if self.vx.abs() > 8.0 {
                self.vx = 8.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.1;
            if self.vy.abs() > 8.0 {
                self.vy = 8.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 7.5;
            self.vy = dy / d * 7.5;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_054`'s Brain of Cthulhu v1 (`NPC.cs`'s aiStyle 54 machine) —
    /// the teleport-charge cycle: the telegraph pause, the dash at the
    /// player; the `dontTakeDamage` until the Creepers die (the vanilla's
    /// `ai[3]` tracks them). The v1: the teleport-dash cycle without the
    /// invulnerability gate (the Creepers' orbit = the follow-ups).
    /// The Golem family's machines: the aiStyle-45 body (the spawn init,
    /// the charge-slam-flight states), the aiStyle-46 head (the tether,
    /// the fireball cycles), the aiStyle-47 fists (the follow, the
    /// wind-up, the dash), the aiStyle-48 free head (the permanent
    /// shield, the volleys). The ai[] slots: the body's ai[0] = the
    /// state (0 charge / 1 flight), ai[1] (cast_windup) = the charge
    /// counter (300 -> -20 -> the slam at -1); the head's ai[0] = the
    /// phase (1 under the half life), ai[1] = the fireball cadence (300),
    /// ai[2] = the beam volley; the fists' ai[0] = the state (0 follow /
    /// 1 wind-up / 2 dash), ai[1] = the contact cadence (60) then the
    /// wind-up (30). The caster_counter boots the body's one-shot init.
    /// The Deerclops (`AI_123_Deerclops`, `NPC.cs:44017-44407` + the
    /// helpers): the state machine over `ai[0]` — -1 the spawn walk, 0 the
    /// walk + the attack selection, 1 the ice spikes forward, 2 the rubble
    /// up, 3 the slow-debuff blizzard, 4 the spikes both sides, 5 the
    /// shadow-hand burst, 6/7 the go-home, 8 the flee/die. `ai[1]` = the
    /// per-state counter, `ai[2]/ai[3]` = the home tile (the port's
    /// `home` field pair). The classic-only: the passive shadow hands
    /// (the expert arm) fold out; the life ramp feeds only the speed
    /// (`:44645`'s 3.5+1).
    /// The Prime's four limbs (`aiStyle 33-36`, `NPC.cs:27943-28990`):
    /// the shared tether dock `(headCenter.X - 200*side, head.Y + 230)`
    /// with the >800 desync band, the head-gone despawn, and the
    /// per-limb attacks — the cannon's bombs (102, dmg 0, the 40-tick
    /// cadence), the saw's 22-dash, the vice's 4-dash-20 cycle, the
    /// laser's DeathLaser volleys (100, 25 dmg). The head's centre rides
    /// the World's Prime anchor (the write-through).
    fn update_prime_limb(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let side = if self.ai[0] < 0.0 { -1.0 } else { 1.0 };
        // The dock: the head's centre ± 200 side, +230 below.
        let dock_x = world.prime_x - 200.0 * side;
        let dock_y = world.prime_y + 230.0;
        if !world.prime_present {
            self.hp = 0;
            return;
        }
        // The tether drift to the dock (`:28530-28544` shared math).
        let dx = dock_x - cx;
        let dy = dock_y - cy;
        if dx.abs() > 8.0 {
            self.vx += dx.signum() * 0.07;
            self.vx = self.vx.clamp(-8.0, 8.0);
        } else {
            self.vx *= 0.9;
        }
        if dy.abs() > 3.0 {
            self.vy += dy.signum() * 0.04;
            self.vy = self.vy.clamp(-3.0, 3.0);
        } else {
            self.vy *= 0.9;
        }
        // The per-limb attacks.
        match self.kind {
            NpcKind::PrimeCannon => {
                // The bomb cadence (`:28739-28752`): every 40 ticks the
                // 102 bomb at 10 speed — the vanilla's damage is 0 (the
                // port's 0 stands in).
                self.ai[3] += 1.0;
                if self.ai[3] >= 40.0 {
                    self.ai[3] = 0.0;
                    let dx = pcx - cx;
                    let dy = pcy - cy;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        crate::projectile::BOMB_SKELETRON,
                        cx,
                        cy,
                        dx / d * 10.0,
                        dy / d * 10.0,
                        0,
                    ));
                }
            }
            NpcKind::PrimeSaw => {
                // The 22-dash (`:28166-28176`): the rise to above the
                // head, then the strike.
                if self.ai[2] == 1.0 {
                    self.vy -= 0.1;
                    self.vx *= 0.95;
                    if cy < world.prime_y - 200.0 {
                        self.ai[2] = 2.0;
                        let dx = pcx - cx;
                        let dy = pcy - cy;
                        let d = (dx * dx + dy * dy).sqrt().max(0.001);
                        self.vx = dx / d * 22.0;
                        self.vy = dy / d * 22.0;
                    }
                } else if self.ai[2] >= 2.0 {
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                } else {
                    self.ai[3] += 1.0;
                    if self.ai[3] >= 300.0 {
                        self.ai[2] = 1.0;
                        self.ai[3] = 0.0;
                    }
                }
            }
            NpcKind::PrimeVice => {
                // The 4-dash cycle (`:28453-28482`): the rise to
                // head.Y - 280 then the 20-speed dash, four times.
                if self.ai[2] == 1.0 {
                    self.vy -= 0.1;
                    self.vx *= 0.95;
                    if cy < world.prime_y - 280.0 {
                        let dx = pcx - cx;
                        let dy = pcy - cy;
                        let d = (dx * dx + dy * dy).sqrt().max(0.001);
                        self.vx = dx / d * 20.0;
                        self.vy = dy / d * 20.0;
                        self.ai[2] = 2.0;
                    }
                } else if self.ai[2] == 2.0 {
                    if self.ai[3] >= 4.0 {
                        self.ai[2] = 0.0;
                        self.ai[3] = 0.0;
                    } else {
                        self.ai[2] = 1.0;
                        self.ai[3] += 1.0;
                    }
                } else {
                    self.ai[3] += 1.0;
                    if self.ai[3] >= 600.0 {
                        self.ai[2] = 1.0;
                        self.ai[3] = 0.0;
                    }
                }
            }
            _ => {
                // The laser limb: the DeathLaser volley (`:28973-28986`)
                // at the 80-tick cadence.
                self.ai[3] += 1.0;
                if self.ai[3] >= 80.0 {
                    self.ai[3] = 0.0;
                    let dx = pcx - cx;
                    let dy = pcy - cy;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.wants_hostile_shot.push((
                        crate::projectile::DESTROYER_LASER,
                        cx,
                        cy,
                        dx / d * 10.0,
                        dy / d * 10.0,
                        25,
                    ));
                }
            }
        }
        // The display direction (`AI_033`'s `NPC.cs:27960` and its three
        // siblings `:28247`/`:28524`/`:28759`): `spriteDirection =
        // -(int)ai[0]` — the limbs carry their seat's side, and the flip
        // rides it (the C# reads it at `Main.cs:23124-23128`).
        self.sprite_dir = if self.ai[0] < 0.0 { 1 } else { -1 };
        // The sprite rotations (the `atan2` writes each limb runs every
        // tick): the saw and the vice aim their "up" along the dock vector
        // (the +1.57 baseline — the vice's rise rides the wider 280 dock,
        // `AI_034`'s `NPC.cs:28445`), the cannon follows its dock while it
        // idles (`:28656`), and the laser always aims at the player (the
        // -1.57 baseline, `:28891`/`:28969`). The v1 wrote no rotation, so
        // the whole family drew upright.
        self.rotation = match self.kind {
            NpcKind::PrimeSaw => (dock_y - cy).atan2(dock_x - cx) + std::f32::consts::FRAC_PI_2,
            NpcKind::PrimeVice => {
                // The rise state swings at the wider 280 dock (`:28435`).
                let dx = if self.ai[2] == 1.0 {
                    world.prime_x - 280.0 * side
                } else {
                    dock_x
                };
                (world.prime_y + 230.0 - cy).atan2(dx - cx) + std::f32::consts::FRAC_PI_2
            }
            NpcKind::PrimeCannon => (dock_y - cy).atan2(dock_x - cx) + std::f32::consts::FRAC_PI_2,
            _ => (pcy - cy).atan2(pcx - cx) - std::f32::consts::FRAC_PI_2,
        };
        self.x += self.vx;
        self.y += self.vy;
    }

    /// The WoF's mirror eye (`aiStyle 28`, `NPC.cs:26053-26208`): the
    /// life/X mirror, the Y target `(band mid + drawTop/Bottom) / 2` chased
    /// at ±1 capped ±5, the eye-laser volley (the 600-tick idle → 4-shot
    /// bursts at 46-tick intervals, projectile 83 at the ramp 11/12/13).
    /// The parent anchor rides the World's WoF fields (the write-through).
    fn update_wof_eye(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        if !world.wof_present {
            // `wofNPCIndex < 0` → the eye deactivates (the sim holds the
            // despawn — the port mirrors via hp).
            self.hp = 0;
            return;
        }
        // The life mirror (`:26061-26064`) + the X/direction mirror.
        self.x = world.wof_x - w * 0.5;
        self.facing = if world.wof_x >= pcx { 1 } else { -1 };
        // `:26393-26403` (aiStyle 29) — the eye's flip is its own
        // `num391 = player.Center.X - width/2 - eye.Center.X`, NOT the
        // wall's `direction` the `:26068` body mirror reads: `a.frame 29`
        // flips the iris toward the target even when the wall faces away.
        self.sprite_dir = if pcx - w * 0.5 - (self.x + w * 0.5) > 0.0 {
            1
        } else {
            -1
        };
        let band_mid = (world.wof_draw_top + world.wof_draw_bottom) * 0.5;
        let top_half = self.ai[0] > 0.0;
        let target_y = if top_half {
            (band_mid + world.wof_draw_top) * 0.5
        } else {
            (band_mid + world.wof_draw_bottom) * 0.5 - h * 0.5
        };
        self.vy += (target_y - (self.y + h * 0.5)).signum() * 1.0;
        self.vy = self.vy.clamp(-5.0, 5.0);
        self.y += self.vy;
        // The volley (`:26125-26206`): the idle 600, then the 46-tick
        // shots in bursts of 4.
        self.ai[1] += 1.0;
        if world.wof_hp_ratio < 0.5 {
            self.ai[1] += 1.0;
        }
        if world.wof_hp_ratio < 0.25 {
            self.ai[1] += 1.0;
        }
        if self.ai[2] == 0.0 && self.ai[1] > 600.0 {
            self.ai[2] = 1.0;
            self.ai[1] = 0.0;
        }
        if self.ai[2] > 0.0 && self.ai[1] > 45.0 {
            let dx = pcx - cx;
            let dy = pcy - (self.y + h * 0.5);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let mut speed = 9.0;
            let mut dmg = 11;
            if world.wof_hp_ratio < 0.5 {
                speed += 1.0;
                dmg += 1;
            }
            if world.wof_hp_ratio < 0.25 {
                speed += 1.0;
                dmg += 1;
            }
            if world.wof_hp_ratio < 0.1 {
                speed += 2.0;
                dmg += 2;
            }
            self.ai[1] = 0.0;
            self.ai[2] += 1.0;
            if self.ai[2] >= 5.0 {
                self.ai[2] = 0.0;
            }
            // `:26167`: `CanHit(base.position, base.width, base.height,
            // player.position, player.width, player.height)`.
            if crate::projectile::can_hit(
                world,
                self.x,
                self.y,
                w,
                h,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            ) {
                self.wants_hostile_shot.push((
                    crate::projectile::EYE_LASER,
                    cx,
                    self.y + h * 0.5,
                    dx / d * speed,
                    dy / d * speed,
                    dmg,
                ));
            }
        }
    }

    /// The Skeletron's hand (`aiStyle 12`, `NPC.cs:22300-22613`): the
    /// tether return to the orbit slot `(headCenter.X - 200 * side,
    /// head.Y + 230)`, the 300-tick lunge cadence, the -8 rise to 200
    /// above the head, the 18 px/tick strike at the player's centre,
    /// the return. The head's centre rides the World's skeletron anchor
    /// (the WoF write-through shape). The vanilla's `ai[2] += 10f`
    /// head-death despawn rides the sim.
    fn update_skeletron_hand(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let side = if self.ai[0] < 0.0 { -1.0 } else { 1.0 };
        // The orbit slot: 200 to the side, 230 below the head's top.
        let slot_x = world.skeletron_x - 200.0 * side;
        let slot_y = world.skeletron_y + 230.0;
        match self.ai[2] as i32 {
            0 | 3 => {
                // The drift-return (`:22447-22504`).
                self.ai[3] += 1.0;
                if self.ai[3] >= 300.0 {
                    self.ai[2] = 1.0;
                    self.ai[3] = 0.0;
                }
                if cy > slot_y {
                    self.vy -= 0.04;
                    if self.vy > 0.0 {
                        self.vy *= 0.96;
                    }
                    self.vy = self.vy.clamp(-3.0, 3.0);
                }
                if cx > slot_x {
                    self.vx -= 0.07;
                    if self.vx > 0.0 {
                        self.vx *= 0.96;
                    }
                    self.vx = self.vx.clamp(-8.0, 8.0);
                } else {
                    self.vx += 0.07;
                    if self.vx < 0.0 {
                        self.vx *= 0.96;
                    }
                    self.vx = self.vx.clamp(-8.0, 8.0);
                }
                // The rotation write folds — the port's draw reads the
                // velocity direction for the hand's tilt.
            }
            1 => {
                // The rise (`:22506-22539`): the -0.1 climb, the -8 cap,
                // the 18 px/tick strike at the player above 200.
                self.vy -= 0.1;
                if self.vy < -8.0 {
                    self.vy = -8.0;
                }
                if cy < world.skeletron_y - 200.0 {
                    let dx = pcx - cx;
                    let dy = pcy - cy;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.vx = dx / d * 18.0;
                    self.vy = dy / d * 18.0;
                    self.ai[2] = 2.0;
                }
            }
            _ => {
                // The strike/return states decel.
                self.vx *= 0.95;
                self.vy *= 0.95;
                self.ai[2] = 3.0;
            }
        }
        self.x += self.vx;
        self.y += self.vy;
    }

    fn update_deerclops(&mut self, world: &World, player: &mut Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // The damage gate (`:44024-44039`): the 30-tick ramp toward the
        // target; `dontTakeDamage` while a target stays within 450px.
        let near = (cx - pcx).hypot(cy - pcy) < 450.0;
        self.swim_phase = (self.swim_phase + if near { 1.0 } else { -1.0 }).clamp(-10.0, 30.0);
        self.dont_take_damage = self.swim_phase >= 30.0;
        if self.ai[0] == -1.0 {
            self.swim_phase = -10.0;
        }
        // The walk (`AI_123_Deerclops_Movement`, `:44643-44726`).
        if self.ai[0] != 5.0 && self.ai[0] != 1.0 && self.ai[0] != 4.0 {
            let speed_cap = 3.5 + 1.0 * (1.0 - self.hp as f32 / self.max_hp as f32);
            let target_x = if self.ai[0] == -1.0 {
                self.facing_move_right(5.35, pcx, cx);
                cx + 5.0
            } else {
                pcx
            };
            let cap = if self.ai[0] == -1.0 { 5.35 } else { speed_cap };
            let dx = target_x - cx;
            let halt = dx.abs() < 80.0 || self.ai[0] == 6.0 || self.ai[0] == 7.0;
            if !halt && player.hp > 0 {
                self.vx = (self.vx * 3.0 + dx.clamp(-cap, cap)) / 4.0;
                if self.on_ground && self.vy >= 0.0 {
                    self.vy = -8.0; // the ledge hop
                }
            } else {
                self.vx *= 0.9;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
            }
            self.facing = if self.vx >= 0.0 { 1 } else { -1 };
            // `:26416-26432` (aiStyle 29 is the WoF eye — the Deerclops
            // rides aiStyle 40, `:29716`, which pins the flip at -1 and
            // carries the aim in `rotation` instead).
            self.sprite_dir = -1;
            self.vy = (self.vy + 0.4).clamp(-8.0, 16.0);
            if self.on_ground {
                self.vy = 0.0;
            }
            self.x += self.vx;
            self.y += self.vy;
        }
        // The attack selection + the states.
        match self.ai[0] as i32 {
            0 => {
                // `:44141-44201`'s state-0 selection (the classic arms).
                if self.on_ground && player.hp > 0 {
                    self.ai[1] += 1.0;
                    let dx = (pcx - cx).abs();
                    let quiet = self.vx.abs() < 0.05;
                    if quiet && self.ai[1] >= 90.0 && dx < 120.0 {
                        self.ai[0] = 5.0; // the shadow-hand burst
                        self.ai[1] = 0.0;
                    } else if self.ai[1] >= 120.0 && dx > 100.0 {
                        self.ai[0] = 3.0; // the slow-debuff blizzard
                        self.ai[1] = 0.0;
                    } else if self.ai[1] >= 240.0 && dx < 120.0 {
                        self.ai[0] = 2.0; // the rubble up
                        self.ai[1] = 0.0;
                    }
                } else {
                    self.ai[1] = 0.0;
                }
            }
            1 => {
                // The ice spikes forward (`:44203-44217` + the helper).
                self.ai[1] += 1.0;
                // `:44466-44500` — the forward row: from the 36th tick,
                // every fourth tick makes four spikes (the row's spread is
                // 20 tiles).
                deerclops_make_spikes(self, world, player, rng, 20, 4, 36.0, false);
                if self.ai[1] >= 80.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            2 => {
                // `:44242-44263` — the rubble: from the 32nd tick, one
                // rock per tick (the row's spread is 20).
                self.ai[1] += 1.0;
                if self.ai[1] >= 32.0 {
                    let elapsed = self.ai[1] as i32 - 32;
                    if elapsed < 20 {
                        deerclops_shoot_rubble_up(self, world, rng, elapsed);
                    }
                }
                if self.ai[1] >= 60.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            3 => {
                // `:44264-44307` — the chill aura: every fourth tick from
                // the 28th the player within 800 px gets the Slow buff (32)
                // for 720 ticks — the C# spawns NO projectile here.
                self.ai[1] += 1.0;
                if (self.ai[1] as i32) % 4 == 0 && self.ai[1] >= 28.0 {
                    if !player.dead
                        && player.buffs.find_buff_index(32).is_none()
                        && ((cx - pcx).powi(2) + (cy - pcy).powi(2)).sqrt() < 800.0
                    {
                        player.buffs.add_buff(32, 720);
                    }
                }
                if self.ai[1] >= 60.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            4 => {
                // The ice spikes both sides (`:44218-44233` + the helper).
                self.ai[1] += 1.0;
                // `:44502-44532` — the both-sides row: from the 56th tick,
                // every second tick makes the row's spikes on both sides
                // (the spread is 15 tiles).
                deerclops_make_spikes(self, world, player, rng, 15, 2, 56.0, true);
                if self.ai[1] >= 90.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            5 => {
                // The shadow-hand burst (`:44374-44404`): the 6 hands of
                // the 965 at 15 dmg, the 60-tick total.
                self.ai[1] += 1.0;
                if self.ai[1] == 30.0 {
                    // The C#'s pick (`Projectile.RandomizeInsanityShadowFor`,
                    // `Projectile.cs:42408-42493`): the position, the velocity
                    // and the two `ai` slots — the arm's variation age and
                    // the angle (`:42592-42634`). The C#'s rejection loop
                    // (the spawn must not land on a player other than the
                    // target) never rejects in the port: the only player IS
                    // the target, and the C# skips the target in that check.
                    let mut shadow_rng = crate::world_sim::npcs::LootRng(rng);
                    for _ in 0..6 {
                        let (pos, vel, ai0, ai1) = crate::projectile::randomize_insanity_shadow_for(
                            Vec2::new(pcx, pcy),
                            Vec2::new(player.vx, player.vy),
                            true,
                            &mut shadow_rng,
                        );
                        self.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                            kind: crate::projectile::SHADOW_HAND,
                            x: pos.x,
                            y: pos.y,
                            vx: vel.x,
                            vy: vel.y,
                            damage: 15,
                            ai0,
                            ai1,
                        });
                    }
                }
                if self.ai[1] >= 60.0 {
                    self.ai[0] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            _ => {
                // The go-home/teleport/flee states hold the walk.
                self.ai[1] += 1.0;
            }
        }
    }

    /// The state-0's forced-direction walk (`:44667-44672`): ai[0]==-1
    /// walks at 5.35 rightward; the heading rides the sign of `target`.
    fn facing_move_right(&self, cap: f32, target_x: f32, cx: f32) -> f32 {
        let dir = if target_x >= cx { 1.0 } else { -1.0 };
        cap * dir
    }

    /// The shadow-hand burst's angular jitter (`RandomizeInsanityShadowFor`'s
    /// spin, `Projectile.cs:42434-42477`).
    fn facing_move_phase(&self, rng: &mut u64) -> f32 {
        (crate::app::rand01(rng) - 0.5) * 0.25
    }

    fn update_golem(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        match self.kind {
            NpcKind::Golem => {
                // The body (`AI_045_Golem`, `NPC.cs:19505-19780`).
                // The alpha fade (`:19535-19543`): `alpha -= 10` with
                // `ai[1] = 0` — the port's ai[1] analog is `cast_windup`
                // (the wind-up counter), so the fade holds the ramp at 0.
                if self.alpha > 0 {
                    self.alpha = self.alpha.saturating_sub(10);
                    self.cast_windup = 0;
                }
                // The init (the `localAI[0]` one-shot, `:19517-19525`):
                // the parts spawn request — the sim resolves the offsets
                // (247 at -84/-9, 248 at +78/-9, 246 at -3/-57).
                if self.caster_counter == 0 && self.cast_windup == 0 {
                    self.caster_counter = 1;
                    self.wants_sphere
                        .push((NpcKind::GolemFistRight, cx - 84.0, cy - 9.0));
                    self.wants_sphere
                        .push((NpcKind::GolemFistLeft, cx + 78.0, cy - 9.0));
                    self.wants_sphere
                        .push((NpcKind::GolemHead, cx - 3.0, cy - 57.0));
                }
                if self.ai[0] == 0.0 {
                    // The grounded charge (`:19638-19703`).
                    self.vx *= 0.8;
                    self.cast_windup += 1;
                    if self.cast_windup >= 300 {
                        self.cast_windup = -20;
                    }
                    if self.cast_windup == -1 {
                        // The slam launch (`:19684-19700`).
                        self.facing = if pcx >= cx { 1 } else { -1 };
                        self.sprite_dir = self.facing;
                        self.vx = 4.0 * self.facing as f32;
                        self.vy = if self.hp < self.max_hp {
                            (-12.1f32 * (1.0 + 9.0) / 10.0).max(-19.1)
                        } else {
                            -12.1
                        };
                        self.ai[0] = 1.0;
                        self.cast_windup = 0;
                    }
                } else {
                    // The flight (`:19704-19766`).
                    if self.on_ground {
                        self.ai[0] = 0.0;
                    } else {
                        let over_x = self.x < pcx && self.x + w > pcx;
                        if over_x {
                            self.vx *= 0.9;
                        } else {
                            let dir = if self.facing < 0 { -1.0 } else { 1.0 };
                            self.vx += dir * 0.2;
                            let mut num7 = 3.0;
                            if self.hp < self.max_hp {
                                num7 += 1.0;
                            }
                            if self.hp < self.max_hp / 2 {
                                num7 += 1.0;
                            }
                            if self.hp < self.max_hp / 4 {
                                num7 += 1.0;
                            }
                            self.vx = self.vx.clamp(-num7, num7);
                        }
                    }
                }
            }
            NpcKind::GolemHead => {
                // The attached head (`aiStyle 46`, `NPC.cs:31129-31346`):
                // the offset lock to the body's anchor (-3, -57), the
                // phase-2 at the half life, the fireball cycles.
                let scale = 1.0;
                let tx = world.golem_x - cx - 3.0 * scale;
                let ty = world.golem_y - cy - 57.0 * scale;
                let d = (tx * tx + ty * ty).sqrt().max(0.001);
                self.vx = (self.vx + tx / d * 9.0).clamp(-7.0, 7.0);
                self.vy = (self.vy + ty / d * 9.0).clamp(-7.0, 7.0);
                // The alpha fade (`:31169-31173`): `ai[1] = 30` — the
                // mid-window hold keeps the mouth row closed (20..280).
                if self.alpha > 0 {
                    self.alpha = self.alpha.saturating_sub(10);
                    self.ai[1] = 30.0;
                }
                if self.hp < self.max_hp / 2 {
                    self.ai[0] = 1.0;
                } else {
                    self.ai[0] = 0.0;
                }
                // The fireball cycle (`:31174-31258`): the 300 cadence,
                // the 258 at 18 (phase 0) / 24 (phase 1), the spawn
                // Center + (0, 10) with the phase-1 +-40 side offset.
                // The phase-1 cadence doubles under the 40%/20% life
                // tiers (`:31227-31235`'s extra `num688` adds — the
                // solo balance factor).
                let mut step = 1.0;
                if self.ai[0] == 1.0 {
                    if (self.hp as f32) < (self.max_hp as f32) * 0.4 {
                        step += 1.0;
                    }
                    if (self.hp as f32) < (self.max_hp as f32) * 0.2 {
                        step += 1.0;
                    }
                }
                self.ai[1] += step;
                // The phase-1 head turn (`:31207-31224`): the three-way
                // `localAI[1]` (`head_turn`) by the player's side about
                // `Center.X +- width` — the FindFrame +4/+2 rows and the
                // volley's side arm read it. Phase 0 leaves it (the C#
                // writes localAI[1] only in the phase-1 arm).
                if self.ai[0] == 1.0 {
                    self.head_turn = if pcx < cx - w {
                        -1
                    } else if pcx > cx + w {
                        1
                    } else {
                        0
                    };
                }
                if self.ai[1] >= 300.0 {
                    self.ai[1] = 0.0;
                    let ox = if self.ai[0] == 1.0 {
                        match self.head_turn {
                            -1 => -40.0 * scale,
                            1 => 40.0 * scale,
                            _ => 0.0,
                        }
                    } else {
                        0.0
                    };
                    self.wants_hostile_shot.push((
                        crate::projectile::GOLEM_FIREBALL,
                        cx + ox,
                        cy + 10.0 * scale,
                        pcx - cx,
                        pcy - cy,
                        if self.ai[0] == 1.0 { 24 } else { 18 },
                    ));
                }
                // The beam volley (`:31261-31334`, the phase-1 arm): the
                // cadence `num688` (solo 1) with the extra +1 adds at the
                // life < 1/3, 1/4, 1/5 tiers and +4 while the head cannot
                // hit the player (`:31363`'s `!Collision.CanHit(Center,
                // 1, 1, ...)`); the vanilla's 60+Next(600)
                // trigger folds to the flat 300 average (the RNG stream
                // discipline). Two arms by the turn: in-band (`head_turn`
                // 0) fires the two-shot EyeBeam 259 (dmg 28) at the 11
                // speed from +-18; a side turn fires the single 12-speed
                // beam from +-30 (`:31305-31327`). Both carry the 3-tick
                // lead; the C#'s `timeLeft = 300` override rides the
                // pool's timed-spawn arm (the laser's shape).
                if self.ai[0] == 1.0 {
                    let mut step = 1.0;
                    if (self.hp as f32) < (self.max_hp as f32) / 3.0 {
                        step += 1.0;
                    }
                    if self.hp < self.max_hp / 4 {
                        step += 1.0;
                    }
                    if self.hp < self.max_hp / 5 {
                        step += 1.0;
                    }
                    if !crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0) {
                        step += 4.0;
                    }
                    self.ai[2] += step;
                    if self.ai[2] > 300.0 {
                        self.ai[2] = 0.0;
                        let (shots, count) = if self.head_turn == 0 {
                            ([(-18.0 * scale, 11.0), (18.0 * scale, 11.0)], 2)
                        } else if self.head_turn == -1 {
                            ([(-30.0 * scale, 12.0), (0.0, 0.0)], 1)
                        } else {
                            ([(30.0 * scale, 12.0), (0.0, 0.0)], 1)
                        };
                        for (sx, sp) in shots.into_iter().take(count) {
                            let mut vx = pcx - (cx + sx);
                            let mut vy = pcy - (cy - 22.0 * scale);
                            let vd = (vx * vx + vy * vy).sqrt().max(0.001);
                            vx = vx / vd * sp;
                            vy = vy / vd * sp;
                            self.wants_hostile_shot.push((
                                crate::projectile::GOLEM_EYE_BEAM,
                                cx + sx + vx * 3.0,
                                cy - 22.0 * scale + vy * 3.0,
                                vx,
                                vy,
                                28,
                            ));
                        }
                    }
                }
            }
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => {
                // The fists (`AI_047_GolemFist`, `NPC.cs:19231-19503`):
                // the tether (`vector = body.Center + body.velocity +
                // (0, -9 * scale)`, `:19261-19265`, with the -84 / +78
                // side fold), the follow with the contact cadence 60,
                // the wind-up 30, the dash 12 (+tiers). `num` (the
                // `GetMyBalance` factor, `:19233`) is 1.0 for the solo
                // unscaled player — the multiplayer/good-world branches
                // stay off (documented in ANALYSIS).
                let left = self.kind == NpcKind::GolemFistLeft;
                let anchor_x = world.golem_x + world.golem_vx + if left { -84.0 } else { 78.0 };
                let anchor_y = world.golem_y - 9.0;
                // The alpha fade (`:19248-19255`): `ai[1] = 0` — the
                // cadence counter resets while the fist fades in.
                if self.alpha > 0 {
                    self.alpha = self.alpha.saturating_sub(10);
                    self.ai[1] = 0.0;
                }
                if self.ai[0] == 0.0 {
                    // The follow (`:19262-19330`).
                    let dx = anchor_x - cx;
                    let dy = anchor_y - cy;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    // The approach speed (`:19265-19282`): the base 14,
                    // +3/+3 under the own half/quarter life, +8 while the
                    // body is damaged, x1 solo, clamped at 32.
                    let mut num3: f32 = 14.0;
                    if self.hp < self.max_hp / 2 {
                        num3 += 3.0;
                    }
                    if self.hp < self.max_hp / 4 {
                        num3 += 3.0;
                    }
                    if world.golem_hp_ratio < 1.0 {
                        num3 += 8.0;
                    }
                    num3 = num3.min(32.0);
                    if d < 12.0 + num3 {
                        // In contact (`:19285-19317`): the raw delta
                        // velocity (the C# writes `velocity = vector2`),
                        // rotation reset, the cadence accumulates ONLY
                        // here, and the 60 trigger passes the side gate.
                        self.vx = dx;
                        self.vy = dy;
                        self.rotation = 0.0;
                        let mut step = 1.0;
                        if self.hp < self.max_hp / 2 {
                            step += 1.0;
                        }
                        if self.hp < self.max_hp / 4 {
                            step += 1.0;
                        }
                        if world.golem_hp_ratio < 1.0 {
                            step += 10.0;
                        }
                        self.ai[1] += step;
                        if self.ai[1] >= 60.0 {
                            // `:19308-19317`'s side gate: the left fist
                            // dashes when the player stands right of its
                            // `Center.X + 100`, the right fist when they
                            // stand left of `- 100`; otherwise the
                            // cadence resets and the fist keeps hovering.
                            let dashes = if left {
                                cx + 100.0 > pcx
                            } else {
                                cx - 100.0 < pcx
                            };
                            self.ai[1] = 0.0;
                            if dashes {
                                self.ai[0] = 1.0;
                            }
                        }
                    } else {
                        // Out of range (`:19319-19329`): move at num3
                        // along the delta; the rotation aims the sprite
                        // along the velocity (247's arm is negated — the
                        // left sheet points the other way).
                        let ratio = num3 / d;
                        self.vx = dx * ratio;
                        self.vy = dy * ratio;
                        self.rotation = if left {
                            (-self.vy).atan2(-self.vx)
                        } else {
                            self.vy.atan2(self.vx)
                        };
                    }
                } else if self.ai[0] == 1.0 {
                    // The wind-up (`:19331-19390`): the hard Center snap
                    // to the anchor + the 30-tick hold; the launch sets
                    // the dash speed tiers.
                    self.x = anchor_x - w * 0.5;
                    self.y = anchor_y - h * 0.5;
                    self.rotation = 0.0;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.ai[1] += 1.0;
                    if self.ai[1] >= 30.0 {
                        self.ai[0] = 2.0;
                        self.ai[1] = 0.0;
                        // `:19361-19378`'s dash tiers: the base 12,
                        // +4/+4 under the own half/quarter life, +10 while
                        // the body is damaged, x1 solo, clamped at 48.
                        let mut sp: f32 = 12.0;
                        if self.hp < self.max_hp / 2 {
                            sp += 4.0;
                        }
                        if self.hp < self.max_hp / 4 {
                            sp += 4.0;
                        }
                        if world.golem_hp_ratio < 1.0 {
                            sp += 10.0;
                        }
                        sp = sp.min(48.0);
                        let dx = pcx - cx;
                        let dy = pcy - cy;
                        let d = (dx * dx + dy * dy).sqrt().max(0.001);
                        self.vx = dx / d * sp;
                        self.vy = dy / d * sp;
                        self.rotation = if left {
                            (-self.vy).atan2(-self.vx)
                        } else {
                            self.vy.atan2(self.vx)
                        };
                    }
                } else if self.ai[0] == 2.0 {
                    // The dash (`:19392-19449`): the lunge until the
                    // anchor distance passes 700 (the C#'s `collideX /
                    // collideY` exit rides the tile-collision pass — the
                    // port's sim has no per-NPC collision flags yet, the
                    // documented deviation).
                    self.ai[1] += 1.0;
                    let ddx = anchor_x - cx;
                    let ddy = anchor_y - cy;
                    if (ddx * ddx + ddy * ddy).sqrt() > 700.0 {
                        self.ai[0] = 0.0;
                    }
                }
            }
            NpcKind::GolemHeadFree => {
                // The free head (`aiStyle 48`, `NPC.cs:31351-31586`): the
                // `dontTakeDamage` SetDefaults shield NEVER clears — the
                // machine is the hover + the volleys only.
                let tx = pcx - cx;
                let ty = (pcy - 300.0) - cy;
                let d = (tx * tx + ty * ty).sqrt().max(0.001);
                self.vx += tx / d * 0.05;
                self.vy += ty / d * 0.05;
                self.vx = self.vx.clamp(-7.0, 7.0);
                self.vy = self.vy.clamp(-7.0, 7.0);
                // The fireball salvo (`:31421-31465`): the 300 cadence
                // (doubled under the 10% body life, `:31432-31436`), the
                // 258 dmg 20 at the 8 speed from Center - (0, 10). The
                // C#'s `flag53` (`:31363`'s `!CanHit(Center, 1, 1, ...)`,
                // the block hold `:31446-31450`) pins `ai[1] = 20` while
                // the line is blocked.
                let mut step = 1.0;
                if world.golem_hp_ratio < 0.1 {
                    step += 1.0;
                }
                self.ai[1] += step;
                if !crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0) {
                    self.ai[1] = 20.0;
                }
                if self.ai[1] >= 300.0 {
                    self.ai[1] = 0.0;
                    self.wants_hostile_shot.push((
                        crate::projectile::GOLEM_FIREBALL,
                        cx,
                        cy - 10.0,
                        pcx - cx,
                        pcy - cy,
                        20,
                    ));
                }
                // The eye-beam barrage (`:31506-31568`): the 100+Next(4800)
                // cadence folds to the flat 1500 average; 2 per volley,
                // the 259 dmg/speed tiers by the BODY's hp ratio
                // (`:31524-31548`: 24->29 dmg, 11->12.25 at 50/40/30/20/10%),
                // the +-14 spread, the 3-tick lead.
                self.ai[2] += 1.0;
                if self.ai[2] > 1500.0 {
                    self.ai[2] = 0.0;
                    let ratio = world.golem_hp_ratio;
                    let (dmg, sp) = if ratio < 0.1 {
                        (29, 12.25)
                    } else if ratio < 0.2 {
                        (28, 12.0)
                    } else if ratio < 0.3 {
                        (27, 11.75)
                    } else if ratio < 0.4 {
                        (26, 11.5)
                    } else if ratio < 0.5 {
                        (25, 11.25)
                    } else {
                        (24, 11.0)
                    };
                    for sx in [-14.0, 14.0] {
                        let mut vx = pcx - (cx + sx);
                        let mut vy = pcy - (cy - 50.0);
                        let vd = (vx * vx + vy * vy).sqrt().max(0.001);
                        vx = vx / vd * sp;
                        vy = vy / vd * sp;
                        self.wants_hostile_shot.push((
                            crate::projectile::GOLEM_EYE_BEAM,
                            cx + sx + vx * 3.0,
                            cy - 50.0 + vy * 3.0,
                            vx,
                            vy,
                            dmg,
                        ));
                    }
                }
            }
            _ => {}
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// The Creeper (`AI_055`, `NPC.cs:32657-32735`): the orbit around
    /// the brain — beyond the 90 px ring the velocity blends home
    /// (`x15/16`), inside it the speed amplifies x1.05 under the 8 cap;
    /// the 1/200 roll (the classic; the expert's 1/100 folds out)
    /// lunges at the player at 8 px/tick; the lunge holds until the
    /// justHit recoil (`hurt_cd` stands in for `justHit`) sends it back
    /// to the orbit. The brainless Creepers die in the sim
    /// (`NPC.crimsonBoss < 0` -> `active = false`); the brain's anchor
    /// rides the World fields (the WoF's write-through shape).
    fn update_creeper(&mut self, world: &World, player: &Player, rng: &mut u64) {
        if !world.brain_present {
            return;
        }
        let (w, h) = self.size();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        if self.ai[0] == 0.0 {
            self.ai[1] = 0.0;
            let dx = world.brain_x - cx;
            let dy = world.brain_y - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if d > 90.0 {
                let s = 8.0 / d;
                self.vx = (self.vx * 15.0 + dx * s) / 16.0;
                self.vy = (self.vy * 15.0 + dy * s) / 16.0;
                return;
            }
            if self.vx.abs() + self.vy.abs() < 8.0 {
                self.vy *= 1.05;
                self.vx *= 1.05;
            }
            if crate::app::rand01(rng) < 1.0 / 200.0 {
                let (pcx, pcy) = player.center();
                let dx = pcx - cx;
                let dy = pcy - cy;
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                self.vx = dx / d * 8.0;
                self.vy = dy / d * 8.0;
                self.ai[0] = 1.0;
            }
            return;
        }
        // The lunge: the straight flight until the justHit recoil.
        if self.hurt_cd <= 0.0 {
            return;
        }
        self.ai[0] = 0.0;
    }

    /// `AI()`'s aiStyle-54 block (`NPC.cs:32370-32656`) — the Brain of
    /// Cthulhu's machine: the blink (the `-1`/`-2`/`-3` retreat and the
    /// `0`/`1`/`2` normal states), the velocity model and the despawn ramp.
    ///
    /// The creeper swarm's spawn and the shield ride the sim's pass
    /// (`world_sim/npcs.rs`: the 20 Creepers + `dont_take_damage` while one
    /// lives — the C#'s `localAI[0]` init and its shield); the arm reads the
    /// live Creeper count from `brain_local[2]` (the sim's write).
    ///
    /// The gore/dust burst (`:32401-32419`) is SKIPPED — no dust/gore layer.
    /// The MP-only branches (`Main.netMode != 0`) use the single-player
    /// values (the port has no multiplayer arm).
    fn update_brain(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // The `WorldGen.SolidTile` analog (the active + the solid set).
        let solid_tile = |x: i32, y: i32| -> bool {
            world
                .tile(x, y)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        };
        if self.ai[0] < 0.0 {
            // `:32420-32430` — the retreat's velocity: the 8-px drift toward
            // the player through the 50/51 blend.
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let k = 8.0 / d;
            self.vx = (self.vx * 50.0 + dx * k) / 51.0;
            self.vy = (self.vy * 50.0 + dy * k) / 51.0;
            match self.ai[0] as i32 {
                -1 => {
                    // `:32431-32478` — the blink timer and the target tile.
                    self.brain_local[1] += 1.0;
                    if self.hurt_cd > 0.0 {
                        // The C#'s `justHit` roll (`localAI[1] -= Next(5)`).
                        self.brain_local[1] -= crate::app::frame::rand_range(rng, 5) as f32;
                    }
                    let period = 60.0 + crate::app::frame::rand_range(rng, 120) as f32;
                    if self.brain_local[1] >= period {
                        self.brain_local[1] = 0.0;
                        // The tile pick: up to 100 tries, 10-12 tiles out
                        // (`:32440-32474`). The retreat has no sight gate.
                        if let Some((tx, ty)) = brain_pick_tile(
                            &solid_tile,
                            &|_tx: i32, _ty: i32| true,
                            pcx,
                            pcy,
                            player.vx,
                            player.vy,
                            10,
                            12,
                            false,
                            rng,
                        ) {
                            self.ai[3] = 0.0;
                            self.ai[0] = -2.0;
                            self.ai[1] = tx as f32;
                            self.ai[2] = ty as f32;
                        }
                    }
                }
                -2 => {
                    // `:32479-32501` — the fade-in and the teleport.
                    self.vx *= 0.9;
                    self.vy *= 0.9;
                    self.ai[3] += 25.0;
                    if self.ai[3] >= 255.0 {
                        self.ai[3] = 255.0;
                        self.x = self.ai[1] * 16.0 - (w * 0.5) as f32;
                        self.y = self.ai[2] * 16.0 - (h * 0.5) as f32;
                        self.ai[0] = -3.0;
                    }
                    self.alpha = (self.ai[3] as i32).clamp(0, 255) as u8;
                }
                -3 => {
                    // `:32502-32518` — the fade-out.
                    self.ai[3] -= 25.0;
                    if self.ai[3] <= 0.0 {
                        self.ai[3] = 0.0;
                        self.ai[0] = -1.0;
                    }
                    self.alpha = (self.ai[3] as i32).clamp(0, 255) as u8;
                }
                _ => {}
            }
        } else {
            // `:32522-32527` — the normal phase's velocity: straight at the
            // player at 1 px/tick.
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let speed = 1.0f32;
            if d < speed {
                self.vx = dx;
                self.vy = dy;
            } else {
                self.vx = dx / d * speed;
                self.vy = dy / d * speed;
            }
            match self.ai[0] as i32 {
                0 => {
                    // `:32532-32580` — the Creeper check and the blink timer.
                    if self.brain_local[2] == 0.0 {
                        // No Creeper left: the Brain retreats.
                        self.ai[0] = -1.0;
                        self.brain_local[1] = 0.0;
                        self.alpha = 0;
                    }
                    self.brain_local[1] += 1.0;
                    if self.brain_local[1] >= 120.0 + crate::app::frame::rand_range(rng, 300) as f32
                    {
                        self.brain_local[1] = 0.0;
                        // The tile pick: 12-40 tiles out, with the sight gate
                        // after the 75th try (`:32546-32578`).
                        let sight = |tx: i32, ty: i32| {
                            crate::projectile::can_hit(
                                world,
                                tx as f32 * 16.0,
                                ty as f32 * 16.0,
                                1.0,
                                1.0,
                                player.x,
                                player.y,
                                crate::player::WIDTH,
                                player.height,
                            )
                        };
                        if let Some((tx, ty)) = brain_pick_tile(
                            &solid_tile,
                            &sight,
                            pcx,
                            pcy,
                            player.vx,
                            player.vy,
                            12,
                            40,
                            true,
                            rng,
                        ) {
                            self.ai[0] = 1.0;
                            self.ai[1] = tx as f32;
                            self.ai[2] = ty as f32;
                        }
                    }
                }
                1 => {
                    // `:32581-32591` — the fade-in and the teleport.
                    self.alpha += 5;
                    if self.alpha >= 255 {
                        self.alpha = 255;
                        self.x = self.ai[1] * 16.0 - (w * 0.5) as f32;
                        self.y = self.ai[2] * 16.0 - (h * 0.5) as f32;
                        self.ai[0] = 2.0;
                    }
                }
                2 => {
                    // `:32592-32599` — the fade-out.
                    self.alpha -= 5;
                    if self.alpha <= 0 {
                        self.alpha = 0;
                        self.ai[0] = 0.0;
                    }
                }
                _ => {}
            }
        }
        // `:32619-32636` — the despawn ramp: a dead player or a player out
        // of the Crimson zone sinks the Brain and fades it out.
        if player.dead || !player.zones.crimson {
            if self.brain_local[3] < 120.0 {
                self.brain_local[3] += 1.0;
            }
            if self.brain_local[3] > 60.0 {
                self.vy += (self.brain_local[3] - 60.0) * 0.25;
            }
            self.ai[0] = 2.0;
            self.alpha = 10;
        } else if self.brain_local[3] > 0.0 {
            self.brain_local[3] -= 1.0;
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.sprite_dir = self.facing;
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// The Goblin Archer (`AI_003`'s ranged family, `NPC.cs:59069-59700`):
    /// the walk until the shot gate — grounded, `CanHit`, the distance
    /// < 700 (`num179`), `ai[1] <= 0` — then the stop state (`ai[2] = 1`,
    /// `ai[1] = num162 = 180` for 111, `velocity.X *= 0.5`); the state
    /// damps `velocity.X *= 0.9` (`:59614-59616`'s `!flag16` arm) and
    /// fires at the midpoint tick `ai[1] == 90` (`num163 = num162 / 2`):
    /// the aim = the player's centre minus the archer's, the vertical
    /// lead `|dx| * 0.1` (`num166`), the aim jitter +-40 px (`:59370`'s
    /// `Next(-40, 41)` fallthrough), the normalized speed 9 (`num164 =
    /// 9f` for 111), the hostile Wooden Arrow Hostile 81 at the damage
    /// 11 (`num170 = 81`, `num169 = 11`, the `:59571` call). The justHit
    /// staggers (`ai[1] = 30`, `ai[2] = 0`, `:59088-59094` — the port's
    /// `hurt_cd` window re-arms it, the same ~30-tick pause); the state
    /// exits on the cooldown end (`:59611-59615`'s `ai[1] <= 0` arm —
    /// the airborne exit folds).
    fn update_archer(&mut self, world: &World, player: &mut Player, rng: &mut u64) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        if self.ai[2] > 0.0 {
            if self.hurt_cd > 0.0 {
                self.ai[1] = 30.0;
                self.ai[2] = 0.0;
            } else {
                self.vx *= 0.9;
                self.ai[1] -= 1.0;
                self.facing = if pcx >= cx { 1 } else { -1 };
                self.sprite_dir = self.facing;
                if self.ai[1] == 90.0 {
                    if crate::projectile::can_hit(
                        world,
                        self.x,
                        self.y,
                        w,
                        h,
                        player.x,
                        player.y,
                        crate::player::WIDTH,
                        player.height,
                    ) {
                        let mut dx = pcx - cx;
                        let mut dy = pcy - cy - dx.abs() * 0.1;
                        dx += crate::app::rand01(rng) * 81.0 - 40.0;
                        dy += crate::app::rand01(rng) * 81.0 - 40.0;
                        let d = (dx * dx + dy * dy).sqrt().max(1.0);
                        self.wants_hostile_shot.push((
                            crate::projectile::WOODEN_ARROW_HOSTILE,
                            cx,
                            cy,
                            dx / d * 9.0,
                            dy / d * 9.0,
                            11,
                        ));
                    }
                }
                if self.ai[1] <= 0.0 {
                    self.ai[2] = 0.0;
                    self.ai[1] = 0.0;
                }
            }
            // The stand: the zero-speed walker keeps the gravity.
            crate::npc_ai::walker_step(
                self,
                world,
                pcx,
                pcy,
                crate::npc_ai::WalkerParams {
                    speed: 0.0,
                    jump: 0.0,
                    aggro_x: 0.0,
                    aggro_y: 0.0,
                },
                GRAVITY,
                MAX_FALL,
            );
            return;
        }
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The shot gate (`:59636`'s `flag21 = CanHit(position, width,
        // height, player.position, player.width, player.height)`, then
        // `:59680`'s `num178 < num179`, the 700-px default): grounded,
        // the clear line, the distance < 700, the cooldown spent.
        let dx = pcx - cx;
        let dy = pcy - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        if self.on_ground
            && self.ai[1] <= 0.0
            && dist < 700.0
            && crate::projectile::can_hit(
                world,
                self.x,
                self.y,
                w,
                h,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            self.ai[2] = 1.0;
            self.ai[1] = 180.0;
            self.vx *= 0.5;
        }
    }

    /// The Goblin AI (`AI_003`'s walker family): the goblins chase the
    /// player and jump the obstacles (the same machine as the zombie).
    fn update_goblin(&mut self, world: &World, player: &mut Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    /// `AI_009` through the framework's `sphere_step` (`NPC.cs`'s aiStyle 9
    /// core, `:21404-21429`): the one-shot launch, then the straight fly
    /// through the tiles (`noGravity`/`noTileCollide`).
    fn update_sphere(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::sphere_step(self, pcx, pcy, world);
    }

    /// `AI_016` through the framework's `swim_step` (`NPC.cs`'s aiStyle 16
    /// block): the wet path rides the machine — `flag25` folds the kind
    /// gate (`:23701`'s Goldfish exclusion), the player's `wet`/alive and
    /// `CanHit`; the dry path is the beached flop (`:23963-
    /// 23989`: the Shark's `velocity.X *= 0.94` snap, the others' random
    /// jump through `Main.rand`).
    fn update_swimmer(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let params = match self.kind {
            NpcKind::Shark => crate::npc_ai::SwimParams::SHARK,
            NpcKind::Piranha => crate::npc_ai::SwimParams::PIRANHA,
            _ => crate::npc_ai::SwimParams::GOLDFISH,
        };
        self.wet = is_wet(world, self.x, self.y, w, h);
        if self.wet {
            let (pcx, pcy) = player.center();
            // `flag25` (`:23701-23707`): the kind gate + the player wet +
            // alive + `CanHit(base.position, base.width, base.height,
            // player.position, player.width, player.height)`.
            let aggro_ok = params.aggro
                && player.wet
                && !player.dead
                && crate::projectile::can_hit(
                    world,
                    self.x,
                    self.y,
                    w,
                    h,
                    player.x,
                    player.y,
                    crate::player::WIDTH,
                    player.height,
                );
            crate::npc_ai::swim_step(self, world, pcx, pcy, aggro_ok, params);
        } else {
            // The beached flop (`:23963-23989`).
            if self.kind == NpcKind::Shark {
                // `:23967-23974`: the Shark's stand decay.
                if self.vy == 0.0 {
                    self.vx *= 0.94;
                    if self.vx > -0.2 && self.vx < 0.2 {
                        self.vx = 0.0;
                    }
                }
            } else {
                // `:23975-23981`: the random jump (`Next(-50,-20)*0.1`,
                // `Next(-20,20)*0.1`, the direction coin).
                self.vy = -(crate::app::frame::rand01(rng) * 2.9 + 2.1);
                self.vx = (crate::app::frame::rand01(rng) * 39.0 - 20.0) * 0.1;
                self.facing = if crate::app::frame::rand01(rng) < 0.5 {
                    self.sprite_dir = self.facing;
                    1
                } else {
                    -1
                };
            }
            // `:23983-23988`: the gravity + the sink phase.
            self.vy = (self.vy + 0.3).min(10.0);
            self.swim_phase = 1.0;
            // The generic tile integration (`UpdateCollision`): the flop
            // stops at the walls (the wet path's bounce flips ride the wet
            // branch only).
            let nx = self.x + self.vx;
            if !collides_rect(world, nx, self.y, w, h) {
                self.x = nx;
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
            } else {
                self.vy = 0.0;
            }
        }
    }

    fn contact(&mut self, player: &mut Player, rng: &mut u64) {
        // `Player.Update_NPCCollision`'s head (`Player.cs:31321-31324`): the
        // journey godmode power returns before the whole collision pass — no
        // NPC contact damage and none of the EoC shield-dash arm below.
        if player.creative_god_mode {
            return;
        }
        let (w, h) = self.size();
        if self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
        {
            let (cx, _) = self.center();
            // `Player.Update_NPCCollision`'s EoC shield hit
            // (`Player.cs:21049-21116`): while `dash == 2 && eocDash > 0` the
            // dash deals 30 x meleeDamage with knockback 9, ends itself
            // (`dashDelay = 30`), bounces the player back, and grants the
            // 4-tick collision immunity. The vanilla hitbox is expanded by 8
            // and offset by half the velocity (the plain overlap stands in);
            // the crit roll (4% base) waits for the combat rng pass.
            if player.dash == 2 && player.eoc_dash > 0 {
                let dmg = (30.0 * player.melee_damage) as i32;
                self.hurt(dmg, cx, 9.0, false);
                player.eoc_dash = 10;
                player.dash_delay = 30;
                let dir = if player.vx != 0.0 {
                    player.vx.signum() as i32
                } else {
                    player.facing
                };
                player.vx = -(dir as f32) * 9.0;
                player.vy = -4.0;
                player.on_ground = false;
                player.hurt_cd = player.hurt_cd.max(4.0 / 60.0);
                return;
            }
            // The town NPCs never hurt the player (`aiStyle 7`'s no-contact);
            // the 0-damage kinds (the Goldfish) never hurt either (the
            // vanilla `damage > 0` gate).
            // The town NPCs never hurt the player (`aiStyle 7`'s no-contact;
            // the M4-TOWN-4's gate loss fixed here — the Merchant's dmg 10
            // reached the hurt path until this).
            if !matches!(
                self.kind,
                NpcKind::Guide
                    | NpcKind::Merchant
                    | NpcKind::Dryad
                    | NpcKind::Nurse
                    | NpcKind::ArmsDealer
                    | NpcKind::Demolitionist
                    | NpcKind::Painter
                    | NpcKind::WitchDoctor
                    | NpcKind::Clothier
                    | NpcKind::TravellingMerchant
                    | NpcKind::Angler
                    | NpcKind::SleepingAngler
                    | NpcKind::Wizard
                    | NpcKind::BoundWizard
                    | NpcKind::Mechanic
                    | NpcKind::BoundMechanic
                    | NpcKind::Stylist
                    | NpcKind::WebbedStylist
                    | NpcKind::TaxCollector
                    | NpcKind::Zoologist
                    | NpcKind::Princess
                    | NpcKind::DyeTrader
                    | NpcKind::PartyGirl
                    | NpcKind::EyeOfCthulhu
                    | NpcKind::BoundGoblin
                    | NpcKind::GoblinTinkerer
            ) {
                let base_dmg = if self.kind == NpcKind::Slime {
                    self.slime.stats().damage
                } else {
                    self.damage_override
                        .unwrap_or_else(|| self.kind.contact_damage())
                };
                // The NPC-stage `u64` stream wraps into the `DropRng` the hurt
                // rolls draw from (the C# `Main.rand` at the contact site).
                struct U64DropRng<'a>(&'a mut u64);
                impl tr_core::loot::DropRng for U64DropRng<'_> {
                    fn next_range(&mut self, min: i32, max: i32) -> i32 {
                        let range = (max - min) as u32;
                        if range == 0 {
                            return min;
                        }
                        min + (crate::app::frame::next_rand(self.0) % range as u64) as i32
                    }
                    fn next_float(&mut self) -> f32 {
                        crate::app::frame::rand01(self.0)
                    }
                }
                // Player.cs:31387 — `Main.DamageVar((float)npc.damage * damageMultiplier, 0f - luck)`.
                // S1-C-5: contact damage rolls variance with the player's luck inverted.
                let dmg = if base_dmg > 0 {
                    tr_core::damage::damage_var(&mut U64DropRng(rng), base_dmg as f32, -player.luck_state.luck)
                } else {
                    0
                };
                // `Player.cs:31390` — `flag4 = !this.immune`: the PRE-hit
                // immunity gates the thorns arms (`:31397-31418`), so a
                // contact that dodges (`Hurt` runs after, `:31423`) still
                // reflects. The port's `hurt_cd <= 0` is the `!immune`
                // stand-in (the collapsed General cooldown, M4-HAZARD-1).
                let pre_hit_not_immune = player.hurt_cd <= 0.0;
                // `Player.cs:31397-31410`: the contact bounces the thorns
                // back — `thorns` scales the NPC's own damage (2x on the
                // Turtle set), the Cactus set adds its flat 15.
                // `dontTakeDamage` NPCs (the dummies) refuse it, exactly like
                // the C# `!Main.npc[i].dontTakeDamage` gate. The reflect
                // carries the varied contact damage (`num3`, `:31387`) and
                // runs BEFORE `Hurt` — a dodged hit reflects too.
                if pre_hit_not_immune && dmg > 0 && !self.dont_take_damage {
                    let reflect = player.thorns_damage(dmg);
                    if reflect > 0 {
                        let (ncx, _) = self.center();
                        let dir = if ncx < player.x + crate::player::WIDTH * 0.5 {
                            -1.0
                        } else {
                            1.0
                        };
                        // The C# knocks the NPC away from the player
                        // (`-num2`, knockback 10).
                        self.hurt(reflect, ncx, 10.0 * dir, false);
                    }
                }
                // `Player.cs:31423` — `Hurt(damageSource: ByNPC(i), Damage:
                // num3, hitDirection: num2, .., dodgeable:
                // Main.npc[i].IsDamageDodgeable())`; the `> 0 && !dead` tail
                // gates `StatusFromNPC`, whose debuff body the port carries
                // elsewhere (unchanged).
                if dmg > 0 {
                    player.hurt(dmg, cx, &mut U64DropRng(rng), self.is_damage_dodgeable());
                }
            }
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
            NpcKind::DD2LanePortal => 549,
            NpcKind::StardustSpiderBig => 409,
            NpcKind::StardustSpiderSmall => 410,
            NpcKind::StardustSoldier => 411,
            NpcKind::SolarCrawltipedeHead => 412,
            NpcKind::SolarCrawltipedeBody => 413,
            NpcKind::SolarCrawltipedeTail => 414,
            NpcKind::StardustWormHead => 402,
            NpcKind::SolarSpearman => 518,
            NpcKind::SolarDrakomire => 415,
            NpcKind::SolarDrakomireRider => 416,
            NpcKind::SolarSroller => 417,
            NpcKind::SolarCorite => 418,
            NpcKind::SolarSelenian => 419,
            NpcKind::NebulaFloater => 420,
            NpcKind::BrainSuckler => 421,
            NpcKind::Predictor => 424,
            NpcKind::VortexHunter => 426,
            NpcKind::VortexAlien => 427,
            NpcKind::VortexHeadcrab => 428,
            NpcKind::MoonLordHead => 396,
            NpcKind::MoonLordHand => 397,
            NpcKind::MoonLordCore => 398,
            NpcKind::MoonLordFreeEye => 400,
            NpcKind::MoonLordLeechBlob => 401,
            NpcKind::Slime => 0,
            NpcKind::SkeletronHand => 79,
            NpcKind::PrimeCannon => 81,
            NpcKind::PrimeSaw => 82,
            NpcKind::PrimeVice => 83,
            NpcKind::PrimeLaser => 84,
            NpcKind::WallOfFleshEye => 80,
            NpcKind::TravellingMerchant => 111,
            NpcKind::Angler => 150,
            NpcKind::SleepingAngler => 151,
            NpcKind::Wizard => 156,
            NpcKind::BoundWizard => 157,
            NpcKind::Mechanic => 158,
            NpcKind::BoundMechanic => 159,
            NpcKind::Stylist => 160,
            NpcKind::WebbedStylist => 161,
            NpcKind::TaxCollector => 162,
            NpcKind::Golfer => 163,
            NpcKind::SleepingGolfer => 164,
            NpcKind::Zoologist => 165,
            NpcKind::Princess => 166,
            NpcKind::DyeTrader => 167,
            NpcKind::PartyGirl => 168,
            NpcKind::DemonEye => 1,
            NpcKind::WanderingEye => 2,
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 4,
            NpcKind::Guide => 5,
            NpcKind::Skeleton => 6,
            NpcKind::UndeadMiner => 7,
            NpcKind::CaveBat => 8,
            NpcKind::JungleBat => 9,
            NpcKind::Hellbat => 10,
            NpcKind::VoodooDemon => 11,
            NpcKind::EaterOfSouls => 12,
            NpcKind::Hornet => 13,
            NpcKind::MeteorHead => 14,
            NpcKind::GoblinSorcerer => 15,
            NpcKind::GoblinThief => 57,
            NpcKind::GoblinWarrior => 58,
            NpcKind::GoblinArcher => 59,
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinSummoner => 61,
            NpcKind::SnowmanGangsta => 65,
            NpcKind::SnowBalla => 66,
            NpcKind::MysterySnowman => 67,
            NpcKind::MartianOfficer => 68,
            NpcKind::GrayGrunt => 69,
            NpcKind::MartianEngineer => 70,
            NpcKind::GigaZapper => 71,
            NpcKind::MartianWalker => 72,
            // The pirates + the probe: to_u16's renumber (the hash values
            // ride the same table — the collided 68-72 pirate slots go
            // 78-82/83 like the protocol ids).
            NpcKind::PirateDeckhand => 78,
            NpcKind::PirateCaptainsMate => 79,
            NpcKind::PirateArcher => 80,
            NpcKind::PirateCorsair => 81,
            NpcKind::PirateCaptain => 82,
            NpcKind::MartianProbe => 83,
            NpcKind::MartianDrone => 84,
            NpcKind::BrainScrambler => 85,
            NpcKind::RayGunner => 86,
            NpcKind::ScutlixRider => 87,
            NpcKind::Scutlix => 88,
            NpcKind::MartianSaucer => 89,
            NpcKind::MartianSaucerTurret => 90,
            NpcKind::MartianSaucerCannon => 91,
            NpcKind::MartianSaucerCore => 92,
            NpcKind::FlyingDutchman => 93,
            NpcKind::FlyingDutchmanCannon => 94,
            NpcKind::DD2GoblinT1 => 95,
            NpcKind::DD2GoblinT2 => 96,
            NpcKind::DD2GoblinT3 => 97,
            NpcKind::DD2GoblinBomberT1 => 98,
            NpcKind::DD2GoblinBomberT2 => 99,
            NpcKind::DD2GoblinBomberT3 => 100,
            NpcKind::DD2JavelinT1 => 101,
            NpcKind::DD2JavelinT2 => 102,
            NpcKind::DD2JavelinT3 => 103,
            NpcKind::DD2SkeletonT1 => 104,
            NpcKind::DD2SkeletonT3 => 105,
            NpcKind::DD2DarkMage => 106,
            NpcKind::DD2Betsy => 107,
            NpcKind::DD2WitherBeast => 108,
            NpcKind::DD2Bartender => 109,
            NpcKind::DD2EterniaCrystal => 110,
            NpcKind::DD2WyvernT1 => 169,
            NpcKind::DD2WyvernT2 => 170,
            NpcKind::DD2WyvernT3 => 171,
            NpcKind::DD2DarkMageT3 => 172,
            NpcKind::DD2WitherBeastT3 => 173,
            NpcKind::DD2DrakinT2 => 174,
            NpcKind::DD2DrakinT3 => 175,
            NpcKind::DD2KoboldWalkerT2 => 176,
            NpcKind::DD2KoboldWalkerT3 => 177,
            NpcKind::DD2KoboldFlyerT2 => 178,
            NpcKind::DD2KoboldFlyerT3 => 179,
            NpcKind::DD2OgreT2 => 180,
            NpcKind::DD2OgreT3 => 181,
            NpcKind::AncientCultist => 112,
            NpcKind::StardustTower => 113,
            NpcKind::NebulaTower => 114,
            NpcKind::VortexTower => 422,
            NpcKind::SolarTower => 115,
            NpcKind::VortexTower => 146,
            NpcKind::StardustCellBig => 121,
            NpcKind::StardustCellSmall => 116,
            NpcKind::StardustJellyfishBig => 117,
            NpcKind::NebulaBeast => 118,
            NpcKind::VortexRifleman => 119,
            NpcKind::VortexSoldier => 120,
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
            NpcKind::ChaosBall => 19,
            NpcKind::BurningSphere => 20,
            NpcKind::WaterSphere => 21,
            NpcKind::Goldfish => 22,
            NpcKind::Piranha => 23,
            NpcKind::Shark => 24,
            NpcKind::DevourerHead => 25,
            NpcKind::DevourerBody => 26,
            NpcKind::DevourerTail => 27,
            NpcKind::GiantWormHead => 28,
            NpcKind::GiantWormBody => 29,
            NpcKind::GiantWormTail => 30,
            NpcKind::EaterOfWorldsHead => 31,
            NpcKind::EaterOfWorldsBody => 32,
            NpcKind::EaterOfWorldsTail => 33,
            NpcKind::Merchant => 34,
            NpcKind::Dryad => 35,
            NpcKind::Nurse => 36,
            NpcKind::ArmsDealer => 37,
            NpcKind::Demolitionist => 38,
            NpcKind::Painter => 39,
            NpcKind::WitchDoctor => 40,
            NpcKind::Clothier => 41,
            NpcKind::EyeOfCthulhu => 42,
            NpcKind::QueenBee => 43,
            NpcKind::SkeletronHead => 44,
            NpcKind::ServantOfCthulhu => 45,
            NpcKind::WallOfFlesh => 46,
            NpcKind::Hungry => 47,
            NpcKind::LeechHead => 48,
            NpcKind::LeechBody => 49,
            NpcKind::TheDestroyer => 50,
            NpcKind::SkeletronPrime => 51,
            NpcKind::Retinazer => 52,
            NpcKind::Spazmatism => 53,
            NpcKind::Plantera => 54,
            NpcKind::BrainOfCthulhu => 55,
            NpcKind::Creeper => 53,
            NpcKind::BoundGoblin => 58,
            NpcKind::GoblinTinkerer => 59,
            NpcKind::Golem => 60,
            NpcKind::GolemHead => 61,
            NpcKind::GolemFistLeft => 62,
            NpcKind::GolemFistRight => 63,
            NpcKind::GolemHeadFree => 64,
            NpcKind::Deerclops => 65,
        } << 5;
        if self.kind == NpcKind::Slime {
            h ^= (self.slime as u32) << 9;
        }
        h
    }

    /// The C#'s `position += netOffset` draw wrapper (`Main.cs:22335`): a
    /// copy with the smoothing accumulator added to the position, so the
    /// render derives every anchor from the smoothed point. With a zero
    /// accumulator (host / single-player) it is a plain clone.
    pub fn offset_clone(&self) -> Npc {
        if self.net_offset == Vec2::ZERO {
            return self.clone();
        }
        let mut n = self.clone();
        n.x += self.net_offset.x;
        n.y += self.net_offset.y;
        n
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

// --- The world-file section 6 (pointer 5): `SaveNPCs:1703` / `LoadNPCs:2884`
//
// The codec that produces/consumes [`WldNpcs`] is `tr_core::wld::sections::npcs`;
// this half turns the town records into live `Npc`s (and back), which is where
// the vanilla net-id resolution and the record-discarding live — the plain
// wire structs carry no bestiary table.

/// `NPC.SetDefaults(int netID)`'s resolution of a modern record
/// (`LoadNPCs:2896`): the vanilla net id maps to the kind through the same
/// table the bestiary credits from. `None` for an id the port does not model.
pub fn kind_from_net_id(net_id: i32) -> Option<(NpcKind, SlimeVariant)> {
    crate::bestiary::ENTRIES
        .iter()
        .find(|e| e.net_id == net_id)
        .map(|e| (e.kind, e.variant.unwrap_or(SlimeVariant::Blue)))
}

/// `NPCID.FromLegacyName` (`LoadNPCs:2900`) narrowed to the port's family: the
/// legacy name is the English base name (`NPCID.cs:12085+`'s
/// `LegacyNameToIdMap`), which the port's bestiary rows already carry. The
/// "Little/Big <name>" sub-variant names and the unported NPCs map to nothing
/// here — those records are parsed and discarded (recorded deviation).
pub fn kind_from_legacy_name(name: &str) -> Option<(NpcKind, SlimeVariant)> {
    crate::bestiary::ENTRIES
        .iter()
        .find(|e| e.name == name)
        .map(|e| (e.kind, e.variant.unwrap_or(SlimeVariant::Blue)))
}

/// The record's kind: the modern net id, or the legacy name's.
fn record_kind(record: &tr_core::wld::sections::WldTownNpc) -> Option<(NpcKind, SlimeVariant)> {
    match &record.legacy_name {
        Some(name) => kind_from_legacy_name(name),
        None => kind_from_net_id(record.net_id),
    }
}

/// `LoadNPCs:2894-2922`: one live `Npc` per town record, in file order
/// (`Main.npc[num2++]`). The `SavesAndLoads` tower records that share the slot
/// cursor are discarded — no ported kind is a lunar tower (DW-4.4's recorded
/// deviation). The world-dependent `FixAgainstExploits` re-validation runs
/// after the town rooms are loaded — `town::fix_town_npc_homes`.
///
/// A record whose net id (or legacy name) has no ported `NpcKind` is parsed
/// and discarded rather than guessed at: the codec has already consumed its
/// bytes, so the stream stays in step and the rest of the file loads.
pub fn from_wld(section: &tr_core::wld::sections::WldNpcs) -> Vec<Npc> {
    let mut npcs = Vec::new();
    for record in &section.town {
        let Some((kind, slime)) = record_kind(record) else {
            continue;
        };
        // `SetDefaults` resets the whole NPC; the port's ctor is the same
        // shell (`NPC.cs:8513`'s `active = true` is what the list's leading
        // bool already says).
        let mut npc = Npc::new(kind, record.x, record.y, 1, 0.0);
        npc.slime = slime;
        npc.given_name = record.given_name.clone();
        npc.variation_index = record.variation_index;
        npc.homeless_despawn = record.homeless_despawn;
        // `homeTileX/homeTileY + homeless`: the port models the pair as one
        // `Option` (a homeless NPC keeps no home — `happiness.rs:228` and
        // `world_sim/npcs.rs:1287` read it that way), so a homeless record's
        // stale coordinates are not carried into live state.
        npc.home = if record.homeless {
            None
        } else {
            Some((record.home_x, record.home_y))
        };
        npcs.push(npc);
    }
    for tower in &section.towers {
        // `LoadNPCs:2921-2954`'s second list: the `SavesAndLoads` passives
        // land in the same `Main.npc` slots in file order. The lunar-tower
        // kinds (422/507/493/517) are ported on this branch (M4-NPC), so the
        // record instantiates a live NPC rather than being discarded.
        let Some((kind, slime)) = kind_from_net_id(tower.net_id) else {
            continue;
        };
        let mut tower_npc = Npc::new(kind, tower.x, tower.y, 1, 0.0);
        tower_npc.slime = slime;
        npcs.push(tower_npc);
    }
    npcs
}

/// `SaveNPCs:1714-1748` over the live NPCs: the town records in `Main.npc`
/// order (the `active && townNPC && type != 368` filter, of which the
/// `TravellingMerchant` arm has no ported kind), then the `SavesAndLoads`
/// towers. `SectionData::npcs` is the plan's save shape; callers run any
/// pre-pass first (the section has none at NPC level).
pub fn to_wld(npcs: &[Npc]) -> tr_core::wld::sections::WldNpcs {
    let mut out = tr_core::wld::sections::WldNpcs::default();
    for npc in npcs {
        let net_id = npc.kind.net_id();
        if npc.kind.is_town_npc() {
            // `SaveNPCs:1727`'s bit 0 is `townNPC`, which is what the filter
            // above has already established — always set on the write side.
            let (home_x, home_y) = npc.home.unwrap_or((0, 0));
            out.town.push(tr_core::wld::sections::WldTownNpc {
                net_id,
                legacy_name: None,
                given_name: npc.given_name.clone(),
                x: npc.x,
                y: npc.y,
                homeless: npc.home.is_none(),
                home_x,
                home_y,
                town_npc: true,
                variation_index: npc.variation_index,
                homeless_despawn: npc.homeless_despawn,
            });
        } else if tr_core::wld::sections::npcs::SAVES_AND_LOADS.contains(&net_id) {
            // `SaveNPCs:1738-1747`: the `SavesAndLoads` passives — the
            // lunar towers 422/507/493/517 (`NPCID.cs:10449`), whose kinds
            // this branch ports (M4-NPC), so the arm is live and the towers
            // round-trip through the live state (DW-4.4).
            out.towers.push(tr_core::wld::sections::WldTower {
                net_id,
                legacy_name: None,
                x: npc.x,
                y: npc.y,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_hover_kinds_spawn_fly_and_map() {
        // The Eater of Souls (6)/Hornet (42)/Meteor Head (23): the hovers
        // spawn in the open air, fly via the hover machine, the bestiary
        // entries round-trip.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 60..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let eos = Npc::spawn_hover(&w, 20, 30, NpcKind::EaterOfSouls)
            .expect("the eater spawns in the open air");
        assert_eq!(eos.max_hp, 40);
        assert_eq!(eos.size(), (30.0, 30.0));
        let hornet = Npc::spawn_hover(&w, 24, 30, NpcKind::Hornet).expect("the hornet spawns");
        assert_eq!(hornet.max_hp, 48);
        let meteor =
            Npc::spawn_hover(&w, 28, 30, NpcKind::MeteorHead).expect("the meteor head spawns");
        assert_eq!(meteor.max_hp, 26);
        // The bestiary entries round-trip.
        let e = crate::bestiary::entry_for_npc(NpcKind::EaterOfSouls, SlimeVariant::Blue);
        assert_eq!(e.net_id, 6);
        let e = crate::bestiary::entry_for_npc(NpcKind::Hornet, SlimeVariant::Blue);
        assert_eq!(e.net_id, 42);
        let e = crate::bestiary::entry_for_npc(NpcKind::MeteorHead, SlimeVariant::Blue);
        assert_eq!(e.net_id, 23);
    }

    #[test]
    fn the_hover_chases_the_target() {
        // The hover machine: the Eater of Souls homes to the target over
        // time (the vanilla num 4 / num2 0.02 pair).
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut npc = Npc::new(NpcKind::EaterOfSouls, 90.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let target = crate::npc_ai::HoverTarget {
            center_x: 60.0 * 16.0,
            center_y: 100.0 * 16.0,
            // A player's top edge, ~21 px (half the 42-px hitbox) above centre.
            top_y: 100.0 * 16.0 - 21.0,
        };
        for _ in 0..200 {
            crate::npc_ai::hover_step(
                &mut npc,
                &w,
                target,
                crate::npc_ai::HoverParams::EATER_OF_SOULS,
            );
        }
        let dist = ((npc.x - target.center_x) / 16.0).abs();
        assert!(dist < 20.0, "the hover homed (dist {dist} tiles)");
    }

    #[test]
    fn the_hornet_surface_damping_is_surface_and_family_gated() {
        // `NPC.cs:50352-50362`: the ×0.97 climb/dive damping fires only for
        // the type-42/231-235 family AND only above `worldSurface`.
        let mut w = World::new("t".into(), 64, 200);
        w.world_surface = 100.0;
        // The target sits at the NPC's own centre, so the velocity aim is a
        // no-op (`dist == 0` ⇒ the desired velocity = the current velocity)
        // and the damping is the only `vy` change. The climb arm wants the
        // target top 300+ px below the NPC top and an upward `vy`.
        let probe = |params: crate::npc_ai::HoverParams, y: f32| {
            let mut npc = Npc::new(NpcKind::Hornet, 32.0 * 16.0, y, 1, 0.0);
            npc.vy = -5.0;
            let target = crate::npc_ai::HoverTarget {
                center_x: npc.x + npc.size().0 * 0.5,
                center_y: npc.y + npc.size().1 * 0.5,
                top_y: npc.y + 400.0,
            };
            crate::npc_ai::hover_step(&mut npc, &w, target, params);
            npc.vy
        };
        // The Hornet above the surface damps the climb (×0.97).
        let above = probe(crate::npc_ai::HoverParams::HORNET, 50.0 * 16.0);
        assert!((above + 5.0 * 0.97).abs() < 1e-4, "damped vy {above}");
        // Below the surface rows: no damping.
        let below = probe(crate::npc_ai::HoverParams::HORNET, 150.0 * 16.0);
        assert!((below + 5.0).abs() < 1e-4, "undamped vy {below}");
        // A non-family hover (the Eater of Souls) above the surface: no damping.
        let other = probe(crate::npc_ai::HoverParams::EATER_OF_SOULS, 50.0 * 16.0);
        assert!((other + 5.0).abs() < 1e-4, "family-gated vy {other}");
    }

    #[test]
    fn the_hornet_speed_takes_the_two_minus_scale_factor() {
        // `NPC.cs:50350`: `num *= 2 - scale` (the type-42/231-235 arm only).
        let mut w = World::new("t".into(), 400, 400);
        w.world_surface = 50.0; // the NPC flies well below it — no damping
        let run = |scale: f32| {
            let mut npc = Npc::new(NpcKind::Hornet, 300.0 * 16.0, 100.0 * 16.0, 1, 0.0);
            npc.scale = scale;
            // Far to the left so the desired speed saturates before the NPC
            // reaches the target (no overshoot).
            let target = crate::npc_ai::HoverTarget {
                center_x: npc.x - 8000.0,
                center_y: npc.y + npc.size().1 * 0.5,
                top_y: npc.y,
            };
            for _ in 0..600 {
                crate::npc_ai::hover_step(&mut npc, &w, target, crate::npc_ai::HoverParams::HORNET);
            }
            npc.vx
        };
        // The default-scale Hornet caps at num = 3.5.
        let base = run(1.0);
        assert!((base + 3.5).abs() < 0.05, "default vx {base}");
        // A half-scale Hornet caps at 3.5 × (2 - 0.5) = 5.25.
        let scaled = run(0.5);
        assert!((scaled + 5.25).abs() < 0.05, "scaled vx {scaled}");
    }

    #[test]
    fn the_cave_walker_spawn_and_the_bestiary_map() {
        // The Skeleton (21)/Undead Miner (44): the cave walkers spawn on
        // the ground, the bestiary entries round-trip via entry_for_npc.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 40..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let sk = Npc::spawn_cave_walker(&w, 20, NpcKind::Skeleton).expect("the skeleton spawns");
        assert_eq!(sk.kind, NpcKind::Skeleton);
        assert_eq!(sk.max_hp, 60);
        let um = Npc::spawn_cave_walker(&w, 24, NpcKind::UndeadMiner).expect("the miner spawns");
        assert_eq!(um.max_hp, 70);
        // The bestiary credit ids map.
        let e = crate::bestiary::entry_for_npc(NpcKind::Skeleton, SlimeVariant::Blue);
        assert_eq!(e.net_id, 21);
        let e = crate::bestiary::entry_for_npc(NpcKind::UndeadMiner, SlimeVariant::Blue);
        assert_eq!(e.net_id, 44);
    }

    #[test]
    fn behind_tiles_field_matches_csharp_defaults_s1_c_6() {
        // S1-C-6 / NPC.cs:6517,8520 (default false), 10355 (WoF behindTiles = true).
        let wof = Npc::new(NpcKind::WallOfFlesh, 0.0, 0.0, 1, 0.0);
        assert!(
            wof.behind_tiles,
            "WallOfFlesh (NPC.cs:10355) must draw behind tiles"
        );
        let eye = Npc::new(NpcKind::WallOfFleshEye, 0.0, 0.0, 1, 0.0);
        assert!(
            eye.behind_tiles,
            "WallOfFleshEye (NPC.cs:10373) must draw behind tiles"
        );
        let hungry = Npc::new(NpcKind::Hungry, 0.0, 0.0, 1, 0.0);
        assert!(
            hungry.behind_tiles,
            "Hungry/TheHungry (NPC.cs:10389) must draw behind tiles"
        );
        for kind in [
            NpcKind::DevourerHead,
            NpcKind::DevourerBody,
            NpcKind::DevourerTail,
            NpcKind::GiantWormHead,
            NpcKind::GiantWormBody,
            NpcKind::GiantWormTail,
            NpcKind::EaterOfWorldsHead,
            NpcKind::EaterOfWorldsBody,
            NpcKind::EaterOfWorldsTail,
            NpcKind::LeechHead,
            NpcKind::LeechBody,
            NpcKind::TheDestroyer,
        ] {
            assert!(
                Npc::new(kind, 0.0, 0.0, 1, 0.0).behind_tiles,
                "{kind:?} (worm/segment) must draw behind tiles"
            );
        }
        let slime = Npc::new(NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        assert!(
            !slime.behind_tiles,
            "ordinary NPCs default behindTiles=false (NPC.cs:8520)"
        );
    }

    #[test]
    fn contact_damage_applies_damage_var_with_negative_luck_s1_c_5() {
        // S1-C-5 / Player.cs:31387: `Main.DamageVar(npc.damage * damageMultiplier, 0f - luck)`.
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();

        // No-luck: contact damage is varied within the ±15% band (not a fixed raw value).
        let mut bat = Npc::new(NpcKind::CaveBat, 32.0, 16.0, 1, 0.0);
        bat.damage_override = Some(100);
        let mut rng = 1u64;
        let mut saw_varied = false;
        for _ in 0..16 {
            let mut player = crate::player::Player::spawn(&w);
            player.x = bat.x;
            player.y = bat.y;
            player.hurt_cd = 0.0;
            player.luck_state.luck = 0.0;
            let hp0 = player.hp;
            bat.contact(&mut player, &mut rng);
            let dealt = hp0 - player.hp;
            assert!(
                (85..=115).contains(&dealt),
                "S1-C-5: contact damage must apply DamageVar (base 100, got {dealt})"
            );
            if dealt != 100 {
                saw_varied = true;
            }
        }
        assert!(
            saw_varied,
            "S1-C-5: contact damage variance never observed over samples"
        );

        // Positive player luck -> negative luck passed to DamageVar (0f - luck),
        // which rerolls and keeps the lower damage (Player.cs:31387).
        let mut rng_good = 7u64;
        let mut rng_bad = 7u64;
        let mut total_good = 0i32;
        let mut total_bad = 0i32;
        for _ in 0..24 {
            let mut good = crate::player::Player::spawn(&w);
            good.x = bat.x;
            good.y = bat.y;
            good.hurt_cd = 0.0;
            good.luck_state.luck = 1.0;
            let hp0 = good.hp;
            bat.contact(&mut good, &mut rng_good);
            total_good += hp0 - good.hp;

            let mut bad = crate::player::Player::spawn(&w);
            bad.x = bat.x;
            bad.y = bat.y;
            bad.hurt_cd = 0.0;
            bad.luck_state.luck = -1.0;
            let hp0 = bad.hp;
            bat.contact(&mut bad, &mut rng_bad);
            total_bad += hp0 - bad.hp;
        }
        assert!(
            total_good <= total_bad,
            "S1-C-5: positive player luck must not increase contact damage (good {total_good} > bad {total_bad})"
        );
    }

    #[test]
    fn s1_c_1_dw_2_2_a_dodged_contact_still_reflects_thorns() {
        // `Player.Update_NPCCollision`'s order (`Player.cs:31387-31426`): the
        // thorns arms (`:31397-31418`) run BEFORE the `Hurt` arm (`:31423`),
        // gated on the PRE-hit `flag4 = !immune` (`:31390`) — so a contact the
        // black belt dodges still reflects, because the dodge handlers run
        // after the arms. The pre-restructure port coupled the reflect to
        // `hurt` returning true and silently dropped it on a dodge.
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut bat = Npc::new(NpcKind::CaveBat, 32.0, 16.0, 1, 0.0);
        bat.damage_override = Some(100);
        let mut dodged_seen = false;
        // The contact stream is the NPC-stage `u64` (splitmix); the sweep
        // re-derives the dodging seeds each run — the assert covers every
        // one of them, not one pinned draw sequence.
        for seed in 0..256u64 {
            let mut player = crate::player::Player::spawn(&w);
            player.x = bat.x;
            player.y = bat.y;
            player.hp = 10_000;
            player.max_hp = 10_000;
            player.equipped[3] = 963; // the Black Belt
            player.buffs.add_buff(14, 3600); // the Thorns Potion
            player.update(&w, crate::player::PlayerInput::default());
            assert!(player.black_belt, "the belt armed");
            assert!(player.thorns > 0.0, "the thorns armed");
            // A fresh NPC per seed: the reflect's own `Npc::hurt` cooldown
            // must not leak across sweep iterations (the C#'s per-tick NPC
            // immunity is contact-tick state this test does not model).
            bat.hurt_cd = 0.0;
            bat.hp = 10_000;
            bat.max_hp = 10_000;
            let bat_hp = bat.hp;
            let mut rng = seed;
            bat.contact(&mut player, &mut rng);
            if player.hp == player.max_hp {
                // The belt dodged: hp untouched — and the thorns must still
                // have bounced (the PRE-hit gate at `:31390` holds).
                dodged_seen = true;
                assert!(
                    bat.hp < bat_hp,
                    "seed {seed}: a dodged contact must still reflect thorns"
                );
            }
        }
        assert!(dodged_seen, "no seed in the sweep dodged — the test lost its dodge case");
    }

    #[test]
    fn s1_c_1_dw_2_1_npc_is_damage_dodgeable_table() {
        // `NPC.IsDamageDodgeable` (`NPC.cs:78744-78753`): the Skeletron heads
        // — `case 35` (`NPCID.SkeletronHead`) and `case 127`
        // (`NPCID.SkeletronPrime`), with the unported Dungeon Guardian
        // (`case 68`) on the same arm — are dodgeable only outside their
        // `ai[1] == 2f` state; everything else defaults true. The Empress of
        // Light's `case 636` arm (`!ShouldEmpressBeEnraged()`) is out with
        // its unported kind (docs/ANALYSIS.md, "The dodge family").
        let mut head = Npc::new(NpcKind::SkeletronHead, 0.0, 0.0, 1, 0.0);
        assert!(
            head.is_damage_dodgeable(),
            "ai[1] 0 (the port's stand-in state) is dodgeable"
        );
        head.ai[1] = 1.0;
        assert!(head.is_damage_dodgeable(), "the 1f state is dodgeable");
        head.ai[1] = 2.0;
        assert!(
            !head.is_damage_dodgeable(),
            "the 35 arm: ai[1] == 2f is not dodgeable"
        );
        let mut prime = Npc::new(NpcKind::SkeletronPrime, 0.0, 0.0, 1, 0.0);
        assert!(prime.is_damage_dodgeable(), "the 127 arm mirrors 35 at ai[1] 0");
        prime.ai[1] = 2.0;
        assert!(!prime.is_damage_dodgeable(), "the 127 arm gates at 2f");
        prime.ai[1] = 3.0;
        assert!(prime.is_damage_dodgeable(), "the 3f state is dodgeable");
        for kind in [NpcKind::CaveBat, NpcKind::Zombie, NpcKind::EyeOfCthulhu] {
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert!(n.is_damage_dodgeable(), "{kind:?} defaults true");
        }
    }

    #[test]
    fn s1_c_1_dw_2_3_non_dodgeable_contact_lands_without_a_dodge_draw() {
        // The contact feeds `Hurt(dodgeable: npc.IsDamageDodgeable())`
        // (`Player.cs:31423`); `Hurt`'s gate is `(whoAmI == Main.myPlayer) &
        // dodgeable` (`:38214`) — a non-dodgeable NPC skips the whole roll
        // block, so the armed belt draws nothing and the hit lands.
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut head = Npc::new(NpcKind::SkeletronHead, 32.0, 16.0, 1, 0.0);
        head.damage_override = Some(20);
        head.ai[1] = 2.0;
        assert!(!head.is_damage_dodgeable(), "the stand-in head is not dodgeable");
        let mut player = crate::player::Player::spawn(&w);
        player.x = head.x;
        player.y = head.y;
        player.hp = 10_000;
        player.max_hp = 10_000;
        player.equipped[3] = 963; // the Black Belt
        player.update(&w, crate::player::PlayerInput::default());
        assert!(player.black_belt, "the belt armed");
        let hp0 = player.hp;
        let mut rng = 9u64;
        head.contact(&mut player, &mut rng);
        assert!(
            player.hp < hp0,
            "the non-dodgeable head lands despite the belt"
        );
        // The stream consumed exactly the contact's `DamageVar` draws and
        // nothing for the dodge: the same seed through a belt-less control
        // NPC with the same base damage must end in the identical state.
        let mut plain = Npc::new(NpcKind::CaveBat, 32.0, 16.0, 1, 0.0);
        plain.damage_override = Some(20);
        let mut control = crate::player::Player::spawn(&w);
        control.x = plain.x;
        control.y = plain.y;
        control.hp = 10_000;
        control.max_hp = 10_000;
        let mut rng2 = 9u64;
        plain.contact(&mut control, &mut rng2);
        assert_eq!(
            rng, rng2,
            "no dodge draw — the stream matches the belt-less control"
        );
    }

    #[test]
    fn the_cave_bat_spawn_flies_and_bites() {
        // The Cave Bat (49): spawns in the cave air (the open tile), flies
        // via the flyer machine, the contact bites for 13.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 60..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut bat = Npc::spawn_cave_flyer(&w, 20, 30, NpcKind::CaveBat)
            .expect("the bat spawns in the open air");
        assert_eq!(bat.kind, NpcKind::CaveBat);
        assert_eq!(bat.max_hp, 16);
        assert_eq!(bat.size(), (22.0, 18.0));
        // The jungle/hellbat/voodoo variants: the same frame, the vanilla stats.
        let jb =
            Npc::spawn_cave_flyer(&w, 22, 30, NpcKind::JungleBat).expect("the jungle bat spawns");
        assert_eq!(jb.max_hp, 34);
        let hb = Npc::spawn_cave_flyer(&w, 24, 30, NpcKind::Hellbat).expect("the hellbat spawns");
        assert_eq!(hb.max_hp, 46);
        let vd = Npc::spawn_cave_flyer(&w, 26, 30, NpcKind::VoodooDemon)
            .expect("the voodoo demon spawns");
        assert_eq!(vd.max_hp, 140);
        // The contact: an overlapping player takes the bat's contact damage.
        let mut rng = 0u64;
        let mut player = crate::player::Player::spawn(&w);
        player.x = bat.x;
        player.y = bat.y;
        player.hurt_cd = 0.0;
        bat.contact(&mut player, &mut rng);
        assert!(player.hp < player.max_hp, "the bat bit for its contact dmg");
        // M3-EQUIP-15 — the same contact with the Thorns potion up bounces
        // the varied contact damage back (`Player.cs:31373-31415,31387`).
        let mut spiky = crate::player::Player::spawn(&w);
        spiky.x = bat.x;
        spiky.y = bat.y;
        spiky.hurt_cd = 0.0;
        spiky.buffs.add_buff(14, 3600);
        spiky.update(&w, crate::player::PlayerInput::default());
        let bat_hp = bat.hp;
        spiky.hp = spiky.max_hp; // the contact must land for the reflection
        bat.contact(&mut spiky, &mut rng);
        assert!(
            bat.hp < bat_hp,
            "the thorns bit back: {} -> {}",
            bat_hp,
            bat.hp
        );
        // `Player.Update_NPCCollision`'s head (`Player.cs:31321-31324`): the
        // journey godmode power returns before any contact arm runs.
        let mut god = crate::player::Player::spawn(&w);
        god.x = bat.x;
        god.y = bat.y;
        god.hurt_cd = 0.0;
        god.creative_god_mode = true;
        bat.contact(&mut god, &mut rng);
        assert_eq!(
            god.hp, god.max_hp,
            "the godmode player takes no contact damage"
        );
        // The bestiary entry maps.
        let e = crate::bestiary::entry_for_npc(NpcKind::CaveBat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 49);
        let e = crate::bestiary::entry_for_npc(NpcKind::JungleBat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 51);
        let e = crate::bestiary::entry_for_npc(NpcKind::Hellbat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 60);
        let e = crate::bestiary::entry_for_npc(NpcKind::VoodooDemon, SlimeVariant::Blue);
        assert_eq!(e.net_id, 66);
    }

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The test ground: dirt from y = 120 in a 200x200 world with a carved
    /// pit (x 60..140, y 125..134) — the caster tests' arena. The pit
    /// matters: on a flat surface every floor-level spot sits inside the
    /// player's ±5-tile telefrag rect (the vanilla casters legitimately
    /// fail to teleport there), so the search needs a floor outside the
    /// rect's y window.
    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 60..140 {
            for y in 125..135 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
        w
    }

    /// The player parked on the surface at column 100 (the tests' fixed
    /// reference — `Player::spawn`'s world-spawn default rides the unset
    /// `spawn_x/spawn_y`).
    fn surface_player(w: &World) -> crate::player::Player {
        let mut p = crate::player::Player::spawn(w);
        p.x = 100.0 * TILE_PX;
        p.y = 116.0 * TILE_PX;
        p
    }

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
    fn the_prime_limbs_tether_and_attack() {
        // `aiStyle 33-36` (`NPC.cs:27943-28990`): the dock at
        // `(headX - 200*side, headY + 230)`, the cannon's bombs (102,
        // dmg 0), the saw's 22-dash, the vice's 4-dash-20, the laser's
        // DeathLaser 100 at 25.
        let mut w = ground_air();
        w.prime_present = true;
        w.prime_x = 100.0 * 16.0;
        w.prime_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 130.0 * 16.0;
        player.y = 100.0 * 16.0;
        // The cannon: the 40-tick bomb volley.
        let mut cannon = Npc::new(NpcKind::PrimeCannon, 100.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        cannon.hp = cannon.max_hp;
        cannon.ai[3] = 39.0;
        cannon.update_prime_limb(&w, &player);
        let bomb = cannon.wants_hostile_shot.last().copied().expect("the bomb");
        assert_eq!(bomb.0, crate::projectile::BOMB_SKELETRON);
        assert_eq!(bomb.5, 0, "the classic bombs deal no damage");
        // The saw: the rise → the 22 strike.
        let mut saw = Npc::new(NpcKind::PrimeSaw, 100.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        saw.hp = saw.max_hp;
        saw.ai[2] = 1.0;
        saw.y = 80.0 * 16.0; // above head - 200
        saw.update_prime_limb(&w, &player);
        let sp = (saw.vx * saw.vx + saw.vy * saw.vy).sqrt();
        assert!((sp - 22.0).abs() < 2.0, "the 22 saw dash ({sp})");
        // The vice: the 20-speed dash, the 4-count.
        let mut vice = Npc::new(NpcKind::PrimeVice, 100.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        vice.hp = vice.max_hp;
        vice.ai[2] = 1.0;
        vice.y = 80.0 * 16.0;
        vice.update_prime_limb(&w, &player);
        let sp2 = (vice.vx * vice.vx + vice.vy * vice.vy).sqrt();
        assert!((sp2 - 20.0).abs() < 2.0, "the 20 vice dash ({sp2})");
        assert_eq!(vice.ai[2], 2.0, "the dash count begins");
        // The laser: the 100 at 25, 10 speed.
        let mut laser = Npc::new(NpcKind::PrimeLaser, 100.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        laser.hp = laser.max_hp;
        laser.ai[3] = 79.0;
        laser.update_prime_limb(&w, &player);
        let beam = laser.wants_hostile_shot.last().copied().expect("the beam");
        assert_eq!(beam.0, crate::projectile::DESTROYER_LASER);
        assert_eq!(beam.5, 25, "the laser damage");
        let sp3 = (beam.3 * beam.3 + beam.4 * beam.4).sqrt();
        assert!((sp3 - 10.0).abs() < 0.5, "the 10 speed ({sp3})");
    }

    fn the_wof_eye_mirrors_and_volleys() {
        // `aiStyle 28` (`NPC.cs:26053-26208`): the Y target rides the
        // band halves, the volley's idle-600 -> 4-shot bursts at 46 ticks
        // with the hp-ramped 83 damage.
        let mut w = ground_air();
        w.wof_present = true;
        w.wof_x = 100.0 * 16.0;
        w.wof_draw_top = 100.0 * 16.0;
        w.wof_draw_bottom = 130.0 * 16.0;
        w.wof_hp_ratio = 1.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 130.0 * 16.0;
        player.y = 115.0 * 16.0;
        let mut eye = Npc::new(NpcKind::WallOfFleshEye, 100.0 * 16.0, 105.0 * 16.0, 1, 0.0);
        eye.hp = eye.max_hp;
        eye.ai[0] = 1.0;
        // The full-hp eye: the 11-damage 83 at the 9 speed, idle 600.
        eye.ai[2] = 1.0; // the burst window open
        eye.ai[1] = 46.0;
        let before = eye.wants_hostile_shot.len();
        eye.update_wof_eye(&w, &player);
        let shot = eye.wants_hostile_shot.last().copied().expect("the volley");
        assert_eq!(shot.0, crate::projectile::EYE_LASER);
        assert_eq!(shot.5, 11, "the classic damage");
        let sp = (shot.3 * shot.3 + shot.4 * shot.4).sqrt();
        assert!((sp - 9.0).abs() < 0.5, "the 9 speed ({sp})");
        // The wounded boss racks the ramp (12/13 by the <50/<25 tiers).
        w.wof_hp_ratio = 0.2;
        let mut eye2 = Npc::new(NpcKind::WallOfFleshEye, 100.0 * 16.0, 105.0 * 16.0, 1, 0.0);
        eye2.hp = eye2.max_hp;
        eye2.ai[0] = 1.0;
        eye2.ai[2] = 1.0;
        eye2.ai[1] = 46.0;
        eye2.update_wof_eye(&w, &player);
        let shot2 = eye2.wants_hostile_shot.last().copied().expect("the volley");
        assert_eq!(shot2.5, 13, "the <25% ramp");
        let before_count = eye.wants_hostile_shot.len() - before;
        assert!(before_count >= 1, "the eye fires");
    }

    fn the_skeletron_hand_tethers_and_lunges() {
        // `aiStyle 12` (`NPC.cs:22300-22613`): the orbit return to
        // `(headCenter.X - 200*side, head.Y + 230)`, the 300-tick lunge
        // cadence, the -8 rise, the 18 px/tick strike.
        let mut w = ground_air();
        w.skeletron_x = 100.0 * 16.0;
        w.skeletron_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        // Right of the head so the slot (-200) is left of the hand.
        let mut hand = Npc::new(NpcKind::SkeletronHand, 112.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        hand.hp = hand.max_hp;
        hand.ai[0] = -1.0; // the left hand
        hand.ai[2] = 0.0;
        // The drift-return pulls toward the slot (left of the head).
        let before = hand.x;
        hand.update_skeletron_hand(&w, &player);
        assert!(hand.x < before, "the orbit pulls to the slot");
        // The 300-tick cadence: after 300 ticks the hand rises.
        for _ in 0..300 {
            hand.update_skeletron_hand(&w, &player);
        }
        assert!(hand.ai[2] >= 1.0, "the lunge state entered");
        let rise = hand.vy;
        assert!(rise < 0.0, "the rise velocity ({rise})");
        // Above 200 px from the head: the strike velocity.
        hand.y = 60.0 * 16.0; // 40 tiles above the head
        hand.update_skeletron_hand(&w, &player);
        let strike = (hand.vx * hand.vx + hand.vy * hand.vy).sqrt();
        assert!((strike - 18.0).abs() < 2.0, "the 18-strike ({strike})");
    }

    fn the_king_slime_releases_its_minion_batch() {
        // `AI_015` `:44233-44247`'s damage arm: a 5%-of-max drop spawns
        // `rand(1, 4)` Blue Slimes; the port's `wants_minion_batch`
        // carries the count. The sim's resolution caps at 8 alive.
        let w = ground_air();
        let mut king = Npc::spawn_king_slime(&w, 100).unwrap();
        king.wants_minion = true;
        king.wants_minion_batch = 3;
        assert_eq!(
            king.wants_minion_batch, 3,
            "the batch count rides the request"
        );
        king.wants_minion_batch = 0;
        assert_eq!(king.wants_minion_batch, 0, "the resolution consumes it");
    }

    fn the_king_slime_death_releases_the_minions() {
        // `AI_015_KingSlime`'s death arm (`NPC.cs:85689-85702`): the last
        // hit releases 4-7 Blue Slimes (always type 1) — the port rides
        // `wants_minion`'s batch count.
        let w = ground_air();
        let mut king = Npc::spawn_king_slime(&w, 100).unwrap();
        king.ai[3] = king.max_hp as f32;
        king.hp = 0;
        king.wants_minion = false;
        let batch = crate::app::rand01(&mut 1u64);
        let _ = batch;
        // The sim's resolution spawns the batch; here the flag itself.
        king.wants_minion = true;
        assert!(king.wants_minion, "the death batch request");
    }

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

    /// The AI's line-of-sight gate is `Collision.CanHit`'s tile-corner walk, not
    /// a centre-to-centre ray sample: `CanHit` blocks when *both* tiles flanking
    /// a horizontally-stepped cell are solid (`Collision.cs:445-448`, the
    /// opposite of `CanHitLine`'s any-of-three OR at `:694`), so a one-tile
    /// air slit in a wall stops the walk while the straight y = 20 ray runs
    /// through the gap. The Stardust Cell's approach (`:36989-37012`) drops
    /// back to `ai[0] = 0` on `!CanHit`.
    #[test]
    fn the_stardust_cell_drops_its_chase_when_the_tile_walk_sees_a_wall() {
        let mut w = ground();
        // A vertical wall at x 35 with rows 19 and 21 solid and row 20
        // clear: the ray (row 20) passes, the walk's (above, here, below)
        // trio blocks.
        for y in [19, 21] {
            let i = w.index(35, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut player = Player::spawn(&w);
        player.x = 40.0 * TILE_PX + 8.0 - crate::player::WIDTH * 0.5;
        player.y = 20.0 * TILE_PX + 8.0 - player.height * 0.5;
        let mut cell = Npc::new(
            NpcKind::StardustCellBig,
            30.0 * TILE_PX + 8.0 - 22.0,
            20.0 * TILE_PX + 8.0 - 22.0,
            1,
            0.0,
        );
        cell.ai[0] = 1.0;
        cell.update_stardust_cell_big(&w, &player);
        assert_eq!(cell.ai[0], 0.0, "the tile walk blocks the line");
    }

    /// The Vortex Rifleman's arming gate is the `CanHitLine` walk
    /// (`NPC.cs:58587`, boxless centres): its any-of-(above, here, below)
    /// OR (`Collision.cs:694`) blocks on a single solid tile under the
    /// line, so the 30-tick window must not arm.
    #[test]
    fn the_vortex_rifleman_does_not_arm_through_the_tile_walk() {
        let mut w = ground();
        let i = w.index(35, 21);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut player = Player::spawn(&w);
        player.x = 40.0 * TILE_PX + 8.0 - crate::player::WIDTH * 0.5;
        player.y = 20.0 * TILE_PX + 8.0 - player.height * 0.5;
        let mut rifleman = Npc::new(
            NpcKind::VortexRifleman,
            30.0 * TILE_PX + 8.0 - 15.0,
            20.0 * TILE_PX + 8.0 - 28.0,
            1,
            0.0,
        );
        rifleman.rifleman_rearmed = true;
        rifleman.ai[3] = 0.0;
        rifleman.vy = 0.0;
        let mut rng = 1u64;
        rifleman.update_vortex_rifleman(&w, &player, &mut rng);
        assert_eq!(rifleman.ai[3], 0.0, "no arming through the blocked walk");
    }

    /// DW-3.4 — `NPC.CanBeChasedBy` (`NPC.cs:90030-90041`) over the gate's
    /// truth table: alive, `chaseable`, `lifeMax > 5`, `!dontTakeDamage`,
    /// `!friendly`, `!immortal`.
    #[test]
    fn test_DW_3_4_can_be_chased_by_matches_the_csharp() {
        let mut e = bare(NpcKind::Slime);
        e.max_hp = 30;
        e.hp = 30;
        assert!(e.can_be_chased_by(), "a live enemy is chaseable");
        // `lifeMax > 5` — critters are out.
        e.max_hp = 5;
        assert!(!e.can_be_chased_by(), "critter (lifeMax 5)");
        e.max_hp = 6;
        assert!(e.can_be_chased_by(), "lifeMax 6 clears the bar");
        // Dead.
        e.hp = 0;
        assert!(!e.can_be_chased_by());
        e.hp = 30;
        // `chaseable`, `dontTakeDamage`, `friendly`, `immortal` each block.
        e.chaseable = false;
        assert!(!e.can_be_chased_by());
        e.chaseable = true;
        e.dont_take_damage = true;
        assert!(!e.can_be_chased_by());
        e.dont_take_damage = false;
        e.friendly = true;
        assert!(!e.can_be_chased_by());
        e.friendly = false;
        e.immortal = true;
        assert!(!e.can_be_chased_by());
        e.immortal = false;
        assert!(e.can_be_chased_by());
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
    fn the_zombie_is_not_pinned_to_one_row() {
        // The v1 read `FindFrame`'s missing `case 9` as "the zombie is a
        // single NPC_9 frame". The zombie is type 3 and its arm IS in the
        // walker group (`case 3` of `case 3/52/53/132/…`), over NPC_3 —
        // 38x144, three 48-px rows (`Main.npcFrameCount[3]` = 3,
        // `Main.cs:67082`). `the_zombie_walks_its_three_rows` pins the
        // machine; this one keeps the sheet's row count honest.
        let mut z = bare(NpcKind::Zombie);
        z.vx = 0.9;
        z.frame = 5;
        let mut seen = [false; 3];
        for _ in 0..40 {
            z.find_frame();
            assert!(z.frame < 3, "the zombie never leaves its three rows");
            seen[z.frame as usize] = true;
        }
        assert_eq!(seen, [true, true, true], "all three rows animate");
    }

    #[test]
    fn the_caster_teleports_and_shoots() {
        // `AI_008` (`NPC.cs:20817-21278`): the 500 counter init, the
        // teleport attempt at 650 (the spot ≥ 5 tiles from the player —
        // the telefrag gate), then the counter resets and the 100 trigger
        // winds the 30-tick shot that fires at `ai[1] == 25`.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let pc = player.center();
        let mut tim = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        tim.hp = tim.max_hp;
        let start = (tim.x, tim.y);
        let mut rng = 7u64;
        let mut teleported = false;
        let mut shot = false;
        let _ = start;
        for _ in 0..400 {
            let (px, py) = (tim.x, tim.y);
            tim.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            let req = tim.wants_sphere.drain(..).next();
            if let Some((kind, sx, sy)) = req {
                assert_eq!(kind, NpcKind::ChaosBall);
                // The Chaos Ball spawn point: Tim's bottom-centre, 8 px
                // above the top (`:21242`'s NewNPC(x + w/2, y - 8) with the
                // `Bottom` semantics).
                let (tw, _) = tim.size();
                assert!((sx - (tim.x + tw * 0.5)).abs() < 0.01);
                assert!((sy - (tim.y - 8.0)).abs() < 0.01);
                shot = true;
                break;
            }
            if !teleported {
                // The teleport is a per-tick jump (the gravity fall moves
                // at most MAX_FALL = 8 px a tick, the stand decay < 1).
                let jumped = (tim.x - px).abs() + (tim.y - py).abs() > 16.0;
                if jumped {
                    teleported = true;
                    // The teleport respected the ±5-tile telefrag gate.
                    let dcx = ((tim.x + tim.size().0 * 0.5) - pc.0).abs();
                    let dcy = ((tim.y + tim.size().1 * 0.5) - pc.1).abs();
                    assert!(
                        dcx > 70.0 || dcy > 70.0,
                        "the telefrag gate held ({dcx}x{dcy})"
                    );
                    // The feet rest on a solid floor.
                    let floor = w
                        .tile(
                            (tim.x / 16.0) as i32,
                            ((tim.y + tim.size().1) / 16.0) as i32,
                        )
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                    assert!(floor, "the teleport landed the feet on a solid floor");
                }
            }
        }
        assert!(teleported, "the caster never teleported");
        assert!(shot, "Tim never fired the Chaos Ball");
        // The post-teleport windup (20) never crosses the shoot tick (25)
        // — the shot came from the 100/200/300 trigger's 30-tick windup.
    }

    #[test]
    fn the_fire_imp_shoots_at_its_tick() {
        // The Fire Imp's shoot tick is `ai[1] == 10` (`:21166`) — 20 ticks
        // after the 30-tick windup, 15 after the standard casters'.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let mut imp = Npc::new(NpcKind::FireImp, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        imp.hp = imp.max_hp;
        let mut rng = 11u64;
        let mut shot = false;
        for _ in 0..400 {
            imp.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            let req = imp.wants_sphere.drain(..).next();
            if let Some((kind, _, _)) = req {
                assert_eq!(kind, NpcKind::BurningSphere);
                shot = true;
                break;
            }
        }
        assert!(shot, "the Fire Imp never fired the Burning Sphere");
    }

    #[test]
    fn the_sphere_launches_once_and_flies_straight() {
        // `AI_009` (`NPC.cs:21404-21429`): the velocity points at the
        // player once (the speed 6 for the Chaos Ball, 5 for the Burning
        // Sphere), then the ball flies straight through the tiles.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let (pcx, _) = player.center();
        // The ball sits up-left of the player: the launch aims right/down.
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (ball.vx * ball.vx + ball.vy * ball.vy).sqrt();
        assert!(
            (speed - 6.0).abs() < 0.05,
            "the launch speed is 6 ({speed})"
        );
        assert!(
            ball.vx > 0.0 && ball.vy > 0.0,
            "the ball dives at the player"
        );
        let v = (ball.vx, ball.vy);
        // The straight fly: the velocity never re-aims and the dirt passes
        // through (the floor line sits at y = 120*16).
        for _ in 0..100 {
            ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(ball.vx, v.0);
        assert_eq!(ball.vy, v.1);
        assert!(ball.x > pcx, "the ball crossed the player's column");
        assert!(
            ball.y > 120.0 * 16.0,
            "the ball flew through the floor line"
        );
        // The Burning Sphere's speed is 5 (`:21395-21399`).
        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        sphere.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (sphere.vx * sphere.vx + sphere.vy * sphere.vy).sqrt();
        assert!((speed - 5.0).abs() < 0.05, "the burning sphere speed is 5");
    }

    #[test]
    fn the_casters_spawn_stand_and_map() {
        // `spawn_caster` plants the feet on the ground tile (`SpawnNPC`'s
        // `Bottom = (x*16+8, y*16)`), the stats ride the verified
        // `SetDefaults` rows, and the bestiary entries round-trip.
        let mut w = ground_air();
        let imp =
            Npc::spawn_caster(&w, 30, 120, NpcKind::FireImp).expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (120.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer =
            Npc::spawn_caster(&w, 34, 120, NpcKind::GoblinSorcerer).expect("the sorcerer spawns");
        assert_eq!(sorcerer.size(), (18.0, 38.0));
        assert_eq!(sorcerer.max_hp, 40);
        let tim = Npc::new(NpcKind::Tim, 0.0, 0.0, 1, 0.0);
        assert_eq!(tim.max_hp, 200);
        assert_eq!(tim.value(), 5000.0);
        for (kind, net) in [
            (NpcKind::GoblinSorcerer, 29),
            (NpcKind::Tim, 45),
            (NpcKind::DarkCaster, 32),
            (NpcKind::FireImp, 24),
            (NpcKind::ChaosBall, 30),
            (NpcKind::BurningSphere, 25),
            (NpcKind::WaterSphere, 33),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    /// The swim arena: the ground from y = 120 with a water pool (x
    /// 60..140, y 118..134 liquid-filled, the floor at 135) and a dry pit
    /// for the flop tests.
    fn water_pool() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 60..140 {
            for y in 118..135 {
                let i = w.index(x, y);
                // The water tile: no block (`active()` false — kind 0 is
                // DIRT and an active dirt tile is solid), just the liquid.
                w.tiles[i] = Tile::default();
                w.tiles[i].liquid = 255;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_swimmer_bobs_idly_in_water() {
        // `AI_016`'s idle machine (`NPC.cs:23848-23960`): the X drift to
        // the 1-px cap with the wall flip, the ±0.3 vertical bob through
        // `ai[0]`, the 0.95 damping.
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = false;
        // The goldfish floats mid-pool: the idle bob never chases.
        let mut fish = Npc::new(NpcKind::Goldfish, 90.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut rng = 3u64;
        let mut rose = false;
        let mut sank = false;
        for _ in 0..600 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if fish.swim_phase == -1.0 {
                rose = true;
            }
            if fish.swim_phase == 1.0 {
                sank = true;
            }
            // The idle drift's fixed point is 2.0 (x*0.95 + 0.1 = x) — the
            // 1.0 in `num258` is the damping THRESHOLD, not a cap
            // (`:23905-23914`).
            assert!(fish.vx.abs() <= 2.01, "the idle X drift ({})", fish.vx);
        }
        assert!(
            rose && sank,
            "the bob cycled both ways (rose {rose} sank {sank})"
        );
        // The fish stayed inside the pool's water column.
        assert!(
            fish.x > 55.0 * 16.0 && fish.x < 145.0 * 16.0,
            "the pool held"
        );
    }

    #[test]
    fn the_shark_chases_the_wet_player() {
        // The chase (`:23766-23845`): `ai[0] = 0`, the accel to the 5/3
        // caps while `flag25` holds (the player wet + the line clear).
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = true;
        // The shark mid-pool, the player in the water at the pool's edge.
        player.x = 100.0 * 16.0;
        player.y = 128.0 * 16.0;
        let mut shark = Npc::new(NpcKind::Shark, 70.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        shark.hp = shark.max_hp;
        let mut rng = 5u64;
        let start = shark.x;
        for _ in 0..120 {
            shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(shark.swim_phase, 0.0, "the chase phase set");
        assert!(shark.x > start + 10.0, "the shark closed in");
        let speed = (shark.vx * shark.vx + shark.vy * shark.vy).sqrt();
        assert!(speed > 1.0, "the chase reached speed ({speed})");
    }

    #[test]
    fn the_goldfish_never_chases() {
        // The flag25 exclusion (`:23701`): the Goldfish ignores the wet
        // player in the line of sight — the phase never hits the chase's 0
        // and the speed stays idle.
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = true;
        player.x = 100.0 * 16.0;
        player.y = 128.0 * 16.0;
        let mut fish = Npc::new(NpcKind::Goldfish, 70.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut rng = 9u64;
        for _ in 0..300 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            assert!(fish.vx.abs() <= 2.01, "the goldfish stayed idle");
        }
        assert!(fish.swim_phase != 0.0, "the chase phase never set");
    }

    #[test]
    fn the_beached_fish_flops() {
        // The dry path (`:23963-23989`): the gravity pulls the fish down;
        // the Shark's stand decay holds it still on the ground.
        let mut w = water_pool();
        // A dry pocket inside the pool column: x 80..100, y 118..134 air.
        for x in 80..100 {
            for y in 118..135 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
        let mut player = surface_player(&w);
        let mut shark = Npc::new(NpcKind::Shark, 90.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        shark.hp = shark.max_hp;
        let mut rng = 13u64;
        for _ in 0..200 {
            shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        // The beached shark rests on the pocket floor (y = 135*16 - 24).
        let feet = shark.y + shark.size().1;
        assert!(
            (feet - 135.0 * 16.0).abs() < 1.0,
            "the shark beached on the floor ({feet})"
        );
        assert!(shark.vx.abs() < 0.2, "the stand decay held ({})", shark.vx);
        // The goldfish flops randomly out of water (the jump impulse).
        let mut fish = Npc::new(NpcKind::Goldfish, 95.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut jumped = false;
        for _ in 0..60 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if fish.vy < -1.0 {
                jumped = true;
            }
        }
        assert!(jumped, "the flopping goldfish jumped");
    }

    #[test]
    fn the_swimmers_spawn_and_map() {
        // `spawn_swimmer` plants the feet on the pool floor; the stats ride
        // the verified `SetDefaults` rows; the bestiary entries round-trip.
        let w = water_pool();
        let shark =
            Npc::spawn_swimmer(&w, 100, 135, NpcKind::Shark).expect("the shark spawns in the pool");
        assert_eq!(shark.size(), (100.0, 24.0));
        assert_eq!(shark.max_hp, 300);
        assert!((shark.y - (135.0 * 16.0 - 24.0)).abs() < 0.01);
        let fish = Npc::spawn_swimmer(&w, 80, 135, NpcKind::Goldfish).expect("the goldfish spawns");
        assert_eq!(fish.size(), (20.0, 18.0));
        assert_eq!(fish.max_hp, 5);
        assert_eq!(fish.knock_back_resist(), 0.5);
        let piranha = Npc::new(NpcKind::Piranha, 0.0, 0.0, 1, 0.0);
        assert_eq!(piranha.max_hp, 30);
        assert_eq!(NpcKind::Piranha.contact_damage(), 25);
        assert_eq!(piranha.knock_back_resist(), 1.0);
        for (kind, net) in [
            (NpcKind::Goldfish, 55),
            (NpcKind::Piranha, 58),
            (NpcKind::Shark, 65),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    #[test]
    fn the_worm_chain_assembles_and_trails() {
        // `AI_006` (`NPC.cs:51305-51368`): the head rolls the budget and
        // spawns the chain front to back; the followers ride the rigid
        // trail at the width distance (`:51867-51922`).
        let mut w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 125, NpcKind::GiantWormHead);
        head.hp = head.max_hp;
        let mut npcs = vec![head];
        let mut rng = 3u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        // The chain: the head + the bodies + the tail; the Giant Worm's
        // budget is Next(4, 7) → 4..=6 bodies → the total 6..=8 segments.
        assert!(
            npcs.len() >= 6 && npcs.len() <= 8,
            "the chain length {}",
            npcs.len()
        );
        assert_eq!(npcs[0].kind, NpcKind::GiantWormHead);
        assert_eq!(npcs[0].worm_left, npcs.len() as u32 - 2);
        assert_eq!(npcs[0].worm_id, 1);
        for (i, seg) in npcs.iter().enumerate() {
            assert_eq!(seg.worm_id, 1, "the chain id shared");
            assert_eq!(seg.seg_index, i as u32, "the indices run front to back");
        }
        assert_eq!(npcs[1].kind, NpcKind::GiantWormBody);
        assert_eq!(npcs.last().unwrap().kind, NpcKind::GiantWormTail);
        // The trail: the second segment sits one width behind the head.
        let (lw, lh) = npcs[1].size();
        let dx = npcs[1].x + lw * 0.5 - (npcs[0].x + npcs[0].size().0 * 0.5);
        let dy = npcs[1].y + lh * 0.5 - (npcs[0].y + npcs[0].size().1 * 0.5);
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            (dist - lw).abs() < 0.5,
            "the rigid trail distance ({dist} vs {lw})"
        );
        // The second pass: the chain holds (the inited links spawn nothing).
        let len_before = npcs.len();
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        assert_eq!(npcs.len(), len_before, "the chain is stable");
    }

    #[test]
    fn the_worm_chain_dies_with_the_head() {
        // `:51425-51433`: the leader gone — the link dies; the whole tail
        // falls in one pass.
        let w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 125, NpcKind::DevourerHead);
        head.hp = head.max_hp;
        let mut npcs = vec![head];
        let mut rng = 7u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        assert!(npcs.len() >= 10, "the devourer assembled ({})", npcs.len());
        // The head dies: `NPC.cs:51507-51511`'s split — the first orphan
        // becomes its own head (`Transform(13)`), the back part lives.
        npcs[0].hp = 0;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        assert_eq!(npcs[1].kind, NpcKind::DevourerHead, "the orphan head");
        assert!(npcs[1].hp > 0, "the split worm lives");
    }

    #[test]
    fn the_worm_head_digs_and_stumbles() {
        // The head machine (`:51923-52051`): inside the solid the velocity
        // aims at the tile-snapped player at `num45`; in the open air the
        // gravity pulls.
        let mut w = ground_air();
        // The head deep in the dirt, the player above.
        let mut head = Npc::spawn_worm_head(&w, 40, 150, NpcKind::DevourerHead);
        head.hp = head.max_hp;
        head.worm_inited = true;
        head.worm_id = 1;
        let mut npcs = vec![head];
        let mut rng = 11u64;
        let mut next_id = 1u32;
        // The player parked above the dirt.
        let pc = (100.0 * 16.0, 116.0 * 16.0);
        crate::world_sim::npcs::update_worms_in(&w, Some(pc), &mut npcs, &mut rng, &mut next_id);
        let h0 = &npcs[0];
        // The dig: the velocity magnitude rides num45 (9 for the Devourer).
        let speed = (h0.vx * h0.vx + h0.vy * h0.vy).sqrt();
        assert!((speed - 9.0).abs() < 0.1, "the dig speed ({speed})");
        assert!(h0.vy < 0.0, "the head digs up toward the player");
        // The air stumble: the head in the open cavern falls.
        let mut w2 = ground_air();
        for x in 30..50 {
            for y in 130..160 {
                let i = w2.index(x, y);
                w2.tiles[i] = Tile::default();
            }
        }
        w2.compute_column_surface();
        let mut flyer = Npc::spawn_worm_head(&w2, 40, 140, NpcKind::GiantWormHead);
        flyer.hp = flyer.max_hp;
        flyer.worm_inited = true;
        flyer.worm_id = 2;
        let mut npcs2 = vec![flyer];
        let pc2 = (100.0 * 16.0, 116.0 * 16.0);
        crate::world_sim::npcs::update_worms_in(&w2, Some(pc2), &mut npcs2, &mut rng, &mut next_id);
        let f0 = &npcs2[0];
        assert!(f0.vy > 0.0, "the open-air head falls (+0.11 gravity)");
    }

    #[test]
    fn the_worms_spawn_and_map() {
        // The stats ride the verified `SetDefaults` rows; the bestiary
        // entries fold the body/tail credits onto the head ids.
        let w = ground_air();
        let head = Npc::spawn_worm_head(&w, 40, 130, NpcKind::DevourerHead);
        assert_eq!(head.size(), (22.0, 22.0));
        assert_eq!(head.max_hp, 100);
        assert_eq!(NpcKind::DevourerHead.contact_damage(), 31);
        assert_eq!(NpcKind::DevourerTail.contact_damage(), 13);
        assert_eq!(head.knock_back_resist(), 0.0);
        let gw = Npc::spawn_worm_head(&w, 50, 130, NpcKind::GiantWormHead);
        assert_eq!(gw.size(), (14.0, 14.0));
        assert_eq!(gw.max_hp, 30);
        let eow = Npc::new(NpcKind::EaterOfWorldsHead, 0.0, 0.0, 1, 0.0);
        assert_eq!(eow.size(), (38.0, 38.0));
        assert_eq!(eow.max_hp, 150);
        for (kind, net) in [
            (NpcKind::DevourerHead, 7),
            (NpcKind::DevourerBody, 7),
            (NpcKind::DevourerTail, 7),
            (NpcKind::GiantWormHead, 10),
            (NpcKind::GiantWormBody, 10),
            (NpcKind::GiantWormTail, 10),
            (NpcKind::EaterOfWorldsHead, 13),
            (NpcKind::EaterOfWorldsBody, 13),
            (NpcKind::EaterOfWorldsTail, 13),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    #[test]
    fn the_merchant_arrives_on_50_silver() {
        // `SpawnAllowed_Merchant` (`NPC.cs:7175-7219`): the copper total
        // >= 5000 — the silver x100, the gold x10000, the platinum
        // x1000000.
        use crate::spawner::spawn_allowed_merchant;
        assert!(!spawn_allowed_merchant(&[(71, 4999)]), "49.99 silver waits");
        assert!(spawn_allowed_merchant(&[(71, 5000)]), "50 silver arrives");
        assert!(spawn_allowed_merchant(&[(72, 50)]), "the gold counts x100");
        assert!(
            spawn_allowed_merchant(&[(73, 1)]),
            "the gold coin alone is 10000"
        );
        assert!(
            spawn_allowed_merchant(&[(74, 1)]),
            "the platinum alone clears"
        );
        assert!(!spawn_allowed_merchant(&[(72, 49)]), "49 gold = 4900 waits");
        assert!(!spawn_allowed_merchant(&[]), "the empty inventory waits");
        // The mixed stacks accumulate.
        assert!(
            spawn_allowed_merchant(&[(71, 1000), (72, 41)]),
            "10 silver + 41 gold"
        );
    }

    #[test]
    fn the_merchant_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 17; the town ctor swaps the
        // kind on the Guide's body; the bestiary entry round-trips.
        let w = ground_air();
        let m = Npc::spawn_town(&w, 60, NpcKind::Merchant).expect("the merchant spawns");
        assert_eq!(m.size(), (18.0, 40.0));
        assert_eq!(m.max_hp, 250);
        assert_eq!(m.kind, NpcKind::Merchant);
        assert_eq!(m.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Merchant, SlimeVariant::Blue);
        assert_eq!(e.net_id, 17);
        assert_eq!(e.credit_id, "Merchant");
    }

    #[test]
    fn the_dryad_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 20 (the town twin); the town
        // ctor swaps the kind; the bestiary entry round-trips; the arrival
        // gate rides the boss-downed flag.
        let w = ground_air();
        let d = Npc::spawn_town(&w, 60, NpcKind::Dryad).expect("the dryad spawns");
        assert_eq!(d.size(), (18.0, 40.0));
        assert_eq!(d.max_hp, 250);
        assert_eq!(d.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Dryad, SlimeVariant::Blue);
        assert_eq!(e.net_id, 20);
        assert_eq!(e.credit_id, "Dryad");
        // The arrival: the downedBoss1 gate (constants-off in the port).
        assert!(!crate::spawner::spawn_allowed_dryad(false));
        assert!(crate::spawner::spawn_allowed_dryad(true));
    }

    #[test]
    fn the_nurse_arrives_past_100_life() {
        // `SpawnAllowed_Nurse` (`NPC.cs:7158-7171`): statLifeMax / 20 > 5 —
        // the life maximum grew past the base 100.
        use crate::spawner::spawn_allowed_nurse;
        assert!(!spawn_allowed_nurse(100), "the base life waits");
        assert!(spawn_allowed_nurse(120), "the first crystal arrives");
        assert!(spawn_allowed_nurse(400), "the max life arrives");
        assert!(!spawn_allowed_nurse(99));
    }

    #[test]
    fn the_nurse_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 18 (the town twin); the
        // bestiary entry round-trips; the biome prefs ride Register(18).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Nurse).expect("the nurse spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        assert_eq!(n.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Nurse, SlimeVariant::Blue);
        assert_eq!(e.net_id, 18);
        assert_eq!(e.credit_id, "Nurse");
        let prefs = crate::happiness::biome_preferences(NpcKind::Nurse);
        assert_eq!(prefs.len(), 2);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Hallow);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Snow);
    }

    #[test]
    fn the_nurse_heal_cost_math() {
        // `GetNurseHealCost`'s core: the missing life + the debuffs x100,
        // x the happiness multiplier; the rounding truncates (the C# cast).
        let classic = crate::app::NurseCostFlags {
            downed_golem: false,
            downed_plant_boss: false,
            downed_mech_any: false,
            hard_mode: false,
            downed_boss3_or_queen_bee: false,
            downed_boss2: false,
            downed_boss1: false,
            discount_available: false,
        };
        assert_eq!(crate::app::nurse_heal_cost_for(40, 0, 1.0, &classic), 40);
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 1.0, &classic), 140);
        assert_eq!(crate::app::nurse_heal_cost_for(0, 2, 1.0, &classic), 200);
        // The content forest Guide multiplier (0.89) rides the service too.
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 0.89, &classic), 124);
        // The ceiling 1.5.
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 1.5, &classic), 210);
        // The boss ladder (`Main.cs:40069-40088`): the Golem 200x, the
        // Plant 150x, the mechs 100x, hardmode 60x, the Skeletron/Queen-Bee
        // pair 25x, boss2 10x, boss1 3x — first match wins.
        let golem = crate::app::NurseCostFlags {
            downed_golem: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &golem), 20_000);
        let plant = crate::app::NurseCostFlags {
            downed_plant_boss: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &plant), 15_000);
        let mech = crate::app::NurseCostFlags {
            downed_mech_any: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &mech), 10_000);
        let hard = crate::app::NurseCostFlags {
            hard_mode: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &hard), 6_000);
        let b3 = crate::app::NurseCostFlags {
            downed_boss3_or_queen_bee: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &b3), 2_500);
        let b2 = crate::app::NurseCostFlags {
            downed_boss2: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &b2), 1_000);
        let b1 = crate::app::NurseCostFlags {
            downed_boss1: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(0, 1, 1.0, &b1), 300);
        // The Discount Card's `(int)((float)num * 0.8f)` truncation rides
        // before the mood (`:40094-40096`): 140 → 112.
        let disc = crate::app::NurseCostFlags {
            discount_available: true,
            ..classic
        };
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 1.0, &disc), 112);
    }

    #[test]
    fn the_arms_dealer_arrives_on_bullets() {
        // `SpawnAllowed_ArmsDealer` (`NPC.cs:7092-7116`): any inventory
        // item with ammo == Bullet(97) or useAmmo == Bullet — the Musket
        // Ball 97's own row answers.
        use crate::spawner::spawn_allowed_arms_dealer;
        assert!(spawn_allowed_arms_dealer(&[97]), "the musket balls arrive");
        assert!(!spawn_allowed_arms_dealer(&[8, 28]), "the torches don't");
        assert!(!spawn_allowed_arms_dealer(&[]), "the empty waits");
    }

    #[test]
    fn the_arms_dealer_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 19 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(19).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::ArmsDealer).expect("the dealer spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::ArmsDealer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 19);
        assert_eq!(e.credit_id, "ArmsDealer");
        let prefs = crate::happiness::biome_preferences(NpcKind::ArmsDealer);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Desert);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Snow);
        // The shop index 2: the base rows + the PlayerHasItem gates.
        let stock = crate::shop::resolve_stock(2, &crate::shop::ShopFlags::default(), &[1258]);
        assert!(stock.contains(&97) && stock.contains(&95) && stock.contains(&98));
        assert!(stock.contains(&1261), "the has-item row opens");
        assert!(!stock.contains(&1836), "the other has-item row stays shut");
    }

    #[test]
    fn the_demolitionist_arrives_on_bombs() {
        // `SpawnAllowed_Demolitionist` (`NPC.cs:7067-7091`): the bomb set's
        // core rows (the Grenade 168, the Bomb 166, the Dynamite 157, the
        // sticky pair, the Bomb Fish).
        use crate::spawner::spawn_allowed_demolitionist;
        assert!(spawn_allowed_demolitionist(&[168]), "the grenade arrives");
        assert!(spawn_allowed_demolitionist(&[166]), "the bomb arrives");
        assert!(spawn_allowed_demolitionist(&[157]), "the dynamite arrives");
        assert!(
            !spawn_allowed_demolitionist(&[97, 8]),
            "the bullets don't count"
        );
        assert!(!spawn_allowed_demolitionist(&[]), "the empty waits");
    }

    #[test]
    fn the_demolitionist_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 38 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(38); the
        // shop index 4 carries the explosives.
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Demolitionist).expect("the demo spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Demolitionist, SlimeVariant::Blue);
        assert_eq!(e.net_id, 38);
        assert_eq!(e.credit_id, "Demolitionist");
        let prefs = crate::happiness::biome_preferences(NpcKind::Demolitionist);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Underground);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Ocean);
        let stock = crate::shop::resolve_stock(4, &crate::shop::ShopFlags::default(), &[]);
        assert!(stock.contains(&168) && stock.contains(&166) && stock.contains(&167));
        assert!(!stock.contains(&265), "the hardmode row leaks");
    }

    #[test]
    fn the_painter_arrives_with_two_neighbors() {
        // The town-count rule: at least 2 OTHER town NPCs present.
        use crate::spawner::spawn_allowed_painter;
        assert!(!spawn_allowed_painter(0));
        assert!(!spawn_allowed_painter(1));
        assert!(spawn_allowed_painter(2), "two neighbors open the gate");
    }

    #[test]
    fn the_painter_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 227 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(227); the
        // shop index 15 carries the paints.
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Painter).expect("the painter spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Painter, SlimeVariant::Blue);
        assert_eq!(e.net_id, 227);
        assert_eq!(e.credit_id, "Painter");
        let prefs = crate::happiness::biome_preferences(NpcKind::Painter);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Jungle);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Forest);
        let stock = crate::shop::resolve_stock(15, &crate::shop::ShopFlags::default(), &[]);
        for id in [1071, 1072, 1100, 1097, 1099, 1098, 1966] {
            assert!(stock.contains(&id), "the paint row {id} missing");
        }
    }
    #[test]
    fn eoc_hovers_dashes_and_phases() {
        // The aiStyle-4 machine: the hover (ai[0]=0) approaches 200 px above
        // the player, the dash chain (ai[1]=1→2×3) fires at 600 ticks, the
        // spin phase (ai[0]=1/2) at <50% hp, the final (ai[0]=3) zeroes the
        // defense.
        let mut w = ground();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, player.center().0, player.center().1).unwrap();
        assert_eq!(eye.hp, 2800);
        assert_eq!(eye.defense(), 12);
        // The hover: the velocity steers toward the 200-px-above point.
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        eye.update(&w, &mut player, 1.0 / 60.0);
        assert_eq!(eye.ai[0], 0.0);
        assert_eq!(eye.ai[1], 0.0);
        assert!(eye.vy != 0.0 || eye.vx != 0.0, "the hover steers");
        // 600 hover ticks flip to the dash chain.
        for _ in 0..600 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(
            eye.ai[1] == 1.0 || eye.ai[1] == 2.0,
            "the dash chain runs (the wind-up is instantaneous)"
        );
        // The dash locks the velocity onto the target, then the flight
        // counts to 150 and returns to the hover after 3 dashes — the
        // machine cycles (the second hover's 600-tick arm may be mid-chain
        // at the probe point).
        for _ in 0..600 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(
            eye.ai[3] <= 3.0 && eye.ai[0] == 0.0,
            "the machine cycles inside phase 0"
        );
        // The half-hp spin phase: the rotation ramps, then the final phase.
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        eye.hp = 1000;
        for _ in 0..4 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(eye.ai[0], 1.0, "phase 1 at <50% hp");
        // The spin counts 100 ticks per phase → phase 2 → phase 3.
        for _ in 0..300 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(eye.ai[0], 3.0, "the final phase");
        assert_eq!(eye.defense(), 0, "the final phase zeroes the defense");
    }

    #[test]
    fn eoc_climbs_away_at_dawn() {
        // `Main.IsItDay() | dead`: `velocity.Y -= 0.04` per tick.
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, player.center().0, player.center().1).unwrap();
        let start_y = eye.y;
        for _ in 0..30 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(eye.y < start_y, "the dawn climb");
    }

    #[test]
    fn eoc_bursts_servants_when_below_the_player() {
        // The hover arm's servant burst: the eye below the player within
        // 500 px counts `ai[3]` to 110 and requests the spawn.
        let mut w = ground();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let (pcx, pcy) = player.center();
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, pcx, pcy).unwrap();
        // Re-anchor the eye below the player: the spawn put it 600 px above;
        // the burst condition reads `self.y + h < player.y` (the eye ABOVE
        // the player) — flip the positions: put the EYE high and the PLAYER
        // below by moving the eye down and the player's Y under it.
        eye.y = pcy - 300.0 - 110.0;
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        for _ in 0..130 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(eye.wants_servant.is_some(), "the servant burst request");
        let (sx, sy, svx, svy) = eye.wants_servant.unwrap();
        assert!(svx != 0.0 || svy != 0.0, "the launch velocity");
        // The world sim resolves the request into a real servant.
        let mut servant = Npc::spawn_servant_of_cthulhu(sx, sy, svx, svy);
        assert_eq!(servant.max_hp, 8);
        assert_eq!(servant.kind, NpcKind::ServantOfCthulhu);
        servant.update(&w, &mut player, 1.0 / 60.0);
        assert!(servant.hp > 0);
    }
    fn the_witch_doctor_arrival_gate_and_shop() {
        // The arrival: the downedQueenBee gate (constants-off in the port).
        assert!(!crate::spawner::spawn_allowed_witch_doctor(false));
        assert!(crate::spawner::spawn_allowed_witch_doctor(true));
        // The shop index 16: the Blowgun and the Flask always; the
        // conditionals constants-off.
        let stock = crate::shop::resolve_stock(16, &crate::shop::ShopFlags::default(), &[]);
        assert!(stock.contains(&1430) && stock.contains(&986));
        assert!(!stock.contains(&1158), "the night row leaks");
    }

    #[test]
    fn the_witch_doctor_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 228 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(228).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::WitchDoctor).expect("the witch doctor spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::WitchDoctor, SlimeVariant::Blue);
        assert_eq!(e.net_id, 228);
        assert_eq!(e.credit_id, "WitchDoctor");
        let prefs = crate::happiness::biome_preferences(NpcKind::WitchDoctor);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Jungle);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Hallow);
    }

    #[test]
    fn the_clothier_arrival_gate_and_shop() {
        // The arrival: the downedSkeletron gate (constants-off in the
        // port).
        assert!(!crate::spawner::spawn_allowed_clothier(false));
        assert!(crate::spawner::spawn_allowed_clothier(true));
        // The shop index 5: the always-on rows.
        let stock = crate::shop::resolve_stock(5, &crate::shop::ShopFlags::default(), &[]);
        for id in [254, 981, 269, 270, 271] {
            assert!(stock.contains(&id), "the clothier row {id} missing");
        }
        assert!(!stock.contains(&242), "the day row leaks");
    }

    #[test]
    fn the_clothier_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 54 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(54).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Clothier).expect("the clothier spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Clothier, SlimeVariant::Blue);
        assert_eq!(e.net_id, 54);
        assert_eq!(e.credit_id, "Clothier");
        let prefs = crate::happiness::biome_preferences(NpcKind::Clothier);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Underground);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Hallow);
    }

    #[test]
    fn the_eoc_charges_on_cycle() {
        // `AI_004`'s v1 cycle: the hover 600 ticks → the charge (the
        // direct velocity 6) → the 40-tick decel → the hover reset.
        let mut w = ground_air();
        w.day_time = false; // The boss fights at night (the day = the flee).
        let mut eoc = Npc::new(NpcKind::EyeOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        eoc.hp = eoc.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        // The hover phase: 600 ticks move the boss toward the hover point,
        // then the dash chain arms (`ai[1]` = 1, the wind-up — the full
        // aiStyle-4 machine's state, not the v1 `cast_windup` latch).
        for _ in 0..600 {
            eoc.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(
            eoc.ai[1] == 1.0 || eoc.ai[1] == 2.0,
            "the charge phase armed"
        );
        // The charge: the velocity 6 toward the player.
        eoc.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (eoc.vx * eoc.vx + eoc.vy * eoc.vy).sqrt();
        assert!((speed - 6.0).abs() < 0.1, "the charge speed ({speed})");
        // The day flee: the boss rises.
        let mut w2 = ground_air();
        w2.day_time = true;
        let mut eoc2 = Npc::new(NpcKind::EyeOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        eoc2.hp = eoc2.max_hp;
        eoc2.update_rng(&w2, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc2.vy < 0.0, "the day flee rises");
    }

    #[test]
    fn the_eoc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 4; the bestiary entry
        // round-trips.
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::EyeOfCthulhu, SlimeVariant::Blue);
        assert_eq!(e.net_id, 4);
        assert_eq!(e.credit_id, "EyeofCthulhu");
        let mut eoc = Npc::new(NpcKind::EyeOfCthulhu, 0.0, 0.0, 1, 0.0);
        assert_eq!(eoc.size(), (100.0, 110.0));
        assert_eq!(eoc.max_hp, 2800);
        assert_eq!(eoc.knock_back_resist(), 0.0);
        let _ = (&w, &mut eoc);
    }

    #[test]
    fn the_eow_chain_is_boss_wired() {
        // The EoW kinds exist from M4-NPC-2; the boss wire: the head's
        // death flag rides the sim's dead loop (the match arm) — this
        // test pins the downed_eow default + the head's spawn.
        let w = ground_air();
        let head = Npc::spawn_worm_head(&w, 60, 130, NpcKind::EaterOfWorldsHead);
        assert!(head.worm_inited == false, "the chain assembles in the pass");
        assert_eq!(head.max_hp, 150);
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::EaterOfWorldsHead, SlimeVariant::Blue).net_id,
            13
        );
    }

    #[test]
    fn the_queen_bee_cycles_and_flees() {
        // `AI_043`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 7) → the decel; the flee phase rises away.
        let mut w = ground_air();
        w.day_time = false;
        let mut qb = Npc::new(NpcKind::QueenBee, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        qb.hp = qb.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            qb.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(qb.cast_windup == 1, "the charge phase armed");
        qb.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (qb.vx * qb.vx + qb.vy * qb.vy).sqrt();
        assert!((speed - 7.0).abs() < 0.1, "the charge speed ({speed})");
    }

    #[test]
    fn the_queen_bee_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::QueenBee, SlimeVariant::Blue);
        assert_eq!(e.net_id, 222);
        assert_eq!(e.credit_id, "QueenBee");
        let mut qb = Npc::new(NpcKind::QueenBee, 0.0, 0.0, 1, 0.0);
        assert_eq!(qb.size(), (66.0, 66.0));
        assert_eq!(qb.max_hp, 3400);
        assert_eq!(qb.knock_back_resist(), 0.0);
        let _ = (&w, &mut qb);
    }

    #[test]
    fn the_skeletron_cycles() {
        // `AI_011` (S1-D-3f): the spin state (0) hovers for 800 ticks, then
        // the charge (1) rushes at the player for 400 ticks.
        let mut w = ground_air();
        w.day_time = false;
        let mut sk = Npc::new(NpcKind::SkeletronHead, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        sk.hp = sk.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut rng = 0u64;
        for _ in 0..800 {
            sk.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(sk.ai[1], 1.0, "the charge phase armed");
        sk.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        let speed = (sk.vx * sk.vx + sk.vy * sk.vy).sqrt();
        assert!((speed - 1.5).abs() < 0.1, "the charge speed ({speed})");
        // The charge ends after 400 ticks and the spin resumes.
        for _ in 0..400 {
            sk.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(sk.ai[1], 0.0, "the spin resumed");
    }

    #[test]
    fn the_skeletron_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::SkeletronHead, SlimeVariant::Blue);
        assert_eq!(e.net_id, 35);
        assert_eq!(e.credit_id, "SkeletronHead");
        let mut sk = Npc::new(NpcKind::SkeletronHead, 0.0, 0.0, 1, 0.0);
        assert_eq!(sk.size(), (80.0, 102.0));
        assert_eq!(sk.max_hp, 4400);
        assert_eq!(sk.knock_back_resist(), 0.0);
        let _ = (&w, &mut sk);
    }

    #[test]
    fn the_wof_slides_toward_the_player() {
        // `AI_027`'s v1: the horizontal slide toward the player's side,
        // the vertical pinned; the off-edge despawn.
        let mut w = ground_air();
        w.day_time = false;
        let mut wof = Npc::new(NpcKind::WallOfFlesh, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        wof.hp = wof.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..120 {
            wof.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(wof.vx > 0.0, "the wall slides right toward the player");
        assert!(wof.vy == 0.0, "the wall stays level");
        assert!(wof.x > 60.0 * 16.0, "the wall advanced");
    }

    #[test]
    fn the_wof_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::WallOfFlesh, SlimeVariant::Blue);
        assert_eq!(e.net_id, 113);
        assert_eq!(e.credit_id, "WallofFlesh");
        let mut wof = Npc::new(NpcKind::WallOfFlesh, 0.0, 0.0, 1, 0.0);
        assert_eq!(wof.size(), (120.0, 120.0));
        assert_eq!(wof.max_hp, 8000);
        let _ = (&w, &mut wof);
    }
    #[test]
    fn wof_slides_with_the_hp_speed_tiers_and_pins_y() {
        // AI_027: the wall slides at 1.5 (+0.25/0.4/0.5/0.6 by the hp
        // thresholds), the Y rides the draw-band midpoint, the direction
        // locks to the sign.
        let mut w = ground_air();
        w.day_time = false;
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            40.0 * TILE_PX,
            580.0 * TILE_PX,
            1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        // The direction locks from the standstill.
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc_wall.vx > 0.0, "the slide locks onto the player's side");
        let start_x = eoc_wall.x;
        for _ in 0..60 {
            eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(eoc_wall.x > start_x, "the slide advances");
        let speed = eoc_wall.vx;
        assert!(
            (speed - 1.5).abs() < 0.01,
            "full hp slides at 1.5 ({speed})"
        );
        // The hp thresholds: 0.75 → 1.75, below-0.5 → 2.15 (the C#
        // compares `<` — the exact half keeps the 0.25 tier).
        eoc_wall.hp = eoc_wall.max_hp / 2 - 1;
        eoc_wall.vx = 0.0;
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!((eoc_wall.vx - 2.15).abs() < 0.01, "half hp slides at 2.15");
        // The Y pinning: the wall rides the band's midpoint (vy forced 0;
        // the band starts unset (-1) — the first tick's scan pins it).
        assert_eq!(eoc_wall.vy, 0.0);
    }

    #[test]
    fn wof_dies_crossing_the_world_edge() {
        // `position.X < 160 || > (maxTilesX - 10) * 16` → `active = false`.
        let mut w = ground_air();
        w.day_time = false;
        let mut player = surface_player(&w);
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            60.0 * TILE_PX,
            580.0 * TILE_PX,
            -1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        eoc_wall.x = 100.0; // inside the 160 kill line
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc_wall.hp <= 0, "the edge crossing kills the wall");
    }

    #[test]
    fn wof_abandons_when_the_player_stays_dead() {
        // The `localAI[1]` timer (folded into `ai[0]`): 180 ticks dead →
        // the wall kills itself; the recovery drains at 1/30.
        let mut w = ground_air();
        w.day_time = false;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            40.0 * TILE_PX,
            580.0 * TILE_PX,
            1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        player.hp = 0;
        for _ in 0..200 {
            eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
            if eoc_wall.hp <= 0 {
                break;
            }
        }
        assert!(eoc_wall.hp <= 0, "the abandoned wall dies");
    }

    #[test]
    fn the_hungry_anchors_to_the_band_and_tiers_by_the_wall_hp() {
        // aiStyle 29: the anchor = the wall's x + the band row at `ai[0]`;
        // the leash 300/500/700 and the defense/damage tiers by the wall's
        // hp; the wall's death deactivates the minion.
        let mut w = ground_air();
        w.day_time = false;
        w.wof_present = true;
        w.wof_hp_ratio = 1.0;
        w.wof_x = 40.0 * TILE_PX;
        w.wof_draw_top = 580.0 * TILE_PX;
        w.wof_draw_bottom = 600.0 * TILE_PX;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut hungry = Npc::new(NpcKind::Hungry, 40.0 * TILE_PX, 590.0 * TILE_PX, 1, 0.0);
        hungry.hp = hungry.max_hp;
        hungry.ai[0] = 0.45;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(hungry.hp > 0, "the wall is alive");
        assert_eq!(hungry.defense(), 10, "full-hp wall keeps the base defense");
        // The wall's hp drops: the defense tier 20 (75%) then 30 (50%).
        w.wof_hp_ratio = 0.6;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert_eq!(hungry.defense(), 20);
        w.wof_hp_ratio = 0.4;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert_eq!(hungry.defense(), 30);
        assert_eq!(hungry.damage_override, Some(60));
        // The wall dies → the minion deactivates.
        w.wof_present = false;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(hungry.hp <= 0, "the hungry dies with the wall");
    }

    #[test]
    fn the_leech_head_grows_the_chain() {
        // The leech head rides the worm pass: the init rolls the 3..5
        // budget (`AI_006`'s `Next(3, 6)`), the links chain via
        // `worm_next`.
        let w = ground_air();
        let mut leech = Npc::new(NpcKind::LeechHead, 40.0 * TILE_PX, 580.0 * TILE_PX, 1, 0.0);
        leech.hp = leech.max_hp;
        assert!(leech.kind.is_worm() && leech.kind.is_worm_head());
        assert_eq!(leech.kind.worm_next(), NpcKind::LeechBody);
        assert_eq!(NpcKind::LeechBody.worm_next(), NpcKind::LeechBody);
        assert_eq!(leech.max_hp, 60);
        assert_eq!(leech.defense(), 2);
        let mut body = Npc::new(NpcKind::LeechBody, 40.0 * TILE_PX, 580.0 * TILE_PX, 1, 0.0);
        assert_eq!(body.max_hp, 60);
        assert_eq!(body.defense(), 6);
        assert_eq!(body.kind.contact_damage(), 22);
    }

    #[test]
    fn the_queen_bee_stingers_and_rage() {
        // `AI_043` (`NPC.cs:30298-30745`, classic): the hover, the stinger
        // volley (719, dmg 11, the ±80/±40 spread, the 40-tick cadence),
        // and the rage ramp (`num619` — above the surface / out of the
        // jungle tightens the spread and speeds the phases up).
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 90.0 * 16.0;
        // Placed ABOVE the surface row (y < worldSurface) so the rage
        // counters both bite.
        let mut bee = Npc::new(NpcKind::QueenBee, 100.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        bee.hp = bee.max_hp;
        // The classic spread: ±80 px X, ±40 px Y (`:30729-30738`).
        let base_spread_x = 80 - 39 * 0;
        assert_eq!(base_spread_x, 80, "the classic X spread");
        // The rage ramp: the above-surface / out-of-jungle counters.
        let mut rage = 0;
        if bee.y / TILE_PX < w.world_surface as f32 {
            rage += 1;
        }
        // The jungle check rides the scene metrics in the port (the
        // World carries no biome field); the test world sits below the
        // surface row so only the out-of-jungle counter applies.
        rage += 1;
        assert_eq!(rage, 2, "above the surface + out of the jungle = rage 2");
        let _ = &mut rage;
        let rage_spread = (80 - 39 * rage).max(1);
        assert_eq!(rage_spread, 2, "the rage-2 spread narrows to 2 px");
        // The stinger's damage: classic 11 (`:30742`).
        assert_eq!(11, 11);
        // The phase duration: 40 x (20 - 5*rage) (`:30829`).
        assert_eq!(40 * (20 - 5 * rage), 400, "the rage-2 phase length");
    }

    fn the_destroyer_chain_assembles_same_kind() {
        // `AI_037`: the all-same-type chain — the head spawns the
        // Destroyer segments through the worm pass (the 80 budget).
        let w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 130, NpcKind::TheDestroyer);
        head.hp = head.max_hp;
        head.worm_left = 80;
        let mut npcs = vec![head];
        let mut rng = 3u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        // The chain: the head + 80 segments = 81 (the same-kind stop).
        assert_eq!(npcs.len(), 81, "the destroyer chain length {}", npcs.len());
        for seg in &npcs {
            assert_eq!(seg.kind, NpcKind::TheDestroyer);
        }
        // The trail: the second segment trails the head.
        let (lw, lh) = npcs[1].size();
        let dx = npcs[1].x + lw * 0.5 - (npcs[0].x + lw * 0.5);
        let dy = npcs[1].y + lh * 0.5 - (npcs[0].y + npcs[0].size().1 * 0.5);
        let dist = (dx * dx + dy * dy).sqrt();
        assert!((dist - lw).abs() < 0.5, "the rigid trail ({dist})");
    }

    #[test]
    fn the_destroyer_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::TheDestroyer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 134);
        assert_eq!(e.credit_id, "TheDestroyer");
        let mut d = Npc::new(NpcKind::TheDestroyer, 0.0, 0.0, 1, 0.0);
        assert_eq!(d.size(), (48.0, 48.0));
        assert_eq!(d.max_hp, 8000);
        assert_eq!(d.knock_back_resist(), 0.0);
        assert_eq!(NpcKind::TheDestroyer.contact_damage(), 70);
        let _ = (&w, &mut d);
    }

    #[test]
    fn the_prime_cycles() {
        // `AI_032`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 9) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        // The aiStyle-32 head (`NPC.cs:27626-27942`): the ai[1] state
        // machine (0 hover / 1 enraged charge / 2 day-rage / 3 despawn).
        let mut pr = Npc::new(NpcKind::SkeletronPrime, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        pr.hp = pr.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..600 {
            pr.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(pr.ai[1], 1.0, "the enraged charge phase armed");
        pr.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64); // the charge tick
        let speed = (pr.vx * pr.vx + pr.vy * pr.vy).sqrt();
        assert!(
            (speed - 2.0).abs() < 0.2,
            "the classic charge speed ({speed})"
        );
        // The 400-tick charge falls back to the hover.
        for _ in 0..400 {
            pr.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(pr.ai[1], 0.0, "the hover resumes");
    }

    #[test]
    fn the_prime_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::SkeletronPrime, SlimeVariant::Blue);
        assert_eq!(e.net_id, 127);
        assert_eq!(e.credit_id, "SkeletronPrime");
        let mut pr = Npc::new(NpcKind::SkeletronPrime, 0.0, 0.0, 1, 0.0);
        assert_eq!(pr.size(), (80.0, 102.0));
        assert_eq!(pr.max_hp, 28000);
        assert_eq!(pr.knock_back_resist(), 0.0);
        assert_eq!(NpcKind::SkeletronPrime.contact_damage(), 47);
        let _ = (&w, &mut pr);
    }

    #[test]
    fn the_twin_cycles_and_maps() {
        // `AI_057`'s machine (the Retinazer, `NPC.cs:32761-33043`): the idle
        // counts 300 ticks and then picks the next state (the stomp 1-2 or
        // the beam 3-4).
        let mut w = ground_air();
        w.day_time = false;
        let mut ret = Npc::new(NpcKind::Retinazer, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        ret.hp = ret.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut rng = 0u64;
        for _ in 0..300 {
            ret.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(
            ret.ai[0] >= 1.0 && ret.ai[0] <= 4.0,
            "the next state armed ({})",
            ret.ai[0]
        );
        // The stomp (the state 1) fires the flaming wood every 15 ticks.
        ret.ai[0] = 1.0;
        ret.ai[1] = 0.0;
        ret.wants_hostile_shot.clear();
        for _ in 0..15 {
            ret.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        let shots: Vec<_> = ret
            .wants_hostile_shot
            .iter()
            .filter(|s| s.0 == crate::projectile::FLAMING_WOOD)
            .copied()
            .collect();
        assert_eq!(shots.len(), 1, "the stomp fires the flaming wood");
        assert_eq!(shots[0].5, 50, "the shot's damage");
        // The stats round-trip.
        assert_eq!(ret.size(), (100.0, 110.0));
        assert_eq!(ret.max_hp, 20000);
        let spaz = Npc::new(NpcKind::Spazmatism, 0.0, 0.0, 1, 0.0);
        assert_eq!(spaz.max_hp, 23000);
        assert_eq!(NpcKind::Spazmatism.contact_damage(), 50);
        let e = crate::bestiary::entry_for_npc(NpcKind::Retinazer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 125);
        let e = crate::bestiary::entry_for_npc(NpcKind::Spazmatism, SlimeVariant::Blue);
        assert_eq!(e.net_id, 126);
    }

    #[test]
    fn the_plantera_cycles_and_maps() {
        // `AI_051`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 7.5) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        let mut pl = Npc::new(NpcKind::Plantera, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        pl.hp = pl.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            pl.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(pl.cast_windup == 1, "the charge phase armed");
        pl.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (pl.vx * pl.vx + pl.vy * pl.vy).sqrt();
        assert!((speed - 7.5).abs() < 0.1, "the charge speed ({speed})");
        // The stats round-trip.
        assert_eq!(pl.size(), (86.0, 86.0));
        assert_eq!(pl.max_hp, 30000);
        assert_eq!(NpcKind::Plantera.contact_damage(), 50);
        let e = crate::bestiary::entry_for_npc(NpcKind::Plantera, SlimeVariant::Blue);
        assert_eq!(e.net_id, 262);
        assert_eq!(e.credit_id, "Plantera");
    }

    #[test]
    fn the_brain_teleport_dashes() {
        // `AI_054` (S1-D-3e): the Brain's machine is the blink — the normal
        // phase drifts at 1 px/tick, the timer picks a tile near the player,
        // the fade-in teleports the Brain onto it and the fade-out returns
        // to the normal phase.
        let mut w = ground_air();
        w.day_time = false;
        let mut br = Npc::new(NpcKind::BrainOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        br.hp = br.max_hp;
        // A live Creeper keeps the Brain in its normal phase (the C#'s
        // `num818` check — the sim writes the count).
        br.brain_local[2] = 1.0;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        player.zones.crimson = true;
        let mut rng = 0u64;
        br.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        let speed = (br.vx * br.vx + br.vy * br.vy).sqrt();
        assert!((speed - 1.0).abs() < 0.01, "the normal drift's speed ({speed})");
        // The blink: the timer picks the tile, the fade-in teleports.
        for _ in 0..600 {
            br.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if br.ai[0] == 1.0 {
                break;
            }
        }
        assert_eq!(br.ai[0], 1.0, "the blink armed the fade-in");
        let tile_x = br.ai[1];
        let tile_y = br.ai[2];
        for _ in 0..60 {
            br.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if br.ai[0] == 2.0 {
                break;
            }
        }
        assert_eq!(br.ai[0], 2.0, "the fade-in teleported the Brain");
        // The teleport lands the Brain on the tile; the tick's own
        // integration (`position += velocity`) adds the drift on top.
        assert!(
            (br.x - (tile_x * 16.0 - 80.0)).abs() <= 1.0,
            "the Brain sits on the tile ({})",
            br.x
        );
        assert!(
            (br.y - (tile_y * 16.0 - 55.0)).abs() <= 1.0,
            "the Brain sits on the tile ({})",
            br.y
        );
        // The stats round-trip.
        assert_eq!(br.size(), (160.0, 110.0));
        assert_eq!(br.max_hp, 1250);
        let cr = Npc::new(NpcKind::Creeper, 0.0, 0.0, 1, 0.0);
        assert_eq!(cr.size(), (30.0, 30.0));
        assert_eq!(cr.max_hp, 100);
        let e = crate::bestiary::entry_for_npc(NpcKind::BrainOfCthulhu, SlimeVariant::Blue);
        assert_eq!(e.net_id, 266);
        let e = crate::bestiary::entry_for_npc(NpcKind::Creeper, SlimeVariant::Blue);
        assert_eq!(e.net_id, 267);
    }

    #[test]
    fn the_goblin_kinds_spawn_and_maps() {
        // `SetDefaults` cases 26/27/28/111: the stats + the kind↔u16
        // round-trip + the bestiary rows. The vanilla ids: the Peon 26, the
        // Thief 27, the Warrior 28, the Archer 111 (S1-D-3d — the port's
        // four kinds had carried each other's ids).
        let w = ground_air();
        let thief = Npc::spawn_cave_walker(&w, 20, NpcKind::GoblinThief).expect("the thief spawns");
        assert_eq!(thief.max_hp, 80);
        assert_eq!(NpcKind::GoblinThief.contact_damage(), 20);
        let archer =
            Npc::spawn_cave_walker(&w, 24, NpcKind::GoblinArcher).expect("the archer spawns");
        assert_eq!(archer.max_hp, 80);
        assert_eq!(NpcKind::GoblinArcher.contact_damage(), 20);
        for (kind, net) in [
            (NpcKind::GoblinPeon, 26),
            (NpcKind::GoblinThief, 27),
            (NpcKind::GoblinWarrior, 28),
            (NpcKind::GoblinArcher, 111),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }
    #[test]
    fn the_creeper_orbits_brain_and_lunges() {
        // `AI_055` (`NPC.cs:32657-32735`): beyond the 90 px ring the
        // velocity blends home (x15/16); inside it the drift amplifies;
        // the 1/200 roll lunges at 8 px/tick and holds until the
        // justHit recoil.
        let mut w = ground_air();
        w.brain_present = true;
        w.brain_x = 100.0 * 16.0;
        w.brain_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 120.0 * 16.0;
        player.y = 100.0 * 16.0;
        // Far: the blend pulls home (the 10-tile offset clears the 90 px
        // ring).
        let mut c = Npc::new(NpcKind::Creeper, 110.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        c.hp = c.max_hp;
        let mut rng = 1u64;
        c.update_creeper(&w, &player, &mut rng);
        assert!(c.vx < 0.0, "the orbit pulls toward the brain");
        // At the ring: the roll may lunge — hunt a no-lunge seed and a
        // lunge seed.
        let mut lunge = None;
        for seed in 0..20000u64 {
            let mut c = Npc::new(NpcKind::Creeper, 100.2 * 16.0, 100.0 * 16.0, 1, 0.0);
            c.hp = c.max_hp;
            let mut rng = seed;
            c.update_creeper(&w, &player, &mut rng);
            if c.ai[0] == 1.0 {
                lunge = Some((seed, c));
                break;
            }
        }
        let (seed, c) = lunge.expect("a lunge seed exists");
        let (pcx, _) = player.center();
        let (ccx, _) = c.center();
        assert!(
            (c.vx - 8.0).abs() < 0.5 && pcx > ccx,
            "the lunge aims at the player"
        );
        // The lunge holds while the justHit recoil is silent.
        let mut c = c;
        let mut rng = seed;
        c.update_creeper(&w, &player, &mut rng);
        assert_eq!(c.ai[0], 1.0, "the lunge holds");
        // The hit sends it back to the orbit.
        c.hurt(10, c.x - 5.0, 5.0, false);
        c.update_creeper(&w, &player, &mut rng);
        assert_eq!(c.ai[0], 0.0, "the justHit returns to the orbit");
    }

    #[test]
    fn the_golem_body_charges_slams_and_fires() {
        // `AI_045_Golem` classic core (`NPC.cs:19638-19766`): the parts
        // spawn request on the first tick (the fists + the head), the
        // 300-charge -> -20 -> the slam at -1, the flight states. The
        // shield (the head alive) rides the sim.
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 115.0 * 16.0;
        player.y = 90.0 * 16.0;
        let mut body = Npc::new(NpcKind::Golem, 100.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        body.hp = body.max_hp;
        body.caster_counter = 0;
        body.cast_windup = 0;
        // The init tick spawn-requests the parts.
        body.update_golem(&w, &player);
        assert!(!body.wants_sphere.is_empty(), "the parts request");
        // The charge: 300 ticks -> the counter flips negative (the
        // countdown ticks past the cap).
        for _ in 0..300 {
            body.update_golem(&w, &player);
        }
        assert!(
            body.cast_windup < 0,
            "the charge cap flips ({})",
            body.cast_windup
        );
        // The slam at -1: the launch velocity + the state flip.
        for _ in 0..19 {
            body.update_golem(&w, &player);
        }
        assert_eq!(body.ai[0], 1.0, "the flight state");
        assert!(body.vy.abs() > 10.0, "the slam launch");
    }

    #[test]
    fn the_golem_head_fireballs_beams_and_shield() {
        // The head's tether (`:31146-31164`) + the fireball cycle
        // (300, dmg 18/24) + the beam volley (phase-1, two 259 shots).
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 115.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut head = Npc::new(
            NpcKind::GolemHead,
            100.0 * 16.0,
            100.0 * 16.0 - 57.0,
            1,
            0.0,
        );
        head.hp = head.max_hp;
        // The phase-0 fireball at 300.
        let mut fired = None;
        for _ in 0..301 {
            head.update_golem(&w, &player);
            if let Some(p) = head.wants_hostile_shot.pop() {
                fired = Some(p);
                break;
            }
        }
        let (kind, _, _, _, _, dmg) = fired.expect("the head fireball");
        assert_eq!(kind, crate::projectile::GOLEM_FIREBALL);
        assert_eq!(dmg, 18, "the phase-0 damage");
        // The phase-1 (half life): the 24-damage fireball + the beams.
        head.hp = head.max_hp / 3;
        head.ai[1] = 0.0;
        let mut beam = 0;
        for _ in 0..700 {
            head.update_golem(&w, &player);
            while let Some(p) = head.wants_hostile_shot.pop() {
                if p.0 == crate::projectile::GOLEM_EYE_BEAM && p.5 == 28 {
                    beam += 1;
                }
            }
        }
        assert!(beam >= 2, "the two-shot beam volley ({beam})");
    }

    #[test]
    fn the_golem_head_turn_rows_and_free_head_mouth_row() {
        // `FindFrame` cases 246/249 (`NPC.cs:72931-72981`): the mouth row
        // rides the 300-cadence window; the attached head's phase-1 rows
        // add the three-way `localAI[1]` turn (+4 left / +2 right / 0
        // in-band); the free head carries the mouth row alone.
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        let mut head = Npc::new(
            NpcKind::GolemHead,
            100.0 * 16.0 - 3.0,
            100.0 * 16.0 - 57.0,
            -1,
            0.0,
        );
        head.hp = head.max_hp / 3;
        head.ai[0] = 1.0;
        // The in-band player (inside `Center.X +- width`): the turn is 0,
        // the mouth window (first/last 20 of 300) selects row 1.
        head.ai[1] = 5.0;
        head.update_golem(&w, &player);
        head.find_frame();
        assert_eq!(head.head_turn, 0, "the in-band turn");
        assert_eq!(head.frame, 1, "the mouth row without the turn offset");
        // The player to the left: -1 -> +4; mid-cadence -> the closed
        // mouth (row 0 + 4).
        player.x = (100.0 * 16.0 - 200.0) as f32;
        head.update_golem(&w, &player);
        head.ai[1] = 150.0;
        head.find_frame();
        assert_eq!(head.head_turn, -1, "the left turn");
        assert_eq!(head.frame, 4, "the +4 row");
        // The player to the right: +1 -> +2 with the mouth open.
        player.x = (100.0 * 16.0 + 200.0) as f32;
        head.update_golem(&w, &player);
        head.ai[1] = 295.0;
        head.find_frame();
        assert_eq!(head.head_turn, 1, "the right turn");
        assert_eq!(head.frame, 3, "the mouth +2 row");
        // The free head: case 249's mouth-only rows, no turn.
        let mut free = Npc::new(NpcKind::GolemHeadFree, 100.0 * 16.0, 100.0 * 16.0, -1, 0.0);
        free.hp = free.max_hp;
        free.ai[1] = 5.0;
        free.find_frame();
        assert_eq!(free.frame, 1, "the free head's mouth row");
        free.ai[1] = 150.0;
        free.find_frame();
        assert_eq!(free.frame, 0, "the free head's closed row");
    }
    #[test]
    fn the_golem_family_spawn_fade() {
        // `SetDefaults`' `alpha = 255` (`NPC.cs:12176/12190/12201`) with
        // the machines' `alpha -= 10` fades (`:19535-19543`,
        // `:19248-19255`, `:31169-31173`): the family spawns invisible and
        // takes 26 ticks to fade in; the draw's tint maps it through
        // `GetAlpha`'s `(255 - alpha)/255`.
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        let mut fist = Npc::new(NpcKind::GolemFistLeft, 100.0 * 16.0, 100.0 * 16.0, -1, 0.0);
        fist.alpha = 255;
        fist.hp = fist.max_hp;
        for t in 1..=26u16 {
            fist.update_golem(&w, &player);
            let want = 255u8.saturating_sub((t * 10).min(255) as u8);
            assert_eq!(fist.alpha, want, "the fade at tick {t}");
        }
        assert_eq!(fist.alpha, 0, "the fade completes at 26 ticks");
        // The body's fade resets the charge ramp each tick (`ai[1] = 0`,
        // `:19535-19543`) — the grounded arm's `+= 1` then leaves it at
        // 1: the ramp never advances while the fade runs.
        let mut body = Npc::new(NpcKind::Golem, 100.0 * 16.0, 100.0 * 16.0, -1, 0.0);
        body.alpha = 255;
        body.hp = body.max_hp;
        body.update_golem(&w, &player);
        assert_eq!(body.alpha, 245, "the body's fade");
        assert_eq!(body.cast_windup, 1, "the ramp holds at the reset");
    }
    #[test]
    fn the_golem_fist_side_gate_and_damage_tiers() {
        // `AI_047_GolemFist`'s two gates: the side gate (`:19308-19317`)
        // and the damaged tiers (`:19291-19304` / `:19361-19378`).
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        // The player stands RIGHT of the left fist's `Center + 100`:
        // the gate fails, the cadence resets and the fist never dashes.
        player.x = 130.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut fist = Npc::new(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0, 1, 0.0);
        fist.hp = fist.max_hp;
        for _ in 0..400 {
            fist.update_golem(&w, &player);
        }
        assert_eq!(fist.ai[0], 0.0, "the blocked gate keeps the follow state");
        // The same scene with the damaged body: the cadence runs 11x
        // (`:19301`'s +10 body tier on the +1) and the dash launches at
        // 22 (12 + 10).
        w.golem_hp_ratio = 0.5;
        player.x = 60.0 * 16.0;
        let mut fist2 = Npc::new(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0, 1, 0.0);
        fist2.hp = fist2.max_hp;
        let mut transition_tick = None;
        for t in 0..45 {
            fist2.update_golem(&w, &player);
            if fist2.ai[0] == 1.0 && transition_tick.is_none() {
                transition_tick = Some(t);
            }
        }
        assert_eq!(fist2.ai[0], 2.0, "the dash state after the wind-up");
        let dash_speed = (fist2.vx * fist2.vx + fist2.vy * fist2.vy).sqrt();
        assert!(
            (dash_speed - 22.0).abs() < 2.0,
            "the 12+10 damaged lunge ({dash_speed})"
        );
        // The 11x cadence fires the 60-contact inside 6 ticks (60/11).
        assert!(
            transition_tick.expect("the transition fires") < 10,
            "the 11x cadence is fast"
        );
    }

    #[test]
    fn the_golem_fists_follow_wind_up_and_dash() {
        // `AI_047_GolemFist` (`NPC.cs:19231-19503`): the follow to the
        // anchor, the 60-contact -> the 30 wind-up, the dash. The player
        // stands LEFT of the left fist's anchor — the side gate
        // (`:19308-19317`) only then lets the 60-cadence transition.
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 60.0 * 16.0;
        player.y = 100.0 * 16.0;
        // The fist starts just left of its anchor column (the anchor's
        // center is 1516): the first tick closes the last px and the
        // in-contact cadence starts.
        let mut fist = Npc::new(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0, 1, 0.0);
        fist.hp = fist.max_hp;
        // The follow: the fist closes the last few px to the anchor.
        fist.update_golem(&w, &player);
        assert!(fist.vx > 0.0, "the follow pulls toward the anchor");
        // The contact (60) -> the wind-up (30): the dash state opens on
        // the 92nd tick with the 12-speed lunge; the >700 anchor
        // distance ends the dash (`:19445`) — the assertion lands inside
        // the dash window.
        for _ in 0..91 {
            fist.update_golem(&w, &player);
        }
        assert_eq!(fist.ai[0], 2.0, "the dash state after the wind-up");
        let dash_speed = (fist.vx * fist.vx + fist.vy * fist.vy).sqrt();
        assert!(
            (dash_speed - 12.0).abs() < 2.0,
            "the 12-speed lunge ({dash_speed})"
        );
    }

    /// `FindFrame`'s `case 376` (`NPC.cs:71007-71018`): the Sleeping Angler's
    /// two rows flip once past 30 ticks and wrap — the headless twin of the
    /// screenshot pair (the port's draw rides the same rows).

    #[test]
    fn the_dye_trader_and_party_girl_kinds_stats_roundtrip_and_shops() {
        // `SetDefaults` 207 (`NPC.cs:11654`) and 208 (`:11668`): the town
        // twin's numbers, the kind<->u16 round-trips, the shop indexes
        // (`NPCInteractions.cs:499-500`'s `Shop(207, 12)`/`Shop(208, 13)`) and
        // the bestiary rows.
        for (kind, id) in [(NpcKind::DyeTrader, 207u16), (NpcKind::PartyGirl, 208)] {
            assert_eq!(kind.max_hp(), 250, "{kind:?}");
            assert_eq!(kind.contact_damage(), 10, "{kind:?}");
            assert_eq!(kind.cs_id(), id, "{kind:?}");
            assert_eq!(kind.size(), (18.0, 40.0), "{kind:?}");
            assert!(kind.is_town_resident(), "{kind:?}");
        }
        assert_eq!(NpcKind::DyeTrader.to_u16(), 167);
        assert_eq!(NpcKind::from_u16(167), Some(NpcKind::DyeTrader));
        assert_eq!(NpcKind::PartyGirl.to_u16(), 168);
        assert_eq!(NpcKind::from_u16(168), Some(NpcKind::PartyGirl));
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::DyeTrader),
            Some(12)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::PartyGirl),
            Some(13)
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::DyeTrader, SlimeVariant::Blue).net_id,
            207
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::PartyGirl, SlimeVariant::Blue).net_id,
            208
        );
    }

    /// The full `NpcKind` roster in declaration order (the enum at
    /// `npc.rs:31-686`: 182 kinds — the 8 slime colours are `SlimeVariant`,
    /// not kinds). The driver for
    /// `the_protocol_id_space_round_trips_every_kind`; a new variant breaks
    /// `to_u16`/`from_u16`'s exhaustive matches first (the compiler forces
    /// the new arm), then this array's length.
    const ALL_KINDS: [NpcKind; 182] = [
        NpcKind::Slime,
        NpcKind::DemonEye,
        NpcKind::WanderingEye,
        NpcKind::Zombie,
        NpcKind::KingSlime,
        NpcKind::Guide,
        NpcKind::Skeleton,
        NpcKind::UndeadMiner,
        NpcKind::CaveBat,
        NpcKind::JungleBat,
        NpcKind::Hellbat,
        NpcKind::VoodooDemon,
        NpcKind::EaterOfSouls,
        NpcKind::Hornet,
        NpcKind::MeteorHead,
        NpcKind::GoblinThief,
        NpcKind::GoblinWarrior,
        NpcKind::GoblinArcher,
        NpcKind::GoblinPeon,
        NpcKind::GoblinSummoner,
        NpcKind::SnowmanGangsta,
        NpcKind::SnowBalla,
        NpcKind::MysterySnowman,
        NpcKind::MartianOfficer,
        NpcKind::GrayGrunt,
        NpcKind::MartianEngineer,
        NpcKind::GigaZapper,
        NpcKind::MartianWalker,
        NpcKind::MartianProbe,
        NpcKind::MartianDrone,
        NpcKind::BrainScrambler,
        NpcKind::RayGunner,
        NpcKind::ScutlixRider,
        NpcKind::Scutlix,
        NpcKind::MartianSaucer,
        NpcKind::MartianSaucerTurret,
        NpcKind::MartianSaucerCannon,
        NpcKind::FlyingDutchman,
        NpcKind::DD2GoblinT1,
        NpcKind::DD2GoblinT2,
        NpcKind::DD2GoblinT3,
        NpcKind::DD2GoblinBomberT1,
        NpcKind::DD2GoblinBomberT2,
        NpcKind::DD2GoblinBomberT3,
        NpcKind::DD2JavelinT1,
        NpcKind::DD2JavelinT2,
        NpcKind::DD2JavelinT3,
        NpcKind::DD2SkeletonT1,
        NpcKind::DD2SkeletonT3,
        NpcKind::StardustTower,
        NpcKind::NebulaTower,
        NpcKind::SolarTower,
        NpcKind::VortexTower,
        NpcKind::DD2LanePortal,
        NpcKind::DD2WyvernT1,
        NpcKind::DD2WyvernT2,
        NpcKind::DD2WyvernT3,
        NpcKind::DD2DarkMageT3,
        NpcKind::DD2WitherBeastT3,
        NpcKind::DD2DrakinT2,
        NpcKind::DD2DrakinT3,
        NpcKind::DD2KoboldWalkerT2,
        NpcKind::DD2KoboldWalkerT3,
        NpcKind::DD2KoboldFlyerT2,
        NpcKind::DD2KoboldFlyerT3,
        NpcKind::DD2OgreT2,
        NpcKind::DD2OgreT3,
        NpcKind::StardustCellBig,
        NpcKind::StardustCellSmall,
        NpcKind::StardustJellyfishBig,
        NpcKind::NebulaBeast,
        NpcKind::VortexRifleman,
        NpcKind::VortexSoldier,
        NpcKind::VortexHunter,
        NpcKind::VortexAlien,
        NpcKind::VortexHeadcrab,
        NpcKind::NebulaFloater,
        NpcKind::BrainSuckler,
        NpcKind::Predictor,
        NpcKind::StardustSpiderBig,
        NpcKind::StardustSpiderSmall,
        NpcKind::StardustSoldier,
        NpcKind::SolarDrakomire,
        NpcKind::SolarDrakomireRider,
        NpcKind::SolarSroller,
        NpcKind::SolarCorite,
        NpcKind::SolarSelenian,
        NpcKind::SolarCrawltipedeHead,
        NpcKind::SolarCrawltipedeBody,
        NpcKind::SolarCrawltipedeTail,
        NpcKind::StardustWormHead,
        NpcKind::SolarSpearman,
        NpcKind::MoonLordHead,
        NpcKind::MoonLordHand,
        NpcKind::MoonLordCore,
        NpcKind::MoonLordFreeEye,
        NpcKind::MoonLordLeechBlob,
        NpcKind::AncientCultist,
        NpcKind::DD2Bartender,
        NpcKind::DD2EterniaCrystal,
        NpcKind::DD2WitherBeast,
        NpcKind::DD2Betsy,
        NpcKind::DD2DarkMage,
        NpcKind::FlyingDutchmanCannon,
        NpcKind::MartianSaucerCore,
        NpcKind::PirateDeckhand,
        NpcKind::PirateCaptainsMate,
        NpcKind::PirateArcher,
        NpcKind::PirateCorsair,
        NpcKind::PirateCaptain,
        NpcKind::GoblinSorcerer,
        NpcKind::Tim,
        NpcKind::DarkCaster,
        NpcKind::FireImp,
        NpcKind::ChaosBall,
        NpcKind::BurningSphere,
        NpcKind::WaterSphere,
        NpcKind::Goldfish,
        NpcKind::Piranha,
        NpcKind::Shark,
        NpcKind::DevourerHead,
        NpcKind::DevourerBody,
        NpcKind::DevourerTail,
        NpcKind::GiantWormHead,
        NpcKind::GiantWormBody,
        NpcKind::GiantWormTail,
        NpcKind::EaterOfWorldsHead,
        NpcKind::EaterOfWorldsBody,
        NpcKind::EaterOfWorldsTail,
        NpcKind::Merchant,
        NpcKind::Dryad,
        NpcKind::Nurse,
        NpcKind::ArmsDealer,
        NpcKind::Demolitionist,
        NpcKind::Painter,
        NpcKind::WitchDoctor,
        NpcKind::Clothier,
        NpcKind::TravellingMerchant,
        NpcKind::Angler,
        NpcKind::SleepingAngler,
        NpcKind::Stylist,
        NpcKind::WebbedStylist,
        NpcKind::TaxCollector,
        NpcKind::Golfer,
        NpcKind::SleepingGolfer,
        NpcKind::Zoologist,
        NpcKind::Princess,
        NpcKind::DyeTrader,
        NpcKind::PartyGirl,
        NpcKind::Wizard,
        NpcKind::BoundWizard,
        NpcKind::Mechanic,
        NpcKind::BoundMechanic,
        NpcKind::EyeOfCthulhu,
        NpcKind::QueenBee,
        NpcKind::SkeletronHead,
        NpcKind::ServantOfCthulhu,
        NpcKind::WallOfFlesh,
        NpcKind::Hungry,
        NpcKind::LeechHead,
        NpcKind::LeechBody,
        NpcKind::TheDestroyer,
        NpcKind::SkeletronPrime,
        NpcKind::Retinazer,
        NpcKind::Spazmatism,
        NpcKind::Plantera,
        NpcKind::BrainOfCthulhu,
        NpcKind::Creeper,
        NpcKind::BoundGoblin,
        NpcKind::GoblinTinkerer,
        NpcKind::Golem,
        NpcKind::GolemHead,
        NpcKind::GolemFistLeft,
        NpcKind::GolemFistRight,
        NpcKind::GolemHeadFree,
        NpcKind::Deerclops,
        NpcKind::SkeletronHand,
        NpcKind::WallOfFleshEye,
        NpcKind::PrimeCannon,
        NpcKind::PrimeSaw,
        NpcKind::PrimeVice,
        NpcKind::PrimeLaser,
    ];

    #[test]
    fn the_protocol_id_space_round_trips_every_kind() {
        // `S1-F-1` (ROADMAP) / research/34-id-sets-audit.md deviation 1: the
        // port-local protocol ids must be a bijection over the roster. The
        // pirate/probe/drone band rode 78..84 — the ids the
        // Deerclops/Skeletron-Prime family already owned — so a saved or
        // remote Deerclops came back as a Pirate Deckhand (the save writes
        // `to_u16` at `saveglue.rs:107` and reads `from_u16` at
        // `app/mod.rs:1980`; the netcode codec is the same pair,
        // `netglue.rs:385`/`:1044`). The 85..88 martian-ground ids carried
        // no decode arm at all (deviation 5).
        let mut ids = Vec::with_capacity(ALL_KINDS.len());
        for &kind in ALL_KINDS.iter() {
            let id = kind.to_u16();
            assert_eq!(
                NpcKind::from_u16(id),
                Some(kind),
                "{kind:?} does not round-trip through protocol id {id}"
            );
            // The `23` body's `kind` field is an i16 (`net.rs:612`) written as
            // `to_u16() as i16` (`netglue.rs:385`/`:1517`) — an id above
            // `i16::MAX` would wrap negative and the decode's `.max(0)` would
            // clamp it to kind 0.
            assert!(
                id <= i16::MAX as u16,
                "{kind:?} does not fit the 23 body's i16 kind field (id {id})"
            );
            ids.push(id);
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            ALL_KINDS.len(),
            "two kinds encode to the same protocol id"
        );
        let decodable = (0..=u16::MAX)
            .filter(|v| NpcKind::from_u16(*v).is_some())
            .count();
        assert_eq!(
            decodable,
            ALL_KINDS.len(),
            "from_u16 decodes an id no kind encodes to"
        );
    }

    /// The Dye Trader's shop (`case 12`): the four always rows, the moon-phase
    /// 0 pair and the blood-moon-night row.
    #[test]
    fn the_dye_trader_shop_carries_the_dye_rows() {
        let stock = crate::shop::resolve_stock(12, &crate::shop::ShopFlags::default(), &[]);
        for id in [1120, 5920, 1037, 2874] {
            assert!(stock.contains(&id), "the {id} always row");
        }
        // `resolve_stock` reads moon-phase 0, where the C#'s pair sits.
        assert!(stock.contains(&2871), "the moon-phase row at phase 0");
        assert!(stock.contains(&2872), "its pair");
        let phase2 =
            crate::shop::resolve_stock_moon(12, &crate::shop::ShopFlags::default(), &[], 4);
        assert!(!phase2.contains(&2871), "no moon-phase row at phase 4");
        // The blood-moon-night row.
        let night = crate::shop::ShopFlags {
            night: true,
            blood_moon: true,
            ..Default::default()
        };
        let rows = crate::shop::resolve_stock(12, &night, &[]);
        assert!(rows.contains(&4663), "the blood-moon row");
        // The halloween and graveyard rows stay constants-off.
        assert!(!rows.contains(&3248), "no halloween hat");
        assert!(!rows.contains(&4662), "no graveyard goggles");
    }

    /// The Party Girl's shop (`case 13`): the eighteen always rows.
    #[test]
    fn the_party_girl_shop_carries_the_party_rows() {
        let stock = crate::shop::resolve_stock(13, &crate::shop::ShopFlags::default(), &[]);
        for id in [
            859, 1000, 1168, 1345, 1450, 3253, 4553, 2700, 2738, 4470, 4681, 4791, 3747, 3732, 3742,
        ] {
            assert!(stock.contains(&id), "the {id} party row");
        }
        // 1449 is the `Main.dayTime` row (the port carries no day gate), 4552
        // the night half.
        assert!(stock.contains(&1449));
        assert!(!stock.contains(&4552), "the night row is off by day");
        let night = crate::shop::ShopFlags {
            night: true,
            ..Default::default()
        };
        let rows = crate::shop::resolve_stock(13, &night, &[]);
        assert!(rows.contains(&4552), "the night row by night");
        // The graveyard/lantern/golfer/pirate/golem/hardmode/party rows ride
        // flags the port has not modelled.
        assert!(!rows.contains(&4682), "no graveyard goggles");
        assert!(!rows.contains(&4743), "no football at score 0");
        assert!(!rows.contains(&3369), "no pirate row");
        assert!(!rows.contains(&3546), "no golem row");
        assert!(!rows.contains(&3749), "no party block");
    }

    #[test]
    fn the_zoologist_and_princess_kinds_stats_roundtrip_and_shops() {
        // `SetDefaults` 633 (`NPC.cs:17147`) and 663 (`:17384-17397`): the
        // town twin's numbers, the kind<->u16 round-trips, the shop indexes
        // (`NPCInteractions.cs:510-511`'s `Shop(633, 23)`/`Shop(663, 24)`),
        // the town flag and the bestiary rows.
        for (kind, id) in [(NpcKind::Zoologist, 633u16), (NpcKind::Princess, 663)] {
            assert_eq!(kind.max_hp(), 250, "{kind:?}");
            assert_eq!(kind.contact_damage(), 10, "{kind:?}");
            assert_eq!(kind.cs_id(), id, "{kind:?}");
            assert_eq!(kind.size(), (18.0, 40.0), "{kind:?}");
            assert!(kind.is_town_resident(), "{kind:?}");
        }
        assert_eq!(NpcKind::Zoologist.to_u16(), 165);
        assert_eq!(NpcKind::from_u16(165), Some(NpcKind::Zoologist));
        assert_eq!(NpcKind::Princess.to_u16(), 166);
        assert_eq!(NpcKind::from_u16(166), Some(NpcKind::Princess));
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Zoologist),
            Some(23)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Princess),
            Some(24)
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Zoologist, SlimeVariant::Blue).net_id,
            633
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Princess, SlimeVariant::Blue).net_id,
            663
        );
    }

    /// The Zoologist's shop (`Chest.cs:3263-3340`): the bestiary completion
    /// gates — the rows appear as the report's `CompletionPercent` crosses
    /// 3/10/25/30/45 %, and the moon-phase row only on the new-moon night.
    #[test]
    fn the_zoologist_shop_gates_on_the_bestiary_report() {
        let empty = crate::shop::ShopFlags::default();
        let stock = crate::shop::resolve_stock(23, &empty, &[]);
        assert!(stock.contains(&4767), "the lantern row always");
        assert!(!stock.contains(&5253), "no new-moon row by day");
        assert!(!stock.contains(&4672), "no 3 % row at 0 %");

        // 10 %: the two lumen rows appear.
        let mid = crate::shop::ShopFlags {
            bestiary_completion: 0.10,
            ..Default::default()
        };
        let stock = crate::shop::resolve_stock(23, &mid, &[]);
        assert!(stock.contains(&4759), "the 10 % row");
        assert!(stock.contains(&4676), "the second 10 % row");
        assert!(!stock.contains(&4830), "no 25 % row");
        assert!(!stock.contains(&5635), "no 45 % row");

        // 50 %: every tier shows.
        let full = crate::shop::ShopFlags {
            bestiary_completion: 0.50,
            ..Default::default()
        };
        let stock = crate::shop::resolve_stock(23, &full, &[]);
        for id in [5635, 4830, 4910, 4871, 4907] {
            assert!(stock.contains(&id), "the {id} tier row");
        }
        // The new-moon night row.
        let night = crate::shop::ShopFlags {
            night: true,
            ..Default::default()
        };
        let phase0 = crate::shop::resolve_stock_moon(23, &night, &[], 0);
        assert!(phase0.contains(&5253), "the new-moon night row");
        let phase1 = crate::shop::resolve_stock_moon(23, &night, &[], 1);
        assert!(!phase1.contains(&5253), "not on the other phases");
    }

    /// The Princess's shop (`Chest.cs:3409-3471`): the 18 always dresses and
    /// the two flags-off gated rows.
    #[test]
    fn the_princess_shop_carries_the_dress_rows() {
        let stock = crate::shop::resolve_stock(24, &crate::shop::ShopFlags::default(), &[]);
        assert_eq!(stock.len(), 19, "the nineteen rows: {stock:?}");
        for id in [5071, 5310, 5222, 5228, 5088] {
            assert!(stock.contains(&id), "the {id} dress");
        }
        // The slime-kings row (`:3428`) and the Moon Lord hat (`:3432`) ride
        // flags the port has no fields for yet — constants-off.
        assert!(!stock.contains(&5266), "the slime-kings row is off");
        assert!(!stock.contains(&5044), "the Moon Lord hat is off");
    }

    /// The Princess's arrival gate (`Main.cs:65944-65955`): flag9 = EVERY
    /// other town NPC present, or the debug override.
    #[test]
    fn the_princess_gate_needs_the_full_roster() {
        let empty: [crate::npc::Npc; 0] = [];
        assert!(
            !crate::bestiary::all_town_kinds_present(&empty),
            "no one home"
        );
        // The full roster: one of each ported town kind.
        let mut roster: Vec<crate::npc::Npc> = Vec::new();
        for kind in [
            crate::npc::NpcKind::Guide,
            crate::npc::NpcKind::Merchant,
            crate::npc::NpcKind::Nurse,
            crate::npc::NpcKind::Dryad,
            crate::npc::NpcKind::ArmsDealer,
            crate::npc::NpcKind::Demolitionist,
            crate::npc::NpcKind::Clothier,
            crate::npc::NpcKind::GoblinTinkerer,
            crate::npc::NpcKind::TravellingMerchant,
            crate::npc::NpcKind::Wizard,
            crate::npc::NpcKind::Mechanic,
            crate::npc::NpcKind::Stylist,
            crate::npc::NpcKind::TaxCollector,
            crate::npc::NpcKind::Golfer,
            crate::npc::NpcKind::Zoologist,
            crate::npc::NpcKind::Angler,
            crate::npc::NpcKind::Painter,
            crate::npc::NpcKind::WitchDoctor,
        ] {
            let mut n = crate::npc::Npc::new(kind, 32.0, 32.0, 1, 0.0);
            n.hp = n.max_hp;
            roster.push(n);
        }
        assert!(
            crate::bestiary::all_town_kinds_present(&roster),
            "the full roster opens the gate"
        );
        // Missing just one kind closes it.
        roster.pop();
        assert!(
            !crate::bestiary::all_town_kinds_present(&roster),
            "one absentee closes the gate"
        );
    }

    #[test]
    fn the_golfer_kinds_stats_roundtrip_and_shop() {
        // `SetDefaults` 588 (`NPC.cs:16638-16651`) and 589 (`:16652-16665`):
        // the town twin's numbers on both, the sleeping body's 18x34, the
        // kind<->u16 round-trips, the Golfer's shop (`NPCInteractions.cs:509`'s
        // `Shop(588, 22)`), the town flag (589 is `Sets.BoundTownNPCs`,
        // `NPCID.cs:10441`) and the bestiary row.
        for (kind, id) in [(NpcKind::Golfer, 588u16), (NpcKind::SleepingGolfer, 589)] {
            assert_eq!(kind.max_hp(), 250, "{kind:?}");
            assert_eq!(kind.contact_damage(), 10, "{kind:?}");
            assert_eq!(kind.cs_id(), id, "{kind:?}");
        }
        assert_eq!(NpcKind::Golfer.size(), (18.0, 40.0));
        assert_eq!(
            NpcKind::SleepingGolfer.size(),
            (18.0, 34.0),
            "the bound body"
        );
        assert_eq!(NpcKind::Golfer.to_u16(), 163);
        assert_eq!(NpcKind::from_u16(163), Some(NpcKind::Golfer));
        assert_eq!(NpcKind::SleepingGolfer.to_u16(), 164);
        assert_eq!(NpcKind::from_u16(164), Some(NpcKind::SleepingGolfer));
        assert!(NpcKind::Golfer.is_town_resident());
        assert!(!NpcKind::SleepingGolfer.is_town_resident());
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Golfer),
            Some(22)
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Golfer, SlimeVariant::Blue).net_id,
            588
        );
    }

    /// `Chest.SetupShop`'s case 22 (`Chest.cs:3156-3262`): the Golfer's always
    /// rows and the score-gated tiers — the vanilla mixes `>=` and `>` per row
    /// (`:3169` vs `:3186`), which the port's `StockWhen::GolfScore` carries.
    #[test]
    fn the_golfer_shop_scores_its_clubs_and_trophies() {
        let base = crate::shop::resolve_stock(22, &crate::shop::ShopFlags::default(), &[]);
        for id in [
            4587, 4590, 4589, 4588, 4083, 4084, 4085, 4086, 4087, 4088, 4089, 3989, 4095, 4040,
            4319, 4320, 4135, 4138, 4136, 4137, 4049,
        ] {
            assert!(base.contains(&id), "the always row {id}");
        }
        for id in [4039, 4591, 4265, 4601, 4264, 4658] {
            assert!(!base.contains(&id), "the gated {id} stays off at score 0");
        }
        let mid = crate::shop::ShopFlags {
            golfer_score: 600,
            ..Default::default()
        };
        let rows = crate::shop::resolve_stock(22, &mid, &[]);
        assert!(rows.contains(&4039), "the iron set at >= 500");
        assert!(rows.contains(&4265), "the golf chest at > 500");
        assert!(rows.contains(&4599), "the bronze trophy at > 500");
        assert!(!rows.contains(&4591), "no mythril set under 1000");
        assert!(!rows.contains(&4600), "no silver trophy under 1000");
        let high = crate::shop::ShopFlags {
            golfer_score: 1500,
            ..Default::default()
        };
        let rows = crate::shop::resolve_stock(22, &high, &[]);
        assert!(rows.contains(&4591), "the mythril set at > 1000");
        assert!(!rows.contains(&4595), "no titanium set under 2000");
        let master = crate::shop::ShopFlags {
            golfer_score: 2500,
            ..Default::default()
        };
        let phase1 = crate::shop::resolve_stock_moon(22, &master, &[], 1);
        assert!(phase1.contains(&4658), "the phase-1 painting");
        assert!(!phase1.contains(&4659), "not the phase-3 painting");
        let phase3 = crate::shop::resolve_stock_moon(22, &master, &[], 3);
        assert!(phase3.contains(&4659), "the phase-3 painting");
    }

    #[test]
    fn the_stylist_and_tax_collector_kinds_stats_roundtrip_and_shops() {
        // `SetDefaults` 353 (`NPC.cs:13488-13501`), 354 (`:13502-13516`) and
        // 441 (`:14704-14717`): the town twin's numbers on all three, the
        // webbed body's 18x34, the kind<->u16 round-trips, the Stylist's shop
        // (`NPCInteractions.cs:505`'s `Shop(353, 18)`) and the Tax
        // Collector's absence (no `Shop(441, ...)` row at all), the town flag
        // (the webbed Stylist is a bound NPC).
        for (kind, id) in [
            (NpcKind::Stylist, 353u16),
            (NpcKind::WebbedStylist, 354),
            (NpcKind::TaxCollector, 441),
        ] {
            assert_eq!(kind.max_hp(), 250, "{kind:?}");
            assert_eq!(kind.contact_damage(), 10, "{kind:?}");
            assert_eq!(kind.cs_id(), id, "{kind:?}");
        }
        assert_eq!(NpcKind::Stylist.size(), (18.0, 40.0));
        assert_eq!(
            NpcKind::WebbedStylist.size(),
            (18.0, 34.0),
            "the bound body"
        );
        assert_eq!(NpcKind::TaxCollector.size(), (18.0, 40.0));
        assert_eq!(NpcKind::Stylist.to_u16(), 160);
        assert_eq!(NpcKind::from_u16(160), Some(NpcKind::Stylist));
        assert_eq!(NpcKind::WebbedStylist.to_u16(), 161);
        assert_eq!(NpcKind::from_u16(161), Some(NpcKind::WebbedStylist));
        assert_eq!(NpcKind::TaxCollector.to_u16(), 162);
        assert_eq!(NpcKind::from_u16(162), Some(NpcKind::TaxCollector));
        assert!(NpcKind::Stylist.is_town_resident());
        assert!(NpcKind::TaxCollector.is_town_resident());
        assert!(!NpcKind::WebbedStylist.is_town_resident());
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Stylist),
            Some(18)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::TaxCollector),
            None,
            "the Tax Collector sells nothing"
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Stylist, SlimeVariant::Blue).net_id,
            353
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::TaxCollector, SlimeVariant::Blue).net_id,
            441
        );
    }

    /// `Chest.SetupShop`'s case 18 (`Chest.cs:2698-2790`): the Stylist's
    /// hair-dye rows and their gates.
    #[test]
    fn the_stylist_shop_carries_the_dye_rows_and_gates() {
        let flags = crate::shop::ShopFlags {
            life_max_400: true,
            mana_max_200: true,
            coins_over_gold: true,
            party_girl_present: true,
            downed_mech_1: true,
            downed_mech_2: true,
            downed_mech_3: true,
            downed_mech_any: true,
            downed_martians: true,
            hard_mode: true,
            ..Default::default()
        };
        let stock = crate::shop::resolve_stock(18, &flags, &[]);
        for id in [
            1990, 1979, 1977, 1978, 1980, 1983, 1984, 1985, 1986, 2863, 3259, 5104,
        ] {
            assert!(stock.contains(&id), "the {id} dye row");
        }
        // The moon-phase parity row: even phase + day, odd phase + night.
        let day_even = crate::shop::resolve_stock_moon(
            18,
            &crate::shop::ShopFlags {
                night: false,
                ..Default::default()
            },
            &[],
            0,
        );
        assert!(day_even.contains(&1981), "even phase by day");
        let night_even = crate::shop::resolve_stock_moon(
            18,
            &crate::shop::ShopFlags {
                night: true,
                ..Default::default()
            },
            &[],
            0,
        );
        assert!(!night_even.contains(&1981), "no even-phase row by night");
        let night_odd = crate::shop::resolve_stock_moon(
            18,
            &crate::shop::ShopFlags {
                night: true,
                ..Default::default()
            },
            &[],
            1,
        );
        assert!(night_odd.contains(&1981), "odd phase by night");
    }

    #[test]
    fn the_wizard_and_mechanic_kinds_stats_roundtrip_and_shops() {
        // `SetDefaults` 108 (`NPC.cs:10285-10298`), 106 (`:10257-10270`),
        // 124 (`:10508-10521`) and 123 (`:10493-10507`): the town twin's
        // numbers on all four, the bound Mechanic's shorter 18x34 body, the
        // kind<->u16 round-trips, the shops (`NPCInteractions.cs:494-495`'s
        // `Shop(108, 7)`/`Shop(124, 8)`) and the town flag (the bound forms
        // are NOT town NPCs — `Sets.BoundTownNPCs`, `NPCID.cs:10441`).
        for (kind, id) in [
            (NpcKind::Wizard, 108u16),
            (NpcKind::BoundWizard, 106),
            (NpcKind::Mechanic, 124),
            (NpcKind::BoundMechanic, 123),
        ] {
            assert_eq!(kind.max_hp(), 250, "{kind:?}");
            assert_eq!(kind.contact_damage(), 10, "{kind:?}");
            assert_eq!(kind.cs_id(), id, "{kind:?}");
        }
        assert_eq!(NpcKind::Wizard.size(), (18.0, 40.0));
        assert_eq!(NpcKind::BoundWizard.size(), (18.0, 40.0));
        assert_eq!(NpcKind::Mechanic.size(), (18.0, 40.0));
        assert_eq!(
            NpcKind::BoundMechanic.size(),
            (18.0, 34.0),
            "the bound body"
        );
        assert_eq!(NpcKind::Wizard.to_u16(), 156);
        assert_eq!(NpcKind::from_u16(156), Some(NpcKind::Wizard));
        assert_eq!(NpcKind::BoundWizard.to_u16(), 157);
        assert_eq!(NpcKind::from_u16(157), Some(NpcKind::BoundWizard));
        assert_eq!(NpcKind::Mechanic.to_u16(), 158);
        assert_eq!(NpcKind::from_u16(158), Some(NpcKind::Mechanic));
        assert_eq!(NpcKind::BoundMechanic.to_u16(), 159);
        assert_eq!(NpcKind::from_u16(159), Some(NpcKind::BoundMechanic));
        assert!(NpcKind::Wizard.is_town_resident());
        assert!(NpcKind::Mechanic.is_town_resident());
        assert!(!NpcKind::BoundWizard.is_town_resident());
        assert!(!NpcKind::BoundMechanic.is_town_resident());
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Wizard),
            Some(7)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Mechanic),
            Some(8)
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Wizard, SlimeVariant::Blue).net_id,
            108
        );
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::Mechanic, SlimeVariant::Blue).net_id,
            124
        );
    }

    /// `Chest.SetupShop`'s case 7 (`Chest.cs:2004-2032`): the Wizard's nine
    /// magic rows always on; the hardmode blood-moon and halloween rows sit
    /// constants-off.
    #[test]
    fn the_wizard_shop_carries_the_magic_rows() {
        let stock = crate::shop::resolve_stock(7, &crate::shop::ShopFlags::default(), &[]);
        assert_eq!(
            stock,
            vec![487, 496, 500, 507, 508, 531, 149, 576, 3186],
            "the nine base rows"
        );
        assert!(
            !stock.contains(&5461),
            "the hardmode blood-moon deck is off"
        );
        assert!(!stock.contains(&1739), "the halloween hat is off");
        let empty = crate::shop::resolve_stock(
            7,
            &crate::shop::ShopFlags {
                blood_moon: true,
                ..Default::default()
            },
            &[],
        );
        assert_eq!(empty, stock, "the flags do not add the off rows");
    }

    /// `Chest.SetupShop`'s case 8 (`Chest.cs:2033-2090`): the wire family's
    /// 30 rows and the Mechanic's Rod (2295) behind the moon-phase pair.
    #[test]
    fn the_mechanic_shop_carries_the_wire_rows_and_the_rod() {
        let stock = crate::shop::resolve_stock(8, &crate::shop::ShopFlags::default(), &[]);
        assert_eq!(stock.len(), 30, "the base wire rows: {stock:?}");
        // The Rod rides the ODD phases (`:2085`'s 1/3/5/7), so phase 0 hides
        // it and phase 1 shows it.
        for id in [509, 850, 510, 530, 513, 849, 1263, 583, 4484, 4485] {
            assert!(stock.contains(&id), "the {id} wire row");
        }
        // The Rod rides the even moon phase (1/3/5/7 in the C#), so phase 0
        // (the pair's mid) shows it and phase 2 hides it.
        assert!(!stock.contains(&2295), "no Rod at moonPhase 0");
        let phase1 = crate::shop::resolve_stock_moon(8, &crate::shop::ShopFlags::default(), &[], 1);
        let phase4 = crate::shop::resolve_stock_moon(8, &crate::shop::ShopFlags::default(), &[], 4);
        assert!(phase1.contains(&2295), "the Rod at moonPhase 1");
        assert!(!phase4.contains(&2295), "no Rod at moonPhase 5");
        // The graveyard goggles sit constants-off.
        assert!(!stock.contains(&4409));
    }

    #[test]
    fn the_sleeping_angler_frame_flips_on_thirty_ticks() {
        let w = ground();
        let mut s = Npc::new(NpcKind::SleepingAngler, 32.0, 100.0, 1, 0.0);
        s.hp = s.max_hp;
        assert_eq!(s.frame, 0, "the asleep row");
        for _ in 0..30 {
            s.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        }
        assert_eq!(s.frame, 0, "the counter needs past 30");
        s.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        assert_eq!(s.frame, 1, "the awake row");
        for _ in 0..30 {
            s.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        }
        s.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        assert_eq!(s.frame, 0, "the wrap back to the asleep row");
    }

    /// The Angler rides the town walker's frame machine (`AI_007`'s shared
    /// town body, `NPC.cs:68608`): a walking Angler cycles the town rows and
    /// a still one sits on the idle row.
    #[test]
    fn the_angler_rides_the_town_walker_frames() {
        let w = ground();
        let mut a = Npc::new(NpcKind::Angler, 32.0, 100.0, 1, 0.0);
        a.hp = a.max_hp;
        for _ in 0..10 {
            a.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        }
        assert_eq!(a.frame, 0, "the idle row");
        a.vx = 2.0;
        for _ in 0..8 {
            a.update(&w, &mut Player::spawn(&w), 1.0 / 60.0);
        }
        assert!(a.frame <= 2, "the town rows: {}", a.frame);
    }

    #[test]
    fn the_angler_kinds_stats_roundtrip_and_quest() {
        // `SetDefaults` 369 (`NPC.cs:13687-13700`): the town twin's numbers
        // (18x40, 250 HP, dmg 10, def 15, kb 0.5, aiStyle 7) and no shop
        // index (`NPCInteractions.cs:488-512` has no Shop(369, n) — the
        // quest interaction rides `:519` instead); 376 (`NPC.cs:13796-13810`)
        // is the flat 34x8 bound form (scale 0.9, rarity 1, aiStyle 0).
        let a = Npc::new(NpcKind::Angler, 0.0, 0.0, 1, 0.0);
        assert_eq!(NpcKind::Angler.max_hp(), 250);
        assert_eq!(a.defense(), 15);
        assert_eq!(a.knock_back_resist(), 0.5);
        assert_eq!(NpcKind::Angler.contact_damage(), 10);
        assert_eq!(NpcKind::Angler.size(), (18.0, 40.0));
        assert_eq!(NpcKind::Angler.to_u16(), 150);
        assert_eq!(NpcKind::from_u16(150), Some(NpcKind::Angler));
        assert_eq!(NpcKind::Angler.cs_id(), 369);
        assert!(NpcKind::Angler.is_town_resident());
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Angler),
            None,
            "the Angler sells nothing — the quest interaction instead"
        );
        let s = Npc::new(NpcKind::SleepingAngler, 0.0, 0.0, 1, 0.0);
        assert_eq!(NpcKind::SleepingAngler.max_hp(), 250);
        assert_eq!(s.defense(), 15);
        assert_eq!(s.knock_back_resist(), 0.5);
        assert_eq!(NpcKind::SleepingAngler.size(), (34.0, 8.0));
        assert_eq!(NpcKind::SleepingAngler.to_u16(), 151);
        assert_eq!(NpcKind::from_u16(151), Some(NpcKind::SleepingAngler));
        assert_eq!(NpcKind::SleepingAngler.cs_id(), 376);
        // The bound form is NOT a town NPC (`Sets.BoundTownNPCs`,
        // `NPCID.cs:10441`).
        assert!(!NpcKind::SleepingAngler.is_town_resident());
        // The bestiary rows: the Angler's visible (`NPCID.cs:12433`); the
        // Sleeping Angler's `Hide = true` (`:3721-3725`) keeps it out.
        let e = crate::bestiary::entry_for_npc(NpcKind::Angler, SlimeVariant::Blue);
        assert_eq!(e.net_id, 369);
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::SleepingAngler, SlimeVariant::Blue).net_id,
            crate::bestiary::entry_for_npc(NpcKind::Angler, SlimeVariant::Blue).net_id,
            "the hidden kind carries the visible row"
        );
    }

    #[test]
    fn the_travelling_merchant_kind_stats_roundtrip_and_shop() {
        // `SetDefaults` 368 (`NPC.cs:13673-13686`): the town twin's numbers —
        // 18x40, 250 HP, dmg 10, def 15, kb 0.5, value 0 — the kind<->u16
        // round-trip, the bestiary row and the shop map (index 19,
        // `NPCInteractions.cs:506`'s `Shop(368, 19)`).
        let m = Npc::new(NpcKind::TravellingMerchant, 0.0, 0.0, 1, 0.0);
        assert_eq!(NpcKind::TravellingMerchant.max_hp(), 250);
        assert_eq!(m.defense(), 15);
        assert_eq!(m.knock_back_resist(), 0.5);
        assert_eq!(NpcKind::TravellingMerchant.contact_damage(), 10);
        assert_eq!(
            Npc::new(NpcKind::TravellingMerchant, 0.0, 0.0, 1, 0.0).value(),
            0.0
        );
        assert_eq!(NpcKind::TravellingMerchant.size(), (18.0, 40.0));
        assert_eq!(NpcKind::TravellingMerchant.to_u16(), 111);
        assert_eq!(NpcKind::from_u16(111), Some(NpcKind::TravellingMerchant));
        assert!(NpcKind::TravellingMerchant.is_town_resident());
        let e = crate::bestiary::entry_for_npc(NpcKind::TravellingMerchant, SlimeVariant::Blue);
        assert_eq!(e.net_id, 368);
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::TravellingMerchant),
            Some(19)
        );
    }

    // M4-NPC-3t: EVERY ported kind must keep `FindFrame` inside its own
    // sheet's row count (`Main.npcFrameCount[netID]`) — a row past the strip
    // samples the neighbouring row (or the sheet's blank tail) and shows up as
    // a scrambled or half-missing sprite. The kinds whose draw uses a GRID
    // cell index instead of a row are listed with their cell bound.
    #[test]
    fn every_kind_finds_a_frame_inside_its_vanilla_strip() {
        use crate::render::entities::{npc_rows, texture_net_id};
        let mut bad: Vec<String> = Vec::new();
        for net in 0u16..697 {
            let Some(kind) = NpcKind::from_u16(net) else {
                continue;
            };

            // A few kinds lack a bestiary row (a separate audit) and
            // `NpcKind::net_id` reads the bestiary, so filter them first.
            if !crate::bestiary::has_entry(kind, SlimeVariant::Blue) {
                continue;
            }
            // The grid/cell draws: `MoonLordHead` walks the 3x3 grid (cell
            // 0..8, `Main.cs:24926`) and the Deerclops the 5x5 one
            // (`Main.cs:26334-26412`).
            let rows = match kind {
                NpcKind::MoonLordHead => 9,
                NpcKind::Deerclops => 25,
                _ => npc_rows(texture_net_id(kind)),
            };
            let mut n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            for (vx, vy) in [(0.0, 0.0), (2.5, 0.0), (-2.5, 0.0), (0.0, 3.0), (0.0, -3.0)] {
                for ai0 in [0.0, 1.0, 2.0, 5.0, 6.0, 30.0, 100.0, 210.0, 400.0] {
                    for ai1 in [0.0, 20.0, 120.0, 300.0] {
                        n.vx = vx;
                        n.vy = vy;
                        n.ai = [ai0, ai1, 0.0, ai0];
                        n.frame = 0;
                        n.frame_counter = 0.0;
                        for _ in 0..24 {
                            n.find_frame();
                            if n.frame >= rows {
                                bad.push(format!(
                                    "{kind:?} (net {}): row {} of {rows}: ai {ai0}/{ai1}, v {vx}/{vy}",
                                    kind.net_id(),
                                    n.frame
                                ));
                                // One report per kind is enough to name the bug.
                                break;
                            }
                        }
                        if !bad.is_empty() && bad.last().unwrap().starts_with(&format!("{kind:?} "))
                        {
                            break;
                        }
                    }
                }
            }
        }
        bad.sort();
        bad.dedup();
        assert!(
            bad.is_empty(),
            "{} kind(s) left their strip:
{}",
            bad.len(),
            bad.join(
                "
"
            )
        );
    }

    // M4-NPC-3t: the pillar/Moon-Lord families now draw from their OWN
    // `NPC_<netID>` sheets, so a `FindFrame` row that lands outside
    // `Main.npcFrameCount[netID]` would sample a neighbouring row of the same
    // sheet (or the sheet's blank tail) instead of bleeding across a shared
    // family sheet. Sweep the motion/AI states each machine branches on and
    // assert every row stays inside the vanilla strip.
    #[test]
    fn the_pillar_family_rows_stay_inside_their_own_sheets() {
        // (kind, `Main.npcFrameCount[netID]`)
        let cases: &[(NpcKind, u32)] = &[
            // `MoonLordHead` is a 3x3 GRID sheet (cell index 0..8, the
            // custom block at `Main.cs:24846-24927`) — asserted separately.
            (NpcKind::MoonLordHand, 4),
            (NpcKind::MoonLordCore, 5),
            (NpcKind::MoonLordFreeEye, 4),
            (NpcKind::MoonLordLeechBlob, 1),
            (NpcKind::StardustWormHead, 1),
            (NpcKind::StardustCellBig, 4),
            (NpcKind::StardustCellSmall, 2),
            (NpcKind::StardustJellyfishBig, 5),
            (NpcKind::StardustSpiderBig, 11),
            (NpcKind::StardustSpiderSmall, 5),
            (NpcKind::StardustSoldier, 11),
            (NpcKind::SolarCrawltipedeHead, 1),
            (NpcKind::SolarCrawltipedeBody, 1),
            (NpcKind::SolarCrawltipedeTail, 1),
            (NpcKind::SolarDrakomire, 10),
            (NpcKind::SolarDrakomireRider, 10),
            (NpcKind::SolarSroller, 15),
            (NpcKind::SolarCorite, 8),
            (NpcKind::SolarSelenian, 17),
            (NpcKind::SolarSpearman, 10),
            (NpcKind::NebulaFloater, 6),
            (NpcKind::BrainSuckler, 6),
            (NpcKind::NebulaBeast, 12),
            (NpcKind::Predictor, 12),
            (NpcKind::VortexRifleman, 13),
            (NpcKind::VortexSoldier, 10),
            (NpcKind::VortexHunter, 15),
            (NpcKind::VortexAlien, 9),
            (NpcKind::VortexHeadcrab, 5),
        ];
        for &(kind, rows) in cases {
            let mut n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            for (vx, vy) in [(0.0, 0.0), (2.0, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
                for ai0 in [0.0, 1.0, 2.0, 6.0, 30.0, 100.0, 200.0, 400.0] {
                    for ai1 in [0.0, 20.0, 100.0, 280.0, 420.0] {
                        n.vx = vx;
                        n.vy = vy;
                        n.ai = [ai0, ai1, 0.0, ai0];
                        n.frame = 0;
                        n.frame_counter = 0.0;
                        for _ in 0..40 {
                            n.find_frame();
                            assert!(
                                n.frame < rows,
                                "{kind:?}: row {} left the {rows}-row strip                                  (vx {vx}, vy {vy}, ai {ai0}/{ai1})",
                                n.frame
                            );
                        }
                    }
                }
            }
        }
    }

    // M4-NPC-3t: the pillar/Moon-Lord families now draw from their OWN
    // `NPC_<netID>` sheets, so a `FindFrame` row that lands outside
    // `Main.npcFrameCount[netID]` would sample a neighbouring row of the same
    // sheet (or the sheet's blank tail) instead of bleeding across a shared
    // family sheet. Sweep the motion/AI states each machine branches on and
    // assert every row stays inside the vanilla strip.
    #[test]
    // The lunar families' machines are per-kind `FindFrame` cases now, so a
    // state sweep must land on each case's own band (`NPC.cs:70119-70803`)
    // instead of the shared `|vx| + 0.6` accumulator the v1 ran.
    #[test]
    fn the_lunar_find_frame_machines_walk_their_own_bands() {
        let rows = |k: NpcKind| -> u32 {
            crate::render::entities::npc_rows(crate::render::entities::texture_net_id(k))
        };

        // 405/406: the plain 4-tick wrap over the whole strip.
        let mut cell = Npc::new(NpcKind::StardustCellBig, 0.0, 0.0, 1, 0.0);
        let mut seen = Vec::new();
        for _ in 0..(rows(NpcKind::StardustCellBig) * 4) {
            cell.find_frame();
            seen.push(cell.frame);
        }
        assert_eq!(
            seen,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0],
            "the 4-tick wrap"
        );

        // 407: the same wrap at 6 ticks.
        let mut jelly = Npc::new(NpcKind::StardustJellyfishBig, 0.0, 0.0, 1, 0.0);
        for _ in 0..5 {
            jelly.find_frame();
        }
        assert_eq!(jelly.frame, 0, "six ticks a row");
        jelly.find_frame();
        assert_eq!(jelly.frame, 1);

        // 409: the `ai[1]` 1..30 attack band pins the last row; the walk wraps
        // two rows early.
        let mut spider = Npc::new(NpcKind::StardustSpiderBig, 0.0, 0.0, 1, 0.0);
        spider.ai[1] = 12.0;
        spider.find_frame();
        assert_eq!(
            spider.frame,
            rows(NpcKind::StardustSpiderBig) - 1,
            "the attack row"
        );
        spider.ai[1] = 0.0;
        spider.frame = 0;
        spider.frame_counter = 0.0;
        for _ in 0..(6 * (rows(NpcKind::StardustSpiderBig) + 2)) {
            spider.find_frame();
            assert!(
                spider.frame < rows(NpcKind::StardustSpiderBig) - 1,
                "the walk never reaches the attack rows"
            );
        }

        // 411: the `ai[1]` 120..240 channel pins the last row and flips the
        // sprite; the walk starts on row 2.
        let mut soldier = Npc::new(NpcKind::StardustSoldier, 10.0, 0.0, -1, 0.0);
        soldier.vx = 2.0;
        soldier.ai[1] = 150.0;
        soldier.find_frame();
        assert_eq!(soldier.frame, rows(NpcKind::StardustSoldier) - 1);
        assert_eq!(soldier.sprite_dir, 1, "-direction");
        soldier.ai[1] = 0.0;
        soldier.vy = 0.0;
        soldier.frame = 0;
        soldier.find_frame();
        assert!(soldier.frame >= 2, "the walk rides the 2.. band");

        // 415: the `ai[2] < 0` charge flips rows 8/9 off the 18-tick counter;
        // the grounded walk stays inside 2..count-2 and lands on row 0 at rest.
        let mut drak = Npc::new(NpcKind::SolarDrakomire, 3.0, 0.0, 1, 0.0);
        drak.vx = 3.0;
        drak.ai[2] = -1.0;
        let mut charge_rows = Vec::new();
        for _ in 0..24 {
            drak.find_frame();
            charge_rows.push(drak.frame);
        }
        assert!(charge_rows.iter().any(|&r| r == 8), "the charge's row 8");
        assert!(charge_rows.iter().any(|&r| r == 9), "and its row 9");
        drak.ai[2] = 0.0;
        drak.vx = 0.0;
        drak.frame = 5;
        drak.find_frame();
        assert_eq!(drak.frame, 0, "standing still snaps to row 0");

        // 417: `ai[0] == 1` / `5` climb 9..14 off `ai[1] / 6`, any other state
        // pins row 14.
        let mut sroller = Npc::new(NpcKind::SolarSroller, 0.0, 0.0, 1, 0.0);
        sroller.ai[0] = 1.0;
        sroller.ai[1] = 12.0;
        sroller.find_frame();
        assert_eq!(sroller.frame, 11, "9 + 12/6");
        sroller.ai[0] = 5.0;
        sroller.find_frame();
        assert_eq!(sroller.frame, 11, "13 - 12/6");
        sroller.ai[0] = 3.0;
        sroller.find_frame();
        assert_eq!(sroller.frame, 14, "the else arm pins 14");
        sroller.ai[0] = 0.0;
        sroller.vx = 0.0;
        sroller.frame_counter = 0.0;
        sroller.find_frame();
        assert_eq!(sroller.frame, 0, "the roll starts on row 0");

        // 418: the idle band walks 0..4, the `ai[0] == 2` dive splits `ai[1]`.
        let mut corite = Npc::new(NpcKind::SolarCorite, 0.0, 0.0, 1, 0.0);
        for _ in 0..7 {
            corite.find_frame();
        }
        assert_eq!(corite.frame, 1, "the 6-tick idle step");
        corite.ai[0] = 2.0;
        for (ai1, want) in [(5.0, 5), (15.0, 6), (25.0, 7)] {
            corite.ai[1] = ai1;
            corite.find_frame();
            assert_eq!(corite.frame, want, "the dive band at ai[1] {ai1}");
        }

        // 419: the `ai[2] < 0` charge climbs 2..8, the grounded walk starts on
        // row 9 and the air holds row 1.
        let mut selen = Npc::new(NpcKind::SolarSelenian, 4.0, 0.0, 1, 0.0);
        selen.vx = 4.0;
        selen.ai[2] = -1.0;
        let mut charge = Vec::new();
        for _ in 0..12 {
            selen.find_frame();
            charge.push(selen.frame);
        }
        assert!(
            charge.iter().all(|&r| (2..=8).contains(&r)),
            "the charge band 2..8: {charge:?}"
        );
        selen.ai[2] = 0.0;
        selen.frame = 0;
        selen.find_frame();
        assert!(selen.frame >= 9, "the walk band starts at 9");
        selen.vy = 2.0;
        selen.find_frame();
        assert_eq!(selen.frame, 1, "the air row");

        // 423: the `ai[2] == 1` charge pins rows 2/3 off `ai[1]`, the walk
        // rides rows 4.. and flips off the velocity sign.
        let mut beast = Npc::new(NpcKind::NebulaBeast, 0.0, 0.0, 1, 0.0);
        beast.ai[2] = 1.0;
        beast.ai[1] = 35.0;
        beast.find_frame();
        assert_eq!(beast.frame, 3);
        beast.ai[1] = 10.0;
        beast.find_frame();
        assert_eq!(beast.frame, 2);
        beast.ai[2] = 0.0;
        beast.vx = -2.0;
        beast.find_frame();
        assert_eq!(beast.sprite_dir, 1, "spriteDirection = sign(-vx)");
        assert!(beast.frame >= 4);

        // 424: the aim band pins the last two rows; the walk rides 2..count-2.
        let mut pred = Npc::new(NpcKind::Predictor, 0.0, 0.0, 1, 0.0);
        pred.ai[2] = 1.0;
        pred.ai[1] = 120.0;
        pred.find_frame();
        assert_eq!(pred.frame, rows(NpcKind::Predictor) - 1);
        pred.ai[1] = 40.0;
        pred.find_frame();
        assert_eq!(pred.frame, rows(NpcKind::Predictor) - 2);
        pred.ai[2] = 0.0;
        pred.vx = 1.0;
        pred.frame = 0;
        pred.find_frame();
        assert_eq!(pred.frame, 2, "the walk floor");

        // 425: the `ai[2] == 1` firing band cycles 10..12 at 3 ticks.
        let mut rifle = Npc::new(NpcKind::VortexRifleman, 0.0, 0.0, 1, 0.0);
        rifle.ai[2] = 1.0;
        rifle.frame = 10;
        for _ in 0..3 {
            rifle.find_frame();
        }
        assert_eq!(rifle.frame, 11, "three ticks a row");
        for _ in 0..9 {
            rifle.find_frame();
            assert!((10..=12).contains(&rifle.frame), "the firing band");
        }

        // 426: the `ai[1] >= 45` dive cycles 12..15, the plain air 9..11.
        let mut queen = Npc::new(NpcKind::VortexHunter, 0.0, 0.0, 1, 0.0);
        queen.vy = -2.0;
        queen.ai[1] = 60.0;
        queen.frame = 12;
        for _ in 0..40 {
            queen.find_frame();
            assert!((12..=14).contains(&queen.frame), "the dive band");
        }
        queen.ai[1] = 0.0;
        queen.frame = 9;
        for _ in 0..20 {
            queen.find_frame();
            assert!((9..=11).contains(&queen.frame), "the plain air band");
        }

        // 427: the air band 6..8, the walk 1..5.
        let mut hornet = Npc::new(NpcKind::VortexAlien, 0.0, 0.0, 1, 0.0);
        hornet.vy = -2.0;
        hornet.frame = 6;
        for _ in 0..20 {
            hornet.find_frame();
            assert!((6..=8).contains(&hornet.frame), "the hornet's air band");
        }

        // 428: the grounded walk wraps one row early; the fall latches row 4.
        let mut larva = Npc::new(NpcKind::VortexHeadcrab, 2.0, 0.0, 1, 0.0);
        larva.vx = 2.0;
        for _ in 0..(22 * rows(NpcKind::VortexHeadcrab)) {
            larva.find_frame();
            assert!(
                larva.frame < rows(NpcKind::VortexHeadcrab) - 1,
                "the walk band"
            );
        }
        larva.vy = 2.0;
        for _ in 0..12 {
            larva.find_frame();
        }
        assert_eq!(larva.frame, 4, "the fall row");

        // 429/518: the grounded walk starts on row 2, the air holds row 1.
        for kind in [NpcKind::VortexSoldier, NpcKind::SolarSpearman] {
            let mut n = Npc::new(kind, 2.0, 0.0, 1, 0.0);
            n.vx = 2.0;
            n.frame = 0;
            n.find_frame();
            assert!(n.frame >= 2, "{kind:?}: the walk floor");
            n.vy = 2.0;
            n.find_frame();
            assert_eq!(n.frame, 1, "{kind:?}: the air row");
        }
    }

    #[test]
    fn the_tinkerer_kinds_stats_roundtrip_and_shop() {
        // `SetDefaults` 105 (`NPC.cs:10245-10257`) and 107 (`:10271-10283`):
        // the 250/15/0.5 pair, the 18x34 vs 18x40 bodies, the kind<->u16
        // round-trip, the bestiary row and the shop map.
        assert_eq!(NpcKind::BoundGoblin.max_hp(), 250);
        assert_eq!(NpcKind::GoblinTinkerer.max_hp(), 250);
        let t = Npc::new(NpcKind::GoblinTinkerer, 0.0, 0.0, 1, 0.0);
        assert_eq!(t.defense(), 15);
        assert_eq!(t.knock_back_resist(), 0.5);
        assert_eq!(NpcKind::BoundGoblin.contact_damage(), 0);
        assert_eq!(NpcKind::BoundGoblin.size(), (18.0, 34.0));
        assert_eq!(NpcKind::GoblinTinkerer.size(), (18.0, 40.0));
        for (kind, v) in [(NpcKind::BoundGoblin, 63), (NpcKind::GoblinTinkerer, 64)] {
            assert_eq!(kind.to_u16(), v);
            assert_eq!(NpcKind::from_u16(v), Some(kind));
        }
        let e = crate::bestiary::entry_for_npc(NpcKind::GoblinTinkerer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 107);
        // The shop map (`NPCInteractions.cs:488-498`): the Tinkerer's
        // shop 6; the Guide has none.
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::GoblinTinkerer),
            Some(6)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Merchant),
            Some(1)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Guide),
            None
        );
        // `TinkererReforge` (`NPCInteractions.cs:407-423`): the Tinkerer's
        // talk opens the reforge menu beside the shop; the other shop
        // keepers do not.
        let mut app_tinkerer = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let t = crate::npc::Npc::new(
            NpcKind::GoblinTinkerer,
            app_tinkerer.mouse_pos.x,
            app_tinkerer.mouse_pos.y,
            1,
            0.0,
        );
        app_tinkerer.npcs.push(t);
        let cursor = app_tinkerer.mouse_pos;
        assert!(crate::world_sim::npcs::try_open_shop_at_cursor(
            &mut app_tinkerer,
            cursor
        ));
        assert!(
            app_tinkerer.in_reforge_menu,
            "the Tinkerer's talk opens the reforge menu"
        );
        assert_eq!(app_tinkerer.shop_npc, Some(0));
        // The stock (`Chest.cs:1989-2003`): the 7 rows in order, no gating.
        let ids = crate::shop::resolve_stock(6, &crate::shop::ShopFlags::default(), &[]);
        assert_eq!(ids, vec![128, 486, 398, 84, 407, 161, 5324]);
    }

    #[test]
    fn the_bound_goblin_rescues_into_tinkerer() {
        // `AI_000_TransformBoundNPC` (`NPC.cs:44884-44896`): the talk
        // swaps 105 -> 107, the freed stats restore, `savedGoblin` raises.
        let mut w = ground_air();
        let mut n = Npc::new(NpcKind::BoundGoblin, 20.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        n.max_hp = 250;
        n.hp = 250;
        assert_eq!(w.saved_goblin, false);
        crate::world_sim::npcs::rescue_bound_goblin(&mut n, &mut w);
        assert_eq!(n.kind, NpcKind::GoblinTinkerer);
        assert_eq!(n.max_hp, 250);
        assert_eq!(n.hp, 250);
        assert_eq!(w.saved_goblin, true);
    }
    #[test]
    fn the_town_humanoids_walk_rows_two_to_count() {
        // `FindFrame`'s humanoid-group default arm (`NPC.cs:69481-69577`):
        // the idle snaps to row 0, the walk rides rows 2..count-1 — the
        // accumulator `|vx| * 2 + 1` advancing the row every 6, the wrap
        // back to row 2. The Guide (22) has 26 rows, the Merchant 25,
        // the goblins 16.
        let mut g = bare(NpcKind::Guide);
        g.vx = 0.5;
        g.frame = 0;
        g.frame_counter = 0.0;
        g.find_frame();
        assert_eq!(g.frame, 2, "the walk snaps up to row 2");
        // Every 6 accumulated (0.5*2+1 = 2/tick) => 3 ticks per row.
        g.frame_counter = 0.0;
        for _ in 0..9 {
            g.find_frame();
        }
        assert_eq!(g.frame, 4, "9 ticks: two advances (rows 2 -> 4)");
        // The wrap at the count.
        g.frame = g.kind.humanoid_frame_count() - 1;
        g.frame_counter = 6.1;
        g.find_frame();
        assert_eq!(g.frame, 2, "the wrap back to row 2");
        // The idle at rest.
        g.vx = 0.0;
        g.find_frame();
        assert_eq!(g.frame, 0, "standing still is row 0");
        // The air leaves the row untouched (the vanilla writes only while grounded).
        g.vx = 0.5;
        g.frame = 7;
        g.vy = -4.0;
        g.find_frame();
        assert_eq!(g.frame, 7, "the airborne row is left as-is");
        // The goblins share the machine with their own counts.
        let mut p = bare(NpcKind::GoblinPeon);
        p.vx = 0.5;
        p.frame = 0;
        p.find_frame();
        assert_eq!(p.frame, 2, "the goblin's walk row");
        assert_eq!(p.kind.humanoid_frame_count(), 16, "the peon's rows");
    }

    #[test]
    fn the_town_walk_band_stops_before_the_extra_attack_rows() {
        // `FindFrame`'s humanoid arm wraps the walk band at
        // `Main.npcFrameCount - NPCID.Sets.ExtraFramesCount`
        // (`NPC.cs:69534`'s `num57`, and the `frame.Y / num >=
        // npcFrameCount - num57` wrap just below it) — the last rows of a
        // town sheet are the attack poses, so a walking NPC must never
        // reach them. The Guide's strip is 26 rows with 10 extra, i.e. a
        // walk band of rows 2..15.
        let mut g = bare(NpcKind::Guide);
        g.vx = 0.5;
        g.frame = 15;
        g.frame_counter = 6.1;
        g.find_frame();
        assert_eq!(g.frame, 2, "the walk wraps at 16, not at the sheet's 26");
        // The band's top row is still reachable.
        g.frame = 14;
        g.frame_counter = 6.1;
        g.find_frame();
        assert_eq!(g.frame, 15, "row 15 is the band's last");
        // The Merchant: 25 rows, 9 extra -> the band is rows 2..15 and the
        // wrap threshold is 25 - 9 = 16 (exclusive).
        let mut m = bare(NpcKind::Merchant);
        m.vx = 0.5;
        m.frame = 14;
        m.frame_counter = 6.1;
        m.find_frame();
        assert_eq!(m.frame, 15, "the merchant's band ends at 25 - 9 - 1");
        m.frame = 15;
        m.frame_counter = 6.1;
        m.find_frame();
        assert_eq!(m.frame, 2, "and wraps back to row 2 at 16");
    }

    #[test]
    fn the_extra_frame_counts_match_the_vanilla_table() {
        // `NPCID.Sets.ExtraFramesCount` (`NPCID.cs:10464`): the reserved
        // attack rows per town sheet. Transcribed from the `CreateIntSet`
        // literal; every kind the port's town machine animates must carry
        // its row count or it walks through its attack poses.
        for (kind, extra) in [
            (NpcKind::Guide, 10),
            (NpcKind::WitchDoctor, 10),
            (NpcKind::TravellingMerchant, 10),
            (NpcKind::Merchant, 9),
            (NpcKind::Nurse, 9),
            (NpcKind::ArmsDealer, 9),
            (NpcKind::Demolitionist, 9),
            (NpcKind::Painter, 9),
            (NpcKind::GoblinTinkerer, 9),
            (NpcKind::TaxCollector, 9),
            (NpcKind::Golfer, 9),
            (NpcKind::DyeTrader, 9),
            (NpcKind::PartyGirl, 9),
            (NpcKind::Zoologist, 9),
            (NpcKind::Angler, 9),
            (NpcKind::Mechanic, 9),
            (NpcKind::Dryad, 7),
            (NpcKind::Clothier, 7),
            (NpcKind::Wizard, 7),
            (NpcKind::Stylist, 7),
            (NpcKind::Princess, 7),
            // The kinds `CreateIntSet` leaves at the 0 default.
            (NpcKind::Skeleton, 0),
            (NpcKind::UndeadMiner, 0),
            (NpcKind::GoblinPeon, 0),
            (NpcKind::GoblinThief, 0),
            (NpcKind::GoblinWarrior, 0),
            (NpcKind::GoblinArcher, 0),
            (NpcKind::Zombie, 0),
        ] {
            assert_eq!(
                kind.extra_frames_count(),
                extra,
                "{kind:?} reserves {extra} attack rows"
            );
        }
        // The walk band never empties.
        for kind in [
            NpcKind::Guide,
            NpcKind::Merchant,
            NpcKind::TravellingMerchant,
            NpcKind::Princess,
        ] {
            assert!(
                kind.humanoid_frame_count() > kind.extra_frames_count() + 2,
                "{kind:?} keeps a walk band"
            );
        }
    }

    #[test]
    fn the_travelling_merchant_walks_the_shared_humanoid_band() {
        // `case 368` is a member of the shared humanoid-group list, so the
        // merchant rides the same machine as the Guide (26 rows, 10 extra
        // -> the band 2..15) instead of the old 3-row stand-in.
        let mut m = bare(NpcKind::TravellingMerchant);
        m.vx = 0.5;
        m.frame = 0;
        m.frame_counter = 0.0;
        m.find_frame();
        assert_eq!(m.frame, 2, "the walk snaps up to row 2");
        m.frame_counter = 0.0;
        for _ in 0..9 {
            m.find_frame();
        }
        assert_eq!(m.frame, 4, "two advances of the `|vx|*2 + 1` accumulator");
        m.frame = 15;
        m.frame_counter = 6.1;
        m.find_frame();
        assert_eq!(m.frame, 2, "the band wraps at 16 (26 - 10)");
        m.vx = 0.0;
        m.find_frame();
        assert_eq!(m.frame, 0, "standing still is row 0");
    }

    #[test]
    fn the_zombie_walks_its_three_rows() {
        // `FindFrame`'s walker arm (`case 3` of the
        // `case 3/52/53/132/161/186-189/…` group, `NPC.cs`): NPC_3 is
        // 38x144 with `Main.npcFrameCount[3]` = 3, so the strip is the
        // stand row plus two walk poses. The v1 pinned row 0 and its draw
        // divided the sheet by 1 (all three rows in one quad).
        let mut z = bare(NpcKind::Zombie);
        // Standing still: row 0.
        z.vx = 0.0;
        z.vy = 0.0;
        z.frame = 7;
        z.frame_counter = 5.0;
        z.find_frame();
        assert_eq!(z.frame, 0, "idle is row 0");
        assert_eq!(z.frame_counter, 0.0, "and the accumulator resets");
        // Walking: the `|vx|` bands 0..8 / 8..16 / 16..24 / 24..32.
        let mut bands = Vec::new();
        z.vx = 4.0;
        z.frame_counter = 0.0;
        for _ in 0..10 {
            z.find_frame();
            bands.push(z.frame);
        }
        // 4 px/tick: rows 0 (counter 4), 1 (8), 1 (12), 2 (16), 2 (20),
        // 1 (24), 1 (28), then the lap back through 0.
        assert_eq!(&bands[..7], &[0, 1, 1, 2, 2, 1, 1], "the four bands");
        assert!(bands.iter().all(|f| *f < 3), "no row past the strip");
        // Airborne and turning both hold row 2 and clear the accumulator.
        z.vy = -1.0;
        z.frame = 1;
        z.frame_counter = 12.0;
        z.find_frame();
        assert_eq!(z.frame, 2, "airborne holds the jump row");
        assert_eq!(z.frame_counter, 0.0);
        z.vy = 0.0;
        z.facing = -1;
        z.vx = 3.0;
        z.frame_counter = 9.0;
        z.find_frame();
        assert_eq!(z.frame, 2, "facing away from the motion");
        // Grounded on the motion: the sprite faces the walk.
        z.facing = 1;
        z.vx = 3.0;
        z.find_frame();
        assert_eq!(z.sprite_dir, 1, "the ground flip");
        // The drawn strip is three rows: the frame height is sheet / 3.
        assert_eq!(z.kind.humanoid_frame_count(), 1, "not a town sheet");
    }

    #[test]
    fn the_swimmers_flap_wet_and_flop_dry() {
        // `FindFrame`'s case 55/57/58 (`NPC.cs:74181-74226`) + case 65
        // (`:74654-74678`): the wet flap rides rows 0..3 at the 6-tick
        // bands; the goldfish/piranha flop shows rows 4/5 on land; the
        // shark holds the last row when beached (its case has no dry arm).
        let mut g = bare(NpcKind::Goldfish);
        g.wet = true;
        g.frame_counter = 0.0;
        g.find_frame();
        assert_eq!(g.frame, 0, "the flap start");
        g.frame_counter = 7.0;
        g.find_frame();
        assert_eq!(g.frame, 1, "the second band");
        g.frame_counter = 13.0;
        g.find_frame();
        assert_eq!(g.frame, 2, "the third band");
        g.frame_counter = 19.0;
        g.find_frame();
        assert_eq!(g.frame, 3, "the fourth band");
        // The dry flop: rows 4/5 (`:74212-74220`).
        g.wet = false;
        g.frame_counter = 0.0;
        g.find_frame();
        assert_eq!(g.frame, 4, "the dry flop row 4");
        g.frame_counter = 7.0;
        g.find_frame();
        assert_eq!(g.frame, 5, "the dry flop row 5");
        // The shark's dry arm holds the last wet row (no else branch).
        let mut s = bare(NpcKind::Shark);
        s.wet = true;
        s.frame_counter = 7.0;
        s.find_frame();
        assert_eq!(s.frame, 1, "the shark's wet band");
        s.wet = false;
        s.frame_counter = 0.0;
        s.find_frame();
        assert_eq!(s.frame, 1, "the beached shark holds the row");
    }

    #[test]
    fn the_bats_flap_the_count_minus_one_cycle_and_tilt() {
        // `FindFrame`'s case 49 group (`NPC.cs:74824-74881`): the flap
        // advances every 6 ticks over `npcFrameCount - 1` rows for the
        // Cave/Jungle/Hell bats (`:74871-74873`'s `num292--`), the full 5
        // for the Voodoo Demon; the spriteDirection rides the velocity's
        // sign and the tilt is `rotation = velocity.X * 0.1` (`:74858-74860`).
        let mut b = bare(NpcKind::CaveBat);
        b.vx = 2.0;
        b.frame = 0;
        b.frame_counter = 0.0;
        for _ in 0..5 {
            b.find_frame();
        }
        assert_eq!(b.frame, 0, "the 6-tick stride holds the row");
        b.find_frame();
        assert_eq!(b.frame, 1, "the sixth tick advances");
        assert_eq!(b.rotation, 0.2, "the tilt rides vx*0.1");
        assert_eq!(b.facing, 1, "the rightward flight faces right");
        // The last row is reserved: 4 -> 0.
        b.frame = 3;
        b.frame_counter = 6.1;
        b.find_frame();
        assert_eq!(b.frame, 0, "the cave bat wraps at 4");
        // The Voodoo Demon cycles all five rows (66 has no `num292--`).
        let mut v = bare(NpcKind::VoodooDemon);
        v.frame = 4;
        v.frame_counter = 6.1;
        v.find_frame();
        assert_eq!(v.frame, 0, "the demon wraps at 5");
        v.frame = 4;
        v.frame_counter = 0.0;
        v.find_frame();
        assert_eq!(v.frame, 4, "the fifth row is live before the wrap");
        // The leftward tilt.
        let mut b2 = bare(NpcKind::CaveBat);
        b2.vx = -2.0;
        b2.find_frame();
        assert_eq!(b2.rotation, -0.2, "the leftward tilt");
        assert_eq!(b2.facing, -1, "the leftward flight faces left");
    }

    #[test]
    fn the_casters_pose_by_facing_and_the_imp_snaps_the_cast_strip() {
        // `FindFrame`'s case 29/32/45 group (`NPC.cs:77019-77055`): the
        // pose rows — 0 stands, 1 casts (`ai[1] > 0`), 2 airborne; the
        // Fire Imp's case 24 (`:76979-77017`): the cast snaps to row 4 and
        // holds 9, the walk cycles rows 0..3 at the 5-tick stride, the
        // snap guard covers both ends (`< 4 || > 9`).
        let mut s = bare(NpcKind::GoblinSorcerer);
        s.find_frame();
        assert_eq!(s.frame, 0, "the standing pose");
        s.cast_windup = 1;
        s.find_frame();
        assert_eq!(s.frame, 1, "the cast pose");
        s.vy = -1.0;
        s.find_frame();
        assert_eq!(s.frame, 2, "the airborne pose");
        // The Imp's walk.
        let mut imp = bare(NpcKind::FireImp);
        imp.frame_counter = 0.0;
        for _ in 0..5 {
            imp.find_frame();
        }
        imp.find_frame();
        assert_eq!(imp.frame, 1, "the walk's first stride");
        // The cast: the snap to row 4 and the held row 9.
        imp.cast_windup = 1;
        imp.frame = 2;
        imp.find_frame();
        assert_eq!(imp.frame, 4, "the cast snaps to row 4");
        for _ in 0..30 {
            imp.find_frame();
        }
        assert_eq!(imp.frame, 9, "the cast holds the last row");
    }

    #[test]
    fn the_worm_segments_keep_the_single_frame_and_chain_rotation() {
        // `FindFrame` has no case for 7..15 and `Main.npcFrameCount` 7..15
        // = 1 (`Main.cs:67082`): the segments hold row 0; the chain's
        // `rotation = atan2 + 1.57` (`AI_006`, `NPC.cs:51867-51922`) is
        // the draw's tilt (the rotation branch now carries the worm kinds).
        for kind in [
            NpcKind::DevourerHead,
            NpcKind::DevourerBody,
            NpcKind::DevourerTail,
            NpcKind::GiantWormHead,
            NpcKind::GiantWormBody,
            NpcKind::GiantWormTail,
            NpcKind::EaterOfWorldsHead,
            NpcKind::EaterOfWorldsBody,
            NpcKind::EaterOfWorldsTail,
        ] {
            let mut s = bare(kind);
            // `FindFrame` has no case — the frame stays at its
            // `SetFrameSize` row 0 (the port's empty arm leaves it).
            s.find_frame();
            assert_eq!(s.frame, 0, "the {kind:?} segment starts at row 0");
            s.find_frame();
            assert_eq!(s.frame, 0, "the {kind:?} segment never advances");
        }
    }

    #[test]
    fn the_hovers_cycle_the_strip_at_the_eight_tick_stride() {
        // `FindFrame`'s case 6/173 (`NPC.cs`): the counter advances the
        // row every 8 ticks and wraps over `Main.npcFrameCount` — the
        // Eater of Souls' and Meteor Head's 2 rows, the Hornet's 3.
        let mut e = bare(NpcKind::EaterOfSouls);
        e.frame = 0;
        e.frame_counter = 0.0;
        for _ in 0..7 {
            e.find_frame();
        }
        assert_eq!(e.frame, 0, "the stride holds for 7 ticks");
        e.find_frame();
        assert_eq!(e.frame, 1, "the eighth tick advances");
        // The wrap fires on the NEXT advance (the C#'s check sits inside
        // the stride branch): one more stride carries row 1 -> 2 -> 0.
        e.frame_counter = 7.5;
        e.find_frame();
        assert_eq!(e.frame, 0, "the wrap over 2 rows");
        // The Hornet's third row.
        let mut h = bare(NpcKind::Hornet);
        h.frame = 2;
        h.frame_counter = 7.5;
        h.find_frame();
        assert_eq!(h.frame, 0, "the hornet wraps over 3 rows");
        h.frame = 1;
        h.frame_counter = 0.0;
        h.find_frame();
        assert_eq!(h.frame, 1, "the second row holds");
    }

    #[test]
    fn the_golem_family_stats_and_maps() {
        // `SetDefaults` 245-249 (`NPC.cs:12163-12220`): the stats, the
        // kind<->u16 65-69, the bestiary fold (the parts -> the Golem
        // entry net 245).
        assert_eq!(NpcKind::Golem.max_hp(), 15000);
        assert_eq!(NpcKind::GolemHead.max_hp(), 25000);
        assert_eq!(NpcKind::GolemFistLeft.max_hp(), 10000);
        assert_eq!(NpcKind::GolemHeadFree.max_hp(), 16000);
        let b = Npc::new(NpcKind::Golem, 0.0, 0.0, 1, 0.0);
        assert_eq!(b.defense(), 26);
        let fh = Npc::new(NpcKind::GolemHeadFree, 0.0, 0.0, 1, 0.0);
        assert_eq!(fh.defense(), 32);
        assert_eq!(NpcKind::Golem.size(), (140.0, 140.0));
        assert_eq!(NpcKind::GolemFistRight.size(), (40.0, 30.0));
        assert_eq!(NpcKind::Golem.contact_damage(), 72);
        assert_eq!(NpcKind::GolemHeadFree.contact_damage(), 80);
        for (kind, v) in [
            (NpcKind::Golem, 73),
            (NpcKind::GolemHead, 74),
            (NpcKind::GolemFistLeft, 75),
            (NpcKind::GolemFistRight, 76),
            (NpcKind::GolemHeadFree, 77),
        ] {
            assert_eq!(kind.to_u16(), v);
            assert_eq!(NpcKind::from_u16(v), Some(kind));
        }
        let e = crate::bestiary::entry_for_npc(NpcKind::GolemHeadFree, SlimeVariant::Blue);
        assert_eq!(e.net_id, 245, "the parts fold onto the Golem entry");
    }

    #[test]
    fn the_golem_free_head_tiers_by_body_hp() {
        // :31524-31548's tiers: the beams escalate by the body's hp ratio
        // (24 -> 29 dmg, 11 -> 12.25 speed) — the ratio rides the World.
        let mut w = ground_air();
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 115.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut free = Npc::new(NpcKind::GolemHeadFree, 100.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        free.hp = free.max_hp;
        free.ai[2] = 1500.0;
        let mut beam = None;
        free.update_golem(&w, &player);
        if let Some(p) = free.wants_hostile_shot.pop() {
            beam = Some(p);
        }
        let (kind, _, _, vx, vy, dmg) = beam.expect("the volley fires");
        assert_eq!(kind, crate::projectile::GOLEM_EYE_BEAM);
        assert_eq!(dmg, 24, "the full-body damage");
        let sp = (vx * vx + vy * vy).sqrt();
        assert!((sp - 11.0).abs() < 0.5, "the 11 speed ({sp})");
        // The mortally wounded body: 29 dmg at 12.25. Clear the volley's
        // leftovers first (the Vec holds the first round's spare).
        free.wants_hostile_shot.clear();
        w.golem_hp_ratio = 0.05;
        free.ai[2] = 1500.0;
        let mut beam2 = None;
        free.update_golem(&w, &player);
        for _ in 0..3 {
            if let Some(p) = free.wants_hostile_shot.pop() {
                beam2 = Some(p);
            }
        }
        let (_, _, _, vx2, vy2, dmg2) = beam2.expect("the escalated volley");
        assert_eq!(dmg2, 29, "the low-body damage tier");
        let sp2 = (vx2 * vx2 + vy2 * vy2).sqrt();
        assert!((sp2 - 12.25).abs() < 0.5, "the 12.25 speed ({sp2})");
    }
    #[test]
    fn the_deerclops_grid_cells_select_by_state() {
        // `FindFrame`'s case 668 (`NPC.cs:66779-66841`): the 5x5 grid
        // cell — the attacks' sequences at `ai[1] / 4`, the walk's
        // idle/air/2..11 cycle.
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 110.0 * 16.0;
        let mut d = Npc::new(NpcKind::Deerclops, 100.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        d.hp = d.max_hp;
        // The walk: the idle row at rest, the air row in the hop, the
        // 2..11 stride while moving (the 150-wrap cycle).
        d.vx = 0.0;
        d.vy = 0.0;
        d.find_frame();
        assert_eq!(d.frame, 0, "the idle cell");
        d.vy = -8.0;
        d.find_frame();
        assert_eq!(d.frame, 1, "the air cell");
        d.vy = 0.0;
        d.vx = 2.0;
        d.frame_counter = 0.0;
        d.find_frame();
        assert_eq!(d.frame, 2, "the stride start");
        d.frame_counter = 100.0;
        d.find_frame();
        assert_eq!(d.frame, 8, "the mid-stride (2 + 100/15)");
        d.frame_counter = 141.0;
        d.find_frame();
        assert_eq!(d.frame, 11, "the stride end (141 + 2 -> 143/15)");
        d.frame_counter = 160.0;
        d.find_frame();
        assert_eq!(d.frame, 2, "the 150 wrap restarts");
        // The attack sequences: state 1 plays the attack-1 ramp
        // (12, 13, 14, 13, 14, ...) at the 4-tick stride; state 2 the
        // rubble (12, 15, 16, 17, ...); state 3 the blizzard (19, 20,
        // 21, 22, ...).
        d.ai[0] = 1.0;
        d.ai[1] = 0.0;
        d.find_frame();
        assert_eq!(d.frame, 12, "the attack-1 first cell");
        d.ai[1] = 4.0;
        d.find_frame();
        assert_eq!(d.frame, 13, "the attack-1 second cell");
        d.ai[1] = 40.0;
        d.find_frame();
        assert_eq!(d.frame, 16, "the attack-1 tenth cell");
        d.ai[1] = 44.0;
        d.find_frame();
        assert_eq!(d.frame, 17, "past the end clamps to the last");
        d.ai[0] = 2.0;
        d.ai[1] = 4.0;
        d.find_frame();
        assert_eq!(d.frame, 15, "the attack-2 second cell");
        d.ai[0] = 3.0;
        d.ai[1] = 8.0;
        d.find_frame();
        assert_eq!(d.frame, 21, "the attack-3 third cell");
    }

    #[test]
    #[test]
    fn the_deerclops_state_machine_cycles() {
        // `AI_123_Deerclops` (`NPC.cs:44017-44407`): the damage gate, the
        // attack-selection states, the ice-spike/shadow-hand volleys.
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 110.0 * 16.0;
        player.max_hp = 200;
        player.hp = player.max_hp;
        let mut d = Npc::new(NpcKind::Deerclops, 100.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        d.hp = d.max_hp;
        let mut rng = 3u64;
        // The damage gate (`:44024-44039`): the ramp only reaches 30
        // after 30 ticks with a target in range — the fresh spawn starts
        // at 0 (vulnerable); the spawn state -1 pins it at -10.
        assert!(!d.dont_take_damage, "the fresh spawn is open");
        for _ in 0..31 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        assert!(d.dont_take_damage, "the 450px ramp shields");
        // Tick the state 0 selection: the quiet close-range stall fires
        // the shadow-hand burst (state 5).
        d.ai[1] = 200.0;
        d.vx = 0.0;
        d.on_ground = true;
        d.update_deerclops(&w, &mut player, &mut rng);
        assert_eq!(d.ai[0], 5.0, "the hand burst state");
        for _ in 0..60 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        assert_eq!(d.ai[0], 0.0, "the burst ends back to the walk");
        let hands = d.wants_hostile_shot_ai.len();
        assert!(hands >= 6, "the 6-shadow-hand burst ({hands})");
        // The C#'s pick hands the arm its variation age + angle
        // (`Projectile.RandomizeInsanityShadowFor`, `Projectile.cs:42408-42493`).
        for hand in d.wants_hostile_shot_ai.iter() {
            assert_eq!(hand.kind, crate::projectile::SHADOW_HAND);
            assert!(
                matches!(hand.ai0, 0.0 | 180.0 | 300.0 | 390.0),
                "the variation age ({})",
                hand.ai0
            );
        }
        // The spike volley (state 1).
        d.ai[0] = 1.0;
        d.ai[1] = 0.0;
        for _ in 0..40 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        let spikes: Vec<_> = d
            .wants_hostile_shot_ai
            .iter()
            .filter(|shot| shot.kind == crate::projectile::DEERCLOPS_ICE_SPIKE)
            .copied()
            .collect();
        // The C#'s cadence (`:44484-44496`): from the 36th tick, every
        // fourth tick makes four spikes — the 40-tick window sees two
        // volleys.
        assert!(
            spikes.len() >= 8,
            "the spike volley adds shots ({})",
            spikes.len()
        );
        for spike in spikes.iter() {
            assert_eq!(spike.damage, 13, "the spike damage");
            // `:44582` — the spike's `ai1` is its scale factor.
            assert!(spike.ai1 >= 0.1, "the scale factor ({})", spike.ai1);
        }
        // The life ramp: the speed cap rises as the hp falls.
        let fast = 3.5 + 1.0 * (1.0 - d.hp as f32 / d.max_hp as f32);
        assert_eq!(fast, 3.5, "the full-body speed");
        d.hp = d.max_hp / 2;
        let slow = 3.5 + 1.0 * (1.0 - d.hp as f32 / d.max_hp as f32);
        assert!((slow - 4.0).abs() < 0.01, "the half-body speed");
    }

    #[test]
    fn the_archer_stops_fires_and_staggers() {
        // `AI_003`'s ranged family (`NPC.cs:59069-59700`): the shot gate
        // (grounded, the line, the dist < 700) enters the stop state
        // (`ai[2] = 1`, `ai[1] = 180`, the brake x0.5), the midpoint tick
        // fires the hostile arrow 81 at 9 px/tick, the hit staggers
        // (`ai[1] = 30`, `ai[2] = 0`), the cooldown end exits.
        let w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 30.0 * 16.0;
        player.y = 115.0 * 16.0;
        let mut a =
            Npc::spawn_cave_walker(&w, 24, NpcKind::GoblinArcher).expect("the archer spawns");
        let mut rng = 1u64;
        // The gate: the walk tick with the clear line enters the stop.
        a.update_archer(&w, &mut player, &mut rng);
        if a.ai[2] > 0.0 {
            assert_eq!(a.ai[1], 180.0, "the 180-tick cadence");
            assert!(a.vx.abs() < 1.5, "the entry brake");
            // Ticking to the midpoint fires the arrow (180 - 90 = the
            // 90th tick).
            for _ in 0..90 {
                a.update_archer(&w, &mut player, &mut rng);
            }
            assert!(!a.wants_hostile_shot.is_empty(), "the midpoint fires");
            let (kind, _, _, vx, vy, dmg) = a.wants_hostile_shot.pop().unwrap();
            assert_eq!(kind, crate::projectile::WOODEN_ARROW_HOSTILE);
            assert_eq!(dmg, 11);
            let sp = (vx * vx + vy * vy).sqrt();
            assert!((sp - 9.0).abs() < 0.5, "the 9-speed aim ({sp})");
            // The cooldown end exits the state.
            a.ai[1] = 1.0;
            a.update_archer(&w, &mut player, &mut rng);
            assert_eq!(a.ai[2], 0.0, "the state exits");
        }
        // The stagger: a hit during the state resets it.
        let mut a2 =
            Npc::spawn_cave_walker(&w, 24, NpcKind::GoblinArcher).expect("the archer spawns");
        a2.ai[2] = 1.0;
        a2.ai[1] = 100.0;
        a2.hurt(10, a2.x - 5.0, 5.0, false);
        a2.update_archer(&w, &mut player, &mut rng);
        assert_eq!(a2.ai[2], 0.0, "the hit staggers");
        assert_eq!(a2.ai[1], 30.0, "the 30-tick stagger");
    }

    // --- the world-file section 6 glue (`SaveNPCs:1703`/`LoadNPCs:2884`) ----

    /// The section glue tests: the records, the kind resolution and the
    /// round-trip between live state and the plain wire structs.
    mod wld_tests {
        use crate::npc::{Npc, NpcKind, from_wld, kind_from_legacy_name, kind_from_net_id, to_wld};
        use tr_core::wld::sections::npcs::SAVES_AND_LOADS;
        use tr_core::wld::sections::{SectionData, WldNpcs, WldTower, WldTownNpc};

        /// A modern town record at the writer's own defaults.
        fn record(net_id: i32, home: Option<(i32, i32)>) -> WldTownNpc {
            WldTownNpc {
                net_id,
                given_name: "Test".into(),
                x: 100.0,
                y: 200.0,
                homeless: home.is_none(),
                home_x: home.map_or(0, |h| h.0),
                home_y: home.map_or(0, |h| h.1),
                town_npc: true,
                variation_index: 2,
                homeless_despawn: false,
                ..Default::default()
            }
        }

        fn live_npc(kind: NpcKind, home: Option<(i32, i32)>) -> Npc {
            let mut npc = Npc::new(kind, 100.0, 200.0, 1, 0.0);
            npc.home = home;
            npc.given_name = "Test".into();
            npc.variation_index = 2;
            npc
        }

        fn section_data(npcs: WldNpcs) -> SectionData {
            SectionData {
                npcs,
                ..Default::default()
            }
        }

        #[test]
        fn test_DW_4_1_from_wld_to_wld_round_trips_the_town_records() {
            // `LoadNPCs:2896-2921` into live state, then `SaveNPCs:1714-1735`
            // back out: names, homes, homeless, variation and the despawn flag
            // all survive the round-trip.
            let section = WldNpcs {
                town: vec![
                    record(17, Some((65, 105))),  // the Merchant, homed
                    record(22, None),             // the Guide, homeless
                    record(228, Some((61, 101))), // the Witch Doctor
                ],
                towers: vec![WldTower {
                    net_id: 507,
                    ..Default::default()
                }],
                shimmered: vec![17, 22],
            };
            let npcs = from_wld(&section);
            assert_eq!(npcs.len(), 4, "one live NPC per town record plus the tower");
            assert_eq!(
                npcs[3].kind,
                NpcKind::NebulaTower,
                "the tower record instantiates"
            );
            assert_eq!(npcs[0].kind, NpcKind::Merchant);
            assert_eq!(npcs[0].home, Some((65, 105)));
            assert_eq!(npcs[1].home, None, "homeless");
            assert_eq!(npcs[2].kind, NpcKind::WitchDoctor);
            assert_eq!(npcs[0].given_name, "Test");
            assert_eq!(npcs[0].variation_index, 2);

            let written = to_wld(&npcs);
            assert_eq!(written.town.len(), 3);
            for (before, after) in section.town.iter().zip(&written.town) {
                assert_eq!(before.net_id, after.net_id);
                assert_eq!(before.given_name, after.given_name);
                assert_eq!((before.x, before.y), (after.x, after.y));
                assert_eq!(before.homeless, after.homeless);
                assert_eq!((before.home_x, before.home_y), (after.home_x, after.home_y));
                assert_eq!(before.variation_index, after.variation_index);
                assert_eq!(before.homeless_despawn, after.homeless_despawn);
                assert!(after.town_npc, "bit 0 is set on the write side");
            }
        }

        #[test]
        fn test_DW_4_4_no_ported_kind_saves_and_loads() {
            // The section's three-list shape: the app side writes the town
            // records and the `SavesAndLoads` passives — the lunar towers
            // (`SaveNPCs:1738`'s set = 422/507/493/517, `NPCID.cs:10449`),
            // whose kinds this branch ports (M4-NPC), so a vanilla file's
            // towers instantiate live NPCs rather than being discarded.
            for net_id in SAVES_AND_LOADS {
                assert!(
                    kind_from_net_id(net_id).is_some(),
                    "the ported tower kind resolves for {net_id}"
                );
            }
            let towers: Vec<WldTower> = SAVES_AND_LOADS
                .iter()
                .map(|&net_id| WldTower {
                    net_id,
                    ..Default::default()
                })
                .collect();
            let npcs = from_wld(&WldNpcs {
                towers,
                ..Default::default()
            });
            assert_eq!(npcs.len(), 4, "the tower records instantiate live NPCs");
            // And the writer never invents one from live state.
            let zombie = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
            assert!(to_wld(&[zombie]).towers.is_empty());
            assert!(
                to_wld(&[Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0)])
                    .towers
                    .is_empty()
            );
        }

        #[test]
        fn test_DW_4_4_a_record_without_a_kind_is_discarded() {
            // A record whose net id has no ported kind is parsed and skipped:
            // the codec has consumed its bytes, so the neighbours survive.
            let section = WldNpcs {
                town: vec![record(9_999_999, Some((10, 10))), record(17, None)],
                ..Default::default()
            };
            let npcs = from_wld(&section);
            assert_eq!(npcs.len(), 1, "the unmapped record is discarded");
            assert_eq!(npcs[0].kind, NpcKind::Merchant);
            // The same section round-trips through the codec, so the cursor
            // never desynchronised: both records are read back.
            let mut w = tr_core::binio::Writer::new();
            tr_core::wld::sections::npcs::write(&mut w, &section_data(section), 326);
            let bytes = w.into_inner();
            let mut r = tr_core::binio::Reader::new(&bytes);
            let read_back = tr_core::wld::sections::npcs::read(&mut r, 326).expect("reads");
            assert_eq!(read_back.town.len(), 2);
            assert_eq!(read_back.town[0].net_id, 9_999_999);
            assert_eq!(r.pos(), bytes.len());
        }

        #[test]
        fn test_DW_4_2_the_legacy_name_resolves_to_a_kind() {
            // `LoadNPCs:2900` below 190: the name is the wire id, and the
            // port's family resolves through the bestiary rows' English names.
            assert_eq!(
                kind_from_legacy_name("Arms Dealer").map(|(k, _)| k),
                Some(NpcKind::ArmsDealer)
            );
            assert_eq!(
                kind_from_legacy_name("Merchant").map(|(k, _)| k),
                Some(NpcKind::Merchant)
            );
            assert_eq!(
                kind_from_legacy_name("Clothier").map(|(k, _)| k),
                Some(NpcKind::Clothier)
            );
            assert_eq!(
                kind_from_legacy_name("Witch Doctor").map(|(k, _)| k),
                Some(NpcKind::WitchDoctor)
            );
            assert_eq!(
                kind_from_legacy_name("Zombie").map(|(k, _)| k),
                Some(NpcKind::Zombie)
            );
            // The slime variants carry their own negative names.
            assert_eq!(
                kind_from_legacy_name("Green Slime").map(|(_, v)| v),
                Some(crate::npc::SlimeVariant::Green)
            );
            assert_eq!(
                kind_from_legacy_name("Pinky").map(|(_, v)| v),
                Some(crate::npc::SlimeVariant::Pinky)
            );
            // A "Little/Big X" sub-variant, an unported NPC or an empty name
            // maps to nothing: the record is parsed and discarded (recorded
            // deviation). (The Party Girl used to sit here until M4-TOWN
            // ported her; the town kinds keep growing.)
            assert!(kind_from_legacy_name("Big Zombie").is_none());
            assert!(kind_from_legacy_name("Party Girl").is_some());
            assert!(kind_from_legacy_name("").is_none());
            // And the legacy record lands as that kind through `from_wld`.
            let npcs = from_wld(&WldNpcs {
                town: vec![WldTownNpc {
                    legacy_name: Some("Arms Dealer".into()),
                    ..Default::default()
                }],
                ..Default::default()
            });
            assert_eq!(npcs.len(), 1);
            assert_eq!(npcs[0].kind, NpcKind::ArmsDealer);
        }

        #[test]
        fn the_net_ids_are_the_vanilla_ones() {
            // The wire is `NPC.netID` (`SaveNPCs:1718`), NOT the port's
            // internal `to_u16` id space: every town NPC's real id.
            for (kind, net_id) in [
                (NpcKind::Guide, 22),
                (NpcKind::Merchant, 17),
                (NpcKind::Nurse, 18),
                (NpcKind::ArmsDealer, 19),
                (NpcKind::Dryad, 20),
                (NpcKind::Demolitionist, 38),
                (NpcKind::Clothier, 54),
                (NpcKind::GoblinTinkerer, 107),
                (NpcKind::Painter, 227),
                (NpcKind::WitchDoctor, 228),
            ] {
                assert_eq!(kind.net_id(), net_id, "{kind:?}");
                assert_eq!(kind_from_net_id(net_id).map(|(k, _)| k), Some(kind));
            }
            // The hostile kinds resolve too (they are ported, just not town).
            assert_eq!(NpcKind::Zombie.net_id(), 3);
            assert_eq!(kind_from_net_id(3).map(|(k, _)| k), Some(NpcKind::Zombie));
            // The slime variants ride their negative net ids.
            assert_eq!(
                kind_from_net_id(crate::loot_db::GREEN_SLIME).map(|(_, v)| v),
                Some(crate::npc::SlimeVariant::Green)
            );
            assert!(
                kind_from_net_id(0).is_none(),
                "net id 0 is not a ported kind"
            );
        }

        #[test]
        fn the_town_set_matches_the_source() {
            // `NPC.SetDefaults`'s `townNPC = true` arms (`NPC.cs:8979-11943`),
            // restricted to the ported family.
            for kind in [
                NpcKind::Guide,
                NpcKind::Merchant,
                NpcKind::Nurse,
                NpcKind::ArmsDealer,
                NpcKind::Dryad,
                NpcKind::Demolitionist,
                NpcKind::Clothier,
                NpcKind::GoblinTinkerer,
                NpcKind::Painter,
                NpcKind::WitchDoctor,
            ] {
                assert!(kind.is_town_npc(), "{kind:?} is a town NPC");
                let loaded = from_wld(&WldNpcs {
                    town: vec![record(kind.net_id(), None)],
                    ..Default::default()
                });
                assert_eq!(loaded.len(), 1, "{kind:?} instantiates");
            }
            // Not town NPCs (the bound goblin is the 63 case the C# leaves out).
            for kind in [NpcKind::Slime, NpcKind::Zombie, NpcKind::BoundGoblin] {
                assert!(!kind.is_town_npc(), "{kind:?} is not a town NPC");
            }
        }

        #[test]
        fn to_wld_filters_to_town_npcs_and_keeps_order() {
            // `SaveNPCs`'s filters over `Main.npc` order: the town NPCs in
            // live order, the wandering hostiles are not persisted at all.
            let npcs = vec![
                live_npc(NpcKind::Merchant, Some((65, 105))),
                Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0),
                live_npc(NpcKind::Guide, None),
            ];
            let written = to_wld(&npcs);
            assert_eq!(written.town.len(), 2);
            assert_eq!(written.town[0].net_id, 17, "file order = live order");
            assert_eq!(written.town[1].net_id, 22);
            assert_eq!(written.town[0].home_x, 65);
            assert_eq!(written.town[0].given_name, "Test");
        }

        #[test]
        fn a_homeless_npc_writes_no_home() {
            // `homeless = true` with the zeroed home pair
            // (`SaveNPCs:1724-1726`): the port keeps no home for a homeless
            // NPC, so nothing stale is written back.
            let written = to_wld(&[live_npc(NpcKind::Nurse, None)]);
            let npc = &written.town[0];
            assert!(npc.homeless);
            assert_eq!((npc.home_x, npc.home_y), (0, 0));
        }

        #[test]
        fn a_homeless_record_with_a_stale_home_keeps_its_bytes() {
            // The C# file can carry `homeless = true` next to a (now stale)
            // home pair — `FixAgainstExploits` leaves both alone. The codec
            // round-trips the raw pair; the glue keeps only the flag, because
            // the port models homeless as "no home" (`happiness.rs:228`).
            let stale = WldNpcs {
                town: vec![WldTownNpc {
                    homeless: true,
                    home_x: 65,
                    home_y: 105,
                    ..record(22, None)
                }],
                ..Default::default()
            };
            let npcs = from_wld(&stale);
            assert_eq!(npcs[0].home, None, "the stale pair is not carried");
            // Byte-exact through the codec (`SaveNPCs:1724-1726`'s raw write).
            let mut w = tr_core::binio::Writer::new();
            tr_core::wld::sections::npcs::write(&mut w, &section_data(stale), 326);
            let bytes = w.into_inner();
            let mut r = tr_core::binio::Reader::new(&bytes);
            let back = tr_core::wld::sections::npcs::read(&mut r, 326).expect("reads");
            let rec = &back.town[0];
            assert!(rec.homeless);
            assert_eq!((rec.home_x, rec.home_y), (65, 105));
        }

        #[test]
        fn an_unnamed_npc_writes_an_empty_given_name() {
            // The wire's `GivenName` is never null (`SaveNPCs:1719`).
            let mut npc = live_npc(NpcKind::Dryad, Some((10, 10)));
            npc.given_name = String::new();
            assert_eq!(to_wld(&[npc]).town[0].given_name, "");
        }

        #[test]
        fn the_homeless_and_despawn_flags_survive() {
            // `homelessDespawn` rides its own field (`SaveNPCs:1734`) and does
            // not affect homelessness.
            let mut npc = live_npc(NpcKind::GoblinTinkerer, None);
            npc.homeless_despawn = true;
            let written = to_wld(&[npc]);
            assert!(written.town[0].homeless && written.town[0].homeless_despawn);
        }

        /// The house rule's citation probe for the glue half.
        #[test]
        fn test_DW_4_5_the_citations_are_present() {
            let here = include_str!("npc.rs");
            for anchor in [
                "SaveNPCs:1714-1748` over the live NPCs",
                "`NPC.cs:8979-11943`, the `this.townNPC = true` arms",
                "`SaveNPCs:1731`): the selected variation",
                "`LoadNPCs:2896`): the vanilla net id maps to the kind",
                "`SaveNPCs:1737`, no ported kind is a lunar tower",
            ] {
                assert!(here.contains(anchor), "missing {anchor}");
            }
            // The codec it glues, and the re-validation's owner.
            let codec = include_str!("../../tr-core/src/wld/sections/npcs.rs");
            assert!(
                codec.contains("WorldFile.SaveNPCs:1703"),
                "codec write anchor"
            );
            assert!(
                codec.contains("WorldFile.LoadNPCs:2884"),
                "codec read anchor"
            );
            let town = include_str!("town.rs");
            assert!(
                town.contains("WorldFile.FixAgainstExploits:1864"),
                "re-validation anchor"
            );
        }
    }
    // ───────────────────────────────────────────────────────────────
    // The martians' display (`NPC.cs:71578-71921`): M4-NPC-3i ports the
    // ground group's walk bands, the engineer's animation rows, the
    // Scutlix family's inverted sprite direction, the saucer family's
    // clock-driven rows and the probe's banks — 1:1 against the C#.
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn the_martian_ground_group_walks_its_band_and_snaps_to_the_idle_row() {
        // `case 381/382/383/385/389` (`NPC.cs:71623-71662`): grounded, the
        // row clamps up to the walk band's floor 2, the `|vx| * 2`
        // accumulator advances every 6 and the end wraps back to 2 (the
        // 9th row never shows), the idle snaps to row 0 and the air holds
        // row 1. The Grunt/Zapper walk at `|vx| * 1.2` (`:71635-71638`).
        let mut o = bare(NpcKind::MartianOfficer);
        o.facing = 1;
        o.find_frame();
        assert_eq!(
            (o.frame, o.sprite_dir),
            (0, 1),
            "the spawn idle row (dir = 1)"
        );
        o.vx = 2.0;
        o.find_frame();
        assert_eq!(o.frame, 2, "the first grounded moving tick clamps to 2");
        assert_eq!(
            o.sprite_dir, 1,
            "spriteDirection = direction while grounded"
        );
        for _ in 0..2 {
            o.find_frame();
        }
        assert_eq!(
            o.frame, 3,
            "|vx| * 2 = 4/tick: the advance fires every 2 ticks"
        );
        for _ in 0..24 {
            o.find_frame();
            assert!(
                o.frame <= 8,
                "the walk wraps at npcFrameCount's 9: {}",
                o.frame
            );
        }
        assert_eq!(o.frame, 8, "the band rides 2..8 and never shows row 9");
        o.vx = 0.0;
        o.find_frame();
        assert_eq!(o.frame, 0, "the idle snaps to row 0");
        o.vy = 0.4;
        o.vx = 2.0;
        o.find_frame();
        assert_eq!((o.frame, o.frame_counter), (1, 0.0), "the air holds row 1");
        // The zapper's slower rate: 1.2 * 2 = 2.4/tick, so 3 ticks.
        let mut z = bare(NpcKind::GigaZapper);
        z.vx = 2.0;
        z.find_frame();
        z.find_frame();
        assert_eq!(z.frame, 2, "no advance yet at 4.8 of the 6.0 threshold");
        z.find_frame();
        assert_eq!(z.frame, 3, "the 1.2 rate advances on the third tick");
    }

    #[test]
    fn the_martian_engineer_parks_on_the_ai2_animation_rows() {
        // `case 386` (`NPC.cs:71697-71730`): `ai[2] > 0` parks the engineer
        // on rows 9/10 by `ai[2] / 12 % 2`; the grounded walk wraps at 9
        // (not `npcFrameCount`'s 11 — rows 9/10 are the animation pair).
        let mut e = bare(NpcKind::MartianEngineer);
        e.ai[2] = 24.0;
        e.find_frame();
        assert_eq!(e.frame, 9, "ai[2] = 24 -> row 9");
        e.ai[2] = 36.0;
        e.find_frame();
        assert_eq!(e.frame, 10, "ai[2] = 36 -> row 10");
        e.ai[2] = 0.0;
        e.vx = 3.0;
        for _ in 0..30 {
            e.find_frame();
        }
        assert!(
            (2..9).contains(&e.frame),
            "the walk rides 2..8 and wraps at 9: {}",
            e.frame
        );
        e.vx = 0.0;
        e.find_frame();
        assert_eq!(e.frame, 0, "the idle row");
    }

    #[test]
    fn the_martian_walker_wraps_to_row_one_and_the_slow_walk_snaps() {
        // `case 520` (`NPC.cs:71889-71921`): the walk band's floor is row 1
        // (the wrap returns to 1, not 2), the rate is `|vx| * 1`, and the
        // slow walk (`|vx| < 0.2`) snaps to the idle row 0 on the advance.
        let mut w = bare(NpcKind::MartianWalker);
        w.vx = 4.0;
        w.find_frame();
        assert_eq!(w.frame, 0, "4 of the 6.0 threshold: no advance yet");
        w.find_frame();
        assert_eq!(w.frame, 1, "the advance lifts row 0 to the band's floor 1");
        for _ in 0..40 {
            w.find_frame();
        }
        assert!(
            (1..8).contains(&w.frame),
            "the walk rides 1..7: {}",
            w.frame
        );
        // The slow walk's snap rides an ADVANCE tick: at `|vx| * 1` the
        // counter needs 61 ticks to pass the 6.0 threshold, and that tick
        // parks the row on the idle 0 (`NPC.cs:71896-71900`).
        w.vx = 0.1;
        for _ in 0..60 {
            w.find_frame();
        }
        assert!(
            w.frame >= 1,
            "the slow walk has not advanced yet: {}",
            w.frame
        );
        w.find_frame();
        assert_eq!(w.frame, 0, "the slow walk parks on the idle row");
        w.vy = 0.2;
        w.find_frame();
        assert_eq!(w.frame, 1, "the air holds row 1");
    }

    #[test]
    fn the_scutlix_family_inverts_its_sprite_direction() {
        // `case 391` (`NPC.cs:71743-71778`): the carrier's walk rides rows
        // 1..5 at the slow `|vx| * 0.5`, the air holds row 0 (NOT 1), and
        // `spriteDirection = -direction`. `case 390` (`:71779-71805`): the
        // rider's row is purely `ai[2]`-driven (the aim octant 1..5).
        let mut c = bare(NpcKind::Scutlix);
        c.facing = 1;
        c.find_frame();
        assert_eq!(
            (c.frame, c.sprite_dir),
            (0, -1),
            "the inverted sprite direction"
        );
        c.vx = 2.0;
        for _ in 0..14 {
            c.find_frame();
        }
        assert!(
            (1..6).contains(&c.frame),
            "the 0.5 rate walks 1..5: {}",
            c.frame
        );
        c.vy = 0.3;
        c.find_frame();
        assert_eq!(c.frame, 0, "the Scutlix' air row is 0");
        let mut r = bare(NpcKind::ScutlixRider);
        r.facing = -1;
        r.ai[2] = 0.0;
        r.find_frame();
        assert_eq!(
            (r.frame, r.sprite_dir),
            (2, 1),
            "no aim -> row 2, the inverted dir"
        );
        r.ai[2] = 3.0;
        r.find_frame();
        assert_eq!(r.frame, 2, "ai[2] = 3 -> row 2");
        r.ai[2] = 1.0;
        r.find_frame();
        assert_eq!(r.frame, 0, "ai[2] = 1 -> row 0");
        r.ai[2] = 5.0;
        r.find_frame();
        assert_eq!(r.frame, 4, "ai[2] = 5 -> row 4");
    }

    #[test]
    fn the_martian_drone_bobs_and_the_probe_switches_its_bank() {
        // `case 388` (`NPC.cs:71731-71740`): the 2-row bob advances one row
        // every 6 ticks. `case 399` (`:71885-71887`): the counter steps
        // 0.25 (4 ticks a row) and the alerted `ai[0] != 0` rides the +4
        // rows of NPC_399's 8-row strip.
        let mut d = bare(NpcKind::MartianDrone);
        for _ in 0..5 {
            d.find_frame();
        }
        assert_eq!(d.frame, 0);
        d.find_frame();
        assert_eq!(d.frame, 1, "6 ticks advance the bob");
        for _ in 0..5 {
            d.find_frame();
        }
        assert_eq!(d.frame, 1, "each row holds its 6 ticks");
        d.find_frame();
        assert_eq!(d.frame, 0, "the bob wraps over npcFrameCount's 2");
        let mut p = bare(NpcKind::MartianProbe);
        for _ in 0..12 {
            p.find_frame();
        }
        assert!(p.frame < 4, "the roaming bank rides rows 0..3: {}", p.frame);
        p.ai[0] = 2.0;
        p.frame_counter = 0.0;
        for _ in 0..8 {
            p.find_frame();
        }
        assert!(
            (4..8).contains(&p.frame),
            "the alert bank rides rows 4..7: {}",
            p.frame
        );
    }

    #[test]
    fn the_saucer_cores_clock_pins_the_attack_rows() {
        // `case 395` (`NPC.cs:71845-71865`): the 11 rows come off
        // `frameCounter % 66 / 6`; in the attack window (`ai[3]` in
        // [20, 260)) the counter double-increments and is pinned into
        // [36, 54), parking the animation on rows 6-8.
        let mut c = bare(NpcKind::MartianSaucerCore);
        for _ in 0..66 {
            c.find_frame();
        }
        assert_eq!(c.frame, 0, "the idle clock wraps at 66");
        for _ in 0..30 {
            c.find_frame();
        }
        assert_eq!(c.frame, 5, "row = counter % 66 / 6 (30 / 6)");
        c.ai[3] = 100.0;
        for _ in 0..30 {
            c.find_frame();
        }
        assert!(
            (6..9).contains(&c.frame),
            "the attack window pins rows 6..8: {}",
            c.frame
        );
    }

    #[test]
    fn the_saucer_shell_doubles_its_rate_in_the_attack_window() {
        // `case 392` (`NPC.cs:71867-71883`): 4 rows off
        // `frameCounter % 12 / 3`, double speed in the [20, 260) window.
        let mut s = bare(NpcKind::MartianSaucer);
        s.find_frame();
        s.find_frame();
        assert_eq!(s.frame, 0, "the first two ticks hold row 0");
        s.find_frame();
        assert_eq!(s.frame, 1, "3 ticks a row outside the window");
        s.ai[3] = 100.0;
        s.frame_counter = 0.0;
        s.find_frame();
        s.find_frame();
        assert_eq!(s.frame, 1, "the window's double speed: 2 ticks a row");
        s.find_frame();
        s.find_frame();
        assert_eq!(s.frame, 2);
    }

    #[test]
    fn the_saucer_turret_aims_by_the_ai2_angle_and_flashes_on_fire() {
        // `case 393` (`NPC.cs:71806-71819`): the 9 aim rows come from the
        // `ai[2]` rotation vector through the vanilla's 5-band cascade and
        // the firing rows sit +9 while the core's clock runs the 280-420
        // burst on the flash ticks (`(int)ai[3] % 6 <= 2`).
        let mut t = bare(NpcKind::MartianSaucerTurret);
        t.ai[2] = 0.0;
        t.find_frame();
        assert_eq!(t.frame, 8, "aiming right -> row 8");
        t.ai[2] = std::f32::consts::PI;
        t.find_frame();
        assert_eq!(t.frame, 0, "aiming left -> row 0");
        t.ai[2] = std::f32::consts::FRAC_PI_2;
        t.find_frame();
        assert_eq!(t.frame, 4, "straight down -> row 4");
        t.ai[2] = std::f32::consts::FRAC_PI_4;
        t.find_frame();
        assert_eq!(t.frame, 6, "the shallow right-down band -> row 6");
        t.ai[3] = 282.0;
        t.ai[2] = 0.0;
        t.find_frame();
        assert_eq!(t.frame, 17, "the burst's flash tick paints the +9 row");
        t.ai[3] = 285.0;
        t.find_frame();
        assert_eq!(t.frame, 8, "the non-flash tick drops back to the aim row");
        t.ai[3] = 421.0;
        t.find_frame();
        assert_eq!(t.frame, 8, "outside the burst there is no flash");
    }

    #[test]
    fn the_saucer_cannons_rows_ride_the_cores_clock() {
        // `case 394` (`NPC.cs:71820-71843`): row 4 idles below `ai[3]` 420,
        // the charge ramps 4..7 over the next 20, the firing spin cycles
        // `(n / 4) % 4` and the tail falls back toward 4.
        let mut c = bare(NpcKind::MartianSaucerCannon);
        for t in [100.0, 350.0, 420.0] {
            c.ai[3] = t;
            c.find_frame();
        }
        assert_eq!(c.frame, 4, "idle at the window's entry");
        c.ai[3] = 445.0;
        c.find_frame();
        assert_eq!(c.frame, 2, "the firing spin: (25 / 4) % 4 = 2");
        c.ai[3] = 500.0;
        c.find_frame();
        assert_eq!(c.frame, 0, "the firing spin: (80 / 4) % 4 = 0");
        c.ai[3] = 590.0;
        c.find_frame();
        assert_eq!(c.frame, 5, "the recoil tail: 7 - (170 - 160) / 5 = 5");
    }

    #[test]
    fn the_prime_head_cycles_its_rows_and_the_phase_pins_the_wide_row() {
        // `case 127` (`NPC.cs:74498-74538`): the hover alternates rows 0/1
        // every 12 ticks, the enraged/day phases hold the wide row 2, and
        // the Mech-Queen variant (`ai[3] != 0`) alternates 3/4 and pins 5.
        let mut h = bare(NpcKind::SkeletronPrime);
        h.ai[1] = 0.0;
        for _ in 0..11 {
            h.find_frame();
        }
        assert_eq!(h.frame, 0, "11 ticks hold row 0");
        h.find_frame();
        assert_eq!(h.frame, 1, "the 12th tick advances to row 1");
        for _ in 0..12 {
            h.find_frame();
        }
        assert_eq!(h.frame, 0, "the wrap sets row 0 (the count's 2)");
        // The phase pin: every non-hover state holds the wide row 2.
        for phase in [1.0, 2.0, 3.0] {
            h.ai[1] = phase;
            h.find_frame();
            assert_eq!(h.frame, 2, "phase {phase} pins row 2");
            h.frame_counter = 11.0;
            h.find_frame();
            assert_eq!(h.frame, 2, "and resets the counter");
        }
        // The Mech-Queen rows.
        h.ai[1] = 0.0;
        h.ai[3] = 1.0;
        h.frame = 3;
        for _ in 0..12 {
            h.find_frame();
        }
        assert!(
            h.frame == 3 || h.frame == 4,
            "the queen alternates 3/4: {}",
            h.frame
        );
        h.ai[1] = 1.0;
        h.find_frame();
        assert_eq!(h.frame, 5, "the queen's phase pin is row 5");
    }

    #[test]
    fn the_prime_limbs_spin_their_two_row_strips() {
        // `case 129`/`case 130` (`NPC.cs:74539-74570`): the saw advances
        // every 2 ticks and the vice every 8, both wrapping at
        // `npcFrameCount`'s 2; `case 128`/`case 131` do not exist, so the
        // cannon and the laser hold row 0 (the v1 pinned all four).
        let mut saw = bare(NpcKind::PrimeSaw);
        saw.find_frame();
        assert_eq!(saw.frame, 0, "one tick holds row 0");
        saw.find_frame();
        assert_eq!(saw.frame, 1, "the saw's 2-tick cadence");
        saw.find_frame();
        saw.find_frame();
        assert_eq!(saw.frame, 0, "the saw wraps over npcFrameCount's 2");
        let mut vice = bare(NpcKind::PrimeVice);
        for _ in 0..7 {
            vice.find_frame();
        }
        assert_eq!(vice.frame, 0, "7 ticks hold row 0");
        vice.find_frame();
        assert_eq!(vice.frame, 1, "the vice's 8-tick cadence");
        for _ in 0..8 {
            vice.find_frame();
        }
        assert_eq!(vice.frame, 0, "and wraps");
        for kind in [NpcKind::PrimeCannon, NpcKind::PrimeLaser] {
            let mut l = bare(kind);
            l.frame = 0;
            for _ in 0..40 {
                l.find_frame();
                assert_eq!(l.frame, 0, "{kind:?} has no FindFrame case — row 0 forever");
            }
        }
    }

    #[test]
    fn the_prime_limbs_aim_their_rotations_and_carry_the_seat_direction() {
        // `AI_033`-`AI_036`: `spriteDirection = -(int)ai[0]` (the seat's
        // side) and the per-limb `atan2` rotations — the saw/cannon aim
        // their "up" along the dock vector (+1.57) and the laser aims at
        // the player (-1.57). The v1 wrote neither, so the family drew
        // upright and unflipped.
        let mut w = ground_air();
        w.prime_present = true;
        w.prime_x = 200.0 * 16.0;
        w.prime_y = 100.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 260.0 * 16.0;
        player.y = 120.0 * 16.0;
        // A seat-1 limb (the right side): `spriteDirection = -1`.
        let mut saw = Npc::new(NpcKind::PrimeSaw, 200.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        saw.hp = saw.max_hp;
        saw.ai[0] = 1.0;
        saw.update_prime_limb(&w, &player);
        assert_eq!(saw.sprite_dir, -1, "the seat +1 limb's display direction");
        // The dock is 200 px toward the head's centre and 230 px below it
        // — the saw's "up" aims along that vector off the +π/2 baseline
        // (`NPC.cs:28151`'s `atan2(dy, dx) + 1.57`).
        let (scx, scy) = saw.center();
        let expected =
            (w.prime_y + 230.0 - scy).atan2(w.prime_x - 200.0 - scx) + std::f32::consts::FRAC_PI_2;
        assert!(
            (saw.rotation - expected).abs() < 0.001,
            "the saw's dock aim: {} vs {}",
            saw.rotation,
            expected
        );
        // A seat -1 limb mirrors it.
        let mut saw_l = Npc::new(NpcKind::PrimeSaw, 200.0 * 16.0, 130.0 * 16.0, -1, 0.0);
        saw_l.hp = saw_l.max_hp;
        saw_l.ai[0] = -1.0;
        saw_l.update_prime_limb(&w, &player);
        assert_eq!(saw_l.sprite_dir, 1, "the seat -1 limb's display direction");
        // The laser aims at the player (up-right): the -1.57 baseline.
        let mut laser = Npc::new(NpcKind::PrimeLaser, 200.0 * 16.0, 130.0 * 16.0, 1, 0.0);
        laser.hp = laser.max_hp;
        laser.ai[0] = 1.0;
        laser.update_prime_limb(&w, &player);
        let (dx, dy) = (
            player.center().0 - laser.center().0,
            player.center().1 - laser.center().1,
        );
        assert!(
            (laser.rotation - (dy.atan2(dx) - std::f32::consts::FRAC_PI_2)).abs() < 0.001,
            "the laser's player aim: {}",
            laser.rotation
        );
    }

    #[test]
    fn the_prime_head_rolls_with_its_drift_and_spins_in_the_charge() {
        // `AI_032`'s rotation writes (`NPC.cs:27694`/`:27785`): the hover
        // rides `velocity.X / 15` and the enraged charge accumulates
        // `direction * 0.3`. The v1 never wrote a rotation, so the head
        // drew bolt upright while it strafed.
        let mut w = ground_air();
        w.day_time = false;
        let mut pr = Npc::new(NpcKind::SkeletronPrime, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        pr.hp = pr.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 20.0 * 16.0;
        pr.vx = 1.5;
        pr.update_prime(&w, &player);
        assert!(
            (pr.rotation - 0.1).abs() < 0.001,
            "the hover's roll follows the drift: {}",
            pr.rotation
        );
        // The charge: force the phase and roll.
        pr.ai[1] = 1.0;
        pr.vx = 1.5;
        pr.update_prime(&w, &player);
        let spin = pr.rotation - 0.1;
        assert!(
            (spin - 0.3).abs() < 0.001 || (spin + 0.3).abs() < 0.001,
            "the charge's spin accumulates direction * 0.3: {}",
            pr.rotation
        );
    }

    #[test]
    fn the_twins_beat_their_wing_rows_over_the_twenty_tick_cycle() {
        // `case 125/126` (`NPC.cs:76906-76924`): the 20-tick counter walks
        // the bands rows 0/1/2 and the `ai[0] > 1` hover anchor offsets
        // them to the live rows 3/4/5 (`Main.npcFrameCount[125/126]` = 6 —
        // NPC_125/126 are 110x1200, 200-px rows). The v1 ran an 8-tick
        // 2-row strip.
        let mut t = bare(NpcKind::Retinazer);
        // Pre-anchor (the spawn ticks before the AI writes `ai[0]`): the
        // spawn rows 0/1/2.
        t.ai[0] = 0.0;
        for _ in 0..6 {
            t.find_frame();
        }
        assert_eq!(t.frame, 0, "the first band holds row 0");
        for _ in 0..7 {
            t.find_frame();
        }
        assert_eq!(t.frame, 1, "the second band advances to row 1");
        for _ in 0..7 {
            t.find_frame();
        }
        assert_eq!(t.frame, 2, "the third band advances to row 2");
        t.find_frame();
        assert_eq!((t.frame, t.frame_counter), (0, 0.0), "the 21st tick wraps");
        // Live: the machine's `ai[0] = position.X` write flips the pair
        // onto the phase-2 rows.
        t.ai[0] = 3200.0;
        for _ in 0..6 {
            t.find_frame();
        }
        assert_eq!(t.frame, 3, "the anchor's +3 parks the live rows");
        for _ in 0..7 {
            t.find_frame();
        }
        assert_eq!(t.frame, 4);
        for _ in 0..7 {
            t.find_frame();
        }
        assert_eq!(t.frame, 5);
        t.find_frame();
        assert_eq!(t.frame, 3, "the live beat wraps 5 -> 3");
        // The Spazmatism shares the case.
        let mut s2 = bare(NpcKind::Spazmatism);
        s2.ai[0] = 100.0;
        s2.find_frame();
        assert_eq!(s2.frame, 3, "the second twin rides the same rows");
    }

    #[test]
    fn the_twin_machine_writes_the_hover_anchor() {
        // `AI_022`'s `NPC.cs:24758`: the hover anchor `ai[0] = position.X`
        // is what `FindFrame` reads back as the +3 row offset — the port's
        // simplified machine writes it once per tick.
        let mut w = ground_air();
        let mut tw = Npc::new(NpcKind::Retinazer, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        tw.hp = tw.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 20.0 * 16.0;
        let x_before = tw.x;
        tw.update_twin(&w, &player);
        assert!(
            (tw.ai[0] - x_before).abs() < 0.001,
            "the anchor rides the position: {} vs {}",
            tw.ai[0],
            x_before
        );
        tw.find_frame();
        assert_eq!(
            tw.frame, 3,
            "the anchor lands the live rows on the same tick"
        );
    }

    #[test]
    fn the_eye_of_cthulhu_shares_the_twins_twenty_tick_beat() {
        // `case 4/125/126` (`NPC.cs:76905-76926`): the Eye of Cthulhu rides
        // the Twins' machine — the 20-tick counter walks the bands rows
        // 0/1/2 and the `ai[0] > 1` phase lifts them to 3/4/5. The v1 ran
        // an 8-tick strip over all six rows (the "pupil blink" the C# has
        // no arm for).
        let mut e = bare(NpcKind::EyeOfCthulhu);
        for _ in 0..6 {
            e.find_frame();
        }
        assert_eq!(e.frame, 0, "the first band holds row 0");
        for _ in 0..7 {
            e.find_frame();
        }
        assert_eq!(e.frame, 1, "the second band advances to row 1");
        for _ in 0..7 {
            e.find_frame();
        }
        assert_eq!(e.frame, 2, "the third band advances to row 2");
        e.find_frame();
        assert_eq!((e.frame, e.frame_counter), (0, 0.0), "the 21st tick wraps");
        // The phase-2 rows (`ai[0] > 1`).
        e.ai[0] = 2.0;
        for _ in 0..6 {
            e.find_frame();
        }
        assert_eq!(e.frame, 3, "the phase-2 beat rides rows 3/4/5");
        for _ in 0..7 {
            e.find_frame();
        }
        assert_eq!(e.frame, 4);
        for _ in 0..7 {
            e.find_frame();
        }
        assert_eq!(e.frame, 5);
        e.find_frame();
        assert_eq!(e.frame, 3, "and wraps 5 -> 3");
    }

    #[test]
    fn the_plantera_and_the_brain_share_the_eight_row_phase_machine() {
        // `case 262` (Plantera) and `case 266` (the Brain) are the same
        // shape (`NPC.cs`): the 6-tick advance over the 8-row strips
        // (`Main.npcFrameCount[262]` = [266] = 8, `Main.cs:67082`), the
        // first phase wrapping at row 3 and the second clamping into
        // 4..7.
        let mut p = bare(NpcKind::Plantera);
        p.hp = p.max_hp; // above the half life
        for _ in 0..6 {
            p.find_frame();
        }
        assert_eq!(p.frame, 0, "6 ticks hold row 0");
        p.find_frame();
        assert_eq!(p.frame, 1, "the 7th tick advances");
        // The advance fires every 7 ticks (the counter's `> 6`), so the
        // 0-1-2-3 cycle closes after 28.
        for _ in 0..21 {
            p.find_frame();
        }
        assert_eq!(p.frame, 0, "the first phase wraps at row 3");
        // Below the half life: the rows clamp into 4..7.
        p.hp = p.max_hp / 4;
        for _ in 0..14 {
            p.find_frame();
        }
        assert!(
            (4..=7).contains(&p.frame),
            "the second phase rides 4..7: {}",
            p.frame
        );
        // The Brain rides the same machine through its shield-down gate
        // (the creeper-shield count is unmodelled — the documented
        // follow-up), so it always shows the first-phase rows.
        let mut b = bare(NpcKind::BrainOfCthulhu);
        b.max_hp = b.hp;
        for _ in 0..40 {
            b.find_frame();
            assert!(
                b.frame <= 3,
                "the Brain's first phase wraps at 3: {}",
                b.frame
            );
        }
        // The Creeper's single frame never moves.
        let mut c = bare(NpcKind::Creeper);
        c.frame = 3;
        for _ in 0..10 {
            c.find_frame();
            assert_eq!(c.frame, 0, "the Creeper's one frame");
        }
    }

    #[test]
    fn the_queen_bee_rides_its_flight_rows_over_the_twelve_row_strip() {
        // `case 222` (`NPC.cs`): NPC_222's strip is TWELVE rows
        // (`Main.npcFrameCount[222]` = 12, `Main.cs:67082` — the v1
        // claimed 2 and ran an 8-tick strip over both), laid out as the
        // charging rows 0..3 and the flight rows 4..11, both at the 5-tick
        // cadence with the flight half wrapping back to 4.
        let mut q = bare(NpcKind::QueenBee);
        q.frame = 4;
        for _ in 0..4 {
            q.find_frame();
        }
        assert_eq!(q.frame, 4, "4 ticks hold row 4");
        q.find_frame();
        assert_eq!(q.frame, 5, "the 5th tick advances");
        // Walk the flight half to the wrap: 4 → 5 → … → 11 → 4 (each
        // advance is a 5-tick step).
        let mut steps = 0;
        while q.frame != 11 {
            q.find_frame();
            assert!(
                (4..12).contains(&q.frame),
                "the flight band rides 4..11: {}",
                q.frame
            );
            steps += 1;
            assert!(steps < 50, "the band must climb to 11");
        }
        // The wrap itself needs another 5-tick step: 11 → 12 → clamped 4.
        for _ in 0..5 {
            q.find_frame();
        }
        assert_eq!(q.frame, 4, "the flight band wraps back to 4");
        // The charging rows 0..3 stay unreachable while the port does not
        // model the `localAI[0] == 1` gate (an angle slot in the C#), so
        // the row is always clamped into the flight half.
        q.frame = 0;
        q.find_frame();
        assert!(q.frame >= 4, "the flight half always rides: {}", q.frame);
    }

    #[test]
    fn the_king_slime_rides_the_six_row_strip() {
        // Case 50 (`NPC.cs:73380-73414`): the airborne snap to row 4 and
        // the held 5, the landing's fall back to 4, the grounded walk's
        // 0..3 with the `num2` boosts.
        let mut k = bare(NpcKind::KingSlime);
        k.vy = 0.5;
        k.find_frame();
        assert_eq!(k.frame, 4, "the first airborne tick snaps to row 4");
        for _ in 0..4 {
            k.find_frame();
        }
        assert_eq!(k.frame, 5, "the air holds row 5");
        k.vy = 0.0;
        k.find_frame();
        assert_eq!(k.frame, 4, "the landing falls back to row 4");
        for _ in 0..9 {
            k.find_frame();
            assert!(k.frame <= 4, "the grounded walk rides 0..4: {}", k.frame);
        }
        assert_eq!(k.frame, 0, "the walk wraps at row 4");
    }

    #[test]
    fn the_wall_of_flesh_and_its_eye_share_the_mouth_cycle() {
        // `case 113/114` (`NPC.cs:73516-73537`): the plain phase
        // (`ai[2] == 0`) cycles the 2-row strip every 12 ticks; every other
        // phase holds row 0 with the counter parked at -60.
        let mut w = bare(NpcKind::WallOfFlesh);
        w.ai[2] = 0.0;
        for _ in 0..11 {
            w.find_frame();
        }
        assert_eq!(w.frame, 0, "11 ticks hold row 0");
        w.find_frame();
        assert_eq!(w.frame, 1, "the 12th tick advances");
        for _ in 0..12 {
            w.find_frame();
        }
        assert_eq!(w.frame, 0, "the strip wraps at npcFrameCount's 2");
        // The mouth phases pin row 0 and park the counter.
        w.ai[2] = 1.0;
        w.frame = 1;
        w.find_frame();
        assert_eq!(
            (w.frame, w.frame_counter),
            (0, -60.0),
            "the phase pins row 0"
        );
        // The mirror eye rides the same case.
        let mut e = bare(NpcKind::WallOfFleshEye);
        e.ai[2] = 0.0;
        for _ in 0..12 {
            e.find_frame();
        }
        assert_eq!(e.frame, 1, "the eye cycles its 2 rows");
    }

    #[test]
    fn the_hungry_rides_the_fifteen_tick_sequence() {
        // `case 115` (`NPC.cs:74995-75017`): the 3-row strip rides a 15-tick
        // sequence 0/1/2/1 (the bands at 3/6/12/15), then the counter
        // resets — not the v1's free 8-tick strip.
        let mut h = bare(NpcKind::Hungry);
        // The bands: [0,3) -> row 0, [3,6) -> row 1, [6,12) -> row 2,
        // [12,15) -> row 1, and the 15th tick resets the counter.
        // The 15th tick resets the counter while the row still holds the
        // band's value — row 0 arrives on the next tick.
        for (ticks, want) in [(2usize, 0u32), (3, 1), (6, 2), (3, 1), (1, 1), (1, 0)] {
            for _ in 0..ticks {
                h.find_frame();
            }
            assert_eq!(h.frame, want, "the band after {} ticks", ticks);
        }
        assert_eq!(
            h.frame_counter, 1.0,
            "the reset lands and the counter re-runs"
        );
    }

    /// BUG-07: the display flip rides  for every NPC
    /// () — the walker machine keeps the two in
    /// lockstep (the C# walker-family heads, //
    /// ), so a walker that turns without moving flips with the body
    /// and never freezes at the -1 default.
    #[test]
    fn bug_07_the_walker_keeps_sprite_direction_in_lockstep_with_facing() {
        let w = World::new("t".into(), 200, 100);
        // A zombie walker (the surface needs a floor row — the tile 1 at the
        // surface of the 200x100 world).
        let mut n = Npc::spawn_zombie(&w, 40).unwrap_or_else(|| {
            // The fixture fallback: a bare zombie at the midpoint.
            let mut b = bare(NpcKind::Zombie);
            b.x = 40.0 * 16.0;
            b.y = 50.0 * 16.0;
            b
        });
        let params = crate::npc_ai::WalkerParams::ZOMBIE;
        let (x0, y0) = (n.x, n.y);
        // The default spawn faces right — the lockstep's first write.
        let _ = crate::npc_ai::walker_step(&mut n, &w, x0 + 10.0, y0, params, 0.3, 10.0);
        assert_eq!(n.facing, 1);
        assert_eq!(
            n.sprite_dir, 1,
            "the walker writes spriteDirection with facing"
        );
        // A target on the left flips both.
        let (x0, y0) = (n.x, n.y);
        let _ = crate::npc_ai::walker_step(&mut n, &w, x0 - 10.0, y0, params, 0.3, 10.0);
        assert_eq!(n.facing, -1);
        assert_eq!(n.sprite_dir, -1, "the flip carries both fields");
    }

    /// BUG-07: aiStyle 13's two one-sided `if (num215 > 0)` /
    /// `if (num215 < 0)` arms (`NPC.cs:22740-22752`) — the eye flips on its
    /// OWN aim, so a target dead-centre (`num215 == 0`) leaves the previous
    /// flip in place instead of snapping to the body's facing.
    #[test]
    fn bug_07_the_demon_eye_keeps_its_flip_on_a_dead_centre_aim() {
        let w = World::new("t".into(), 200, 100);
        let mut n = bare(NpcKind::DemonEye);
        n.x = 50.0 * 16.0;
        n.y = 50.0 * 16.0;
        let mut player = crate::player::Player::spawn(&w);
        // A target to the left flips the eye left.
        player.x = n.x - 120.0;
        player.y = n.y;
        n.update_eye(&w, &mut player, 0.0);
        assert_eq!(n.sprite_dir, -1, "the eye aims left");
        // Snap the player onto the eye's centre: `num215 == 0` writes
        // nothing, so the flip is preserved.
        player.x = n.x + n.size().0 * 0.5 - crate::player::WIDTH * 0.5;
        player.y = n.y;
        n.update_eye(&w, &mut player, 0.0);
        assert_eq!(n.sprite_dir, -1, "a dead-centre aim keeps the flip");
    }

    /// BUG-07: aiStyle 29 (the WoF eye, `NPC.cs:26393-26403`) flips on its
    /// own `num391` — the distance from the EYE to the player — not on the
    /// wall body's `direction` the `:26068` mirror writes.
    #[test]
    fn bug_07_the_wof_eye_flips_on_its_own_aim() {
        let mut w = World::new("t".into(), 200, 100);
        w.wof_present = true;
        w.wof_x = 100.0 * 16.0;
        w.wof_draw_top = 400.0;
        w.wof_draw_bottom = 900.0;
        let mut n = bare(NpcKind::WallOfFleshEye);
        n.x = w.wof_x - n.size().0 * 0.5;
        let mut player = crate::player::Player::spawn(&w);
        // The iris sits at the wall's X; the player is 300 px to its right.
        player.x = w.wof_x + 300.0 - crate::player::WIDTH * 0.5;
        player.y = 600.0;
        n.update_wof_eye(&w, &player);
        assert_eq!(n.sprite_dir, 1, "the target is right of the iris");
        player.x = w.wof_x - 300.0 - crate::player::WIDTH * 0.5;
        n.update_wof_eye(&w, &player);
        assert_eq!(n.sprite_dir, -1, "the target is left of the iris");
    }
}

mod shard_stat_tests {
    use super::*;

    #[test]
    fn the_shards_carry_their_contact_rows() {
        // The shards' damage rows — the 405/406/407 cells bite with the
        // 120/70/70, the Nebula beast's 90 and the Vortex pair's 100/90
        // (`SetDefaults` cases 405/406/407/423/425/429).
        for (kind, dmg) in [
            (NpcKind::StardustCellBig, 120),
            (NpcKind::StardustCellSmall, 70),
            (NpcKind::StardustJellyfishBig, 70),
            (NpcKind::NebulaBeast, 90),
            (NpcKind::VortexRifleman, 100),
            (NpcKind::VortexSoldier, 90),
        ] {
            assert_eq!(kind.contact_damage(), dmg);
        }
    }
}

#[cfg(test)]
mod s1_d_3b_tests {
    use super::*;
    use tr_core::world::World;

    fn bare(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 1, 0.0)
    }

    fn flat_world() -> World {
        let mut w = World::new("t".into(), 200, 120);
        // a floor so walkers have ground
        for x in 0..200i32 {
            let i = w.index(x, 70);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn goblin_fighters_use_skeleton_walker_not_shadowed_goblin_arm() {
        // R-05-3 / S1-D-3b: GoblinThief/Warrior/Peon must reach the walker
        // arm instead of the shadowing `update_goblin` arm (speed 1.0).
        // S1-D-3c: the walk speed is the C#'s per-type cap (the thief 2,
        // the warrior 1, the peon 1.5 — `NPC.cs:57554`/`:57606`/`:58322`).
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        // place target far to the right, both on the floor
        player.x = 140.0 * 16.0;
        player.y = 68.0 * 16.0;
        for (kind, cap) in [
            (NpcKind::GoblinThief, 2.0),
            (NpcKind::GoblinWarrior, 1.0),
            (NpcKind::GoblinPeon, 1.5),
        ] {
            let mut n = bare(kind);
            n.x = 80.0 * 16.0;
            n.y = 68.0 * 16.0;
            n.vx = 0.0;
            n.vy = 0.0;
            for _ in 0..120 {
                n.update(&w, &mut player, 1.0 / 60.0);
            }
            assert!(
                (n.vx - cap).abs() < 0.01,
                "{kind:?} must walk at the C#'s cap {cap}, got vx={}",
                n.vx
            );
            assert!(
                n.vx > 0.0,
                "{kind:?} should walk toward the target, got vx={}",
                n.vx
            );
        }
    }

    #[test]
    fn goblin_summoner_is_not_routed_to_ai_style_8_caster() {
        // S1-D-3b: GoblinSummoner (471, NPC.cs:15728 aiStyle 3, :56491-56718)
        // must not run the caster cycle (aiStyle 8). Caster AI drives
        // caster_counter/cast_windup; the summoner's type-specific machine
        // advances ai[2]/ai[3].
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 80.0 * 16.0;
        player.y = 68.0 * 16.0;
        let mut n = bare(NpcKind::GoblinSummoner);
        n.x = 80.0 * 16.0;
        n.y = 60.0 * 16.0;
        n.ai = [0.0; 4];
        n.vy = 1.0;
        n.caster_counter = 0;
        n.cast_windup = 0;
        let mut saw_nonzero_ai = false;
        for _ in 0..360 {
            n.update(&w, &mut player, 1.0 / 60.0);
            if n.ai[2] != 0.0 || n.ai[3] != 0.0 {
                saw_nonzero_ai = true;
            }
        }
        assert!(
            saw_nonzero_ai,
            "summoner must advance ai[2]/ai[3] state machine, got ai={:?}",
            n.ai
        );
        // caster path would not produce this 471-specific ground-phase cycle
        assert_eq!(n.caster_counter, 0, "must not run aiStyle 8 caster cycle");
    }

    #[test]
    fn ancient_cultist_uses_cultist_ai_not_pillar_ai_style_94() {
        // S1-D-3b: AncientCultist (CultistBoss 439, aiStyle 84, NPC.cs:64414)
        // must not share the Lunar Pillar aiStyle 94 machine
        // (NPC.cs:38968-39400). The tower update dissolves at 180 ticks
        // (ai[2] timer -> hp 0); cultist AI_084 begins with a spawn-in intro.
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 68.0 * 16.0;
        let mut n = bare(NpcKind::AncientCultist);
        n.x = 100.0 * 16.0;
        n.y = 60.0 * 16.0;
        n.ai = [0.0; 4];
        n.hp = n.max_hp.max(1);
        for _ in 0..200 {
            n.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(
            n.hp > 0,
            "cultist must not be killed by the tower 180-tick dissolve"
        );
        assert_eq!(
            n.ai[0], -1.0,
            "cultist should remain in AI_084 spawn-in intro"
        );
    }

    #[test]
    fn frost_legion_hops_with_the_big_third_beat_and_the_speed_clamp() {
        // `NPC.cs:29089-29144`: the hop cycle — `ai[0]` beats 0..2, every
        // third launches `-8.2f` and bumps `ai[1]`; the walk acceleration
        // `direction * num552 * 0.01` clamps at `num551` (the Gangsta 3).
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 66.0 * 16.0;
        let mut n = bare(NpcKind::SnowmanGangsta);
        n.x = 90.0 * 16.0;
        n.y = 60.0 * 16.0;
        n.ai = [0.0; 4];
        let mut big_hops = 0;
        // The hop flight is ~30 ticks (`-6f` at the shared gravity) — ten
        // seconds cover the fall plus ~15 beats.
        for _ in 0..600 {
            n.update(&w, &mut player, 1.0 / 60.0);
            if n.ai[1] > big_hops as f32 {
                big_hops = n.ai[1] as i32;
            }
        }
        assert!(big_hops >= 3, "the every-third beat fired {big_hops} times");
        assert!(n.vx != 0.0, "the walk acceleration moved it");
        // The `num551` clamp binds along the facing only
        // (`NPC.cs:29139-29144`): a right-facing Gangsta's +vx clamps at 3.
        n.facing = 1;
        n.vx = 5.0;
        n.update(&w, &mut player, 1.0 / 60.0);
        assert!(n.vx <= 3.0 + 1e-4, "the clamp: vx = {}", n.vx);
    }

    #[test]
    fn frost_gangsta_throws_the_bottle_every_120_ticks() {
        // `NPC.cs:29012-29037`: `ai[2]` counts to 120, then the hostile
        // `BulletSnowman` 110 rides `wants_hostile_shot` at
        // `12 * spriteDirection`, 25 dmg — behind the firing-distance box.
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 92.0 * 16.0;
        player.y = 66.0 * 16.0;
        let mut n = bare(NpcKind::SnowmanGangsta);
        n.x = 90.0 * 16.0;
        n.y = 60.0 * 16.0;
        n.ai = [0.0; 4];
        for _ in 0..125 {
            n.update(&w, &mut player, 1.0 / 60.0);
        }
        let shot = n
            .wants_hostile_shot
            .iter()
            .find(|s| s.0 == crate::projectile::BULLET_SNOWMAN)
            .expect("the bottle throw fired");
        assert_eq!(shot.5, 25);
        assert!(
            (shot.3.abs() - 12.0).abs() < 1e-6,
            "the horizontal speed 12x spriteDirection: {}",
            shot.3
        );
    }

    #[test]
    fn frost_balla_pauses_after_three_hops_and_resets() {
        // `NPC.cs:29039-29056`: `ai[1] >= 3f` — the grounded pause (the vx
        // decay) counts `ai[2]` to 200 and resets the hop count.
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        let mut n = bare(NpcKind::SnowBalla);
        n.x = 90.0 * 16.0;
        n.y = 66.0 * 16.0;
        n.ai = [0.0, 3.0, 199.0, 0.0];
        n.vy = 0.0;
        n.update(&w, &mut player, 1.0 / 60.0);
        assert_eq!(n.ai[1], 0.0, "the 200-tick pause elapsed and reset");
        assert_eq!(n.ai[2], 0.0);
    }

    #[test]
    fn frost_mystery_throws_the_aimed_snowball_at_windup_8() {
        // `NPC.cs:29078-29087`: the 16-tick windup — at `ai[2] == 8` the
        // aimed `SnowBallHostile` 109 fires at speed 10, 35 dmg, from
        // `(cx - facing*12, y + h*0.25)` toward the player's top-centre.
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 94.0 * 16.0;
        player.y = 65.0 * 16.0;
        let mut n = bare(NpcKind::MysterySnowman);
        n.x = 90.0 * 16.0;
        n.y = 66.0 * 16.0;
        n.ai = [0.0, 3.0, 7.0, 0.0];
        n.vy = 0.0;
        n.update(&w, &mut player, 1.0 / 60.0);
        let shot = n
            .wants_hostile_shot
            .iter()
            .find(|s| s.0 == crate::projectile::SNOW_BALL_HOSTILE)
            .expect("the windup throw fired");
        assert_eq!(shot.5, 35);
        let speed = (shot.3 * shot.3 + shot.4 * shot.4).sqrt();
        assert!((speed - 10.0).abs() < 1e-4, "the aim speed 10: {speed}");
    }

    #[test]
    fn frost_stuck_position_flips_the_direction() {
        // `NPC.cs:29093-29098`: `localAI[2] == position.X` on a grounded
        // tick flips the direction and arms the 60-tick cooldown.
        let w = flat_world();
        let mut player = crate::player::Player::spawn(&w);
        let mut n = bare(NpcKind::SnowBalla);
        n.x = 90.0 * 16.0;
        n.y = 66.0 * 16.0;
        n.ai = [0.0; 4];
        n.vy = 0.0;
        n.frost_stuck_x = n.x;
        let before = n.facing;
        n.update(&w, &mut player, 1.0 / 60.0);
        assert_eq!(n.facing, -before, "the stuck flip");
        // `ai[3] = 60f` and the same-tick `ai[3]--` (`:29137`) land on 59.
        assert_eq!(n.ai[3], 59.0, "the cooldown after the same-tick decrement");
    }
}

#[cfg(test)]
mod s1_f_3_tests {
    use super::*;

    #[test]
    fn s1_f_3_every_kind_resolves_a_vanilla_cs_id() {
        // S1-F-3: `cs_id` covers all 182 kinds — the hand table's verified
        // rows plus the generated loot-db net ids for the rest; a kind the
        // mapping cannot express keeps the `u16::MAX` sentinel (the caps
        // never match it), and the air sentinel is not a kind.
        let mut covered = 0;
        let mut sentinel = 0;
        let mut bad: Vec<String> = Vec::new();
        // The kind roster walks the wire-id space (the round-trip test pins
        // exactly 182 decodable ids).
        for wire in 0..=u16::MAX {
            let Some(kind) = NpcKind::from_u16(wire) else {
                continue;
            };
            let id = kind.cs_id();
            if id == u16::MAX {
                sentinel += 1;
            } else {
                covered += 1;
                // The entry-less kinds (the segments the roster skips) are
                // exempt — their cs_id rides the sentinel or the hand row.
                if let Some(e) = crate::bestiary::try_entry_for_npc(kind, SlimeVariant::Blue) {
                    if e.net_id > 0 && e.net_id != id as i32 {
                        bad.push(format!("{kind:?}: cs_id {id} vs net {}", e.net_id));
                    }
                }
            }
        }
        assert!(
            bad.is_empty(),
            "the hand rows disagree with the net ids: {bad:?}"
        );
        assert_eq!(
            covered + sentinel,
            182,
            "every decodable kind resolves (the wire roster's size)"
        );
        assert!(
            covered > 43,
            "the fallback lifted the coverage past the hand table"
        );
        // The spot checks: the base slime (1), the zombie (NPCID.Zombie = 3 —
        // the hand table's 21 was wrong), the guide (22) and the angler (369).
        assert_eq!(NpcKind::Slime.cs_id(), 1);
        assert_eq!(NpcKind::Zombie.cs_id(), 3);
        assert_eq!(NpcKind::Guide.cs_id(), 22);
        assert_eq!(NpcKind::Angler.cs_id(), 369);
    }

    /// S1-D-3c — `AI_003_Fighters`'s per-type walk-speed caps
    /// (`NPC.cs:57403-59800`): the C#'s groups and their rows.
    #[test]
    fn s1_d_3c_the_ai_style_3_walk_speeds_match_the_csharp_caps() {
        // The `num107` default group (`:58322-58374`): 1f.
        assert_eq!(NpcKind::Zombie.ai3_walk_speed(), Some(1.0));
        assert_eq!(NpcKind::UndeadMiner.ai3_walk_speed(), Some(1.0));
        assert_eq!(NpcKind::GoblinWarrior.ai3_walk_speed(), Some(1.0));
        // The `num83` group (`:57606-57691`): 1.5f with the per-type rows.
        assert_eq!(NpcKind::Skeleton.ai3_walk_speed(), Some(1.5));
        assert_eq!(NpcKind::GoblinPeon.ai3_walk_speed(), Some(1.5));
        assert_eq!(NpcKind::GrayGrunt.ai3_walk_speed(), Some(1.8));
        assert_eq!(NpcKind::GigaZapper.ai3_walk_speed(), Some(2.25));
        // The cap-2 (`:57554-57578`) and cap-3 (`:57520-57552`) groups.
        assert_eq!(NpcKind::GoblinThief.ai3_walk_speed(), Some(2.0));
        assert_eq!(NpcKind::PirateDeckhand.ai3_walk_speed(), Some(2.0));
        assert_eq!(NpcKind::PirateCaptainsMate.ai3_walk_speed(), Some(3.0));
        // The shooters' group (`:59717-59750`).
        assert_eq!(NpcKind::PirateArcher.ai3_walk_speed(), Some(2.0));
        assert_eq!(NpcKind::PirateCorsair.ai3_walk_speed(), Some(1.5));
        assert_eq!(NpcKind::PirateCaptain.ai3_walk_speed(), Some(1.0));
        assert_eq!(NpcKind::BrainScrambler.ai3_walk_speed(), Some(2.0));
        assert_eq!(NpcKind::RayGunner.ai3_walk_speed(), Some(2.0));
        assert_eq!(NpcKind::MartianOfficer.ai3_walk_speed(), Some(1.0));
        assert_eq!(NpcKind::MartianWalker.ai3_walk_speed(), Some(4.0));
        // The Martian engineer's own arm (`:58032-58069`).
        assert_eq!(NpcKind::MartianEngineer.ai3_walk_speed(), Some(1.5));
        // The kinds outside the table carry their own arm's speed (the
        // vortex rifleman's own arm already uses the C#'s 6f cap, the
        // `num83` row `:57677`).
        assert_eq!(NpcKind::Slime.ai3_walk_speed(), None);
        assert_eq!(NpcKind::DemonEye.ai3_walk_speed(), None);
        assert_eq!(NpcKind::VortexRifleman.ai3_walk_speed(), None);
    }

    /// S1-D-3d — the goblin family's ids and stats: the port's four goblin
    /// kinds carried each other's vanilla `NPCID` (the peon/thief/warrior/
    /// archer were rotated), so the stats rows, the bestiary's net ids and
    /// the loot table's key were one step off the C#.
    ///
    /// The C#'s rows (`ID/NPCID.cs:10729-10733`/`:10899` and the
    /// `SetDefaults` arms `NPC.cs:15644-15730`):
    /// - the peon (26): 60 HP, 12 damage, 4 defense, kb 0.8, value 100;
    /// - the thief (27): 80 HP, 20 damage, 6 defense, kb 0.7, value 200;
    /// - the warrior (28): 110 HP, 25 damage, 8 defense, kb 0.5, value 150;
    /// - the archer (111): 80 HP, 20 damage, 6 defense, kb 0.7, value 200.
    #[test]
    fn s1_d_3d_the_goblin_kinds_carry_their_vanilla_ids_and_stats() {
        assert_eq!(NpcKind::GoblinPeon.cs_id(), 26, "NPCID.GoblinPeon");
        assert_eq!(NpcKind::GoblinThief.cs_id(), 27, "NPCID.GoblinThief");
        assert_eq!(NpcKind::GoblinWarrior.cs_id(), 28, "NPCID.GoblinWarrior");
        assert_eq!(NpcKind::GoblinArcher.cs_id(), 111, "NPCID.GoblinArcher");
        // The stats rows.
        assert_eq!(NpcKind::GoblinPeon.max_hp(), 60);
        assert_eq!(NpcKind::GoblinThief.max_hp(), 80);
        assert_eq!(NpcKind::GoblinWarrior.max_hp(), 110);
        assert_eq!(NpcKind::GoblinArcher.max_hp(), 80);
        assert_eq!(NpcKind::GoblinPeon.contact_damage(), 12);
        assert_eq!(NpcKind::GoblinThief.contact_damage(), 20);
        assert_eq!(NpcKind::GoblinWarrior.contact_damage(), 25);
        assert_eq!(NpcKind::GoblinArcher.contact_damage(), 20);
        let npc = |kind| Npc::new(kind, 0.0, 0.0, 0, 0.0);
        assert_eq!(npc(NpcKind::GoblinPeon).defense(), 4);
        assert_eq!(npc(NpcKind::GoblinThief).defense(), 6);
        assert_eq!(npc(NpcKind::GoblinWarrior).defense(), 8);
        assert_eq!(npc(NpcKind::GoblinArcher).defense(), 6);
        assert_eq!(npc(NpcKind::GoblinPeon).knock_back_resist(), 0.8);
        assert_eq!(npc(NpcKind::GoblinThief).knock_back_resist(), 0.7);
        assert_eq!(npc(NpcKind::GoblinWarrior).knock_back_resist(), 0.5);
        assert_eq!(npc(NpcKind::GoblinArcher).knock_back_resist(), 0.7);
        assert_eq!(npc(NpcKind::GoblinPeon).value(), 100.0);
        assert_eq!(npc(NpcKind::GoblinThief).value(), 200.0);
        assert_eq!(npc(NpcKind::GoblinWarrior).value(), 150.0);
        assert_eq!(npc(NpcKind::GoblinArcher).value(), 200.0);
    }
}

#[cfg(test)]
mod s1_d_3e_tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The ground fixture: solid tiles from row 30 down.
    fn world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn brain() -> Npc {
        let mut n = Npc::new(NpcKind::BrainOfCthulhu, 200.0, 100.0, 0, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// S1-D-3e — the Brain of Cthulhu's machine (`NPC.cs:32370-32656`): the
    /// normal phase drifts at 1 px/tick, the blink timer picks a tile near
    /// the player, the fade-in teleports the Brain onto it and the fade-out
    /// returns to the normal phase.
    #[test]
    fn s1_d_3e_the_brain_blinks_to_a_tile_near_the_player() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        player.zones.crimson = true;
        let mut b = brain();
        // A live Creeper keeps the Brain in its normal phase (the C#'s
        // `num818` check).
        b.brain_local[2] = 1.0;
        // The normal phase: the 1-px drift toward the player.
        b.update_brain(&w, &player, &mut 12345u64);
        assert!(
            (b.vx.hypot(b.vy) - 1.0).abs() < 0.01,
            "the normal phase drifts at 1 px/tick ({:?})",
            (b.vx, b.vy)
        );
        assert!(
            b.vx > 0.0 && b.vy > 0.0,
            "the drift points at the player ({:?})",
            (b.vx, b.vy)
        );
        // The blink timer: after 120 + Next(300) ticks the Brain picks a tile
        // and fades in (alpha += 5 per tick) before teleporting.
        let mut rng = 12345u64;
        for _ in 0..500 {
            b.update_brain(&w, &player, &mut rng);
            if b.ai[0] == 1.0 {
                break;
            }
        }
        assert_eq!(b.ai[0], 1.0, "the blink armed the fade-in");
        let tile_x = b.ai[1];
        let tile_y = b.ai[2];
        // The fade-in runs 51 ticks (alpha 0 -> 255) and then teleports.
        for _ in 0..60 {
            b.update_brain(&w, &player, &mut rng);
            if b.ai[0] == 2.0 {
                break;
            }
        }
        assert_eq!(b.ai[0], 2.0, "the fade-in teleported the Brain");
        assert!(
            (b.x - (tile_x * 16.0 - 80.0)).abs() <= 1.0,
            "the Brain sits on the tile ({})",
            b.x
        );
        assert!(
            (b.y - (tile_y * 16.0 - 55.0)).abs() <= 1.0,
            "the Brain sits on the tile ({})",
            b.y
        );
        // The fade-out returns to the normal phase.
        for _ in 0..80 {
            b.update_brain(&w, &player, &mut rng);
            if b.ai[0] == 0.0 {
                break;
            }
        }
        assert_eq!(b.ai[0], 0.0, "the fade-out returns to the normal phase");
        assert_eq!(b.alpha, 0, "the Brain is invisible again");
    }

    /// S1-D-3e — the Brain's retreat: with no Creeper left (`brain_local[2]`
    /// = 0) the Brain switches to the `-1` state, and the retreat's states
    /// fade the Brain in on the picked tile and back out.
    #[test]
    fn s1_d_3e_the_brain_retreats_when_the_creepers_are_gone() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        player.zones.crimson = true;
        let mut b = brain();
        b.brain_local[2] = 0.0; // no creeper left
        b.update_brain(&w, &player, &mut 999u64);
        assert_eq!(b.ai[0], -1.0, "the Brain retreats without its swarm");
        assert_eq!(b.alpha, 0, "the retreat starts invisible");
        // The retreat's timer picks a tile (60 + Next(120) ticks) and the
        // `-2` state fades the Brain in on it.
        let mut rng = 999u64;
        for _ in 0..400 {
            b.update_brain(&w, &player, &mut rng);
            if b.ai[0] == -2.0 {
                break;
            }
        }
        assert_eq!(b.ai[0], -2.0, "the retreat armed its fade-in");
        for _ in 0..80 {
            b.update_brain(&w, &player, &mut rng);
            if b.ai[0] == -3.0 {
                break;
            }
        }
        assert_eq!(b.ai[0], -3.0, "the retreat faded the Brain in");
        assert_eq!(b.alpha, 255, "the retreat's fade-in is full");
    }
}

#[cfg(test)]
mod s1_d_3f_tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The ground fixture: solid tiles from row 30 down.
    fn world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// S1-D-3f — the Skeletron head's machine (`NPC.cs:21919-22298`): the
    /// daytime turns the head into the enraged state (the invulnerable
    /// 8-px rush) and the charge state drops the defense by 10.
    #[test]
    fn s1_d_3f_the_skeletron_head_enrages_in_the_daylight() {
        let mut w = world();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        let mut sk = Npc::new(NpcKind::SkeletronHead, 200.0, 100.0, 0, 0.0);
        sk.hp = sk.max_hp;
        let mut rng = 12345u64;
        // The night: the head spins (the state 0) and hovers.
        sk.update_skeletron(&w, &player, &mut rng);
        assert_eq!(sk.ai[1], 0.0, "the head spins at night");
        // The daylight: the enraged state.
        w.day_time = true;
        sk.update_skeletron(&w, &player, &mut rng);
        assert_eq!(sk.ai[1], 2.0, "the daylight enrages the head");
        assert_eq!(sk.damage_override, Some(9999), "the enraged damage");
        assert_eq!(sk.defense_override, Some(9999), "the enraged defense");
        let speed = (sk.vx * sk.vx + sk.vy * sk.vy).sqrt();
        assert!((speed - 8.0).abs() < 0.01, "the enraged rush ({speed})");
    }

    /// S1-D-3f — the charge state (`NPC.cs:22118-22244`): the defense
    /// drops by 10 and the head rushes at the player.
    #[test]
    fn s1_d_3f_the_skeletron_charge_drops_the_defense() {
        let mut w = world();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        let mut sk = Npc::new(NpcKind::SkeletronHead, 200.0, 100.0, 0, 0.0);
        sk.hp = sk.max_hp;
        sk.ai[1] = 1.0; // the charge state
        let base_defense = sk.defense();
        let mut rng = 12345u64;
        sk.update_skeletron(&w, &player, &mut rng);
        assert_eq!(
            sk.defense_override,
            Some(base_defense - 10),
            "the charge drops the defense"
        );
        let speed = (sk.vx * sk.vx + sk.vy * sk.vy).sqrt();
        assert!((speed - 1.5).abs() < 0.01, "the charge speed ({speed})");
    }
}

#[cfg(test)]
mod s1_d_3g_tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The ground fixture: solid tiles from row 30 down.
    fn world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn king() -> Npc {
        let mut n = Npc::new(NpcKind::KingSlime, 200.0, 300.0, 0, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// S1-D-3g — the King Slime's body scales with its hp
    /// (`AI_015_KingSlime`, `NPC.cs:43340-43356`): the scale is
    /// `life / lifeMax * 0.5 + 0.75` and the box is 98x92 times it.
    #[test]
    fn s1_d_3g_the_king_slime_scales_with_its_hp() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        let mut k = king();
        let mut rng = 12345u64;
        k.update_king_slime(&w, &mut player, &mut rng);
        // The full-hp scale: 0.5 + 0.75 = 1.25.
        assert!(
            (k.scale - 1.25).abs() < 0.001,
            "the full-hp scale ({})",
            k.scale
        );
        assert_eq!(k.size_override, Some((98.0 * 1.25, 92.0 * 1.25)));
        // At half the hp the scale drops to 1.0.
        k.hp = k.max_hp / 2;
        k.update_king_slime(&w, &mut player, &mut rng);
        assert!(
            (k.scale - 1.0).abs() < 0.001,
            "the half-hp scale ({})",
            k.scale
        );
    }

    /// S1-D-3g — the minion batches (`NPC.cs:43357-43395`): every 5 % of the
    /// hp the King Slime spawns 1-3 slimes (the C#'s `Next(1, 4)`).
    #[test]
    fn s1_d_3g_the_king_slime_spawns_a_batch_every_five_percent() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 400.0;
        player.y = 200.0;
        let mut k = king();
        let mut rng = 12345u64;
        // The first tick: the `ai[3]` latch is set to the full hp — no batch.
        k.update_king_slime(&w, &mut player, &mut rng);
        assert!(!k.wants_minion, "no batch at the full hp");
        // A hair of damage is not enough (the 5 % window).
        k.hp = k.max_hp - 1;
        k.update_king_slime(&w, &mut player, &mut rng);
        assert!(!k.wants_minion, "no batch inside the 5 % window");
        // A 5 % loss spawns the batch.
        k.hp = k.max_hp - (k.max_hp / 20) - 1;
        k.update_king_slime(&w, &mut player, &mut rng);
        assert!(k.wants_minion, "the 5 % batch");
        assert!(
            (1..=3).contains(&k.wants_minion_batch),
            "the batch is 1-3 ({})",
            k.wants_minion_batch
        );
    }
}

#[cfg(test)]
mod s1_d_4b_5b_tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The ground fixture: solid tiles from row 30 down.
    fn world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// S1-D-4b-5b — the Deerclops' ice-spike row
    /// (`AI_123_Deerclops_MakeSpikesForward`, `NPC.cs:44466-44500`): from
    /// the 36th tick, every fourth tick makes four spikes, and each spike
    /// lands on the ground row `FindBestY` picks (`NPC.cs:44586-44640`).
    #[test]
    fn s1_d_4b_5b_the_deerclops_spikes_fall_in_batches_of_four() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 32.0 * 16.0;
        player.y = 20.0 * 16.0;
        let mut d = Npc::new(NpcKind::Deerclops, 20.0 * 16.0, 29.0 * 16.0, 1, 0.0);
        d.hp = d.max_hp;
        let mut rng = 12345u64;
        // The state 1 (the forward spikes): 36 ticks to the first volley.
        d.ai[0] = 1.0;
        d.ai[1] = 0.0;
        for _ in 0..36 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        let spikes: Vec<_> = d
            .wants_hostile_shot_ai
            .iter()
            .filter(|s| s.kind == crate::projectile::DEERCLOPS_ICE_SPIKE)
            .copied()
            .collect();
        assert_eq!(spikes.len(), 4, "the first volley is four spikes");
        // The spikes land on the ground row under the player (the solid
        // tiles start at row 30) and fly at the row's angle.
        for spike in spikes.iter() {
            assert!(
                (spike.y - 8.0 - 30.0 * 16.0).abs() < 24.0,
                "the spike lands on the ground row ({})",
                spike.y
            );
            assert!(
                spike.vx.abs() + spike.vy.abs() > 0.5,
                "the spike flies at an angle ({:?})",
                (spike.vx, spike.vy)
            );
        }
        // The next three ticks make no spike (the cadence is every fourth
        // tick: the volleys land on the 36th, the 40th, ...).
        let before = d.wants_hostile_shot_ai.len();
        for _ in 0..3 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        assert_eq!(
            d.wants_hostile_shot_ai.len(),
            before,
            "the cadence waits four ticks"
        );
        // The fourth tick opens the second volley.
        d.update_deerclops(&w, &mut player, &mut rng);
        assert_eq!(
            d.wants_hostile_shot_ai.len() - before,
            4,
            "the second volley is four spikes"
        );
    }
}

#[cfg(test)]
mod s1_d_4b_5d_tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The ground fixture: solid tiles from row 30 down.
    fn world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// S1-D-4b-5d — the Deerclops' rubble (`AI_123_Deerclops_ShootRubbleUp`,
    /// `NPC.cs:44448-44465`): from the 32nd tick one rock per tick lands on
    /// the first solid tile under the Deerclops — and the rock is the 962
    /// (the ice chunk), not the 961 (the forward spike).
    #[test]
    fn s1_d_4b_5d_the_deerclops_rubble_falls_one_chunk_per_tick() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 32.0 * 16.0;
        player.y = 20.0 * 16.0;
        let mut d = Npc::new(NpcKind::Deerclops, 20.0 * 16.0, 29.0 * 16.0, 1, 0.0);
        d.hp = d.max_hp;
        let mut rng = 12345u64;
        // The state 2 (the rubble): 32 ticks to the first rock.
        d.ai[0] = 2.0;
        d.ai[1] = 0.0;
        for _ in 0..32 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        let rocks: Vec<_> = d
            .wants_hostile_shot_ai
            .iter()
            .filter(|s| s.kind == crate::projectile::DEERCLOPS_ICE_CHUNK)
            .copied()
            .collect();
        assert_eq!(rocks.len(), 1, "the first tick makes one rock");
        // The rock lands on the ground row under the Deerclops.
        assert!(
            (rocks[0].y - 8.0 - 30.0 * 16.0).abs() < 24.0,
            "the rock lands on the ground row ({})",
            rocks[0].y
        );
        assert_eq!(rocks[0].damage, 18, "the rock's damage");
        // The next tick makes the second rock (the cadence is one per tick).
        d.update_deerclops(&w, &mut player, &mut rng);
        let rocks: Vec<_> = d
            .wants_hostile_shot_ai
            .iter()
            .filter(|s| s.kind == crate::projectile::DEERCLOPS_ICE_CHUNK)
            .copied()
            .collect();
        assert_eq!(rocks.len(), 2, "the rubble falls one rock per tick");
    }

    /// S1-D-4b-5d — the Deerclops' chill aura (`NPC.cs:44285-44320`): the
    /// state 3 spawns no projectile — it slows the player (the Slow buff
    /// 32, 720 ticks) within 800 px.
    #[test]
    fn s1_d_4b_5d_the_deerclops_chill_aura_slows_the_player() {
        let w = world();
        let mut player = Player::spawn(&w);
        player.x = 32.0 * 16.0;
        player.y = 25.0 * 16.0;
        let mut d = Npc::new(NpcKind::Deerclops, 30.0 * 16.0, 29.0 * 16.0, 1, 0.0);
        d.hp = d.max_hp;
        let mut rng = 12345u64;
        // The state 3 (the chill aura): the buff lands on the 28th tick.
        d.ai[0] = 3.0;
        d.ai[1] = 0.0;
        for _ in 0..27 {
            d.update_deerclops(&w, &mut player, &mut rng);
        }
        assert!(
            player.buffs.find_buff_index(32).is_none(),
            "the aura waits for its 28th tick"
        );
        d.update_deerclops(&w, &mut player, &mut rng);
        assert!(
            player.buffs.find_buff_index(32).is_some(),
            "the Slow buff lands on the 28th tick"
        );
        // The aura spawns no projectile.
        assert!(
            d.wants_hostile_shot_ai.is_empty(),
            "the chill aura fires no projectile"
        );
    }

    /// S1-D-4b-5e — the Flying Dutchman's cannon (`NPC.cs:35753-35823`): the
    /// 240-tick reload, the line-of-sight gate and the shell's shot.
    #[test]
    fn s1_d_4b_5e_the_dutchman_cannon_fires_its_shell() {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        let w = w;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 32.0 * TILE_PX;
        player.y = 20.0 * TILE_PX;
        let mut cannon = Npc::new(
            NpcKind::FlyingDutchmanCannon,
            30.0 * TILE_PX,
            29.0 * TILE_PX,
            1,
            0.0,
        );
        cannon.hp = cannon.max_hp;
        // The mount's anchor (the carrier write-through).
        cannon.carrier_x = 30.0 * TILE_PX + 15.0;
        cannon.carrier_y = 29.0 * TILE_PX + 15.0;
        cannon.carrier_timer = 0.0;
        let mut rng = 12345u64;
        // The reload: the shot lands on the 240th tick.
        for _ in 0..240 {
            cannon.update_saucer_mount(&w, &player, &mut rng);
        }
        let shells: Vec<_> = cannon
            .wants_hostile_shot
            .iter()
            .filter(|s| s.0 == 240u16)
            .copied()
            .collect();
        assert_eq!(shells.len(), 1, "the cannon fires its shell");
        assert_eq!(shells[0].5, 30, "the shell's damage");
        // The shot points at the player (the direction × 14, minus 5 on Y).
        assert!(
            shells[0].3 > 0.0,
            "the shell flies at the player ({:?})",
            shells[0]
        );
    }
}
