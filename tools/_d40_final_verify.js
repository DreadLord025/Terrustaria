// W4-d40 sheets phase: final comprehensive verification.
'use strict';
const fs = require('fs');
const path = require('path');
const t = fs.readFileSync(path.join(__dirname, '..', 'crates', 'tr-app', 'src', 'render', 'entities.rs'), 'utf8');
const w = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_wiring.json'), 'utf8'));
// texture_net_id 18 rows
const i = t.indexOf('pub(crate) fn texture_net_id');
const j = t.indexOf('\n}', i);
const seg = t.slice(i, j);
const tex = w.filter(o => o.needsTexRow);
console.log('tex18:', tex.every(o => seg.includes('NpcKind::' + o.name + ' => ' + o.id + ',')));
// rotation 22
const ri = t.indexOf('| NpcKind::DungeonSpirit');
const rj = t.indexOf('\n            )', ri);
const rseg = t.slice(ri, rj);
const rot = w.filter(o => o.rotates);
console.log('rot22:', rot.every(o => rseg.includes('| NpcKind::' + o.name)));
// nudge 6
const ni = t.indexOf('fn npc_anchor_nudge');
const nj = t.indexOf('\n}', ni);
const nseg = t.slice(ni, nj);
const nudgeRows = ['DeadlySphere => num46 = 7.0', 'EnchantedNightcrawler => num46 = 2.0', 'GraniteFlyer => num46 = 14.0', 'Mothron => num47 = 22.0', 'MothronEgg => num46 -= 2.0', 'ThePossessed if n.ai[2] == 1.0 => num46 = 14.0'];
console.log('nudge6:', nudgeRows.every(s => nseg.includes(s)));
// draw arm 56
const di = t.indexOf('NpcKind::ArmedZombie\r\n');
const dj = t.indexOf('}', t.indexOf('pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)', di));
const darm = t.slice(di, dj + 1);
console.log('draw56:', w.every(o => darm.includes('NpcKind::' + o.name)));
console.log('draw pumpkin_moon_draw:', darm.includes('pumpkin_moon_draw'));
// D40_SHEETS 36
const ci = t.indexOf('pub(crate) const D40_SHEETS');
const cj = t.indexOf('];', ci);
const cids = [...t.slice(ci, cj).matchAll(/^\s*(\d+),/gm)].map(x => +x[1]);
const packed = w.filter(o => o.packed).map(o => o.id);
console.log('D40_SHEETS == packed:', JSON.stringify(cids) === JSON.stringify(packed));
// tests
console.log('test1 56 cases:', (t.slice(t.indexOf('fn the_d40_rows_match'), t.indexOf('\n    }', t.indexOf('fn the_d40_rows_match'))).match(/\(NpcKind::/g) || []).length === 56);
console.log('test2 36 cases:', (t.slice(t.indexOf('fn the_d40_strips'), t.indexOf('\n    }', t.indexOf('fn the_d40_strips'))).match(/\(NpcKind::/g) || []).length === 36);
// audit
console.log('audit 429:', t.includes('396u16..=429'));
console.log('audit D40 loop:', t.includes('for &id in D40_SHEETS'));
// exhaustive draw match
const mi = t.indexOf('let (tex, frame_h, size, pos) = match n.kind {');
const mj = t.indexOf('\n        };', mi);
const mseg = t.slice(mi, mj);
const mkinds = new Set([...mseg.matchAll(/NpcKind::(\w+)/g)].map(x => x[1]));
console.log('draw match exhaustive (326):', mkinds.size === 326);
