// W4-f27 slice 12 strip/grid-sweep witness — WRITTEN NOT RUN (code-only).
// JS twin of tools/_f27_s12_replay.py's sweep: the 11-kind in-sheet /
// in-grid check over line-for-line C# transcriptions (NPC.cs:67258-67451
// Betsy 551, :67559-67583 skeletons 566/567, :67025-67257 wither beasts
// 568/569, :67711-67755 drakins 570/571, :67998-68421 dark mages 564/565
// the RAW-cell 5x9 grid, :67584-67709 ogres 576/577 the RAW-cell 5x10
// grid). Math.fround pins the f32 products the port uses (the C#
// counter is a double fed by float products, NPC.cs:6485 — every strip
// accumulator this wave is an integer step, exact in both; the dark
// mage's hover |v|*0.1+1 is the one non-integer feed, f32-rounded).
// Run later with: node tools/_f27_s12_sweep.js
"use strict";

const fr = Math.fround;

function betsy(s, ai0, ai1) {
  // NPC.cs:67258-67451 (case 551): the ai[0] phase dispatch. The
  // counters are (int)ai[1] truncations; the entry resets DEAD.
  if (ai0 === 4) { // :67261-67313 (base 60, tail 123)
    s.row = 5; s.counter = Math.trunc(ai1);
    if (s.counter >= 5) s.row = 6;
    if (s.counter >= 54) s.row = 7;
    if (s.counter >= 57) s.row = 8;
    if (s.counter >= 60) s.row = 9 + (Math.trunc(s.counter / 3) % 2);
    if (s.counter >= 123) s.row = 8;
    if (s.counter >= 128) s.row = 7;
    if (s.counter >= 133) s.row = 0;
  } else if (ai0 === 3) { // :67314-67367 (base 40, tail 98)
    s.row = 5; s.counter = Math.trunc(ai1);
    if (s.counter >= 5) s.row = 6;
    if (s.counter >= 34) s.row = 7;
    if (s.counter >= 37) s.row = 8;
    if (s.counter >= 40) s.row = 9 + (Math.trunc(s.counter / 3) % 2);
    if (s.counter >= 98) s.row = 8;
    if (s.counter >= 103) s.row = 7;
    if (s.counter >= 108) s.row = 0;
  } else if (ai0 === 5) { s.row = 3; } // :67368-67371
  else if (ai0 === 6) { // :67372-67444 (the 8-tick landing ladder)
    s.row = 1; s.counter = Math.trunc(ai1);
    if (s.counter >= 8) s.row = 2;
    if (s.counter >= 16) s.row = 3;
    if (s.counter >= 24) s.row = 4;
    if (s.counter >= 32) s.row = 3;
    if (s.counter >= 40) s.row = 4;
    if (s.counter >= 48) s.row = 3;
    if (s.counter >= 56) s.row = 2;
    if (s.counter >= 64) s.row = 1;
    if (s.counter >= 72) s.row = 0;
  } else { s.row = 0; } // :67445-67448
}

function skeleton(s, vx, vy, direction) {
  // NPC.cs:67559-67583 (case 566/567): the grounded walk, idle row 0.
  if (vy === 0) s.sprite = direction; // :67561-67564
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 8; return; // :67565-67570
  }
  if (vx === 0) { s.counter = 0; s.row = 0; return; } // :67571-67576
  s.counter += Math.abs(vx); // :67577-67582
  if (s.counter >= 64 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8);
}

const AURA = [8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 11, 12, 13, 14];

function wither(s, vx, vy, direction, ai0) {
  // NPC.cs:67025-67257 (case 568/569): the aura ladder + the walk.
  if (ai0 > 0) { // :67027-67233
    if (s.row < 5 || s.row > 16) s.counter = 0; // :67031-67034 LIVE
    s.sprite = direction; // :67030
    s.row = 7; s.counter += 1; // :67035-67036
    for (let k = 1; k <= 27; k++) { // :67037-67226 (5*k rungs)
      if (s.counter >= 5 * k) s.row = AURA[k - 1];
    }
    if (s.counter >= 270) { s.row = 14; s.counter -= 10; } // :67227-67231
    return;
  }
  if (vy === 0) s.sprite = direction; // :67235-67238
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 4; return; // :67239-67244
  }
  if (vx === 0) { s.counter = 0; s.row = 6; return; } // :67245-67250
  s.counter += Math.abs(vx); // :67251-67256
  if (s.counter >= 56 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8);
}

