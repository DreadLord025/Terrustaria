//! The `MechSpawn` caps — `NPC.MechSpawn` (`NPC.cs:7420-7505`) and
//! `Item.MechSpawn` (`Item.cs:49385-49405`): the same-type entity counts
//! within the distance rings must stay under 3 (near), 6 (mid) and 10
//! (global). The NPC rings are 200/600 px, the item rings 300/800 px.

/// `NPC.MechSpawn` (`NPC.cs:7420-7505`): `entities` yields the active NPCs'
/// positions and C# NPC ids. The same-type groups (the bird 74/297/298, the
/// slime 46/540/303/337, the bat 362-365, the penguin 602/603, 608/609,
/// 616/617, the bunny 55/230, the dragonflies) merge — the port's kinds map
/// 1:1 to their C# ids, so the merge groups only matter once those NPCs land.
pub fn npc_mech_spawn<I>(entities: I, x: f32, y: f32) -> bool
where
    I: IntoIterator<Item = (f32, f32, u16)>,
{
    let (mut global, mut near, mut mid) = (0u32, 0u32, 0u32);
    for (ex, ey, etype) in entities {
        if etype as i32 != 1 && etype as i32 != 2 {
            // The group tables for the other statue NPCs arrive with
            // M4-NPC-1's families; only the Blue Slime (1) and the Demon Eye
            // (2) are spawnable now.
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
    !(near >= 3 || mid >= 6 || global >= 10)
}

/// `Item.MechSpawn` (`Item.cs:49385-49405`): the item rings are 300/800 px.
pub fn item_mech_spawn<I>(items: I, x: f32, y: f32) -> bool
where
    I: IntoIterator<Item = (f32, f32, u16)>,
{
    let (mut global, mut near, mut mid) = (0u32, 0u32, 0u32);
    for (ix, iy, itype) in items {
        global += 1;
        let dist = ((ix - x).powi(2) + (iy - y).powi(2)).sqrt();
        if dist < 300.0 {
            near += 1;
        }
        if dist < 800.0 {
            mid += 1;
        }
    }
    !(near >= 3 || mid >= 6 || global >= 10)
}
