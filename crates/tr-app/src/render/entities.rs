//! Entity drawing: the player/NPC texture atlases and frame-UV helpers, the
//! world-space player and NPC passes, the held-item swing, the fullscreen-map
//! head icon and the UI-space character preview.
use super::BgTex;
use super::player_layers::{
    CompositeArmState, LayerSprite, PlayerCompositeState, assemble_layers, front_arm_cell,
    swing_stand_in,
};
use super::sky::sky_tex;
use crate::assets;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::registry;
use crate::ui;
use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::lighting::{LightMap, LightStyle};
use tr_render::{Atlas, SpriteRenderer};

/// Default-player texture parts (`PlayerTextureID`): head, eyes, torso skin, shirt,
/// legs, pants, shoes, hair. Frames are 40x56 on the 40-px-wide strips.
/// `Default` is hand-written: the two 50-entry mount arrays exceed the
/// `[T; N]: Default` bound for N > 32.
pub(crate) struct PlayerAssets {
    pub(crate) head: Option<BgTex>,
    pub(crate) eye_whites: Option<BgTex>,
    pub(crate) eyes: Option<BgTex>,
    pub(crate) torso: Option<BgTex>,
    pub(crate) shirt: Option<BgTex>,
    /// Back-arm skin frame from `Player_0_7` (composite arm frame 1).
    pub(crate) arm: Option<BgTex>,
    /// `Player_0_5`: the under-skin torso layer drawn behind the back arm
    /// (`TextureAssets.Players[skinVar, 5]`, `PlayerDrawLayers.cs:1345`).
    pub(crate) undershirt: Option<BgTex>,
    /// `Player_0_13`: the shirt sleeve over the back arm
    /// (`TextureAssets.Players[skinVar, 13]`, `PlayerDrawLayers.cs:1362`).
    pub(crate) arm_shirt: Option<BgTex>,
    pub(crate) legs: Option<BgTex>,
    pub(crate) pants: Option<BgTex>,
    pub(crate) shoes: Option<BgTex>,
    pub(crate) hair: Option<BgTex>,
    /// Armor sheets by vanilla armor slot (`TextureAssets.ArmorHead[n]` etc.,
    /// `AssetInitializer.cs:165-185`): index 0 is the empty slot, 1..4 the
    /// copper/iron/silver/gold sets. The body entry is the 9x4 composite
    /// `Armor/Armor_{n}` (torso + shoulders + arms, `PlayerDrawSet.cs:2241`).
    pub(crate) armor_head: [Option<BgTex>; 5],
    pub(crate) armor_body: [Option<BgTex>; 5],
    pub(crate) armor_legs: [Option<BgTex>; 5],
    /// The mount sheets, indexed by [`crate::mount::mount_index`]: the back
    /// strips (`MountData.backTexture`) and the front strips
    /// (`frontTexture`) of the ground + hover/flyer + cart sets.
    pub(crate) mount_back: [Option<BgTex>; 50],
    pub(crate) mount_front: [Option<BgTex>; 50],
}

impl Default for PlayerAssets {
    fn default() -> Self {
        Self {
            head: None,
            eye_whites: None,
            eyes: None,
            torso: None,
            shirt: None,
            arm: None,
            undershirt: None,
            arm_shirt: None,
            legs: None,
            pants: None,
            shoes: None,
            hair: None,
            armor_head: [None; 5],
            armor_body: [None; 5],
            armor_legs: [None; 5],
            mount_back: [None; 50],
            mount_front: [None; 50],
        }
    }
}

/// `Player.armor[0..3]`'s item kinds (0 = empty) — head, body, legs, the draw
/// input the layer builders resolve into armor slots via the registry.
pub(crate) type ArmorKinds = [u16; 3];

pub(crate) fn make_player_assets(atlas: &Atlas) -> PlayerAssets {
    // `AssetInitializer.cs:165-185`: the slot number is the file number.
    let armor_sheets = |prefix: &str| {
        std::array::from_fn(|slot| {
            if slot == 0 {
                None
            } else {
                sky_tex(atlas, &format!("{prefix}_{slot}"), None)
            }
        })
    };
    PlayerAssets {
        head: sky_tex(atlas, "Player_0_0", None),
        eye_whites: sky_tex(atlas, "Player_0_1", None),
        eyes: sky_tex(atlas, "Player_0_2", None),
        // Torso and shirt sheets are 9 columns wide; keep only the first column.
        torso: sky_tex(atlas, "Player_0_3", Some((0, 0, 40, 224))),
        shirt: sky_tex(atlas, "Player_0_6", Some((0, 0, 40, 224))),
        arm: sky_tex(atlas, "Player_0_7", None),
        undershirt: sky_tex(atlas, "Player_0_5", None),
        arm_shirt: sky_tex(atlas, "Player_0_13", None),
        legs: sky_tex(atlas, "Player_0_10", None),
        pants: sky_tex(atlas, "Player_0_11", None),
        shoes: sky_tex(atlas, "Player_0_12", None),
        hair: sky_tex(atlas, "Player_Hair_1", None),
        armor_head: armor_sheets("Armor_Head"),
        armor_body: {
            let mut body: [Option<BgTex>; 5] = [None; 5];
            for (slot, tex) in body.iter_mut().enumerate().skip(1) {
                *tex = sky_tex(atlas, &format!("Armor/Armor_{slot}"), None);
            }
            body
        },
        armor_legs: armor_sheets("Armor_Legs"),
        // The mount strips resolve by the rows' own sheet names (`Mount.cs:
        // 696-2567`); a missing file stays `None` and the draw skips it.
        mount_back: std::array::from_fn(|i| {
            crate::mount::MOUNTS.get(i).and_then(|row| {
                if row.back_sheet.is_empty() {
                    None
                } else {
                    sky_tex(atlas, row.back_sheet, None)
                }
            })
        }),
        mount_front: std::array::from_fn(|i| {
            crate::mount::MOUNTS.get(i).and_then(|row| {
                if row.front_sheet.is_empty() {
                    None
                } else {
                    sky_tex(atlas, row.front_sheet, None)
                }
            })
        }),
    }
}

/// UV of one 40x56 frame in a player part strip — tabled in
/// [`player_layers`] (single source; re-exported for the sibling passes).
pub(crate) use super::player_layers::{flip_uv, player_cell_uv, player_frame_uv};

