//! The `UIElement` hit order (`Terraria/UI/UIElement.cs:324-352`) — the
//! back-to-front child walk every screen's click table performs, ported once as
//! pure code over a flat element list.
//!
//! Screen-model ruling (plan §"Chosen Approach"): this is the **flat** case. The
//! C#'s recursion (`:341`, the hit child is walked again) is a no-op on the
//! port's screens — each screen's hit table is one flat list of rects — so the
//! recursion is not modelled and the flatten must preserve the C#'s *append*
//! order (`DrawChildren`'s `:160-166` draws in order; `GetElementAt` walks it
//! backwards, so the last-appended child wins a click). The declared-but-never-set
//! `PassThroughMouseInteraction` (`:45`, read at `:333`) has no port: no screen
//! relies on it in 1.4.5.8, so the first hit always wins.
//!
//! The **event bubble** (`UIElement.cs:414-547`) *is* modelled: every `On*`
//! handler in the C# raises its own delegate and then recurses into `Parent`
//! (`:420-423`, `:432-435`, …), and [`bubble_chain`] reproduces that walk over
//! the flat table's `parent` links. The leaf control therefore reaches its
//! container's handlers — the `UICharacterListItem`/`UIWorldListItem` button
//! rows bubbling to the card panel (`UICharacterListItem.cs:71-121`). Ancestor
//! `IsMouseHovering` is that same walk: `MouseOver` sets the flag and then
//! recurses (`:510-521`), so a hovered leaf makes every member of the chain
//! hovered.
//!
//! `IgnoresMouseInteraction` (`:43`) is modelled per element: the C#'s two
//! behavioural users (`UIWorldGenDebug.cs:851` disabled buttons, `:1322` the global
//! hide) are debug-screen territory, but the decorative users (labels inside
//! composite buttons, icon overlays — 50 sites under `GameContent/UI/`) are the
//! ported screens' shapes, so the flag is a first-class field here.

use super::in_rect;

/// One clickable rect with the identity the pointer machine compares
/// (`UserInterface.InputPointerCache` holds element references; the port's ids are
/// the `u32` assigned here).
///
/// `UIElement.IgnoresMouseInteraction` (`UIElement.cs:43`): when set the element is
/// invisible to `GetElementAt` but still drawn — the decorative/child overlays the
/// C# marks this way (`UIBestiaryEntryIcon.cs:24`, `GroupOptionButton.cs:179`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Element {
    /// The element's outer rect (`GetDimensions`, `UIElement.cs:385-388`).
    pub rect: [f32; 4],
    /// The identity the pointer machine's `LastDown`/`LastClicked` compare.
    pub id: u32,
    /// `IgnoresMouseInteraction` (`UIElement.cs:43`).
    pub ignores_mouse: bool,
    /// `UIElement.Parent` (`UIElement.cs:52`) — the ancestor a raised event
    /// bubbles to (`:420-423`). `None` is the C#'s `Parent == null` (a root or
    /// a flat element), where the recursion stops.
    pub parent: Option<u32>,
}

impl Element {
    /// A plain clickable element (`IgnoresMouseInteraction == false`, no
    /// parent — the flat rows the screens have always used).
    pub fn hit(rect: [f32; 4], id: u32) -> Self {
        Self {
            rect,
            id,
            ignores_mouse: false,
            parent: None,
        }
    }

    /// A child of the element `parent` (`UIElement.Parent`) — the four
    /// `UICharacterListItem`/`UIWorldListItem` buttons under their card panel
    /// (`UICharacterListItem.cs:71-121`), so a click on one bubbles to the card.
    pub fn child(rect: [f32; 4], id: u32, parent: u32) -> Self {
        Self {
            rect,
            id,
            ignores_mouse: false,
            parent: Some(parent),
        }
    }

    /// `UIElement.ContainsPoint` (`:354-357`) — `Rectangle.Contains`, the
    /// inclusive left/top and exclusive right/bottom edges `ui::in_rect` ports.
    pub fn contains_point(&self, point: glam::Vec2) -> bool {
        in_rect(point, self.rect)
    }
}

