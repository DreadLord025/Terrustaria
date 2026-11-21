# 32 — Sound / audio engine

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The C# side is `Terraria.Audio` — 23 files, 3 193 lines (`decompiled/Terraria/Audio/`):
the static façade `SoundEngine.cs` (223), the legacy playback engine `LegacySoundPlayer.cs`
(1 382, the `type` switch with ~110 `case` labels at `:440-1382`), the XACT music engine
`LegacyAudioSystem.cs` (356), the tracked-sound layer `SoundPlayer.cs` (119) +
`ActiveSound.cs` (166) + `SoundStyle.cs`/`LegacySoundStyle.cs`/`CustomSoundStyle.cs`
(36/97/30), the track wrappers (`MP3AudioTrack.cs` 39, `OGGAudioTrack.cs` 66,
`WAVAudioTrack.cs` 80, `CueAudioTrack.cs` 52, `MusicCueHolder.cs` 125,
`IAudioTrack.cs` 16) and the small helpers (`DisabledAudioSystem.cs` 66,
`IAudioSystem.cs` 26, `SoundPlayOverrides.cs` 5, `SoundType.cs` 7,
`SoundInstanceGarbageCollector.cs` 35, `ProjectileAudioTracker.cs` 24,
`VampireSizzleTracker.cs` 17, `AudioTrackPlayCallback.cs` 2). The style table is
`Terraria/ID/SoundID.cs` (1 173 lines, **506** `LegacySoundStyle` fields) and the track
table is `Terraria/ID/MusicID.cs` (106 constants = 105 tracks + `Count`).

**ReLogic carries no audio at all** in this decompile — a recursive search of
`decompiled-libs/ReLogic/` for `Sound|Audio|Music|Wave|Cue|MP3|OGG` file names returns
zero hits. The XACT engine, `SoundBank`, `WaveBank` and `AudioTrack` live in the external
`Microsoft.Xna.Framework.Audio` assembly, so the "MP3 track loader in ReLogic" premise is
false for 1.4.5.8: `MP3AudioTrack.cs` is a `Terraria.Audio` class, selected by
`LegacyAudioSystem.FindReplacementTrack` (`LegacyAudioSystem.cs:82-122`).

The Rust side is three files — `crates/tr-app/src/audio.rs` (1 496: the cue bank, the
rodio PCM player, the ambient-loop engine, the XACT music system), `crates/tr-app/src/sound.rs`
(231: the pure attenuation maths) and `crates/tr-app/src/ambient_sound.rs` (532: the
ambience emitters) — plus `crates/tr-content/src/sound.rs` (43, the `.xnb` PCM reader),
`crates/tr-content/src/xact.rs` (370) and `crates/tr-content/src/adpcm.rs` (307).

## Key entities

### C#

- `Terraria/Audio/SoundEngine.cs::SoundEngine` — the static façade. `Initialize` `:19-34`
  (a `LegacyAudioSystem` or a `DisabledAudioSystem`), `Load` `:36-43`, `Update` `:45-66`
  (audio-system tick + focus pause/resume), `PlaySound` overloads `:83-109`, the tracked
  family `:120-149`, `StopTrackedSounds` `:151-157`, `GetTrackableSoundByStyleId` `:159-166`,
  `StopAmbientSounds` `:168-174`, `TestAudioSupport` `:185-222`.
- `Terraria/Audio/LegacySoundPlayer.cs::PlaySound` (`:360`) — the 1-1 playback maths:
  the `soundVolume == 0` early-out `:369-372`, the `x == -1 || y == -1` 2D sentinel
  `:376-379`, the worldgen/server guards `:382-389`, the distance attenuation `:390-397`
  (`SoundAttenuationDistance`, `:160`), the pan clamp `:399-406`, the two zero-volume
  early-outs `:411-414` and `:433-436`, the ambient-slider branch `:417-428`, then the
  per-`type` switch (`:438-1382`) that picks the asset family, re-creates the instance and
  sets `Pitch`.
- `Terraria/Audio/LegacySoundPlayer.cs::LoadAll` (`:174-259`) — every asset path:
  `Dig_0..2` `:179-181`, `Thunder_0..5` `:182-187`, `Research_0..3` `:188-191`,
  `Tink_0..2` `:192-194`, `Player_Hit_0..2` `:195-197`, `Female_Hit_0..2` `:198-200`,
  `Player_Killed` `:201`, `Chat` `:202`, `Grass` `:203`, `Door_Opened/Closed` `:204-205`,
  `Menu_Tick/Open/Close` `:206-208`, `Shatter` `:209`, `Camera` `:210`, the `Coin_`/`Drip_`/
  `Zombie_`/`Liquid_`/`Roar_`/`Splash_` families `:211-234`, `Double_Jump` `:235`, `Run` `:236`,
  `Coins` `:237`, `Unlock` `:238`, `MaxMana` `:239`, `Drown` `:240`, the `Item_`/`NPC_Hit_`/
  `NPC_Killed_` families `:241-252` and the trackable `Sounds/Custom/*` list `:253-258`.
