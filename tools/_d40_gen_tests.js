// W4-d40 sheets phase: generate the two test witnesses (frame-count +
// strips-divide) and the packing-audit extension, as exact edit payloads.
'use strict';
const fs = require('fs');
const path = require('path');
const w = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_wiring.json'), 'utf8'));
const packed = w.filter(o => o.packed);

// Test 1: the d40 frame-count witness — every kind's texture_net_id routes
// to its own vanilla id, and npc_rows gives the Main.cs:67082 count.
const fcCases = w.map(o => '            (NpcKind::' + o.name + ', ' + o.id + ', ' + o.rows + '),').join('\n');
const test1 = [
  '    /// W4-d40 sheets phase: the 56 kinds\' `texture_net_id` routes to',
  '    /// their OWN vanilla id (the Hide-fold/no-entry kinds via the explicit',
  '    /// rows, the own-entry kinds via the wildcard), and `npc_rows` gives',
  '    /// the `Main.npcFrameCount[type]` count (`Main.cs:67082`, verified',
  '    /// == NPC_FRAME_COUNT verbatim). A drift shows as a strip that only',
  '    /// animates its first few rows (the family-rows test\'s shape).',
  '    #[test]',
  '    fn the_d40_rows_match_between_the_draw_and_the_machine() {',
  '        let cases: &[(NpcKind, u16, u32)] = &[',
  fcCases,
  '        ];',
  '        for &(kind, id, rows) in cases {',
  '            let net = texture_net_id(kind);',
  '            assert_eq!(net, id, "{kind:?} draws its own NPC_{id} sheet");',
  '            assert_eq!(npc_rows(net), rows, "{kind:?}: NPC_{net} rows");',
  '        }',
  '    }',
].join('\n');

// Test 2: the packed strips divide into their vanilla rows (the real .xnb
// dims measured by the c4 LZX harness). The deferred 20 stay OUT of D40_SHEETS.
const stripCases = packed.map(o => '            (NpcKind::' + o.name + ', ' + o.w + '.0, ' + o.h + '.0, ' + o.rows + '), // NPC_' + o.id).join('\n');
const deferredIds = w.filter(o => !o.packed).map(o => o.id);
const test2 = [
  '    /// W4-d40 sheets phase: the 36 packed strips divide into their',
  '    /// `Main.npcFrameCount[type]` rows (`Main.cs:67082-67154`) — the real',
  '    /// `.xnb` dims measured by the c4 LZX harness (`tools/_d40_dims.js`).',
  '    /// `pumpkin_moon_draw` uses `vanilla_row_h` (INTEGER division, the',
  '    /// `NPC.SetFrameSize` form `NPC.cs:66666`), so even the non-dividing',
  '    /// NPC_444 (74/3) loses the leftover 2 px at the bottom instead of',
  '    /// bleeding them into the last frame. A wrong divisor shows as a',
  '    /// fractional row here, a squashed sprite in game.',
  '    #[test]',
  '    fn the_d40_strips_divide_into_their_vanilla_rows() {',
  '        // (kind, sheet w, sheet h, npcFrameCount[type])',
  '        let cases: &[(NpcKind, f32, f32, u32)] = &[',
  stripCases,
  '        ];',
  '        for &(kind, w, h, rows) in cases {',
  '            let net = texture_net_id(kind);',
  '            assert_eq!(npc_rows(net), rows, "{kind:?}: NPC_{net} rows");',
  '            assert!(D40_SHEETS.contains(&(net as u32)), "{kind:?} packed");',
  '            let t = sheet(w, h);',
  '            let npc = bare(kind);',
  '            let (tex, fh, size, _) = pumpkin_moon_draw(Some(&t), rows, &npc, 20.0, 40.0);',
  '            assert!(tex.is_some());',
  '            // SetFrameSize integer division: the row is floor(h / rows).',
  '            assert_eq!(fh, (h as i32 / rows as i32) as f32, "{kind:?}: integer row");',
  '            assert_eq!(size, [w, fh], "{kind:?}: the frame keeps the sheet width");',
  '        }',
  '        // The 20 deferred kinds stay OUT of the packing list (the wyvern',
  '        // precedent): their draw arms ask npc_sheet, get None, and skip.',
  '        for id in [' + deferredIds.map(id => id + 'u32').join(', ') + '] {',
  '            assert!(!D40_SHEETS.contains(&id), "NPC_{id} must stay deferred");',
  '        }',
  '    }',
].join('\n');

fs.writeFileSync(path.join(__dirname, '_d40_test1.txt'), test1);
fs.writeFileSync(path.join(__dirname, '_d40_test2.txt'), test2);
console.log('test1 lines:', test1.split('\n').length, 'test2 lines:', test2.split('\n').length);
console.log('strip cases:', packed.length, 'deferred:', deferredIds.length);
