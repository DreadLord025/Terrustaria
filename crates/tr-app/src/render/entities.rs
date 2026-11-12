//! Entity drawing: the player/NPC texture atlases and frame-UV helpers, the
//! world-space player and NPC passes, the held-item swing, the fullscreen-map
//! head icon and the UI-space character preview.
use super::BgTex;
use super::player_layers::{
    CompositeArmState, LayerSprite, PlayerCompositeState, assemble_layers, front_arm_cell, hair_uv,
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
    /// `Player_0_8`: the under-shirt sleeve on both arm stacks
    /// (`TextureAssets.Players[skinVar, 8]`, `PlayerDrawLayers.cs:1357` and
    /// `:3698`, tinted `colorUnderShirt`).
    pub(crate) under_arm: Option<BgTex>,
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
            under_arm: None,
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
        // Torso and shirt sheets are 9 columns wide (`Player_0_3`/`_0_6` are
        // 360x224 = 9x4 cells); keep the two columns `torso_cell` can reach,
        // cell (0,0) plus the row-5 override's column 1 (`PlayerDrawSet.cs:
        // 1940,1975`). The old 40-px crop clamped the override back to 0.
        torso: sky_tex(atlas, "Player_0_3", Some((0, 0, 80, 224))),
        shirt: sky_tex(atlas, "Player_0_6", Some((0, 0, 80, 224))),
        arm: sky_tex(atlas, "Player_0_7", None),
        undershirt: sky_tex(atlas, "Player_0_5", None),
        under_arm: sky_tex(atlas, "Player_0_8", None),
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
    /// `NPC_108.xnb` — the Wizard's town sheet (23 rows, `Main.cs:67094`).
    pub(crate) wizard: Option<BgTex>,
    /// `NPC_106.xnb` — the bound Wizard's single-row sheet (`:67094`).
    pub(crate) bound_wizard: Option<BgTex>,
    /// `NPC_124.xnb` — the Mechanic's town sheet (23 rows, `Main.cs:67096`).
    pub(crate) mechanic: Option<BgTex>,
    /// `NPC_123.xnb` — the bound Mechanic's single-row sheet (`:67096`).
    pub(crate) bound_mechanic: Option<BgTex>,
    /// `NPC_353.xnb` — the Stylist's town sheet (23 rows, `Main.cs:67119`).
    pub(crate) stylist: Option<BgTex>,
    /// `NPC_354.xnb` — the webbed Stylist's single-row sheet (`:67119`).
    pub(crate) webbed_stylist: Option<BgTex>,
    /// `NPC_441.xnb` — the Tax Collector's town sheet (25 rows, `:67128`).
    pub(crate) tax_collector: Option<BgTex>,
    /// `NPC_588.xnb` — the Golfer's town sheet (25 rows, `Main.cs:67142`).
    pub(crate) golfer: Option<BgTex>,
    /// `NPC_589.xnb` — the sleeping Golfer's sheet (`:67142`).
    pub(crate) sleeping_golfer: Option<BgTex>,
    /// `NPC_633.xnb` — the Zoologist's town sheet (23 rows, `Main.cs:67147`).
    pub(crate) zoologist: Option<BgTex>,
    /// `NPC_663.xnb` — the Princess's town sheet (`:67152`).
    pub(crate) princess: Option<BgTex>,
    /// `NPC_207.xnb` — the Dye Trader's town sheet (25 rows, `Main.cs:67120`).
    pub(crate) dye_trader: Option<BgTex>,
    /// `NPC_208.xnb` — the Party Girl's town sheet (23 rows, `:67120`).
    pub(crate) party_girl: Option<BgTex>,
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
    /// The pillar/Moon-Lord families' per-kind sheets, keyed by the vanilla
    /// net id: `TextureAssets.Npc[type]` is one sheet PER KIND, and the v1
    /// shared one sheet across a whole family (see [`Self::npc_sheet`]).
    pub(crate) npc_sheets: std::collections::HashMap<u16, BgTex>,
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
    /// `NPC_552/553/554` — the DD2 goblin attackers' FIRST tier sheet.
    /// The v1 read `NPC_553` for all three tiers AND for the bombers and
    /// the javelin throwers (whose sheets are `NPC_555-557` and
    /// `NPC_561-563`), so two thirds of the Old One's Army's goblins drew
    /// the wrong art: `NPC_553` is the tier-2 goblin (58/66/66 px wide,
    /// 14 rows). Each kind now loads its own sheet through the
    /// `dd2_goblin_sheet` lookup below.
    pub(crate) dd2_goblins: Option<BgTex>,
    /// `NPC_555/556/557` — the DD2 goblin bombers (17-frame strips of
    /// 46x58: `Main.npcFrameCount[555..557] = 17`, `Main.cs:67082`).
    pub(crate) dd2_bombers: Option<BgTex>,
    /// `NPC_561/562/563` — the DD2 javelin throwers (14-frame strips of
    /// 84x76).
    pub(crate) dd2_javelins: Option<BgTex>,
    /// `NPC_552/554` — the DD2 goblin tiers 1 and 3 (their own sheets).
    pub(crate) dd2_goblin_t1: Option<BgTex>,
    pub(crate) dd2_goblin_t3: Option<BgTex>,
    /// `NPC_566/567` — the DD2 skeletons' materialise sheets (9 and 9
    /// frames of 48x52 — `Main.npcFrameCount[566] = [567] = 9`).
    pub(crate) dd2_skeletons: Option<BgTex>,
    /// `NPC_564` — the Dark Mage (20-frame caster strip of 34x62).
    pub(crate) dark_mage: Option<BgTex>,
    /// `NPC_551` — Betsy (11-frame dragon strip of 190x90).
    pub(crate) betsy: Option<BgTex>,
    /// `NPC_568` — the Wither Beast (14-frame humanoid strip of 18x40).
    pub(crate) wither_beast: Option<BgTex>,
    /// `NPC_558/559/560` — the etherian wyverns (5-frame strips of 38x38).
    pub(crate) dd2_wyverns: Option<BgTex>,
    /// `NPC_574/575` — the kobold gliders (10-frame strips of 38x38).
    pub(crate) dd2_gliders: Option<BgTex>,
    /// `NPC_565` — the Dark Mage T3 (the T1's 20-frame 34x62 strip).
    pub(crate) dark_mage_t3: Option<BgTex>,
    /// `NPC_569` — the Wither Beast T3 (17-frame humanoid strip of 18x40).
    pub(crate) wither_beast_t3: Option<BgTex>,
    /// `NPC_570/571` — the drakins (16-frame strips of 42x58).
    pub(crate) dd2_drakins: Option<BgTex>,
    /// `NPC_572/573` — the kobold walkers (18-frame strips of 18x40).
    pub(crate) dd2_kobolds: Option<BgTex>,
    /// `NPC_576/577` — the ogres (10-frame strips of 96x124).
    pub(crate) dd2_ogres: Option<BgTex>,
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
    /// `TextureAssets.Extra[13]` — the Moon Lord core's two torso halves
    /// (drawn at the core's Centre from origins `(w, 278)` and `(0, 278)`,
    /// `Main.cs:24754-24755`).
    pub(crate) ml_torso: Option<BgTex>,
    /// `TextureAssets.Extra[16]` — the core's inner plate (origin `(112, 101)`,
    /// `Main.cs:24756`).
    pub(crate) ml_core_plate: Option<BgTex>,
    /// `TextureAssets.Extra[17]` — the hand's base plate drawn at the hand's
    /// Centre from the origin `(26, 42)` mirrored per side
    /// (`Main.cs:24797-24816`).
    pub(crate) ml_hand_base: Option<BgTex>,
    /// `TextureAssets.Extra[19]` — the Moon Lord's eye sprite the hand and the
    /// free eye both draw (`Main.cs:24823`, `:24960`).
    pub(crate) ml_eye: Option<BgTex>,
    pub(crate) glow_stardust: Option<BgTex>,
    pub(crate) glow_nebula: Option<BgTex>,
    pub(crate) glow_vortex: Option<BgTex>,
    pub(crate) glow_solar: Option<BgTex>,
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

impl NpcAssets {
    /// `TextureAssets.Npc[type]` for the pillar/Moon-Lord families — the
    /// sheet a kind draws from, by its vanilla net id.
    pub(crate) fn npc_sheet(&self, id: u16) -> Option<&BgTex> {
        self.npc_sheets.get(&id)
    }
}

/// `sky_tex` for the NPC pass, with the sheet-packing audit folded in: asking
/// the atlas for a name `load_static_sheets` never packed yields a silent
/// no-draw (the M4-NPC-3v bug), so debug builds flag it outright.
fn npc_tex(atlas: &Atlas, name: &str) -> Option<BgTex> {
    debug_assert!(
        crate::assets::STATIC_SHEET_NAMES.contains(&name),
        "{name} is asked for by the NPC pass but never packed"
    );
    sky_tex(atlas, name, None)
}

pub(crate) fn make_npc_assets(atlas: &Atlas) -> NpcAssets {
    NpcAssets {
        slime: npc_tex(atlas, "NPC_1"),
        eye: npc_tex(atlas, "NPC_2"),
        wandering_eye: npc_tex(atlas, "NPC_133"),
        zombie: npc_tex(atlas, "NPC_3"),
        guide: npc_tex(atlas, "NPC_22"),
        merchant: npc_tex(atlas, "NPC_17"),
        dryad: npc_tex(atlas, "NPC_20"),
        nurse: npc_tex(atlas, "NPC_18"),
        arms_dealer: npc_tex(atlas, "NPC_19"),
        demolitionist: npc_tex(atlas, "NPC_38"),
        painter: npc_tex(atlas, "NPC_227"),
        witch_doctor: npc_tex(atlas, "NPC_228"),
        clothier: npc_tex(atlas, "NPC_54"),
        travelling_merchant: npc_tex(atlas, "NPC_368"),
        angler: npc_tex(atlas, "NPC_369"),
        wizard: npc_tex(atlas, "NPC_108"),
        bound_wizard: npc_tex(atlas, "NPC_106"),
        mechanic: npc_tex(atlas, "NPC_124"),
        bound_mechanic: npc_tex(atlas, "NPC_123"),
        stylist: npc_tex(atlas, "NPC_353"),
        webbed_stylist: npc_tex(atlas, "NPC_354"),
        tax_collector: npc_tex(atlas, "NPC_441"),
        golfer: npc_tex(atlas, "NPC_588"),
        sleeping_golfer: npc_tex(atlas, "NPC_589"),
        zoologist: npc_tex(atlas, "NPC_633"),
        princess: npc_tex(atlas, "NPC_663"),
        dye_trader: npc_tex(atlas, "NPC_207"),
        party_girl: npc_tex(atlas, "NPC_208"),
        sleeping_angler: npc_tex(atlas, "NPC_376"),
        eoc: npc_tex(atlas, "NPC_4"),
        queen_bee: npc_tex(atlas, "NPC_222"),
        skeletron_head: npc_tex(atlas, "NPC_35"),
        wof: npc_tex(atlas, "NPC_113"),
        destroyer: npc_tex(atlas, "NPC_134"),
        prime: npc_tex(atlas, "NPC_127"),
        retinazer: npc_tex(atlas, "NPC_125"),
        spazmatism: npc_tex(atlas, "NPC_126"),
        plantera: npc_tex(atlas, "NPC_262"),
        brain: npc_tex(atlas, "NPC_266"),
        creeper: npc_tex(atlas, "NPC_267"),
        bound_goblin: npc_tex(atlas, "NPC_105"),
        goblin_tinkerer: npc_tex(atlas, "NPC_107"),
        golem: npc_tex(atlas, "NPC_245"),
        golem_head: npc_tex(atlas, "NPC_246"),
        golem_fist: npc_tex(atlas, "NPC_247"),
        golem_head_free: npc_tex(atlas, "NPC_249"),
        deerclops: npc_tex(atlas, "NPC_668"),
        skeletron_hand: npc_tex(atlas, "NPC_36"),
        wof_eye: npc_tex(atlas, "NPC_114"),
        prime_cannon: npc_tex(atlas, "NPC_128"),
        prime_saw: npc_tex(atlas, "NPC_129"),
        prime_vice: npc_tex(atlas, "NPC_130"),
        prime_laser: npc_tex(atlas, "NPC_131"),
        npc_sheets: PILLAR_SHEETS
            .iter()
            .filter_map(|&id| {
                npc_tex(atlas, &format!("NPC_{id}")).map(|t| (id as u16, t))
            })
            .collect(),
        prime_eyes: npc_tex(atlas, "Bone_Eyes"),
        bone_laser: npc_tex(atlas, "Bone_Laser"),
        skeleton: npc_tex(atlas, "NPC_21"),
        undead_miner: npc_tex(atlas, "NPC_44"),
        cave_bat: npc_tex(atlas, "NPC_49"),
        jungle_bat: npc_tex(atlas, "NPC_51"),
        hellbat: npc_tex(atlas, "NPC_60"),
        voodoo_demon: npc_tex(atlas, "NPC_66"),
        eater_of_souls: npc_tex(atlas, "NPC_6"),
        hornet: npc_tex(atlas, "NPC_42"),
        meteor_head: npc_tex(atlas, "NPC_23"),
        goblin_sorcerer: npc_tex(atlas, "NPC_29"),
        snowman_gangsta: npc_tex(atlas, "NPC_143"),
        snow_balla: npc_tex(atlas, "NPC_144"),
        mystery_snowman: npc_tex(atlas, "NPC_145"),
        martian_officer: npc_tex(atlas, "NPC_383"),
        gray_grunt: npc_tex(atlas, "NPC_385"),
        martian_engineer: npc_tex(atlas, "NPC_386"),
        gigazapper: npc_tex(atlas, "NPC_389"),
        martian_walker_big: npc_tex(atlas, "NPC_520"),
        martian_probe: npc_tex(atlas, "NPC_399"),
        martian_drone: npc_tex(atlas, "NPC_388"),
        brain_scrambler: npc_tex(atlas, "NPC_381"),
        ray_gunner: npc_tex(atlas, "NPC_382"),
        scutlix_rider: npc_tex(atlas, "NPC_390"),
        scutlix: npc_tex(atlas, "NPC_391"),
        saucer: npc_tex(atlas, "NPC_392"),
        saucer_turret: npc_tex(atlas, "NPC_393"),
        saucer_cannon: npc_tex(atlas, "NPC_394"),
        saucer_core: npc_tex(atlas, "NPC_395"),
        dutchman: npc_tex(atlas, "NPC_491"),
        dutchman_cannon: npc_tex(atlas, "NPC_492"),
        dd2_goblins: npc_tex(atlas, "NPC_553"),
        dd2_bombers: npc_tex(atlas, "NPC_555"),
        dd2_javelins: npc_tex(atlas, "NPC_561"),
        dd2_goblin_t1: npc_tex(atlas, "NPC_552"),
        dd2_goblin_t3: npc_tex(atlas, "NPC_554"),
        dd2_skeletons: npc_tex(atlas, "NPC_566"),
        dark_mage: npc_tex(atlas, "NPC_564"),
        betsy: npc_tex(atlas, "NPC_551"),
        wither_beast: npc_tex(atlas, "NPC_568"),
        dd2_wyverns: npc_tex(atlas, "NPC_558"),
        dd2_gliders: npc_tex(atlas, "NPC_574"),
        dark_mage_t3: npc_tex(atlas, "NPC_565"),
        wither_beast_t3: npc_tex(atlas, "NPC_569"),
        dd2_drakins: npc_tex(atlas, "NPC_570"),
        dd2_kobolds: npc_tex(atlas, "NPC_572"),
        dd2_ogres: npc_tex(atlas, "NPC_576"),
        tavern: npc_tex(atlas, "NPC_550"),
        crystal: npc_tex(atlas, "NPC_548"),
        cultist: npc_tex(atlas, "NPC_422"),
        tower_stardust: npc_tex(atlas, "NPC_493"),
        tower_nebula: npc_tex(atlas, "NPC_507"),
        tower_solar: npc_tex(atlas, "NPC_517"),
        tower_vortex: npc_tex(atlas, "NPC_422"),
        ml_torso: npc_tex(atlas, ML_TORSO_SHEET),
        ml_core_plate: npc_tex(atlas, ML_CORE_PLATE_SHEET),
        ml_hand_base: npc_tex(atlas, ML_HAND_BASE_SHEET),
        ml_eye: npc_tex(atlas, ML_EYE_SHEET),
        glow_stardust: npc_tex(atlas, "Glow_132"),
        glow_nebula: npc_tex(atlas, "Glow_143"),
        glow_vortex: npc_tex(atlas, "Glow_149"),
        glow_solar: npc_tex(atlas, "Glow_162"),
        pirates: npc_tex(atlas, "NPC_212"),
        pirate_captain: npc_tex(atlas, "NPC_216"),
        goblin_thief: npc_tex(atlas, "NPC_27"),
        goblin_warrior: npc_tex(atlas, "NPC_28"),
        goblin_archer: npc_tex(atlas, "NPC_111"),
        goblin_peon: npc_tex(atlas, "NPC_26"),
        goblin_summoner: npc_tex(atlas, "NPC_471"),
        tim: npc_tex(atlas, "NPC_45"),
        dark_caster: npc_tex(atlas, "NPC_32"),
        fire_imp: npc_tex(atlas, "NPC_24"),
        chaos_ball: npc_tex(atlas, "NPC_30"),
        burning_sphere: npc_tex(atlas, "NPC_25"),
        water_sphere: npc_tex(atlas, "NPC_33"),
        goldfish: npc_tex(atlas, "NPC_55"),
        piranha: npc_tex(atlas, "NPC_58"),
        shark: npc_tex(atlas, "NPC_65"),
        devourer_head: npc_tex(atlas, "NPC_7"),
        devourer_body: npc_tex(atlas, "NPC_8"),
        devourer_tail: npc_tex(atlas, "NPC_9"),
        giant_worm_head: npc_tex(atlas, "NPC_10"),
        giant_worm_body: npc_tex(atlas, "NPC_11"),
        giant_worm_tail: npc_tex(atlas, "NPC_12"),
        eow_head: npc_tex(atlas, "NPC_13"),
        eow_body: npc_tex(atlas, "NPC_14"),
        eow_tail: npc_tex(atlas, "NPC_15"),
        king_slime: npc_tex(atlas, "NPC_50"),
        eye_of_cthulhu: npc_tex(atlas, "NPC_4"),
        servant_of_cthulhu: npc_tex(atlas, "NPC_5"),
        hungry: npc_tex(atlas, "NPC_115"),
        leech: npc_tex(atlas, "NPC_117"),
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
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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
    let fh = tex.map_or(40.0, |t| vanilla_row_h(t, count));
    let fw = tex.map_or(18.0, |t| t.w);
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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

/// `Main.NPCAddHeight` (`Main.cs:22002-22184`) verbatim: the per-kind
/// pixel shift the C# adds to `num47` — the OTHER half of the NPC anchor
/// that the port never carried (BUG-07's anchor sweep). The C# reads the
/// NET id, not the enum, so the three ids the port reuses under a second
/// kind (`108` = the Wizard AND the DD2 Wither Beast — the C# has 568
/// there; `124` = the Mechanic AND the Moon Lord Core — the C# has 124;
/// `64` = the Goblin Tinkerer AND the Golem Head) are resolved by rank:
/// a town NPC never takes the monster row. Some kinds' rows are absent
/// because the port has no kind for them (the snowman gangsta's 48, the
/// celestial pillars' 141-145, the AoW's 621-623).
fn npc_add_height(n: &Npc) -> f32 {
    use NpcKind::*;
    // `n.townNPC` in the C# — the port's `is_town_npc` list is narrower
    // than the vanilla's (it omits the Mechanic/the Wizard/the Tavernkeep
    // and the rest of the post-1.1 arrivals), so the vanilla's own
    // `townNPC = true` family is added here rather than in the enum
    // helper the housing/town machine reads.
    let town = n.kind.is_town_npc()
        || matches!(
            n.kind,
            Mechanic
                | BoundMechanic
                | Wizard
                | BoundWizard
                | Stylist
                | WebbedStylist
                | Angler
                | SleepingAngler
                | TaxCollector
                | Golfer
                | SleepingGolfer
                | Zoologist
                | Princess
                | DyeTrader
                | PartyGirl
                | TravellingMerchant
        );
    let v = match n.kind {
        // `:22011` — 125/126 (the Twins) 30, `:22034` 126 again.
        Retinazer | Spazmatism => 30.0,
        // `:22015` 54 (the Clothier).
        Clothier => 2.0,
        // `:22019` 182 (the Angry Nimbus — no port kind).
        // `:22023` 178 (the Vortex Pillar's slot in the C# is 422; 178 is
        // the Crawltipede — no port kind).
        // `:22031` 6/173 (the Eater of Souls / the Meteor Head).
        EaterOfSouls | MeteorHead => 26.0,
        // `:22035` 94 (the Goblin Archer).
        GoblinArcher => 14.0,
        // `:22039` 7/8/9 (the Devourer's segments + the Zombie).
        DevourerHead | DevourerBody | DevourerTail | Zombie => 13.0,
        // `:22043` 98/99/100 (the DD2 bombers).
        DD2GoblinBomberT1 | DD2GoblinBomberT2 | DD2GoblinBomberT3 => 13.0,
        // `:22049` 10/11/12 (the Giant Worm's segments).
        GiantWormHead | GiantWormBody | GiantWormTail => 8.0,
        // `:22055` 13/14/15 (the Eater of Worlds' segments).
        EaterOfWorldsHead | EaterOfWorldsBody | EaterOfWorldsTail => 26.0,
        // `:22059` 175 (the Sea Snail — no port kind).
        // `:22063` 520 (the Martian Walker).
        MartianWalker => 2.0,
        // `:22067` 412-414 (the DD2 Dark Mage / T3 / the Betsy — the port's
        // ids 564/565 are the mage, 551 the Betsy; the C# range is the
        // mage trio's ORIGINAL ids 562-564, see `NPCID`).
        DD2DarkMage | DD2DarkMageT3 => 18.0,
        // `:22075` 49/51 (the cave/jungle bats — not the hellbat's 60).
        CaveBat | JungleBat => 4.0,
        // `:22079` 60 (the Hellbat).
        Hellbat => 10.0,
        // `:22083` 62/66/156 (the Demon Eye's 62 is a charm id; 66 the
        // Voodoo Demon, 156 the Wandering Eye — no port kind for 156).
        VoodooDemon => 14.0,
        // `:22087` 63/64/103 (the goblin townsfolk + the javelin T3) — the
        // Golem Head's 64 is resolved out by the town test above.
        BoundGoblin | GoblinTinkerer | DD2JavelinT3 => 4.0,
        // `:22091` 65 (the Shark).
        Shark => 5.0,
        // `:22095` 69 (the Gray Grunt).
        GrayGrunt => 4.0,
        // `:22099` 70 (the Skeletron) and `:22103` 72 (the Wall of Flesh).
        SkeletronHead => -4.0,
        WallOfFlesh => -2.0,
        // `:22107` 83/84 (the Skeletron's hands + the other hand id).
        SkeletronHand => 20.0,
        // `:22111` 150/151/158 (the Blue/Goldfish variants — the port's
        // single 55 Goldfish keeps the vanilla 65 row), `:22125` 87-92 (the
        // Goldfish school) 56.
        Goldfish | Piranha => 56.0,
        // `:22129` 134-136 (the Destroyer's segments).
        TheDestroyer => 30.0,
        // `:22133` 169 (the DD2 Kobold Flyer — the port's 572-575).
        DD2KoboldFlyerT2 | DD2KoboldFlyerT3 => 8.0,
        // `:22137` 174 (the DD2 Drakin).
        DD2DrakinT2 | DD2DrakinT3 => 6.0,
        // `:22141` 369 (the Angler) and `:22145` 376 (the Sleeping Angler).
        Angler | SleepingAngler => 2.0,
        // `:22149` 579 (the Bouncy Slime — no port kind), `:22153` 612/613
        // (the T3 goblins' rig) 2.
        // The port's DD2 Skeleton T3 wears the same 612/613 rig row; the
        // T2/T3 javelin pair rides 581/582 (see `npc_anchor_nudge`).
        DD2GoblinT3 | DD2SkeletonT3 => 2.0,
        // `:22169` 269/270/271/274/277 — the Painted/vanity town ids' -2.
        // `:22174` the `townNPC && ai[0] == 5` sitting row: -4 as a flat
        // shift, -6 for the Mechanic's 124 and -14 for the town slimes.
        _ => 0.0,
    };
    // `:22174`: the `townNPC && ai[0] == 5` sitting row — a `num -= 4`
    // that the sitting ladder then deepens for the 124 (the Mechanic) and
    // the town slimes.
    let mut shift = v;
    if town && n.ai[0] == 5.0 {
        shift -= 4.0;
        if matches!(n.kind, Mechanic | BoundMechanic) {
            shift -= 2.0;
        }
        if n.kind == DyeTrader {
            shift -= 2.0;
        }
    }
    shift * n.scale
}

/// `Main.cs:22992-22994`'s three anchor terms for the generic NPC draw:
/// `halfSize` (the `Main.cs:25453` sheet-derived frame centre, overridden
/// per kind by the `:23048` switch), `halfSize.Y += …` (`:23070`/`:23074`)
/// and a dedicated `halfSize.Y = …` (`:23077`, `:23082`).
///
/// Returned as the single vertical shift the C#'s formula
/// `pos.Y + height + 4 + halfSize.Y + num47 + num46` produces relative to
/// the port's old sheet-centred `bottom + 4 - frame_h/2` anchor — i.e.
/// `(height + 4 + halfSize.Y + num47 + num46) - frame_h/2`.
fn boss_half_size(kind: NpcKind, frame_h: f32) -> f32 {
    match kind {
        // `:23051-23056` — the boss trio's `halfSize = (55, 107)`.
        NpcKind::EyeOfCthulhu | NpcKind::Retinazer | NpcKind::Spazmatism => 107.0,
        // `:23077` — the Plantera head's absolute 77.
        NpcKind::Plantera => 77.0,
        // `:23082` — the Golem head's absolute 21.
        NpcKind::GolemHead => 21.0,
        // `:23070` — the `halfSize.Y += 4` group (the goblin townsfolk and
        // the javelin T3, whose frame rows are 56 px tall like every
        // humanoid: the shift lifts the art 4 px off the ground so the
        // spawn animation clears it).
        NpcKind::BoundGoblin | NpcKind::GoblinTinkerer | NpcKind::DD2JavelinT3 => {
            frame_h * 0.5 + 4.0
        }
        // `:23074` — the Gray Grunt's `halfSize.Y += 8`.
        NpcKind::GrayGrunt => frame_h * 0.5 + 8.0,
        // `:23060` — the moon/slug pair's `num46` (`626`/`627` are absent
        // from the port) and `692`'s wet switch (`:23066`): `num46` 10 dry /
        // 6 wet, folded into `npc_anchor_nudge` because it is a `num46`
        // term rather than a `halfSize` one.
        _ => frame_h * 0.5,
    }
}

/// `Main.cs:22991-23115`'s `num45` / `num46` / `num47` terms for the
/// generic NPC draw. `num47` starts at [`npc_add_height`], the `:22997`
/// chain overwrites it or adds `num46` for a dozen ids, and the
/// `:23048` switch adds the last few. Only the ids the port models are
/// listed; the rest of the switch (the DD2 support ring effects, the
/// `694`-only offsets, the `469`/`357`/`467` ids) has no port kind.
fn npc_anchor_nudge(n: &Npc) -> f32 {
    use NpcKind::*;
    let mut num46 = 0.0;
    // `float num47 = Main.NPCAddHeight(rCurrentNPC);` — the function
    // multiplies by `scale` ITSELF (`:22184`), and the `:23048` switch then
    // adds its own `50 * scale` terms.
    let mut num47 = npc_add_height(n);
    let mut num45 = 0.0;
    match n.kind {
        // The vanilla's 124 — the port's `Mechanic` kind — at `:23000`.
        Mechanic | BoundMechanic => num46 = 2.0,
        // `:23026` — the id 477 group's `num47 = 22` (an absolute, not an
        // add: the DD2 Ogre's sheet is far taller than its box).
        DD2OgreT2 | DD2OgreT3 => num47 = 22.0,
        // `:23022` — 483's `num46 = 14` (the DD2 Kobold Walker).
        DD2KoboldWalkerT2 | DD2KoboldWalkerT3 => num46 = 14.0,
        // `:23020` — 484's `num46 = 2` (the DD2 Wither Beast T3's rig).
        DD2WitherBeastT3 => num46 = 2.0,
        // `:23014` — 581's `num46 = -6` (the port's DD2 Javelin T2). 581 is
        // NOT in the `:23214` support group's overwrite list, so the -6
        // stands alone.
        DD2JavelinT2 => num46 = -6.0,
        // `:23038` — 612/613's `num46 -= 2` (the DD2 goblin T3 riders); the
        // port's T1/T2 pair is its stand-in.
        DD2GoblinT1 | DD2GoblinT2 => num46 = -2.0,
        // `:23106` — 688's `num46 += 10`.
        DD2GoblinT1 => num46 += 10.0,
        // `:23077-23091` — the `num47` adds of the same switch.
        Plantera => num47 += 26.0,
        BrainOfCthulhu => num47 += 50.0 * n.scale,
        // `:23086` — the Golem head's `num47 += 16`.
        GolemHead | GolemHeadFree => num47 += 16.0,
        // `:23214-23243` — the DD2 support group's `num47` OVERWRITES: the
        // javelineers' -8, the bombers' -5, the kobold walkers' -4 and the
        // Wither Beasts' -3. (The drakins' -2 and the goblin attackers' own
        // rows are absent from the port's `NPCAddHeight` list.)
        DD2JavelinT1 | DD2JavelinT2 => num47 = -8.0,
        DD2GoblinBomberT1 | DD2GoblinBomberT2 | DD2GoblinBomberT3 => num47 = -5.0,
        DD2KoboldWalkerT2 => num47 = -4.0,
        DD2WitherBeast => num47 = -3.0,
        _ => {}
    }
    num45 + num46 + num47
}

/// The generic NPC anchor (`Main.cs:25453-25514`) for a sheet-only draw:
/// the frame is `t.w x (t.h / count)` pixels and XNA draws it with
/// `origin = halfSize` at
/// `position + (width/2 - W/2 + halfSize.X, height - H/count + 4 +
/// halfSize.Y + num47 + num46)`, which collapses to the top-left
/// `[n.x + w/2 - W/2, n.y + h + 4 - H/count]` — the form every port draw
/// uses. The `halfSize`/`num45`/`num46`/`num47` overrides are applied once
/// in [`draw_npcs`] so this stays the shared seam.
/// `Main.npcFrameCount` (`Main.cs:67082-67131`) verbatim: the sheet's row
/// count per NET id, i.e. the divisor `NPC.SetFrameSize` uses
/// (`NPC.cs:90780-90800`) for the per-row frame height.
pub(crate) const NPC_FRAME_COUNT: [u32; 697] = [
    1, 2, 2, 3, 6, 2, 2, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 2, 25, 23, 25, 21, 15, 26, 2, 10, 1, 16, 16,
    16, 3, 1, 15, 6, 1, 3, 2, 2, 21, 25, 1, 1, 1,
    3, 3, 15, 3, 7, 7, 6, 5, 6, 5, 3, 3, 23, 6,
    3, 6, 6, 2, 5, 6, 5, 7, 7, 4, 5, 8, 1, 5,
    1, 2, 4, 16, 5, 4, 4, 15, 16, 16, 16, 2, 4, 6,
    6, 18, 16, 1, 1, 1, 1, 1, 1, 4, 3, 1, 1, 1,
    1, 1, 1, 5, 6, 7, 16, 1, 1, 25, 23, 12, 20, 21,
    1, 2, 2, 3, 6, 1, 1, 1, 15, 4, 11, 1, 23, 6,
    6, 6, 1, 2, 2, 1, 3, 4, 1, 2, 1, 4, 2, 1,
    15, 3, 25, 4, 5, 7, 3, 2, 12, 12, 4, 4, 4, 8,
    8, 13, 5, 6, 4, 15, 23, 3, 15, 8, 5, 4, 13, 15,
    12, 4, 14, 14, 3, 2, 5, 3, 2, 3, 23, 5, 14, 16,
    5, 2, 2, 12, 3, 3, 3, 3, 2, 2, 2, 2, 2, 7,
    14, 15, 16, 8, 3, 15, 15, 16, 2, 3, 20, 25, 23, 26,
    4, 4, 16, 16, 20, 20, 20, 2, 2, 2, 2, 8, 12, 3,
    4, 2, 4, 25, 26, 26, 6, 3, 3, 3, 3, 3, 5, 4,
    4, 5, 4, 6, 7, 15, 4, 7, 6, 1, 1, 2, 4, 3,
    5, 3, 3, 3, 4, 5, 6, 4, 2, 1, 8, 4, 4, 1,
    8, 1, 4, 15, 15, 15, 15, 15, 15, 16, 15, 15, 15, 15,
    15, 3, 3, 3, 3, 3, 3, 16, 3, 6, 12, 21, 21, 20,
    16, 15, 15, 5, 5, 6, 6, 5, 2, 7, 2, 6, 6, 6,
    6, 6, 15, 15, 15, 15, 15, 11, 4, 2, 2, 3, 3, 3,
    16, 15, 16, 10, 14, 12, 1, 10, 8, 3, 3, 2, 2, 2,
    2, 7, 15, 15, 15, 6, 3, 10, 10, 6, 9, 8, 9, 8,
    20, 10, 6, 23, 1, 4, 24, 2, 4, 6, 6, 13, 15, 15,
    15, 15, 4, 4, 26, 23, 8, 2, 4, 4, 4, 4, 2, 2,
    4, 12, 12, 9, 9, 9, 1, 9, 11, 2, 2, 9, 5, 6,
    4, 18, 8, 11, 1, 4, 5, 8, 4, 1, 1, 1, 1, 4,
    2, 5, 4, 11, 5, 11, 1, 1, 1, 10, 10, 15, 8, 17,
    6, 6, 1, 12, 12, 13, 15, 9, 5, 10, 7, 7, 7, 7,
    7, 7, 7, 4, 4, 16, 16, 25, 5, 7, 3, 13, 2, 6,
    2, 19, 19, 19, 20, 26, 3, 1, 1, 1, 1, 1, 16, 21,
    9, 16, 7, 6, 18, 13, 20, 12, 12, 20, 6, 14, 14, 14,
    14, 6, 1, 3, 25, 19, 20, 22, 2, 4, 4, 4, 11, 9,
    8, 1, 9, 1, 8, 8, 12, 12, 11, 11, 11, 11, 11, 11,
    11, 11, 11, 1, 6, 9, 1, 1, 1, 1, 1, 1, 4, 1,
    10, 1, 8, 4, 1, 5, 8, 8, 8, 8, 9, 9, 5, 4,
    8, 16, 8, 2, 3, 3, 6, 6, 7, 13, 4, 4, 4, 4,
    1, 1, 1, 8, 25, 11, 14, 14, 14, 17, 17, 17, 5, 5,
    5, 14, 14, 14, 9, 9, 9, 9, 17, 17, 16, 16, 18, 18,
    10, 10, 10, 10, 4, 1, 6, 9, 6, 4, 4, 4, 14, 4,
    25, 13, 3, 7, 6, 6, 1, 4, 4, 4, 4, 4, 4, 4,
    15, 15, 8, 8, 2, 6, 15, 15, 6, 13, 5, 5, 7, 5,
    14, 14, 4, 6, 21, 1, 1, 1, 11, 12, 6, 6, 17, 6,
    16, 21, 16, 23, 5, 16, 2, 28, 28, 6, 6, 6, 6, 6,
    6, 6, 7, 7, 7, 7, 7, 7, 7, 3, 4, 6, 27, 16,
    2, 2, 4, 3, 4, 23, 6, 1, 1, 2, 8, 8, 14, 6,
    6, 6, 6, 6, 2, 4, 14, 14, 14, 14, 14, 14, 14, 1,
    1, 13, 6, 13, 1, 3, 16, 3, 30, 3, 1,
];

/// `TextureAssets.Npc[]` index for a kind — the sheet the draw arms sample and
/// so the sheet whose row count the `FindFrame` machine must respect. For most
/// kinds this is `NpcKind::net_id`; the lunar families spell it out because
/// their bestiary rows were keyed on the art's first member, not the drawn
/// kind.
pub(crate) fn texture_net_id(kind: NpcKind) -> u16 {
    match kind {
        NpcKind::StardustWormHead => 402,
        NpcKind::StardustCellBig => 405,
        NpcKind::StardustCellSmall => 406,
        NpcKind::StardustJellyfishBig => 407,
        NpcKind::StardustSpiderBig => 409,
        NpcKind::StardustSpiderSmall => 410,
        NpcKind::StardustSoldier => 411,
        NpcKind::SolarCrawltipedeHead => 412,
        NpcKind::SolarCrawltipedeBody => 413,
        NpcKind::SolarCrawltipedeTail => 414,
        NpcKind::SolarDrakomire => 415,
        NpcKind::SolarDrakomireRider => 416,
        NpcKind::SolarSroller => 417,
        NpcKind::SolarCorite => 418,
        NpcKind::SolarSelenian => 419,
        NpcKind::NebulaFloater => 420,
        NpcKind::BrainSuckler => 421,
        NpcKind::NebulaBeast => 423,
        NpcKind::Predictor => 424,
        NpcKind::VortexRifleman => 425,
        NpcKind::VortexHunter => 426,
        NpcKind::VortexAlien => 427,
        NpcKind::VortexHeadcrab => 428,
        NpcKind::VortexSoldier => 429,
        NpcKind::SolarSpearman => 518,
        NpcKind::MoonLordHead => 396,
        NpcKind::MoonLordHand => 397,
        NpcKind::MoonLordCore => 398,
        NpcKind::MoonLordFreeEye => 400,
        NpcKind::MoonLordLeechBlob => 401,
        _ => kind.net_id() as u16,
    }
}

/// The vanilla row count for a net id, falling back to a single-frame strip.
pub(crate) fn npc_rows(net_id: u16) -> u32 {
    NPC_FRAME_COUNT.get(net_id as usize).copied().unwrap_or(1)
}

/// The net ids whose sheets live in [`NpcAssets::npc_sheets`]: the stardust/
/// solar nebula/vortex pillar families and the Moon Lord's parts. Each one is
/// its own `TextureAssets.Npc[type]` texture in the vanilla; the v1 loaded one
/// sheet per FAMILY, so every kind but the first drew a row of a foreign sheet.
/// The Moon Lord parts' extra overlay sheets (`Main.cs:24754-24756`,
/// `:24797-24816`, `:24823`). Named constants so the sheet-packing audit can
/// assert each one is in `assets::STATIC_SHEET_NAMES` — the M4-NPC-3v bug was
/// a draw arm asking `sky_tex` for a sheet the atlas never packed.
pub(crate) const ML_TORSO_SHEET: &str = "Extra_13";
pub(crate) const ML_CORE_PLATE_SHEET: &str = "Extra_16";
pub(crate) const ML_HAND_BASE_SHEET: &str = "Extra_17";
pub(crate) const ML_EYE_SHEET: &str = "Extra_19";

const PILLAR_SHEETS: &[u32] = &[
    396, 397, 398, 400, 401, // the Moon Lord's parts
    402, 403, 404, 405, 406, 407, // the stardust pillar's mobs
    409, 410, 411, 412, 413, 414, // the stardust spiders/soldier/worm
    415, 416, 417, 418, 419, // the solar pillar's mobs
    420, 421, 423, 424, 425, // the nebula pillar's mobs
    426, 427, 428, 429, // the vortex pillar's mobs
    518, // the solar spear-man
];

/// `NPC.SetFrameSize`'s frame height: `TextureAssets.Npc[type].Height() /
/// Main.npcFrameCount[type]` in INTEGER division (`NPC.cs:90780-90800`), so a
/// sheet whose height is not a multiple of its row count loses the leftover
/// pixels at the bottom instead of bleeding them into the last frame.
fn vanilla_row_h(tex: &BgTex, rows: u32) -> f32 {
    (tex.h as i32 / rows.max(1) as i32) as f32
}

/// The Moon Lord hand's / free eye's body draw: `DrawNPCDirect` puts the frame
/// at the NPC's CENTRE with a fixed origin (`Main.cs:24765-24845`,
/// `:24951-25059`) instead of the frame-centre origin the generic path uses.
/// The hand mirrors its origin on the left side (`ai[2] != 0`):
/// `origin.X = sheet.Width - origin.X`.
fn ml_origin_draw<'a>(
    tex: Option<&'a BgTex>,
    rows: u32,
    n: &Npc,
    _w: f32,
    _h: f32,
    origin: [f32; 2],
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let fh = tex.map_or(40.0, |t| t.h / rows as f32);
    let fw = tex.map_or(40.0, |t| t.w);
    let (cx, cy) = n.center();
    let ox = if n.kind == NpcKind::MoonLordHand && n.ai[2] != 0.0 {
        fw - origin[0]
    } else {
        origin[0]
    };
    (tex, fh, [fw, fh], [cx - ox, cy - origin[1]])
}

/// The Moon Lord's extra parts — the core's two torso halves + inner plate and
/// the hand's base plate (`Main.cs:24754-24756`, `:24797-24816`). The C# draws
/// each at the NPC's Centre from a fixed origin; the second torso half is
/// mirrored (`SpriteEffects.FlipHorizontally`).
///
/// Parked (ANALYSIS M4-NPC-3v): the arm CHAINS the core/hand draw between
/// themselves (`Extra[14]`/`Extra[15]`, rotated toward the other part's centre
/// and requiring the `ai[3]` owner lookup), the eye sprite each hand carries
/// (its position rides the unported `localAI` ellipse) and the whole
/// `GetMoonLordColoring` tint.
fn ml_extra_parts(
    n: &Npc,
    torso: Option<BgTex>,
    plate: Option<BgTex>,
    base: Option<BgTex>,
) -> Vec<(BgTex, [f32; 2], [f32; 2], bool)> {
    let (cx, cy) = n.center();
    let mut out = Vec::new();
    match n.kind {
        NpcKind::MoonLordCore => {
            if let Some(t) = torso {
                let (w, h) = (t.w, t.h);
                // `origin14 = (width, 278)`, `origin15 = (0, 278)` on the same
                // sheet; the second draw is horizontally flipped.
                out.push((t, [cx - w, cy - 278.0], [w, h], false));
                out.push((t, [cx, cy - 278.0], [w, h], true));
            }
            if let Some(t) = plate {
                out.push((t, [cx - 112.0, cy - 101.0], [t.w, t.h], false));
            }
        }
        NpcKind::MoonLordHand => {
            if let Some(t) = base {
                let ox = if n.ai[2] != 0.0 { t.w - 26.0 } else { 26.0 };
                out.push((t, [cx - ox, cy - 42.0], [t.w, t.h], n.sprite_dir > 0));
            }
        }
        _ => {}
    }
    out
}

/// The Moon Lord head's own draw (`Main.cs:24846-24927`): `NPC_396` is a
/// 573x804 sheet of 3x3 cells (191x268 each) drawn at `scale = 2` from the
/// origin `(191, 130) / 2` with the cell `Frame(3, 3, frame.Y / 3,
/// frame.Y % 3)` — the column is `frame.Y / 3` and the row `frame.Y % 3`
/// (`Main.npcFrameCount[396]` = 1 is only the dead generic path's divisor).
fn moon_lord_head_draw<'a>(
    tex: Option<&'a BgTex>,
    n: &Npc,
    _w: f32,
    _h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let Some(t) = tex else {
        return (None, 268.0, [382.0, 536.0], [n.x, n.y]);
    };
    let (cell_w, cell_h) = (t.w / 3.0, t.h / 3.0);
    // `origin19` is the CELL's pixel origin `(191, 130) / 2` scaled by the
    // draw's `2f`.
    let (cx, cy) = n.center();
    (
        Some(t),
        cell_h,
        [cell_w * 2.0, cell_h * 2.0],
        [cx - 191.0, cy - 130.0],
    )
}

/// The Moon Lord head's 3x3 grid cell (`Main.cs:24926`'s
/// `value40.Frame(3, 3, frame.Y / 3, frame.Y % 3)`): the column is
/// `frame.Y / 3`, the row `frame.Y % 3`.
fn moon_lord_head_uv(tex: &BgTex, cell: u32) -> [f32; 4] {
    let span = (tex.uv[2] - tex.uv[0], tex.uv[3] - tex.uv[1]);
    let (du, dv) = (span.0 / 3.0, span.1 / 3.0);
    let (col, row) = ((cell / 3) as f32, (cell % 3) as f32);
    [
        tex.uv[0] + du * col,
        tex.uv[1] + dv * row,
        tex.uv[0] + du * (col + 1.0),
        tex.uv[1] + dv * (row + 1.0),
    ]
}

/// The `NPC.scale`-driven draws (the stardust cells' ramp, `AI()`'s aiStyle-95
/// arm `NPC.cs:39393`): the vanilla multiplies the sprite by `npc.scale` about
/// its `origin = frame size / 2` (`Main.cs:23124`'s `scale` argument), so the
/// quad grows symmetrically about the frame's centre.
fn scaled_anchor<'a>(
    tex: Option<&'a BgTex>,
    count: u32,
    n: &Npc,
    w: f32,
    h: f32,
) -> (Option<&'a BgTex>, f32, [f32; 2], [f32; 2]) {
    let (tex, fh, size, pos) = generic_anchor(tex, count, n, w, h);
    let s = n.scale;
    (
        tex,
        fh,
        [size[0] * s, size[1] * s],
        [
            pos[0] - size[0] * (s - 1.0) * 0.5,
            pos[1] - size[1] * (s - 1.0) * 0.5,
        ],
    )
}