- `Terraria/Audio/LegacySoundPlayer.cs::DoesSoundScaleWithAmbientVolume` (`:1440-1460`) —
  the ambient id set `30,31,32,33,34,35,39,43,44,45,46,67,68,69` (the switch at `:1444-1457`).
- `Terraria/Audio/ActiveSound.cs::DetermineIntendedVolume` (`:142-165`) — the tracked-sound
  gain: positional pan + `Style.Volume * Volume` + the `SoundType` slider
  (`Sound`→`soundVolume`, `Ambient`→`ambientVolume`, `Music`→`musicVolume`).
- `Terraria/Audio/SoundStyle.cs::GetRandomPitch` (`:40-42`) —
  `random.NextFloat() * PitchVariance - PitchVariance * 0.5f`.
- `Terraria/Audio/LegacySoundStyle.cs` — `Style` picks a random variation in
  `[style, style+Variations)` `:20-30`; `IsTrackable` is `SoundId == 42` `:32`;
  `WithVolume` `:65-68`; `WithVolumePerceived` (the `pow(v, 1.6609)` curve) `:70-73`;
  `WithPitchVariance` `:75-78`; `AsMusic/AsAmbient/AsSound` `:80-93`.
- `Terraria/Audio/SoundPlayer.cs` — the tracked-sound slot pool
  (`SlotVector<ActiveSound>(4096)` `:9`), the 10 000 000-px² screen cull `:17-20`,
  `GetActiveSoundCount` `:107-119` (the `MaxTrackedInstances` cap, checked by
  `SoundEngine.PlayTrackedSound` `:126-129`).
- `Terraria/Audio/LegacyAudioSystem.cs` — `Init` `:137-149` (the `TerrariaMusic.xgs`
  engine, `Sound Bank.xsb`, `Wave Bank.xwb`, `AudioTracks[maxMusic]`), `LoadCue` `:160-166`,
  `FindReplacementTrack` `:82-122` (`.ogg`/`.wav`/`.mp3` replacement tracks),
  `UpdateAmbientCueState` `:216-248` and `UpdateAmbientCueTowardStopping` `:250-277`
  (the ±0.005 ambient-cue ramps), `IsTrackPlaying` `:279-290`, `UpdateCommonTrack`
  `:292-318` (the `PlayCallbacks[i](i, TrackLoopCounts[i])` hook `:308-311`),
  `UpdateCommonTrackTowardStopping` `:320-352`.
- `Terraria/Main.cs::UpdateAudio` (`:11017-11606`) — the music clock: the
  `ActiveMusicBox` chain `:11076-11350`, the `DecideOnNewMusic` ladder `:12187-12988`,
  the `gameMenu` arm `:12455-12532`, the rain/wind ambience tracks `:11527-11600`,
  `Main.Ambience` `:13466-13515` (the waterfall/lavafall liquid loops).

### Rust

- `crates/tr-app/src/audio.rs::SFX_CUES` (`:22-75`) — the cue bank: 26 entries of
  `(asset name, legacy sound type)`. The type is the C# `LegacySoundPlayer.PlaySound`
  `type` argument (0 dig, 1 player hit, 2 item, 3 npc hit, 7 grab, 8/9 door, 10/11/12 menu,
  15 roar, 16 double jump, 18 coin, 22 unlock, 43 thunder).
- `crates/tr-app/src/audio.rs::AMBIENT_LOOP_CUES` (`:84-85`) — the two blizzard loops with
  their `Style.Volume` (0.5 / 1.0).
- `crates/tr-app/src/audio.rs::Audio` (`:95-120`) — the rodio player:
  `set_listener` `:216-218`, `play` `:221-223`, `played_count` `:229-231`,
  `play_at` `:233-253`, `set_liquid_loop` `:261-294`, `set_ambient_loop` `:307-337`.
- `crates/tr-app/src/audio.rs::MusicEnv` (`:396-500`) / `::MusicSystem` (`:507-528`) —
  the music state: `update` `:587-657`, `decide_new_music` `:663-906`,
  `apply_music_box_override` `:911-919`, `update_common_track` `:922-932`,
  `update_common_track_toward_stopping` `:935-956`, `start_track` `:959-1001`;
  `music_box_track` (`:1016-1123`) is the 100-pair box→track table.
- `crates/tr-app/src/sound.rs::play_sound` (`:63-127`) — the port of
  `LegacySoundPlayer.PlaySound`'s maths; `does_sound_scale_with_ambient_volume` `:20-25`;
  `pan_gains` `:134-141`.
- `crates/tr-app/src/ambient_sound.rs` — `blizzard_loop_volumes` `:69-85`,
  `get_liquid_change_type` `:107-118`, `liquid_change_cue` `:143-177`,
  `thunder_strike_position` `:186-200`, `critter_sound_cue` `:226-289`,
  `thunder_cue_slot` `:296-307`, `thunder_strike` `:313-342`.
- `crates/tr-app/src/app/frame.rs` — the per-frame wiring: `set_listener` `:231-237`,
  the ¼ s liquid-loop tick `:244-288`, the front-end music branch `:295-297`,
  the player-hit cue `:364`, `music.update` `:410`.
- `crates/tr-app/src/app/mod.rs::update_menu_music` (`:937-953`).

## Dependencies

