# Bug report — Terraria-Rust player-facing audit (2026-11-10)

Scope: the 20 defects reported by the owner from actual play, reproduced/diagnosed
against the decompiled 1.4.5.8 sources (`decompiled/`) and the Rust tree
(`crates/`). Every entry carries the **exact** Rust site, the **exact** C# site it
diverges from, the root cause, and a fix plan. Where the diagnosis is partial the
confidence is stated explicitly instead of guessing (AGENTS.md golden rule).

Branch: `docs-2026-11-10-roadmap-audit` (base `origin/main` `f5e9d297`).

## How the audit was done

1. `graphify` first (`graphify-out/merged-graph.json`, 35 442 nodes) with
   `graphify explain` / `path` / `affected` for every symptom, then grep only to
   confirm line numbers.
2. For every suspect the matching C# function was read in full (not skimmed):
   `Main.DrawNPC`, `Player.ItemCheck_*`, `ItemSlot.*`, `WorldGen.KillTile*`,
   `TileDrawing.*`, `Main.UpdateAudio*`, `UI*.DrawSelf`.
3. The Rust site was then read and compared statement by statement; the constant,
   RNG order and update order were checked, not just the shape.
4. Nothing in this report is inferred from the symptom alone: each root cause
   points at code that is either missing, unreachable or reads the wrong field.

Legends:

- **Severity** — S1 blocks play/soft-locks; S2 breaks a core system; S3 cosmetic or
  narrow; S4 polish.
- **Confidence** — High = code path proven; Medium = one branch needs a visual/log
  repro; Low = suspected, recorded for later verification.

## Summary

| # | Bug | Severity | Confidence | Primary site |
| --- | --- | --- | --- | --- |
| 1 | No music on the front-end menus | S3 | High | `app/frame.rs:335`, `main.rs` |
| 2 | Character-select preview draws no character | S2 | High | `ui/char_select.rs:11-13` |
| 3 | Inventory favourites are lost on reload | S2 | High | `player_file.rs:140-150,325` |
| 4 | World-creation seed is ignored | S1 | High | `app/flow.rs:186`, `app/mod.rs` `terrstaria_world` |
| 5 | "Play" always boots the pre-loaded startup world | S1 | High | `main.rs:99-114`, `app/flow.rs:186` |
| 6 | No respawn countdown on the death screen | S2 | High | `ui/mod.rs:133-149` |
| 7 | NPC/boss sprite facing is wrong; many animations missing | S2 | High | `render/entities.rs:2176-2185`, `npc.rs:1809-1822` |
| 8 | Invasion progress bar is drawn with no invasion | S2 | High | `ui/mod.rs:159-247`, `app/frame.rs:1400` |
| 9 | Weapons deal no NPC damage; tools deal a fixed 15 | S1 | High | `world_sim/editing.rs:625-627,664-673` |
| 10 | Melee damages town NPCs (friendly fire) | S2 | High | `world_sim/editing.rs:637-706` |
| 11 | No in-game Settings screen (only the main-menu one) | S2 | High | `ui/mod.rs`, `app/loop.rs` (no route) |
| 12 | Escape does not close the map | S3 | High | `app/loop.rs:274-307` |
| 13 | Smart Cursor is always on, no toggle key | S2 | High | `world_sim/smart_cursor.rs:26` |
| 14 | Bestiary/Emote buttons squeezed; Emote button invisible | S2 | High | `ui/bestiary.rs:28`, `ui/inventory.rs` |
| 15 | Opening a Solar Selenian bestiary page panics | S1 | High | `ui/bestiary.rs:928` |
| 16 | Felling a tree leaves the trunk/canopy floating | S2 | High | `registry.rs:60-88`, `tr-core/edit.rs` |
| 17 | Front-end clicks are silent (no UI sounds) | S3 | High | `menu/*.rs`, `app/loop.rs` |
| 18 | Emote wheel "Back" is untranslated and mis-centred | S4 | High | `ui/emotes.rs:173-184` |
| 19 | Corrupt/hallowed/crimson cactus renders the wrong frame | S3 | Medium | `tr-core/drawdata.rs:1165-1201` |
| 20 | Trash slot has no quick-trash, no death clear, wrong offset | S2 | High | `app/mod.rs:1011`, `app/clicks.rs:787`, `ui/inventory.rs:483` |

---

## 1. No music on the front-end menus — S3 / High

**Symptom.** The main menu, character select and world select are silent; music
starts only after a world is entered.

**Expected.** `Main.UpdateAudio` runs every frame in every `Main.gameMenu` state and
`UpdateAudio_DecideOnNewMusic` selects the menu track, so the title screen plays
music (vanilla `Main.cs:12187-12988`, `:11076+`).

**Actual.** Music selection/playback is only driven from the in-world frame path:
`crates/tr-app/src/app/frame.rs:335` (the `update_audio`/music call). Nothing in the
front-screen path touches the audio system, and `main.rs` only builds the audio
device, never ticks it before the world is up.

**Root cause.** The audio clock was wired to the world step instead of the
frame step. There is no menu-side `MusicEnv` feed at all.

**Fix plan.**
1. Move the audio tick out of the world branch so it runs on *every* frame
   (`frame.rs`, before the `if in_world` split).
2. Feed `MusicEnv` a "menu" input (all zone flags false, `cur_music = 6`) — the
   vanilla title track — and let the existing `M6-MUSIC-1` ladder handle the rest.
3. Respect `settings.music` (already parsed) and `SettingPlayWhenUnfocused`.
4. Test: a front-screen frame produces a non-zero `cur_music`, and the world-entry
   transition does not restart the track if it is the same id.

**Fixed 2026-11-10 (`BUG-01-menu-music`).** `App::update_menu_music` runs on the
front-screen branch of `frame()` and `decide_new_music` grew the vanilla's
`if (Main.gameMenu)` arm verbatim in its order: the drunk ladder (remix 70 inside
it, not-the-bees 86, plain 60), remix 8, the anniversary 11 and the tail's
default 51 — the world-creation screen's seed text feeds the flags through
`SeedFlags::from_seed_text`, and `musicVolume == 0` still silences. The
decision half of `MusicSystem::update` moved ahead of the audio-device gate so
`cur_music` is observable in headless/screenshot builds. Parked (documented in
the code): the `menuMode == 3000` credits track 89, the `playOldTile` /
`titleMusicStyle` 6/50 arms and the `netMode == 2` silence. 5 tests: the ladder
order, the zero-volume gate, the front-screen tick's `cur_music` and the
creation screen's two seed flavours.

