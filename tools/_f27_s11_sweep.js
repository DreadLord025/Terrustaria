// W4-f27 slice 11 strip-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f27_s11_replay.py's sweep: the 15-kind x 6-machine
// x 40-tick in-sheet check over line-for-line C# transcriptions
// (NPC.cs:70879-70951 armed zombies 435/436, :68616->:68647-69655
// bartender 550, :67452-67492 goblins 552-554, :67797-67837 bombers
// 555-557, :67838-67873 wyverns 558-560, :67756-67796 javelineers
// 561-563). Math.fround pins the f32 products the port uses (the C#
// counter is a double fed by float products, NPC.cs:6485 — every
// accumulator this wave is an integer step, exact in both; the wyvern
// rotation is the one transcendental, the sweep checks rows only).
// Run later with: node tools/_f27_s11_sweep.js
"use strict";

const fr = Math.fround;

function armedZombie(s, vx, vy, direction, ai2) {
  // NPC.cs:70879-70951 (case 430..436/591; 435/436 this wave).
  if (ai2 > 0) { // :70887-70905
    if (ai2 < 5) s.row = 3;
    else if (ai2 < 10) s.row = 4;
    else if (ai2 < 15) s.row = 5;
    else s.row = 6;
    return;
  }
  if (vy === 0) { // :70907-70917
    if (direction === 1) s.sprite = 1;
    if (direction === -1) s.sprite = -1;
  }
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 2; return; // :70918-70923
  }
  if (vx === 0) { s.counter = 0; s.row = 0; return; } // :70924-70929
  s.counter += Math.abs(vx); // :70930-70950
  if (s.counter < 8) s.row = 0;
  else if (s.counter < 16) s.row = 1;
  else if (s.counter < 24) s.row = 2;
  else if (s.counter < 32) s.row = 1;
  else s.counter = 0; // the row NOT written (holds)
}

function bartender(s, vx, vy, direction) {
  // NPC.cs:68616 -> :68647-69655 (the town walk, num57 = 9, count 25).
  if (vy === 0) {
    if (direction === 1) s.sprite = 1; // :68649-68656
    if (direction === -1) s.sprite = -1;
    if (vx === 0) { s.row = 0; s.counter = 0; } // :69515-69519
    else {
      if (s.row < 2) s.row = 2; // :69572-69575 (num109 = row 2)
      s.counter += fr(Math.abs(vx) * 2); // :69552-69553
      s.counter += 1;
      if (s.counter > 6) { s.row += 1; s.counter = 0; } // :69576-69580
      if (s.row >= 25 - 9) s.row = 2; // :69581-69584
    }
  } else { s.counter = 0; s.row = 1; } // :69642-69646
}

function goblin(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67452-67492 (case 552/553/554).
  if (ai0 > 0) { // :67455-67469
    s.sprite = direction;
    if (s.row < 10 || s.row > 13 || ai1 === 29) { s.row = 10; s.counter = 0; }
    s.counter += 1;
    if (s.counter >= 6 && s.row < 13) { s.counter = 0; s.row += 1; }
    return;
  }
  if (vy === 0) s.sprite = direction; // :67470-67473
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 8; return; // :67474-67479
  }
  if (vx === 0) { s.counter = 0; s.row = 9; return; } // :67480-67485
  s.counter += Math.abs(vx); // :67486-67491
  if (s.counter >= 64 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8);
}

function bomber(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67797-67837 (case 555/556/557).
  if (ai1 > 0 && ai0 > 0) { // :67800-67814
    s.sprite = direction;
    if (s.row < 10 || s.row > 16 || ai1 === 41) { s.row = 10; s.counter = 0; }
    s.counter += 1;
    if (s.counter >= 6 && s.row < 16) { s.counter = 0; s.row += 1; }
    return;
  }
  if (vy === 0) s.sprite = direction; // :67815-67818
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 1; return; // :67819-67824
  }
  if (vx === 0) { s.counter = 0; s.row = 0; return; } // :67825-67830
  s.counter += Math.abs(vx); // :67831-67836
  if (s.counter >= 64 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8) + 2;
}

function wyvern(s, vx, vy, ai0) {
  // NPC.cs:67838-67873 (case 558/559/560).
  if (vx > 0) s.sprite = 1; // :67842-67849
  if (vx < 0) s.sprite = -1;
  let num51 = fr(Math.atan2(vy, vx)); // :67850 (Utils.cs:1655)
  if (vx < 0) num51 = fr(num51 + fr(Math.PI)); // :67851-67854 (the REAL PI)
  if (ai0 !== 2) num51 = fr(vx * fr(0.1)); // :67855-67858
  s.rotation = num51; // :67859
  if (ai0 === 2) { s.row = 4; } // :67860-67863
  else {
    s.counter += 1; // :67864-67872
    if (s.counter >= 6) {
      s.row += 1; s.counter = 0;
      if (s.row >= 4) s.row = 0;
    }
  }
}

function javelin(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67756-67796 (case 561/562/563) — the goblin shape.
  if (ai1 > 0 && ai0 > 0) { // :67759-67773
    s.sprite = direction;
    if (s.row < 10 || s.row > 13 || ai1 === 89) { s.row = 10; s.counter = 0; }
    s.counter += 1;
    if (s.counter >= 6 && s.row < 13) { s.counter = 0; s.row += 1; }
    return;
  }
  if (vy === 0) s.sprite = direction; // :67774-67777
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 8; return; // :67778-67783
  }
  if (vx === 0) { s.counter = 0; s.row = 9; return; } // :67784-67789
  s.counter += Math.abs(vx); // :67790-67795
  if (s.counter >= 64 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8);
}

const COUNTS = {
  armedZombie: 7, bartender: 25, goblin: 14,
  bomber: 17, wyvern: 5, javelin: 14,
};

let violations = 0;
let combos = 0;
for (const vx of [-4, -1, 0, 1, 4]) {
  for (const vy of [-3, 0, 3]) {
    for (const direction of [-1, 0, 1]) {
      for (const ai0 of [0, 1, 2]) {
        for (const ai1 of [0, 1, 29, 41, 89]) {
          for (const ai2 of [0, 3, 7, 12, 20]) {
            combos += 1;
            const machines = [
              ["armedZombie", (s) => armedZombie(s, vx, vy, direction, ai2)],
              ["bartender", (s) => bartender(s, vx, vy, direction)],
              ["goblin", (s) => goblin(s, vx, vy, direction, ai0, ai1)],
              ["bomber", (s) => bomber(s, vx, vy, direction, ai0, ai1)],
              ["wyvern", (s) => wyvern(s, vx, vy, ai0)],
              ["javelin", (s) => javelin(s, vx, vy, direction, ai0, ai1)],
            ];
            for (const [name, step] of machines) {
              const s = { row: 0, counter: 0, sprite: 0, rotation: 0 };
              for (let tick = 0; tick < 40; tick++) {
                step(s);
                if (s.row >= COUNTS[name]) {
                  violations += 1;
                  console.log(`VIOLATION ${name} tick ${tick} row ${s.row} ` +
                    `(vx ${vx}, vy ${vy}, dir ${direction}, ai0 ${ai0}, ai1 ${ai1}, ai2 ${ai2})`);
                }
              }
            }
          }
        }
      }
    }
  }
}
console.log(`sweep: ${combos} combos x 6 machines x 40 ticks, ${violations} violations`);
if (violations !== 0) process.exit(1);
console.log("the sweep agrees with the C# transcriptions (0 violations)");