- Calls into: `Main.Camera.Center` / `Main.soundVolume` / `Main.ambientVolume` /
  `Main.musicVolume` / `Main.gameMenu` (the listener), `Main.rand` (the pitch + slot rolls),
  `SceneMetrics` (the zone flags, `ActiveMusicBox`, `MusicBoxSilence`),
  `SoundID`/`MusicID` (the style and track tables), `WorldGen.GetLiquidChangeType`
  (the merge cues), `NPC.UpdateNPC_CritterSounds` (the critter table),
  `Chest.Unlock/Lock`, `CraftingEffects.SpawnEffects_BeforeGrantingItem`,
  `WorldGen.OpenDoor/CloseDoor`, `DelegateMethods` (the minecart cues), `FocusHelper`
  (the pause + thunder gates).
- Called by: ~**1 411** `SoundEngine.PlaySound(...)` call sites and **64**
  `SoundEngine.PlayTrackedSound(...)` call sites across `decompiled/Terraria/`
  (the counts come from a recursive `Select-String` over the decompile).
- Shared data structures: `SoundID`'s 506 `LegacySoundStyle` fields (each carrying a sound
  id, a style/variation range, a volume, a pitch variance and a `SoundType`),
  `MusicID`'s 105 track ids, the XACT `Sound Bank.xsb` → `Wave Bank.xwb` cue→entry map, and
  the 527 `Sounds/*.xnb` PCM assets in `assets/Content/Sounds/`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `LegacySoundPlayer.PlaySound` attenuation (distance, pan, sliders, 2D sentinel, the two zero-volume early-outs) | PORTED | `crates/tr-app/src/sound.rs::play_sound` (`:63-127`) | 1:1 against `LegacySoundPlayer.cs:360-436`; 9 unit tests. Missing: the worldgen/server guards (`:382-389`). |
