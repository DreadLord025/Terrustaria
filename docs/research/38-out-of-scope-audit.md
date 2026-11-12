# 38 — Out-of-scope confirmation (Social/Steam/Testing)

> Wave 4 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The question this module answers: **what does the Rust port deliberately not port, and is
that exclusion honest?** The C# tree was swept for platform/services-shaped subsystems and
each was checked against `crates/` and against the docs' own out-of-scope statements.

The decompile contains three such families:

1. `Terraria/Social/**` — **72 files, ≈6 200 lines**: the whole social-API layer.
   `SocialAPI.cs` (139 lines) is the static hub with eight module slots
   (`Friends`, `Achievements`, `Cloud`, `Network`, `Overlay`, `Workshop`, `Platform`,
   `DLCs`) plus `JoinRequests`; `Base/` (24 files) holds the platform-neutral
   interfaces (`ISocialModule.cs` 8, `NetSocialModule.cs` 38, `CloudSocialModule.cs` 63,
   `RichPresenceState.cs` 58, `ServerJoinRequestsManager.cs` 74,
   `AWorkshopEntry.cs` 129, `WorkshopIssueReporter.cs` 91, …); `Steam/` (19 files,
   ≈2 490 lines: `WorkshopHelper.cs` 441, `NetClientSocialModule.cs` 267,
   `NetServerSocialModule.cs` 237, `WorkshopSocialModule.cs` 220, `Lobby.cs` 157,
   `SteamP2PReader.cs` 158, `SteamFileReadStream.cs` 155, `NetSocialModule.cs` 150, …);
   `WeGame/` (26 files, ≈2 850 lines: the Tencent IPC stack — `IPCBase.cs` 203,
   `NetClientSocialModule.cs` 556, `NetServerSocialModule.cs` 407, `WeGameP2PReader.cs`
   182, `WeGameP2PWriter.cs` 115, …).
2. `Terraria/Testing/**` — **34 files, ≈6 000 lines**: the in-game dev harness
   (`ToolkitDebugCommands.cs` 890, `StateSnapshot.cs` 473, `DetailedFPS.cs` 428,
   `QuickLoad.cs` 406, `WindowsPerformanceDiagnostics.cs` 413, `LockstepDebug.cs` 258,
   `RecordReplay.cs` 177, `StateSnapshotDefinition.cs` 181, the `ChatCommands/` and
   `Cloning/` (deep-clone codegen ≈1 200 lines) sub-trees).
3. Platform edges around them: `nativefiledialog.cs` (146, global namespace, P/Invoke to
   the `nfd` native library — `nativefiledialog.cs:10`), `Terraria/Utilities/FileBrowser/`
   (`FileBrowser.cs` 24 + `NativeFileDialog.cs` 15), `Terraria/Utilities/CrashDump.cs` 119 +
   `CrashWatcher.cs` 84, `Terraria/TimeLogger.cs` 839, `Terraria/GameContent/RGB/**`
   (53 files, the peripheral-lighting shaders + the Steam-Deck `VirtualKeyboard.cs` 450).

Two things the brief asked about **do not exist in 1.4.5.8** and are therefore not
exclusions at all: a `GameBanana` mod browser (zero hits for `gamebanana` across every
`.cs`/`.json`/`.txt` under `decompiled/`) and a `Terraria.Configuration` server-config
namespace (no `namespace Terraria.Configuration`; the only `ServerConfig` hits are the
world-gen options' `ServerConfigName`, e.g.
`decompiled/Terraria/WorldBuilding/WorldGenerationOptions.cs:104`). `Terraria.Mono` is
also absent — that namespace is 1.2-era.

## Key entities

- `Terraria/Social/SocialAPI.cs::SocialAPI.Initialize` (139-line file) — chooses
  `SocialMode.Steam` / `WeGame` / `None` and loads the module set; `SocialAPI.Network`
  is the slot every social call goes through. Nothing in `crates/` has an equivalent
  (no `SocialAPI`/`Social`/`Steam` identifier anywhere under `crates/`).
- `Terraria/IngameOptions.cs::IngameOptions.Draw` — `flag5 = SocialAPI.Network != null &&
  SocialAPI.Network.CanInvite()` (`IngameOptions.cs:178`), the invite row
  (`IngameOptions.cs:264` → `SocialAPI.Network.OpenInviteInterface()`) and the row
  count `num7 = 5 + (flag5 ? 1 : 0) + 2` (`IngameOptions.cs:179`). **Without Steam the
  vanilla hides the row too** — so the port's 8-row left column is 1:1, not a gap.
