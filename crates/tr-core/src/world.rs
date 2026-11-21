//! In-memory world: tile grid + metadata.

use crate::tile::Tile;

/// Chunk edge in tiles. Occupancy is per chunk and drives both draw culling and
/// (later) parallel light work.
pub const CHUNK: i32 = 64;

#[derive(Clone, Debug)]
pub struct World {
    pub name: String,
    /// Sparse per-tile extras for the rare paint/coating states that do not fit
    /// the packed 12-byte `Tile` (`WorldGen.paintWall`/`paintCoatWall`,
    /// `WorldGen.cs:44189-44253`): wall paint (5 bits),
    /// `invisibleBlock`/`invisibleWall`/`fullbrightWall` (one bit each) —
    /// `C# Tile.bTileHeader&0x1F` (wall color) and the `bTileHeader3`/
    /// `sTileHeader` coating bits. Keyed by `index(x, y)`; absent = zero.
    /// Tile-side paint lives in `Tile.meta`, tile fullbright in `Tile.flags`
    /// (`FLAG_FULLBRIGHT`).
    pub extras: std::collections::HashMap<u32, u8>,
    /// The four wire colours as a sparse side table (`Tile.wire()`/`wire2()`/
    /// `wire3()`/`wire4()` are `sTileHeader` bits in the C# — `Tile.cs:623-657`
    /// — and the packed 12-byte `Tile` carries no header). Written by the
    /// wire-placing items, read by the wiring engine (`crate::wiring`).
    pub wires: crate::wiring::WireState,
    /// `TileEntity.ByID`/`ByPosition` (`TileEntity.cs:15-16`) — the placed
    /// tile entities and their position book (`crate::tile_entities`).
    pub tile_entities: crate::tile_entities::TileEntities,
    /// `Main.sign[0..31999]` (`Sign.cs:8`, `SaveSigns:1678`): the placed signs.
    /// The store holds exactly the non-null signs in slot order, which is the
    /// whole observable state of the C# array — `LoadSigns` defines every slot,
    /// so the null slots carry nothing. The section's fix-up halves live in
    /// `crate::wld::sections::signs`.
    pub signs: Vec<crate::wld::sections::WldSign>,
    /// `PressurePlateHelper.PressurePlatesPressed`
    /// (`LoadWeightedPressurePlates:3373`): the pressed plates, points only —
    /// the `bool[255]` pressure arrays are rebuilt on load and never persist.
    pub pressure_plates: Vec<crate::wld::sections::WldPressurePlate>,
    pub width: i32,
    pub height: i32,
    /// Row-major tile storage: `tiles[y * width + x]`.
    pub tiles: Vec<Tile>,
    /// `Main.tileFrameImportant`: tile types whose frames are stored in the save file.
    pub frame_important: Vec<bool>,
    /// Per-tile framing variants packed 2+2 bits — `Tile.bTileHeader2` bits 4-5
    /// (`frameNumber`) and bits 6-7 (`wallFrameNumber`) (`Tile.cs:464-482`).
    /// Keyed by `index(x, y)`: `frameNumber | wallFrameNumber << 2`. These are
    /// runtime-only fields in the C# (`SaveWorldTiles` `WorldFile.cs:1432-1604`
    /// never writes them), rolled from `WorldGen.genRand.Next(0, 3)` when a
    /// framing event resets (`WorldGen.cs:81705-81729`, `Framing.cs:318-321`,
    /// `Framing.cs:390-396`) and reused from storage otherwise.
    pub variants: Vec<u8>,
    /// The runtime reset-roll stream of the framing variants — the port's
    /// `WorldGen.genRand` stand-in for `SquareTileFrame`'s centre re-roll
    /// (`WorldGen.cs:81705-81709`, `Framing.cs:389-395`), the same shape as
    /// [`crate::wiring::WireState`]'s `rand` (S1-L-6c). Vanilla reseeds
    /// `Main.rand` from the clock on load, so the runtime rolls are not
    /// reproducible there; the port seeds deterministically from the world seed
    /// via [`World::seed_frame_rng`].
    pub frame_rng: crate::unified_random::UnifiedRandom,
    /// The placement random-style stream — the port's `Main.rand` stand-in
    /// for `TileObject.CanPlace`'s `RandomStyleRange`/`SpecificRandomStyles`
    /// draws (`TileObject.cs:787`/:819, S1-F-7b). A dedicated stream like
    /// `craft_rng`/`frame_rng` (vanilla shares one `Main.rand`; the port keeps
    /// per-consumer streams so the worldgen contract streams stay untouched),
    /// seeded beside `frame_rng` via [`World::seed_frame_rng`].
    pub place_rng: crate::unified_random::UnifiedRandom,
    /// `TileObjectPreviewData.randomCache` (`TileObjectPreviewData.cs:31`) —
    /// the random-style stickiness cache `TileObject.CanPlace` reads
    /// (`TileObject.cs:764-792`/:796-824). `None` = the C# null static (the
    /// null-create :764-767 zero-initialises); the only writer is the
    /// `onlyCheck` preview writeback (:826-837), which the port does not run,
    /// so production stays `None`.
    pub tile_object_random_cache: Option<crate::tileobject::RandomCache>,
    /// Y of the first solid tile at the center of the map (approx. spawn altitude).
    pub spawn_y: i32,
    /// The bed spawn override (`Player.SpawnX/SpawnY`, `Player.cs:54635`'s
    /// `ChangeSpawn`) — `None` = the generated spawn (`RemoveSpawn` clears).
    pub bed_spawn: Option<(i32, i32)>,
    /// Spawn point from the save header (tile coordinates).
    pub spawn_x: i32,
    /// `Main.dungeonX` (`Main.cs:1115`) — the dungeon anchor column. Reset
    /// clears it to 0 (`WorldGen.cs:7047`); the skyblock noDungeon arm sets
    /// −1 (`:3229`); the dungeon-brick discovery scans write it
    /// (`:72486`/`:73796`). `initializeHardMode`'s anchor bias reads it
    /// (`:31593`). The `.wld` header field (`WorldFile.cs:1281`).
    pub dungeon_x: i32,
    /// `Main.dungeonY` (`Main.cs:1117`) — the dungeon anchor row (the same
    /// reset/discovery sites as [`Self::dungeon_x`]).
    pub dungeon_y: i32,
    /// `Main.worldSurface`: below this row the sun does not reach the surface.
    pub world_surface: f64,
    /// `Main.rockLayer`: top of the underground rock layer.
    pub rock_layer: f64,
    /// `Main.time`: world time in ticks (0..=54000).
    pub time: f64,
    /// `Main.dayTime`: whether the saved world is in daylight.
    pub day_time: bool,
    /// `Main.moonType` (0..8): which `Images/Moon_*` sheet is drawn.
    pub moon_type: u8,
    /// `Main.moonPhase` (0..7): the current-moon row in the sheet; advances one
    /// step each time night turns into day (`Main.cs:19982`).
    pub moon_phase: u8,
    /// `Main.sundialCooldown` (`Main.cs:2161`): ticks down once per dawn until
    /// the sundial may fast-forward again (0..=8). Saved in TRWLD v4.
    pub sundial_cooldown: u8,
    /// `Main.moondialCooldown` (`Main.cs:2165`): ticks down once per dusk
    /// (0..=8). Saved in TRWLD v4.
    pub moondial_cooldown: u8,
    /// `Main.fastForwardTimeToDawn` (`Main.cs:2159`): the sundial's fast-forward
    /// request, consumed by the next dawn transition. Runtime-only, not saved
    /// (the C# has written it to `.wld` since v264, but it is transient).
    pub fast_forward_to_dawn: bool,
    /// `Main.fastForwardTimeToDusk` (`Main.cs:2163`): the moondial's
    /// fast-forward request, consumed by the next dusk transition. Runtime-only.
    pub fast_forward_to_dusk: bool,
    /// `Main.hellBackStyle` (0..2): picks the animated magma sheet `Background_125..127`
    /// and the hell floor row `Background_185..187` (`Main.cs:54000-54008`). Vanilla
    /// rolls it in `WorldGen.RandomizeCaveBackgrounds` (`WorldGen.cs:8247`); it is not
    /// part of the `.wld` format and is re-rolled per session at load (`Main.cs:6603`),
    /// so the port fixes it at generation like `moon_type`. Loaded worlds keep 0.
    pub hell_back_style: u8,
    // Per-biome surface background *style* variants (`WorldGen.corruptBG`,
    // `jungleBG`, `hallowBG`, `crimsonBG`, `mushroomBG`). The `.wld` stores the
    // style byte (`IO/WorldFile.cs:1318-1325`, `:1375`) and `WorldGen.setBG`
    // expands it into the three `Background_<id>` textures — see
    // [`surface_bg_texture_ids`].
    pub corrupt_bg_style: u8,
    pub jungle_bg_style: u8,
    pub hallow_bg_style: u8,
    pub crimson_bg_style: u8,
    pub mushroom_bg_style: u8,
    /// Y of the first solid tile in each column (background/sky horizon).
    pub column_surface: Vec<i32>,
    /// Chunk grid size (`ceil(width / CHUNK) * ceil(height / CHUNK)`).
    pub chunks_x: i32,
    pub chunks_y: i32,
    /// One flag per chunk: contains at least one active tile, wall or liquid.
    pub chunk_has: Vec<bool>,
    /// Generation seed (`0` for worlds loaded from the original `.wld` format).
    pub seed: u64,
    /// The classic secret-seed flags (`Main.drunkWorld` & friends — the
    /// `WorldGen.Reset()` set, `WorldGen.cs:11115-11123`): the world's
    /// special-world identity. Runtime readers (loot/fish/variant/wind
    /// conditions) and the save layers read them; the worldgen passes get
    /// them through [`crate::worldgen::GenOptions`]. Persisted in TRWLD v6
    /// and read from the `.wld` header flags block.
    pub seeds: crate::seeds::SeedFlags,
    /// M5-SEED-2: the 37 modular micro-seed flags (`WorldGen.SecretSeed`,
    /// `WorldGen.cs:341-413`) — the `activeSecretSeedCount` population and
    /// the per-seed gates the worldgen arms and the runtime readers read.
    /// Persisted in TRWLD v7; `.wld` worlds re-derive them from the seed
    /// text (`WorldFileData.SetSeed` + `CheckInputForSecretSeed`), because
    /// the C# stores no micro-seed bytes in the file.
    pub micro_seeds: crate::seeds::MicroSeedFlags,
    /// Bumped by every tile/wall/liquid mutation; the light (D2) and instance (G6)
    /// caches key on it so an edit invalidates them. Loading does not bump it.
    pub revision: u64,
    /// `Main.eclipse` — the hardmode event flag (readers: `AmbienceServer
    /// .cs:34`, `:66`). Persisted in the flags block — the C# writes the
    /// save-time snapshot `_tempEclipse` (`WorldFile.cs:1280`).
    pub eclipse: bool,
    /// `Main.bloodMoon` (`AmbienceServer.cs:50`); persisted the same way
    /// (`_tempBloodMoon`, `WorldFile.cs:1279`).
    pub blood_moon: bool,
    /// `Main.AnnouncementBoxDisabled` (`Main.cs:3073`): the `-disableannouncementbox`
    /// launch switch gates the announcement box's case 425 (`Wiring.cs:1190`).
    pub announcement_box_disabled: bool,
    /// `Main.AnnouncementBoxRange` (`Main.cs:66530`, stock −1): the range in
    /// px inside which the box's text reaches the player (`Wiring.cs:1202`).
    pub announcement_box_range: i32,
    /// `Main.raining` / `Main.rainTime` / `Main.maxRaining` — the rain
    /// event's persisted state. The C#'s flag block writes the save-time
    /// snapshot (`_tempRaining`/`_tempRainTime`/`_tempMaxRain`,
    /// `WorldFile.cs:1312-1314`; `SetTempToOngoing` `:1058-1060` copies the
    /// live values before the write) and `SetOngoingToTemps` (`:159-162`)
    /// restores them on load — `Main.cloudAlpha` comes back as `maxRaining`.
    /// The live state lives on the App's `Rain` (`tr-app/src/rain.rs`);
    /// these are the file payload the save path syncs both ways (S1-J-7).
    pub raining: bool,
    pub rain_time: i32,
    pub max_raining: f32,
    /// `Sandstorm.Happening` / `TimeLeft` / `Severity` / `IntendedSeverity`
    /// (`Sandstorm.cs:14-20`): the sandstorm event's persisted state. The
    /// C# writes the save-time snapshot `_tempSandstorm*` in the flags block
    /// (`WorldFile.cs:1369-1372`, filled by `SetTempToOngoing:1053-1056`) and
    /// restores it on load (`SetOngoingToTemps:169-172`). The live state lives
    /// on the App (`App.sandstorm`, `tr_core::scene::Sandstorm`); these are the
    /// file payload the save path syncs both ways.
    pub sandstorm_happening: bool,
    pub sandstorm_time_left: i32,
    pub sandstorm_severity: f32,
    pub sandstorm_intended_severity: f32,
    /// `CurrentFrameFlags.SleepingPlayersCount == ActivePlayersCount &&
    /// SleepingPlayersCount > 0` (`Main.cs:17856-17859` → `UpdateTimeRate`'s
    /// all-asleep flag, `Main.cs:6209`): every present player is fully asleep
    /// (`PlayerSleepingHelper.FullyFallenAsleep`, 120 ticks). The sim feeds it
    /// per frame — `Main.UpdateWorld_Players` computes the counts before
    /// `UpdateTime` reads them. Runtime-only, not saved.
    pub all_players_asleep: bool,
    /// The downed-boss flags (`NPC.downed*` statics, `NPC.cs:6340-6380`):
    /// the world-persistent boss states. The save wire rides the .wld
    /// flags block (M6-SAVE's world sections).
    pub downed_slime_king: bool,
    /// `NPC.downedQueenSlime` (`Chest.cs:3428`'s stock gate) — the vanilla
    /// `.wld` flags block carries it (`WorldFile.cs:1396`); the port's own
    /// save rides the v15 tail (S1-F-16).
    pub downed_queen_slime: bool,
    pub downed_eoc: bool,
    pub downed_eow: bool,
    pub downed_skeletron: bool,
    pub downed_queen_bee: bool,
    pub downed_mech_boss1: bool,
    pub downed_mech_boss3: bool,
    pub downed_mech_boss2: bool,
    pub downed_plant_boss: bool,
    /// The Goblin Army invasion (`Main.invasionType = 1`).
    pub invasion_goblins: bool,
    /// The remaining invasion enemies (`Main.invasionSize`).
    pub invasion_size: i32,
    /// `World.downedGoblins` — the Goblin Tinkerer's gate.
    pub downed_goblins: bool,
    /// `NPC.downedFrost` (`NPC.cs:6339`): the defeated Frost Legion.
    pub downed_frost: bool,
    /// `NPC.downedPirates` (`NPC.cs:6343`): the defeated Pirate Invasion.
    pub downed_pirates: bool,
    /// `NPC.downedMartians` (`NPC.cs:6347`): the defeated Martian Madness.
    pub downed_martians: bool,
    /// `NPC.savedGoblin` — the rescue flag (`AI_007_TownEntities_
    /// UpdateSavedStates` case 107, `NPC.cs:52944-52946`).
    pub saved_goblin: bool,
    /// The Brain of Cthulhu's anchor (`NPC.crimsonBoss`'s centre) — the
    /// Creepers' orbit reads it (the WoF's write-through shape).
    pub brain_present: bool,
    pub brain_x: f32,
    pub brain_y: f32,
    /// `NPC.downedGolemBoss` (`:6365`) — the Golem's defeat (the event
    /// flag 6, `NPC.cs:79346`'s case 245).
    pub downed_golem: bool,
    /// `NPC.downedFishron` (`NPC.cs:6369`) — Duke Fishron's defeat (the
    /// event flag 7, `NPC.cs:79349`'s case 370). S1-D-3 W-9 fishron
    /// wiring.
    pub downed_fishron: bool,
    /// `NPC.downedDeerclops` — the frozen boss's defeat (the event flag
    /// 25, `NPC.cs:79355`).
    pub downed_deerclops: bool,
    /// The Skeletron Prime head's centre + the present flag (the limbs'
    /// tether reads — the WoF write-through shape).
    pub prime_present: bool,
    pub prime_x: f32,
    pub prime_y: f32,
    /// The head's `position.X` — the faithful limb arms' INTEGER
    /// half-width centres read it (`NPC.cs:28249` etc.); the legacy
    /// `prime_x` stays the FLOAT centre for the v1 saw arm.
    pub prime_head_x: f32,
    /// The head's `width` — an INT in the C# (`width / 2` is INTEGER
    /// division, `NPC.cs:28249`/`:28528`).
    pub prime_head_width: i32,
    /// The head's `velocity` — the vice's rise/fly X/Y blends read it
    /// (`NPC.cs:28446`/`:28489`).
    pub prime_head_vx: f32,
    pub prime_head_vy: f32,
    /// The head's `ai[1]` phase: 0 hover, 1 charge, 2 rage, 3 despawn
    /// (`NPC.cs:28326`/`:28537`/`:28772`'s limb gates).
    pub prime_head_ai1: f32,
    /// The Skeletron head's centre (`NPC.cs:21930`'s spawn point + the
    /// hands' tether reads) — the skeletron_x/y pair.
    pub skeletron_x: f32,
    pub skeletron_y: f32,
    /// `FindFirstNPC(35)`'s `ai[3]` mirror (S1-E-14b): the Dark Caster's
    /// `aiStyle 8` arm reads the first active Skeletron head's red-hat
    /// latch (`NPC.cs:20820-20827`); false with no live head. B-1 audit
    /// fix: this is the RAW `ai[3] == 1` latch ONLY — the hands count no
    /// longer rides `ai[3]` (it lives in `skeletron_hands_alive`), so the
    /// last-hand phase can never spuriously activate red-hat mode.
    pub skeletron_red_hat: bool,
    /// B-1 (S3): the head's live-hands count — the C#'s LOCAL `num168`
    /// (`NPC.cs:21963-21972`, the expert scan of active type+1). A World
    /// mirror, NOT `head.ai[3]`: the C# head's `ai[3]` is the red-hat
    /// latch (`RedHatSkeletronAdjustmentsEnabled`, `NPC.cs:66673`).
    pub skeletron_hands_alive: f32,
    /// B-1 (H1): `Main.npc[hand.ai[1]].active && aiStyle == 11` — the
    /// head-present check the hand's death cascade reads
    /// (`NPC.cs:22303-22312`; the prime limb's `prime_present` pattern).
    pub skeletron_head_present: bool,
    /// B-1 (H2): the head's `ai[1]` state mirror — the hand's dock-return
    /// gate (`NPC.cs:22324-22328`: 3 → EncourageDespawn(10), != 0 → the
    /// spin-dock steering, 0 → the 300-tick attack timer).
    pub skeletron_head_state: f32,
    /// B-1 (H11): the head's RAW `ai[3]` slot — the hand's
    /// `localAI[3] = head.ai[3]` copy (`NPC.cs:22315`) mirrors the value,
    /// not a bool-ised predicate.
    pub skeletron_head_ai3: f32,
    /// `NPC.golemBoss`'s anchor (the body's centre, `NPC.cs:19507`) — the
    /// head/fists/free-head tethers read it (the WoF write-through
    /// shape).
    pub golem_present: bool,
    /// `WorldGen.spawnMeteor` (`WorldGen.cs:4208`): the pending meteorite
    /// landing — the night bookend's `Next(50)` after the boss-2 downing
    /// (`Main.cs:65230`) and the orb-boss deaths (`NPC.cs:79450`) raise it;
    /// the 16200-tick resolution lands the ore (`HandleMeteorFall`).
    pub spawn_meteor: bool,
    /// The body's `life / lifeMax` (`Main.npc[NPC.golemBoss].life` reads
    /// of the fists' chase/dash tiers, `:19274`/`:19370`, and the free
    /// head's power tiers, `:31524-31548`) — the WoF write-through shape.
    pub golem_hp_ratio: f32,
    pub golem_x: f32,
    pub golem_y: f32,
    /// `NPC.unlockedSlimeRainbowSpawn` (`WorldItem.GetShimmered`'s 4986 arm,
    /// `WorldItem.cs:1720-1735`) — a `Main`-global in the C#; a world field is
    /// the port's standing-in (ANALYSIS "Shimmer transforms").
    pub unlocked_slime_rainbow_spawn: bool,
    /// `NPC.unlockedSlimeRedSpawn` (`Player.cs:52429-52432` — the fished-out
    /// 682 raises it; `Projectile.cs:20819`'s enemy roll gates on it) — the
    /// same Main-global standing-in (S1-P-i35).
    pub unlocked_slime_red_spawn: bool,
    /// `NPC.combatBookWasUsed` (`WorldFile.cs`'s combatBookWasUsed flag;
    /// `FishingConditions.DidNotUseCombatBook` reads it, `Projectile.cs`'s
    /// `PrepareFishingContext` context) — the world-field standing-in
    /// (S1-P-i35).
    pub combat_book_was_used: bool,
    /// The body's `velocity.X` — the fists' tether anchor folds it in
    /// (`AI_047`'s `vector = Main.npc[golemBoss].Center + nPC.velocity +
    /// (0, -9 * scale)`, `NPC.cs:19258-19259`) — the WoF write-through
    /// shape. The head's tether (`aiStyle 46`, `:31148-31160`) does NOT
    /// carry the term.
    pub golem_vx: f32,
    /// The body's `velocity.Y` — the SAME anchor's Y term (S1-D-3: the
    /// v1 folded only X; `nPC.velocity` is the full vector,
    /// `NPC.cs:19258`).
    pub golem_vy: f32,
    /// The body's `life` / `lifeMax` — the free head's cadence ladders
    /// mix DOUBLE (`lifeMax * 0.8`, `:31420`) and INTEGER (`lifeMax / 2`,
    /// `:31477`) comparisons, so the ratio alone cannot carry them
    /// (S1-D-3). The sim's anchor pass writes both.
    pub golem_hp: i32,
    pub golem_max_hp: i32,
    /// The body's part scan (`AI_045`'s `flag`/`flag2`/`flag3`,
    /// `NPC.cs:19544-19566`): the attached head 246 / left fist 247 /
    /// right fist 248 alive — the charge ladder's +2-per-dead-part
    /// inputs (`:19646-19657`) and the shield's `dontTakeDamage = flag`
    /// (`:19567`). The sim's anchor pass scans the pool (S1-D-3).
    pub golem_head_alive: bool,
    pub golem_fist_left_alive: bool,
    pub golem_fist_right_alive: bool,
    /// `Main.slimeRain` — the Slime Rain event.
    pub slime_rain: bool,
    /// `Main.slimeRainTime` — the remaining event ticks (32400-54000 at
    /// the start, the dayRate-scaled decay).
    pub slime_rain_time: f64,
    /// `Main.slimeRainKillCount` — the kill counter (the negative half
    /// after the King spawns = the vanilla's hysteresis).
    pub slime_rain_kills: i32,
    /// `Main.slimeWarningTime` (`Main.cs:1051`) — the announcement
    /// countdown armed by `StartSlimeRain`/`StopSlimeRain`
    /// (`:64776`/`:64811`) and run down by `UpdateSlimeRainWarning`
    /// (`:65504-65522`); `slimeWarningDelay` is 420 (`:66802`).
    pub slime_warning_time: i32,
    /// `Main.pumpkinMoon` (`AmbienceServer.cs:50`). Runtime-only, not saved.
    pub pumpkin_moon: bool,
    /// `Main.snowMoon` (`AmbienceServer.cs:52`). Runtime-only, not saved.
    pub snow_moon: bool,
    /// `NPC.waveNumber` (`NPC.cs:6051`, no initializer — 0 until an event
    /// starts, 1 at the start, 20 terminal): the running moon event's
    /// current wave. Runtime-only, not saved.
    pub wave_number: i32,
    /// `NPC.waveKills` (`NPC.cs:6050`): the current wave's points — the
    /// progress walk adds the per-kind table value × the scalar and zeroes
    /// it when the wave advances. Runtime-only, not saved.
    pub wave_kills: f32,
    /// `NPC.totalInvasionPoints` (`NPC.cs:6049`): the run's total points.
    /// Write-only in the port — its C# readers are the out-of-scope
    /// broadcast (`Main.cs:10949`) and the MP sync; kept for state parity.
    /// Runtime-only, not saved.
    pub total_invasion_points: f32,
    /// `Main.hardMode` (`AmbienceServer.cs:61`). Runtime-only, not saved.
    pub hard_mode: bool,
    /// `WorldGen.altarCount` (`WorldGen.cs:4202`): the smashed altars drive
    /// the hardmode ore tiers.
    pub altars_smashed: i32,
    /// `WorldGen.shadowOrbSmashed` (`WorldGen.cs:4158`): the broken crimson/
    /// corruption orb arms the morning invasion rolls.
    pub shadow_orb_smashed: bool,
    /// `WorldGen.shadowOrbCount` (`WorldGen.cs:54293`): the shared
    /// orb/heart/mana-crystal smash counter — the third break in a world
    /// (a crimson heart = the Brain, a shadow orb = the Eater of Worlds)
    /// summons the boss (`:54294-54321`).
    pub shadow_orb_count: i32,
    /// `Main.invasionType` (`Main.cs:64455`): 1 = the Goblin Army.
    /// `DD2Event.Ongoing` (the Old One's Army) — the crystal summon
    /// raises it, the win/loss clears it.
    pub dd2_event_ongoing: bool,
    /// `NPC.waveNumber` — the Old One's Army's wave (1-3 tiers).
    pub dd2_wave: i32,
    /// `NPC.waveKills` — the current wave's kill count.
    pub dd2_wave_kills: f32,
    /// `DD2Event._crystalsDropping_toDrop`-family bookkeeping's port: the
    /// event's spawn hold (`_timeLeftUntilSpawningBegins`).
    pub dd2_spawn_hold: i32,
    /// `NPC.ShieldStrengthTower*` (`NPC.cs:6399-6405`): the four Lunar
    /// towers' shield strengths — raised at the spawn (`WorldGen.cs:86688`),
    /// spent by damage, zeroed at the reset (`NPC.cs:95247`).
    pub tower_shield_solar: i32,
    pub tower_shield_vortex: i32,
    pub tower_shield_nebula: i32,
    pub tower_shield_stardust: i32,
    /// `NPC.LunarShieldPowerNormal` (`NPC.cs:6407`) — the shields' max
    /// (100; `ShieldStrengthTowerMax` halves it post-Moon-Lord).
    pub lunar_shield_power_normal: i32,
    /// `NPC.downedMoonlord` — the final boss beaten (halves the shield max).
    pub downed_moon_lord: bool,
    /// `NPC.downedAncientCultist` (`NPC.cs:79393`): the cultist beaten.
    pub downed_ancient_cultist: bool,