/// Default-derivable: every sheet is an `Option` guard (the headless flow
/// tests construct the empty set).
#[derive(Default)]
pub(crate) struct NpcAssets {
    pub(crate) slime: Option<BgTex>,
    pub(crate) eye: Option<BgTex>,
    pub(crate) wandering_eye: Option<BgTex>,
    pub(crate) zombie: Option<BgTex>,
    pub(crate) guide: Option<BgTex>,
    pub(crate) merchant: Option<BgTex>,
    pub(crate) dryad: Option<BgTex>,
    pub(crate) nurse: Option<BgTex>,
    pub(crate) arms_dealer: Option<BgTex>,
    pub(crate) demolitionist: Option<BgTex>,
    pub(crate) painter: Option<BgTex>,
    pub(crate) witch_doctor: Option<BgTex>,
    pub(crate) clothier: Option<BgTex>,
    /// `NPC_368.xnb` — the Travelling Merchant sheet.
    pub(crate) travelling_merchant: Option<BgTex>,
    /// `NPC_369.xnb` — the Angler's town sheet (23 rows, `Main.cs:67120`).
    pub(crate) angler: Option<BgTex>,
    /// `NPC_376.xnb` — the Sleeping Angler's 2-row sheet (`Main.cs:67121`).
    pub(crate) sleeping_angler: Option<BgTex>,
    pub(crate) eoc: Option<BgTex>,
    pub(crate) queen_bee: Option<BgTex>,
    pub(crate) skeletron_head: Option<BgTex>,
    pub(crate) wof: Option<BgTex>,
    pub(crate) destroyer: Option<BgTex>,
    pub(crate) prime: Option<BgTex>,
    pub(crate) retinazer: Option<BgTex>,
    pub(crate) spazmatism: Option<BgTex>,
    pub(crate) plantera: Option<BgTex>,
    pub(crate) brain: Option<BgTex>,
    pub(crate) creeper: Option<BgTex>,
    pub(crate) bound_goblin: Option<BgTex>,
    pub(crate) goblin_tinkerer: Option<BgTex>,
    pub(crate) golem: Option<BgTex>,
    pub(crate) golem_head: Option<BgTex>,
    pub(crate) golem_fist: Option<BgTex>,
    pub(crate) golem_head_free: Option<BgTex>,
    pub(crate) deerclops: Option<BgTex>,
    pub(crate) skeletron_hand: Option<BgTex>,
    pub(crate) wof_eye: Option<BgTex>,
    pub(crate) prime_cannon: Option<BgTex>,
    pub(crate) prime_saw: Option<BgTex>,
    pub(crate) prime_vice: Option<BgTex>,
    pub(crate) prime_laser: Option<BgTex>,
    /// `Images/Bone_Eyes` — the Prime head's glow mask (`Main.cs:25634`,
    /// 140x470: its rows ride the head's 156-px grid) and
    /// `Images/Bone_Laser` — the laser limb's glow (`Main.cs:25641`,
    /// 40x62). The vanilla draws both over the body at the body's own
    /// frame/rotation/anchor with the (200,200,200,0) mask tint — the
    /// port's premultiplied pipeline renders the overlays with the
    /// sheets' own alpha instead (an alpha-0 tint draws nothing there).
    pub(crate) prime_eyes: Option<BgTex>,
    pub(crate) bone_laser: Option<BgTex>,
    pub(crate) skeleton: Option<BgTex>,
    pub(crate) undead_miner: Option<BgTex>,
    pub(crate) cave_bat: Option<BgTex>,
    pub(crate) jungle_bat: Option<BgTex>,
    pub(crate) hellbat: Option<BgTex>,
    pub(crate) voodoo_demon: Option<BgTex>,
    pub(crate) eater_of_souls: Option<BgTex>,
    pub(crate) hornet: Option<BgTex>,
    pub(crate) meteor_head: Option<BgTex>,
    pub(crate) goblin_sorcerer: Option<BgTex>,
    /// `NPC_143/144/145` — the Frost Legion strips (4/5/7 frames of 26x40).
    pub(crate) snowman_gangsta: Option<BgTex>,
    pub(crate) snow_balla: Option<BgTex>,
    pub(crate) mystery_snowman: Option<BgTex>,
    /// The martian ground group — one sheet PER KIND, never shared:
    /// `NPC_383` the Officer (9 rows of 46), `NPC_385` the Gray Grunt (9
    /// of 44), `NPC_386` the Engineer (11 of 46 — the walk wraps at 9 and
    /// rows 9/10 are the `ai[2]` animation pair), `NPC_389` the Gigazapper
    /// (9 of 46) (`Main.npcFrameCount`, `Main.cs:67082`). The v1 drew all
    /// four off `NPC_383`, so the Grunt's 44-px rows read a 46-px slice
    /// and the Engineer's rows 9/10 never existed.
    pub(crate) martian_officer: Option<BgTex>,
    pub(crate) gray_grunt: Option<BgTex>,
    pub(crate) martian_engineer: Option<BgTex>,
    pub(crate) gigazapper: Option<BgTex>,
    /// `NPC_520` — the Martian Walker (8 frames of 34x110,
    /// `Main.npcFrameCount[520]`), `NPC_399` — the Martian Probe (8 frames
    /// of 42x72), `NPC_388` — the Martian Drone (2 frames of 40x30).
    pub(crate) martian_walker_big: Option<BgTex>,
    pub(crate) martian_probe: Option<BgTex>,
    pub(crate) martian_drone: Option<BgTex>,
    /// `NPC_381` the Brain Scrambler / `NPC_382` the Ray Gunner (9 rows of
    /// 48 / 46) — the two shooters ride the same FindFrame case as the
    /// walkers but each keeps its own sheet.
    pub(crate) brain_scrambler: Option<BgTex>,
    pub(crate) ray_gunner: Option<BgTex>,
    /// `NPC_396` — the Moon Lord's eye (8 rows of 38x56), `NPC_397` —
    /// the hand (4 rows of 46x66), `NPC_398` — the core (5 rows),
    /// `NPC_400` — the True Eye (4 rows of 60x60), `NPC_401` — the leech
    /// clot (1 frame of 30x30).
    /// `NPC_426` — the VortexHunter (15 rows of 50x62), `NPC_427` —
    /// the VortexAlien (9 rows of 40x28), `NPC_428` — the
    /// VortexHeadcrab (5 rows of 24x26).
    /// `NPC_420` — the NebulaFloater (6 rows of 40x40), `NPC_421` — the
    /// BrainSuckler (6 rows of 44x44), `NPC_424` — the Predictor (12
    /// rows of 22x56).
    /// `NPC_409` — the StardustSpiderBig (4 rows), `NPC_410` — the
    /// StardustSpiderSmall (11 rows), `NPC_411` — the StardustSoldier
    /// (5 rows), `NPC_415` — the SolarDrakomire (1 row), `NPC_416` —
    /// the rider (10 rows), `NPC_417` — the SolarSroller (10 rows),
    /// `NPC_418` — the SolarCorite (15 rows), `NPC_419` — the
    /// SolarSelenian (8 rows).
    pub(crate) tower_mobs: Option<BgTex>,
    pub(crate) nebula_trio: Option<BgTex>,
    pub(crate) vortex_trio: Option<BgTex>,
    pub(crate) moon_lord_parts: Option<BgTex>,
    /// `NPC_390` — the Scutlix ray rider (5 frames of 18x40), `NPC_391` —
    /// the Scutlix carrier (6 frames of 60x40).
    pub(crate) scutlix_rider: Option<BgTex>,
    pub(crate) scutlix: Option<BgTex>,
    /// `NPC_392/393/394/395` — the saucer family (the shell 150x80 4
    /// frames, the turret 40x16 18, the cannon 46x36 8, the core 120x90
    /// 11).
    pub(crate) saucer: Option<BgTex>,
    pub(crate) saucer_turret: Option<BgTex>,
    pub(crate) saucer_cannon: Option<BgTex>,
    pub(crate) saucer_core: Option<BgTex>,
    /// `NPC_491` — the Flying Dutchman (1 frame of 350x120), `NPC_492` —
    /// its cannons (9 frames of 30x30).
    pub(crate) dutchman: Option<BgTex>,
    pub(crate) dutchman_cannon: Option<BgTex>,
    /// `NPC_552/553/554` — the DD2 goblin attackers (14-frame strips of
    /// 18x40; the sheets are the shared humanoid shape).
    pub(crate) dd2_goblins: Option<BgTex>,
    /// `NPC_555-567` — the DD2 support family (the bombers/javelineers'
    /// humanoid strips + the skeletons' 14-frame materialize sheets).
    pub(crate) dd2_support: Option<BgTex>,
    /// `NPC_564` — the Dark Mage (20-frame caster strip of 34x62).
    pub(crate) dark_mage: Option<BgTex>,
    /// `NPC_551` — Betsy (11-frame dragon strip of 190x90).
    pub(crate) betsy: Option<BgTex>,
    /// `NPC_568` — the Wither Beast (14-frame humanoid strip of 18x40).
    pub(crate) wither_beast: Option<BgTex>,
    /// `NPC_550` — the tavernkeeper (16-frame humanoid strip of 18x40),
    /// `NPC_548` — the Etheria Crystal (40x40).
    pub(crate) tavern: Option<BgTex>,
    pub(crate) crystal: Option<BgTex>,
    /// `NPC_422` — the Ancient Cultist (1-frame strip of 130x270).
    pub(crate) cultist: Option<BgTex>,
    /// `NPC_493/507/517/422` — the four Lunar pillars (single-frame
    /// 174x364 sheets — the frame overhangs the 130x270 box on every
    /// side, so the draw is Centre-anchored). The v1 shared one field
    /// (`towers`, NPC_493) across all three kinds, so the Nebula and
    /// Solar pillars rendered the Stardust artwork, and the Vortex pillar
    /// had no kind at all.
    pub(crate) tower_stardust: Option<BgTex>,
    pub(crate) tower_nebula: Option<BgTex>,
    pub(crate) tower_solar: Option<BgTex>,
    pub(crate) tower_vortex: Option<BgTex>,
    /// `Images/Glow_132/143/149/162` — the per-pillar glow masks
    /// (`Main.cs:24159-24275`'s dedicated tower arm draws four ghost
    /// copies of each at the velocity-rotated offsets).
    pub(crate) glow_stardust: Option<BgTex>,
    pub(crate) glow_nebula: Option<BgTex>,
    pub(crate) glow_vortex: Option<BgTex>,
    pub(crate) glow_solar: Option<BgTex>,
    /// `NPC_406/407/423/425/429` — the pillar shards.
    pub(crate) shards: Option<BgTex>,
    pub(crate) shard_hovers: Option<BgTex>,
    /// `NPC_212-216` — the pirate strips (16/20 frames of 18x40).
    pub(crate) pirates: Option<BgTex>,
    pub(crate) pirate_captain: Option<BgTex>,
    /// `NPC_26/27/28/111` — the invasion fighters (16/21-frame humanoid
    /// strips; the port renders the 16-frame family shape the zombie arm
    /// walks).
    pub(crate) goblin_thief: Option<BgTex>,
    pub(crate) goblin_warrior: Option<BgTex>,
    pub(crate) goblin_archer: Option<BgTex>,
    pub(crate) goblin_peon: Option<BgTex>,
    /// `NPC_471` — the Goblin Summoner (20 frames).
    pub(crate) goblin_summoner: Option<BgTex>,
    pub(crate) tim: Option<BgTex>,
    pub(crate) dark_caster: Option<BgTex>,
    pub(crate) fire_imp: Option<BgTex>,
    pub(crate) chaos_ball: Option<BgTex>,
    pub(crate) burning_sphere: Option<BgTex>,
    pub(crate) water_sphere: Option<BgTex>,
    pub(crate) goldfish: Option<BgTex>,
    pub(crate) piranha: Option<BgTex>,
    pub(crate) shark: Option<BgTex>,
    pub(crate) devourer_head: Option<BgTex>,
    pub(crate) devourer_body: Option<BgTex>,
    pub(crate) devourer_tail: Option<BgTex>,
    pub(crate) giant_worm_head: Option<BgTex>,
    pub(crate) giant_worm_body: Option<BgTex>,
    pub(crate) giant_worm_tail: Option<BgTex>,
    pub(crate) eow_head: Option<BgTex>,
    pub(crate) eow_body: Option<BgTex>,
    pub(crate) eow_tail: Option<BgTex>,
    pub(crate) king_slime: Option<BgTex>,
    /// `NPC_4.xnb` — the Eye of Cthulhu sheet (6 frames of 100×110).
    pub(crate) eye_of_cthulhu: Option<BgTex>,
    /// `NPC_5.xnb` — the Servant of Cthulhu sheet (2 frames of 20×20).
    pub(crate) servant_of_cthulhu: Option<BgTex>,
    /// `NPC_115.xnb` — the Hungry sheet (3 frames).
    pub(crate) hungry: Option<BgTex>,
    /// `NPC_117.xnb`/`NPC_118.xnb` — the Leech segments (1 frame).
    pub(crate) leech: Option<BgTex>,
}

pub(crate) fn make_npc_assets(atlas: &Atlas) -> NpcAssets {
    NpcAssets {
        slime: sky_tex(atlas, "NPC_1", None),
        eye: sky_tex(atlas, "NPC_2", None),
        wandering_eye: sky_tex(atlas, "NPC_133", None),
        zombie: sky_tex(atlas, "NPC_3", None),
        guide: sky_tex(atlas, "NPC_22", None),
        merchant: sky_tex(atlas, "NPC_17", None),
        dryad: sky_tex(atlas, "NPC_20", None),
        nurse: sky_tex(atlas, "NPC_18", None),
        arms_dealer: sky_tex(atlas, "NPC_19", None),
        demolitionist: sky_tex(atlas, "NPC_38", None),
        painter: sky_tex(atlas, "NPC_227", None),
        witch_doctor: sky_tex(atlas, "NPC_228", None),
        clothier: sky_tex(atlas, "NPC_54", None),
        travelling_merchant: sky_tex(atlas, "NPC_368", None),
        angler: sky_tex(atlas, "NPC_369", None),
        sleeping_angler: sky_tex(atlas, "NPC_376", None),
        eoc: sky_tex(atlas, "NPC_4", None),
        queen_bee: sky_tex(atlas, "NPC_222", None),
        skeletron_head: sky_tex(atlas, "NPC_35", None),
        wof: sky_tex(atlas, "NPC_113", None),
        destroyer: sky_tex(atlas, "NPC_134", None),
        prime: sky_tex(atlas, "NPC_127", None),
        retinazer: sky_tex(atlas, "NPC_125", None),
        spazmatism: sky_tex(atlas, "NPC_126", None),
        plantera: sky_tex(atlas, "NPC_262", None),
        brain: sky_tex(atlas, "NPC_266", None),
        creeper: sky_tex(atlas, "NPC_267", None),
        bound_goblin: sky_tex(atlas, "NPC_105", None),
        goblin_tinkerer: sky_tex(atlas, "NPC_107", None),
        golem: sky_tex(atlas, "NPC_245", None),
        golem_head: sky_tex(atlas, "NPC_246", None),
        golem_fist: sky_tex(atlas, "NPC_247", None),
        golem_head_free: sky_tex(atlas, "NPC_249", None),
        deerclops: sky_tex(atlas, "NPC_668", None),
        skeletron_hand: sky_tex(atlas, "NPC_36", None),
        wof_eye: sky_tex(atlas, "NPC_114", None),
        prime_cannon: sky_tex(atlas, "NPC_128", None),
        prime_saw: sky_tex(atlas, "NPC_129", None),
        prime_vice: sky_tex(atlas, "NPC_130", None),
        prime_laser: sky_tex(atlas, "NPC_131", None),
        tower_mobs: sky_tex(atlas, "NPC_409", None),
        nebula_trio: sky_tex(atlas, "NPC_420", None),
        vortex_trio: sky_tex(atlas, "NPC_426", None),
        moon_lord_parts: sky_tex(atlas, "NPC_396", None),
        prime_eyes: sky_tex(atlas, "Bone_Eyes", None),
        bone_laser: sky_tex(atlas, "Bone_Laser", None),
        skeleton: sky_tex(atlas, "NPC_21", None),
        undead_miner: sky_tex(atlas, "NPC_44", None),
        cave_bat: sky_tex(atlas, "NPC_49", None),
        jungle_bat: sky_tex(atlas, "NPC_51", None),
        hellbat: sky_tex(atlas, "NPC_60", None),
        voodoo_demon: sky_tex(atlas, "NPC_66", None),
        eater_of_souls: sky_tex(atlas, "NPC_6", None),
        hornet: sky_tex(atlas, "NPC_42", None),
        meteor_head: sky_tex(atlas, "NPC_23", None),
        goblin_sorcerer: sky_tex(atlas, "NPC_29", None),
        snowman_gangsta: sky_tex(atlas, "NPC_143", None),
        snow_balla: sky_tex(atlas, "NPC_144", None),
        mystery_snowman: sky_tex(atlas, "NPC_145", None),
        martian_officer: sky_tex(atlas, "NPC_383", None),
        gray_grunt: sky_tex(atlas, "NPC_385", None),
        martian_engineer: sky_tex(atlas, "NPC_386", None),
        gigazapper: sky_tex(atlas, "NPC_389", None),
        martian_walker_big: sky_tex(atlas, "NPC_520", None),
        martian_probe: sky_tex(atlas, "NPC_399", None),
        martian_drone: sky_tex(atlas, "NPC_388", None),
        brain_scrambler: sky_tex(atlas, "NPC_381", None),
        ray_gunner: sky_tex(atlas, "NPC_382", None),
        scutlix_rider: sky_tex(atlas, "NPC_390", None),
        scutlix: sky_tex(atlas, "NPC_391", None),
        saucer: sky_tex(atlas, "NPC_392", None),
        saucer_turret: sky_tex(atlas, "NPC_393", None),
        saucer_cannon: sky_tex(atlas, "NPC_394", None),
        saucer_core: sky_tex(atlas, "NPC_395", None),
        dutchman: sky_tex(atlas, "NPC_491", None),
        dutchman_cannon: sky_tex(atlas, "NPC_492", None),
        dd2_goblins: sky_tex(atlas, "NPC_553", None),
        dd2_support: sky_tex(atlas, "NPC_556", None),
        dark_mage: sky_tex(atlas, "NPC_564", None),
        betsy: sky_tex(atlas, "NPC_551", None),
        wither_beast: sky_tex(atlas, "NPC_568", None),
        tavern: sky_tex(atlas, "NPC_550", None),
        crystal: sky_tex(atlas, "NPC_548", None),
        cultist: sky_tex(atlas, "NPC_422", None),
        tower_stardust: sky_tex(atlas, "NPC_493", None),
        tower_nebula: sky_tex(atlas, "NPC_507", None),
        tower_solar: sky_tex(atlas, "NPC_517", None),
        tower_vortex: sky_tex(atlas, "NPC_422", None),
        glow_stardust: sky_tex(atlas, "Glow_132", None),
        glow_nebula: sky_tex(atlas, "Glow_143", None),
        glow_vortex: sky_tex(atlas, "Glow_149", None),
        glow_solar: sky_tex(atlas, "Glow_162", None),
        shards: sky_tex(atlas, "NPC_406", None),
        shard_hovers: sky_tex(atlas, "NPC_425", None),
        pirates: sky_tex(atlas, "NPC_212", None),
        pirate_captain: sky_tex(atlas, "NPC_216", None),
        goblin_thief: sky_tex(atlas, "NPC_27", None),
        goblin_warrior: sky_tex(atlas, "NPC_28", None),
        goblin_archer: sky_tex(atlas, "NPC_111", None),
        goblin_peon: sky_tex(atlas, "NPC_26", None),
        goblin_summoner: sky_tex(atlas, "NPC_471", None),
        tim: sky_tex(atlas, "NPC_45", None),
        dark_caster: sky_tex(atlas, "NPC_32", None),
        fire_imp: sky_tex(atlas, "NPC_24", None),
        chaos_ball: sky_tex(atlas, "NPC_30", None),
        burning_sphere: sky_tex(atlas, "NPC_25", None),
        water_sphere: sky_tex(atlas, "NPC_33", None),
        goldfish: sky_tex(atlas, "NPC_55", None),
        piranha: sky_tex(atlas, "NPC_58", None),
        shark: sky_tex(atlas, "NPC_65", None),
        devourer_head: sky_tex(atlas, "NPC_7", None),
        devourer_body: sky_tex(atlas, "NPC_8", None),
        devourer_tail: sky_tex(atlas, "NPC_9", None),
        giant_worm_head: sky_tex(atlas, "NPC_10", None),
        giant_worm_body: sky_tex(atlas, "NPC_11", None),
        giant_worm_tail: sky_tex(atlas, "NPC_12", None),
        eow_head: sky_tex(atlas, "NPC_13", None),
        eow_body: sky_tex(atlas, "NPC_14", None),
        eow_tail: sky_tex(atlas, "NPC_15", None),
        king_slime: sky_tex(atlas, "NPC_50", None),
        eye_of_cthulhu: sky_tex(atlas, "NPC_4", None),
        servant_of_cthulhu: sky_tex(atlas, "NPC_5", None),
        hungry: sky_tex(atlas, "NPC_115", None),
        leech: sky_tex(atlas, "NPC_117", None),
    }
}

