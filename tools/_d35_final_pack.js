// W4-d35 sheets phase: FINAL packing-order witness.
// The picked 21 sheets are appended to STATIC_SHEET_NAMES in ascending id
// order (the loader's list order = the packer's input-order tie-break).
// Measures the exact final layout: HEAD base + the 21, main and font sets,
// skyline waste tie-break, verify() + determinism, and records each sheet's
// rect for the commit evidence.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const picked = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_picked_idorder.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'picked', head + `
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
  const names = picked.slice().sort((a,b)=>a-b).map(id => 'NPC_' + id);
  const base = buildSheetList({});
  const baseFont = buildSheetList({ font: true });
  const sm = addNew(base, names), sf = addNew(baseFont, names);
  const rm = skylinePack(sm, 'waste'), rm2 = skylinePack(sm, 'waste');
  const rf = skylinePack(sf, 'waste');
  verify(sm, rm); verify(sf, rf);
  const det = JSON.stringify(rm.layout) === JSON.stringify(rm2.layout);
  console.log('picked (' + picked.length + '):', picked.join(','));
  console.log('main:', sm.length, 'sheets ->', rm.height, 'px | font:', sf.length, '->', rf.height, 'px | deterministic:', det);
  if (rm.height > 8192 || rf.height > 8192) throw new Error('OVERFLOW');
  const rects = {};
  for (let i = 0; i < sm.length; i++) {
    if (names.includes(sm[i].name)) rects[sm[i].name] = { ...sm[i], ...rm.layout[i] };
  }
  return { main: rm.height, font: rf.height, sheets: sm.length, rects };
`);
const out = fn(require, __dirname, picked);
fs.writeFileSync(path.join(__dirname, '_d35_final_layout.json'), JSON.stringify(out, null, 1));
console.log('wrote _d35_final_layout.json');
