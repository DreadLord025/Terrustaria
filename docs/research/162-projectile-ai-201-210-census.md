# 162 — Projectile aiStyles 201-210 COVERAGE census (W4-d21)

> Renumbered 26d → **162** per the research-number collision registry
> (`150-research-number-registry.md` §6, W4-d54 `0426245f2`): d12 owns
> the 26d key (00:06:06), d21's add (02:40:39) is a loser → 162
> (uncontested, the sequential 151-236 block). Executed by the d21 SEAL
> round; the FINAL band table + the 1-210 ledger live in
> `308-projectile-ai-201-210-final.md`. The `.rs` module-header cites
> still say `26d-…` — the in-flight fold-2 lane owns those files; the
> rename's code-comment sweep rides its commit (or the merge).

> Researched 2026-11-21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Task contract: port the MISSING arms verbatim + witnesses; 206 landed on
> the sibling a9 branch (`8c4fded8`) — verify-only, NOT re-ported.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs`) — on `a725acb0` the ONLY 201-210 arm present is **210**
  (`ai_yoyo_shot`, M4-PROJ-4b, the Eye-of-Cthulhu dash 1130 + the Kraken wave
  1135).
- C# arm spans: `AI()`'s if/else chain tail (`:36767-36805`) dispatches
  `AI_201_ThrownMelee` … `AI_210_YoyoShots`; the bodies are cited per style
  below. Kind lists parsed from `SetDefaults`' `else if (this.type == N)`
  chain (`:10095-10667`).
- Claim census: `git branch -a` + `git worktree list` — the 206 machine
  landed on `task/W4-a9-minion-four-machines` (`8c4fded8`,
  `forbidden_minion_step` + the 1120 sign shot). The a9 branch is NOT merged
  into this base; its 206 arm is verify-only here.

## Verdict

**Of the ten styles 201-210: 210 was already ported; 206 landed a9
(verify-only); 207 is an EMPTY AI with NO rowed kinds; the seven styles
201, 202, 203, 204, 205, 208, 209 were MISSING (no rows, no arms) and are
ported by this task.**

## Coverage table (201-210)

| style | state on base | C# body | kinds (SetDefaults) | this task |
| --- | --- | --- | --- | --- |
| 201 | M | `AI_201_ThrownMelee` :39954-39983 + HandleMovement :19798-19802 | 1082 AntlionClaw :10095, 1083 StylistScissors :10105 | PORTED |
| 202 | M | `AI_202_TorchGodHelper` :39912-39952 | 1084 TorchGodHelper :10115 | PORTED |
| 203 | M | `AI_203_StormLightning` :39641-39855 + Colliding :15863 + Kill :70771-70854 | 1091 StormLightning :10175, 1117 LightningStrikeShot :10427, 1122 ArcSurge :10437 | PORTED |
| 204 | M | `AI_204_Digtoise` :38463-38801 + DoPalAppearEffect :38803 | 1098 Digtoise :10241, 1123 TrustyDigtoise :10504 | PORTED |
| 205 | M | `AI_205_RemoteControlCar` :38076-38461 + WalkDownSlope + Kill :70855 | 1105 RemoteControlCar :10294 | PORTED |
| 206 | P (a9) | `AI_206_ForbiddenMinion` :37501-37641 | 1119 ForbiddenMinion :10464 (+1120 sign shot) | VERIFY-ONLY (a9 8c4fded8) |
| 207 | EMPTY | `AI_207_SnappingStone` :37492-37494 = `{ }` | NONE (no SetDefaults writes aiStyle 207) | NO-OP arm (census) |
| 208 | M | `AI_208_GlacierFangShot` :36903-37049 + Colliding :15397 | 1115 GlacierFangShot :10401 | PORTED |
| 209 | M | `AI_209_ChlorophyteClaymoreBladeSlam` :69622-69631 | 1129 ChlorophyteClaymoreBladeSlam :10580 | PORTED |
| 210 | P | `AI_210_YoyoShots` :36809-36900 | 1130 EoCDash :10597, 1135 KrakenWave :10653 | ALREADY PORTED (ai_yoyo_shot) |

## Finding F1 — aiStyle 207 is an empty AI with no kinds

`AI_207_SnappingStone` (`Projectile.cs:37492-37494`) is `{ }` — an empty
body. NO `SetDefaults` arm writes `aiStyle = 207` (grep `this.aiStyle = 207`
returns nothing). The Snapping Stone accessory's strike effects
(1121/1131/1132, `TryMakingSnappingStoneStrike` `:14283-14395`) carry
aiStyle 0 (the stock reset) — they are on-hit damage procs, not AI machines.
The accessory proc itself (`accSnappingStone`, the `StatusNPC`/melee/ranged
hit seams `:13399-13407`) is the on-hit layer's follow-up, NOT an AI arm.
The dispatch keeps a documented no-op for the style; no row lands.

## Finding F2 — the 206 machine is a9's, verify-only here

`8c4fded8` (task/W4-a9-minion-four-machines) landed `AI_206_ForbiddenMinion`
as the pool-side `forbidden_minion_step` + the dispatch arm + the box-LOS
elimination check + the 1120 sign shot (row/Kill-burst/gem-tail cue/trailing
mode) + `proj_frames[1119] = 9` + 2 witnesses. That branch is NOT merged
into `a725acb0`; this task does NOT re-port 206 (the Lead's verify-only
instruction). On this base the 1119 row exists (`data.rs:3443`, aiStyle 206)
but rides the `_ => {}` no-op until a9 merges — the documented i25 deferral
(`UNARMED_ROWED` census, `projectile.rs`). The a9 merge resolves it.

## Deliberate folds (ANALYSIS "The projectile aiStyles 201-210 (W4-d21)")

- `gravDir == -1` (the 205's flipped-gravity input XOR): upright-only port.
- `player.petting` (the 204's state-5 joy): no petting layer — unreachable.
- `Main.Difficulty >= Legendary` (the 203's num3/num4): single-difficulty
  port (below Legendary) → 14/5.
- the `LightningGenerator` bolt paths + `MultiPointHitbox` (the 203's Kill
  damage): fold to the 64-px explosion rect at the centre.
- `SoundEngine.PlayTrackedLoopedSound` (the 205's engine loop): no tracked-
  loop layer — the localAI[1]/[2] state runs, the sound body is the skip.
- the digtoise `PickTile` calls: ride the App drain (the hit-tile book +
  the best-pickaxe read the pool cannot borrow), the golf/bobber intent
  precedent.
- the dust/gore bodies, `Lighting.AddLight`, the `ParticleOrchestrator`
  requests, `DroneCameraTracker`, `CheckSectionsInCaseOwnerIsWatching`,
  `NetMessage.SendTileSquare`/`SendData`: no ported layers; their EXPLICIT
  `Main.rand` gates stay on the `UpdateProjectiles` stream in C# order.
