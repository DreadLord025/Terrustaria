# 112 — DOC-SWEEP lane 30: UI screens (W4-d85, 2026-11-21)

Code-only sweep over the W4-d85 lane's six screens (debug commands /
loadouts lists + row elements, reports page, Advanced world creation +
secret seeds). Branch `task/W4-d85-ui-screens-2`, base `1d7d39a0`.
No cargo/tests/graphify invoked — witness tests WRITTEN, NOT RUN.

## 1. ScenarioTestLoadout.Apply — full census

`Testing/ScenarioTestLoadout.cs:88-106` has six halves. Census against
`crates/tr-app/src/loadout.rs::apply`:

| C# half | Source | Port state before | After this sweep |
| --- | --- | --- | --- |
| Clock (`Main.netMode != 1`) | `:90-94` | PORTED (`world.day_time`/`time`, off-client gate) | unchanged |
| Life/mana stats | `:95-98` | PORTED (`hp`/`max_hp`/`mana`/`mana_max`) | unchanged |
| `selectedItemState.Select` | `:99` | PORTED (`hotbar::SelectionState::select`) | unchanged |
| Four item groups (inventory / ammo 54..57 / armor / miscEquips) | `:100-103` → `Apply(Item[], records)` `:179-191` | PORTED (`apply_items` × 4, `TryGetId`-false skip, prefix + stack) | unchanged |
| Minions: kill + **respawn** | `:104` → `:124-137` | kill half only (`minion_slots > 0` owned sweep); respawn was a documented deviation | **RESPAWN PORTED** (`6220169d`) |
| Buffs: direct slot writes | `:105` → `:155-162` | PORTED (`buff_type`/`buff_time` at the record index, name reverse) | unchanged |

The respawn half (`6220169d`): `MinionRespawner.RestoreMinionsFor`
(`DataStructures/MinionRespawner.cs:34-47`) parks the mouse at the screen
centre — the port passes the camera, which IS that world point — then
runs `MinionSpawnFromInventoryItem.TryRespawn`
(`DataStructures/MinionSpawnFromInventoryItem.cs:30-47`) per record:

- `FindMatchingItem` (`:49-61`): the FIRST inventory 0..49 slot matching
  type AND prefix (`ItemMatches` `:21-28`); no match returns silently.
- `buffType > 0` → `AddBuff(buffType, buffTime == 0 ? 3600 : buffTime)`
  (`:37-45`) through the ported `BuffSlots::add_buff` chain.
- `SilentlyShootItem` (`Player.cs:47051-47054`) =
  `ItemCheck_Shoot(whoAmI, item, 0, withAudioVisualFeedback: false)`.
  The port routes the summon-staff spawn arms (`SpawnMinionOnCursor`,
  `Player.cs:49573-49603` + the per-staff arms `:48436-48559` — the i25
  use-path census): weaponDamage 0, the UNSCALED prefixed item damage,
  `GetWeaponKnockback`'s minionKB fold (`:47151`), the cursor + reach
  clamp, the per-staff ai seeds (`:49582-49598`), the Pygmy/Pirate kind
  picks on the `UpdatePlayers` stream, the Spider mod-3 cycle, the
  Foxsparks `localAI[0] = 30`, and the Stardust Dragon's fresh-chain arm
  (`:48520-48536`; spawn damage is `Damage` = 0, `originalDamage` carries
  the item damage). **FINAL-phase correction**: the Apply kill sweep only
  kills `minionSlots > 0` (`ScenarioTestLoadout.cs:129`) and the dragon
  head 625 / tail 628 carry ZERO slots (`Projectile.cs:6731-6738`), so a
  live head+tail SURVIVES — the grow-existing-chain arm (`:48537-48553`)
  IS reachable and is now ported (two bodies at the player centre with
  the aim velocity, the key relink, `ai[1] = 1` + netUpdate +
  originalDamage on all three); a half chain (head xor tail) spawns
  nothing. **Cap census**: `SpawnMinionOnCursor` (`:49573-49603`) has NO
  minion-slot-cap check; `FreeUpPetsAndMinions` (`:51701-51851`) rides the
  USE START (`:42780`), which `SilentlyShootItem` bypasses — an over-cap
  respawn dies on the first pool tick (`Projectile.cs:16712`), exactly as
  the C#. The buff grant precedes the spawn in both (`:37-45` then `:46`).
- Remaining deviation: the non-summon per-item tail of `ItemCheck_Shoot`
  stays on the use path — unreachable here, since a loadout's `Minions`
  records are `TrackMinionSpawnFromItemUse` spawns (summon staffs only,
  `ProjectileID.cs:550` — default true minus 388/625/627/628, which are
  the Optic twin / dragon parts spawned BY those staffs).
