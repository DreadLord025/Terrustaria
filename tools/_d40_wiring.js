// W4-d40 sheets phase: per-kind wiring analysis. For each of the 56 kinds,
// report: sheet dims (real .xnb), npcFrameCount (Main.cs verbatim), the
// SetFrameSize integer-division row height (NPC.cs:66666), whether the strip
// divides evenly, the bestiary visibility (own entry vs Hide-fold), the
// texture_net_id routing need, and the rotation whitelist verdict (from the
// C# AI/FindFrame census). Writes _d40_wiring.json.
'use strict';
const fs = require('fs');
const path = require('path');
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_dims.json'), 'utf8'));
const picked = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_picked_final.json'), 'utf8'));

// npcFrameCount from Main.cs:67082 (verbatim, already verified == NPC_FRAME_COUNT)
const cs = fs.readFileSync(path.join(__dirname, '..', 'decompiled', 'Terraria', 'Main.cs'), 'utf8').split('\n');
let s = -1;
for (let k = 0; k < cs.length; k++) { if (cs[k].includes('Main.npcFrameCount = new int[697]')) { s = k; break; } }
const fc = [];
for (let k = s + 2; k < cs.length; k++) { const l = cs[k]; if (/^\s*\};/.test(l)) break; fc.push(...[...l.matchAll(/\d+/g)].map(x => +x[0])); }

// kind name -> vanilla id (from npc/data.rs to_u16, the port's protocol band)
const kinds = {
  ArmedZombie: 430, ArmedZombieEskimo: 431, ArmedZombiePincussion: 432, ArmedZombieSlimed: 433,
  ArmedZombieSwamp: 434, ArmedZombieTwiggy: 435, ArmedZombieCenx: 436, CultistTablet: 437,
  CultistDevote: 438, CultistBossClone: 440, GoldBird: 442, GoldBunny: 443, GoldButterfly: 444,
  GoldFrog: 445, GoldGrasshopper: 446, GoldMouse: 447, GoldWorm: 448, BoneThrowingSkeleton: 449,
  BoneThrowingSkeleton2: 450, BoneThrowingSkeleton3: 451, BoneThrowingSkeleton4: 452,
  SkeletonMerchant: 453, CultistDragonHead: 454, CultistDragonBody1: 455, CultistDragonBody2: 456,
  CultistDragonBody3: 457, CultistDragonBody4: 458, CultistDragonTail: 459, Butcher: 460,
  CreatureFromTheDeep: 461, Fritz: 462, Nailhead: 463, CrimsonBunny: 464, CrimsonGoldfish: 465,
  Psycho: 466, DeadlySphere: 467, DrManFly: 468, ThePossessed: 469, CrimsonPenguin: 470,
  ShadowFlameApparition: 472, BigMimicCorruption: 473, BigMimicCrimson: 474, BigMimicHallow: 475,
  BigMimicJungle: 476, Mothron: 477, MothronEgg: 478, MothronSpawn: 479, Medusa: 480,
  GreekSkeleton: 481, GraniteGolem: 482, GraniteFlyer: 483, EnchantedNightcrawler: 484,
  Grubby: 485, Sluggy: 486, Buggy: 487, TargetDummy: 488,
};

// Hide-fold kinds: net_id() folds to the head, so texture_net_id needs an
// explicit vanilla row (the d35 DiggerBody/SeekerBody/VileSpit precedent).
// Own-entry kinds ride the `_ => kind.net_id()` wildcard (= vanilla id).
const hideFold = {
  ArmedZombie: 3, ArmedZombieEskimo: 161, ArmedZombiePincussion: 186, ArmedZombieSlimed: 187,
  ArmedZombieSwamp: 188, ArmedZombieTwiggy: 189, ArmedZombieCenx: 200, BoneThrowingSkeleton: 21,
  CultistDragonBody1: 454, CultistDragonBody2: 454, CultistDragonBody3: 454,
  CultistDragonBody4: 454, CultistDragonTail: 454, MothronEgg: 477,
};
// BoneThrowingSkeleton2/3/4 + TargetDummy have NO bestiary entry and NO fold
// (cs_id hand rows 450/451/452/488); net_id() falls through to... let's check.
// They have no entry, so entry_for_npc would panic. But cs_id has hand rows.
// net_id() = entry_for_npc(...).net_id — for these, entry_for_npc has no fold
// and no entry -> panic. So they MUST get a texture_net_id row too OR a fold.
// Actually the d40 data wave gave them cs_id hand rows but no bestiary entry.
// Let's mark them as needing explicit texture_net_id rows (vanilla id).
const noEntryHandRow = { BoneThrowingSkeleton2: 450, BoneThrowingSkeleton3: 451, BoneThrowingSkeleton4: 452, TargetDummy: 488 };

