# 36 — Cinematics, film & the Old One's Army

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Two unrelated C# subsystems share this module.

**(a) The cinematic/film framework** — `decompiled/Terraria/Cinematics/`, 6 files, 687 lines
total: `CinematicManager.cs` (34), `Film.cs` (114), `FrameEvent.cs` (2), `FrameEventData.cs`
(20), `DSTFilm.cs` (156), `DD2Film.cs` (361). `Film` is a *timeline*, not a shot list: it holds
a `List<Sequence>` where each `Sequence` = `(FrameEvent callback, start frame, duration)`; the
"shot" data (x/y, zoom, death/fade) the audit brief anticipates **does not exist in vanilla** —
the two concrete films drive the scene by poking NPC fields (`ai[0]`, `color`, `Opacity`,
`scale`, `rotation`) inside per-frame callbacks, there is no camera/zoom channel at all.
Supporting hooks: `Main.cs:16789` (`CinematicManager.Instance.Update(gameTime)`, the only call
site of the whole framework), `Main.hideUI` (`Main.cs:2085`, toggled by F11 at
`Main.cs:18402-18424`), `Player.isControlledByFilm` (`Player.cs:1816`, consumed at
`Player.cs:24765`), `Terraria/DataStructures/EntitySource_Film.cs`.

**(b) The Old One's Army (OOA)** — `decompiled/Terraria/GameContent/Events/DD2Event.cs`
(1 816 lines) plus the supporting C#: `Terraria/DataStructures/EntitySource_OldOnesArmy.cs`,
`NPCID.cs:254` / `:10432` (`Sets.BelongsToInvasionOldOnesArmy`, 31 ids 548-578),
`NPCID.cs:11773-11833` (the `DD2*` id constants), `Player.cs:3715` (`ZoneOldOneArmy`),
`Player.cs:16720` (`SceneMetrics.CloseEnoughToDD2LanePortal`), `NPC.cs:253`/`:396`
(`NPC.ZoneOldOneArmy`), `Projectile.cs:12947`.

Two corrections to the brief, verified in the source:

* **There is no "Old One" boss in vanilla 1.4.5.8.** `NPCID.cs:11773-11833` lists the OOA
  roster — `DD2EterniaCrystal` 548, `DD2LanePortal` 549, `DD2Bartender` 550, `DD2Betsy` 551,
  then 552-578 — and no `OldOne*` id exists. The event's "boss" is the **Etherian Crystal**
  (548, the thing you defend) plus **Betsy** (551, the tier-3 wave-7 dragon).
* **There is no "film mode" in `Main` and no in-game movie.** `Main.gameMenu` is the
  main-menu flag; nothing in the decompiled tree ever calls `CinematicManager.PlayFilm`
  (`CinematicManager.cs:28`) — the film queue is always empty, so `DD2Film` and `DSTFilm` are
  **orphaned, never-played classes** in 1.4.5.8. A faithful port of them is dead-code parity.

Rust side: `crates/tr-core/src/dd2.rs` (693 lines, the wave tables + the lane-spawn rolls),
`crates/tr-core/src/world.rs:220-274` (the `dd2_*` state), `crates/tr-app/src/world_sim/npcs.rs`
(the event driver + the lane spawn), `crates/tr-app/src/world_sim/editing.rs:7850-7899` (the
Etherian Crystal summon), `crates/tr-app/src/npc.rs` (the 31 OOA kinds' AI + the crystal's
aiStyle-105 machine). **There is no cinematics module in `crates/` at all** — a grep for
`film` / `cinematic` over `crates/**/*.rs` returns nothing.

## Key entities

- `Terraria/Cinematics/Film.cs::Film` (`:7`) — the timeline: `Sequence` (`:9-29`),
  `AddSequence` (`:49`), `AppendSequence` (`:56`), `AddSequences` (`:61`),
  `AppendSequences` (`:69`), `AppendEmptySequence` (`:80`), `AppendKeyFrame` (`:85`),
  `AppendKeyFrames` (`:90`), `AddKeyFrame` (`:100`), `AddKeyFrames` (`:106`),
  `OnUpdate` (`:114-129`), `OnBegin` (`:131`), `OnEnd` (`:136`).
- `Terraria/Cinematics/Film.cs::FrameEventData` (`:3-31`) — `AbsoluteFrame`, `Start`,
  `Duration`, `Frame`, `IsFirstFrame`, `IsLastFrame`, `Remaining`.
