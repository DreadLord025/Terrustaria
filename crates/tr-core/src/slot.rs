//! `ReLogic.Utilities`'s slot types (`SlotId.cs`, `SlotVector.cs`) — the
//! versioned handle + the free-list pool the tracked-sound layer stores its
//! [`crate::scene`]-adjacent ids through. Ported verbatim: the 16-bit index,
//! the 15-bit version key, the active bit, the 65535-invalid sentinel and
//! the free-list threading through the inactive slots' index fields.

/// `ReLogic.Utilities.SlotId` (`SlotId.cs:3-62`): `Value` packs the slot
/// index (16 bits), the version key (bits 16-30) and the active flag (bit
/// 31); `Invalid` is the 65535 sentinel (an inactive index). `ToFloat` /
/// `FromFloat` are the bit reinterprets the NPC `localAI` storage rides
/// (`NPC.cs:40984`'s `SlotId.FromFloat(this.localAI[3])`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotId(pub u32);

impl SlotId {
    /// `SlotId.Invalid` (`SlotId.cs:6`).
    pub const INVALID: SlotId = SlotId(65535);

    const INDEX_MASK: u32 = 65535;
    const ACTIVE_MASK: u32 = 2147483648;
    const KEY_MASK: u32 = 2147418112;

    /// `SlotId.IsValid` (`SlotId.cs:17`).
    pub fn is_valid(self) -> bool {
        (self.0 & Self::INDEX_MASK) != 65535
    }

    /// `SlotId.IsActive` (`SlotId.cs:20-28`).
    pub fn is_active(self) -> bool {
        if (self.0 & Self::ACTIVE_MASK) != 0 {
            self.is_valid()
        } else {
            false
        }
    }

    /// `SlotId.Index` (`SlotId.cs:31`).
    pub fn index(self) -> u32 {
        self.0 & Self::INDEX_MASK
    }

    /// `SlotId.Key` (`SlotId.cs:34`).
    fn key(self) -> u32 {
        self.0 & Self::KEY_MASK
    }

    /// `SlotId.ToInactive(freeHead)` (`SlotId.cs:37-39`).
    fn to_inactive(self, free_head: u32) -> SlotId {
        SlotId(self.key() | free_head)
    }

    /// `SlotId.ToActive(index)` (`SlotId.cs:41-45`): the version bumps by
    /// 65536 and the active flag sets.
    fn to_active(self, index: u32) -> SlotId {
        let key = Self::KEY_MASK & (self.key() + 65536);
        SlotId(Self::ACTIVE_MASK | key | index)
    }

    /// `SlotId.ToFloat` (`SlotId.cs:55-57`): the `uint` bits reinterpreted
    /// as an `f32`.
    pub fn to_float(self) -> f32 {
        f32::from_bits(self.0)
    }

    /// `SlotId.FromFloat` (`SlotId.cs:59-61`).
    pub fn from_float(value: f32) -> SlotId {
        SlotId(value.to_bits())
    }
}

/// `ReLogic.Utilities.SlotVector<T>` (`SlotVector.cs:9-217`): the fixed
/// capacity pool whose inactive slots thread a free list through their
/// `SlotId`'s index field. `Add` takes the free head, bumps its version and
/// marks it active; `Remove` returns the slot to the list; `Has` accepts an
/// id only when its index is active AND the stored id matches — the
/// generation check that makes a stale handle (`SlotId::INVALID`-adjacent
/// reuse) read as `null` in the C#'s `GetActiveSound`.
pub struct SlotVector<T> {
    array: Vec<(SlotId, T)>,
    free_head: u32,
    used_space_length: u32,
    count: usize,
}

impl<T: Default> SlotVector<T> {
    /// `SlotVector(capacity)` + `Clear` (`SlotVector.cs:65-83`): slot `i`
    /// starts inactive with its free-list pointer at `i + 1`, the last slot
    /// pointing at the 65535 end marker.
    pub fn new(capacity: usize) -> Self {
        let mut array = Vec::with_capacity(capacity);
        for i in 0..capacity - 1 {
            array.push((SlotId(i as u32 + 1), T::default()));
        }
        array.push((SlotId(65535), T::default()));
        Self {
            array,
            free_head: 0,
            used_space_length: 0,
            count: 0,
        }
    }

    /// `SlotVector.Add` (`SlotVector.cs:85-101`): the 65535 free head is the
    /// full-pool rejection (the pool is 4096, far above the tracked layer's
    /// needs, but the guard ports verbatim).
    pub fn add(&mut self, value: T) -> SlotId {
        if self.free_head == 65535 {
            return SlotId(65535);
        }
        let free_head = self.free_head;
        let (next_free, _) = self.array[free_head as usize];
        if self.free_head >= self.used_space_length {
            self.used_space_length = self.free_head + 1;
        }
        self.free_head = next_free.index();
        let id = next_free.to_active(free_head);
        self.array[free_head as usize] = (id, value);
        self.count += 1;
        id
    }