    /// `NPC.LunarApocalypseIsUp` (`NPC.cs:6419`) — the towers' apocalypse
    /// phase (`WorldGen.cs:86687`'s write, `:86752`'s clear).
    pub lunar_apocalypse_is_up: bool,
    /// `NPC.MoonLordCountdown` / `NPC.MaxMoonLordCountdown`
    /// (`NPC.cs:6031-6033`) — the countdown to the Moon Lord's spawn
    /// (`StartImpendingDoom`'s writes, `Main.cs:64987`'s tick).
    pub moon_lord_countdown: i32,
    pub max_moon_lord_countdown: i32,
    /// `NPC.TowerActive*` (`NPC.cs:6411-6417`): the four towers alive.
    pub tower_active_solar: bool,
    pub tower_active_vortex: bool,
    pub tower_active_nebula: bool,
    pub tower_active_stardust: bool,
    /// `NPC.downedTower*` — the towers defeated.
    pub downed_tower_solar: bool,
    pub downed_tower_vortex: bool,
    pub downed_tower_nebula: bool,
    pub downed_tower_stardust: bool,
    /// `DD2Event.LostThisRun` — the crystal's death (`NPC.cs`'s case-548
    /// death-flag) sets it (`:79346`'s event-flag family); the weapon
    /// rosters clear with the event's stop.
    pub dd2_lost: bool,
    /// `DD2Event.LaneSpawnRate` (`:1813`) — the lane gate's spawn interval
    /// in frames (the wave roster's rate).
    pub dd2_lane_rate: i32,
    /// `NPC.AnyNPCs(548)` (`NPC.cs:41014`): whether the etherian crystal
    /// still stands — the lane gates' aiStyle-106 arm parks their fade while
    /// it does.
    pub dd2_crystal_alive: bool,
    /// `DD2Event._downedDarkMageT1` / `_downedOgreT2` (`:61-63`): the wave
    /// bosses' death latches — the tier tables' kill-worth rows set them and
    /// the wave-5/-7 kill caps read them (`:993`, `:1181`). The invasion's
    /// start clears them (`:268-269`); the C# never saves them (`Save`
    /// writes only the three tier flags, `:151-156`), so neither do we.
    pub dd2_latches: crate::dd2::DownedLatches,
    /// `DD2Event.WonThisRun` (`DD2Event.cs:57`) — `StartVictoryScene`
    /// (`:483`) latches it; `StartInvasion` clears it (`:272`). The win
    /// sentinel's `CheckProgress` refusal (`:364`) and the scene-end
    /// `StopInvasion(win: true)` read it.
    pub dd2_won: bool,
    /// `DD2Event.DownedInvasionT1/2/3` (`DD2Event.cs:49-53`) — the tier
    /// victory latches (`WinInvasionInternal`, `:316-327`), persisted by
    /// `DD2Event.Save` (`:151-156`). Distinct from `dd2_latches` (the
    /// per-run wave-boss deaths, never saved).
    pub dd2_downed_t1: bool,
    pub dd2_downed_t2: bool,
    pub dd2_downed_t3: bool,
    /// `DD2Event._crystalsDropping_lastWave/_toDrop/_alreadyDropped`
    /// (`DD2Event.cs:79-83`) — the per-kill energy-crystal budget
    /// (`ShouldDropCrystals`, `:815-912`); `StartInvasion` zeroes it
    /// (`:257-259`).
    pub dd2_crystal_drops: crate::dd2_victory::CrystalDropState,
    /// `DD2Event._spawnedBetsyT3` (`DD2Event.cs:65`) — the tier-3 wave-7
    /// Betsy summon latch (`SummonBetsy`, `:916`); `StartInvasion` clears
    /// it (`:270`).
    pub dd2_betsy_spawned: bool,
    pub invasion_type: i32,
    /// `Main.invasionSizeStart` (`Main.cs:64457`): the initial invasion
    /// points (`invasion_size` rides the Goblin Army block's field).
    pub invasion_size_start: i32,
    /// `Main.invasionX` (`Main.cs:64475`'s assign): the marching column
    /// (the tile X the army stands at).
    pub invasion_x: f64,
    /// `Main.invasionWarn` (`Main.cs:64422`'s 3600): the ticks between the
    /// approaching broadcasts.
    pub invasion_warn: i32,
    /// `Main.invasionDelay` (`UpdateTime_StartDay`'s decrement): the armed
    /// days before the invasion may start.
    pub invasion_delay: i32,
    /// The morning roll's `StartInvasion` request — the app resolves it
    /// wholesale (the players' life-max gate + the size/position assign).
    pub invasion_arm_pending: bool,
    /// `Main.wofDrawAreaTop`/`Bottom` (`Main.cs:757`/`:759`): the WoF's
    /// underworld draw band — INTs in the C#, written by the wall's
    /// aiStyle-27 arm (`npc_ai_style27::style27_step`, the ease + clamps +
    /// 160-px rule) and read by the eye/Hungry arms. Runtime-only.
    /// S1-D-3 W-1: converted f32 → i32 (the C# statics are `int`; the
    /// band midpoint `/2` is INTEGER division, `NPC.cs:25843`/`:26069`).
    pub wof_draw_top: i32,
    pub wof_draw_bottom: i32,
    /// `Main.wofNPCIndex` (`Main.cs:755`): the wall's slot, written every
    /// wall tick (`NPC.cs:25724`); -1 when no wall lives. `wof_present`
    /// is the port's living-wall mirror of `index >= 0`.
    pub wof_npc_index: i32,
    /// A WoF is alive (the Hungry deactivates without one) + its hp ratio
    /// and x (the Hungry's anchor reads).
    pub wof_present: bool,
    pub wof_hp_ratio: f32,
    pub wof_x: f32,
    /// The wall's raw life ints + box/velocity mirror (S1-D-3 W-1): the
    /// eye/Hungry ladders compare `(double)life < (double)lifeMax * k`
    /// (`NPC.cs:26061`/`:26225`/`:26238`), the Hungry anchor adds the
    /// wall's INTEGER half-width (`:26280`), the expert catch-up reads
    /// `velocity.X`/`Center.X` (`:26359`/`:26363`). Written per-NPC inside
    /// the update loop so later slots read the wall's fresh state, like
    /// the C#'s live `Main.npc[Main.wofNPCIndex]` reads.
    pub wof_life: i32,
    pub wof_life_max: i32,
    pub wof_width: i32,
    pub wof_vx: f32,
    pub wof_center_x: f32,
    /// The wall's `direction` — the eye's mirror copies it verbatim
    /// (`NPC.cs:26067-26068`: `direction = mainNpc.direction;
    /// spriteDirection = direction`). The `num391`-driven flip the port
    /// used to cite at `:26393` is aiStyle 29's HUNGRY, not the eye.
    pub wof_dir: i32,
    /// `NPC.savedAngler` (`NPC.cs:6293`): the Sleeping Angler has been
    /// talked into the Angler — the 376 spawn arms stop looking.
    pub saved_angler: bool,
    /// `NPC.savedWizard` (`NPC.cs:6289`) / `NPC.savedMech` (`NPC.cs:6291`):
    /// the bound Wizard/Mechanic rescue flags (`:52948`/`:52951`).
    pub saved_wizard: bool,
    /// `NPC.savedStylist` (`NPC.cs:6295`).
    pub saved_stylist: bool,
    /// `NPC.savedTaxCollector` (`NPC.cs:6285`).
    pub saved_tax_collector: bool,
    /// `NPC.savedGolfer` (`NPC.cs:6299`): the sleeping Golfer has been
    /// talked into the Golfer — the 589 spawn arm stops looking.
    pub saved_golfer: bool,
    /// `NPC.unlockedPrincessSpawn` (`Main.cs:65948`): the debug override
    /// that opens the Princess's arrival without the full roster.
    pub unlocked_princess: bool,
    /// `NPC.unlockedPartyGirlSpawn` (`Main.cs:65792`): the debug
    /// override that opens the Party Girl's arrival without the roster.
    pub unlocked_party_girl: bool,
    /// `Player.taxMoney`/`taxTimer` (`Player.cs:828-830`): the Tax
    /// Collector's purse + its `Player.taxRate` (3600) clock.
    pub tax_money: i32,
    pub tax_timer: i32,
    /// `NPC.taxCollector` (`NPC.cs:6083`): a live Tax Collector exists — the
    /// `Player.CollectTaxes`'s housed-town count (`Player.cs:24142-24149`):
    /// the sim sets it each pass.
    pub tax_payers: i32,
    /// tax pump runs while true (`Main.cs:64995`).
    pub tax_collector: bool,
    pub saved_mech: bool,
    /// `Main.anglerQuest` (`Main.cs:1785`): the INDEX into the 41-entry
    /// quest-fish pool (`Main.cs:67032`'s `anglerQuestItemNetIDs`).
    pub angler_quest: u8,
    /// `Main.anglerQuestFinished` (`Main.cs:1783`): today's quest is
    /// handed in (`NPCChatText_DoAnglerQuest`, `Main.cs:40251`); the
    /// day reset clears it (`AnglerQuestSwap`, `Main.cs:3761`).
    pub angler_quest_finished: bool,
    /// `Main.anglerWhoFinishedToday` (`Main.cs:1781`): the C#'s player
    /// name list — the port's single player collapses to a bool.
    pub angler_quest_turned_in: bool,
    /// `UpdateTime_StartDay`'s `Main.AnglerQuestSwap()` request
    /// (`Main.cs:65373`): the clock arms it (the clock has no RNG), the
    /// world pass resolves it on `Main.rand`'s app stream (the
    /// `invasion_arm_pending` shape).
    pub angler_swap_pending: bool,
    /// `Main.travelShop` (`Main.cs:1779`; the 40 slots of
    /// `Main.TravelShopMaxSlots`, allocated at `Main.cs:67030`): the
    /// Travelling Merchant's per-visit stock — `Chest.SetupTravelShop`
    /// (`Chest.cs:1275-1358`) regenerates it for every visit, the shop
    /// index 19 (`Chest.cs:2791-2810`) reads it back. A `0` slot is
    /// an empty one.
    pub travel_shop: [u16; 40],