| `DoesSoundScaleWithAmbientVolume` ambient id set | PORTED | `sound.rs::does_sound_scale_with_ambient_volume` (`:20-25`) | Exact match with the C# switch at `LegacySoundPlayer.cs:1444-1457`. |
| Pan law | PARTIAL | `sound.rs::pan_gains` (`:134-141`) + `audio.rs::Panned` (`:348-388`) | XNA's `Pan` lives in XAudio2 (not in the decompile); the port uses a linear far-channel attenuation, documented as such in the source comment. |
| Per-sound **pitch** (both sources) | MISSING | — | `LegacySoundPlayer.cs:450`/`:476` set `Pitch = Main.rand.Next(-10,11)*0.01f`; `SoundStyle.GetRandomPitch` (`SoundStyle.cs:40-42`) drives the tracked path. The port has no pitch control at all — rodio sources cannot. The thunder roll is consumed and discarded (ANALYSIS:1717-1721). |
| Per-sound **volume** (`LegacySoundStyle.Volume`, `WithVolume`, `WithVolumePerceived`) | MISSING | — | `SoundEngine.cs:99` multiplies `type.Volume * volumeScale`; `WithVolumePerceived` uses the `pow(v,1.6609)` curve (`LegacySoundStyle.cs:72`). The port has a flat `volume_scale` only. |
| **Variation families** (`LegacySoundStyle.Variations`, the random style pick) | MISSING | — | `LegacySoundStyle.cs:20-30` picks `[style, style+Variations)` per play (e.g. `NPC_Hit_1..3`, `Dig_0..2`). The port maps one asset per cue. |
| `SoundID` style table (506 fields) | MISSING | `crates/tr-app/src/audio.rs::SFX_CUES` (`:22-75`) — 26 entries | 13 distinct cue names are actually played (plus thunder via a variable) out of **527** `.xnb` assets on disk. |
| **Sound slot pool / instance reuse** (`SoundInstanceDig[3]`, `SoundItem[]`, `SoundNpcHit[]`, `SoundNpcKilled[]`) | MISSING | — | The C# stops and re-creates the instance per play (`LegacySoundPlayer.cs:445-447`), so one voice per family. The port spawns a fresh rodio source per play, so repeated cues stack. |
| **Tracked-sound layer** (`SoundPlayer`, `ActiveSound`, `SlotId`, `MaxTrackedInstances`) | MISSING | — | `SoundPlayer.cs` (the 4096-slot vector, the 10⁷-px² cull `:17-20`), `ActiveSound.cs` (`DetermineIntendedVolume` `:142-165`), `SoundEngine.PlayTrackedSound` `:120-149`. 64 C# call sites have no port. ANALYSIS:1731-1734 admits it. |
| `SoundPlayOverrides` (`ActiveSound.UseOverrides` `:37-43`) | MISSING | — | 5-line struct, no analogue. |
| `SoundInstanceGarbageCollector` | MISSING | — | 35 lines; no analogue (a rodio sink never dies). |
| `ProjectileAudioTracker` / `VampireSizzleTracker` | MISSING | — | 24 + 17 lines; no Rust analogue found in the audio modules (behaviour not verified in detail). |
| `LegacyAudioSystem` music engine (fade, per-track sinks, `IsTrackPlaying`, the ±0.005 ramps) | PARTIAL | `audio.rs::MusicSystem::update` (`:587-657`), `update_common_track` `:922-932`, `update_common_track_toward_stopping` `:935-956` | The fade table, the `62..=88 → ×0.9` / `52 → ×1.15` volume rules and the `musicFade[curMusic] > 0.25` audible test are ported. Missing: `PlayCallbacks` / `TrackLoopCounts` (`LegacyAudioSystem.cs:308-311`), the `IsPlaying || !IsStopped` nuance (`:326`), and the decode is synchronous per start (ANALYSIS:1688). |
| `Main.UpdateAudio` `DecideOnNewMusic` ladder | PORTED | `audio.rs::decide_new_music` (`:663-906`) | The vanilla order: menu arm → boss → eclipse → shimmer → town → sandstorm → underworld → space → temple → dungeon → mushroom → corrupt/crimson → meteor → graveyard → desert → jungle → snow → underground → hallow → storm → ocean → day/night tail. The boss arm only knows the single `boss_music` id (`:701-710`) — the vanilla's per-NPC type→slot map is a stub. |
| Music-box chain (`Main.cs:11076-11350`) | PORTED | `audio.rs::music_box_track` (`:1016-1123`, 100 pairs) + `apply_music_box_override` (`:911-919`) | The override runs after `decide_new_music` (`:599-600`), so the box beats the boss/eclipse arms — the vanilla order. |
| Front-end (menu) music | PORTED | `app/mod.rs::update_menu_music` (`:937-953`) + `app/frame.rs:295-297` | BUG-01, done 2026-11-10. The `gameMenu` arm's drunk/remix/anniversary/default ladder is ported (`audio.rs:682-700`). Parked: `menuMode == 3000`/credits 89, the `titleMusicStyle` 6/50 arms, `netMode == 2`. |
| MP3/OGG/WAV replacement tracks | MISSING | — | `LegacyAudioSystem.FindReplacementTrack` (`:82-122`) + `MP3AudioTrack.cs`/`OGGAudioTrack.cs`/`WAVAudioTrack.cs` load `Music/<name>.{mp3,ogg,wav}` from the content sources. The port reads only the XACT banks. |
| `TerrariaMusic.xgs` XACT engine / `VeryExpensiveWayToGetAllAudioRenderers` | MISSING | `crates/tr-content/src/xact.rs` reads `.xsb`/`.xwb` directly | A deliberate simplification: the `.xgs` container and the renderer details are not parsed. |
| `Main.Ambience` liquid loops (waterfall 34 / lavafall 35) | PORTED | `audio.rs::set_liquid_loop` (`:261-294`) + `app/frame.rs:244-288` | The ¼ s tick, the attenuation, the `strength/50` clamp and the ×0.2/×0.65 multipliers. The pan is baked at creation — XNA re-pans the live instance. |
| Rain / wind ambience tracks (28 / 45) | PORTED | `audio.rs:639-656` | The music-pool tracks with their own ±0.005 fades. |
| Blizzard ambient loops (`SceneState.cs:189-253`) | PORTED | `ambient_sound.rs::blizzard_loop_volumes` (`:69-85`) + `audio.rs::set_ambient_loop` (`:307-337`) + `scene_scan.rs:341/343` | M6-AUDIO-1. The gate is the raw `ZoneRain && ZoneSnow`, the ±0.01 `MoveTowards` ramp, the `min(1,cloudAlpha*2)*outsideWeatherEffectIntensity` split and the `Style.Volume` 0.5/1. |
| Thunder strike | PARTIAL | `ambient_sound.rs::thunder_strike` (`:313-342`) + `weather.rs:132` | The position math, the `Next(2)` flip and the `worldSurface*16-500` clamp are ported. The `case 43` re-roll-on-playing slot loop (`LegacySoundPlayer.cs:454-476`) has no model and the pitch is discarded. |
| Critter sounds (`NPC.cs:92244-92340`) | PARTIAL | `ambient_sound.rs::critter_sound_cue` (`:226-289`) | The table and the roll-consumption contract are ported and tested, but there is **no caller** — the port has no critter NPC kinds (M4-NPC). |
| Fighter ambient moans (`NPC.cs:57336-57367`) + the idle counter (`:57034-57149`) + `NotDiscouraged` (`:60300-60330`) | PORTED (code-only, tests not run) | `ambient_sound.rs::fighter_ambient_moans`/`fighter_idle_counter`/`fighter_not_discouraged` + `audio.rs::play_fighter_moan` + `world_sim/npcs.rs` wiring | S1-E-21 wave E 2026-11-18: the seven independent `Next(N)==0` arms (type 162 double-matches), the whole gate incl. the exclusion list + the `CritterThatCanTurnOnPlayers` default (`NPCID.cs:10436`), cases 14/26/29 gate-first on the `UpdateNPCs` stream; the counter ticks before `update_rng`, the moans after the AI draws before the critter emitter. Deviations: ANALYSIS "The fighter ambience wave". |
| Liquid-change cues (`WorldGen.cs:4632-4650`) | PARTIAL | `ambient_sound.rs::liquid_change_cue` (`:143-177`) | The table is ported, but the merge-tile pass (`Liquid.CreateLiquidMergeTile`, `Liquid.cs:1324`) is not, so no caller. |
| `SoundEngine.Update` focus pause/resume (`:54-63`), `StopAmbientSounds` (`:168-174`) | MISSING | — | `FocusHelper.PauseSounds` is not modelled; the port always claims focus (ANALYSIS:1737-1738). |
| UI / menu clicks (`Menu_Tick`/`Menu_Open`/`Menu_Close`, `Grab`, `Chat`, `Camera`, `Shatter`) | PARTIAL | `audio.rs:47-49`; `clicks.rs`, `flow.rs`, `loop.rs`, `app/mod.rs:1479/1558` | BUG-17 (done 2026-11-10) added the front-end cues. `Chat`/`Camera`/`Shatter` have no port. |
| Chest open/close/unlock, doors, craft grant, minecart cues, dial, boss roar | PARTIAL | `audio.rs:33-58`, `chest.rs:811-813/1241/1343-1383`, `editing.rs` (dig/roar), `extract.rs:62/98`, `items.rs:144-196` | ~64 Rust `play`/`play_at` call sites total, against 1 411 C# call sites. |
| `TrashItem` cue | BROKEN | `clicks.rs:896` | Played by name but **not in the cue table** → no asset, no sound (the source comment already calls it "the recorded gap"). |