/// UV of one slime / demon-eye frame (two frames stacked vertically).
fn npc_frame_uv(tex: &BgTex, frame_h: f32, frame: u32) -> [f32; 4] {
    let dv = (tex.uv[3] - tex.uv[1]) * (frame_h / tex.h);
    let r = frame as f32;
    [
        tex.uv[0],
        tex.uv[1] + dv * r,
        tex.uv[2],
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// The Deerclops' 5x5 grid cell (`DrawNPCDirect_Deerclops`'s
/// `value.Frame(5, 5, frame.Y / 5, frame.Y % 5, 2, 2)`, `Main.cs:26338`):
/// NPC_668's 1090x1200 sheet is 5x5 cells of 218x240 drawn 220x242 (the
/// +2/+2 size offsets), the cell index in `npc.frame` (0..24).
fn deerclops_frame_uv(tex: &BgTex, cell: u32) -> [f32; 4] {
    let span = (tex.uv[2] - tex.uv[0], tex.uv[3] - tex.uv[1]);
    let (du, dv) = (span.0 / 5.0, span.1 / 5.0);
    let (ox, oy) = (span.0 * (2.0 / tex.w), span.1 * (2.0 / tex.h));
    let (col, row) = (cell / 5, cell % 5);
    let (c, r) = (col as f32, row as f32);
    [
        tex.uv[0] + du * c,
        tex.uv[1] + dv * r,
        tex.uv[0] + du * (c + 1.0) + ox,
        tex.uv[1] + dv * (r + 1.0) + oy,
    ]
}

/// The humanoid walkers' draw: the sheet's real row (`t.h / count`, the
/// per-kind `Main.npcFrameCount` rows — 56 for most humanoids, the Tim's
/// 70) at the generic anchor — the frame centred on the box's centre-X,
/// its centre 4 px below the box's bottom (`Main.cs:25511-25516`). The
/// v1 drew every humanoid as a fixed 38x48, stretching the art.
fn humanoid_draw<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(48.0, |t| t.h / count as f32);
    let fw = tex.map_or(38.0, |t| t.w);
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
    )
}

/// The martian walkers'/shooters' draw: the sheet's real row
/// (`Main.npcFrameCount`'s per-kind count — the Officer's 46, the Grunt's
/// 44, the Engineer's 46-over-11, the shooters' 48/46) at the generic
/// anchor (`Main.cs:25511-25516`) — the v1 stretched every martian to its
/// 18x40 box, so a 44-px Grunt row drew a 46-px slice of `NPC_383`'s art.
fn martian_draw<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(40.0, |t| t.h / count as f32);
    let fw = tex.map_or(18.0, |t| t.w);
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
    )
}

/// The saucer family's draw (`Main.DrawNPCDirect`'s `case 392/393/394/395`,
/// `Main.cs:24586-24612`): unlike the generic NPCs the saucers draw at the
/// CENTRE with `origin = halfSize` (rotated by the core's inherited
/// `npc.rotation`, so the whole family tilts as one body) and the turret's
/// origin rides `halfSize + UnitY * -8` (its quad sits 8 px lower). The v1
/// drew all four as unrotated box-size quads at the hitbox corner.
fn saucer_draw<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(90.0, |t| t.h / count as f32);
    let fw = tex.map_or(120.0, |t| t.w);
    let lift = if n.kind == NpcKind::MartianSaucerTurret {
        8.0
    } else {
        0.0
    };
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5 + lift],
    )
}

/// The Skeletron Prime family's draw: the sheet's real row
/// (`Main.npcFrameCount`'s per-kind count — the head's 156-px rows over 6,
/// the saw/vice's 78/70 over 2, the cannon's 76 and the laser's 62 over 1)
/// at the generic anchor (`Main.cs:25511-25516`) — the v1 read `t.h` as
/// the frame height (the whole strip as one frame) and pinned the box's
/// corner.
fn prime_draw<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(52.0, |t| t.h / count as f32);
    let fw = tex.map_or(52.0, |t| t.w);
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
    )
}

/// The Lunar pillars' draw (`Main.cs:24159-24263`): the single-frame
/// 174x364 sheet centred on the NPC's Centre (the `origin = halfSize`
/// form — unlike the generic bottom anchor, which would sink the frame's
/// 47-px overhang into the ground). `Main.npcFrameCount` 422/493/507/517
/// = 1, so the row height is the whole sheet.
fn tower_draw<'a>(
    tex: Option<&'a BgTex>,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(364.0, |t| t.h);
    let fw = tex.map_or(174.0, |t| t.w);
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
    )
}

/// The halfSize override for a kind (`Main.cs:23049`'s switch): the
/// point that lands on the NPC's anchor, so a value below the frame's
/// centre lifts the art and one above pushes it down. The C# overrides
/// `halfSize = (55, 107)` for the Eye of Cthulhu and the Twins — the
/// sheet-derived half-frame for their 166-px rows is 83 (and 100 for the
/// Twins' 200-px rows), so the override shifts the frame down by the
/// difference. `None` = the sheet-derived value.
fn boss_half_size(kind: NpcKind, frame_h: f32) -> f32 {
    match kind {
        NpcKind::EyeOfCthulhu | NpcKind::Retinazer | NpcKind::Spazmatism => 107.0,
        NpcKind::Plantera => 77.0,
        _ => frame_h * 0.5,
    }
}

/// `Main.cs:23048`'s per-kind vertical anchor nudge (`num47`, plus the
/// `num46` terms the same switch writes): a flat pixel shift added to the
/// generic anchor's Y after the `halfSize` override's own shift.
fn npc_anchor_nudge(n: &Npc) -> f32 {
    match n.kind {
        // `:23077` — the Plantera head's 26 (`halfSize.Y = 77` is folded
        // by `boss_half_size`).
        NpcKind::Plantera => 26.0,
        // `:23078` — the Brain of Cthulhu's `num47 += 50 * scale`.
        NpcKind::BrainOfCthulhu => 50.0 * n.scale,
        // `:23103`'s group — the DD2 goblins' and the Wither Beasts'
        // negative nudges (561-563: -8; 555-557: -5; 572/573: -4;
        // 570/571: -2; 568/569: -3).
        NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => -8.0,
        NpcKind::DD2GoblinBomberT1 | NpcKind::DD2GoblinBomberT2 | NpcKind::DD2GoblinBomberT3 => {
            -5.0
        }
        NpcKind::DD2WitherBeast => -3.0,
        _ => 0.0,
    }
}

/// The boss draws (the Eye of Cthulhu and the Twins — the `case 4/125/126`
/// family): the sheet's real rows at the generic anchor with the C#'s
/// `halfSize` override applied as the delta it produces.
fn boss_draw<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(166.0, |t| t.h / count as f32);
    let fw = tex.map_or(110.0, |t| t.w);
    // The C#'s anchor lands the frame's centre at `box_bottom + 4` and the
    // `halfSize` override shifts it by `override − frame_h / 2`.
    let shift = boss_half_size(n.kind, fh) - fh * 0.5;
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5 + shift],
    )
}

/// Snap a sprite's top-left to an integer screen pixel so nearest sampling keeps
/// the same source texels while the camera moves (no crawling pixel grid).
#[inline]
fn snap_to_screen(pos: [f32; 2], camera: Vec2, zoom: f32, screen: Vec2) -> [f32; 2] {
    let sx = (pos[0] - camera.x) * zoom + screen.x * 0.5;
    let sy = (pos[1] - camera.y) * zoom + screen.y * 0.5;
    [
        camera.x + (sx.round() - screen.x * 0.5) / zoom,
        camera.y + (sy.round() - screen.y * 0.5) / zoom,
    ]
}

/// The held item's atlas quad and frame size — the `GetItemDrawFrame` stand-in:
/// the hotbar icon quad and its atlas dimensions while `Item.cs` textures stay
/// unframed (`ItemSlot.DrawItemIcon`'s clamp applies to the HUD slot, not the
/// world sprite).
fn held_item_quad(lookups: &assets::Lookups, kind: u16) -> Option<([f32; 4], [f32; 2])> {
    if let Some((uv, w, h)) = lookups.item_icon(kind) {
        return Some((uv, [w, h]));
    }
    if registry::is_weapon(kind) {
        return lookups.shuriken_uv().map(|uv| (uv, [16.0, 16.0]));
    }
    if let Some(wall) = registry::place_wall(kind) {
        return lookups.wall_icon_uv(wall).map(|uv| (uv, [16.0, 16.0]));
    }
    registry::place_tile(kind)
        .and_then(|t| lookups.plain_tex(t))
        .map(|f| (f.uv, [16.0, 16.0]))
}

/// Pushes a builder's sprite list. Unrotated sprites snap their top-left to an
/// integer screen pixel (the entity-snapping invariant); the rotated held item
/// keeps its sub-pixel pivot like the old arc pass. Returns the quad count.
pub(crate) fn push_layers(
    r: &mut SpriteRenderer,
    layers: &[LayerSprite],
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let mut quads = 0;
    for s in layers {
        if s.dye != 0 {
            // `DrawData.shader` ≠ 0 — the layer draws through its dye pass
            // (`PlayerDrawHelper.SetShaderForData`, `PlayerDrawHelper.cs:29-60`).
            r.push_sprite_dye(s.pos, s.size, s.uv, s.color, s.rotation, s.dye);
        } else if s.rotation != 0.0 {
            r.push_sprite_rot(s.pos, s.size, s.uv, s.color, s.rotation);
        } else {
            r.push_sprite(
                snap_to_screen(s.pos, camera, zoom, screen),
                s.size,
                s.uv,
                s.color,
            );
        }
        quads += 1;
    }
    quads
}