- `Terraria/Social/Base/ServerJoinRequestsManager.cs` (74) + `UserJoinToServerRequest.cs`
  (40) — the "Allowed Users" / join-request approval queue.
- `Terraria/Social/Base/AchievementsSocialModule.cs` (22) → `Terraria/Social/Steam/AchievementsSocialModule.cs`
  (129) — the Steam *unlock* half of achievements.
- `Terraria/Social/Base/CloudSocialModule.cs` (63) → `Steam/CloudSocialModule.cs` (108) —
  Steam Cloud saves.
- `Terraria/Social/Steam/WorkshopHelper.cs` (441) + `Steam/WorkshopSocialModule.cs` (220)
  + `Base/AWorkshopEntry.cs` (129) + `Base/TexturePackWorkshopEntry.cs` (11) — the
  Workshop hub (mods + texture packs).
- `Terraria/Social/Steam/NetClientSocialModule.cs` (267) / `NetServerSocialModule.cs`
  (237) / `Lobby.cs` (157) / `SteamP2PReader.cs` (158) — the Steam P2P transport and
  the lobby the 889 "Players"/"Allowed Users" screens read.
- `nativefiledialog.cs::NFD_OpenDialog` (146) → `Terraria/Utilities/FileBrowser/NativeFileDialog.cs`
  → `FileBrowser.OpenFilePanel` — the native file-open dialog.
- `Terraria/Testing/ChatCommands/ToolkitDebugCommands.cs` (890) — the in-game debug
  command table; `Terraria/Testing/RecordReplay.cs` (177) + `StateSnapshot.cs` (473) +
  `LockstepDebug.cs` (258) — the deterministic replay/snapshot harness.
- Rust counterparts that *do* exist: `crates/tr-app/src/achievements.rs` +
  `crates/tr-app/src/achievements_data.rs` + `crates/tr-app/src/ui/achievements.rs`
  (the local achievement registry/menu), `crates/tr-app/src/net.rs` (3 392 lines),
  `crates/tr-app/src/app/netglue.rs` (2 897), `crates/tr-app/src/app/netmodules.rs` (528)
  (the port-internal multiplayer), `crates/tr-app/src/menu/pages.rs::multiplayer_items`
  (the Steam-join hint row), `crates/tr-app/src/ui/keyboard.rs` (1 325, the virtual
  keyboard), `crates/tr-app/src/assets.rs:1567-1571` + `:1997-1998` (the cloud sheets
  loaded-but-unused).

## Dependencies