    /// `NPC.butterflyChance` (`NPC.cs:6115`): the ambience Butterflies gate reads
    /// `< 6` (`AmbienceServer.cs:65`). Reset to 0 each world load/day
    /// (`NPC.cs:95172`); its update logic is NPC-spawn-owned (`NPC.cs:93378-93397`)
    /// and lands with M4 spawns, so the port keeps the reset value.
    pub butterfly_chance: i32,
    /// `WorldGen.SavedOreTiers` (`WorldGen.cs:3283-3297`): the seven metal
    /// tiers — the four pre-hardmode metals rolled once at the world setup
    /// (`Main.cs:11266-11297`) and the hardmode trio `-1` until an altar is
    /// smashed. The port's `ores.rs` owns the value; it rides the World so
    /// the save can carry it (M6-SAVE-1f — the vanilla writes it in the
    /// world-flags block, `WorldFile.cs:1315-1321` + `:1444-1450`).
    pub ore_tiers: crate::ores::SavedOreTiers,
    /// `Main.chest[]` as it exists at the end of generation — the vanilla
    /// stocks every chest's inventory **inline with its placement** inside
    /// `AddBuriedChest` (`WorldGen.cs:35871-37580`), and the records save
    /// with the world. S1-A-4 moved the port's stocking from the load-time
    /// re-roll to these gen-time rolls: a freshly generated world carries
    /// its own loot, a `.wld` load's records come from the file (the C#'s
    /// `LoadChests` — `Main.chest` is the file, nothing rolls), and the
    /// empty list simply means "no chest placed yet". The App's runtime
    /// store materialises from these at `enter_loaded_world`.
    pub chests: Vec<crate::save::ChestRecord>,

