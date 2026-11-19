# LEAD DIAGNOSIS — s52 gate3 single failure: TEST PREMISE WRONG, port is C#-faithful (delete before commit)

gate3: 3877 passed, 1 failed = `the_speed_ladder_matches_the_csharp` line 708: `speed_of(setup(30000), &ctx, &dead) - 12.0` assertion.

**The test expects dead-target → ×2 = 12. The C# gives ×4 = 24.** Trace NPC.cs:32086-32094:
1. `if (player.dead) flag59 = true;`
2. The retreat gate's LAST conjunct is `|| flag59` — so when flag59 is true, the retreat gate FIRES: `localAI[0] -= 4f; flag58 = true;`
3. Speed ladder (:32206-32213): `if (flag58) num786 *= 2f; if (flag59) num786 *= 2f;` — BOTH flags true → 6 × 2 × 2 = **24**.

Your PORT implements this correctly (flag59 set from p_dead, retreat gate includes `|| flag59`, both multiplies). The TEST's comment "Dead target: ×2 → 12" is WRONG — dead target ALSO trips the retreat gate (flag58), giving ×4 = 24. The test's own NEXT assertion (dead_high = 24) is correct for the wrong reason (it thinks dead+retreat=24, but actually dead ALONE=24, dead+retreat=24 too since both flags already set).

FIX (test-only, port untouched):
- Line 708: change expected 12.0 → 24.0, comment: "Dead target trips BOTH flag59 and the retreat gate's || flag59 conjunct → flag58 also set → ×2 ×2 = 24 (NPC.cs:32091-32094 + :32206-32213)."
- The dead_high assertion (24) stays correct.
- The "high" (above-surface retreat) assertion 12 is correct (flag58 only, flag59 false).

This is your ONE fix round for gate3's failure. Apply, re-gate, land. The port is faithful — do NOT change the impl.