- `Terraria/Cinematics/CinematicManager.cs::CinematicManager` (`:6`) — the singleton
  (`Instance` `:8`), the FIFO film list (`:10`), `Update` (`:12-26`), `PlayFilm` (`:28`),
  `StopAll` (`:33`, empty body).
- `Terraria/Cinematics/DD2Film.cs::DD2Film` (`:11`) — the 396-line dryad/ogre/portal cutscene;
  the shot list is the constructor (`:25-50`), the per-frame events `DryadPortalFade`
  (`:199-232`, the fade = colour lerp + `Opacity` + `scale` + the dust), `DryadPortalKnock`
  (`:149-179`), `RemoveEnemyDamage`/`RestoreEnemyDamage` (`:181-197`), `OnBegin` (`:338`),
  `OnEnd` (`:371`).
- `Terraria/Cinematics/DSTFilm.cs::DSTFilm` (`:6`) — the Deerclops cutscene: `BuildSequence`
  (`:35-49`), `PrepareScene` (`:51-68`, it rewrites `Main.time`/weather), `ControlPlayer`
  (`:131-139`), `OnBegin`/`OnEnd` (`:21-33`, `Main.hideUI`).
- `Terraria/GameContent/Events/DD2Event.cs::DD2Event` (`:15`) — the event: `StartInvasion`
  (`:253-291`), `StopInvasion` (`:293`), `WinInvasionInternal` (`:314`), `FindProperDifficulty`
  (`:349`), `CheckProgress` (`:362-480`), `StartVictoryScene` (`:481-505`), `ReportLoss`
  (`:507-515`), `GetInvasionStatus` (`:517`), `GetEnemiesForWave` (`:536`),
  `GetMonsterPointsWorth` (`:546`), `SpawnMonsterFromGate` (`:556`), `SummonCrystal` (`:572`),
  `RaiseGoblins` (`:716`), `FindArenaHitbox` (`:753`), `DropMedals` (`:802`),
  `ShouldDropCrystals` (`:815`), `SummonBetsy` (`:914`), `SetEnemySpawningOnHold` (`:944`),
  `Difficulty_1/2/3_GetEnemiesForWave` (`:953`, `:1132`, `:1437`),
  `Difficulty_1/2/3_GetRequiredWaveKills` (`:977`, `:1162`, `:1468`),
  `Difficulty_1/2/3_GetMonsterPointsWorth` (`:1106`, `:1199`, `:1509`),
  `Difficulty_1/2/3_SpawnMonsterFromGate` (`:1011`, `:1225`, `:1530`),
  `IsStandActive` (`:1765`), `RequestToSkipWaitTime` (`:1779`), `AttemptToSkipWaitTime` (`:1795`).
- `crates/tr-core/src/dd2.rs` — `enemies_for_wave` (`:27`), `required_wave_kills` (`:41`),
  `wave_budget_passed` (`:65`), `check_progress` (`:90`), `enemies_for_wave_t2` (`:164`),
  `enemies_for_wave_t3` (`:179`), `required_wave_kills_t2` (`:195`),
  `required_wave_kills_t3` (`:218`), `find_tier` (`:240`), `lane_spawn_roll_d1/d2/d3`
  (`:342`/`:431`/`:569`), the constant-false `event_ongoing` stub (`:17-19`).
- `crates/tr-app/src/world_sim/npcs.rs` — `dd2_report_kill` (`:1479`),
  `spawn_dd2_from_gate` (`:1591`), `dd2_pool_id` (`:1647`), `update_dd2_event` (`:1690-1715`),
  `report_invasion_kill` (`:1717`).
- `crates/tr-app/src/world_sim/editing.rs::use_etherian_crystal` (`:7850-7899`) — the event's
  only trigger (item 3828 over the Eternia Crystal Stand 466).
- `crates/tr-core/src/world.rs` — `dd2_event_ongoing` (`:222`), `dd2_wave` (`:224`),
  `dd2_wave_kills` (`:226`), `dd2_spawn_hold` (`:229`), `dd2_lost` (`:267`),
  `dd2_lane_rate` (`:270`), `dd2_crystal_alive` (`:274`).

## Dependencies

