# 101 — DOC-SWEEP lane 22: the temple/jungle USE-side census

Worker W4-g43 (`task/W4-g43-biome-jungle-temple`, the DOC-SWEEP phase after the
placement round `9420da89`). **Code-only**: file reads/writes + git only — no
cargo/rustc/tests, no compiler invoked; the witness tests are WRITTEN NOT RUN.
All C# cites are `decompiled/Terraria/…` (1.4.5.8); all Rust cites are
`crates/…` on this branch. The five lane items, each with the C# anchor, the
port verdict and the action taken.

## 0. Naming reconciliation (read first)

The brief's "solar tablet 2300 item use at altar → Golem summon" conflates two
distinct mechanics. The decompilation separates them:

| Brief phrase | Real C# | Port verdict |
| --- | --- | --- |
| "solar tablet 2300" | **item 2767** SolarTablet (`ItemID.cs:7217`); 2300 is a dye (`Item.cs:23143`'s dye block) | the 2767 USE arm was UNPORTED → landed |
| "use at altar → Golem summon" | the **Lihzahrd Power Cell 1293** at the altar tile 237 (`Player.cs:32760-32792`), NOT the tablet | ported (`use_golem_altar`), fidelity gaps fixed |
| "temple door 237? jungle key 1534?" | the door is **tile 10 style 11** (Lihzahrd Door item 1137, `Item.cs:13970-13981`); the key is **item 1141** TempleKey (`ItemID.cs:3965`); 237 is the altar TILE, 1534 is the Jungle Key (a biome chest key, `shimmer_tables.rs`) | the unlock USE arm was UNPORTED → landed |

The Solar Tablet starts a **solar eclipse** (`Main.eclipse`), it does not summon
the Golem. The Golem is summoned by the Power Cell at the altar. Both are now
ported faithfully.

## 1. Lihzahrd altar USE (Power Cell 1293 → Golem 245)

**C#**: `Player.TileInteractionsUse`'s `type == 237` branch
(`Player.cs:32760-32792`): `flag2 = true` (the click is consumed whether or not
the gates pass); the gates `!NPC.AnyNPCs(245) && Main.hardMode &&
NPC.downedPlantBoss` (`:32764`); the 58-slot inventory scan
(`for n < 58`, `:32766`) pays one Power Cell 1293 (`SetDefaults(0)` on empty);
`SoundEngine.PlaySound(15)` (the roar, `:32782`) then `NPC.SpawnOnPlayer(plr,
245)` (`:32785`). The cursor icon is 1293 (`:34309-34314`).

`NPC.SpawnOnPlayer`'s case-245 (`NPC.cs:80475-80531`): the ±20-tile scan
(`j < num2+20`, `k < num3+20` — EXCLUSIVE upper, `:80494/:80496`) for the altar
origin (`type == 237 && frameX == 18 && frameY == 0`, last match wins); the
`num4 <= 0 || num5 <= 0` refusal (`:80505`); the 50-tile `WorldGen.SolidTile`
scans up (`num8 > num5-50`) and down (`l < num5+50`) from the altar row
(`:80511-80526`); `NewNPC(X: num4*16+8, Y: (num6+num6+num7)/3*16)` (`:80527` —
the BOTTOM centre, `:80770` `nPC.Bottom = (X,Y)` → `position = (X-w/2, Y-h)`);
`target = plr` + `targetSetFrame = EverLastingTicker` (`:80528-80529`); the
`Announcement.HasAwoken` broadcast in `ChatColors.BossOrEvent` (`:80530`).

**Port (before)**: `editing/doors.rs::use_golem_altar` + `spawn_golem_on_altar`
— ported, but five deviations: the inclusive `..=` scan, no ≤0 refusal, the
top-left spawn (140 px too high for the 140×140 body), no target, the
server-message "The Golem has awoken!" literal; the cell scan was grid-only
(missed the coin/ammo rows); `SolidTile` lacked `!inActive()`.

**Action (commit `6a0546d2`)**: all five fixed + the `inventory[0..58]` scan
(grid + coins + ammo, NOT the trash — unlike the chest key's `<= 58`) + the
`!inActive()` conjunct. The existing altar witness re-pinned to the
bottom-centre coords + target; 1 new witness (the ammo-row cell pays/summons).
The Golem body's AI (`AI_045`, `npc_ai_golem.rs`) spawns the head/fists on its
first tick (`npc.rs:17102-17110`) — the summon needs only the body, verified.

## 2. Solar Tablet USE (item 2767 → solar eclipse)

**C#**: `Player.ItemCheck_UseEventItems` (`Player.cs:43988-44008`): the action
gate `ItemTimeIsZero && itemAnimation > 0 && sItem.type == 2767 && Main.dayTime
&& !Main.eclipse`; `SoundEngine.PlaySound(15)` (`:43990`), `ApplyItemTime`
(`:43991`), `Main.eclipse = true` (`:43994`), the `Lang.misc[20]` "A solar
eclipse is happening!" row (`ChatColors.World`), the remix `Lang.misc[106]`
variant (`:43995-44002`). The USE-START gate
`ItemCheck_CheckCanUse_Inner` (`:52269-52272`) additionally refuses while
`!Main.hardMode`. No boss spawns — the eclipse's spawn table rides the
spawner's `Main.eclipse` reads.

**Port (before)**: UNPORTED — the 2767 was absent from the `is_summon` gate
list and had no use arm.

**Action (commit `c95c2bcd`)**: `use_solar_tablet` (the day/eclipse/hardMode
gates folded, the `apply_moon_item` consume/re-arm, the flag + the localized
row with the remix variant); the 2767 joins `is_summon`. 2 witnesses (the
day-use pays/arms/flags; the three free refusals). The same commit lands the
**Abeemination 1133 `ZoneJungle` USE-START gate** (`:52253-52256`) that was
missing from `use_boss_summon_item`'s zone ladder (the old "1133 ungated"
comment described the action arm `:43861-43873`, not the start gate) + 1
witness.

## 3. Temple DOOR lock (Temple Key 1141, Plantera-gated entry)

**C#**: the door is tile 10 style 11 (Lihzahrd Door item 1137,
`Item.cs:13970-13981` `createTile = 10, placeStyle = 12` — note the item's
placeStyle 12 maps to the door's style window; the temple gen writes style 11,
`WorldGen.cs:33949`). `WorldGen.IsLockedDoor` (`:69191-69197`): tile 10,
`frameY` 594-646, `frameX < 54` — the Lihzahrd style-11 frames 594/612/630 ride
this window. `Player.TileInteractionsUse`'s `type == 10` branch
(`Player.cs:32793-32855`): `flag2 = true`; `IsLockedDoor` → the 58-slot scan
(`num48 < 58`, `:32800`) pays one Temple Key 1141 + `WorldGen.UnlockDoor`
(`:32810`); the `!flag9 && useVoidBag()` fallback walks bank4 (`:32818-32837`);
else `OpenDoor` (`:32841`, which itself refuses a locked door, `:37654-37657`).
`WorldGen.UnlockDoor` (`:37599-37628`): the up-walk to the `frameY == 594`
anchor (the `< 594` / `num <= 0` refusals), `SoundEngine.PlaySound(22, i*16,
num*16+16)` (`:37614`), the three-row `frameY += 54` (`:37621`, 594/612/630 →
648/666/684 — `IsLockedDoor` stops matching), 4 dusts/row (`:37622-37625`),
`NotifyProgressionEvent(22)` (`:37627`, TEMPLE_RAIDER,
`AchievementInitializer.cs:112-114`).

