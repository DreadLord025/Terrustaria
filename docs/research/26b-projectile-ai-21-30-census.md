# 26b — Projectile aiStyles 21-30 dispatch census (W4-d3)

> Researched 2026-11-20 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` at this revision.

## Census

| style | C# arm | Rust arm | Verdict |
| --- | --- | --- | --- |
| 21 | inline 25585-25606 (music notes) + HandleMovement 19299-19309 | `ai_music_note` (6921), dispatch 17512 | PORTED — verify-only |
| 22 | inline 25607-25860 (Ice Rod) | `ai_ice_block` (7555) + `ice_block_probe_kill` (7694), dispatch 17403 | PORTED — verify-only |
| 23 | inline 25861-25935 (Eye Fire / Flames Trap) | `ai_eye_fire` (11284), dispatch 17568 | PORTED — verify-only |
| 24 | inline 25936-26014 (Crystal Storm) | `ai_crystal_storm` (7730), dispatch 17413 | PORTED — verify-only |
| 25 | inline 26015-26415 (the boulder family) + HandleMovement 18941-19277 + RainbowBoulder helpers 20024-20315 + Shimmer 22504-22515 + Kill arms 70688-70701 / 71192-71255 / 74841-74981 / 75080-75123 | **none** — no arm, no `proj_data` rows for any of the 13 kinds | **MISSING — ported by this task** |
| 26 | `AI_026` (58102) | `projectile_ai026_a/b/c` slices, dispatch 19526 | PORTED — verify-only |
| 27 | inline 26420-26554 (beams) | `ai_unholy_trident` (3983) + `unholy_step` (4077), dispatch 17463 | PORTED — verify-only |
| 28 | inline 26555-26675 (Frost bolt) | `ai_frost_bolt` (7856), dispatch 17456 | PORTED — verify-only (deviation below) |
| 29 | `AI_029_GemStaffs` (37643-37827) | `ai_gem_staff` (8174) + `ai_crystal_pulse`, dispatch 17753 | PORTED — verify-only |
| 30 | inline 26680-26785 (spore fade) | `projectile_ai_spear::ai_spore` + `kill_step`, dispatch 17218 | PORTED — verify-only |

## Verify-only notes (ported arms, no behaviour change)

- **21** — rotation clamp / spriteDirection / pitch-floor decel verbatim; the
  `Main.rand.Next(3)` dust gate (25594) and the soundDelay latch (25600-25605)
  are documented skips (dust/sound layers). The HandleMovement 21 bounce is
  folded into the arm (per-axis probe stand-in).
- **22** — the three phases verbatim incl. the doubled up/down probe walk
  (25769-25778); lava/shimmer kills (25742-25751) and the `Next(30)` dust gate
  (25681) documented skips.
- **23** — verbatim; the dust block's `Next(2)`/`Next(3)` gates (25896-25901)
  ride the documented dust skip.
- **24** — verbatim for the rowed kinds (94/1134 + generic); the 1114 wet-kill
  branch has no row; the 94's `Next(4)` dust gate (25958) rides the dust skip.
- **26** — slices A/B/C cover the rowed kinds incl. the 266 slime minion; the
  row-less pets keep the no-op fall-through (documented).
- **27** — verbatim incl. the hostile twin's accel freeze at 30 and the shared
  pulse; the 114's `Next(2)`+`NextFloat` dust gates (26449) ride the dust skip.
- **28** — **deviation found:** the 967/1026 per-kind frame machines
  (26557-26601: `localAI[0]` cadence, `frameCounter`/`frame` anim,
  `spriteDirection`/`rotation` for 1026) are skipped although both kinds now
  have rows (`HOUNDIUS_SHOOTIUS_FIREBALL` 967, `DEAD_CELLS_BARNACLE_SHOT`
  1026) and the slot carries `frame`/`frame_counter`. Visual-only (no RNG in
  the frame machines; the dust gates ride the dust skip). Left as verify-only
  per the task contract; recorded here for the follow-up.
- **29** — the 619/620/522/731 per-kind branches have no rows; the rowed kinds
  (gem bolts + 521) ride `ActuallyGemStaffs` / the pulse arm faithfully.
- **30** — the 907/335 type arms have no rows; the 131 default arm is verbatim.

## The missing style 25 — the boulder family

Kinds (SetDefaults): 99 Boulder / 727 RollingCactus / 1013 BouncyBoulder /
1014 LifeCrystalBoulder / 1053 LavaBoulder / 1054 SpiderBoulder / 1055
Ghoulder / 1057 BoulderThatSpawnsPet (31×31, friendly+hostile, trap,
penetrate −1; 1054 timeLeft 300, 1055 timeLeft 900 — `:1833-1850`); 1005
MiniBoulder (15×15, `:1851-1860`); 1021 MoonBoulder (31×31, hostile-only,
extraUpdates 1, timeLeft ×3 — `:1861-1870`); 655 BeeHive (31×31,
`:7038-7047`); 1047 RainbowBoulder (31×31, friendly, penetrate −1, timeLeft
3600, ignoreWater — `:9918-9927`); 1048 Poulder (31×31, `:9928-9937`).
The 727's bounce spawns 763 RollingCactusSpike (10×10, aiStyle 1,
extraUpdates 1 — `:7855-7865`).

Ported by this task in `crates/tr-app/src/projectile_ai_boulder.rs` (the
slice-module precedent): the AI arm, the per-kind HandleMovement bounce
ladder, the RainbowBoulder smart-bounce machine, the Shimmer anti-gravity
arm, the Kill() death hooks, the data rows, the dispatch arm and the boulder
statue wiring. See the module doc for the per-kind RNG census and the
documented skips.