fn generic_anchor<'a>(
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
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
    )
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
    // The C#'s anchor lands the frame's centre at `box_bottom + 4`; the
    // `halfSize` override and the `num47` nudge are applied once for every
    // generic-anchored kind after the match (see `draw_npcs`).
    (
        tex,
        fh,
        [fw, fh],
        [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
    )
}

/// The kinds whose draw rides the generic anchor (`Main.cs:25505-25512`)
/// and therefore takes the `halfSize`/`num47` overrides. The towers'
/// dedicated arm (`Main.cs:24159`), the saucer family's Centre-anchored
/// block (`:24586`), the Deerclops' Bottom-anchored one (`:26338`) and the
/// Prime masks' overlays compute their own positions and are excluded.
fn uses_generic_anchor(kind: NpcKind) -> bool {
    !matches!(
        kind,
        NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower
            | NpcKind::MartianSaucer
            | NpcKind::MartianSaucerTurret
            | NpcKind::MartianSaucerCannon
            | NpcKind::MartianSaucerCore
            | NpcKind::Deerclops
    )
}

/// Snap a sprite's top-left to an integer screen pixel so nearest sampling keeps
/// the same source texels while the camera moves (no crawling pixel grid).
#[inline]
/// The `snap_to_screen` seam the projectile draw path uses (M4-PROJ-4c) —
/// the same world-pixel → screen snapping the entity draws run.
pub fn snap_to_screen_pub(pos: [f32; 2], camera: Vec2, zoom: f32, screen: Vec2) -> [f32; 2] {
    snap_to_screen(pos, camera, zoom, screen)
}

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
    // The hair rides its own trimmed frame (`PlayerDrawSet.cs:1770-1786`:
    // `Height -= 2`, `Y -= 336` = six body rows up) — the preview animates rows
    // 6..19 (`ui::char_creation::animated_frame_row`), which the body row would
    // map straight past `Player_Hair_*`'s 14 rows.
    if let Some(tex) = assets.hair.as_ref() {
        let uv = flip_uv(hair_uv(tex, body), false);
        r.push_ui_sprite(pos, [40.0 * scale, 54.0 * scale], uv, hair);
        quads += 1;
    }
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