/// The starter player through the pure layer builders (`assemble_layers`):
/// `LegacyPlayerRenderer`'s layer order with the held item in its
/// `weaponDrawOrder` slot (`PlayerDrawSet.cs:358`). `held` carries the selected
/// item kind and the swing timer (`None` = empty hand — remote players).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_player(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    lookups: &assets::Lookups,
    player: &Player,
    held: Option<(u16, f32)>,
    // `Player.armor[0..3]` item kinds (head/body/legs, 0 = empty); the layer
    // builders swap the shirt/pants/hair pieces for the armor sheets.
    armor: ArmorKinds,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let (cx, cy) = player.center();
    let (tx, ty) = ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
    let light_c = style.shade(light.get(tx, ty), tx, ty, anim_time);
    let swing = held.map_or(0.0, |(_, s)| s);
    // `Player.armor[0..3]`'s slot ids (`head`/`body`/`legs`, `Player.cs`): the
    // registry item → slot resolution the C# `Item.equipSlot` fields do at
    // `SetDefaults`. The generated slots are i16 with −1 = none
    // (`Item.cs:48998-49000`); slot 0 is REAL (the Familiar set, ids 269-271).
    let [head_kind, body_kind, legs_kind] = armor;
    let head_armor = registry::get(head_kind).map_or(-1, |d| d.head_slot);
    let body_armor = registry::get(body_kind).map_or(-1, |d| d.body_slot);
    let leg_armor = registry::get(legs_kind).map_or(-1, |d| d.leg_slot);
    // `PlayerDrawSet.cs:318-347`: the seated/sleeping draw inputs — the
    // seat posOffset (`GetSittingOffsetInfo`, stack index 0 in SP) with the
    // torso drop, and the sleep base offset (`GetSleepingOffsetInfo`'s
    // `SetOffsetbyBed` value mirrored to the facing).
    let (seat_pos_offset, seat_y_offset) = if player.sitting {
        (
            [player.seat_pos_offset.0, player.seat_pos_offset.1],
            player.seat_y_offset,
        )
    } else {
        ([0.0, 0.0], 0)
    };
    let (sleep_pos_offset, full_rotation) = if player.sleeping {
        let off = super::player_layers::bed_visual_offset(player.sleeping_style);
        (
            [off[0] * player.facing as f32, off[1]],
            player.full_rotation,
        )
    } else {
        ([0.0, 0.0], 0.0)
    };
    let state = PlayerCompositeState {
        origin: [
            player.x - 10.0 + seat_pos_offset[0] + sleep_pos_offset[0],
            player.y - 10.0 + seat_pos_offset[1] + sleep_pos_offset[1],
        ],
        leg_frame: player.leg_frame,
        body_frame: player.body_frame,
        facing: player.facing as f32,
        // `LegacyPlayerRenderer` receives the player at
        // `position + (0, gfxOffY)` (`LegacyPlayerRenderer.cs:457,468`).
        gfx_off_y: player.gfx_off_y,
        swing,
        hold_style: held.map_or(0, |(kind, _)| crate::item::hold_style(kind)),
        // The held item's real useStyle (`Item.useStyle` — the per-style
        // body rows and the held-item draw both read it).
        use_style: held.map_or(1, |(kind, _)| crate::item::use_style(kind)),
        item_uv: held
            .and_then(|(kind, _)| held_item_quad(lookups, kind))
            .map(|(uv, _)| uv),
        aim: player.swing_aim,
        item_size: held
            .and_then(|(kind, _)| held_item_quad(lookups, kind))
            .map(|(_, size)| size)
            .unwrap_or([0.0, 0.0]),
        light: light_c,
        head_armor,
        body_armor,
        leg_armor,
        front_arm: swing_stand_in(
            held.map_or(1, |(kind, _)| crate::item::use_style(kind)),
            swing,
            player.facing as f32,
        ),
        back_arm: CompositeArmState::DISABLED,
        // `DrawPlayer_02_MountBehindPlayer`/`DrawPlayer_23_MountFront`
        // (`PlayerDrawLayers.cs:211`, `:2912`) — the live mount's inputs.
        mount: if player.mount.is_active() {
            player
                .mount
                .data()
                .map(|d| super::player_layers::MountDraw {
                    id: d.id,
                    frame: player.mount.frame(),
                    player_offset: player.mount.player_offset(),
                    is_idle: player.mount.is_at_idle(),
                })
        } else {
            None
        },
        // `UpdateDyes`' cache (`Player.cs:9587-9614`): the per-layer dye the
        // builders stamp onto the armor/mount pieces (`apply_dye`).
        c_head: player.c_head,
        c_body: player.c_body,
        c_legs: player.c_legs,
        c_mount: player.c_mount,
        sitting: player.sitting,
        seat_y_offset,
        sleeping: player.sleeping,
        full_rotation,
    };
    let mut layers = assemble_layers(&state, assets);
    // `DrawPlayer_TransformDrawData` (`PlayerDrawLayers.cs:4092-4119`): the
    // sleeping pose rotates the whole set around `fullRotationOrigin`
    // (`PlayerSleepingHelper.cs:74` sets it to the hitbox centre).
    if state.sleeping && state.full_rotation != 0.0 {
        let pivot = [
            player.x + player.full_rotation_origin.0,
            player.y + player.full_rotation_origin.1 + player.gfx_off_y,
        ];
        super::player_layers::apply_player_rotation(&mut layers, pivot, state.full_rotation);
    }
    push_layers(r, &layers, camera, zoom, screen)
}

/// `Main.PlayerRenderer.DrawPlayer` in UI space (`UICharacter.cs:118`): the
/// existing frame-UV machinery of [`draw_player`] at a fixed screen rect,
/// full-white light (the preview is unlit), the draft's colours, `scale` =
/// `UICharacter._characterScale` (1.5 creation / 1.0 card), `frame_row` =
/// `UICharacter`'s animation row (0 static, `ui::char_creation::
/// animated_frame_row(time)` animated, `:85-95`). `pos` is
/// `GetPlayerPosition`'s hitbox anchor (`UICharacter.cs:122-130` — the port's
/// `preview_anchor`); the 40x56 art box rides the hitbox at the same
/// `(-10, -10)` offset the world draw uses, scaled. The preview faces right —
/// `PreparePreview_Main`'s `direction = 1` (`UICharacterCreation.cs:616`).
///
/// The port decodes only the base body sheets (no per-variant sets), so every
/// `skinVariant`/`hair` id renders the base male frame — the documented
/// preview deviation that mirrors the char-select card's.
pub(crate) fn draw_player_preview(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    pos: [f32; 2],
    scale: f32,
    frame_row: u32,
    colors: &ui::char_creation::PreviewColors,
) -> u32 {
    let byte = |c: u8| c as f32 / 255.0;
    let skin = [
        byte(colors.skin[0]),
        byte(colors.skin[1]),
        byte(colors.skin[2]),
        1.0,
    ];
    let eye_white = [1.0, 1.0, 1.0, 1.0];
    let eye = [
        byte(colors.eye[0]),
        byte(colors.eye[1]),
        byte(colors.eye[2]),
        1.0,
    ];
    let shirt = [
        byte(colors.shirt[0]),
        byte(colors.shirt[1]),
        byte(colors.shirt[2]),
        1.0,
    ];
    let undershirt = [
        byte(colors.undershirt[0]),
        byte(colors.undershirt[1]),
        byte(colors.undershirt[2]),
        1.0,
    ];
    let pants = [
        byte(colors.pants[0]),
        byte(colors.pants[1]),
        byte(colors.pants[2]),
        1.0,
    ];
    let shoes = [
        byte(colors.shoe[0]),
        byte(colors.shoe[1]),
        byte(colors.shoe[2]),
        1.0,
    ];
    let hair = [
        byte(colors.hair[0]),
        byte(colors.hair[1]),
        byte(colors.hair[2]),
        1.0,
    ];

    let pos = [pos[0] - 10.0 * scale, pos[1] - 10.0 * scale];
    let size = [40.0 * scale, 56.0 * scale];
    let body = frame_row;
    let mut quads = 0;
    let (arm_col, arm_row) = front_arm_cell(body);
    let frame_uv = |t: Option<&BgTex>, row: u32| t.map(|t| player_frame_uv(t, row));
    let cell_uv = |t: Option<&BgTex>, col: u32, row: u32| t.map(|t| player_cell_uv(t, col, row));
    let mut part = |tex: Option<&BgTex>, uv: Option<[f32; 4]>, color: [f32; 4]| {
        if tex.is_some() {
            if let Some(uv) = uv {
                r.push_ui_sprite(pos, size, uv, color);
                quads += 1;
            }
        }
    };
    // `PlayerFrame`: the legs animate on `legFrame`, the body/arms/head on
    // `bodyFrame` (`UICharacter.UpdateAnim` writes all three, `:92-93`).
    part(
        assets.legs.as_ref(),
        frame_uv(assets.legs.as_ref(), body),
        skin,
    );
    part(
        assets.pants.as_ref(),
        frame_uv(assets.pants.as_ref(), body),
        pants,
    );
    part(
        assets.shoes.as_ref(),
        frame_uv(assets.shoes.as_ref(), body),
        shoes,
    );
    // Back arm behind the torso, front arm in front of the shirt (composite
    // layers), the under-shirt between the torso and the shirt.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, (arm_row + 2).min(3)),
        skin,
    );
    part(
        assets.torso.as_ref(),
        cell_uv(assets.torso.as_ref(), 0, 0),
        skin,
    );
    part(
        assets.torso.as_ref(),
        cell_uv(assets.torso.as_ref(), 0, 0),
        undershirt,
    );
    part(
        assets.shirt.as_ref(),
        cell_uv(assets.shirt.as_ref(), 0, 0),
        shirt,
    );
    // Same `CreateCompositeData` table for the preview's front arm — column 8
    // is the composite back-arm use column, not a walking pose.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, arm_row),
        skin,
    );
    part(
        assets.head.as_ref(),
        frame_uv(assets.head.as_ref(), body),
        skin,
    );
    part(
        assets.eye_whites.as_ref(),
        frame_uv(assets.eye_whites.as_ref(), body),
        eye_white,
    );
    part(
        assets.eyes.as_ref(),
        frame_uv(assets.eyes.as_ref(), body),
        eye,
    );
    part(
        assets.hair.as_ref(),
        frame_uv(assets.hair.as_ref(), body),
        hair,
    );
    quads
}

/// `Main.MapPlayerRenderer.DrawPlayerHead` (`Main.cs:55954`): the local player's
/// 84x84 head icon for the fullscreen map.
///
/// This engine has no render targets, so the `PlayerHeadDrawRenderTargetContent`
/// composite (skin head + eye whites + eyes + hair) and the
/// `AnOutlinedDrawRenderTargetContent` black/border rings are reproduced by pushing
/// the same quads straight into the screen-space `ui` range (drawn after
/// `map`/`map_back`, matching vanilla's frame → tiles → head order).
///
/// `pos` is the icon **centre** (`head_icon_pos`). The target is blitted centred
/// with origin `(42, 42)` (`MapHeadRenderer.cs:50`), and the head frame sits at
/// target-local `(26, 28)` (`PlayerDrawHeadSet.cs:150-155` + `147`), so its screen
/// top-left is `pos + ([-16, -14]) * scale` and its size is `[40, 56] * scale`.
/// `scale` is `Main.UIScale` — this engine has no UI scale, so it is passed `1.0`.
/// `border` is `Main.GetPlayerHeadBordersColor` (White in singleplayer).
pub(crate) fn draw_player_map_icon(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    player: &Player,
    pos: [f32; 2],
    scale: f32,
    border: [f32; 4],
) -> u32 {
    // `PlayerDrawHeadSet.cs:129-133` (`BoringSetup`): the raw skin/eye/hair colours,
    // alpha 1. The icon is unlit — `MapHeadRenderer.cs:50` blits the target with
    // `Color.White`, so there is no tile-light multiply.
    let skin = [1.0, 125.0 / 255.0, 90.0 / 255.0, 1.0];
    let eye_white = [1.0, 1.0, 1.0, 1.0];
    let eye = [105.0 / 255.0, 90.0 / 255.0, 75.0 / 255.0, 1.0];
    let hair = [215.0 / 255.0, 90.0 / 255.0, 55.0 / 255.0, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];
    // `PlayerDrawHeadSet.cs:142-146`: `direction < 0` flips horizontally.
    let flip = player.facing < 0;
    // `PlayerDrawHeadSet.cs:148-149`: the neutral frame row (`bodyFrameMemory` with
    // `Y = 0`). `HairFrame` shortens this by 1px (`:82-90`), ignored.
    let frame = 0u32;
    let parts: [(Option<&BgTex>, [f32; 4]); 4] = [
        (assets.head.as_ref(), skin),
        (assets.eye_whites.as_ref(), eye_white),
        (assets.eyes.as_ref(), eye),
        (assets.hair.as_ref(), hair),
    ];
    let size = [40.0 * scale, 56.0 * scale];
    let base = [pos[0] - 16.0 * scale, pos[1] - 14.0 * scale];
    let mut quads = 0;
    // One `Draw(helperTarget, offset, tint)` — all four parts of the composite, so
    // the union of their alphas is the silhouette the outline rings are built from.
    let mut copy = |offset: [f32; 2], tint: Option<[f32; 4]>| {
        let at = [base[0] + offset[0] * scale, base[1] + offset[1] * scale];
        for &(tex, color) in parts.iter() {
            if let Some(tex) = tex {
                let uv = flip_uv(player_frame_uv(tex, frame), flip);
                r.push_ui_sprite(at, size, uv, tint.unwrap_or(color));
                quads += 1;
            }
        }
    };
    // `AnOutlinedDrawRenderTargetContent.cs:38-49`: 8 black ring copies at Manhattan
    // radius 4 (`num = 2`, `num2 = num * 2 = 4`).
    for offset in [
        [-4.0, 0.0],
        [4.0, 0.0],
        [0.0, -4.0],
        [0.0, 4.0],
        [-2.0, -2.0],
        [-2.0, 2.0],
        [2.0, -2.0],
        [2.0, 2.0],
    ] {
        copy(offset, Some(black));
    }
    // `AnOutlinedDrawRenderTargetContent.cs:50-60`: 4 border copies at Manhattan
    // radius 2 (`num2 = num = 2`).
    for offset in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
        copy(offset, Some(border));
    }
    // `AnOutlinedDrawRenderTargetContent.cs:62`: the head composite itself.
    copy([0.0, 0.0], None);
    quads
}

