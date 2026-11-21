// W4-d40 sheets phase: measure the REAL retail .xnb dims for the 56 kinds'
// NPC_<id> sheets through the pinned c4 LZX harness (the b4 copy whose
// L-8b fallback survives the wave-2 branch purge). Writes _d40_dims.json.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d40_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const IDS = [
  430, 431, 432, 433, 434, 435, 436, 437, 438, 440,
  442, 443, 444, 445, 446, 447, 448, 449, 450, 451,
  452, 453, 454, 455, 456, 457, 458, 459, 460, 461,
  462, 463, 464, 465, 466, 467, 468, 469, 470, 472,
  473, 474, 475, 476, 477, 478, 479, 480, 481, 482,
  483, 484, 485, 486, 487, 488,
];
const fn = new Function('require', '__dirname', 'IDS', head + `
  const out = {};
  for (const id of IDS) {
    const d = sheetDims('NPC_' + id);
    if (!d) { console.log('MISSING NPC_' + id); continue; }
    out[id] = d;
    console.log('NPC_' + id + ': ' + d.w + 'x' + d.h);
  }
  return out;
`);
const dims = fn(require, __dirname, IDS);
fs.writeFileSync(path.join(__dirname, '_d40_dims.json'), JSON.stringify(dims, null, 1));
console.log('measured', Object.keys(dims).length, 'of', IDS.length);
