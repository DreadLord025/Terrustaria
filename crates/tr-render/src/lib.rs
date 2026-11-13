//! GPU side: sprite atlas, instanced quad renderer, Vulkan device setup.

pub mod atlas;
pub mod clip;
pub mod font_atlas;
pub mod gpu;
pub mod post;
pub mod sprite;
pub mod water_mask;

#[cfg(test)]
mod test_gpu;

pub use atlas::{Atlas, AtlasImage, SheetLayout, SheetRect};
pub use clip::{intersect as clip_intersect, ClipStack, NO_CLIP};
pub use font_atlas::{FontAtlas, FontPageData, PageRect};
pub use gpu::Gpu;
pub use post::{
    pass_index, FilterImages, FilterPass, FilterParams, PassId, PostPipeline, MAX_CHAIN,
    PASS_TABLE,
};
pub use sprite::{Globals, SpriteInstance, SpriteRenderer};
pub use water_mask::{MaskRect, WaterMask};
