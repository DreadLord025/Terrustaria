// W4-f26 slice 7f strip-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f26_7f_replay.py's sweep: the 5-kind x 27-combo x
// 40-tick in-sheet check over line-for-line C# transcriptions
// (NPC.cs:73112-73145 wall scorpion 531, :70067-70103 beast 532,
// :68615->:68647-69655 tax collector 534, :70818->:70828-70845 spiked
// slime 535, :76344->:76347-76390 bride 536). Math.fround pins the f32
// products the port uses (the C# counter is a double fed by float
// products, NPC.cs:6485 — every combo sits far from a step boundary).
// Run later with: node tools/_f26_7f_sweep.js
"use strict";

const fr = Math.fround;

function wallScorpion(s, vx, vy) {
  // NPC.cs:73112-73145 (case 165/237/238/240/531; only 531 here).
  const num194 = fr(0.4); // :73118-73122 (the 531 override)
  s.counter += fr((Math.abs(vx) + Math.abs(vy)) * num194); // :73123
  if (s.counter < 6) s.row = 0; // :73124-73127
  else if (s.counter < 12) s.row = 1; // :73128-73131
  else if (s.counter < 18) s.row = 2; // :73132-73135
  else if (s.counter < 24) s.row = 3; // :73136-73139
  else s.counter = 0; // :73140-73143 — the row NOT written (holds 3)
}

function beast(s, vx, vy, count) {
  // NPC.cs:70067-70103 (case 508/532/580/582; only 532 here — the
  // verbatim 7c antlion machine on the 8-row sheet).
  if (vy !== 0) { s.row = 1; return; } // :70071-70075 counter UNTOUCHED
  if (vx === 0) { s.row = 0; return; } // :70076-70080 counter UNTOUCHED
  if (s.direction < 0 && vx < 0) s.sprite = -1; // :70081-70084
  if (s.direction > 0 && vx > 0) s.sprite = 1; // :70085-70088
  if (s.row < 2) s.row = 2; // :70089-70092
  s.counter += 1 + Math.abs(vx) / 2; // :70093
  if (s.counter > 6) { s.row += 1; s.counter = 0; } // :70094-70098
  if (s.row >= count) s.row = 2; // :70099-70102
}

function taxCollector(s, vx, vy, count) {
  // NPC.cs:68615 -> the humanoid fighter walk with the 534 num108=12
  // override (:69528-69531). num57=0, num109=row 2, idle row 0, air
  // row 1 (not in the :69647 pin list). The ai[0] pose zones are the
  // documented skip.
  if (vy === 0) { // :68647
    if (s.direction === 1) s.sprite = 1; // :68649-68652
    if (s.direction === -1) s.sprite = -1; // :68653-68656
    if (vx === 0) { s.row = 0; s.counter = 0; } // :69515-69519
    else {
      if (s.row < 2) s.row = 2; // :69572-69575 (num109 = 2)
      s.counter += fr(Math.abs(vx) * fr(2)); // :69552
      s.counter += 1; // :69553
      if (s.counter > 12) { s.row += 1; s.counter = 0; } // :69576 num108=12
      if (s.row >= count) s.row = 2; // :69581-69584 (num57 = 0)
    }
  } else { s.counter = 0; s.row = 1; } // :69642-69646
}

function spikedSlime(s, vx, vy, count) {
  // NPC.cs:70818 -> :70828-70845 (the slime strip group; the 302/304
  // sprite snap is the town slimes' — 535 writes no flip).
  let num2 = 0; // :66696-66704 (aiAction 0 in the sweep)
  if (vy < 0) num2 = 2;
  else if (vy > 0) num2 = 3;
  else if (vx !== 0) num2 = 1;
  s.counter += 1; // :70828
  if (num2 > 0) s.counter += 1; // :70829-70832
  if (num2 === 4) s.counter += 1; // :70833-70836
  if (s.counter >= 8) { s.row += 1; s.counter = 0; } // :70837-70841
  if (s.row >= count) s.row = 0; // :70842-70845
}

function bride(s, vx, vy) {
  // NPC.cs:76344 -> :76347-76390 (the zombie walker group — the
  // verbatim wave-A zombie_walk_frame).
  if (vy === 0) { // :76347
    if (s.direction === 1) s.sprite = 1; // :76349-76352
    if (s.direction === -1) s.sprite = -1; // :76353-76356
  }
  if (vy !== 0 || (s.direction === -1 && vx > 0) ||
      (s.direction === 1 && vx < 0)) { // :76358
    s.counter = 0; s.row = 2; return; // :76360-76362
  }
  if (vx === 0) { s.counter = 0; s.row = 0; return; } // :76364-76368
  s.counter += Math.abs(vx); // :76370 NO +1
  if (s.counter < 8) s.row = 0; // :76371-76374
  else if (s.counter < 16) s.row = 1; // :76375-76378
  else if (s.counter < 24) s.row = 2; // :76379-76382
  else if (s.counter < 32) s.row = 1; // :76383-76386
  else s.counter = 0; // :76387-76390 — the row NOT written (holds 1)
}

const KINDS = [
  [531, wallScorpion, 4],
  [532, beast, 8],
  [534, taxCollector, 8],
  [535, spikedSlime, 2],
  [536, bride, 3],
];

let combos = 0;
for (const [id, fn, count] of KINDS) {
  for (const vx of [0, 1, 3]) {
    for (const vy of [0, -2, 2]) {
      for (const direction of [1, -1, 0]) {
        const s = { row: 0, counter: 0, sprite: 0, direction };
        for (let t = 0; t < 40; t++) {
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
console.log("f26 slice 7f: the 135-combo x 40-tick strip sweep PASS");
