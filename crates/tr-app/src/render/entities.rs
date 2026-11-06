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
#[derive(Default)]
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
    /// (`frontTexture`) of the M2-MOUNT-1 ground set.
    pub(crate) mount_back: [Option<BgTex>; 12],
    pub(crate) mount_front: [Option<BgTex>; 12],
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
        goblin_thief: sky_tex(atlas, "NPC_26", None),
        goblin_warrior: sky_tex(atlas, "NPC_27", None),
        goblin_archer: sky_tex(atlas, "NPC_28", None),
        goblin_peon: sky_tex(atlas, "NPC_111", None),
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
        if s.rotation != 0.0 {
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
    // `PlayerDrawSet.cs:343-347` + `GetSittingOffsetInfo`
    // (`PlayerSittingHelper.cs:14-31`): the sitting body draws shifted up by
    // the seat adjustment (-4 px; the index-0 posOffset is zero). The
    // sliced-leg sit pose is the render follow-up.
    let sit_shift = if player.sitting { -4.0 } else { 0.0 };
    let state = PlayerCompositeState {
        origin: [player.x - 10.0, player.y - 10.0 + sit_shift],
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
    };
    push_layers(r, &assemble_layers(&state, assets), camera, zoom, screen)
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
    for n in npcs {
        let (cx, cy) = n.center();
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let (w, _) = n.size();
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
            NpcKind::Zombie => (
                assets.zombie.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Guide rides the zombie's humanoid frame machine (the
            // `Main.npcFrameCount[22]` = 3 rows of the same 38x48 shape).
            NpcKind::Guide => (
                assets.guide.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Merchant: the same humanoid shape over NPC_17.
            NpcKind::Merchant => (
                assets.merchant.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Dryad: the same humanoid shape over NPC_20.
            NpcKind::Dryad => (
                assets.dryad.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Nurse: the same humanoid shape over NPC_18.
            NpcKind::Nurse => (
                assets.nurse.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Arms Dealer: the same humanoid shape over NPC_19.
            NpcKind::ArmsDealer => (
                assets.arms_dealer.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Demolitionist: the same humanoid shape over NPC_38.
            NpcKind::Demolitionist => (
                assets.demolitionist.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Painter: the same humanoid shape over NPC_227.
            NpcKind::Painter => (
                assets.painter.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Witch Doctor: the same humanoid shape over NPC_228.
            NpcKind::WitchDoctor => (
                assets.witch_doctor.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The Clothier: the same humanoid shape over NPC_54.
            NpcKind::Clothier => (
                assets.clothier.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
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
            // The Destroyer: the single-frame 48x48 sheet.
            NpcKind::TheDestroyer => (
                assets.destroyer.as_ref(),
                assets.destroyer.as_ref().map_or(48.0, |t| t.h),
                [48.0, 48.0],
                [n.x, n.y],
            ),
            // The Prime: the single-frame 80x102 sheet.
            NpcKind::SkeletronPrime => (
                assets.prime.as_ref(),
                assets.prime.as_ref().map_or(102.0, |t| t.h),
                [80.0, 102.0],
                [n.x, n.y],
            ),
            // The Twins: the 2-frame 100x110 sheets.
            NpcKind::Retinazer | NpcKind::Spazmatism => {
                let tex = if n.kind == NpcKind::Retinazer {
                    assets.retinazer.as_ref()
                } else {
                    assets.spazmatism.as_ref()
                };
                (
                    tex,
                    tex.map_or(110.0, |t| t.h / 2.0),
                    [100.0, 110.0],
                    [n.x, n.y],
                )
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
            NpcKind::GoblinTinkerer => (
                assets.goblin_tinkerer.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The bound goblin: the single tied frame over NPC_105.
            NpcKind::BoundGoblin => (
                assets.bound_goblin.as_ref(),
                assets.bound_goblin.as_ref().map_or(48.0, |t| t.h),
                [34.0, 54.0],
                [n.x - 8.0, n.y - 10.0],
            ),
            // The goblins: the 3-frame humanoid sheets (the vanilla
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
            NpcKind::Skeleton | NpcKind::UndeadMiner => (
                if n.kind == NpcKind::Skeleton {
                    assets.skeleton.as_ref()
                } else {
                    assets.undead_miner.as_ref()
                },
                assets.skeleton.as_ref().map_or(48.0, |t| t.h / 15.0),
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The goblin army's fighters: the humanoid strips (the 16-frame
            // family 26/27/28/111, the summoner's 20) — the frame height
            // rides each sheet, the display box the zombie's 38x48 shape.
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner => {
                let (tex, frames) = match n.kind {
                    NpcKind::GoblinThief => (assets.goblin_thief.as_ref(), 16.0),
                    NpcKind::GoblinWarrior => (assets.goblin_warrior.as_ref(), 16.0),
                    NpcKind::GoblinArcher => (assets.goblin_archer.as_ref(), 16.0),
                    NpcKind::GoblinPeon => (assets.goblin_peon.as_ref(), 21.0),
                    _ => (assets.goblin_summoner.as_ref(), 20.0),
                };
                (
                    tex,
                    tex.map_or(48.0, |t| t.h / frames),
                    [38.0, 48.0],
                    [n.x - 10.0, n.y - 5.0],
                )
            }
            // The Cave Bat: the 5-frame flap over the 22x18 frames.
            // The casters: the humanoid sheets — the sorcerer's strip is 3
            // frames, the Dark Caster's 6 (`Main.npcFrameCount` 29/32/45,
            // `Main.cs:67082`), the display box the zombie's 38x48 shape.
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                let (tex, frames) = match n.kind {
                    NpcKind::GoblinSorcerer => (assets.goblin_sorcerer.as_ref(), 3.0),
                    NpcKind::Tim => (assets.tim.as_ref(), 3.0),
                    _ => (assets.dark_caster.as_ref(), 6.0),
                };
                (
                    tex,
                    tex.map_or(48.0, |t| t.h / frames),
                    [38.0, 48.0],
                    [n.x - 10.0, n.y - 5.0],
                )
            }
            // The Fire Imp: the 10-frame strip (`Main.npcFrameCount[24]`).
            NpcKind::FireImp => (
                assets.fire_imp.as_ref(),
                assets.fire_imp.as_ref().map_or(48.0, |t| t.h / 10.0),
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
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
                let (tex, frames) = match n.kind {
                    NpcKind::Goldfish => (assets.goldfish.as_ref(), 6.0),
                    NpcKind::Piranha => (assets.piranha.as_ref(), 6.0),
                    _ => (assets.shark.as_ref(), 4.0),
                };
                let (bw, bh) = n.size();
                (
                    tex,
                    tex.map_or(18.0, |t| t.h / frames),
                    [bw + 4.0, bh + 2.0],
                    [n.x - 2.0, n.y - 1.0],
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
                let (bw, bh) = n.size();
                (
                    tex,
                    tex.map_or(bh, |t| t.h),
                    [bw + 4.0, bh + 4.0],
                    [n.x - 2.0, n.y - 2.0],
                )
            }
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let tex = match n.kind {
                    NpcKind::Hornet => assets.hornet.as_ref(),
                    NpcKind::MeteorHead => assets.meteor_head.as_ref(),
                    _ => assets.eater_of_souls.as_ref(),
                };
                let frames = match n.kind {
                    NpcKind::Hornet => 3.0,
                    _ => 2.0,
                };
                let fh = tex.map_or(30.0, |t| t.h / frames);
                let (bw, bh) = n.size();
                (tex, fh, [bw + 4.0, bh], [n.x - 2.0, n.y - 2.0])
            }
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => (
                match n.kind {
                    NpcKind::JungleBat => assets.jungle_bat.as_ref(),
                    NpcKind::Hellbat => assets.hellbat.as_ref(),
                    NpcKind::VoodooDemon => assets.voodoo_demon.as_ref(),
                    _ => assets.cave_bat.as_ref(),
                },
                assets.cave_bat.as_ref().map_or(18.0, |t| t.h / 5.0),
                [26.0, 18.0],
                [n.x - 12.0, n.y - 9.0],
            ),
            NpcKind::KingSlime => (
                assets.king_slime.as_ref(),
                120.0,
                [174.0, 120.0],
                [n.x - 38.0, n.y - 25.0],
            ),
            // The eye: `Main.npcFrameCount[4] = 6` frames of 100x110, the
            // frame == the hitbox (the `Main.DrawNPC` anchor).
            NpcKind::EyeOfCthulhu => (
                assets.eye_of_cthulhu.as_ref(),
                assets.eye_of_cthulhu.as_ref().map_or(110.0, |t| t.h / 6.0),
                [100.0, 110.0],
                [n.x, n.y],
            ),
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
        // `Main.DrawNPC` multiplies the light by `npc.color`; the slime
        // variants carry their `SetDefaults` colours (alpha included — the
        // premultiplied pipeline renders the translucency directly).
        let base = if n.kind == NpcKind::Slime {
            let k = n.slime.stats().color;
            [
                c[0] * k[0] as f32 / 255.0,
                c[1] * k[1] as f32 / 255.0,
                c[2] * k[2] as f32 / 255.0,
                k[3] as f32 / 255.0,
            ]
        } else {
            [c[0], c[1], c[2], 1.0]
        };
        let tint = if n.hurt_cd > 0.0 {
            [c[0].max(0.8), c[1].max(0.8), c[2].max(0.8), 1.0]
        } else {
            base
        };
        let uv = flip_uv(npc_frame_uv(tex, frame_h, n.frame()), n.facing > 0);
        if matches!(n.kind, NpcKind::DemonEye | NpcKind::WanderingEye) && n.rotation != 0.0 {
            // `Main.DrawNPC` draws every NPC at its centre with
            // `origin = frame size / 2` and `rotation = npc.rotation`
            // (`NPC.cs` draw pass); the demon eye aims along its flight
            // (`NPC.cs:74019-74031`). The mirrored quad's corners are
            // symmetric about the centre, so flipping does not move them —
            // rotate the quad about its centre, exactly like XNA does.
            let center = snap_to_screen(
                [pos[0] + size[0] * 0.5, pos[1] + size[1] * 0.5],
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
