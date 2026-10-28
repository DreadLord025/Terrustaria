//! GPU side: sprite atlas, instanced quad renderer, Vulkan device setup.

pub mod atlas;
pub mod gpu;
pub mod sprite;

pub use atlas::{Atlas, AtlasImage, SheetLayout, SheetRect};
pub use gpu::Gpu;
pub use sprite::{Globals, SpriteInstance, SpriteRenderer};