**The Plantera gate is the key's DROP, not a runtime check**: the Temple Key
drops from Plantera (`loot_db_gen.rs`'s NPC 262 row, `common(1141, 1, 1, 1)`).
There is no `downedPlantBoss` check in the door-use path — a pre-Plantera
player simply has no key, so the locked click is consumed free and the door
never opens. (The brief's "door use refusal pre-Plantera" is this emergent
gate, not an explicit flag read.)

**Port (before)**: `IsLockedDoor` ported (`edit.rs:992`), `open_door`'s refusal
ported — but the USE side was missing: a locked click fell into `open_door`'s
refusal and silently did nothing; the Temple Key was unusable.

**Action (commit `9498309a`)**: `edit::unlock_door` (the up-walk + the three-row
shift, returns the anchor row) + `is_locked_door` made pub;
`editing/doors.rs::unlock_temple_door` (the inventory scan + the void-bag
fallback + the Unlock-22 cue + progression 22). The door opens on the NEXT
click. 2 witnesses (the key pay + shift + next-click open + no-key free
refusal; the void-bag bank4 fallback gated on the Void Bag 4131). The 12 dusts
are the unported dust layer (the tile-edit dust precedent).

## 4. Lihzahrd furniture place-conditions (census-only)

**C#**: `Player.PlaceThing_Tiles_PlaceIt` (`Player.cs:40108-40170`) routes on
the registry def (`TileObject.Place`) or the legacy `WorldGen.PlaceTile` — NO
wall/biome/temple check anywhere in the placement path. The Lihzahrd furniture
(chair 1143, table 1144, workbench 1145, etc.) places anywhere its anchor
allows. The temple's protection is NOT a placement gate but:
- the **wiring gate** `CanDoWireStuffHere` (`:46920-46943`): wall 350 always
  refuses; wall 87 refuses until `downedGolemBoss` (the dual-dungeons temple
  style check `:46934`). Ported (`world_sim/editing/wiring.rs:61`).
- the **rod's wall-87 refusal** (`ItemCheck_UseTeleportRod` `:45508`): the
  Rod of Discord/Harmony refuses a teleport into wall 87 pre-Plantera below the
  surface (or anywhere on remix). Ported via the teleport-danger gate
  (`player.rs:11578`, `teleport_tile_is_dangerous` — the
  `TeleportHelpers.TileIsDangerous` wall-87 arm).
- the **210 pick-power gate** (`:53942`): Lihzahrd brick 226 / altar 237 need
  pick ≥ 210 (the Picksaw). Ported (`tool.rs:258`).

**Verdict**: NO temple-only placement gate exists in the C# — nothing to port.
The three real gates above are all ported. Census-only, no code.

## 5. Jungle shrine chests (c31 coordination)

**C#**: `JungleShrines` (`WorldGen.cs:15977-16140`): the hut search (the mud
`type == 60` guard `:16000`, the ±30 tile/hive/temple/wall exclusion scan
`:16004-16017`, the `structures.CanPlace` gate `:16018`), the brick shell + the
shrine walls + the torch + the mud pillars/cap. `ChestsInJungleShrines`
(`:17211-17244`): the `GetNextJungleChestItem` cycle, `AddBuriedChest(style 10)`
with the kill-and-retry fallback.

