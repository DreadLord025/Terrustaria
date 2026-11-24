//! Snap points — the gamepad-navigation anchors of the retained UI
//! (`Terraria/UI/SnapPoint.cs` + `UIElement.SetSnapPoint`/`GetSnapPoint`/
//! `GetSnapPoints`, `UIElement.cs:124-145,271-283`).
//!
//! Screen-model ruling: same flat-table shape as `framework.rs` — the C#'s
//! `_snapPoint` field is an `Option<SnapPoint>` the caller stores per element id,
//! and `GetSnapPoints`'s tree walk (`:271-283`, self first then each child in
//! append order) is a pre-order walk over the flat table's `parent` links
//! (`framework::Element.parent`), which preserves the C# order because the flat
//! table preserves `Elements`' append order (`DrawChildren` `:160-166`).
//!
//! No gamepad consumer exists yet (`UILinkPointNavigator` is unported — research
//! 18 deviation 10); this is the anchor maths + collection order the navigator
//! will read, ported faithfully and pinned by hand-computed tests.

/// `Terraria/UI/SnapPoint.cs:7-39` — a named, id-tagged anchor on an element.
/// `Calculate` (`:27-31`) places it at
/// `dimensions.Position() + offset + anchor * (dimensions.Width, dimensions.Height)`
/// where `dimensions` is the element's `GetDimensions()` rect — the
/// margin-shrunk `_dimensions` box `uistate::resolve`/`resolve_with_margins`
/// return, NOT the inner (padded) rect.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapPoint {
    /// `SnapPoint.Name` (`SnapPoint.cs:9`).
    pub name: String,
    /// `SnapPoint.Id` (`:15`) — the navigator's global link id.
    pub id: i32,
    /// `_anchor` (`:11`) — fractions of the element's size, default (0.5, 0.5).
    pub anchor: [f32; 2],
    /// `_offset` (`:13`) — pixels added after the anchor, default (0, 0).
    pub offset: [f32; 2],
    /// `Position` (`:17`) — set by [`SnapPoint::calculate`].
    pub position: [f32; 2],
}

impl SnapPoint {
    /// `UIElement.SetSnapPoint` (`UIElement.cs:124-135`): the nullable
    /// `anchor`/`offset` parameters default to `new Vector2(0.5f)` and
    /// `Vector2.Zero` (`:126-133`) before the ctor (`SnapPoint.cs:19-25`).
    pub fn set(name: &str, id: i32, anchor: Option<[f32; 2]>, offset: Option<[f32; 2]>) -> Self {
        Self {
            name: name.to_string(),
            id,
            anchor: anchor.unwrap_or([0.5, 0.5]),
            offset: offset.unwrap_or([0.0, 0.0]),
            position: [0.0, 0.0],
        }
    }

    /// `SnapPoint.Calculate` (`SnapPoint.cs:27-31`) — `rect` is the element's
    /// `GetDimensions()` `(x, y, w, h)`. The addition order matches the C#
    /// expression: `Position() + _offset + _anchor * size`.
    pub fn calculate(&mut self, rect: [f32; 4]) {
        self.position = [
            rect[0] + self.offset[0] + self.anchor[0] * rect[2],
            rect[1] + self.offset[1] + self.anchor[1] * rect[3],
        ];
    }

    /// `SnapPoint.ThisIsAHackThatChangesTheSnapPointsInfo` (`SnapPoint.cs:33-38`)
    /// — re-anchors an existing point in place (the C# keeps `Name`).
    pub fn change_info(&mut self, anchor: [f32; 2], offset: [f32; 2], id: i32) {
        self.anchor = anchor;
        self.offset = offset;
        self.id = id;
    }
}

/// `UIElement.GetSnapPoint` (`UIElement.cs:137-145`): when a point is set it is
/// **recalculated on read** (`:142`), then returned; `None` when unset.
pub fn get_snap_point(point: &mut Option<SnapPoint>, rect: [f32; 4]) -> Option<&SnapPoint> {
    if let Some(p) = point.as_mut() {
        p.calculate(rect);
    }
    point.as_ref()
}