- Calls into (C#): `Main.hideUI`, `Main.time`/`Main.raining`/`Main.windSpeedCurrent` (DSTFilm),
  `NPC.NewNPC`, `Projectile.NewProjectile`, `WorldGen.SolidTile`, `EmoteBubble.NewBubble`,
  `CombatText.clearAll`, `SoundEngine.PlaySound`, `Main.npc`, `Main.projectile`,
  `NPCUtils.TargetClosestOldOnesInvasion` (`NPC.cs:62997`), `Main.ReportInvasionProgress`
  (`Main.cs:47687`), `NetMessage.SendData` (`:279`, `:286`, `:392`).
- Called by (C#): `Main.cs:16789` (the cinematic update) and
  `Player.cs:32425` (the Eternia Crystal use → `DD2Event.SummonCrystal` → `StartInvasion`),
  `NPC.cs:40959-40966` (the crystal's `ai[0] >= 600` → `StopInvasion`).
- Shared data structures: `NPC.waveKills` / `NPC.waveNumber` / `NPC.totalInvasionPoints`
  (the wave counters, `NPC.cs`), `Main.invasionType` (`INVASION_ID = 3`, `DD2Event.cs:47`),
  `World.dd2_*` (Rust), `NpcKind::DD2*` (Rust, 31 kinds).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Film`/`FrameEvent`/`FrameEventData` timeline | MISSING | — | No `crates/` module; grep `film|cinematic` over `crates/**/*.rs` = 0 hits. Severity: cosmetic (the C# framework is never driven — `PlayFilm` has 0 callers). Port scope S. |
| `CinematicManager` (the queue + the per-frame update) | MISSING | — | `CinematicManager.cs:12-31`; the only hook `Main.cs:16789`. Severity cosmetic; port scope S. |
| `DSTFilm` (the Deerclops cutscene) | MISSING | — | `DSTFilm.cs:1-177`; needs `Main.hideUI`, `Player.isControlledByFilm` (`Player.cs:1816`/`:24765`) and the DST shader item 5113. Dead code in vanilla. Port scope S. |
| `DD2Film` (the dryad/ogre cutscene) | MISSING | — | `DD2Film.cs:1-396`; needs `EmoteBubble`, `CombatText.clearAll`, the dust spawn, `EntitySource_Film`. Dead code in vanilla. Port scope M. |
| `Main.hideUI` + the F11 toggle | not verified | — | `Main.cs:2085`, `:18402-18424`. Not grepped in `crates/` in this pass — open item (T-A). |
| OOA wave rosters (D1) | PORTED | `tr-core/src/dd2.rs::enemies_for_wave` (`:27`) | **Verified verbatim** vs `DD2Event.cs:953-975`: {552}@90, {552,555}@60, {552,555,561}@55, {552,555,561,558}@50, {552,555,561,558,564}@40, default {552}@60 — identical. |
| OOA wave rosters (D2) | PORTED | `dd2.rs::enemies_for_wave_t2` (`:164`) | **Verified verbatim** vs `DD2Event.cs:1132-1160` (7 rows + the default, rates 90/70/60/55/50/45/42). |
| OOA wave rosters (D3) | PORTED | `dd2.rs::enemies_for_wave_t3` (`:179`) | **Verified verbatim** vs `DD2Event.cs:1437-1466` incl. Betsy 551 on wave 7 @90. |
| OOA wave budgets (D1) | PARTIAL | `dd2.rs::required_wave_kills` (`:41`) | 60/80/100/120/140 + the 139 cap match `DD2Event.cs:977-1009`; **missing** the case-6 write-back (`waveNumber = 5`, `currentKillCount = 1`) and the `StartVictoryScene` call; the 139 clamp is unconditional (the C# clamps only when the T1 Dark Mage is *not* downed). Severity behavioural (the win path). |
| OOA wave budgets (D2/D3) | PORTED (unverified) | `dd2.rs::required_wave_kills_t2/t3` (`:195`/`:218`) | The C# D2/D3 budget bodies (`:1162-1195`, `:1468-1500`) were **not re-read in this pass** — the rows match the ROADMAP citation, not a fresh diff. Severity: unknown. |
| `FindProperDifficulty` (the tier pick) | PORTED | `dd2.rs::find_tier` (`:240`) | hardmode+golem → 3, hardmode+mech → 2, else 1 (`DD2Event.cs:349-370`). PASS. |
| `CheckProgress` (the kill → wave advance) | PARTIAL | `world_sim/npcs.rs::dd2_report_kill` (`:1479`) | The wave advance + the 1800-tick break + the invasion-bar report land (`DD2Event.cs:376-404`); **the kill's worth is the constant `KILL_WORTH_T1 = 1.0`** (`:1490`) instead of the per-kind/per-tier `GetMonsterPointsWorth` (`DD2Event.cs:546-554`), and the port **drops kills outside the current wave's roster** (`:1481-1484`) while the C# scores every OOA kill. Severity behavioural. |
| The lane spawn (the 549 portal → the wave pick) | PORTED | `dd2.rs::lane_spawn_roll_d1/d2/d3`, `npcs.rs::spawn_dd2_from_gate` (`:1591`), `npc.rs::update_dd2_lane` | M4-EVENT-18's arm-by-arm port; the single-player live-caps only (the C#'s `ActivePlayersCount` 1.3-scaling loop is documented, not ported). Severity behavioural (multiplayer only). |
| The event trigger (Etherian Crystal 3828 → stand 466) | PORTED | `world_sim/editing.rs::use_etherian_crystal` (`:7850-7899`) | The gates, the stand frame math, the consume and the crystal spawn land (`Player.cs:32425-32444`). PASS. |
| The crystal's win-death (`ai[0] >= 600` → `StopInvasion`) | PORTED | `npc.rs::wants_event_stop` (`:2633`), `npcs.rs::update_dd2_event` (`:1690-1715`) | The flag + the one-pass resolve land (`NPC.cs:40959-40966`). PASS. |
| The event's **loss** path (`ReportLoss`/`StopInvasion(false)`) | MISSING | — | `DD2Event.cs:293-350`, `:507-515`; `update_dd2_event` resolves only the win-death, so a destroyed crystal never clears `dd2_event_ongoing`. Severity behavioural. |
| `StartVictoryScene` (the crystal's victory anim + the portals' close) | MISSING | — | `DD2Event.cs:481-505`. Severity behavioural/visual. |
| The wave-break skip (`RequestToSkipWaitTime`/`IsStandActive`) | MISSING | — | `DD2Event.cs:1765-1801` (interacting with the stand cuts the 1800-tick break to 60). Severity behavioural. |
| `DropMedals` (the wave-4/5 medals) | MISSING | — | `DD2Event.cs:405-420`, `:802-813`. Severity behavioural (loot). |
| The Etherian crystal drops (`ShouldDropCrystals`) | MISSING | — | `DD2Event.cs:79-83`, `:815-913`, `ClearAllDD2EnergyCrystalsInChests` `:666`. Severity behavioural (loot). |
| The arena hitbox / `ShouldBlockBuilding` | MISSING | — | `DD2Event.cs:753-800`. Severity behavioural (building rules). |
| `WipeEntities`/`ClearAllTowersInGame`/`ClearAllDD2HostilesInGame` | MISSING | — | `DD2Event.cs:629-693`, called from `StartInvasion` (`:289`). Severity behavioural. |
| `DD2Event.Save`/`Load` (the downed-tier persistence) | MISSING | — | `DD2Event.cs:151-170` writes the three `DownedInvasionT1/T2/T3` flags; `world.rs` has no dd2 tier fields. Severity determinism/persistence. |
| The boss damage tracker (`DD2Event.DamageTracker`) | MISSING | — | `DD2Event.cs:17-39`, started at `:277`; the kill-time/health HUD has no port. Severity cosmetic. |
| `ZoneOldOneArmy` (player + NPC) / `CloseEnoughToDD2LanePortal` | MISSING | — | `Player.cs:3715`/`:16720`, `NPC.cs:253`/`:396`; the NPC aggro/despawn gate `NPC.cs:799` therefore never applies. Severity behavioural (low). |
| `DD2Event.Ongoing` as a live flag | PARTIAL (constants-off) | `tr-core/src/dd2.rs::event_ongoing` (`:17-19`) | A stub that returns `false`; `editing.rs:945` reads it instead of the world's real `dd2_event_ongoing`. The sentry-persistence gate (`NPC.cs:776`/`:2800`) is therefore off — documented in `dd2.rs:12-16`. Severity behavioural (low). |
| The OOA's invasion id (`INVASION_ID = 3`) | MISSING | — | `DD2Event.cs:47`, `Main.cs:64344`; the port keeps `invasion_type = 0` and only writes `app.invasion_progress*` (`npcs.rs:1499-1500`), so the OOA is outside the invasion framework. Severity behavioural (UI). |
| The 31 OOA NPC kinds (the roster + their AI) | PORTED | `npc.rs` (the aiStyle-105/106/107/108/109/110 arms), `bestiary.rs:776-1010`, `render/entities.rs:333` | The ROADMAP rows M4-EVENT-5d/5e/5f/5g/5h/5i/5k/5m/5n/5o/5p/5q/18/19 cover them; the kinds round-trip the three tables. The AI families themselves are module 25/26's scope. |
| The OOA achievements/emotes | PORTED (data) | `achievements_data.rs:811`, `emotes.rs:152-153` | The data rows exist; the event's completion conditions are not driven by the port's event state. Severity behavioural (low). |

## Tests

- `crates/tr-core/src/dd2.rs` — `the_wave_budgets_climb` (`:119`), `the_wave_rosters_grow`
  (`:134`), `the_kill_progress_advances_the_wave` (`:146`), `the_tier2_rosters_and_budgets_climb`
  (`:255`), `the_tier3_rosters_peak_at_betsy` (`:269`), `the_tier_gates_pick_the_difficulty`
  (`:281`). The rosters' assertions cover D1/D2/D3, the budgets and the tier gates.
- The app-side DD2 tests live in `crates/tr-app/src/world_sim/npcs.rs` / `editing.rs`; their
  names were **not enumerated in this pass** (the ROADMAP rows claim 3 app tests for
  M4-EVENT-5o, 7 for M4-EVENT-18, 7 for M4-EVENT-19).
- **No test covers the cinematics** — there is no Rust code to test.

## Deviations & risks

| # | Deviation | Severity | Admitted in ANALYSIS/ROADMAP? |
| --- | --- | --- | --- |
| D1 | The whole `Terraria/Cinematics/` framework is absent from the port | cosmetic | No ROADMAP row and no ANALYSIS section cover the cinematics — a **documentation gap**, not an acknowledged deviation |
| D2 | The films are dead code in vanilla 1.4.5.8 (`PlayFilm` has 0 call sites) — porting them buys no player-visible behaviour | cosmetic (scope risk) | Not recorded anywhere |
| D3 | `dd2_report_kill` uses a constant worth 1.0 instead of the per-kind/per-tier `GetMonsterPointsWorth` (`DD2Event.cs:546-554`, `:1106`, `:1199`, `:1509`) | behavioural (the wave budget burns at the wrong rate for every tier ≥ 2) | Not admitted |
| D4 | `dd2_report_kill` drops kills whose kind is not in the current wave's roster (`npcs.rs:1481-1484`); the C# scores every OOA kill | behavioural | Not admitted |
| D5 | The D1 wave-6 branch is a bare `1` (`dd2.rs:54`): the C#'s `waveNumber = 5` / `currentKillCount = 1` write-back and its `StartVictoryScene` call (`DD2Event.cs:998-1005`) are missing | behavioural (the win transition) | Partly: ROADMAP M4-EVENT-5o claims "the wave-6 sentinel is the win (`WonThisRun`)" — the sentinel is there, the transition is not |
| D6 | The 139 kill-cap is unconditional (`dd2.rs:48-52`); the C# caps only when the T1 Dark Mage is *not* downed (`DD2Event.cs:993-996`) | behavioural (narrow) | Not admitted |
| D7 | The event's **loss** path is absent → a destroyed Etherian Crystal never ends the event (`DD2Event.cs:507-515` vs `npcs.rs:1690-1715`) | behavioural | Not admitted |
| D8 | `StartVictoryScene` (`:481-505`), the wave-break skip (`:1779-1801`), the medals (`:802`), the crystal drops (`:815`), the arena/building block (`:753`), the entity wipes (`:629-693`) and the tier persistence (`:151-170`) are all missing | behavioural | Not admitted — the ROADMAP rows stop at "the wave engine" |
| D9 | `tr_core::dd2::event_ongoing()` is a constant-false stub (`dd2.rs:17-19`) while `World.dd2_event_ongoing` carries the real flag; `editing.rs:945` reads the stub | behavioural (low) | Yes — `dd2.rs:12-16` ("the stub reports false — which makes the O'A sentries wipable like the rest (recorded in ANALYSIS)") |
| D10 | The OOA never enters the invasion framework (`invasion_type` stays 0 instead of `INVASION_ID = 3`, `DD2Event.cs:47`) → the OOA's wave bar is outside the vanilla's invasion-progress path (`Main.cs:64344`, `:47687-47694`) | behavioural (UI) | Not admitted |
| D11 | `ZoneOldOneArmy` is absent, so the OOA enemies' aggro/despawn gate (`NPC.cs:799`) and the "close to a lane portal" metric (`Player.cs:16720`) do not exist | behavioural (low) | Not admitted |
| D12 | The multiplayer's `ActivePlayersCount` scaling in the lane spawn (the C#'s 1.3× loop in `Difficulty_N_SpawnMonsterFromGate`) is not ported | behavioural (MP only) | Yes — ROADMAP M4-EVENT-18 ("single-player caps only") |
| D13 | The `DD2Event.DamageTracker`'s kill-time/boss-health display is missing | cosmetic | Not admitted |

## Task seeds

- **T-A (S)** — Verify/port the F11 UI-hide toggle (`Main.cs:2085`, `:18402-18424`) and the
  `Main.hideUI` state the films need; record the finding in `docs/PORT_MAP.md`.
- **T-B (S)** — Port the film framework verbatim into `crates/tr-core/src/cinematics.rs`
  (`Film.cs:1-144` + `FrameEvent.cs` + `FrameEventData.cs` + `CinematicManager.cs:1-41`,
  ~150 lines) + a `Main.cs:16789`-equivalent hook. Dead-code parity only — no vanilla
  behaviour depends on it.
- **T-C (S)** — Port `DSTFilm` (`DSTFilm.cs:1-177`): the `Main.hideUI` toggle, the
  `Player.isControlledByFilm` flag + the `Player.cs:24765` control gate, the Deerclops
  (668) / Chester (960) spawns, the DST shader item 5113.
- **T-D (M)** — Port `DD2Film` (`DD2Film.cs:1-396`): the dryad (20)/ogre (576)/portal (549)
  staging, the emote bubbles (87, 3), `CombatText.clearAll`, the fade/dust, the friendly
  flip. Depends on T-B.
- **T-E (M)** — The OOA's loss path: `DD2Event.ReportLoss` + `StopInvasion(false)`
  (`DD2Event.cs:293-350`, `:507-515`) → `update_dd2_event` (`npcs.rs:1690`) must clear
  `dd2_event_ongoing` and broadcast the defeat. Depends on nothing.
- **T-F (S)** — The per-kind/per-tier kill worth: port `GetMonsterPointsWorth`
  (`DD2Event.cs:546-554` → `:1106`, `:1199`, `:1509`) into `dd2.rs` and drop the constant
  `KILL_WORTH_T1` (`dd2.rs:82-84`, `npcs.rs:1490`); also remove the roster gate
  (`npcs.rs:1481-1484`) so every OOA kill scores. Depends on T-F's table only.
- **T-G (S)** — The D1 wave-6 win transition: the `waveNumber`/`currentKillCount` write-back
  and the victory-scene call (`DD2Event.cs:998-1005`) in `dd2.rs::required_wave_kills` +
  `npcs.rs::dd2_report_kill`.
- **T-H (M)** — `DD2Event.StartVictoryScene` (`DD2Event.cs:481-505`): the crystal's
  `ai[0]=2/ai[1]=2` victory animation and the 549 portals' close. Depends on T-G.
- **T-I (S)** — The wave-break skip: `IsStandActive` + `RequestToSkipWaitTime` +
  `AttemptToSkipWaitTime` (`DD2Event.cs:1765-1801`) wired to the stand's tile interaction.
- **T-J (S)** — `DropMedals` (`DD2Event.cs:802-813`, called at `:405-420`) + the wave-4/5
  medal drops → the Defender Medal item 3817.
- **T-K (M)** — The OOA's persistence: `DD2Event.Save`/`Load` (`DD2Event.cs:151-170`) — the
  three downed-tier flags in the world save (M6-SAVE scope).
- **T-L (M)** — The event's cleanups: `WipeEntities`/`ClearAllTowersInGame`/
  `ClearAllDD2HostilesInGame`/`ClearAllDD2EnergyCrystalsInChests` (`DD2Event.cs:629-693`)
  at `StartInvasion` (`:289`), plus the arena hitbox and `ShouldBlockBuilding` (`:753-800`).
- **T-M (S)** — The zone metric: `Player.SceneMetrics.CloseEnoughToDD2LanePortal` →
  `Player.ZoneOldOneArmy` (`Player.cs:3715`/`:16720`) + `NPC.ZoneOldOneArmy`
  (`NPC.cs:253`/`:396`) + the `NPC.cs:799` despawn gate.
- **T-N (S)** — The OOA's invasion id: set `invasion_type = 3` (`DD2Event.cs:47`) and route
  the progress through `invasion_progress_icon = 3` / `invasion_progress_wave`
  (`Main.cs:47687-47694`, `Main.cs:64344`).
- **T-O (S)** — Replace the `DD2Event.Ongoing` stub (`dd2.rs:17-19`) with the world's real
  flag (`editing.rs:945`) and re-enable the sentry-persistence gate (`NPC.cs:776`/`:2800`).
