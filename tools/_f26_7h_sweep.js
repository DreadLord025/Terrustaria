// W4-f26 slice 7h strip-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f26_7h_replay.py's sweep: the 8-kind x 27-combo x
// 40-tick in-sheet check + the Hallow Boss 108-phase check over
// line-for-line C# transcriptions (NPC.cs:67022-67024 the HallowBoss
// phase pin 636 — AI_120_HallowBoss_IsInPhase2 :45814-45821 = ai[3]
// == 1 || ai[3] == 3; :76458-76512 the gem squirrels 639-645, the
// VERBATIM f27 slice-10 squirrel machine). Math.fround pins the f32
// products the port uses (the C# counter is a double fed by float
// products, NPC.cs:6485 — the pinned cadences use exact half/integer
// feeds: vx 2 gives 2.0f*0.5f == 1.0f exactly, vx 1 gives 1.5 exact).
// Run later with: node tools/_f26_7h_sweep.js
"use strict";

const fr = Math.fround;

function hallowBoss(s, ai3) {
  // NPC.cs:67022-67024 (case 636) — the arm writes ONLY frame.Y.
  const phase2 = ai3 === 1 || ai3 === 3; // :45816-45820 PURE ai[3]
  s.row = phase2 ? 1 : 0; // :67023 num : 0
}

function gemSquirrel(s, vx, vy, count) {
  // NPC.cs:76458-76512 (case 299/538/539/639-645; only 639-645 here).
  // The aiStyle-3 backpedal :76478-76483 is DEAD (aiStyle 7,
  // :17226-17236) — the documented dead branch.
  if (vy === 0) { // :76468
    if (s.direction === 1) s.sprite = 1; // :76470-76473
    if (s.direction === -1) s.sprite = -1; // :76474-76477
    if (vx === 0) { s.row = 0; s.counter = 0; return; } // :76484-76488
    s.counter += fr(Math.abs(vx) * fr(0.5)); // :76490
    s.counter += 1; // :76491 ++
    if (s.counter > 6) { s.row += 1; s.counter = 0; } // :76492-76496
    if (s.row >= count - 1) s.row = 1; // :76497-76500 wrap to ROW 1
  } else if (vy < 0) {
    s.counter = 0; s.row = 2; // :76502-76506
  } else {
    s.counter = 0; s.row = 4; // :76507-76511
  }
}

const KINDS = [[636, "boss", 2]];
for (let id = 639; id <= 645; id++) KINDS.push([id, "squirrel", 6]);

function tick(machine, s, vx, vy, count) {
  if (machine === "boss") hallowBoss(s, 0);
  else gemSquirrel(s, vx, vy, count);
}

function sweep() {
  let combos = 0;
  for (const [id, machine, count] of KINDS) {
    for (const vx of [0, 1, 3]) {
      for (const vy of [0, -2, 2]) {
        for (const direction of [1, -1, 0]) {
          const s = { row: 0, counter: 0, sprite: 0, direction };
          for (let t = 0; t < 40; t++) {
            tick(machine, s, vx, vy, count);
            if (!(s.row >= 0 && s.row < count)) {
              throw new Error(`kind ${id} combo ${combos} row ${s.row} escapes ${count}`);
            }
          }
          combos++;
        }
      }
    }
  }
  if (combos !== 216) throw new Error(`combos ${combos} != 216`);
  // The Hallow Boss phase sweep: ai3 -1/0/1/2/3/4 x 18 combos.
  let phases = 0;
  for (const ai3 of [-1, 0, 1, 2, 3, 4]) {
    for (const vx of [0, 2, -2]) {
      for (const vy of [0, -2, 2]) {
        for (const direction of [1, -1]) {
          const s = { row: 1, counter: 5, sprite: -1, direction };
          hallowBoss(s, ai3);
          if (s.row !== (ai3 === 1 || ai3 === 3 ? 1 : 0)) {
            throw new Error(`phase ${ai3} row ${s.row}`);
          }
          if (s.counter !== 5 || s.sprite !== -1) {
            throw new Error(`phase side effects ${s.counter}/${s.sprite}`);
          }
          phases++;
        }
      }
    }
  }
  if (phases !== 108) throw new Error(`phases ${phases} != 108`);
}

sweep();
console.log("f26 slice 7h: the 216-combo sweep + the 108-phase sweep PASS");
