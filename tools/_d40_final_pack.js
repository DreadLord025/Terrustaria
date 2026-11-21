// W4-d40 sheets phase: confirm the SOLO id-order-greedy pack (the d35 method)
// on the HEAD base — the 34 picked sheets' final skyline height (main + font),
// verify() no-overlap/in-bounds + double-run determinism, and the per-sheet
// greedy turn heights for the witness record. Also reports the CUMULATIVE
// (pushed-union) height for the same picked set, to document the overflow.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d40_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const picked = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_picked_final.json'), 'utf8'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_pushed_sheets.json'), 'utf8'));
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_dims.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'picked', 'pushed', 'dims', head + `
  const union = [...new Set(Object.values(pushed).flat())];
  const addNew = (base, names) => {
    const seen = new Set(base.map(s => s.name));
    const out = base.slice();
    for (const n of names) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (!d) continue;
      seen.add(n);
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.dupDropped = base.dupDropped;
    return out;
  };
  const base = buildSheetList({});
  const baseFont = buildSheetList({ font: true });
  const names = picked.map(i => 'NPC_' + i);
  const sm = addNew(base, names), sf = addNew(baseFont, names);
  const r1 = skylinePack(sm, 'waste'), r2 = skylinePack(sm, 'waste');
  const rf = skylinePack(sf, 'waste');
  verify(sm, r1); verify(sf, rf);
  const det = JSON.stringify(r1.layout) === JSON.stringify(r2.layout) && r1.height === r2.height;
  console.log('SOLO picked ' + picked.length + ' sheets: main ' + r1.height + ' font ' + rf.height +
    ' (bound 8192) deterministic=' + det + ' verified=no-overlap/in-bounds');
  // cumulative with the pushed union (the overflow documentation)
  const cm = addNew(addNew(base, union), names);
  const cr = skylinePack(cm, 'waste');
  console.log('CUMULATIVE pushed-union + picked ' + picked.length + ': ' + cr.height +
    ' px ' + (cr.height > 8192 ? 'OVERFLOW (+' + (cr.height - 8192) + ')' : 'ok'));
  // the deferred ids (56 - picked)
  const all = Object.keys(dims).map(Number).sort((a,b)=>a-b);
  const deferred = all.filter(i => !picked.includes(i));
  console.log('DEFERRED (' + deferred.length + '): ' + deferred.join(','));
  // per-deferred-sheet greedy-turn height (what it WOULD add at its turn)
  let acc = picked.slice();
  for (const id of deferred) {
    const trial = acc.concat([id]).sort((a,b)=>a-b);
    const hm = skylinePack(addNew(base, trial.map(i=>'NPC_'+i)), 'waste').height;
    console.log('  deferred NPC_' + id + ' (' + dims[id].w + 'x' + dims[id].h + '): solo-with-picked height ' + hm +
      (hm > 8192 ? ' OVERFLOW (+' + (hm-8192) + ')' : ' fits'));
  }
  return { main: r1.height, font: rf.height, det, cumulative: cr.height, picked, deferred };
`);
const out = fn(require, __dirname, picked, pushed, dims);
fs.writeFileSync(path.join(__dirname, '_d40_final_pack.json'), JSON.stringify(out, null, 1));
