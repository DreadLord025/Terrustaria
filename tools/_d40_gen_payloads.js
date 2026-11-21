// W4-d40 sheets phase: generate the exact edit payloads (strings) for the
// wiring, written to _d40_payloads.json so the edits are mechanical and
// verified against the measured dims/rows. No game logic here — just the
// const-list rows, the texture_net_id rows, the nudge rows, the draw arm,
// the rotation whitelist, the bestiary additions, and the portrait upgrades.
'use strict';
const fs = require('fs');
const path = require('path');
const w = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_wiring.json'), 'utf8'));
const boxes = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_boxes.json'), 'utf8'));
const packed = w.filter(o => o.packed);
const packedOwn = w.filter(o => o.packed && o.ownEntry);
const rot = w.filter(o => o.rotates);
const texRows = w.filter(o => o.needsTexRow);

const P = {};
// 1. STATIC_SHEET_NAMES rows (36)
P.staticRows = packed.map(o => '        "NPC_' + o.id + '",').join('\n');
// 2. BESTIARY_NPC_SHEET_IDS additions (22 packed own-entry, id order)
P.bestiaryIds = packedOwn.map(o => o.id).join(', ');
// 3. bestiary_npc_sheets vec rows (22, parallel order)
P.bestiaryVec = packedOwn.map(o => '                atlas.sheet("NPC_' + o.id + '").copied(),').join('\n');
// 4. D40_SHEETS const rows (36)
P.d40Const = packed.map(o => '    ' + o.id + ', // ' + o.name + (o.ownEntry ? '' : ' (Hide-fold)')).join('\n');
// 5. texture_net_id rows (18)
P.texRows = texRows.map(o => '        NpcKind::' + o.name + ' => ' + o.id + ',').join('\n');
// 6. nudge rows (6) — the C# Main.cs:23003-23046 else-if overrides
P.nudgeRows = [
  '        // W4-d40: the `Main.cs:23003-23046` num46/num47 else-if overrides',
  '        // for the 421-490 band (the generic draw anchor, `:25511`).',
  '        DeadlySphere => num46 = 7.0, // `:23003` type == 467',
  '        EnchantedNightcrawler => num46 = 2.0, // `:23019` type == 484',
  '        GraniteFlyer => num46 = 14.0, // `:23023` type == 483',
  '        Mothron => num47 = 22.0, // `:23027` type == 477',
  '        MothronEgg => num46 -= 2.0, // `:23031` type == 478',
  '        ThePossessed if n.ai[2] == 1.0 => num46 = 14.0, // `:23043` type == 469 && ai[2] == 1',
].join('\n');
// 7. draw arm (56 kinds grouped)
P.drawArm = [
  '            // W4-d40 sheets phase: the 56 kinds of the 421-490 band draw',
  '            // their OWN `NPC_<id>` sheet (`D40_SHEETS`, the 36 packed; the',
  '            // 20 deferred return None and the draw skips — the wyvern',
  '            // precedent) at `SetFrameSize`\'s integer-division row height',
  '            // (`NPC.cs:66666`) over `Main.npcFrameCount[type]`',
  '            // (`Main.cs:67082`, `npc_rows`), generic centred anchor',
  '            // (`Main.cs:25511-25516`) — the blood-moon/small-kinds shape.',
  '            // The Hide-fold kinds (ArmedZombie*, BoneThrowingSkeleton*,',
  '            // CultistDragonBody*, MothronEgg, TargetDummy) draw their own',
  '            // sheets via `texture_net_id`\'s vanilla rows (the d35',
  '            // DiggerBody/SeekerBody fishron-hazard precedent).',
  ...w.map((o, i) => (i === 0 ? '            NpcKind::' : '            | NpcKind::') + o.name),
  '            => {',
  '                let ty = texture_net_id(n.kind);',
  '                pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)',
  '            }',
].join('\n');
// 8. rotation whitelist (22)
P.rotRows = rot.map(o => '                    | NpcKind::' + o.name).join('\n');
// 9. portrait upgrades (22 packed own-entry)
P.portraits = packedOwn.map(o => {
  const b = boxes[o.name];
  return {
    name: o.name, id: o.id, rows: o.rows, sheetW: o.w, boxH: b.h, boxW: b.w,
    arm: '        (NpcKind::' + o.name + ', _) => {\n' +
      '            let h = entry_sheet(lookups, entry)\n' +
      '                .map(|s| s.height as f32 / ' + o.rows + '.0)\n' +
      '                .unwrap_or(' + b.h + ');\n' +
      '            (sheet_i, h, ' + o.w + '.0, h, white)\n' +
      '        }',
  };
});
fs.writeFileSync(path.join(__dirname, '_d40_payloads.json'), JSON.stringify(P, null, 1));
console.log('payloads written. counts:');
console.log('staticRows', packed.length, 'bestiaryIds', packedOwn.length, 'bestiaryVec', packedOwn.length);
console.log('d40Const', packed.length, 'texRows', texRows.length, 'nudgeRows 6', 'drawArm kinds', w.length, 'rotRows', rot.length, 'portraits', packedOwn.length);