/// NPC draw pass. `behind_tiles` selects the pass (`Main.cs:22320,22336`):
/// `DrawNPCs(true)` draws only `npc.behindTiles`, `DrawNPCs(false)` the rest.
#[allow(clippy::too_many_arguments)]
// The frame arms below must stay a single `match`: an appended arm never
// reaches the GPU when an earlier arm already covers the kind, and the compiler
// only warns (`unreachable_patterns`). Several M4-NPC display fixes were lost
// that way — the Eye of Cthulhu's six-row strip and the goblins' humanoid band
// each had a faithful arm sitting behind a stale v1 arm — so a duplicate is a
// hard error here.
#[deny(unreachable_patterns)]
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
    behind_tiles: bool,
) -> u32 {
    let white = r.white_uv();
    let mut quads = 0;
    for npc in npcs {
        // Main.cs:22336 — filter to the requested pass.
        if npc.behind_tiles != behind_tiles {
            continue;
        }
        // The C# wraps the whole NPC draw with `position += offset`
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
            // `humanoid_frame_count`). The Zombie rides the same anchor over
            // NPC_3 — 38x144, three 48-px rows (`Main.npcFrameCount[3]` = 3,
            // `Main.cs:67082`); dividing by 1 squeezed all three rows into
            // one quad.
            NpcKind::Zombie => humanoid_draw(assets.zombie.as_ref(), 3, n, w, h),
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
            // The Wizard/Mechanic: the same humanoid shape over
            // NPC_108/124 — 23 rows each (`Main.cs:67094`/`:67096`).
            NpcKind::Wizard => humanoid_draw(assets.wizard.as_ref(), 23, n, w, h),
            NpcKind::Mechanic => humanoid_draw(assets.mechanic.as_ref(), 23, n, w, h),
            // The Stylist (23 rows) and the Tax Collector (25 rows,
            // `Main.cs:67128`) ride the same humanoid anchor.
            NpcKind::Stylist => humanoid_draw(assets.stylist.as_ref(), 23, n, w, h),
            NpcKind::TaxCollector => humanoid_draw(assets.tax_collector.as_ref(), 25, n, w, h),
            // The Golfer: the same humanoid shape over NPC_588 — 25 rows
            // (`Main.cs:67142`).
            NpcKind::Golfer => humanoid_draw(assets.golfer.as_ref(), 25, n, w, h),
            // The Zoologist and the Princess: the same humanoid shape over
            // NPC_633/663 — 23 rows each (`Main.cs:67147`/`:67152`).
            NpcKind::Zoologist => humanoid_draw(assets.zoologist.as_ref(), 23, n, w, h),
            NpcKind::Princess => humanoid_draw(assets.princess.as_ref(), 23, n, w, h),
            // The Dye Trader (25 rows) and the Party Girl (23 rows).
            NpcKind::DyeTrader => humanoid_draw(assets.dye_trader.as_ref(), 25, n, w, h),
            NpcKind::PartyGirl => humanoid_draw(assets.party_girl.as_ref(), 23, n, w, h),
            // The sleeping Golfer: NPC_589's sheet over the flat 18x34
            // body.
            NpcKind::SleepingGolfer => (
                assets.sleeping_golfer.as_ref(),
                assets.sleeping_golfer.as_ref().map_or(34.0, |t| t.h),
                [18.0, 34.0],
                [n.x, n.y],
            ),
            // The webbed Stylist: one row over the flat 18x34 body.
            NpcKind::WebbedStylist => (
                assets.webbed_stylist.as_ref(),
                assets.webbed_stylist.as_ref().map_or(34.0, |t| t.h),
                [18.0, 34.0],
                [n.x, n.y],
            ),
            // The bound forms: single-row sheets over their own boxes.
            NpcKind::BoundWizard => (
                assets.bound_wizard.as_ref(),
                assets.bound_wizard.as_ref().map_or(40.0, |t| t.h),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::BoundMechanic => (
                assets.bound_mechanic.as_ref(),
                assets.bound_mechanic.as_ref().map_or(34.0, |t| t.h),
                [18.0, 34.0],
                [n.x, n.y],
            ),
            // The Sleeping Angler: NPC_376's two rows over the flat 34x8
            // body — the sheet is 40 wide, drawn centred on the box.
            NpcKind::SleepingAngler => (
                assets.sleeping_angler.as_ref(),
                assets.sleeping_angler.as_ref().map_or(8.0, |t| t.h / 2.0),
                [34.0, 8.0],
                [n.x, n.y],
            ),
            // The Eye of Cthulhu is drawn by its own arm further down (the
            // 110x996 sheet's six 166-px rows, `Main.npcFrameCount[4]` = 6).
            // The v1 arm that stood here — `t.h / 2` over a 100x110 quad at
            // the box's corner — sampled HALF the sheet into one sprite and
            // silently shadowed the faithful arm.
            // The Queen Bee: the 2-frame 66x66 sheet.
            // NPC_222 is 172x1824 — twelve 152-px rows
            // (`Main.npcFrameCount[222]` = 12, `Main.cs:67082`; the v1
            // divided by 2 and drew a 66x66 quad at the box's corner,
            // sampling a 912-px slice of the sheet as "one frame").
            NpcKind::QueenBee => boss_draw(assets.queen_bee.as_ref(), 12, n, w, h),
            // The Skeletron head: the single-frame 80x102 sheet.
                        // NPC_35 is 114x264 — two 132-px rows (`Main.npcFrameCount[35]` =
            // 2: the portrait row plus the red-hat row); the v1 drew an
            // 80x102 box quad from `t.h`.
            NpcKind::SkeletronHead => boss_draw(assets.skeletron_head.as_ref(), 2, n, w, h),
            // The WoF: the single-frame sheet over the 120x120 hitbox.
                        // NPC_113 is 142x212 — two 106-px rows
            // (`Main.npcFrameCount[113]` = 2, `Main.cs:67082`; the v1 read
            // `t.h` as the frame height and drew a 120x120 box quad).
            NpcKind::WallOfFlesh => boss_draw(assets.wof.as_ref(), 2, n, w, h),
            // The hungry: `Main.npcFrameCount[115] = 3` frames of 30×30; the
            // leech segments: 1 frame of 14×14.
                        // NPC_115 is 44x114 — three 38-px rows
            // (`Main.npcFrameCount[115]` = 3); the v1 kept the row height but
            // drew a 30x30 box quad.
            NpcKind::Hungry => boss_draw(assets.hungry.as_ref(), 3, n, w, h),
                        // NPC_117 is 18x30 — one frame (`Main.npcFrameCount[117]` = 1); the
            // v1 drew a 14x14 box quad.
            NpcKind::LeechHead | NpcKind::LeechBody => boss_draw(assets.leech.as_ref(), 1, n, w, h),
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
            // The DD2 goblin attackers: ONE sheet per tier — NPC_552 (tier
            // 1) 58x672, NPC_553 (tier 2) 66x672, NPC_554 (tier 3) 66x672,
            // all 14 rows (`Main.npcFrameCount[552..554] = 14`,
            // `Main.cs:67082`). The v1 read NPC_553 for all three and drew
            // every one inside an 18x40 box at the hitbox corner, so the
            // tiers shared one artwork and the art sampled a 48-px slice.
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => {
                let t = match n.kind {
                    NpcKind::DD2GoblinT1 => assets.dd2_goblin_t1.as_ref(),
                    NpcKind::DD2GoblinT3 => assets.dd2_goblin_t3.as_ref(),
                    _ => assets.dd2_goblins.as_ref(),
                };
                generic_anchor(t, 14, n, w, h)
            }
            // The bombers: NPC_555/556/557 are 46x986 with 17 rows each
            // (`Main.npcFrameCount[555..557] = 17`) — the v1 read NPC_556 for
            // all three and divided by 14.
            NpcKind::DD2GoblinBomberT1 | NpcKind::DD2GoblinBomberT2 | NpcKind::DD2GoblinBomberT3 => {
                generic_anchor(assets.dd2_bombers.as_ref(), 17, n, w, h)
            }
            // The skeletons' materialise strips: NPC_566/567 are 48x468 with
            // 9 rows (`Main.npcFrameCount[566] = [567] = 9`) — the v1 read
            // NPC_556 (a bomber) and divided by 14.
            NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => {
                generic_anchor(assets.dd2_skeletons.as_ref(), 9, n, w, h)
            }
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
            // The Old One's Army's tier-2/3 families (M4-EVENT-19): their
            // own `Main.npcFrameCount` rows — the wyverns' 5, the gliders'
            // 10, the Dark Mage T3's 20 (the T1's strip), the Wither Beast
            // T3's 17, the drakins' 16, the kobold walkers' 18, the ogres'
            // 10.
            NpcKind::DD2WyvernT1 | NpcKind::DD2WyvernT2 | NpcKind::DD2WyvernT3 => (
                assets.dd2_wyverns.as_ref(),
                assets.dd2_wyverns.as_ref().map_or(38.0, |t| t.h / 5.0),
                [38.0, 38.0],
                [n.x, n.y],
            ),
            NpcKind::DD2KoboldFlyerT2 | NpcKind::DD2KoboldFlyerT3 => (
                assets.dd2_gliders.as_ref(),
                assets.dd2_gliders.as_ref().map_or(38.0, |t| t.h / 10.0),
                [38.0, 38.0],
                [n.x, n.y],
            ),
            NpcKind::DD2DarkMageT3 => (
                assets.dark_mage_t3.as_ref(),
                assets.dark_mage_t3.as_ref().map_or(62.0, |t| t.h / 20.0),
                [34.0, 62.0],
                [n.x, n.y],
            ),
            NpcKind::DD2WitherBeastT3 => (
                assets.wither_beast_t3.as_ref(),
                assets.wither_beast_t3.as_ref().map_or(40.0, |t| t.h / 17.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 => (
                assets.dd2_drakins.as_ref(),
                assets.dd2_drakins.as_ref().map_or(58.0, |t| t.h / 16.0),
                [42.0, 58.0],
                [n.x, n.y],
            ),
            NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 => (
                assets.dd2_kobolds.as_ref(),
                assets.dd2_kobolds.as_ref().map_or(40.0, |t| t.h / 18.0),
                [18.0, 40.0],
                [n.x, n.y],
            ),
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 => (
                assets.dd2_ogres.as_ref(),
                assets.dd2_ogres.as_ref().map_or(124.0, |t| t.h / 10.0),
                [96.0, 124.0],
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
            NpcKind::StardustTower | NpcKind::NebulaTower | NpcKind::SolarTower | NpcKind::VortexTower
            | NpcKind::DD2LanePortal => {
                let tex = match n.kind {
                    NpcKind::StardustTower => assets.tower_stardust.as_ref(),
                    NpcKind::NebulaTower => assets.tower_nebula.as_ref(),
                    NpcKind::SolarTower => assets.tower_solar.as_ref(),
                    NpcKind::VortexTower => assets.tower_vortex.as_ref(),
                    _ => assets.tower_vortex.as_ref(),
                };
                tower_draw(tex, n, w, h)
            }
            // The pillar families' own sheets (the v1 drew every one of
            // them off ONE sheet — NPC_409 for the whole stardust/solar
            // set, NPC_420/426 for the nebula/vortex trios, NPC_406 for the
            // shards, NPC_425 for the beast AND the vortex soldiers — so
            // only the sheet's first kind ever showed its real art and the
            // rest sampled a wrong row of a foreign sheet). Each arm now
            // reads its own `NPC_<netID>` sheet at the vanilla row count
            // (`Main.npcFrameCount`, `Main.cs:67082`) through the generic
            // anchor (`Main.cs:25511-25516`), and the frame's real size
            // (`NPC.SetFrameSize`, `NPC.cs:90780-90800`) is the quad.
            NpcKind::StardustCellBig => scaled_anchor(assets.npc_sheet(405), 4, n, w, h),
            // The cell's ramp rides `n.scale` (`AI()`'s aiStyle-95 arm,
            // `NPC.cs:39393`) — the strip's own frame height stays put.
            NpcKind::StardustCellSmall => scaled_anchor(assets.npc_sheet(406), 2, n, w, h),
            NpcKind::StardustJellyfishBig => generic_anchor(assets.npc_sheet(407), 5, n, w, h),
            NpcKind::NebulaBeast => generic_anchor(assets.npc_sheet(423), 12, n, w, h),
            // The vortex trio: the hunter 426 (15 rows), the alien 427 (9)
            // and the headcrab 428 (5) — one sheet each.
            NpcKind::VortexHunter => generic_anchor(assets.npc_sheet(426), 15, n, w, h),
            NpcKind::VortexAlien => generic_anchor(assets.npc_sheet(427), 9, n, w, h),
            NpcKind::VortexHeadcrab => generic_anchor(assets.npc_sheet(428), 5, n, w, h),
            // The stardust/solar tower mobs: one sheet per kind
            // (`Main.npcFrameCount`: 409 = 11, 410 = 5, 411 = 11, 415 = 10,
            // 416 = 10, 417 = 15, 418 = 8, 419 = 17, 518 = 10).
            NpcKind::StardustSpiderBig => generic_anchor(assets.npc_sheet(409), 11, n, w, h),
            NpcKind::StardustSpiderSmall => generic_anchor(assets.npc_sheet(410), 5, n, w, h),
            NpcKind::StardustSoldier => generic_anchor(assets.npc_sheet(411), 11, n, w, h),
            NpcKind::SolarDrakomire => generic_anchor(assets.npc_sheet(415), 10, n, w, h),
            NpcKind::SolarDrakomireRider => generic_anchor(assets.npc_sheet(416), 10, n, w, h),
            NpcKind::SolarSroller => generic_anchor(assets.npc_sheet(417), 15, n, w, h),
            NpcKind::SolarCorite => generic_anchor(assets.npc_sheet(418), 8, n, w, h),
            NpcKind::SolarSelenian => generic_anchor(assets.npc_sheet(419), 17, n, w, h),
            NpcKind::SolarSpearman => generic_anchor(assets.npc_sheet(518), 10, n, w, h),
            // The pillar worms: the crawltipede's three 1-row segments
            // (412/413/414, `Main.npcFrameCount` 1 each) and the stardust
            // worm head's single frame (402).
            NpcKind::SolarCrawltipedeHead => generic_anchor(assets.npc_sheet(412), 1, n, w, h),
            NpcKind::SolarCrawltipedeBody => generic_anchor(assets.npc_sheet(413), 1, n, w, h),
            NpcKind::SolarCrawltipedeTail => generic_anchor(assets.npc_sheet(414), 1, n, w, h),
            NpcKind::StardustWormHead => generic_anchor(assets.npc_sheet(402), 1, n, w, h),
            // The nebula trio: the floater 420 (6 rows), the brain suckler
            // 421 (6) and the predictor 424 (12).
            NpcKind::NebulaFloater => generic_anchor(assets.npc_sheet(420), 6, n, w, h),
            NpcKind::BrainSuckler => generic_anchor(assets.npc_sheet(421), 6, n, w, h),
            NpcKind::Predictor => generic_anchor(assets.npc_sheet(424), 12, n, w, h),
            // The vortex rifleman is net 425 (13 rows) and the soldier 429
            // (10 rows) — the v1 drew both off NPC_425 over 13 rows.
            NpcKind::VortexRifleman => generic_anchor(assets.npc_sheet(425), 13, n, w, h),
            NpcKind::VortexSoldier => generic_anchor(assets.npc_sheet(429), 10, n, w, h),
            // The Moon Lord family: the head 396 has a 3x3 GRID sheet drawn
            // by its own block (`Main.cs:24846-24927`) — park that block handed
            // in `moon_lord_head_draw`; the hand 397 (4 rows), the core 398
            // (5), the free eye 400 (4) and the leech 401 (1) ride the
            // generic arm at their `Main.npcFrameCount` rows (their custom
            // blocks' extra parts are parked, ANALYSIS M4-NPC-3t).
            NpcKind::MoonLordHead => moon_lord_head_draw(assets.npc_sheet(396), n, w, h),
            // The hand's own block (`Main.cs:24765-24845`): the frame is drawn
            // at the NPC's Centre from the origin `(120, 180)` — mirrored to
            // `(width - 120, 180)` for the LEFT hand (`ai[2] != 0`), which is
            // 44 px lower than the frame-centre anchor the generic path used.
            NpcKind::MoonLordHand => ml_origin_draw(assets.npc_sheet(397), 4, n, w, h, [120.0, 180.0]),
            // The core (`Main.cs:24701-24764`) draws its body at the Centre with
            // `origin = frame.Size() / 2` — the generic anchor already does
            // that; its torso halves and inner plate ride the extra pass.
            NpcKind::MoonLordCore => generic_anchor(assets.npc_sheet(398), 5, n, w, h),
            // The free eye (`Main.cs:24951-25059`) uses the fixed origin
            // `(40, 40)` on a 78x120 row, i.e. 20 px higher than the frame
            // centre.
            NpcKind::MoonLordFreeEye => ml_origin_draw(assets.npc_sheet(400), 4, n, w, h, [40.0, 40.0]),
            NpcKind::MoonLordLeechBlob => generic_anchor(assets.npc_sheet(401), 1, n, w, h),
            // The javelin throwers: NPC_561/562/563 are 84x1064 with 14 rows
            // (`Main.npcFrameCount[561..563] = 14`) — the v1 read the
            // bombers' NPC_556 sheet and divided by 14, drawing a 52-px
            // slice of a 46-px-wide sheet inside a 26x52 box.
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => {
                generic_anchor(assets.dd2_javelins.as_ref(), 14, n, w, h)
            }
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
                        // NPC_134 is 50x102 — one frame (`Main.npcFrameCount[134]` = 1, no
            // `FindFrame` case); the v1 drew a 48x48 box quad at `t.h`.
            NpcKind::TheDestroyer => boss_draw(assets.destroyer.as_ref(), 1, n, w, h),
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
                // NPC_262 is 116x1232 — eight 154-px rows
                // (`Main.npcFrameCount[262]` = 8, `Main.cs:67082`; the v1
                // divided by 5 and drew an 86x86 box-sized quad at the
                // box's corner).
                boss_draw(assets.plantera.as_ref(), 8, n, w, h)
            ),
            // The Brain: the single-frame 160x110 sheet.
            NpcKind::BrainOfCthulhu => (
                // NPC_266 is 200x1456 — eight 182-px rows
                // (`Main.npcFrameCount[266]` = 8; the v1 read `t.h` as the
                // frame height, sampling the whole 8-row strip as one
                // 160x110 quad).
                boss_draw(assets.brain.as_ref(), 8, n, w, h)
            ),
            // The Creeper: the single-frame 30x30 sheet.
            NpcKind::Creeper => (
                // NPC_267 is 34x34 — one frame (`Main.npcFrameCount[267]`
                // = 1; the v1's 30x30 was the Creeper's box, not the
                // sheet's frame).
                boss_draw(assets.creeper.as_ref(), 1, n, w, h)
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
                    [n.x + w * 0.5 - 93.0, n.y + h * 0.5 - fh * 0.5],
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
                    [n.x + w * 0.5 - 56.0, n.y + h * 0.5 - fh * 0.5],
                )
            }
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => {
                let t = assets.golem_fist.as_ref();
                let fh = t.map_or(40.0, |t| t.h);
                (
                    t,
                    fh,
                    [50.0, fh],
                    [n.x + w * 0.5 - 25.0, n.y + h * 0.5 - fh * 0.5],
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
                        // NPC_36 is 58x156 — two 78-px rows (`Main.npcFrameCount[36]` = 2);
            // the v1 drew a 52x52 box quad.
            NpcKind::SkeletronHand => boss_draw(assets.skeletron_hand.as_ref(), 2, n, w, h),
            // The WoF's mirror eye: the 100x100 sheet (the vanilla's
            // `WorldofFleshEye` texture).
                        // NPC_114 is 130x212 — two 106-px rows (`Main.npcFrameCount[114]` = 2).
            NpcKind::WallOfFleshEye => boss_draw(assets.wof_eye.as_ref(), 2, n, w, h),
            // The Prime's limbs (`Main.npcFrameCount` 128/129/130/131 =
            // 1/2/2/1, `Main.cs:67082`): the cannon 38x76, the saw's 2
            // rows of 78, the vice's 2 rows of 70, the laser 40x62 — each
            // at the generic anchor (the v1 read `t.h` as the frame height
            // and pinned every kind to row 0).
            NpcKind::PrimeCannon => prime_draw(assets.prime_cannon.as_ref(), 1, n, w, h),
            NpcKind::PrimeSaw => prime_draw(assets.prime_saw.as_ref(), 2, n, w, h),
            NpcKind::PrimeVice => prime_draw(assets.prime_vice.as_ref(), 2, n, w, h),
            NpcKind::PrimeLaser => prime_draw(assets.prime_laser.as_ref(), 1, n, w, h),
            // The goblins are drawn by the humanoid arm further down (the
            // 16-row strips, `Main.npcFrameCount[26/27/28]` = 16 and
            // `[111]` = 21). The v1 arm that stood here divided the sheet
            // by 3 — its comment claimed `npcFrameCount = 3` — and pinned
            // the quad to magic `[n.x - 10, n.y - 5]` offsets, so a goblin
            // sampled a 1/3 slice of its strip and floated off its box; it
            // also shadowed the faithful arm.
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
                    [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
                )
            }
            // The worms: the single-frame segments rotated along the chain
            // (`rotation = atan2 + 1.57`), drawn only in the open — the
            // vanilla `behindTiles` NPCs render in the behind-tiles pass
            // (Main.cs:22320,22336) — filtered in `draw_npcs`.
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
                    [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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
                    [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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
                    [n.x + w * 0.5 - fw * 0.5, n.y + h * 0.5 - fh * 0.5],
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
                        // NPC_5 is 20x64 — two 32-px frames
            // (`Main.npcFrameCount[5]` = 2); the v1 kept the row height but
            // drew a 20x20 quad.
            NpcKind::ServantOfCthulhu => boss_draw(assets.servant_of_cthulhu.as_ref(), 2, n, w, h),
        };
        let Some(tex) = tex else {
            continue;
        };
        // `Main.cs:23048`'s per-kind anchor nudge (`num47` + `num46`): a
        // flat Y shift the C#'s generic draw (and the towers' dedicated
        // arm) add to the anchor. Applied once, here, for every kind.
        // `Main.cs:23048`'s per-kind anchor overrides, applied once here
        // for every generic-anchored kind: the `halfSize` override shifts
        // the frame by `override − frame_h / 2` and the `num47`/`num46`
        // nudge adds its flat pixels. The kinds with dedicated
        // `DrawNPCDirect` arms (the towers' Centre anchor, the saucers'
        // block, the Deerclops' Bottom anchor) compute their own
        // positions and skip both terms, as in the C#.
        // The C#'s whole Y anchor: `pos.Y + height + 4 + halfSize.Y +
        // num47 + num46 + gfxOffY`, drawn from `halfSize` (the frame's
        // centre). Relative to the frame-centred `pos` the arm produced
        // (`bottom + 4 - frame_h/2`), the missing terms are `halfSize.Y -
        // frame_h/2` (the `:23048` overrides) plus `num45/num46/num47`.
        let anchor_shift = if uses_generic_anchor(n.kind) {
            boss_half_size(n.kind, frame_h) - frame_h * 0.5 + npc_anchor_nudge(n)
        } else {
            0.0
        };
        let pos = [pos[0], pos[1] + anchor_shift];
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
        // `Main.DrawNPC` flips on `spriteDirection == 1` for EVERY NPC
        // (`Main.cs:23124-23128`) — BUG-07: the whitelist read `facing` for
        // the kinds whose `spriteDirection` the AI machines do maintain (the
        // walker's lockstep write in `npc_ai.rs`), which flipped them
        // against the sheet whenever the body turned without moving. The
        // whitelist is gone: one flip source, the vanilla one.
        let flip_dir = n.sprite_dir;
        let uv = if n.kind == NpcKind::Deerclops {
            flip_uv(deerclops_frame_uv(tex, n.frame()), flip_dir < 0)
        } else if n.kind == NpcKind::MoonLordHead {
            flip_uv(moon_lord_head_uv(tex, n.frame()), flip_dir > 0)
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
        // The Moon Lord parts' fixed-origin extras (`Main.cs:24754-24756`,
        // `:24797-24816`): unrotated, light-tinted overlays at the part's
        // Centre.
        for (t, top_left, size, mirror) in
            ml_extra_parts(n, assets.ml_torso, assets.ml_core_plate, assets.ml_hand_base)
        {
            let euv = flip_uv(t.uv, mirror);
            r.push_sprite(
                snap_to_screen(top_left, camera, zoom, screen),
                size,
                euv,
                tint,
            );
            quads += 1;
        }
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

    /// A sheet of the given pixel size, packed into the whole atlas UV
    /// square (what `sky_tex` hands the draw arms).
    fn sheet(w: f32, h: f32) -> BgTex {
        BgTex {
            uv: [0.0, 0.0, 1.0, 1.0],
            w,
            h,
        }
    }

    /// The nine 🟡 M4-NPC display rows' arithmetic: every one of these
    /// strips must divide into `Main.npcFrameCount[type]` whole rows, which
    /// is how each arm's row count was chosen. The sizes are the real
    /// `assets/Content/Images/NPC_*.xnb` dimensions, so a wrong divisor
    /// shows up as a fractional row here instead of a squashed sprite in
    /// game. (`npcFrameCount` values are `Main.cs:67082`.)
    #[test]
    fn the_boss_strips_divide_into_their_vanilla_rows() {
        // (kind, sheet w, sheet h, npcFrameCount[type], the arm)
        let cases: &[(NpcKind, f32, f32, u32)] = &[
            (NpcKind::EyeOfCthulhu, 110.0, 996.0, 6),      // NPC_4
            (NpcKind::Retinazer, 110.0, 1200.0, 6),        // NPC_125
            (NpcKind::Spazmatism, 110.0, 1200.0, 6),       // NPC_126
            (NpcKind::QueenBee, 172.0, 1824.0, 12),        // NPC_222
            (NpcKind::SkeletronHead, 114.0, 264.0, 2),     // NPC_35
            (NpcKind::WallOfFlesh, 142.0, 212.0, 2),       // NPC_113
            (NpcKind::WallOfFleshEye, 130.0, 212.0, 2),    // NPC_114
            (NpcKind::Plantera, 116.0, 1232.0, 8),         // NPC_262
            (NpcKind::BrainOfCthulhu, 200.0, 1456.0, 8),   // NPC_266
            (NpcKind::KingSlime, 174.0, 720.0, 6),         // NPC_50
            (NpcKind::PrimeSaw, 38.0, 156.0, 2),           // NPC_129
            (NpcKind::PrimeVice, 40.0, 140.0, 2),          // NPC_130
            (NpcKind::PrimeCannon, 38.0, 76.0, 1),         // NPC_128
            (NpcKind::PrimeLaser, 40.0, 62.0, 1),          // NPC_131
            (NpcKind::GrayGrunt, 40.0, 432.0, 9),          // NPC_381
            (NpcKind::Zombie, 38.0, 144.0, 3),             // NPC_3
            (NpcKind::StardustTower, 174.0, 364.0, 1),     // NPC_493
            (NpcKind::NebulaTower, 174.0, 364.0, 1),       // NPC_507
            (NpcKind::SolarTower, 174.0, 364.0, 1),        // NPC_517
        ];
        for &(kind, w, h, rows) in cases {
            let t = sheet(w, h);
            let npc = bare(kind);
            let (tex, fh, size, _) = boss_draw(Some(&t), rows, &npc, 20.0, 40.0);
            assert!(tex.is_some());
            assert_eq!(
                fh,
                h / rows as f32,
                "{kind:?}: {h} px over {rows} rows must be a whole row"
            );
            assert_eq!(size, [w, fh], "{kind:?}: the frame keeps the sheet width");
            // And the strip the machine walks never leaves those rows.
            assert!(fh.fract() == 0.0, "{kind:?}: a fractional row means a wrong divisor");
        }
    }

    #[test]
    fn the_family_rows_match_between_the_draw_and_the_machine() {
        // The draw arm slices the sheet by its own row literal and `FindFrame`
        // walks `npc_rows(texture_net_id(kind))`; a drift between the two shows
        // as a strip that only animates its first few rows. (396 is the 3x3
        // grid: the generic path divides by `Main.npcFrameCount[396]` = 1.)
        let cases: &[(NpcKind, u32)] = &[
            (NpcKind::StardustWormHead, 1),
            (NpcKind::StardustCellBig, 4),
            (NpcKind::StardustCellSmall, 2),
            (NpcKind::StardustJellyfishBig, 5),
            (NpcKind::StardustSpiderBig, 11),
            (NpcKind::StardustSpiderSmall, 5),
            (NpcKind::StardustSoldier, 11),
            (NpcKind::SolarCrawltipedeHead, 1),
            (NpcKind::SolarCrawltipedeBody, 1),
            (NpcKind::SolarCrawltipedeTail, 1),
            (NpcKind::SolarDrakomire, 10),
            (NpcKind::SolarDrakomireRider, 10),
            (NpcKind::SolarSroller, 15),
            (NpcKind::SolarCorite, 8),
            (NpcKind::SolarSelenian, 17),
            (NpcKind::NebulaFloater, 6),
            (NpcKind::BrainSuckler, 6),
            (NpcKind::NebulaBeast, 12),
            (NpcKind::Predictor, 12),
            (NpcKind::VortexRifleman, 13),
            (NpcKind::VortexHunter, 15),
            (NpcKind::VortexAlien, 9),
            (NpcKind::VortexHeadcrab, 5),
            (NpcKind::VortexSoldier, 10),
            (NpcKind::SolarSpearman, 10),
            (NpcKind::MoonLordHead, 1),
            (NpcKind::MoonLordHand, 4),
            (NpcKind::MoonLordCore, 5),
            (NpcKind::MoonLordFreeEye, 4),
            (NpcKind::MoonLordLeechBlob, 1),
        ];
        for &(kind, rows) in cases {
            let net = texture_net_id(kind);
            assert_eq!(npc_rows(net), rows, "{kind:?}: NPC_{net} rows");
        }
    }

    #[test]
    fn the_packing_list_has_no_duplicate_sheets() {
        // A duplicate would just be decoded twice and packed twice — a silent
        // atlas bloat, and a sign the list is drifting.
        use crate::assets::STATIC_SHEET_NAMES;
        let mut seen = std::collections::BTreeSet::new();
        let mut dups: Vec<&str> = Vec::new();
        for name in STATIC_SHEET_NAMES {
            if !seen.insert(*name) {
                dups.push(name);
            }
        }
        assert!(dups.is_empty(), "duplicate sheet names: {dups:?}");
    }

    #[test]
    fn every_sheet_a_draw_arm_asks_for_is_packed() {
        // The M4-NPC-3v bug: `NpcAssets::npc_sheets` is built from the atlas by
        // name, and `load_static_sheets`' list was missing `NPC_396-429`/`518`,
        // so the whole lunar family silently drew nothing — the unit tests
        // fabricated their own `BgTex` and could not see it. This audit pins
        // the names the draw arms ask `sky_tex` for to the packing list.
        use crate::assets::STATIC_SHEET_NAMES;
        let packed = |name: &str| STATIC_SHEET_NAMES.contains(&name);
        for &id in PILLAR_SHEETS {
            let name = format!("NPC_{id}");
            assert!(
                packed(&name),
                "{name} is asked for by `npc_sheet({id})` but never packed"
            );
        }
        for name in [
            ML_TORSO_SHEET,
            ML_CORE_PLATE_SHEET,
            ML_HAND_BASE_SHEET,
            ML_EYE_SHEET,
        ] {
            assert!(packed(name), "{name} is asked for by the Moon Lord's parts");
        }
        // Every lunar kind must resolve to a sheet the pillar set carries —
        // `npc_sheet(texture_net_id(kind))` is the draw's only source.
        for net in 396u16..=430 {
            let Some(kind) = NpcKind::from_u16(net) else {
                continue;
            };
            let tex_id = texture_net_id(kind);
            assert!(
                PILLAR_SHEETS.contains(&(tex_id as u32)),
                "{kind:?} draws from NPC_{tex_id}, outside PILLAR_SHEETS"
            );
        }
    }

    #[test]
    fn the_moon_lord_parts_draw_from_their_own_origins() {
        // The hand (`Main.cs:24765-24845`): body at the NPC's CENTRE from the
        // origin `(120, 180)`, mirrored to `(width - 120, 180)` for the left
        // hand (`ai[2] != 0`). `NPC_397` is 246 px wide, four rows tall.
        let hand_tex = sheet(246.0, 1088.0);
        let mut hand = bare(NpcKind::MoonLordHand);
        let (cx, cy) = hand.center();
        let (_, fh, size, pos) = ml_origin_draw(Some(&hand_tex), 4, &hand, 0.0, 0.0, [120.0, 180.0]);
        assert_eq!(fh, 272.0, "the hand's row height");
        assert_eq!(size, [246.0, 272.0]);
        assert_eq!(pos, [cx - 120.0, cy - 180.0], "the origin, not the frame centre");
        // The left hand mirrors the origin about the sheet's width.
        hand.ai[2] = 1.0;
        let (_, _, _, pos_l) = ml_origin_draw(Some(&hand_tex), 4, &hand, 0.0, 0.0, [120.0, 180.0]);
        assert_eq!(pos_l, [cx - 126.0, cy - 180.0], "(246 - 120, 180)");

        // The free eye (`Main.cs:24951-25059`): origin `(40, 40)` on an 78x120
        // row — 20 px above the frame centre the generic anchor would use.
        let eye_tex = sheet(78.0, 480.0);
        let eye = bare(NpcKind::MoonLordFreeEye);
        let (ecx, ecy) = eye.center();
        let (_, efh, esize, epos) = ml_origin_draw(Some(&eye_tex), 4, &eye, 0.0, 0.0, [40.0, 40.0]);
        assert_eq!(efh, 120.0);
        assert_eq!(esize, [78.0, 120.0]);
        assert_eq!(epos, [ecx - 40.0, ecy - 40.0]);
        assert_eq!(epos[1], ecy - 40.0);

        // The core's extras (`Main.cs:24754-24756`): two torso halves from
        // `(w, 278)` and `(0, 278)` (the second flipped) plus the inner plate
        // from `(112, 101)`.
        let torso = sheet(300.0, 400.0);
        let plate = sheet(200.0, 200.0);
        let core = bare(NpcKind::MoonLordCore);
        let (ccx, ccy) = core.center();
        let parts = ml_extra_parts(&core, Some(torso), Some(plate), None);
        assert_eq!(parts.len(), 3, "two torso halves plus the plate");
        assert_eq!(parts[0].1, [ccx - 300.0, ccy - 278.0], "the right half");
        assert!(!parts[0].3, "drawn unflipped");
        assert_eq!(parts[1].1, [ccx, ccy - 278.0], "the left half mirrors at x = 0");
        assert!(parts[1].3, "the second half is FlipHorizontally");
        assert_eq!(parts[2].1, [ccx - 112.0, ccy - 101.0], "the inner plate");
        assert!(!parts[2].3);

        // The hand's base plate (`Main.cs:24797-24816`) from `(26, 42)`, the
        // origin mirrored on the left hand and the sprite flipped with the
        // body.
        let base = sheet(120.0, 120.0);
        let mut hb = bare(NpcKind::MoonLordHand);
        let (hx, hy) = hb.center();
        let parts = ml_extra_parts(&hb, None, None, Some(base));
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].1, [hx - 26.0, hy - 42.0]);
        hb.ai[2] = 1.0;
        let parts = ml_extra_parts(&hb, None, None, Some(base));
        assert_eq!(parts[0].1, [hx - 94.0, hy - 42.0], "(120 - 26, 42)");
    }

    #[test]
    fn the_moon_lord_head_draws_its_grid_cell() {
        // `Main.cs:24846-24927`: `NPC_396` (573x804) is a 3x3 grid of 191x268
        // cells drawn at `scale = 2` from the origin `(191, 130) / 2`, the cell
        // `Frame(3, 3, frame.Y / 3, frame.Y % 3)`.
        let t = sheet(573.0, 804.0);
        let n = bare(NpcKind::MoonLordHead);
        let (cx, cy) = n.center();
        // Frame 4 = column 1, row 1 of the grid.
        let mut n2 = n;
        n2.frame = 4;
        let (_, fh, size, pos) = moon_lord_head_draw(Some(&t), &n2, 140.0, 140.0);
        assert_eq!(fh, 268.0, "the cell height, not the whole sheet");
        assert_eq!(size, [382.0, 536.0], "191x268 at the draw's 2x scale");
        assert_eq!(pos, [cx - 191.0, cy - 130.0], "origin (95.5, 65) at 2x");
        let uv = moon_lord_head_uv(&t, 4);
        assert!((uv[0] - 1.0 / 3.0).abs() < 1e-6, "column 1 starts at 1/3");
        assert!((uv[1] - 1.0 / 3.0).abs() < 1e-6, "row 1 starts at 1/3");
        assert!((uv[2] - 2.0 / 3.0).abs() < 1e-6);
        assert!((uv[3] - 2.0 / 3.0).abs() < 1e-6);
        // Cell 0 is the top-left, cell 8 the bottom-right — the C# indexes the
        // COLUMN with `frame.Y / 3` and the ROW with `frame.Y % 3`.
        let uv8 = moon_lord_head_uv(&t, 8);
        assert!((uv8[0] - 2.0 / 3.0).abs() < 1e-6, "cell 8 sits in column 2");
        assert!((uv8[1] - 2.0 / 3.0).abs() < 1e-6, "and row 2");
    }

    #[test]
    fn the_pillar_families_each_own_their_sheet() {
        // The v1 pointed every stardust/solar/nebula/vortex kind at one shared
        // family sheet (`tower_mobs` = NPC_409, `moon_lord_parts` = NPC_396…),
        // so eight different mobs drew the same strip. Every family kind must
        // now name its own `NPC_<netID>` and take that sheet's row count.
        // The vanilla net ids (`NPCID.cs:11481-11533`, `:11713`).
        let cases: &[(NpcKind, u16)] = &[
            (NpcKind::StardustWormHead, 402),
            (NpcKind::StardustCellBig, 405),
            (NpcKind::StardustCellSmall, 406),
            (NpcKind::StardustJellyfishBig, 407),
            (NpcKind::StardustSpiderBig, 409),
            (NpcKind::StardustSpiderSmall, 410),
            (NpcKind::StardustSoldier, 411),
            (NpcKind::SolarCrawltipedeHead, 412),
            (NpcKind::SolarCrawltipedeBody, 413),
            (NpcKind::SolarCrawltipedeTail, 414),
            (NpcKind::SolarDrakomire, 415),
            (NpcKind::SolarDrakomireRider, 416),
            (NpcKind::SolarSroller, 417),
            (NpcKind::SolarCorite, 418),
            (NpcKind::SolarSelenian, 419),
            (NpcKind::NebulaFloater, 420),
            (NpcKind::BrainSuckler, 421),
            (NpcKind::NebulaBeast, 423),
            (NpcKind::Predictor, 424),
            (NpcKind::VortexRifleman, 425),
            (NpcKind::VortexHunter, 426),
            (NpcKind::VortexAlien, 427),
            (NpcKind::VortexHeadcrab, 428),
            (NpcKind::VortexSoldier, 429),
            (NpcKind::SolarSpearman, 518),
        ];
        for &(kind, net) in cases {
            assert!(
                PILLAR_SHEETS.contains(&(net as u32)),
                "{kind:?}: NPC_{net} is not loaded"
            );
        }
    }

    #[test]
    fn the_eoc_frame_is_one_of_the_sheets_six_rows() {
        // `Main.npcFrameCount[4]` = 6 (`Main.cs:67082`) and NPC_4 measures
        // 110x996, so a frame is 110x166. The stale v1 arm divided the sheet
        // by 2 and drew a 100x110 quad at the box's corner, sampling half the
        // strip into one sprite; it sat in front of the faithful arm and so
        // won. `draw_npcs` now denies that shadowing outright.
        let t = sheet(110.0, 996.0);
        let n = bare(NpcKind::EyeOfCthulhu);
        let (w, h) = n.kind.size();
        let (tex, fh, size, pos) = boss_draw(Some(&t), 6, &n, w, h);
        assert!(tex.is_some());
        assert_eq!(fh, 166.0, "996 / 6 rows");
        assert_eq!(size, [110.0, 166.0], "the frame is wider than the 100x110 box");
        // The generic anchor: the frame's centre lands on the NPC's Centre
        // (`Main.cs:25511-25516`'s `origin = halfSize` cancels the
        // `- frame / 2` term, leaving `Center + num46 + num47`).
        assert_eq!(pos[0], n.x + w * 0.5 - 55.0);
        assert_eq!(pos[1], n.y + h * 0.5 - 83.0);
        // One frame's UV spans exactly 1/6 of the strip.
        let uv = npc_frame_uv(&t, fh, 2);
        assert!((uv[1] - 2.0 / 6.0).abs() < 1e-6, "row 2 starts at 1/3");
        assert!((uv[3] - 3.0 / 6.0).abs() < 1e-6, "and ends at 1/2");
    }

    #[test]
    fn the_goblin_frame_is_one_of_the_humanoid_strips_rows() {
        // `Main.npcFrameCount[26/27/28]` = 16 (`Main.cs:67082`); the stale v1
        // arm divided the strip by 3 and pinned the quad to magic
        // `[n.x - 10, n.y - 5]` offsets. The humanoid arm derives the frame
        // from the sheet and centres it on the box
        let t = sheet(38.0, 768.0);
        let n = bare(NpcKind::GoblinPeon);
        let (w, h) = n.kind.size();
        let (_, fh, size, pos) = humanoid_draw(Some(&t), 16, &n, w, h);
        assert_eq!(fh, 48.0, "768 / 16 rows");
        assert_eq!(size, [38.0, 48.0]);
        assert_eq!(pos[0], n.x + w * 0.5 - 19.0);
        assert_eq!(pos[1], n.y + h * 0.5 - 24.0);
        let uv = npc_frame_uv(&t, fh, 5);
        assert!((uv[1] - 5.0 / 16.0).abs() < 1e-6);
        assert!((uv[3] - 6.0 / 16.0).abs() < 1e-6);
        // The archer's strip is 21 rows (`[111]`), not 16.
        let (_, fh, _, _) = humanoid_draw(Some(&t), 21, &bare(NpcKind::GoblinArcher), w, h);
        assert!((fh - 768.0 / 21.0).abs() < 1e-4);
    }

    /// `Main.cs:22991-23115`'s `num45`/`num46`/`num47` chain — the
    /// `Main.NPCAddHeight` base plus the `:23048` switch's per-kind terms.
    #[test]
    fn the_anchor_nudges_follow_the_c_sharp_switch() {
        let mut n = bare(NpcKind::Plantera);
        n.scale = 1.0;
        assert_eq!(npc_anchor_nudge(&n), 26.0, "the Plantera head's num47");
        let mut b = bare(NpcKind::BrainOfCthulhu);
        b.scale = 0.5;
        assert_eq!(npc_anchor_nudge(&b), 25.0, "the Brain's 50 * scale");
        // The support group's `num47` is an OVERWRITE (`:23226-23243`), so
        // its own `NPCAddHeight` row (4/-5/-4/-2/-3 for the goblin rows)
        // never reaches the draw: the switch replaces it.
        // (the T2's `num46 = -6` from `:23014` is a separate `else if`
        // chain, so it does not see this overwrite.)
        assert_eq!(npc_anchor_nudge(&bare(NpcKind::DD2JavelinT1)), -8.0);
        // (the T3's own `NPCAddHeight` row is 4 (`:22087`), which the
        // switch's -8 overwrite replaces.)
        assert_eq!(npc_anchor_nudge(&bare(NpcKind::DD2JavelinT3)), 4.0);
        // (`NPCAddHeight`'s T2 row is -6 (`:23014`), replaced by the -8.
        // The vanilla's 581 — the port calls it T2 — takes only the
        // overwrite: history.)
        // (the vanilla's 581, the port's T2, takes `NPCAddHeight`'s 2
        // (`:22153`, the 612/613 rig row) and the `:23038` `num46 -= 2`
        // from the same chain — it is NOT in the support group's overwrite
        // list, so both apply.)
        assert_eq!(
            npc_anchor_nudge(&bare(NpcKind::DD2JavelinT2)),
            -6.0,
            "the T2's -6 num46"
        );
        for kind in [
            NpcKind::DD2GoblinBomberT1,
            NpcKind::DD2GoblinBomberT2,
            NpcKind::DD2GoblinBomberT3,
        ] {
            assert_eq!(npc_anchor_nudge(&bare(kind)), -5.0, "{kind:?}");
        }
        assert_eq!(npc_anchor_nudge(&bare(NpcKind::DD2WitherBeast)), -3.0);
        // `Main.NPCAddHeight`'s own rows (`Main.cs:22002-22184`): the Twins'
        // 30 (`:22011`), the Eater of Souls' 26 (`:22031`), the zombie's 13
        // (`:22039`), the hellbat's 10 (`:22079`), the Skeletron's -4
        // (`:22099`) and the Wall of Flesh's -2 (`:22103`).
        for (kind, want) in [
            (NpcKind::Retinazer, 30.0),
            (NpcKind::Spazmatism, 30.0),
            (NpcKind::EaterOfSouls, 26.0),
            (NpcKind::Zombie, 13.0),
            (NpcKind::DevourerHead, 13.0),
            (NpcKind::GiantWormHead, 8.0),
            (NpcKind::Hellbat, 10.0),
            (NpcKind::CaveBat, 4.0),
            (NpcKind::Shark, 5.0),
            (NpcKind::GrayGrunt, 4.0),
            (NpcKind::SkeletronHead, -4.0),
            (NpcKind::WallOfFlesh, -2.0),
            (NpcKind::SkeletronHand, 20.0),
            (NpcKind::Goldfish, 56.0),
            (NpcKind::TheDestroyer, 30.0),
            (NpcKind::GoblinArcher, 14.0),
            // The town rows: the Clothier's 2 (`:22015` — it is a 2-px
            // row, untouched by the sitting shift) and the vanilla's 124
            // (the port's Mechanic) at 2 (`:23000`), which the sitting
            // ladder then takes to -4.
            (NpcKind::Clothier, 2.0),
            (NpcKind::Mechanic, 2.0),
        ] {
            assert_eq!(npc_anchor_nudge(&bare(kind)), want, "{kind:?}");
        }
        // The `townNPC && ai[0] == 5` sitting row: `NPCAddHeight`'s -4
        // replaces nothing for the Clothier (its 2 is its own row), the
        // C# sums (`num -= 4`), so 2 + -4 = -2.
        let mut sitting = bare(NpcKind::Clothier);
        sitting.ai[0] = 5.0;
        assert!(sitting.kind.is_town_npc(), "the town test must hold");

        assert_eq!(npc_anchor_nudge(&sitting), -2.0, "the Clothier's 2 + -4");
        // The vanilla's 124 (the port's Mechanic): the `:23000` `num46 = 2`
        // then the sitting ladder's `-4 - 2` (`:22174`'s 124 arm) => -4.
        let mut sit_mech = bare(NpcKind::Mechanic);
        sit_mech.ai[0] = 5.0;
        assert_eq!(npc_anchor_nudge(&sit_mech), -4.0, "the Mechanic's 2 - 6");
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
