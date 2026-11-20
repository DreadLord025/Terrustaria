# 26c — Projectile aiStyles 1-105 COVERAGE census (W4-f19)

> Researched 2026-11-20 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Verify-only task: every missing style in 1-105 is CLAIMED by a live d-wave
> branch, so no port lands here (task contract: port only UNCLAIMED arms).

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731-19779`) holds **96 numeric arms** plus ONE kind-guarded
  arm (`_ if projectile_ai_empress::handles(kind)`, `:19584`) that serves
  aiStyles 171/173/179/180 for the five Empress kinds (872/873/874/919/923).
- Claim census: `git branch -a` + `git worktree list` — the branch IS the claim
  (AGENTS.md). All of `task/W4-d1-proj-ai-1-10` … `task/W4-d21-proj-ai-201-210`
  exist and are checked out in sibling worktrees (`TR-W4-d1` … `TR-W4-d21`).
  Pushed so far: d2, d3, d6, d13 (d3 = the 21-30 census `57abe80a`, see finding
  F4). Fresh claims also exist for the leftover pool: `task/W4-g1/g2/g3-proj-ai-missing-a/b/c`
  (worktrees `TR-W4-g1/g2/g3`, all at `a725acb0`, no commits yet).
- C# arm spans: `AI()`'s if/else chain (`:23199-36815`); kind lists parsed from
  `SetDefaults`' `else if (this.type == N)` chain (`:697-10800`).

## Verdict

**63 of 105 styles have a dispatch arm; 42 do not. All 42 missing styles sit
inside claimed d-wave ranges (d1-d11). UNCLAIMED missing arms in 1-105: ZERO.**

## Coverage table (1-105)

P = ported (dispatch line in `projectile.rs`), M = missing (C# body span +
SetDefaults kinds), claim = the d-wave branch that owns the style's decade.
"gate" = kind-gated partial arm; "part" = family partial (rows/types deferred).

| style | state | Rust dispatch / C# body | kinds (M rows) | claim |
| --- | --- | --- | --- | --- |
| 1 | P gate | :16742 (`AI_001` :54002-57825, per-type riders partial) | — | d1 |
| 2 | P | :17486 (`ai_style2`, :23205-23689) | — | d1 |
| 3 | P | :16916 (:23690-23693 → `AI_003_Boomerang`) | — | d1 |
| 4 | P | :17504 (:23694-23829 Vilethorn) | — | d1 |
| 5 | P gate | :16939 (:23830-24099 falling stars, kind-gated) | — | d1 |
| 6 | M | :24100-24228 (×0.95 drag + 180-tick kill) | 10,11,463,1019,1051 | d1 |
| 7 | P | :16979 (:24229-24232 hooks → grapple.rs) | — | d1 |
| 8 | P | :17412 (:24233-24362) | — | d1 |
| 9 | P | :17767 (:24363-24366 → `AI_009`) | — | d1 |
| 10 | P part | :16961 (:24367-24370 → `AI_010`, 736-738 trio only) | — | d1 |
| 11 | M | :24371-24500 (72/86/87 sprite flip + flight) | 18,72,86,87 | d2 |
| 12 | P | :17278 (:24501-24635 Aqua/Golden Shower) | — | d2 |
| 13 | M | :24636-24829 (owner-dead / 2000-px cut) | 23,262,271,273,481 | d2 |
| 14 | P | :17083 (:24830-25307 darts/spears) | — | d2 |
| 15 | P | :18011 (:25308-25311 → `AI_015_Flails`) | — | d2 |
| 16 | P | :17258 (:25312-25315 bombs) | — | d2 |
| 17 | M | :25316-25366 (spiky-ball roll: ×0.98 ground drag, +0.2 gravity) | 43,201-205,527-531 | d2 |
| 18 | P | :17272 (:25367-25476 Demon Scythe) | — | d2 |
| 19 | P | :17171 (:25477-25480 → `AI_019_Spears`) | — | d2 |
| 20 | P | :17411 (:25481-25584 drills) | — | d2 |
| 21 | P | :17512 (:25585-25606 music notes) | — | d3 |
| 22 | P | :17403 (:25607-25860 Ice Rod) | — | d3 |
| 23 | P | :17568 (:25861-25935 Eye Fire) | — | d3 |
| 24 | P | :17413 (:25936-26014 Crystal Storm) | — | d3 |
| 25 | M | :26015-26415 (boulder family; d3 census `57abe80a`) | 99,655,727,1005,1013,1014,1021,1047,1048,1053-1055,1057 | d3 |
| 26 | P | :19526 (:26416-26419 → `AI_026`, slices a/b/c) | — | d3 |
| 27 | P | :17463 (:26420-26554 Unholy Trident) | — | d3 |
| 28 | P | :17456 (:26555-26675 Frost bolt; 967/1026 frame machines skipped — 26b) | — | d3 |
| 29 | P | :17753 (:26676-26679 → `AI_029_GemStaffs`) | — | d3 |
| 30 | P | :17218 (:26680-26785 spore fade) | — | d3 |
| 31 | P | :17296 (:26786-26886 Clentaminator) | — | d4 |
| 32 | M | :26887-27048 (timeLeft=10, 255-NPC scan at ai0≥20) | 155 | d4 |
| 33 | P | :17153 (:27049-27135) | — | d4 |
| 34 | P | :17522 (:27136-27188 fireworks) | — | d4 |
| 35 | M | :27189-27291 (ai0>30 fall + ×0.985 drag, 14 clamp) | 171,475,505,506 | d4 |
| 36 | P | :17598 (:27292-27471 bees/bat) | — | d4 |
| 37 | M | :27472-27499 (spear trap; row deviation — finding F3) | 186 | d4 |
| 38 | M | :27500-27512 (flame jet trap, 6-tick sound cycle) | 187 | d4 |
| 39 | M | :27513-27779 (alpha −50 + owner 2000-px cut) | 190 | d4 |
| 40 | P | :16973 (:27780-27783 leaf blower) | — | d4 |
| 41 | M | :27784-27792 (rand.Next(3) frame + rotation += vel.X×0.01) | 221 | d5 |
| 42 | M | :27793-27859 (crystal leaf, `player.crystalLeaf` gate) | 226 | d5 |
| 43 | M | :27860-27892 (atan2 rotation + π, type-227 sound/dust) | 227 | d5 |
| 44 | P | :17224 (:27893-27949) | — | d5 |
| 45 | P | :17784 (:27950-28106 Nimbus chain) | — | d5 |
| 46 | M | :28107-28199 (shimmer kill + 2400-tick life) | 250,251 | d5 |
| 47 | P | :17771 (:28200-28203 magnet sphere) | — | d5 |
| 48 | P | :17773 (:28204-28242 magnet bolt) | — | d5 |
| 49 | M | :28243-28322 (bunny cannon, ai0 −2/−3 states) | 281 | d5 |
| 50 | P | :17457 (:28323-28401 Inferno Fork) | — | d5 |
| 51 | P | :17468 (:28402-28503 Spectre Staff) | — | d6 |
| 52 | M | :28504-28559 (player-index homing, speed 4) | 298,305 | d6 |
| 53 | P | :18074 (:28560-28563 → `AI_053_FrostHydra`) | — | d6 |
| 54 | P | :19483 (:28564-28772 Raven) | — | d6 |
| 55 | M | :28773-28844 (3-frame counter machine) | 321 | d6 |
| 56 | M | :28845-28866 (rotation=ai0, spriteDirection=−ai1, <16 accel) | 329 | d6 |
| 57 | P | :17230 (:28867-28905 North Pole snowflake) | — | d6 |
| 58 | P gate | :17496 (:28906-28937 SantaNK1 present, kind-gated) | — | d6 |
| 59 | P | :17474 (:28938-28993 Spectre Wrath) | — | d6 |
| 60 | M | :28994-29309 (scale −0.015, death vel ×5) | 358,406,1042 | d6 |
| 61 | P | :19608 (:29310-29313 → `AI_061_FishingBobber`) | — | d7 |
| 62 | P | :19361 (:29314-29317 flying minion) | — | d7 |
| 63 | M | :29318-29418 (owner-inactive kill + anchor machine) | 379 | d7 |
| 64 | P | :19561 (:29419-29533 sharknado cloud) | — | d7 |
| 65 | P | :19568 (:29534-29603 sharknado bolt) | — | d7 |
| 66 | P | :19419 (:29604-30161 Twins) | — | d7 |
| 67 | P gate | :17632 (:30162-30165 → `AI_067_FreakingPirates`, pirate trio + frog + 3 pets) | — | d7 |
| 68 | P | :17574 (:30166-30239 Molotov) | — | d7 |
| 69 | M | :30240-30303 (owner-facing rotation −1.57, dead kill) | 404 | d7 |
| 70 | P | :17546 (:30304-30354 Flairon bubble) | — | d7 |
| 71 | P | :17564 (:30355-30454 Typhoon) | — | d8 |
| 72 | M | :30455-30479 (localAI0>3 alpha −25 floor 50) | 410 | d8 |
| 73 | M | :30480-30604 (tile-338 gate at ai0/ai1 coords) | 419,420,421,422 | d8 |
| 74 | M | :30605-30616 (extraUpdates==1 localAI decay kill) | 434 | d8 |
| 75 | P part | :17708 (:30617-30620 Last Prism / Charged Blaster / 445 drill hold) | — | d8 |
| 76 | M | :30621-30670 (heldProj + mount-type gates) | 441,453 | d8 |
| 77 | P | :17533 (:30671-30761 Electrosphere) | — | d8 |
| 78 | P | :17422 (:30762-30790 → `AI_078` Xenopopper) | — | d8 |
| 79 | P | :17539 (:30791-30910 Saucer deathray) | — | d8 |
| 80 | P | :17542 (:30911-31000 Saucer missile) | — | d8 |
| 81 | M | :31001-31213 (penetrate carry + vel/11 scale, ai0 −1 arm) | 451 | d9 |
| 82 | P | :17809 (:31214-31321 Phantasmal Eye) | — | d9 |
| 83 | P | :17816 (:31322-31376 Phantasmal Sphere) | — | d9 |
| 84 | P | :17823 (:31377-31757 laser family, kind-split) | — | d9 |
| 85 | P | :17874 (:31758-31826 Moon Leech) | — | d9 |
| 86 | M | :31827-31900 (Item120 sound latch + ai0 counter) | 464 | d9 |
| 87 | M | :31901-31936 (position.Y=ai0, height=ai1 anchor) | 482 | d9 |
| 88 | P | :17937 (:31937-32261 Vortex bolt) | — | d9 |
| 89 | M | :32262-32386 (alpha ±12/−5, ai0<300 window) | 490 | d9 |
| 90 | M | :32387-32514 (`player.magicLantern` gate, timeLeft=2) | 492 | d9 |
| 91 | M | :32515-32557 (scale=1−localAI0, 20-px box resize) | 496 | d10 |
| 92 | M | :32558-32820 (1007/511-513/1128 type flags + frame rand) | 511,512,513,1007,1128 | d10 |
| 93 | M | :32821-32882 (alpha −25 fade) | 514 | d10 |
| 94 | M | :32883-32963 (4-tick frame cycler) | 518 | d10 |
| 95 | M | :32964-33003 (localAI0>2 alpha −20 floor 100) | 523 | d10 |
| 96 | M | :33004-33028 (ai0 += 0.6, kill >500) | 524 | d10 |
| 97 | M | :33029-33133 (4-tick frame ladder) | 525 | d10 |
| 98 | M | :33134-33152 (ai0/ai1 target reach kill + normalize) | 526 | d10 |
| 99 | P part | :17951 (:33157-33160 → `AI_099_2_Yoyos`; counterweight sub-arm MISSING — finding F1) | — | d10 |
| 100 | M | :33161-33164 → `AI_100_Medusa` :44476-44632 | 535 | d10 |
| 101 | M | :33165-33221 (20-tick alpha lerp + expectedType switch) | 536,591 | d11 |
| 102 | P | :17879 (:33222-33447 Nebula Eye) | — | d11 |
| 103 | P | :17884 (:33448-33477 Stardust mark) | — | d11 |
| 104 | M | :33478-33511 (ai0==1 scale ×0.995, alpha +3, kill ≥250) | 565 | d11 |
| 105 | M | :33512-33515 → `AI_105_SporeSac` :40043-40192 | 567,568 | d11 |

Missing styles (42): 6, 11, 13, 17, 25, 32, 35, 37, 38, 39, 41, 42, 43, 46,
49, 52, 55, 56, 60, 63, 69, 72, 73, 74, 76, 81, 86, 87, 89, 90, 91, 92, 93,
94, 95, 96, 97, 98, 100, 101, 104, 105. Every one maps to a live d-wave claim
(d1: 6; d2: 11/13/17; d3: 25; d4: 32/35/37/38/39; d5: 41/42/43/46/49;
d6: 52/55/56/60; d7: 63/69; d8: 72/73/74/76; d9: 81/86/87/89/90;
d10: 91-98/100; d11: 101/104/105). No port lands from this task.

## Finding F1 — aiStyle 99's counterweight SUB-arm is missing (claimed: d10)

The C# splits style 99 in TWO arms: `else if ((this.aiStyle == 99 &&
this.type >= 556 && this.type <= 561) || this.type == 1079)` (`:33153-33156`)
→ `AI_099_1_Counterweights` (`:67380-67732`: `timeLeft = 6` refresh, the 250-px
string-orbit machine), THEN `else if (this.aiStyle == 99)` (`:33157-33160`)
→ `AI_099_2_Yoyos` (`:67733+`). The port has ONLY the yoyo arm
(`projectile.rs:17951`, `ai_yoyo`); the counterweight kinds 556-561/1079 have
NO `proj_data` rows (the only mention is the comment at `data.rs:742` — the
draw path knows them, `assets.rs:706-708`, the AI does not). A counterweight
spawned today silently no-ops in `_ => {}`-equivalent fashion (no row → the
spawn itself no-ops). Claimed by `task/W4-d10-proj-ai-91-100` (99 ∈ 91-100);
recorded here so the d10 port covers the sub-arm, not just the yoyo half.

## Finding F2 — four rowed styles lack arms, OUTSIDE the 1-105 census range

The summon-lane wave-3 landing added data rows whose aiStyles have no dispatch
arm (they ride `_ => {}` today — inert if forced active):

| style | kinds (rows) | C# arm | row cites (`projectile/data.rs`) | claim |
| --- | --- | --- | --- | --- |
| 156 | 755 Sanguine bat, 946 Empress Blade | `AI_156_BatOfLight` (`:36587-36590`) | :3400, :3414 | d16 (151-160) |
| 164 | 831/970 Storm Tiger gems | `AI_164_StormTigerGem` (`:36619-36622`) | :3404, :3422 | d17 (161-170) |
| 169 | 864 Smolstar | `AI_169_Smolstars` (`:36639-36642`) | :3409 | d17 (161-170) |
| 206 | 1119 | inline (`:36787+`) | :3443 | d21 (201-210) |

All four styles are CLAIMED by live d-wave branches (d16/d17/d21 worktrees
exist at `a725acb0`). Not ported here (outside the 1-105 scope AND claimed);
recorded so the wave ports know the rows already exist. The neighbouring
Empress styles 171/173/179/180 are NOT missing — their five kinds ride the
kind-guarded empress arm (`projectile.rs:19584`); the style-171 weapon variants
931/1039 have no rows yet (the arm comment at `:19577-19579` says so).

## Finding F3 — style 37's row is a DELIBERATE stand-in (do not double-fix)

The Spear Trap's spear (186) is C# `aiStyle 37`, 10×14, tileCollide false,
ignoreWater, friendly AND hostile, penetrate −1, timeLeft 300, trap
(`Projectile.cs:2728-2740`). The port's row is the 185 spiky-ball stand-in
(14×14, aiStyle **14**, tileCollide true, timeLeft 900 —
`projectile/data.rs:2264-2273`), a RECORDED deviation (ANALYSIS "The projectile
data-row deviations (S1-D-5)"): correcting the row to 37 before the arm exists
would park the spear in `_ => {}` forever. The stale doc comments at
`projectile.rs:301-306` still call 186 "the boulder (aiStyle 37)" — the boulder
is kind 99 / aiStyle 25; 186 is the spear trap. The d4 port (37 ∈ 31-40) must
land the arm AND the row correction together and retire the ANALYSIS row.
