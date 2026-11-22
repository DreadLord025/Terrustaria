// W4-d35 ATLAS-REPACK phase: skyline vs maxrects-BAF vs 2-page measurement.
// Reuses the pinned b4 harness (_d35_atlas_sim_pinned.js: LZX decoder over the
// REAL retail .xnb dims, assets.rs enumeration mirror, skyline + maxrects
// packers, verify()) by eval'ing its source up to main().
//
// Scenarios (each run with and without the boot font):
//   base            HEAD a725acb0's enumeration (gore landed)
//   +pushed         every sheet-name row the 26 in-flight W4 branches ADD
//                   (tools/_d35_pushed_sheets.json, re-extracted this phase;
//                   union 339 names incl. d35's own 21 / d37's 46 / d38's 35)
//   +d40            FORECAST: W4-d40 (421-490, data-only on its branch today)
//                   will pack its 56 kinds' NPC_<id> sheets in its sheets
//                   phase — the ids come from its npc/data.rs match arms.
//   +d35deferred    this branch's 12 deferred sheets (79/80/84/85/86/103/
//                   104/109/110/120/122/140), the worst case if a later
//                   repack admits them all.
//   +L8b            the pinned mount additions (the witness set).
//
// For every scenario: skyline (waste tie-break, the landed packer) vs
// maxrects BAF (the candidate), both verified (no overlap incl. PAD,
// in-bounds) and double-run determinism-checked. If a packer still exceeds
// 8192, a 2-page split is simulated: same order, same packer, page 0 capped
// at 8192, overflow sheets re-packed into page 1 (fresh bin).
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets.json'), 'utf8'));

// W4-d40's 56 kinds (npc/data.rs match arms on origin/task/W4-d40 at 8fe1d6d3).
const D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140];

