//! The `TileBatch` layered-batch sort (S1-G-11, research/41 D-3/D-11).
//!
//! XNA's `SpriteSortMode.Deferred` does **not** sort: it flushes sprites in
//! submission order (`Immediate` does the same per draw; only `Texture`,
//! `BackToFront` and `FrontToBack` reorder, and the Terraria sources use none
//! of them — the whole game runs `Deferred`/`Immediate`). The vanilla
//! composite-order change that the atlas would otherwise lose comes from
//! `TileBatch`'s **layered** mode instead (`TileBatch.cs:35-67`, `:421-536`,
//! `:672-725`):
//!
//! * `SetLayer(layer, stack)` arms a one-shot `LayerStack = layer << 8 | stack`
//!   for the next draw (`TileBatch.cs:235-255`);
//! * a draw whose texture differs from the current batch **without** an armed
//!   layer auto-increments the layer (`TileBatch.cs:433-436` — this is what
//!   lifts glow/flame overlays above the tile sprites);
//! * draws are bucketed per `(LayerStack, Texture)` (`SwitchBatch`/`CreateBatch`,
//!   `:485-536`), keeping push order inside a bucket;
//! * `End()` sorts the buckets by `SortKey = (LayerStack << 16) | textureId`
//!   (`:49`, `:674`), where `textureId` is the texture's **first-use ordinal
//!   within the pass** (`:513-521`), then flushes them in that order.
//!
//! The port reproduces that on top of the single atlas: instances carry a
//! `sort` key and the draw ranges are stably reordered by it before upload
//! (`sprite.rs::draw_ranges`). Instances with `sort == 0` keep push order
//! relative to each other; the sort is stable, so an unlayered range is
//! byte-identical to the pre-S1-G-11 output (the frozen default).
//!
//! Two documented deviations from the C# mechanism:
//!
//! 1. **Texture ids.** The vanilla id is the runtime first-use ordinal of a
//!    `Texture2D` reference (load-order dependent, and the painted variants go
//!    through `TilePaintSystemV2`'s render-target cache). The port assigns
//!    stable per-sheet ids ([`crate::sprite::SpriteRenderer::sheet_batch_id`])
//!    so the sort is deterministic across runs and atlas repacks — the
//!    ordering *between layers* is exact, the ordering between two different
//!    sheets inside one layer follows the port's stable id order instead of
//!    the vanilla load order (ANALYSIS "The atlas composite order (S1-G-11)").
//! 2. **`LayerBatchKey.Equals` ignores `Texture`** (`TileBatch.cs:75-78`) while
//!    `GetHashCode` includes it (`:80-83`): same-layer different-texture draws
//!    merge into one bucket only when the two references' hash codes collide
//!    — nondeterministic in the vanilla itself. The port keys by
//!    `(LayerStack, Texture)` always, the collision-free reading.

/// `TileDrawing`'s layer constants (`TileDrawing.cs:791-794`).
pub const LAYER_LIQUID_BEHIND_TILES: u32 = 0;
pub const LAYER_BEHIND_TILES: u32 = 1;
pub const LAYER_TILES: u32 = 2;
pub const LAYER_OVER_TILES: u32 = 3;

/// `WaterfallManager`'s own pass-local layers (`WaterfallManager.cs:1178-1179`):
/// its `Begin`/`End` is a separate `TileBatch` pass, so these restart at 0.
pub const WATERFALL_LAYER_RAIN: u32 = 0;
pub const WATERFALL_LAYER_FALLS: u32 = 1;

/// `TileBatch.SetLayer` (`TileBatch.cs:235-242`): `layer < 2^24`, stack a byte.
pub fn layer_stack(layer: u32, stack: u8) -> u32 {
    debug_assert!(layer < 1 << 24, "Max Layer Exceeded");
    (layer << 8) | stack as u32
}

/// `LayerBatch.SortKey` (`TileBatch.cs:49`): `(LayerStack << 16) | textureId`.
/// `texture_id` is the batch texture's first-use ordinal (u16 in the C#; the
/// port's sheet ids are u32, so the composition is widened to u64 — same
/// lexicographic `(layer, stack, texture)` order).
pub fn sort_key(layer_stack: u32, texture_id: u32) -> u64 {
    ((layer_stack as u64) << 16) | texture_id as u64
}

/// The armed-layer state between draws — `TileBatch.GetNextSpriteIndex`'s
/// `_nextLayerStack` handling (`TileBatch.cs:421-439`). Pure so the state
/// machine is unit-testable without a renderer.
#[derive(Default, Clone, Copy, Debug)]
pub struct BatchCursor {
    /// The one-shot layer armed by `SetLayer` (`_nextLayerStack`).
    pub next_layer: Option<u32>,
    /// The current batch's `LayerStack` (`_currentBatchKey.LayerStack`).
    pub current_layer: u32,
    /// The current batch's texture id (`_currentBatchKey.Texture`); `None`
    /// until the first draw (the C#'s null `Texture2D` reference).
    pub current_texture: Option<u32>,
}