- `/saveloadout` (the `Prepare`/`Set` writer half,
  `ToolkitDebugCommands.cs:848-859`) stays unported — g55's lane
  (`task/W4-g55-ui-loadout-save` exists on origin).

Witnesses: `loadout.rs::the_respawn_grants_the_buff_and_spawns_at_the_cursor`,
`::the_staff_arms_respawn_their_kinds`, and the FINAL-phase pair
`::the_dragon_grow_arm_relinks_the_surviving_chain`,
`::the_dragon_half_chain_spawns_nothing_and_the_fresh_arm_needs_both_dead`
(written, not run).

## 2. WorldCreationAdvanced — full option census

`WorldGenerationOptions.cs:30-41` registers exactly ten options; all ten
are in `tr_core::seeds::OPTION_TABLE` and `KEY_NAMES`
(`ui/world_creation_advanced.rs`), with the `ServerConfigName` /
`SpecialSeedNames` / `SpecialSeedValues` rows matching:

| Option | KeyName | Cascade | Port |
| --- | --- | --- | --- |
| Normal | Seed_Normal | enable clears all; subscriber re-derives all-off | PORTED (row 11) |
| NotTheBees | Seed_NotTheBees | none | PORTED |
| Drunk | Seed_Drunk | none (value 5162020) | PORTED |
| Anniversary | Seed_Celebration | none (celebrationmk10, 5162021/5162011) | PORTED |
| DontStarve | Seed_TheConstant | none (4 names) | PORTED |
| ForTheWorthy | Seed_ForTheWorthy | none | PORTED |
| NoTraps | Seed_NoTraps | none | PORTED |
| Remix | Seed_Remix | none (dontdigup) | PORTED |
| Everything | Seed_Everything | 7-dep cascade both ways + early-return disable guard | PORTED (row 11) |
| Skyblock | Seed_Skyblock | none | PORTED |

**No option is unported.** The evil-choice / world-size "sliders" are
NOT sliders: `UIWorldCreation.UpdateSliders` (`UIWorldCreation.cs:582-599`)
only re-selects the `GroupOptionButton<WorldSizeId/WorldDifficultyId/
WorldEvilId>` rows — the three-button rows landed with the base screen
(`world_creation.rs::size_buttons`/`evil_buttons`/`difficulty_buttons`,
`Click::Size/Evil/Difficulty`). Nothing to port there.

