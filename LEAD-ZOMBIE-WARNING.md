# LEAD WARNING — FOREIGN UNCOMMITTED WORK IS A REGRESSION (do NOT commit it; delete before your commit)

Lead investigated the staged npc.rs + 672-line module rewrite (timestamps 00:09-00:14, code-only era):

**It REVERTS the landed wiring.** The pushed branch tip d0420bea (W49, ALREADY WIRED: `31 => spazmatism_ai` axis claim + kind arm, Style31Env::CLASSIC_SOLO) is the CORRECT form. The foreign uncommitted work renames spazmatism_ai→style31_step, Style31Env→Style31Context, and REMOVES the 31=> axis claim (back to UNWIRED + update_spazmatism stand-in). This is a zombie code-only session's rewrite that never saw the wiring.

**ACTION for L7 when you reach s31:**
1. DISCARD the foreign uncommitted work: `git checkout -- crates/tr-app/src/npc_ai_style31.rs` + `git reset HEAD crates/tr-app/src/npc.rs` + `git checkout -- crates/tr-app/src/npc.rs` (verify git status clean after).
2. The branch HEAD d41c667e (local sync merge over origin/main) + pushed tip d0420bea carry the WIRED form. Re-sync HEAD over current origin/main, gate, land.
3. Main currently does NOT have npc_ai_style31.rs (verified) — s31 is genuinely unlanded; your landing adds the WIRED machine.
4. If the foreign session is still alive and re-writes: it is a zombie (W49-restart ses_e0f90557dffarOkFKNapdMercM died rate-limit ~04:00; if it resumed it must be stopped). Report to lead; do not race it.

DO NOT commit the regression. The wired d0420bea form is C#-faithful and gate-proven (W49's original report: 15 witnesses).