const fn = new Function('require', '__dirname', 'pushed', 'D40_IDS', 'D35_DEFERRED', head + `
  const addNew = (base, names) => {
    const seen = new Set(base.map(s => s.name));
    const out = base.slice();
    const missing = [];
    for (const n of names) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (!d) { missing.push(n); continue; }
      seen.add(n);
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.dupDropped = base.dupDropped;
    out.missing = missing;
    return out;
  };
  // 2-page split, two forms:
  // (a) SPILL (maxrects): pack the whole set into a cap-bounded page 0; the
  //     sheets with no fitting free rect spill to a fresh page 1 (same
  //     packer, same order). This is the page-aware build_ref contract.
  // (b) BOUNDARY (skyline, no cap support): pack unbounded once; sheets
  //     whose rect fits entirely under the cap keep their page-0 position
  //     (UV-neutral below the boundary); the tail re-packs into page 1.
  const spillTwoPage = (sheets, cap) => {
    const r0full = maxrectsPack(sheets, cap);
    const cur0 = [], cur1 = [], keep = [];
    for (let i = 0; i < sheets.length; i++) {
      const p = r0full.layout[i];
      if (p) { cur0.push(sheets[i]); keep.push(p); } else cur1.push(sheets[i]);
    }
    const r0 = { height: r0full.height, layout: keep };
    const r1 = cur1.length ? maxrectsPack(cur1, cap) : { height: 0, layout: [] };
    if (r1.layout.some(p => !p)) throw new Error('page 1 overflow too');
    return { h0: r0.height, h1: r1.height, n0: cur0.length, n1: cur1.length, r0, r1, cur0, cur1 };
  };
  const boundaryTwoPage = (sheets, cap) => {
    const full = skylinePack(sheets, 'waste');
    const cur0 = [], cur1 = [], keep = [];
    for (let i = 0; i < sheets.length; i++) {
      const p = full.layout[i], s = sheets[i];
      if (p.y + s.h <= cap - PAD) { cur0.push(s); keep.push(p); }
      else cur1.push(s);
    }
    const h0 = keep.length ? Math.max(...keep.map((p, k) => p.y + cur0[k].h)) + PAD : PAD;
    const r0 = { height: h0, layout: keep };
    const r1 = cur1.length ? skylinePack(cur1, 'waste') : { height: 0, layout: [] };
    return { h0, h1: r1.height, n0: cur0.length, n1: cur1.length, r0, r1, cur0, cur1 };
  };
  // (c) INCREMENTAL: page 0 = the HEAD base set packed ALONE (its layout is
  //     exactly today's — every existing UV unchanged), page 1 = only the
  //     additions. The cheapest merger contract.
  const incremental = (base, extra, pack) => {
    const r0 = pack(base), r1 = extra.length ? pack(extra) : { height: 0, layout: [] };
    return { h0: r0.height, h1: r1.height, n0: base.length, n1: extra.length, r0, r1, cur0: base, cur1: extra };
  };
  const union = [...new Set(Object.values(pushed).flat())];
  const d40Names = D40_IDS.map(id => 'NPC_' + id);
  const defNames = D35_DEFERRED.map(id => 'NPC_' + id);
  const results = [];
  // Resume support: scenarios already recorded in the JSON are skipped (the
  // full sweep takes ~15 min; a SIGTERM'd run can finish its tail cheaply).
  const outFile = path.join(__dirname, '_d35_repack_measure.json');
  let done = new Set();
  try {
    for (const r of JSON.parse(fs.readFileSync(outFile, 'utf8'))) { done.add(r.scenario); results.push(r); }
  } catch (e) {}
  const save = () => fs.writeFileSync(outFile, JSON.stringify(results, null, 1));
  for (const font of [false, true]) {
    const base = buildSheetList({ font });
    const sets = [['base', base, []]];
    let acc = addNew(base, union);
    sets.push(['+pushed(' + union.length + ')', acc, union]);
    acc = addNew(acc, d40Names);
    sets.push(['+d40forecast(' + d40Names.length + ')', acc, union.concat(d40Names)]);
    acc = addNew(acc, defNames);
    sets.push(['+d35deferred(' + defNames.length + ')', acc, union.concat(d40Names, defNames)]);
    acc = addNew(acc, L8B_ADDITIONS);
    sets.push(['+L8b(' + L8B_ADDITIONS.length + ')', acc, union.concat(d40Names, defNames, L8B_ADDITIONS)]);
    for (const [label, sheets, addNames] of sets) {
      const tag = (font ? 'font ' : '') + label;
      if (done.has(tag)) { console.log(tag + ': (cached)'); continue; }
      const sky = skylinePack(sheets, 'waste');
      const sky2 = skylinePack(sheets, 'waste');
      verify(sheets, sky);
      if (JSON.stringify(sky.layout) !== JSON.stringify(sky2.layout)) throw new Error('skyline nondeterministic: ' + tag);
      const mr = maxrectsPack(sheets);
      const mr2 = maxrectsPack(sheets);
      verify(sheets, mr);
      if (JSON.stringify(mr.layout) !== JSON.stringify(mr2.layout)) throw new Error('maxrects nondeterministic: ' + tag);
      const area = sheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
      const row = { scenario: tag, sheets: sheets.length,
        missing: sheets.missing || [],
        areaMpx: +(area / 1e6).toFixed(2),
        areaBound: Math.ceil(area / (MAX_WIDTH - PAD)) + PAD,
        skyline: sky.height, maxrects: mr.height,
        skyOver: Math.max(0, sky.height - 8192), mrOver: Math.max(0, mr.height - 8192) };
      if (mr.height > 8192 || sky.height > 8192) {
        const sp = spillTwoPage(sheets, 8192);
        verify(sp.cur0, sp.r0); if (sp.cur1.length) verify(sp.cur1, sp.r1);
        row.spillMaxrects = { page0: sp.h0, page1: sp.h1, n0: sp.n0, n1: sp.n1 };
        const bd = boundaryTwoPage(sheets, 8192);
        verify(bd.cur0, bd.r0); if (bd.cur1.length) verify(bd.cur1, bd.r1);
        row.boundarySkyline = { page0: bd.h0, page1: bd.h1, n0: bd.n0, n1: bd.n1 };
      }
      console.log(tag + ': ' + sheets.length + ' sheets, area ' + row.areaMpx +
        ' Mpx (lower bound ' + row.areaBound + ' px) | skyline ' + sky.height +
        (row.skyOver ? ' OVER+' + row.skyOver : '') + ' | maxrects ' + mr.height +
        (row.mrOver ? ' OVER+' + row.mrOver : '') +
        (row.spillMaxrects ? ' | spill-mr ' + row.spillMaxrects.page0 + '/' + row.spillMaxrects.page1 + ' (' + row.spillMaxrects.n1 + ' on p1)' : '') +
        (row.boundarySkyline ? ' | bound-sky ' + row.boundarySkyline.page0 + '/' + row.boundarySkyline.page1 + ' (' + row.boundarySkyline.n1 + ' on p1)' : ''));
      if (addNames.length) {
        const extras = sheets.slice(base.length);
        const incSky = incremental(base, extras, (ss) => skylinePack(ss, 'waste'));
        const incMr = incremental(base, extras, (ss) => maxrectsPack(ss));
        verify(incSky.cur1, incSky.r1); verify(incMr.cur1, incMr.r1);
        row.incrementalSkyline = { page0: incSky.h0, page1: incSky.h1, n1: incSky.n1 };
        row.incrementalMaxrects = { page0: incMr.h0, page1: incMr.h1, n1: incMr.n1 };
        console.log('  incremental (page0 = HEAD base unchanged): sky ' + incSky.h0 + '/' + incSky.h1 +
          ' | mr ' + incMr.h0 + '/' + incMr.h1 + ' (' + incSky.n1 + ' additions on p1)');
      }
      if (row.missing.length) console.log('  missing files skipped: ' + row.missing.join(', '));
      results.push(row);
      save();
    }
  }
  return results;
`);
const results = fn(require, __dirname, pushed, D40_IDS, D35_DEFERRED);
console.log('wrote tools/_d35_repack_measure.json (' + results.length + ' scenarios)');