The one REAL gap found: the **Everything icon's glitch animation**.
`WorldSeedOption_Everything.ProvideUIElement`
(`WorldSeedOption_Everything.cs:69-98`) returns a `UIImageFramed` over a
7×16 sheet with a per-tick closure (frame 0 holds `Next(15,121)` ticks,
later frames advance after `Next(3,4)` == 3, frames 4/8/12 roll
`Next(3)` then `Next(7)` for the variation). The port drew the whole
sheet static. **Ported** (`700d1482`): `EverythingGlitch` walker +
`everything_frame_src` (`Utils.Frame(7,16,var,frame)`,
`Utils.cs:1344-1349`), ticked per frame from `frame.rs` on the bare
global stream (the front end runs outside every swap), reset at
`enter_front` (the C# closures are per-ctor), the live cell drawn at its
own size (`UIImageFramed.cs:20-21/48`). The other nine options are plain
whole-sheet `UIImage`s (`AWorldGenerationOption.cs:66-72`) — unchanged.

Witnesses: `world_creation_advanced.rs::the_everything_glitch_walks_the_frame_sheet`,
`::the_everything_frame_cell_matches_utils_frame` (written, not run).

## 3. ReportsPage — the SUBMIT census (answer: there is none)

`UIReportsPage.cs:16-248` is a **read-only log viewer**. Full census of
the report flow:

- **Producers**: `Main.IssueReporter` (`GeneralIssueReporter`,
  `Main.cs:2169/:10832`) — the ONLY non-Steam `AddReport` call site in
  the whole decompile is the resource-pack music replacement failure
  (`Audio/LegacyAudioSystem.cs:114-119`, "A resource pack failed to load
  {assetPath}!"). `SocialAPI.Workshop.IssueReporter`
  (`WorkshopIssueReporter.cs`) — Steam-only, fed by the publish/download
  failure arms (`WorkshopSocialModule.cs`, `AWorkshopPublishInfoState.cs:736`,
  `WorkshopHelper.cs`), 1000-entry ring (`:10/:18-26`).
- **Viewer**: `Main.OpenReportsMenu` (`Main.cs:52352-52363`) merges the
  two reporters (general first, Workshop when `SocialAPI.Workshop != null`),
  `PopulateLogs` orders by `timeReported` (`UIReportsPage.cs:144-146`).
  Entries: the Workshop hub's Logs button (`UIWorkshopHub.cs:295-300`,
  hides the indicator + sound 10) and the `IssueReportsIndicator`'s
  click (`IssueReportsIndicator.cs:64-68`).
- **SUBMIT**: the page has NO submit/send button or handler — the only
  click is `GoBackClick` (`:206-211`). The only "send" anywhere in the
  report flow is the **SteamUGC publish** (`Social/Steam/WorkshopHelper.cs:
  247-318`: `SteamUGC.CreateItem` → `StartItemUpdate` → `SetItemTitle/
  Description/Content/Tags/Preview/Visibility` → `SubmitItemUpdate`,
  results reported back through `ReportDelayedUploadProblem`). That is
  Steam-API-only (module 38 out-of-scope), NOT multiplayer netcode —
  there is no MP packet for reports either.

**Ported this sweep** (`ecbde4c4`): the `GeneralIssueReporter` data layer
on the App (`App.issue_reporter`), `open_reports` running the merge +
`GoToBottom` over the merged rows, and the Workshop ring rule pinned
(`workshop_add_report` + `WORKSHOP_MAX_REPORTS`). The log stays empty in
practice — the port has no resource-pack pipeline, the exact vanilla
state with no packs and no Steam. The indicator + Workshop hub entries
stay d84's lane.

Witnesses: `reports.rs::the_general_reporter_appends_and_merges`,
`::the_workshop_ring_caps_at_1000` (written, not run).

## 4. research/18 + 19 small rows taken

- **18's GoBack dead-code verification** (`ef9c8954`): full
  `decompiled/` + `decompiled-libs/` sweep for `.GoBack()` — 8 hits, all
  UIState-own private/static methods (`UICharacterCreation.cs:1207/:2378`,
  `UIWorldCreation.cs:667/:1375`, `UIWorldCreationAdvanced.cs:472/:530`,
  `UIWorldCreationAdvancedSecretSeedsList.cs:383/:410`); ZERO callers of
  `UserInterface.GoBack` (`UserInterface.cs:305-313`). The `_history`
  list (`:51`, appended `SetState` `:317-321`, trimmed at 32) is
  write-only in 1.4.5.8. Evidence recorded in `pointer.rs:6-15`; the
  research/18 seed + status row marked verified. The Advanced-tab seed
  marked done (rows 11-12 + this sweep's glitch icon).
- **19's S4 display trims** (`9b8cd40d`):
  `UICharacterNameButton.TrimDisplayIfOverElementDimensions`
  (`UICharacterNameButton.cs:106-131`) ported as
  `world_creation::trim_plate_display` — the int-truncated edge
  comparison, `Utils.TrimLastCharacter`'s surrogate guard
  (`Utils.cs:2839-2855`), the drop-one-more-plus-"…" tail (`:127-130`) —
  wired into `draw_plate` at padding 27 (name, `UIWorldCreation.cs:737`)
  / 40 (seed, `:746`, `WorldFileData.MAX_USER_SEED_TEXT_LENGTH`,
  `WorldFileData.cs:481`) on both the base screen and the Advanced tab.
  The seed's two citations (`UIWorldListItem.cs:418`,
  `UICharacterListItem.cs:199`) were misattributed — those are the
  rename-keyboard openers, which never call the trim; the only other
  call site is `UISearchBar.cs:83` (padding 0), left with the search-bar
  port. Deviation 14 updated; S4 closed.

Not taken (claimed/oversized): 19's M1 rename read-modify-write (f44's
save lane), M4 random world-name composition (needs the localization
tables + RNG-order proof), L1 UIWorldLoad (already landed S1-E-17 —
the row is stale), L2 `.trwld` metadata (c43/f45's lanes), 18's margins
(d87), list scissor (d88), layer stack (L, unclaimed but not small).

## Commits

| Hash | Mechanic |
| --- | --- |
| `6220169d` | loadout Apply minion-respawn half |
| `700d1482` | Everything seed icon glitch animation |
| `9b8cd40d` | creation plate display trims (19 S4) |
| `ef9c8954` | GoBack dead-code verification (18 seed) |
| `ecbde4c4` | reports reporter data layer + SUBMIT census |

Verification: code-only lane — no cargo/tests/graphify run; all witness
tests written, not run. Anchors rows added for the three code mechanics.