    // --- world identity / file metadata (S1-J-2, `IO/WorldFile.cs`) ---------
    /// `Main.ActiveWorldFileData.WorldGeneratorVersion` (`WorldFile.cs:1225`).
    pub generator_version: u64,
    /// `Main.ActiveWorldFileData.UniqueId` (`WorldFile.cs:1226`), 16 bytes.
    pub unique_id: [u8; 16],
    /// `Main.ActiveWorldFileData.WorldId` (`WorldFile.cs:1227`, footer `:1756`).
    pub world_id: i32,
    /// `Main.ActiveWorldFileData.LastPlayed` as `DateTime.ToBinary` ticks
    /// (`WorldFile.cs:1251` write, `:2048`/`:313` read). 0 = unset/min.
    pub last_played: i64,
    /// `Main.ActiveWorldFileData.CreationTime` as `DateTime.ToBinary` ticks
    /// (`WorldFile.cs:1250` write, `:2044` read; S1-J-3b). 0 = unset/min —
    /// the port stamps no wall clock; a loaded file's value round-trips.
    pub creation_time: i64,
}

impl World {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            tiles: vec![Tile::default(); (width as usize) * (height as usize)],
            variants: vec![0; (width as usize) * (height as usize)],
            frame_rng: crate::unified_random::UnifiedRandom::new(0),
            place_rng: crate::unified_random::UnifiedRandom::new(0),
            tile_object_random_cache: None,
            // `Main.AnnouncementBoxDisabled`'s stock is false (the launch
            // switch off) and `Main.AnnouncementBoxRange`'s stock is -1
            // (`Main.cs:66530`).
            announcement_box_disabled: false,
            announcement_box_range: -1,
            wires: crate::wiring::WireState::default(),
            tile_entities: crate::tile_entities::TileEntities::default(),
            signs: Vec::new(),
            pressure_plates: Vec::new(),
            frame_important: Vec::new(),
            ore_tiers: crate::ores::SavedOreTiers::new(),
            chests: Vec::new(),
            spawn_y: height / 4,
            bed_spawn: None,
            spawn_x: width / 2,
            // `WorldGen.Reset` (`WorldGen.cs:7047-7048`): both 0 until the
            // dungeon pass publishes the anchor.
            dungeon_x: 0,
            dungeon_y: 0,
            world_surface: height as f64 * 0.25,
            rock_layer: height as f64 * 0.45,
            time: 0.0,
            day_time: true,
            moon_type: 0,
            moon_phase: 0,
            downed_slime_king: false,
            downed_queen_slime: false,
            downed_eoc: false,
            downed_eow: false,
            downed_skeletron: false,
            downed_queen_bee: false,
            downed_mech_boss1: false,
            downed_mech_boss3: false,
            downed_mech_boss2: false,
            downed_plant_boss: false,
            invasion_goblins: false,
            invasion_size: 0,

            saved_goblin: false,
            brain_present: false,
            brain_x: 0.0,
            brain_y: 0.0,
            downed_golem: false,
            downed_fishron: false,
            downed_deerclops: false,
            golem_present: false,
            golem_hp_ratio: 1.0,
            spawn_meteor: false,
            golem_x: 0.0,
            skeletron_x: 0.0,
            prime_present: false,
            prime_x: 0.0,
            prime_y: 0.0,
            prime_head_x: 0.0,
            prime_head_width: 0,
            prime_head_vx: 0.0,
            prime_head_vy: 0.0,
            prime_head_ai1: 0.0,
            skeletron_y: 0.0,
            skeletron_red_hat: false,
            skeletron_hands_alive: 0.0,
            skeletron_head_present: false,
            skeletron_head_state: 0.0,
            skeletron_head_ai3: 0.0,
            golem_y: 0.0,
            golem_vy: 0.0,
            golem_hp: 0,
            golem_max_hp: 0,
            golem_head_alive: false,
            golem_fist_left_alive: false,
            golem_fist_right_alive: false,
            unlocked_slime_rainbow_spawn: false,
            unlocked_slime_red_spawn: false,
            combat_book_was_used: false,
            golem_vx: 0.0,
            slime_rain: false,
            slime_rain_time: 0.0,
            slime_rain_kills: 0,
            slime_warning_time: 0,
            sundial_cooldown: 0,
            moondial_cooldown: 0,
            fast_forward_to_dawn: false,
            fast_forward_to_dusk: false,
            hell_back_style: 0,
            corrupt_bg_style: 0,
            jungle_bg_style: 0,
            hallow_bg_style: 0,
            crimson_bg_style: 0,
            mushroom_bg_style: 0,
            column_surface: Vec::new(),
            chunks_x: 0,
            chunks_y: 0,
            chunk_has: Vec::new(),
            seed: 0,
            seeds: crate::seeds::SeedFlags::default(),
            micro_seeds: crate::seeds::MicroSeedFlags::default(),
            revision: 0,
            extras: std::collections::HashMap::new(),
            eclipse: false,
            blood_moon: false,
            raining: false,
            rain_time: 0,
            max_raining: 0.0,
            sandstorm_happening: false,
            sandstorm_time_left: 0,
            sandstorm_severity: 0.0,
            sandstorm_intended_severity: 0.0,
            all_players_asleep: false,
            pumpkin_moon: false,
            snow_moon: false,
            wave_number: 0,
            wave_kills: 0.0,
            total_invasion_points: 0.0,
            hard_mode: false,