/// First enemy pass: the blue slime, tinted and lit like the player, with an overhead
/// health bar once damaged.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_npcs(
    r: &mut SpriteRenderer,
    assets: &NpcAssets,
    npcs: &[Npc],
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let white = r.white_uv();
    let mut quads = 0;
    for npc in npcs {
        // The C# wraps the whole NPC draw with `position += netOffset`
        // (`Main.cs:22335`, restored at `:22450`) — the port clones the NPC
        // and adds the accumulator to its position, which every arm derives
        // its anchor from. Zero on the host / single-player.
        let n_owned = npc.offset_clone();
        let n: &Npc = if npc.net_offset == Vec2::ZERO {
            npc
        } else {
            &n_owned
        };
        let (cx, cy) = n.center();
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        // The width + the height: the Golem family's draw anchors the
        // frame off the box's bottom edge (the vanilla's
        // `position.Y + height + 4`, `Main.cs:25513`) and the Deerclops'
        // off `Bottom` (`Main.cs:26337`).
        let (w, h) = n.size();
        let (tex, frame_h, size, pos) = match n.kind {
            NpcKind::Slime => (
                assets.slime.as_ref(),
                26.0,
                // The 32x26 frame scales with the variant
                // (`SetDefaults_ForNetId`'s size scale) and is anchored at the
                // frame centre over the hitbox centre (`Main.DrawNPC`'s
                // `origin = frame.Size() / 2`).
                [32.0 * n.slime.stats().scale, 26.0 * n.slime.stats().scale],
                [
                    cx - 16.0 * n.slime.stats().scale,
                    cy - 13.0 * n.slime.stats().scale,
                ],
            ),
            NpcKind::DemonEye => (
                assets.eye.as_ref(),
                24.0,
                [38.0, 24.0],
                [n.x - 4.0, n.y + 2.0],
            ),
            NpcKind::WanderingEye => (
                assets.wandering_eye.as_ref(),
                // `Main.npcFrameCount[133] = 4` (Main.cs:67082).
                assets.wandering_eye.as_ref().map_or(24.0, |t| t.h / 4.0),
                assets
                    .wandering_eye
                    .as_ref()
                    .map_or([38.0, 24.0], |t| [38.0, t.h / 4.0]),
                [cx - 19.0, cy - 12.0],
            ),
            // The humanoid walkers: the sheet's real row at the generic
            // anchor (the frame centred on the box's centre-X, its centre
            // 4 px below the box's bottom, `Main.cs:25511-25516`); the
            // rows ride each kind's `Main.npcFrameCount` (`npc.rs`'s
            // `humanoid_frame_count`). The Zombie's sheet is the single
            // 30x48 NPC_9 frame (no `FindFrame` case — static).
            NpcKind::Zombie => humanoid_draw(assets.zombie.as_ref(), 1, n, w, h),
            NpcKind::Guide => humanoid_draw(assets.guide.as_ref(), 26, n, w, h),
            NpcKind::Merchant => humanoid_draw(assets.merchant.as_ref(), 25, n, w, h),
            NpcKind::Dryad => humanoid_draw(assets.dryad.as_ref(), 21, n, w, h),
            NpcKind::Nurse => humanoid_draw(assets.nurse.as_ref(), 23, n, w, h),
            NpcKind::ArmsDealer => humanoid_draw(assets.arms_dealer.as_ref(), 25, n, w, h),
            NpcKind::Demolitionist => humanoid_draw(assets.demolitionist.as_ref(), 25, n, w, h),
            NpcKind::Painter => humanoid_draw(assets.painter.as_ref(), 25, n, w, h),
            NpcKind::WitchDoctor => humanoid_draw(assets.witch_doctor.as_ref(), 26, n, w, h),
            NpcKind::Clothier => humanoid_draw(assets.clothier.as_ref(), 23, n, w, h),
            NpcKind::TravellingMerchant => humanoid_draw(
                assets.travelling_merchant.as_ref(),
                26,
                n,
                w,
                h,
            ),
            // The Angler: the same humanoid shape over NPC_369 — 23 rows
            // (`Main.npcFrameCount[369]`, `Main.cs:67120`).
            NpcKind::Angler => humanoid_draw(assets.angler.as_ref(), 23, n, w, h),
            // The Sleeping Angler: NPC_376's two rows over the flat 34x8
            // body — the sheet is 40 wide, drawn centred on the box.
            NpcKind::SleepingAngler => (
                assets.sleeping_angler.as_ref(),
                assets.sleeping_angler.as_ref().map_or(8.0, |t| t.h / 2.0),
                [34.0, 8.0],
                [n.x, n.y],
            ),
            // The EoC: the 2-frame 100x110 sheet centred on the hitbox.
            NpcKind::EyeOfCthulhu => (
                assets.eoc.as_ref(),
                assets.eoc.as_ref().map_or(110.0, |t| t.h / 2.0),
                [100.0, 110.0],
                [n.x, n.y],
            ),
            // The Queen Bee: the 2-frame 66x66 sheet.
            NpcKind::QueenBee => (
                assets.queen_bee.as_ref(),
                assets.queen_bee.as_ref().map_or(66.0, |t| t.h / 2.0),
                [66.0, 66.0],
                [n.x, n.y],
            ),
            // The Skeletron head: the single-frame 80x102 sheet.
            NpcKind::SkeletronHead => (
                assets.skeletron_head.as_ref(),
                assets.skeletron_head.as_ref().map_or(102.0, |t| t.h),
                [80.0, 102.0],
                [n.x, n.y],
            ),
            // The WoF: the single-frame sheet over the 120x120 hitbox.
            NpcKind::WallOfFlesh => (
                assets.wof.as_ref(),
                assets.wof.as_ref().map_or(120.0, |t| t.h),
                [120.0, 120.0],
                [n.x, n.y],
            ),
            // The hungry: `Main.npcFrameCount[115] = 3` frames of 30×30; the
            // leech segments: 1 frame of 14×14.
            NpcKind::Hungry => (
                assets.hungry.as_ref(),
                assets.hungry.as_ref().map_or(30.0, |t| t.h / 3.0),
                [30.0, 30.0],
                [n.x, n.y],
            ),
            NpcKind::LeechHead | NpcKind::LeechBody => (
                assets.leech.as_ref(),
                assets.leech.as_ref().map_or(14.0, |t| t.h),
                [14.0, 14.0],
                [n.x, n.y],
            ),
            // The frost trio: the 4/5/7-frame strips over the 26x40 frames.
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => {
                let (tex, frames) = match n.kind {
                    NpcKind::SnowmanGangsta => (assets.snowman_gangsta.as_ref(), 4.0),
                    NpcKind::SnowBalla => (assets.snow_balla.as_ref(), 5.0),
                    _ => (assets.mystery_snowman.as_ref(), 7.0),
                };
                (
                    tex,
                    tex.map_or(40.0, |t| t.h / frames),
                    [26.0, 40.0],
                    [n.x, n.y],
                )
            }
            // The martians: one sheet per kind at the generic anchor — the
            // Officer's 9 rows of 46, the Grunt's 9 of 44, the Engineer's
            // 11 of 46, the Gigazapper's 9 of 46 (`Main.npcFrameCount`,
            // `Main.cs:67082`); the v1 shared NPC_383 across all four and
            // drew the 18x40 box.
            NpcKind::MartianOfficer => martian_draw(assets.martian_officer.as_ref(), 9, n, w, h),
            NpcKind::GrayGrunt => martian_draw(assets.gray_grunt.as_ref(), 9, n, w, h),
            NpcKind::MartianEngineer => {
                martian_draw(assets.martian_engineer.as_ref(), 11, n, w, h)
            }
            NpcKind::GigaZapper => martian_draw(assets.gigazapper.as_ref(), 9, n, w, h),
            // The probe: the 8-frame NPC_399 strip over the 42x72 frames
            // (80-px rows — the v1's box-sized quad sliced the row).
            NpcKind::MartianProbe => martian_draw(assets.martian_probe.as_ref(), 8, n, w, h),
            // The drone: the 2-frame NPC_388 strip over the 40x30 frames.
            NpcKind::MartianDrone => martian_draw(assets.martian_drone.as_ref(), 2, n, w, h),
            // The shooters: `NPC_381`'s 9 rows of 48 / `NPC_382`'s 9 of 46.
            NpcKind::BrainScrambler => {
                martian_draw(assets.brain_scrambler.as_ref(), 9, n, w, h)
            }
            NpcKind::RayGunner => martian_draw(assets.ray_gunner.as_ref(), 9, n, w, h),
            // The rider: the 5-frame NPC_390 strip over the 18x40 frames.
            NpcKind::ScutlixRider => martian_draw(assets.scutlix_rider.as_ref(), 5, n, w, h),
            // The carrier: the 6-frame NPC_391 strip over the 60x40 frames.
            NpcKind::Scutlix => martian_draw(assets.scutlix.as_ref(), 6, n, w, h),
            // The saucer family (`Main.cs:24586-24612`): every mount draws
            // at the CENTRE rotated by the core's inherited rotation — the
            // shell's 4 frames of 96, the turret's 18 of 32, the cannon's
            // 8 of 34, the core's 11 of 40.
            NpcKind::MartianSaucer => saucer_draw(assets.saucer.as_ref(), 4, n, w, h),
            NpcKind::MartianSaucerTurret => {
                saucer_draw(assets.saucer_turret.as_ref(), 18, n, w, h)
            }
            NpcKind::MartianSaucerCannon => {
                saucer_draw(assets.saucer_cannon.as_ref(), 8, n, w, h)
            }
            NpcKind::MartianSaucerCore => saucer_draw(assets.saucer_core.as_ref(), 11, n, w, h),
            // The Dutchman: the 1-frame ship strip + the cannons'
            // 9-frame strip.
            NpcKind::FlyingDutchman => (
                assets.dutchman.as_ref(),
                assets.dutchman.as_ref().map_or(120.0, |t| t.h),
                [350.0, 120.0],
                [n.x, n.y],
            ),
            NpcKind::FlyingDutchmanCannon => (
                assets.dutchman_cannon.as_ref(),
                assets.dutchman_cannon.as_ref().map_or(30.0, |t| t.h / 9.0),
                [30.0, 30.0],
                [n.x, n.y],
            ),
            // The DD2 goblins: the 14-frame NPC_552-554 strips over the
            // 18x40 frames (the sheets are the shared humanoid shape).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => (
                assets.dd2_goblins.as_ref(),
                assets.dd2_goblins.as_ref().map_or(40.0, |t| t.h / 14.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            // The support family: the bombers' 18x40 and the javelin
            // throwers' 26x52 strips (the shared humanoid sheets), the
            // skeletons' 18x40 strips.
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3
            | NpcKind::DD2SkeletonT1
            | NpcKind::DD2SkeletonT3 => (
                assets.dd2_support.as_ref(),
                assets.dd2_support.as_ref().map_or(40.0, |t| t.h / 14.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            // The Dark Mage: the 20-frame NPC_564 strip over the 34x62 frames.
            NpcKind::DD2DarkMage => (
                assets.dark_mage.as_ref(),
                assets.dark_mage.as_ref().map_or(62.0, |t| t.h / 20.0),
                [34.0, 62.0],
                [n.x, n.y],
            ),
            // Betsy: the 11-frame NPC_551 strip over the 190x90 frames.
            NpcKind::DD2Betsy => (
                assets.betsy.as_ref(),
                assets.betsy.as_ref().map_or(90.0, |t| t.h / 11.0),
                [190.0, 90.0],
                [n.x, n.y],
            ),
            // The WitherBeast: the 14-frame NPC_568 strip over the 18x40 frames.
            NpcKind::DD2WitherBeast => (
                assets.wither_beast.as_ref(),
                assets.wither_beast.as_ref().map_or(40.0, |t| t.h / 14.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            // The tavernkeeper: the 16-frame NPC_550 strip over the 18x40.
            NpcKind::DD2Bartender => (
                assets.tavern.as_ref(),
                assets.tavern.as_ref().map_or(40.0, |t| t.h / 16.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            // The crystal: the NPC_548 strip over the 40x40 frames.
            NpcKind::DD2EterniaCrystal => (
                assets.crystal.as_ref(),
                assets.crystal.as_ref().map_or(40.0, |t| t.h),
                [40.0, 40.0],
                [n.x, n.y],
            ),
            // The cultist: the 1-frame NPC_422 strip over the 130x270 frames.
            NpcKind::AncientCultist => (
                assets.cultist.as_ref(),
                assets.cultist.as_ref().map_or(270.0, |t| t.h),
                [130.0, 270.0],
                [n.x, n.y],
            ),
            // The four pillars: `Main.cs:24159-24263`'s dedicated arm
            // centres the single 174x364 frame on the NPC's Centre with
            // `origin = halfSize` (the generic bottom anchor would sink
            // the overhanging art 47 px into the ground) — `tower_draw`
            // computes it (the other agent's merge had all four kinds ride
            // the shared `towers` sheet — NPC_493 — at the box's corner).
            NpcKind::StardustTower | NpcKind::NebulaTower | NpcKind::SolarTower | NpcKind::VortexTower => {
                let tex = match n.kind {
                    NpcKind::StardustTower => assets.tower_stardust.as_ref(),
                    NpcKind::NebulaTower => assets.tower_nebula.as_ref(),
                    NpcKind::SolarTower => assets.tower_solar.as_ref(),
                    _ => assets.tower_vortex.as_ref(),
                };
                tower_draw(tex, n, w, h)
            }
            // The shards: the cell/jellyfish/beast 2-row strips + the
            // vortex walkers' 14-frame humanoid strips.
            NpcKind::StardustCellBig => (
                assets.shards.as_ref(),
                assets.shards.as_ref().map_or(44.0, |t| t.h / 4.0),
                [44.0 * n.scale, 44.0 * n.scale],
                [n.x, n.y],
            ),
            // The cell's ramp rides `n.scale` (`AI()`'s aiStyle-95 arm,
            // `NPC.cs:39393`) — the strip's own frame height stays put.
            NpcKind::StardustCellSmall => (
                assets.shards.as_ref(),
                assets.shards.as_ref().map_or(22.0, |t| t.h / 2.0),
                [22.0 * n.scale, 22.0 * n.scale],
                [n.x, n.y],
            ),
            NpcKind::StardustJellyfishBig => (
                assets.shards.as_ref(),
                assets.shards.as_ref().map_or(60.0, |t| t.h / 5.0),
                [40.0, 60.0],
                [n.x, n.y],
            ),
            NpcKind::NebulaBeast => (
                assets.shard_hovers.as_ref(),
                assets.shard_hovers.as_ref().map_or(44.0, |t| t.h / 12.0),
                [50.0, 44.0],
                [n.x, n.y],
            ),
            // The vortex trio: the hunter's 50x62 / the alien's 40x28 /
            // the headcrab's 24x26 over the 426 sheet's 15 rows.
            NpcKind::VortexHunter => (
                assets.vortex_trio.as_ref(),
                assets.vortex_trio.as_ref().map_or(62.0, |t| t.h / 15.0),
                [50.0, 62.0],
                [n.x, n.y],
            ),
            NpcKind::VortexAlien => (
                assets.vortex_trio.as_ref(),
                assets.vortex_trio.as_ref().map_or(28.0, |t| t.h / 15.0),
                [40.0, 28.0],
                [n.x, n.y],
            ),
            NpcKind::VortexHeadcrab => (
                assets.vortex_trio.as_ref(),
                assets.vortex_trio.as_ref().map_or(26.0, |t| t.h / 15.0),
                [24.0, 26.0],
                [n.x, n.y],
            ),
            // The nebula trio: the floater 40x40 / the suckler 44x44 / the
            // predictor 22x56 over the 420 sheet's 6 rows.
            // The stardust/solar tower mobs' strips over the 409 sheet.
            NpcKind::StardustSpiderBig => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(42.0, |t| t.h / 4.0),
                [34.0, 42.0],
                [n.x, n.y],
            ),
            NpcKind::StardustSpiderSmall => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(22.0, |t| t.h / 11.0),
                [22.0, 22.0],
                [n.x, n.y],
            ),
            NpcKind::StardustSoldier => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(56.0, |t| t.h / 5.0),
                [22.0, 56.0],
                [n.x, n.y],
            ),
            NpcKind::SolarDrakomire => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(40.0, |t| t.h / 1.0),
                [60.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::SolarDrakomireRider => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(40.0, |t| t.h / 13.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::SolarSroller => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(52.0, |t| t.h / 10.0),
                [46.0, 52.0],
                [n.x, n.y],
            ),
            NpcKind::SolarCorite => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(38.0, |t| t.h / 15.0),
                [38.0, 38.0],
                [n.x, n.y],
            ),
            // The pillar worms: the crawltipede chain's 20x20 single
            // frames (the vanilla assembles the chain from the head's
            // spawn) and the stardust worm head.
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(20.0, |t| t.h / 1.0),
                [20.0, 20.0],
                [n.x, n.y],
            ),
            NpcKind::StardustWormHead => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(32.0, |t| t.h / 1.0),
                [32.0, 32.0],
                [n.x, n.y],
            ),
            NpcKind::SolarSpearman => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(56.0, |t| t.h / 10.0),
                [22.0, 56.0],
                [n.x, n.y],
            ),
            NpcKind::SolarSelenian => (
                assets.tower_mobs.as_ref(),
                assets.tower_mobs.as_ref().map_or(56.0, |t| t.h / 8.0),
                [22.0, 56.0],
                [n.x, n.y],
            ),
            NpcKind::NebulaFloater => (
                assets.nebula_trio.as_ref(),
                assets.nebula_trio.as_ref().map_or(40.0, |t| t.h / 6.0),
                [40.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::BrainSuckler => (
                assets.nebula_trio.as_ref(),
                assets.nebula_trio.as_ref().map_or(44.0, |t| t.h / 6.0),
                [44.0, 44.0],
                [n.x, n.y],
            ),
            NpcKind::Predictor => (
                assets.nebula_trio.as_ref(),
                assets.nebula_trio.as_ref().map_or(56.0, |t| t.h / 6.0),
                [22.0, 56.0],
                [n.x, n.y],
            ),
            NpcKind::VortexRifleman | NpcKind::VortexSoldier => (
                assets.shard_hovers.as_ref(),
                assets.shard_hovers.as_ref().map_or(56.0, |t| t.h / 13.0),
                [30.0, 56.0],
                [n.x, n.y],
            ),
            // The Moon Lord family: the per-part rows (38x56 / 46x66 /
            // 60x60 / 30x30) over the 396 sheet's strips — the core and
            // the leech ride their own 1-row strips.
            NpcKind::MoonLordHead => (
                assets.moon_lord_parts.as_ref(),
                assets.moon_lord_parts.as_ref().map_or(56.0, |t| t.h / 8.0),
                [38.0, 56.0],
                [n.x, n.y],
            ),
            NpcKind::MoonLordHand => (
                assets.moon_lord_parts.as_ref(),
                assets.moon_lord_parts.as_ref().map_or(66.0, |t| t.h / 8.0),
                [46.0, 66.0],
                [n.x, n.y],
            ),
            NpcKind::MoonLordCore => (
                assets.moon_lord_parts.as_ref(),
                assets.moon_lord_parts.as_ref().map_or(66.0, |t| t.h / 8.0),
                [46.0, 66.0],
                [n.x, n.y],
            ),
            NpcKind::MoonLordFreeEye => (
                assets.moon_lord_parts.as_ref(),
                assets.moon_lord_parts.as_ref().map_or(60.0, |t| t.h / 8.0),
                [60.0, 60.0],
                [n.x, n.y],
            ),
            NpcKind::MoonLordLeechBlob => (
                assets.moon_lord_parts.as_ref(),
                assets.moon_lord_parts.as_ref().map_or(30.0, |t| t.h / 8.0),
                [30.0, 30.0],
                [n.x, n.y],
            ),
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => (
                assets.dd2_support.as_ref(),
                assets.dd2_support.as_ref().map_or(52.0, |t| t.h / 14.0),
                [26.0, 52.0],
                [n.x, n.y],
            ),
            NpcKind::MartianWalker => (
                assets.martian_walker_big.as_ref(),
                assets
                    .martian_walker_big
                    .as_ref()
                    .map_or(110.0, |t| t.h / 8.0),
                [34.0, 110.0],
                [n.x, n.y],
            ),
            // The pirates: the 16/20-frame strips over the 18x40 frames
            // (the Captain's 20-frame sheet rides NPC_216).
            NpcKind::PirateDeckhand
            | NpcKind::PirateCaptainsMate
            | NpcKind::PirateArcher
            | NpcKind::PirateCorsair
            | NpcKind::PirateCaptain => {
                let (tex, frames) = if n.kind == NpcKind::PirateCaptain {
                    (assets.pirate_captain.as_ref(), 20.0)
                } else {
                    (assets.pirates.as_ref(), 16.0)
                };
                (
                    tex,
                    tex.map_or(40.0, |t| t.h / frames),
                    [18.0, 40.0],
                    [n.x, n.y],
                )
            }
            // The Destroyer: the single-frame 48x48 sheet.
            NpcKind::TheDestroyer => (
                assets.destroyer.as_ref(),
                assets.destroyer.as_ref().map_or(48.0, |t| t.h),
                [48.0, 48.0],
                [n.x, n.y],
            ),
            // The Prime: the head's 6-row strip (NPC_127 is 140x940 —
            // 156-px rows, `Main.npcFrameCount[127]` = 6, `Main.cs:67082`;
            // the v1 read `t.h` and sampled the whole strip as one frame
            // at the box's 80x102).
            NpcKind::SkeletronPrime => prime_draw(assets.prime.as_ref(), 6, n, w, h),
            // The Twins: `Main.npcFrameCount[125/126]` = 6 — NPC_125/126
            // are 110x1200 (200-px rows), and `FindFrame`'s case 125/126
            // (`NPC.cs:76906-76924`) walks the spawn rows 0-2 and the live
            // rows 3-5 (the `ai[0]` hover anchor's +3 offset) over a
            // 20-tick beat. The v1 ran an 8-tick 2-row strip over a
            // 600-px slice of the sheet at the box's corner.
            NpcKind::Retinazer | NpcKind::Spazmatism => {
                let tex = if n.kind == NpcKind::Retinazer {
                    assets.retinazer.as_ref()
                } else {
                    assets.spazmatism.as_ref()
                };
                boss_draw(tex, 6, n, w, h)
            }
            // The Plantera: the 5-frame 86x86 sheet.
            NpcKind::Plantera => (
                assets.plantera.as_ref(),
                assets.plantera.as_ref().map_or(86.0, |t| t.h / 5.0),
                [86.0, 86.0],
                [n.x, n.y],
            ),
            // The Brain: the single-frame 160x110 sheet.
            NpcKind::BrainOfCthulhu => (
                assets.brain.as_ref(),
                assets.brain.as_ref().map_or(110.0, |t| t.h),
                [160.0, 110.0],
                [n.x, n.y],
            ),
            // The Creeper: the single-frame 30x30 sheet.
            NpcKind::Creeper => (
                assets.creeper.as_ref(),
                assets.creeper.as_ref().map_or(30.0, |t| t.h),
                [30.0, 30.0],
                [n.x, n.y],
            ),
            // The Goblin Tinkerer: the 3-frame town sheet over NPC_107.
            NpcKind::GoblinTinkerer => humanoid_draw(assets.goblin_tinkerer.as_ref(), 25, n, w, h),
            // The bound goblin: the single tied frame over NPC_105.
            NpcKind::BoundGoblin => (
                assets.bound_goblin.as_ref(),
                assets.bound_goblin.as_ref().map_or(48.0, |t| t.h),
                [34.0, 54.0],
                [n.x - 8.0, n.y - 10.0],
            ),
            // The Golem family (the v1 single-frame draws): the body
            // 140x140, the heads 70x70, the fists 40x30 (the SetDefaults
            // boxes; the sheets draw at the box size).
            // The Golem family: the multi-frame draws over the real
            // sheets. `Main.npcFrameCount` 245=7/246=6/247-248=1/249=2
            // (`Main.cs:67108`) — the sheets divide into rows of
            // `t.h / count` (1218/7 = 174, 708/6 = 118, 276/2 = 138,
            // 40/1 = 40) and `npc_frame_uv` selects the row the machines
            // wrote (`FindFrame` cases 245/246/249). The generic draw
            // anchor (`Main.cs:25511-25516`) centers the frame on the
            // box's centre-X and puts the frame's centre 4 px below the
            // box's bottom.
            NpcKind::Golem => {
                let t = assets.golem.as_ref();
                let fh = t.map_or(174.0, |t| t.h / 7.0);
                (
                    t,
                    fh,
                    [186.0, fh],
                    [n.x + w * 0.5 - 93.0, n.y + h + 4.0 - fh * 0.5],
                )
            }
            NpcKind::GolemHead | NpcKind::GolemHeadFree => {
                let (t, count) = if n.kind == NpcKind::GolemHead {
                    (assets.golem_head.as_ref(), 6.0)
                } else {
                    (assets.golem_head_free.as_ref(), 2.0)
                };
                let fh = t.map_or(if count == 6.0 { 118.0 } else { 138.0 }, |t| {
                    t.h / count
                });
                (
                    t,
                    fh,
                    [112.0, fh],
                    [n.x + w * 0.5 - 56.0, n.y + h + 4.0 - fh * 0.5],
                )
            }
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => {
                let t = assets.golem_fist.as_ref();
                let fh = t.map_or(40.0, |t| t.h);
                (
                    t,
                    fh,
                    [50.0, fh],
                    [n.x + w * 0.5 - 25.0, n.y + h + 4.0 - fh * 0.5],
                )
            }
            // The Deerclops: `DrawNPCDirect_Deerclops` (`Main.cs:26334-26412`)
            // — the 5x5 grid's cell (218x240 drawn 220x242) anchored at the
            // NPC's Bottom with the origin (106 or 114, 238): the cell's
            // centre lands 4 px against the facing direction and its bottom
            // 4 px below the hitbox (`origin.Y -= 4`).
            NpcKind::Deerclops => {
                let t = assets.deerclops.as_ref();
                let cw = t.map_or(220.0, |t| t.w / 5.0 + 2.0);
                let ch = t.map_or(242.0, |t| t.h / 5.0 + 2.0);
                let lean = if n.facing < 0 { 4.0 } else { -4.0 };
                (
                    t,
                    ch,
                    [cw, ch],
                    [n.x + w * 0.5 + lean - cw * 0.5, n.y + h + 4.0 - ch],
                )
            }
            // The Skeletron's hand: the 52x52 single-frame sheet.
            NpcKind::SkeletronHand => (
                assets.skeletron_hand.as_ref(),
                assets.skeletron_hand.as_ref().map_or(52.0, |t| t.h),
                [52.0, 52.0],
                [n.x, n.y],
            ),
            // The WoF's mirror eye: the 100x100 sheet (the vanilla's
            // `WorldofFleshEye` texture).
            NpcKind::WallOfFleshEye => (
                assets.wof_eye.as_ref(),
                assets.wof_eye.as_ref().map_or(100.0, |t| t.h),
                [100.0, 100.0],
                [n.x, n.y],
            ),
            // The Prime's limbs (`Main.npcFrameCount` 128/129/130/131 =
            // 1/2/2/1, `Main.cs:67082`): the cannon 38x76, the saw's 2
            // rows of 78, the vice's 2 rows of 70, the laser 40x62 — each
            // at the generic anchor (the v1 read `t.h` as the frame height
            // and pinned every kind to row 0).
            NpcKind::PrimeCannon => prime_draw(assets.prime_cannon.as_ref(), 1, n, w, h),
            NpcKind::PrimeSaw => prime_draw(assets.prime_saw.as_ref(), 2, n, w, h),
            NpcKind::PrimeVice => prime_draw(assets.prime_vice.as_ref(), 2, n, w, h),
            NpcKind::PrimeLaser => prime_draw(assets.prime_laser.as_ref(), 1, n, w, h),            // The goblins: the 3-frame humanoid sheets (the vanilla
            // `Main.npcFrameCount[26/27/28/111]` = 3; 38x48 each frame).
            NpcKind::GoblinPeon
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher => {
                let tex = match n.kind {
                    NpcKind::GoblinPeon => assets.goblin_peon.as_ref(),
                    NpcKind::GoblinThief => assets.goblin_thief.as_ref(),
                    NpcKind::GoblinWarrior => assets.goblin_warrior.as_ref(),
                    _ => assets.goblin_archer.as_ref(),
                };
                (
                    tex,
                    tex.map_or(48.0, |t| t.h / 3.0),
                    [38.0, 48.0],
                    [n.x - 10.0, n.y - 5.0],
                )
            }
            // The skeleton family: the same humanoid frame shape — the
            // strip's frame height rides the sheet (15 frames).
            NpcKind::Skeleton | NpcKind::UndeadMiner => {
                let tex = if n.kind == NpcKind::Skeleton {
                    assets.skeleton.as_ref()
                } else {
                    assets.undead_miner.as_ref()
                };
                humanoid_draw(tex, 15, n, w, h)
            }
            // The goblin army's fighters: the humanoid strips (the 16-frame
            // family 26/27/28/111, the summoner's 20) — the frame height
            // rides each sheet, the display box the zombie's 38x48 shape.
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner => {
                let (tex, count) = match n.kind {
                    NpcKind::GoblinPeon => (assets.goblin_peon.as_ref(), 16),
                    NpcKind::GoblinThief => (assets.goblin_thief.as_ref(), 16),
                    NpcKind::GoblinWarrior => (assets.goblin_warrior.as_ref(), 16),
                    NpcKind::GoblinArcher => (assets.goblin_archer.as_ref(), 21),
                    _ => (assets.goblin_summoner.as_ref(), 20),
                };
                humanoid_draw(tex, count, n, w, h)
            }
            // The Cave Bat: the 5-frame flap over the 22x18 frames.
            // The casters: the humanoid sheets — the sorcerer's strip is 3
            // frames, the Dark Caster's 6 (`Main.npcFrameCount` 29/32/45,
            // `Main.cs:67082`), the display box the zombie's 38x48 shape.
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                let (tex, count) = match n.kind {
                    NpcKind::GoblinSorcerer => (assets.goblin_sorcerer.as_ref(), 3),
                    NpcKind::Tim => (assets.tim.as_ref(), 3),
                    _ => (assets.dark_caster.as_ref(), 6),
                };
                humanoid_draw(tex, count, n, w, h)
            }
            // The Fire Imp: the 10-frame strip (`Main.npcFrameCount[24]`).
            NpcKind::FireImp => humanoid_draw(assets.fire_imp.as_ref(), 10, n, w, h),
            // The spheres: the single-frame 16x16 sheets.
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                let tex = match n.kind {
                    NpcKind::ChaosBall => assets.chaos_ball.as_ref(),
                    NpcKind::BurningSphere => assets.burning_sphere.as_ref(),
                    _ => assets.water_sphere.as_ref(),
                };
                let (bw, bh) = n.size();
                (
                    tex,
                    tex.map_or(16.0, |t| t.h),
                    [bw + 4.0, bh],
                    [n.x - 2.0, n.y - 2.0],
                )
            }
            // The swimmers: the 6/6/4-frame strips (`Main.npcFrameCount`
            // 55/58/65, `Main.cs:67082`) over the hitbox.
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => {
                // The swimmers: `Main.npcFrameCount` 55/58 = 6, 65 = 4
                // (`Main.cs:67082`) — the sheets divide into 28-px rows
                // (NPC_55/58's 38x168) and 48-px rows (NPC_65's 120x192),
                // the frames wider than the boxes (38/100); the generic
                // anchor centres each frame on the box's centre-X with its
                // centre 4 px below the box's bottom (`Main.cs:25511-25516`).
                let (tex, count) = match n.kind {
                    NpcKind::Goldfish => (assets.goldfish.as_ref(), 6),
                    NpcKind::Piranha => (assets.piranha.as_ref(), 6),
                    _ => (assets.shark.as_ref(), 4),
                };
                let fh = tex.map_or(28.0, |t| t.h / count as f32);
                let fw = tex.map_or(38.0, |t| t.w);
                (
                    tex,
                    fh,
                    [fw, fh],
                    [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
                )
            }
            // The worms: the single-frame segments rotated along the chain
            // (`rotation = atan2 + 1.57`), drawn only in the open — the
            // vanilla `behindTiles` hides them under the ground (the
            // behindTiles layer lands with the render-layers work).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => {
                let tex = match n.kind {
                    NpcKind::DevourerHead => assets.devourer_head.as_ref(),
                    NpcKind::DevourerBody => assets.devourer_body.as_ref(),
                    NpcKind::DevourerTail => assets.devourer_tail.as_ref(),
                    NpcKind::GiantWormHead => assets.giant_worm_head.as_ref(),
                    NpcKind::GiantWormBody => assets.giant_worm_body.as_ref(),
                    NpcKind::GiantWormTail => assets.giant_worm_tail.as_ref(),
                    NpcKind::EaterOfWorldsHead => assets.eow_head.as_ref(),
                    NpcKind::EaterOfWorldsBody => assets.eow_body.as_ref(),
                    _ => assets.eow_tail.as_ref(),
                };
                // The worms are single-frame (`Main.npcFrameCount` 7..15
                // = 1, `Main.cs:67082`): the sheets' frames — NPC_7/8/9
                // 30x48, NPC_10/11/12 18x30, NPC_13/14/15 46x90 — are
                // wider and taller than the segment boxes (22/14/38); the
                // generic anchor centres each frame on the box's centre-X
                // with its centre 4 px below the bottom
                // (`Main.cs:25511-25516`) — the v1 stretched every segment
                // to the box+4 shape. The segments' chain rotation
                // (`atan2 + 1.57`) rides the draw's rotation branch below.
                let fh = tex.map_or(48.0, |t| t.h);
                let fw = tex.map_or(30.0, |t| t.w);
                (
                    tex,
                    fh,
                    [fw, fh],
                    [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
                )
            }
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                // The hovers: `FindFrame`'s case 6 (`NPC.cs`, the 8-tick
                // advance over `Main.npcFrameCount` — 6 = 2, 42 = 3, 23 =
                // 2, `Main.cs:67082`) matches the port's machine; the
                // sheets' frames are NPC_6 42x80, NPC_42 48x40, NPC_23
                // 26x28 — wider than the boxes (30/34/22) — so the generic
                // anchor centres each frame on the box's centre-X with its
                // centre 4 px below the bottom (`Main.cs:25511-25516`); the
                // v1 stretched the art to the box+4 shape.
                let (tex, count) = match n.kind {
                    NpcKind::Hornet => (assets.hornet.as_ref(), 3),
                    NpcKind::MeteorHead => (assets.meteor_head.as_ref(), 2),
                    _ => (assets.eater_of_souls.as_ref(), 2),
                };
                let fh = tex.map_or(80.0, |t| t.h / count as f32);
                let fw = tex.map_or(42.0, |t| t.w);
                (
                    tex,
                    fh,
                    [fw, fh],
                    [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
                )
            }
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => {
                // The bats: `Main.npcFrameCount` 49/51/60/66 = 5
                // (`Main.cs:67082`); NPC_49/51/60 are 44x160 (5 rows of
                // 32), NPC_66 86x440 (5 rows of 88); the generic anchor
                // centres each frame on the box's centre-X with its centre
                // 4 px below the bottom (`Main.cs:25511-25516`) — the v1
                // read the row height off the cave bat for EVERY kind and
                // stretched a fixed 26x18 quad.
                let t = match n.kind {
                    NpcKind::JungleBat => assets.jungle_bat.as_ref(),
                    NpcKind::Hellbat => assets.hellbat.as_ref(),
                    NpcKind::VoodooDemon => assets.voodoo_demon.as_ref(),
                    _ => assets.cave_bat.as_ref(),
                };
                let fh = t.map_or(32.0, |t| t.h / 5.0);
                let fw = t.map_or(44.0, |t| t.w);
                (
                    t,
                    fh,
                    [fw, fh],
                    [n.x + w * 0.5 - fw * 0.5, n.y + h + 4.0 - fh * 0.5],
                )
            }
            NpcKind::KingSlime => (
                assets.king_slime.as_ref(),
                120.0,
                [174.0, 120.0],
                [n.x - 38.0, n.y - 25.0],
            ),
            // The Eye of Cthulhu: NPC_4 is 110x996 — six 166-px rows
            // (`Main.npcFrameCount[4]` = 6) whose frames are LARGER than
            // the 100x110 box; the v1 squeezed a 166-px row into a 110-px
            // quad at the box's corner. The boss trio (4/125/126) also
            // overrides `halfSize` to (55, 107) (`Main.cs:23051-23056`),
            // which lands the frame's centre 24 px lower than the
            // sheet-derived origin would (`107 - 166/2`).
            NpcKind::EyeOfCthulhu => boss_draw(assets.eye_of_cthulhu.as_ref(), 6, n, w, h),
            // The servant: `Main.npcFrameCount[5] = 2` frames of 20x20.
            NpcKind::ServantOfCthulhu => (
                assets.servant_of_cthulhu.as_ref(),
                assets
                    .servant_of_cthulhu
                    .as_ref()
                    .map_or(20.0, |t| t.h / 2.0),
                [20.0, 20.0],
                [n.x, n.y],
            ),
        };
        let Some(tex) = tex else {
            continue;
        };
        // `Main.cs:23048`'s per-kind anchor nudge (`num47` + `num46`): a
        // flat Y shift the C#'s generic draw (and the towers' dedicated
        // arm) add to the anchor. Applied once, here, for every kind.
        let pos = [pos[0], pos[1] + npc_anchor_nudge(n)];
        // `Main.DrawNPC` multiplies the light by `npc.color`; the slime
        // variants carry their `SetDefaults` colours (alpha included — the
        // premultiplied pipeline renders the translucency directly).
        // `NPC.GetAlpha`'s `(255 - alpha)/255` (`NPC.cs:93406-93412`)
        // rides every NPC: the Golem family spawns at `alpha = 255` and
        // fades in over 26 ticks (the others start at 0 — a no-op tint).
        let fade = 1.0 - n.alpha as f32 / 255.0;
        let base = if n.kind == NpcKind::Slime {
            let k = n.slime.stats().color;
            [
                c[0] * k[0] as f32 / 255.0 * fade,
                c[1] * k[1] as f32 / 255.0 * fade,
                c[2] * k[2] as f32 / 255.0 * fade,
                k[3] as f32 / 255.0 * fade,
            ]
        } else {
            [c[0] * fade, c[1] * fade, c[2] * fade, fade]
        };
        let tint = if n.hurt_cd > 0.0 {
            [
                c[0].max(0.8) * fade,
                c[1].max(0.8) * fade,
                c[2].max(0.8) * fade,
                fade,
            ]
        } else {
            base
        };
        // The Deerclops' sheet is a 5x5 grid, not a column: the cell
        // index in `npc.frame` decodes as column `cell / 5`, row
        // `cell % 5`, and the C#'s `spriteEffects ^ FlipHorizontally`
        // (Main.cs:26386) cancels the generic flip — the sheet faces
        // RIGHT by default, so the flip rides `facing < 0`. The martians
        // ride their own `spriteDirection` (`NPC.cs:6515`, read by
        // `Main.cs:23124-23128`): the Scutlix family's `FindFrame` inverts
        // it against `direction` (`NPC.cs:71746-71753`/`:71795`), so
        // `facing` cannot stand in for them.
        let flip_dir = if n.kind.flips_on_sprite_dir() {
            n.sprite_dir
        } else {
            n.facing
        };
        let uv = if n.kind == NpcKind::Deerclops {
            flip_uv(deerclops_frame_uv(tex, n.frame()), flip_dir < 0)
        } else {
            flip_uv(npc_frame_uv(tex, frame_h, n.frame()), flip_dir > 0)
        };
        if n.rotation != 0.0
            && matches!(
                n.kind,
                NpcKind::DemonEye
                    | NpcKind::WanderingEye
                    | NpcKind::GolemFistLeft
                    | NpcKind::GolemFistRight
                    | NpcKind::Goldfish
                    | NpcKind::Piranha
                    | NpcKind::Shark
                    | NpcKind::CaveBat
                    | NpcKind::JungleBat
                    | NpcKind::Hellbat
                    | NpcKind::VoodooDemon
                    | NpcKind::DevourerHead
                    | NpcKind::DevourerBody
                    | NpcKind::DevourerTail
                    | NpcKind::GiantWormHead
                    | NpcKind::GiantWormBody
                    | NpcKind::GiantWormTail
                    | NpcKind::EaterOfWorldsHead
                    | NpcKind::EaterOfWorldsBody
                    | NpcKind::EaterOfWorldsTail
                    // The saucer family (`Main.cs:24586-24612`) rotates
                    // about its CENTRE (the `origin = halfSize` draw point
                    // — the quad the anchors computed sits at that centre,
                    // the turret's 8 px lower), and the drone rides the
                    // velocity's folded rotation (`NPC.cs:35220-35229`).
                    | NpcKind::MartianSaucer
                    | NpcKind::MartianSaucerTurret
                    | NpcKind::MartianSaucerCannon
                    | NpcKind::MartianSaucerCore
                    | NpcKind::MartianDrone
                    // The Prime family (`AI_032`'s `rotation = velocity.X / 15`
                    // and its limbs' dock/aim rotations, `NPC.cs:27689`/
                    // `:28151`/`:28350`/`:28656`/`:28891`): the head rolls
                    // with its drift and the limbs aim their "up" along the
                    // dock or the player.
                    | NpcKind::SkeletronPrime
                    | NpcKind::PrimeCannon
                    | NpcKind::PrimeSaw
                    | NpcKind::PrimeVice
                    | NpcKind::PrimeLaser
            )
        {
            // `Main.DrawNPC` draws every NPC at its centre with
            // `origin = frame size / 2` and `rotation = npc.rotation`
            // (`NPC.cs` draw pass); the demon eye aims along its flight
            // (`NPC.cs:74019-74031`). The mirrored quad's corners are
            // symmetric about the centre, so flipping does not move them —
            // rotate the quad about its centre, exactly like XNA does.
            let center = snap_to_screen(
                if n.kind.is_martian() {
                    [n.x + w * 0.5, n.y + h * 0.5]
                } else {
                    [pos[0] + size[0] * 0.5, pos[1] + size[1] * 0.5]
                },
                camera,
                zoom,
                screen,
            );
            let (hw, hh) = (size[0] * 0.5, size[1] * 0.5);
            let (s, co) = n.rotation.sin_cos();
            let top_left = [center[0] - hw * co + hh * s, center[1] - hw * s - hh * co];
            r.push_sprite_rot(top_left, size, uv, tint, n.rotation);
        } else {
            r.push_sprite(snap_to_screen(pos, camera, zoom, screen), size, uv, tint);
        }
        // The Lunar pillars' glow ghosts (`Main.cs:24177-24233`): each
        // tower's dedicated arm draws FOUR masked copies of its own sheet
        // at `velocity.RotatedBy(i·π/2) · offset` (the offset 4 +
        // |light − 0.5|·4, the Solar's 2 + ·9 with the extra +X 2), tinted
        // by the vanilla's (64,64,64,0)·Opacity mask — the port's
        // premultiplied pipeline renders them with the glow sheets' own
        // alpha instead (the mask tint draws nothing there).
        if matches!(
            n.kind,
            NpcKind::StardustTower
                | NpcKind::NebulaTower
                | NpcKind::SolarTower
                | NpcKind::VortexTower
        ) {
            let gtex = match n.kind {
                NpcKind::StardustTower => assets.glow_stardust.as_ref(),
                NpcKind::NebulaTower => assets.glow_nebula.as_ref(),
                NpcKind::SolarTower => assets.glow_solar.as_ref(),
                _ => assets.glow_vortex.as_ref(),
            };
            if let Some(gtex) = gtex {
                let guv = npc_frame_uv(gtex, frame_h, n.frame());
                // The C# derives the offset from `npcColor`:
                // `4 + |light − 0.5| · 4` (the Solar's `2 + ·9`).
                let light_len =
                    (c[0] - 0.5).hypot(c[1] - 0.5).hypot(c[2] - 0.5);
                let base_off = if n.kind == NpcKind::SolarTower {
                    2.0 + light_len * 9.0
                } else {
                    4.0 + light_len * 4.0
                };
                for i in 0..4u32 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_2;
                    let (sx, sy) = a.sin_cos();
                    let ghost = [
                        pos[0] + (n.vx * sy - n.vy * sx) * base_off,
                        pos[1] + (n.vx * sx + n.vy * sy) * base_off
                            + if n.kind == NpcKind::SolarTower { 2.0 } else { 0.0 },
                    ];
                    r.push_sprite(snap_to_screen(ghost, camera, zoom, screen), size, guv, [
                        1.0, 1.0, 1.0, 1.0
                    ]);
                    quads += 1;
                }
            }
        }
        // The Prime family's glow masks (`Main.cs:25585-25642`): the head's
        // `Bone_Eyes` and the laser limb's `Bone_Laser` ride the body's
        // OWN frame/rotation/anchor, so the same transform draws them a
        // second time off the overlay sheets (the Mech-Queen's chain block
        // and the trail after-images stay the documented deviations).
        let glow = match n.kind {
            NpcKind::SkeletronPrime => assets.prime_eyes.as_ref(),
            NpcKind::PrimeLaser => assets.bone_laser.as_ref(),
            _ => None,
        };
        if let Some(gtex) = glow {
            let guv = npc_frame_uv(gtex, frame_h, n.frame());
            if n.rotation != 0.0 {
                let center = snap_to_screen(
                    if n.kind.is_martian() {
                        [n.x + w * 0.5, n.y + h * 0.5]
                    } else {
                        [pos[0] + size[0] * 0.5, pos[1] + size[1] * 0.5]
                    },
                    camera,
                    zoom,
                    screen,
                );
                let (hw, hh) = (size[0] * 0.5, size[1] * 0.5);
                let (s, co) = n.rotation.sin_cos();
                let top_left = [center[0] - hw * co + hh * s, center[1] - hw * s - hh * co];
                r.push_sprite_rot(top_left, size, guv, [1.0, 1.0, 1.0, 1.0], n.rotation);
            } else {
                r.push_sprite(snap_to_screen(pos, camera, zoom, screen), size, guv, [
                    1.0, 1.0, 1.0, 1.0
                ]);
            }
            quads += 1;
        }
        quads += 1;
        if n.hp < n.max_hp {
            let frac = (n.hp as f32 / n.max_hp as f32).clamp(0.0, 1.0);
            r.push_sprite([n.x, n.y - 9.0], [w, 4.0], white, [0.02, 0.02, 0.02, 0.75]);
            r.push_sprite(
                [n.x, n.y - 9.0],
                [w * frac, 4.0],
                white,
                [0.85, 0.15, 0.15, 0.95],
            );
            quads += 2;
        }
    }
    quads
}


