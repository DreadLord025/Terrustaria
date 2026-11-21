# 240 — W4-g62 hit-dust FINAL + the hit-dust-family declaration (code-only)

**Number:** 240 — first-free per registry-150 (`150-research-number-registry.md`,
d54's scan: free block 237+; a fresh `git ls-tree` sweep over EVERY origin tip
2026-11-21 found 237 double-claimed e23/g48/g61, 238 double-claimed c8/d42,
239 taken d42 — **240 is the first number no tip carries**). Renumber at merge
if the registry assigns differently.

**Status:** ✅ FINAL (code-only; witnesses WRITTEN NOT RUN).
**Lane:** W4-g62 (`task/W4-g62-npc-hit-dust`, base tip `32af98b44`).
**Companions:** g59/g60 (death halves), g61 (spawn dust), c13 (hit sounds),
d55/e5/f10/f5 (gore bodies), g60 research/110 §12.8 (the keep-one bible),
f5 research/111 (the 853-site FINAL receipt).

---

## 1. The FOUR hit-half gore bodies — census + port (the f5 hand-off EXECUTED)

f5's FINAL receipt (research/111, tip `d1d507461`) split its 853-site wave-D
recount: **849 death-half (f5-owned, citation-covered 849/849, 0 missed) + 4
hit-half** — `:86532`/`:86947`/`:86951`/`:88648`, "bodies a wiring hand-off".
g60's research/110 §12.8.3 pinned the same four as unowned after the
f10-overrun wholesale delete (f10's copies sit in the drop bands; d55's pushed
window `:81660-81941` covers NONE of them — its tip `ec78d152` re-grepped: no
133/186-hit/632-hit/694 body). **g62 now owns all four** — batch 35,
`npc_hit_dust.rs::replay_hit_dust` (the `GorePool` parameter added; the drain
passes `app.gore`, the S1-E-4 pool already on main):

| site | kind | C# window | gate | body | scale | arg draws |
| --- | --- | --- | --- | --- | --- | --- |
| `:86532` | 133 WanderingEye | `:86529-86533` | `life < lifeMax*0.5f && localAI[0] == 0` (f32 casts, INSIDE `life > 0`) | ONE gore **1** at position, velocity | **1f default** | ZERO (constant args) |
| `:86947` | 186 PincushionZombie | `:86945-86948` | `Main.rand.Next(5) == 0` after the ×100 dust-5 loop | ONE gore **242** at position, velocity | **1f default** | the Next(5) — ALREADY consumed here since batch 1, now the body spawns INSIDE the gate |
| `:86951` | 632 MaggotZombie | `:86949-86952` | the same Next(5) shape | ONE gore **1217** at position, velocity | **1f default** | the Next(5), same |
| `:88648` | 694 Deerclops | `:88645-88649` | LIVE half, NO dust; count `max(1,min(5,(int)(dmg/lifeMax·3)))` | count × gore **1008** at `Centre + ((-w + (int)(w·NextFloat))·0.5, (-h + (int)(h·NextFloat))·0.5)` | **this.scale** | 2 NextFloat per gore (X then Y, int truncation BEFORE the ×0.5) — ALREADY consumed here since batch 32, now feeding the position |

C# re-read verbatim (`NPC.cs:86521-86544`, `:86937-86967`, `:88641-88659`,
≤100-line windows). The 694 shape is f5's death twin (`:88654`,
`water_bolt_mimic_death`) with the LIVE clamp — the death half's FLAT 5 +
1007 siblings stay f5's (`:88652-88659`, a different site).

**Owner contract for the merger:** these four bodies are g62's — f10's
overrun copies (`wandering_eye_hit` fn 29 in the §12.8.3 drop list, the
zombie-block/deerclops overrun fns) DELETE wholesale per g60's bands; f5's
dispatcher comments already defer them ("g62's lane", `npc_hit_effect_d.rs`
`:1401`/`:2108` + the 694 hit-half note); d55 never had them. No strip row:
g62's replay draws each argument ONCE and spawns the body in the same pass.

**The OTHER hit-half gore siblings stay argument-consumed here, body-owned
by their lanes** (the e2 split): the confetti slime's 5 (`:84738-84740`) =
f10's LEGIT `party_slime_confetti` (window ≤ `:85559`, NOT overrun); the
Queen minion 667's 3 (`:82043-82047`) = f10's legit DD2 window. Their wiring
strips the re-drawn dust AND the re-drawn argument picks (g60 §12.4
precedent) — g62's consumed draws are the keep-one copy.

## 2. The 133 eye-pop latch — the port shape (the ONE non-mechanical bit)

g60 §12.8.3's latch note: "a 5-line arm, NOT a reason to keep f10's fn".
Ported as three pieces, C# write order preserved:

1. **Capture gate** (`HitDustSnapshot::capture`): `eye_pop_latched =
   cs_id == 133 && hp > 0 && (hp as f32) < (max_hp as f32) * 0.5 &&
   local_ai[0] == 0.0` — strike-time state, the f32 casts verbatim
   (`:86529`), the strict `<` (at exactly half: NO pop).
2. **Strike-entry write** (`npc.rs` `hurt`/`hurt_from_projectile`/
   `strike_pooled`, right after the capture push): `localAI[0] = 1`
   (`:86531` — the C# writes the latch BEFORE the NewGore; the port's
   drain replays the body later, so the write rides the strike, not the
   drain — a second same-tick strike captures unlatched, exactly like
   the C#).
3. **Drain body** (`replay_hit_dust`'s 133 arm): after the ×100 dust-5
   loop, `if s.eye_pop_latched` → ONE `gore.new_gore(…, 1, 1.0)` — zero
   argument draws, only the pool's own 2 kicks (`Gore.cs:1327-1328`).

The killing blow never pops (the gate sits under `life > 0`; the death
half's 155 pair `:86541-86542` is f5's `wandering_eye_death`).

## 3. The g60f-widening MUTUAL RECEIPT (independently re-verified)

g60's §12.8.1 claims all 6 hunks of g62's `32af98b44` contract consumed
byte-identical at `19431e385` (+ the DeathFx half), re-verified `9a6c543f7`
/ pinned `30fea0fe1`. **g62 re-ran the check itself** (g60's own
`g60_verify_widen.py`, pure-text Python, no cargo): g60 tip `30fea0fe1`'s
`npc.rs`/`npc/data.rs` vs g62 `32af98b44`'s —

```
fields: mine=34 g62=34 equal=True
init: mine=8 g62=8 equal=True
update_trails: mine=40 g62=40 equal=True
transform: mine=7 g62=7 equal=True
witnesses: mine=105 g62=105 equal=True
table: mine=19 g62=19 equal=True
ALL BYTE-IDENTICAL
```

**CONFIRMED: the 6-hunk contract is CONSUMED, fields 34/34, witnesses
105/105.** The merge dedupes the shared `Npc` surface trivially; g62's only
additions stay g62's (the pooled tail call, `HitDustSnapshot.old_pos[60]` +
`trail_len`, the census doc notes). The 402 sites stay disjoint (g60 DEATH
`:83611`, g62 HIT `:83579`).

**Concurrency note (observed, NOT committed here):** during this phase a
separate uncommitted "trail-kind rowing" follow-up (677 Shimmerfly / 620
GoblinShark adoption + its own receipt paragraph) appeared interleaved in
this worktree's working tree. This commit stages ONLY the batch-35 hunks;
that follow-up's table/gate widening (677→60, 620→20) is its author's to
commit. If it lands, g60's byte-identical hunks still dedupe against the
unchanged parts and the widened table/gate resolve to the g62-side superset
(no semantic conflict — g60 owns no 677/620 reader).

## 4. The HIT-DUST-FAMILY declaration

**The W4-g62 hit-dust family is COMPLETE:**
- **629 live `Dust.NewDust*` sites** in `NPC.cs:81654-89834` — zero missed
  (the batch-30/32 census sweeps; every site cited in the module ledger or
  classified death-half).
- **4 hit-half gore bodies** (`:86532`/`:86947`/`:86951`/`:88648`) — OWNED
  and ported (batch 35, this doc §1); the f5 research/111 hand-off closed.
- **The 133 eye-pop latch** (`:86529-86533`) — OWNED and ported (§2); the
  g60f §12.8.3 unowned-after-delete item closed.
- **The TrailCache widening** (402→36 / 519→20 / 677→60 storage) — LANDED
  (`d3b008c78`/`945a6ea28`/`32af98b44`), g60's half CONSUMED (§3).
- Every-strike arms, the gold-critter pre-chain block, the gore-argument
  consumption order, the strike-time capture + UpdateNPCs drain fold — all
  as recorded in ANALYSIS "The NPC hit-dust lane (W4-g62)".

**Witnesses (WRITTEN NOT RUN, code-only):** 63 module witnesses in
`npc_hit_dust.rs` (61 + the 2 new batch-35: the eye-pop latch gate/body
twin + the 632 gated-body both-outcomes sweep; the pincushion + Deerclops
witnesses RE-DERIVED to mirror the gore slots) + 5 `npc.rs` witnesses (the
strike-capture seam + the 2 g60-adopted widened trail witnesses + the 2
widening witnesses) + 1 new `world_sim/npcs.rs` seam witness
(`w4_g62_the_drain_latches_the_eye_and_spawns_one_pop` — the strike-entry
latch write + the one-pop drain + the no-second-pop). **69 total.**

**Remaining lane boundaries (NOT g62's):** death halves g59/g60; spawn dust
g61; hit sounds c13; the confetti/667 gore bodies f10's legit window; the
519/522/685 kind rows their wiring lanes'; the 677/620 rows the concurrent
trail-kind follow-up's (§3 note).