impl BatchCursor {
    /// `SetLayerStack` (`TileBatch.cs:244-255`): arm the next draw's layer.
    pub fn set_layer(&mut self, stack: u32) {
        self.next_layer = Some(stack);
    }

    /// The layer a draw with `texture` lands on (`TileBatch.cs:425-437`):
    /// the armed layer wins; else a texture change bumps the current layer
    /// by one; else the draw joins the current batch. Consumes the arming.
    pub fn take_layer(&mut self, texture: u32) -> u32 {
        let layer = if let Some(armed) = self.next_layer.take() {
            armed
        } else if self.current_texture != Some(texture) {
            self.current_layer + 1
        } else {
            self.current_layer
        };
        self.current_layer = layer;
        self.current_texture = Some(texture);
        layer
    }
}

/// Stable-sort instance indices by their `sort` key (0 = unlayered, keeps
/// push order among the zeros — `TileBatch.End`'s layered flush,
/// `TileBatch.cs:672-725`, applied to the port's fused range). Pure: takes
/// the keys, returns the draw order. A key list that is already ascending
/// (the frozen default: all zeros) returns the identity permutation.
pub fn sorted_draw_order(keys: &[u64]) -> Vec<u32> {
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by_key(|&i| keys[i as usize]);
    order
}

/// Whether the sort would move anything (all-zero / already-ascending keys
/// mean the upload can stay the pre-S1-G-11 straight copy).
pub fn needs_reorder(keys: &[u64]) -> bool {
    keys.windows(2).any(|w| w[0] > w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_stack_and_sort_key_match_the_csharp_composition() {
        // TileBatch.cs:241 — (layer << 8) | stack.
        assert_eq!(layer_stack(LAYER_TILES, 0), 0x200);
        assert_eq!(layer_stack(LAYER_TILES, 1), 0x201);
        // TileBatch.cs:49 — (LayerStack << 16) | Texture.
        assert_eq!(sort_key(0x200, 5), 0x0200_0005);
        // Layer dominates texture: any texture on layer 3 sorts after every
        // texture on layer 2 (the glow-over-tiles invariant).
        assert!(sort_key(layer_stack(3, 0), 0) > sort_key(layer_stack(2, 1), u16::MAX as u32));
        // Stack breaks ties inside a layer before the texture id does
        // (glow stack 1 over every stack-0 tile of the same layer).
        assert!(sort_key(layer_stack(2, 1), 0) > sort_key(layer_stack(2, 0), u16::MAX as u32));
    }

    #[test]
    fn armed_layer_wins_then_texture_change_auto_increments() {
        // TileBatch.cs:425-437.
        let mut c = BatchCursor::default();
        c.set_layer(layer_stack(LAYER_TILES, 0));
        assert_eq!(c.take_layer(10), 0x200);
        // Same texture, no arming: same batch.
        assert_eq!(c.take_layer(10), 0x200);
        // Texture change without SetLayer: layer + 1 (the glow lift).
        assert_eq!(c.take_layer(11), 0x201);
        assert_eq!(c.take_layer(11), 0x201);
        // Re-arming returns to the named layer regardless of texture.
        c.set_layer(layer_stack(LAYER_TILES, 0));
        assert_eq!(c.take_layer(12), 0x200);
        // Back to the previous texture without arming bumps again — the
        // C# compares against the CURRENT batch only, not a history.
        assert_eq!(c.take_layer(10), 0x201);
    }

    #[test]
    fn stable_sort_keeps_push_order_inside_a_batch() {
        // Three buckets: layer-2 texture 7, layer-2 texture 3, layer-3
        // texture 7. Within-bucket order must survive the sort.
        let keys = [
            sort_key(0x200, 7), // 0
            sort_key(0x200, 3), // 1
            sort_key(0x200, 7), // 2
            sort_key(0x300, 7), // 3
            sort_key(0x200, 3), // 4
        ];
        assert!(needs_reorder(&keys));
        assert_eq!(sorted_draw_order(&keys), vec![1, 4, 0, 2, 3]);
    }

    #[test]
    fn unlayered_ranges_are_the_identity_permutation() {
        // The frozen default: every sort key 0 keeps push order exactly.
        let keys = [0u64; 9];
        assert!(!needs_reorder(&keys));
        assert_eq!(sorted_draw_order(&keys), (0..9).collect::<Vec<u32>>());
        assert_eq!(sorted_draw_order(&[]), Vec::<u32>::new());
    }
}
