//! `Terraria.HitTile` (`HitTile.cs:9-477`) — the per-player mining damage
//! book: 500 slots keyed by `(x, y, hitType)` (1 = tile, 2 = wall), each
//! carrying an accumulated `damage` that breaks the tile at 100
//! (`Player.PickTile`, `Player.cs:53769`). The port keeps the slot data,
//! the LRU order and the TTL decay verbatim; the draw-side fields
//! (`animationTimeElapsed`/`animationDirection`, the `DrawFreshAnimations`
//! miner's wobble, `:335-471`) stay in the render layer's crack overlay.
//!
//! RNG (`HitTile.cs`): `HitTileObject.Clear` rolls `crackStyle` from the
//! static `HitTile.rand` (`:36-45`, the no-repeat-of-`lastCrack` loop) and
//! `AddDamage` draws `Main.rand.NextFloat()` for the wobble direction
//! (`:66`). The port's crack-style stream is the world-seeded
//! `App::mine_crack_rng` (the recorded determinism deviation, ANALYSIS
//! "The tile-crack overlay"); the wobble direction is render-only and not
//! drawn. Both draws are therefore OUT of this module — the sim book takes
//! no RNG, so the mining tick adds no draws to any stream (the C#'s
//! `Main.rand` wobble draw rides the `UpdatePlayers` swap; its absence is
//! recorded with the other render-only deviations).

/// `HitTileObject` (`HitTile.cs:11-68`) — the sim half.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitTileObject {
    pub x: i32,
    pub y: i32,
    pub damage: i32,
    /// `HitTileObject.type` — 0 = unused, 1 = tile, 2 = wall.
    pub kind: i32,
    pub time_to_live: i32,
    /// `HitTileObject.crackStyle` — rolled by the render layer on Clear
    /// (the port's `render::tile_cracks::CrackStyles`, W4-g77); the book
    /// keeps the slot free of it.
    pub crack_style: i32,
}

impl HitTileObject {
    /// `HitTileObject.Clear` (`HitTile.cs:29-46`) minus the crack-style
    /// roll (the render layer owns it — see the module doc).
    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.damage = 0;
        self.kind = 0;
        self.time_to_live = 0;
    }

    /// `HitTileObject.Prepare` (`HitTile.cs:48-53`).
    pub fn prepare(&mut self, x: i32, y: i32, hit_type: i32) {
        self.x = x;
        self.y = y;
        self.kind = hit_type;
    }

    /// `HitTileObject.SetPosition` (`HitTile.cs:55-59`).
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// `HitTileObject.AddDamage` (`HitTile.cs:61-67`) minus the wobble
    /// draw (module doc): the damage adds, the TTL resets to 60.
    pub fn add_damage(&mut self, damage_amount: i32) {
        self.damage += damage_amount;
        self.time_to_live = TIMETOLIVE;
    }
}

/// `HitTile.TILE` (`HitTile.cs:72`).
pub const TILE: i32 = 1;
/// `HitTile.WALL` (`HitTile.cs:74`).
pub const WALL: i32 = 2;
/// `HitTile.MAX_HITTILES` (`HitTile.cs:76`) — the order array is 501 long.
pub const MAX_HITTILES: usize = 500;
/// `HitTile.TIMETOLIVE` (`HitTile.cs:78`).
pub const TIMETOLIVE: i32 = 60;

/// The per-player `HitTile` book (`HitTile.cs:84-88`).
#[derive(Clone, Debug)]
pub struct HitTile {
    pub data: [HitTileObject; MAX_HITTILES + 1],
    order: [usize; MAX_HITTILES + 1],
    buffer_location: usize,
}

impl Default for HitTile {
    fn default() -> Self {
        Self::new()
    }
}

impl HitTile {
    /// `HitTile()` (`HitTile.cs:115-126`): every slot cleared, the order
    /// identity, the buffer at 0. The static `rand` reseed is the render
    /// layer's (module doc).
    pub fn new() -> Self {
        let mut data = [HitTileObject::default(); MAX_HITTILES + 1];
        let mut order = [0usize; MAX_HITTILES + 1];
        for i in 0..=MAX_HITTILES {
            data[i].clear();
            order[i] = i;
        }
        Self {
            data,
            order,
            buffer_location: 0,
        }
    }

    /// `HitTile.TryFinding` (`HitTile.cs:128-147`): the order walk for a
    /// live slot of the hit type at the cell; the first unused slot past
    /// index 0 ends the scan.
    pub fn try_finding(&self, x: i32, y: i32, hit_type: i32) -> Option<usize> {
        for i in 0..=MAX_HITTILES {
            let num = self.order[i];
            let obj = self.data[num];
            if obj.kind == hit_type {
                if obj.x == x && obj.y == y {
                    return Some(num);
                }
            } else if i != 0 && obj.kind == 0 {
                break;
            }
        }
        None
    }

