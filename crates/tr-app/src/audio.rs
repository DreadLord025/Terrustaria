//! Audio: original PCM SFX via `SoundEffectReader`, MS-ADPCM music from XACT banks.
//!
//! The music engine mirrors `Main.UpdateAudio` + `LegacyAudioSystem.UpdateCommonTrack`
//! (`Main.cs:11017`, `LegacyAudioSystem.cs:292`): a per-track `musicFade` ramp of
//! ±0.005 per frame, one rodio [`Sink`] per track standing in for the XACT
//! `AudioTrack`, and `Main.newMusic`/`Main.curMusic` selection. Positional SFX use
//! [`crate::sound`] for the `LegacySoundPlayer.PlaySound` attenuation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::Source;

use crate::sound::{self, Listener};

/// `MusicID.Count` (`MusicID.cs:228`); `musicFade` is `Main.maxMusic` long.
pub const MAX_MUSIC: usize = 105;

/// `TrackIndex` → `LegacySoundPlayer.PlaySound` `type`. The cue assets the game
/// currently triggers by name.
const SFX_CUES: &[(&str, u32)] = &[
    ("Dig_0", 0),
    ("Dig_1", 0),
    ("Dig_2", 0),
    // `WorldGen.KillTile_PlaySounds` families (`WorldGen.cs:66427-66575`):
    // sound id 6 (`SoundGrass`, `LegacySoundPlayer.cs:706`), 13 (`Shatter`,
    // `:783`), 21 (`Tink_0..2`, `:924`), 18 (`Coins`, `:865`), and the
    // `SoundID.Item*` / `NPCHit25` styles of the item/NPC cases.
    ("Grass", 6),
    ("Shatter", 13),
    ("Tink_0", 21),
    ("Tink_1", 21),
    ("Tink_2", 21),
    ("Coins", 18),
    ("Item_27", 2),
    // W4-f47 — `WorldItem.ShimmerEffect`'s cue (`SoundID.Item176`,
    // `WorldItem.cs:1668`): the case-2 style-176 ×0.9 volume tweak.
    ("Item_176", 2),
    // S1-E-21 wave D — the Moon Lord legs' fast-run footstep
    // (`Player.cs:35959-35964` swaps `hermesStepSound` to Item style 24).
    ("Item_24", 2),
    ("Item_30", 2),
    ("Item_48", 2),
    ("Item_49", 2),
    ("Item_50", 2),
    ("Item_52", 2),
    ("Item_127", 2),
    ("Item_173", 2),
    ("Item_177", 2),
    // W4-e19 — the shimmer transmute cue (`WorldItem.ShimmerEffect`,
    // `WorldItem.cs:1668`: `SoundID.Item176 = (2, 176)`, `SoundID.cs:1560`).
    ("Item_176", 2),
    ("NPC_Hit_25", 3),
    ("Item_1", 2),
    // S1-E-21f — the coin-pickup chime family (`SoundCoin[5]`,
    // `LegacySoundPlayer.cs:42`/:211-213): case 38 (`:1337-1350`) picks
    // `Coin_<Next(5)>` — the pickup sites `Player.cs:39444`/`:39491`/
    // `:39534`/`:39559` (`IsACoin ? 38 : 7`) and the revenge `moneyPing`
    // (`NPC.cs:90054`). NOTE: legacy id 18 plays the DIFFERENT `Coins`
    // asset (`Sounds/Coins.xnb`, the tile-place coin-pile cue) — the
    // pre-wave stand-in row that aliased `Coin_0` to id 18 is retired;
    // `sound_type` must answer 38 for the case-38 pitch/attenuation path.
    ("Coin_0", 38),
    ("Coin_1", 38),
    ("Coin_2", 38),
    ("Coin_3", 38),
    ("Coin_4", 38),
    // S1-E-21f — the hurt family's slots 1/2 (`SoundPlayerHit[3]`,
    // `LegacySoundPlayer.cs:195-197`): case 1 (`:503-515`) picks
    // `Player_Hit_<Next(3)>`.
    ("Player_Hit_0", 1),
    ("Player_Hit_1", 1),
    ("Player_Hit_2", 1),
    // W4-g22 — the female hurt family (`SoundFemaleHit[3]`,
    // `LegacySoundPlayer.cs:198-200`): case 20 (`:911-923`) picks
    // `Female_Hit_<Next(3)>`. The menuMode 2 gender toggle's male->female
    // preview plays it (`Main.cs:49045`); the voice-variant-2 hurt arm
    // (`Player.cs:38587`) is the same case.
    ("Female_Hit_0", 20),
    ("Female_Hit_1", 20),
    ("Female_Hit_2", 20),
    // The legacy "Player Killed" id 5 (`Player.PlayDeathSound`,
    // `Player.cs:38745`) — the `Sounds/Player_Killed.xnb` asset.
    ("Player_Killed", 5),
    // W4-e10 — the female hurt family (`SoundFemaleHit[3]`,
    // `LegacySoundPlayer.cs:50`/:198-200): case 20 (`:911-923`) picks
    // `Female_Hit_<Next(3)>` — the `voiceOverride 2` / `voiceVariant 2`
    // arms (`Player.cs:38587`/:38707).
    ("Female_Hit_0", 20),
    ("Female_Hit_1", 20),
    ("Female_Hit_2", 20),
    // W4-e10 — the hurt voice's mount/armor int arms (`Player.cs:38655-
    // :38691`): `PlaySound(3, vector, 6/47/5/2)` (the Unicorn 52, the
    // Velociraptor 54, the Basilisk 61, the werewolf, the Necro set) and
    // `PlaySound(4, vector, 4)` (the Goat 55 / Wall Slime 56).
    ("NPC_Hit_5", 3),
    ("NPC_Hit_47", 3),
    // W4-e10 — `SoundID.BellHurt` (`SoundID.cs`, id 2 style 35): the
    // cowbell voice's hurt/death cue (`Player.cs:38599`/:38730) — the
    // case-2 style-35 music-pitch arm (`LegacySoundPlayer.cs:635-639`).
    ("Item_35", 2),
    // S1-E-21f — the drowning gasp (`Player.CheckDrowning`'s breath-zero
    // arm, `Player.cs:23257`): `SoundID.Drown = 23` (`SoundID.cs:68`),
    // case 23 (`LegacySoundPlayer.cs:947-956`) — Volume/Pan only, no draws.
    ("Drown", 23),
    // W4-e10 — the full-mana chime (`SoundID.MaxMana = 25`,
    // `Sounds/MaxMana.xnb`, `LegacySoundPlayer.cs:239`): case 25
    // (`:963-968`) — Volume/Pan only, no draws. The full-bar regen trigger
    // (`Player.cs:19552`) and `EmitMaxManaEffect` (`:49800`).
    ("MaxMana", 25),
    ("NPC_Hit_1", 3),
    // The NPC hit family (the C#'s sound id 3, `LegacySoundPlayer.cs:646`):
    // the per-kind hit cues (`NPC.HitSound`, `NPC.cs:6475`) — the Skeletron's
    // "NPC_Hit_2", the skeleton's, the bosses' rows.
    ("NPC_Hit_1", 3),
    ("NPC_Hit_2", 3),
    ("NPC_Hit_3", 3),
    ("NPC_Hit_4", 3),
    ("NPC_Hit_6", 3),
    ("NPC_Hit_7", 3),
    ("NPC_Hit_8", 3),
    ("NPC_Hit_9", 3),
    ("NPC_Hit_11", 3),
    // S1-E-21g — the case-15/36 roar bank (`Sounds/Roar_0..2`,
    // `LegacySoundPlayer.cs:227-230`): the boss spawn/phase roars
    // (`NPC.cs:20375`/`:21946`/`:22169`/`:32418`), the EoC dash roars
    // (`:20500`/`:20694` — case 36) and the Sand Shark's quiet style-4
    // remap (`:40581`, case 15's ×0.25 arm).
    ("Roar_1", 15),
    ("Roar_2", 15),
    // S1-E-21g — the case-36 dash roars share the `SoundRoar` slots but
    // ride the legacy id 36 (`LegacySoundPlayer.cs:1309-1325`); the bank
    // resolves the asset by name, the play methods pass the case id.
    // S1-E-21g — the Pixie wing hum (`Sounds/Pixie`,
    // `LegacySoundPlayer.cs:178`, case 27 `:979-996`).
    ("Pixie", 27),
    // S1-E-21g — the wave-G death cues the AI-side plays name: the
    // leech head's tunnel-entry burst (`NPCDeath13`, `NPC.cs:50968`),
    // the tower dissolve (`NPCDeath22`, `:39059`) and the Moon Lord
    // crash cry (`NPCDeath61`, `:41432`); `NPC_Killed_6` (the parts'
    // laser wind-up, `:41838`/`:42338`/`:42611`) is already above.
    ("NPC_Killed_13", 4),
    ("NPC_Killed_61", 4),
    // S1-E-21h d77 — the Sharknado bolt's death cry (`Projectile.cs:73763`,
    // `PlaySound(4, Center, 19)` — the int overload, case 4 style 19; the
    // `Sounds/NPC_Killed_19.xnb` asset verified on disk).
    ("NPC_Killed_19", 4),
    // S1-E-21g — the attack/shot cues the caster teleport plays
    // (`SoundID.Item8`, `NPC.cs:20862`/`:20931`) and the EoC servant
    // launch (`SoundID.Item14`, the goblin-fist arm `:19415`) — Item_8
    // is already in the wave-A bank; Item_14/17 join here.
    ("Item_14", 2),
    ("Item_17", 2),
    // The Headless Horseman's hit cue (`NPC.cs:12978`'s `NPCHit12`,
    // S1-F-13b's row) — S1-E-21 wave C closes the bank gap.
    ("NPC_Hit_12", 3),
    // S1-D-3 W-9 fishron wiring — Duke Fishron's hit cue
    // (`NPC.cs:13713`'s `NPCHit14`) — the S1-E-21 bank-gap closure.
    ("NPC_Hit_14", 3),
    // S1-D-3z — the Blood Zombie's hit cue (`NPC.cs:14613`'s NPCHit18).
    ("NPC_Hit_18", 3),
    // W4-d42 - the roster's hit cue (`SoundID.NPCHit32`).
    ("NPC_Hit_32", 3),
    ("NPC_Hit_19", 3),
    ("NPC_Hit_21", 3),
    // W4-d34 kinds 1-70: the Vulture's NPCHit28 (`NPC.cs:9563`), the Antlion's
    // NPCHit31 (`:9674`) — the `Sounds/NPC_Hit_*.xnb` assets ship; the bank close.
    ("NPC_Hit_28", 3),
    ("NPC_Hit_31", 3),

    // W4-d23 — the Flying Snake's hit cue (`NPC.cs:11920`'s NPCHit23):
    // the S1-E-21 wave-C bank-gap close.
    ("NPC_Hit_23", 3),
    // The small-kinds wiring wave (S1-D-3-wire-smallkinds): the Angry
    // Nimbus' NPCHit30 (`NPC.cs:12229`), the Dungeon Spirit's NPCHit36
    // (`:12747`) and the Force Bubble's NPCHit43 (`:13889`) — the
    // `Sounds/NPC_Hit_*.xnb` assets exist; S1-E-21 wave C's bank-gap close.
    ("NPC_Hit_30", 3),
    ("NPC_Hit_36", 3),
    ("NPC_Hit_43", 3),
    // W4-c4 merge dedupe: the antlion pair's NPCHit32 is the d42 row
    // above; the BloodFeeder's NPCHit13 is the d37 row below (one bank
    // row per cue - the spine-end precedent).
    ("NPC_Hit_39", 3),
    ("NPC_Hit_40", 3),
    ("NPC_Hit_42", 3),
    // S1-E-21h d75 part 4 — the 1054 SpiderBoulder's death cue
    // (`Projectile.cs:74865`, `SoundID.NPCHit29` = (3, 29),
    // `SoundID.cs:1285`; `Sounds/NPC_Hit_29.xnb` verified on disk). The
    // row rides d3's pushed tip 2d01693ca — the bank closes the gap
    // ahead of the merge.
    ("NPC_Hit_29", 3),
    // The Force Bubble's hit cue (`NPC.cs:13889`'s `NPCHit43`) and the
    // Tesla Turret's (`NPC.cs:13930`'s `NPCHit53`, S1-F-13d).
    ("NPC_Hit_43", 3),
    ("NPC_Hit_53", 3),
    ("NPC_Hit_51", 3),
    ("NPC_Hit_55", 3),
    ("NPC_Hit_57", 3),
    // W4-d37 — the NPCID 211-280 band's hit cues (the `SetDefaults` HitSound
    // rows): NPCHit5 (ice golem), NPCHit13 (blood feeder/jelly, ichor
    // sticker), NPCHit20 (blood crawlers), NPCHit23 (flying snake), NPCHit29
    // (black recluse wall), NPCHit45 (mushi ladybug), NPCHit46 (parrot),
    // NPCHit49 (reaper). The `Sounds/NPC_Hit_*.xnb` assets ship with the
    // content; S1-E-21 wave C's bank-gap close.
    ("NPC_Hit_5", 3),
    ("NPC_Hit_13", 3),
    ("NPC_Hit_20", 3),
    ("NPC_Hit_23", 3),
    ("NPC_Hit_29", 3),
    ("NPC_Hit_45", 3),
    ("NPC_Hit_46", 3),
    ("NPC_Hit_49", 3),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_56", 3),
    ("NPC_Killed_60", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Killed_17", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_48", 3),
    ("NPC_Killed_50", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_34", 3),
    ("NPC_Hit_35", 3),
    ("NPC_Hit_47", 3),
    ("NPC_Hit_52", 3),
    ("NPC_Killed_37", 4),
    ("NPC_Killed_38", 4),
    ("NPC_Killed_49", 4),
    ("NPC_Killed_55", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_44", 3),
    ("NPC_Killed_46", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_41", 3),
    ("NPC_Killed_43", 4),
    // W4-d40: the roster wave's NPC hit/death cues (the `Sounds/NPC_*.xnb`
    // assets ship with the content; the S1-E-21 bank-gap close).
    ("NPC_Hit_15", 3),
    // W4-g8 lane 1 batch 1 — the d22-d33 kinds' hit cues: the Vulture's
    // NPCHit28 (`NPC.cs:9563`), the Antlion's NPCHit31 (`:9674`), the
    // Pixie's NPCHit5 (`:9781`), the Wraith's NPCHit54 (`:9862`). The
    // jellies' NPCHit25 (`:9591`/`:9605`) is already above; the weapons'/
    // Mimic's NPCHit4 (`:9887`/`:9900`/`:9913`) and the Spike Ball's
    // NPCHit1 (`:9689`) are in the wave-A bank.
    ("NPC_Hit_5", 3),
    ("NPC_Hit_28", 3),
    ("NPC_Hit_31", 3),
    ("NPC_Hit_54", 3),
    // W4-g8 lane 1 batch 2 — the WallCreeper's NPCHit29 (`NPC.cs:11067`/
    // `:11082`) and the Derpling's NPCHit22 (`:11247`).
    ("NPC_Hit_29", 3),
    ("NPC_Hit_22", 3),
    // W4-g8 lane 1 batch 3 — the FlyingSnake's NPCHit23 (`NPC.cs:11920`).
    ("NPC_Hit_23", 3),
    // W4-f42 — the sandstorm debris + desert family's cues (the S1-E-21
    // wave C bank-gap close): the Vulture's NPCHit28 (`NPC.cs:9563`), the
    // Antlion/WalkingAntlion's NPCHit31 (`:9674`/`:14862`), the
    // FlyingAntlion's NPCHit32 (`:14877`); the deaths NPCDeath31/34/35
    // (`:9564`/`:9675`+`:14863`/`:14878`). The Tumbleweed's Hit11/Death15
    // and the mummies' Hit1/Death6 are already above.
    ("NPC_Hit_28", 3),
    ("NPC_Hit_31", 3),
    ("NPC_Hit_32", 3),
    ("NPC_Killed_31", 4),
    ("NPC_Killed_34", 4),
    ("NPC_Killed_35", 4),
    // W4-d9 wiring (d40 rows): the Phantasm Dragon's NPCHit56/NPCDeath60
    // (`NPC.cs:14425-14426`) — the `Sounds/NPC_*.xnb` assets ship with the
    // content; the S1-E-21 bank-gap close.
    ("NPC_Hit_56", 3),
    ("NPC_Killed_60", 4),
    // W4-d41 — the cavern crawdads' NPCHit33 (`NPC.cs:14674`) and the
    // shellies' NPCHit38 (`:14686`) — the bank-gap close.
    ("NPC_Hit_33", 3),
    ("NPC_Hit_38", 3),
    // W4-d41 — the salamanders' NPCHit50 (`NPC.cs:14699`).
    ("NPC_Hit_50", 3),
    // W4-d41 — the antlion charger's NPCHit31 (`NPC.cs:14836`) and the
    // swarmer's NPCHit32 (`:14850`).
    ("NPC_Hit_31", 3),
    ("NPC_Hit_32", 3),
    // W4-d41 — the desert ghouls' NPCHit37 (`NPC.cs:14890`).
    ("NPC_Hit_37", 3),
    // W4-d41 — the sand poachers' NPCHit13 (`NPC.cs:14941`).
    ("NPC_Hit_13", 3),
    // W4-d36 census batch 0 - the tortoises' NPCHit24 (`NPC.cs:10939`/
    // `:10954`); the `Sounds/NPC_Hit_24.xnb` asset ships with the content.
    ("NPC_Hit_24", 3),
    // W4-d36 E4 batch 2 — the Black Recluse's NPCHit29 (`NPC.cs:11038`,
    // the spider cue); the `Sounds/NPC_Hit_29.xnb` asset ships with the
    // content.
    ("NPC_Hit_29", 3),
    // W4-d36 E4 batch 3 — the pigron trio's NPCHit27 (`NPC.cs:11136`/
    // `:11149`/`:11287`, the pig cue); the `Sounds/NPC_Hit_27.xnb` asset
    // ships with the content.
    ("NPC_Hit_27", 3),
    // W4-d36 E4 batch 4 — the Lihzahrd pair's NPCHit26 (`NPC.cs:11541`/
    // `:11554`, the lizard cue); the `Sounds/NPC_Hit_26.xnb` asset ships
    // with the content.
    ("NPC_Hit_26", 3),
    ("NPC_Killed_1", 4),
    ("NPC_Killed_2", 4),
    ("NPC_Killed_3", 4),
    ("NPC_Killed_4", 4),
    ("NPC_Killed_5", 4),
    ("NPC_Killed_6", 4),
    // S1-E-21h d77 — the Stardust Jellyfish's death cry
    // (`Projectile.cs:72571`'s `PlaySound(4, x, y, 7)`) — the
    // `Sounds/NPC_Killed_7.xnb` asset verified on disk.
    // S1-E-21h f1 owed-arm phase — the Stardust jellyfish 539's death cry
    // (`Projectile.cs:72571`, `SoundID.NPCDeath7`; the `Sounds/
    // NPC_Killed_7.xnb` asset verified on disk).
    ("NPC_Killed_7", 4),
    // The Wyvern's death cue (`NPC.cs:9951`'s `NPCDeath8`, S1-D-3 wyvern)
    // — the `Sounds/NPC_Killed_8.xnb` asset ships with the content.
    ("NPC_Killed_8", 4),
    // W4-f2 reconcile (e9 union, research/94 §3) — the Vile Spit's
    // `ai[0] == 2` tick cry (`SoundID`-less `PlaySound(4, pos, 9)`,
    // `NPC.cs:21480`; `Sounds/NPC_Killed_9.xnb`).
    ("NPC_Killed_9", 4),
    ("NPC_Killed_10", 4),
    ("NPC_Killed_11", 4),
    ("NPC_Killed_12", 4),
    ("NPC_Killed_14", 4),
    ("NPC_Killed_15", 4),
    // The Headless Horseman's death cue (`NPC.cs:12987`'s `NPCDeath18`,
    // S1-F-13b's row) — S1-E-21 wave C closes the bank gap.
    ("NPC_Killed_18", 4),
    // W4-c4 merge dedupe: the BloodFeeder's NPCDeath19 is the d37 row
    // below (one bank row per cue).
    // S1-D-3 W-9 fishron wiring — Duke Fishron's death cue
    // (`NPC.cs:13714`'s `NPCDeath20`) — the S1-E-21 bank-gap closure.
    ("NPC_Killed_20", 4),
    // S1-D-3z — the Blood Zombie's death cue (`NPC.cs:14614`'s NPCDeath21).
    ("NPC_Killed_21", 4),
    // W4-d42 - the roster's death cue (`SoundID.NPCDeath35`).
    ("NPC_Killed_35", 4),
    ("NPC_Killed_22", 4),
    ("NPC_Killed_24", 4),
    // W4-d34 kinds 1-70: the jellyfish pair's NPCDeath28 (`NPC.cs:9592`/`:9606`),
    // the Vulture's NPCDeath31 (`:9564`), the Antlion's NPCDeath34 (`:9675`).
    ("NPC_Killed_28", 4),
    ("NPC_Killed_31", 4),
    ("NPC_Killed_34", 4),

    // W4-d23 — the Flying Snake's death cue (`NPC.cs:11922`'s NPCDeath26).
    ("NPC_Killed_26", 4),
    // W4-d36 census batch 0 - the tortoises' NPCDeath27 (`NPC.cs:10941`/
    // `:10956`); the `Sounds/NPC_Killed_27.xnb` asset ships with the content.
    ("NPC_Killed_27", 4),
    // W4-d36 E4 batch 2 — the Black Recluse's NPCDeath32 (`NPC.cs:11039`,
    // the spider death cue); the `Sounds/NPC_Killed_32.xnb` asset ships
    // with the content.
    ("NPC_Killed_32", 4),
    // W4-d36 E4 batch 3 — the pigron trio's NPCDeath30 (`NPC.cs:11137`/
    // `:11150`/`:11288`, the pig death cue); the `Sounds/NPC_Killed_30.xnb`
    // asset ships with the content.
    ("NPC_Killed_30", 4),
    // W4-d36 E4 batch 4 — the Lihzahrd pair's NPCDeath29 (`NPC.cs:11542`/
    // `:11555`, the lizard death cue); the `Sounds/NPC_Killed_29.xnb`
    // asset ships with the content.
    ("NPC_Killed_29", 4),
    // The small-kinds wiring wave (S1-D-3-wire-smallkinds): the Angry
    // Nimbus' NPCDeath33 (`NPC.cs:12230`), the Dungeon Spirit's NPCDeath39
    // (`:12748`) and the Force Bubble's NPCDeath45 (`:13890`) — the
    // `Sounds/NPC_Killed_*.xnb` assets exist; the bank-gap close.
    // W4-d35 (A): the 73/74/75/77/78 cues - NPCHit5/NPCDeath7 already
    // landed with W4-d37 (the ice golem/beetles rows); dedupe keeps those.
    // W4-d35 (B): the 79/80/82/83/84 hit/death cues (the Sounds/NPC_*.xnb
    // assets ship; the S1-E-21 wave-C bank-gap close).
    ("NPC_Hit_54", 3),
    ("NPC_Killed_52", 4),
    // W4-d35 (E): the 103 GreenJellyfish NPCDeath28 already landed with
    // W4-d34 (the BlueJellyfish row); dedupe keeps that row.
    // W4-d35 (F): the 110/112/116/120/121 hit/death cues (the Sounds/NPC_*.xnb
    // assets ship; the S1-E-21 wave-C bank-gap close).
    ("NPC_Killed_9", 4),
    ("NPC_Killed_33", 4),
    ("NPC_Killed_39", 4),
    ("NPC_Killed_45", 4),
    // W4-c3 merge dedupe: the Water Bolt Mimic's NPCDeath52
    // (`NPC.cs:17746`) is already registered by the d35 (B) row above
    // (the spine-end precedent - one bank row per cue).
    // W4-c4 merge dedupe: the antlion pair's NPCDeath35 is the d42 row
    // above (one bank row per cue).
    ("NPC_Killed_42", 4),
    ("NPC_Killed_44", 4),
    // W4-f2 closeout (e9b tail union, research/94 §10) — Medusa's
    // stone-gaze cry (`NPC.cs:56403`, `PlaySound(4, pos, 17)` int
    // overload) and the balloon pops' `NPCDeath63` (`NPC.cs:43726`/
    // `:47951`/`:48062`, the style façade).
    ("NPC_Killed_17", 4),
    ("NPC_Killed_63", 4),
    // The Force Bubble's death cue (`NPC.cs:13890`'s `NPCDeath45`) and the
    // Tesla Turret's (`NPC.cs:13931`'s `NPCDeath56`, S1-F-13d).
    ("NPC_Killed_45", 4),
    ("NPC_Killed_56", 4),
    // W4-f2 reconcile (e9 union, research/94 §3) — the Ghost's flee cry
    // (`NPC.cs:24693`'s `SoundID.NPCDeath55`, the aiStyle-22 type-316
    // `localAI[3]` 0→1 latch; the style façade).
    ("NPC_Killed_55", 4),
    ("NPC_Killed_54", 4),
    ("NPC_Killed_57", 4),
    ("NPC_Killed_59", 4),
    ("NPC_Killed_62", 4),
    // W4-d33 — the Windy Balloon's pop cue (`NPC.cs:16716`'s
    // NPCDeath63, `SoundID.cs:1377`). d42 batch-3's bank row,
    // constants-copy (keep-one d37c-E3).
    ("NPC_Killed_63", 4),
    ("NPC_Killed_66", 4),
    // W4-d37 — the NPCID 211-280 band's death cues (the `SetDefaults`
    // DeathSound rows): NPCDeath7 (ice golem), NPCDeath16 (beetles),
    // NPCDeath19 (blood feeder/jelly, ichor sticker), NPCDeath23 (blood
    // crawlers), NPCDeath26 (flying snake), NPCDeath32 (black recluse wall),
    // NPCDeath47 (mushi ladybug), NPCDeath48 (parrot), NPCDeath51 (reaper).
    ("NPC_Killed_7", 4),
    ("NPC_Killed_16", 4),
    ("NPC_Killed_19", 4),
    ("NPC_Killed_23", 4),
    ("NPC_Killed_26", 4),
    ("NPC_Killed_32", 4),
    ("NPC_Killed_47", 4),
    ("NPC_Killed_48", 4),
    ("NPC_Killed_51", 4),
    // W4-d125 RETIRE dedupe fold (research/319 §2): the Empress 636's
    // death cue (`NPC.cs:17197` `SoundID.NPCDeath65`) — d43's batch-1
    // row kept verbatim; the d125 roster's `death_sound` arm reads it.
    ("NPC_Killed_65", 4),
    // W4-g8 lane 1 batch 1 — the d22-d33 kinds' death cues: the Pixie's
    // NPCDeath7 (`NPC.cs:9783`), the jellies' NPCDeath28 (`:9592`/`:9606`),
    // the Vulture's NPCDeath31 (`:9564`), the Antlion's NPCDeath34
    // (`:9675`), the Wraith's NPCDeath52 (`:9863`). The weapons'/Mimic's
    // NPCDeath6 (`:9888`/`:9901`/`:9914`) and the Spike Ball's NPCDeath1
    // (`:9690`) are in the wave-A bank.
    ("NPC_Killed_7", 4),
    ("NPC_Killed_28", 4),
    ("NPC_Killed_31", 4),
    ("NPC_Killed_34", 4),
    ("NPC_Killed_52", 4),
    // W4-g8 lane 1 batch 2 — the WallCreeper's NPCDeath32 (`NPC.cs:11068`/
    // `:11083`) and the Derpling's NPCDeath25 (`:11248`).
    ("NPC_Killed_32", 4),
    ("NPC_Killed_25", 4),
    // W4-g8 lane 1 batch 3 — the FlyingSnake's NPCDeath26 (`NPC.cs:11922`).
    ("NPC_Killed_26", 4),
    // W4-d41 — the cavern crawdads' NPCDeath36 (`NPC.cs:14675`) and the
    // shellies' NPCDeath41 (`:14687`) — the bank-gap close.
    ("NPC_Killed_36", 4),
    ("NPC_Killed_41", 4),
    // W4-d41 — the salamanders' NPCDeath53 (`NPC.cs:14700`).
    ("NPC_Killed_53", 4),
    // W4-d41 — the antlion charger's NPCDeath34 (`NPC.cs:14837`) and the
    // swarmer's NPCDeath35 (`:14851`).
    ("NPC_Killed_34", 4),
    ("NPC_Killed_35", 4),
    // W4-d41 — the desert ghouls' NPCDeath40 (`NPC.cs:14891`).
    ("NPC_Killed_40", 4),
    // W4-d41 — the sand poachers' NPCDeath19 (`NPC.cs:14942`).
    ("NPC_Killed_19", 4),
    ("NPC_Killed_52", 4),
    ("NPC_Hit_58", 3),
    ("NPC_Killed_67", 4),
    ("NPC_Killed_63", 4),
    ("NPC_Killed_9", 4),
    ("NPC_Killed_64", 4),
    ("NPC_Hit_41", 3),
    ("NPC_Killed_43", 4),
    ("NPC_Killed_65", 4),
    // The NPC death family (the sound id 4, `LegacySoundPlayer.cs:676`): the
    // per-kind death cues (`NPC.DeathSound`, `NPC.cs:6477`).
    ("Double_Jump", 16),
    // S1-E-21 wave D — the fast-run footstep (`SoundID.Run = 17`,
    // `SoundID.cs:56`; `Player.SpawnFastRunParticles`, `Player.cs:20214`;
    // `SoundRun` loads `Sounds/Run`, `LegacySoundPlayer.cs:236`).
    ("Run", 17),
    // `SoundEngine.PlaySound(8/:9)` — `WorldGen.OpenDoor` (`WorldGen.cs:37732`)
    // and `CloseDoor` (`:31906`).
    ("Door_Opened", 8),
    ("Door_Closed", 9),
    // S1-E-21i — the wiring switch click (`Wiring.HitSwitch`'s seven
    // `PlaySound(28, ...)` sites, `Wiring.cs:267`/`:272`/`:285`/`:310`/
    // `:323`/`:339`/`:375`); `SoundMech[0]` loads `Sounds/Mech_0`
    // (`LegacySoundPlayer.cs:22`/:176, the one-entry array).
    ("Mech_0", 28),
    // `SoundID.Item4 = new LegacySoundStyle(2, 4)` (`SoundID.cs:1387`) — the
    // dial arm (`Player.cs:32642`/:32652, `SoundEngine.PlaySound(SoundID.Item4,
    // ...)`); the legacy Item track, like `Item_1` above.
    ("Item_4", 2),
    // `SoundEngine.PlaySound(7)` — the craft grant (`CraftingEffects.cs:51`,
    // `SpawnEffects_BeforeGrantingItem` ← `OnCraft`); `SoundID.Grab = 7`
    // (`SoundID.cs`), the `Grab.xnb` cue.
    ("Grab", 7),
    // The chest-open click's menu cues (`Player.cs:33242/:33237/:33251` —
    // open 10 / close 11 / re-open 12; the per-tick range close plays 11 at
    // `:31082`/:31088): `SoundID.MenuOpen/MenuClose/MenuTick = 10/11/12`
    // (`SoundID.cs:42/44/46`).
    ("Menu_Open", 10),
    ("Menu_Close", 11),
    ("Menu_Tick", 12),
    // W4-c24 — the Journey research cues (`SoundID.Research = 63` /
    // `ResearchComplete = 64`, `SoundID.cs:148/:150`): the sacrifice thunk
    // picks `SoundResearch[Next(1, 4)]` (`LegacySoundPlayer.cs:480-492`), the
    // completion chime is `SoundResearch[0]` (`:493-502`); the bank loads
    // `Sounds/Research_0..3` (`:188-191`).
    ("Research_0", 64),
    ("Research_1", 63),
    ("Research_2", 63),
    ("Research_3", 63),
    // `SoundEngine.PlaySound(24)` — the town-NPC talk open
    // (`Main.cs:43083`); `SoundID.Chat = 24` (`SoundID.cs:70`), the
    // `Sounds/Chat` cue (`LegacySoundPlayer.cs:202`, case 24 `:957-962`).
    ("Chat", 24),
    // W4-f48 parry wave — `SoundEngine.PlaySound(25, base.Center)` on the
    // shield-parry cooldown's zero tick (`Player.cs:31724`); legacy id 25
    // is the MaxMana cue (`SoundID.MaxMana = 25`, `LegacySoundPlayer.cs`
    // case 25 — `SoundInstanceMaxMana`, Volume/Pan only, no draws).
    ("MaxMana", 25),
    // `SoundEngine.PlaySound(40)` — the capture camera's shutter
    // (`CaptureInterface.cs:1592`); `SoundID.Camera = 40` (`SoundID.cs:102`),
    // the `Sounds/Camera` cue (`LegacySoundPlayer.cs:210`, case 40 `:1359`).
    ("Camera", 40),
    // `SoundEngine.PlaySound(22, X*16, Y*16)` — `Chest.Unlock`/`Chest.Lock`
    // (`Chest.cs:408`/:482); `SoundID.Unlock = 22` (`SoundID.cs:66`), the
    // positional `Unlock.xnb` cue.
    ("Unlock", 22),
    // `SoundID.Item53/55/56` — the minecart cues: the landing thud
    // (`DelegateMethods.cs:248-251`), the rolling rattle
    // (`Player.cs:19870/:19928`) and the bouncy bumper
    // (`DelegateMethods.cs:253-256`); the legacy Item track like `Item_1`.
    ("Item_53", 2),
    ("Item_55", 2),
    ("Item_56", 2),
    // W4-c14 - the Pogo Stick's hop cue (`SoundID.Item168`,
    // `Player.cs:19784`/`:20764`; `Sounds/Item_168.xnb`).
    ("Item_168", 2),
    // W4-c14 - the Meowcart's case-37 slot pair (`LegacySoundPlayer.cs:1328`
    // `Next(57, 59)`; `Mount.cs:661`/`:666` play legacy id 37).
    ("Item_57", 37),
    ("Item_58", 37),
    // S1-E-21 wave A — the item-use family: every `Sounds/Item_<style>.xnb`
    // an `Item.UseSound` row names (`itemdata.rs::use_sound` →
    // `soundid.rs::SOUND_STYLES`, `SoundID.cs:1384-1582`'s
    // `new LegacySoundStyle(2, style)` rows), plus 18/19 — the style-1
    // variation remap's targets (`LegacySoundPlayer.cs:539-550`).
    ("Item_2", 2),
    ("Item_3", 2),
    ("Item_5", 2),
    ("Item_6", 2),
    ("Item_7", 2),
    ("Item_8", 2),
    ("Item_9", 2),
    ("Item_10", 2),
    ("Item_11", 2),
    ("Item_12", 2),
    ("Item_13", 2),
    // S1-E-21h — the projectile-wave bank gaps: the bomb blast's Item14
    // (`Projectile.cs:78331`/:78386), the dart/petal latch's Item17
    // (`:55314`/:55324`/:55358`), the laser latch's Item33 (`:55232`/
    // `:55242`/:55309`) and the Inferno Fork's Item74 (`:28363`).
    ("Item_14", 2),
    ("Item_17", 2),
    ("Item_33", 2),
    ("Item_74", 2),
    // S1-E-21h d77 — the Kill()-tail bank gaps: the Flairon bubble's Item54
    // (`Projectile.cs:72851`), the snowball's Item51 (`:74762`), the
    // Houndius fireball's Item45 (`:75278`), the electrosphere's Item94
    // (`:73530`), the Xenopopper's Item96 (`:73400`), the barnacle's
    // Item167 (`:75288`) and the rocket families' Item62
    // (`Kill_Bombs_DoUsualKillCode` `:77737`-`:78146`). All seven `.xnb`
    // assets verified on disk.
    ("Item_45", 2),
    ("Item_51", 2),
    ("Item_54", 2),
    ("Item_62", 2),
    ("Item_94", 2),
    ("Item_96", 2),
    // S1-E-21h f1 — the fourth projectile-sound batch's bank gaps: the
    // Houndius Kill's Item45 (`Projectile.cs:75278`), the Flairon bubble's
    // Item54 (`:72851`), the Crystal Pulse's Item110 (`:73294`), the
    // Xenopopper's Item96 (`:73400`), the Electrosphere's Item94 (`:73530`),
    // the snowball's Item51 (`:74762`) and the barnacle's Item167 (`:75288`).
    ("Item_45", 2),
    ("Item_51", 2),
    ("Item_54", 2),
    ("Item_94", 2),
    ("Item_96", 2),
    ("Item_110", 2),
    ("Item_167", 2),
    ("Item_15", 2),
    ("Item_16", 2),
    ("Item_18", 2),
    ("Item_19", 2),
    ("Item_20", 2),
    ("Item_21", 2),
    ("Item_23", 2),
    ("Item_25", 2),
    ("Item_28", 2),
    ("Item_29", 2),
    ("Item_31", 2),
    ("Item_32", 2),
    ("Item_34", 2),
    ("Item_36", 2),
    // S1-P-i45: the reforge's non-top-tier cue (`SoundID.Item37`,
    // `Main.cs:41931`; `Sounds/Item_37.xnb`).
    ("Item_37", 2),
    // W4-e28 - the Ammo Box's use cue (`SoundID.Item149 = new
    // LegacySoundStyle(2, 149)`, `SoundID.cs:1533`; `Player.cs:32634`'s
    // `PlaySound(SoundID.Item149, base.position)`; `Sounds/Item_149.xnb`).
    ("Item_149", 2),
    // W4-f34 — the Dead Cells Potion Station's use cue (`SoundID.Item179 =
    // new LegacySoundStyle(2, 179).WithVolume(0.6f)`, `SoundID.cs:1563`;
    // `Player.cs:32616`'s `PlaySound(SoundID.Item179, base.position)`;
    // `Sounds/Item_179.xnb`). The 0.6 style volume rides the doors arm's
    // volume_scale (the unpitched e28 seam).
    ("Item_179", 2),
    ("Item_38", 2),
    ("Item_39", 2),
    ("Item_40", 2),
    ("Item_41", 2),
    ("Item_43", 2),
    ("Item_44", 2),
    // S1-E-21h d75 part 4 — the Meowmere star's case-37 variation pair
    // (`LegacySoundPlayer.cs:1326-1336`: `Next(57, 59)` picks the
    // SoundItem slot; `Sounds/Item_57.xnb`/`Item_58.xnb` verified on
    // disk). The 502 row landed with d75 part 5 (`:5732-5742`) — the
    // owed bounce gate is live through `spawn_with_data`.
    ("Item_57", 2),
    ("Item_58", 2),
    ("Item_59", 2),
    ("Item_60", 2),
    ("Item_61", 2),
    // S1-E-21h d76 DOC-SWEEP — the Kill()-tail blast cue (the Jim's Drone /
    // RC Car's `SoundID.Item62`, `Projectile.cs:70865`; the asset is on disk).
    // S1-E-21h f1 fifth batch — the rocket families' Item62
    // (`Kill_Bombs_DoUsualKillCode` `Projectile.cs:77737`/`:77781`/`:77820`/
    // `:77859`/`:77897`/`:77983`/`:78025`/`:78146`, the II-kind / shrapnel /
    // cluster / Rocket-III cues; the `.xnb` asset verified on disk).
    ("Item_62", 2),
    ("Item_63", 2),
    ("Item_64", 2),
    ("Item_66", 2),
    ("Item_67", 2),
    ("Item_69", 2),
    ("Item_71", 2),
    ("Item_72", 2),
    ("Item_73", 2),
    ("Item_75", 2),
    ("Item_76", 2),
    ("Item_77", 2),
    ("Item_78", 2),
    ("Item_79", 2),
    ("Item_80", 2),
    ("Item_81", 2),
    ("Item_82", 2),
    ("Item_83", 2),
    ("Item_84", 2),
    ("Item_85", 2),
    // S1-E-21h f1 final phase — the 1037/1049 Kill arm's Item89
    // (`Projectile.cs:73868`, cue-first before the 192×scale recentre;
    // the rows ride e3's pushed tip `1d26efa9`, the arm is inert until
    // the merge; the `.xnb` asset verified on disk).
    ("Item_89", 2),
    // S1-E-21h f1 final phase — the 1054 SpiderBoulder's NPCHit29
    // (`Projectile.cs:74867`, `SoundID.NPCHit29 = (3, 29)`,
    // `SoundID.cs:1285` — case 3's 20..54 ×0.5 arm + one pitch draw;
    // the row rides d3's pushed tip `2d01693c`, the arm is inert until
    // the merge; the `.xnb` asset verified on disk).
    ("NPC_Hit_29", 3),
    ("Item_88", 2),
    // W4-c14 M2-MOUNT-4 — the Santank gift-box drop cue
    // (`Mount.cs:3300`, `SoundID.Item89` — `Sounds/Item_89.xnb`).
    ("Item_89", 2),
    ("Item_90", 2),
    ("Item_92", 2),
    ("Item_95", 2),
    ("Item_97", 2),
    ("Item_98", 2),
    ("Item_99", 2),
    ("Item_100", 2),
    ("Item_102", 2),
    ("Item_103", 2),
    ("Item_105", 2),
    ("Item_106", 2),
    ("Item_108", 2),
    ("Item_109", 2),
    ("Item_111", 2),
    // S1-E-21h d77 — the Crystal Pulse's death cue (`Projectile.cs:73294`,
    // `SoundID.Item110`; the `.xnb` verified on disk).
    ("Item_110", 2),
    ("Item_113", 2),
    // S1-E-21h d76 — the second projectile-sound wave's bank gaps: the
    // Cannon Explosive's portal pair (`Projectile.cs:54089`/`:54093`), the
    // Phantasmal Bolt's `Next(2)` cue pick (`:55274`/`:55278`), the golf
    // strike's Item126 (`:52467`), the Laser Drill's hum (`:66248`), the
    // whip tip's Item153/193 (`:47732`/`:47728`) and the Sun Dance's
    // Item159 (`:44988`). All `.xnb` assets verified on disk.
    ("Item_114", 2),
    ("Item_115", 2),
    ("Item_124", 2),
    ("Item_125", 2),
    ("Item_126", 2),
    ("Item_132", 2),
    ("Item_153", 2),
    ("Item_159", 2),
    ("Item_193", 2),
    // Item42 IS a legacy id-2 style (`SoundID.cs:1425`) — the SantaNK1
    // missile's latch (`Projectile.cs:55161`, S1-E-21h d76). The landed
    // wave's "trackable" deferral was a misclassification; the Flaming
    // Wood 325's twin (`:55194`) stays as landed (slice A, keep-one).
    ("Item_42", 2),
    ("Item_117", 2),
    ("Item_128", 2),
    ("Item_130", 2),
    // W4-e10 — the buff-station family's cues (`Player.cs:32616`/:32634):
    // the Dead Cells Potion Station 699 plays `SoundID.Item179`, the Ammo
    // Box 287 plays `SoundID.Item149` (`Sounds/Item_179.xnb`/:`Item_149.xnb`).
    ("Item_149", 2),
    ("Item_151", 2),
    ("Item_152", 2),
    ("Item_156", 2),
    ("Item_157", 2),
    ("Item_158", 2),
    ("Item_174", 2),
    // W4-e10 — the Dead Cells Potion Station 699's cue (`Player.cs:32616`,
    // `SoundID.Item179`; `Sounds/Item_179.xnb`, volume 0.6).
    ("Item_179", 2),
    ("Item_181", 2),
    ("Item_183", 2),
    ("Item_184", 2),
    ("Item_185", 2),
    ("Item_186", 2),
    ("Item_187", 2),
    ("Item_188", 2),
    ("Item_196", 2),
    ("Item_210", 2),
    // W4-f2 closeout (e9b tail union, research/94 §10) — the NPC AI sites'
    // Item cues (`SoundID.Item167/170/172` = `LegacySoundStyle(2, N)`,
    // `SoundID.cs:1551-1556`; `Item150` `:1534`): the Queen Slime's landing
    // thud (`NPC.cs:45558`), the Blood Nautilus' volley/scream
    // (`:47263`/`:47268`/`:47293`) and the Big Mimic's projectile-reflect
    // tink (`:66429`).

    ("Item_170", 2),
    ("Item_172", 2),
    ("Item_150", 2),
    // `SoundID.Roar` = 15 (`SoundID.cs`) — the boss-summon roar
    // (`Player.cs:43765`'s `SoundEngine.PlaySound(15, ...)`; the `Roar_0`
    // cue family).
    ("Roar_0", 15),
    // `LegacySoundPlayer` case 43 (`LegacySoundPlayer.cs:454-476`) — the
    // thunder strike: `SoundThunder[Next(6)]` from `Sounds/Thunder_0..5`,
    // played by `UpdateWeather` when the thunder delay hits 0
    // (`Main.cs:58759-58761`). Ambient type: the ambient-volume slider.
    ("Thunder_0", 43),
    ("Thunder_1", 43),
    ("Thunder_2", 43),
    ("Thunder_3", 43),
    ("Thunder_4", 43),
    ("Thunder_5", 43),
    // `LegacySoundPlayer` case 19 (`:871-911`) — the splash family; the
    // liquid-merge shimmer arm plays `SoundSplash[4]` (`WorldGen.cs:4644`).
    // S1-E-21f — the player entry/exit splashes (`Player.cs:27806`/`:27819`/
    // `:27832`/`:27846` + the `:27900-27940` exit twins): styles 0 (water),
    // 1 (honey/lava, the default Style 1), 2/3 (shimmer, ×0.75).
    ("Splash_0", 19),
    ("Splash_1", 19),
    ("Splash_2", 19),
    ("Splash_3", 19),
    ("Splash_4", 19),
    // S1-E-21f — slot 5 completes the `SoundSplash[6]` bank
    // (`LegacySoundPlayer.cs:206-210`); case 19's 4/5 cross-suppression
    // (`:892-909`) gates the returned instance only — the port spawns a
    // fresh source (the standing D4 deviation), so both slots play.
    ("Splash_5", 19),
    // The zombie bank's critter slots (`LegacySoundPlayer.cs:214-222`'s
    // `Sounds/Zombie_ + k`, the 131-entry bank `:112`) behind the critter
    // table's legacy ids (S1-E-25: `NPC.UpdateNPC_CritterSounds`): the sound
    // type is the legacy id the case resolves through (30/31/32/33/44/46/
    // 67/68/69 — the ambient-scaling list `:1444-1460`).
    ("Zombie_10", 30),
    ("Zombie_11", 30),
    ("Zombie_12", 30),
    ("Zombie_13", 31),
    ("Zombie_14", 32),
    ("Zombie_15", 33),
    ("Zombie_16", 32),
    ("Zombie_17", 32),
    ("Zombie_18", 32),
    ("Zombie_19", 32),
    ("Zombie_106", 44),
    ("Zombie_107", 44),
    ("Zombie_108", 44),
    ("Zombie_110", 46),
    ("Zombie_111", 46),
    ("Zombie_112", 46),
    ("Zombie_113", 46),
    ("Zombie_114", 46),
    ("Zombie_118", 67),
    ("Zombie_119", 67),
    ("Zombie_120", 67),
    ("Zombie_126", 68),
    ("Zombie_127", 68),
    ("Zombie_128", 68),
    ("Zombie_129", 69),
    ("Zombie_130", 69),
    // S1-E-21e — the fighter-moan slots (`AI_003_Fighters`,
    // `NPC.cs:57340-57367`): case 14's default `Next(3)` (`:817`), the
    // style-542 slot 7 (`:796`) and the 489/586 `Next(21, 24)` (`:808`);
    // case 26's `Next(3, 5)` (`:971`); case 29's Style-is-the-slot rows
    // 6/7/8 (`:1045`) and the full per-style switch's named slots (the
    // later waves' callers); case 45's pinned slot 109 (the duck quack,
    // `NPC.cs:23678-23681`, `:1060`). Cases 14/26/29 are NOT on the
    // ambient-scaling list (`LegacySoundPlayer.cs:1444-1457`) — they ride
    // `Main.soundVolume`; case 45 does scale.
    ("Zombie_0", 14),
    ("Zombie_1", 14),
    ("Zombie_2", 14),
    ("Zombie_3", 26),
    ("Zombie_4", 26),
    ("Zombie_5", 29),
    ("Zombie_6", 29),
    ("Zombie_7", 29),
    ("Zombie_8", 29),
    // S1-E-21h d75 part 3 — the Phantasmal Deathray's first-tick cue
    // (`Projectile.cs:31504`, `PlaySound(29, x, y, 104)`): case 29's
    // Style-is-the-slot row 104 (the ×0.55 volume arm,
    // `LegacySoundPlayer.cs:1029-1032`), the `Sounds/Zombie_104.xnb` asset
    // verified on disk.
    ("Zombie_104", 29),
    ("Zombie_21", 14),
    ("Zombie_22", 14),
    ("Zombie_23", 14),
    // W4-f2 - the IdleSounds case-29 slots (NPC.cs:90081-90205, the
    // Style-is-the-slot rows 24-87, LegacySoundPlayer.cs:1009-1011's
    // x0.5 band): the full switch S1-E-21e ported 'for the later waves'
    // callers' - this wave wires them. Case 29 is NOT ambient-scaling.
    ("Zombie_24", 29),
    ("Zombie_25", 29),
    ("Zombie_26", 29),
    ("Zombie_27", 29),
    ("Zombie_28", 29),
    ("Zombie_29", 29),
    ("Zombie_30", 29),
    ("Zombie_31", 29),
    ("Zombie_32", 29),
    ("Zombie_33", 29),
    ("Zombie_34", 29),
    ("Zombie_35", 29),
    ("Zombie_36", 29),
    ("Zombie_37", 29),
    ("Zombie_38", 29),
    ("Zombie_39", 29),
    ("Zombie_40", 29),
    ("Zombie_41", 29),
    ("Zombie_42", 29),
    ("Zombie_43", 29),
    ("Zombie_44", 29),
    ("Zombie_45", 29),
    ("Zombie_46", 29),
    ("Zombie_47", 29),
    ("Zombie_48", 29),
    ("Zombie_49", 29),
    ("Zombie_50", 29),
    ("Zombie_51", 29),
    ("Zombie_52", 29),
    ("Zombie_53", 29),
    ("Zombie_54", 29),
    ("Zombie_55", 29),
    ("Zombie_56", 29),
    ("Zombie_57", 29),
    ("Zombie_58", 29),
    ("Zombie_59", 29),
    ("Zombie_60", 29),
    ("Zombie_61", 29),
    ("Zombie_62", 29),
    ("Zombie_63", 29),
    ("Zombie_64", 29),
    ("Zombie_65", 29),
    ("Zombie_66", 29),
    ("Zombie_67", 29),
    ("Zombie_68", 29),
    ("Zombie_69", 29),
    ("Zombie_70", 29),
    ("Zombie_71", 29),
    ("Zombie_72", 29),
    ("Zombie_73", 29),
    ("Zombie_74", 29),
    ("Zombie_75", 29),
    ("Zombie_76", 29),
    ("Zombie_77", 29),
    ("Zombie_78", 29),
    ("Zombie_79", 29),
    ("Zombie_80", 29),
    ("Zombie_81", 29),
    ("Zombie_82", 29),
    ("Zombie_83", 29),
    ("Zombie_84", 29),
    ("Zombie_85", 29),
    ("Zombie_86", 29),
    ("Zombie_87", 29),
    // W4-f2 closeout (e9b tail union, research/94 §10) — the Queen Slime's
    // idle moan bank (case 65, `LegacySoundPlayer.cs:1382-1402`:
    // `Next(115, 118)` behind the 115/116/117 playing guards, ×0.5, NO
    // pitch draw — the `NPC.cs:45759` site).
    ("Zombie_115", 65),
    ("Zombie_116", 65),
    ("Zombie_117", 65),
    ("Zombie_109", 45),
    // S1-E-21h d77 deferred-arm wave — the Phantasmal Eye's death moan
    // (`Projectile.cs:72989`, `PlaySound(29, x, y, 103)`): case 29's
    // Style-is-the-slot row 103, the ×0.4 arm (`LegacySoundPlayer.cs:1025-
    // 1028`); the asset is on disk (`Sounds/Zombie_103.xnb`).
    ("Zombie_103", 29),

];

/// The `Sounds/Custom/` NPC cues — the DD2 families' hit/death sounds and
/// the Deerclops' (`SoundID.CreateTrackable`, `SoundID.cs:1627+/:1691`).
///
/// The C#'s trackable styles pick a random variation
/// (`LegacySoundStyle.Style`, `LegacySoundStyle.cs:33-38`); the port plays
/// the first one (the `_0` file) — the same asset family, a fixed member.
pub(crate) const NPC_CUSTOM_CUES: &[&str] = &[
    "dd2_betsy_death_0",
    "dd2_betsy_hurt_0",
    // W4-c3 (the style-10 family): the Water Bolt Mimic's HitSound —
    // `SoundID.DD2_BookStaffCast = CreateTrackable("dd2_book_staff_cast",
    // 3)` (`SoundID.cs:1676`, `NPC.cs:17745`); the `_0` pin, the DD2
    // families' convention.
    "dd2_book_staff_cast_0",
    "dd2_crystal_cart_impact_0",
    "dd2_dark_mage_death_0",
    "dd2_dark_mage_hurt_0",
    "dd2_drakin_death_0",
    "dd2_drakin_hurt_0",
    "dd2_goblin_bomber_death_0",
    "dd2_goblin_bomber_hurt_0",
    "dd2_goblin_death_0",
    "dd2_goblin_hurt_0",
    "dd2_javelin_throwers_death_0",
    "dd2_javelin_throwers_hurt_0",
    "dd2_kobold_death_0",
    "dd2_kobold_flyer_death_0",
    "dd2_kobold_flyer_hurt_0",
    "dd2_kobold_hurt_0",
    // W4-d42 - the lightning bug's trackables (`SoundID.cs:1691+`).
    "dd2_lightning_bug_hurt_0",
    // W4-d42 - the lightning bug's trackables (`SoundID.cs:1691+`).
    "dd2_lightning_bug_death_0",
    "dd2_ogre_death_0",
    "dd2_ogre_hurt_0",
    "dd2_skeleton_death_0",
    "dd2_skeleton_hurt_0",
    "dd2_wither_beast_death_0",
    "dd2_wither_beast_hurt_0",
    "dd2_wyvern_death_0",
    "dd2_wyvern_hurt_0",
    "deerclops_death",
    "deerclops_hit_0",
    // S1-E-21g — the EoW head's dig rumble (`SoundID.EOWDiggin`,
    // `SoundID.cs:1763`'s `CreateTrackable("eow_dig")`, one variation).
    "eow_dig",
    "pal_cattiva_pain",
    "pal_foxparks_pain",
    // W4-f2 closeout (e9b tail union, research/94 §10) — the Pals'
    // pain/joy cries (`AI_127_Pal`, `NPC.cs:42983`/`:42987`/`:42997`/
    // `:43001`; `SoundID.cs:1714-1721`'s `CreateTrackable(...)
    // .WithVolume(0.35f)`, one variation each) and the Statue Mimic's
    // scare/jump/laugh (`AI_126_StatueMimic`, `NPC.cs:43562`/`:43569`/
    // `:43580`/`:43608`; `SoundID.cs:1741-1743`, the laugh three
    // variations — the `_0` pin like wave A).
    "pal_cattiva_joy",
    "pal_foxparks_joy",
    "statuemimic_scare",
    "statuemimic_jump",
    "statuemimic_laugh_0",
    // S1-E-21h f1 final phase — the 1047 RainbowBoulder's Kill cue
    // (`Projectile.cs:71194`, `SoundID.RainbowBoulder =
    // CreateTrackable("rainbow_boulder")`, `SoundID.cs:1709`, one
    // variation; the row rides d3's pushed tip `2d01693c`, the cue is
    // inert until the merge; `Sounds/Custom/rainbow_boulder.xnb`
    // verified on disk).
    "rainbow_boulder",
];

/// The `Sounds/Custom/` trackable cues the liquid-merge events play
/// (`SoundID.cs:1600-1602`'s `CreateTrackable(name, 3, SoundType.Ambient)` —
/// three variations each, S1-E-25's `WorldGen.PlayLiquidChangeSound`).
const LIQUID_CHANGE_CUES: &[&str] = &[
    "liquids_water_lava_0",
    "liquids_water_lava_1",
    "liquids_water_lava_2",
    "liquids_honey_water_0",
    "liquids_honey_water_1",
    "liquids_honey_water_2",
    "liquids_honey_lava_0",
    "liquids_honey_lava_1",
    "liquids_honey_lava_2",
];

/// The trackable cues the tracked-sound layer plays by name
/// (`Sounds/Custom/<cue>.xnb`, `LegacySoundPlayer.cs:257`; S1-E-23): the
/// SceneState blizzard loops (`SoundID.cs:1598-1599`) — the styles carry
/// their own volume (`ambient_sound::blizzard_strong_loop_style`).
const TRACKED_CUES: &[&str] = &[
    "blizzard_strong_loop",
    "blizzard_inside_building_loop",
    // `SoundID.AchievementComplete` (`SoundID.cs:1597`) — the unlock toast's
    // cue (`AchievementInitializer.cs:731-734`, S1-E-26 wave A).
    "achievement_complete",
    // `SoundID.MenuAccept` (`SoundID.cs:1710`, `CreateTrackable("menu_accept")`)
    // — the secret-seed unlock cue (`WorldGen.SecretSeed.Enable`,
    // `WorldGen.cs:539`, W4-g24).
    "menu_accept",
    // W4-f12 — `SoundID.Hungry` (`SoundID.cs:1711`,
    // `CreateTrackable("hungry")`): the 333-expiry cue the `UpdateBuffs`
    // frame's hunger ladder plays (`Player.cs:12138`).
    "hungry",
];

/// The `Sounds/Custom/` trackables the item-use wave names
/// (`Item.UseSound` → `SoundID.CreateTrackable`, `SoundID.cs:1607-1729`;
/// S1-E-21 wave A). Like [`NPC_CUSTOM_CUES`] the port pins the first
/// variation (`_0`) of the multi-variation families — the variation pick
/// rides `LegacySoundStyle.Random` in the C#, an audio-only stream.
pub(crate) const ITEM_CUSTOM_CUES: &[&str] = &[
    // S1-P-i45: the reforge's top-tier cue (`SoundID.BestReforge`,
    // `Main.cs:41909`; `Sounds/Custom/best_reforge.xnb`, one variation).
    "best_reforge",
    "dd2_defense_tower_spawn",
    "dd2_betsy_fireball_shot_0",
    "dd2_book_staff_cast_0",
    "dd2_ghastly_glaive_pierce_0",
    "dd2_monk_staff_swing_0",
    "dd2_sonic_boom_blade_slash_0",
    "dd2_sky_dragons_fury_swing_0",
    "abigail_summon",
    "pal_summon",
    "deadcells_barrel_launcher_fire",
    "deadcells_mushroom_summon",
    // S1-E-21h d75 part 2 — the bobber's sonar bite cue
    // (`SoundID.SonarPotion = CreateTrackable("sonar_potion")
    // .WithVolume(0.65f).WithPitchVariance(0.03f)`, `SoundID.cs:1740`;
    // `Projectile.cs:20751`/:20765 — `Sounds/Custom/sonar_potion.xnb`,
    // one variation).
    "sonar_potion",
    // S1-E-21h d75 part 4 — the 1047 RainbowBoulder's hard-bounce cue
    // (`SoundID.RainbowBoulder = CreateTrackable("rainbow_boulder")`,
    // `SoundID.cs:1709`; `Projectile.cs:20104` —
    // `Sounds/Custom/rainbow_boulder.xnb`, one variation). The bounce
    // arm rides d3's `rainbow_check_bounce` (`wants_sound` (42, 274));
    // this bank row makes the intent audible after the merge.
    // W4-g15 lane 5 — `SoundID.RainbowBoulder = CreateTrackable("rainbow_boulder")`
    // (`SoundID.cs:1709`, one variation): the Heroicis-set spawn cue
    // (`ParticleOrchestrator.Spawn_HeroicisSetSpawnSound`,
    // `ParticleOrchestrator.cs:897-900`, the `TryGrantingMakeAWishSet` grant).
    "rainbow_boulder",
];

/// `SoundID.MenuAccept` (`SoundID.cs:1068`/:1710,
/// `CreateTrackable("menu_accept")` — one variation, volume 1, no pitch
/// variance, `SoundType.Sound`, uncapped): the tracked style
/// `WorldGen.SecretSeed.Enable` plays (`WorldGen.cs:537-540`, W4-g24) when
/// a secret seed unlocks with `playSound: true`.
pub fn menu_accept_style() -> crate::tracked_sound::TrackedStyle {
    crate::tracked_sound::TrackedStyle {
        cue: "menu_accept",
        variations: 1,
        volume: 1.0,
        pitch_variance: 0.0,
        sound_type: crate::tracked_sound::SoundType::Sound,
        max_tracked_instances: 0,
    }
}

/// S1-P-i41 — the `Sounds/Custom/` trackables the mount system plays:
/// `SoundID.PalChillet = CreateTrackable("pal_chillet").WithVolume(0.35f)`
/// (`SoundID.cs:1716`), the Chillemt mounts' equip cue (`Mount.cs:6285`).
/// One variation, pinned like the item-use trackables. W4-g10 FINAL adds
/// `PalChilletJoy` (`SoundID.cs:1717`, the petting joy cue
/// `Player.cs:33354`) — same 0.35 volume, one variation.
pub(crate) const MOUNT_CUSTOM_CUES: &[&str] = &["pal_chillet", "pal_chillet_joy"];

/// W4-d21 fold 2 — the `Sounds/Custom/` trackables the projectile AI arms
/// play: `SoundID.PalDigtoiseJoy = CreateTrackable("pal_digtoise_joy")
/// .WithVolume(0.35f)` (`SoundID.cs:1724`), the digtoise's state-5 petted
/// joy (`Projectile.cs:38517`). One variation, pinned. The other Pal cues
/// (`pal_digtoise` `:1723`, `pal_summon`, the foxsparks/cattiva joys) ride
/// their machines' follow-ups — the 204's idle cues are the documented
/// sound-body skip, the 1093/1094/1112/1113/1118 AI machines are unported.
pub(crate) const PROJ_CUSTOM_CUES: &[&str] = &["pal_digtoise_joy"];

/// W4-d85 FINAL — the `Sounds/Custom/` trackables the UI layer plays:
/// `SoundID.DD2_ExplosiveTrapExplode` (`SoundID.cs:1604`'s
/// `CreateTrackable("dd2_explosive_trap_explode", 3)`), the issue-report
/// indicator's notify cue (`IssueReportsIndicator.AttemptLettingPlayerKnow`,
/// `IssueReportsIndicator.cs:24`). The legacy id-42 play picks a variation
/// (`LegacySoundStyle.Style`, `LegacySoundStyle.cs:24-27` — the audio-only
/// `Random` stream), so all three assets register (the liquid-change
/// precedent, not the item-use first-variation pin).
pub(crate) const UI_CUSTOM_CUES: &[&str] = &[
    "dd2_explosive_trap_explode_0",
    "dd2_explosive_trap_explode_1",
    "dd2_explosive_trap_explode_2",
];

/// W4-e10 — the `Sounds/Custom/` trackables the player-voice system plays
/// (`Player.PlayHurtSound` `Player.cs:38569-38712` / `PlayDeathSound`
/// `:38715-38746`): the DST hurt pair, the default hurt, the animal/retro
/// hurt family and the retro/balloon death cues (`SoundID.cs`'s
/// `CreateTrackable` rows, the generated `soundid.rs` `trackable` field).
/// Like [`NPC_CUSTOM_CUES`] the port pins the first variation (`_0`) of the
/// multi-variation DST families; the single-variation cues play bare.
pub(crate) const PLAYER_VOICE_CUES: &[&str] = &[
    "dst_male_hit_0",
    "dst_female_hit_0",
    "player_hit_default",
    "player_hit_chicken",
    "player_hit_chicken_rare",
    "player_hit_frog",
    "player_hit_goat",
    "player_hit_retro",
    "player_hit_cat",
    "player_hit_dog",
    "player_hit_turkey",
    "player_hit_goblin",
    "player_hit_crow",
    "player_hit_balloon",
    "player_hit_undead",
    "player_hit_vampire",
    "player_hit_fairy",
    "player_death_retro",
    // W4-e10 — the hunger transition's cue (`SoundID.Hungry`,
    // `SoundID.cs`'s `CreateTrackable("hungry")`, one variation):
    // `UpdateHungerBuffs`' 333-expiry arm (`Player.cs:12138`).
    "hungry",
];

/// Whether the cue name is in the SFX bank (the item-use coverage test's
/// seam, S1-E-21 wave A).
pub(crate) fn sfx_cue_registered(cue: &str) -> bool {
    SFX_CUES.iter().any(|(n, _)| *n == cue)
}

/// `SoundID.MenuAccept` (`SoundID.cs:1068`/:1710,
/// `CreateTrackable("menu_accept")` — one variation, volume 1, no pitch
/// variance, `SoundType.Sound`, uncapped): the tracked style the quit-path
/// witnesses register their pool entry through (the W4-g24 doc-sweep seam,
/// kept by the W4-g73 quit-stop keep-one; g24's `WorldGen.SecretSeed.Enable`
/// caller rides the g24 branch, so the default build has no production
/// reader here — the allow keeps it warning-clean until that merge).
#[allow(dead_code)]
pub fn menu_accept_style() -> crate::tracked_sound::TrackedStyle {
    crate::tracked_sound::TrackedStyle {
        cue: "menu_accept",
        variations: 1,
        volume: 1.0,
        pitch_variance: 0.0,
        sound_type: crate::tracked_sound::SoundType::Sound,
        max_tracked_instances: 0,
    }
}

/// S1-E-21 wave C — resolve an NPC hit/death cue name to its
/// `LegacySoundStyle` row (`crate::soundid::SOUND_STYLES`): the legacy
/// `NPC_Hit_N`/`NPC_Killed_N` asset maps to `NPCHitN`/`NPCDeathN`
/// (`SoundID.cs`), the `Sounds/Custom/` trackable pins (`dd2_*_0`,
/// `deerclops_hit_0`) map back to their style through the `trackable`
/// field. `None` = an unregistered name (a bank gap).
pub(crate) fn npc_sound_style(
    cue: &str,
    prefix: &str,
    sound_id: u8,
) -> Option<&'static crate::soundid::SoundStyleDef> {
    if let Some(n) = cue.strip_prefix(prefix) {
        let style: i32 = n.parse().ok()?;
        return crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.sound_id == sound_id && s.style == style);
    }
    let base = cue.strip_suffix("_0").unwrap_or(cue);
    crate::soundid::SOUND_STYLES
        .iter()
        .find(|s| s.sound_id == 42 && s.trackable == Some(base))
}

fn sound_type(cue: &str) -> u32 {
    SFX_CUES
        .iter()
        .find(|(n, _)| *n == cue)
        .map(|(_, t)| *t)
        // The liquid-change trackables ride the legacy path through the
        // trackable case (`SoundEngine.PlaySound(style)` →
        // `LegacySoundPlayer.PlaySound(style.SoundId = 42, …)`), whose
        // ambient-scaling switch treats 42 like any unlisted id.
        .or_else(|| {
        (LIQUID_CHANGE_CUES.contains(&cue) || NPC_CUSTOM_CUES.contains(&cue) || ITEM_CUSTOM_CUES.contains(&cue) || MOUNT_CUSTOM_CUES.contains(&cue) || PROJ_CUSTOM_CUES.contains(&cue) || UI_CUSTOM_CUES.contains(&cue)).then_some(42)
        (LIQUID_CHANGE_CUES.contains(&cue) || NPC_CUSTOM_CUES.contains(&cue) || ITEM_CUSTOM_CUES.contains(&cue) || MOUNT_CUSTOM_CUES.contains(&cue) || PLAYER_VOICE_CUES.contains(&cue)).then_some(42)
        (LIQUID_CHANGE_CUES.contains(&cue) || NPC_CUSTOM_CUES.contains(&cue) || ITEM_CUSTOM_CUES.contains(&cue) || MOUNT_CUSTOM_CUES.contains(&cue) || TRACKED_CUES.contains(&cue) || UI_CUSTOM_CUES.contains(&cue)).then_some(42)
    })
        .unwrap_or(0)
}

/// One prepared cue play — the shared tail of the slot-tracked paths
/// ([`Audio::play_thunder`], [`Audio::play_critter`], the case-19 splash):
/// the attenuation's final volume/pan plus the decoded PCM, resampled for
/// the instance pitch.
struct Prepared {
    channels: u16,
    rate: u32,
    /// The PCM frames, already resampled for the pitch.
    samples: Vec<i16>,
    volume: f32,
    pan: f32,
}

/// The XNA `SoundEffectInstance.Pitch` (octaves) as a playback-rate resample:
/// the instance frequency scales by `2^pitch`, so the buffer is re-stepped
/// at that rate — frame-aligned (each channel group stays intact), nearest
/// frame, no interpolation (the legacy engine skips/repeats frames).
/// `ratio > 1` is a higher pitch (a shorter buffer).
fn resample_i16(samples: &[i16], channels: u16, ratio: f32) -> Vec<i16> {
    let ch = channels.max(1) as usize;
    let frames = samples.len() / ch;
    let out_frames = ((frames as f32) / ratio) as usize;
    let mut out = Vec::with_capacity(out_frames * ch);
    for f in 0..out_frames {
        let src = (((f as f32) * ratio) as usize).min(frames.saturating_sub(1));
        out.extend_from_slice(&samples[src * ch..(src + 1) * ch]);
    }
    out
}

pub struct Audio {
    _stream: Option<rodio::OutputStream>,
    handle: Option<rodio::OutputStreamHandle>,
    cues: HashMap<&'static str, (u16, u32, Vec<i16>)>,
    /// The two `SoundInstanceLiquid[n]` stand-ins (`LegacySoundPlayer.cs:322`):
    /// 0 = the waterfall, 1 = the lavafall. `Some` while the loop plays.
    liquid_loops: [Option<rodio::Sink>; 2],
    /// The live pan gains the playing liquid-loop sources read (the XNA
    /// `Pan` write reaches the instance mid-play, `:1267`/`:1297`).
    liquid_pans: [Option<LiquidPanGains>; 2],
    /// The `StopWaterfallAmbienceAudio` call count (a test seam — the
    /// headless Apps have no liquid sink, so the quit flows assert the
    /// sentinel-stop pair through this; the W4-g24/g73 quit-stop keep-one).
    /// sentinel-stop pair through this, W4-g24 doc-sweep).
    liquid_ambience_stops: u64,
    /// The decoded SceneState ambient loops (`blizzard_strong_loop` /
    /// `blizzard_inside_building_loop`, from `Sounds/Custom/<name>.xnb` —
    /// the trackable path, `LegacySoundPlayer.cs:257`). The blizzard loops
    /// play through the tracked layer (S1-E-23) — the styles are
    /// `ambient_sound::blizzard_*_style` and the slots live on the port's
    /// `SceneState`.
    tracked: crate::tracked_sound::SoundPlayer,
    /// `SoundStyle._random` (`SoundStyle.cs:8-9`): the static stream
    /// `GetRandomPitch` draws from (`:40-43`) — distinct from
    /// [`Audio::style_random`] (`LegacySoundStyle.Random`). The C# seeds
    /// both with `Environment.TickCount` (unobservable); the port seeds a
    /// fixed 1 (ANALYSIS, S1-E-23).
    sound_style_random: tr_core::unified_random::UnifiedRandom,
    /// The decoded `Sounds/Liquid_{0,1}` loops (channels, rate, samples).
    liquid_cues: [Option<(u16, u32, Vec<i16>)>; 2],
    /// Per-frame `Main.Camera.Center` + the volume sliders (see [`crate::sound`]).
    listener: Listener,
    /// The played-cue counter (a test seam — the headless Apps have no output
    /// device, so the bug/UI flows assert the click sounds through this
    /// instead of a real playback).
    played: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// `LegacySoundStyle.Random` (`LegacySoundStyle.cs:9`): the static
    /// `UnifiedRandom` the trackable styles' `Style` getter draws its
    /// variation pick from (`:21-28`, `Next(_style, _style + Variations)`).
    /// The C# seeds it with `Environment.TickCount` (an unobservable stream);
    /// the port seeds a fixed 1 (ANALYSIS, S1-E-25).
    style_random: tr_core::unified_random::UnifiedRandom,
    /// The `SoundInstanceNpcHit[59]` slots (`LegacySoundPlayer.cs:74`): the
    /// live instance per family entry — case 3's playing guard reads 57
    /// (`:651`) and the play replaces the picked slot's instance (`:663-667`'s
    /// Stop + CreateInstance), S1-E-21 wave C.
    npc_hit_slots: [Option<rodio::Sink>; 59],
    /// The `SoundInstanceNpcKilled[]` slots (`LegacySoundPlayer.cs:78`,
    /// `SoundID.NPCDeathCount` long) — case 4's playing guard reads 10
    /// (`:686`); the play overwrites the slot WITHOUT a Stop (`:690`).
    npc_killed_slots: HashMap<usize, rodio::Sink>,
    /// The `SoundInstanceThunder[6]` slots (`LegacySoundPlayer.cs:113`): the
    /// live instance per family entry — case 43's re-roll loop reads them
    /// (`:456-464`) and the play replaces the picked slot's instance
    /// (`:466-476`'s Stop + CreateInstance).
    thunder_slots: [Option<rodio::Sink>; 6],
    /// The `SoundInstanceSplash[6]` slots (`LegacySoundPlayer.cs:120`) —
    /// case 19's playing guard (`:872-874`).
    splash_slots: [Option<rodio::Sink>; 6],
    /// The `SoundInstanceZombie[131]` slots (`LegacySoundPlayer.cs:114`) the
    /// critter family guards and overwrites — a map over the slots the
    /// critter table can name (`:10-19`, `:106-108`, `:110-114`, `:118-120`,
    /// `:126-130`). An overwritten sink is detached, not stopped: case 44
    /// replaces the array entry while the old instance keeps playing
    /// (`:1051-1062`).
    zombie_slots: HashMap<usize, rodio::Sink>,
    /// The `SoundInstanceMech[1]` slot (`LegacySoundPlayer.cs:24`): case
    /// 28's playing guard reads it (`:998-1001`) and the play overwrites
    /// the entry WITHOUT a Stop (`:1002`) — the wiring switch click,
    /// S1-E-21i.
    mech_slots: [Option<rodio::Sink>; 1],
    /// The `SoundInstanceRoar[3]` slots (`LegacySoundPlayer.cs:118`/`:326-329`,
    /// `Sounds/Roar_0..2` `:227-230`): case 15's null-or-Stopped guard reads
    /// them (`:834`) and replaces the picked slot on a fresh play (`:836`);
    /// case 36 overwrites WITHOUT any guard (`:1316`). S1-E-21g.
    roar_slots: [Option<rodio::Sink>; 3],
    /// The `SoundInstancePixie` slot (`LegacySoundPlayer.cs:129`/`:178`):
    /// case 27's playing branch RE-DRAWS the pitch on the live instance
    /// and returns null (`:980-986`) — the draw is consumed either way;
    /// a fresh play stops the old instance (`:987-990`). The wing-hum
    /// loop stand-in, S1-E-21g (the Pixie 75's `Next(40)` ambience arm,
    /// `NPC.cs:24853-24856`); no ported Pixie kind yet — the seam is
    /// table-complete for the future kind.
    pixie_slot: Option<rodio::Sink>,
    /// W4-c14 doc-sweep — the `SoundInstanceItem[SoundID.ItemSoundCount]`
    /// slots (`LegacySoundPlayer.cs:72`, 211 entries `SoundID.cs:1595`):
    /// case 2's 53/55/159 playing guards read them (`:559`/`:583`) and a
    /// fresh play stops + re-creates the slot (`:590-593`/`:607`); the
    /// stack-allowed styles (`:595-605`) never write a slot. A map over the
    /// styles the port plays (the `npc_killed_slots` precedent).
    item_slots: HashMap<usize, rodio::Sink>,
}

impl Audio {
    pub fn new(content: Option<&Path>) -> Self {
        let mut cues = HashMap::new();
        let mut stream = None;
        let mut handle = None;
        if let Some(dir) = content.map(|c| c.join("Sounds")) {
            match rodio::OutputStream::try_default() {
                Ok((s, h)) => {
                    stream = Some(s);
                    handle = Some(h.clone());
                    // The music system shares this output stream.
                    MusicSystem::set_handle_from_system(h);
                }
                Err(e) => log::warn!("audio: no output device ({e})"),
            }
            for (name, _) in SFX_CUES {
                let path = dir.join(format!("{name}.xnb"));
                match load_sound(&path) {
                    Ok(s) => {
                        let samples = s
                            .data
                            .chunks_exact(2)
                            .map(|b| i16::from_le_bytes([b[0], b[1]]))
                            .collect();
                        cues.insert(*name, (s.channels, s.sample_rate, samples));
                    }
                    Err(e) => log::debug!("sound {name}: {e:#}"),
                }
            }
            // The liquid-change trackables + the tracked layer's cues:
            // `Sounds/Custom/<name>[_<n>].xnb` (`LegacySoundPlayer.cs:257`'s
            // trackable path, S1-E-25/S1-E-23).
            if let Some(custom) = content.map(|c| c.join("Sounds").join("Custom")) {
                for name in LIQUID_CHANGE_CUES
                    .iter()
                    .chain(TRACKED_CUES.iter())
                    .chain(NPC_CUSTOM_CUES.iter())
                    .chain(ITEM_CUSTOM_CUES.iter())
                    .chain(MOUNT_CUSTOM_CUES.iter())
                    .chain(PROJ_CUSTOM_CUES.iter())
                    .chain(UI_CUSTOM_CUES.iter())
                    .chain(PLAYER_VOICE_CUES.iter())
                {
                    let path = custom.join(format!("{name}.xnb"));
                    match load_sound(&path) {
                        Ok(s) => {
                            let samples = s
                                .data
                                .chunks_exact(2)
                                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                                .collect();
                            cues.insert(*name, (s.channels, s.sample_rate, samples));
                        }
                        Err(e) => log::debug!("trackable cue {name}: {e:#}"),
                    }
                }
            }
        }
        // `LegacySoundPlayer.cs:223-225`: `Sounds/Liquid_{0,1}` — the
        // waterfall and lavafall loop assets.
        let mut liquid_cues: [Option<(u16, u32, Vec<i16>)>; 2] = [None, None];
        let sounds_dir = content.map(|c| c.join("Sounds"));
        for (n, slot) in liquid_cues.iter_mut().enumerate() {
            let Some(sounds_dir) = &sounds_dir else { break };
            let path = sounds_dir.join(format!("Sounds/Liquid_{n}.xnb"));
            match load_sound(&path) {
                Ok(snd) => {
                    let samples = snd
                        .data
                        .chunks_exact(2)
                        .map(|b| i16::from_le_bytes([b[0], b[1]]))
                        .collect();
                    *slot = Some((snd.channels, snd.sample_rate, samples));
                }
                Err(e) => log::debug!("liquid loop {n}: {e:#}"),
            }
        }
        log::info!("audio: {} PCM cues loaded", cues.len());
        Self {
            _stream: stream,
            handle,
            cues,
            played: Default::default(),
            liquid_loops: [None, None],
            liquid_pans: [None, None],
            liquid_ambience_stops: 0,
            tracked: crate::tracked_sound::SoundPlayer::default(),
            sound_style_random: tr_core::unified_random::UnifiedRandom::new(1),
            liquid_cues,
            listener: Listener {
                camera_center: (0.0, 0.0),
                sound_volume: 1.0,
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
                generating_or_loading: false,
                server: false,
            },
            style_random: tr_core::unified_random::UnifiedRandom::new(1),
            npc_hit_slots: std::array::from_fn(|_| None),
            npc_killed_slots: HashMap::new(),
            thunder_slots: [None, None, None, None, None, None],
            splash_slots: [None, None, None, None, None, None],
            zombie_slots: HashMap::new(),
            mech_slots: [None],
            roar_slots: [None, None, None],
            pixie_slot: None,
            item_slots: HashMap::new(),
        }
    }

    /// Update the listener state the attenuation reads (`Main.Camera.Center`,
    /// `Main.soundVolume`/`Main.ambientVolume`, `Main.gameMenu`).
    pub fn set_listener(&mut self, listener: Listener) {
        self.listener = listener;
    }

    /// A 2D sound (`PlaySound(type)`, no position → no attenuation).
    pub fn play(&self, cue: &str) {
        self.play_at(cue, -1.0, -1.0, 1.0);
    }

    /// A positioned sound (`PlaySound(type, x, y, volumeScale)`), attenuated by
    /// [`crate::sound::play_sound`] against the current listener.
    /// The played-cue count (the test seam — the headless Apps have no output
    /// device, so the UI flows assert their cues through this).
    pub fn played_count(&self) -> u64 {
        self.played.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn play_at(&self, cue: &str, x: f32, y: f32, volume_scale: f32) {
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some((channels, rate, samples)) = self.cues.get(cue) else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let Some(playback) = sound::play_sound(sound_type(cue), x, y, self.listener, volume_scale)
        else {
            return;
        };
        let (left, right) = sound::pan_gains(playback.pan);
        let buf = rodio::buffer::SamplesBuffer::new(*channels, *rate, samples.clone());
        let source = Panned {
            inner: buf.convert_samples::<f32>(),
            channels: *channels,
            left: playback.volume * left,
            right: playback.volume * right,
            channel: 0,
        };
        let _ = handle.play_raw(source);
    }

    /// The slot-tracked play paths' shared head: resolve the cue, attenuate
    /// it, and resample the PCM for the pitch (XNA octaves — the rate scales
    /// by `2^pitch`; 0 keeps the buffer untouched).
    fn prepare(&self, cue: &str, x: f32, y: f32, volume_scale: f32, pitch: f32) -> Option<Prepared> {
        let (channels, rate, samples) = self.cues.get(cue)?;
        let playback = sound::play_sound(sound_type(cue), x, y, self.listener, volume_scale)?;
        let ratio = 2.0f32.powf(pitch.clamp(-1.0, 1.0));
        let samples = if (ratio - 1.0).abs() < f32::EPSILON {
            samples.clone()
        } else {
            resample_i16(samples, *channels, ratio)
        };
        Some(Prepared {
            channels: *channels,
            rate: *rate,
            samples,
            volume: playback.volume,
            pan: playback.pan,
        })
    }

    /// `SoundInstanceThunder[slot].State == Playing` (`LegacySoundPlayer.cs:459`
    /// /`:461`) — case 43's re-roll loop reads it live. A finished sink is the
    /// C#'s `SoundState.Stopped`; headless (no output device) nothing ever
    /// plays, so the loop breaks on its first pass like the engine's fresh
    /// instance table.
    pub fn thunder_playing(&self, slot: u32) -> bool {
        self.thunder_slots
            .get(slot as usize)
            .and_then(|s| s.as_ref())
            .is_some_and(|sink| !sink.empty())
    }

    /// Case 43's play (`LegacySoundPlayer.cs:466-476`): the picked slot's
    /// live instance is stopped outright and the strike's cue plays at the
    /// attenuated volume with the pitch jitter — the slot tracking is what
    /// [`Audio::thunder_playing`] reads.
    pub fn play_thunder(&mut self, slot: u32, cue: &str, x: f32, y: f32, pitch: f32) {
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(prepared) = self.prepare(cue, x, y, 1.0, pitch) else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = rodio::Sink::try_new(handle) else { return };
        let (left, right) = sound::pan_gains(prepared.pan);
        let buf = rodio::buffer::SamplesBuffer::new(
            prepared.channels,
            prepared.rate,
            prepared.samples,
        )
        .convert_samples::<f32>();
        sink.append(Panned {
            inner: buf,
            channels: prepared.channels,
            left: prepared.volume * left,
            right: prepared.volume * right,
            channel: 0,
        });
        let idx = slot as usize % 6;
        if let Some(old) = self.thunder_slots[idx].take() {
            old.stop();
        }
        self.thunder_slots[idx] = Some(sink);
    }

    /// `SoundInstanceZombie[slot].State == Playing` — the critter family's
    /// guard half (`LegacySoundPlayer.cs:1064-1069` and friends).
    pub fn zombie_playing(&self, slot: usize) -> bool {
        self.zombie_slots
            .get(&slot)
            .is_some_and(|sink| !sink.empty())
    }

    /// One critter-family voice (`LegacySoundPlayer.cs`'s zombie cases): the
    /// resolved [`crate::ambient_sound::CritterVoice`] plays at the
    /// attenuated volume × the case multiplier, tracked on its slot. The
    /// `x`/`y` are the NPC's world pixels — the C#'s
    /// `(int)base.position.X/(int)base.position.Y` casts truncate here.
    /// An overwritten slot detaches the old sink, not stops it: case 44
    /// replaces the array entry while the old instance plays out
    /// (`:1051-1062`).
    pub fn play_critter(&mut self, voice: crate::ambient_sound::CritterVoice, x: f32, y: f32) {
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let x = x.trunc();
        let y = y.trunc();
        let Some(prepared) = self.prepare(voice.asset, x, y, voice.volume_mult, voice.pitch)
        else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = rodio::Sink::try_new(handle) else { return };
        let (left, right) = sound::pan_gains(prepared.pan);
        let buf = rodio::buffer::SamplesBuffer::new(
            prepared.channels,
            prepared.rate,
            prepared.samples,
        )
        .convert_samples::<f32>();
        sink.append(Panned {
            inner: buf,
            channels: prepared.channels,
            left: prepared.volume * left,
            right: prepared.volume * right,
            channel: 0,
        });
        if let Some(old) = self.zombie_slots.insert(voice.slot, sink) {
            old.detach();
        }
    }

    /// S1-E-21e — one fighter-moan play (`AI_003_Fighters`' ambient arms,
    /// `NPC.cs:57340-57367`: `PlaySound(14/26/29, (int)position.X,
    /// (int)position.Y[, Style])` — the int overload, `SoundEngine.cs:102-108`,
    /// draws NO `GetRandomPitch` NextFloat). The attenuation gate
    /// (`LegacySoundPlayer.cs:369-436`) runs FIRST — a muted or out-of-range
    /// play consumes nothing — then the case's slot/pitch draws ride the
    /// caller's stream (the moan rolls run under the `UpdateNPCs` swap,
    /// `Main.cs:17767`) through [`crate::ambient_sound::critter_legacy_voice`]:
    ///
    /// * case 14 (`:793-824`): the style-542 slot 7 (no draw), the 489/586
    ///   `Next(21, 24)` or the default `Next(3)` — ONE slot draw, NO pitch
    ///   draw, ×0.4, the array entry overwritten without a Stop.
    /// * case 26 (`:969-978`): `Next(3, 5)` + one `Next(-10, 11)` pitch,
    ///   ×0.9, overwrite without a Stop.
    /// * case 29 (`:1008-1050`): the Style IS the slot, the per-style volume
    ///   arms, the playing guard returns BEFORE the pitch draw, ×mult, one
    ///   `Next(-10, 11)` pitch.
    ///
    /// Cases 14/26/29 are not on the ambient-scaling list
    /// (`LegacySoundPlayer.cs:1444-1457`) — the gate scales `Main.soundVolume`.
    /// Returns the asset played; `None` when the gate refused, the id is not
    /// a moan case, or a case-29 playing guard blocked the slot.
    pub fn play_fighter_moan(
        &mut self,
        legacy_id: u32,
        style: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The moan arms' three cases only — anything else is a caller bug.
        if !matches!(legacy_id, 14 | 26 | 29) {
            return None;
        }
        // The attenuation gate first — the int overload draws nothing
        // before the switch (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(legacy_id, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let voice = crate::ambient_sound::critter_legacy_voice(
            legacy_id,
            style as u32,
            &mut |lo, hi| rng.next_range(lo, hi),
            &|slot| self.zombie_playing(slot),
        )?;
        // The slot handling is the critter family's: detach-not-stop on the
        // overwrite (case 14/26 replace the entry without a Stop, case 29's
        // guard already refused a busy slot).
        self.play_critter(voice, x, y);
        Some(voice.asset)
    }

    /// `WorldGen.PlayLiquidChangeSound` (`WorldGen.cs:4632-4650`, S1-E-25):
    /// the merge event's cue, played at `x * 16 + 8, y * 16 + 8` (the C#'s
    /// `count = 1` offset). The `Liquids*` rows ride the trackable style
    /// path — the `Style` getter picks the variation per play
    /// (`LegacySoundStyle.cs:21-28`, the engine's own static stream); the
    /// shimmer arm rides case 19 with its playing guard and pitch roll. The
    /// roll here is the engine's style stream — the C# draws it from
    /// `Main.rand` mid-liquid-pass, and the port's liquid engine is pure
    /// (ANALYSIS, S1-E-25).
    pub fn play_liquid_change(&mut self, kind: tr_core::liquid::TileChangeType, x: i32, y: i32) {
        let Some(cue) = crate::ambient_sound::liquid_change_cue(kind) else {
            return;
        };
        let x = x as f32 * 16.0 + 8.0;
        let y = y as f32 * 16.0 + 8.0;
        if cue.sound_type != 19 {
            let variation = self
                .style_random
                .next_range(0, cue.variations as i32)
                .max(0) as u32;
            let asset = format!("{}_{}", cue.cue, variation);
            self.play_at(&asset, x, y, 1.0);
            return;
        }
        // Case 19 (`LegacySoundPlayer.cs:871-884`): the playing guard
        // returns silent before any draw; variant 4 cuts the volume ×0.75
        // and jitters `Next(-20, 1) * 0.01`.
        if self.splash_slots[4].as_ref().is_some_and(|s| !s.empty()) {
            return;
        }
        let (lo, hi) = cue.pitch_roll.unwrap_or((-20, 1));
        let pitch = self.style_random.next_range(lo, hi) as f32 * 0.01;
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(prepared) = self.prepare(cue.cue, x, y, cue.volume_scale, pitch) else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = rodio::Sink::try_new(handle) else { return };
        let (left, right) = sound::pan_gains(prepared.pan);
        let buf = rodio::buffer::SamplesBuffer::new(
            prepared.channels,
            prepared.rate,
            prepared.samples,
        )
        .convert_samples::<f32>();
        sink.append(Panned {
            inner: buf,
            channels: prepared.channels,
            left: prepared.volume * left,
            right: prepared.volume * right,
            channel: 0,
        });
        if let Some(old) = self.splash_slots[4].take() {
            old.detach();
        }
        self.splash_slots[4] = Some(sink);
    }

    /// S1-E-21 wave A — `Player.ItemCheck_StartActualUse`'s sound arm
    /// (`Player.cs:51681-51698`): `SoundEngine.PlaySound(sItem.UseSound,
    /// base.Center, sItem.useSoundPitch)` for the style at
    /// [`crate::soundid::SOUND_STYLES`] index `style_index`. W4-d21 fold 2:
    /// the projectile trackables ([`Audio::play_proj_sound`]'s id-42 arm)
    /// share the case-42 tail.
    ///
    /// The façade (`SoundEngine.cs:93-99`) passes `type.Style`,
    /// `type.Volume * volumeScale` (volumeScale 1 here) and
    /// `pitchOffset + type.GetRandomPitch()`; the `GetRandomPitch` draw rides
    /// the audio-only `SoundStyle._random` stream (`sound_style_random`),
    /// never a gameplay stream. The legacy case-2 volume/pitch maths
    /// (`LegacySoundPlayer.cs:516-644`) draws `Main.rand` — the caller passes
    /// the `UpdatePlayers` stream (`Player.ItemCheck` runs under the
    /// `Main.cs:17754` swap), and the draws run only AFTER the attenuation
    /// gate passed, exactly the C#'s order (`:390-436` precedes the switch).
    ///
    /// Trackable styles (`SoundId == 42`, case 42 `:1376-1381`) play the
    /// pinned first variation like [`NPC_CUSTOM_CUES`] (no pitch roll).
    /// Returns the cue asset played (the test seam); `None` when the
    /// attenuation gate refused. The instance stop/re-create pool is the
    /// standing D4 deviation.
    pub fn play_use_sound(
        &mut self,
        style_index: u16,
        x: f32,
        y: f32,
        pitch_offset: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        self.play_use_sound_scaled(style_index, x, y, pitch_offset, 1.0, rng)
    }

    /// S1-E-21h d76 DOC-SWEEP — the volumeScale-carrying peer of
    /// [`Audio::play_use_sound`]. `volume_scale` is the C# call's fourth
    /// argument (`SoundEngine.cs:99` folds it into `type.Volume * volumeScale`,
    /// which the legacy player applies as the POST-gate `instance.Volume *=
    /// volumeScale`, `LegacySoundPlayer.cs:1411`). The Corrupt Whip's ×0.6 tip
    /// cue (`Projectile.cs:47720`) and the Lights Bane slash's `0.15 * ai[0]`
    /// (`:42237`) ride it. The attenuation gate still reads the UNSCALED
    /// `style.volume` (`:369-436` precedes the scale), and the scale multiplies
    /// the case-2 volume AFTER the tweak — exactly the C# order. `1.0` is the
    /// no-scale default every other caller passes.
    /// W4-e7 lane 23 — [`Self::play_use_sound`] with the C# call's
    /// `volumeScale` argument (`SoundEngine.cs:88-99` passes
    /// `type.Volume * volumeScale` into the engine; the legacy player's
    /// `soundEffectInstance.Volume *= volumeScale`, `LegacySoundPlayer.cs:1411`,
    /// multiplies AFTER the attenuation walk). The Snapping Stone strike
    /// family's 0.08/0.5 scales ride it (`snapping_stone.rs`); every other
    /// caller keeps the 1.0 wrapper above. The draw order is unchanged:
    /// the `GetRandomPitch` NextFloat, the attenuation gate, then case 2's
    /// `Main.rand` roll — the scale only touches the final gain.
    /// S1-E-21h d77 — the `volumeScale`-carrying peer of
    /// [`Audio::play_use_sound`]: the style façade's fourth argument
    /// (`SoundEngine.cs:93-99` — `type.Volume * volumeScale` rides the
    /// legacy player's `volumeScale` parameter, applied at
    /// `LegacySoundPlayer.cs:1411` AFTER the attenuation gate and the
    /// case-2 volume tweaks, so a quiet scale never changes the gate's
    /// refusal nor the RNG census). The DD2 Goblin Bomb's death cue
    /// (`SoundID.DD2_GoblinBomb = new LegacySoundStyle(2, 14).WithVolume(0.5f)`,
    /// `SoundID.cs:1596`, played at `Projectile.cs:71906`) is the first
    /// caller; every other style play keeps the 1.0 default through
    /// [`Audio::play_use_sound`].
    /// [`Audio::play_use_sound`]. f1 adopts the seam VERBATIM from d77's
    /// pushed tip (`9d35874a`, `task/W4-d77-proj-sounds-3`) so the merge's
    /// queue shape is the 6-tuple on both sides.
    pub fn play_use_sound_scaled(
        &mut self,
        style_index: u16,
        x: f32,
        y: f32,
        pitch_offset: f32,
        call_volume_scale: f32,
        volume_scale: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let style = &crate::soundid::SOUND_STYLES[style_index as usize];
        // `SoundEngine.cs:99` — `pitchOffset + type.GetRandomPitch()`; the
        // NextFloat draw is consumed even at variance 0 (`SoundStyle.cs:42`).
        let draw = self.sound_style_random.next_float();
        let pitch_offset =
            pitch_offset + sound::get_random_pitch(style.pitch_variance, draw);
        // The façade's volumeScale is `type.Volume * 1` (`SoundEngine.cs:99`)
        // — the style volume rides INSIDE `play_item_style`'s case gain
        // (W4-e7 keep-one: passing it here too would double it against the
        // `gain *= volume_scale` tail, `LegacySoundPlayer.cs:1411`).
        self.play_item_style(style_index, x, y, pitch_offset, 1.0, rng)
    }

    /// M2-MOUNT-4 (W4-c14) — the Santank ability cues (`Mount.cs:3300`
    /// Item89 / `:3311` Item11): `SoundEngine.PlaySound(SoundID.ItemN.SoundId,
    /// (int)x, (int)y, SoundID.ItemN.Style, 0.2f)` — the INT overload
    /// (`SoundEngine.cs:102-108`), so NO `GetRandomPitch` NextFloat (the
    /// style façade `:93-99` is NOT the caller here), pitchOffset 0, and
    /// volumeScale 0.2 rides `LegacySoundPlayer.cs:1411` AFTER the case-2
    /// tweak. The case-2 body is the shared [`Self::play_item_style`]: the
    /// 11/89 default arm draws ONE `Next(-6, 7) * 0.01` (`:632`) on the
    /// caller's stream (the fire runs inside `Player.Update` under the
    /// `Main.cs:17754` `UpdatePlayers` swap), only after the attenuation
    /// gate; the default instance arm stops + re-creates the style slot
    /// (`:589-594` — neither style is on the stack-allowed list `:595-605`).
    /// The positions are the C#'s truncated ints (the caller truncates).
    pub fn play_mount_ability_sound(
        &mut self,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let idx = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.sound_id == 2 && s.style == style_num)?;
        self.play_item_style(idx as u16, x, y, 0.0, 0.2, rng)
    }

    /// The shared `LegacySoundPlayer.PlaySound` case-2/case-42 body
    /// (`LegacySoundPlayer.cs:360-1415`): the attenuation gate, the case
    /// tweak + pitch draw, the bank resolve, the emit and the instance-slot
    /// bookkeeping. `volume_scale` is the LegacySoundPlayer argument — the
    /// style façade passes `type.Volume` (the `SoundEngine.cs:99` product),
    /// the int overload passes the caller's raw scale (the mount cues' 0.2).
    fn play_item_style(
        &mut self,
        style_index: u16,
        x: f32,
        y: f32,
        pitch_offset: f32,
        volume_scale: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let style = &crate::soundid::SOUND_STYLES[style_index as usize];
        // The attenuation gate first — the C# walks the position/volume
        // checks (`:369-436`) before the type switch draws anything. The
        // caller's `volumeScale` rides `:1411` AFTER the gate, so folding it
        // into the probe's scale cannot change a refusal (the gate never
        // reads the scale).
        if sound::play_sound(
            u32::from(style.sound_id),
            x,
            y,
            self.listener,
            volume_scale,
            style.volume * volume_scale,
        )
        .is_none()
        {
            return None;
        }
        let (cue, mut gain, pitch) = if style.sound_id == 42 {

        }        let (cue, cue_volume, pitch) = if style.sound_id == 42 {
            // Case 42 (`:1376-1381`): no volume tweak, no pitch switch —
            // only the caller's offset, clamped at `:1410`.
            let base = style.trackable.expect("trackable style carries an asset");
            let cue = if style.variations == 1 {
                base.to_string()
            } else {
                format!("{base}_0")
            };
            (cue, volume_scale, pitch_offset.clamp(-1.0, 1.0))
        } else {
            debug_assert_eq!(style.sound_id, 2, "item-use styles are the id-2 family");
            // W4-c14 doc-sweep — the 53/55/159 playing guards
            // (`LegacySoundPlayer.cs:559-562`/`:583-586`): while the slot's
            // instance is Playing the play returns null BEFORE the pitch
            // draw (the `npc_hit_case3` 57-guard precedent). Headless no
            // slot ever plays, so the guard never trips (the case-28
            // witness precedent).
            let playing = self
                .item_slots
                .get(&(style.style as usize))
                .is_some_and(|s| !s.empty());
            if sound::item_case2_playing_guard(style.style, playing) {
                return None;
            }
            let mut roll = |lo: i32, hi: i32| rng.next_range(lo, hi);
            let (num, tweak, pitch) = sound::item_use_case2(style.style, &mut roll);
            let cue = format!("Item_{num}");
            (
                cue,
                // `:1411` — the case tweak rides num2, the volumeScale
                // multiplies AFTER (the façade's scale is style.volume,
                // the mount int-overload's is 0.2).
                volume_scale * tweak,
                (pitch + pitch_offset).clamp(-1.0, 1.0),
            )
        };
        // Resolve against the loaded bank's static names (the cue is always
        // in SFX_CUES/ITEM_CUSTOM_CUES/PROJ_CUSTOM_CUES — an unregistered
        // name is a bank gap).
        // in SFX_CUES/ITEM_CUSTOM_CUES/PLAYER_VOICE_CUES — an unregistered
        // name is a bank gap). W4-e10: the player-voice trackables join the
        // resolution chain (the hurt/death style arms ride this façade).
        let name = SFX_CUES
            .iter()
            .map(|(n, _)| *n)
            .chain(ITEM_CUSTOM_CUES.iter().copied())
            .chain(PROJ_CUSTOM_CUES.iter().copied())
            .find(|n| *n == cue);
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(name) = name else { return None };
        // `LegacySoundPlayer.cs:1411` — the call's volumeScale multiplies the
        // instance volume AFTER the case tweak (the gate above read the
        // unscaled `style.volume`).
        self.emit(name, x, y, volume_scale * call_volume_scale, pitch);
        // `soundEffectInstance.Volume *= volumeScale` (`LegacySoundPlayer.cs:1411`,
        // W4-e7) — the caller's scale multiplies the case gain (the tweak is
        // already folded into `gain` above; the 1.0 façade wrapper keeps every
        // pre-W4-e7 play byte-identical).
        gain *= volume_scale;
        let sink = self.emit(name, x, y, gain, pitch);
        // W4-c14 doc-sweep — the case-2 instance pool (`:580-607`): the
        // guarded styles 53/55/159 and the default arm stop the slot's old
        // instance and re-create it; the stack-allowed list (`:595-605`:
        // 9/10/24/26/34/43/103/156/162/205) never touches a slot (the old
        // sound keeps playing — the fresh-source behaviour the port had).
        if style.sound_id == 2
            && !matches!(style.style, 9 | 10 | 24 | 26 | 34 | 43 | 103 | 156 | 162 | 205)
        {
            let idx = style.style as usize;
            if let Some(old) = self.item_slots.remove(&idx) {
                old.stop();
            }
            if let Some(sink) = sink {
                self.item_slots.insert(idx, sink);
        }

        }            .chain(PLAYER_VOICE_CUES.iter().copied())
        // `:1411` — `soundEffectInstance.Volume *= volumeScale`: the caller's
        // scale multiplies the case's own volume AFTER the tweaks.
        self.emit(name, x, y, cue_volume * volume_scale, pitch);
        Some(name)
    }

    /// S1-P-i41 — the Chillemt mounts' equip cue (`Mount.SetMount`
    /// `Mount.cs:6283-6286`: `SoundEngine.PlaySound(SoundID.PalChillet,
    /// mountedPlayer.Center)`). The style façade (`SoundEngine.cs:93-99`)
    /// consumes the `GetRandomPitch` NextFloat on the audio-only stream
    /// (variance 0 — the draw still happens), then case 42
    /// (`LegacySoundPlayer.cs:1376-1381`): no volume tweak, no pitch switch,
    /// the style's own 0.35 volume, the one-variation trackable asset.
    pub fn play_chillet_sound(&mut self, x: f32, y: f32) -> Option<&'static str> {
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.name == "PalChillet")?;
        let draw = self.sound_style_random.next_float();
        let pitch = sound::get_random_pitch(style.pitch_variance, draw);
        if sound::play_sound(u32::from(style.sound_id), x, y, self.listener, style.volume)
            .is_none()
        {
            return None;
        }
        let cue = "pal_chillet";
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, style.volume, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// W4-g10 FINAL — the Chillemt mounts' PETTING joy cue
    /// (`Player.PetMount` `Player.cs:33352-33355`:
    /// `SoundEngine.PlaySound(SoundID.PalChilletJoy, base.Center)` for the
    /// `(uint)(type - 62) <= 3` gate). The same case-42 trackable façade as
    /// [`Self::play_chillet_sound`] — the style's own 0.35 volume, the
    /// variance-0 NextFloat still consumed on the audio-only stream, the
    /// one-variation `pal_chillet_joy` asset (`SoundID.cs:1717`).
    pub fn play_petting_joy_sound(&mut self, x: f32, y: f32) -> Option<&'static str> {
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.name == "PalChilletJoy")?;
        let draw = self.sound_style_random.next_float();
        let pitch = sound::get_random_pitch(style.pitch_variance, draw);
        if sound::play_sound(u32::from(style.sound_id), x, y, self.listener, style.volume)
            .is_none()
        {
            return None;
        }
        let cue = "pal_chillet_joy";
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, style.volume, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-P-i41 — the QuickMount no-space tink (`Player.cs:6057`:
    /// `SoundEngine.PlaySound(0, (int)position.X, (int)position.Y)`). The int
    /// overload (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat;
    /// case 0 (`LegacySoundPlayer.cs:440-452`) draws `Main.rand.Next(3)` for
    /// the Dig variation and `Next(-10, 11) * 0.01` for the pitch — both ride
    /// the caller's stream (the player tick runs under the `Main.cs:17754`
    /// `UpdatePlayers` swap) and only AFTER the attenuation gate passed.
    pub fn play_no_space_sound(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(0, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let variation = rng.next_range(0, 3);
        let pitch =
            sound::legacy_family_pitch(sound::Family::Dig, &mut |lo, hi| rng.next_range(lo, hi));
        let cue = match variation {
            0 => "Dig_0",
            1 => "Dig_1",
            _ => "Dig_2",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// The shared output tail of the pitched play paths: prepare (attenuate +
    /// resample for the pitch) and append one panned source to a fresh sink.
    /// Headless (no output device) this is a no-op past `prepare`. Returns
    /// the sink so the slot-tracked families can model the C# instance
    /// arrays (`SoundInstanceNpcHit`/`SoundInstanceNpcKilled`).
    fn emit(&self, name: &str, x: f32, y: f32, volume_scale: f32, pitch: f32) -> Option<rodio::Sink> {
        let prepared = self.prepare(name, x, y, volume_scale, pitch)?;
        let handle = self.handle.as_ref()?;
        let sink = rodio::Sink::try_new(handle).ok()?;
        let (left, right) = sound::pan_gains(prepared.pan);
        let buf = rodio::buffer::SamplesBuffer::new(
            prepared.channels,
            prepared.rate,
            prepared.samples,
        )
        .convert_samples::<f32>();
        sink.append(Panned {
            inner: buf,
            channels: prepared.channels,
            left: prepared.volume * left,
            right: prepared.volume * right,
            channel: 0,
        });
        Some(sink)
    }

    /// W4-d85 FINAL — the issue-report indicator's notify cue
    /// (`IssueReportsIndicator.AttemptLettingPlayerKnow`,
    /// `IssueReportsIndicator.cs:24`: `SoundEngine.PlaySound(
    /// SoundID.DD2_ExplosiveTrapExplode)` — the unpositioned legacy play).
    /// The façade (`SoundEngine.cs:93-99`) consumes the `GetRandomPitch`
    /// NextFloat on the audio-only stream (variance 0 — the draw still
    /// happens) AND the variation pick (`LegacySoundStyle.Style`,
    /// `LegacySoundStyle.cs:24-27`, the audio-only `Random` — the tracked
    /// layer's `style_random` precedent); then case 42
    /// (`LegacySoundPlayer.cs:1376-1381`): no volume tweak, no pitch switch,
    /// the style's own volume 1, the picked `<cue>_<n>` asset. Unpositioned
    /// (`x == -1`) — no attenuation, no pan (`:376-379`).
    pub fn play_issue_report_notify(&mut self) -> Option<&'static str> {
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.name == "DD2_ExplosiveTrapExplode")?;
        let draw = self.sound_style_random.next_float();
        let pitch = sound::get_random_pitch(style.pitch_variance, draw);
        if sound::play_sound(u32::from(style.sound_id), -1.0, -1.0, self.listener, style.volume)
            .is_none()
        {
            return None;
        }
        let n = self.style_random.next_range(0, style.variations as i32);
        let cue = match n {
            0 => "dd2_explosive_trap_explode_0",
            1 => "dd2_explosive_trap_explode_1",
            _ => "dd2_explosive_trap_explode_2",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, -1.0, -1.0, style.volume, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21 wave C — `NPC.StrikeNPC`'s hit-sound arm
    /// (`NPC.cs:81544-81553`: `SoundEngine.PlaySound(this.HitSound,
    /// base.position)`), also the eel-whip DoT tick's `HitSound` at
    /// `base.Center` (`:91852`). `cue` is the kind's
    /// [`crate::npc::NpcKind::hit_sound`] asset name.
    ///
    /// The façade (`SoundEngine.cs:93-99`) consumes the `GetRandomPitch`
    /// NextFloat on the audio-only `sound_style_random` (every `NPCHitN`
    /// style has variance 0 — the draw still happens), then the attenuation
    /// gate (`LegacySoundPlayer.cs:369-436`), then case 3 (`:646-672`): the
    /// 20..54/55/56 ×0.5 and 57 ×0.6 volume arms, the 57 playing guard
    /// (`SoundInstanceNpcHit[57]`, `:651-654`) and the ONE
    /// `Main.rand.Next(-10, 11) * 0.01` pitch draw (`:670`). The caller
    /// passes the striking stage's named stream — `UpdatePlayers` for the
    /// melee strike (`Main.cs:17754`), `UpdateProjectiles` for
    /// `Projectile.Damage` (`:17775`), the bare-`Main.rand` stand-in for the
    /// net-message strikes (`Main.cs:16895` runs outside every swap). The
    /// play stops and re-creates the family slot's instance (`:663-667`).
    ///
    /// The DD2/Deerclops trackables ride case 42 (`:1376-1381`): no volume
    /// tweak, no pitch draw, the style's own `Volume` (DeerclopsHit 0.3) —
    /// the port pins the first variation like wave A (the D3 pin).
    /// Returns the cue played; `None` when the gate or the 57 guard refused.
    pub fn play_npc_hit_sound(
        &mut self,
        cue: &'static str,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // Resolve against the loaded bank's static names — an unregistered
        // name is a bank gap (the wave-A convention).
        if !sfx_cue_registered(cue) && !NPC_CUSTOM_CUES.contains(&cue) {
            return None;
        }
        let style = npc_sound_style(cue, "NPC_Hit_", 3)?;
        // `SoundEngine.cs:99` — the NextFloat draw is consumed even at
        // variance 0 (`SoundStyle.cs:42`), before the legacy gate.
        let draw = self.sound_style_random.next_float();
        let pitch_offset = sound::get_random_pitch(style.pitch_variance, draw);
        self.npc_hit_play(cue, style, pitch_offset, 1.0, x, y, rng)
    }

    /// S1-E-21g — the int-overload peer of [`Audio::play_npc_hit_sound`]
    /// (`SoundEngine.cs:102-108`: `PlaySound(int type, int x, int y,
    /// int Style)` calls the legacy player DIRECTLY — NO `GetRandomPitch`
    /// NextFloat). The wave-G call sites: the EoC's phase hand-off
    /// `PlaySound(3, pos)` (`NPC.cs:20364`, the default Style 1) and the
    /// Brain of Cthulhu's retreat entry (`NPC.cs:32408`). Only the styles
    /// the wired sites name resolve; anything else is a caller bug.
    pub fn play_npc_hit_int(
        &mut self,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The C# sites pass `(int)base.position.X/Y` — truncate here.
        let (x, y) = (x.trunc(), y.trunc());
        let cue = match style_num {
            1 => "NPC_Hit_1",
            // W4-d41 — the Tumbleweed's hop cue (`NPC.cs:62568`:
            // `PlaySound(3, base.Center, 11)`).
            11 => "NPC_Hit_11",
            // S1-E-21h f1 final phase — the 1054 SpiderBoulder's Kill
            // play (`Projectile.cs:74867`, `PlaySound(SoundID.NPCHit29,
            // x, y)` — the int overload, NO NextFloat; case 3's 20..54
            // ×0.5 volume arm + one `Next(-10, 11)` pitch, `:646-672`).
            29 => "NPC_Hit_29",
            _ => return None,
        };
        let style = npc_sound_style(cue, "NPC_Hit_", 3)?;
        self.npc_hit_play(cue, style, 0.0, 1.0, x, y, rng)
    }

    /// The shared case-3 tail of both hit overloads: the attenuation gate
    /// (`LegacySoundPlayer.cs:369-436`), the case-3 maths (`:646-672`),
    /// the slot Stop+re-create (`:663-667`) and the emit.
    fn npc_hit_play(
        &mut self,
        cue: &'static str,
        style: &crate::soundid::SoundStyleDef,
        pitch_offset: f32,
        call_volume_scale: f32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(u32::from(style.sound_id), x, y, self.listener, style.volume)
            .is_none()
        {
            return None;
        }
        let (volume_scale, pitch) = if style.sound_id == 42 {
            // Case 42 (`:1376-1381`): the caller's offset only (`:1410`).
            (style.volume, pitch_offset.clamp(-1.0, 1.0))
        } else {
            let playing_57 = self.npc_hit_slots[57].as_ref().is_some_and(|s| !s.empty());
            let (tweak, pitch) = sound::npc_hit_case3(style.style, playing_57, &mut |lo, hi| {
                rng.next_range(lo, hi)
            })?;
            (
                style.volume * tweak,
                (pitch + pitch_offset).clamp(-1.0, 1.0),
            )
        };
        // `LegacySoundPlayer.cs:1411` — the call's volumeScale, post-tweak.
        let volume_scale = volume_scale * call_volume_scale;
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if style.sound_id == 3 {
            // `:663-667` — Stop the slot's old instance, then re-create.
            let idx = style.style as usize;
            if let Some(old) = self.npc_hit_slots[idx].take() {
                old.stop();
            }
            self.npc_hit_slots[idx] = self.emit(cue, x, y, volume_scale, pitch);
        } else {
            let _ = self.emit(cue, x, y, volume_scale, pitch);
        }
        Some(cue)
    }

    /// S1-E-21 wave C — `NPC.checkDead`'s death-sound arm
    /// (`NPC.cs:78186-78194`: `SoundEngine.PlaySound(legacySoundStyle,
    /// base.position)` — the kind's `DeathSound`; the 229 shimmer-variant
    /// override to `NPCDeath6` has no ported kind, the invasion pirate 229
    /// is unported). `cue` is [`crate::npc::NpcKind::death_sound`].
    ///
    /// Same façade as [`Audio::play_npc_hit_sound`], then case 4
    /// (`LegacySoundPlayer.cs:673-695`): the 23..57 ×0.5 and 61/62 ×0.6
    /// volume arms, the 10 playing guard (`SoundInstanceNpcKilled[10]`,
    /// `:686-689`) and the ONE `Main.rand.Next(-10, 11) * 0.01` pitch draw
    /// (`:693`). The port's death seam runs in the NPC pass, so the caller
    /// passes the `UpdateNPCs` stream (`Main.cs:17767`). Unlike case 3 the
    /// slot is overwritten WITHOUT a Stop (`:690`) — the old voice plays
    /// out (the sink is detached).
    pub fn play_npc_death_sound(
        &mut self,
        cue: &'static str,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if !sfx_cue_registered(cue) && !NPC_CUSTOM_CUES.contains(&cue) {
            return None;
        }
        let style = npc_sound_style(cue, "NPC_Killed_", 4)?;
        let draw = self.sound_style_random.next_float();
        let pitch_offset = sound::get_random_pitch(style.pitch_variance, draw);
        self.npc_death_play(cue, style, pitch_offset, 1.0, x, y, rng)
    }

    /// S1-E-21g — one case-4 play resolved from a `SoundID.NPCDeathN`
    /// STYLE façade (`SoundEngine.cs:93-99` — the `GetRandomPitch`
    /// NextFloat IS consumed, on the audio-only stream). The wave-G
    /// sites: the Moon Lord core's crash cry (`NPCDeath61`,
    /// `NPC.cs:41432`), the leech head's tunnel-entry burst
    /// (`NPCDeath13`, `:50968`). The int-overload sites (styles 22/6)
    /// use [`Audio::play_npc_death_int`] instead — no NextFloat.
    pub fn play_npc_death_style(
        &mut self,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // `SoundEngine.cs:88-91` — the Vector2 façade truncates.
        let (x, y) = (x.trunc(), y.trunc());
        let cue = match style_num {
            13 => "NPC_Killed_13",
            // W4-f2 reconcile (e9 union) — the Ghost's flee cry
            // (`NPC.cs:24693`, `PlaySound(SoundID.NPCDeath55,
            // (int)Center.X, (int)Center.Y)` — the façade consumes the
            // NextFloat on the audio-only stream; the 316 machine's
            // wiring consumes the row).
            55 => "NPC_Killed_55",
            61 => "NPC_Killed_61",
            // W4-f2 closeout (e9b tail union, research/94 §10) — the
            // balloon pops' `this.DeathSound` = `NPCDeath63`
            // (`SetDefaults` 686/594, `NPC.cs:17636`/`:16716`; the
            // ClumsySlimeBalloon `:43726` + the WindyBalloon
            // `:47951`/`:48062` sites, the style façade).
            63 => "NPC_Killed_63",
            _ => return None,
        };
        let style = npc_sound_style(cue, "NPC_Killed_", 4)?;
        let draw = self.sound_style_random.next_float();
        let pitch_offset = sound::get_random_pitch(style.pitch_variance, draw);
        self.npc_death_play(cue, style, pitch_offset, 1.0, x, y, rng)
    }

    /// S1-E-21g — the int-overload peer of [`Audio::play_npc_death_sound`]
    /// (`SoundEngine.cs:102-108` — NO `GetRandomPitch` NextFloat). The
    /// wave-G call sites: the tower dissolve's `PlaySound(4, Center, 22)`
    /// (`NPC.cs:39059`) and the Moon parts' laser wind-up
    /// `PlaySound(4, pos, 6)` (`NPC.cs:41838`/`:42338`/`:42611`).
    /// S1-E-21h d77 adds style 19 — the Sharknado bolt's `Kill()` cry
    /// (`Projectile.cs:73763`), drained on the `UpdateProjectiles` stream —
    /// and style 7, the Stardust Jellyfish's `Kill()` cry (`:72571`, the
    /// re-probe wave: the row lives on d11, the cue's position is the
    /// derived centre − 40 post-recentre read).
    pub fn play_npc_death_int(
        &mut self,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The C# sites pass `(int)base.position.X/Y` (or the truncated
        // Center via `:83-85`) — truncate here.
        let (x, y) = (x.trunc(), y.trunc());
        let cue = match style_num {
            // W4-f2 reconcile (e9 union) — the Vile Spit's every-
            // `ai[0]==2`-tick cry (`NPC.cs:21480`, `PlaySound(4, pos, 9)`
            // — the int overload, NO NextFloat; the 112/666 kinds'
            // wiring consumes the row).
            9 => "NPC_Killed_9",
            6 => "NPC_Killed_6",
            // S1-E-21h d77 — the Stardust Jellyfish's style-7 cry
            // (`Projectile.cs:72571`).
            7 => "NPC_Killed_7",
            19 => "NPC_Killed_19",
            // S1-E-21h f1 final phase — the 655 arm's default-Style
            // NPCDeath play (`Projectile.cs:75082`, `PlaySound(4, x, y)` —
            // the int overload's Style default 1); the row rides d3's
            // pushed tip `2d01693c`, the cue is inert until the merge.
            1 => "NPC_Killed_1",
            6 => "NPC_Killed_6",
            // S1-E-21h f1 owed-arm phase — the Stardust jellyfish 539's
            // death cry (`Projectile.cs:72571`, `PlaySound(4, pos, 7)`); the
            // row landed on d11's pushed tip, the cue arms here.
            7 => "NPC_Killed_7",
            22 => "NPC_Killed_22",
            // W4-f2 closeout (e9b tail union, research/94 §10) — Medusa's
            // stone-gaze cry (`NPC.cs:56403`, `PlaySound(4,
            // (int)position.X, (int)position.Y, 17)` — the int overload,
            // NO NextFloat; the 480 kind's wiring consumes the row).
            17 => "NPC_Killed_17",
            _ => return None,
        };
        let style = npc_sound_style(cue, "NPC_Killed_", 4)?;
        self.npc_death_play(cue, style, 0.0, 1.0, x, y, rng)
    }

    /// The shared case-4 tail of both death overloads: the attenuation
    /// gate, the case-4 maths (`LegacySoundPlayer.cs:673-695`), the
    /// no-Stop slot overwrite (`:690`) and the emit.
    fn npc_death_play(
        &mut self,
        cue: &'static str,
        style: &crate::soundid::SoundStyleDef,
        pitch_offset: f32,
        call_volume_scale: f32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(u32::from(style.sound_id), x, y, self.listener, style.volume)
            .is_none()
        {
            return None;
        }
        let (volume_scale, pitch) = if style.sound_id == 42 {
            (style.volume, pitch_offset.clamp(-1.0, 1.0))
        } else {
            let playing_10 = self
                .npc_killed_slots
                .get(&10)
                .is_some_and(|s| !s.empty());
            let (tweak, pitch) = sound::npc_killed_case4(style.style, playing_10, &mut |lo, hi| {
                rng.next_range(lo, hi)
            })?;
            (
                style.volume * tweak,
                (pitch + pitch_offset).clamp(-1.0, 1.0),
            )
        };
        // `LegacySoundPlayer.cs:1411` — the call's volumeScale multiplies the
        // instance volume AFTER the case-4 tweak (the Corrupt Whip's ×0.6 tip
        // cue, `Projectile.cs:47720`); the gate above read the unscaled volume.
        let volume_scale = volume_scale * call_volume_scale;
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if style.sound_id == 4 {
            // `:690` — the array entry is replaced with no Stop.
            if let Some(sink) = self.emit(cue, x, y, volume_scale, pitch) {
                if let Some(old) = self.npc_killed_slots.insert(style.style as usize, sink) {
                    old.detach();
                }
            }
        } else {
            let _ = self.emit(cue, x, y, volume_scale, pitch);
        }
        Some(cue)
    }

    /// S1-E-21g — one case-15 boss-roar play (`NPC.cs:20375`/`:22169`/
    /// `:32418`/`:87297`, `PlaySound(15, pos, style)` — the int overload,
    /// NO `GetRandomPitch`). The attenuation gate runs FIRST
    /// (`LegacySoundPlayer.cs:369-436`); case 15 (`:826-842`) draws
    /// NOTHING — the style-4 remap + the `SoundInstanceRoar[num]`
    /// null-or-Stopped guard are the whole decision
    /// ([`crate::sound::roar_case15`]). Returns the asset played; `None`
    /// when the gate or the slot guard refused.
    pub fn play_roar(
        &mut self,
        style: i32,
        x: f32,
        y: f32,
    ) -> Option<&'static str> {
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(15, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let probe = sound::roar_case15(style, false)?;
        let playing = self.roar_slots[probe.0]
            .as_ref()
            .is_some_and(|s| !s.empty());
        let (slot, tweak) = sound::roar_case15(style, playing)?;
        let cue = match slot {
            0 => "Roar_0",
            1 => "Roar_1",
            _ => "Roar_2",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // `:836` — a fresh CreateInstance replaces the (Stopped) slot.
        if let Some(old) = self.roar_slots[slot].take() {
            old.stop();
        }
        self.roar_slots[slot] = self.emit(cue, x, y, tweak, 0.0);
        Some(cue)
    }

    /// S1-E-21g — one case-36 dash-roar play (the EoC's `NPC.cs:20500`/
    /// `:20694`, `PlaySound(36, pos, style)` — the int overload). The
    /// gate first; case 36 (`LegacySoundPlayer.cs:1309-1325`) has NO
    /// guard and NO draws — the Style is the slot, `-1` → slot 0 at
    /// `Pitch += 0.6` ([`crate::sound::roar_case36`]). The slot is
    /// overwritten outright (`:1316`).
    pub fn play_dash_roar(
        &mut self,
        style: i32,
        x: f32,
        y: f32,
    ) -> Option<&'static str> {
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(36, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let (slot, pitch) = sound::roar_case36(style);
        let cue = match slot {
            0 => "Roar_0",
            1 => "Roar_1",
            _ => "Roar_2",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Some(old) = self.roar_slots[slot].take() {
            old.stop();
        }
        self.roar_slots[slot] = self.emit(cue, x, y, 1.0, pitch);
        Some(cue)
    }

    /// S1-E-21g — one case-27 Pixie wing-hum play (`NPC.cs:24853-24856`,
    /// the type-75 `Next(40) == 0` ambience arm; no ported Pixie kind yet
    /// — the seam is table-complete for the future kind, the E21e duck
    /// precedent). The int overload draws no NextFloat. The gate first;
    /// case 27 (`LegacySoundPlayer.cs:979-996`): a LIVE instance re-draws
    /// the pitch (`:984`) and returns null — the ONE
    /// `Main.rand.Next(-10, 11) * 0.01` draw is consumed EITHER WAY; a
    /// fresh play stops the old instance (`:987-990`) and draws the same
    /// roll (`:994`). Returns the asset on a fresh play; `None` when the
    /// gate refused or the live-instance branch consumed the draw.
    pub fn play_pixie(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(27, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        // The pitch draw rides the caller's stream in BOTH branches.
        let pitch = rng.next_range(-10, 11) as f32 * 0.01;
        if self.pixie_slot.as_ref().is_some_and(|s| !s.empty()) {
            // `:980-986` — the live instance's Volume/Pan/Pitch update,
            // then `return null` (no new source).
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Some(old) = self.pixie_slot.take() {
            old.stop();
        }
        self.pixie_slot = self.emit("Pixie", x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some("Pixie")
    }

    /// S1-E-21g — one `SoundID.ItemN` style play from an NPC AI site
    /// (the caster teleport's Item8 `NPC.cs:20862`/`:20931`/`:21232`,
    /// the Brain's blinks `:32505`/`:32622`, the Spazmatism flamethrower's
    /// Item34 `:27493`): the same façade as wave A
    /// ([`Audio::play_use_sound`]) — the `GetRandomPitch` NextFloat on the
    /// audio-only stream, the attenuation gate, then case 2's one
    /// `Main.rand` pitch draw on the caller's `UpdateNPCs` stream.
    pub fn play_npc_item_cue(
        &mut self,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // `SoundEngine.cs:88-91` — the Vector2 façade truncates.
        let (x, y) = (x.trunc(), y.trunc());
        let idx = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.sound_id == 2 && s.style == style_num)?;
        self.play_use_sound(idx as u16, x, y, 0.0, rng)
    }

    /// S1-E-21g — the EoW head's dig rumble (`NPC.cs:51990`,
    /// `SoundID.EOWDiggin` — the `CreateTrackable("eow_dig")` style,
    /// `SoundID.cs:1763`, case 42 `LegacySoundPlayer.cs:1376-1381`): the
    /// façade's NextFloat rides the audio-only stream (variance 0.02),
    /// the gate, then NO gameplay draw — the pinned single variation.
    pub fn play_eow_dig(
        &mut self,
        x: f32,
        y: f32,
    ) -> Option<&'static str> {
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.trackable == Some("eow_dig"))?;
        let draw = self.sound_style_random.next_float();
        let pitch = sound::get_random_pitch(style.pitch_variance, draw);
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(42, x, y, self.listener, style.volume).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("eow_dig", x, y, style.volume, pitch.clamp(-1.0, 1.0));
        Some("eow_dig")
    }

    /// W4-f2 closeout (e9b tail union, research/94 §10) — one case-65
    /// Queen Slime idle-moan play (`NPC.cs:45759`, `PlaySound(65,
    /// base.position)` — the int overload, NO `GetRandomPitch` NextFloat).
    /// The attenuation gate first (`LegacySoundPlayer.cs:369-436`), then
    /// case 65 (`:1382-1402`): the 115/116/117 playing guards refuse
    /// BEFORE any draw, `Next(115, 118)` picks the slot, ×0.5 volume, NO
    /// pitch draw. The slot is overwritten (detached, the case creates a
    /// fresh instance `:1397`). Returns the asset played; `None` when the
    /// gate or a playing guard refused.
    pub fn play_queen_slime_moan(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(65, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let voice = crate::ambient_sound::critter_legacy_voice(
            65,
            0,
            &mut |lo, hi| rng.next_range(lo, hi),
            &|slot| self.zombie_playing(slot),
        )?;
        self.play_critter(voice, x, y);
        Some(voice.asset)
    }

    /// W4-f2 closeout (e9b tail union, research/94 §10) — one
    /// `Sounds/Custom/` NPC trackable play by asset name (the Pal pain/joy
    /// cries `NPC.cs:42983-43001` and the Statue Mimic's scare/jump/laugh
    /// `:43562-43608`; `SoundID.CreateTrackable`, `SoundID.cs:1714-1743`).
    /// The style façade (`SoundEngine.cs:93-99`) consumes the
    /// `GetRandomPitch` NextFloat on the audio-only stream (the Pal rows
    /// variance 0, the Statue Mimic rows 0.1 — the draw still happens),
    /// then the attenuation gate, then case 42
    /// (`LegacySoundPlayer.cs:1376-1381`): no volume tweak, no gameplay
    /// draw, the style's own volume, the pinned first variation (the
    /// multi-variation laugh plays `_0`, the wave-A pin). `cue` must be a
    /// registered [`NPC_CUSTOM_CUES`] name. Returns the cue played;
    /// `None` when the name is unregistered or the gate refused.
    pub fn play_npc_trackable(
        &mut self,
        cue: &'static str,
        x: f32,
        y: f32,
    ) -> Option<&'static str> {
        if !NPC_CUSTOM_CUES.contains(&cue) {
            return None;
        }
        // The multi-variation families pin `_0` (the bank's name); the
        // style row's trackable is the base name (strip the suffix).
        let base = cue.strip_suffix("_0").unwrap_or(cue);
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.sound_id == 42 && s.trackable == Some(base))?;
            }
    /// W4-g15 lane 5 — the Heroicis-set spawn cue
    /// (`ParticleOrchestrator.Spawn_HeroicisSetSpawnSound`,
    /// `ParticleOrchestrator.cs:897-900`: `SoundEngine.PlaySound(SoundID.
    /// RainbowBoulder, (int)PositionInWorld.X, (int)PositionInWorld.Y)`).
    /// `SoundID.RainbowBoulder = CreateTrackable("rainbow_boulder")`
    /// (`SoundID.cs:1709`, one variation, volume 1, variance 0) — case 42
    /// (`LegacySoundPlayer.cs:1376-1381`): the façade's NextFloat rides the
    /// audio-only stream, the gate, then NO gameplay draw. The int overload
    /// truncates the position (`SoundEngine.cs:88-91`).
    pub fn play_rainbow_boulder(&mut self, x: f32, y: f32) -> Option<&'static str> {
        let style = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.trackable == Some("rainbow_boulder"))?;
        let draw = self.sound_style_random.next_float();
        let pitch = sound::get_random_pitch(style.pitch_variance, draw);
        let x = x.trunc();
        let y = y.trunc();
        if sound::play_sound(42, x, y, self.listener, style.volume).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, style.volume, pitch.clamp(-1.0, 1.0));
        Some(cue)
        self.emit("rainbow_boulder", x, y, style.volume, pitch.clamp(-1.0, 1.0));
        Some("rainbow_boulder")
    }

    /// S1-E-21 wave B — one tile/wall-placement cue
    /// (`tr_core::edit::place_tile_sound`, the `WorldGen.PlaceTile` tail
    /// switch `WorldGen.cs:60263-60282`; `PlaceObject` `:51441`,
    /// `PlaceThing_Tiles_PlaceIt` `Player.cs:40124` and `PlaceWall` `:75141`
    /// play the plain 0).
    ///
    /// The `SoundID.Item30`/`Item52` arms are style plays — the same façade
    /// as wave A (`SoundEngine.cs:93-99`: the `GetRandomPitch` NextFloat on
    /// the audio-only `sound_style_random`, then case 2's `Main.rand` pitch
    /// roll after the attenuation gate; style 52's ×0.35 volume,
    /// `LegacySoundPlayer.cs:567-569`). The 18/0 arms are the int overload
    /// (`SoundEngine.cs:102-108` — NO `GetRandomPitch` draw): case 18
    /// (`:865-869`) plays `Coins` with no draws and no pitch; case 0
    /// (`:440-452`) draws `Main.rand.Next(3)` for the Dig variation and
    /// `Next(-10, 11) * 0.01` for the pitch — both ride the caller's stream
    /// (placement runs from `Player.ItemCheck`/`PlaceThing` under the
    /// `Main.cs:17754` `UpdatePlayers` swap, the `break_tile` precedent) and
    /// only AFTER the attenuation gate passed (`:369-436` precedes the
    /// switch). Returns the cue played; `None` when the gate refused. The
    /// `SoundInstanceDig` stop/re-create pool is the standing D4 deviation.
    pub fn play_place_sound(
        &mut self,
        (id, style): tr_core::edit::SoundCue,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if id == 2 {
            let idx = crate::soundid::SOUND_STYLES
                .iter()
                .position(|s| s.sound_id == 2 && s.style == i32::from(style))?;
            return self.play_use_sound(idx as u16, x, y, 0.0, rng);
        }
        if id != 0 && id != 18 {
            return None;
        }
        // The attenuation gate first — the int overload draws nothing before
        // the switch (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(u32::from(id), x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let (cue, pitch) = if id == 0 {
            // Case 0 (`:442`/:450): the variation pick, then the pitch roll.
            let variation = rng.next_range(0, 3);
            let pitch =
                sound::legacy_family_pitch(sound::Family::Dig, &mut |lo, hi| rng.next_range(lo, hi));
            (
                match variation {
                    0 => "Dig_0",
                    1 => "Dig_1",
                    _ => "Dig_2",
                },
                pitch,
            )
        } else {
            // Case 18 (`:865-869`): Volume/Pan only — no pitch, no draws.
            ("Coins", 0.0)
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21 wave D — one fast-run footstep play
    /// (`Player.SpawnFastRunParticles`, `Player.cs:20214`:
    /// `PlaySound(hermesStepSound.SoundType, (int)position.X,
    /// (int)position.Y, hermesStepSound.SoundStyle)`). The int overload
    /// (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat.
    ///
    /// Type 17 (`SoundID.Run`, the default cue — `Player.cs:35947-35949`):
    /// case 17 (`LegacySoundPlayer.cs:854-864`) stops the previous instance
    /// (the standing D4 deviation), plays `Sounds/Run` and rolls the pitch
    /// `Main.rand.Next(-10, 11) * 0.01` (`:862`) — the draw rides the
    /// caller's stream (`Player.Update` runs under the `Main.cs:17754`
    /// `UpdatePlayers` swap) and only AFTER the attenuation gate passed
    /// (`:369-436` precedes the switch). The style is unused (`-1`).
    ///
    /// Type 2 style 24 (the Moon Lord legs' override, `Player.cs:35959-35964`):
    /// wave A's style façade — the `GetRandomPitch` NextFloat on the
    /// audio-only `sound_style_random`, then case 2's `Main.rand` pitch roll
    /// after the gate. Returns the cue played; `None` when the gate refused
    /// or the type is not a footstep family.
    pub fn play_step_sound(
        &mut self,
        (sound_type, style): (u16, i16),
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound_type == 2 {
            // The Moon Lord legs' Item-24 click — wave A's case-2 path.
            let idx = crate::soundid::SOUND_STYLES
                .iter()
                .position(|s| s.sound_id == 2 && s.style == i32::from(style))?;
            return self.play_use_sound(idx as u16, x, y, 0.0, rng);
        }
        if sound_type != 17 {
            return None;
        }
        // The attenuation gate first — the int overload draws nothing before
        // the switch (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(17, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        // Case 17 (`:862`): the run-step pitch roll.
        let pitch =
            sound::legacy_family_pitch(sound::Family::Run, &mut |lo, hi| rng.next_range(lo, hi));
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Run", x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some("Run")
    }

    /// S1-E-21f — one player-hurt play (`Player.PlayHurtSound`,
    /// `Player.cs:38569-38712`): the legacy id-1 family (voice variant 1,
    /// the default `Player.cs:56045` arm `:38704`). The local player's cue
    /// is unpositioned (`:38572-38575`: `vector = (-1, -1)`), so the C#'s
    /// attenuation walk takes the `x == -1` flag arm — no distance gate,
    /// only the volume-slider checks.
    ///
    /// Case 1 (`LegacySoundPlayer.cs:503-515`): ONE `Main.rand.Next(3)`
    /// variation pick (`:505`) — `Player_Hit_<n>`, NO pitch draw; the
    /// caller's `pitchOffset` (`Utils.Remap(voicePitchOffset, -1, 1, -0.7,
    /// 0.7)`, `Player.cs:38699-38700`) clamps at `:1410`. The int overload
    /// (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat. The
    /// pick rides the `UpdatePlayers` stream (`Player.Hurt` runs under the
    /// `Main.cs:17754` swap) and only AFTER the gate passed. The instance
    /// Stop + re-create (`:506-510`) is the standing D4 deviation.
    /// Returns the cue played; `None` when the gate refused.
    pub fn play_hurt_sound(
        &mut self,
        pitch_offset: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The unpositioned gate first — the flag arm still checks the
        // volume slider (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(1, -1.0, -1.0, self.listener, 1.0).is_none() {
            return None;
        }
        let slot = sound::player_hit_case1(&mut |lo, hi| rng.next_range(lo, hi));
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let cue = match slot {
            0 => "Player_Hit_0",
            1 => "Player_Hit_1",
            _ => "Player_Hit_2",
        };
        self.emit(cue, -1.0, -1.0, 1.0, pitch_offset.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// The shared case-20 (female hurt) tail: the attenuation gate
    /// (`LegacySoundPlayer.cs:369-436`), the ONE `Main.rand.Next(3)` slot
    /// pick (`:913` — `Female_Hit_<n>`, Volume/Pan only, NO pitch draw),
    /// the caller's `pitchOffset` clamp (`:1410`) and the emit. The
    /// `SoundInstanceFemaleHit[n]` Stop + re-create (`:914-918`) is the
    /// standing D4 deviation. W4-e10.
    fn female_hit_play(
        &mut self,
        x: f32,
        y: f32,
        pitch_offset: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(20, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let slot = rng.next_range(0, 3);
        let cue = match slot {
            0 => "Female_Hit_0",
            1 => "Female_Hit_1",
            _ => "Female_Hit_2",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, 1.0, pitch_offset.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// W4-g22 — one female-hurt play (the menuMode 2 gender toggle's
    /// male->female preview, `Main.cs:49043-49047` `PlaySound(20)`; the
    /// voice-variant-2 hurt arm `Player.cs:38587` is the same case). The
    /// preview is unpositioned (`PlayVoicePreview` parks the player at
    /// `(-1, -1)`, `UICharacterCreation.cs:1522-1524`), so the attenuation
    /// walk takes the `x == -1` flag arm — no distance gate, only the
    /// volume-slider checks. The merge compose: delegates to W4-e10's
    /// shared case-20 tail (the same one `Next(3)` pick, pitchOffset 0 —
    /// the int overload draws NO `GetRandomPitch` NextFloat).
    pub fn play_female_hit_sound(
        &mut self,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        self.female_hit_play(-1.0, -1.0, 0.0, rng)
    }

    /// W4-e10 — the female-voice hurt play (`Player.PlayHurtSound`'s
    /// `voiceOverride 2` arm `Player.cs:38587` / the `voiceVariant 2`
    /// default arm `:38707`): case 20, unpositioned for the local player
    /// (`:38572-38575`), the one `Next(3)` draw rides the `UpdatePlayers`
    /// stream after the gate. The int overload draws no `GetRandomPitch`.
    pub fn play_female_hurt_sound(
        &mut self,
        pitch_offset: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        self.female_hit_play(-1.0, -1.0, pitch_offset, rng)
    }

    /// W4-e10 — one player-voice STYLE play (`Player.PlayHurtSound`'s
    /// `SoundID.*` arms `Player.cs:38590-38648`/`:38686`/`:38696` and
    /// `PlayDeathSound`'s `:38724-38736`): the wave-A façade
    /// (`SoundEngine.cs:93-99` — the `GetRandomPitch` NextFloat on the
    /// audio-only stream, the gate, then case 2 for the id-2 rows
    /// (BellHurt) or case 42 for the trackables — the pinned first
    /// variation, no gameplay draw). `pitch_offset` is the C# call's
    /// offset (the DST world arm and the default-voice arm pass the
    /// remapped `voicePitchOffset`; the override arms pass 0).
    pub fn play_voice_style(
        &mut self,
        style_name: &str,
        x: f32,
        y: f32,
        pitch_offset: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let idx = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.name == style_name)?;
        self.play_use_sound(idx as u16, x, y, pitch_offset, rng)
    }

    /// W4-e10 — one player-hurt NPC-family play (`Player.PlayHurtSound`'s
    /// mount/armor arms, `Player.cs:38655`/`:38660`/`:38665`/`:38670`/
    /// `:38675`/`:38681`/`:38691`: `PlaySound(3|4, vector, style)` — the
    /// int overload, NO `GetRandomPitch`). Case 3 (`LegacySoundPlayer.cs:
    /// 646-672`): the 20..54/55/56 ×0.5 volume arms (the Basilisk's 47 and
    /// the Goat/Slime's 4), the ONE `Next(-10, 11) * 0.01` pitch draw
    /// (`:670`) on the `UpdatePlayers` stream after the gate, the slot
    /// Stop + re-create (`:663-667`).
    pub fn play_hurt_npc_int(
        &mut self,
        sound_id: u16,
        style_num: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let cue = match (sound_id, style_num) {
            (3, 2) => "NPC_Hit_2",
            (3, 5) => "NPC_Hit_5",
            (3, 6) => "NPC_Hit_6",
            (3, 47) => "NPC_Hit_47",
            (4, 4) => "NPC_Killed_4",
            _ => return None,
        };
        let style = npc_sound_style(cue, if sound_id == 3 { "NPC_Hit_" } else { "NPC_Killed_" }, sound_id as u8)?;
        if sound_id == 3 {
            self.npc_hit_play(cue, style, 0.0, x, y, rng)
        } else {
            self.npc_death_play(cue, style, 0.0, x, y, rng)
        }
    }

    /// S1-E-21f — the player-death play (`Player.PlayDeathSound`,
    /// `Player.cs:38715-38746`: the default arm `PlaySound(5,
    /// base.position)` `:38745` — the TOP-LEFT position, not the centre).
    /// Case 5 (`LegacySoundPlayer.cs:696-705`): Volume/Pan only — NO
    /// draws, no pitch. The int overload draws no `GetRandomPitch`. The
    /// `SoundInstancePlayerKilled` Stop + re-create (`:697-701`) is the
    /// standing D4 deviation. Returns the cue; `None` when the gate
    /// refused.
    pub fn play_death_sound(&mut self, x: f32, y: f32) -> Option<&'static str> {
        if sound::play_sound(5, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Player_Killed", x, y, 1.0, 0.0);
        Some("Player_Killed")
    }

    /// S1-E-21f — one extra-jump play (the `Player.cs:20793-21004` chain's
    /// eight `PlaySound(16, (int)position.X, (int)position.Y)` sites:
    /// Sandstorm `:20793`, Blizzard `:20803`, Sail `:20815`, Unicorn
    /// `:20871`, WallOfFleshGoat `:20899`, Basilisk `:20927`, Santank
    /// `:20957`, Cloud `:21004`). The int overload (`SoundEngine.cs:102-108`)
    /// draws NO `GetRandomPitch` NextFloat.
    ///
    /// Case 16 (`LegacySoundPlayer.cs:843-853`): ONE
    /// `Main.rand.Next(-10, 11) * 0.01` pitch draw (`:851`) on the
    /// `UpdatePlayers` stream (`Player.Update`'s `Main.cs:17754` swap),
    /// only AFTER the attenuation gate passed. The
    /// `SoundInstanceDoubleJump` Stop + re-create is the standing D4
    /// deviation. Returns the cue; `None` when the gate refused.
    pub fn play_double_jump_sound(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The attenuation gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(16, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let pitch = sound::legacy_family_pitch(sound::Family::DoubleJump, &mut |lo, hi| {
            rng.next_range(lo, hi)
        });
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Double_Jump", x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some("Double_Jump")
    }

    /// S1-E-21f — one splash play (the player entry/exit sites,
    /// `Player.cs:27806`/`:27819`/`:27832`/`:27846` + the `:27900-27940`
    /// exit twins: `PlaySound(19, (int)position.X, (int)position.Y[, Style])`
    /// — water style 0, honey/lava the default style 1, shimmer styles
    /// 2 (entry) / 3 (exit)). The int overload draws NO `GetRandomPitch`.
    ///
    /// Case 19 (`LegacySoundPlayer.cs:871-910`): the
    /// `SoundInstanceSplash[style]` playing guard (`:872-874`) refuses
    /// BEFORE any draw; styles 2/3 cut ×0.75, every style rolls ONE
    /// `Next(-10, 11) * 0.01` pitch (`:888`) on the `UpdatePlayers`
    /// stream, after the attenuation gate. The slot is overwritten
    /// (detached, the case creates a fresh instance `:876`). Returns the
    /// cue played; `None` when the gate or the playing guard refused.
    pub fn play_splash_sound(
        &mut self,
        style: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if !(0..6).contains(&style) {
            return None;
        }
        // The attenuation gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(19, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let playing = self.splash_slots[style as usize]
            .as_ref()
            .is_some_and(|s| !s.empty());
        let (tweak, pitch) =
            sound::splash_case19(style, playing, &mut |lo, hi| rng.next_range(lo, hi))?;
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let cue = match style {
            0 => "Splash_0",
            1 => "Splash_1",
            2 => "Splash_2",
            3 => "Splash_3",
            4 => "Splash_4",
            _ => "Splash_5",
        };
        // `:876` — the array entry is replaced; the old instance was
        // refused by the guard, so nothing to detach in practice.
        let sink = self.emit(cue, x, y, tweak, pitch.clamp(-1.0, 1.0));
        if let Some(old) = self.splash_slots[style as usize].take() {
            old.detach();
        }
        self.splash_slots[style as usize] = sink;
        Some(cue)
    }

    /// S1-E-21f — one pickup chime (`Player.GetItem`'s four fill sites,
    /// `Player.cs:39444`/`:39491`/`:39534`/`:39559`:
    /// `PlaySound(newItem.IsACoin ? 38 : 7)` — UNPOSITIONED, the flag arm).
    /// The int overload draws NO `GetRandomPitch` NextFloat.
    ///
    /// Case 7 (`LegacySoundPlayer.cs:717-727`): ONE `Next(-10, 11) * 0.01`
    /// pitch (`:725`). Case 38 (`:1337-1350`): `Main.starGame` cuts ×0.15,
    /// then TWO draws — `Next(5)` slot, `Next(-40, 41) * 0.002` pitch. The
    /// draws ride the caller's stream: the world-drop pickup runs inside
    /// `Player.GrabItems` → `Player.PickupItem` under the `Main.cs:17754`
    /// `UpdatePlayers` swap; the chest/inventory transfers run from the
    /// click handlers OUTSIDE every swap (the bare-`Main.rand` stand-in,
    /// the wave-C net-strike precedent). Both run only AFTER the gate
    /// passed. The Grab/Coin instance overwrites are the standing D4
    /// deviation. Returns the cue played; `None` when the gate refused.
    pub fn play_pickup_sound(
        &mut self,
        is_coin: bool,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let id = if is_coin { 38 } else { 7 };
        // The unpositioned gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(id, -1.0, -1.0, self.listener, 1.0).is_none() {
            return None;
        }
        let (cue, tweak, pitch) = if is_coin {
            // Case 38 — the port has no `Main.starGame` (the star seed is
            // unported; ANALYSIS S1-E-21f), so the ×0.15 arm is off.
            let (slot, tweak, pitch) =
                sound::coin_pickup_case38(false, &mut |lo, hi| rng.next_range(lo, hi));
            let cue = match slot {
                0 => "Coin_0",
                1 => "Coin_1",
                2 => "Coin_2",
                3 => "Coin_3",
                _ => "Coin_4",
            };
            (cue, tweak, pitch)
        } else {
            let pitch =
                sound::legacy_family_pitch(sound::Family::Grab, &mut |lo, hi| rng.next_range(lo, hi));
            ("Grab", 1.0, pitch)
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, -1.0, -1.0, tweak, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-P-i43 — the chest eat-animation chime (`Chest.UpdateChestFrames`,
    /// `Chest.cs:3628-3631`: `PlaySound(7, new Vector2(chest.x * 16 + 16,
    /// chest.y * 16 + 16))` at `eatingAnimationTime == 9 && frame == 1`) —
    /// POSITIONED case 7: the attenuation gate first, then the same
    /// `Next(-10, 11) * 0.01` pitch draw as [`Audio::play_pickup_sound`].
    /// The `Main.cs:17810` call site sits outside every SwapRandom block, so
    /// the caller passes the bare-`Main.rand` stand-in (`rngs.global()`, the
    /// Lead's faithful-cue decision). Dormant until the eat timer's producer
    /// (`AskForChestToEatItem`) lands. Returns the cue played; `None` when
    /// the gate refused (no draw spent).
    pub fn play_chest_eat_sound(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The attenuation gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(7, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let pitch =
            sound::legacy_family_pitch(sound::Family::Grab, &mut |lo, hi| rng.next_range(lo, hi));
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Grab", x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some("Grab")
    }

    /// S1-E-21f — the revenge pickup's coin ping (`NPC.moneyPing`,
    /// `NPC.cs:90052-90054`: `PlaySound(38, pos)` at the DROP's top-left
    /// position — positioned, unlike the inventory chime). Same case-38
    /// draws as [`Audio::play_pickup_sound`]; the caller passes the
    /// `UpdateItems` stream (`WorldItem.UpdateItem` runs under the
    /// `Main.cs:18024` swap). The dust burst (`:90068-90072`) is a visual
    /// seam. Returns the cue played; `None` when the gate refused.
    pub fn play_coin_ping(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The attenuation gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(38, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let (slot, tweak, pitch) =
            sound::coin_pickup_case38(false, &mut |lo, hi| rng.next_range(lo, hi));
        let cue = match slot {
            0 => "Coin_0",
            1 => "Coin_1",
            2 => "Coin_2",
            3 => "Coin_3",
            _ => "Coin_4",
        };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, tweak, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21f — the drowning gasp (`Player.CheckDrowning`'s breath-zero
    /// arm, `Player.cs:23255-23258`: `PlaySound(23)` — UNPOSITIONED).
    /// Case 23 (`LegacySoundPlayer.cs:947-956`): Volume/Pan only — NO
    /// draws, no pitch; the int overload draws no `GetRandomPitch`. The
    /// `SoundInstanceDrown` Stop + re-create is the standing D4
    /// deviation. Returns the cue; `None` when the gate refused.
    pub fn play_drown_sound(&mut self) -> Option<&'static str> {
        if sound::play_sound(23, -1.0, -1.0, self.listener, 1.0).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Drown", -1.0, -1.0, 1.0, 0.0);
        Some("Drown")
    }

    /// W4-e10 — the full-mana chime (`Player.cs:19552`'s full-bar regen
    /// trigger and `EmitMaxManaEffect` `:49800`: `PlaySound(25)` —
    /// UNPOSITIONED). Case 25 (`LegacySoundPlayer.cs:963-968`):
    /// Volume/Pan only — NO draws, no pitch; the int overload draws no
    /// `GetRandomPitch`. The `SoundInstanceMaxMana` re-create is the
    /// standing D4 deviation. Returns the cue; `None` when the gate
    /// refused.
    pub fn play_max_mana_sound(&mut self) -> Option<&'static str> {
        if sound::play_sound(25, -1.0, -1.0, self.listener, 1.0).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("MaxMana", -1.0, -1.0, 1.0, 0.0);
        Some("MaxMana")
    }

    /// S1-E-21h — one projectile-family cue play: the AI arms' first-tick
    /// latches (`AI_001`'s `:55209-55365` chain, the aiStyle-8/27/28/50
    /// gates), the boomerang whoosh (`AI_003` `:38868-38891`), the Magic
    /// Missile hum (`AI_009` `:43361-43367`), the Light's Bane slash
    /// (`AI_188` `:42231-42239`) and the bomb blast
    /// (`Kill_Bombs_DoUsualKillCode` `:78331`/:78386). The id-2 sites are
    /// `SoundID.ItemN` style plays — the same façade as wave A
    /// (`SoundEngine.cs:93-99`: the `GetRandomPitch` NextFloat on the
    /// audio-only `sound_style_random`, then the attenuation gate
    /// `LegacySoundPlayer.cs:369-436`, then case 2's `Main.rand` pitch
    /// roll). The gameplay draw rides the caller's stream — the pool pass
    /// drains the queue under the `Main.cs:17775` `UpdateProjectiles`
    /// swap. `pitch_offset` is the C# call's third argument (the ported
    /// sites all pass none; the 301 Stacked Discs' −0.33 arm is deferred).
    /// Returns the cue played; `None` when the gate refused or the style
    /// is not a legacy id-2 row or a landed id-42 trackable (the unported
    /// trackable families ride their own façades — deferred, ANALYSIS
    /// S1-E-21h). W4-d21 fold 2: the id-42 arm routes the landed
    /// projectile trackables ([`PROJ_CUSTOM_CUES`]) through the case-42
    /// path (`play_use_sound`, the digtoise joy `Projectile.cs:38517`).
    /// roll). S1-E-21h d76 adds the whip tip's id-3/4 NPC-family cues
    /// (`AI_165_Whip` `:47720`/`:47724`) — the case-3/4 façades. The
    /// gameplay draw rides the caller's stream — the pool pass drains the
    /// queue under the `Main.cs:17775` `UpdateProjectiles` swap.
    /// `pitch_offset` is the C# call's third argument (the ported sites all
    /// pass none; the 301 Stacked Discs' −0.33 arm is deferred).
    /// `volume_scale` is the C# call's fourth argument (`SoundEngine.cs:99`
    /// `volumeScale`): `1.0` for every default-volume site; the Corrupt Whip's
    /// ×0.6 tip cue (`:47720`) and the Lights Bane slash's `0.15 * ai[0]`
    /// (`:42237`) carry their scale here (S1-E-21h d76 DOC-SWEEP — the
    /// post-gate `instance.Volume *= volumeScale`, `LegacySoundPlayer.cs:1411`).
    /// Returns the cue played; `None` when the gate refused or the family is
    /// unported (the case-42 trackables ride their own façades — deferred,
    /// ANALYSIS).
    pub fn play_proj_sound(
        &mut self,
        (id, style): (u16, i32),
        x: f32,
        y: f32,
        pitch_offset: f32,
        volume_scale: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if id == 42 {
            // The trackable case (`LegacySoundPlayer.cs:1376-1381`): the
            // style resolves through the generated rows, but only the
            // landed projectile cues play — the rest stay the documented
            // skip (their machines are unported).
            let idx = crate::soundid::SOUND_STYLES
                .iter()
                .position(|s| s.sound_id == 42 && s.style == style)?;
            let s = &crate::soundid::SOUND_STYLES[idx];
            let base = s.trackable?;
            let cue = if s.variations == 1 {
                base.to_string()
            } else {
                format!("{base}_0")
            };
            if !PROJ_CUSTOM_CUES.contains(&cue.as_str()) {
                return None;
            }
            return self.play_use_sound(idx as u16, x, y, pitch_offset, rng);
        // S1-E-21h d76 — the whip tip's NPC-family cues (`AI_165_Whip`
        // `Projectile.cs:47720`/`:47724`): the Crimson Whip's NPCHit9 (id 3)
        // and the Corrupt Whip's NPCDeath1 (id 4) ride their own case-3/4
        // façades, resolved through the same `npc_sound_style` helper the
        // wave-C/G plays use. The Corrupt Whip's ×0.6 volumeScale rides
        // `volume_scale` (the DOC-SWEEP port — the case-4 tail applies it
        // post-tweak, `LegacySoundPlayer.cs:1411`).
        if id == 3 || id == 4 {
            let (prefix, sid) = if id == 3 { ("NPC_Hit_", 3) } else { ("NPC_Killed_", 4) };
            let cue = format!("{prefix}{style}");
            let style_def = npc_sound_style(&cue, prefix, sid)?;
            // `SoundEngine.cs:99` — the GetRandomPitch NextFloat rides the
            // audio-only stream (variance 0 on every NPCHitN/NPCDeathN row,
            // the draw still happens), then the case gate + pitch roll.
            let draw = self.sound_style_random.next_float();
            let offset = pitch_offset + sound::get_random_pitch(style_def.pitch_variance, draw);
            let name = SFX_CUES.iter().map(|(n, _)| *n).find(|n| *n == cue)?;
            return if id == 3 {
                self.npc_hit_play(name, style_def, offset, volume_scale, x, y, rng)
            } else {
                self.npc_death_play(name, style_def, offset, volume_scale, x, y, rng)
            };

        }
        }        self.play_proj_sound_scaled((id, style), x, y, pitch_offset, 1.0, rng)
    }

    /// S1-E-21h d77 — the `volumeScale`-carrying peer of
    /// [`Audio::play_proj_sound`]: the queue's sixth term. The DD2 Goblin
    /// Bomb's `Kill()` cue (`Projectile.cs:71906`) plays
    /// `SoundID.DD2_GoblinBomb = new LegacySoundStyle(2, 14).WithVolume(0.5f)`
    /// (`SoundID.cs:1596`) — the SAME case-2 style-14 asset as `Item14`
    /// (which the id-2 style lookup resolves), but the C# style object
    /// carries `type.Volume = 0.5`; the queue reproduces that as
    /// `volume_scale = 0.5` (`SoundEngine.cs:99`'s
    /// `type.Volume * volumeScale` with the call's own scale 1). Every
    /// other queued cue keeps 1.0. The scale folds into
    /// [`Audio::play_use_sound_scaled`].
    /// S1-E-21h f1 — the `volumeScale`-carrying peer of
    /// [`Audio::play_proj_sound`]: the queue's sixth term (d77's seam,
    /// `9d35874a`, adopted so both branches push the 6-tuple). The DD2
    /// Goblin Bomb's `Kill()` cue (`Projectile.cs:71906`) plays
    /// (`SoundID.cs:1596`) — the SAME case-2 style-14 asset as `Item14`,
    /// but the style object carries `type.Volume = 0.5`; the queue
    /// reproduces that as `volume_scale = 0.5` (`SoundEngine.cs:99`'s
    /// f1 queued cue keeps 1.0. KEEP-ONE at merge: d77's drain routes the
    /// int ids 0/13/6/4/29 BEFORE this call, so its copy of this fn drops
    /// the int match below — f1 keeps it here (its drain special-cases
    /// only id 0), and at scale 1.0 both shapes play identically.
    pub fn play_proj_sound_scaled(
        &mut self,
        (id, style): (u16, i32),
        x: f32,
        y: f32,
        pitch_offset: f32,
        volume_scale: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // S1-E-21h d75 part 2 — id 42 joins the drain's families: the
        // bobber's sonar bite cue (`SoundID.SonarPotion`, style 311,
        // `Projectile.cs:20751`/:20765) rides wave A's trackable façade —
        // the `GetRandomPitch` NextFloat on the audio-only stream, case 42
        // draws nothing from the pass stream.
        if id != 2 && id != 42 {
        // DEPRECATED-BY-UNION (S1-E-21h f1 RETIRE): the id-0/13/4 match
        // below — d77's drain routes 0/13/6/4/29 BEFORE this call
        // (`world_sim/projectiles.rs:468-485`, unchanged at the moved tip
        // `5f10560b2`), so at merge this match DROPS (at scale 1.0 both
        // shapes play identically). The `(3, 29)` / `(42, 274)` routes
        // BELOW the match are f1-UNIQUE (d77's drain has no id-3/id-42
        // arm — re-grepped at `5f10560b2`) — FOLD them into d77c (either
        // drain-side or here, the part-7 fold companions).
        // S1-E-21h f1 — the `Kill()` tail's int-overload families ride the
        // same queue: id 0 (the Dig thud, `PlaySound(0, pos)` — the int
        // overload's default Style 1 is unused by case 0), id 13 (the
        // Shatter, `PlaySound(13, pos)`) and id 4 (the NPCDeath style plays,
        // `PlaySound(4, pos, style)`). Each delegates to its landed façade —
        // attenuation BEFORE the case draws, the gameplay draws on the
        // caller's (`UpdateProjectiles`) stream. The int overload draws NO
        // `GetRandomPitch` NextFloat (`SoundEngine.cs:102-108`) and every
        // f1 int site passes the stock scale 1 — `volume_scale` is unused
        // on these arms.
        match id {
            0 => return self.play_place_sound((0, 0), x, y, rng),
            13 => return self.play_wall_break_sound((13, 0), x, y, rng),
            4 => return self.play_npc_death_int(style, x, y, rng),
            _ => {}
        }
        // S1-E-21h f1 final phase — the queue's extended int/trackable
        // routes (the second owed wave): id 3 style 29 (the 1054
        // SpiderBoulder's `PlaySound(SoundID.NPCHit29, x, y)`,
        // `Projectile.cs:74867` — the int overload, case 3's ×0.5 arm +
        // one pitch draw) and id 42 style 274 (the 1047 RainbowBoulder's
        // `PlaySound(SoundID.RainbowBoulder, base.Center)`, `:71194` —
        // the one-variation trackable, case 42, NO gameplay draw). Both
        // rows ride d3's pushed tip; the cues are inert until the merge.
        if id == 3 && style == 29 {
            return self.play_npc_hit_int(29, x, y, rng);
        if id == 42 && style == 274 {
            return self.play_npc_hit_sound("rainbow_boulder", x, y, rng);
        }
        if id != 2 {
        }
            return None;
        }
        let idx = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.sound_id == id as u8 && s.style == style)?;
        self.play_use_sound_scaled(idx as u16, x, y, pitch_offset, volume_scale, rng)
    }

    /// S1-E-21h d75 part 4 — one star-chime play (the 502 Meowmere's
    /// bounce cue, `Projectile.cs:18209`: `PlaySound(37, (int)X, (int)Y,
    /// 5 + (int)ai[0])` — the INT overload, NO GetRandomPitch draw).
    /// Case 37 (`LegacySoundPlayer.cs:1326-1336`): the `Next(57, 59)`
    /// slot pick FIRST, the `Style * 0.05` volume scale (the starGame
    /// ×0.15 arm folds off — the seed is unported, the case-38
    /// precedent), then the ONE `Next(-40, 41) * 0.01` pitch — both on
    /// the caller's `UpdateProjectiles` stream, attenuation gate first.
    /// Returns the cue; `None` when the gate refused.
    pub fn play_star_chime(
        &mut self,
        style: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(37, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let (slot, tweak, pitch) =
            sound::star_chime_case37(style, &mut |lo, hi| rng.next_range(lo, hi));
        let cue = if slot == 57 { "Item_57" } else { "Item_58" };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, tweak, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21h d75 part 4 — one FoxparksFlame play (the 1106's per-life
    /// flame cue, `Projectile.cs:41369`:
    /// `SoundEngine.PlaySound(SoundID.FoxparksFlame, base.Center)`). The
    /// NAMED style (`SoundID.cs:1725`: `new LegacySoundStyle(2, 34)
    /// .WithVolume(0.15f)`) — NOT the plain `Item34` row the `(2, 34)`
    /// tuple resolves to (volume 1.0), so the façade pins the row BY NAME
    /// (the `play_chillet_sound` precedent). The style overload's
    /// `GetRandomPitch` NextFloat rides the audio-only stream (variance 0 —
    /// the draw still happens), the attenuation gate precedes case 2's
    /// one gameplay pitch draw on the caller's `UpdateProjectiles` stream.
    /// Returns the cue; `None` when the gate refused.
    pub fn play_foxparks_flame(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let idx = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.name == "FoxparksFlame")?;
        self.play_use_sound(idx as u16, x, y, 0.0, rng)
    }

    /// S1-E-21h d75 part 2 — one Shatter play with a volume scale (the
    /// Crystal Shard's first-tick cue, `Projectile.cs:25943`:
    /// `PlaySound(13, x, y, 0, 0.35f)` — the int overload, NO `GetRandomPitch`
    /// draw). Case 13 (`LegacySoundPlayer.cs:783-792`): the previous
    /// instance's Stop (the standing D4 deviation), a fresh instance at
    /// `num2 * volumeScale` — NO pitch, NO draws. The attenuation gate runs
    /// first (`:369-436`); the play runs on the `UpdateProjectiles` stream's
    /// drain but draws nothing from it. Returns the cue; `None` when the
    /// gate refused.
    pub fn play_shatter_sound(
        &mut self,
        x: f32,
        y: f32,
        volume_scale: f32,
    ) -> Option<&'static str> {
        if sound::play_sound(13, x, y, self.listener, volume_scale).is_none() {
            return None;
        }
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit("Shatter", x, y, volume_scale, 0.0);
        Some("Shatter")
            .position(|s| s.sound_id == 2 && s.style == style)?;
        self.play_use_sound_scaled(idx as u16, x, y, pitch_offset, volume_scale, rng)
    }

    /// S1-E-21i — one wall-break cue (`tr_core::edit::kill_wall_sound`, the
    /// `WorldGen.KillWall_PlaySounds` table `WorldGen.cs:60423-60441`,
    /// played from `KillWall` `:60385` BEFORE the dust and the drop).
    ///
    /// All four arms are the int overload (`SoundEngine.cs:102-108` — NO
    /// `GetRandomPitch` draw) except the NPCHit25 style arm:
    /// - 13 (Shatter, `LegacySoundPlayer.cs:783-792`): no draws, no pitch.
    /// - 6 (Grass, `:706-716`): one `Main.rand.Next(-30, 31) * 0.01` (`:714`).
    /// - 3 style 25 (`SoundID.NPCHit25`, `SoundID.cs:1281`): wave C's style
    ///   façade — the audio-only `GetRandomPitch` NextFloat, the ×0.5 volume
    ///   arm and the one `Next(-10, 11)` pitch draw (`:670`).
    /// - 0 (Dig, `:440-452`): wave B's case-0 path — the `Next(3)` variation
    ///   pick + the `Next(-10, 11) * 0.01` pitch.
    ///
    /// The gameplay draws ride the caller's stream (the hammer break runs
    /// in `Player.PickWall` `Player.cs:45949` under the `Main.cs:17754`
    /// `UpdatePlayers` swap; the explosion walk `Projectile.cs:78745` under
    /// `:17775` `UpdateProjectiles`; the net replay `MessageBuffer.cs:1177`
    /// on the bare stream) and only AFTER the attenuation gate passed
    /// (`:369-436` precedes the switch). Returns the cue played; `None`
    /// when the gate refused. The instance stop/re-create pools are the
    /// standing D4 deviation.
    pub fn play_wall_break_sound(
        &mut self,
        (id, style): tr_core::edit::SoundCue,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        match id {
            13 => {
                // The attenuation gate first — case 13 draws nothing.
                if sound::play_sound(13, x, y, self.listener, 1.0).is_none() {
                    return None;
                }
                self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.emit("Shatter", x, y, 1.0, 0.0);
                Some("Shatter")
            }
            6 => {
                if sound::play_sound(6, x, y, self.listener, 1.0).is_none() {
                    return None;
                }
                // Case 6 (`:714`): the grass pitch roll.
                let pitch = sound::legacy_family_pitch(sound::Family::Grass, &mut |lo, hi| {
                    rng.next_range(lo, hi)
                });
                self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.emit("Grass", x, y, 1.0, pitch.clamp(-1.0, 1.0));
                Some("Grass")
            }
            // The NPCHit25 style arm — wave C's case-3 façade.
            3 if style == 25 => self.play_npc_hit_sound("NPC_Hit_25", x, y, rng),
            // The default Dig arm — wave B's case-0 path.
            0 => self.play_place_sound((0, 1), x, y, rng),
            _ => None,
        }
    }

    /// W4-c14 - one Meowcart play (`Mount.MeowcartLandingSound`
    /// `Mount.cs:659-662`: `PlaySound(37, x, y, 5)`; `MeowcartBumperSound`
    /// `:664-667`: `PlaySound(37, x, y, 3)`). The int overload
    /// (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat.
    ///
    /// Case 37 (`LegacySoundPlayer.cs:1326-1336`): the `Next(57, 59)` slot
    /// pick (`Item_57`/`Item_58`), the `Style * 0.05` volume scale (the
    /// non-starGame arm) and the `Next(-40, 41) * 0.01` pitch - TWO draws
    /// on the caller's `UpdatePlayers` stream (the cart ride runs in
    /// `Player.Update`, `Main.cs:17754`), only AFTER the attenuation gate
    /// passed. Returns the cue played; `None` when the gate refused.
    pub fn play_meowcart_sound(
        &mut self,
        style: i32,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        if sound::play_sound(37, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let (slot, volume, pitch) =
            sound::meowcart_case37(style, &mut |lo, hi| rng.next_range(lo, hi));
        let cue = if slot == 57 { "Item_57" } else { "Item_58" };
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(cue, x, y, volume, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21i - one door open/close play (`WorldGen.OpenDoor`
    /// `WorldGen.cs:37727`: `PlaySound(8, i * 16, j * 16)`; `CloseDoor`
    /// `:31908`: `PlaySound(9, ...)` — both success-only). The int overload
    /// (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat.
    ///
    /// Cases 8/9 (`LegacySoundPlayer.cs:728-749`): one
    /// `Main.rand.Next(-20, 21) * 0.01` pitch draw (`:736`/`:747`) on the
    /// caller's stream — the door interaction runs in `Player.ItemCheck`
    /// (`Player.cs:32841`/`:32848`/`:32859`'s tile-use path) under the
    /// `Main.cs:17754` `UpdatePlayers` swap — only AFTER the attenuation
    /// gate passed. The
    /// previous instance's Stop (`:729-732`/`:740-743`) is the standing D4
    /// deviation (no guard — a fresh source per play). Returns the cue
    /// played; `None` when the gate refused.
    pub fn play_door_sound(
        &mut self,
        open: bool,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        let id = if open { 8 } else { 9 };
        // The attenuation gate first — the int overload draws nothing
        // before the switch (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(id, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        // Cases 8/9 (`:736`/`:747`): the shared door pitch window.
        let pitch =
            sound::legacy_family_pitch(sound::Family::Door, &mut |lo, hi| rng.next_range(lo, hi));
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let cue = if open { "Door_Opened" } else { "Door_Closed" };
        self.emit(cue, x, y, 1.0, pitch.clamp(-1.0, 1.0));
        Some(cue)
    }

    /// S1-E-21i — one wiring switch click (`Wiring.HitSwitch`'s seven
    /// `SoundEngine.PlaySound(28, x, y, 0)` sites, `Wiring.cs:267`/`:272`/
    /// `:285`/`:310`/`:323`/`:339`/`:375`). The int overload
    /// (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch` NextFloat; the
    /// explicit Style 0 indexes the one-entry `SoundMech` array
    /// (`LegacySoundPlayer.cs:22`/:176).
    ///
    /// Case 28 (`LegacySoundPlayer.cs:997-1007`): the
    /// `SoundInstanceMech[0].State == Playing` guard (`:998-1001`) returns
    /// BEFORE the one `Main.rand.Next(-10, 11) * 0.01` pitch draw (`:1005`)
    /// — [`crate::sound::mech_case28`]. The draw rides the caller's stream:
    /// the stage swap wrapping the `HitSwitch` call (`UpdatePlayers` for
    /// the player/minecart trips and the tile-use click,
    /// `UpdatePressurePlates` for the weighted plates, `UpdateNPCs` for the
    /// NPC trips, `UpdateWorldGen` for the wiring tick and the logic
    /// sensors, the bare stream for the net replays) — never the wiring
    /// stand-in stream. The play overwrites the slot WITHOUT a Stop
    /// (`:1002`). Returns the cue played; `None` when the gate or the
    /// playing guard refused (both draw-free past the gate).
    pub fn play_switch_click(
        &mut self,
        x: f32,
        y: f32,
        rng: &mut tr_core::unified_random::UnifiedRandom,
    ) -> Option<&'static str> {
        // The attenuation gate first (`LegacySoundPlayer.cs:369-436`).
        if sound::play_sound(28, x, y, self.listener, 1.0).is_none() {
            return None;
        }
        let playing_0 = self.mech_slots[0].as_ref().is_some_and(|s| !s.empty());
        let pitch = sound::mech_case28(playing_0, &mut |lo, hi| rng.next_range(lo, hi))?;
        self.played.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // `:1002` — the array entry is replaced with no Stop.
        let old = self.mech_slots[0].take();
        self.mech_slots[0] = self.emit("Mech_0", x, y, 1.0, pitch.clamp(-1.0, 1.0));
        if let Some(old) = old {
            old.detach();
        }
        Some("Mech_0")
    }

    /// `LegacySoundPlayer`'s case 34/35 (`LegacySoundPlayer.cs:1249-1307`): the
    /// two persistent liquid loops whose Volume/Pan update in place every ¼ s
    /// tick (`Main.Ambience`, `Main.cs:13466-13509`). `volume <= 0` stops the
    /// channel outright (the vanilla stops the instance; no fade). The pan
    /// rides shared per-loop gains (`liquid_pans`) so the live instance
    /// re-pans exactly like XNA's `SoundInstanceLiquid[n].Pan = num3`
    /// (`:1267`/`:1297`); `pitch` is the re-volume arm's
    /// `Pitch = -0.2f` (case 34, `:1268`) / `-0f` (case 35, `:1298`) — XNA
    /// octaves, so the sink's speed is `2^pitch` (the fresh-create arms pass
    /// `None` and keep the default rate, `:1272-1275`/`:1302-1305`).
    pub fn set_liquid_loop(&mut self, channel: usize, volume: f32, pan: f32, pitch: Option<f32>) {
        let Some((channels, rate, samples)) = self.liquid_cues.get(channel).cloned().flatten()
        else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let (left, right) = crate::sound::pan_gains(pan);
        if volume <= 0.0 {
            if let Some(Some(sink)) = self.liquid_loops.get_mut(channel) {
                sink.stop();
                self.liquid_loops[channel] = None;
            }
            return;
        }
        if let Some(Some(sink)) = self.liquid_loops.get_mut(channel) {
            sink.set_volume(volume);
            // The live re-pan (`:1267`/`:1297`): the shared gains the
            // playing source reads per sample.
            if let Some(gains) = &self.liquid_pans[channel] {
                gains.store_left(left);
                gains.store_right(right);
            }
            if let Some(p) = pitch {
                sink.set_speed(2.0f32.powf(p.clamp(-1.0, 1.0)));
            }
            return;
        }
        // A fresh looped sink: the decoded PCM wrapped in an infinite repeat.
        let gains = LiquidPanGains::new(left, right);
        let buf =
            rodio::buffer::SamplesBuffer::new(channels, rate, samples).convert_samples::<f32>();
        let source = PannedShared {
            inner: buf.repeat_infinite(),
            channels,
            gains: gains.clone(),
            channel: 0,
        };
        if let Ok(sink) = rodio::Sink::try_new(handle) {
            sink.append(source);
            sink.set_volume(volume);
            self.liquid_loops[channel] = Some(sink);
            self.liquid_pans[channel] = Some(gains);
        }
    }

    /// `SoundInstanceLiquid[n] != null && State == Playing`
    /// (`LegacySoundPlayer.cs:1264`/`:1294`): the re-volume arm's guard —
    /// headless (no output device) nothing ever plays, the fresh-create
    /// convention. A stopped-but-not-reaped sink reads as finished.
    pub fn liquid_loop_playing(&self, channel: usize) -> bool {
        self.liquid_loops
            .get(channel)
            .and_then(|s| s.as_ref())
            .is_some_and(|sink| !sink.empty())
    }

    /// `WorldGen.StopWaterfallAmbienceAudio` (`WorldGen.cs:6689-6699`): the
    /// `PlaySound(34, -1, -1, 0)` + `PlaySound(35, -1, -1, 0)` pair — the
    /// sentinel stop arm of both liquid-loop cases (`LegacySoundPlayer.cs:
    /// 1257-1262`/:1287-1292: `x == -1 || y == -1` ORs into the stop
    /// condition BEFORE the volume walk matters and `Stop()`s the live
    /// `SoundInstanceLiquid[n]` outright — no attenuation, no fade, no RNG:
    /// the case bodies never reach a pitch draw). Called by `SaveAndQuit`
    /// (`:6704`) and inlined by `JustQuit` (`:6631-6632`) BEFORE
    /// `StopTrackedSounds`, so the loops die at the quit moment rather than
    /// at the next ¼ s `Main.Ambience` re-volume tick. The port kills the
    /// sinks DIRECTLY (the g24 shape — `set_liquid_loop(ch, 0.0)`'s stop arm
    /// gates on the decoded cue + output handle first, so the headless
    /// no-device path would never reach it); stopping a dead slot is a
    /// no-op, the C# null/Playing guard's equivalent.
    /// Keep-one (W4-g73 retire): the g24 doc-sweep `c3eccb77` and this
    /// branch's closeout `4e7e3f58` both wired `:6704` — ONE seam survives
    /// (this fn + the one `save_and_quit` call), g24's counter seam kept,
    /// the closeout's route-through-`set_liquid_loop` body retired.
    /// 1257-1262`/:1287-1292: `x == -1 || y == -1` stops the instance
    /// outright, no attenuation, no fade). Called by `SaveAndQuit`
    /// at the next ¼ s `Main.Ambience` re-volume tick (the frame's
    /// menu-mode quiet arm covers the same effect a quarter-second later;
    /// this seam is the immediate one). No RNG: the sentinel plays draw
    /// NOTHING (the case bodies never reach a pitch draw).
    pub fn stop_waterfall_ambience(&mut self) {
        self.liquid_ambience_stops += 1;
        for channel in 0..2 {
            if let Some(Some(sink)) = self.liquid_loops.get_mut(channel) {
                sink.stop();
                self.liquid_loops[channel] = None;
            }
        }
    }

    /// The `StopWaterfallAmbienceAudio` call count (the headless test seam,
    /// the `played_count` convention).
    #[allow(dead_code)] // test-only reader on this branch (the quit witnesses)
    pub fn liquid_ambience_stop_count(&self) -> u64 {
        self.liquid_ambience_stops
    }

    /// `SoundEngine.StopTrackedSounds` (`SoundEngine.cs:151-157` →
    /// `SoundPlayer.StopAll`, `SoundPlayer.cs:79-86`): every tracked
    /// instance stops and the pool clears. The `!Main.dedServ &&
    /// IsAudioSupported` gate is the port's always-true stand-in (the
    /// headless pool still tracks entries). Called by `SaveAndQuit`
    /// (`WorldGen.cs:6715`) and `JustQuit` (`:6646`).
    pub fn stop_tracked_sounds(&mut self) {
        self.tracked.stop_all();
    }

    /// `SoundEngine.PlayTrackedSound(SoundStyle)`'s global arm
    /// (`SoundEngine.cs:114-122` + `SoundPlayer.cs:44-52`): a global
    /// tracked play — no position, so `ActiveSound` skips the attenuation
    /// and the pan (`ActiveSound.cs:63-66`). S1-E-23.
    pub fn play_tracked_global(&mut self, style: crate::tracked_sound::TrackedStyle) -> tr_core::slot::SlotId {
        self.play_tracked_inner(style, None, Default::default(), false, None)
    }

    /// `SoundEngine.PlayTrackedSound(SoundStyle, Vector2, overrides)`
    /// (`SoundEngine.cs:103-112` + `SoundPlayer.cs:11-23`): the positioned
    /// tracked play — the 10 000-px screen-centre guard
    /// (`SoundPlayer.cs:17-20`'s `DistanceSquared > 100000000`) and the
    /// `MaxTrackedInstances` cap (`SoundEngine.cs:104-107`) run before the
    /// pool add. S1-E-23.
    pub fn play_tracked(
        &mut self,
        style: crate::tracked_sound::TrackedStyle,
        x: f32,
        y: f32,
        overrides: crate::tracked_sound::SoundPlayOverrides,
    ) -> tr_core::slot::SlotId {
        // `SoundPlayer.cs:17-20`: the distance guard against the screen
        // centre (`Main.screenPosition + screenWidth/2, screenHeight/2` —
        // the port's camera centre).
        let dx = x - self.listener.camera_center.0;
        let dy = y - self.listener.camera_center.1;
        if dx * dx + dy * dy > 100000000.0 {
            return tr_core::slot::SlotId::INVALID;
        }
        self.play_tracked_inner(style, Some((x, y)), overrides, false, None)
    }

    /// `SoundEngine.PlayTrackedLoopedSound` (`SoundEngine.cs:133-144` +
    /// `SoundPlayer.cs:25-37`): the looped tracked play with its
    /// `LoopedPlayCondition` (`ActiveSound.cs:91-101`). S1-E-23.
    pub fn play_tracked_looped(
        &mut self,
        style: crate::tracked_sound::TrackedStyle,
        x: f32,
        y: f32,
        condition: crate::tracked_sound::LoopedPlayCondition,
        overrides: crate::tracked_sound::SoundPlayOverrides,
    ) -> tr_core::slot::SlotId {
        let dx = x - self.listener.camera_center.0;
        let dy = y - self.listener.camera_center.1;
        if dx * dx + dy * dy > 100000000.0 {
            return tr_core::slot::SlotId::INVALID;
        }
        self.play_tracked_inner(style, Some((x, y)), overrides, true, Some(condition))
    }

    /// The tracked plays' shared tail (`ActiveSound.cs:45-101`): the
    /// variation pick (`LegacySoundStyle.cs:21-28`, the engine's
    /// [`Audio::style_random`]), the pitch roll (`SoundStyle.cs:40-43`, the
    /// engine's [`Audio::sound_style_random`]), the sink at the intended
    /// initial gain (`:85`), then the pool add.
    fn play_tracked_inner(
        &mut self,
        style: crate::tracked_sound::TrackedStyle,
        position: Option<(f32, f32)>,
        overrides: crate::tracked_sound::SoundPlayOverrides,
        looped: bool,
        condition: Option<crate::tracked_sound::LoopedPlayCondition>,
    ) -> tr_core::slot::SlotId {
        // `SoundEngine.cs:104-107`: the cap.
        if self.tracked.style_is_at_cap(&style) {
            return tr_core::slot::SlotId::INVALID;
        }
        let asset = style.random_sound(&mut self.style_random);
        let pitch = style.get_random_pitch(&mut self.sound_style_random);
        let Some((channels, rate, samples)) = self.cues.get(asset.as_str()) else {
            return tr_core::slot::SlotId::INVALID;
        };
        let is_global = position.is_none();
        let pos = position.unwrap_or((0.0, 0.0));
        // `ActiveSound.cs:148`'s pan — baked at creation (the engine
        // deviation: the vanilla re-pans the live instance every update).
        let pan = if is_global {
            0.0
        } else {
            let dx = pos.0 - self.listener.camera_center.0;
            (dx / (crate::sound::MAX_WORLD_VIEW_WIDTH * 0.5)).clamp(-1.0, 1.0)
        };
        let mut active = crate::tracked_sound::ActiveSound::new(
            style,
            pos,
            is_global,
            overrides,
            condition,
            None,
            pan,
            pitch,
        );
        // `:85`'s `Volume = DetermineIntendedVolume()` — the initial gain
        // the sink starts at (the per-frame update keeps it live).
        let initial_gain = active.determine_intended_volume(&self.listener);
        if let Some(handle) = &self.handle {
            if let Ok(s) = rodio::Sink::try_new(handle) {
                let buf = rodio::buffer::SamplesBuffer::new(*channels, *rate, samples.clone())
                    .convert_samples::<f32>();
                let (left, right) = sound::pan_gains(pan);
                if looped {
                    // `ActiveSound.cs:96`'s `IsLooped = true`.
                    s.append(Panned {
                        inner: buf.repeat_infinite(),
                        channels: *channels,
                        left: initial_gain * left,
                        right: initial_gain * right,
                        channel: 0,
                    });
                } else {
                    s.append(Panned {
                        inner: buf,
                        channels: *channels,
                        left: initial_gain * left,
                        right: initial_gain * right,
                        channel: 0,
                    });
                }
                s.set_volume(initial_gain.clamp(0.0, 1.0));
                active.attach_sink(s);
            }
        }
        self.tracked.add(active)
    }

    /// `SoundEngine.Update` (`SoundEngine.cs:48-68`): the pause edge
    /// (`FocusHelper.PauseSounds` — `!GameplayActive && netMode == 0`, the
    /// caller's seam) then the pool's per-frame update
    /// (`SoundPlayer.cs:88-105`, which re-computes every `ActiveSound`'s
    /// volume against the live listener and reaps the dead).
    pub fn update_tracked(&mut self, pause_sounds: bool) {
        self.tracked.update_pause_state(pause_sounds);
        self.tracked.update(&self.listener);
    }

    /// `SoundPlayer.GetActiveSound` (`SoundPlayer.cs:54-61`) — the callers
    /// hold the `SlotId` and mutate the sound through it
    /// (`SceneState.cs:233`'s `activeSound3.Volume = ...`).
    pub fn get_active_sound(
        &self,
        id: tr_core::slot::SlotId,
    ) -> Option<&crate::tracked_sound::ActiveSound> {
        self.tracked.get_active_sound(id)
    }

    pub fn get_active_sound_mut(
        &mut self,
        id: tr_core::slot::SlotId,
    ) -> Option<&mut crate::tracked_sound::ActiveSound> {
        self.tracked.get_active_sound_mut(id)
    }

    /// `SoundEngine.FindActiveSound(style)` (`SoundEngine.cs` →
    /// `SoundPlayer.FindActiveSound`): the first live instance of the style,
    /// or `None` — the achievement-completion subscriber's "already playing"
    /// guard (`AchievementInitializer.cs:731`).
    pub fn find_active_tracked(
        &self,
        style: &crate::tracked_sound::TrackedStyle,
    ) -> Option<&crate::tracked_sound::ActiveSound> {
        self.tracked.find_active_sound(style)
    }

    /// The pool-entry count of one tracked style (no `IsPlaying` filter —
    /// the `find_active_sound` semantics) — the headless test seam: no
    /// output device drains an entry, so the count is the number of plays
    /// landed (the W4-g24 quit-path witnesses, kept by the keep-one).
    #[allow(dead_code)] // test-only reader on this branch (the quit witnesses)
    /// The pool-entry count of one tracked style (no `IsPlaying` filter) —
    /// the headless test seam: no output device drains an entry, so the
    /// count is the number of plays landed (W4-g24's seed-submit witness).
    pub fn tracked_count_for(&self, style: &crate::tracked_sound::TrackedStyle) -> usize {
        self.tracked.count_style_entries(style)
    }

    /// The test seam: register a silent one-sample cue under `name` so the
    /// tracked/slot paths can run headless (the real decode needs the
    /// retail Content junction; `Audio::new(None)` loads nothing). The
    /// vanilla `SoundEngine.IsAudioSupported` gate is the port's
    /// always-true headless stand-in — a registered cue plays.
    #[cfg(test)]
    pub(crate) fn insert_test_cue(&mut self, name: &'static str) {
        self.cues.insert(name, (1, 22050, vec![0]));
    }
}

fn load_sound(path: &Path) -> anyhow::Result<tr_content::Sound> {
    tr_content::load_sound(path)
}

/// `SoundEffectInstance.Pan` + `Volume`: scale each channel by its gain.
///
/// XNA applies `Pan` in XAudio2, which the decompilation does not contain, so the
/// reference pan law from [`crate::sound::pan_gains`] is used.
struct Panned<S> {
    inner: S,
    channels: u16,
    left: f32,
    right: f32,
    channel: u16,
}

/// The liquid loops' live pan gains — the XNA `SoundEffectInstance.Pan`
/// write (`LegacySoundPlayer.cs:1267`/`:1297`) reaches the PLAYING source:
/// the audio thread reads the atomics per sample while the ¼ s ambience
/// tick stores the new `pan_gains` pair. `Arc` clones share one cell.
#[derive(Clone)]
struct LiquidPanGains {
    left: std::sync::Arc<std::sync::atomic::AtomicU32>,
    right: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl LiquidPanGains {
    fn new(left: f32, right: f32) -> Self {
        use std::sync::atomic::AtomicU32;
        Self {
            left: std::sync::Arc::new(AtomicU32::new(left.to_bits())),
            right: std::sync::Arc::new(AtomicU32::new(right.to_bits())),
        }
    }

    fn store_left(&self, v: f32) {
        self.left.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    fn store_right(&self, v: f32) {
        self.right.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    fn load(&self) -> (f32, f32) {
        use std::sync::atomic::Ordering;
        (
            f32::from_bits(self.left.load(Ordering::Relaxed)),
            f32::from_bits(self.right.load(Ordering::Relaxed)),
        )
    }
}

/// [`Panned`] with shared live gains — the liquid-loop source shape.
struct PannedShared<S> {
    inner: S,
    channels: u16,
    gains: LiquidPanGains,
    channel: u16,
}

impl<S: Iterator<Item = f32>> Iterator for PannedShared<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let (left, right) = self.gains.load();
        let gain = match self.channel {
            0 => left,
            1 => right,
            _ => 1.0,
        };
        self.channel = if self.channels == 0 {
            0
        } else {
            (self.channel + 1) % self.channels
        };
        Some(sample * gain)
    }
}

impl<S: Source<Item = f32>> Source for PannedShared<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

impl<S: Iterator<Item = f32>> Iterator for Panned<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let gain = match self.channel {
            0 => self.left,
            1 => self.right,
            _ => 1.0,
        };
        self.channel = if self.channels == 0 {
            0
        } else {
            (self.channel + 1) % self.channels
        };
        Some(sample * gain)
    }
}

impl<S: Source<Item = f32>> Source for Panned<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// Global output handle shared with [`Audio`] (the XACT `audioSystem` singleton).
static MUSIC_HANDLE: std::sync::Mutex<Option<rodio::OutputStreamHandle>> =
    std::sync::Mutex::new(None);

/// The per-frame world state `Main.UpdateAudio_DecideOnNewMusic` reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct MusicEnv {
    /// `Main.musicVolume`.
    pub music_volume: f32,
    /// `Main.dayTime`.
    pub day: bool,
    /// Scene centre below the world surface (`SceneMetrics.Center.Y >= worldSurface*16
    /// + screenHeight/2`), i.e. the `Main.cs:12815` branch.
    pub underground: bool,
    /// `Main.cloudAlpha > 0` (raining/storming).
    /// `Main.cloudAlpha` (the rain coverage the volume follows).
    pub cloud_alpha: f32,
    pub rain: bool,
    // — the M1.4-BG-20 ambience-track inputs (`Main.cs:11527-11600`) —
    /// `SceneMetrics.ZoneRain`.
    pub zone_rain: bool,
    /// `SceneMetrics.ZoneSnow`.
    pub zone_snow: bool,
    /// `SceneMetrics.SurfaceAtmospherics`.
    pub surface_atmospherics: bool,
    /// `Main.windSpeedCurrent`.
    pub wind_speed: f32,
    /// `Main.shimmerAlpha` (the shimmer mutes both ambience tracks).
    pub shimmer_alpha: f32,
    /// `Main.atmo` (the depth factor, `Main.cs:63185`).
    pub atmo: f32,
    /// `Main.ambientVolume` (the user slider).
    pub ambient_volume: f32,
    /// Music id of the nearest active boss (`NPCID → newMusic`), if any.
    pub boss_music: Option<u32>,
    /// `Main.time` (0..54000).
    pub clock: f64,
    // — the M6-MUSIC-1 chain inputs (`UpdateAudio_DecideOnNewMusic`,
    // `Main.cs:12187-12988`), in ladder order —
    /// `Main.eclipse` with the surface gate (`:12644`).
    pub eclipse: bool,
    /// `SceneMetrics.InTorchGodMinigame` (`Main.cs:12542`, W4-f33): the Torch
    /// God event runs → track 101, ABOVE the boss-flag ladder (the C# order:
    /// the credits-roll 89 arm, then this, then the bosses).
    pub in_torch_god_minigame: bool,
    /// `SceneMetrics.ZoneShimmer` (`:12668`).
    pub shimmer: bool,
    /// `SceneMetrics.TownNPCCount` (`:12199`) — 0 until M4-TOWN lands.
    pub town_npcs: i32,
    /// `SceneMetrics.ZoneShadowCandle` — the town-music cut (`:12199`).
    pub shadow_candle: bool,
    /// `SceneMetrics.ZoneSandstorm` (`:12680`).
    pub sandstorm: bool,
    /// The scene centre below `Main.UnderworldLayer` (`:12682`).
    pub underworld: bool,
    /// `Main.bloodMoon` — the night tail (`:12968`).
    pub blood_moon: bool,
    /// `Main._shouldUseWindyDayMusic` (`:12961`).
    pub windy_music: bool,
    // — the M6-AUDIO-2 inputs (`Main.cs:11076-11350`) —
    /// `SceneMetrics.ActiveMusicBox`: the box id playing, -1 = none.
    pub music_box: i32,
    /// `SceneMetrics.MusicBoxSilence` — the silent box cuts the music.
    pub music_box_silence: bool,
    // — the M6-MUSIC-biome-tracks inputs (`:12686-12920`), ladder order —
    /// The space band: `num5 < 1f` with
    /// `num5 = (centerTileY - (65 + 10*(maxTilesX/4200)^2)) / (worldSurface/5)`
    /// (`:12533-12534`).
    pub space_band: bool,
    /// The Lihzahd temple wall behind the scene centre (`:12690`, wall 87).
    pub temple_wall: bool,
    /// `SceneMetrics.ZoneDungeon` (`:12694`).
    pub dungeon: bool,
    /// The mushroom background: `bgStyle == 9` on the surface band
    /// (`:12700`; `undergroundBackground` is pinned 0 in the port).
    pub mushroom_bg: bool,
    /// `SceneMetrics.ZoneCorrupt` (`:12702`).
    pub corrupt: bool,
    /// `SceneMetrics.ZoneCrimson` (`:12731`).
    pub crimson: bool,
    /// The corrupt/crimson split's blood count lead (`:12704`).
    pub blood_leads_evil: bool,
    /// `SceneMetrics.ZoneMeteor` (`:12744`).
    pub meteor: bool,
    /// `SceneMetrics.ZoneGraveyard` (`:12746`).
    pub graveyard: bool,
    /// `SceneMetrics.ZoneDesert` (`:12750`).
    pub desert: bool,
    /// `SceneMetrics.ZoneJungle` (`:12752`).
    pub jungle: bool,
    /// `SceneMetrics.ZoneSnow` (`:12792`).
    pub snow: bool,
    /// `SceneMetrics.ZoneHallow` (`:12816/:12875`).
    pub hallow: bool,
    /// `Main._shouldUseStormMusic` (`:12887`).
    pub storm: bool,
    /// `WorldGen.oceanDepths` at the screen centre (`:12896`).
    pub ocean: bool,
    /// The depth bands in pixels: `rockLayer*16 + screenHeight/2`,
    /// `(rockLayer-50)*16 + screenHeight/2` (the jungle's hysteresis),
    /// `worldSurface*16 + screenHeight/2` — plus the scene centre Y.
    pub center_y: f32,
    pub surface_band_px: f32,
    pub rock_band_px: f32,
    pub jungle_deep_band_px: f32,
    /// `Main.cloudAlpha == 0 && !windy` — the town arm's quiet-sky gate.
    pub quiet_sky: bool,
    /// `Main.gameMenu` (`Main.cs:12455`) — the front-end screens, where the
    /// title-track ladder runs instead of every world branch (BUG-01).
    pub menu: bool,
    /// The worldgen seed flags the menu ladder reads (`WorldGen.drunkWorldGen`
    /// and friends, `Main.cs:12463-12494`) — on the front end they come from
    /// the world-creation screen's seed text.
    pub menu_flags: tr_core::seeds::SeedFlags,
    // — the W4-g22 MENU-FINAL menu-track arms (`Main.cs:12465-12489`) —
    /// The micro-seed flags beside the classic ones: `Main.vampireSeed` is
    /// `SecretSeed.vampirism.Enabled` (`WorldGen.cs:555-558`), read by the
    /// drunk arm's vampire title track (`:12469-12472`).
    pub menu_micro_flags: tr_core::seeds::MicroSeedFlags,
    /// `Main.menuMode == 3000` (`:12486-12489`): the credits page plays
    /// JourneysEnd (89) over every title track except the world-seed ones.
    pub menu_credits: bool,
}

/// `Main.onlyShimmerOceanWorldsGeneration` (`Main.cs:2686-2696`): the
/// property is `drunkWorldGen && tenthAnniversaryWorldGen &&
/// !remixWorldGen && !everythingWorldGen && !notTheBees` — pure flag
/// tests over the same `WorldGen.*` statics the menu ladder reads, so
/// the front-end's seed flags decide it (W4-g22 MENU-FINAL, the 72 arm).
fn shimmer_ocean_generation(flags: tr_core::seeds::SeedFlags) -> bool {
    use tr_core::seeds::SeedFlags;
    flags.contains(SeedFlags::DRUNK)
        && flags.contains(SeedFlags::TENTH_ANNIVERSARY)
        && !flags.contains(SeedFlags::REMIX)
        && !flags.contains(SeedFlags::ZENITH)
        && !flags.contains(SeedFlags::NOT_THE_BEES)
}

/// MS-ADPCM music player using the XACT wave bank.
///
/// This ports the XACT track lifecycle: `AudioTracks[i]` are the rodio [`Sink`]s,
/// `musicFade[i]` is `Main.musicFade`, and `newMusic`/`curMusic` follow
/// `Main.UpdateAudio` (`Main.cs:11017-11606`).
pub struct MusicSystem {
    wavebank_path: Option<PathBuf>,
    /// `Music_{id}` → wave bank entry index, from `Sound Bank.xsb`.
    wave_indices: HashMap<u32, usize>,
    /// `Main.curMusic` / `Main.newMusic`. Public for the front-end tick's
    /// observability (BUG-01): the decision half runs without an audio
    /// device, so `cur_music` is the track the menus play.
    pub cur_music: u32,
    new_music: u32,
    /// `Main.musicFade[0..MAX_MUSIC]`.
    music_fade: Vec<f32>,
    /// `AudioTracks[0..MAX_MUSIC]`; `Some` while a track is loaded/playing.
    tracks: Vec<Option<rodio::Sink>>,
    handle: Option<rodio::OutputStreamHandle>,
    /// `Main.dayMusic` / `Main.ugMusic`.
    day_music: u32,
    ug_music: u32,
    /// `Main.rand` stand-in for `Main.rand.Next(2)`.
    rng: u64,
    /// `Main.lastMusicPlayed`.
    last_music_played: u32,
}

impl MusicSystem {
    pub fn new(content_dir: Option<&Path>) -> Self {
        let mut wave_indices = HashMap::new();
        let wavebank_path = content_dir.map(|c| c.join("Wave Bank.xwb"));
        if let Some(dir) = content_dir {
            match tr_content::XactSoundBank::load(&dir.join("Sound Bank.xsb")) {
                Ok(bank) => {
                    for (name, _, track) in &bank.cues {
                        if let Some(id) = name
                            .strip_prefix("Music_")
                            .and_then(|n| n.parse::<u32>().ok())
                        {
                            wave_indices.insert(id, *track as usize);
                        }
                    }
                    log::info!("music: {} cue indices loaded", wave_indices.len());
                }
                Err(e) => log::warn!("music: cannot read the sound bank ({e:#})"),
            }
        }
        Self {
            wavebank_path,
            wave_indices,
            cur_music: 0,
            new_music: 0,
            music_fade: vec![0.0; MAX_MUSIC],
            tracks: (0..MAX_MUSIC).map(|_| None).collect(),
            handle: None,
            day_music: 0,
            ug_music: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
            last_music_played: 0,
        }
    }

    /// Set the global audio handle after initialization.
    pub fn set_handle_from_system(handle: rodio::OutputStreamHandle) {
        *MUSIC_HANDLE.lock().unwrap() = Some(handle);
    }

    fn next_rand(&mut self) -> u32 {
        // xorshift64*; only the parity of `Main.rand.Next(2)` is observable.
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as u32
    }

    /// `LegacyAudioSystem.IsTrackPlaying` — a loaded sink is playing (music loops).
    fn is_track_playing(&self, i: usize) -> bool {
        self.tracks.get(i).is_some_and(Option::is_some)
    }

    /// `Main.UpdateAudio`, minus the focus/moon-lord/ambient specials the port has
    /// no state for. Call once per frame.
    pub fn update(&mut self, env: MusicEnv) {
        // The decision half runs wherever an audio device is absent (the
        // headless/screenshot builds decide the track and fall through), so
        // `cur_music` is observable without rodio — only the fade/volume
        // loop below needs the device.
        self.decide_new_music(env);
        if env.music_volume == 0.0 {
            self.new_music = 0;
        }
        // `Main.cs:11076-11350`: the box chain sits in `UpdateAudio` AFTER
        // `DecideOnNewMusic` returns — the box (or the silence box) wins
        // over every world branch including the bosses.
        self.apply_music_box_override(&env);
        self.cur_music = self.new_music;

        if self.handle.is_none() {
            self.handle = MUSIC_HANDLE.lock().unwrap().clone();
            if self.handle.is_none() {
                return;
            }
        }

        let is_main_track_audible = self
            .music_fade
            .get(self.cur_music as usize)
            .is_some_and(|f| *f > 0.25);

        for i in 1..MAX_MUSIC {
            // Only loaded cues have an `AudioTrack`.
            if !self.wave_indices.contains_key(&(i as u32)) {
                continue;
            }
            let mut volume = self.music_fade[i] * env.music_volume;
            if (62..=88).contains(&i) {
                volume *= 0.9;
            } else if i == 52 {
                volume *= 1.15;
                if volume > 1.0 {
                    volume = 1.0;
                }
            }
            if i == self.cur_music as usize {
                self.update_common_track(true, i, volume);
            } else {
                self.update_common_track_toward_stopping(i, volume, is_main_track_audible);
            }
        }

        // `Main.cs:11527-11600`: the two ambience tracks ride the same pool —
        // the rain loop (28) while ZoneRain and not ZoneSnow, the wind loop
        // (45) while |windSpeed| >= 20/50 and SurfaceAtmospherics; the weather
        // factor scales the system volume and shimmer mutes both.
        if let Some(volume) = ambience_rain_volume(&env) {
            self.update_common_track(true, 28, env.ambient_volume * volume);
        } else {
            self.update_common_track_toward_stopping(
                28,
                env.ambient_volume * ambience_rain_volume(&env).unwrap_or(0.0),
                is_main_track_audible,
            );
        }
        if let Some(volume) = ambience_wind_volume(&env) {
            self.update_common_track(true, 45, env.ambient_volume * volume);
        } else {
            self.update_common_track_toward_stopping(
                45,
                env.ambient_volume * ambience_wind_volume(&env).unwrap_or(0.0),
                is_main_track_audible,
            );
        }
    }

    /// `Main.UpdateAudio_DecideOnNewMusic`, reduced to the states this port models:
    /// boss, underground (`Main.ugMusic`), rain, surface day (`Main.dayMusic`) and
    /// night. The biome branches (`Main.cs:12690-12925`) need `SceneMetrics` and are
    /// left for when those zones exist.
    fn decide_new_music(&mut self, env: MusicEnv) {
        // `if (Main.musicVolume == 0f) Main.newMusic = 0;`
        if env.music_volume == 0.0 {
            self.new_music = 0;
            return;
        }
        self.last_music_played = self
            .music_fade
            .iter()
            .enumerate()
            .filter(|(_, f)| **f > 0.0)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .filter(|(_, f)| **f == 1.0)
            .map(|(i, _)| i as u32)
            .unwrap_or(self.last_music_played);

        // `if (Main.gameMenu)` (`Main.cs:12455-12532`) — the front-end
        // ladder, an exclusive arm that returns. The vanilla's `netMode == 2`
        // (`newMusic = 0`) tail is unreachable: the port is single-player.
        if env.menu {
            let flags = env.menu_flags;
            self.new_music = if flags.contains(tr_core::seeds::SeedFlags::DRUNK) {
                if flags.contains(tr_core::seeds::SeedFlags::REMIX) {
                    70 // `remixWorldGen` (`:12463`)
                } else if shimmer_ocean_generation(flags) {
                    // W4-g22 MENU-FINAL — `Main.onlyShimmerOceanWorldsGeneration`
                    // (`:12465-12468`): drunk + anniversary WITHOUT remix/
                    // everything/bees (the property, `Main.cs:2686-2696`)
                    // plays OtherworldSnow (72). The property's gates are
                    // pure flag tests — ported as `shimmer_ocean_generation`.
                    72
                } else if env
                    .menu_micro_flags
                    .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM)
                {
                    // W4-g22 MENU-FINAL — `Main.vampireSeed` (`:12469-12472`):
                    // the vampirism micro seed (`WorldGen.cs:555-558`) plays
                    // OtherworldCrimson (75).
                    75
                } else if flags.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES) {
                    86 // `notTheBeesWorld` (`:12473-12476`)
                } else {
                    60 // the plain drunk title (`:12477-12480`)
                }
            } else if flags.contains(tr_core::seeds::SeedFlags::REMIX) {
                8 // `remixWorldGen` outside drunk (`:12482-12485`)
            } else if env.menu_credits {
                // W4-g22 MENU-FINAL — `Main.menuMode == 3000` (`:12486-12489`):
                // the credits page plays JourneysEnd (89); the port's
                // `Page::Credits` feeds `menu_credits`.
                89
            } else if flags.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY) {
                11 // `tenthAnniversaryWorldGen` (`:12490-12493`)
            } else {
                // The tail's `default:` (`:12517-12519`) — no titleMusicStyle
                // setting in the port (the `playOldTile` sun/moon-grab 6 and
                // the async-load intro 50 arms above it are unported states,
                // census-recorded in research/90's MENU-FINAL section).
                // `musicNoCrossFade[51] = true` (`:12217`): the title track
                // skips the cross-fade ramp (`:12521-12524`).
                self.music_fade[51] = 1.0;
                51
            };
            return;
        }
        // `:12542-12545` (W4-f33): the Torch God minigame — track 101, ABOVE
        // the boss-flag ladder (the C# order: credits-roll 89, then this,
        // then the bosses). The credits-roll event is unported (folds off).
        if env.in_torch_god_minigame {
            self.new_music = 101;
            return;
        }
        if let Some(boss) = env.boss_music {
            // The boss-flag ladder (`:12560-12620`): the event tracks (DD2
            // 92, the frost/pumpkin moons 97/98, the mech bosses 34/24/57/58,
            // Deerclops 38, the pillars…) sit between King Slime's 93
            // (flag19, `:12596`) and the generic `boss` 5 (flag, `:12611`) —
            // none of those NPCs exist in the port yet, so the effective
            // order is 93 then 5.
            self.new_music = boss;
            return;
        }
        // `:12644`: the eclipse — surface only (`!remixWorld` and the scene
        // centre above `worldSurface * 16 + screenHeight/2`).
        if env.eclipse {
            self.new_music = 27;
            return;
        }
        // `:12668`: the shimmer zone.
        if env.shimmer {
            self.new_music = 91;
            return;
        }
        // `flag23` (`:12199`): `TownNPCCount >= 3 && !ZoneShadowCandle` —
        // the town day 46 / night 47 tracks (`:12670-12678`) with their
        // quiet-sky gates (`cloudAlpha == 0 && !windy`, or the underground
        // view) and the graveyard cut. The port has no town NPCs yet
        // (M4-TOWN).
        let quiet_or_underground = env.quiet_sky || env.underground;
        if env.town_npcs >= 3
            && !env.shadow_candle
            && !env.graveyard
            && quiet_or_underground
            && (!env.blood_moon || env.day)
        {
            self.new_music = if env.day { 46 } else { 47 };
            return;
        }
        // `:12680`: the sandstorm.
        if env.sandstorm {
            self.new_music = 40;
            return;
        }
        // `:12682`: the underworld — `Center.Y > UnderworldLayer * 16`.
        if env.underworld {
            self.new_music = 36;
            return;
        }
        // `:12686`: the space band (`num5 < 1f`).
        if env.space_band {
            self.new_music = if env.day { 42 } else { 15 };
            return;
        }
        // `:12690`: the Lihzahd temple wall (87).
        if env.temple_wall {
            self.new_music = 26;
            return;
        }
        // `:12694`: the dungeon zone.
        if env.dungeon {
            self.new_music = 23;
            return;
        }
        // `:12700`: the mushroom background (the surface band; the port's
        // `undergroundBackground` is pinned 0).
        if env.mushroom_bg {
            self.new_music = 29;
            return;
        }
        // `:12702-12742`: the evil biomes — the corrupt arm splits on the
        // crimson lead, the underground/surface halves pick 33/10/8/16.
        if env.corrupt {
            if env.crimson && env.blood_leads_evil {
                self.new_music = if env.underground { 33 } else { 16 };
            } else if env.underground {
                self.new_music = 10;
            } else {
                self.new_music = 8;
            }
            return;
        }
        if env.crimson {
            self.new_music = if env.underground { 33 } else { 16 };
            return;
        }
        // `:12744-12751`: the meteor and the graveyard.
        if env.meteor {
            self.new_music = 2;
            return;
        }
        if env.graveyard {
            self.new_music = 53;
            return;
        }
        // `:12750` (the UG-desert arm sits constants-off — the sandstone
        // wall sets the port does not carry yet); `:12751`: the desert.
        if env.desert {
            self.new_music = 21;
            return;
        }
        // `:12752-12791`: the jungle — the non-remix arm with the 54 band's
        // 50-tile hysteresis above the rock layer.
        if env.jungle {
            if env.center_y > env.rock_band_px {
                self.new_music = 54;
            } else if self.new_music == 54 && env.center_y > env.jungle_deep_band_px {
                self.new_music = 54;
            } else if env.storm && env.center_y < env.surface_band_px {
                self.new_music = 52;
            } else if env.day {
                self.new_music = 7;
            } else {
                self.new_music = 55;
            }
            return;
        }
        // `:12792-12811`: the snow — underground 20, surface 14.
        if env.snow {
            self.new_music = if env.underground { 20 } else { 14 };
            return;
        }
        if env.underground {
            // `:12816-12821`: the hallow keeps its own underground track.
            if env.hallow {
                self.new_music = 11;
                return;
            }
            // `Main.cs:12830`: alternate Underground (4) / UndergroundAlt (31).
            if self.ug_music == 0 {
                self.ug_music = 4;
            }
            if !self.is_track_playing(4) && !self.is_track_playing(31) {
                if self.music_fade[4] == 1.0 {
                    self.music_fade[31] = 1.0;
                }
                if self.music_fade[31] == 1.0 {
                    self.music_fade[4] = 1.0;
                }
                if self.next_rand() % 2 == 0 {
                    self.ug_music = 4;
                    self.music_fade[31] = 0.0;
                } else {
                    self.ug_music = 31;
                    self.music_fade[4] = 0.0;
                }
            }
            self.new_music = self.ug_music;
            return;
        }
        // `:12875-12894`: the surface fallthrough — the daytime hallow, then
        // the storm pair (blood 2, else the storm track 52).
        if env.day && env.hallow {
            if env.storm {
                self.new_music = 52;
            } else if env.rain {
                self.new_music = 19;
            } else if env.windy_music {
                self.new_music = 44;
            } else {
                self.new_music = 9;
            }
            return;
        }
        if env.storm {
            self.new_music = if env.blood_moon { 2 } else { 52 };
            return;
        }
        // `:12896-12915`: the ocean bands — blood 2, the town pair, else the
        // ocean day 22 / night 43.
        if env.ocean {
            if env.blood_moon {
                self.new_music = 2;
            } else if env.town_npcs >= 3 && !env.shadow_candle {
                self.new_music = if env.day { 46 } else { 47 };
            } else {
                self.new_music = if env.day { 22 } else { 43 };
            }
            return;
        }
        // The surface tail (`:12925-12986`): night — blood moon 2, rain 19,
        // else OverworldNight 3.
        if !env.day {
            self.new_music = if env.blood_moon {
                2
            } else if env.rain {
                19
            } else {
                3 // OverworldNight
            };
            return;
        }
        // Day (`:12936-12966`): the rain morning split, then the 1/18
        // alternates with the windy-day override after the pick.
        if env.rain {
            self.new_music = if env.clock < 10800.0 { 59 } else { 19 };
            return;
        }
        if self.day_music == 0 {
            self.day_music = 1;
        }
        if !self.is_track_playing(1) && !self.is_track_playing(18) {
            self.day_music = if self.next_rand() % 2 == 0 { 1 } else { 18 };
        }
        self.new_music = self.day_music;
        if env.windy_music {
            self.new_music = 44;
        }
    }

    /// The `UpdateAudio` box chain (`Main.cs:11076-11350`) — the override
    /// pass that runs after `decide_new_music` returns; the box (or the
    /// silence box) wins over every world branch including the bosses.
    fn apply_music_box_override(&mut self, env: &MusicEnv) {
        if env.music_box_silence {
            self.new_music = 0;
        } else if env.music_box >= 0 {
            if let Some(track) = music_box_track(env.music_box) {
                self.new_music = track;
            }
        }
    }

    /// `LegacyAudioSystem.UpdateCommonTrack` (`LegacyAudioSystem.cs:292`).
    fn update_common_track(&mut self, active: bool, i: usize, total_volume: f32) {
        self.music_fade[i] += 0.005;
        if self.music_fade[i] > 1.0 {
            self.music_fade[i] = 1.0;
        }
        if !self.is_track_playing(i) && active {
            self.start_track(i, total_volume);
        } else if let Some(Some(sink)) = self.tracks.get(i) {
            sink.set_volume(total_volume);
        }
    }

    /// `LegacyAudioSystem.UpdateCommonTrackTowardStopping` (`LegacyAudioSystem.cs:320`).
    fn update_common_track_toward_stopping(
        &mut self,
        i: usize,
        total_volume: f32,
        is_main_track_audible: bool,
    ) {
        if !self.is_track_playing(i) {
            self.music_fade[i] = 0.0;
            return;
        }
        if is_main_track_audible {
            self.music_fade[i] -= 0.005;
        } else if self.cur_music == 0 {
            self.music_fade[i] = 0.0;
        }
        if self.music_fade[i] <= 0.0 {
            self.music_fade[i] = 0.0;
            self.stop_track(i);
        } else if let Some(Some(sink)) = self.tracks.get(i) {
            sink.set_volume(total_volume);
        }
    }

    /// `AudioTracks[i].Reuse(); SetVariable("Volume"); Play()`.
    fn start_track(&mut self, i: usize, volume: f32) {
        let Some(handle) = self.handle.as_ref() else {
            return;
        };
        let Some(&entry_index) = self.wave_indices.get(&(i as u32)) else {
            return;
        };
        let Some(wb_path) = self.wavebank_path.as_ref() else {
            return;
        };
        let Ok(bank) = tr_content::XactWaveBank::open(wb_path) else {
            return;
        };
        let Some(entry) = bank.entries.get(entry_index) else {
            return;
        };
        let format = entry.format;
        let block_align = format.block_align();
        if format.codec() != tr_content::Codec::Adpcm || block_align == 0 {
            log::debug!("music {i}: unsupported codec {:?}", format.codec());
            return;
        }
        let Ok(data) = bank.read_data(wb_path, entry_index) else {
            return;
        };
        let Ok(samples) = tr_content::decode_msadpcm(
            &data,
            format.sample_rate,
            format.channels,
            block_align as u16,
        ) else {
            return;
        };
        let Ok(sink) = rodio::Sink::try_new(handle) else {
            return;
        };
        let buffer =
            rodio::buffer::SamplesBuffer::new(format.channels, format.sample_rate, samples);
        sink.append(buffer.repeat_infinite());
        sink.set_volume(volume);
        self.tracks[i] = Some(sink);
        log::info!("music: track {i} started (entry {entry_index})");
    }

    /// `AudioTracks[i].SetVariable("Volume", 0); Stop()`.
    fn stop_track(&mut self, i: usize) {
        if let Some(Some(sink)) = self.tracks.get(i) {
            sink.stop();
        }
        self.tracks[i] = None;
    }
}

/// `UpdateAudio`'s box→track mapping (`Main.cs:11076-11350`) — the 100
/// `ActiveMusicBox == N → newMusic = M` pairs extracted verbatim; `None`
/// for the boxes the source does not map (its chain is exhaustive 0..99,
/// so only out-of-range ids land here).
pub fn music_box_track(box_id: i32) -> Option<u32> {
    const PAIRS: [(i32, u32); 100] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 6),
        (4, 4),
        (5, 5),
        (6, 7),
        (7, 8),
        (8, 10),
        (9, 9),
        (10, 12),
        (11, 11),
        (12, 13),
        (13, 14),
        (14, 15),
        (15, 16),
        (16, 17),
        (17, 18),
        (18, 19),
        (19, 20),
        (20, 21),
        (21, 22),
        (22, 23),
        (23, 24),
        (24, 25),
        (25, 26),
        (26, 27),
        (27, 29),
        (28, 30),
        (29, 31),
        (30, 32),
        (31, 33),
        (32, 38),
        (33, 37),
        (34, 35),
        (35, 36),
        (36, 34),
        (37, 39),
        (38, 40),
        (39, 41),
        (40, 44),
        (41, 48),
        (42, 42),
        (43, 43),
        (44, 46),
        (45, 47),
        (46, 49),
        (47, 51),
        (48, 52),
        (49, 53),
        (50, 54),
        (51, 55),
        (52, 56),
        (53, 57),
        (54, 58),
        (55, 59),
        (56, 60),
        (57, 61),
        (58, 62),
        (59, 63),
        (60, 64),
        (61, 65),
        (62, 66),
        (63, 67),
        (64, 68),
        (65, 69),
        (66, 70),
        (67, 71),
        (68, 72),
        (69, 73),
        (70, 74),
        (71, 75),
        (72, 76),
        (73, 77),
        (74, 78),
        (75, 79),
        (76, 80),
        (77, 81),
        (78, 82),
        (79, 83),
        (80, 84),
        (81, 85),
        (82, 86),
        (83, 87),
        (84, 88),
        (85, 89),
        (86, 90),
        (87, 91),
        (88, 93),
        (89, 96),
        (90, 95),
        (91, 94),
        (92, 97),
        (93, 98),
        (94, 92),
        (95, 99),
        (96, 101),
        (97, 100),
        (98, 104),
        (99, 103),
    ];
    PAIRS
        .iter()
        .find(|(id, _)| *id == box_id)
        .map(|(_, track)| *track)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-E-21g — the roar plays are stream-NEUTRAL: case 15
    /// (`LegacySoundPlayer.cs:826-842`) and case 36 (`:1309-1325`) draw
    /// nothing from `Main.rand` and no NextFloat — the attenuation gate
    /// is the only refusal, and the style-4 remap picks `Roar_1`.
    #[test]
    fn s1_e_21g_the_roar_plays_draw_nothing() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_roar(0, 0.0, 0.0), Some("Roar_0"));
        assert_eq!(audio.play_roar(4, 0.0, 0.0), Some("Roar_1"));
        assert_eq!(audio.play_dash_roar(-1, 0.0, 0.0), Some("Roar_0"));
        assert_eq!(audio.play_dash_roar(2, 0.0, 0.0), Some("Roar_2"));
        assert_eq!(rng.peek(), peek, "the roar cases consume no draws");
        // The gate refuses the out-of-range play — still draw-free.
        assert_eq!(audio.play_roar(0, 5000.0, 0.0), None);
        assert_eq!(audio.play_dash_roar(0, 5000.0, 0.0), None);
        assert_eq!(rng.peek(), peek);
        // Headless the slots never play, so the case-15 guard's
        // null-or-Stopped test passes every time (the fresh-slot table,
        // the wave-C convention).
        assert_eq!(audio.play_roar(0, 0.0, 0.0), Some("Roar_0"));
    }

    /// W4-g73 falls-adjacent — the liquid loops' live pan: the shared gains
    /// the playing source reads update in place (the XNA
    /// `SoundInstanceLiquid[n].Pan = num3` write,
    /// `LegacySoundPlayer.cs:1267`/`:1297`), and the headless slot guard
    /// reads not-playing (the fresh-create convention).
    #[test]
    fn the_liquid_loop_pan_gains_update_the_live_source() {
        let gains = LiquidPanGains::new(1.0, 0.5);
        let reader = gains.clone();
        assert_eq!(reader.load(), (1.0, 0.5));
        // The ¼ s re-pan stores reach the same cell the source reads.
        let (left, right) = sound::pan_gains(-0.75);
        gains.store_left(left);
        gains.store_right(right);
        assert_eq!(reader.load(), (left, right));
        assert_eq!(reader.load(), (1.0, 0.25));
        // The source shape applies the gains per channel.
        let mut src = PannedShared {
            inner: [1.0f32, 1.0, 1.0, 1.0].into_iter(),
            channels: 2,
            gains: reader,
            channel: 0,
        };
        assert_eq!(src.next(), Some(1.0));
        assert_eq!(src.next(), Some(0.25));
        // Headless: no output device, so the slot never plays.
        let audio = Audio::new(None);
        assert!(!audio.liquid_loop_playing(0));
        assert!(!audio.liquid_loop_playing(1));
    }

    /// `WorldGen.StopWaterfallAmbienceAudio` (`WorldGen.cs:6689-6699`) +
    /// `StopTrackedSounds` (`:6646`/`:6715` → `SoundPlayer.StopAll`,
    /// `SoundPlayer.cs:79-86`): the sentinel pair stops BOTH liquid
    /// channels draw-free and the tracked pool empties. Headless there is
    /// no sink, so the stop count is the witness seam; a tracked entry
    /// registered through the test cue proves the pool drain. (The
    /// W4-g24/g73 quit-stop keep-one: this supersedes the closeout's
    /// `the_quit_stop_kills_both_liquid_loop_channels` — same stop, the
    /// counter + RNG-neutral + drain assertions are the superset.)
    #[test]
    fn w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain() {
        let mut audio = Audio::new(None);
        let mut rng = tr_core::unified_random::UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.liquid_ambience_stop_count(), 0);
        audio.stop_waterfall_ambience();
        audio.stop_waterfall_ambience();
        assert_eq!(audio.liquid_ambience_stop_count(), 2, "each call counts");
        assert!(!audio.liquid_loop_playing(0), "dead slots stay dead");
        assert!(!audio.liquid_loop_playing(1));
        assert_eq!(rng.peek(), peek, "the sentinel pair draws nothing");
        // The tracked pool: one entry lands, StopAll clears it.
        audio.insert_test_cue("menu_accept");
        let style = menu_accept_style();
        let _ = audio.play_tracked_global(style.clone());
        assert_eq!(audio.tracked_count_for(&style), 1, "the entry landed");
        audio.stop_tracked_sounds();
        assert_eq!(audio.tracked_count_for(&style), 0, "StopAll cleared the pool");

        }    /// W4-g21 d79-receipt fix — the Destroyer burrow roar's drain
    /// (`world_sim/npcs.rs`'s `ev.roar` arm, `NPC.cs:50093`) rides
    /// `play_roar(1, …)`: the C# `PlaySound(15, x, y)` takes the int
    /// overload's DEFAULT Style = 1 (`SoundEngine.cs:102`), so the faithful
    /// asset is `Roar_1` — NOT the old bare `play_at("Roar_0")` (wrong slot,
    /// no guard; d79 §F4's keep-d79 ruling applied on the landed S1-E-21g
    /// seam). Style 1 maps straight to slot 1 at ×1.0 (`roar_case15`'s
    /// non-4 arm) and draws nothing. WRITTEN NOT RUN (code-only).
    fn w4_g21_the_destroyer_roar_default_style_is_one() {
        use tr_core::unified_random::UnifiedRandom;
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(audio.play_roar(1, 0.0, 0.0), Some("Roar_1"));
        assert_eq!(crate::sound::roar_case15(1, false), Some((1, 1.0)));
        assert_eq!(rng.peek(), peek, "the default-style roar is draw-free");
    }

    /// S1-E-21g — the int-overload peers draw exactly ONE case pitch on
    /// the passed stream and NO NextFloat; the style façades
    /// (`NPCDeath61`/`NPCDeath13`) add the audio-only NextFloat but the
    /// same one gameplay draw.
    #[test]
    fn s1_e_21g_the_int_and_style_death_peers_follow_the_overload_split() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // The int overload: one Next(-10, 11), no style-stream movement.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        let style_before = audio.sound_style_random.clone();
        assert_eq!(
            audio.play_npc_death_int(22, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_22")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_eq!(
            audio.sound_style_random, style_before,
            "the int overload draws no NextFloat"
        );
        // The style façade: the same one gameplay draw + the NextFloat.
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(
            audio.play_npc_death_style(61, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_61")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_ne!(
            audio.sound_style_random, style_before,
            "the façade consumed the NextFloat"
        );
        // The hit int peer (the EoC/Brain hand-off): one draw, no NextFloat.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let style_before = audio.sound_style_random.clone();
        assert_eq!(
            audio.play_npc_hit_int(1, 0.0, 0.0, &mut rng),
            Some("NPC_Hit_1")
        );
        assert_eq!(rng.peek(), witness.peek());
        assert_eq!(audio.sound_style_random, style_before);
        // Muted: draw-free on every peer.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_npc_hit_int(1, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_death_int(6, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_death_style(13, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted plays are draw-free");
    }


    /// S1-E-21g — the caster Item8 puff rides wave A's case-2 path: the
    /// NextFloat on the audio-only stream, then ONE default
    /// `Next(-6, 7) * 0.01` gameplay draw after the gate.
    #[test]
    fn s1_e_21g_the_item_cue_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            audio.play_npc_item_cue(8, 0.0, 0.0, &mut rng),
            Some("Item_8")
        );
        assert_eq!(rng.peek(), witness.peek(), "case 2's one pitch draw");
        // Out of range: the gate refuses BEFORE the case draw.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_npc_item_cue(8, 5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
        // The EoW dig trackable: no gameplay draw at all (case 42).
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_eow_dig(0.0, 0.0), Some("eow_dig"));
        assert_eq!(rng.peek(), peek, "the trackable is draw-free");
    }

    /// S1-E-21g — case 27's live-instance branch consumes the pitch draw
    /// and returns null; headless the slot never plays so the fresh
    /// branch draws the same ONE roll (`LegacySoundPlayer.cs:984`/`:994`).
    #[test]
    fn s1_e_21g_the_pixie_draws_one_pitch_per_call() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(audio.play_pixie(0.0, 0.0, &mut rng), Some("Pixie"));
        assert_eq!(rng.peek(), witness.peek(), "one pitch draw");
        // The gate refuses before the draw.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_pixie(5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the inaudible play is draw-free");
    }

    /// S1-P-i41 — the QuickMount no-space tink (`Player.cs:6057`): the int
    /// overload (`SoundEngine.cs:102-108`) draws NO `GetRandomPitch`
    /// NextFloat; case 0 (`LegacySoundPlayer.cs:440-452`) draws `Next(3)`
    /// then `Next(-10, 11)` on the caller's stream, only past the gate.
    #[test]
    fn s1_p_i41_the_no_space_tink_draws_variation_then_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let variation = witness.next_range(0, 3);
        let _pitch = witness.next_range(-10, 11);
        let cue = audio.play_no_space_sound(0.0, 0.0, &mut rng);
        let want = match variation {
            0 => "Dig_0",
            1 => "Dig_1",
            _ => "Dig_2",
        };
        assert_eq!(cue, Some(want), "the Next(3) pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // Muted / inaudible: the gate precedes the switch — no draws.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_no_space_sound(50000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the inaudible play is draw-free");
    }

    /// S1-P-i41 — the Chillemt equip cue (`Mount.cs:6283-6286`): the style
    /// façade's `GetRandomPitch` NextFloat rides the audio-only stream
    /// (variance 0 — the draw still happens); the gameplay stream stays
    /// untouched, and the cue resolves against the trackable bank.
    #[test]
    fn s1_p_i41_the_chillet_cue_draws_on_the_audio_only_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let style_peek = audio.sound_style_random.peek();
        assert_eq!(audio.play_chillet_sound(0.0, 0.0), Some("pal_chillet"));
        assert_eq!(rng.peek(), peek, "no gameplay draw");
        assert_ne!(
            audio.sound_style_random.peek(),
            style_peek,
            "the GetRandomPitch NextFloat consumed"
        );
        // Inaudible: the façade's NextFloat fires BEFORE the gate
        // (`SoundEngine.cs:93-99` computes the pitch, then the legacy player
        // refuses) — the audio stream still advances, the cue is None.
        let style_peek = audio.sound_style_random.peek();
        assert_eq!(audio.play_chillet_sound(50000.0, 0.0), None);
        assert_ne!(audio.sound_style_random.peek(), style_peek);
    }

    /// A headless system (no sinks) — `decide_new_music` only touches the
    /// fade table and the picked ids.
    fn system() -> MusicSystem {
        MusicSystem::new(None)
    }

    fn env(day: bool) -> MusicEnv {
        MusicEnv {
            music_volume: 1.0,
            day,
            cloud_alpha: 0.0,
            zone_rain: false,
            zone_snow: false,
            surface_atmospherics: false,
            wind_speed: 0.0,
            shimmer_alpha: 0.0,
            atmo: 1.0,
            ambient_volume: 1.0,
            underground: false,
            rain: false,
            boss_music: None,
            clock: 27000.0,
            eclipse: false,
            in_torch_god_minigame: false,
            shimmer: false,
            town_npcs: 0,
            shadow_candle: false,
            sandstorm: false,
            underworld: false,
            blood_moon: false,
            windy_music: false,
            music_box: -1,
            music_box_silence: false,
            space_band: false,
            temple_wall: false,
            dungeon: false,
            mushroom_bg: false,
            corrupt: false,
            crimson: false,
            blood_leads_evil: false,
            meteor: false,
            graveyard: false,
            desert: false,
            jungle: false,
            snow: false,
            hallow: false,
            storm: false,
            ocean: false,
            center_y: 0.0,
            surface_band_px: 1000.0,
            rock_band_px: 2000.0,
            jungle_deep_band_px: 1500.0,
            quiet_sky: true,
            menu: false,
            menu_flags: Default::default(),
            menu_micro_flags: Default::default(),
            menu_credits: false,
        }
    }

    #[test]
    fn the_ladder_order_beats_the_surface_tail() {
        // The boss scan outranks everything the port models; the eclipse and
        // the sandstorm cut in front of the underworld and the underground.
        let mut m = system();
        let mut e = env(true);
        e.boss_music = Some(93);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 93);
        let mut m = system();
        let mut e = env(true);
        e.eclipse = true;
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 27);
        let mut m = system();
        let mut e = env(true);
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 40);
        let mut m = system();
        let mut e = env(true);
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 36);
        let mut m = system();
        let mut e = env(true);
        e.underground = true;
        m.decide_new_music(e);
        // The underground pair alternates 4/31 (`Main.cs:12830`'s coin flip).
        assert!(matches!(m.new_music, 4 | 31));
    }

    /// W4-e19 aether census — the ZoneShimmer music gate (`Main.cs:12666-12669`,
    /// `MusicID.Aether = 91`, `MusicID.cs:200`): the shimmer zone beats the
    /// town/sandstorm/underworld/biome tail but loses to the eclipse and the
    /// boss scan. The TOW ladder's 72 (`:11951-11953`) waits for the drunk-world
    /// swap (`Main.cs:11055-11074`, unported — the port always runs the normal
    /// ladder, ANALYSIS W4-e19 aether).
    #[test]
    fn w4_e19_the_aether_zone_picks_track_91_in_ladder_order() {
        let mut m = system();
        let mut e = env(true);
        e.shimmer = true;
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        e.jungle = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 91, "the aether beats the tail");
        // The eclipse outranks it (`:12644` before `:12666`).
        let mut m = system();
        let mut e = env(true);
        e.shimmer = true;
        e.eclipse = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 27);
        // The boss scan outranks everything (`:12560`).
        let mut m = system();
        let mut e = env(true);
        e.shimmer = true;
        e.boss_music = Some(93);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 93);
    }

    #[test]
    fn w4_f33_the_torch_god_minigame_track_beats_the_boss_ladder() {
        // `Main.cs:12542-12545` — InTorchGodMinigame → track 101, ABOVE the
        // boss-flag ladder (the C# order: credits-roll 89, then this, then
        // the bosses). The event outranks even a live boss.
        let mut m = system();
        let mut e = env(true);
        e.in_torch_god_minigame = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 101);
        // A boss nearby does not shadow the event track.
        let mut m = system();
        let mut e = env(true);
        e.in_torch_god_minigame = true;
        e.boss_music = Some(93);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 101, "the event outranks the boss");
        // The event over, the boss ladder resumes.
        let mut m = system();
        let mut e = env(true);
        e.boss_music = Some(93);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 93);
        // The menu ladder still wins (the gameMenu arm returns first).
        let mut m = system();
        let mut e = env(true);
        e.in_torch_god_minigame = true;
        e.menu = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51, "the title track on the front end");
    }

    /// The closeout music-priority census (research/93 addendum §4):
    /// every below-ladder world input armed AT ONCE behind the minigame
    /// flag — track 101 still holds (the arm returns before the boss
    /// ladder and every biome branch, `Main.cs:12542-12545`). The ONE
    /// override above it is the C#-faithful music-box chain
    /// (`Main.cs:11076-11350` runs AFTER DecideOnNewMusic); the volume
    /// mute is the user's slider, not a track. WRITTEN NOT RUN.
    #[test]
    fn w4_f33_the_minigame_track_holds_against_every_world_branch() {
        let mut m = system();
        let mut e = env(false);
        e.in_torch_god_minigame = true;
        // Every input the ladder reads below the minigame arm, armed.
        e.boss_music = Some(93);
        e.eclipse = true;
        e.shimmer = true;
        e.town_npcs = 5;
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        e.space_band = true;
        e.temple_wall = true;
        e.dungeon = true;
        e.mushroom_bg = true;
        e.corrupt = true;
        e.crimson = true;
        e.blood_leads_evil = true;
        e.meteor = true;
        e.graveyard = true;
        e.desert = true;
        e.jungle = true;
        e.snow = true;
        e.hallow = true;
        e.storm = true;
        e.ocean = true;
        e.blood_moon = true;
        e.rain = true;
        e.windy_music = true;
        e.quiet_sky = false;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 101, "no world branch overrides the event");
        // The music box DOES override (the C# box chain sits behind
        // DecideOnNewMusic — the faithful exception, not a deviation).
        e.music_box = 42;
        m.decide_new_music(e);
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 42, "the box beats the minigame (C# order)");
        // The silence box cuts it to 0.
        e.music_box = -1;
        e.music_box_silence = true;
        m.decide_new_music(e);
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 0);
        // The flag cleared, the same armed env falls to the boss ladder.
        e.music_box_silence = false;
        e.in_torch_god_minigame = false;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 93, "the event over, the ladder resumes");
    }

    #[test]
    fn the_surface_tail_blood_rain_and_wind() {
        // Night: blood moon 2 over rain 19 over night 3 (`:12968-12979`).
        let mut m = system();
        let mut e = env(false);
        e.blood_moon = true;
        e.rain = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 2);
        let mut m = system();
        let mut e = env(false);
        e.rain = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 19);
        let mut m = system();
        m.decide_new_music(env(false));
        assert_eq!(m.new_music, 3);
        // Day rain: the morning split on the clock (`:12940-12947`).
        let mut m = system();
        let mut e = env(true);
        e.rain = true;
        e.clock = 5000.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 59);
        e.clock = 20000.0;
        let mut m = system();
        m.decide_new_music(e);
        assert_eq!(m.new_music, 19);
        // The windy-day override rides AFTER the 1/18 pick (`:12961`).
        let mut m = system();
        let mut e = env(true);
        e.windy_music = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 44);
    }

    #[test]
    fn the_music_box_overrides_the_whole_ladder() {
        // `Main.cs:11076+`: the box chain sits behind `DecideOnNewMusic`,
        // so a playing box beats the boss/eclipse/biome arms; the silent
        // box cuts the music to 0; the box id 85 → the credits track 89
        // (`SceneMetrics.cs:349`'s CanPlayCreditsRoll trigger).
        let mut m = system();
        let mut e = env(true);
        e.boss_music = Some(93);
        e.music_box = 0;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 1);
        let mut m = system();
        let mut e = env(true);
        e.eclipse = true;
        e.sandstorm = true;
        e.music_box = 85;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 89);
        let mut m = system();
        let mut e = env(true);
        e.music_box = 42;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 42);
        let mut m = system();
        let mut e = env(true);
        e.music_box_silence = true;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 0);
        // The no-box case leaves the ladder untouched (the day pair
        // alternates 1/18 — `Main.cs:12945`'s coin flip).
        let mut m = system();
        m.decide_new_music(env(true));
        assert!(matches!(m.new_music, 1 | 18));
    }

    #[test]
    fn the_biome_ladder_matches_the_source_order() {
        // The dungeon/mushroom/corrupt/meteor/graveyard/desert run in front
        // of the jungle/snow arms (`:12694-12811`), and the underground gate
        // keeps its hallow arm.
        let mut m = system();
        let mut e = env(true);
        e.dungeon = true;
        e.corrupt = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 23);
        let mut m = system();
        let mut e = env(true);
        e.mushroom_bg = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 29);
        let mut m = system();
        let mut e = env(true);
        e.corrupt = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 10);
        let mut m = system();
        let mut e = env(true);
        e.corrupt = true;
        e.crimson = true;
        e.blood_leads_evil = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 16);
        let mut m = system();
        let mut e = env(true);
        e.meteor = true;
        e.graveyard = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 2);
        let mut m = system();
        let mut e = env(true);
        e.graveyard = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 53);
        let mut m = system();
        let mut e = env(true);
        e.desert = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 21);
        // The jungle surface day is 7, the deep band 54; the snow splits
        // 14 surface / 20 underground.
        let mut m = system();
        let mut e = env(true);
        e.jungle = true;
        e.center_y = 500.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 7);
        let mut m = system();
        let mut e = env(true);
        e.jungle = true;
        e.center_y = 2500.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 54);
        let mut m = system();
        let mut e = env(true);
        e.snow = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 14);
        let mut m = system();
        let mut e = env(true);
        e.snow = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 20);
        // The hallow keeps the underground gate; the storm pair and the
        // ocean sit behind it (`:12816/:12875-12915`).
        let mut m = system();
        let mut e = env(true);
        e.underground = true;
        e.hallow = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 11);
        let mut m = system();
        let mut e = env(true);
        e.storm = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 52);
        let mut m = system();
        let mut e = env(true);
        e.ocean = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 22);
        let mut m = system();
        let mut e = env(false);
        e.ocean = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 43);
    }

    #[test]
    fn the_town_gate_needs_three_npcs_and_no_shadow_candle() {
        // `flag23` (`:12199`): TownNPCCount >= 3 && !ZoneShadowCandle; the
        // blood-moon night keeps the regular tail (`:12674`'s gates).
        let mut m = system();
        let mut e = env(true);
        e.town_npcs = 3;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 46);
        let mut m = system();
        let mut e = env(false);
        e.town_npcs = 3;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 47);
        let mut m = system();
        let mut e = env(true);
        e.town_npcs = 3;
        e.shadow_candle = true;
        m.decide_new_music(e);
        assert_ne!(m.new_music, 46);
    }

    #[test]
    fn dw_2_1_item4_cue_maps_to_the_item_legacy_track() {
        // `SoundID.Item4 = new LegacySoundStyle(2, 4)` (`SoundID.cs:1387`) —
        // the dial arm's cue (`Player.cs:32642`/:32652), the same legacy
        // Item track as `Item_1`.
        assert_eq!(sound_type("Item_4"), 2);
    }

    #[test]
    fn w4_e28_item149_cue_maps_to_the_item_legacy_track() {
        // W4-e28 — the Ammo Box's use cue: `SoundID.Item149 = new
        // LegacySoundStyle(2, 149)` (`SoundID.cs:1533`), played by
        // `Player.cs:32634` at `base.position`; `Sounds/Item_149.xnb`.
        assert_eq!(sound_type("Item_149"), 2);
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Item_149"));
    }

    #[test]
    fn w4_f34_item179_cue_maps_to_the_item_legacy_track() {
        // W4-f34 — the Dead Cells Potion Station's use cue:
        // `SoundID.Item179 = new LegacySoundStyle(2, 179).WithVolume(0.6f)`
        // (`SoundID.cs:1563`), played by `Player.cs:32616` at
        // `base.position`; `Sounds/Item_179.xnb`. The 0.6 style volume rides
        // the doors arm's volume_scale, not this type mapping.
        assert_eq!(sound_type("Item_179"), 2);
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Item_179"));
    }

    #[test]
    fn the_cue_table_carries_the_legacy_ids() {
        // `CraftingEffects.cs:51` — `SoundEngine.PlaySound(7)`, `SoundID.Grab`
        // (`SoundID.cs`); the grant sound rides the legacy type 7.
        assert_eq!(sound_type("Grab"), 7);
        // Every cue's asset name is its table name (`Sounds/{name}.xnb`).
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Grab"));
    }

    #[test]
    fn the_chest_cue_table_carries_the_legacy_menu_and_unlock_ids() {
        // The chest open path's menu cues — open 10 (`Player.cs:33242`),
        // close 11 (`:33237`, the tick close `:31082`/:31088), re-open 12
        // (`:33251`) — and `Chest.Unlock`/`Chest.Lock`'s positional unlock
        // (`Chest.cs:408`/:482): `SoundID.MenuOpen/MenuClose/MenuTick = 10/
        // 11/12` and `SoundID.Unlock = 22` (`SoundID.cs:42/:44/:46/:66`).
        assert_eq!(sound_type("Menu_Open"), 10);
        assert_eq!(sound_type("Menu_Close"), 11);
        assert_eq!(sound_type("Menu_Tick"), 12);
        assert_eq!(sound_type("Unlock"), 22);
        for name in ["Menu_Open", "Menu_Close", "Menu_Tick", "Unlock"] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
    }

    /// S1-E-21 wave A — the item-use play's RNG contract: the case-2 draws
    /// ride the caller's `Main.rand` stream (the `UpdatePlayers` swap) and
    /// run only when the attenuation gate passed; the `GetRandomPitch`
    /// NextFloat rides the audio-only `SoundStyle._random` stream.
    #[test]
    fn s1_e_21a_the_use_sound_draws_ride_the_passed_stream_only_when_audible() {
        use tr_core::unified_random::UnifiedRandom;
        let idx = |name: &str| {
            crate::soundid::SOUND_STYLES
                .iter()
                .position(|s| s.name == name)
                .unwrap() as u16
        };
        // An inaudible play (soundVolume 0) draws NOTHING from the gameplay
        // stream — the C# returns at `:369-372` before the switch.
        let mut audio = Audio::new(None);
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_use_sound(idx("Item1"), 0.0, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted play consumes no gameplay draw");
        // An audible style-2 play draws exactly one Next(-6, 7)
        // (`LegacySoundPlayer.cs:632`) — the same value a bare stream draw
        // produces.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let want_pitch = witness.next_range(-6, 7) as f32 * 0.01;
        let cue = audio.play_use_sound(idx("Item2"), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Item_2"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!(want_pitch.abs() <= 0.06, "the default window ±0.06");
        // Style 1 draws twice: Next(3) remap (`:541`) + the pitch roll.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let remap = witness.next_range(0, 3);
        let _ = witness.next_range(-6, 7);
        let cue = audio.play_use_sound(idx("Item1"), 0.0, 0.0, 0.0, &mut rng);
        let want = match remap {
            1 => "Item_18",
            2 => "Item_19",
            _ => "Item_1",
        };
        assert_eq!(cue, Some(want), "the style-1 remap rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // A trackable (case 42) draws nothing from the gameplay stream —
        // no pitch switch, the pinned variation (the port's D3 pin).
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let cue = audio.play_use_sound(idx("AbigailSummon"), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("abigail_summon"));
        assert_eq!(rng.peek(), peek, "case 42 consumes no gameplay draw");
    }

    /// W4-d85 FINAL — the issue-report notify cue
    /// (`IssueReportsIndicator.cs:24`): the three variations register on the
    /// legacy id-42 path, the play picks one on the audio-only `style_random`
    /// (no gameplay stream exists for the unpositioned UI play), and the
    /// played-count seam ticks even headless.
    #[test]
    fn the_issue_report_notify_cue_registers_and_plays() {
        for n in ["dd2_explosive_trap_explode_0", "dd2_explosive_trap_explode_1", "dd2_explosive_trap_explode_2"] {
            assert!(UI_CUSTOM_CUES.contains(&n), "{n} in the UI list");
            assert_eq!(sound_type(n), 42, "{n} rides the trackable path");
        }
        let mut audio = Audio::new(None);
        let before = audio.played_count();
        let cue = audio.play_issue_report_notify();
        assert!(
            cue.is_some_and(|c| UI_CUSTOM_CUES.contains(&c)),
            "a variation of the family played ({cue:?})"
        );
        assert_eq!(audio.played_count(), before + 1, "the count ticks headless");
        // The variation pick rides the audio-only stream — a second play
        // advances it (the C#'s `LegacySoundStyle.Random`, unobservable seed).
        let second = audio.play_issue_report_notify();
        assert!(second.is_some());

        }    /// M2-MOUNT-4 (W4-c14) — the Santank ability cues' RNG/volume contract
    /// (`Mount.cs:3300` Item89 / `:3311` Item11): the INT overload
    /// (`SoundEngine.cs:102`) consumes NO `GetRandomPitch` NextFloat (the
    /// audio-only stream stays untouched), the attenuation gate precedes
    /// the ONE case-2 `Next(-6, 7)` pitch draw (`LegacySoundPlayer.cs:632`),
    /// and the 0.2 volumeScale rides `:1411` after the tweak (both styles
    /// have tweak 1.0 and volume 1.0, so the final gain is 0.2× the gate's).
    fn m4_the_santank_cues_draw_one_pitch_roll_and_no_style_random() {
        use tr_core::unified_random::UnifiedRandom;
        // The bank join: Item_89 was the gap (88/90 present, 89 absent).
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Item_89"));
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Item_11"));
        // A muted play draws NOTHING (`:369-372` precedes the switch) and
        // never touches the audio-only style stream.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(11);
        let peek = rng.peek();
        assert_eq!(audio.play_mount_ability_sound(89, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted play consumes no gameplay draw");
        // An audible play: exactly ONE Next(-6, 7) on the passed stream —
        // the same value a bare draw produces (the default case-2 window).
        let mut witness = UnifiedRandom::new(11);
        let want_pitch = witness.next_range(-6, 7) as f32 * 0.01;
        let cue = audio.play_mount_ability_sound(89, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Item_89"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!(want_pitch.abs() <= 0.06, "the default window ±0.06");
        // The missile cue rides the same shape.
        let _ = witness.next_range(-6, 7);
        let cue = audio.play_mount_ability_sound(11, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Item_11"));
        // The int overload consumes NO GetRandomPitch draw: two plays from
        // a fresh Audio leave the audio-only stream at its seed (the style
        // façade would advance it once per play).
        let mut rng = UnifiedRandom::new(3);
        let _ = audio.play_mount_ability_sound(89, 0.0, 0.0, &mut rng);
        let _ = audio.play_mount_ability_sound(11, 0.0, 0.0, &mut rng);
        let mut fresh = tr_core::unified_random::UnifiedRandom::new(1);
        assert_eq!(
            audio.sound_style_random.peek(),
            fresh.peek(),
            "the int overload never draws the style stream"
    }

    /// S1-E-21a — the NPC hit/death cue families: the sound id 3 (the hits)
    /// and the sound id 4 (the deaths) carry the per-kind cues, and the
    /// `Sounds/Custom/` trackables ride the legacy id 42.
    #[test]
    fn the_npc_hit_and_death_cues_are_registered() {
        // The two families (`LegacySoundPlayer.cs:646`/`:676`).
        assert_eq!(sound_type("NPC_Hit_1"), 3);
        assert_eq!(sound_type("NPC_Hit_2"), 3);
        assert_eq!(sound_type("NPC_Killed_1"), 4);
        assert_eq!(sound_type("NPC_Killed_2"), 4);
        for name in ["NPC_Hit_1", "NPC_Hit_2", "NPC_Killed_1", "NPC_Killed_2"] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
        // The player's death cue (the legacy id 5, `Player.PlayDeathSound`,
        // `Player.cs:38737`).
        assert_eq!(sound_type("Player_Killed"), 5);
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Player_Killed"));
        // The custom trackables (`SoundID.CreateTrackable`, `SoundID.cs:1627+`)
        // ride the legacy trackable path (the sound id 42).
        assert_eq!(sound_type("dd2_goblin_hurt_0"), 42);
        assert_eq!(sound_type("deerclops_hit_0"), 42);
        for name in ["dd2_goblin_hurt_0", "deerclops_hit_0", "deerclops_death"] {
            assert!(NPC_CUSTOM_CUES.contains(&name), "{name} in the custom list");
        }
    }

    /// S1-E-21 wave C — the NPC hit play's RNG contract
    /// (`LegacySoundPlayer.cs:646-672`): exactly one
    /// `Main.rand.Next(-10, 11) * 0.01` pitch draw (`:670`) on the caller's
    /// stream, only after the attenuation gate passed; the GetRandomPitch
    /// NextFloat rides the audio-only stream; the trackables (case 42)
    /// draw nothing from the gameplay stream.
    #[test]
    fn s1_e_21c_the_npc_hit_draws_ride_the_passed_stream_only_when_audible() {
        use tr_core::unified_random::UnifiedRandom;
        // An audible play draws exactly one Next(-10, 11).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_range(-10, 11);
        let cue = audio.play_npc_hit_sound("NPC_Hit_1", 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("NPC_Hit_1"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!((-0.1..=0.1).contains(&(roll as f32 * 0.01)));
        // Muted (soundVolume 0): the gate refuses before the switch —
        // no gameplay draw (`:369-372` precedes `:646`).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_npc_hit_sound("NPC_Hit_1", 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted play is draw-free");
        // Out of attenuation range: also draw-free (`:390-397`).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_npc_hit_sound("NPC_Hit_1", 5000.0, 0.0, &mut rng),
            None
        );
        assert_eq!(rng.peek(), peek, "the inaudible play is draw-free");
        // A trackable hit (case 42, `:1376-1381`): no gameplay draw, the
        // pinned first variation.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_npc_hit_sound("dd2_goblin_hurt_0", 0.0, 0.0, &mut rng),
            Some("dd2_goblin_hurt_0")
        );
        assert_eq!(rng.peek(), peek, "case 42 consumes no gameplay draw");
    }

    /// S1-E-21h — the projectile cue play's RNG contract: every ported
    /// site is a legacy id-2 style play, so the case-2 default pitch arm
    /// (`LegacySoundPlayer.cs:632`, one `Next(-6, 7)`) rides the caller's
    /// stream ONLY after the attenuation gate passed — the E21e/E21c
    /// attenuation-BEFORE-draws precedent, on the `UpdateProjectiles`
    /// stream the pool drain passes.
    #[test]
    fn s1_e_21h_the_proj_sound_draws_ride_the_passed_stream_only_when_audible() {
        use tr_core::unified_random::UnifiedRandom;
        // An audible bomb-blast play draws exactly one Next(-6, 7).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_range(-6, 7);
        let cue = audio.play_proj_sound((2, 14), 0.0, 0.0, 0.0, 1.0, &mut rng);
        assert_eq!(cue, Some("Item_14"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!((-0.06..=0.06).contains(&(roll as f32 * 0.01)));
        // Muted (soundVolume 0): the gate refuses before the switch —
        // no gameplay draw (`:369-372` precedes `:516`).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_proj_sound((2, 7), 0.0, 0.0, 0.0, 1.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted play is draw-free");
        // Out of attenuation range: also draw-free (`:390-397`).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_proj_sound((2, 60), 5000.0, 0.0, 0.0, 1.0, &mut rng),
            None
        );
        assert_eq!(rng.peek(), peek, "the inaudible play is draw-free");
        // The wave's whole style set resolves through the generated rows
        // to registered cues (the bank gaps closed: 14/17/33/74).
        for style in [5, 7, 8, 9, 11, 12, 14, 17, 20, 28, 33, 60, 74] {
            let mut audio = Audio::new(None);
            let mut rng = UnifiedRandom::new(3);
            let cue = audio
                .play_proj_sound((2, style), 0.0, 0.0, 0.0, 1.0, &mut rng)
                .unwrap_or_else(|| panic!("style {style} resolves"));
            assert_eq!(cue, SFX_CUES.iter().map(|(n, _)| *n).find(|n| *n == cue).unwrap());
        }
        // A non-id-2/42 family refuses draw-free.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_proj_sound((42, 0), 0.0, 0.0, 0.0, 1.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// W4-d21 fold 2 — the digtoise joy cue (`SoundID.PalDigtoiseJoy`,
    /// `Projectile.cs:38517`, style 289 / id 42): the trackable case-42
    /// path plays the pinned single variation, NO gameplay draw (the
    /// GetRandomPitch NextFloat rides the audio-only stream). An unlanded
    /// id-42 style (the achievement, style 0) still refuses draw-free.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_d21_the_digtoise_joy_cue_plays_the_trackable() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let cue = audio.play_proj_sound((42, 289), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("pal_digtoise_joy"));
        assert_eq!(rng.peek(), peek, "the trackable draws no gameplay roll");
        assert!(PROJ_CUSTOM_CUES.contains(&"pal_digtoise_joy"));
        assert_eq!(sound_type("pal_digtoise_joy"), 42);
        // The unlanded trackables keep the documented skip.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_proj_sound((42, 0), 0.0, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21h d76 — the second wave's bank gaps resolve (the cannon portal
    /// pair 114/115, the phantasmal pick 124/125, the golf strike 126, the
    /// drill hum 132, the whip tip 153/193, the sun dance 159, the missile
    /// latch 42) and the whip tip's id-3/4 NPC families ride the case-3/4
    /// façades (one gameplay pitch draw each, after the gate).
    #[test]
    fn s1_e_21h_d76_the_proj_bank_gaps_and_the_whip_npc_families() {
        use tr_core::unified_random::UnifiedRandom;
        for style in [42, 114, 115, 124, 125, 126, 132, 153, 159, 193] {
            let mut audio = Audio::new(None);
            let mut rng = UnifiedRandom::new(3);
            let cue = audio
                .play_proj_sound((2, style), 0.0, 0.0, 0.0, 1.0, &mut rng)
                .unwrap_or_else(|| panic!("style {style} resolves"));
            assert_eq!(cue, SFX_CUES.iter().map(|(n, _)| *n).find(|n| *n == cue).unwrap());
        }
        // The Crimson Whip's NPCHit9 (id 3): one Next(-10, 11) pitch draw.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_proj_sound((3, 9), 0.0, 0.0, 0.0, 1.0, &mut rng),
            Some("NPC_Hit_9")
        );
        assert_eq!(rng.peek(), witness.peek(), "the case-3 pitch draw");
        // The Corrupt Whip's NPCDeath1 (id 4): one Next(-10, 11) pitch draw.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_proj_sound((4, 1), 0.0, 0.0, 0.0, 1.0, &mut rng),
            Some("NPC_Killed_1")
        );
        assert_eq!(rng.peek(), witness.peek(), "the case-4 pitch draw");
        // The muted whip play draws nothing.

        }    /// S1-E-21h d75 part 2 — the Shatter play (`Projectile.cs:25943`,
    /// `PlaySound(13, x, y, 0, 0.35f)`): case 13 draws NOTHING
    /// (`LegacySoundPlayer.cs:783-792`), the gate precedes the play, the
    /// muted/inaudible plays refuse draw-free.
    fn s1_e_21h_d75_p2_the_shatter_play_is_draw_free() {
        let peek = rng.peek();
        assert_eq!(audio.play_shatter_sound(0.0, 0.0, 0.35), Some("Shatter"));
        assert_eq!(rng.peek(), peek, "case 13 consumes no gameplay draw");
        // Muted: the gate refuses.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        assert_eq!(audio.play_shatter_sound(0.0, 0.0, 0.35), None);
        // Out of attenuation range: refuses.
        assert_eq!(audio.play_shatter_sound(5000.0, 0.0, 0.35), None);
    }

    /// S1-E-21h d75 part 2 — the sonar trackable joins the drain's families:
    /// `(42, 311)` resolves to the pinned `sonar_potion` asset through wave
    /// A's façade — the `GetRandomPitch` NextFloat is audio-only, case 42
    /// draws nothing from the gameplay stream; the muted play refuses.
    fn s1_e_21h_d75_p2_the_sonar_trackable_rides_the_proj_drain() {
            audio.play_proj_sound((42, 311), 0.0, 0.0, 0.0, &mut rng),
            Some("sonar_potion")
        assert_eq!(rng.peek(), peek, "case 42 consumes no gameplay draw");
        assert!(ITEM_CUSTOM_CUES.contains(&"sonar_potion"), "the bank carries it");
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_proj_sound((4, 1), 0.0, 0.0, 0.0, 1.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21h d76 DOC-SWEEP — the volumeScale term rides the queue→drain
    /// path without touching the RNG contract: the Corrupt Whip's ×0.6 tip
    /// cue (`Projectile.cs:47720`, the case-4 post-tweak multiply
    /// `LegacySoundPlayer.cs:1411`) and the Lights Bane slash's `0.15*ai[0]`
    /// (`:42237`, the case-2 peer) draw EXACTLY the same gameplay draws as
    /// their volume-1 twins — the scale is a volume-only term. WRITTEN NOT
    /// RUN (code-only mode).
    #[test]
    fn s1_e_21h_d76_sweep_the_volume_scale_is_draw_free() {
        use tr_core::unified_random::UnifiedRandom;
        // The case-4 pair: ×0.6 vs 1.0 — one Next(-10, 11) pitch draw each.
        for vol in [0.6f32, 1.0] {
            let mut audio = Audio::new(None);
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let _ = witness.next_range(-10, 11);
            assert_eq!(
                audio.play_proj_sound((4, 1), 0.0, 0.0, 0.0, vol, &mut rng),
                Some("NPC_Killed_1"),
                "vol {vol}: the cue still plays"
            );
            assert_eq!(rng.peek(), witness.peek(), "vol {vol}: the same one pitch draw");
        }
        // The case-2 pair (the Lights Bane Item60): ×0.15 vs 1.0 — the same
        // case-2 draw count either way.
        for vol in [0.15f32, 1.0] {
            let mut audio = Audio::new(None);
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let cue1 = audio.play_proj_sound((2, 60), 0.0, 0.0, 0.0, vol, &mut rng);
            let mut audio2 = Audio::new(None);
            let cue2 = audio2.play_proj_sound((2, 60), 0.0, 0.0, 0.0, 1.0, &mut witness);
            assert_eq!(cue1, cue2, "vol {vol}: the same cue as the volume-1 twin");
            assert_eq!(rng.peek(), witness.peek(), "vol {vol}: the same case-2 draws");
        }
        // The scaled play still gates on attenuation BEFORE any draw: an
        // out-of-range ×0.6 play is draw-free.

        }        assert_eq!(audio.play_proj_sound((42, 311), 0.0, 0.0, 0.0, &mut rng), None);
        // The portal pair's bank gaps closed.
        for style in [114, 115] {
    /// S1-E-21h f1 — the fourth batch's bank gaps resolve (45/51/54/94/96/
    /// 110/167) and the int-id families route to their landed façades: the
    /// id-0 Dig thud (case 0's `Next(3)` variation + `Next(-10,11)` pitch),
    /// the id-13 Shatter (no draws) and the id-4 NPCDeath style-6 (case 4's
    /// one `Next(-10,11)` pitch). Written, NOT run — code-only mode.
    fn s1_e_21h_f1_the_bank_gaps_and_int_id_routes_resolve() {
        for style in [45, 51, 54, 94, 96, 110, 167] {
            let mut rng = UnifiedRandom::new(3);
            let cue = audio
                .play_proj_sound((2, style), 0.0, 0.0, 0.0, &mut rng)
                .unwrap_or_else(|| panic!("style {style} resolves"));
            assert_eq!(cue, SFX_CUES.iter().map(|(n, _)| *n).find(|n| *n == cue).unwrap());
    /// S1-E-21h d75 part 4 — the RainbowBoulder trackable joins the bank:
    /// d3's `rainbow_check_bounce` (`Projectile.cs:20104`) queues
    /// `(42, 274)`; the drain's id-42 façade resolves the pinned
    /// `rainbow_boulder` asset (`SoundID.cs:1709`, one variation, volume
    /// 1.0, variance 0). Case 42 draws NOTHING from the gameplay stream;
    /// the muted play refuses draw-free.
    fn s1_e_21h_d75_p4_the_rainbow_boulder_trackable_rides_the_proj_drain() {
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_proj_sound((4, 1), 5000.0, 0.0, 0.0, 0.6, &mut rng),
            None
        );
        assert_eq!(rng.peek(), peek, "the scaled inaudible play is draw-free");
            audio.play_proj_sound((42, 274), 0.0, 0.0, 0.0, &mut rng),
            Some("rainbow_boulder")
        assert_eq!(rng.peek(), peek, "case 42 consumes no gameplay draw");
        assert!(
            ITEM_CUSTOM_CUES.contains(&"rainbow_boulder"),
            "the bank carries it"
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_proj_sound((42, 274), 0.0, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21h d75 part 4 — the star chime's case-37 contract
    /// (`LegacySoundPlayer.cs:1326-1336`): TWO gameplay draws — the
    /// `Next(57, 59)` slot pick FIRST, then the `Next(-40, 41)` pitch;
    /// the Style×0.05 volume rides the façade (style 6 → ×0.30); the
    /// int overload draws no GetRandomPitch. Muted refuses draw-free.
    #[test]
    fn s1_e_21h_d75_p4_the_star_chime_draws_slot_then_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(57, 59);
        let _ = witness.next_range(-40, 41);
        let want = if slot == 57 { "Item_57" } else { "Item_58" };
        assert_eq!(audio.play_star_chime(6, 0.0, 0.0, &mut rng), Some(want));
        assert_eq!(rng.peek(), witness.peek(), "exactly two draws, slot first");
        assert!(SFX_CUES.contains(&("Item_57", 2)) && SFX_CUES.contains(&("Item_58", 2)));
        // Muted: the gate refuses before any draw.
        assert_eq!(audio.play_star_chime(10, 0.0, 0.0, &mut rng), None);
    /// S1-E-21h d75 part 4 — the FoxparksFlame façade pins the NAMED
    /// (2, 34) row at volume 0.15 (`SoundID.cs:1725`), not the plain
    /// Item34 (volume 1.0) the generic tuple lookup finds first. The
    /// style play draws one audio-only GetRandomPitch + one case-2
    /// gameplay pitch (`Next(-6, 7)`) after the gate; muted refuses
    /// draw-free.
    fn s1_e_21h_d75_p4_the_foxparks_flame_pins_the_named_quiet_row() {
        let row = crate::soundid::SOUND_STYLES
            .iter()
            .find(|s| s.name == "FoxparksFlame")
            .expect("the named row");
        assert_eq!((row.sound_id, row.style), (2, 34));
        assert_eq!(row.volume, 0.15, "the WithVolume(0.15f) arm");
        // id 0 — the Dig thud: case 0 draws the variation pick THEN the
        // pitch (`LegacySoundPlayer.cs:442`/`:450`), two gameplay draws.
        let variation = witness.next_range(0, 3);
        let _ = witness.next_range(-10, 11);
        let cue = audio.play_proj_sound((0, 0), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some(match variation { 0 => "Dig_0", 1 => "Dig_1", _ => "Dig_2" }));
        assert_eq!(rng.peek(), witness.peek(), "the Dig thud's two case-0 draws");
        // id 13 — the Shatter: case 13 draws NOTHING (`:783-792`).
        assert_eq!(audio.play_proj_sound((13, 0), 0.0, 0.0, 0.0, &mut rng), Some("Shatter"));
        assert_eq!(rng.peek(), peek, "the Shatter is draw-free");
        // id 4 style 6 — the NPCDeath play: case 4's one pitch draw
        // (`:693`), no GetRandomPitch (the int overload).
        assert_eq!(audio.play_proj_sound((4, 6), 0.0, 0.0, 0.0, &mut rng), Some("NPC_Killed_6"));
        assert_eq!(rng.peek(), witness.peek(), "the NPCDeath style-6 pitch draw");
        // Muted: every int-id route refuses before its case draws.
        for (id, style) in [(0u16, 0i32), (13, 0), (4, 6)] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_proj_sound((id, style), 0.0, 0.0, 0.0, &mut rng), None);
            assert_eq!(rng.peek(), peek, "the muted id-{id} play is draw-free");
        }
    /// S1-E-21h f1 fifth batch — the rocket families' Item62 bank gap
    /// resolves (`Kill_Bombs_DoUsualKillCode` `Projectile.cs:77737`-`:78146`),
    /// and the style plays its one case-2 `Next(-6, 7)` pitch draw on the
    /// caller's stream only when audible. Written, NOT run — code-only mode.
    fn s1_e_21h_f1_b5_the_rocket_item62_bank_gap_resolves() {
        let _ = witness.next_range(-6, 7);
        assert_eq!(audio.play_foxparks_flame(0.0, 0.0, &mut rng), Some("Item_34"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        let cue = audio.play_proj_sound((2, 62), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Item_62"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one case-2 pitch draw");
        assert_eq!(cue, SFX_CUES.iter().map(|(n, _)| *n).find(|n| *n == cue).unwrap());
        // Muted: the gate refuses before the case-2 switch — no draw.
        assert_eq!(audio.play_foxparks_flame(0.0, 0.0, &mut rng), None);
    /// S1-E-21h d77 deferred-arm wave — the queue's `volumeScale` term
    /// (`SoundEngine.cs:93`'s fourth argument): the DD2 Goblin Bomb's
    /// `Kill()` cue (`Projectile.cs:71906`) plays the case-2 style-14 asset
    /// at `type.Volume = 0.5` (`SoundID.cs:1596`'s `.WithVolume(0.5f)`).
    /// The scale rides `LegacySoundPlayer.cs:1411` AFTER the gate and the
    /// case-2 tweaks — the RNG census and the refusal shape stay the
    /// stock-scale play's. WRITTEN NOT RUN (code-only).
    fn s1_e_21h_d77_the_goblin_bomb_scale_rides_after_the_gate_draw_free_census() {
        assert_eq!(audio.play_proj_sound((2, 62), 0.0, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted Item62 play is draw-free");
    /// S1-E-21h f1 owed-arm phase — the Stardust jellyfish 539's NPCDeath
    /// style-7 play (`Projectile.cs:72571`, `PlaySound(4, pos, 7)` — the
    /// int overload, NO `GetRandomPitch` draw) routes to the new
    /// `NPC_Killed_7` bank row and plays case 4's ONE `Next(-10, 11)` pitch
    /// draw after the gate (`sound.rs` `npc_killed_case4`, the style-6
    /// precedent). Written, NOT run — code-only mode.
    fn s1_e_21h_f1_owed_the_jellyfish_npcdeath7_routes_and_draws() {
        // The bank row exists: name → id 4 (the NPC_Killed family).
            SFX_CUES.contains(&("NPC_Killed_7", 4)),
            "the NPC_Killed_7 bank row (the Sounds/NPC_Killed_7.xnb asset ships)"
        let cue = audio.play_proj_sound((4, 7), 0.0, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("NPC_Killed_7"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one case-4 pitch draw, no NextFloat");
        // Muted: the gate refuses before the case-4 draw.
        assert_eq!(audio.play_proj_sound((4, 7), 0.0, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted NPCDeath7 play is draw-free");
    /// S1-E-21h f1 final phase — the queue's sixth term (`volumeScale`,
    /// `SoundEngine.cs:93`'s fourth argument) adopted from d77's pushed
    /// seam (`9d35874a`). The DD2 Goblin Bomb's `Kill()` cue
    /// (`Projectile.cs:71906`) plays the case-2 style-14 asset at
    /// `type.Volume = 0.5` (`SoundID.cs:1596`'s `.WithVolume(0.5f)`); the
    /// scale rides `LegacySoundPlayer.cs:1411` AFTER the gate and the
    /// stock-scale play's. f1's int-overload arms (0/13/4) keep their
    /// routes at any scale (every f1 int site passes 1.0). Written, NOT
    /// run — code-only mode.
    fn s1_e_21h_f1_final_the_volume_scale_rides_after_the_gate_draw_free_census() {
        // The 0.5-scale play draws the SAME one Next(-6, 7) as the stock
        // play (the scale is not an RNG input) and resolves Item_14.
        let cue = audio.play_proj_sound_scaled((2, 14), 0.0, 0.0, 0.0, 0.5, &mut rng);
        assert_eq!(cue, Some("Item_14"), "the goblin bomb is the Item14 asset");
        // Muted: the gate refuses BEFORE the switch regardless of scale.
        assert_eq!(
            audio.play_proj_sound_scaled((2, 14), 0.0, 0.0, 0.0, 0.5, &mut rng),
        assert_eq!(rng.peek(), peek, "the muted scaled play is draw-free");
        // The style row itself keeps the C#'s 0.5 (`WithVolume`) — the
        // queue's term reproduces the call-site scale, not the row's.
            .find(|s| s.name == "DD2_GoblinBomb")
            .expect("the style row exists");
        assert_eq!((row.sound_id, row.style), (2, 14));
        assert_eq!(row.volume, 0.5, "SoundID.cs:1596 WithVolume(0.5f)");
        // The Phantasmal Eye's case-29 slot-103 moan resolves through the
        // fighter-moan seam: one Next(-10, 11) pitch draw after the gate,
        // the ×0.4 arm (`LegacySoundPlayer.cs:1025-1028`), bank row present.
        assert_eq!(rng.peek(), peek, "the muted 0.5-scale play is draw-free");
        // The int arms keep their façades under the scaled peer (f1's
        // drain shape): the id-4 cry still draws its ONE case-4 pitch.
        let cue = audio.play_fighter_moan(29, 103, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Zombie_103"), "the eye's death moan slot");
        assert_eq!(rng.peek(), witness.peek(), "exactly one pitch draw");
        assert!(SFX_CUES.iter().any(|(n, t)| *n == "Zombie_103" && *t == 29));
    /// S1-E-21h d77 re-probe wave — the id-4 int façade gains style 7 (the
    /// Stardust Jellyfish's `Kill()` cry, `Projectile.cs:72571`): one
    /// `Next(-10, 11)` pitch draw after the gate (the case-4 contract), the
    /// `NPC_Killed_7` bank row present, the muted play draw-free. WRITTEN
    /// NOT RUN (code-only).
    fn s1_e_21h_d77_the_jellyfish_cry_rides_the_case4_int_facade() {
            audio.play_proj_sound_scaled((4, 7), 0.0, 0.0, 0.0, 1.0, &mut rng),
            Some("NPC_Killed_7")
        assert_eq!(rng.peek(), witness.peek(), "the int arm ignores the scale");
    /// S1-E-21h f1 FINAL phase — the second owed wave's façade routes:
    /// the 1054 SpiderBoulder's NPCHit29 (`Projectile.cs:74867`, the int
    /// overload — NO NextFloat, case 3's 20..54 ×0.5 arm + ONE
    /// `Next(-10, 11)` pitch after the gate, `LegacySoundPlayer.cs:646-
    /// 672`), the 1047 RainbowBoulder's case-42 trackable (`:71194`,
    /// `SoundID.cs:1709` — one variation, NO gameplay draw) and the 655
    /// BeeHive's default-Style NPCDeath1 (`:75082`, case 4's one pitch).
    /// Written, NOT run — code-only mode.
    fn s1_e_21h_f1_final_the_second_owed_wave_routes_resolve() {
        // The bank rows exist.
        assert!(SFX_CUES.contains(&("NPC_Hit_29", 3)));
        assert!(SFX_CUES.contains(&("Item_89", 2)));
        assert!(NPC_CUSTOM_CUES.contains(&"rainbow_boulder"));
        // NPCHit29: exactly one case-3 pitch draw, no NextFloat.
        let cue = audio.play_npc_death_int(7, 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("NPC_Killed_7"), "the jellyfish's style-7 cry");
        assert!(SFX_CUES.iter().any(|(n, t)| *n == "NPC_Killed_7" && *t == 4));
        // Muted: the gate refuses BEFORE the draw.
        assert_eq!(audio.play_npc_death_int(7, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted cry is draw-free");
            audio.play_proj_sound((3, 29), 0.0, 0.0, 0.0, &mut rng),
            Some("NPC_Hit_29")
        assert_eq!(rng.peek(), witness.peek(), "one case-3 pitch, no NextFloat");
        // RainbowBoulder: the case-42 trackable draws NOTHING.
        assert_eq!(rng.peek(), peek, "the trackable play is draw-free");
        // NPCDeath1 (the BeeHive's default Style): one case-4 pitch.
            audio.play_proj_sound((4, 1), 0.0, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_1")
        assert_eq!(rng.peek(), witness.peek(), "one case-4 pitch");
        // Item89 (the 1037/1049 cue): the case-2 one-draw contract.
            audio.play_proj_sound((2, 89), 0.0, 0.0, 0.0, &mut rng),
            Some("Item_89")
        assert_eq!(rng.peek(), witness.peek(), "one case-2 pitch");
        // Muted: every route refuses before its draws.
        for (id, style) in [(3u16, 29i32), (42, 274), (4, 1), (2, 89)] {
            assert_eq!(rng.peek(), peek, "({id},{style}) muted is draw-free");
    }

    /// S1-E-21 wave C — the NPC death play's RNG contract
    /// (`LegacySoundPlayer.cs:673-695`): one `Next(-10, 11)` pitch draw
    /// (`:693`) after the gate; the muted play draws nothing.
    #[test]
    fn s1_e_21c_the_npc_death_draws_ride_the_passed_stream_only_when_audible() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_npc_death_sound("NPC_Killed_2", 0.0, 0.0, &mut rng),
            Some("NPC_Killed_2")
        );
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_npc_death_sound("NPC_Killed_2", 0.0, 0.0, &mut rng),
            None
        );
        assert_eq!(rng.peek(), peek);
        // The Deerclops' death trackable: no gameplay draw.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_npc_death_sound("deerclops_death", 0.0, 0.0, &mut rng),
            Some("deerclops_death")
        );
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21 wave C — the per-type cue selection rides the generated
    /// style table: the legacy asset name maps to `NPCHitN`/`NPCDeathN`
    /// (`SoundID.cs`), the trackable pins map back through the `trackable`
    /// field, and the case-3/4 volume arms reach the played style.
    #[test]
    fn s1_e_21c_the_hit_and_death_cues_resolve_their_style_rows() {
        let hit1 = npc_sound_style("NPC_Hit_1", "NPC_Hit_", 3).unwrap();
        assert_eq!((hit1.sound_id, hit1.style), (3, 1));
        let hit57 = npc_sound_style("NPC_Hit_57", "NPC_Hit_", 3).unwrap();
        assert_eq!((hit57.sound_id, hit57.style), (3, 57));
        let death10 = npc_sound_style("NPC_Killed_10", "NPC_Killed_", 4).unwrap();
        assert_eq!((death10.sound_id, death10.style), (4, 10));
        // The trackables: the `_0` pin maps back to its style; the
        // Deerclops' hit carries the 0.3 style volume (`SoundID.cs`'s
        // `WithVolume(0.3f)`).
        let gob = npc_sound_style("dd2_goblin_hurt_0", "NPC_Hit_", 3).unwrap();
        assert_eq!((gob.sound_id, gob.variations), (42, 6));
        let deer = npc_sound_style("deerclops_hit_0", "NPC_Hit_", 3).unwrap();
        assert_eq!(deer.sound_id, 42);
        assert!((deer.volume - 0.3).abs() < 1e-6);
        let deer_death = npc_sound_style("deerclops_death", "NPC_Killed_", 4).unwrap();
        assert_eq!((deer_death.sound_id, deer_death.variations), (42, 1));
        // The two bank-gap closures: the Headless Horseman's rows.
        assert!(sfx_cue_registered("NPC_Hit_12"));
        assert!(sfx_cue_registered("NPC_Killed_18"));
        // An unknown name resolves to nothing (a bank gap, never a guess).
        assert!(npc_sound_style("NPC_Hit_999", "NPC_Hit_", 3).is_none());
    }

    /// S1-E-21 wave B — the tile-place play's RNG contract: the case-0
    /// `Main.rand.Next(3)` variation pick + `Next(-10, 11) * 0.01` pitch
    /// (`LegacySoundPlayer.cs:442-450`) ride the caller's stream (the
    /// `UpdatePlayers` swap at the placement sites) in that order; the
    /// Coins case (`:865-869`) draws NOTHING; a muted play draws nothing
    /// (the C# returns at `:433-436` before the switch).
    #[test]
    fn s1_e_21b_the_dig_variation_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let variation = witness.next_range(0, 3);
        let _pitch = witness.next_range(-10, 11) as f32 * 0.01;
        let cue = audio.play_place_sound((0, 1), 0.0, 0.0, &mut rng);
        let want = match variation {
            0 => "Dig_0",
            1 => "Dig_1",
            _ => "Dig_2",
        };
        assert_eq!(cue, Some(want), "the Next(3) pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // Coins (case 18) draws nothing — no pitch, no variation.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let cue = audio.play_place_sound((18, 1), 0.0, 0.0, &mut rng);
        assert_eq!(cue, Some("Coins"));
        assert_eq!(rng.peek(), peek, "the Coins play consumes no draw");
        // The style arms (127 → Item30, 314 → Item52) ride wave A's case-2
        // path: one Next(-6, 7) pitch draw after the gate, plus the
        // GetRandomPitch NextFloat on the audio-only stream.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            audio.play_place_sound((2, 30), 0.0, 0.0, &mut rng),
            Some("Item_30")
        );
        assert_eq!(rng.peek(), witness.peek(), "one case-2 gameplay draw");
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            audio.play_place_sound((2, 52), 0.0, 0.0, &mut rng),
            Some("Item_52")
        );
        assert_eq!(rng.peek(), witness.peek());
        // Muted (soundVolume 0): every arm draws NOTHING from the gameplay
        // stream — the gate precedes the switch (`:433-436`).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        for cue_id in [(0u16, 1u16), (18, 1), (2, 30), (2, 52)] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_place_sound(cue_id, 0.0, 0.0, &mut rng), None);
            assert_eq!(rng.peek(), peek, "{cue_id:?}: the muted play is draw-free");
        }
        // Every placement cue resolves to a registered bank asset.
        for name in ["Dig_0", "Dig_1", "Dig_2", "Coins", "Item_30", "Item_52"] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
    }

    /// S1-E-21 wave D — the fast-run footstep's RNG contract: case 17's
    /// `Main.rand.Next(-10, 11) * 0.01` pitch (`LegacySoundPlayer.cs:862`)
    /// rides the caller's stream (the `UpdatePlayers` swap) and runs only
    /// after the attenuation gate passed; the int overload draws no
    /// `GetRandomPitch` NextFloat (`SoundEngine.cs:102-108`). The Moon Lord
    /// legs' Item-24 arm rides wave A's style façade instead.
    #[test]
    fn s1_e_21d_the_footstep_pitch_draw_rides_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        // Audible: exactly one Next(-10, 11) gameplay draw, nothing else.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_range(-10, 11);
        assert_eq!(audio.play_step_sound((17, -1), 0.0, 0.0, &mut rng), Some("Run"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!(roll.abs() <= 10, "the window is -10..=10");
        // Muted (soundVolume 0): the gate precedes the switch — no draw.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_step_sound((17, -1), 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted play is draw-free");
        // The Moon Lord legs' arm (type 2 style 24): wave A's façade — one
        // case-2 Next(-6, 7) pitch draw on the gameplay stream after the
        // gate (`Item_24` has no volume/pitch special arm).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            audio.play_step_sound((2, 24), 0.0, 0.0, &mut rng),
            Some("Item_24")
        );
        assert_eq!(rng.peek(), witness.peek(), "one case-2 gameplay draw");
        // Both footstep cues resolve to registered bank assets.
        assert_eq!(sound_type("Run"), 17);
        for name in ["Run", "Item_24"] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
    }

    /// W4-c14 - the Meowcart case-37 contract (`Mount.cs:659-667`,
    /// `LegacySoundPlayer.cs:1326-1336`): TWO draws per play - the
    /// `Next(57, 59)` slot pick then the `Next(-40, 41)` pitch - on the
    /// caller's stream after the gate; the style scales the volume
    /// (5 × 0.05 landing / 3 × 0.05 bumper); a muted play draws nothing.
    #[test]
    fn w4_c14_the_meowcart_case37_draws_the_slot_then_the_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(57, 59);
        let _ = witness.next_range(-40, 41);
        let cue = audio
            .play_meowcart_sound(5, 0.0, 0.0, &mut rng)
            .expect("audible");
        assert_eq!(cue, if slot == 57 { "Item_57" } else { "Item_58" });
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // The case maths: the slot window, the volume scale, the pitch window.
        let mut roll = UnifiedRandom::new(3);
        let (slot, volume, pitch) =
            sound::meowcart_case37(5, &mut |lo, hi| roll.next_range(lo, hi));
        assert!((57..59).contains(&slot));
        assert!((volume - 0.25).abs() < 1e-6, "5 * 0.05");
        assert!(pitch.abs() <= 0.40 + 1e-6);
        let (_, volume, _) = sound::meowcart_case37(3, &mut |_, _| 57);
        assert!((volume - 0.15).abs() < 1e-6, "3 * 0.05");
        // Muted: draw-free.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_meowcart_sound(5, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
        // Both slot assets resolve to the legacy id 37.
        for name in ["Item_57", "Item_58"] {
            assert_eq!(sound_type(name), 37, "{name}");
        }
    }

    /// W4-c14 doc-sweep — the 53/55/159 playing guards
    /// (`LegacySoundPlayer.cs:559-562`/`:583-586`) refuse a fresh play while
    /// the slot's instance is live, BEFORE the case-2 pitch draw. Headless
    /// (no output device) no slot ever plays, so a second cart-rattle play
    /// still draws — the engine's fresh-instance behaviour (the case-28
    /// witness precedent). The guard's refusal path is pinned by feeding a
    /// live slot directly.
    #[test]
    fn w4_c14_the_cart_rattle_guard_refuses_while_the_slot_is_live() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Headless: the 55 rattle plays twice, one case-2 draw each (the
        // slot never fills without a device, so the guard never trips).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-20, -11);
        let _ = witness.next_range(-20, -11);
        assert_eq!(audio.play_npc_item_cue(55, 0.0, 0.0, &mut rng), Some("Item_55"));
        assert_eq!(audio.play_npc_item_cue(55, 0.0, 0.0, &mut rng), Some("Item_55"));
        assert_eq!(rng.peek(), witness.peek(), "two rattles, two pitch draws");
        // The refusal path, pinned through the pure guard decision (the
        // `mech_case28`/`npc_hit_case3` precedent): a LIVE 53/55/159 slot
        // refuses; every other style plays regardless of its slot.
        for style in [53, 55, 159] {
            assert!(sound::item_case2_playing_guard(style, true), "{style} refuses live");
            assert!(!sound::item_case2_playing_guard(style, false), "{style} plays idle");
        }
        for style in [1, 24, 37, 56, 168] {
            assert!(!sound::item_case2_playing_guard(style, true), "{style} never guards");
        }
        // A live slot's read is the `!sink.empty()` predicate the facade uses.
        assert!(
            !audio.item_slots.get(&55).is_some_and(|s| !s.empty()),
            "headless the slot stays empty/absent, the guard reads not-playing"
        );
        // A muted rattle is draw-free (the gate precedes the guard + draw).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_npc_item_cue(55, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21e - the fighter moans' RNG contract: the moan arms' rolls ride
    /// the caller's stream (`UpdateNPCs`, `Main.cs:17767`), and each legacy
    /// case's slot/pitch draws run AFTER the attenuation gate on the SAME
    /// stream (`LegacySoundPlayer.cs:369-436`'s prologue precedes the
    /// switch). Case 14 draws one slot and no pitch (`:793-824`), case 26 a
    /// slot + a pitch (`:969-978`), case 29 one pitch behind the playing
    /// guard (`:1008-1050`); a muted or out-of-range play consumes nothing.
    #[test]
    fn s1_e_21e_the_moan_draws_ride_the_passed_stream_after_the_gate() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Case 14 default (`:817`): exactly one Next(3) slot draw, NO pitch.
        // Seed 7 draws 1 → Zombie_1.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _slot = witness.next_range(0, 3);
        assert_eq!(
            audio.play_fighter_moan(14, 1, 0.0, 0.0, &mut rng),
            Some("Zombie_1")
        );
        assert_eq!(rng.peek(), witness.peek(), "one slot draw, no pitch draw");
        // Case 14 style 542 (`:796-803`): slot 7 pinned, ZERO draws.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_fighter_moan(14, 542, 0.0, 0.0, &mut rng),
            Some("Zombie_7")
        );
        assert_eq!(rng.peek(), peek, "the pinned slot draws nothing");
        // Case 14 styles 489/586 (`:805-813`): the Next(21, 24) slot arm,
        // still no pitch. Seed 7 draws 22 → Zombie_22.
        for style in [489, 586] {
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let _slot = witness.next_range(21, 24);
            assert_eq!(
                audio.play_fighter_moan(14, style, 0.0, 0.0, &mut rng),
                Some("Zombie_22")
            );
            assert_eq!(rng.peek(), witness.peek(), "one slot draw, no pitch draw");
        }
        // Case 26 (`:969-978`): Next(3, 5) + one Next(-10, 11) pitch. Seed 7
        // draws 3 then 8 → Zombie_3.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _slot = witness.next_range(3, 5);
        let _pitch = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_fighter_moan(26, 1, 0.0, 0.0, &mut rng),
            Some("Zombie_3")
        );
        assert_eq!(rng.peek(), witness.peek(), "slot + pitch");
        // Case 29 (`:1008-1050`): the Style IS the slot, one pitch draw.
        // Seed 7 draws -2; the moan styles 6/7/8 keep mult 1.0.
        for style in [6, 7, 8] {
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let _pitch = witness.next_range(-10, 11);
            assert_eq!(
                audio.play_fighter_moan(29, style, 0.0, 0.0, &mut rng),
                Some(match style {
                    6 => "Zombie_6",
                    7 => "Zombie_7",
                    _ => "Zombie_8",
                })
            );
            assert_eq!(rng.peek(), witness.peek(), "one pitch draw");
        }
        // Muted (soundVolume 0): cases 14/26/29 are NOT ambient-scaling
        // (`LegacySoundPlayer.cs:1444-1457`) — the gate precedes the switch
        // and the play is draw-free.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        for (case, style) in [(14u32, 1i32), (14, 542), (14, 489), (26, 1), (29, 7)] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_fighter_moan(case, style, 0.0, 0.0, &mut rng), None);
            assert_eq!(rng.peek(), peek, "case {case}: the muted play is draw-free");
        }
        // Out of range (distance >= SOUND_ATTENUATION_DISTANCE): the gate
        // refuses before any draw, volume back at 1.
        let mut l = audio.listener;
        l.sound_volume = 1.0;
        audio.set_listener(l);
        for (case, style) in [(14u32, 1i32), (26, 1), (29, 7)] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(
                audio.play_fighter_moan(case, style, 3000.0, 0.0, &mut rng),
                None
            );
            assert_eq!(rng.peek(), peek, "case {case}: the far play is draw-free");
        }
        // Every moan asset resolves to a registered bank row.
        for name in [
            "Zombie_0",
            "Zombie_1",
            "Zombie_2",
            "Zombie_3",
            "Zombie_4",
            "Zombie_5",
            "Zombie_6",
            "Zombie_7",
            "Zombie_8",
            "Zombie_21",
            "Zombie_22",
            "Zombie_23",
            "Zombie_109",
        ] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
    }

    /// W4-f2 — the IdleSounds case-29 bank rows (styles 24-87,
    /// `NPC.cs:90081-90205`): every slot registers at legacy id 29, the
    /// play resolves the slot's own asset with ONE pitch draw on the
    /// passed stream (the case-29 contract), and the ×0.5 band rides the
    /// volume (the headless play returns the asset name).
    #[test]
    fn w4_f2_the_idle_case29_bank_rows_play_with_one_pitch_draw() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        for style in 24i32..=87 {
            let name = format!("Zombie_{style}");
            assert!(
                SFX_CUES.iter().any(|(n, t)| *n == name && *t == 29),
                "{name} registers at legacy id 29"
            );
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let _pitch = witness.next_range(-10, 11);
            let played = audio.play_fighter_moan(29, style, 0.0, 0.0, &mut rng);
            assert_eq!(played.map(|s| s.to_string()), Some(name.clone()));
            assert_eq!(rng.peek(), witness.peek(), "style {style}: one pitch draw");
        }
    }

    /// W4-f2 reconcile (e9 union, research/94 §3/§5) — the Vile Spit's
    /// int style-9 cry (`NPC.cs:21480`) and the Ghost's NPCDeath55 façade
    /// (`:24693`): both resolve to registered bank rows, the int peer
    /// draws ONE case-4 pitch with no NextFloat, the façade adds the
    /// audio-only NextFloat; muted plays draw nothing.
    #[test]
    fn w4_e9_the_spit_and_ghost_death_rows_follow_the_overload_split() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        assert!(sfx_cue_registered("NPC_Killed_9"));
        assert!(sfx_cue_registered("NPC_Killed_55"));
        // The int overload (style 9): one Next(-10, 11), no NextFloat.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        let style_before = audio.sound_style_random.clone();
        assert_eq!(
            audio.play_npc_death_int(9, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_9")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_eq!(audio.sound_style_random, style_before, "no NextFloat");
        // The style façade (NPCDeath55): the same draw + the NextFloat.
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(
            audio.play_npc_death_style(55, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_55")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_ne!(audio.sound_style_random, style_before, "the NextFloat");
        // Muted: both peers draw nothing (the gate precedes the switch).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_npc_death_int(9, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_death_style(55, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// W4-f2 closeout (e9b tail union, research/94 §10) — the tail bank
    /// rows: Medusa's int style-17 gaze (`NPC.cs:56403`), the balloon
    /// pops' NPCDeath63 façade (`:43726`/`:47951`/`:48062`), the Queen
    /// Slime's case-65 moan bank (`:45759` → `LegacySoundPlayer.cs:1382-
    /// 1402`, ONE `Next(115, 118)` slot draw, no pitch), the four Item
    /// cues (`Item167/170/172/150` — case 2's one `Next(-6, 7)` each) and
    /// the Pal/StatueMimic trackables (case 42 — the façade NextFloat on
    /// the audio-only stream, NO gameplay draw, the laugh's `_0` pin).
    #[test]
    fn w4_f2_the_tail_bank_rows_follow_the_overload_split() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        for name in [
            "NPC_Killed_17",
            "NPC_Killed_63",
            "Zombie_115",
            "Zombie_116",
            "Zombie_117",
            "Item_167",
            "Item_170",
            "Item_172",
            "Item_150",
        ] {
            assert!(sfx_cue_registered(name), "{name} in the bank");
        }
        for name in [
            "pal_cattiva_pain",
            "pal_foxparks_pain",
            "pal_cattiva_joy",
            "pal_foxparks_joy",
            "statuemimic_scare",
            "statuemimic_jump",
            "statuemimic_laugh_0",
        ] {
            assert!(NPC_CUSTOM_CUES.contains(&name), "{name} in the custom list");
        }
        // Medusa's gaze: the int overload, one case-4 pitch, no NextFloat.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        let style_before = audio.sound_style_random.clone();
        assert_eq!(
            audio.play_npc_death_int(17, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_17")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_eq!(audio.sound_style_random, style_before, "no NextFloat");
        // The balloon pop: the façade adds the audio-only NextFloat.
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(
            audio.play_npc_death_style(63, 0.0, 0.0, &mut rng),
            Some("NPC_Killed_63")
        );
        assert_eq!(rng.peek(), witness.peek(), "one gameplay draw");
        assert_ne!(audio.sound_style_random, style_before, "the NextFloat");
        // The Queen Slime moan: ONE slot draw Next(115, 118), no pitch.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(115, 118);
        let style_before = audio.sound_style_random.clone();
        let expected: &'static str = match slot {
            115 => "Zombie_115",
            116 => "Zombie_116",
            _ => "Zombie_117",
        };
        let played = audio
            .play_queen_slime_moan(0.0, 0.0, &mut rng)
            .expect("the gate passes");
        assert_eq!(played, expected);
        assert_eq!(rng.peek(), witness.peek(), "the slot pick is the only draw");
        assert_eq!(audio.sound_style_random, style_before, "no NextFloat");
        // The four Item cues: the façade + case 2's one Next(-6, 7).
        let mut witness2 = UnifiedRandom::new(7);
        let _ = witness2.next_range(-6, 7);
        for style in [167, 170, 172, 150] {
            let mut rng = UnifiedRandom::new(7);
            let expected: &'static str = match style {
                167 => "Item_167",
                170 => "Item_170",
                172 => "Item_172",
                _ => "Item_150",
            };
            let played = audio
                .play_npc_item_cue(style, 0.0, 0.0, &mut rng)
                .expect("the bank row resolves");
            assert_eq!(played, expected);
            assert_eq!(rng.peek(), witness2.peek(), "style {style}: one pitch");
        }
        // The trackables: NO gameplay draw; the façade NextFloat rides
        // the audio-only stream; the laugh pins `_0`.
        for cue in [
            "pal_cattiva_pain",
            "pal_foxparks_joy",
            "statuemimic_scare",
            "statuemimic_laugh_0",
        ] {
            let style_before = audio.sound_style_random.clone();
            assert_eq!(audio.play_npc_trackable(cue, 0.0, 0.0), Some(cue));
            assert_ne!(audio.sound_style_random, style_before, "{cue}: the NextFloat");
        }
        // Unregistered names refuse (the bank-gap guard); eow_dig is a
        // registered NPC custom cue but has its own play (play_eow_dig).
        assert_eq!(audio.play_npc_trackable("eow_dig", 0.0, 0.0), Some("eow_dig"));
        assert_eq!(audio.play_npc_trackable("not_a_cue", 0.0, 0.0), None);
        // Muted: every tail play draws nothing on either stream.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let style_before = audio.sound_style_random.clone();
        assert_eq!(audio.play_npc_death_int(17, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_death_style(63, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_queen_slime_moan(0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_item_cue(167, 0.0, 0.0, &mut rng), None);
        assert_eq!(audio.play_npc_trackable("pal_cattiva_pain", 0.0, 0.0), None);
        assert_eq!(rng.peek(), peek, "no gameplay draws while muted");
        // The façade NextFloats are consumed BEFORE the gate
        // (`SoundEngine.cs:99`) — the audio-only stream still advances on
        // the façade peers; the int peers (17/65) never draw it.
        assert_ne!(audio.sound_style_random, style_before);
    }

    /// S1-E-21i — the wall-break play's RNG contract
    /// (`WorldGen.cs:60423-60441`'s four arms): Shatter (13) draws NOTHING,
    /// Grass (6) draws one `Next(-30, 31)`, NPCHit25 draws one
    /// `Next(-10, 11)` (wave C's façade), Dig (0) draws the `Next(3)`
    /// variation + one `Next(-10, 11)` (wave B's case 0) — each on the
    /// caller's stream, only after the attenuation gate passed.
    #[test]
    fn s1_e_21i_the_wall_break_draws_ride_the_passed_stream_only_when_audible() {
        use tr_core::unified_random::UnifiedRandom;
        // Shatter: no gameplay draw, no pitch (`LegacySoundPlayer.cs:783-792`).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_wall_break_sound((13, 1), 0.0, 0.0, &mut rng),
            Some("Shatter")
        );
        assert_eq!(rng.peek(), peek, "case 13 consumes no gameplay draw");
        // Grass: exactly one Next(-30, 31) (`:714`).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_range(-30, 31);
        assert_eq!(
            audio.play_wall_break_sound((6, 1), 0.0, 0.0, &mut rng),
            Some("Grass")
        );
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!(roll.abs() <= 30, "the window is -30..=30");
        // NPCHit25: wave C's case-3 façade — one Next(-10, 11) (`:670`).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_wall_break_sound((3, 25), 0.0, 0.0, &mut rng),
            Some("NPC_Hit_25")
        );
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Dig: wave B's case-0 path — Next(3) + Next(-10, 11) (`:442`/:450).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let variation = witness.next_range(0, 3);
        let _ = witness.next_range(-10, 11);
        let cue = audio.play_wall_break_sound((0, 1), 0.0, 0.0, &mut rng);
        let want = match variation {
            0 => "Dig_0",
            1 => "Dig_1",
            _ => "Dig_2",
        };
        assert_eq!(cue, Some(want), "the variation pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // Muted (soundVolume 0): every arm is draw-free — the gate
        // (`:369-436`) precedes the switch.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        for cue_id in [(13u16, 1u16), (6, 1), (3, 25), (0, 1)] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_wall_break_sound(cue_id, 0.0, 0.0, &mut rng), None);
            assert_eq!(rng.peek(), peek, "{cue_id:?}: the muted play is draw-free");
        }
        // Out of attenuation range: also draw-free (`:390-397`).
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            audio.play_wall_break_sound((6, 1), 5000.0, 0.0, &mut rng),
            None
        );
        assert_eq!(rng.peek(), peek);
        // The wall cues resolve to registered bank assets.
        assert_eq!(sound_type("Mech_0"), 28);
        for name in ["Shatter", "Grass", "NPC_Hit_25", "Dig_0", "Mech_0"] {
            assert!(SFX_CUES.iter().any(|(n, _)| *n == name), "{name} in the table");
        }
    }

    /// S1-E-21i — the door play's RNG contract: cases 8/9
    /// (`LegacySoundPlayer.cs:736`/`:747`) draw exactly one
    /// `Next(-20, 21) * 0.01` on the caller's stream after the gate; the
    /// int overload draws no `GetRandomPitch` NextFloat; a muted or
    /// out-of-range play draws nothing.
    #[test]
    fn s1_e_21i_the_door_pitch_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        for open in [true, false] {
            let mut audio = Audio::new(None);
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let roll = witness.next_range(-20, 21);
            let want = if open { "Door_Opened" } else { "Door_Closed" };
            assert_eq!(audio.play_door_sound(open, 0.0, 0.0, &mut rng), Some(want));
            assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
            assert!(roll.abs() <= 20, "the window is -20..=20");
        }
        // Muted: both arms draw-free.
        let mut audio = Audio::new(None);
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        for open in [true, false] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_door_sound(open, 0.0, 0.0, &mut rng), None);
            assert_eq!(rng.peek(), peek, "the muted door play is draw-free");
        }
        // Out of range: draw-free too.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_door_sound(true, 5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// S1-E-21i — the switch-click play's RNG contract: case 28
    /// (`LegacySoundPlayer.cs:997-1007`) draws exactly one
    /// `Next(-10, 11) * 0.01` (`:1005`) on the caller's stream after the
    /// gate; the playing guard (`:998-1001`) refuses BEFORE the draw; a
    /// muted or out-of-range play draws nothing. Headless (no output
    /// device) the slot never plays, so the guard never trips — the
    /// engine's fresh-instance behaviour.
    #[test]
    fn s1_e_21i_the_switch_click_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_range(-10, 11);
        assert_eq!(audio.play_switch_click(0.0, 0.0, &mut rng), Some("Mech_0"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        assert!(roll.abs() <= 10, "the window is -10..=10");
        // A second click still plays headless — the sink is empty (the
        // C#'s guard reads a Playing instance; none exists without a
        // device), so another draw rides the stream.
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        let _ = witness.next_range(-10, 11);
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(audio.play_switch_click(0.0, 0.0, &mut rng), Some("Mech_0"));
        assert_eq!(audio.play_switch_click(0.0, 0.0, &mut rng), Some("Mech_0"));
        assert_eq!(rng.peek(), witness.peek(), "two clicks, two draws");
        // Muted: draw-free.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_switch_click(0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted click is draw-free");
        // Out of range: draw-free.
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_switch_click(5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }

    /// The front-end ladder (`Main.cs:12455-12532`'s `if (Main.gameMenu)`
    /// arm): the title tracks are picked without any world state, and the
    /// menu arm returns before every world branch — a boss nearby or a
    /// rain storm cannot bleed into it (BUG-01).
    #[test]
    fn the_menu_ladder_picks_the_title_track() {
        use tr_core::seeds::SeedFlags;
        // No seed flags: the tail's `default:` → the title track 51.
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51, "the plain title track");
        // The vanilla order: the world-dev flags first, then the specials.
        for (bit, want) in [
            (SeedFlags::DRUNK, 60),
            (SeedFlags::REMIX, 8),
            (SeedFlags::TENTH_ANNIVERSARY, 11),
        ] {
            let mut m = system();
            let mut e = env(false);
            e.menu = true;
            e.menu_flags = SeedFlags::from(bit);
            m.decide_new_music(e);
            assert_eq!(m.new_music, want, "flag {bit}");
        }
        // Drunk + remix outranks the drunk default (`:12470`), and the
        // not-the-bees 86 is nested *inside* the drunk arm (`:12488`) — it
        // never fires on its own.
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK) | SeedFlags::REMIX.into();
        m.decide_new_music(e);
        assert_eq!(m.new_music, 70, "drunk + remix");
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK) | SeedFlags::NOT_THE_BEES.into();
        m.decide_new_music(e);
        assert_eq!(m.new_music, 86, "drunk + not the bees");
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::NOT_THE_BEES);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51, "not the bees alone is not drunk");
        // The menu arm returns: the world state it is fed is irrelevant.
        let mut m = system();
        let mut e = env(true);
        e.menu = true;
        e.boss_music = Some(93);
        e.rain = true;
        e.blood_moon = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51, "the menu arm is exclusive");
    }

    /// W4-g22 MENU-FINAL witness — the three ported menu-track arms: the
    /// shimmer-ocean drunk title 72 (`Main.cs:12465-12468`, the property
    /// `:2686-2696`), the vampire drunk title 75 (`:12469-12472`,
    /// `WorldGen.cs:555-558`), the credits JourneysEnd 89 (`:12486-12489`)
    /// — plus the ladder order between them and the title track's
    /// no-cross-fade jump (`:12217` + `:12521-12524`).
    #[test]
    fn menu_final_the_missing_title_arms_land() {
        use tr_core::seeds::{MicroSeedFlags, SeedFlags};
        // Drunk + anniversary without remix/everything/bees = the property
        // holds -> 72, ahead of the plain drunk 60.
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK | SeedFlags::TENTH_ANNIVERSARY);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 72, "the shimmer-ocean title");
        // Every property gate cuts it back: remix wins with 70 first,
        // everything/bees drop to the plain drunk 60, no anniversary too.
        for (extra, want) in [
            (SeedFlags::REMIX, 70),
            (SeedFlags::ZENITH, 60),
            (SeedFlags::NOT_THE_BEES, 86),
        ] {
            let mut m = system();
            let mut e = env(false);
            e.menu = true;
            e.menu_flags =
                SeedFlags::from(SeedFlags::DRUNK | SeedFlags::TENTH_ANNIVERSARY | extra);
            m.decide_new_music(e);
            assert_eq!(m.new_music, want, "gate {extra}");
        }
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 60, "drunk alone is not shimmer-ocean");
        // The vampirism micro seed plays 75 inside the drunk arm only.
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK);
        e.menu_micro_flags = MicroSeedFlags::from(MicroSeedFlags::VAMPIRISM);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 75, "the vampire title");
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_micro_flags = MicroSeedFlags::from(MicroSeedFlags::VAMPIRISM);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51, "vampirism alone is not drunk");
        // The shimmer-ocean arm outranks the vampire arm (C# else-if order).
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK | SeedFlags::TENTH_ANNIVERSARY);
        e.menu_micro_flags = MicroSeedFlags::from(MicroSeedFlags::VAMPIRISM);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 72, "shimmer-ocean before vampire");
        // Credits: 89 outside the seed arms, AFTER remix's 8, BEFORE the
        // anniversary's 11 (the `:12486` else-if position).
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_credits = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 89, "the credits track");
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_credits = true;
        e.menu_flags = SeedFlags::from(SeedFlags::REMIX);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 8, "remix outranks the credits");
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_credits = true;
        e.menu_flags = SeedFlags::from(SeedFlags::TENTH_ANNIVERSARY);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 89, "the credits outrank the anniversary");
        // The plain title jumps its fade (musicNoCrossFade[51], `:12217`).
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 51);
        assert_eq!(m.music_fade[51], 1.0, "no cross-fade on the title");
        // The other picks do NOT jump (only 51 is in the no-fade set).
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.menu_flags = SeedFlags::from(SeedFlags::DRUNK);
        m.decide_new_music(e);
        assert_eq!(m.music_fade[60], 0.0, "the drunk title ramps normally");
    }

    /// The front-end tick keeps the volume gate: `musicVolume == 0` silences
    /// the menu track the same way as the world ones (`Main.cs:12461`).
    #[test]
    fn the_menu_track_honours_the_zero_volume_gate() {
        let mut m = system();
        let mut e = env(false);
        e.menu = true;
        e.music_volume = 0.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 0);
    }

    /// S1-E-21f — the player hurt/death/drown plays' RNG contract: case 1
    /// draws exactly ONE `Next(3)` slot pick on the passed stream
    /// (`LegacySoundPlayer.cs:505`), cases 5/23 draw NOTHING; a muted gate
    /// refuses before any draw. The int overload draws no GetRandomPitch.
    #[test]
    fn s1_e_21f_the_hurt_death_drown_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Case 1: one Next(0, 3) — the Player_Hit_<n> slot pick.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(0, 3);
        let cue = audio.play_hurt_sound(0.0, &mut rng);
        let want = match slot {
            0 => "Player_Hit_0",
            1 => "Player_Hit_1",
            _ => "Player_Hit_2",
        };
        assert_eq!(cue, Some(want), "the slot pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Case 5 (death) and case 23 (drown): Volume/Pan only — no draws.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_death_sound(0.0, 0.0), Some("Player_Killed"));
        assert_eq!(audio.play_drown_sound(), Some("Drown"));
        assert_eq!(rng.peek(), peek, "the draw-free cases consume nothing");
        // Muted: the gate precedes the switch — every play is draw-free.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_hurt_sound(0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted hurt is draw-free");
        assert_eq!(audio.play_death_sound(0.0, 0.0), None);
        assert_eq!(audio.play_drown_sound(), None);
        // The bank rows resolve (the case-38 coin family included).
        assert_eq!(sound_type("Player_Hit_1"), 1);
        assert_eq!(sound_type("Player_Hit_2"), 1);
        assert_eq!(sound_type("Drown"), 23);
        assert_eq!(sound_type("Coin_0"), 38, "the id-18 alias is retired");
        for n in 0..5 {
            let name = format!("Coin_{n}");
            assert!(SFX_CUES.iter().any(|(c, _)| *c == name), "{name} in the table");
        }
    }

    /// W4-e10 — the player-voice plays' RNG contract: case 20 (female hurt)
    /// draws exactly ONE `Next(3)` slot pick (`LegacySoundPlayer.cs:913`),
    /// the chicken-rare pick draws ONE `Next(5)` (`Player.cs:38602`) then the
    /// style façade draws NO gameplay roll (the trackable case 42), the
    /// mount/armor NPC int arms draw ONE `Next(-10, 11)` pitch (case 3/4);
    /// a muted gate refuses before any draw. The style façade's
    /// `GetRandomPitch` NextFloat rides the audio-only stream, never the
    /// passed gameplay stream.
    #[test]
    fn w4_e10_the_voice_plays_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Case 20: one Next(0, 3) — the Female_Hit_<n> slot pick.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(0, 3);
        let cue = audio.play_female_hurt_sound(0.0, &mut rng);
        let want = match slot {
            0 => "Female_Hit_0",
            1 => "Female_Hit_1",
            _ => "Female_Hit_2",
        };
        assert_eq!(cue, Some(want), "the slot pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // The chicken-rare pick: one Next(0, 5), then the trackable façade
        // draws nothing on the gameplay stream.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let rare = witness.next_range(0, 5) == 0;
        let roll = rng.next_range(0, 5);
        let name = if roll == 0 { "ChickenHurtRare" } else { "ChickenHurt" };
        audio.play_voice_style(name, -1.0, -1.0, 0.0, &mut rng);
        assert_eq!(rare, roll == 0);
        assert_eq!(rng.peek(), witness.peek(), "the façade draws no gameplay roll");
        // The mount/armor NPC int arms: one Next(-10, 11) pitch (case 3).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        witness.next_range(-10, 11);
        audio.play_hurt_npc_int(3, 6, -1.0, -1.0, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "case 3 draws one pitch");
        // Muted: the gate precedes every draw.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_female_hurt_sound(0.0, &mut rng), None);
        assert_eq!(audio.play_hurt_npc_int(3, 6, -1.0, -1.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted voice plays are draw-free");
        // The bank rows resolve.
        assert_eq!(sound_type("Female_Hit_1"), 20);
        assert_eq!(sound_type("NPC_Hit_47"), 3);
        assert_eq!(sound_type("Item_35"), 2);
        assert_eq!(sound_type("player_hit_chicken"), 42);
        assert_eq!(sound_type("dst_male_hit_0"), 42);
    }

    /// W4-g22 — the female-hurt play (case 20, the menuMode 2 gender
    /// toggle's male->female preview `Main.cs:49045`): ONE `Next(3)` slot
    /// pick on the passed stream (`LegacySoundPlayer.cs:913`), NO pitch
    /// draw; the muted gate refuses before any draw. The int overload draws
    /// no GetRandomPitch. Written, not run (code-only mode).
    #[test]
    fn g22_the_female_hit_draws_one_next3_and_no_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(0, 3);
        let cue = audio.play_female_hit_sound(&mut rng);
        let want = match slot {
            0 => "Female_Hit_0",
            1 => "Female_Hit_1",
            _ => "Female_Hit_2",
        };
        assert_eq!(cue, Some(want), "the slot pick rides the stream");
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Muted: the gate precedes the switch — draw-free.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_female_hit_sound(&mut rng), None);
        assert_eq!(rng.peek(), peek, "the muted female hit is draw-free");
        // The bank rows resolve to the legacy id 20.
        for n in 0..3 {
            let name = format!("Female_Hit_{n}");
            assert_eq!(sound_type(&name), 20);
            assert!(SFX_CUES.iter().any(|(c, _)| *c == name), "{name} in the table");
        }
    }

    /// S1-E-21f — the double-jump and splash plays: case 16 draws one
    /// `Next(-10, 11)` pitch after the gate; case 19's playing guard is
    /// modelled by the splash slots (headless: always fresh), one pitch per
    /// play; an out-of-range play draws nothing.
    #[test]
    fn s1_e_21f_the_jump_and_splash_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Case 16: one Next(-10, 11) (`:851`).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            audio.play_double_jump_sound(0.0, 0.0, &mut rng),
            Some("Double_Jump")
        );
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Case 19: one pitch per style; the shimmer styles play their slot.
        for (style, cue, window) in [
            (0, "Splash_0", (-10, 11)),
            (1, "Splash_1", (-10, 11)),
            (2, "Splash_2", (-10, 11)),
            (3, "Splash_3", (-10, 11)),
            (4, "Splash_4", (-20, 1)),
            (5, "Splash_5", (-20, 1)),
        ] {
            let mut rng = UnifiedRandom::new(7);
            let mut witness = UnifiedRandom::new(7);
            let _ = witness.next_range(window.0, window.1);
            assert_eq!(
                audio.play_splash_sound(style, 0.0, 0.0, &mut rng),
                Some(cue),
                "style {style}"
            );
            assert_eq!(rng.peek(), witness.peek(), "style {style}: one draw");
        }
        // An unknown style plays nothing and draws nothing.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_splash_sound(6, 0.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
        // Out of attenuation range: draw-free (`:390-397`).
        let fns: [fn(&mut Audio, &mut UnifiedRandom) -> bool; 2] = [
            |a, r| a.play_double_jump_sound(5000.0, 0.0, r).is_none(),
            |a, r| a.play_splash_sound(0, 5000.0, 0.0, r).is_none(),
        ];
        for f in fns {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert!(f(&mut audio, &mut rng));
            assert_eq!(rng.peek(), peek, "the out-of-range play is draw-free");
        }
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Splash_5"));
    }

    /// S1-E-21f — the pickup chimes: case 7 draws ONE ±0.10 pitch; case 38
    /// draws TWO — `Next(5)` slot then `Next(-40, 41) * 0.002` pitch. The
    /// unpositioned chime ignores distance; the positioned coin ping does
    /// not. Muted plays draw nothing.
    #[test]
    fn s1_e_21f_the_pickup_chime_draws_ride_the_passed_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // Case 7 (Grab): one Next(-10, 11) (`:725`).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(audio.play_pickup_sound(false, &mut rng), Some("Grab"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Case 38 (coin): Next(5) then Next(-40, 41) (`:1343`/:1347).
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let slot = witness.next_range(0, 5);
        let _ = witness.next_range(-40, 41);
        let cue = audio.play_pickup_sound(true, &mut rng);
        assert_eq!(cue, Some(match slot {
            0 => "Coin_0",
            1 => "Coin_1",
            2 => "Coin_2",
            3 => "Coin_3",
            _ => "Coin_4",
        }));
        assert_eq!(rng.peek(), witness.peek(), "exactly two gameplay draws");
        // The positioned revenge ping: same two draws, distance-gated.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(0, 5);
        let _ = witness.next_range(-40, 41);
        assert!(audio.play_coin_ping(0.0, 0.0, &mut rng).is_some());
        assert_eq!(rng.peek(), witness.peek());
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_coin_ping(5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the out-of-range ping is draw-free");
        // Muted: both chimes draw-free (the unpositioned flag arm still
        // checks the volume slider, `:369-371`).
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        for coin in [false, true] {
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(audio.play_pickup_sound(coin, &mut rng), None);
            assert_eq!(rng.peek(), peek);
        }
    }

    /// S1-P-i43 — the chest eat-animation chime (`Chest.cs:3628-3631`,
    /// `PlaySound(7, chest centre)`): POSITIONED case 7 — one
    /// `Next(-10, 11)` pitch draw on the caller's bare stream when the
    /// attenuation passes, none when it refuses (the Lead's faithful-cue
    /// decision applied to the lid animator's dormant chime).
    #[test]
    fn s1_p_i43_the_chest_eat_chime_draws_one_case7_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        let mut audio = Audio::new(None);
        // In range (the listener sits at the origin): one draw.
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(audio.play_chest_eat_sound(16.0, 16.0, &mut rng), Some("Grab"));
        assert_eq!(rng.peek(), witness.peek(), "exactly one gameplay draw");
        // Out of range: the gate refuses, no draw.
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_chest_eat_sound(5000.0, 0.0, &mut rng), None);
        assert_eq!(rng.peek(), peek, "the out-of-range chime is draw-free");
        // Muted: draw-free too.
        let mut l = audio.listener;
        l.sound_volume = 0.0;
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(audio.play_chest_eat_sound(16.0, 16.0, &mut rng), None);
        assert_eq!(rng.peek(), peek);
    }
}