            altars_smashed: 0,
            shadow_orb_smashed: false,
            shadow_orb_count: 0,
            downed_goblins: false,
            downed_frost: false,
            downed_pirates: false,
            downed_martians: false,
            dd2_event_ongoing: false,
            dd2_wave: 1,
            dd2_wave_kills: 0.0,
            dd2_spawn_hold: 300,
            dd2_lost: false,
            dd2_lane_rate: crate::dd2::LANE_SPAWN_RATE,
            dd2_crystal_alive: false,
            dd2_latches: crate::dd2::DownedLatches::default(),
            dd2_won: false,
            dd2_downed_t1: false,
            dd2_downed_t2: false,
            dd2_downed_t3: false,
            dd2_crystal_drops: crate::dd2_victory::CrystalDropState::default(),
            dd2_betsy_spawned: false,
            tower_shield_solar: 0,
            tower_shield_vortex: 0,
            tower_shield_nebula: 0,
            tower_shield_stardust: 0,
            lunar_shield_power_normal: 100,
            downed_moon_lord: false,
            downed_ancient_cultist: false,
            lunar_apocalypse_is_up: false,
            moon_lord_countdown: 0,
            max_moon_lord_countdown: 3600,
            tower_active_solar: false,
            tower_active_vortex: false,
            tower_active_nebula: false,
            tower_active_stardust: false,
            downed_tower_solar: false,
            downed_tower_vortex: false,
            downed_tower_nebula: false,
            downed_tower_stardust: false,
            invasion_type: 0,
            invasion_size_start: 0,
            invasion_x: 0.0,
            invasion_warn: 0,
            invasion_delay: 0,
            invasion_arm_pending: false,
            wof_draw_top: -1,
            wof_draw_bottom: -1,
            wof_npc_index: -1,
            wof_present: false,
            wof_hp_ratio: 1.0,
            wof_x: 0.0,
            wof_life: 0,
            wof_life_max: 0,
            wof_width: 0,
            wof_vx: 0.0,
            wof_center_x: 0.0,
    wof_dir: 1,
            saved_angler: false,
            saved_wizard: false,
            saved_stylist: false,
            saved_tax_collector: false,
            saved_golfer: false,
            unlocked_princess: false,
            unlocked_party_girl: false,
            tax_money: 0,
            tax_timer: 0,
            tax_collector: false,
            tax_payers: 0,
            saved_mech: false,
            angler_quest: 0,
            angler_quest_finished: false,
            angler_quest_turned_in: false,
            angler_swap_pending: false,
            travel_shop: [0; 40],
            butterfly_chance: 0,
            generator_version: 0,
            unique_id: [0u8; 16],
            world_id: 0,
            last_played: 0,
            creation_time: 0,
        }
    }

    /// `Main.IsFastForwardingTime` (`Main.cs:6160-6167`): a dial fast-forward is
    /// armed.
    pub fn is_fast_forwarding(&self) -> bool {
        // C# shape: `if (!fastForwardTimeToDawn) return fastForwardTimeToDusk; return true;`
        self.fast_forward_to_dawn || self.fast_forward_to_dusk
    }

    /// `Main.UpdateTimeRate` (`Main.cs:6199-6224`): 60 while a dial is
    /// fast-forwarding (`:6201-6205`), **×5 while every player is fully
    /// asleep** (`:6209-6215` — the `SleepingPlayersCount ==
    /// ActivePlayersCount && > 0` flag), 1 otherwise. The creative-power
    /// branches (`:6216-6218` freeze → 0) stay with the creative powers.
    pub fn day_rate(&self) -> i32 {
        // `Main.cs:6200-6203`: `if (IsFastForwardingTime()) { dayRate = 60; return; }`
        if self.is_fast_forwarding() {
            return 60;
        }
        // `:6212-6215`: `if (!Main.gameMenu && flag) num *= 5` — the SP port
        // has no gameMenu state here; the flag is the all-asleep input.
        if self.all_players_asleep {
            return 5;
        }
        // `dayRate = 1` is the ported fallback (`:6221-6224` gameMenu arm).
        1
    }

    /// `NPC.MoonEventRequiredPointsPerWaveLookup` (`NPC.cs:95289-95300`):
    /// the per-wave `waveKills` requirement, indexed by `waveNumber`. The
    /// index-0 and index-20 entries are 0 — the terminal waves carry no
    /// requirement and never advance.
    pub const MOON_WAVE_REQUIREMENTS: [i32; 21] = [
        0, 25, 40, 50, 80, 100, 160, 180, 200, 250, 300, 375, 450, 525, 675, 850, 1025, 1325,
        1550, 2000, 0,
    ];

    /// `Main.startPumpkinMoon` (`Main.cs:10979-10996`): the flag, the twin's
    /// clear, the blood moon's clear, then the single-player arm — the points
    /// reset and `waveNumber = 1`. The wave-1 broadcast and the damage
    /// tracker are UI/MP (out). Draws nothing.
    pub fn start_pumpkin_moon(&mut self) {
        self.pumpkin_moon = true;
        self.snow_moon = false;
        self.blood_moon = false;
        self.total_invasion_points = 0.0;
        self.wave_kills = 0.0;
        self.wave_number = 1;
    }

    /// `Main.startSnowMoon` (`Main.cs:10998-11017`): the snow twin — same
    /// shape, the flags mirrored.
    pub fn start_snow_moon(&mut self) {
        self.snow_moon = true;
        self.pumpkin_moon = false;
        self.blood_moon = false;
        self.total_invasion_points = 0.0;
        self.wave_kills = 0.0;
        self.wave_number = 1;
    }

    /// `Main.stopMoonEvent` (`Main.cs:10953-10976`): each flag's clear is
    /// unconditional and carries the `netMode != 1` point/wave resets — the
    /// port is single-player, so both arms run; with no event running the
    /// whole call is a no-op (the C# guards on the flag too).
    pub fn stop_moon_event(&mut self) {
        if self.pumpkin_moon || self.snow_moon {
            self.pumpkin_moon = false;
            self.snow_moon = false;
            self.total_invasion_points = 0.0;
            self.wave_kills = 0.0;
            self.wave_number = 0;
        }
    }

    /// `NPC.GetMoonEventPointScalar` (`NPC.cs:78432-78440`): master 2.5 /
    /// expert 2 / normal 1. The port is single-difficulty, so the normal
    /// arm's 1x is the port's reduction (recorded in ANALYSIS).
    pub const MOON_POINT_SCALAR: f32 = 1.0;

    /// `CheckProgressFrostMoon`'s per-type table (`NPC.cs:78518-78549`) —
    /// the Snow/Frost Moon family's kill points, 0 for every other type.
    fn frost_moon_points(npc_type: u16) -> f32 {
        match npc_type {
            338 | 339 | 340 => 1.0,
            341 => 20.0,
            342 => 2.0,
            343 => 18.0,
            344 => 50.0,
            345 => 150.0,
            346 => 100.0,
            347 => 8.0,
            348 | 349 => 4.0,
            350 => 3.0,
            351 => 10.0,
            352 => 5.0,
            _ => 0.0,
        }
    }

    /// `CheckProgressPumpkinMoon`'s per-type table (`NPC.cs:78658-78682`) —
    /// the Pumpkin Moon family's kill points, 0 for every other type.
    fn pumpkin_moon_points(npc_type: u16) -> f32 {
        match npc_type {
            305..=314 => 1.0,
            315 => 50.0,
            325 => 75.0,
            326 => 2.0,
            327 => 150.0,
            329 => 5.0,
            330 => 10.0,
            _ => 0.0,
        }
    }

    /// The shared walk body — `CheckProgressFrostMoon`/`CheckProgressPumpkinMoon`
    /// (`NPC.cs:78472-78586`, `:78600-78719`): the requirement read at the
    /// kill's wave, the points added, the advance when the requirement is met
    /// and non-zero. The wave-text broadcast and the
    /// `ReportInvasionProgress` bar are UI/MP — out. Draws nothing.
    fn moon_event_kill_walk(&mut self, npc_type: u16, points: fn(u16) -> f32) {
        // `MoonEventRequiredPointsPerWaveLookup[NPC.waveNumber]` — the C# index
        // runs 0..20 (the start's 1 and the stop's 0 are both valid, the table
        // has 21 rows); an out-of-range wave reads the terminal 0 rather than
        // panicking.
        let requirement = Self::MOON_WAVE_REQUIREMENTS
            .get(self.wave_number as usize)
            .copied()
            .unwrap_or(0);
        let gained = points(npc_type) * Self::MOON_POINT_SCALAR;
        self.wave_kills += gained;
        self.total_invasion_points += gained;
        if self.wave_kills >= requirement as f32 && requirement != 0 {
            // `NPC.waveKills = 0f; NPC.waveNumber++;` — the C# recomputes the
            // requirement right after for the broadcast (out of scope).
            self.wave_kills = 0.0;
            self.wave_number += 1;
        }
    }

    /// `NPC.CheckProgressFrostMoon` (`NPC.cs:78445-78586`) on the kill of
    /// `npc_type` — self-gates on `Main.snowMoon`, so a kill under any other
    /// state (or another event's family) scores nothing.
    pub fn report_frost_moon_kill(&mut self, npc_type: u16) {
        if !self.snow_moon {
            return;
        }
        self.moon_event_kill_walk(npc_type, Self::frost_moon_points);
    }

    /// `NPC.CheckProgressPumpkinMoon` (`NPC.cs:78588-78719`) on the kill of
    /// `npc_type` — self-gates on `Main.pumpkinMoon`.
    pub fn report_pumpkin_moon_kill(&mut self, npc_type: u16) {
        if !self.pumpkin_moon {
            return;
        }
        self.moon_event_kill_walk(npc_type, Self::pumpkin_moon_points);
    }

    /// `UpdateTime_StartDay`'s invasion roll (`Main.cs:65439-65462`): the
    /// smashed orb arms the goblin rolls — 1/3 before the first defeat,
    /// 1/30 (hardmode) / 1/30-1/60 (the classic) after. `rng01` is the
    /// deterministic day-roll (the caller's stream).
    pub fn roll_morning_invasion(&mut self, rng01: f64) {
        if !self.shadow_orb_smashed {
            return;
        }
        if !self.downed_goblins {
            if rng01 < 1.0 / 3.0 {
                self.invasion_delay = 0;
                self.invasion_arm_pending = true;
            }
        } else if rng01
            < if self.hard_mode {
                1.0 / 60.0
            } else {
                1.0 / 30.0
            }
        {
            self.invasion_delay = 0;
            self.invasion_arm_pending = true;
        }
    }

    /// `UpdateTime_StartNight`'s blood-moon roll
    /// (`Main.cs:65333-65352`): 1-in-9 (1-in-6 on the tenth-anniversary
    /// world) at every dusk, gated on no eye spawn and a moon phase other
    /// than the new moon, and only when a player with more than 120 max
    /// life exists. On a hit the flag raises and both dial cooldowns
    /// force-reset (`:65344-65346`).
    pub fn roll_blood_moon(
        &mut self,
        rng01: f64,
        tenth_anniversary: bool,
        any_player_life_over_120: bool,
        spawn_eye: bool,
    ) -> bool {
        // `:65335-65341` — the roll gate.
        if spawn_eye || self.moon_phase == 4 {
            return false;
        }
        if rng01 >= 1.0 / f64::from(if tenth_anniversary { 6 } else { 9 }) {
            return false;
        }
        // `:65342-65349` — the 120-HP player gate.
        if !any_player_life_over_120 {
            return false;
        }
        self.start_blood_moon();
        true
    }

    /// The blood-moon hit body (`Main.cs:65344-65346`), the roll factored out:
    /// the flag up and both dial cooldowns force-reset ("instantly reusable
    /// after certain celestial events"). `roll_blood_moon`'s success path and
    /// the F3 panel's start row share it (S1-E-37).
    pub fn start_blood_moon(&mut self) {
        self.blood_moon = true;
        self.sundial_cooldown = 0;
        self.moondial_cooldown = 0;
    }

    /// `UpdateTime_StartDay`'s solar-eclipse roll (`Main.cs:65421-65430`):
    /// 1-in-20 at every dawn, hardmode with any mech boss down, then the
    /// same dial-cooldown force-reset (`:65423-65426`).
    pub fn roll_eclipse(
        &mut self,
        rng01: f64,
        hard_mode: bool,
        downed_mech_boss_any: bool,
    ) -> bool {
        // `:65421-65422` — the roll gate.
        if !(hard_mode && downed_mech_boss_any) {
            return false;
        }
        if rng01 >= 1.0 / 20.0 {
            return false;
        }
        self.eclipse = true;
        self.sundial_cooldown = 0;
        self.moondial_cooldown = 0;
        true
    }

    /// `NPC.downedMechBossAny` — any of the three mech bosses down.
    pub fn downed_mech_boss_any(&self) -> bool {
        self.downed_mech_boss1 || self.downed_mech_boss2 || self.downed_mech_boss3
    }

    /// `Main.Sundialing` (`Main.cs:6169-6181`), single-player branch: arm the
    /// fast-forward to dawn and start the 8-day cooldown. The `netMode == 1`
    /// message branch is not ported.
    pub fn sundialing(&mut self) {
        if self.sundial_cooldown == 0 {
            // `Main.cs:6177-6179`: `fastForwardTimeToDawn = true;
            // sundialCooldown = 8;` (the SP arm; `SendData(7)` is net-only).
            self.fast_forward_to_dawn = true;
            self.sundial_cooldown = 8;
        }
    }

    /// `Main.Moondialing` (`Main.cs:6184-6196`), single-player branch: arm the
    /// fast-forward to dusk and start the 8-day cooldown. The `netMode == 1`
    /// message branch is not ported.
    pub fn moondialing(&mut self) {
        if self.moondial_cooldown == 0 {
            // `Main.cs:6192-6194`: `fastForwardTimeToDusk = true;
            // moondialCooldown = 8;` (the SP arm).
            self.fast_forward_to_dusk = true;
            self.moondial_cooldown = 8;
        }
    }

    /// Finds the first solid tile per column, used to place the sky horizon and the
    /// underground background. Trees and plants are not solid, so they do not count.
    pub fn compute_column_surface(&mut self) {
        use crate::tiledata as td;
        self.column_surface.resize(self.width as usize, self.height);
        for x in 0..self.width {
            let mut top = self.height;
            for y in 0..self.height {
                let t = &self.tiles[self.index(x, y)];
                // `WorldGen.cs:15715`: leaf blocks (192) go non-solid after the
                // Living Trees pass — they never cut the column surface.
                if t.active()
                    && (t.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(t.kind)
                    && t.kind != 192
                {
                    top = y;
                    break;
                }
            }
            self.column_surface[x as usize] = top;
        }
    }

    /// Recomputes chunk occupancy. Must be called after loading/framing and after any
    /// bulk edit; per-tile edits can call [`World::mark_chunk`].
    pub fn compute_chunks(&mut self) {
        self.chunks_x = (self.width + CHUNK - 1) / CHUNK;
        self.chunks_y = (self.height + CHUNK - 1) / CHUNK;
        self.chunk_has.clear();
        self.chunk_has
            .resize((self.chunks_x as usize) * (self.chunks_y as usize), false);
        let w = self.width as usize;
        for y in 0..self.height {
            let cy = (y / CHUNK) as usize;
            let row = y as usize * w;
            for x in 0..self.width {
                let t = &self.tiles[row + x as usize];
                if t.active() || t.wall != 0 || t.liquid != 0 {
                    self.chunk_has[cy * self.chunks_x as usize + (x / CHUNK) as usize] = true;
                }
            }
        }
    }

    /// Recomputes the first solid tile of one column after a block edit.
    pub fn recompute_column_surface(&mut self, x: i32) {
        use crate::tiledata as td;
        if x < 0 || x >= self.width {
            return;
        }
        let mut top = self.height;
        for y in 0..self.height {
            let t = &self.tiles[self.index(x, y)];
            if t.active()
                && (t.kind as usize) < td::TILE_COUNT
                && td::TILE_SOLID.has(t.kind)
                && t.kind != 192
            {
                top = y;
                break;
            }
        }
        if let Some(slot) = self.column_surface.get_mut(x as usize) {
            *slot = top;
        }
    }

    /// Marks the chunk owning `(x, y)` as occupied (after placing a tile/wall).
    #[inline]
    pub fn mark_chunk(&mut self, x: i32, y: i32) {
        if self.chunks_x == 0 || self.chunks_y == 0 || !self.in_bounds(x, y) {
            return;
        }
        let cx = (x / CHUNK) as usize;
        let cy = (y / CHUNK) as usize;
        self.chunk_has[cy * self.chunks_x as usize + cx] = true;
    }

    /// True when the chunk contains nothing at all (fully empty air).
    #[inline]
    pub fn chunk_has(&self, cx: i32, cy: i32) -> bool {
        if cx < 0 || cy < 0 || cx >= self.chunks_x || cy >= self.chunks_y {
            return false;
        }
        self.chunk_has[(cy as usize) * (self.chunks_x as usize) + cx as usize]
    }

    /// Surface row of a column, falling back to the map bottom outside the world.
    #[inline]
    pub fn surface_of(&self, x: i32) -> i32 {
        if x < 0 || x >= self.width {
            return self.height;
        }
        self.column_surface
            .get(x as usize)
            .copied()
            .unwrap_or(self.height)
    }

    #[inline]
    pub fn index(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Raw extras byte for a tile index (0 when absent).
    #[inline]
    pub fn extras_at(&self, idx: usize) -> u8 {
        self.extras.get(&(idx as u32)).copied().unwrap_or(0)
    }

    /// Seeds the framing reset-roll stream from the world seed (the port's
    /// `genRand` stand-in — vanilla seeds `Main.rand` from the clock on load,
    /// `Main.cs:6461-6463`, so its runtime rolls are not reproducible; the
    /// wiring engine carries the same world-seeded shape). Call once per
    /// generated or loaded world, before any edit can reframe.
    pub fn seed_frame_rng(&mut self) {
        self.frame_rng = crate::unified_random::UnifiedRandom::new(self.seed as i32);
        // S1-F-7b: the placement random-style stream (`TileObject.cs:787`/
        // :819) seeds from the same world seed — a distinct `UnifiedRandom`
        // instance, so its draws never interleave with the framing rolls.
        self.place_rng = crate::unified_random::UnifiedRandom::new(self.seed as i32);
    }

    /// `Tile.frameNumber()` (`Tile.cs:464-470`, `bTileHeader2` bits 4-5).
    #[inline]
    pub fn frame_number(&self, idx: usize) -> u8 {
        self.variants.get(idx).copied().unwrap_or(0) & 0x3
    }

    /// `Tile.frameNumber(byte)` (`Tile.cs:468-470`).
    #[inline]
    pub fn set_frame_number(&mut self, idx: usize, n: u8) {
        let slot = self
            .variants
            .get_mut(idx)
            .expect("variants sized with the tile grid");
        *slot = (*slot & !0x3) | (n & 0x3);
    }

    /// `Tile.wallFrameNumber()` (`Tile.cs:472-478`, `bTileHeader2` bits 6-7).
    #[inline]
    pub fn wall_frame_number(&self, idx: usize) -> u8 {
        (self.variants.get(idx).copied().unwrap_or(0) >> 2) & 0x3
    }

    /// `Tile.wallFrameNumber(byte)` (`Tile.cs:476-478`).
    #[inline]
    pub fn set_wall_frame_number(&mut self, idx: usize, n: u8) {
        let slot = self
            .variants
            .get_mut(idx)
            .expect("variants sized with the tile grid");
        *slot = (*slot & !0xC) | ((n & 0x3) << 2);
    }

    #[inline]
    pub fn set_extras_at(&mut self, idx: usize, byte: u8) {
        if byte == 0 {
            self.extras.remove(&(idx as u32));
        } else {
            self.extras.insert(idx as u32, byte);
        }
    }

    /// `Tile.wallColor()` (`Tile.cs:335-343`, `bTileHeader & 0x1F`).
    #[inline]
    pub fn wall_paint(&self, x: i32, y: i32) -> u8 {
        self.extras_at(self.index(x, y)) & 0x1f
    }

    #[inline]
    pub fn set_wall_paint(&mut self, x: i32, y: i32, paint: u8) {
        let idx = self.index(x, y);
        let b = self.extras_at(idx);
        self.set_extras_at(idx, (b & !0x1f) | (paint & 0x1f));
    }

    /// `Tile.invisibleBlock()` (`Tile.cs:528-531`, Echo coating).
    #[inline]
    pub fn invisible_block(&self, x: i32, y: i32) -> bool {
        self.extras_at(self.index(x, y)) & 0x20 != 0
    }

    #[inline]
    pub fn set_invisible_block(&mut self, x: i32, y: i32, on: bool) {
        let idx = self.index(x, y);
        let b = self.extras_at(idx);
        self.set_extras_at(idx, if on { b | 0x20 } else { b & !0x20 });
    }

    /// `Tile.invisibleWall()` (`Tile.cs:545-548`, Echo coating).
    #[inline]
    pub fn invisible_wall(&self, x: i32, y: i32) -> bool {
        self.extras_at(self.index(x, y)) & 0x40 != 0
    }

    #[inline]
    pub fn set_invisible_wall(&mut self, x: i32, y: i32, on: bool) {
        let idx = self.index(x, y);
        let b = self.extras_at(idx);
        self.set_extras_at(idx, if on { b | 0x40 } else { b & !0x40 });
    }

    /// `Tile.fullbrightWall()` (`Tile.cs:718-721`, Glow coating).
    #[inline]
    pub fn fullbright_wall(&self, x: i32, y: i32) -> bool {
        self.extras_at(self.index(x, y)) & 0x80 != 0
    }

    #[inline]
    pub fn set_fullbright_wall(&mut self, x: i32, y: i32, on: bool) {
        let idx = self.index(x, y);
        let b = self.extras_at(idx);
        self.set_extras_at(idx, if on { b | 0x80 } else { b & !0x80 });
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.in_bounds(x, y) {
            Some(&self.tiles[self.index(x, y)])
        } else {
            None
        }
    }

    #[inline]
    /// `tile.active()` at a coordinate (false out of bounds).
    pub fn tile_active(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).is_some_and(|t| t.active())
    }

    /// `tile.frameX` at a coordinate (0 out of bounds).
    pub fn tile_frame_x(&self, x: i32, y: i32) -> i32 {
        self.tile(x, y).map(|t| t.frame_x as i32).unwrap_or(0)
    }

    /// `tile.frameY` at a coordinate (0 out of bounds).
    pub fn tile_frame_y(&self, x: i32, y: i32) -> i32 {
        self.tile(x, y).map(|t| t.frame_y as i32).unwrap_or(0)
    }

    pub fn set_tile_frame_x(&mut self, x: i32, y: i32, v: i32) {
        if let Some(t) = self.tile_mut(x, y) {
            t.frame_x = v as i16;
        }
    }

    pub fn set_tile_frame_y(&mut self, x: i32, y: i32, v: i32) {
        if let Some(t) = self.tile_mut(x, y) {
            t.frame_y = v as i16;
        }
    }

    /// Swaps the tile's kind with `other` when the current kind is `expected`
    /// (the 421/422 wire-lamp pair, `Wiring.cs:1013-1026`).
    pub fn swap_tile_kind(&mut self, x: i32, y: i32, other: u16) {
        if let Some(t) = self.tile_mut(x, y) {
            if t.kind != other {
                t.kind = other;
            }
        }
    }

    pub fn tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if self.in_bounds(x, y) {
            let i = self.index(x, y);
            self.revision = self.revision.wrapping_add(1);
            Some(&mut self.tiles[i])
        } else {
            None
        }
    }

    #[inline]
    pub fn is_active(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).is_some_and(|t| t.active())
    }

    pub fn memory_bytes(&self) -> usize {
        self.tiles.len() * std::mem::size_of::<Tile>()
    }
}