### Coverage by sound category

| Category | C# source | Rust | Status |
| --- | --- | --- | --- |
| Item use / pickup | `SoundGrab` `:177`, `SoundItem[]` `:241-244`, `SoundCoin[]` `:211-214` | `play_use_sound` (wave A) + `play_pickup_sound`/`play_coin_ping` (S1-E-21f: cases 7/38, `Coin_0..4` id 38, the GetItem chimes + the revenge moneyPing; the chest transfers on the bare stream) | PORTED (code-only, tests not run — the starGame arm off, the per-fill chime collapse; ANALYSIS "The player-sounds wave") |
| NPC hit / killed | `SoundNpcHit[]` `:245-248`, `SoundNpcKilled[]` `:249-252` | `play_npc_hit_sound`/`play_npc_death_sound` (case 3/4 maths + pitch + pools, S1-E-21c-npchit) | PORTED (the trackable `_0` pin remains) |
| Player hurt / death | `SoundPlayerHit[]` `:195-197`, `SoundFemaleHit[]`, `SoundPlayerKilled` `:201` | `play_hurt_sound`/`play_death_sound` (S1-E-21f: case 1's `Next(3)` slot pick `Player_Hit_0..2`, case 5 at the top-left `:38745`; the S1-E-21b queues drain on `UpdatePlayers`) | PORTED (code-only — the voice variants beyond 1 deferred) |
| Player movement sounds (jumps/splash/flap/drown/grapple) | `Player.cs:20793-21004` (16), `:20846` (Item16), `:27772-27944` (19 + wetCount), `:26835-26858` (Item32), `:23257` (23), `:6408` (UseSound) | `player.rs` queues + `play_double_jump_sound`/`play_splash_sound`/`play_drown_sound` + the Item16/Item32 façades + `grapple::quick_grapple_item_kind` (S1-E-21f, code-only — tests written not run; the wing default-machine stand-in + the any-liquid splash union documented) | PORTED |
| Tile break / dig | `SoundDig[0..2]` `:179-181`, case 0 `:440-452` | `Dig_0/1/2` (`editing.rs`) | PARTIAL (no instance reuse, no pitch) |
| Footsteps / run | `SoundRun` `:236` | `audio.rs::play_step_sound` + `player.rs::spawn_fast_run_particles` | PORTED (S1-E-21 wave D, 2026-11-18 — vanilla has no per-tile-type footstep cues; the fast-run `hermesStepSound` only, `Player.cs:20212-20216`/`:35947-35964`) |
| Item tink / research | `SoundTink[]` `:192-194`, `SoundResearch[]` `:188-191` | — | MISSING |
| Grass / shatter / camera / chat | `SoundGrass` `:203`, `SoundShatter` `:209`, `SoundCamera` `:210`, `SoundChat` `:202` | — | MISSING |
| Liquid loops (waterfall/lava) | case 34/35 `:1249-1336`, `Main.Ambience` | `set_liquid_loop` | PORTED |
| Ambient loops (blizzard) | `SceneState.cs:189-253` | `blizzard_loop_volumes` + `set_ambient_loop` | PORTED |
| Rain / wind loops | `Main.cs:11527-11600` | `audio.rs:639-656` | PORTED |
| Thunder | case 43 `:454-476` + `Main.cs:58747-58766` | `thunder_strike` | PARTIAL |
| Critters | `NPC.cs:92244-92340` | `critter_sound_cue` | PARTIAL (no caller) |
| Fighter ambient moans | `NPC.cs:57336-57367`, cases 14/26/29 `LegacySoundPlayer.cs:793-824`/`:969-978`/`:1008-1050` | `fighter_ambient_moans` + `play_fighter_moan` + the `update_npcs` wiring (S1-E-21 wave E, 2026-11-18, code-only — tests written not run; the duck case-45 arm is table-complete, its call site awaits a duck kind) | PORTED |
| Liquid-merge cues | `WorldGen.cs:4632-4650` | `liquid_change_cue` | PARTIAL (no caller) |
| Music tracks | `LegacyAudioSystem` + `Main.UpdateAudio` | `MusicSystem` | PARTIAL |
| Music box | `Main.cs:11076-11350` | `music_box_track` | PORTED |
| Front-end title music | `Main.cs:12455-12532` | `update_menu_music` | PORTED |
| Tracked sounds | `SoundPlayer`/`ActiveSound` | — | MISSING |
| Custom music packs (.mp3/.ogg/.wav) | `LegacyAudioSystem.cs:82-122` | — | MISSING |

## Tests

`crates/tr-app/src/audio.rs` — 10 tests: `the_ladder_order_beats_the_surface_tail`,
`the_surface_tail_blood_rain_and_wind`, `the_music_box_overrides_the_whole_ladder`,
`the_biome_ladder_matches_the_source_order`, `the_town_gate_needs_three_npcs_and_no_shadow_candle`,
`dw_2_1_item4_cue_maps_to_the_item_legacy_track`, `the_cue_table_carries_the_legacy_ids`,
`the_chest_cue_table_carries_the_legacy_menu_and_unlock_ids`, `the_menu_ladder_picks_the_title_track`,
`the_menu_track_honours_the_zero_volume_gate`.

`crates/tr-app/src/sound.rs` — 9 tests: `ambient_volume_set_matches_the_switch`,
`silent_sound_volume_drops_non_ambient_sounds`,
`volume_falls_off_linearly_over_the_attenuation_distance`,
`pan_is_clamped_to_the_view_half_width`, `negative_coordinates_are_two_dimensional`,
`ambient_sounds_use_the_ambient_slider`, `ambient_sounds_are_silent_in_the_menu`,
`volume_scale_is_applied_last_and_clamped`, `pan_gains_attenuate_the_far_channel`.

`crates/tr-app/src/ambient_sound.rs` — 9 tests: `move_towards_steps_by_the_amount_and_snaps_past_the_target`,
`blizzard_volumes_ramp_and_split`, `liquid_change_types_and_cues`,
`thunder_position_flips_and_clamps_y`, `thunder_skip_sound_blocks_the_cue`,
`thunder_strike_tick_returns_the_position_once`, `thunder_cue_slot_wraps_the_six_family`,
`critter_gates_fire_only_through_their_rolls`, `critter_roll_is_consumed_even_when_the_roll_fails`.

`crates/tr-content/src/xact.rs` — 3 (`mini_format_decodes_the_msadpcm_music_format`,
`reads_the_retail_wave_bank_metadata`, `sound_bank_maps_music_cues_to_wave_entries`);
`crates/tr-content/src/adpcm.rs` — 3 (`stereo_block_header_is_field_interleaved`,
`decodes_retail_music_to_audio`, `decoded_music_is_smooth_across_block_boundaries`).

No test outside these modules exercises the audio path (a recursive grep of
`crates/*/tests/` for `audio|sound|music|cue|sfx` finds nothing) — the ROADMAP's
BUG-17 "2 tests" for the front-end click cues are not in the audio modules either.

## Deviations & risks

| # | Deviation | Severity | Admitted? |
| --- | --- | --- | --- |
| D1 | **No pitch at all.** The C# sets `Pitch` on every legacy instance (e.g. `LegacySoundPlayer.cs:450`, `:476`) and the tracked path adds `Style.GetRandomPitch()` (`ActiveSound.cs:83`, `SoundStyle.cs:40-42`). The port has no pitch control; the thunder pitch roll is consumed and thrown away. | behavioural | Yes (ANALYSIS:1717-1721) — but only for the thunder roll; the general gap is not written down. |
| D2 | **No per-style volume.** `LegacySoundStyle.Volume` / `WithVolume` / `WithVolumePerceived` (`LegacySoundStyle.cs:65-73`) are folded into `SoundEngine.cs:99`; the port has a flat `volume_scale`. The two blizzard loops are the only styles with a non-1 volume and they carry it in a special table (`audio.rs:84-85`). | behavioural | No. |
| D3 | **No variation families.** `LegacySoundStyle.Variations` (`:20-30`) is a per-play random pick across e.g. `NPC_Hit_1..3`, `Dig_0..2`, `Splash_0..3`; the port pins one asset per cue. | behavioural | No. |
| D4 | **No sound instance pool.** The C# pre-creates one instance per family and *stops then re-creates* it per play (`:445-447`), so a second dig cuts the first off. The port spawns a fresh source per play → overlapping cues stack. | behavioural | No. |
| D5 | **The tracked-sound layer is absent** (`SoundPlayer`/`ActiveSound`/`SlotId`, 64 C# call sites, the `MaxTrackedInstances` cap, the 4096-slot pool, `SoundPlayOverrides`). | behavioural | Yes (ANALYSIS:1731-1734) |
| D6 | **Cue coverage.** 26 of 527 shipped `.xnb` assets are in the table; 13 names are actually played. 64 Rust play sites against 1 411 C# `PlaySound` sites + 64 `PlayTrackedSound` sites. Missing families: footsteps/run, tink, research, grass, shatter, camera, chat, drip, zombie, splash, drown, max-mana, double-jump, coins. | scope | Partly — PORT_MAP:240 says "partial" but does not quantify it. |
| D7 | **`TrashItem` is played but has no cue** (`clicks.rs:896`) → silent. | cosmetic | Yes (the source comment). |
| D8 | **No `WorldGen.isGeneratingOrLoadingWorld` / `netMode == 2` guard** in `sound.rs::play_sound` (`LegacySoundPlayer.cs:382-389`) → a positional cue can fire while the world generates. | behavioural | No. |
| D9 | **The music `PlayCallbacks` / `TrackLoopCounts` hook** (`LegacyAudioSystem.cs:308-311`, `:341`) has no port, and `update_common_track_toward_stopping` uses "a sink exists" instead of the C#'s `IsPlaying || !IsStopped` (`:326`). | behavioural (minor) | No. |
| D10 | **The music decode is synchronous** on track start (~5.6 MB of PCM, not cached). | performance | Yes (ANALYSIS:1688-1689) |
| D11 | **Custom music packs (.mp3/.ogg/.wav) are unsupported** (`LegacyAudioSystem.cs:82-122`) and the `.xgs` engine file is never read. | behavioural (cosmetic for the vanilla game) | No — PORT_MAP:251 only says "partial" for the `.xwb`/ADPCM row. |
| D12 | **Focus pause/resume and `StopAmbientSounds`** are not ported (`SoundEngine.cs:54-63`, `:168-174`). | behavioural (minor) | Yes (ANALYSIS:1737-1738) |
| D13 | **The liquid-loop pan is baked at creation** (`audio.rs:257-260`); XNA re-pans the live instance every tick. | behavioural (minor) | Yes (the source comment). |
| D14 | **The critter and liquid-merge cue tables have no caller** — the critter NPC kinds (M4-NPC) and the liquid merge pass (`Liquid.cs:1324`) are unported, so those two emitters are constants-off. | scope | Yes (ANALYSIS:1722-1727) |
| D15 | **The boss-music arm is a stub** — `MusicEnv.boss_music` carries one id; the vanilla scans the screen rect + 5000 px and maps each boss type to a slot (King Slime 50 → 19 → 93, the generic `boss` 1 → 5, plus the event/mech/moon tracks). `audio.rs:701-710` only forwards the id it is given. | behavioural | Partly (`audio.rs:704-707` names the missing tracks in a comment) |
| D16 | **The pan law is an approximation** (XAudio2 is not in the decompile); the port uses a linear far-channel attenuation. | cosmetic | Yes (`sound.rs:129-133`) |
| D17 | **The ROADMAP summary row is stale**: `docs/ROADMAP.md:78` still says "front-end music and UI sounds missing (BUG-01/17)" although both were fixed on 2026-11-10, and it calls the bank "the 8-cue bank" while the table now holds 26 entries. `docs/PORT_MAP.md:240` likewise predates BUG-01/BUG-17. | cosmetic (docs) | No. |
| D18 | The `M6-AUDIO-1` row claims the blizzard/thunder/critter/liquid-merge work; re-reading the Rust confirms the tables and the engine half, but two of the four emitters still have no caller (D14) and the thunder pitch/slot gaps persist (D1). The row's own text admits both. | — | Yes (the row text) |

## Task seeds

1. **S — Port the `SoundID` style table.** Generate a Rust table from
   `Terraria/ID/SoundID.cs` (506 fields) carrying `(sound_id, style, variations, volume,
   pitch_variance, sound_type)` and wire `Audio::play_style` to it. Unlocks D2/D3/D6 and
   removes the ad-hoc `SFX_CUES` id literals. Follows the `tools/GenItemData` convention.
   Depends on nothing.
2. **S — Add pitch to the playback path.** `rodio` has no pitch control; a `Pitch` source
   (resampling) or a per-cue pre-shift is needed. Start with the `case 43` thunder family
   (the roll already exists) and the dig family (`:450`). Fixes D1.
3. **M — Model the sound instance pool.** A per-family "stop the previous instance" map
   mirroring `LegacySoundPlayer`'s `SoundInstance*` arrays and the stop-then-recreate
   semantics (`:445-447`). Fixes D4.
4. **M — Extend the cue coverage in waves.** Priority order by C# call-site count:
   the footsteps/run, tink, NPC-hit/killed families, splash/drip, grass/shatter, chat,
   camera, drown/max-mana. Each wave is a small PR with the `SoundID` table from task 1.
   Fixes D6 (partly).
5. **M — Add the `PlaySound` guards** (`WorldGen.isGeneratingOrLoadingWorld`,
   `Main.netMode == 2`) to `sound.rs::play_sound`, and the focus pause/resume +
   `StopAmbientSounds` to the App frame. Fixes D8/D12.
6. **M — Port the tracked-sound layer** (`ActiveSound`/`SlotId`/`SoundPlayer`): a slot
   vector with the 4096 cap, the 10⁷-px² cull, `MaxTrackedInstances`, the
   `SoundType`-driven slider routing and the `SoundPlayOverrides` volume. Needs a
   long-lived handle per tracked sound — larger than the current one-shot model.
   Fixes D5.
7. **M — Music polish.** The `PlayCallbacks`/`TrackLoopCounts` hook, the
   `IsPlaying || !IsStopped` nuance, the boss→slot map (the vanilla's per-boss-type
   table), the `titleMusicStyle` 6/50 and credits-89 menu arms, and the `.mp3`/`.ogg`/
   `.wav` replacement-track loader. Fixes D9/D11/D15.
8. **S — Wire the two orphaned emitters.** The critter cue table needs the critter NPC
   kinds (M4-NPC) and the liquid-merge cue table needs `Liquid.CreateLiquidMergeTile`
   (`Liquid.cs:1324`); each is a small caller once its dependency lands. Fixes D14.
9. **S — Docs pass.** Refresh `docs/ROADMAP.md:78` (the stale Audio summary row) and
   `docs/PORT_MAP.md:240` (the stale `Audio/*` row) with the quantified coverage
   (26/527 cues, 64/1 475 call sites). Fixes D17.

## The W4 main-sounds family — declaration + coverage update (2026-11-21, W4-g21)

This module's seeds 2/3/4/6 are being executed by the **W4 sound family** —
six sibling lanes off the same base `a725acb04`, all code-only (witnesses
WRITTEN NOT RUN). The family declaration (who owns what, so no lane
re-ports another's seam):

| lane | branch | claim | tip at this update |
| --- | --- | --- | --- |
| **g21** (this) | `task/W4-g21-sounds-main-1` | `Main.cs` `PlaySound` sites **1-90** (`:3244`-`:43936`) + the batch 2-5 hosts (F-keys, quick-stack/sort buttons, house banners, hotbar-lock/block-replace/torch-biome icons) | `ad3282ec6` |
| g22 | `task/W4-g22-sounds-main-2` | sites **91-180** (`:44118`-`:49727`) — the MENU region (`main_sounds.rs`, 151/151 menu sites claimed); RETIRED, direction-B merger instruction published (`research/90-doc-sweep-menu-sounds.md` `250488c34`: g22 base + g23 4-commit fold) | `250488c34` |
| g23 | `task/W4-g23-sounds-main-3` | sites **181-271** (`:49734`-`:66248`); deprecated INTO g22 (g22 RETIRE R1) | `945391e33` |
| g24 | `task/W4-g24-sounds-worldgen` | `WorldGen.cs` sites; folds independently (zero menu overlap) | — |
| g25 | `task/W4-g25-sounds-item-sites` | `Item.cs`/item-use sites | — |
| d79 | `task/W4-d79-sound-pitch-pool` | seeds **2+3** (pitch + the instance pool): the `LegacySoundPlayer` ALL-CASE census + the sibling-rewire contract (`research/W4-d79-pitch-final.md` §F2/§F4) | `03ebefad7` |
| d80 | `task/W4-d80-tracked-sound-layer` | seed **6** (the tracked-sound entity loops on `tracked_sound.rs`) | `075b954d2` |

**g21 coverage delta** (this lane, base → `ad3282ec6` + this receipts
commit): 26 new bare-play sites (12 clicks + 9 loop + 3 ui/mod + 2
world_sim/npcs) = d79's `7da02f439`-era +21 plus batch 5's +5 (3 `Unlock`
clicks + 2 hover-latch ticks) — 21 draw-free menu cues (10/11/12), the 3
`Unlock` (22) icon clicks (draw-free int-façade plays) + 2 Chat; plus the
nurse `Item4` legacy façade (`app/mod.rs` via `play_use_sound`, NOT a
bare play — the style-overload `GetRandomPitch` NextFloat + case 2's ONE
pitch roll) and the batch 2-5 state hosts (the census tables in ANALYSIS
"The Main.cs sound sites wave, part 1"). The Roar fix below REMOVES one
base bare `play_at` (net −1 on the base's 88+2 census).
The d79 §F4 rewire contract claims every new bare site as a pool cue:
the 21 menu sites → `play_menu`, the 2 Chat sites → DELETE (d79's phase-7
seams win), the 3 Unlock sites → `play_single_voice("Unlock")` (case 22,
no draws — the batch-5 addendum to d79's `7da02f439`-era table), the nurse
Item4 → d79's case-2 façade (already the landed `play_use_sound` shape).
The `Roar_0`→`play_roar(1)` Destroyer fix (d79 §F4's keep-d79 ruling) is
APPLIED on this branch (`npcs.rs` `ev.roar` arm + the
`w4_g21_the_destroyer_roar_default_style_is_one` witness); the Chat
DELETEs and the kill-table `play_kill_cue` rewire stay UNION-TIME (d79's
seams do not exist on this base; g21's own witnesses pin the cues).
The g50 `quick_stack.rs` constants-copy keep-one receipt is published on
both trees (g21 canonical, g50 copied — `1c023f0ad`). The g22 menu region
is DISJOINT from this lane's hosts (the shared-file hunk census in
ANALYSIS "The W4-g21 co-lane receipts" §4). Seeds status: 2/3 in flight
(d79), 4 in flight (this family), 6 in flight (d80); 1/5/7/8/9 unchanged.