/// `Main.cs:11528-11546`: the rain track's weather factor — `Some` while the
/// rain zone is active and snow isn't.
fn ambience_rain_volume(env: &MusicEnv) -> Option<f32> {
    if !(env.zone_rain && !env.zone_snow) {
        return None;
    }
    let num10 = 0.5;
    let mut num11 = env.cloud_alpha / 9.0 * 10.0 * num10 + (1.0 - num10);
    if env.shimmer_alpha > 0.0 {
        num11 *= 1.0 - env.shimmer_alpha;
    }
    let v = (num11.min(1.0)) * env.atmo.powi(4);
    Some(v)
}

/// `Main.cs:11562-11595`: the wind track's weather factor — `Some` while
/// `|windSpeedCurrent| >= 20/50` and `SurfaceAtmospherics`.
fn ambience_wind_volume(env: &MusicEnv) -> Option<f32> {
    let num8 = 20.0f32;
    if env.wind_speed.abs() < num8 / 50.0 || !env.surface_atmospherics {
        return None;
    }
    let num6 = 0.7;
    let mut num7 = env.wind_speed.abs() * num6 + (1.0 - num6);
    if env.shimmer_alpha > 0.0 {
        num7 *= 1.0 - env.shimmer_alpha;
    }
    let v = (num7.min(1.0)) * 0.9 * env.atmo.powi(4);
    Some(v)

// balance-close (union shared delimiter)
})
}

// balance-close (union shared delimiter)
}}}}}}}}}))
}