- **Calls into (C# → social):** `Terraria.Program`/`LaunchInitializer` →
  `SocialAPI.Initialize`; `IngameOptions.Draw` → `Network.CanInvite`/`OpenInviteInterface`;
  the multiplayer menu → `Friends.OpenJoinInterface`; `AchievementsHelper` →
  `AchievementsSocialModule`; save/load → `CloudSocialModule`; the workshop publish
  screen → `FileBrowser.OpenFilePanel` (`Terraria/GameContent/UI/States/AWorkshopPublishInfoState.cs:682`
  — **the file dialog's only caller**, so its absence is a *consequence* of the workshop
  exclusion, not an independent gap).
- **Called by (Rust):** `menu/pages.rs::multiplayer_items` (row 1 = "Join via Steam" →
  `Act::Hint("Steam join is not ported yet")`, `menu/pages.rs:~404-412`),
  `app/settings_ui.rs` (the in-game options window), `net.rs` (TCP transport),
  `assets.rs` (the unused cloud button sheets).
- **Shared data structures:** `lobby_id` (`net.rs:1006`, read at `:1142`, default `0` at
  `:3043`/`:3187`) is the port's stand-in for the Steam lobby id — documented at
  `docs/ANALYSIS.md:7271` ("`lobbyId` (0 without a social API)"); `NetworkText` is
  literal-mode only (`docs/ANALYSIS.md:7256-7260`); `SocialAPI.Cloud == null` is used as
  the *branch discriminator* the vanilla itself uses (`docs/ANALYSIS.md:2136`).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Terraria/Social/SocialAPI.cs` (module hub) | EXCLUDED-BY-DESIGN | — | No `Social*`/`Steam` identifier under `crates/` (docs-wide grep); ROADMAP.md:1743-1744 excludes `Social/*`. |
| `Terraria/Social/Steam/**` (19 files, ≈2 490 lines) | EXCLUDED-BY-DESIGN | — | Same. The Steamworks DLL sits next to the tree (`decompiled/Terraria.Libraries.Steamworks.NET.Windows.Steamworks.NET.dll`) and is not vendored by the port. |
| `Terraria/Social/WeGame/**` (26 files, ≈2 850 lines) | EXCLUDED-BY-DESIGN | — | Same, but **never named in the docs** — only the path `Social/*` covers it. |
| `Terraria/Social/Base/NetSocialModule` + Steam P2P (`NetClient/NetServer`, `Lobby`, `SteamP2PReader/Writer`) | EXCLUDED-BY-DESIGN (transport divergence, undocumented) | `crates/tr-app/src/net.rs:2579` `TcpListener::bind`, `:2868` `TcpStream::connect` | The port is **TCP-only**; the vanilla's Steam P2P (UDP-like, NAT-traversing) and the lobby browser have no analogue. Not stated anywhere in ROADMAP/ANALYSIS. |
| `Terraria/Social/Base/AchievementsSocialModule` → Steam unlock | EXCLUDED-BY-DESIGN | local only: `crates/tr-app/src/achievements.rs` | The *game* half (137-row registry, conditions, menu) **is** ported (M6-ACH-1/2, ROADMAP.md:995-1001, :1604-1605); only the Steam *reporting* half is absent. `achievements.rs:18-20` documents the `achievements.trdat` format deviation but never says "no Steam unlock". |
| `Terraria/Social/Base/CloudSocialModule` (Steam Cloud) | EXCLUDED-BY-DESIGN | — | `docs/ANALYSIS.md:2136` ("`SocialAPI.Cloud == null` in the port, exactly the branch the original takes without a cloud account"); `assets.rs:1567-1571`/`:1997-1998` load the cloud sheets "for completeness but unused". **Honest.** |
| `Terraria/Social/Base/WorkshopSocialModule` + `Steam/WorkshopHelper.cs` (441) + `AWorkshopEntry.cs` (129) | EXCLUDED-BY-DESIGN | — | The Steam Workshop hub (mods + texture packs) has no port. ROADMAP.md:955 lists "Workshop hub 13/16/17/18, ResourcePacks 17/15" among the C# screens **without** marking them excluded → the reader cannot tell it is a deliberate omission. |
| `Terraria/Social/Base/OverlaySocialModule`, `RichPresenceState.cs`, `FriendsSocialModule.cs`, `DLCSocialModule.cs`, `PlatformSocialModule.cs` | EXCLUDED-BY-DESIGN | — | Overlay / rich presence / friends / DLC entitlements / platform bridge. |
| `Terraria/Social/Base/ServerJoinRequestsManager.cs` (74) | EXCLUDED-BY-DESIGN | — | `docs/ANALYSIS.md:1949-1950`: the 889 "Allowed Users" rows "stay blank — the Steam-only lobby branch (`Main.cs:48749-48777`) never fills them without `SocialAPI.Network`". **Honest.** |
| Steam invite row in `IngameOptions` (`IngameOptions.cs:178/:264`) | PRESENT-BY-CORRECT-OMISSION | `crates/tr-app/src/app/settings_ui.rs` | `flag5` is false without a social API, so the vanilla hides the row too; the port's 8 rows match `num7` (`:179`). **Not a gap.** |
| "Join via Steam" menu row (`Friends.OpenJoinInterface`) | PRESENT-AS-HINT | `crates/tr-app/src/menu/pages.rs:~404-412` | The row is kept (vanilla row shape) and its action is `Hint("Steam join is not ported yet")`; documented in `docs/ANALYSIS.md:1942-1951`. **Honest**, but a user-facing dead row. |
| `Terraria/Testing/**` (34 files, ≈6 000 lines) | **UNDOCUMENTED-ABSENCE** | — | No mention in ROADMAP non-goals, ANALYSIS or PORT_MAP. Includes the deterministic harness (`RecordReplay` 177, `StateSnapshot` 473, `LockstepDebug` 258, `QuickLoad` 406) and the in-game debug command table (890). The only related doc line is `docs/ANALYSIS.md:4143` (the *game's* `TheDebugOptions.ManaV2` cheat), which is a different file. |
| `Terraria/GameContent/RGB/*Shader*.cs` (peripheral lighting) | EXCLUDED-BY-DESIGN | — | ROADMAP.md:1151 ("**exclude** (platform-specific, like `Social/*`)") + non-goals ROADMAP.md:1743-1744. The file count (53) still matches the tree. |
| `Terraria/GameContent/RGB/VirtualKeyboard.cs` (450) + `DebugKeyboard.cs` (48) | **PRESENT** (path claim over-broad) | `crates/tr-app/src/ui/keyboard.rs` (1 325) | `docs/PORT_MAP.md:179` — "`UIVirtualKeyboard.cs` → `crates/tr-app/src/ui/keyboard.rs` — done (visual; desktop layer only)"; the 50-key grid sits behind `TR_VK_GRID=1` (`crates/tr-app/src/env.rs:700-702`). The non-goal names the whole `GameContent/RGB/*` path, so the keyboard is excluded by wording yet ported in fact. |
| `Terraria/GameContent/UI/States/UIBestiaryTest.cs` + `UITextWrappingTest.cs` | **UNDOCUMENTED-ABSENCE** | — | In-game UI test screens; ROADMAP.md:955 lists "BestiaryTest 17" as a C# screen with no port marker. |
| `Terraria/TimeLogger.cs` (839) | **UNDOCUMENTED-ABSENCE** | — | Frame-time profiler that writes a zip; no port analogue, no doc mention. |
| `Terraria/Utilities/CrashDump.cs` (119) + `CrashWatcher.cs` (84) | **UNDOCUMENTED-ABSENCE** | — | MiniDump-on-crash; the port has no crash handler. |
| `nativefiledialog.cs` (146) + `Terraria/Utilities/FileBrowser/**` | EXCLUDED-BY-DESIGN (consequence) | — | Its only caller is the workshop publish screen (`AWorkshopPublishInfoState.cs:682`) → covered by the workshop exclusion. |
| `Terraria/Utilities/MessageBox.cs` (11), `PlatformUtilities.cs` (11) | EXCLUDED-BY-DESIGN | — | Win32 message box / platform probe; the port is native-Rust. |
| ReLogic `ReLogic.OS.*` (Windows/Linux/OSX) + `ReLogic.Peripherals.RGB.*` | EXCLUDED-BY-DESIGN | n/a (Rust-native windowing) | `decompiled-libs/ReLogic/` platform packages; replaced by wgpu/winit. |
| `Main.dedServ` dedicated-server mode (169 sites) | **UNDOCUMENTED-ABSENCE** | — | The vanilla's headless server (`-server`, `SocialAPI.Network = NetServerSocialModule`, server config) has no port; the port hosts in-window on TCP 7777 (`net.rs:2579`). The non-goals only promise that "the handshake (M6-NET-1) is port-internal". |
| Achievements (game-side, **not** platform) | **PRESENT** | `crates/tr-app/src/achievements.rs`, `achievements_data.rs`, `ui/achievements.rs` | 137-row registry, tracker, menu; ROADMAP.md:995-1001 + M6-ACH-1/2 rows (:1604-1605) DONE. |
| Multiplayer protocol (**not** platform) | **PRESENT** (port-internal) | `crates/tr-app/src/net.rs` (3 392), `app/netglue.rs` (2 897), `app/netmodules.rs` (528) | M6-NET-1..5 "done 2026-11-09" (ROADMAP.md networking row). Handshake states 0→10, entity/tile/netmodule/projectile sync. |

## Tests

- **No cargo test can cover the excluded subsystems** — they are absent by construction.
  A `grep "#\[test\]"` over `crates/` finds no `social`/`steam`/`workshop`/`cloud` test
  module: the only near-miss names are `craft.rs:114` `CLOUD_BALLOONS` (the item family)
  and `creative_sacrifices.rs:362` "Banana"/"BananaDaiquiri" (sacrifice rows) — both
  unrelated to the cloud/workshop services.
- The **surviving** social-adjacent surface has tests:
  `crates/tr-app/src/achievements.rs:439-543` — seven `#[test]` bodies (registry
  round-trip, condition/tracker behaviour, save/load of `achievements.trdat`,
  fresh-registry load); `crates/tr-app/src/net.rs` + `app/netglue.rs` carry the
  handshake/codec tests (not enumerated in this pass).
- The excluded `Terraria/Testing/**` harness means the port has **no replay/snapshot
  parity test** equivalent to `RecordReplay`/`StateSnapshot`/`LockstepDebug` — its 1:1
  claims rest on unit tests + screenshot hashes only.

## Deviations & risks

1. **README is stale on multiplayer — severity: cosmetic (doc), player-facing.**
   `README.md:171` lists "**Networking** (multiplayer)" under "What is *not* implemented
   yet", but `net.rs` (3 392 lines) + `netglue.rs` (2 897) + `netmodules.rs` (528) and the
   M6-NET-1..5 rows (done 2026-11-09) contradict it. **FALSE.**
2. **README is stale on the in-game options window — severity: cosmetic (doc).**
   `README.md:166-168` says "only the in-game options window vanilla opens over the
   inventory is still missing", but M7-UI-4 is **done 2026-11-11** (ROADMAP.md:1652).
   **FALSE.**
3. **ROADMAP non-goal "until M6-NET is complete" is stale — severity: cosmetic (doc),
   planning-relevant.** `docs/ROADMAP.md:1740-1742` still frames the network gap as
   "until M6-NET is complete" and says "the entity/tile/module sync is still
   single-player-first". M6-NET-1..5 all landed (ROADMAP.md networking row, done
   2026-11-09), so the *real* remaining gap is the **excluded Steam transport/lobby
   layer**, not an unfinished M6-NET. **FALSE/STALE.**
4. **The RGB non-goal is over-broad by path — severity: cosmetic.** `docs/ROADMAP.md:1151`
   and `:1743-1744` exclude "RGB peripheral lighting (`GameContent/RGB/*`)". That folder
   also holds `VirtualKeyboard.cs` (450, the Steam-Deck on-screen keyboard — **ported**,
   PORT_MAP.md:179), `DebugKeyboard.cs` (48), `CommonConditions.cs` (399) and
   `NoiseHelper.cs` (82). The exclusion should name the `*Shader*.cs` files plus the
   peripheral packages, otherwise the docs contradict PORT_MAP.
5. **`Terraria.Social` is excluded as a *path*, never as modules — severity: behavioural
   (doc).** No doc enumerates the eight modules. A reader sees a **fully ported
   achievement UI** (`ui/achievements.rs`) and can only guess that the Steam *unlock*
   is absent; `achievements.rs:18-20` documents only the save-format deviation. Same
   for the TCP-vs-P2P transport, the workshop, the overlay, rich presence and DLC.
6. **`Terraria/Testing/**` (34 files, ≈6 000 lines) is an UNDOCUMENTED-ABSENCE —
   severity: medium.** It contains the vanilla's deterministic replay/snapshot harness
   (`RecordReplay.cs` 177, `StateSnapshot.cs` 473, `LockstepDebug.cs` 258,
   `QuickLoad.cs` 406) — precisely the tool a "1:1 reimplementation" project would use to
   prove parity — plus the in-game debug command table (890) and the deep-clone codegen
   (≈1 200). Nothing in ROADMAP/ANALYSIS/PORT_MAP admits the omission.
7. **The in-game dev screens are an UNDOCUMENTED-ABSENCE — severity: cosmetic.**
   `UIBestiaryTest.cs` (26 974 B) and `UITextWrappingTest.cs` (12 990 B) appear in
   ROADMAP.md:955's screen inventory with no port/exclusion marker.
8. **`Terraria/TimeLogger.cs` (839) and `Terraria/Utilities/CrashDump.cs` (119) +
   `CrashWatcher.cs` (84) are UNDOCUMENTED-ABSENCES — severity: low** (dev tooling /
   crash reporting; the port has no crash handler at all).
9. **Dedicated-server mode (`Main.dedServ`, 169 sites) is an UNDOCUMENTED-ABSENCE —
   severity: medium (behavioural).** The docs promise only a "port-internal" handshake;
   they never say the vanilla's headless `-server` mode is not ported. For a "services"
   module this is the single largest honest gap.
10. **`Terraria.Social.WeGame.*` (26 files, ≈2 850 lines) is never named** — only the
    path `Social/*` covers it — **severity: cosmetic (doc)**.
11. **README never mentions the platform exclusions at all — severity: cosmetic (doc).**
    `README.md:142-172` lists gameplay gaps but says nothing about Steam/WeGame, cloud
    saves, the Workshop/mod browser, peripheral RGB or dedicated server, while the
    README's pitch is "the same game with a different engine underneath"
    (`README.md:3-8`). A one-line "not included by design" list would close it.
12. **Dead user-facing row** — the Multiplayer's "Join via Steam" row is a hint only
    (`menu/pages.rs:~404-412`); documented, but it is a visible dead end.
    Severity: cosmetic.
13. **DONE-row re-verification (this module's rows):**
    - `M6-NET-1..5` (done 2026-11-09) — **TRUE** for the protocol: `net.rs` carries the
      frame codec, the 0→10 handshake states, the entity/tile/netmodule/projectile
      messages and `netmodules.rs` the `82` envelope. **Caveat**: it is TCP-only, so
      "the multiplayer protocol" is ported while "the Steam multiplayer" is not.
    - `M6-ACH-1` / `M6-ACH-2` (done 2026-11-10) — **TRUE** for the registry/menu
      (137 rows, `achievements_data.rs`, `ui/achievements.rs`); the *Steam reporting*
      half was never in scope, and is not admitted anywhere.
    - `M7-UI-4` (done 2026-11-11) — **TRUE** and correctly scoped: the Steam invite row
      is gated by `flag5` (`IngameOptions.cs:178`) and the port's 8-row column matches
      the vanilla's no-Steam count (`IngameOptions.cs:179`).
    - `M6-CHAT-1` / `M6-EMOTE-1` — the chat command surface is ported; the vanilla's
      *testing* command table (`Terraria/Testing/ChatCommands/`) is a different,
      unported set.

## Task seeds

- **S — Refresh `README.md` "What is *not* implemented yet"** (drop the multiplayer and
  in-game-options claims) **and add a "Not included by design" list**: Steam/WeGame
  integration, cloud saves, the Steam Workshop/mod browser + texture packs, peripheral
  RGB lighting, dedicated-server mode, modding/tModLoader. *No deps.*
- **S — Correct the ROADMAP non-goals** (`docs/ROADMAP.md:1736-1744`): replace "until
  M6-NET is complete" with the actual remaining gap (the Steam transport/lobby/social
  layer), drop "the entity/tile/module sync is still single-player-first" (M6-NET-2..5
  landed), narrow the RGB path to `*Shader*.cs` + the peripheral packages, and name
  WeGame explicitly. *No deps.*
- **S — Add an ANALYSIS subsection "Excluded subsystems (platform & services)"** that
  enumerates the eight `SocialAPI` modules one by one with the port's substitute for
  each (local achievements + `achievements.trdat`; TCP transport instead of Steam P2P;
  blank lobby/Allowed-Users rows; the "Steam join is not ported yet" hint row; no cloud
  save; no workshop). *No deps.*
- **S — Add PORT_MAP rows for the excluded C# paths** so the cross-language map shows
  the gap rather than silence: `Terraria/Social/**`, `Terraria/Testing/**`,
  `nativefiledialog.cs`, `Terraria/TimeLogger.cs`, `Terraria/Utilities/CrashDump.cs`,
  `Terraria/GameContent/RGB/*Shader*.cs`. *No deps.*
- **M — Decide the `Terraria/Testing` harness** (dev-tooling parity): either (a) record
  the exclusion in ROADMAP non-goals + ANALYSIS (cheap, honest), or (b) port a Rust
  equivalent of the deterministic replay/snapshot pair (`RecordReplay` 177 +
  `StateSnapshot` 473 + `StateSnapshotDefinition` 181) as the project's parity-proof
  tool — needs a design note first, then an L implementation. *Depends on nothing;
  (b) blocks nothing else.*
- **M — Decide and document the dedicated-server story.** The C# has 169 `dedServ` sites
  (`Main.dedServ`, `SocialAPI.Network = NetServerSocialModule`, the server config); the
  port has none. Write a one-page decision (host-in-window only vs a headless
  `-server` mode) and record it in ANALYSIS/ROADMAP. *Depends on the README/ROADMAP
  refresh (S) so the two agree.*
- **M — Verify the achievements split**: confirm the vanilla's *local* unlock is inside
  `AchievementManager` (not inside `AchievementsSocialModule`), so the port's local
  achievement behaviour is complete and only the Steam push is missing; add the finding
  to ANALYSIS. *No deps.*
- **L — Optional: a trait-shaped `SocialApi` seam in Rust** (a `NoSocial` impl behind
  the same method names the C# uses) so the port's call sites mirror the vanilla's
  structure and a future Steam backend becomes a drop-in. *Only if the owner wants the
  1:1 structure; not needed for parity today.*
- **L — Do NOT schedule**: Steam Workshop hub / mod browser, Steam Cloud, WeGame IPC,
  peripheral RGB, `Terraria.Mono` (non-existent), the GameBanana browser
  (non-existent in 1.4.5.8). Record these as closed non-goals so the next audit does
  not re-open them.
