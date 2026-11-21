# 107 — WG-SOUND FINAL: the co-lane sync + the census re-sweep (W4-g24)

Status: ✅ code-only (no cargo/rustc/rustfmt/tests/graphify invoked; witnesses
WRITTEN NOT RUN). Branch `task/W4-g24-sounds-worldgen`, the FINAL phase on top
of the doc-sweep (`7ca8b087`, research/106). Four items: the b3 co-lane sync,
the 83-site census FINAL re-verify, the secret-seed menu-cue census vs the
g23 menu union, and the gen-progress screen sound census (d85/f23 co-lanes).

---

## §1 — b3 co-lane sync: adoption status + the keep-one (item 1)

**Re-read**: `origin/task/W4-b3-npc-ai-town-wire` @ `96a5d88f` (fetched fresh;
the branch tip is UNCHANGED since research/106 §4 — b3 phase 2 has NOT
landed). `npc.rs::ai_007_town_family` (`:10121`) still builds the solo
`TownEnv` + `TownDoorLatch` and passes `let mut doors = NoOpTownDoors;`
(`:10200`) into `town_npc_ai`. The `NoOpTownDoors` impl (`npc.rs:40-52`)
returns false on all three methods — the arm takes the C#'s all-fail
direction flip.

**Keep-one adjudication**: there is NO competing door-sound implementation
to reconcile. b3 did NOT wire its own doors-adapter — its `doors.rs`
interact/zombie-bash cue sites (`:372`/`:376`, the landed S1-E-21i
`play_door_sound` seams) are untouched shared heritage, not new b3 work.
The ONLY town-seam impls are b3's placeholder `NoOpTownDoors` (silent by
design, branch-local) and g24's sound-complete `AppTownDoors`
(`world_sim/editing/doors.rs:785`). **`AppTownDoors` is the survivor.**

**The published contract** (`npc_ai_town.rs` trait doc, repointed):
1. the swap is ONE binding at the b3 call site — `let mut doors =
   NoOpTownDoors;` → `let mut doors = AppTownDoors { app };` — plus the
   caller re-shape (b3's arm takes `world: &World, player: &Player`
   snapshots; the adapter holds `&mut App`, so the phase-2 caller folds
   those borrows into the App first, the `ai_002_eye_family` precedent);
