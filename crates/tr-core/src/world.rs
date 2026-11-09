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
    pub width: i32,
    pub height: i32,
    /// Row-major tile storage: `tiles[y * width + x]`.
    pub tiles: Vec<Tile>,
    /// `Main.tileFrameImportant`: tile types whose frames are stored in the save file.
    pub frame_important: Vec<bool>,
    /// Y of the first solid tile at the center of the map (approx. spawn altitude).
    pub spawn_y: i32,
    /// The bed spawn override (`Player.SpawnX/SpawnY`, `Player.cs:54635`'s
    /// `ChangeSpawn`) — `None` = the generated spawn (`RemoveSpawn` clears).
    pub bed_spawn: Option<(i32, i32)>,
    /// Spawn point from the save header (tile coordinates).
    pub spawn_x: i32,
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
    /// Bumped by every tile/wall/liquid mutation; the light (D2) and instance (G6)
    /// caches key on it so an edit invalidates them. Loading does not bump it.
    pub revision: u64,
    /// `Main.eclipse` — runtime event flag; no M1 event sets it (readers:
    /// `AmbienceServer.cs:34`, `:66`). Runtime-only, not saved.
    pub eclipse: bool,
    /// `Main.bloodMoon` (`AmbienceServer.cs:50`). Runtime-only, not saved.
    pub blood_moon: bool,
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
    /// `NPC.downedDeerclops` — the frozen boss's defeat (the event flag
    /// 25, `NPC.cs:79355`).
    pub downed_deerclops: bool,
    /// The Skeletron Prime head's centre + the present flag (the limbs'
    /// tether reads — the WoF write-through shape).
    pub prime_present: bool,
    pub prime_x: f32,
    pub prime_y: f32,
    /// The Skeletron head's centre (`NPC.cs:21930`'s spawn point + the
    /// hands' tether reads) — the skeletron_x/y pair.
    pub skeletron_x: f32,
    pub skeletron_y: f32,
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
    /// The body's `velocity.X` — the fists' tether anchor folds it in
    /// (`AI_047`'s `vector = Main.npc[golemBoss].Center + nPC.velocity +
    /// (0, -9 * scale)`, `NPC.cs:19261-19265`) — the WoF write-through
    /// shape. The head's tether (`aiStyle 46`, `:31148-31160`) does NOT
    /// carry the term.
    pub golem_vx: f32,
    /// `Main.slimeRain` — the Slime Rain event.
    pub slime_rain: bool,
    /// `Main.slimeRainTime` — the remaining event ticks (32400-54000 at
    /// the start, the dayRate-scaled decay).
    pub slime_rain_time: f64,
    /// `Main.slimeRainKillCount` — the kill counter (the negative half
    /// after the King spawns = the vanilla's hysteresis).
    pub slime_rain_kills: i32,
    /// `Main.pumpkinMoon` (`AmbienceServer.cs:50`). Runtime-only, not saved.
    pub pumpkin_moon: bool,
    /// `Main.snowMoon` (`AmbienceServer.cs:52`). Runtime-only, not saved.
    pub snow_moon: bool,
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
    /// `Main.wofDrawAreaTop`/`Bottom` (`Main.cs`'s statics): the WoF's
    /// underworld draw band, written by the wall's AI_027 scan and read by
    /// the Hungry's anchor (`aiStyle 29`). Runtime-only.
    pub wof_draw_top: f32,
    pub wof_draw_bottom: f32,
    /// A WoF is alive (the Hungry deactivates without one) + its hp ratio
    /// and x (the Hungry's anchor reads).
    pub wof_present: bool,
    pub wof_hp_ratio: f32,
    pub wof_x: f32,
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
}

impl World {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            tiles: vec![Tile::default(); (width as usize) * (height as usize)],
            wires: crate::wiring::WireState::default(),
            tile_entities: crate::tile_entities::TileEntities::default(),
            frame_important: Vec::new(),
            spawn_y: height / 4,
            bed_spawn: None,
            spawn_x: width / 2,
            world_surface: height as f64 * 0.25,
            rock_layer: height as f64 * 0.45,
            time: 0.0,
            day_time: true,
            moon_type: 0,
            moon_phase: 0,
            downed_slime_king: false,
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
            downed_deerclops: false,
            golem_present: false,
            golem_hp_ratio: 1.0,
            spawn_meteor: false,
            golem_x: 0.0,
            skeletron_x: 0.0,
            prime_present: false,
            prime_x: 0.0,
            prime_y: 0.0,
            skeletron_y: 0.0,
            golem_y: 0.0,
            unlocked_slime_rainbow_spawn: false,
            golem_vx: 0.0,
            slime_rain: false,
            slime_rain_time: 0.0,
            slime_rain_kills: 0,
            sundial_cooldown: 0,
            moondial_cooldown: 0,
            fast_forward_to_dawn: false,
            fast_forward_to_dusk: false,
            hell_back_style: 0,
            column_surface: Vec::new(),
            chunks_x: 0,
            chunks_y: 0,
            chunk_has: Vec::new(),
            seed: 0,
            seeds: crate::seeds::SeedFlags::default(),
            revision: 0,
            extras: std::collections::HashMap::new(),
            eclipse: false,
            blood_moon: false,
            all_players_asleep: false,
            pumpkin_moon: false,
            snow_moon: false,
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
            wof_draw_top: -1.0,
            wof_draw_bottom: -1.0,
            wof_present: false,
            wof_hp_ratio: 1.0,
            wof_x: 0.0,
            saved_angler: false,
            saved_wizard: false,
            saved_stylist: false,
            saved_tax_collector: false,
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
        self.blood_moon = true;
        // `:65344-65346` — "instantly reusable after certain celestial
        // events".
        self.sundial_cooldown = 0;
        self.moondial_cooldown = 0;
        true
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

    /// Approximate surface (first active tile scanning down the middle column).
    pub fn compute_spawn_y(&mut self) {
        let x = self.width / 2;
        for y in 0..self.height {
            if self.is_active(x, y) {
                self.spawn_y = (y - 6).max(0);
                return;
            }
        }
    }

    pub fn memory_bytes(&self) -> usize {
        self.tiles.len() * std::mem::size_of::<Tile>()
    }
}

#[cfg(test)]
mod tests {
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