---

## 2. Character-select preview draws no character — S2 / High

**Symptom.** On the character-select screen the preview panel shows only the blue
background; the selected character's sprite never appears.

**Expected.** `UICharacter.DrawSelf` renders the player preview (back layers, body,
armour, dyes, hair, colours) inside the 59×58 panel
(`Terraria/GameContent/UI/Elements/UICharacter.cs`).

**Actual.** `crates/tr-app/src/ui/char_select.rs:11-13` states the preview sprite
draw is a documented no-op ("Task 7's `draw_player_preview`; until then the preview
draws the `PlayerBackground` only"). The panel rect is produced at `:375`, but no
player layers are drawn into it.

**Root cause.** Missing feature, explicitly parked. The player draw path exists for
the world renderer (`render/entities.rs`), but it is not reusable in UI space (it
expects a world camera, light and a `Player` with live physics state).

**Fix plan.**
1. Extract the player layer walk from `render/entities.rs` into a
   `draw_player_layers(r, lookups, player, origin, scale, tint)` helper usable by
   both the world and UI passes (the C# already shares `PlayerDrawLayers`).
2. Call it from the preview rect with the character's stored appearance
   (`player_file` already reads colours/armour).
3. Test: a deterministic `TR_UI=charsel` screenshot contains non-background pixels
   inside the preview rect.

**Fixed 2026-11-10 (`BUG-02-char-preview`).** The extraction step already
existed for the creation screen — `render::entities::draw_player_preview` over
`ui::char_creation::PreviewColors` — so the card reuses it instead of a second
walk. The missing half was the data: `save_list::PlayerEntry` now carries
`Appearance { hair, skin_variant, colors: [[u8; 3]; 7] }`, filled from
`PlayerFile` in `list_players` (the port's `player_file.rs:238-242` block, the
C#'s `Player.cs:55279-55285` ReadRGB order — hair, skin, eye, shirt, undershirt,
pants, shoe) and defaulted to `new Player()`'s initializers
(`Player.cs:2554-2566`) for the session/fake cards; the session entry carries
the creation draft's colours when it came out of character creation. The card
draws the sprite into the 59x58 panel at `characterScale = 1`
(`UICharacterListItem.cs:170`) on the static animation row 0
(`UICharacter.cs:89`) at the `GetPlayerPosition` anchor
(`UICharacter.cs:122-130`), the `PlayerBackground` sheet still underneath.
Documented deviation (unchanged): the preview decodes only the base body
sheets, so every skin variant / hair id renders the base frame — the port has
no per-variant player sets. 4 tests: the vanilla default colours, the seven-
triplet → RGBA mapping, the disk card's `.plr` round-trip, and the existing
charsel layout suite.

---

## 3. Inventory favourites are lost on reload — S2 / High

**Symptom.** Starring an item (favourite) works until the world/character is
reloaded; after loading, no slot is starred.

**Expected.** The 279+ `.plr` section stores a per-inventory-slot `favorited` bool;
`SavePlayer` writes it and `LoadPlayer` restores it (`Player.cs` section walk around
the inventory block, release 279+).

**Actual.**
- `crates/tr-app/src/player_file.rs:149` reads the favourites bitfield and only
  collapses it to `favorite_bits == 1` for a single flag on the character.
- `crates/tr-app/src/player_file.rs:325` writes `w.u64(0)` — a hardcoded zero.
- The live model keeps favourites positionally on the `App`
  (`favorite_slots`, `app/mod.rs:987-1001`) and never serializes them.

**Root cause.** The favourites were modelled as UI state instead of as item data;
the `.plr` writer was stubbed with a zero.

**Fix plan.**
1. Add `favorites: u64` (or `[bool; 59]`) to `PlayerFileWrite`/`PlayerFileRead`,
   write it at the same offset the reader already consumes, and hydrate
   `App::favorite_slots` on load.
2. Wire the existing `toggle_favorite` (`app/mod.rs:987`) to mark the save dirty.
3. Test: toggle two slots → save → load → the same two slots are starred; and the
   vanilla round-trip test (`M6-SAVE-2`) still passes.

**Fixed 2026-11-10 (`BUG-03-favorites-persist`).** `PlayerFile`/`PlayerFileWrite`
carry `inv_favorites: u64` — bit `i` is `inventory[i].favorited`, the exact
`Player.cs:54820-54824` layout (type, stack, prefix, favorited) the reader had
already been consuming at `:55340-55362` for release ≥ 114. The writer emitted a
hardcoded `false` per slot and the reader dropped the byte; both halves carry the
real value now. `save_list::PlayerEntry` gains the field (filled from the `.plr`),
`App::hydrate_favorites`/`App::favorite_bits` convert between the live
`favorite_slots` and the bitfield, and the creation save writes the live stars.
**Found on the way:** the port's reader skipped the armour/dye `favorited` byte
at release ≥ 322 (`Player.cs:55315`/`:55336`), so a real vanilla 326 file
desynced the section walk from the armour block onward — now gated and consumed.
Parked: the armour/dye favourites have no port UI (kept consumed-and-dropped)
and the in-game re-save of a loaded character rides BUG-05's `WorldPlay`, since
the port's only player-file writer today is the creation path. 5 tests: the
sample's bits, the save/load round trip, the version gate, the App-side toggle
pair through the bitfield, and the disk card's stars.

---

## 4. World-creation seed is ignored — S1 / High

**Symptom.** Typing a seed (or a secret-seed text) when creating a world has no
effect: the generated world is always the default.

**Expected.** `WorldGen.SetSeed`/`WorldGenerationOptions` consume the entered text
(`M5-SEED-1`/`M5-SEED-2` already port the parser and the flags) and the generator
runs with that seed.

**Actual.** The creation UI stores the text, but the play path never regenerates:
`app/flow.rs:186` (`FlowClick::WorldPlay(_) => (None, None, None)`) closes the
screens without creating anything, and the world that is eventually loaded is the
startup one (`main.rs:99-114`), whose seed comes from `TR_SEED` or the literal `1`
(`app/mod.rs` `terrstaria_world`, ~`:3305-3325`).

**Root cause.** Two gaps compound: `WorldPlay` is a stub, and the creation action
saves the intended seed into the world file but nothing reads it back on load
because the load path is the boot path.

**Fix plan.**
1. Implement `WorldPlay(i)`: resolve the selected world file, `app::load_world` it
   (the boot code already exists), apply the side sections, and swap `App.world`.
2. On creation, pass the parsed seed text through `seeds::resolve_seed_text` into
   `WorldGenerator` (the `M5-SEED-1/2` framework is already in place) instead of
   the `TR_SEED`/1 fallback.
3. Test: two worlds created with seeds "A" and "B" have different
   `world_hash`/tile digests; reloading the created world reproduces the same seed.

**Fixed 2026-11-10 (`BUG-04-world-seed`).** The typed seed already reached the
generator: `create_world_in` feeds `GenOptions.seed` from
`world_creation::seed_value`/`seed_identity` (`WorldFileData.TranslateSeed`'s
port, `WorldFileData.cs:386-397`) and the header records it — a test pins
`seed == 12345` from the written bytes. The observable half was the play path:
`FlowClick::WorldPlay` closed the screens without loading anything, so the world
you played was the boot one and the seed "had no effect". The click now parks
the card's file on `App.play_world` (the vanilla's `PlayGame` → `Main.LoadWorld`),
and the frame loop runs the new `App::swap_world` at a frame boundary: the
selected file loads through the existing `load_world` (side sections applied),
the world-derived state re-derives through `App::new`'s own body, the atlas
rebuilds from the new world's tile/wall usage and the renderer is recreated from
it, while the window/GPU/audio/music/settings and the shot-capture state survive
— and a failed load keeps the current world with a `front_hint` line instead of a
black screen. Deviation: the port has no loading-screen flow (the swap is
instantaneous in-frame). 3 tests: the Play parking, the two-seed tile digest and
the header's seed. BUG-05 closes the remaining half — the boot path's
pre-baked-world default.

---

## 5. Lobby shows a pre-loaded world ("Play" starts the wrong world) — S1 / High

**Symptom.** The world list shows the player's worlds, but pressing Play opens the
same pre-generated `Terrustaria` world every time — the lobby is effectively
decorative.

**Expected.** `UIWorldListItem` play loads the selected `.wld`/`.trwld`.

**Actual.** `crates/tr-app/src/main.rs:99-114` loads one world at startup
(`TR_WORLD` path, else `app::terrstaria_world()`) and `FlowClick::WorldPlay(_)`
(`app/flow.rs:128`, handled at `:186`) does not load anything — it returns three
`None`s (the screen-closing tuple).

**Root cause.** Same stub as #4: the front-end → world transition was never wired to
the loader.

**Fix plan.** Same as #4 step 1, plus: keep the boot world as an explicit debug/test
path (`TR_WORLD`), not the production path; on `WorldPlay(i)` drop the boot world
and load the selected one. Guard the swap so a failed load keeps the menu alive with
an error line instead of a black screen.

**Fixed 2026-11-10 (`BUG-05-world-play-load`, on top of `BUG-04-world-seed`).**
The `WorldPlay` loader — the file parking on `App.play_world`, the
`App::swap_world` at a frame boundary, the atlas/renderer rebuild and the
failed-load guard that keeps the menu alive with a `front_hint` line — landed on
BUG-04's branch and is the loader this row named. The boot half lands here:
`app::latest_own_world()` picks the player's newest `.trwld` in the saves dir as
the production boot world (the world the lobby's `WorldPlay` opens anyway), and
the pre-baked `Terrustaria` stand-in (seed 1 / `TR_SEED`, regenerated by
`TR_REGEN`) only remains when no `.trwld` exists at all — an out-of-the-box
fallback, no longer the production path. `TR_WORLD=<path>` and a positional path
argument stay the explicit debug/test paths. Deviation: the vanilla has no boot
world (`Main.ActiveWorldFileData` is null until `PlayGame`) while the port's
`App::new` needs one, so the newest own world is the closest shape. 1 test: the
newest-save selection and the empty-dir fallback.

---

## 6. No respawn countdown on the death screen — S2 / High

**Symptom.** On death the screen shows the death text/coins but no "Respawn in
N seconds" countdown; the player respawns after an unexplained delay.

**Expected.** `Main.DrawDeathText` (`Main.cs:45324-45341`) draws
`Lang.misc[56]` ("Respawn in {0} seconds") from the player's `respawnTimer`, and the
timer itself is updated by `Player.UpdateDead` (`Player.cs`).

**Actual.** `crates/tr-app/src/ui/mod.rs:133` explicitly says the respawn timer's
fade "lands with the death-screen pass"; `draw_death_screen` (`:134-149`) draws only
the death broadcast and the dropped-coins line. The sim does respawn the player, but
the seconds value is never surfaced.

**Root cause.** The `respawnTimer` value exists implicitly in the death flow but was
never stored on the `App` nor drawn.

**Fix plan.**
1. Add `respawn_timer` to the death state (set from `Player.respawnTimer`'s vanilla
   defaults: 600 ticks softcore / 3000 expert, and the boss-fight penalty).
2. Decrement on the fixed step beside the existing death handling.
3. Draw `Lang.misc[56]` through `death_text` below the coin line; consume
   `lang::misc` (localization is already wired).
4. Test: kill the player, advance 60 ticks, the drawn string contains "9".

---

## 7. NPC/boss sprite facing is wrong, and parts of the animation set are missing — S2 / High

**Symptom.** Many NPCs are drawn mirrored (facing the wrong way) and never flip as
they turn; several bosses' frames do not animate (a static strip or the wrong row
height).

**Expected.** `Main.DrawNPC` flips on **`spriteDirection == 1`** for *every* NPC
(`Main.cs:23124-23128`); the per-kind `FindFrame` cases set the row and the AI
machines maintain `spriteDirection` (24 write sites: `NPC.cs:19200, 19867, 21603,
21616, 21754, 21759, 22302, 22742, 22747, 24893, 25607, 25956, 26068, 26395, 26400,
27960, 28247, 28524, 28759, 29036, 29056, 29122, 29230, 29716`).

**Actual.**
- `crates/tr-app/src/render/entities.rs:2176-2185` flips on `n.facing`, *except*
  for a hardcoded whitelist `Npc::flips_on_sprite_dir()`
  (`crates/tr-app/src/npc.rs:1809-1822`: martians, Prime limbs, Skeletron Prime,
  the Twins, the four pillars).
- `Npc.sprite_dir` exists (`npc.rs:2574`) and defaults to `-1`, but the port's AI
  machines only write it in a handful of arms, so switching the renderer to
  `sprite_dir` alone would freeze most NPCs at `-1`.

**Root cause.** Two halves of one mechanic are out of sync: the renderer uses
`facing` with an exception list, while the source of truth (`spriteDirection`) is
only partially maintained. The whitelist patches the visible symptoms for a few
kinds and leaves everything else mirrored whenever `spriteDirection != direction`
(e.g. a stationary attacker that turned its body, a limb mounted on the other side,
a saucer rider).

Animation side: the port's `find_frame` (`npc.rs:4361+`) ports only the kinds it
knows; kinds without an arm fall back to a blind strip. The recently re-landed
Golem/Twins/Prime/Deerclops displays (M4-BOSS-19b/19c, M4-NPC-3k, M4-OCR-3j) are
correct, but the remaining families still rely on the generic fallback.

**Fix plan.**
1. Make the renderer read `n.sprite_dir` unconditionally (delete
   `flips_on_sprite_dir`).
2. Audit the 24 C# write sites and add the corresponding `sprite_dir` writes to the
   port's AI machines, keeping `facing` as the *movement* decision only.
3. Add the missing `FindFrame` arms family by family, with the
   `Main.npcFrameCount` row counts and the sheets' measured heights (the pattern
   used by `M4-NPC-3c`'s `humanoid_frame_count`/`humanoid_draw`).
4. Regression: a `TR_NPC=<kind>` lineup screenshot per fixed family, SHA256-stable;
   assert `sprite_dir != facing` cases render flipped.

---

## 8. Invasion progress bar is always drawn — S2 / High

**Symptom.** The invasion bar (and its dark backdrop) is on screen permanently,
even with no goblin/pirate invasion running.

**Expected.** `Main.DrawInvasionProgress` runs only while
`Main.invasionProgressAlpha > 0` (fade-in/out around an active invasion) and is
gated by `Main.InvasionProgressMode` (`Main.cs:45421+`; `invasionProgressMode`
default `2`, `Main.cs:67080`).

**Actual.** `crates/tr-app/src/app/frame.rs:1400-1408` calls
`ui::draw_invasion_progress` unconditionally. `crates/tr-app/src/ui/mod.rs:159-247`
draws the backdrop at a fixed `255/255*0.785` alpha, ignoring a fade alpha entirely.

**Root cause.** The widget was implemented as a static panel; the alpha/mode inputs
were never plumbed.

**Fix plan.**
1. Add `invasion_progress_alpha` to `App`, moved toward 1 while
   `World.invasion_active` and toward 0 otherwise (the vanilla 0.05 step).
2. Gate the call in `frame.rs` on `alpha > 0` and on `settings.invasion_progress_mode`
   (off/warning-only/full).
3. Multiply the backdrop and bar colours by the alpha.
4. Test: no invasion → nothing drawn; start one → 20 frames later the bar is
   visible; end → it fades out.

---

## 9. Melee weapons deal no damage; tools deal a fixed 15 — S1 / High

**Symptom.** Swinging a sword at an NPC does nothing; swinging a pickaxe/axe at an
NPC deals a small fixed amount (15).

**Expected.** `Player.ItemCheck` → `ItemCheck_MeleeHitNPCs` →
`ProcessHitAgainstNPC` (`Player.cs:44241-44260`) applies the item's damage, class
multiplier, crit, `DamageVar` and the NPC's defence.

**Actual.**
- `crates/tr-app/src/world_sim/editing.rs:625-627`:
  ```rust
  if held_use_ammo == 0 { return; }
  ```
  sits at the end of the `if app.mine_down && held_weapon …` block, so **every melee
  weapon** (use_ammo 0) returns before the melee-hit block at `:637-706`.
- The melee-hit block is additionally gated on `held_use_ammo == 0` (`:638`), so
  even ammo weapons (which pass `:625`) can never enter it. The block is
  unreachable in both directions.
- Non-weapons (`held_weapon == false`, e.g. pickaxes) skip the weapon block, reach
  `:637`, and use the hardcoded stand-in `raw = 15.0` (`:671-673`), with the
  knockback 5.5 (`:689-692`).

**Root cause.** The control flow was inverted when the shooter path was added: the
early `return` was meant to *let ammo shooters fall through* but it returns for the
non-ammo case. The fallback 15 was the original placeholder never replaced.

**Fix plan.**
1. Delete the early `return` (`:625-627`) and let the melee block run for
   `held_weapon` items; keep the shooter path from double-firing by making the melee
   block explicitly `held_use_ammo == 0` (already true) and excluding
   `registry::projectile`-only items.
2. Use the real item damage (`held_fx.damage`/`registry::damage`), `useAnimation`
   and `knockBack` for both weapons and tools — tools are weapons in vanilla for
   `ItemCheck` purposes (they have `damage`, `useAnimation`, `knockBack`).
3. Only allow a tool hit on an NPC when the tile under the cursor is not the target
   (vanilla prioritises the tile), matching `ItemCheck`'s tile-vs-NPC order.
4. Test: iron broadsword on a zombie removes `defense`-reduced damage > 0; pickaxe
   deals its own damage (not 15); a gun still fires its projectile.

**Fixed 2026-11-11 (`BUG-09-melee-damage`).** The inverted early return is gone —
`ItemCheck_Shoot` fires from inside the swing arm (`Player.cs:43568-43571`), above
the mining/placing arms, so every item falls through and the melee block keeps its
own `held_use_ammo == 0` gate for the shooters. The hit reads the held row: the
`registry` `damage` (or the prefixed value) × the class multiplier
(`Player.GetWeaponDamage`, `Player.cs:53421-53429`), the crit roll
(`GetWeaponCrit` + `Item.crit`, `:53371`), `Main.DamageVar` with the player's luck
(`:44342`), then `NPC.StrikeNPC`'s `max(1, Damage − Defense/2) × (crit ? 2 : 1)`
(`NPC.cs:81239`, the port's `tr_core::damage::strike_npc_damage`), and the
knockback is the item's `knockBack` with the glove/buff multipliers at 1
(`Player.cs:43664-43679`). `cursor_cell_is_tile_target` restores the vanilla's
tile-first order for the tools (`ItemCheck_UseMiningTools`, `:43593`, ahead of the
melee-hit box block `:43644-43686`), so a pick over a minable tile still mines.
Verified statement-by-statement against the C#; the tests were re-run green
(`bug_09_the_melee_hit_reads_the_item_damage` — now pinning the *amount*: the
wooden sword's 7 and the pick's 5 through `DamageVar` minus the slime's defence,
never the flat 15 — plus `bug_10_melee_refuses_the_town_npcs` and
`test_DW_3_2_melee_hit_passes_the_player_luck_to_damage_var`, and the workspace
suite 1912/0). Parked: the registry carries no `noMelee` (a named generator skip,
ANALYSIS), so the `held_use_ammo == 0` gate is the port's approximation of the
vanilla's `damage > 0 && !noMelee` — a non-ammo projectile weapon can still
swing its own melee damage, which the vanilla refuses. Observed on a throwaway
rig (not a test): the Wooden Boomerang (284) deals 9 to a defence-2 slime on its
first swing, while the Space Gun (127) escapes only because the `gunProj`
id-keyed table gives it a `useAmmo`. The real fix is the `no_melee` field on
`ItemDef` (M3-ITEM) — excluding every `projectile` item instead would be wrong:
the Enchanted Sword (989) melees *and* shoots in vanilla.

---

## 10. Melee damages town NPCs (friendly fire) — S2 / High

**Symptom.** Swinging at a town NPC (Guide, Merchant, …) hurts it; in vanilla town
NPCs are not damaged by player melee.

**Expected.** `ProcessHitAgainstNPC` (`Player.cs:44241-44260`) refuses friendly
targets: `npc.friendly` (town NPCs) is skipped unless the hit comes from the
specific PvP/whip-vs-town path, and the Clothier/Guide "kill" easter eggs are
explicit, not incidental.

**Actual.** `crates/tr-app/src/world_sim/editing.rs:648-704` takes the first NPC
whose box contains the cursor and calls `n.hurt(...)` with no friendly check. The
`Npc` type has `is_town_npc()`/`dont_take_damage`, but the melee arm never consults
them; there is no `friendly` / `kill_guide` / `kill_clothier` state.

**Root cause.** The hit filter only checks geometry.

**Fix plan.**
1. Add the C# gate before `hurt`: `if n.is_town_npc() || n.friendly { skip }`, with
   the explicit exceptions the C# has (the Guide Voodoo Doll, the Clothier's
   Skeletron curse) modelled as flags, not as a blanket allow.
2. Skip `dont_take_damage` NPCs (Golem head shield, etc.) as well — the vanilla
   check is the same block.
3. Test: melee on a Guide leaves `lifeMax` intact; melee on a zombie still damages.

---

## 11. No in-game Settings screen — S2 / High

**Symptom.** Settings can only be reached from the title menu; pressing Escape in
game offers no Settings entry (and there is no in-game options UI).

**Expected.** `Main.IngameOptions` (`Main.cs:45532+`) opens the in-game options
window (UIScale, MapScale, control mode, capture mode, …) from the pause menu.

**Actual.** `crates/tr-app/src/menu/mod.rs` contains the `IngameOptions` draw
helpers, but the `Page` enum has no in-game page and `app/loop.rs`'s Escape handler
opens only the inventory/map/bestiary. There is no route from the pause state to a
settings page.

**Root cause.** The window widgets were ported but never mounted on a screen.

**Fix plan.**
1. Add `Page::IngameOptions` and a "Settings" row to the pause menu.
2. Reuse the existing `menu/mod.rs` draw helpers and the `Settings` fields already
   in `settings.rs` (M6-SETTINGS parsed the vanilla keys).
3. Persist on close through `Settings::save`.
4. Test: `TR_UI=ingame_options` screenshot + a click round-trip toggling music.

---

## 12. Escape does not close the map — S3 / High

**Symptom.** With the map open, pressing Escape does nothing (you must press the
map key again).

**Expected.** `Main.UpdateInput`'s Escape handling closes the map overlay before
falling through to the inventory (`Main.cs` `Main.menuMode`/map branch).

**Actual.** `crates/tr-app/src/app/loop.rs:274-287` handles Escape for the
bestiary, emotes and inventory, then `:298-307` toggles the map on the map key —
there is no Escape arm for `map_open`.

**Root cause.** Missing branch in the Escape priority chain.

**Fix plan.** Insert `if self.map_open { self.map_open = false; return; }` before
the inventory arm, matching the vanilla close order (map → inventory → pause).
Test: open map, Escape, `map_open == false`.

---

## 13. Smart Cursor is always on, and there is no toggle — S2 / High

**Symptom.** The smart-cursor highlight is always active and cannot be turned off
(the vanilla left-Shift toggle does nothing).

**Expected.** `Main.SmartCursorEnabled`/`Main.SmartCursorIsUsed` with the
left-Shift toggle (`Main.cs:67178`); the default is **off** until the player turns
it on.

**Actual.** `crates/tr-app/src/world_sim/smart_cursor.rs:26` gates on
`app.settings.smart_cursor_toggle`, which `Settings::default()` sets to `true`; and
no input path writes the toggle at runtime (the comment at `:109` still says "the
port's settings do not carry the toggle yet").

**Root cause.** The setting is a persisted boolean used directly as the runtime
state; the toggle key and the "used" vs "enabled" distinction are missing.

**Fix plan.**
1. Add `smart_cursor_used` (runtime) separate from `smart_cursor_toggle` (setting),
   default off.
2. Handle the left-Shift press edge in `app/loop.rs` and sync `used`.
3. Gate the lookup on `used`, not on the persisted toggle.
4. Test: default → no re-point; Shift → re-point; Shift again → off.

---

## 14. Bestiary/Emote buttons squeezed; Emote button is invisible — S2 / High

**Symptom.** The Bestiary and Emote buttons in the inventory overlap/squeeze each
other, and the Emote button has no icon (only its hitbox exists).

**Expected.** `Main.DrawBestiaryIcon` (498,278) and `Main.DrawEmoteBubblesButton`
(534,278) draw the `UI/Bestiary` and `UI/EmoteBubbles` button sheets
(`Main.cs:42369-42408`), 30×30 each, side by side.

**Actual.**
- `crates/tr-app/src/ui/bestiary.rs:28` defines `EMOTE_BUTTON_RECT` but the button
  is never drawn — only hit-tested (`emote_button_hit`, `:32-37`) and clicked
  (`app/clicks.rs:296-305`).
- `crates/tr-app/src/ui/inventory.rs` draws the bestiary icon at its rect but
  derives the button's frame from a sheet assumed to be a single-image sheet; the
  `UI/Bestiary` sheet is a horizontal 2-frame strip (normal/hover), so the "one
  full-width frame" treatment squashes both frames into 30 px.

**Root cause.** The emote button's draw call was never added; the bestiary button
uses the wrong frame extraction (`draw_sheet_frame` treating a 2-frame horizontal
sheet as one frame).

**Fix plan.**
1. Add the emote button draw at `EMOTE_BUTTON_RECT` with the hover frame from the
   pointer (the vanilla's `:42397` hover index).
2. Fix the sheet frame helper to take `(frames_x, frames_y, frame)` and slice
   `w / frames_x` (the same fix needed by #15's context).
3. Assert the two rects (498..528 and 534..564) never overlap.
4. Screenshot test `TR_UI=inventory` with both buttons visible.

---

## 15. Opening a Solar Selenian bestiary page panics — S1 / High

**Symptom.** The game panics (index out of bounds) when the Solar Selenian's
bestiary entry is opened.

**Expected.** The page draws.

**Actual.** `crates/tr-app/src/ui/bestiary.rs:928` indexes
`lookups.bestiary_npc_sheets[174]`, but the table has exactly 174 entries
(`assets.rs:1186` `BESTIARY_NPC_SHEET_IDS: [u16; 174]`), so the valid range is
`0..=173`.

**Root cause.** Off-by-one hardcoded slot index. The rest of the file already uses
the derived `Lookups::bestiary_sheet_index` (`assets.rs:1504-1510`) precisely to
avoid this class of drift; this arm was missed.

**Fix plan.**
1. Replace `bestiary_npc_sheets[174]` with `bestiary_sheet_index(<SolarSelenian net
   id>)` (and audit every remaining literal index in the function for the same
   class of bug).
2. Test: open every bestiary entry (the 174-entry loop) — no panic; the existing
   registry test covers the data side.

---

## 16. Felling a tree leaves the trunk/canopy floating — S2 / High

**Symptom.** Breaking a tree's base removes one tile but the rest of the tree stays
in the air; no wood/acorn drops come out.

**Expected.** `WorldGen.KillTile` dispatches the tree arm
(`WorldGen.cs:65214-65231`, cases 5/596/616/634) to `KillTile_GetTreeDrops`
(`WorldGen.cs:66189-66261`) and the `SquareTileFrame`/`KillTile` cascade collapses
the whole trunk (the ground-sand walk is `WorldGen.cs:47687-47737`).

**Actual.**
- `crates/tr-app/src/registry.rs:60-88` (`tile_drop`) has no tree arms, so no
  wood/acorn drops.
- `crates/tr-core/src/edit.rs`'s `can_kill_tile`/`kill_tile` has no whole-tree
  collapse: only the struck cell is cleared.

**Root cause.** The tree object's destruction cascade was never ported; only the
generic single-tile kill exists.

**Fix plan.**
1. Port `KillTile_GetTreeDrops` (the `GetTreeTypeAndTreeBottom` walk, the
   `TreeTypeDropsAcorns` roll, the wood/acorn/seed item ids and the `dropIsWood`
   flag).
2. Port the cascade: after killing a trunk cell, walk up the same column while the
   tile is a tree kind, clearing and dropping each cell; handle the branch/leaf
   merge tiles (191/192) and the sand-ground rule.
3. Keep the RNG order: the drop roll uses `WorldGen.genRand` (`Next(2)` then the
   tree-type pick), so it must be consumed in that order.
4. Test: a 10-tall tree felled from the base yields exactly one wood stack per
   trunk cell + the acorn roll; a world-hash/determinism test is unaffected
   (runtime only).

**Status: FIXED 2026-11-11 (`BUG-16-tree-collapse`).**

What landed: the cascade arm `TileFrameImportant` dispatches for the tree
kinds (`WorldGen.cs:85876-85911`) was missing — the port's `square_tile_frame`
only ran the multi-tile `Check*` object routines, so a tree cell that lost its
ground stayed in the air and the trunk never collapsed. `tr-core/src/edit.rs`
now carries `check_tree` (the forest tree 5 — `WorldGen.CheckTree`,
`WorldGen.cs:54570-54780`) and `check_palm_tree` (323 — `WorldGen.CheckPalmTree`,
`:54346-54400`), with `CheckTreeWithSettings`'s variant kinds
(`:54402-54568`) parameterised by their ground test
(`GemTreeGroundTest`/`VanityTreeGroundTest`/`AshTreeGroundTest`, `:30304-30341`).
The routine's `KillTile` chain is what makes the trunk fall: the cell above a
missing ground dies, and its own `TileFrame` ring reaches the next one up.

The drop layer: `tr-core/src/tree.rs` gained `GetTreeType` (`:63075-63110`),
`GetTreeBottom` (`:63436-63482`), `GetTreeTypeAndTreeBottom` (`:63055-63066`),
`TreeTypeDropsAcorns` (`:63112-63119`) and the wood table; `edit::TreeDropEvent`
carries the per-cell record (the core has no RNG stream) and the app resolves
the C#'s roll order — `KillTile_GetTreeDrops` (`:66189-66261`): the canopy
marker's acorn `Next(2)`, the mushroom's `Next(2)`, the bonus wood
`Next(35) <= axe` and its `Main.rand.Next(3)` second chance; the palm's
`case 323` arm of `KillTile_GetItemDrops` (`:65232-65266`) with the biome
ground walk. The tree kinds 72/583-589/596/616/634 are now frame-important
like the C# (`Main.cs:7678/:7760-7775`), so the cascade reaches them.

Deviations: the palm's canopy re-roll (`CheckPalmTree`, `:54385-54388`) needs
the `genRand` stream, so the core records the request and the app resolves it
after the cascade (the C#'s ring does not fire for that arm — `frameY` is
unchanged); the `Check*` kills propagate the ambient `noItem` like the other
object checks (the C#'s `KillTile(i, j)` defaults to `noItem: false`, so the
worldgen pass drops no wood here). 5 tests.

---

## 17. Front-end clicks are silent (no UI sounds) — S3 / High

**Symptom.** Clicking menu buttons, character rows, world rows and the Back/settings
buttons produces no sound; in-game inventory clicks do click.

**Expected.** `SoundEngine.PlaySound(12)` (menu tick) / `7` (grab) fire from
`Main.MenuMouse`/`UIElement` activation (`Main.cs:1710`, `:1010`). The port already
calls `audio.play("Menu_Tick")`/`("Grab")` from the in-game click handlers
(`app/clicks.rs:249,606,627,638,924,986`).

**Actual.** `crates/tr-app/src/menu/*.rs` contains **no** `audio.play` calls at all;
`app/loop.rs`'s front-click path also has none. Only volume settings live in
`menu/mod.rs`.

**Root cause.** The front-end click/activate path never got the sound wiring the
in-game path has.

**Status: FIXED 2026-11-10** (`fix-front-end-click-sounds`).

What landed: the front-end activation cue in `handle_front_click`
(`UICharacterSelect.cs:110/:118`, `UIWorldListItem.cs:365`) — the Back-family
clicks play `Menu_Close` (11), the rest `Menu_Open` (10); the legacy menu's
click plays `Menu_Tick` (`DoStatefulTickSound`, `Main.cs:3244`); the hover
edge — the front screens' element enter (`FadedMouseOver`,
`UICharacterSelect.cs:124` / `UIWorldSelect.cs:126`) and the legacy menu's row
enter — plays one `Menu_Tick` per enter. `Audio` gained a `played` counter
(the test seam the headless flows assert through — `played_count()`). 2 tests
(the activation cue, the single hover tick).

Note on the report's item 1: the front-end paths call the same `audio.play`
helper the in-game path uses, but the legacy menu's activation cue is
`Menu_Tick` (the C#'s stateful tick), not `Grab` (7) — the `Grab` cue is the
chest/inventory slot movement's cue, not the menu's.

---

## 18. Emote wheel "Back" is untranslated and mis-centred — S4 / High

**Symptom.** The emote wheel's Back button reads the hardcoded English "Back" in
every language, and the label sits slightly off-centre.

**Expected.** `Lang.menu`'s Back string through `Lang.inter`/`Language.GetTextValue`
(the rest of the UI goes through `crate::lang`), centred with the panel's
`HAlign 0.5`.

**Actual.** `crates/tr-app/src/ui/emotes.rs:173-184` draws the literal `"Back"` and
computes `text_w` with `text::hud_text_width` then places it at the rect — the same
pattern as the bestiary's Back, which is centred by a shared helper; this call
bypasses it.

**Root cause.** Hardcoded string + a hand-rolled centring instead of the shared text
helper.

**Fix plan.**
1. Replace `"Back"` with the `lang::menu`/`lang::inter` Back accessor used by the
   bestiary's `back_rect` path.
2. Centre through the shared `text::` helper (the one `ui/bestiary.rs` uses) so the
   measurement and the draw agree.
3. Test: the emote Back label equality with the bestiary Back label in the
   non-English locale fixture.

**Status: FIXED 2026-11-11** (`BUG-18-emote-back-lang`).

The label was the literal `"Back"`: `UIEmotesMenu.cs:78` builds the panel as
`UITextPanel<LocalizedText>(Language.GetText("UI.Back"), 0.7f, large: true)`,
so the port now reads `lang::text(BACK_KEY)` (`BACK_KEY = "UI.Back"`, the same
key the bestiary's `UIBestiaryTest.cs:366` and achievements' back panels use)
and no literal remains. The mis-centring had a second, separate cause the
report's plan did not name: `back_hit`/the draw borrowed
`bestiary::back_rect`, whose `Top(-25)` is the **bestiary's**
(`UIBestiaryTest.cs:371`) — the emote wheel's own panel is `Top(-45)`
(`UIEmotesMenu.cs:82`), so the button sat 20 px too low. `ui/emotes.rs` now
owns `back_rect` with the wheel's `-45` and `back_hit` reads it. 2 tests: the
ru-RU label resolves to "Назад" (never the English literal) through the
shared language fixture, and the wheel's Back box is centred with its own
`-45` box 20 px above the bestiary's. Found on the way (not fixed, out of
this row's scope): the port draws a bespoke `"Emotes"` panel title that has
no C# counterpart — the vanilla shows the `UI.EmoteCategory*` group headers
(`UIEmotesMenu.cs:91-97`) instead.

---

## 19. Corrupt/hallowed/crimson cactus renders the wrong frame — S3 / Medium

**Symptom.** Cacti growing on ebonstone/pearlstone/crimstone sand show the normal
cactus art (or a shifted strip) instead of the biome-tinted cactus.

**Expected.** `TileDrawing.DrawSingleTile`'s `case 80`
(`decompiled/Terraria/GameContent/Drawing/TileDrawing.cs:1132-1147`) calls
`WorldGen.GetCactusType` and adds `54`/`108`/`162` to the **source** rectangle's Y
for evil/good/crimson grounds; `GetTileDrawData`'s own `case 80` only sets
`tileTop = 2` (`:5014-5018`).

**Actual.** `crates/tr-core/src/drawdata.rs:1191-1194`
(`apply_world_corrections`) adds `cactus_source_y_offset` to `add_fr_y`
unconditionally. The normal flat draw path rebuilds the source rect from
`tileFrameY + addFrY`, whereas the C# only mutates the local `rectangle` used by
the half-brick path (`:1007` builds it, `:1295`/`:1299` consume it) — so the port's
shift applies on every cactus, including the ones that should not shift.

**Root cause.** The port folded a `DrawSingleTile`-local rectangle adjustment into
the shared `DrawData`, which is also consumed by the flat path.

**Status: FIXED 2026-11-10** (`fix-cactus-biome-frame`).

The re-verification the report asked for (item 3) confirms the shape: the
mutated `rectangle` (`TileDrawing.cs:1007`) has exactly ONE consumer — the
half-brick arm (`:1298`); every other arm (the flat, `:1295`, the slope and
sliced paths) rebuilds its own rect from `tileFrameY + addFrY` unshifted.
So the fix takes item 1's first arm: `TileDrawData` carries a
`half_brick_src_y` field, `apply_world_corrections`' kind-80 arm writes the
`54/108/162` offset there instead of `add_fr_y`, and the renderer's
half-brick draw (`push_half_shifted`) applies the shift to its source row
while the flat path draws unshifted — the pre-fix behaviour (the shift
folded into the shared `add_fr_y`, which the flat path also consumes) is
gone. 1 new test + 1 updated expectation; the item-2 screenshot pair stays
available via the deterministic `TR_CAM` harness.

---

## 20. Trash slot: no quick-trash, no death clear, wrong offset — S2 / High

**Symptom.** Shift-clicking an item does not send it to the trash (the vanilla
quick-trash), the trash keeps its item after death, and the slot does not move up
when a chest/shop is open (it overlaps the chest UI).

**Expected.**
- Quick-trash: `ItemSlot.MouseHover`'s `cursorOverride == 6` arm destroys the
  hovered stack and pushes it into the trash (`UI/ItemSlot.cs:759-769`,
  `CursorOverride`/`GetAlternateClickAction`), playing `SoundID.TrashItem`.
- Death: `Player.DropItems` clears the trash (`Player.cs:54304`
  `trashItem.TurnToAir()`), plus the event-item purge (`Player.cs:24729-24736`,
  `:31446-31451`).
- Offset: `Main.DrawTrashItemSlot` (`Main.cs:42349-42366`) shifts the slot
  `+168` Y and `+5` X when `Main.ChestOrShopUIVisible`.

**Actual.**
- `crates/tr-app/src/app/mod.rs:1011-1012` (`trash_swap`) only swaps the slot with
  the held item; there is no `cursorOverride == 6` destruction path
  (`app/clicks.rs:787+` documents the trash arm as "out").
- `app/clicks.rs:786` comments the Control-in-use trash arm out explicitly.
- `ui/inventory.rs:49-50` hardcodes `TRASH_X = 448`, `TRASH_Y = 258` and never
  applies the chest/shop `+5/+168`.
- No death/event-purge clear exists in the Rust death path.

**Root cause.** The trash was implemented as a plain 60-slot-style swap slot, not as
the destroy-on-quick-trash + cleared-on-death container the vanilla uses.

**Status: FIXED 2026-11-10** (`fix-trash-slot-behavior`).

What landed: (1) the quick-trash — `evaluate_cursor_override` lights
`AlternateClickAction.Trash`'s cursor 6 on a Shift-held hover over a
non-favorited, non-air inventory slot when no crafting/reforge/chest arm
matches (the C#'s `GetAlternateClickAction` else-chain tail,
`ItemSlot.cs:679-682`), and the click arm replaces the trash with the
stack, empties the slot and plays the cues — the `TrashItem` cue has no
port asset (the recorded gap; the call is a no-op) and `Grab` plays; the
research exception (`:762`) rides the unported Journey research,
documented. (2) `on_player_death` clears the trash
(`Player.DropItems`' first line, `:54304`); the event-item purge
(`:24729-24736`) rides the unported event items and is covered by the
wholesale clear. (3) The trash rect shifts to (453, 426) at scale 0.755
while `ChestOrShopUIVisible` (`Main.cs:42349-42366`), threaded through
`inventory::draw`/`trash_slot_at` — the draw and the hit test share one
rect helper. 3 tests.

---

## Appendix A — related gaps found while auditing (not in the owner's list)

These are the same class of defect (stub, unreachable branch, missing write) and
should ride the same fix branches:

| Area | Finding | Site |
| --- | --- | --- |
| Tree drops | `tile_drop` also lacks the sword-shrine/fake-sword special cases and the palm arm (`#16`) | `registry.rs:60-88` |
| NPC display | the `FindFrame` fallback still animates kinds with no case using a blind strip (the `#7` animation half) | `npc.rs:4361+` |
| NPC display | `sprite_dir` write sites missing in the walker/flyer/boss machines (24 C# sites) | `npc.rs` AI arms |
| Settings | `blood`, the smart-cursor trio, `placement_preview`, `mouse_border_color` are stored but have no consumer | `settings.rs` |
| Map | the map key is a hardcoded toggle; `Main.Input` rebinding not applied | `app/loop.rs:298-307` |
| Cactus | `GetTileDrawData case 80`'s `tileTop = 2` is present, but the draw-local rectangle contract is not modelled | `tr-core/drawdata.rs` |
| Trash | chest/shop scale `0.755` (vs `0.85`) not applied with the offset | `ui/inventory.rs:483-490` |
| Bestiary | remaining literal `bestiary_npc_sheets[N]` indices besides `174` | `ui/bestiary.rs` |
| Saves | `.trwld` still has no native twins for signs/plates/town rooms/bestiary sights (M6-SAVE-1b) | `tr-core/wld` |
| World select | `WorldPlay` also skips the `valid`/path gates on the Play button | `app/flow.rs:186` |

## Appendix B — three cross-cutting themes

1. **Stubbed transitions.** #4, #5 and half of #20 are not bugs in the ported
   mechanics; they are missing wiring between two finished halves (UI → loader,
   cursor → trash). Fixing them is mostly plumbing and should be scheduled before
   more content, because they make the game look broken at the first click.
2. **"Implemented but unreachable" code.** #9's melee block, #13's toggle and
   #11's IngameOptions widgets are all written and tested in isolation, yet no
   production path reaches them. The audit's `grep` for `TODO`/`stand-in` found
   them only because the control flow contradicted the comment; a small
   "reachability" test per feature would catch the next one.
3. **Hand-maintained tables.** #15 (the literal 174) and #7's `flips_on_sprite_dir`
   whitelist are the same failure mode the repo already fixed twice (the
   `bestiary_sheet_index` derivation and the asset-sheet drift). New code should
   derive from the vanilla id, never from a literal.