function drakin(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67711-67755 (case 570/571): the row-14-hold ladder + walk.
  if (ai1 > 0 && ai0 > 0) { // :67713-67731
    s.sprite = direction;
    if (s.row < 10 || s.row > 15 || ai1 === 59) { s.row = 10; s.counter = 0; }
    const step = s.row === 14 ? 35 : 5; // :67721-67725
    s.counter += 1;
    if (s.counter >= step && s.row < 15) { s.counter = 0; s.row += 1; }
    return;
  }
  if (vy === 0) s.sprite = direction; // :67733-67736
  if (vy !== 0 || (direction === -1 && vx > 0) || (direction === 1 && vx < 0)) {
    s.counter = 0; s.row = 1; return; // :67737-67742
  }
  if (vx === 0) { s.counter = 0; s.row = 9; return; } // :67743-67748
  s.counter += Math.abs(vx); // :67749-67754
  if (s.counter >= 64 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8 + 2);
}

const SUMMON = [6, 7, 5, 6, 7, 5, 6, 7, 8, 9, 10, 11, 12];
const ATTACK = [14, 15, 16, 17, 18, 19, 20, 18, 19, 20, 21, 22, 23, 24, 25];
const GATE = [27, 28, 29, 26, 27, 28, 29, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40];

function darkMage(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67998-68421 (case 564/565): the RAW-cell 5x9 grid machine
  // (s.row holds frame.Y verbatim — the cells run to 40).
  if (ai0 > 0 && ai1 === 0) { // :68004-68113 (7-tick summon)
    s.sprite = direction;
    if (s.row < 5 || s.row > 13) s.counter = 0;
    s.row = 5; s.counter += 1;
    for (let k = 1; k <= 13; k++) if (s.counter >= 7 * k) s.row = SUMMON[k - 1];
    if (s.counter >= 98) { s.row = 5; s.counter = 0; }
    return;
  }
  if (ai0 > 0 && ai1 === 1) { // :68114-68237 (8-tick attack)
    s.sprite = direction;
    if (s.row < 13 || s.row > 25) s.counter = 0;
    s.row = 13; s.counter += 1;
    for (let k = 1; k <= 15; k++) if (s.counter >= 8 * k) s.row = ATTACK[k - 1];
    if (s.counter >= 128) { s.row = 14; s.counter = 0; }
    return;
  }
  if (ai0 > 0 && ai1 === 2) { // :68238-68410 (8-tick gate)
    s.sprite = direction;
    if (s.row < 26 || s.row > 40) s.counter = 0;
    s.row = 26; s.counter += 1;
    for (let k = 1; k <= 22; k++) if (s.counter >= 8 * k) s.row = GATE[k - 1];
    if (s.counter >= 184) { s.row = 26; s.counter = 0; }
    return;
  }
  // :68411-68419 — the hover walk (the one non-integer feed).
  s.counter = fr(fr(s.counter + fr(Math.hypot(vx, vy) * 0.1)) + 1);
  if (s.counter >= 40 || s.counter < 0) s.counter = 0;
  s.row = Math.trunc(s.counter / 8);
}

