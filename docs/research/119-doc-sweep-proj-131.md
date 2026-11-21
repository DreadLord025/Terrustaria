# 119 — DOC-SWEEP LANE 35: proj aiStyles 131-140 / DD2 cross-branch chains

Branch `task/W4-d14-proj-ai-131-140` (worktree `TR-W4-d14`), base `a725acb0`,
DONE-phase tip `eaa05ae9`. This sweep re-verifies six cross-branch chains
end-to-end against the decompiled C# and ports ONLY the real gaps, each
with witnesses. Code-only mode: **no cargo/tests/graphify invoked** — every
witness is WRITTEN NOT RUN, re-derived from the cited C# windows by hand
arithmetic (the lane's constraint). C# reads stayed in ≤100-line windows;
junctions were never recursed (direct file paths only).

## Commit ledger (this sweep, on top of `eaa05ae9`)

| item | commit | scope |
| --- | --- | --- |
| 6 | `d0ec3faa` | ogre throw re-derived to the AI_107 defaults + 4 witnesses |
| 3a | `c52dd2af` | Betsy breath witness repointed to the ai-shot list |
| 3b | `a3ef70f3` | MonkStaff T1 item-use witness + the d15-adjacent gap record |
| 4 | `143f43d0` | 673/674 local-owned drain + the heal-scan witness |
| 2 | `23ece031` | stale "672 deferred" comment corrected + the loss-spawn witness |
| 5 | `7c938c0e` | the dedServ velocity-quirk rationale (ANALYSIS + census) |
| 1 | `5ef26b07` | the d15 141-claim cross-note + `UNARMED_ROWED` reconcile |
| docs | (this commit) | report 119 + census/PORT_MAP/anchors updates |

## Item 1 — the 698 explosion / aiStyle-141 cross-claim (d15)

**Chain:** the 140 arm's ground-impact spawn (`Projectile.cs:69500`) →
kind 698 `MonkStaffT1Explosion` (aiStyle 141, row `:7385-7401`) → d15's
`AI_141_MonkStaffT1Explosion` (`:69602-69620` + impact helpers
`:69680-69772`).

**Verified:** `origin/task/W4-d15-proj-ai-141-150` `2cea7454` lands the
141 arm (`projectile_ai_dd2.rs`) with a **field-identical** 698 row —
`(75, 200, 141, -1, 5, 0, false, false, false, (true, -1, false, -1, 255,
1.0))`, byte-for-byte d14's `MONK_STAFF_T1_EXPLOSION` (`data.rs`). d15's
final phase (`053a1118`) arms all ten 141-150 styles. Its coverage pin
takes **140** as the rowed-but-unarmed deferral (`90b368a4` — the mirror
of d14's 141 entry): d15's `UNARMED_ROWED` is `[140, 156, 164, 169, 206]`;
d14's is `[141, 156, 164, 169, 206]` (`projectile.rs:24557`).

**Gap ported:** none (the chain is complete on both sides). The merge
reconciliation is recorded: keep-one 698 row/const dedupe, keep BOTH arms
(140 d14's, 141 d15's), and the pin drops both deferrals → merged
`[156, 164, 169, 206]` (d15's `b1d2a416` cross-note agrees from its side).
Both branches are in flight — `merge-base --is-ancestor` against
`origin/main` exits 1 for d15 (`2cea7454`), d124 (`56fa087e`) and d33
(`34399181`); the claims are not landed on main.

## Item 2 — DD2 crystal-death → loss chain (vs d124 `56fa087e`)

**Chain:** `NPC.checkDead`'s intercept (`NPC.cs:78087-78095`) → the loss
scene (`ai[1] = 1`, `life = lifeMax`, `:40860-40967`) → the portal marks
(`:40870-40879`) → `DD2Event.ReportLoss` (`:40882` → `DD2Event.cs:507-515`)
→ the 672 defeat marker (`:40883`) → the 600-tick stop + `WipeEntities`
(`:40954-40967` → `DD2Event.cs:293-312`/`:629-638`).

**Verified:** `npc.rs::update_eternia_crystal` (the `ai[1] == 1`
rise/fade/600-tick stop) + `world_sim/npcs.rs::update_dd2_event` (the loss
intercept, the portal sweep, `report_loss`, the 672 spawn, the stop/wipe).
d124's `56fa087e` (starter crystals + tier persistence) confines its
`world_sim/npcs.rs` diff to `crystal_summon_tests` (`@@ -11639`) — low
collision with the loss-scene work; no overlap to reconcile.

**Gap ported:** the stale comment at `update_dd2_event` claimed the 672
was "deferred" — it IS spawned (only the scene filters/sound/gore drama
skip). Corrected + a witness: the loss entry spawns exactly one 672,
local-owned, at `Centre − UnitY·40` converted to the top-left
((2415, 1767) for the fixture crystal), zero velocity/damage, and the
scene's later ticks do not re-spawn it (`the_loss_scene_spawns_the_672_
defeat_marker_once`, `dd2_loss_tests`).

## Item 3 — MonkStaff item use + Betsy breath NPC-side spawn

**Chain A (item use):** `Item.cs:33958-33976` (3835: shoot 697, speed 24,
damage 50, kb 7, channel, autoReuse, melee, mana 0) → `ItemCheck_Shoot`'s
generic tail (`Player.cs:49437` — NO per-item arm for 3835) → the 697
aiStyle-140 machine → `StartChanneling` (`Player.cs:52958-52968`).
Census: 3835→697, 3836→699, 3858→707, all `mana: 0`, `channel: true`;
3836 `auto_reuse: false` (`Item.cs:33977-33994` has no autoReuse write).

**Gap ported:** a use-loop witness (`the_monk_staff_t1_use_spawns_the_697_
and_latches_the_channel`, `editing.rs` `use_loop_tests` via
`app_with_staff`): one editing tick spawns the 697 at the local owner,
50 damage, 24 speed, ai (0,0), mana unchanged, `channel` latched with
`channel_item = 3835` and the 697 cancel-key seed. **Recorded unported
(d15-adjacent gaps, NOT ported here):** the 3836 per-item arm
(`Player.cs:48731-48734` — `ai0 = NextFloat() * speed * 0.75 * direction`)
and the 3858 alt-use arm (`:48736-48746` — kind 708 at 1.5× velocity,
`ai0 = (0.3 + 0.7·NextFloat()) * speed * 1.75 * direction`, half damage,
kb +4). The generic tail currently fires 3836/3858 without those seeds.

**Chain B (Betsy breath):** `AI_110_Betsy` phase 3 (`NPC.cs:62019-62029`)
→ `NewProjectile(Center, velocity, 687, num3, 0f, Main.myPlayer, 0f,
base.whoAmI)` → the aiStyle-136 arm anchored to the breathing NPC.

**Gap ported:** the DONE phase's `97c5a984` moved the 687 queue to
`wants_hostile_shot_ai` (`ai1 = self.slot`), but the old witness still
checked `wants_hostile_shot` — a self-inflicted regression. Repointed
(`betsy_flame_breaths_and_volleys_the_fireballs`): the breath now asserts
kind 687, `ai1 == slot`, 35 damage and the ±10 sideways velocity on the
ai-shot list; the volley half (686, plain list) is unchanged.

## Item 4 — the 674 heal proj source (owner-255 gap)

**Chain:** `AI_109_DarkMage` (`NPC.cs:65862-65864`): at `ai[1] == 1` and
`ai[0]` 40/48/56, `WorldUtils.Find(Center.X + direction·240,
Searches.Chain(Down(50), IsSolid))` → `NewProjectile(result.
ToWorldCoordinates(8, 0), Zero, 674, 0, 0f, Main.myPlayer)` → the
aiStyle-133 heal scan (`Projectile.cs:36469-36497`): at 40, heal every
live NPC with damage ≥ 1 / lifeMax ≥ 30 inside 1000 px except 564/565,
min(500, missing), then `Kill()`. The 673 summon marker (`:65846`) shares
the row/arm shape (the opacity ramp, `:36438-36461`).

**Gap ported:** both markers queued on the plain `wants_hostile_shot`
list, whose drain hardcodes owner 255 — the pool's owner gate
(`projectile.rs`, `owner == local_owner || (owner == 255 &&
hostile_local)`) leaves owner-255 slots UNSIMULATED on a client, so the
ramp/heal arms were dead in MP. Fixed with the breath precedent: the
queue sites moved to `wants_hostile_shot_ai` and the drain gives 673/674
`app.local_id` + the centre→top-left conversion (`NewProjectile`
subtracts the half-box, `Projectile.cs:10876-10877`). Witness
(`the_dark_mage_heal_mark_drains_local_owned_and_heals_the_army`,
`dd2_tier_tests`): the heal phase queues the 674 at the scan column's
first solid tile, the drain lands it local-owned at (2579, 1915), and 40
pool ticks heal the wounded goblin min(500, missing), leave the
full-health goblin and the second mage untouched, and kill the marker.
The existing dark-mage witness was repointed to the ai-shot list.

## Item 5 — the dedServ velocity quirk (MP-only rationale)

**Verified:** the 132 state-1 `velocity.Y` ramp
(`min(1, localAI[0]/150) * -0.25`) sits INSIDE `if (!Main.dedServ)`
(`Projectile.cs:36367-36371`), bundled with the two `Filters.Scene`
writes. A dedicated server never moves the marker in state 1; only state
2's unconditional `velocity.Y--` (`:36398`) integrates. The port's sim is
the authoritative server shape, so `ai_elder_wins` keeps the dedServ
behaviour (no state-1 velocity write) — a verbatim source quirk, pinned
by `the_states_advance_and_the_fall_integrates` (`vel.y == 0.0` through
state 1). **Gap ported:** documentation only — the rationale now lives in
ANALYSIS ("W4-d14 doc-sweep — the 672 state-1 dedServ velocity quirk")
and the census's deliberate-skips entry.

## Item 6 — OgreStomp spawn / the ogre AI throw arm census

**Chain:** `AI_107_ImprovedWalkers`' 576/577 arm (`NPC.cs:63029-63122`)
→ the `flag3` throw (`:63881-63913`) → kinds 676 (spit) / 683 (stomp) →
the 683's aiStyle-135 `AI_135_OgreStomp` (`Projectile.cs:68671-68787`).
The 922 QueenSlimeSmash twin's spawn owner is d33's unwired
`npc_ai_style121.rs` (state-4 slam, 40 dmg, `NPC.cs:45552-45562`) —
census-only here (the row + the 135 arm already landed in the DONE phase).

**Gap ported (the real fix):** the first pass used speed **13** / drop
**0.15** — those are the **Drakin's** values (`:63163-63168`), not the
ogre's. The ogre keeps the AI_107 DEFAULTS (`:62948-62962`): `num10 = 11`,
`num15 = 0.1`, `num16 = 0.5`, `num17 = 1`, `num18 = 1`; case 1 fires at
the default `num9 = num8/2 = 35`, case 2 at `num9 = 36` (`:63090`), case 0
sets no `flag3` (no throw). BOTH kinds ride the full shape: the muzzle
aim with the 0.1 ballistic drop, `SafeNormalize(-UnitY) * 11`, the
`RandomVector2(-0.5, 0.5)` jitter (two `NextDouble` draws) and the
`vector5 * num18` launch offset. The charge gate needs
`CanHit || CanHitLine` (`:63857`) alongside grounded + reach; the hop
(`:63914-63924`, case 2 only) clamps an airborne charge to `num2 = 41`
and slams `vy = -13` at `num = 56` — and the C# throw block runs BEFORE
the hop, so the port keeps that order. Four witnesses pin it: the spit
(exact 11-speed/0.1-drop/jitter/offset shape at `ai[0] == 35`), the pound
(683@36 + the hop + the airborne-41 clamp + the same throw shape), the
idle case-0 (charge cycles, never throws) and the LOS gate (a solid wall
blocks the charge entirely).

## Verification (code-only mode)

No cargo/tests/graphify on this branch (the lane's constraint). Each fix
was re-read against the cited C# windows (≤100 lines); each witness is
WRITTEN NOT RUN, its expected values derived by hand from the C#
arithmetic and the fixture geometry. The dispatch coverage pin
(`s1_d_4c_every_rowed_style_has_exactly_one_live_arm`) is unchanged on
this branch — the `UNARMED_ROWED` reconcile to `[156, 164, 169, 206]` is
a MERGE-time step recorded in the pin comment + the census, not applied
here (141 is still unarmed on d14's own tree).