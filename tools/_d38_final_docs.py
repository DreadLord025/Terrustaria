# W4-d38 FINAL - append the ANALYSIS section (byte-exact UTF-8 LF, the
# crossnote tail's own convention) + the anchors canary row.
import io
section = """
## W4-d38 FINAL phase (2026-11-21, code-only mode)

The four-sheet FINAL close of the d38 lane (research/125 §§6-10, the
sibling tips re-read at their CURRENT pushed states):

1. **Atlas lazy-load alignment (sheet 1)**: d35's research/113 went
   FINAL at `e5558b0a9` (§10-12: the NPC-sheet lazy-load census). d38's
   eager-form addendum stands (both lanes agree: BAF-alone dies within
   the d40b wave, the boundary-split 2-page form endorsed, the
   8188-vs-8232 ledger reconciliation); the lazy contract RE-PARTITIONS
   the input — `tools/_d38_final_lazy.js` (RUN) re-projects: E35
   (HEAD minus d38's 35 NPC rows) reproduces the **8115** pre-d38
   baseline exactly (the 35 sheets' eager contribution was precisely
   the +121 overflow), and L91 (the d38 35 + d40 56 forecast slice)
   packs **1554 px on ONE own page** (6638 headroom), inside 113
   §10.4's per-world budgets. Under the §10.6 step-7 contract d38's
   rows leave `STATIC_SHEET_NAMES` for the per-world NPC page via
   `texture_net_id` — the draw path already resolves by name, no d38
   code change; the 216/323 portrait-slot pins are lookup tables, not
   atlas input, unchanged.
2. **FindFrame status (sheet 2)**: the C# switch carries 34 of the 35
   cases (328 PumpkingBlade NO-CASE, count 1, the row-0 pin correct —
   the `case 328` at `NPC.cs:77828` is `DoesntDespawnToInactivity`);
   the port has all 35 arms. The wiring-round gap list is exactly **7
   `humanoid_town_frame` stand-ins** (287/295 → extra6 WIRED(d44),
   294/296 → the wave-B module, 322/323/324 → extra7 MODULE-LANDED):
   no grounded sprite flip, no air tail (the C# fighter group
   `NPC.cs:68647-69655` has both + the `:69647` air-pin-0 list). The
   d44 `7c430def6` master research/68 keep-one plan resolves every
   stand-in at merge (the module hook precedes the body match); the
   27 dedicated arms are faithful twins (read line-for-line this
   phase). New witness
   `w4_d38_final_the_seven_town_standins_are_the_wiring_gap` pins the
   stand-ins' CURRENT behavior so the replacement is a witnessed edit.
3. **Bestiary unlock census (sheet 3)**: all 35 C# banner chains
   (`NPCtoBanner` → `BannerToItem` → `KillsToBanner`) agree with the
   port's `kills_needed_for` — **290 Paladin 25 kills** the one
   non-default ladder, the other 34 default 50. Three gaps documented
   for the wiring round: GAP-1 the 6 critters' sight→tier-4 provider
   (vanilla `CritterUICollectionInfoProvider`) vs the port's kill
   ladder — PRE-EXISTING base-wide (the Goldfish 55 row inherits it);
   GAP-2 the 5 rare-spawn rarity stars (display-only); GAP-3 the 328
   `Hide = true` extra card (vanilla's populator excludes it; the port
   row + slot stay this phase, the drop is a witnessed wiring-round
   edit). New witness
   `w4_d38_final_the_unlock_ladders_match_the_banner_chain` pins all
   35 ladders + the exact tier thresholds.
4. **108-extend cross-ref (sheet 4)**: d37's `bdd59afba` full-wave
   re-census CONFIRMS d38's corridor — **281-337 vanilla-free** (NOT
   d35's 429-461 port-local band), protocol==vanilla mechanically
   re-verified for all 35 (hand cs_id rows + the S1-F-3 bestiary
   fallback), the 22 holes exactly the base-owned ids; the two dedupes
   (GiantCursedSkull keep d38@289 drop c3, PumpkingBlade keep d38@328
   drop c6) and the E4 count pins (550 kinds / 489 entries / ≤357
   slots / 446 eager static sheets / max proto 761) recorded.

### Verification (FINAL)

Code-only: the census scripts RUN and passed
(`tools/_d38_final_census.py`, `_d38_final_ff.py`, `_d38_final_unlock.py`
— outputs committed); the lazy re-projection RUN and passed
(`tools/_d38_final_lazy.js`, 4 scenarios × verify() + double-run
determinism). The 2 new Rust witnesses are WRITTEN, NOT RUN — **no
cargo/rustc/tests invoked**; brace balance verified on both edited
files. The anchors canary rows' markers verified present.
"""
p = 'docs/ANALYSIS.md'
b = io.open(p, 'rb').read()
assert b.endswith(b'\n')
io.open(p, 'ab').write(section.replace('\r\n', '\n').encode('utf-8'))
row = ('docs/research/125-d38-crossnote-atlas-addendum.md|FINAL: the 35 kinds\' FindFrame status|'
       'W4-d38 FINAL - the four-sheet close (research/125 s6-10): the lazy-load alignment (E35 8115 / '
       'L91 1554 one page), the 7-stand-in FindFrame gap list, the unlock census (Paladin 25, GAP-1/2/3), '
       'the 108-extend 281-337 vanilla-free confirmation')
p2 = 'anchors.txt'
s = io.open(p2, encoding='utf-8').read()
assert s.endswith('\n')
path, marker, _ = row.split('|', 2)
assert marker in io.open(path, encoding='utf-8').read(), 'marker missing'
io.open(p2, 'a', encoding='utf-8', newline='').write(row + '\n')
print('ANALYSIS section appended + 1 canary row; marker verified')
