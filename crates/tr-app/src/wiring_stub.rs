//! The `MechSpawn` caps — `NPC.MechSpawn` (`NPC.cs:7420-7506`) and
//! `Item.MechSpawn` (`Item.cs:49385-49406`): the same-type entity counts
//! within the distance rings must stay under 3 (near), 6 (mid) and 10
//! (global). The NPC rings are 200/600 px, the item rings 300/800 px.
//!
//! The M7-WIRE-2 request carries the C# ids to cap on (`cap_ids` — the
//! spawn id itself when empty, plus the seasonal item ids and style 63's
//! 165 veto), and the app counts its entities through their C# ids.

/// `NPC.MechSpawn`'s same-type group table (`NPC.cs:7428-7460`): a request
/// for one member counts every NPC of the group. The dragonfly arm is
/// `NPCID.Sets.IsDragonfly[type]` — the six dragonfly ids 595-600.
fn npc_group(id: u16) -> u16 {
    match id {
        74 | 297 | 298 => 74,            // the birds
        46 | 540 | 303 | 337 => 46,      // the bunnies (party/getGoodWorld variants)
        362 | 363 | 364 | 365 => 362,    // the ducks
        602 | 603 => 602,                // the seagulls
        608 | 609 => 608,
        616 | 617 => 616,                // the turtles
        55 | 230 => 55,                  // the goldfish (and the gold variant)
        595 | 596 | 597 | 598 | 599 | 600 => 595, // the dragonflies
        other => other,
    }
}

/// `NPC.MechSpawn` (`NPC.cs:7420-7506`): `entities` yields the active NPCs'
/// positions and C# NPC ids. `cap_ids` is the request's cap list — the C#
/// `MechSpawn` argument for every id in it must pass (an empty list means
/// "cap on `request_id`"). The distance is measured from the entity's
/// `position` (the port's `n.x/n.y` top-left, as in the C#).
pub fn npc_mech_spawn<I>(entities: I, cap_ids: &[u16], request_id: u16, x: f32, y: f32) -> bool
where
    I: IntoIterator<Item = (f32, f32, u16)>,
{
    let mut ids: Vec<u16> = cap_ids.to_vec();
    if ids.is_empty() {
        ids.push(request_id);
    }
    // Collect the entity positions once — every cap id scans the same list.
    let positions: Vec<(f32, f32, u16)> = entities.into_iter().collect();
    for id in ids {
        let group = npc_group(id);
        let (mut global, mut near, mut mid) = (0u32, 0u32, 0u32);
        for &(ex, ey, etype) in &positions {
            if npc_group(etype) != group {
                continue;
            }
            global += 1;
            let dist = ((ex - x).powi(2) + (ey - y).powi(2)).sqrt();
            if dist < 200.0 {
                near += 1;
            }
            if dist < 600.0 {
                mid += 1;
            }
        }
        if near >= 3 || mid >= 6 || global >= 10 {
            return false;
        }
    }
    true
}

/// `Item.MechSpawn` (`Item.cs:49385-49406`): the item rings are 300/800 px and
/// there are no groups — each cap id must pass on its own exact type count.
pub fn item_mech_spawn<I>(items: I, cap_ids: &[u16], request_id: u16, x: f32, y: f32) -> bool
where
    I: IntoIterator<Item = (f32, f32, u16)>,
{
    let mut ids: Vec<u16> = cap_ids.to_vec();
    if ids.is_empty() {
        ids.push(request_id);
    }
    let positions: Vec<(f32, f32, u16)> = items.into_iter().collect();
    for id in ids {
        let (mut global, mut near, mut mid) = (0u32, 0u32, 0u32);
        for &(ix, iy, itype) in &positions {
            if itype != id {
                continue;
            }
            global += 1;
            let dist = ((ix - x).powi(2) + (iy - y).powi(2)).sqrt();
            if dist < 300.0 {
                near += 1;
            }
            if dist < 800.0 {
                mid += 1;
            }
        }
        if near >= 3 || mid >= 6 || global >= 10 {
            return false;
        }
    }
    true
}
