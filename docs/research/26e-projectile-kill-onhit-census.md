# 26e — Projectile Kill()/OnHit hook COVERAGE census (W4-g3, lane c)

> Researched 2026-11-21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Task contract (lane c): read ALL pushed branches, port ONLY UNCLAIMED
> work. The f19 1-105 census + the d1-d21/a9/g1/g2 claims are the input.

## Trigger — the f19 range is EXHAUSTED

- f19 (`task/W4-f19-proj-ai-verify-1`, tip `ec5dec82`) censused aiStyles
  **1-105**: 63/105 armed, 42 missing, ALL 42 claimed by live d1-d11
  branches (findings F1-F5). UNCLAIMED missing arms in 1-105: **ZERO**.
- The d-wave decades d1-d21 cover **1-210**. A full scan of every
  `this.aiStyle = N` assignment in `Projectile.cs` SetDefaults shows the
  **maximum aiStyle is 210** — there is **NO aiStyle beyond 210** to census.
- The leftover pool g1/g2/g3 (`proj-ai-missing-a/b/c`) are all at `a725acb0`
  with no commits; a9 (`minion-four-machines`) landed 156/164/169/206.
- **Therefore the fallback clause fires:** census C# **Kill()/OnHit hooks
  missing vs projectile.rs**, port verbatim + witnesses, ONLY unclaimed.

## Method

- Kill() spans `:70642-77264`. The port's death-hook seam is the
  `if !self.slots[i].active { ... }` block in `ProjectilePool::update`
  (`projectile.rs:19946-20068`), which calls the per-family kill helpers
  (`spawn_inferno_blast`, `gem_bolt_aoe`, `floating_turret_kill`,
  `celeb2_death`, `sharknado_bolt_kill`, `moonlord_arrow_kill`,
  `xenopopper_bubble_kill`, `dart_electro_firework_kill`,
  `special_arrow_kill`, `special_bullet_kill`, `thrown_consumable_death`,
  `projectile_bobber::kill_hook`, `rocket_kill_census`).
- OnHit = `Damage_PVE_Inner`'s post-strike arms (`:13018-14270`) +
  `StatusNPC`/`StatusPlayer` + the `Damage_PVP`/`Damage_EVP` tails. The
  port's seam is `damage_pve` (`projectile.rs:22518-23253`) and
  `damage_pve_boxed` (`:23261-23399`).
- Claim census: `git branch -a` + `git worktree list` + the pushed tips of
  d2/d3/d6/d13/d15/d16/a9. A hook is UNCLAIMED iff no live branch's commits
  add it AND its kind's AI arm is already landed on main (so the hook is a
  follow-up, not a new decade).

## Verdict

The AI-arm axis is exhausted (1-210, all claimed). The **Kill()/OnHit hook
axis** still has UNCLAIMED, rowed, AI-armed kinds whose death/hit effects
ride `_ => {}` today. Ported here (one hook per commit):

| hook | kind(s) | C# span | effect | claim |
| --- | --- | --- | --- | --- |
| Kill resize+Damage+Item10 | 16 MagicMissile | :71329-71408 | 128 box, pen -1, Damage, restore, Item10 | UNCLAIMED |
| Kill resize+Damage+Item10 | 34 Flamelash | :71409-71489 | 96 box, pen -1, Damage, restore, Item10 | UNCLAIMED |
| Kill resize+Damage+Item10 | 79 RainbowRod | :71257-71328 | 128 box, pen -1, Damage, restore, Item10 | UNCLAIMED |
| Kill resize+Damage+Item14 | 634/635 NebulaBlaze | :71995-72050 | 50 box, pen -1, Damage (no restore) | UNCLAIMED |
| Kill resize+Damage+Item14 | 659 SpiritFlame | :72503-72542 | ai0>=0 gate, 80 box, Damage | UNCLAIMED |
| Kill resize+Damage+Item14 | 661 OnyxBolt | :72226-72277 | 160 box, pen -1, Damage | UNCLAIMED |
| OnHit timeLeft=0 | 659 SpiritFlame | :13222-13225 | any landed hit zeroes life | UNCLAIMED |
| OnHit direction | aiStyle 68 Molotov | :13149-13163 | fuse 3 + face victim | UNCLAIMED |
| OnHit direction | aiStyle 50 Inferno | :13164-13175 | face victim | UNCLAIMED |

All nine kinds/styles already have their AI arm + `proj_data` row on main
(aiStyle 9 aimed spells `ai_magic_missile`, aiStyle 1 slice-C `blaze`,
aiStyle 129 `ai_spirit_flame`, aiStyle 1 `ai_arrow` for 661, aiStyle 68
`molotov_step`, aiStyle 50 `ai_inferno_fork`), so the hooks are pure
follow-ups — no new decade, no overlap with a d-wave claim.

## Deliberate skips (recorded, the house dust/sound convention)

- The Kill arms' `Dust.NewDust*`/`Gore.NewGore`/`ParticleOrchestrator`
  bodies are SKIPPED with their internal rolls (the `celeb2_death` /
  `fairy_queen_death_draws` precedent). The resize + `Damage()` + the
  `SoundID.Item10/14` cue (queued through `pending_proj_sounds`, the
  S1-E-21h seam) are the gameplay state that lands.
- The OnHit direction arms are RNG-free state writes; they run post-strike
  in the port (the aiStyle-16 precedent, `projectile.rs:23083-23100`: the
  direction/fuse are read by the NEXT tick's AI, no draw between).
- `Damage_PVP`/`Damage_EVP`'s aiStyle 7/13/69 tails (`:14698-14715`) are
  the hostile-player-contact path; the port's single-player `damage_player`
  (`:23407`) kills the shot on contact and does not model the PVP latch —
  recorded, not ported here (out of the unclaimed-rowed scope).