// Rotation whitelist verdicts from the C# census (AI + FindFrame writes):
// 442 GoldBird: FindFrame 73634 velocity.X*0.1 (bird group 74/297/298/442)
// 444 GoldButterfly: FindFrame 72745 velocity.X*0.3 (356/444/653/661 group)
// 448 GoldWorm: FindFrame 72804 +=direction*0.1, 72818/72823 slope ±0.785 (357/448/484/606)
// 461 CreatureFromTheDeep: FindFrame 75328/75340/75343 wet atan2 blend + dry decay
// 465 CrimsonGoldfish: AI_016 24010 velocity.Y*direction*0.1 clamp ±0.2 (fish)
// 467 DeadlySphere: AI_085 36991 +=direction*0.3 spin / 37037 velocity.X*0.1
// 469 ThePossessed: FindFrame 75668 atan2+1.57 lunge (ai[2]==1), 75694 =0
// 472 ShadowFlameApparition: FindFrame 71386 atan2(vY*dir, vX*dir)
// 477 Mothron: AI_088 37886 (rot*9+vX*0.1)/10 blend
// 478 MothronEgg: AI_089 38357/38362 +=vX*0.02/0.04
// 479 MothronSpawn: AI_090 38399 (rot*9+vX*0.1)/10 blend
// 483 GraniteFlyer: FindFrame 75185 +=vX*0.2 / 75199 =vX*0.1
// 484 EnchantedNightcrawler: FindFrame 72804/72818/72823 (same as 448)
// 485/486/487 Grubby/Sluggy/Buggy: FindFrame 72859 +=dir*0.1, 72873/72878 slope ±0.785
// 454-459 CultistDragon: AI_006 51878 follower atan2+1.57 / 52201 head atan2+PI/2
// NOT rotating (no rotation write in their AI/FindFrame):
// 430-436 ArmedZombie (AI_003 fighter, no rotation), 437/438 CultistTablet/Devote (AI_083, no rot),
// 440 CultistBossClone (AI_084, only =0 at 64592), 443 GoldBunny (AI_007 hop, no rot),
// 445 GoldFrog (AI_007, no rot), 446 GoldGrasshopper (AI_001, no rot), 447 GoldMouse (AI_007, no rot),
// 449-452 BoneThrowingSkeleton (AI_003, no rot), 453 SkeletonMerchant (AI_007 town, no rot),
// 460 Butcher/462 Fritz/463 Nailhead/466 Psycho (AI_003, no rot), 464 CrimsonBunny (AI_003, no rot),
// 468 DrManFly (AI_003, no rot), 470 CrimsonPenguin (AI_003, no rot),
// 473-476 BigMimic (AI_087 no rot; FindFrame only =0), 480 Medusa (AI_003, no rot),
// 481 GreekSkeleton (AI_003, no rot), 482 GraniteGolem (AI_003, no rot), 488 TargetDummy (AI_092, no rot)
const rotates = new Set([442, 444, 448, 461, 465, 467, 469, 472, 477, 478, 479, 483, 484, 485, 486, 487, 454, 455, 456, 457, 458, 459]);

const out = [];
for (const [name, id] of Object.entries(kinds)) {
  const d = dims[id];
  const rows = fc[id];
  const rowH = Math.floor(d.h / rows); // NPC.cs:66666 integer division
  const divides = d.h % rows === 0;
  const packed = picked.includes(id);
  const ownEntry = !(name in hideFold) && !(name in noEntryHandRow);
  const needsTexRow = (name in hideFold) || (name in noEntryHandRow);
  out.push({
    name, id, w: d.w, h: d.h, rows, rowH, divides, packed,
    ownEntry, needsTexRow, rotates: rotates.has(id),
  });
}
fs.writeFileSync(path.join(__dirname, '_d40_wiring.json'), JSON.stringify(out, null, 1));
// summary
console.log('kind id wxh rows rowH divides packed ownEntry texRow rotates');
for (const o of out) {
  console.log(`${o.name} ${o.id} ${o.w}x${o.h} ${o.rows} ${o.rowH} ${o.divides ? 'Y' : 'N'} ${o.packed ? 'PACK' : 'defer'} ${o.ownEntry ? 'own' : 'fold'} ${o.needsTexRow ? 'TEXROW' : '-'} ${o.rotates ? 'ROT' : '-'}`);
}
console.log('packed:', out.filter(o => o.packed).length, 'deferred:', out.filter(o => !o.packed).length);
console.log('non-dividing strips:', out.filter(o => !o.divides).map(o => o.id + '(' + o.h + '/' + o.rows + ')').join(','));
console.log('rotation whitelist:', out.filter(o => o.rotates).map(o => o.id).join(','));
console.log('needs texture_net_id row:', out.filter(o => o.needsTexRow).map(o => o.name + '=>' + o.id).join(','));