#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// A bare NPC at the spawn point (the port's test helper).
    fn bare(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 1, 0.0)
    }

    /// `Main.cs:23048`'s per-kind anchor nudge (`num47`): the Plantera head's
    /// 26, the Brain's `50 * scale`, and the DD2 group's negative shifts.
    #[test]
    fn the_anchor_nudges_follow_the_c_sharp_switch() {
        let mut n = bare(NpcKind::Plantera);
        n.scale = 1.0;
        assert_eq!(npc_anchor_nudge(&n), 26.0, "the Plantera head's num47");
        let mut b = bare(NpcKind::BrainOfCthulhu);
        b.scale = 0.5;
        assert_eq!(npc_anchor_nudge(&b), 25.0, "the Brain's 50 * scale");
        for kind in [NpcKind::DD2JavelinT1, NpcKind::DD2JavelinT2] {
            assert_eq!(npc_anchor_nudge(&bare(kind)), -8.0, "{kind:?}");
        }
        for kind in [
            NpcKind::DD2GoblinBomberT1,
            NpcKind::DD2GoblinBomberT2,
            NpcKind::DD2GoblinBomberT3,
        ] {
            assert_eq!(npc_anchor_nudge(&bare(kind)), -5.0, "{kind:?}");
        }
        assert_eq!(npc_anchor_nudge(&bare(NpcKind::DD2WitherBeast)), -3.0);
        // The unoverridden kinds stay at 0 (the towers/saucers/Deerclops
        // compute their own anchors).
        for kind in [NpcKind::Slime, NpcKind::MartianSaucer, NpcKind::Deerclops] {
            assert_eq!(npc_anchor_nudge(&bare(kind)), 0.0, "{kind:?}");
        }
    }

    /// The `halfSize` override's effect on the anchor: the EoC's 107 over its
    /// 166-px row's 83 pushes the frame 24 px lower; the Twins' 107 over the
    /// 200-px row's 100 pushes it 7; the Plantera head's absolute 77 over its
    /// 86-px row's 43 pushes it 34.
    #[test]
    fn the_half_size_override_shifts_the_anchor() {
        for (kind, fh, want) in [
            (NpcKind::EyeOfCthulhu, 166.0, 24.0),
            (NpcKind::Retinazer, 200.0, 7.0),
            (NpcKind::Spazmatism, 200.0, 7.0),
            (NpcKind::Plantera, 86.0, 34.0),
            (NpcKind::Slime, 48.0, 0.0),
        ] {
            assert_eq!(
                boss_half_size(kind, fh) - fh * 0.5,
                want,
                "{kind:?}'s halfSize shift"
            );
        }
    }
}