/// `UIElement.GetSnapPoints` (`UIElement.cs:271-283`) over the flat element
/// table: the element's own point first (`:274-277`), then each child's list in
/// append order (`:278-281`). `rects` maps element id → `GetDimensions()` rect;
/// `points` maps element id → its `_snapPoint`. Roots are elements with no
/// parent in the table; the walk is pre-order DFS, so the result is exactly the
/// C#'s depth-first flatten.
pub fn get_snap_points(
    elements: &[super::framework::Element],
    points: &[(u32, SnapPoint)],
    rects: &[(u32, [f32; 4])],
) -> Vec<SnapPoint> {
    let mut out = Vec::new();
    fn walk(
        id: u32,
        elements: &[super::framework::Element],
        points: &[(u32, SnapPoint)],
        rects: &[(u32, [f32; 4])],
        out: &mut Vec<SnapPoint>,
    ) {
        // `:274-277` — this element's point, recalculated (`GetSnapPoint` :142).
        if let Some((_, p)) = points.iter().find(|(eid, _)| *eid == id) {
            let mut p = p.clone();
            if let Some((_, rect)) = rects.iter().find(|(eid, _)| *eid == id) {
                p.calculate(*rect);
            }
            out.push(p);
        }
        // `:278-281` — the children in append order, each recursed.
        for child in elements.iter().filter(|e| e.parent == Some(id)) {
            walk(child.id, elements, points, rects, out);
        }
    }
    for root in elements.iter().filter(|e| e.parent.is_none()) {
        walk(root.id, elements, points, rects, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::framework::Element;

    #[test]
    fn snap_calculate_is_position_plus_offset_plus_anchor_times_size() {
        // Hand-computed `SnapPoint.cs:30`: rect (100, 200, 80, 40),
        // anchor (0.25, 0.75), offset (3, -5):
        // x = 100 + 3 + 0.25*80 = 123; y = 200 + (-5) + 0.75*40 = 225.
        let mut p = SnapPoint::set("test", 7, Some([0.25, 0.75]), Some([3.0, -5.0]));
        p.calculate([100.0, 200.0, 80.0, 40.0]);
        assert_eq!(p.position, [123.0, 225.0]);
    }

    #[test]
    fn set_snap_point_defaults_to_the_centre_with_no_offset() {
        // `UIElement.cs:126-133`: null anchor -> (0.5, 0.5), null offset -> 0.
        let mut p = SnapPoint::set("centre", 1, None, None);
        assert_eq!(p.anchor, [0.5, 0.5]);
        assert_eq!(p.offset, [0.0, 0.0]);
        p.calculate([10.0, 20.0, 100.0, 50.0]);
        assert_eq!(p.position, [60.0, 45.0]);
    }

    #[test]
    fn get_snap_point_recalculates_on_read_and_is_none_when_unset() {
        // `UIElement.cs:137-145`: the read calculates against the CURRENT rect.
        let mut slot = Some(SnapPoint::set("btn", 2, None, None));
        let p = get_snap_point(&mut slot, [0.0, 0.0, 10.0, 10.0]).unwrap();
        assert_eq!(p.position, [5.0, 5.0]);
        let p = get_snap_point(&mut slot, [20.0, 0.0, 10.0, 10.0]).unwrap();
        assert_eq!(p.position, [25.0, 5.0]);
        let mut none: Option<SnapPoint> = None;
        assert!(get_snap_point(&mut none, [0.0; 4]).is_none());
    }

    #[test]
    fn get_snap_points_is_self_then_children_in_append_order() {
        // `UIElement.cs:271-283`: root point, then child 2's subtree, then
        // child 3 — depth-first, append order, elements without points skipped.
        let elements = [
            Element::hit([0.0, 0.0, 400.0, 400.0], 1),
            Element::child([0.0, 0.0, 100.0, 100.0], 2, 1),
            Element::child([0.0, 0.0, 100.0, 100.0], 3, 1),
            Element::child([0.0, 0.0, 10.0, 10.0], 4, 2),
        ];
        let points = vec![
            (1, SnapPoint::set("root", 10, None, None)),
            (4, SnapPoint::set("leaf", 40, None, None)),
            (3, SnapPoint::set("second", 30, None, None)),
        ];
        let rects = vec![
            (1, [0.0, 0.0, 400.0, 400.0]),
            (3, [100.0, 0.0, 100.0, 100.0]),
            (4, [10.0, 10.0, 10.0, 10.0]),
        ];
        let list = get_snap_points(&elements, &points, &rects);
        assert_eq!(
            list.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![10, 40, 30]
        );
        // Each point was recalculated against its own rect (`:142`).
        assert_eq!(list[0].position, [200.0, 200.0]);
        assert_eq!(list[1].position, [15.0, 15.0]);
        assert_eq!(list[2].position, [150.0, 50.0]);
    }

    #[test]
    fn the_hack_reanchors_in_place_and_keeps_the_name() {
        // `SnapPoint.cs:33-38`.
        let mut p = SnapPoint::set("keep", 1, None, None);
        p.change_info([1.0, 0.0], [2.0, 2.0], 99);
        assert_eq!(p.name, "keep");
        assert_eq!(p.id, 99);
        p.calculate([0.0, 0.0, 10.0, 10.0]);
        assert_eq!(p.position, [12.0, 2.0]);
    }
}