/// `WorldGen.setBG` cases 1/2/4/5/8 (`WorldGen.cs:7235-7274`, `7276-7316`,
/// `7424-7459`, `7460-7500`, `7578-7608`): the three `Background_<id>` texture
/// ids for a surface style, `-1` meaning "this layer is not drawn".
///
/// `bg` is the `setBG` slot: 1 corrupt, 2 jungle, 4 hallow, 5 crimson,
/// 8 mushroom. The other slots (forest tree sets, snow, desert, ocean,
/// underworld) have their own tables and are not covered here.
pub fn surface_bg_texture_ids(bg: i32, style: i32) -> Option<[i32; 3]> {
    Some(match bg {
        1 => match style {
            1 => [56, 57, 58],
            2 => [211, 212, 213],
            3 => [225, 226, 227],
            4 => [240, 241, 242],
            51 => [324, 323, 322],
            52 => [324, 226, 322],
            _ => [12, 13, 14],
        },
        2 => match style {
            1 => [59, 60, 61],
            2 => [222, 223, 224],
            3 => [237, 238, 239],
            4 => [284, 285, 286],
            5 => [271, 272, 273],
            6 => [302, 301, 300],
            _ => [15, 16, 17],
        },
        4 => match style {
            1 => [102, 103, 104],
            2 => [219, 220, 221],
            3 => [243, 244, 245],
            4 => [-1, 261, 262],
            5 => [327, 326, 325],
            _ => [29, 30, 31],
        },
        5 => match style {
            1 => [105, 106, 107],
            2 => [174, -1, 175],
            3 => [214, 215, 216],
            4 => [-1, 229, 230],
            5 => [255, 256, 257],
            6 => [339, 338, 337],
            _ => [43, 44, 45],
        },
        8 => match style {
            1 => [231, 232, 233],
            2 => [234, 235, 236],
            3 => [287, 288, 289],
            4 => [321, 320, 319],
            _ => [46, 47, 48],
        },
        _ => return None,
    })
}