    /// `HitTile.TryClearingAndPruning` (`HitTile.cs:149-157`).
    pub fn try_clearing_and_pruning(
        &mut self,
        x: i32,
        y: i32,
        hit_type: i32,
        tile_active: impl Fn(i32, i32) -> bool,
        wall_at: impl Fn(i32, i32) -> u16,
    ) {
        if let Some(num) = self.try_finding(x, y, hit_type) {
            self.clear(num);
            self.prune(tile_active, wall_at);
        }
    }

    /// `HitTile.HitObject` (`HitTile.cs:159-179`): the existing slot for
    /// the cell, or the buffer slot prepared for it.
    pub fn hit_object(&mut self, x: i32, y: i32, hit_type: i32) -> usize {
        for i in 0..=MAX_HITTILES {
            let num = self.order[i];
            let obj = self.data[num];
            if obj.kind == hit_type {
                if obj.x == x && obj.y == y {
                    return num;
                }
            } else if i != 0 && obj.kind == 0 {
                break;
            }
        }
        self.data[self.buffer_location].prepare(x, y, hit_type);
        self.buffer_location
    }

    /// `HitTile.UpdatePosition` (`HitTile.cs:181-187`) — the mannequin /
    /// weapon-rack redirects (`Player.cs:53990`, `:54004`, `:54010`).
    pub fn update_position(&mut self, tile_id: usize, x: i32, y: i32) {
        if tile_id <= MAX_HITTILES {
            self.data[tile_id].set_position(x, y);
        }
    }

    /// `HitTile.AddDamage` (`HitTile.cs:189-206`): the bounds guard, the
    /// zero-damage-on-buffer refusal, the `updateAmount: false` preview
    /// (the `DoesPickTargetTransformOnKill` read, `Player.cs:54049`) and
    /// the LRU sort. Returns the slot's damage after the add.
    pub fn add_damage(&mut self, tile_id: usize, damage_amount: i32, update_amount: bool) -> i32 {
        if tile_id > MAX_HITTILES {
            return 0;
        }
        if tile_id == self.buffer_location && damage_amount == 0 {
            return 0;
        }
        if !update_amount {
            return self.data[tile_id].damage + damage_amount;
        }
        self.data[tile_id].add_damage(damage_amount);
        self.sort_slots(tile_id);
        self.data[tile_id].damage
    }

    /// `HitTile.SortSlots` (`HitTile.cs:208-238`): the buffer slot promotes
    /// to order[0] (the old tail becomes the new buffer, cleared), any
    /// other slot bubbles up to order[1]. The C#'s `order[1] = tileId`
    /// after the not-found walk writes index 501 on a missing slot (an
    /// out-of-bounds the runtime never reaches — `AddDamage` only sorts
    /// slots `HitObject` returned); the port guards it.
    fn sort_slots(&mut self, tile_id: usize) {
        if tile_id == self.buffer_location {
            self.buffer_location = self.order[MAX_HITTILES];
            if tile_id != self.buffer_location {
                self.data[self.buffer_location].clear();
            }
            for num in (1..=MAX_HITTILES).rev() {
                self.order[num] = self.order[num - 1];
            }
            self.order[0] = self.buffer_location;
        } else {
            let mut num = 0;
            while num <= MAX_HITTILES && self.order[num] != tile_id {
                num += 1;
            }
            while num > 1 {
                let num2 = self.order[num - 1];
                self.order[num - 1] = self.order[num];
                self.order[num] = num2;
                num -= 1;
            }
            if num == 1 {
                self.order[1] = tile_id;
            }
        }
    }

    /// `HitTile.Clear` (`HitTile.cs:240-255`): the slot clears and sinks
    /// to the order tail.
    pub fn clear(&mut self, tile_id: usize) {
        if tile_id > MAX_HITTILES {
            return;
        }
        self.data[tile_id].clear();
        let mut i = 0;
        while i < MAX_HITTILES && self.order[i] != tile_id {
            i += 1;
        }
        while i < MAX_HITTILES {
            self.order[i] = self.order[i + 1];
            i += 1;
        }
        self.order[MAX_HITTILES] = tile_id;
    }

