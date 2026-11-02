//! GPU side: sprite atlas, instanced quad renderer, Vulkan device setup.

pub mod atlas;
pub mod gpu;
pub mod post;
pub mod sprite;

pub use atlas::{Atlas, AtlasImage, SheetLayout, SheetRect};
pub use gpu::Gpu;
pub use post::{
    pass_index, FilterImages, FilterParams, PassId, PostPipeline, PASS_TABLE,
};
pub use sprite::{Globals, SpriteInstance, SpriteRenderer};