/// `UIElement.GetElementAt` (`UIElement.cs:324-352`), the flat case: the children
/// are walked **back to front** (`for (num = Elements.Count - 1; num >= 0; num--)`,
/// `:327`), an `IgnoresMouseInteraction` element is skipped (`:330`), and the first
/// hit wins — the C#'s `break` on `!PassThroughMouseInteraction` (`:333`), which
/// every element satisfies because the flag is never set.
///
/// Returns the winning element's id, exactly the object reference the C# returns
/// (`pointer.InputPointerCache.LastDown`/`LastClicked` compare it).
pub fn get_element_at(elements: &[Element], point: glam::Vec2) -> Option<u32> {
    for element in elements.iter().rev() {
        if element.ignores_mouse {
            continue;
        }
        if element.contains_point(point) {
            return Some(element.id);
        }
    }
    None
}

/// `UIElement`'s child→parent event bubble (`UIElement.cs:414-547`): each `On*`
/// handler raises its own delegate and then, while `Parent != null`, recurses
/// into the parent (`:420-423`'s `this.Parent.LeftMouseDown(evt)`, `:432-435`
/// `LeftMouseUp`, `:444-447` `LeftClick`, `:456-459` `LeftDoubleClick`,
/// `:468-471`/`:480-483`/`:492-495` the right button, `:517-520` `MouseOver`,
/// `:530-533` `MouseOut`, `:542-545` `ScrollWheel`). This returns the ordered id
/// chain the event is raised on — `hit` first, then each ancestor up to the
/// root — so a caller can run the leaf handler and then the container handlers
/// the C# would have reached.
///
/// The walk is bounded by `elements.len()` and stops on a parent id that is not
/// in `elements` or on a repeat: the C# cannot loop (a `Parent` reference always
/// points at a live element), but a malformed flat table must not hang.
pub fn bubble_chain(elements: &[Element], hit: u32) -> Vec<u32> {
    let mut chain = vec![hit];
    let mut current = hit;
    for _ in 0..elements.len() {
        let Some(parent) = elements
            .iter()
            .find(|e| e.id == current)
            .and_then(|e| e.parent)
        else {
            break;
        };
        // A `Parent` reference always points at a live element in the C#; a
        // flat table can dangle or cycle, so require the parent to exist and be
        // new before stepping.
        if chain.contains(&parent) || !elements.iter().any(|e| e.id == parent) {
            break;
        }
        chain.push(parent);
        current = parent;
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    use glam::Vec2;

    const NEW: u32 = 1;
    const BACK: u32 = 2;
    const CARD_0: u32 = 3;

    #[test]
    fn dw_2_1_back_to_front_the_last_appended_element_wins() {
        // `UIElement.cs:327-333`: the reverse walk means the *last* child in the
        // list takes the click wherever two rects overlap — the C#'s `break` on
        // the first hit.
        let overlapping = [Element::hit([0.0, 0.0, 100.0, 100.0], CARD_0), Element::hit(
            [0.0, 0.0, 100.0, 100.0],
            BACK,
        )];
        assert_eq!(get_element_at(&overlapping, Vec2::new(50.0, 50.0)), Some(BACK));
        // Empty and a miss.
        assert_eq!(get_element_at(&[], Vec2::new(10.0, 10.0)), None);
        assert_eq!(get_element_at(&overlapping, Vec2::new(150.0, 10.0)), None);
    }

    #[test]
    fn dw_2_1_ignores_mouse_is_invisible_to_the_hit_but_still_addressable() {
        // `:330` — the element is skipped, so the element behind it takes the click.
        let elements = [
            Element::hit([0.0, 0.0, 100.0, 100.0], CARD_0),
            Element {
                rect: [0.0, 0.0, 100.0, 100.0],
                id: NEW,
                ignores_mouse: true,
                parent: None,
            },
        ];
        assert_eq!(get_element_at(&elements, Vec2::new(50.0, 50.0)), Some(CARD_0));
        // An `Ignores`-only list has no hit at all (`:343-351`'s tail, reached
        // through the flat flavour: nothing left to return).
        assert_eq!(
            get_element_at(
                &[Element {
                    rect: [0.0, 0.0, 100.0, 100.0],
                    id: NEW,
                    ignores_mouse: true,
                    parent: None,
                }],
                Vec2::new(50.0, 50.0)
            ),
            None
        );
    }

    #[test]
    fn dw_2_1_a_zero_sized_rect_is_never_a_hit() {
        // `Rectangle.Contains` on a degenerate rect: the inclusive left/top still
        // matches the corner, the exclusive right/bottom does not — so a
        // zero-width/height rect can only be hit at no point.
        let zero_w = Element::hit([10.0, 20.0, 0.0, 50.0], NEW);
        assert!(!zero_w.contains_point(Vec2::new(10.0, 40.0)));
        let zero_h = Element::hit([10.0, 20.0, 50.0, 0.0], BACK);
        assert!(!zero_h.contains_point(Vec2::new(20.0, 20.0)));
    }

    #[test]
    fn dw_2_1_the_edges_follow_rectangle_contains() {
        // `in_rect`'s contract, asserted through the element: inclusive left/top,
        // exclusive right/bottom.
        let e = Element::hit([10.0, 20.0, 40.0, 60.0], NEW);
        assert!(e.contains_point(Vec2::new(10.0, 20.0)));
        assert!(e.contains_point(Vec2::new(49.9, 79.9)));
        assert!(!e.contains_point(Vec2::new(50.0, 40.0)));
        assert!(!e.contains_point(Vec2::new(30.0, 80.0)));
    }

    #[test]
    fn the_event_bubble_is_the_hit_element_then_its_ancestors() {
        // `UIElement.cs:420-423` etc.: each `On*` raises its own delegate and
        // then recurses into `Parent` while `Parent != null`.
        let table = [
            Element::hit([0.0, 0.0, 400.0, 400.0], 1), // the card panel (a root)
            Element::child([10.0, 10.0, 20.0, 20.0], 2, 1), // a button under the card
            Element::child([40.0, 10.0, 20.0, 20.0], 3, 2), // a label under the button
        ];
        // A leaf button's event reaches its panel and beyond (`:420-423`).
        assert_eq!(bubble_chain(&table, 3), vec![3, 2, 1]);
        assert_eq!(bubble_chain(&table, 2), vec![2, 1]);
        // A flat element (no parent) raises only on itself (`:420`'s false arm).
        assert_eq!(bubble_chain(&table, 1), vec![1]);
        // The same chain *is* the ancestor-`IsMouseHovering` set: `MouseOver`
        // sets the flag then recurses (`:510-521`), so hovering 3 hovers 2 and 1.
        assert!(bubble_chain(&table, 3).contains(&1));
    }

    #[test]
    fn the_event_bubble_stops_at_a_missing_parent_and_cannot_cycle() {
        // A parent id absent from the table: the C# cannot produce this (a
        // `Parent` reference is never stale), but the walk must not dangle.
        let dangling = [Element::child([0.0, 0.0, 10.0, 10.0], 7, 999)];
        assert_eq!(bubble_chain(&dangling, 7), vec![7]);
        // A malformed cycle must terminate.
        let cyclic = [
            Element::child([0.0, 0.0, 10.0, 10.0], 4, 5),
            Element::child([0.0, 0.0, 10.0, 10.0], 5, 4),
        ];
        assert_eq!(bubble_chain(&cyclic, 4), vec![4, 5]);
    }

    #[test]
    fn dw_2_3_a_double_click_needs_the_same_element_both_times() {
        // The pointer machine's element identity feeds `InputPointerCache.
        // LastClicked` (`UserInterface.cs:236`): a second press on a *different*
        // id is a plain press, never a double-click.
        let mut machine = crate::ui::pointer::PointerState::default();
        use crate::ui::pointer::{PointerButton, PointerEvent};
        let press = |m: &mut crate::ui::pointer::PointerState, t, id| {
            m.press(t, PointerButton::Left, Some(id))
        };
        let release = |m: &mut crate::ui::pointer::PointerState, t, id| {
            m.release(t, PointerButton::Left, Some(id))
        };
        // Click card 0.
        assert_eq!(press(&mut machine, 1000.0, CARD_0), Some(PointerEvent::Down));
        assert_eq!(release(&mut machine, 1050.0, CARD_0), Some(PointerEvent::Click));
        // A press on a *different* element inside the window is a plain
        // MouseDown — and, per `:236`, it does not clear `LastClicked`, but it
        // *does* move `LastTimeDown` forward (`:241`), so the window is measured
        // from the most recent press of either element.
        assert_eq!(press(&mut machine, 1200.0, BACK), Some(PointerEvent::Down));
        // 550 ms past that press: outside the window, card 0 is a MouseDown.
        assert_eq!(press(&mut machine, 1750.0, CARD_0), Some(PointerEvent::Down));
        // A fresh click then a same-element press inside the window doubles.
        assert_eq!(release(&mut machine, 1800.0, CARD_0), Some(PointerEvent::Click));
        assert_eq!(press(&mut machine, 1900.0, CARD_0), Some(PointerEvent::DoubleClick));
    }
}
