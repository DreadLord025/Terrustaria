// W4-d38: reconcile the c4 quote (7757 static / 8143 full gore+FONT) and the
// g72 quote (7530 static+gore) against the same harness. Font adds 768x8.
'use strict';
const fs = require('fs');
const path = require('path');
const ROOT = path.join(__dirname, '..');
let src = fs.readFileSync(path.join(ROOT, 'tools', 'atlas_sim.js'), 'utf8');
const a = src.indexOf('// The L-8b branch');
const b = src.indexOf('// Projectile kinds:');
src = src.slice(0, a) + 'const L8B_ADDITIONS=[],L8B_REMOVED=[],L8B_STATIC=[];\n' +
  src.slice(b, src.indexOf('function main()'));
const m = { exports: {} };
new Function('module', 'exports', 'require', '__dirname', src +
  '\nmodule.exports={buildSheetList,skylinePack,STATIC_NAMES};')(m, m.exports, require, path.join(ROOT, 'tools'));
const { buildSheetList, skylinePack, STATIC_NAMES } = m.exports;
const D38 = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,297,298,299,
  300,301,302,303,304,316,317,318,322,323,324,328,333,334,335,336,337].map(i => 'NPC_' + i);
const C4 = [224,509,581,587].map(i => 'NPC_' + i);
const withoutD38 = STATIC_NAMES.filter(n => !D38.includes(n));
for (const [label, names] of [
  ['main + gore + font (c4 quote base 8119)', withoutD38],
  ['c4 + gore + font (c4 quote 8143)', withoutD38.concat(C4)],
]) {
  const sheets = buildSheetList({ gores: true, font: true, staticNames: names });
  const sky = skylinePack(sheets, 'waste');
  console.log(`${label}: skyline ${sky.height}`);
}
