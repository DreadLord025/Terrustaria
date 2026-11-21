# 106 — DOC-SWEEP lane 26: the WorldGen.cs sound deferrals (W4-g24)

Status: ✅ code-only (no cargo/rustc/rustfmt/tests/graphify invoked; witnesses
WRITTEN NOT RUN). Branch `task/W4-g24-sounds-worldgen`, the DOC-SWEEP phase on
top of the landed wave (claim `963496a3`, batches `4baaaeb4`/`90027be9`/
`47b9c66f`, docs `4c410bd1`). This note closes the five deferrals the wave
recorded, ports the REAL gaps, and pins the cross-lane keep-one for the merger.

The five deferrals (from the claim + `docs/ANALYSIS.md` "The WorldGen.cs sound
sites wave (W4-g24)"):

1. `StopWaterfallAmbienceAudio` `:6693-6694` — the 34/35 liquid-loop stops.
2. `JustQuit` `:6631-6632` — the save-quit cue pair (no port caller yet).
3. `serverLoadWorldCallBack`'s retry ladder — MP-only? census.
4. `TownDoorOps` fake → b3 `NoOpTownDoors` (`96a5d88f`) — wire real door sounds.
5. c14 dup cue-order — the ANALYSIS keep-one note.

---

## §1 — StopWaterfallAmbienceAudio: the sentinel stop pair (PORTED)

**C#** (`WorldGen.cs:6689-6699`):

```csharp
public static void StopWaterfallAmbienceAudio() {
    try {
        SoundEngine.PlaySound(34, -1, -1, 0);
        SoundEngine.PlaySound(35, -1, -1, 0);
    } catch { }
}
```

The `(-1, -1, 0)` triple is the **sentinel stop arm** of the two liquid-loop
cases, not a volume play: `LegacySoundPlayer.cs:1257-1262` (case 34) and
`:1287-1292` (case 35) take the `num2 <= 0f || x == -1 || y == -1` branch and
`Stop()` the live `SoundInstanceLiquid[n]` outright — no attenuation, no fade,
no RNG (the case bodies never reach a pitch draw). The only caller is
`SaveAndQuit` (`:6704`); `JustQuit` inlines the same pair (`:6631-6632`).

**Port before**: the frame's ¼ s `Main.Ambience` tick (`frame.rs:300-344`)
re-volumes both loops to 0 once `menu.is_some()` flips the quiet flag — the
same EFFECT, but a quarter-second late and only on the tick boundary. There was
no immediate stop at the quit moment.

**Port now** (`audio.rs`): `Audio::stop_waterfall_ambience()` stops both
`liquid_loops` sinks (the `set_liquid_loop(ch, 0.0, _)` stop arm,
`audio.rs:2155-2160`) and bumps the `liquid_ambience_stops` counter (the
headless test seam — no sink exists without an output device).
`Audio::stop_tracked_sounds()` wraps `SoundPlayer::stop_all`
(`tracked_sound.rs:327`, `SoundPlayer.StopAll` `SoundPlayer.cs:79-86`).

**Wired** into `save_and_quit` (`saveglue.rs`): `stop_waterfall_ambience()` at
the `:6704` position (right after the `Menu_Close` 11), `stop_tracked_sounds()`
at the `:6715` position (after the notifications clear). The g75 union's
`sound::liquid_ambience` consumer (`frame.rs`) is the per-tick RE-VOLUME path;
this seam is the immediate QUIT stop — the two are complementary, not a dup
(g75's `liquid_ambience` returns `None`/`Some(volume,pan)` for the live tick;
the quit stop bypasses it and kills the sinks directly, exactly as the C#
sentinel does).

**Witness**: `audio.rs::w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain`
(the stop counter + the draw-free pin + the tracked-pool drain) and
`saveglue.rs::w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool`.

---

## §2 — JustQuit: the /quit debug command (PORTED)

**Census**: the ONLY vanilla caller of `WorldGen.JustQuit` is the `/quit` debug
chat command — `ToolkitDebugCommands.cs:232-237`:

```csharp
[DebugCommand("quit", "Exit world without saving.", CommandRequirement.Client)]
public static bool QuitCommand(DebugMessage message) { WorldGen.JustQuit(); return true; }
```

(`CommandRequirement.Client` = single-player OR a net client; a full-tree grep
of `decompiled/Terraria` finds no other `JustQuit(` call site.) So the deferral's
"no port caller yet" was accurate for the SAVE-QUIT path but missed the debug
command — the port's F3 debug panel IS the chat dispatcher's stand-in (the
S1-E-29 recorded deviation, the `pauseworld` row's precedent,
`debug_panel.rs:798-803`), so a real consumer exists.

**C# body** (`WorldGen.cs:6627-6662`), in order: the inline 34/35 sentinel pair
(`:6631-6632`), the invasion reset (`:6637-6640`), `flag = netMode == 0`
(`:6641`), the mode-10/gameMenu/`Netplay.Disconnect`/netMode-0 block
(`:6642-6645`), `StopTrackedSounds` (`:6646`), the fast-forward clears
(`:6649-6650`), then the single-player arm's `GoToWorldSelect` +
`player[myPlayer].position = (0,0)` (`:6653-6657`) or the multiplayer arm's
mode 0 (`:6660`). `CaptureInterface.ResetFocus`/`StopPlayTimer`/`UpdateTimeRate`/
`ClearVisualPostProcessEffects` are engine-side (the port keeps no play timer;
the frame's listener flip covers the menu transition).

**Port** (`saveglue.rs::just_quit`): the faithful body — the ambience stop, the
invasion reset, the `net_dropped`/role drop (the `net_disconnect` shape without
its reason banner), the tracked drain, the fast-forward clears, then
`enter_front(FrontScreen::WorldSelect)` + the player-origin reset (single) or the
main-menu reopen (client). `GoToWorldSelect` (`Main.cs:48176-48180`: menuMode 888
+ the world-select UIState) maps to the port's front-screen hop, which re-lists
the worlds.

**Wired**: the F3 World tab gains row 15 "Quit (no save)" (`debug_panel.rs`:
`row_count` 15→16, the `world_rows` entry, `world_click` arm 15 → `app.just_quit()`).

**Witnesses**: `saveglue.rs::w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool`
(the JustQuit half: the stops, the world-select hop, the origin reset, the
fast-forward clear, netMode 0) and `debug_panel.rs::w4_g24_the_quit_row_runs_just_quit`.

---

## §3 — serverLoad retry ladder: DOC-SKIP (dedServ-gated, zero audible cues)

**Census** (`WorldGen.cs:6873-6924`, `serverLoadWorldCallBack`): the retry ladder
is `LoadWorld` → on `loadFailed` a second `LoadWorld` → the `.bak` probe →
`menuMode 200/201` (the load-failure UI, `!dedServ`) or the dedServ console
backup-restore ladder (`:6902-6918`). **There are ZERO `PlaySound` sites inside
the ladder** — the only cue in the whole function is the success-tail
`PlaySound(10)` at `:6921`, which the wave ALREADY landed on the Host action
(`clicks.rs::start_hosting`, `:2771`).

**The gate**: `serverLoadWorld()` (`:6926-6929`) is called ONLY from
`Main.DedServ()` (`Main.cs:5671`) — the dedicated-server CLI startup. On dedServ
every `SoundEngine.PlaySound` returns null (`SoundEngine.cs:104-107`'s
`Main.dedServ || !IsAudioSupported` guard), so even the `:6921` success cue is
SILENT on the only path that runs the ladder. The port hosts on the LIVE world
(`start_hosting`), never re-loading through this ladder.

**Verdict**: SKIP, documented. The ladder is (a) MP/dedServ-only, (b) carries no
audible cue of its own, and (c) its one success cue is already landed on the
port's Host seam. No port work; the load-failure UI (`menuMode 200/201`) is out
of scope (the port's load path has its own error surface).

---

## §4 — TownDoorOps: the REAL adapter (PORTED, the b3 phase-2 seam)

**b3 state** (`origin/task/W4-b3-npc-ai-town-wire` @ `96a5d88f`): phase 1 wired
aiStyle 7 to `npc_ai_town::town_npc_ai` with a NO-OP `TownDoorOps`
(`npc.rs:40-52`, `NoOpTownDoors` — every attempt false, so the arm takes the C#'s
all-fail direction flip). b3's own doc cites the phase-2 plan: implement the
trait over `tr_core::edit::{open_door, close_door}` (+ `ShiftTallGate` "when it
lands"). **`ShiftTallGate` IS now ported** (W4-g24 batch 2, `edit::shift_tall_gate`,
`edit.rs:986`), and b3's "UNPORTED in tr-core" note is stale.

**The seam exists on THIS branch**: the `TownDoorOps` trait (`npc_ai_town.rs:737`)
+ the core `open_door`/`close_door`/`shift_tall_gate` are all present; b3's
`NoOpTownDoors` is branch-local (not in main/this branch), so the real adapter is
ported here as the sound-complete implementation b3 phase 2 adopts.

**Port** (`world_sim/editing/doors.rs::AppTownDoors`): a `pub(crate)` adapter
holding `&mut App`, implementing the three trait methods over the core functions
with the REAL cues on the `UpdateNpcs` stream (the C# plays inside
`NPC.UpdateNPC`, `Main.cs:17767` — the zombie bash drain's convention):
- `open_door` → `edit::open_door` + the pitched 8 at `(x*16, y*16)`
  (`WorldGen.cs:37727`), success-only;
- `close_door` → `edit::close_door(forced:false)` + the pitched 9
  (`:31908`), success-only;
- `shift_tall_gate` → `edit::shift_tall_gate(forced:false)` + the pitched 8 at
  the returned cue position (`:51762`, both directions play the OPEN cue — one
  play site).
Each spawns the cleanup drops on `UpdateNpcs` (`spawn_town_door_drops`, the bash
drain's shape). Wrong-kind refusals are silent and draw-free. The trait doc
(`npc_ai_town.rs`) is repointed to the adapter.

**Witness**: `doors.rs::w4_g24_the_town_door_adapter_plays_the_real_cues` (open →
one `Next(-20,21)` pitch on UpdateNpcs + tile 11; close → the pitched 9 + tile 10;
the 388→389 gate flip + cue; the wrong-kind refusal silent/draw-free).

**Merger note**: when b3 phase 2 lands, `NoOpTownDoors` is replaced by
`AppTownDoors` at the `town_npc_ai` call site; the adapter is already
sound-complete, so no further audio work is needed on that seam.

---

## §5 — c14 dup cue-order: the KEEP-ONE (analysis + order fix)

**The dup**: W4-c14 (`origin/task/W4-c14-tile-break-footstep-sounds`,
`5dc20f8c`) and W4-g24 batch 1 (`4baaaeb4`) BOTH ported the `Check*`-level break
tails (`Check2x1`'s piggy Shatter `:47235`, `CheckOrb`'s crimson-4/shadow-13
`:54337`/:54341) as a `TileKill` side channel:
- c14: `TileKill::check_sounds`, recorded **AFTER** the check's kill loop
  (`c14 kill.rs:515-517`, after `kill_plan_cells`), drained in `editing.rs`
  after the hit/pot cues.
- g24: `TileKill::check_cues`, recorded **BEFORE** the kill loop (the original
  `kill.rs:500`), drained at the same `editing.rs` position. g24 additionally
  ported the case-103 BOWL arm (`:47255`) + its dispatch, which c14 lacks.

**The C# order** (the deciding evidence): `Check2x1` plays its Shatter tail at
`:47235`/:47255, which FOLLOW the `:47108-47116` kill block
(`destroyObject=true; KillTile(num,y); KillTile(num+1,y)`); `CheckOrb` plays
`:54335-54342` AFTER the `:54214-54223` kill loop. So the tail cue lands AFTER
the nested kills' own `KillTile_PlaySounds` cues in play order. **c14's
after-loop position is the faithful one; g24's before-loop position was early.**

**The fix (this branch)**: `check_cues.extend(plan.cues)` moved from before
`kill_plan_cells` to after it (`kill.rs:528`), matching c14's position. The
field/drain docs updated. A two-pair order witness
(`kill.rs::w4_g24_the_check_tail_records_after_the_kill_loop`) pins it: two
adjacent piggy pairs on one trigger break frame left-first
(`SQUARE_FRAME_ORDER`'s (-1,0) precedes (+1,0)), so the LEFT pair's tail records
BEFORE the RIGHT pair's nested kill cues — the before-loop recording would have
interleaved them the other way.

**KEEP-ONE recommendation for the merger**: keep ONE side channel. c14's
`check_sounds` has the faithful position; g24's `check_cues` has the bowl
coverage. The clean fold is to keep g24's `check_cues` (now after-loop, so
position-faithful) AND its bowl arm, and DROP c14's `check_sounds` field + drain
(the two are functionally identical once the order matches; the bowl arm is the
only behavioural delta, and it is g24's). Equivalently, rename c14's field to
`check_cues` and add the bowl arm — either way the survivor is the after-loop,
bowl-covering channel. The merger must NOT keep both (a double play of every
piggy/orb tail). The g24 bowl arm + the case-103 dispatch are unique to g24 and
must survive whichever field name wins.

---

## Verification (code-only)

No cargo/rustc/rustfmt/tests/graphify invoked. Witnesses WRITTEN NOT RUN:
- `audio.rs::w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain`
- `saveglue.rs::w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool`
- `debug_panel.rs::w4_g24_the_quit_row_runs_just_quit`
- `doors.rs::w4_g24_the_town_door_adapter_plays_the_real_cues`
- `kill.rs::w4_g24_the_check_tail_records_after_the_kill_loop`

Visibility/borrow checks done by inspection (the `pub(crate)` seams, the
`AppTownDoors` disjoint-field borrows, the `enter_front`/`FrontScreen`/`NetRole`
paths). Anchors added: `stop_waterfall_ambience`, `just_quit`, `AppTownDoors`;
the `check_cues` row updated for the order fix.
