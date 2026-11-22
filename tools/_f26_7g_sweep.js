// W4-f26 slice 7g strip-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f26_7g_replay.py's sweep: the 5-kind x 27-combo x
// 40-tick in-sheet check + the golem 24-pose check over line-for-line
// C# transcriptions (NPC.cs:70879-70951 armed zombies 435/436,
// :75259-75313 rock golem 631, :68619/:68621->:68647-69655 maggot
// zombie 632 / spore skeleton 635). Math.fround pins the f32 products
// the port uses (the C# counter is a double fed by float products,
// NPC.cs:6485 — the pinned golem cadence vx 0.8 gives 0.2f+0.8f ==
// 1.0f exactly, every step edge exact in both).
// Run later with: node tools/_f26_7g_sweep.js
"use strict";

const fr = Math.fround;

function armedZombie(s, vx, vy, ai2) {
  // NPC.cs:70879-70951 (case 430..436/591; only 435/436 here — the
  // verbatim d49 high2 machine).
  if (ai2 > 0) { // :70887 — counter/sprite UNTOUCHED
    if (ai2 < 5) s.row = 3; // :70889-70892
    else if (ai2 < 10) s.row = 4; // :70893-70896
    else if (ai2 < 15) s.row = 5; // :70897-70900
    else s.row = 6; // :70901-70904
    return; // :70905
  }
  if (vy === 0) { // :70907
    if (s.direction === 1) s.sprite = 1; // :70909-70912
    if (s.direction === -1) s.sprite = -1; // :70913-70916
  }
  if (vy !== 0 || (s.direction === -1 && vx > 0) ||
      (s.direction === 1 && vx < 0)) { // :70918
    s.counter = 0; s.row = 2; return; // :70920-70922
  }
  if (vx === 0) { s.counter = 0; s.row = 0; return; } // :70924-70928
  s.counter += Math.abs(vx); // :70930 NO +1
  if (s.counter < 8) s.row = 0; // :70931-70934
  else if (s.counter < 16) s.row = 1; // :70935-70938
  else if (s.counter < 24) s.row = 2; // :70939-70942
  else if (s.counter < 32) s.row = 1; // :70943-70946
  else s.counter = 0; // :70947-70950 — the row NOT written (holds 1)
}

function rockGolem(s, vx, vy, ai2, facingP, overlap, canHit, count) {
  // NPC.cs:75259-75313 (case 631). The pose zone's world reads
  // (:75270-75273) arrive as the three resolved booleans.
  if (vy === 0) { // :75264
    s.sprite = s.direction; // :75266 UNCONDITIONAL
    if (ai2 !== 0 && ai2 < 100) { // :75267
      const amount = fr(fr(ai2 - 68) / 32); // :75269
      if (facingP && !overlap && canHit) { // :75273
        s.row = Math.trunc(fr(15 + fr(fr(count - 15) * amount))); // :75275
      } else if (facingP && overlap) { // :75277
        s.row = Math.trunc(fr(10 + fr(5 * amount))); // :75279
      } else {
        s.row = 0; // :75283
      }
      return; // :75285 counter UNTOUCHED
    }
    if (vx === 0) { s.counter = 0; s.row = 0; return; } // :75287-75291
    s.counter += fr(0.2 + Math.abs(vx)); // :75293
    if (s.counter > 8) { s.row += 1; s.counter = 0; } // :75294-75298
    if (s.row >= 8) s.row = 1; // :75299-75302 LITERAL num31 = 8
    else if (s.row < 1) s.row = 1; // :75303-75306
  } else { s.counter = 0; s.row = 0; } // :75308-75312
}

function humanoid(s, vx, vy, num108, count) {
  // NPC.cs:68619/:68621 -> the humanoid fighter walk. 632: num108=12
  // (:69524-69527); 635: the default 6. num57=0, num109=row 2, idle
  // row 0, air row 1. The ai[0] pose zones are the documented skip.
  if (vy === 0) { // :68647
    if (s.direction === 1) s.sprite = 1; // :68649-68652
    if (s.direction === -1) s.sprite = -1; // :68653-68656
    if (vx === 0) { s.row = 0; s.counter = 0; } // :69515-69519
    else {
      if (s.row < 2) s.row = 2; // :69572-69575 (num109 = 2)
      s.counter += fr(Math.abs(vx) * fr(2)); // :69552
      s.counter += 1; // :69553
      if (s.counter > num108) { s.row += 1; s.counter = 0; } // :69576
      if (s.row >= count) s.row = 2; // :69581-69584 (num57 = 0)
    }
  } else { s.counter = 0; s.row = 1; } // :69642-69646
}

const KINDS = [
  [435, "zombie", 7],
  [436, "zombie", 7],
  [631, "golem", 21],
  [632, "humanoid12", 16],
  [635, "humanoid6", 16],
];

function tick(machine, s, vx, vy, count) {
  if (machine === "zombie") armedZombie(s, vx, vy, 0);
  else if (machine === "golem") rockGolem(s, vx, vy, 0, false, false, false, count);
  else if (machine === "humanoid12") humanoid(s, vx, vy, 12, count);
  else humanoid(s, vx, vy, 6, count);
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
  if (combos !== 135) throw new Error(`combos ${combos} != 135`);
  // The golem pose sweep: ai2 68/84/99 x the 8 flag combos.
  let poses = 0;
  for (const ai2 of [68, 84, 99]) {
    for (const f of [false, true]) {
      for (const o of [false, true]) {
        for (const c of [false, true]) {
          const s = { row: 3, counter: 5, sprite: 0, direction: 1 };
          rockGolem(s, 1, 0, ai2, f, o, c, 21);
          if (!(s.row >= 0 && s.row < 21)) {
            throw new Error(`pose ${ai2}/${f}/${o}/${c} row ${s.row}`);
          }
          if (s.counter !== 5) throw new Error(`pose counter ${s.counter}`);
          poses++;
        }
      }
    }
  }
  if (poses !== 24) throw new Error(`poses ${poses} != 24`);
}

sweep();
console.log("f26 slice 7g: the 135-combo sweep + the 24-pose sweep PASS");
