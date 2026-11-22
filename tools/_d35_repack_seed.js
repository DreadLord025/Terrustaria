// Seed _d35_repack_measure.json from the captured stdout of the SIGTERM'd
// full run (sh_1fc07a121001I3nZFIWWNyx3oc), so the resume run only re-packs
// the two font scenarios that never finished. Numbers are transcribed
// verbatim from that run's console output.
const fs = require('fs');
const path = require('path');
const rows = [
  { scenario: 'base', sheets: 9518, areaMpx: 64.53, areaBound: 7881, skyline: 8188, maxrects: 7988, skyOver: 0, mrOver: 0 },
  { scenario: '+pushed(339)', sheets: 9835, areaMpx: 72.34, areaBound: 8835, skyline: 9382, maxrects: 9998, skyOver: 1190, mrOver: 1806,
    spillMaxrects: { page0: 8192, page1: 700, n0: 2804, n1: 7031 }, boundarySkyline: { page0: 8190, page1: 1232, n0: 967, n1: 8868 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 317 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 317 } },
  { scenario: '+d40forecast(56)', sheets: 9891, areaMpx: 74.05, areaBound: 9044, skyline: 9443, maxrects: 9158, skyOver: 1251, mrOver: 966,
    spillMaxrects: { page0: 8192, page1: 948, n0: 2414, n1: 7477 }, boundarySkyline: { page0: 8163, page1: 1267, n0: 957, n1: 8934 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 373 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 373 } },
  { scenario: '+d35deferred(12)', sheets: 9903, areaMpx: 74.71, areaBound: 9124, skyline: 9478, maxrects: 9254, skyOver: 1286, mrOver: 1062,
    spillMaxrects: { page0: 8192, page1: 1018, n0: 2067, n1: 7836 }, boundarySkyline: { page0: 8192, page1: 1298, n0: 963, n1: 8940 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 385 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 385 } },
  { scenario: '+L8b(0)', sheets: 9903, areaMpx: 74.71, areaBound: 9124, skyline: 9478, maxrects: 9254, skyOver: 1286, mrOver: 1062,
    spillMaxrects: { page0: 8192, page1: 1018, n0: 2067, n1: 7836 }, boundarySkyline: { page0: 8192, page1: 1298, n0: 963, n1: 8940 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 385 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 385 } },
  { scenario: 'font base', sheets: 9519, areaMpx: 64.53, areaBound: 7882, skyline: 8188, maxrects: 7988, skyOver: 0, mrOver: 0 },
  { scenario: 'font +pushed(339)', sheets: 9836, areaMpx: 72.35, areaBound: 8836, skyline: 9386, maxrects: 9998, skyOver: 1194, mrOver: 1806,
    spillMaxrects: { page0: 8192, page1: 700, n0: 2805, n1: 7031 }, boundarySkyline: { page0: 8190, page1: 1232, n0: 967, n1: 8869 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 317 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 317 } },
  { scenario: 'font +d40forecast(56)', sheets: 9892, areaMpx: 74.05, areaBound: 9044, skyline: 9443, maxrects: 9158, skyOver: 1251, mrOver: 966,
    spillMaxrects: { page0: 8192, page1: 948, n0: 2414, n1: 7478 }, boundarySkyline: { page0: 8163, page1: 1268, n0: 957, n1: 8935 },
    incrementalSkyline: { page0: 8188, page1: 1924, n1: 373 }, incrementalMaxrects: { page0: 7988, page1: 1924, n1: 373 } },
];
for (const r of rows) r.missing = [];
fs.writeFileSync(path.join(__dirname, '_d35_repack_measure.json'), JSON.stringify(rows, null, 1));
console.log('seeded ' + rows.length + ' scenarios');
