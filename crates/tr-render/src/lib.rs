//! GPU side: sprite atlas, instanced quad renderer, Vulkan device setup.

pub mod animated_tiles;
pub mod atlas;
pub mod batch;
pub mod chest_frames;
pub mod clip;
pub mod composite;
pub mod font_atlas;
pub mod gpu;
pub mod post;
pub mod quality_light;
pub mod sprite;
pub mod targets;
pub mod tile_visibility;
pub mod water_mask;

#[cfg(test)]
mod test_gpu;

pub use animated_tiles::{
    final_light, is_tile_biome_sightable, is_tile_dangerous, is_tile_spelunkable, light_override,
    shine_color, should_tile_shine, tile_draw_gate, tile_shine2, vision_changer_light, DrawGate,
    VisionDust, VisionLight, BIOME_DUST_GATE, DANGER_DUST_GATE, SPELUNKER_DUST_GATE,
};
pub use atlas::{Atlas, AtlasImage, SheetLayout, SheetRect};
pub use chest_frames::{chest_anchor, chest_frame_adjust, ChestFrameAdjust};
pub use clip::{intersect as clip_intersect, ClipStack, NO_CLIP};
pub use composite::TargetCompositor;
pub use font_atlas::{FontAtlas, FontPageData, PageRect};
pub use gpu::{offscreen_target_on, read_texture_rgba, Gpu};
pub use post::{
    pass_index, FilterImages, FilterPass, FilterParams, PassId, PostPipeline, MAX_CHAIN,
    PASS_TABLE,
};
pub use sprite::{BlendMode, Globals, SpriteInstance, SpriteRenderer};
pub use targets::{
    init_target_layout, is_partially_offscreen, layer_composite_position, render_to_targets,
    render_to_targets_plan, sky_target_active, target_transition, LayerCamera, LayerUpdate,
    RenderTargetFamily, SceneLayer, ScreenTarget, TargetLayout, TargetTransition,
    WorldSceneLayerTarget, MAX_WORLD_VIEW_SIZE, RENDER_TARGET_MAX_HIDEF, RENDER_TARGET_MAX_REACH,
};
pub use water_mask::{MaskRect, WaterMask};
