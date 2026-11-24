
## X7 вЂ” small-lanes-END receipt (resume merger, step 2)

All 31 remaining small-lanes were merged with `git merge --no-ff` and pushed to
`main` with the ref-race protocol (fetch + merge origin/main + retry; never force).
d34-npc-kinds-211-280 was verified as a PURE DUPLICATE (all 290 of its kinds are
already present in main via d37) and recorded as dropped-spare вЂ” no merge.

LANDS (in merge order, with commit shas recorded in git log):
d23 a641fdb6e (step 1, pre-roster), c5 3c1abcb2c, c6 9332b6ec4, d22 d59d25298,
d24 17690e200, d25 8fdcf5475, d120 1a61b4b67, d125 0dfecaee4, g5 4c932c8c5,
g8 016f28f8c, d33 fc182f741, e21 d91d9a8e1, f42 7333f5b48, d9 6b9e44bee,
d39 a26fc5c7d, d41 bcfe8d9c7, e26 4225b3348, g10 fc277844b, f47 28561a4385,
f18 73bff46a4, d1 61a9381fbe, d2 66a8e0d7ab, d3 a38a7b396d, d4 f5f61e7be6,
d5 9e14e87063, d6 6280b29f0d, d7 81623b3a90, d8 a4660e4add, g62 ee2bb31fff,
d43 ac528a7f7, d36 6827b4f8c (FINAL, sealed 27/27).
SKIPS: none by ancestry (all 31 were ancestors=false at start); d34 dropped as
pure duplicate after content verification.

OBLIGATIONS: d32 G2 `NpcKind::DeadlySphere => {}` arm preserved; c3 329 s3 E8
drop-hunks honoured (d34/d38 landed); d125 kept 636 (EmpressOfLight) and 742;
d36 axis-26 kind-gated with Wolf@772 / Tumbleweed@546; e21 Bunny@229 dropped in
favour of d34@270; d33 protocol collisions re-keyed (FairyCritterPink/Green/Blue
359-361 -> 583-585, WindyBalloon 368 -> 594); HallowBoss re-keyed 636 -> 773;
Frankenstein re-keyed 773 -> 780.

FINAL PYTHON PIN RECOMPUTE (real counts, not estimates):
enum variants 688 unique; ALL_KINDS 688 declared/rows; to_u16 688 rows / 688
distinct kinds / 688 distinct ids; from_u16 688 rows / 688 distinct; to/from
bijective; s1_f_3 pin 688; ALL_KINDS.len() pin 688; CS_STYLES 248 distinct;
BESTIARY_NPC_SHEET_IDS declared 439 == actual 439, distinct; ENTRIES 611 distinct
credit ids; render-key 791-805 reservation UNTOUCHED (15 Crab/bones rows).
remaining-to-665 = 0 (665 surpassed by 23).

Cargo `cargo test -p tr-app --no-run` was attempted per the addendum; the final
`Finished` line was not reached вЂ” see the final report for the outstanding
mechanical compile fixes.