impl World {
    /// `WorldGen.setBG(bg, style)` for the five surface variants the port owns
    /// (`WorldGen.cs:7235-7274`, `7276-7316`, `7424-7459`, `7460-7500`,
    /// `7578-7608`). Unknown slots are ignored — their tables are unported.
    pub fn set_surface_bg_style(&mut self, bg: i32, style: i32) {
        let style = style as u8;
        match bg {
            1 => self.corrupt_bg_style = style,
            2 => self.jungle_bg_style = style,
            4 => self.hallow_bg_style = style,
            5 => self.crimson_bg_style = style,
            8 => self.mushroom_bg_style = style,
            _ => {}
        }
    }

    /// The three `Background_<id>` textures of the corrupt surface style
    /// (`WorldGen.setBG` case 1).
    pub fn corrupt_bg(&self) -> [i32; 3] {
        surface_bg_texture_ids(1, self.corrupt_bg_style as i32).unwrap()
    }
    pub fn jungle_bg(&self) -> [i32; 3] {
        surface_bg_texture_ids(2, self.jungle_bg_style as i32).unwrap()
    }
    pub fn hallow_bg(&self) -> [i32; 3] {
        surface_bg_texture_ids(4, self.hallow_bg_style as i32).unwrap()
    }
    pub fn crimson_bg(&self) -> [i32; 3] {
        surface_bg_texture_ids(5, self.crimson_bg_style as i32).unwrap()
    }
    pub fn mushroom_bg(&self) -> [i32; 3] {
        surface_bg_texture_ids(8, self.mushroom_bg_style as i32).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn surface_bg_style_tables_match_set_bg() {
        // Spot rows from `WorldGen.setBG`, including the `default:` rows and
        // the `-1` layers that `DrawSurfaceBG_*` skips.
        assert_eq!(surface_bg_texture_ids(1, 1), Some([56, 57, 58]));
        assert_eq!(surface_bg_texture_ids(1, 51), Some([324, 323, 322]));
        assert_eq!(surface_bg_texture_ids(1, 0), Some([12, 13, 14]));
        assert_eq!(surface_bg_texture_ids(2, 6), Some([302, 301, 300]));
        assert_eq!(surface_bg_texture_ids(4, 4), Some([-1, 261, 262]));
        assert_eq!(surface_bg_texture_ids(5, 2), Some([174, -1, 175]));
        assert_eq!(surface_bg_texture_ids(5, 6), Some([339, 338, 337]));
        assert_eq!(surface_bg_texture_ids(8, 4), Some([321, 320, 319]));
        // Slots with their own tables (snow/desert/ocean/forest/underworld).
        assert_eq!(surface_bg_texture_ids(3, 0), None);
        assert_eq!(surface_bg_texture_ids(9, 0), None);

        let mut w = World::new("t".into(), 40, 40);
        w.set_surface_bg_style(5, 6);
        assert_eq!(w.crimson_bg(), [339, 338, 337]);
        assert_eq!(w.crimson_bg_style, 6);
        w.set_surface_bg_style(3, 1); // unowned slot: unchanged
        assert_eq!(w.crimson_bg_style, 6);
    }

    #[test]
    fn the_blood_moon_roll_matches_the_dusk_gate() {
        // `Main.cs:65333-65352`: 1-in-9 (1-in-6 on the tenth-anniversary
        // world), the new moon and the eye spawn skip it, the 120-HP player
        // gate, both dial cooldowns force-reset.
        let mut w = World::new("t".into(), 40, 40);
        w.sundial_cooldown = 3;
        w.moondial_cooldown = 5;
        assert!(w.roll_blood_moon(1.0 / 18.0, false, true, false));
        assert!(w.blood_moon);
        assert_eq!((w.sundial_cooldown, w.moondial_cooldown), (0, 0));
        // Outside the window nothing happens.
        let mut w = World::new("t".into(), 40, 40);
        assert!(!w.roll_blood_moon(0.5, false, true, false));
        assert!(!w.blood_moon);
        // The phase-4 moon and the eye spawn gate it off.
        let mut w = World::new("t".into(), 40, 40);
        w.moon_phase = 4;
        assert!(!w.roll_blood_moon(0.0, false, true, false));
        let mut w = World::new("t".into(), 40, 40);
        assert!(!w.roll_blood_moon(0.0, false, true, true));
        // Without a 120+ HP player it never raises.
        let mut w = World::new("t".into(), 40, 40);
        assert!(!w.roll_blood_moon(0.0, false, false, false));
        // The anniversary world's 1-in-6 window is wider than the normal
        // 1-in-9, so 1/9 fits it and 1/12 fits too.
        let mut w = World::new("t".into(), 40, 40);
        assert!(w.roll_blood_moon(1.0 / 9.0, true, true, false));
        assert!(w.roll_blood_moon(1.0 / 12.0, true, true, false));
        // And the same 1/9 rolls in the normal window.
        let mut w = World::new("t".into(), 40, 40);
        assert!(w.roll_blood_moon(0.11, false, true, false));
    }

    #[test]
    fn the_eclipse_roll_needs_hardmode_and_a_mech_boss() {
        // `Main.cs:65421-65430`: 1-in-20, hardmode with a mech down, the same
        // dial-cooldown force-reset.
        let mut w = World::new("t".into(), 40, 40);
        w.sundial_cooldown = 8;
        w.hard_mode = true;
        w.downed_mech_boss2 = true;
        assert!(w.roll_eclipse(1.0 / 40.0, true, w.downed_mech_boss_any()));
        assert!(w.eclipse);
        assert_eq!((w.sundial_cooldown, w.moondial_cooldown), (0, 0));
        // Early mode, or no mechs: never.
        let mut w = World::new("t".into(), 40, 40);
        w.hard_mode = true;
        assert!(!w.roll_eclipse(0.0, true, false));
        assert!(!w.roll_eclipse(0.0, false, true));
    }

    use super::*;

    // --- S1-F-13 Phase 1: the moon-event state, the start/stop, the walks ---

    /// DW-1.3 (dirty): the wave-advance thresholds, transcribed from the C#
    /// tables (`NPC.cs:95289-95300`): wave 1 requires 25, wave 2 requires 40,
    /// wave 3 requires 50. A 1-point Snow-Frost kill (338) leaves 24 and
    /// holds wave 1; the next 2-point kill (342) reaches 26 >= 25 and the wave
    /// advances to 2 with `waveKills` back at 0. The Pumpkin Moon's 315 is
    /// 50 points and clears wave 1's 25 in one kill.
    #[test]
    fn the_wave_advances_at_the_c_sharp_thresholds() {
        // Snow: wave 1's requirement is 25 — 352's 5 points reach 20 in four
        // kills (holding wave 1) and the fifth takes 25 >= 25 into wave 2.
        let mut w = World::new("t".into(), 40, 40);
        w.start_snow_moon();
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (1, 0.0, 0.0)
        );
        for _ in 0..4 {
            w.report_frost_moon_kill(352);
        }
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (1, 20.0, 20.0),
            "20 < 25 holds wave 1"
        );
        w.report_frost_moon_kill(352);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (2, 0.0, 25.0),
            "25 >= 25 advances to wave 2 and zeroes the wave's points"
        );
        // Wave 2's requirement is 40: 343 (18) + 341 (20) = 38 < 40 holds,
        // one more 344 (50) clears it into wave 3.
        w.report_frost_moon_kill(343);
        w.report_frost_moon_kill(341);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (2, 38.0, 63.0),
            "38 < 40 holds wave 2 (the run's points keep accruing: 25+18+20)"
        );
        w.report_frost_moon_kill(344);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (3, 0.0, 113.0),
            "88 >= 40 advances to wave 3 (25+18+20+50)"
        );
        // Pumpkin: 315's 50 clears wave 1's 25 in a single kill.
        let mut w = World::new("t".into(), 40, 40);
        w.start_pumpkin_moon();
        w.report_pumpkin_moon_kill(315);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (2, 0.0, 50.0),
            "50 >= 25 advances the pumpkin wave"
        );
    }

    /// DW-1.3: the per-kind points tables — the Frost Moon's
    /// `NPC.cs:78518-78549` switch and the Pumpkin Moon's `:78658-78682`
    /// switch, times the single-mode 1x scalar.
    #[test]
    fn the_wave_kind_tables_match_the_c_sharp_points() {
        let mut w = World::new("t".into(), 40, 40);
        w.start_snow_moon();
        // (type, points) in the C# switch's own order.
        let snow = [
            (338, 1.0),
            (339, 1.0),
            (340, 1.0),
            (341, 20.0),
            (342, 2.0),
            (343, 18.0),
            (344, 50.0),
            (345, 150.0),
            (346, 100.0),
            (347, 8.0),
            (348, 4.0),
            (349, 4.0),
            (350, 3.0),
            (351, 10.0),
            (352, 5.0),
        ];
        let mut expected_total = 0.0;
        // Wave 20 is terminal (requirement 0), so a single kill holds its
        // points and every row's table value reads off `waveKills` directly —
        // the family's big kinds (344/345/346) would otherwise advance past
        // wave 1's 25 on the very first kill.
        for (npc_type, points) in snow {
            let mut w = World::new("t".into(), 40, 40);
            w.start_snow_moon();
            w.wave_number = 20;
            w.report_frost_moon_kill(npc_type);
            assert_eq!(
                (w.wave_kills, w.total_invasion_points),
                (points, points),
                "snow {npc_type} must be worth {points}"
            );
            expected_total += points;
        }
        // One fresh world summing the whole snow family keeps the total: the
        // 1-point kinds hold wave 1 while the big ones advance it, and the
        // run's `totalInvasionPoints` is the sum either way.
        let mut w = World::new("t".into(), 40, 40);
        w.start_snow_moon();
        for (npc_type, _) in snow {
            w.report_frost_moon_kill(npc_type);
        }
        assert_eq!(w.total_invasion_points, expected_total);
        let pumpkin = [
            (305, 1.0),
            (306, 1.0),
            (307, 1.0),
            (308, 1.0),
            (309, 1.0),
            (310, 1.0),
            (311, 1.0),
            (312, 1.0),
            (313, 1.0),
            (314, 1.0),
            (315, 50.0),
            (325, 75.0),
            (326, 2.0),
            (327, 150.0),
            (329, 5.0),
            (330, 10.0),
        ];
        for (npc_type, points) in pumpkin {
            let mut w = World::new("t".into(), 40, 40);
            w.start_pumpkin_moon();
            w.wave_number = 20;
            w.report_pumpkin_moon_kill(npc_type);
            assert_eq!(
                (w.wave_kills, w.total_invasion_points),
                (points, points),
                "pumpkin {npc_type} must be worth {points}"
            );
        }
    }

    /// DW-1.3: the walk only runs for the ACTIVE event's family — a pumpkin
    /// kill during a snow moon (and the twin) adds nothing.
    #[test]
    fn a_non_family_kill_adds_nothing() {
        let mut w = World::new("t".into(), 40, 40);
        w.start_snow_moon();
        w.report_frost_moon_kill(305);
        assert_eq!(
            (w.wave_kills, w.total_invasion_points, w.wave_number),
            (0.0, 0.0, 1),
            "a pumpkin kind during the snow moon scores nothing"
        );
        let mut w = World::new("t".into(), 40, 40);
        w.start_pumpkin_moon();
        w.report_pumpkin_moon_kill(344);
        assert_eq!(
            (w.wave_kills, w.total_invasion_points, w.wave_number),
            (0.0, 0.0, 1),
            "a snow kind during the pumpkin moon scores nothing"
        );
        // And while no event runs neither walk moves.
        let mut w = World::new("t".into(), 40, 40);
        w.report_frost_moon_kill(344);
        w.report_pumpkin_moon_kill(315);
        assert_eq!(
            (w.wave_kills, w.total_invasion_points, w.wave_number),
            (0.0, 0.0, 0)
        );
    }

    /// DW-1.3: the `requirement == 0` terminal waves never advance — wave 20
    /// (lookup index 20) and the post-stop wave 0 (index 0) both hold.
    #[test]
    fn the_terminal_waves_never_advance() {
        for terminal in [0, 20] {
            let mut w = World::new("t".into(), 40, 40);
            w.start_snow_moon();
            w.wave_number = terminal;
            w.report_frost_moon_kill(345);
            assert_eq!(
                (w.wave_number, w.wave_kills, w.total_invasion_points),
                (terminal, 150.0, 150.0),
                "wave {terminal} carries no requirement and never advances"
            );
            let mut w = World::new("t".into(), 40, 40);
            w.start_pumpkin_moon();
            w.wave_number = terminal;
            w.report_pumpkin_moon_kill(327);
            assert_eq!(
                (w.wave_number, w.wave_kills, w.total_invasion_points),
                (terminal, 150.0, 150.0),
                "wave {terminal} carries no requirement and never advances"
            );
        }
    }

    /// DW-1.1 (state half): `Main.startPumpkinMoon`/`startSnowMoon`
    /// (`Main.cs:10979-11017`) — the flags, the mutual clear, the blood
    /// moon's clear and the single-player resets with `waveNumber = 1`.
    #[test]
    fn the_start_functions_raise_one_flag_and_reset_the_waves() {
        let mut w = World::new("t".into(), 40, 40);
        w.snow_moon = true;
        w.blood_moon = true;
        w.wave_number = 9;
        w.wave_kills = 123.0;
        w.total_invasion_points = 456.0;
        w.start_pumpkin_moon();
        assert!(w.pumpkin_moon && !w.snow_moon && !w.blood_moon);
        assert_eq!((w.wave_number, w.wave_kills, w.total_invasion_points), (1, 0.0, 0.0));
        // The twin: the snow start clears the pumpkin's flag and inherits the
        // same resets.
        w.wave_number = 9;
        w.wave_kills = 12.0;
        w.total_invasion_points = 34.0;
        w.start_snow_moon();
        assert!(!w.pumpkin_moon && w.snow_moon && !w.blood_moon);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (1, 0.0, 0.0)
        );
        // A fresh world has no wave state until an event starts.
        let w = World::new("t".into(), 40, 40);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (0, 0.0, 0.0)
        );
    }

    /// DW-1.2 (state half): `Main.stopMoonEvent` (`Main.cs:10953-10976`) — the
    /// flag clears are unconditional, the point/wave resets ride the
    /// single-player arm (only when a flag was up), and the blood moon is
    /// NOT the stop's target.
    #[test]
    fn the_stop_clears_both_flags_and_zeroes_the_waves() {
        let mut w = World::new("t".into(), 40, 40);
        w.start_snow_moon();
        w.report_frost_moon_kill(344);
        w.wave_number = 7;
        w.stop_moon_event();
        assert!(!w.pumpkin_moon && !w.snow_moon);
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (0, 0.0, 0.0)
        );
        // A stop with no event running touches nothing else.
        let mut w = World::new("t".into(), 40, 40);
        w.blood_moon = true;
        w.stop_moon_event();
        assert!(w.blood_moon, "the stop does not clear the blood moon");
        assert_eq!((w.wave_number, w.wave_kills, w.total_invasion_points), (0, 0.0, 0.0));
    }

    #[test]
    fn event_flags_default_false_and_stay_out_of_the_save() {
        // `Main.eclipse/bloodMoon/pumpkinMoon/snowMoon/hardMode` start false
        // (`AmbienceServer.cs` reads them at :34-:76); no M1 event sets them.
        // `NPC.butterflyChance` resets to 0 (`NPC.cs:95172`).
        let w = World::new("t".into(), 100, 100);
        assert!(!w.eclipse);
        assert!(!w.blood_moon);
        assert!(!w.pumpkin_moon);
        assert!(!w.snow_moon);
        assert!(!w.hard_mode);
        assert_eq!(w.butterfly_chance, 0);
    }

    #[test]
    fn dialing_arms_ff_and_starts_cooldown_when_ready() {
        // `Main.Sundialing`'s SP arm (`Main.cs:6169-6181`): cooldown 0 arms the
        // fast-forward and starts the 8-day cooldown.
        let mut w = World::new("t".into(), 8, 8);
        w.sundialing();
        assert!(w.fast_forward_to_dawn);
        assert_eq!(w.sundial_cooldown, 8);
        // A running cooldown gates the re-arm (`Main.cs:6170`).
        w.fast_forward_to_dawn = false;
        w.sundialing();
        assert!(!w.fast_forward_to_dawn, "cooldown 8 blocks the re-arm");
        assert_eq!(w.sundial_cooldown, 8);
        // `Main.Moondialing` mirrors it (`Main.cs:6184-6196`).
        let mut w = World::new("t".into(), 8, 8);
        w.moondialing();
        assert!(w.fast_forward_to_dusk);
        assert_eq!(w.moondial_cooldown, 8);
    }
}

#[cfg(test)]
mod boss_tests {
    use super::*;

    #[test]
    fn the_downed_flags_default_false() {
        let w = World::new("t".into(), 100, 100);
        assert!(!w.downed_slime_king);
        assert!(!w.downed_eoc);
        assert!(!w.downed_eow);
        assert!(!w.downed_skeletron);
        assert!(!w.downed_queen_bee);
    }
}
