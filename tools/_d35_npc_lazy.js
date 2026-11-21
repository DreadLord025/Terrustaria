// W4-d35 ATLAS FINAL phase: the NPC-sheet lazy-load census measurement.
// research/113 section 10. The C# registers ALL TextureAssets.Npc entries
// DoNotLoad (AssetInitializer.cs:127-129) and Main.LoadNPC (Main.cs:
// 59084-59090) fires from NPC.SetFrameSize (NPC.cs:66661-66663) at
// DrawNPCDirect (Main.cs:22513-22516) — i.e. per DRAWN NPC, plus the
// bestiary icon hover (UnlockableNPCEntryIcon.cs:185) and 10 cross-draw
// sites. So the NPC corpus is load-on-demand exactly like the bg corpus
// (section 8). This script measures what that means for the merged-W4
// page budget:
//   A. family census: every atlas family's C# eager/lazy status vs the
//      port's loader half (the full-asset eager-vs-lazy table), with REAL
//      .xnb dims + padded areas.
//   B. NPC corpus alone: all NPC_<id> files (the load-everything upper
//      bound the port packs today via STATIC_SHEET_NAMES).
//   C. merged-407 base re-measured (font + pushed-339 + d40-56 +
//      d35-deferred-12, the 113 section 7 base) + boundary pages.
//   D. realistic per-world NPC working sets (from _d35_npc_census.json):
//      - fresh world: town roster + ambient spawner kinds
//      - long-lived world: + world_sim spawn sites + moon ladders
//      - bestiary-opened: + every ENTRIES net id (the hover draw)
//      mixed into the base AND on their own pages after the base.
//   E. the lazy-union scenario: the merged base with ALL NPC_ sheets
//      pulled OUT of the union input (the section-10 contract form),
//      then the per-world NPC sets appended as own pages.
//   F. the seed-42 bg page (7448 px, _d35_bg_inventory.json pageform)
//      stacked with each NPC set: the realistic total page budget.
// Every layout: verify() + double-run skyline determinism; boundary
// split recursed to N pages. Run: node tools/_d35_npc_lazy.js
// (resume-safe via _d35_npc_lazy.json). No cargo, no Rust touched.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets.json'), 'utf8'));
// PINNED at the section-7 extraction (26 branches / 339 names). The FINAL
// re-extraction (37 branches / 424 names) is recorded separately in
// _d35_pushed_sheets.json; the 407-base scenarios below use this pinned
// copy so the numbers stay comparable with sections 1-9.
const pushed339 = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets_339.json'), 'utf8'));
const census = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_npc_census.json'), 'utf8'));
const bgInv = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_bg_inventory.json'), 'utf8'));

const D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140];

