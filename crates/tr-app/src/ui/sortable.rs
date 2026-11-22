//! The sortable list wrapper — W4-d83 (research/18b row 64):
//! `UISortableElement` (`GameContent/UI/States/UISortableElement.cs:5-22`,
//! the census's one States/ class). A bare `UIElement` carrying an
//! `OrderIndex` the `UIList`'s manual sort reads through `CompareTo`.

/// `UISortableElement` (`UISortableElement.cs:5-22`), research/18b row 64
/// (W4-d83): the manual-order wrapper the creative/journey lists put around
/// their entries (`IManuallyOrderedUIElement` consumers). The whole class
/// is the index + the compare.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortableElement {
    /// `OrderIndex` (`:7`) — the public sort key, set by the ctor (`:11`).
    pub order_index: i32,
}

impl SortableElement {
    /// The ctor (`:9-12`).
    pub fn new(index: i32) -> Self {
        Self { order_index: index }
    }

    /// `CompareTo` (`:14-21`): another `UISortableElement` compares by
    /// `OrderIndex` ascending; anything else falls to the base
    /// `UIElement.CompareTo` (which returns 0 — `UIElement.cs`'s default).
    /// The port models the "other" case as `None` (the caller keeps its
    /// own order); `Some(ord)` is the index comparison.
    pub fn compare_to(&self, other: Option<&SortableElement>) -> Option<std::cmp::Ordering> {
        other.map(|o| self.order_index.cmp(&o.order_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    /// `CompareTo` (`UISortableElement.cs:14-21`): ascending index order,
    /// equal indices compare Equal, a foreign object takes the base arm.
    #[test]
    fn the_sort_compares_indices_ascending() {
        let a = SortableElement::new(3);
        let b = SortableElement::new(7);
        assert_eq!(a.compare_to(Some(&b)), Some(Ordering::Less));
        assert_eq!(b.compare_to(Some(&a)), Some(Ordering::Greater));
        assert_eq!(a.compare_to(Some(&SortableElement::new(3))), Some(Ordering::Equal));
        assert_eq!(a.compare_to(None), None, "the base.CompareTo arm");
        // The list's manual sort: a stable sort by the key.
        let mut items = [SortableElement::new(5), SortableElement::new(1), SortableElement::new(3)];
        items.sort_by(|x, y| x.order_index.cmp(&y.order_index));
        assert_eq!(items.map(|e| e.order_index), [1, 3, 5]);
    }
}