2. the `&mut dyn TownDoorOps` object-safety both impls satisfy
   (`town_npc_ai`'s `doors` param, `npc_ai_town.rs:1520`);
3. the cue contract: open → pitched 8 at `(x*16, y*16)` (`WorldGen.cs:37727`),
   close → pitched 9 (`:31908`), gate → pitched 8 at the returned cue
   position (`:51762`, both directions one play site); success-only, one
   `Next(-20,21)*0.01` pitch draw per play on `UpdateNpcs` (the C# plays
   inside `NPC.UpdateNPC` under the `Main.cs:17767` swap — the same stream
   the b3 caller passes, `world_sim/npcs.rs`); wrong-kind refusals silent
   and draw-free; the cleanup drops spawn on the same stream.

**Witness**: `doors.rs::w4_g24_the_b3_adoption_contract_is_dyn_compatible`
(the dyn coercion + one cue per success through the trait object).

---

## §2 — the PlaySound census FINAL re-sweep (item 2)

**The re-sweep** (script: `Select-String 'SoundEngine.PlaySound'
-CaseSensitive` over `decompiled/Terraria/WorldGen.cs`, ascending):
**79 direct play sites** + 6 table/call-seam lines (`:60385`/`:60423`
the KillWall call/def, `:62550`/`:62575`/`:63558` the three table call
seams, `:66427` the KillTile def) = 85 case-sensitive `PlaySound` lines;
the case-insensitive sweep adds the 3 `playSound` parameter lines
(`:527`/`:537`/`:11068`) = 88. The base claim's "83" was the raw
approximate sweep; the authoritative count is **79 play sites, and every
one is claimed**. ZERO now-unclaimed sites after all lanes landed; the
"small remainders" reduce to two citation repairs (below), no new code.

**The full ledger** (79 sites, C# line → port seam):

| C# sites | count | owner / port seam |
| --- | --- | --- |
| `:4636`/`:4639`/`:4642`/`:4645`/`:4648`/`:4651` | 6 | S1-E-25 `play_liquid_change` (`audio.rs:1009`, drained `world_sim/liquids.rs:27`) |
| `:539` | 1 | g24 batch 1 — `SecretSeed.Enable` tracked MenuAccept (`app/mod.rs:2581`, `audio.rs:518 menu_accept_style`) |
| `:6593` | 1 | g24 batch 1 — `worldGenCallback` head (`session.rs:61`, `finish_creating_world`) |
| `:6603` | 1 | S1-E-17 — the completion tail (`session.rs:217`, `update_world_load`) |
| `:6631`/`:6632` | 2 | g24 doc-sweep — `JustQuit`'s inline sentinel pair (`saveglue.rs::just_quit`) |
| `:6676`/`:6703` | 2 | pre-wave — `save_and_quit` (`saveglue.rs:375`/:348) |
| `:6693`/`:6694` | 2 | g24 doc-sweep — `stop_waterfall_ambience` (`audio.rs:2199`, wired save_and_quit + just_quit) |
| `:6843` | 1 | g24 batch 1 — `playWorldCallBack` tail (`app/mod.rs:2770`, the world-enter close) |
| `:6921` | 1 | g24 batch 1 — the Host action (`clicks.rs:2771`; the dedServ ladder DOC-SKIP, research/106 §3) |
| `:31908`/`:37727` | 2 | S1-E-21i `play_door_sound` (`audio.rs:2059`; the town seam rides `AppTownDoors`, §1) |
| `:37614` | 1 | g24 batch 1 — `fire_unlock_door` (`doors.rs:116`, draw-free case 22) |
| `:47235`/`:47255` | 2 | g24 batch 1 + doc-sweep order fix — `check_cues` piggy/bowl (`kill.rs:168`, after-loop) |
| `:51441` | 1 | g24 batch 1 coverage — `PlaceObject`'s plain 0 rides `place_tile` (`editing.rs:8161` witness) |
| `:51470`/`:51512`/`:51563`/`:51594` | 4 | g24 batch 2 — `shift_trapdoor` + refuse (`edit.rs:821`/:854/:917/:970) |
| `:51762` | 1 | g24 batch 2 — `shift_tall_gate` (`edit.rs:982`, one play site both directions) |
| `:54337`/`:54341` | 2 | g24 batch 1 — `CheckOrb` crimson-4/shadow-13 (`kill.rs:169`, trigger cell) |
| `:56858`/`:56862`/`:56866` | 3 | pre-wave — `pot_break_sound` (`kill.rs:2353`, drained `editing.rs` after the hit cues) |
| `:58015`/`:58026`/`:58115`/`:58126`/`:58145`/`:58156`/`:58175`/`:58186`/`:58205`/`:58216` | 10 | g24 batch 3 — the wiring-tool Dig-0 case-0 upgrade (`world_sim/editing/wiring.rs:318`, cue BEFORE the drop roll) |
| `:60268`/`:60271`/`:60277`/`:60280` | 4 | S1-E-21b — `place_tile_sound` (`editing.rs:7571`, the PlaceTile tail switch) |
| `:60427`/`:60431`/`:60435`/`:60439` | 4 | S1-E-21i/c15 — `kill_wall_sound` (`edit/wall.rs:22`, both call sites covered — below) |
| `:66436`-`:66571` (25 rows) | 25 | S1-E-21c — `kill_tile_sound` (`edit/kill.rs:2384`, the whole type-arm table) |
| `:75141` | 1 | pre-wave — `PlaceWall`'s Dig 0 (`editing.rs:5460` + the fill tail) |
| `:80782`/`:80822` | 2 | i31/i42 — SlopeTile/PoundTile Dig 0 (`editing.rs:1402`, inside the gate) |

Sum: 6+1+1+1+2+2+2+1+1+2+1+2+1+4+1+2+3+10+4+4+25+1+2 = **79**. ✅

**The two table call-seam re-verifications** (the citation repairs this
round landed):
1. `KillWall_PlaySounds` has TWO C# call sites — `KillWall` `:60385` and
   `ReplaceWall` `:62550`. BOTH are covered: the break seam and the
   S1-P-i33 replacement arm (`place_wall_route`, `editing.rs:5375-5392` —
   the OLD wall's cue before its drop on `UpdatePlayers`). The
   `edit/wall.rs` table doc now cites both.
2. `KillTile_PlaySounds` has TWO C# call sites — `KillTile` `:63558` and
   `ReplaceTile` `:62575`. The first is drained at the break seam
   (`editing.rs:5628`, `fail: false`) + the zombie-bash fail seam
   (`npcs.rs:4715`, `fail: true`). The second targets the **unported
   `WorldGen.ReplaceTile` surface** (`Player.cs:39903`'s tile-replacement
   arm) — the standing recorded deviation (ANALYSIS "Liquid settlement":
   the port takes the C#'s own `KillTile` + forced-`PlaceTile(mute,forced)`
   fallback, `:1363-1364`). NOT an unclaimed sound: the cue table itself
   is fully ported; when the ReplaceTile surface lands it reuses
   `kill_tile_sound`, zero new audio work. The `edit/kill.rs` doc now
   carries the call-site census.

**Verdict**: no site unclaimed, no remainder to port. The wave + doc-sweep
+ the landed prior waves cover all 79.

---

## §3 — the secret-seed menu cues vs the g23 menu union (item 3)

**The g23 read** (`945391e3`, `origin/task/W4-g23-sounds-main-3`, the
canonical menu-sounds union): g23 owns the **Main.cs legacy menu** windows
only (g22 `:48489-49727` ∪ g23 `:49734-51052` + the `:52059`/`:51952`
submit/cancel arms), unified onto `Menu::pending_cue` (one activation
drain) with `pending_ticks` kept DRAW-TIME chrome only. Its scope note
explicitly defers the **888 UIStates cues** (character/world creation incl.
the size-row blanket, per-element plays) to **g22b** — no g22b branch was
pushed at union time. **Zero queue overlap** with the secret-seed surface:
the seed screens ride the 888 `FrontScreen` flow (`handle_front_click`'s
blanket 10/11 + the per-arm plays), never the legacy `Menu` queues. The
keep-one is structural, not a dedupe: one `menu_accept_style()` seam
(`audio.rs:518`) serves every `SecretSeed.Enable` caller.

**The secret-seed UI census** (every `PlaySound` on the seed surfaces):

| C# site | cue | port status |
| --- | --- | --- |
| `WorldGen.cs:539` (`Enable`, tracked `SoundID.MenuAccept`) | MenuAccept | g24 batch 1 (seed-submit) + THIS round (the single-text arm); the d85 row-toggle adoption is the contract below |
| `UIWorldCreation.cs:834` (`OnFinishedSettingSeed` classic-option `SelectOption`) | 24 Chat | **PORTED this round** (`app/mod.rs` WorldSeed arm — the batch-1 deferral closed; case 24 draw-free, `LegacySoundPlayer.cs:957-962`) |
| `UIWorldCreation.cs:836-841` (the SINGLE unjoined micro text's `Enable`) | MenuAccept | **PORTED this round** (the batch-1 gate counted only `\|`-joined parts — a plain "pumpkinseason" submit was silent; now plays the tracked MenuAccept, the stateless once-per-submit convention) |
| `UIWorldCreation.cs:672`/`:691`/`:704`/`:712` (GoBack 11, SetName 10, OpenSeedInputMenu 10, NamingAndCreating 10) | 11/10 | landed-by-equivalent (the flow blanket: `WorldCreateBack` → `Menu_Close`, the rest → `Menu_Open`, `flow.rs:1267`/:1269 — the g23 union verified the :672 row) |
| `UIWorldCreation.cs:678` (`FadedMouseOver`) | 12 | the 888 hover gap — `front_hit` covers only the two select screens (`flow.rs:1213`), so the creation screens' hover edges are silent; **g22b-owned** per the g23 scope note (the standing blanket deviation's sibling) |
| `UIWorldCreationAdvanced.cs:210` (`SecretSeedButton_OnLeftClick`) | 12 | d85 routes `Click::SecretSeeds` → `FlowClick::SecretSeedsOpen` → the blanket **10** — the 888 blanket deviation (g22b owns the faithful 12); recorded, not invented |
| `UIWorldCreationAdvanced.cs:479`/`:485` (GoBack 11 / FadedMouseOver 12) | 11/12 | d85: the 11 landed (`WorldAdvancedBack` → `Menu_Close`); the 12 is the same g22b hover gap |
| `UIWorldCreationAdvancedSecretSeedsList.cs:388`/`:395` (GoBack 11 / FadedMouseOver 12) | 11/12 | d85: the 11 landed (`SecretSeedsBack` → `Menu_Close`); the 12 is the hover gap |
| `GroupOptionButton.cs:216`/`:255` (the seed-row hover / LeftMouseDown) | 12/12 | d85: the click 12 landed (`flow.rs` Row arm plays `Menu_Tick` before `toggle_row`); the DrawSelf hover 12 is the hover gap |

**The d85 keep-one contract (the row-toggle MenuAccept)**: d85's
`ClickSecretSeed` port (`ui/secret_seeds_list.rs::toggle_row` + the flow
Row arm) plays ONLY the button's 12 — the C# `ClickSecretSeed`
(`UIWorldCreationAdvancedSecretSeedsList.cs:232`) ALSO calls
`WorldGen.SecretSeed.Enable(optionValue)` with the default
`playSound: true`, so an ENABLE transition plays the tracked MenuAccept
(`WorldGen.cs:539`) on top of the 12; a DISABLE is silent
(`Disable` has no cue, `:544-551`). Adoption note for the d85/g24 merger:
the Row arm's enable branch calls
`app.audio.play_tracked_global(crate::audio::menu_accept_style())` — the
SAME seam the submit arm uses (keep-one: one style fn, one bank row, no
second MenuAccept impl). The C#'s `!seed.Enabled` session gate collapses
to the enable-transition edge on d85's draft state (the port is stateless
per-screen, the batch-1 convention). The particles
(`SpawnParticles`, `:238-255`) stay d85's recorded visual skip.

**Witness**: `flow.rs::w4_g24_the_classic_option_seed_plays_the_chat_cue`
(the classic 24 + the arm exclusivity + the single-text MenuAccept).

---