function ogre(s, vx, vy, direction, ai0, ai1) {
  // NPC.cs:67584-67709 (case 576/577): the RAW-cell 5x10 grid machine
  // (s.row holds frame.Y verbatim — the cells run to 47).
  if (ai0 > 0 && ai1 === 0) { // :67589-67611 (the stomp)
    s.sprite = direction;
    if (s.row < 11 || s.row > 20) { s.row = 11; s.counter = 0; }
    let stride = 4;
    if (s.row === 13 || s.row === 19) stride = 8;
    if (s.row === 14 || s.row === 18) stride = 2;
    s.counter += 1;
    if (s.counter >= stride && s.row < 20) { s.counter = 0; s.row += 1; }
    return;
  }
  if (ai0 > 0 && ai1 === 2) { // :67612-67667 (the pound)
    s.sprite = direction;
    if (s.row < 37 || s.row > 47) { s.row = 39; s.counter = 0; }
    let stride = 5;
    if (s.row === 42) stride = 6;
    if (s.row === 45) stride = 8;
    if (s.row === 46) stride = 4;
    if (s.row === 47) stride = 26;
    if (s.row === 37 || s.row === 38) stride = 7;
    const advance = !(s.row === 46 && vy !== 0) && s.row !== 38;
    if (advance) s.counter += 1;
    if (s.counter >= stride) {
      if (s.row < 47) { s.counter = 0; s.row += 1; }
      else { s.row = 37; s.counter = 0; }
    }
    return;
  }
  if (ai0 > 0 && ai1 === 1) { // :67668-67681 (the swipe)
    s.sprite = direction;
    if (s.row < 21 || s.row > 38) { s.row = 21; s.counter = 0; }
    s.counter += 1;
    if (s.counter >= 5 && s.row < 38) { s.counter = 0; s.row += 1; }
    return;
  }
  // :67682-67707 — the walk (air 43, NO backpedal branch).
  if (vy === 0) s.sprite = direction;
  if (vy !== 0) { s.counter = 0; s.row = 43; return; }
  if (vx === 0) { s.counter = 0; s.row = 0; return; }
  s.counter += Math.abs(vx);
  if (s.counter >= 60 || s.counter < 0) s.counter = 0;
  s.row = 1 + Math.trunc(s.counter / 6);
}

// The sweep: every machine x the velocity/ai grid x 60 ticks, each
// row/cell asserted inside its own sheet/grid bound.
let violations = 0;
let checks = 0;
const VXS = [0, 1, 4, -4, 9];
const VYS = [0, -2, 2];
const DIRS = [1, -1];
const AI0S = [0, 1, 2, 3, 4, 5, 6];
const AI1S = [0, 1, 2, 29, 59];

function sweep(name, fn, bound, usesAi) {
  for (const vx of VXS) for (const vy of VYS) for (const dir of DIRS) {
    const ai0s = usesAi ? AI0S : [0];
    const ai1s = usesAi ? AI1S : [0];
    for (const ai0 of ai0s) for (const ai1 of ai1s) {
      const s = { row: 0, counter: 0, sprite: 1 };
      for (let t = 0; t < 60; t++) {
        fn(s, vx, vy, dir, ai0, ai1);
        checks++;
        if (!(s.row >= 0 && s.row < bound)) {
          violations++;
          if (violations <= 10) {
            console.log(`VIOLATION ${name} t=${t} row=${s.row} bound=${bound} vx=${vx} vy=${vy} dir=${dir} ai0=${ai0} ai1=${ai1}`);
          }
        }
      }
    }
  }
}

// Betsy: the ai-fed ladders (velocity never feeds it).
for (const ai0 of AI0S) for (const ai1 of [0, 5, 40, 60, 72, 98, 123, 133, 200, -5.5]) {
  const s = { row: 0, counter: 0, sprite: 1 };
  for (let t = 0; t < 60; t++) {
    betsy(s, ai0, ai1 + t); // the timer climbs
    checks++;
    if (!(s.row >= 0 && s.row < 11)) {
      violations++;
      if (violations <= 10) console.log(`VIOLATION betsy t=${t} row=${s.row} ai0=${ai0} ai1=${ai1}`);
    }
  }
}
sweep("skeleton", (s, vx, vy, d) => skeleton(s, vx, vy, d), 9, false);
sweep("wither", wither, 17, true);
sweep("drakin", drakin, 16, true);
// The GRID kinds: the bound is the CELL count (45 = the 5x9 grid,
// 50 = the 5x10 grid), NOT npcFrameCount (9/10 the draw rows).
sweep("darkMage", darkMage, 45, true);
sweep("ogre", ogre, 50, true);

console.log(`checks=${checks} violations=${violations}`);
if (violations > 0) process.exitCode = 1;