const fn = new Function('require', '__dirname', 'pushed', 'pushed339', 'census', 'bgInv',
  'D40_IDS', 'D35_DEFERRED', head + `
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
    out.missing = missing;
    return out;
  };
  const boundaryNPages = (sheets, cap) => {
    const pages = [];
    let rest = sheets;
    while (rest.length) {
      const full = skylinePack(rest, 'waste');
      const keep = [], cur = [], tail = [];
      for (let i = 0; i < rest.length; i++) {
        const p = full.layout[i], s = rest[i];
        if (p.y + s.h <= cap - PAD) { cur.push(s); keep.push(p); }
        else tail.push(s);
      }
      if (!cur.length) throw new Error('no progress: sheet taller than cap');
      const h = Math.max(...keep.map((p, k) => p.y + cur[k].h)) + PAD;
      verify(cur, { height: h, layout: keep });
      pages.push({ height: h, n: cur.length });
      rest = tail;
    }
    return pages;
  };
  const areaOf = sheets => sheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
  const sheetsOf = names => {
    const out = [], missing = [];
    for (const n of new Set(names)) {
      const d = sheetDims(n);
      if (!d) { missing.push(n); continue; }
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.missing = missing;
    return out;
  };

  const outFile = path.join(__dirname, '_d35_npc_lazy.json');
  let results = [], done = new Set(), families = null, npcCorpus = null;
  try {
    const prev = JSON.parse(fs.readFileSync(outFile, 'utf8'));
    results = prev.scenarios || [];
    for (const r of results) done.add(r.scenario);
    families = prev.families; npcCorpus = prev.npcCorpus;
  } catch (e) {}
  const save = () => fs.writeFileSync(outFile, JSON.stringify({
    families, npcCorpus, scenarios: results }, null, 1));

  const measure = (label, sheets, note) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    if (JSON.stringify(sky.layout) !== JSON.stringify(sky2.layout)) throw new Error('nondeterministic: ' + label);
    const area = areaOf(sheets);
    const row = { scenario: label, note: note || '', sheets: sheets.length,
      missing: sheets.missing || [],
      areaMpx: +(area / 1e6).toFixed(2),
      areaBound: Math.ceil(area / (MAX_WIDTH - PAD)) + PAD,
      skyline: sky.height, skyOver: Math.max(0, sky.height - 8192) };
    row.pages = boundaryNPages(sheets, 8192);
    console.log(label + ': ' + sheets.length + ' sheets, ' + row.areaMpx +
      ' Mpx (bound ' + row.areaBound + ') | skyline ' + sky.height +
      (row.skyOver ? ' OVER+' + row.skyOver : '') +
      ' | pages ' + row.pages.map(p => p.height + '(' + p.n + ')').join(' / '));
    if (row.missing.length) console.log('  missing: ' + row.missing.join(', '));
    results.push(row); save();
  };

  // ---- A. the family census (C# status from AssetInitializer.cs, the
  // port half from assets.rs's loader arms). Areas from REAL dims.
  if (!families) {
    const fam = [];
    const push = (family, names, cs, csRef, port, portRef) => {
      const ss = sheetsOf(names);
      fam.push({ family, sheets: ss.length, missing: ss.missing,
        areaMpx: +(areaOf(ss) / 1e6).toFixed(3), cs, csRef, port, portRef });
    };
    push('NPC', enumIds('NPC_').map(i => 'NPC_' + i),
      'DoNotLoad ALL', 'AssetInitializer.cs:127-129; LoadNPC Main.cs:59084-59090 per drawn NPC (SetFrameSize NPC.cs:66661-66663 <- DrawNPCDirect Main.cs:22516)',
      'EAGER (STATIC_SHEET_NAMES rows)', 'assets.rs STATIC_SHEET_NAMES + load_static_sheets');
    push('Item', enumIds('Item_').map(i => 'Item_' + i),
      'DoNotLoad ALL (TextureCopyLoad aliases share)', 'AssetInitializer.cs:115-126; LoadItem Main.cs:59076-59082 per drawn item (DrawItem Main.cs:38608, slots :38539)',
      'EAGER (enumerate all)', 'assets.rs:1353-1364 enumerate_item_icon_ids');
    push('Projectile', [...PROJ_KINDS].sort((a,b)=>a-b).map(k => 'Projectile_' + k),
      'DoNotLoad ALL', 'AssetInitializer.cs:131-133; LoadProjectile Main.cs:59092-59098 per drawn proj (DrawProjDirect Main.cs:27935)',
      'EAGER (spawnable_kinds + 734)', 'assets.rs:1329-1343');
    push('Gore', enumIds('Gore_', [910, 911]).map(i => 'Gore_' + i),
      'DoNotLoad ALL', 'AssetInitializer.cs:135-137; LoadGore Main.cs:59100-59106 per active gore (DrawGore Main.cs:21888, DrawGoreBehind :21857)',
      'EAGER (enumerate all, E-4b)', 'assets.rs:1384-1393 enumerate_gore_sheet_ids');
    push('Buff', enumIds('Buff_').map(i => 'Buff_' + i),
      'EAGER (mode)', 'AssetInitializer.cs:418-420',
      'EAGER (enumerate all)', 'assets.rs:1369-1378');
    push('Wall', enumIds('Wall_').map(i => 'Wall_' + i),
      'DoNotLoad ALL', 'AssetInitializer.cs:139-141; LoadWall Main.cs:59108-59114 per drawn wall (WallDrawing.cs:116, TileDrawing.cs:244)',
      'WORLD-SELECTIVE (usage scan)', 'assets.rs load_world_sheets walls arm');
    push('Tiles', enumIds('Tiles_').map(i => 'Tiles_' + i),
      'DoNotLoad ALL', 'AssetInitializer.cs:143-145; LoadTiles Main.cs:59116-59122 per drawn tile (TileDrawing.cs:223/:1482)',
      'WORLD-SELECTIVE (usage scan)', 'assets.rs load_world_sheets tiles arm');
    push('Background', Array.from({length:344},(_,i)=>'Background_'+i),
      'DoNotLoad ALL; only 0/49 eager', 'AssetInitializer.cs:239-242/:422-423; LoadBackground Main.cs:59039-59074 per drawn layer (49 sites)',
      'WORLD-SELECTIVE (5 owned families) + static defaults', 'assets.rs:172-205 + STATIC_SHEET_NAMES bg rows');
    push('Underworld', Array.from({length:14},(_,i)=>'Backgrounds/Underworld '+i),
      'DoNotLoad ALL', 'AssetInitializer.cs:390-393; direct Request per drawn layer Main.cs:52687-52691',
      'UNWIRED (g72)', 'render/underworld_bg.rs plan adapters');
    push('NpcHead', enumIds('NPC_Head_').map(i => 'NPC_Head_' + i),
      'EAGER (mode)', 'AssetInitializer.cs:406-408 (minimap heads + bestiary)',
      'EAGER (named static rows)', 'assets.rs:774+ STATIC_SHEET_NAMES NPC_Head_* rows; e17/f29 append the tail');
    push('NpcHeadBoss', enumIds('NPC_Head_Boss_').map(i => 'NPC_Head_Boss_' + i),
      'EAGER (mode)', 'AssetInitializer.cs:410-412',
      'EAGER (named static rows)', 'STATIC_SHEET_NAMES NPC_Head_Boss_* rows; e17/f29 append the tail');
    push('Extra', enumIds('Extra_').map(i => 'Extra_' + i),
      'EAGER (mode)', 'AssetInitializer.cs:363-366',
      'EAGER (named static rows)', 'STATIC_SHEET_NAMES Extra_* rows');
    push('GlowMask', enumIds('Glow_').map(i => 'Glow_' + i),
      'EAGER (mode)', 'AssetInitializer.cs:352-355',
      'EAGER (GLOW_SHEET_NAMES)', 'assets.rs GLOW_SHEET_NAMES');
    push('Cloud', Array.from({length:42},(_,i)=>'Cloud_'+i),
      'EAGER (mode)', 'AssetInitializer.cs:459-462',
      'EAGER (0..42)', 'assets.rs:1414-1419');
    push('TreeTop/Branch', Array.from({length:32},(_,i)=>['Tree_Tops_'+i,'Tree_Branches_'+i]).flat(),
      'EAGER (mode)', 'AssetInitializer.cs:524-531',
      'EAGER (0..32)', 'assets.rs:1404-1411');
    push('Liquid+LiquidSlope', Array.from({length:15},(_,i)=>['Liquid_'+i,'LiquidSlope_'+i]).flat(),
      'EAGER (mode)', 'AssetInitializer.cs:467-471',
      'EAGER (0..15)', 'load_static_sheets liquid arm');
    push('Misc/water', Array.from({length:15},(_,i)=>'Misc/water_'+i),
      'EAGER (LiquidRenderer content)', 'LiquidRenderer.cs LoadContent',
      'EAGER (0..15)', 'assets.rs:1423-1428');
    push('Waterfall', Array.from({length:28},(_,i)=>'Waterfall_'+i),
      'EAGER (waterfallManager.LoadContent)', 'AssetInitializer.cs:472',
      'EAGER (0..28)', 'load_static_sheets waterfall arm');
    push('Flame', Array.from({length:18},(_,i)=>'Flame_'+i),
      'EAGER (mode)', 'AssetInitializer.cs:382-385',
      'EAGER (0..18)', 'load_static_sheets flame arm');
    fam.push({ family: 'Fonts', sheets: 1, missing: [],
      areaMpx: +((768 + PAD) * (8 + PAD) / 1e6).toFixed(3),
      cs: 'EAGER (mode, DynamicSpriteFont x5)', csRef: 'AssetInitializer.cs:87-94',
      port: 'EAGER (boot font atlas array)', portRef: 'font_atlas.rs / load_fonts' });
    families = fam;
    save();
    for (const f of fam) console.log('family ' + f.family + ': ' + f.sheets +
      ' sheets ' + f.areaMpx + ' Mpx | C# ' + f.cs + ' | port ' + f.port);
  }

  // ---- B. the NPC corpus alone (every NPC_<id> file — what an
  // all-NPC loader would pack; the port packs its named subset today).
  const npcNames = enumIds('NPC_').map(i => 'NPC_' + i);
  const npcSheets = sheetsOf(npcNames);
  if (!npcCorpus) {
    npcCorpus = { files: npcSheets.length, missing: npcSheets.missing,
      areaMpx: +(areaOf(npcSheets) / 1e6).toFixed(2),
      areaBound: Math.ceil(areaOf(npcSheets) / (MAX_WIDTH - PAD)) + PAD };
    save();
  }
  measure('npc-corpus-alone', npcSheets,
    'every NPC_<id>.xnb file (the load-everything upper bound)');

  // ---- C. the merged-407 base (section 7's exact input) + the FINAL
  // re-extraction (424 names) delta.
  const base = buildSheetList({ font: true });
  const union339 = [...new Set(Object.values(pushed339).flat())];
  let merged = addNew(base, union339);
  merged = addNew(merged, D40_IDS.map(id => 'NPC_' + id));
  merged = addNew(merged, D35_DEFERRED.map(id => 'NPC_' + id));
  measure('merged-407-base', merged,
    'font + pushed(339) + d40(56) + d35deferred(12) - the section-7 base, re-witnessed');
  const union424 = [...new Set(Object.values(pushed).flat())];
  const merged424 = addNew(merged, union424);
  measure('merged-final-424', merged424,
    'the FINAL re-extraction (37 branches / 424 names, +85 new) on the 407 base - d125 NOT in it (local tip ahead of origin; +NPC_636/661 measured separately)');
  const d125 = addNew(merged424, ['NPC_636', 'NPC_661']);
  measure('merged-final-424+d125', d125,
    'd125b sheets tip 2dfe026d (local, unpushed): +NPC_636/NPC_661');
  // d125's draw phase also packs Extra_188 (the halo ring) and DEFERS the
  // five tall overlay strips Extra_157/158/159/160/187 to the repack wave
  // (its research/125 section 2) - the deferred five are a worst-case add-on.
  const d125full = addNew(merged424, ['NPC_636', 'NPC_661', 'Extra_188']);
  measure('merged-final-424+d125-full', d125full,
    'd125b full sheets+draw tip: +NPC_636/NPC_661/Extra_188');
  const d125def = addNew(d125full, ['Extra_157','Extra_158','Extra_159','Extra_160','Extra_187']);
  measure('merged-final-424+d125-full+deferred5', d125def,
    'd125 deferred overlay strips admitted by the repack (worst case)');

  // ---- D. the realistic per-world NPC working sets.
  const netNames = nets => [...new Set(nets)].map(n => 'NPC_' + n);
  const townAmb = netNames([...census.townNets, ...Object.values(census.ambientNets)]);
  const longLived = netNames([...census.townNets, ...Object.values(census.spawnNets),
    ...Object.values(census.moonNets), ...census.drawExtraNets]);
  const bestiaryAll = netNames([...census.entryNets, ...census.texOverrideNets,
    ...Object.values(census.moonNets), ...census.drawExtraNets]);
  measure('merged+town-ambient', addNew(merged, townAmb),
    'fresh world: 39 townNPC=true ids + 51 ambient spawner sheets (the port roster census)');
  measure('merged+long-lived', addNew(merged, longLived),
    'long-lived world: town + 141 spawn-site sheets + 31 moon ladders + 10 cross-draw ids');
  measure('merged+bestiary-all', addNew(merged, bestiaryAll),
    'bestiary opened: every ENTRIES net id hovers -> SetFrameSize -> LoadNPC (UnlockableNPCEntryIcon.cs:185)');

  // ---- E. the lazy-union form: pull ALL NPC_ sheets out of the union
  // input, then append the per-world NPC sets as own pages.
  const noNpc = merged.filter(s => !/^NPC_\\d+$/.test(s.name));
  measure('lazy-union-no-npc', noNpc,
    'the 407 base minus every NPC_<id> sheet (the section-10 lazy contract: NPC sheets leave the union input)');
  const npcInBase = merged.length - noNpc.length;
  const ownPage = (label, names, note) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const fresh = sheetsOf(names.filter(n => !noNpc.some(s => s.name === n)));
    const pages = fresh.length ? boundaryNPages(fresh, 8192) : [];
    const row = { scenario: label, note, newSheets: fresh.length,
      missing: fresh.missing, areaMpx: +(areaOf(fresh) / 1e6).toFixed(2),
      ownPages: pages, totalMainPages: null };
    console.log(label + ': +' + fresh.length + ' NPC sheets (' + row.areaMpx +
      ' Mpx) on ' + pages.length + ' own page(s): ' +
      pages.map(p => p.height + '(' + p.n + ')').join(' / '));
    results.push(row); save();
  };
  ownPage('ownpage+town-ambient', townAmb, 'fresh-world NPC set on its own page(s) after the lazy union');
  ownPage('ownpage+long-lived', longLived, 'long-lived NPC set on its own page(s)');
  ownPage('ownpage+bestiary-all', bestiaryAll, 'the bestiary-opened NPC set on its own page(s)');
  ownPage('ownpage+npc-corpus', npcNames, 'the ALL-NPC upper bound on its own page(s) (NOT the contract)');

  // ---- F. the realistic total: lazy union pages + seed-42 bg page +
  // the NPC own pages. The bg page numbers come from the committed
  // _d35_bg_inventory.json pageform rows (measured, section 7.3).
  const lazyPages = boundaryNPages(noNpc, 8192);
  const bg42 = bgInv.scenarios.find(r => r.scenario === 'pageform+seed42-full');
  const bgWorst = bgInv.scenarios.find(r => r.scenario === 'pageform+worst-world');
  const stack = (label, npcLabel, bgRow) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const npcRow = results.find(r => r.scenario === npcLabel);
    const row = { scenario: label,
      unionPages: lazyPages, bgPages: bgRow ? bgRow.bgPages : [],
      npcPages: npcRow.ownPages,
      totalPages: lazyPages.length + (bgRow ? bgRow.bgPages.length : 0) + npcRow.ownPages.length };
    console.log(label + ': union ' + lazyPages.map(p=>p.height).join('/') +
      ' + bg ' + (bgRow ? bgRow.bgPages.map(p=>p.height).join('/') : '-') +
      ' + npc ' + npcRow.ownPages.map(p=>p.height).join('/') +
      ' = ' + row.totalPages + ' pages');
    results.push(row); save();
  };
  stack('TOTAL-fresh-seed42', 'ownpage+town-ambient', bg42);
  stack('TOTAL-longlived-seed42', 'ownpage+long-lived', bg42);
  stack('TOTAL-bestiary-seed42', 'ownpage+bestiary-all', bg42);
  stack('TOTAL-longlived-worstbg', 'ownpage+long-lived', bgWorst);

  // ---- G. the CURRENT port form stacked with the section-7.3 bg pages:
  // the merged union keeps its NPC sheets EAGER (pages 0/1 = 8192/1298),
  // the bg pages append after. This is the §8.4 contract WITHOUT the NPC
  // lazy extension — the baseline the section-10 form improves on.
  const stackEager = (label, bgRow) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const unionPages = [{ height: 8192 }, { height: 1298 }]; // §7.3 measured
    const row = { scenario: label, unionPages, bgPages: bgRow.bgPages,
      npcPages: [], totalPages: unionPages.length + bgRow.bgPages.length };
    console.log(label + ': eager union 8192/1298 + bg ' +
      bgRow.bgPages.map(p=>p.height).join('/') + ' = ' + row.totalPages + ' pages');
    results.push(row); save();
  };
  stackEager('TOTAL-eager-seed42', bg42);
  stackEager('TOTAL-eager-worstbg', bgWorst);
  // the FINAL 424-union form (NPCs still eager) + the same bg pages
  const finalPages = boundaryNPages(merged424, 8192);
  const stackFinal = (label, bgRow) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const row = { scenario: label, unionPages: finalPages, bgPages: bgRow.bgPages,
      npcPages: [], totalPages: finalPages.length + bgRow.bgPages.length };
    console.log(label + ': final-424 union ' + finalPages.map(p=>p.height).join('/') +
      ' + bg ' + bgRow.bgPages.map(p=>p.height).join('/') + ' = ' + row.totalPages + ' pages');
    results.push(row); save();
  };
  stackFinal('TOTAL-final424-eager-seed42', bg42);
  stackFinal('TOTAL-final424-eager-worstbg', bgWorst);

  // ---- H. the FINAL lazy form: the 424-union minus NPCs, the per-world
  // NPC sets appended as own pages, then the bg pages. The merger's
  // realistic-budget witness on the CURRENT branch tips.
  const noNpc424 = merged424.filter(s => !/^NPC_\\d+$/.test(s.name));
  measure('lazy-union-424-no-npc', noNpc424,
    'the FINAL 424 base minus every NPC_<id> sheet (the section-10 lazy contract on the current tips)');
  const lazy424Pages = boundaryNPages(noNpc424, 8192);
  const stackLazy424 = (label, npcLabel, bgRow) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const freshNames = { 'ownpage+town-ambient': townAmb,
      'ownpage+long-lived': longLived, 'ownpage+bestiary-all': bestiaryAll }[npcLabel];
    const fresh = sheetsOf(freshNames.filter(n => !noNpc424.some(s => s.name === n)));
    const npcPages = fresh.length ? boundaryNPages(fresh, 8192) : [];
    const row = { scenario: label, unionPages: lazy424Pages,
      bgPages: bgRow ? bgRow.bgPages : [], npcPages,
      newNpcSheets: fresh.length,
      npcAreaMpx: +(areaOf(fresh) / 1e6).toFixed(2),
      totalPages: lazy424Pages.length + (bgRow ? bgRow.bgPages.length : 0) + npcPages.length };
    console.log(label + ': lazy-424 union ' + lazy424Pages.map(p=>p.height).join('/') +
      ' + bg ' + (bgRow ? bgRow.bgPages.map(p=>p.height).join('/') : '-') +
      ' + npc ' + npcPages.map(p=>p.height+'('+p.n+')').join('/') +
      ' = ' + row.totalPages + ' pages');
    results.push(row); save();
  };
  stackLazy424('TOTAL-lazy424-fresh-seed42', 'ownpage+town-ambient', bg42);
  stackLazy424('TOTAL-lazy424-longlived-seed42', 'ownpage+long-lived', bg42);
  stackLazy424('TOTAL-lazy424-bestiary-seed42', 'ownpage+bestiary-all', bg42);
  stackLazy424('TOTAL-lazy424-longlived-worstbg', 'ownpage+long-lived', bgWorst);
  stackLazy424('TOTAL-lazy424-bestiary-worstbg', 'ownpage+bestiary-all', bgWorst);

  save();
  return results;
`);
const results = fn(require, __dirname, pushed, pushed339, census, bgInv, D40_IDS, D35_DEFERRED);
console.log('wrote tools/_d35_npc_lazy.json (' + results.length + ' scenarios)');
