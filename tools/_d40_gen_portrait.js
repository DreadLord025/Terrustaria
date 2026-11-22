// W4-d40 sheets phase: generate the upgraded portrait_data block. The 22
// packed own-entry kinds get the sheet-row shape (entry_sheet height over
// the frame count, fallback SetDefaults box); the deferred + Hide-fold
// kinds keep the SetDefaults box rest. The 4 mixed arms from the data wave
// are split (CultistDragonHead from Body1/2, Nailhead from Psycho,
// CrimsonBunny from CrimsonGoldfish, EnchantedNightcrawler from Grubby/Sluggy/Buggy).
'use strict';
const fs = require('fs');
const path = require('path');
const w = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_wiring.json'), 'utf8'));
const boxes = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_boxes.json'), 'utf8'));
const P = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_payloads.json'), 'utf8'));
const up = {};
for (const p of P.portraits) up[p.name] = p.arm;

// The full 56-kind portrait block, in id order, with the upgrades applied.
// Grouped box-rest arms stay grouped where ALL members are box-rest.
const lines = [];
lines.push('        // W4-d40 sheets phase: the 421-490 band\'s portraits. The 22');
lines.push('        // PACKED own-entry kinds ride the sheet\'s own row height over');
lines.push('        // `Main.npcFrameCount[type]` (the d35/c4 swimmers\' shape); the');
lines.push('        // fallback rests the SetDefaults box. The 20 deferred kinds and');
lines.push('        // the 14 Hide-fold kinds keep the SetDefaults box rest (their');
lines.push('        // sheets are not packed / their entries fold onto the head).');
// ArmedZombie 430-436: Hide-fold, box rest (grouped)
lines.push('        (NpcKind::ArmedZombie, _)');
lines.push('        | (NpcKind::ArmedZombieEskimo, _)');
lines.push('        | (NpcKind::ArmedZombiePincussion, _)');
lines.push('        | (NpcKind::ArmedZombieSlimed, _)');
lines.push('        | (NpcKind::ArmedZombieSwamp, _)');
lines.push('        | (NpcKind::ArmedZombieTwiggy, _)');
lines.push('        | (NpcKind::ArmedZombieCenx, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 437 CultistTablet: packed own -> upgrade
lines.push(up.CultistTablet);
// 438 CultistDevote: packed own -> upgrade
lines.push(up.CultistDevote);
// 440 CultistBossClone: deferred own -> box rest
lines.push('        (NpcKind::CultistBossClone, _) => (sheet_i, 50.0, 24.0, 50.0, white),');
// 442-448 gold critters: packed own -> upgrade (each separate, different dims)
for (const n of ['GoldBird', 'GoldBunny', 'GoldButterfly', 'GoldFrog', 'GoldGrasshopper', 'GoldMouse', 'GoldWorm']) lines.push(up[n]);
// 449-452 BoneThrowingSkeleton: Hide-fold/no-entry, box rest (grouped)
lines.push('        (NpcKind::BoneThrowingSkeleton, _)');
lines.push('        | (NpcKind::BoneThrowingSkeleton2, _)');
lines.push('        | (NpcKind::BoneThrowingSkeleton3, _)');
lines.push('        | (NpcKind::BoneThrowingSkeleton4, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 453 SkeletonMerchant: deferred own -> box rest
lines.push('        (NpcKind::SkeletonMerchant, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 454 CultistDragonHead: packed own -> upgrade (custom texture NPC_454 draws,
// the sheet sizes the layout frame — the d35 DiggerHead precedent)
lines.push(up.CultistDragonHead);
// 455-459 dragon bodies: Hide-fold, box rest (grouped)
lines.push('        (NpcKind::CultistDragonBody1, _)');
lines.push('        | (NpcKind::CultistDragonBody2, _)');
lines.push('        | (NpcKind::CultistDragonBody3, _)');
lines.push('        | (NpcKind::CultistDragonBody4, _)');
lines.push('        | (NpcKind::CultistDragonTail, _) => (sheet_i, 32.0, 32.0, 32.0, white),');
// 460 Butcher: deferred own -> box rest
lines.push('        (NpcKind::Butcher, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 461 CreatureFromTheDeep: deferred own -> box rest
lines.push('        (NpcKind::CreatureFromTheDeep, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 462 Fritz: packed own -> upgrade
lines.push(up.Fritz);
// 463 Nailhead: packed own -> upgrade (split from Psycho)
lines.push(up.Nailhead);
// 466 Psycho: deferred own -> box rest
lines.push('        (NpcKind::Psycho, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 464 CrimsonBunny: packed own -> upgrade (split from CrimsonGoldfish)
lines.push(up.CrimsonBunny);
// 465 CrimsonGoldfish: packed own -> upgrade
lines.push(up.CrimsonGoldfish);
// 467 DeadlySphere: deferred own -> box rest
lines.push('        (NpcKind::DeadlySphere, _) => (sheet_i, 22.0, 22.0, 22.0, white),');
// 468 DrManFly: deferred own -> box rest
lines.push('        (NpcKind::DrManFly, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 469 ThePossessed: packed own -> upgrade
lines.push(up.ThePossessed);
// 470 CrimsonPenguin: packed own -> upgrade
lines.push(up.CrimsonPenguin);
// 472 ShadowFlameApparition: packed own -> upgrade
lines.push(up.ShadowFlameApparition);
// 473-476 BigMimic: deferred own -> box rest (grouped)
lines.push('        (NpcKind::BigMimicCorruption, _)');
lines.push('        | (NpcKind::BigMimicCrimson, _)');
lines.push('        | (NpcKind::BigMimicHallow, _)');
lines.push('        | (NpcKind::BigMimicJungle, _) => (sheet_i, 44.0, 28.0, 44.0, white),');
// 477 Mothron: deferred own -> box rest
lines.push('        (NpcKind::Mothron, _) => (sheet_i, 50.0, 80.0, 50.0, white),');
// 478 MothronEgg: Hide-fold, box rest
lines.push('        (NpcKind::MothronEgg, _) => (sheet_i, 34.0, 34.0, 34.0, white),');
// 479 MothronSpawn: deferred own -> box rest
lines.push('        (NpcKind::MothronSpawn, _) => (sheet_i, 30.0, 46.0, 30.0, white),');
// 480 Medusa / 481 GreekSkeleton: deferred own -> box rest (grouped)
lines.push('        (NpcKind::Medusa, _)');
lines.push('        | (NpcKind::GreekSkeleton, _) => (sheet_i, 40.0, 18.0, 40.0, white),');
// 482 GraniteGolem: packed own -> upgrade
lines.push(up.GraniteGolem);
// 483 GraniteFlyer: deferred own -> box rest
lines.push('        (NpcKind::GraniteFlyer, _) => (sheet_i, 30.0, 20.0, 30.0, white),');
// 484 EnchantedNightcrawler: packed own -> upgrade (split from Grubby/Sluggy/Buggy)
lines.push(up.EnchantedNightcrawler);
// 485-487 Grubby/Sluggy/Buggy: packed own, SAME dims (18x72, 4 rows) -> grouped upgrade
lines.push('        (NpcKind::Grubby, _)');
lines.push('        | (NpcKind::Sluggy, _)');
lines.push('        | (NpcKind::Buggy, _) => {');
lines.push('            let h = entry_sheet(lookups, entry)');
lines.push('                .map(|s| s.height as f32 / 4.0)');
lines.push('                .unwrap_or(4.0);');
lines.push('            (sheet_i, h, 18.0, h, white)');
lines.push('        }');
// 488 TargetDummy: Hide-fold/no-entry, box rest
lines.push('        (NpcKind::TargetDummy, _) => (sheet_i, 40.0, 18.0, 40.0, white),');

const block = lines.join('\n');
fs.writeFileSync(path.join(__dirname, '_d40_portrait_block.txt'), block);
console.log('portrait block lines:', lines.length);
// verify all 56 kinds present
const names = [...block.matchAll(/NpcKind::(\w+)/g)].map(x => x[1]);
const set = new Set(names);
const missing = w.filter(o => !set.has(o.name));
console.log('kinds covered:', set.size, 'missing:', missing.map(o => o.name).join(',') || 'NONE');
// verify the 22 upgrades present
const upCount = (block.match(/entry_sheet\(lookups, entry\)/g) || []).length;
console.log('entry_sheet upgrades:', upCount, '(expect 22: 21 single + 1 grouped Grubby/Sluggy/Buggy = 22 arms covering 24 kinds? No: 22 packed own-entry kinds, Grubby/Sluggy/Buggy grouped = 1 arm for 3 kinds, so 20 single arms + 1 grouped = 21 arms covering 22 kinds)');
