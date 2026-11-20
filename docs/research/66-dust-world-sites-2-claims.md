# 66 — World dust sites SECOND batch (W4-e4): claims + census

Status: CLAIM doc (W4-e4, the world lane's second batch). Written early per
the lane's coordination rule — the branch list is the live claim record;
this doc pins the census + the line-order split so the batches never
overlap.

## In-flight probe (2026-11-20, `git fetch origin --prune` + branch scan)

| Lane | Claim state | Verdict |
| ---- | ----------- | ------- |
| WorldGen.cs (29 sites) | `task/W4-g14-dust-worldgen-sites` pushed (0 commits yet) | CLAIMED — untouched here |
| Main.cs (24 sites) | `task/W4-g11/g12/g13` pushed; g12's census doc `64-dust-main-sites-claims.md` pins all 23/24 (the 24th, `CloneDust` :13410, rides g12's :13390 site) | CLAIMED — untouched here |
| d54 first-40 world lane | `task/W4-d54-dust-sites-world` exists, 0 commits, clean worktree, no claim doc | reserved below (the e2 precedent: d52's first-40 lane stayed untouched) |
| ParticleOrchestrator.cs | NO claim branch, NO claim doc; d60 defers orchestrator requests (`ParticleRequest` bytes); `prefix.rs` ports only RainbowRodHit/BestReforge RNG contracts (S1-P-i45), not the emitters | UNCLAIMED |
| Mount.cs | NO claim branch (minecart 213 sparks = S1-E-1, research/15 PARTIAL row) | UNCLAIMED beyond the 213 sparks |

## Census (independent re-derivation, `Dust.NewDust*` family only)

Counting convention = research/15's caller table: `NewDust` +
`NewDustPerfect` + `NewDustDirect` call sites; `CloneDust` rides its parent
site's ledger entry (it re-rolls a fresh `NewDust` inside, `Dust.cs:269`).

- `GameContent/Drawing/ParticleOrchestrator.cs` — **61** sites
  (16 NewDust + 37 NewDustPerfect + 8 NewDustDirect), 8 CloneDust siblings
  (:1214/:1225/:1638/:2144/:3333/:3387/:3436/:3486). research/15's "61"
  matches exactly.
- `Mount.cs` — **25** sites (14 + 7 + 4), 2 CloneDust (:6391/:6413).
  research/15's "25" matches exactly.
- `Main.cs` — 24 family hits = 23 NewDust* + 1 CloneDust (:13410); g12's
  census of 23 is the NewDust* count. All g11-g13's.
- `WorldGen.cs` — 29 (matches research/15). All g14's.

## Line-order split (the world lane)

The C#-line-ordered union of the two unclaimed world files is 86 sites.
The first 40 stay **d54's lane** (the first-batch branch, reserved here so
the second batch never overlaps it — the d52/e2 precedent):

- **d54 reserved: ParticleOrchestrator sites 1-40** — lines 423-2582
  (ArmletOfRuinRecharge through CopperSlimeEffect).
- **W4-e4 (this branch): the next 40 unclaimed** —
  ParticleOrchestrator sites 41-61 (21) + Mount sites 1-19 (19):

| Batch | Sites | Lines | Arms |
| ----- | ----- | ----- | ---- |
| 1 (10) | orch 41-50 | 2653-3058 | ShimmerArrow pair, TerraBlade pair, Excalibur pair, ClassyCane, TrueExcalibur head pair + tail head |
| 2 (10) | orch 51-60 | 3061-3882 | TrueExcalibur tail, TrueNightsEdge pair, PaladinsHammer, PrincessWeapon, StardustPunch, RainbowRodHit, LightningSmall, FormatC, FlameWaders |
| 3 (10) | orch 61 + mount 1-9 | 3920 + 3113-4103 | WoFGoatMountFlames; drill block/wall sparks, unicorn dash ring + body, wolf ring + gated puff, witch ring + gated puff, basilisk charge pair |
| 4 (10) | mount 10-19 | 4123-5071 | basilisk charge trail, golf-cart smoke, gem/steam/exhaust/confetti minecart, witch-broom pair, drill-mount trail pair |

Boundary sites owned whole: orch 2582 is d54's, 2653 is e4's; mount 5071
is e4's, 5099+ stays unclaimed for the next wave (f6 dust-world-3).
CloneDust siblings ride their parent: orch :3333/:3387/:3436/:3486 with
sites 54/55/56/57; mount :6391/:6413 stay with sites 23/24 (unclaimed).

## Contracts

- **Pool**: `tr_core::particles::DustPool::new_dust` / `new_dust_perfect` /
  `new_dust_direct` / `clone_dust` (d50's branch
  `task/W4-d50-dust-type-table`, `Dust.cs:75-256` verbatim). Post-spawn
  writes land on `slot_mut(idx)` — the sentinel 6000 takes them exactly
  like the C# (`Main.dust[6000]` exists).
- **RNG**: one slot per C# pass — the orchestrator spawns run SYNCHRONOUSLY
  inside `RequestParticleSpawn`/`BroadcastParticleSpawn`
  (`ParticleOrchestrator.cs:52-90` → `SpawnParticlesDirect` :162), so each
  emitter drains on the stream of the pass that invoked it (the caller
  passes `&mut UnifiedRandom`; never a new stream, S1-A-10). Mount frame/
  effect sites ride `UpdatePlayers`. Argument draws run BEFORE the pool
  internals (C# evaluates arguments first); `PrettySparkle`/`Flame`/
  `Fading`/`RandomizedFrame` particle-pool requests are RNG-free
  (`ParticlePool.RequestParticle`, Graphics/Renderers/ParticlePool.cs:32)
  but their ARGUMENT draws are consumed in order; the particle visuals
  themselves are the documented renderer skip (research/15: no particle
  renderer exists).
- **Gore siblings** (ClassyCane :2888, ElderSlime :2565) stay d55/g17's
  lane; their argument draws are consumed here in order (the e2 split).
- **RainbowRodHit/BestReforge overlap note**: `prefix.rs` (S1-P-i45)
  consumes those spawns' draws count-only on the reforge tail. This batch
  ports the FULL emitter bodies (site 57 + the d54-reserved BestReforge);
  the wiring wave must fold the prefix count-only drains into the real
  emitters — recorded here so the merger reconciles instead of
  double-drawing.
- WRITTEN NOT RUN (code-only mode): witnesses compile in the merge-verify
  wave, not executed here.