**Port**: `structures.rs::pass_jungle_shrines` (`:116-230`) + `pass_chests` —
ported (the shape-level port, ANALYSIS "Structures (M1.4-WGEN-2)"). The chest
loot is `chest_gen.rs`'s `GetNextJungleChestItem` cycle (`:813`).

**Coordination**: the jungle residuals are **c31's claimed scope**
(`task/W4-c31-hive-rewrite` — the hive/larva rewrite, phases 1-4 landed). The
shrine placement is already ported and is NOT a hive residual; no overlap.
Census-only, no code, no re-land.

## 6. Traps WIRING-side coverage (e20 coordination)

**C#**: `Wiring.HitWireSingle`'s case 137 (`Wiring.cs:1760-1993`): `frameY/18`
selects the trap — 0 dart (98, 20 dmg, 12 speed), 1 super-dart (184, 40, 12),
2 flame (187, 40, 5), 3 spiky-ball (185, 40, 4·dir + spread), 4 spear (186, 60,
8), 5 venom-dart (980, 30, 12); the wall negatives −10/−9/−8 (dart/super/flame)
and −7/−6 (ceiling spiky/spear). The temple places styles 1/2 (horizontal,
`mayan_trap_horizontal` `WorldGen.cs:8531-8537`: distance < 10 → style 2 flame,
else style 1 super-dart) and 3/4 (vertical, `:8603-8607`: distance < 16 → style
4 spear, else style 3 spiky-ball).

**Port (before)**: `tr-core/wiring.rs::trap_emit` emits all variants verbatim
(e20's lane — the wall-trap `vy` fix is theirs, NOT re-landed here). The app
drain (`world_sim/wiring.rs::spawn_device_kind`) spawns only kinds with a
`proj_data` row. Coverage: 98/184/980 (aiStyle 1, ported), 185 (aiStyle 14,
ported), 186 (stand-in row, aiStyle 37 unported — the M4-PROJ follow-up),
**187 (NO row, NO AI — the request dropped silently)**. The temple's style-2
flame traps fired nothing.

**Action (commit `3577b4619`)**: the `FLAMETHROWER_TRAP` (187) row
(`Projectile.cs:2741-2751`: 6×6, aiStyle 38, alpha 255, tileCollide false,
ignoreWater, timeLeft 60, trap) + the aiStyle-38 dispatch arm
(`ai_flamethrower_trap`, `AI()` `:27500-27512`): the stationary 6-tick emitter
of the 188 FlamesTrap children (`ai[0]++` to 6, reset, the `SoundID.Item34` cue
intent + the owner-gated `wants_dart_spawns` child with the jet's
velocity/damage/knockBack). `UpdatePosition`'s skip list (`:20435`) holds the
jet in place. The 188 child rides the existing aiStyle-23 `ai_eye_fire` flight.
The four stale "187 is a passthrough" test assertions retired/re-pinned. 1
witness (the stationary 6-tick emission). The spear 186 keeps its stand-in row
(aiStyle 37 stays the M4-PROJ follow-up); the wall-trap `vy` is e20's.

**Temple trap coverage after this sweep**: styles 1 (184), 2 (187→188), 3
(185), 4 (186 stand-in) all spawn. The only remaining gap is the 186's faithful
aiStyle-37 flight (the documented stand-in) — out of this lane's scope.

## Verification

Code-only mode: no cargo/rustc/tests invoked. 8 witness tests WRITTEN NOT RUN
(the tablet ×2, the Abeemination gate ×1, the Temple Key ×2, the altar ammo-row
×1, the flame jet ×1, plus the re-pinned altar coords/target and the four
retired 187-passthrough assertions). The deterministic world-hash pins are
untouched by this sweep (the USE-side arms draw no worldgen RNG; the flame-jet
row adds no gen-time draw). Commits: `c95c2bcd` (tablet + Abeemination),
`9498309a` (Temple Key), `6a0546d2` (altar fidelity), `3577b4619` (flame jet).