    /// `HitTile.Prune` (`HitTile.cs:257-283`): the per-slot TTL decay, then
    /// the bubble that sinks the cleared slots past the live ones. The C#
    /// resets `num = 1` ONCE before the `while (flag)` loop and never
    /// advances it — the bubble re-walks `1..499` every pass until no swap
    /// lands (a verbatim quirk; the port keeps it). The
    /// `tile_active`/`wall_at` closures stand in for the `Main.tile` reads
    /// (`:291`, `:319-331`).
    pub fn prune(
        &mut self,
        tile_active: impl Fn(i32, i32) -> bool,
        wall_at: impl Fn(i32, i32) -> u16,
    ) {
        let mut flag = false;
        for i in 0..=MAX_HITTILES {
            flag |= prune_object(&mut self.data[i], &tile_active, &wall_at);
        }
        if !flag {
            return;
        }
        let num = 1;
        while flag {
            flag = false;
            for j in num..MAX_HITTILES {
                if self.data[self.order[j]].kind == 0 && self.data[self.order[j + 1]].kind != 0 {
                    let num2 = self.order[j];
                    self.order[j] = self.order[j + 1];
                    self.order[j + 1] = num2;
                    flag = true;
                }
            }
        }
    }

    /// The live entries in the C# order walk — the crack overlay's input
    /// (the port draws the local player's book through
    /// `render/tile_cracks.rs`).
    pub fn live_entries(&self) -> impl Iterator<Item = &HitTileObject> {
        self.order
            .iter()
            .map(|&i| &self.data[i])
            .filter(|o| o.kind != 0)
    }
}

