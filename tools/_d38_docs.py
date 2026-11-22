# W4-d38 sheets phase: append the ANALYSIS section (byte-safe utf-8 append).
import os
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
p = os.path.join(ROOT,'docs','ANALYSIS.md')
section = """
## W4-d38 sheets/atlas phase (NPC kinds 281-350, 2026-11-21, code-only mode)

**Scope.** The wave's 35 `NPC_<id>` sheets were packed by the wiring
batches b1-b7 (`STATIC_SHEET_NAMES` + the `D38_B1..B7_SHEETS` rows feeding
`NpcAssets::npc_sheets`). This phase (1) verifies the packing headroom
CUMULATIVELY with the sibling d35 wave's 33 deferred sheets, (2) completes
the draw's rotation whitelist for the kinds whose ported machines write
`rotation`, and (3) lands the frame-strip witnesses over the REAL retail
sheet dims.

### Atlas headroom — measured, cumulative (tools/_d38_atlas_cumulative.js)

The harness reuses `tools/atlas_sim.js`'s real-dims census (the LZX/XNB
first-frame reader over `assets/Content/Images/*.xnb`) and its packer twins
(skyline bottom-left waste tie-break = the landed `skyline_pack`,
`tr-render/src/atlas.rs`; maxrects BAF = comparison only). The purged
`origin/task/S1-L-8b-mount-data` git-show is stubbed IN MEMORY (its 25
additions all landed on main — the base sim itself reports "L-8b additions
(0)"); `tools/atlas_sim.js` is untouched. Every reported layout passes the
harness's `verify` (PAD-inflated no-overlap + in-bounds) and the skyline
double-run determinism check.

Two sheet sets are measured, because the runtime set changed under the
wave: the E-4b gore enumeration LANDED on main (`assets.rs`'s
`enumerate_gore_sheet_ids`, ~1402 sheets), so the honest runtime census
includes gore. c4's quoted baseline (7757 static / 8143 full) was measured
pre-landing plus the boot font; `tools/_d38_c4_recheck.js` reproduces BOTH
numbers exactly (8119 -> 8143 with font), so the ledgers reconcile.

| set (skyline, waste tie-break) | sheets | height | vs 8192 |
| --- | --- | --- | --- |
| static-only, pre-d38 baseline | 8095 | 7711 | 481 headroom |
| static-only, HEAD (d38's 35 packed) | 8130 | **7867** | 325 headroom |
| static-only, HEAD + d35's 33 deferred | 8163 | **8035** | 157 headroom |
| static-only, c4 check (baseline + 224/509/581/587) | 8099 | 7757 | c4's exact quote |
| runtime (gore landed), pre-d38 baseline | 9497 | 8115 | 77 headroom |
| runtime, HEAD (d38's 35) | 9532 | **8236** | **OVERFLOW +44** |
| runtime, HEAD + d35's 33 (CUMULATIVE) | 9565 | **8452** | **OVERFLOW +260** |
| runtime, cumulative + boot font | 9566 | 8452 | OVERFLOW +260 |
| runtime, three-wave (d38 + d35 + c4's 4) | 9569 | 8461 | OVERFLOW +269 |
| maxrects BAF, HEAD (d38) | 9532 | 8058 | 134 headroom |
| maxrects BAF, CUMULATIVE d38 + d35 | 9565 | **8170** | **22 headroom** |
| maxrects BAF, three-wave | 9569 | 8152 | 40 headroom |

Attribution: d38's 35 sheets cost +121 px skyline over the runtime baseline
(8115 -> 8236; 0.72 Mpx padded area), d35's 33 deferred sheets a further
+216 (8236 -> 8452; 0.95 Mpx). The best-fit shelves packer (the pre-G-12b
main) overflows every set here (9116/9204) — the skyline is the only
landed packer that ever fit the gore set.

**Verdict: OVERFLOW documented.** The d38 sheets fit the STATIC list
(7867 < 8192) but NOT the runtime set with the landed gore enumeration
(8236 > 8192). This is NOT a d38 regression alone: the pre-d38 runtime
baseline already stood at 8115 (77 px headroom), and ANY wave of ~35 NPC
sheets tips it. The g72 note's "7530 static+gore" is STALE (measured
before later waves' sheet additions; the same harness now measures 8115
for the pre-d38 set) — the wyvern-deferral premise ("the atlas stands at
the 8192 strict-GPU bound") that d35's draw arms cite is REAL again for
the runtime set, but d38's sheets are already packed and MUST NOT be
silently dropped: the fix is a packer/atlas decision, not a per-wave
deferral lottery.

**Proposals (Lead decision — NOT implemented here):**
1. **maxrects BAF promotion** (preferred, smallest diff): the cumulative
   d38+d35 set packs 8170 (22 px headroom) and even the three-wave set
   8152 (40 px) under maxrects best-area-fit — the harness's
   `maxrectsPack` is already a line-for-line candidate. Promote it into
   `tr-render/src/atlas.rs` behind the same `Atlas::build` seam (the
   G-12b skyline swap precedent: layout changes are UV-neutral because
   every consumer reads `atlas.sheet(name)` rects). Headroom is THIN
   (22 px) — the next wave (~35 sheets ≈ +120-220 px) overflows again.
2. **Second atlas page** (structural, unblocks the whole roadmap): the
   C# itself draws through MANY textures (`Main.spriteBatch` per
   `TextureAssets` row); the port's one-atlas invariant is a port
   simplification, not a vanilla constraint. A page-2 texture for the
   NPC/gore families (the g72 underworld note already asks for "the
   S1-G-13 background-target family or a second texture array") lifts
   the bound permanently. Cost: the instanced sprite pipeline binds one
   texture today — a second bind or a texture array + shader index.
3. Combined: promote maxrects NOW (buys the d35 repack), land page-2
   before the d36/d39/d40/d43 waves pack their sheets (four more sibling
   waves × ~35 kinds each are in flight — at ~+150 px/wave even maxrects
   dies within two waves).

### Rotation whitelist (draw gate, render/entities.rs)

The port gates rotation per kind (`n.rotation != 0.0 && matches!(...)`);
the C# generic draw rotates EVERY NPC (`Main.cs:26632`). Per the C#
rotation-write census of each kind's ported machine:

| kind(s) | writer | citation | whitelist |
| --- | --- | --- | --- |
| 281-286 casters (style 8) | none — `NPC.cs:20817-21376` has ZERO `this.rotation` writes; FindFrame `:77023-77054` rows 0/1/2 | — | OUT |
| 287/290-296 walkers+shooters (style 3) | none — the shooters AIM via frame rows (`ai[2]`, `NPC.cs:76021-76059`/`:76139-76178`); AI_003's rotation writes are the type-471 arm only (`:56507`) | — | OUT |
| 289 GiantCursedSkull (style 10) | `npc_ai_style10.rs:368/:372` aim atan2 (+PI left) | `NPC.cs:21750-21766` | IN (s8) |
| 297/298 birds (style 24) | `npc.rs:9401/:9415` `rotation = vx*0.1` | `NPC.cs:73634` | IN (s8) |
| 299/300/303/337 critters (style 7) | none — AI_007 (`NPC.cs:52965-55563`) has ZERO rotation writes; the hop is frame rows 2/4 | — | OUT |
| 301 Raven (style 17) | `npc.rs:9469` `rotation = vx*0.1` | `NPC.cs:73589` | IN (s8) |
| 302/304/333-336 slimes (style 1) | only the balloon-freeze `rotation = 0` (`npc_ai_slime.rs:477`) — a zero is not a tilt | `NPC.cs:61046-61052` | OUT |
| 316 Ghost (style 22) | `npc.rs:9379` `rotation = vx*0.1` | `NPC.cs:74858` | IN (s9) |
| 317/318 demon eyes (style 2) | `npc.rs:9317/:9388` atan2 (+3.14 literal left) | `NPC.cs:74019-74031` | IN (s9) |
| 322-324 skeletons (style 3) | none (humanoid frame group `NPC.cs:68590-68601`) | — | OUT |
| 328 PumpkingBlade (style 59) | `npc_ai_style59.rs:355/:364/:398` atan2 + the 1.57 LITERAL | `NPC.cs:33296`/`:33305`/`:33340` | IN (s9) |

The eight IN kinds' machines already wrote `rotation` (the wiring batches
ported the writers); the whitelist was the missing half — without it the
draw dropped the tilt on the floor. The 27 OUT kinds are a deliberate
match to the C# (no writer, no rotation), not a deferral.

### Frame-strip witnesses (render/entities.rs `d38_sheet_tests`)

`tools/_d38_dims.py` decodes the REAL `NPC_<id>.xnb` dims through
atlas_sim's LZX reader (the same decoder the packer measurement uses) and
cross-checks `Main.npcFrameCount` (`Main.cs:67082`, all 697 entries)
against the port's verbatim `NPC_FRAME_COUNT` table: all 35 agree. The
witness module pins, per kind: own-sheet draw (`texture_net_id ==
cs_id`), the row count, `vanilla_row_h`'s integer division, the
`pumpkin_moon_draw` width/anchor arithmetic and the last-row UV staying
in-bounds; plus the M4-NPC-3v packing audit (all 35 `D38_*` ids in
`STATIC_SHEET_NAMES`). The ONE non-multiple strip: NPC_286 is 40x170 over
3 rows — `SetFrameSize` (`NPC.cs:90780-90800`) integer-divides to 56 and
DROPS the 2 px tail; the witness pins the remainder rather than papering
over it. NPC_328 is a SINGLE 106x86 frame (`npcFrameCount[328] = 1`); the
C#'s type-328 draw arm (`Main.cs:22669-22699`) draws the Pumpking's ARMS
(`TextureAssets.PumpkingArm`) — the blade's own sheet is the body only.

### Verification

Code-only: the atlas numbers come from RUNNING the node harness (outputs
in `tools/_d38_atlas_cumulative.txt`); the dims/rows table from RUNNING
`tools/_d38_dims.py`. The Rust witnesses (2 tests over 35 kinds) and the
whitelist arms are WRITTEN, NOT RUN — **no cargo/rustc/tests invoked**.
Structural check: brace/bracket balance unchanged vs the base file, one
`d38_sheet_tests` module, 35 STRIPS rows, 8 new whitelist kinds present.
"""
with open(p,'a',encoding='utf-8',newline='') as f:
    f.write(section)
print('appended', len(section.splitlines()), 'lines')
