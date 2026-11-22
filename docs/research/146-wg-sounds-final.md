# 146 — WG-SOUND FINAL: the co-lane sync + the census re-sweep (W4-g24)

> Renumbered 107 → 146 in the retire phase (the f21b net-apply lane owns
> research/107; 145 is double-claimed by e19's 16-box repins + g89's rod
> union, 146 is free across every pushed ref and every live worktree —
> the full-refs + on-disk scan). All in-repo refs re-pointed; the FINAL
> commit messages keep the historical `research/107` name.

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

## §4 — the gen-progress screen sounds (item 4, d85/f23 co-lanes read)

**The screen census** (every candidate file swept, `PlaySound`
case-sensitive):

| C# surface | sites | verdict |
| --- | --- | --- |
| `UIWorldLoad.cs` (the loading screen) | **1** — `:57` `PlaySound(11)`, `Update`'s Inventory-abort arm | LANDED: `session.rs::world_load_queue_abort` plays `Menu_Close` once per queue (the `!QueuedAbort` gate `:54`; the pure `world_load.rs::queue_abort_on_inventory` returns the edge, the caller owns the play — the S1-E-17 split). Witnessed: `flow.rs::s1_e_17_create_opens_the_loading_screen_with_a_live_job` (the +1/-0 played-count pin) + `world_load.rs::inventory_trigger_queues_abort_once` |
| `UIGenProgressBar.cs` / `UIProgressBar.cs` (the bars) | **0** | CUE-LESS — the progress draw plays nothing (the per-pass tick is silent in vanilla) |
| `WorldBuilding/*.cs` (the whole gen-pass machinery incl. `WorldGenerator.cs`, `GenPass.cs`, `GenerationProgress.cs`) | **0** | CUE-LESS — no pass-transition sounds exist; the f23 `pass_name_id`/`get_step_icon` table (`GenPassNameID.cs:223-330`) is icon-only |
| `UIWorldGenDebug.cs` (the F5 debug gen screen, `:50-53` opens it) | **0** | CUE-LESS + DEBUG-DEFERRED (f23's port: the elements/icons landed, the screen has no driver — its own doc); zero sound work owed even when it wires |
| `UIDebugLoadoutsList.cs` (the debug loadouts overlay — the task's "UILoadoutMenu") | **1** — `:123` `FadedMouseOver` `PlaySound(12)` | d85 landed the overlay (`ui/debug_loadouts.rs` + `clicks.rs::handle_debug_loadouts_clicks`); the Back close plays `Menu_Close` (`close_debug_loadouts`, the `IngameFancyUI.Close` `:167` cue), the search box plays `Menu_Tick` (`IngameFancyUI.OpenVirtualKeyboard` `:90`), the SearchCancel arms play 12/11 (`UIWrappedSearchBar.cs:153-164`) — all landed on d85. The `:123` HOVER 12 is the same g22b-class hover gap as §3 (the port has no per-element hover-edge dispatch for the overlays); recorded, not invented. The `OpenUIState` open path itself is CUE-LESS (`IngameFancyUI.cs:69-79` — no `PlaySound`) |
| `UIDebugLoadoutItem.cs` (the rows) | **0** | the row apply (`LeftClick` `:143-148`) plays nothing itself — the close's 11 covers it (d85 landed) |
| `EquipmentLoadout.cs` + `Player.TrySwitchingLoadout` (`Player.cs:5660-5683`) | **1** — `:5673` `PlaySound(12)` on the in-game loadout SWAP | the player loadout system is UNPORTED on every lane (`player.rs:1039`/:4889 "no loadouts" — main, d85, f23 all carry the note). NOT a gen-progress sound; the cue rides the loadout-system task when it lands (one `Menu_Tick` at the swap, after `RecordLoadoutChange`, before the net 147 — recorded for that lane) |

**The bracket cues** (already landed, g24 batch 1 / S1-E-17): the
generation-start 10 (`WorldGen.cs:6593`, `session.rs:61`) and the
completion 10 (`:6603`, `session.rs:217`); the world-enter 11 (`:6843`,
`app/mod.rs:2770`) belongs to the play flow, not the gen screen.

**d85/f23 co-lane reads**: d85 @ `1d7d39a05` (the doc-sweep tip) carries the
debug lists + loadout apply with the cues above; f23 @ `adf5c9083` (the
deviation-pass tip) carries `worldgen_debug_elements.rs` /
`worldgen_pass_icons.rs` / `pipeline.rs::pass_name_id` with ZERO sound
sites (grep-verified). No overlap with g24: the abort-11 and the bracket
10s are g24/S1-E-17 seams on main-heritage code, untouched by either lane.

**Verdict**: the gen-progress surface owes NO new cue. The one audible
site (the abort 11) is landed and witnessed; everything else is cue-less
by census or a recorded gap owned by another lane (the overlay hover 12 →
the g22b-class hover work; the loadout-swap 12 → the loadout-system task).
The `world_load.rs` module doc now carries the census so the next reader
does not re-derive it.

---

## Verification (code-only)

No cargo/rustc/rustfmt/tests/graphify invoked. Witnesses WRITTEN NOT RUN:
- `doors.rs::w4_g24_the_b3_adoption_contract_is_dyn_compatible` (§1)
- `flow.rs::w4_g24_the_classic_option_seed_plays_the_chat_cue` (§3)

Behaviour changes this round: the WorldSeed submit arm's two new cue
branches (§3 — the classic 24 + the single-text MenuAccept); both are
2D menu plays with ZERO `Main.rand` draws (case 24
`LegacySoundPlayer.cs:957-962` draw-free; `menu_accept_style` carries
variations 1 / pitch_variance 0 — the tracked path consumes no gameplay
stream), so no RNG-stream drift. The census repairs (§2) and the doc
contracts (§1/§4) are comment-only.

---

## §5 — RETIRE phase: FINAL-FINAL + the wg-sound-family declaration

Four retire items, each verified against the co-lane's LATEST pushed tip
(fetched fresh 2026-11-21):

**1. This doc's renumber (107 → 146).** The f21b net-apply lane owns
research/107. The free-number scan swept every local + origin ref's
`docs/research/` tree AND every live Desktop worktree's on-disk research
dir: **145 is double-claimed** (e19's `145-16box-cross-lane-repins.md`,
local-only, + g89's `145-rod-teleport-union-composition.md`), **146 is
free everywhere** (no ref carries 146+, no worktree file above 145).
f21 latest `5274a097` carries no 107 file/INDEX row yet (its tail census
lives in research/17) — the rename is the pre-emptive keep-one so f21b
lands 107 unobstructed. All four in-repo refs re-pointed (title/banner,
INDEX, ROADMAP ×2, the two code-comment cites). Commit `2678cfae`.

**2. The quit-stop DUP cross-receipt (g73).** g73's latest tip is
`bce01ece8` — its OWN retire adjudication over its closeout `4e7e3f58`.
It rules **g24's `c3eccb77`+`7ca8b087` the SUPERSET** (the direct
sink-kill body — g73's route-through-`set_liquid_loop` no-ops headless
behind the cue/handle gates — + the `liquid_ambience_stops` counter +
`stop_tracked_sounds` + the full `JustQuit` + the F3 row) and has ALREADY
folded that shape onto its branch. **Keep-one: ONE
`stop_waterfall_ambience` fn + ONE `:6704` call site; the merger keeps
g73's folded copies, g24's identical hunks dedupe onto them**; g24's
`SecretSeed.Enable` → `menu_accept_style` production caller stays on
this branch (g73 carries the fn under a dead_code allow until the
merge). g73 retired its closeout witness into g24's
`w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain` on its side —
no witness dup survives. The ONE correction g73 found here is adopted:
the stale `s1_e_29_..._fifteen_rows` witness (pinned 15 after the
doc-sweep's 15→16 row bump) renamed `..._sixteen_rows`, pins 16/16 + the
`Quit (no save)` tail row. Verdict published: ANALYSIS lane-26 §6 +
research/106 retire addendum. Commit `9fb7137d`.

**3. The b3 phase-2 monitor.** `origin/task/W4-b3-npc-ai-town-wire` tip
UNCHANGED at `96a5d88f` (2026-11-20 21:44, phase 1 only).
`npc.rs::ai_007_town_family` still passes `let mut doors =
NoOpTownDoors;` — **the adoption contract (§1) is UNCONSUMED**; nothing
to verify on the b3 side, the contract + its dyn witness
(`w4_g24_the_b3_adoption_contract_is_dyn_compatible`) stand verbatim.
`AppTownDoors` remains the keep-one survivor.

**4. FINAL-FINAL census verdict + the family declaration (below).**

### The wg-sound-family declaration

The WorldGen/menu/quit sound surface is CLOSED across the family lanes;
this branch is DONE and carries no open audio work:

| lane | tip (read) | scope | g24 overlap verdict |
| --- | --- | --- | --- |
| g21 sounds-main-1 | `7da02f43` | Main.cs sound windows A | none — disjoint windows |
| g22 sounds-main-2 | `250488c3` | Main.cs legacy menus `:48489-49727` | none — the g23 union owns the legacy queues (§3) |
| g23 sounds-main-3 | `945391e3` | Main.cs legacy menus `:49734-51052` + submit/cancel, `Menu::pending_cue` union | ZERO queue overlap with the 888 seed screens (§3) |
| g25 sounds-item-sites | `67d8a698` | Item.cs UseSound census (ZERO direct PlaySound sites) | none — its `saveglue` region is the pre-wave heritage, no quit-stop copy |
| g73 waterfall-draw | `bce01ece8` | waterfall/lavafall draw + the ambience consumers | the quit-stop DUP — RESOLVED, item 2 above |
| b3 npc-ai-town-wire | `96a5d88f` | aiStyle-7 phase 1 | the `AppTownDoors` seam — contract published, adoption pending b3 phase 2 (item 3) |
| d85 ui-screens-2 / f23 ui-elements-4 | `1d7d39a0` / `adf5c908` | the 888 screens / gen icons | §4 — zero sound overlap; the row-toggle adoption contract (§3) awaits their merger |

**g24's own ledger**: 79/79 WorldGen.cs play sites claimed (§2), the
quit paths complete (`save_and_quit` + `just_quit` + the F3 row), the
secret-seed surface censused with the two remainders ported (§3), the
gen-progress surface owes nothing (§4). **22 `w4_g24_*` witnesses** on
this branch (15 wave + 5 doc-sweep + 2 FINAL), ALL WRITTEN NOT RUN
(code-only); one edited this phase (the sixteen-rows fix). The recorded
cross-lane gaps are OWNED elsewhere, not owed here: the g22b-class hover
12s (§3/§4), the loadout-swap 12 (the unported loadout system), the d85
row-toggle MenuAccept (adoption contract published), the b3 doors
binding (adoption contract published).

**Retire verdict: WG-SOUND FINAL-FINAL — DONE.** No unclaimed site, no
dup left unadjudicated, no contract left unpublished. The merger's
keep-one instructions live in ANALYSIS lane-26 §5 (c14), §6 (g73 quit
stop) and this doc §1 (b3) / §3 (g23/d85). Retire commits: `2678cfae`
(renumber), `9fb7137d` (quit-stop cross-receipt + witness fix), this
section's commit (b3 monitor + FINAL-FINAL + declaration).