/// `HitTile.Prune(ref HitTileObject)` (`HitTile.cs:285-333`): the TTL
/// countdown and the damage decay ladder — `< 12` loses 10, `< 24` loses
/// 7, `< 36` loses 5, `< 48` loses 2 per prune; a negative damage or a
/// gone tile/wall clears the slot. Returns whether the slot cleared.
fn prune_object(
    obj: &mut HitTileObject,
    tile_active: &impl Fn(i32, i32) -> bool,
    wall_at: &impl Fn(i32, i32) -> u16,
) -> bool {
    if obj.kind == 0 {
        return false;
    }
    if obj.time_to_live <= 1 {
        obj.clear();
        return true;
    }
    obj.time_to_live -= 1;
    if (obj.time_to_live as f64) < 12.0 {
        obj.damage -= 10;
    } else if (obj.time_to_live as f64) < 24.0 {
        obj.damage -= 7;
    } else if (obj.time_to_live as f64) < 36.0 {
        obj.damage -= 5;
    } else if (obj.time_to_live as f64) < 48.0 {
        obj.damage -= 2;
    }
    if obj.damage < 0 {
        obj.clear();
        return true;
    }
    if obj.kind == TILE {
        if !tile_active(obj.x, obj.y) {
            obj.clear();
            return true;
        }
    } else if wall_at(obj.x, obj.y) == 0 {
        obj.clear();
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always_active(_: i32, _: i32) -> bool {
        true
    }
    fn no_wall(_: i32, _: i32) -> u16 {
        0
    }
    fn some_wall(_: i32, _: i32) -> u16 {
        1
    }

    /// S1-P-i29 — the accumulation: repeated hits on one cell share the
    /// slot and break at 100 (`Player.PickTile` `Player.cs:53769`).
    #[test]
    fn s1_p_i29_hits_accumulate_on_one_slot() {
        let mut book = HitTile::new();
        let id = book.hit_object(10, 20, TILE);
        assert_eq!(book.add_damage(id, 40, true), 40);
        // The same cell finds the same slot.
        let id2 = book.hit_object(10, 20, TILE);
        assert_eq!(id2, id);
        assert_eq!(book.add_damage(id2, 40, true), 80);
        assert_eq!(book.add_damage(id2, 40, true), 120, ">= 100 breaks");
        assert_eq!(book.data[id].time_to_live, 60, "each hit resets the TTL");
    }

    /// S1-P-i29 — the zero-damage-on-buffer refusal (`HitTile.cs:195-198`)
    /// and the `updateAmount: false` preview (`:199-202`).
    #[test]
    fn s1_p_i29_the_buffer_zero_guard_and_the_preview() {
        let mut book = HitTile::new();
        let id = book.hit_object(10, 20, TILE);
        assert_eq!(id, 0, "the first hit takes the buffer");
        // Zero damage on the buffer slot adds nothing.
        assert_eq!(book.add_damage(id, 0, true), 0);
        assert_eq!(book.data[id].damage, 0);
        // The preview reads without writing.
        book.add_damage(id, 30, true);
        assert_eq!(book.add_damage(id, 70, false), 100);
        assert_eq!(book.data[id].damage, 30, "the preview wrote nothing");
    }

    /// S1-P-i29 — the TTL decay ladder (`HitTile.cs:297-313`): a 40-damage
    /// hit decays 2/tick while the TTL is 48..36, then 5, 7, 10 — the
    /// damage drains to nothing if the player stops hitting.
    #[test]
    fn s1_p_i29_the_ttl_decay_ladder() {
        let mut book = HitTile::new();
        let id = book.hit_object(10, 20, TILE);
        book.add_damage(id, 40, true);
        // The first prune after the hit: TTL 60 -> 59, no decay yet.
        book.prune(always_active, no_wall);
        assert_eq!(book.data[id].damage, 40);
        assert_eq!(book.data[id].time_to_live, 59);
        // Gate V3: walk the TTL down to 48 (11 more prunes): the decay arm
        // fires on the prune that LANDS below 48 (`timeToLive--` THEN the
        // `< 48` check, `HitTile.cs:297-313`), so TTL 48 still keeps 40.
        for _ in 0..11 {
            book.prune(always_active, no_wall);
        }
        assert_eq!(book.data[id].time_to_live, 48);
        assert_eq!(book.data[id].damage, 40);
        // TTL 48 -> 47: the < 48 arm loses 2.
        book.prune(always_active, no_wall);
        assert_eq!(book.data[id].damage, 38);
        // Drain to the < 36 arm: TTL 35 loses 5.
        while book.data[id].time_to_live > 35 {
            book.prune(always_active, no_wall);
        }
        let before = book.data[id].damage;
        book.prune(always_active, no_wall);
        assert_eq!(book.data[id].damage, before - 5);
        // The slot eventually clears (damage < 0 or TTL <= 1).
        for _ in 0..60 {
            book.prune(always_active, no_wall);
        }
        assert_eq!(book.data[id].kind, 0, "the abandoned hit decays away");
    }

    /// S1-P-i29 — a gone tile clears its slot on the next prune
    /// (`HitTile.cs:319-326`); a gone wall clears the wall slot
    /// (`:327-331`).
    #[test]
    fn s1_p_i29_the_gone_tile_and_wall_clear() {
        let mut book = HitTile::new();
        let id = book.hit_object(10, 20, TILE);
        book.add_damage(id, 40, true);
        book.prune(|_, _| false, no_wall);
        assert_eq!(book.data[id].kind, 0, "the tile is gone");
        let wid = book.hit_object(10, 20, WALL);
        book.add_damage(wid, 40, true);
        book.prune(always_active, no_wall);
        assert_eq!(book.data[wid].kind, 0, "the wall is gone");
        let wid2 = book.hit_object(11, 20, WALL);
        book.add_damage(wid2, 40, true);
        book.prune(always_active, some_wall);
        assert_eq!(book.data[wid2].kind, WALL, "the wall stands");
    }

    /// S1-P-i29 — the LRU order: a fresh hit promotes to order[0], the
    /// buffer rotates to the old tail (`HitTile.cs:208-238`), and Clear
    /// sinks the slot (`:240-255`).
    #[test]
    fn s1_p_i29_the_lru_order_and_clear() {
        let mut book = HitTile::new();
        let a = book.hit_object(1, 1, TILE);
        book.add_damage(a, 10, true);
        let b = book.hit_object(2, 2, TILE);
        book.add_damage(b, 10, true);
        // Gate V3: the faithful C# `SortSlots` (`HitTile.cs:208-238`) rotates
        // the BUFFER to order[0] (order[0] = the NEW bufferLocation = the old
        // tail), and the just-hit slot lands at order[1]. So after two fresh
        // hits the newest hit `b` leads the LIVE entries at order[1], with the
        // rotated buffer at order[0] and `a` at order[2].
        assert_eq!(book.order[1], b, "the newest hit leads the live entries");
        assert_eq!(book.order[2], a);
        // Re-hitting `a` bubbles it to order[1] (the buffer stays at 0).
        let a2 = book.hit_object(1, 1, TILE);
        assert_eq!(a2, a);
        book.add_damage(a2, 10, true);
        assert_eq!(book.order[1], a);
        // Clear sinks the slot to the tail.
        book.clear(a);
        assert_eq!(book.order[MAX_HITTILES], a);
        assert_eq!(book.data[a].kind, 0);
    }

    /// S1-P-i29 — `TryClearingAndPruning` (`HitTile.cs:149-157`) finds and
    /// clears the cell's slot; `UpdatePosition` (`:181-187`) moves it.
    #[test]
    fn s1_p_i29_try_clearing_and_update_position() {
        let mut book = HitTile::new();
        let id = book.hit_object(10, 20, TILE);
        book.add_damage(id, 40, true);
        book.update_position(id, 9, 20);
        assert_eq!(book.data[id].x, 9);
        // The old cell no longer finds it; the new one does.
        assert!(book.try_finding(10, 20, TILE).is_none());
        assert_eq!(book.try_finding(9, 20, TILE), Some(id));
        book.try_clearing_and_pruning(9, 20, TILE, always_active, no_wall);
        assert_eq!(book.data[id].kind, 0);
    }
}
