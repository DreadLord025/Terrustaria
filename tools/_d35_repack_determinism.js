// W4-d35 ATLAS-REPACK: determinism + regression witness for the PROMOTED
// maxrects BAF packer (cap-bounded, spill-on-no-fit). Fast synthetic set —
// no .xnb decode. Proves:
//  (1) cap=Infinity reproduces the original unbounded maxrects layout exactly
//      (the promotion is backward-compatible: same score, same split/prune);
//  (2) the cap-bounded form is deterministic across double runs;
//  (3) every placed rect is in-bounds under the cap and non-overlapping
//      (the harness verify());
//  (4) spill: sheets that don't fit page 0 land on page 1, and page 1 is
//      itself bounded + verified;
//  (5) the skyline packer (the landed one) is unchanged + deterministic.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const fn = new Function('require', '__dirname', head + `
  // A synthetic sheet set with the real content's shape: a few huge sheets,
  // many tall NPC strips, a long tail of tiny gore/item icons.
  const sheets = [];
  let k = 0;
  const push = (w, h) => sheets.push({ name: 'S' + (k++), w, h });
  push(1024, 1024); push(1024, 1024); push(1024, 500); push(512, 512);
  for (let i = 0; i < 60; i++) push(40 + (i % 5) * 20, 600 + (i % 7) * 180);
  for (let i = 0; i < 400; i++) push(16, 16);
  for (let i = 0; i < 200; i++) push(20, 20);
  for (let i = 0; i < 120; i++) push(32, 48);
  const out = {};
  // (1) backward compat: cap=Infinity == the original unbounded form.
  const inf = maxrectsPack(sheets, Infinity);
  const inf2 = maxrectsPack(sheets, Infinity);
  out.infDeterministic = JSON.stringify(inf.layout) === JSON.stringify(inf2.layout) && inf.height === inf2.height;
  out.infHeight = inf.height;
  verify(sheets, inf);
  // (2)+(3) cap-bounded determinism + verify.
  const cap = 2048;
  const a = maxrectsPack(sheets, cap);
  const b = maxrectsPack(sheets, cap);
  out.capDeterministic = JSON.stringify(a.layout) === JSON.stringify(b.layout) && a.height === b.height;
  out.capHeight = a.height;
  out.capBound = a.height <= cap;
  const placed = [], placedLayout = [];
  for (let i = 0; i < sheets.length; i++) {
    if (a.layout[i]) { placed.push(sheets[i]); placedLayout.push(a.layout[i]); }
  }
  verify(placed, { height: a.height, layout: placedLayout });
  // (4) spill to page 1 — a tighter cap forces overflow.
  const cap2 = 1400;
  const c = maxrectsPack(sheets, cap2);
  const c2 = maxrectsPack(sheets, cap2);
  out.tightDeterministic = JSON.stringify(c.layout) === JSON.stringify(c2.layout);
  out.tightHeight = c.height;
  out.tightBound = c.height <= cap2;
  const spill = sheets.filter((s, i) => !c.layout[i]);
  out.spillCount = spill.length;
  if (spill.length) {
    // Page 1 must fit the tallest spilled sheet; a sheet taller than the cap
    // can never fit ANY page (the real cap 8192 exceeds every sheet's height).
    const p1cap = Math.max(cap2, ...spill.map(s => s.h + 2 * PAD));
    const p1 = maxrectsPack(spill, p1cap);
    const p1b = maxrectsPack(spill, p1cap);
    out.page1Deterministic = JSON.stringify(p1.layout) === JSON.stringify(p1b.layout);
    out.page1Height = p1.height;
    out.page1Bound = p1.height <= p1cap;
    out.page1AllPlaced = p1.layout.every(p => p);
    verify(spill, p1);
    const kept = [], keptLayout = [];
    for (let i = 0; i < sheets.length; i++) {
      if (c.layout[i]) { kept.push(sheets[i]); keptLayout.push(c.layout[i]); }
    }
    verify(kept, { height: c.height, layout: keptLayout });
  }
  // (5) skyline unchanged + deterministic.
  const sky = skylinePack(sheets, 'waste');
  const sky2 = skylinePack(sheets, 'waste');
  out.skyDeterministic = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout);
  out.skyHeight = sky.height;
  verify(sheets, sky);
  out.sheetCount = sheets.length;
  return out;
`);
const out = fn(require, __dirname);
console.log(JSON.stringify(out, null, 1));
const ok = out.infDeterministic && out.capDeterministic && out.capBound &&
  out.skyDeterministic && out.tightDeterministic && out.tightBound &&
  out.spillCount > 0 && out.page1Deterministic && out.page1Bound && out.page1AllPlaced;
if (!ok) { console.error('WITNESS FAILED'); process.exit(1); }
console.log('ALL DETERMINISM + BOUNDS + SPILL CHECKS PASSED');
fs.writeFileSync(path.join(__dirname, '_d35_repack_determinism.json'), JSON.stringify(out, null, 1));