    /// `SlotVector.Clear` (`SlotVector.cs:103-113`).
    pub fn clear(&mut self) {
        *self = Self::new(self.array.len());
    }

    /// `SlotVector.Remove` (`SlotVector.cs:115-126`).
    pub fn remove(&mut self, id: SlotId) -> bool {
        if id.is_active() {
            let index = id.index() as usize;
            let (slot, _) = self.array[index];
            self.array[index] = (slot.to_inactive(self.free_head), T::default());
            self.free_head = index as u32;
            self.count -= 1;
            true
        } else {
            false
        }
    }

    /// `SlotVector.Has(SlotId)` (`SlotVector.cs:128-139`).
    pub fn has(&self, id: SlotId) -> bool {
        let index = id.index() as usize;
        if index >= self.array.len() {
            return false;
        }
        let (slot, _) = self.array[index];
        slot.is_active() && id == slot
    }

    /// `SlotVector.Has(int index)` (`SlotVector.cs:141-150`).
    pub fn has_index(&self, index: usize) -> bool {
        if index >= self.array.len() {
            return false;
        }
        self.array[index].0.is_active()
    }

    /// `SlotVector.Count` (`SlotVector.cs:118`).
    pub fn count(&self) -> usize {
        self.count
    }

    /// `SlotVector`'s `ItemPair` walk (`:52-60` + the enumerator `:39-63`):
    /// every active `(value, id)` pair in index order — what `Update`,
    /// `GetActiveSoundCount` and `FindActiveSound` iterate.
    pub fn pairs(&self) -> impl Iterator<Item = (SlotId, &T)> {
        let used = self.used_space_length as usize;
        self.array
            .iter()
            .take(used)
            .enumerate()
            .filter(|(_, (slot, _))| slot.is_active())
            .map(|(i, (slot, value))| (SlotId(slot.0), value))
    }

    /// The `this[SlotId]` indexer (`SlotVector.cs:83-92`).
    pub fn get(&self, id: SlotId) -> Option<&T> {
        if !self.has(id) {
            return None;
        }
        Some(&self.array[id.index() as usize].1)
    }

    /// The `this[SlotId]` setter (`SlotVector.cs:93-102`).
    pub fn get_mut(&mut self, id: SlotId) -> Option<&mut T> {
        if !self.has(id) {
            return None;
        }
        Some(&mut self.array[id.index() as usize].1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pool_reuses_slots_with_version_bumps() {
        // `SlotVector.Add`/`Remove`/`Has` (`SlotVector.cs:85-139`): a freed
        // slot's next id bumps the version, so a stale handle reads dead.
        let mut pool: SlotVector<u32> = SlotVector::new(4096);
        let a = pool.add(1);
        assert!(pool.has(a) && a.is_active());
        assert_eq!(pool.count(), 1);
        assert!(pool.remove(a));
        assert!(!pool.has(a), "the stale id reads dead after Remove");
        let b = pool.add(2);
        assert_eq!(b.index(), a.index(), "the freed slot is reused");
        assert_ne!(b, a, "the version bumped");
        assert_eq!(*pool.get(b).unwrap(), 2);
        assert_eq!(pool.count(), 1);
    }

    #[test]
    fn the_full_pool_returns_the_invalid_sentinel() {
        // `SlotVector.Add`'s 65535 guard (`SlotVector.cs:86-89`).
        let mut pool: SlotVector<u32> = SlotVector::new(4);
        for i in 0..4 {
            assert!(pool.add(i).is_active());
        }
        assert_eq!(pool.add(99), SlotId(65535));
        assert_eq!(pool.count(), 4);
    }

    #[test]
    fn float_round_trip_preserves_the_bits() {
        // `SlotId.ToFloat`/`FromFloat` (`SlotId.cs:55-61`) — the NPC
        // `localAI` storage dance (`NPC.cs:40984`).
        let id = SlotId(2147549187u32);
        assert_eq!(SlotId::from_float(id.to_float()), id);
        assert_eq!(SlotId::INVALID.to_float().to_bits(), 65535);
    }

    #[test]
    fn pairs_walks_only_the_active_range() {
        let mut pool: SlotVector<u32> = SlotVector::new(4096);
        let a = pool.add(10);
        let _ = pool.add(20);
        pool.remove(a);
        let seen: Vec<u32> = pool.pairs().map(|(_, v)| *v).collect();
        assert_eq!(seen, vec![20], "the removed slot leaves the walk");
    }
}
