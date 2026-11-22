# 170 — Projectile aiStyles 21-30 band status (W4-d3, the OTHER styles)

> Renumbered 26f → 170 per registry-150 (`150-research-number-registry.md`
> §3: f20's `26f-projectile-ai001-overlays-round2.md` wins the key at
> 05:13:47; d3's 09:28:41 loses) — executed at the W4-d3 SEAL, no external
> refs at tip. The SEAL receipts + the boulder-family bible live in
> `331-d3-boulder-family-seal.md` (renumbered 248→331 per registry-150 §9).
>
> Researched 2026-11-21 against the W4-d3 branch tip (`986ab27b`). C# cites
> are `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` at this revision. Companion to `26b`
> (the original 21-30 dispatch census): this doc pins the band's CURRENT
> armed state + the effect-layer gaps, so the "21-30" branch name is honest
> about what is and is not ported.

## Verdict

**Every style 21-30 has a live dispatch arm on main.** Style 25 (the boulder
family) was the only MISSING one and was ported by this task (26b §"The
missing style 25"). The other nine (21/22/23/24/26/27/28/29/30) were already
armed before this branch — they are **verify-only**, not re-ported here. The
remaining work in the band is the **effect layer** (dust/gore/particle bodies
+ a few per-kind frame machines), which the dust-site co-lanes own.

## Per-style status

| style | C# arm | Rust arm (fn @ projectile.rs) | armed | rowed kinds | effect-layer gap |
| --- | --- | --- | --- | --- | --- |
| 21 | inline 25585-25606 (music notes) + HandleMovement 19299-19309 | `ai_music_note` @6930, dispatch @17725 | YES | 76/77/78 (Quarter/Eighth/TiedEighth note) | the `Next(3)` dust-27 gate (25594) + the soundDelay Item26 latch (25600-25605) skip |
| 22 | inline 25607-25860 (Ice Rod) | `ai_ice_block` @7564 + `ice_block_probe_kill`, dispatch @17559 | YES | 80 (IceBlock) | the dust-67 sites (25665/25676/25683/25756) + the lava/shimmer kills (25742-25751) skip |
| 23 | inline 25861-25935 (Eye Fire / Flames Trap) | `ai_eye_fire` @11293, dispatch @17781 | YES | 101 (EyeFire), 188 (FlamesTrap) | the dust block (25896-25928: the `Next(2)`/`Next(3)` gates + the 75/6 dust) skip |
| 24 | inline 25936-26014 (Crystal Storm) | `ai_crystal_storm` @7739, dispatch @17569 | YES | 90 (CrystalShard), 94 (CrystalStorm), 1134 (CrystalShardMelee) | the 1134 latch 6-dust (25945), the 94 `Next(4)` dust (25958), the 1114 dust (25983, unrowed) skip |
| 25 | inline 26015-26415 + HandleMovement 18941-19277 + Rainbow 20024-20315 + Shimmer 22504-22515 + Kill arms | `projectile_ai_boulder` (whole module), dispatch @17580 | YES — **ported by this task** | the 13 boulders + 763 spike | the dust/gore BODIES now ported (this branch); the ParticleOrchestrator bodies stay skipped (the render layer) |
| 26 | `AI_026` (58102-65037) | `projectile_ai026_a/b/c` slices, dispatch @19739 | YES | A: 14 flyer pets; B: 885/889 boss-head pets; C: 266 slime minion | 24 dust sites across the slices; the explicit argument draws kept, the bodies skip (the slice-module docs) |
| 27 | inline 26420-26554 (beams) | `ai_unholy_trident` @3992 + `unholy_step`, dispatch @17676 | YES | 114/115 (Trident+hostile), 116 (SwordBeam), 132 (TerraBeam), 156 (LightBeam), 157 (NightBeam), 173 (EnchantedBeam) | the 114 `Next(2)`+`NextFloat` dust (26449), the 115 20-dust trail (26461), the 173/132 dust (26470-26487), the 116 dust (26499) skip |
| 28 | inline 26555-26675 (Frost bolt) | `ai_frost_bolt` @7865, dispatch @17669 | YES | 359 (FrostBolt), 309 (FrostBlastFriendly), 967 (HoundiusShootiusFireball), 1026 (DeadCellsBarnacleShot) | the 967/1026 frame machines + dust (26557-26601) are g64's trail lane (UNMERGED); the 177/118/119 dust (26602-26646) skip |
| 29 | `AI_029_GemStaffs` (37643-37827) | `ai_gem_staff` @8183 + `ai_crystal_pulse` @7108, dispatch @17966 | YES | 121-126 (gem bolts), 597 (amber), 521 (CrystalPulse) | the 619/620/522/731 per-kind dust (37650-37766, unrowed) + the gem-bolt `Next(3)` dust-226 (37811) + the pulse alpha-0 dust (37707) skip |
| 30 | inline 26680-26785 (spore fade) | `projectile_ai_spear::ai_spore` + `kill_step`, dispatch @17374 | YES (default arm only) | 131 (MushroomSpore) | **the 907/335 type arms have NO rows** (26682-26774) — the 907 `Next(max(4,8-|v|))` dust-43 + the 335 gem-colour dust are unported; the 131 default fade is verbatim |

## The small missing pieces (candidates, NOT ported by this branch)

The directive's "port MISSING small ones" resolves to **none that are both
small and unclaimed**:

1. **Style 30's 907/335 arms** — the only genuinely unrowed style-30 kinds.
   The 907 arm (26682-26721) is a 40-line machine (the `Next(max(4,8-|v|))`
   dust gate, the ai[0] spin/decel, the owner-gated 335 spawn at ai[0]%10,
   the kill at ai[0]>100); the 335 arm (26722-26774) is the gem-colour dust +
   the settle/spin. Both need `proj_data` rows + the dust body. **Deferred:**
   the 907's dust is g64's `trail_907` (UNMERGED, `origin/task/W4-g64`), and
   the 335 is its spawn child — porting the rows here without the trail lane
   would split the kind across two branches. Left to the g64 merge + a
   style-30 row follow-up.
2. **Style 28's 967/1026 frame machines** — 26b flagged these as a deviation
   (the rows exist, the frame anim skips). **g64 already ports them**
   (`trail_967`/`trail_1026` + the cadence pins, UNMERGED). Not re-ported
   here — the g64 merge lands them.
3. **The dust/gore bodies for 21/22/23/24/26/27/29** — these are the dust-site
   co-lanes' scope (d53/e3/f5 the AI()-region sites, g63/g64 the per-kind
   trails, g65 the death bursts, g16/g17 the gore). This branch ports ONLY
   the boulder family's bodies (style 25) + the 763 spike, per the directive.

## Sibling-branch claims (the in-flight probe)

`git fetch origin --prune` at census time: the d2/d4/d5/d7 bands
(`task/W4-d2-proj-ai-11-20`, `-d4-proj-ai-31-40`, `-d5-proj-ai-41-50`,
`-d7-proj-ai-61-70`) carry **no style-21-30 work** — d2 ports 11/13/17, d4
ports 37/38 (+ the DOC-SWEEP 188 damage gate, which touches the style-23
FlamesTrap's `localAI[0]` read but not its AI arm), d5 ports 41/42/43/46/49,
d7 ports 63/69. No overlap with this band. The dust/gore co-lanes (g64/g16/
g65) are UNMERGED and own the effect bodies cited above.

## Witnesses

This doc is a census deliverable (no code). The style-25 boulder bodies +
the 763 spike carry their own stream-parity witnesses in
`projectile_ai_boulder.rs` (WRITTEN NOT RUN, code-only mode).