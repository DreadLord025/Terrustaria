// W4-f26 slice 7e strip-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f26_7e_replay.py's sweep: the 5-kind x 27-combo x
// 10-tick in-sheet check over line-for-line C# transcriptions
// (NPC.cs:69908-69945 ghoul 526/527, :69946-69977 lamia 528/529,
// :73083-73111 scorpion 530). Math.fround pins the f32 products the
// port uses (the C# counter is a double fed by float products,
// NPC.cs:6485 — every combo sits far from a step boundary).
// Run later with: node tools/_f26_7e_sweep.js
"use strict";

const fr = Math.fround;

function ghoul(s, vx, vy, count) {
  // NPC.cs:69908-69945 (the verbatim 7d machine; only 526/527 here).
  if (vy === 0) {
    if (s.direction !== 0) s.sprite = s.direction; // :69914-69917
    if (vx === 0) { s.row = 0; s.counter = 0; return; } // :69918-69923
    if (s.row <= 1) s.row = 2; // :69924-69927
    s.counter += Math.abs(vx); // :69928
    s.counter += 1; // :69929
    if (s.counter > 9) { s.row += 1; s.counter = 0; } // :69930-69934
    if (s.row >= count) s.row = 2; // :69935-69938
  } else { s.row = 1; s.counter = 0; } // :69940-69944
}

function lamia(s, vx, vy, count) {
  // NPC.cs:69946-69977 — NO lift-to-2, wrap count-1 -> 1, air count-1.
  if (vy === 0) {
    if (s.direction !== 0) s.sprite = s.direction; // :69950-69953
    if (vx === 0) { s.row = 0; s.counter = 0; return; } // :69954-69959
    s.counter += Math.abs(vx); // :69960
    s.counter += 1; // :69961
    if (s.counter > 9) { s.row += 1; s.counter = 0; } // :69962-69966
    if (s.row >= count - 1) s.row = 1; // :69967-69970
  } else { s.row = count - 1; s.counter = 0; } // :69972-69976
}

function scorpion(s, vx, vy) {
  // NPC.cs:73083-73111 (case 164/236/239/530; only 530 here).
  if (vy !== 0) { // :73087
    s.counter = 0; // :73089 FIRST
    s.row = vy < 0 ? 4 : 0; // :73090-73097
    return; // :73098 break, sprite UNTOUCHED
  }
  s.sprite = s.direction; // :73100 UNCONDITIONAL
  s.counter += fr(Math.abs(vx) * fr(1.1)); // :73101 |vx|*1.1 NO +1
  while (s.counter >= 6) { // :73102 WHILE multi-step
    s.counter -= 6; // :73104
    s.row += 1; // :73105
    if (s.row > 3) s.row = 0; // :73106-73109 STRICT >
  }
}

const KINDS = [
  [526, ghoul, 8], [527, ghoul, 8],
  [528, lamia, 9], [529, lamia, 9],
  [530, scorpion, 5],
];

let combos = 0;
for (const [id, fn, count] of KINDS) {
  for (const vx of [0, 1, 3]) {
    for (const vy of [0, -2, 2]) {
      for (const direction of [1, -1, 0]) {
        const s = { row: 0, counter: 0, sprite: 0, direction };
        for (let t = 0; t < 10; t++) {
          fn(s, vx, vy, count);
          if (!(s.row >= 0 && s.row < count)) {
            throw new Error(`type ${id} combo ${combos} row ${s.row} escapes the ${count}-row sheet`);
          }
        }
        combos++;
      }
    }
  }
}
if (combos !== 135) throw new Error(`expected 135 combos, got ${combos}`);
console.log("f26 slice 7e: the 135-combo x 10-tick strip sweep PASS");
