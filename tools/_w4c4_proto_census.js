// W4-c4 proto-id census: extract every pushed tip's to_u16/from_u16 arms
// (git-show, read-only) and report the union id map + free-port candidates.
// Usage: node tools/_w4c4_proto_census.js [id...]   (ids = candidates to check)
// Dedupes by data.rs blob hash so 350+ branches cost ~dozens of git-shows.
const { execSync } = require('child_process');
const sh = (c) => execSync(c, { maxBuffer: 64 * 1024 * 1024 }).toString();

const tips = sh('git for-each-ref --format="%(refname:short) %(objectname:short)" refs/remotes/origin/task')
  .trim().split('\n').map((l) => l.split(' '));
const byBlob = new Map(); // blob -> { branches: [], text }
for (const [ref, tip] of tips) {
  let blob;
  try { blob = sh(`git rev-parse "${ref}:crates/tr-app/src/npc/data.rs"`).trim(); }
  catch { continue; } // branch without the file (pre-split tips)
  if (!byBlob.has(blob)) byBlob.set(blob, { branches: [], text: null });
  byBlob.get(blob).branches.push(`${ref}@${tip}`);
}
const armRe = /NpcKind::(\w+)\s*=>\s*(\d+)\s*,/g;
const invRe = /(\d+)\s*=>\s*NpcKind::(\w+)\s*,/g;
function fnBody(text, name) {
  const i = text.indexOf(`fn ${name}(`);
  if (i < 0) return '';
  // to the next `\n    pub fn` / `\n    ///` after the closing brace at fn level
  let depth = 0, j = text.indexOf('{', i);
  for (let k = j; k < text.length; k++) {
    if (text[k] === '{') depth++;
    else if (text[k] === '}') { depth--; if (depth === 0) return text.slice(j, k + 1); }
  }
  return text.slice(j);
}
const union = new Map(); // id -> [{kind, branches}]
let bad = [];
for (const [blob, rec] of byBlob) {
  rec.text = sh(`git cat-file blob ${blob}`);
  const to = fnBody(rec.text, 'to_u16'), from = fnBody(rec.text, 'from_u16');
  const toMap = new Map(), fromMap = new Map();
  for (const m of to.matchAll(armRe)) toMap.set(m[1], +m[2]);
  for (const m of from.matchAll(invRe)) fromMap.set(+m[1], m[2]);
  // bijection check per tip
  for (const [kind, id] of toMap) {
    if (fromMap.get(id) !== kind) bad.push(`${rec.branches[0]}: ${kind}->${id} inverse=${fromMap.get(id)}`);
  }
  if (toMap.size !== fromMap.size) bad.push(`${rec.branches[0]}: arm counts ${toMap.size}/${fromMap.size}`);
  const seenIds = new Map();
  for (const [kind, id] of toMap) {
    if (seenIds.has(id)) bad.push(`${rec.branches[0]}: DUP id ${id} (${seenIds.get(id)} vs ${kind})`);
    seenIds.set(id, kind);
    if (!union.has(id)) union.set(id, []);
    const e = union.get(id).find((x) => x.kind === kind);
    if (e) e.branches.push(...rec.branches); else union.get(id).push({ kind, branches: [...rec.branches] });
  }
}
console.log(`tips=${tips.length} unique data.rs blobs=${byBlob.size} union ids=${union.size}`);
console.log(bad.length ? 'BIJECTION/DUP FAILURES:\n' + bad.join('\n') : 'all tips bijective, zero intra-tip dups');
// cross-branch collisions: same id, different kind
const collisions = [...union.entries()].filter(([, v]) => v.length > 1);
console.log(`cross-branch id collisions: ${collisions.length}`);
for (const [id, v] of collisions.sort((a, b) => a[0] - b[0]))
  console.log(`  ${id}: ${v.map((x) => `${x.kind} (${x.branches[0].split('@')[0].replace('origin/task/', '')}${x.branches.length > 1 ? ` +${x.branches.length - 1}` : ''})`).join(' | ')}`);
// candidate check
for (const arg of process.argv.slice(2)) {
  const id = +arg;
  const v = union.get(id);
  console.log(`candidate ${id}: ${v ? 'USED by ' + v.map((x) => `${x.kind} [${x.branches.join(', ')}]`).join(' | ') : 'FREE on all pushed tips'}`);
}
// free ids in 354..900 (excluding nothing — raw census)
const free = [];
for (let id = 354; id <= 900; id++) if (!union.has(id)) free.push(id);
console.log('free 354-900:', free.join(','));
