//! Section 6 (pointer 5): NPCs — `WorldFile.SaveNPCs:1703` (write) /
//! `WorldFile.LoadNPCs:2884` (read).
//!
//! Wire shape (`SaveNPCs:1703-1749`): three lists sharing one slot cursor.
//! 1. The shimmered town-NPC bitmap's truth indices (`>= 268`): an `i32`
//!    count followed by that many `i32` indices out of `bool[NPCID.Count]`.
//! 2. The town NPCs — **terminator-delimited**: every record starts with the
//!    `active` bool that also terminates the list, then `i32 netID` (`< 190`
//!    files carry a legacy name instead), `string GivenName`, the `Vector2`
//!    position, `bool homeless`, the home `i32 X, i32 Y`, a `BitsByte` whose
//!    bit 0 (`townNPC`) gates the `i32 townNpcVariationIndex` that follows
//!    (`>= 213`) and `bool homelessDespawn` (`>= 315`).
//! 3. The `SavesAndLoads` non-town NPCs (the four lunar towers): `bool active,
//!    i32 netID, Vector2`, terminator-delimited (`>= 140`).
//!
//! The file is untrusted input: a shimmer count or index outside
//! `0..=NPCID.Count`, a record count past the 201 `Main.npc` slots or any
//! short read is refused rather than trusted, no record pre-allocates from a
//! hostile length, and every arm mirrors a C# branch so a crafted stream
//! cannot desynchronise the cursor. The app-side halves of this section
//! (`crate::npc::from_wld` / `to_wld`, the world-dependent `FixAgainstExploits`
//! re-validation and the town rooms) live in the `tr-app` owner modules.
//!
//! One load-side block of `LoadNPCs` is deliberately not ported: the
//! `version < 251` recompute of the eight `unlocked*Spawn` flags
//! (`LoadNPCs:2943-2954`) is live spawning state, which this phase leaves
//! untouched — a `<251` file loads with those flags at their defaults.

use super::{SectionData, SectionPayload};
use crate::binio::{BitsByte, Reader, Writer};

/// `Main.maxNPCs` — the `Main.npc[]` array `SaveNPCs`/`LoadNPCs` index; the
/// town and tower lists share its 201 slots in file order
/// (`LoadNPCs:2890`, `:2928`). A file carrying more records than slots is not
/// a vanilla file (the C# would throw `IndexOutOfRangeException`).
pub const MAX_NPC_SLOTS: usize = 201;
/// `NPCID.Count` (`NPCID.cs:12581`): the length of both `NPC.ShimmeredTownNPCs`
/// (`NPC.cs:95163`) and `TownRoomManager._hasRoom` (`TownRoomManager.cs:12`).
pub const NPCID_COUNT: usize = 697;
/// `NPCID.Sets.SavesAndLoads` (`NPCID.cs:10449`): the four lunar towers
/// (422 Vortex, 507 Nebula, 493 Stardust, 517 Solar) — the only non-town NPCs
/// the section persists.
pub const SAVES_AND_LOADS: [i32; 4] = [422, 507, 493, 517];
/// `NPCID.TravellingMerchant` (368): the town NPC the save loop excludes
/// (`SaveNPCs:1718`); the port has no such kind, so the gate is a documented
/// no-op there.
pub const TRAVELLING_MERCHANT: i32 = 368;

/// One town-NPC record (`SaveNPCs:1714-1735`). The wire form starts with the
/// record-leading `active` bool, which doubles as the list terminator
/// (`LoadNPCs:2895`); the surrounding list is what terminates, so the flag
/// itself is not a field of the stored record.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldTownNpc {
    /// `NPC.netID` (`ReadInt32` for `>= 190`; older files store a legacy name
    /// — [`WldTownNpc::legacy_name`] — which the app glue maps).
    pub net_id: i32,
    /// The `< 190` name-string form of the net id (`NPCID.FromLegacyName`'s
    /// input, `LoadNPCs:2897-2900`). `Some` only on a legacy file — where
    /// `net_id` stays 0; the modern record leaves it `None`.
    pub legacy_name: Option<String>,
    /// `NPC.GivenName` (`SaveNPCs:1719`) — never null on the wire, so the glue
    /// writes an empty string for an NPC that never got one.
    pub given_name: String,
    /// `NPC.position` in world pixels (`SaveNPCs:1722-1723`).
    pub x: f32,
    pub y: f32,
    /// `NPC.homeless` (`SaveNPCs:1724`). Independent of the home pair: a
    /// homeless record may still carry the (now stale) home tiles.
    pub homeless: bool,
    pub home_x: i32,
    pub home_y: i32,
    /// `BitsByte` bit 0 — `nPC.townNPC` (`SaveNPCs:1726`). Town-list records
    /// always have it set; the bit's presence is what gates the variation
    /// index, so the flag is carried on the record.
    pub town_npc: bool,
    /// `nPC.townNpcVariationIndex` (`SaveNPCs:1731`), read only when bit 0 is
    /// set and the file is `>= 213`.
    pub variation_index: i32,
    /// `NPC.homelessDespawn` (read `>= 315`, `LoadNPCs:2917-2920`).
    pub homeless_despawn: bool,
}

/// One `SavesAndLoads` non-town NPC (a lunar tower, `SaveNPCs:1738-1747`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldTower {
    pub net_id: i32,
    /// The `< 190` name-string form of the net id, as on [`WldTownNpc`].
    pub legacy_name: Option<String>,
    pub x: f32,
    pub y: f32,
}

/// The three lists in write order (`SaveNPCs:1703-1749`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldNpcs {
    /// `NPC.ShimmeredTownNPCs` truth indices, `>= 268`
    /// (`SaveNPCs:1705-1713`).
    pub shimmered: Vec<i32>,
    pub town: Vec<WldTownNpc>,
    /// `>= 140` (`SaveNPCs:1738-1748`).
    pub towers: Vec<WldTower>,
}

impl WldNpcs {
    pub fn is_empty(&self) -> bool {
        self.shimmered.is_empty() && self.town.is_empty() && self.towers.is_empty()
    }
}

pub fn implemented() -> bool {
    true
}

pub fn payload(value: WldNpcs) -> SectionPayload {
    SectionPayload::Npcs(value)
}

/// `LoadNPCs:2884-2955`. The town list's leading bool is both a record's
/// `active` and the list's terminator, and the tower list is absent below 140
/// — those two gates are the whole version story of this section.
pub fn read(r: &mut Reader, version: i32) -> Option<WldNpcs> {
    let mut data = WldNpcs::default();
    // `LoadNPCs:2886-2893` — the shimmered bitmap.
    if version >= 268 {
        let count = r.i32()?;
        if !(0..=NPCID_COUNT as i32).contains(&count) {
            return None; // `array.Count(value: true)` cannot exceed the array
        }
        for _ in 0..count {
            let index = r.i32()?;
            if !(0..NPCID_COUNT as i32).contains(&index) {
                return None; // `ShimmeredTownNPCs[reader.ReadInt32()] = true`
            }
            data.shimmered.push(index);
        }
    }
    // `LoadNPCs:2894-2922`: a `true` starts a record, a `false` ends the list.
    while r.boolean()? {
        data.town.push(read_town_record(r, version)?);
        if data.town.len() > MAX_NPC_SLOTS {
            return None; // `Main.npc[num2]` has 201 slots
        }
    }
    // `LoadNPCs:2923-2942`: the same shape for the towers.
    if version >= 140 {
        while r.boolean()? {
            data.towers.push(read_tower_record(r, version)?);
            if data.town.len() + data.towers.len() > MAX_NPC_SLOTS {
                return None; // the two lists share the slot cursor
            }
        }
    }
    Some(data)
}

/// One town record (`LoadNPCs:2896-2921`).
fn read_town_record(r: &mut Reader, version: i32) -> Option<WldTownNpc> {
    let (net_id, legacy_name) = read_npc_id(r, version)?;
    let given_name = r.cs_string_lossy()?;
    let x = r.f32()?;
    let y = r.f32()?;
    let homeless = r.boolean()?;
    let home_x = r.i32()?;
    let home_y = r.i32()?;
    // `LoadNPCs:2906-2909`: the BitsByte only exists from 213, and bit 0 is
    // what tells the reader whether the variation index follows.
    let mut town_npc = false;
    let mut variation_index = 0;
    if version >= 213 {
        let bits = r.bits_byte()?;
        town_npc = bits.bit(0);
        if town_npc {
            variation_index = r.i32()?;
        }
    }
    let homeless_despawn = version >= 315 && r.boolean()?;
    Some(WldTownNpc {
        net_id,
        legacy_name,
        given_name,
        x,
        y,
        homeless,
        home_x,
        home_y,
        town_npc,
        variation_index,
        homeless_despawn,
    })
}

/// One tower record (`LoadNPCs:2924-2935`): the net id and the position.
fn read_tower_record(r: &mut Reader, version: i32) -> Option<WldTower> {
    let (net_id, legacy_name) = read_npc_id(r, version)?;
    Some(WldTower {
        net_id,
        legacy_name,
        x: r.f32()?,
        y: r.f32()?,
    })
}

/// `LoadNPCs:2897-2901` / `:2929-2933`: the record's net id — an `i32` from
/// version 190, a legacy name below it (`NPCID.FromLegacyName`'s input).
fn read_npc_id(r: &mut Reader, version: i32) -> Option<(i32, Option<String>)> {
    if version >= 190 {
        Some((r.i32()?, None))
    } else {
        Some((0, Some(r.cs_string_lossy()?)))
    }
}

/// `SaveNPCs:1703-1749`. The C# writer is single-version (it always emits the
/// current shape); the `version` parameter keeps this half honest for the
/// older layouts the reader accepts, which is what the round-trip tests
/// exercise. Which live NPCs are town NPCs / towers is the app glue's half and
/// lives in `crate::npc::to_wld`.
pub fn write(w: &mut Writer, data: &SectionData, version: i32) {
    // `SaveNPCs:1705-1713`: the truth indices of the shimmered bitmap.
    if version >= 268 {
        w.i32(data.npcs.shimmered.len() as i32);
        for &index in &data.npcs.shimmered {
            w.i32(index);
        }
    }
    // `SaveNPCs:1714-1735`: the town records.
    for npc in &data.npcs.town {
        w.boolean(true); // the record's leading bool
        write_npc_id(w, npc.net_id, npc.legacy_name.as_deref(), version);
        w.cs_string(&npc.given_name);
        w.f32(npc.x);
        w.f32(npc.y);
        w.boolean(npc.homeless);
        w.i32(npc.home_x);
        w.i32(npc.home_y);
        // `SaveNPCs:1725-1733`: bit 0 gates the variation index.
        let mut bits = BitsByte::default();
        bits.set(0, npc.town_npc);
        if version >= 213 {
            w.bits_byte(bits);
            if bits.bit(0) {
                w.i32(npc.variation_index);
            }
        }
        // `SaveNPCs:1734`.
        if version >= 315 {
            w.boolean(npc.homeless_despawn);
        }
    }
    w.boolean(false); // `SaveNPCs:1736`: the town list's terminator
    // `SaveNPCs:1737-1748`: the towers.
    if version >= 140 {
        for tower in &data.npcs.towers {
            w.boolean(true);
            write_npc_id(w, tower.net_id, tower.legacy_name.as_deref(), version);
            w.f32(tower.x);
            w.f32(tower.y);
        }
        w.boolean(false); // `SaveNPCs:1748`
    }
}

/// `SaveNPCs:1718` / `:1739`: the record's net id — an `i32` from 190, the
/// legacy name below it (an empty string when the struct carries none).
fn write_npc_id(w: &mut Writer, net_id: i32, legacy_name: Option<&str>, version: i32) {
    if version >= 190 {
        w.i32(net_id);
    } else {
        w.cs_string(legacy_name.unwrap_or(""));
    }
}

pub fn empty(data: &SectionData) -> bool {
    data.npcs.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    /// A `.NET`-style string: 7-bit length prefix + UTF-8 payload.
    fn cs_string(s: &str) -> Vec<u8> {
        let mut v = vec![s.len() as u8];
        v.extend_from_slice(s.as_bytes());
        v
    }

    fn i32_bytes(v: i32) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    fn zeros(n: usize) -> Vec<u8> {
        vec![0u8; n]
    }

    fn town(
        net_id: i32,
        name: &str,
        x: f32,
        y: f32,
        homeless: bool,
        home: (i32, i32),
        variation: i32,
        despawn: bool,
    ) -> WldTownNpc {
        WldTownNpc {
            net_id,
            legacy_name: None,
            given_name: name.into(),
            x,
            y,
            homeless,
            home_x: home.0,
            home_y: home.1,
            town_npc: true,
            variation_index: variation,
            homeless_despawn: despawn,
        }
    }

    fn tower(net_id: i32, x: f32, y: f32) -> WldTower {
        WldTower {
            net_id,
            legacy_name: None,
            x,
            y,
        }
    }

    /// A section exercising all three lists: three shimmered indices, three
    /// town records (a homed NPC, a homeless one with a stale home, a named
    /// one) and two tower records.
    fn fixture_npcs() -> WldNpcs {
        WldNpcs {
            shimmered: vec![4, 68, 300],
            town: vec![
                town(22, "Andrew", -100.5, 200.25, false, (60, 100), 2, false),
                town(17, "", 0.0, 0.0, true, (0, 0), 0, true),
                town(228, "Daisy", 1024.0, 768.0, false, (61, 101), 7, false),
            ],
            towers: vec![tower(507, 3200.0, 640.0), tower(422, 64.5, 128.25)],
        }
    }

    fn section(data: &SectionData, version: i32) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, data, version);
        w.into_inner()
    }

    fn data(npcs: WldNpcs) -> SectionData {
        SectionData {
            npcs,
            ..Default::default()
        }
    }

    /// Reads a whole section and reports the cursor, so a test can assert that
    /// the read consumed exactly the bytes the write produced.
    fn read_at(bytes: &[u8], version: i32) -> (Option<WldNpcs>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, version), r.pos())
    }

    /// The empty encoding of every layout: the shimmer count (`>= 268`), the
    /// town terminator, the tower terminator (`>= 140`).
    fn empty_bytes(version: i32) -> Vec<u8> {
        let mut v = Vec::new();
        if version >= 268 {
            v.extend_from_slice(&zeros(4));
        }
        v.push(0);
        if version >= 140 {
            v.push(0);
        }
        v
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_4_5_the_section_is_implemented() {
        assert!(implemented(), "phase 4 replaces the stub in place");
        assert!(empty(&data(WldNpcs::default())));
        assert!(!empty(&data(fixture_npcs())));
    }

    // --- DW-4.1: the three lists round-trip -------------------------------

    #[test]
    fn test_DW_4_1_the_three_lists_round_trip_byte_equal() {
        let npcs = fixture_npcs();
        let bytes = section(&data(npcs.clone()), VER);
        let (read_back, pos) = read_at(&bytes, VER);
        assert_eq!(read_back.as_ref(), Some(&npcs), "the struct survives");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
        // Byte-equal means the reader's output re-writes to the same section.
        assert_eq!(section(&data(read_back.unwrap()), VER), bytes);
    }

    #[test]
    fn test_DW_4_1_the_town_record_bytes_are_hand_computed() {
        // `SaveNPCs:1716-1734` for one Merchant: active, netID 17, "Bob",
        // (100.5, 200.25), homeless false, home (60, 100), bit0 → variation 3,
        // homelessDespawn — then the list terminator.
        let npcs = WldNpcs {
            town: vec![town(17, "Bob", 100.5, 200.25, false, (60, 100), 3, true)],
            ..Default::default()
        };
        let mut want = vec![1u8]; // the record's leading bool
        want.extend_from_slice(&i32_bytes(17));
        want.extend_from_slice(&cs_string("Bob"));
        want.extend_from_slice(&100.5f32.to_le_bytes());
        want.extend_from_slice(&200.25f32.to_le_bytes());
        want.push(0); // homeless
        want.extend_from_slice(&i32_bytes(60));
        want.extend_from_slice(&i32_bytes(100));
        want.push(1); // the BitsByte: bit 0
        want.extend_from_slice(&i32_bytes(3)); // variation index
        want.push(1); // homelessDespawn
        want.push(0); // the town list's terminator
        // The empty shimmer list and the empty tower list around it.
        want.splice(0..0, i32_bytes(0));
        want.push(0);
        assert_eq!(section(&data(npcs), VER), want);
    }

    #[test]
    fn test_DW_4_1_the_shimmered_list_is_the_truth_indices() {
        // `SaveNPCs:1706-1713`: the count of true entries, then their indices.
        let bytes = section(
            &data(WldNpcs {
                shimmered: vec![4, 68, 300],
                ..Default::default()
            }),
            VER,
        );
        assert_eq!(&bytes[0..4], &i32_bytes(3));
        assert_eq!(&bytes[4..8], &i32_bytes(4));
        assert_eq!(&bytes[8..12], &i32_bytes(68));
        assert_eq!(&bytes[12..16], &i32_bytes(300));
        assert_eq!(bytes.len(), 16 + 2, "the index list plus the terminators");
    }

    #[test]
    fn test_DW_4_1_the_written_section_reads_back_itself() {
        // The load path (`wld::load_sections`) reads the writer's own output
        // through the section registry, not just this module's seam.
        let npcs = fixture_npcs();
        let bytes = section(&data(npcs.clone()), VER);
        let mut r = Reader::new(&bytes);
        let payload = crate::wld::sections::npcs::read(&mut r, VER).expect("reads");
        assert_eq!(payload, npcs);
        assert_eq!(r.pos(), bytes.len());
    }

    #[test]
    fn test_DW_4_1_the_section_survives_a_whole_wld_round_trip() {
        // The registry flip end to end: a `.wld` whose NPC section carries
        // records is written by `wld::save` and walked by `load_sections`,
        // which now actually reads (and end-offset checks) the section.
        let world = crate::world::World::new("N".into(), 8, 6);
        let npcs = fixture_npcs();
        let file = crate::wld::to_bytes(&world, &data(npcs.clone()), 1).expect("writes");
        let table = crate::wld::SectionTable::open(&file).expect("opens");
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.npcs, npcs, "the section lands back in the bag");
    }

    // --- DW-4.2: the version gates ----------------------------------------

    #[test]
    fn test_DW_4_2_the_empty_encoding_is_version_gated() {
        // `>= 268` writes the shimmer count; `>= 140` the tower terminator.
        assert_eq!(section(&data(WldNpcs::default()), 326), empty_bytes(326));
        assert_eq!(section(&data(WldNpcs::default()), 267), empty_bytes(267));
        assert_eq!(section(&data(WldNpcs::default()), 139), empty_bytes(139));
        assert_eq!(section(&data(WldNpcs::default()), 100), empty_bytes(100));
        assert_eq!(empty_bytes(326).len(), 6);
        assert_eq!(empty_bytes(267).len(), 2);
        assert_eq!(empty_bytes(139).len(), 1);
    }

    #[test]
    fn test_DW_4_2_the_terminator_terminates_the_town_list() {
        // The zero-NPC file: an immediate `false` is the whole town list
        // (`LoadNPCs:2894-2895`), and nothing distinguishes it from a file
        // whose records were all dropped.
        let bytes = empty_bytes(VER);
        let (npcs, pos) = read_at(&bytes, VER);
        assert_eq!(npcs, Some(WldNpcs::default()));
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_DW_4_2_a_record_that_never_terminates_is_a_short_read() {
        // The record-leading bool doubles as the terminator: a stream that
        // ends inside a record is read as another record and fails mid-field.
        let mut bytes = zeros(4); // shimmered: none
        bytes.push(1); // a record starts
        bytes.extend_from_slice(&i32_bytes(22));
        bytes.extend_from_slice(&cs_string("Andrew"));
        let (npcs, _) = read_at(&bytes, VER);
        assert!(npcs.is_none(), "the truncated record refuses");
    }

    #[test]
    fn test_DW_4_2_a_legacy_name_file_reads_the_name_field() {
        // `LoadNPCs:2897-2901` below 190: the record's net id is a name. The
        // hand-built v189 layout, then the round-trip at that version (no
        // BitsByte below 213, no `homelessDespawn` below 315, no tower list
        // below 140).
        let mut bytes = Vec::new();
        bytes.push(1); // a record starts
        bytes.extend_from_slice(&cs_string("Arms Dealer"));
        bytes.extend_from_slice(&cs_string("Joe"));
        bytes.extend_from_slice(&32.0f32.to_le_bytes());
        bytes.extend_from_slice(&64.0f32.to_le_bytes());
        bytes.push(0); // homeless
        bytes.extend_from_slice(&i32_bytes(10));
        bytes.extend_from_slice(&i32_bytes(20));
        bytes.push(0); // the town list's terminator
        bytes.push(0); // the tower list's terminator (`LoadNPCs:2924`, >= 140)
        let (npcs, pos) = read_at(&bytes, 189);
        assert_eq!(pos, bytes.len());
        let want = WldNpcs {
            town: vec![WldTownNpc {
                net_id: 0,
                legacy_name: Some("Arms Dealer".into()),
                given_name: "Joe".into(),
                x: 32.0,
                y: 64.0,
                homeless: false,
                home_x: 10,
                home_y: 20,
                town_npc: false,
                variation_index: 0,
                homeless_despawn: false,
            }],
            ..Default::default()
        };
        assert_eq!(npcs, Some(want.clone()));
        assert_eq!(section(&data(want), 189), bytes);
    }

    #[test]
    fn test_DW_4_2_the_213_variation_gate() {
        // Below 213 the BitsByte is not on the wire, so a record with bit 0
        // set carries no variation index at all (`LoadNPCs:2906-2909`).
        for (version, want_variation) in [(326, 3), (213, 3), (212, 0)] {
            let npcs = WldNpcs {
                town: vec![town(17, "x", 0.0, 0.0, false, (0, 0), 3, false)],
                ..Default::default()
            };
            let bytes = section(&data(npcs), version);
            let (read_back, pos) = read_at(&bytes, version);
            assert_eq!(pos, bytes.len(), "version {version} consumes its bytes");
            assert_eq!(
                read_back.unwrap().town[0].variation_index,
                want_variation,
                "version {version}"
            );
        }
    }

    #[test]
    fn test_DW_4_2_a_cleared_town_npc_bit_reads_no_variation() {
        // A foreign file may clear bit 0: the record is still a town record,
        // but no variation index follows (`SaveNPCs:1727` gates the write on
        // the same bit).
        let mut npcs = WldNpcs {
            town: vec![town(17, "x", 0.0, 0.0, false, (0, 0), 3, false)],
            ..Default::default()
        };
        npcs.town[0].town_npc = false;
        let bytes = section(&data(npcs.clone()), VER);
        let (read_back, pos) = read_at(&bytes, VER);
        assert_eq!(pos, bytes.len());
        let record = &read_back.as_ref().unwrap().town[0];
        assert!(!record.town_npc, "bit 0 cleared");
        assert_eq!(record.variation_index, 0, "no index on the wire");
        // The stored index does not survive: the writer follows the bit, as
        // `SaveNPCs:1727` does, so the record comes back without it.
        npcs.town[0].variation_index = 0;
        assert_eq!(read_back, Some(npcs));
    }

    #[test]
    fn test_DW_4_2_the_315_homeless_despawn_gate() {
        // `LoadNPCs:2917-2920`: the field only exists from 315.
        for (version, present) in [(326, true), (315, true), (314, false)] {
            let npcs = WldNpcs {
                town: vec![town(17, "x", 0.0, 0.0, true, (0, 0), 0, true)],
                ..Default::default()
            };
            let bytes = section(&data(npcs), version);
            let (read_back, pos) = read_at(&bytes, version);
            assert_eq!(pos, bytes.len(), "version {version}");
            assert_eq!(
                read_back.unwrap().town[0].homeless_despawn,
                present,
                "version {version}"
            );
        }
    }

    #[test]
    fn test_DW_4_2_the_tower_list_is_absent_below_140() {
        // Below 140 the tower list is not written at all (`SaveNPCs:1738`'s
        // `LoadNPCs:2923` gate). The record itself is the pre-190 name form
        // (`LoadNPCs:2929-2932`): a pre-190 file has no net id field, so the
        // name is the whole record and the codec leaves `net_id` at 0 — the
        // app glue resolves the kind from the name.
        let npcs = WldNpcs {
            towers: vec![WldTower {
                net_id: 0,
                legacy_name: Some("Lunar Tower Nebula".into()),
                x: 3200.0,
                y: 640.0,
            }],
            ..Default::default()
        };
        assert_eq!(section(&data(npcs.clone()), 139), vec![0]);
        let at_140 = section(&data(npcs.clone()), 140);
        assert_eq!(
            at_140.len(),
            1 + 1 + cs_string("Lunar Tower Nebula").len() + 8 + 1,
            "the v140 layout"
        );
        assert_eq!(
            &at_140[..3],
            &[0, 1, 18],
            "town terminator + record + the name's length prefix"
        );
        let (read_back, pos) = read_at(&at_140, 140);
        assert_eq!(read_back, Some(npcs));
        assert_eq!(pos, at_140.len());
        // Below 140 the tower list is not read at all: the same stream stops
        // right after the town terminator and leaves the tower bytes
        // unconsumed (`LoadNPCs:2923`'s gate) — which is why a section's
        // version and its bytes must agree.
        let (read_back, pos) = read_at(&at_140, 139);
        assert_eq!(read_back, Some(WldNpcs::default()));
        assert_eq!(pos, 1, "the cursor stops at the town terminator");
        assert_eq!(&at_140[pos..], &at_140[1..], "the towers are not consumed");
    }

    // --- DW-4.4: unported records keep the cursor in step ----------------

    #[test]
    fn test_DW_4_4_an_unported_tower_kind_parses_and_lands_on_the_terminator() {
        // The lunar towers have no ported `NpcKind`, so the record is decoded
        // here and discarded by the glue (`npc::from_wld`) — the section must
        // still consume it exactly. A trailer records where the cursor lands.
        let mut bytes = zeros(4); // shimmered: none
        bytes.push(0); // the town terminator
        bytes.push(1); // a tower record starts
        bytes.extend_from_slice(&i32_bytes(507)); // Nebula — unported
        bytes.extend_from_slice(&3200.0f32.to_le_bytes());
        bytes.extend_from_slice(&640.0f32.to_le_bytes());
        bytes.push(1); // a second tower record
        bytes.extend_from_slice(&i32_bytes(9_999_999)); // a hostile net id
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.push(0); // the tower terminator
        let trailer = b"TRAILER";
        let mut stream = bytes.clone();
        stream.extend_from_slice(trailer);
        let (npcs, pos) = read_at(&stream, VER);
        assert_eq!(
            npcs.unwrap().towers,
            vec![tower(507, 3200.0, 640.0), tower(9_999_999, 1.0, 2.0)]
        );
        assert_eq!(pos, bytes.len(), "the cursor lands on the terminator");
        assert_eq!(&stream[pos..], trailer);
    }

    #[test]
    fn test_DW_4_4_a_tower_record_short_read_refuses() {
        let mut bytes = zeros(4);
        bytes.push(0); // the town terminator
        bytes.push(1); // a tower record starts
        bytes.extend_from_slice(&i32_bytes(422));
        assert!(read_at(&bytes, VER).0.is_none());
    }

    // --- the defensive arms ----------------------------------------------

    #[test]
    fn an_implausible_shimmer_count_is_refused() {
        // `NPC.ShimmeredTownNPCs` has 697 entries: a count above that is not
        // a vanilla file (`array.Count(value: true)` cannot produce it), and
        // a negative one is a hostile count.
        let mut bytes = i32_bytes(698);
        bytes.extend_from_slice(&zeros(4 * 698));
        assert!(read_at(&bytes, VER).0.is_none());
        assert!(read_at(&i32_bytes(-1), VER).0.is_none());
    }

    #[test]
    fn an_out_of_range_shimmer_index_is_refused() {
        // `ShimmeredTownNPCs[reader.ReadInt32()] = true` — an index past the
        // array would throw in the C#.
        let bytes = [i32_bytes(1).as_slice(), &i32_bytes(697), &[0u8, 0]].concat();
        assert!(read_at(&bytes, VER).0.is_none());
    }

    #[test]
    fn a_town_record_surplus_over_the_slot_count_is_refused() {
        // `Main.npc[num2]` has 201 slots; a 202-record list is not vanilla.
        let mut bytes = zeros(4); // shimmered: none
        for _ in 0..=MAX_NPC_SLOTS {
            bytes.push(1);
            bytes.extend_from_slice(&i32_bytes(22));
            bytes.extend_from_slice(&cs_string("A"));
            bytes.extend_from_slice(&0.0f32.to_le_bytes());
            bytes.extend_from_slice(&0.0f32.to_le_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&i32_bytes(0));
            bytes.extend_from_slice(&i32_bytes(0));
            bytes.push(1); // the BitsByte: bit 0
            bytes.extend_from_slice(&i32_bytes(0)); // variation index
            bytes.push(0); // homelessDespawn
        }
        assert!(read_at(&bytes, VER).0.is_none());
    }

    #[test]
    fn a_hostile_string_prefix_does_not_allocate() {
        // A 7-bit prefix claiming 4 MB with no payload: a short read, never a
        // multi-megabyte allocation (`LoadNPCs:2902`'s `ReadString`).
        let mut bytes = zeros(4);
        bytes.push(1);
        bytes.extend_from_slice(&i32_bytes(22));
        bytes.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x7F]);
        assert!(read_at(&bytes, VER).0.is_none());
    }

    #[test]
    fn the_position_floats_survive_the_wire() {
        // f32 round-trips bit-exactly, negative and full-range alike
        // (`SaveNPCs:1722-1723` writes the raw `Vector2`).
        let npcs = WldNpcs {
            town: vec![town(17, "p", -1.5, 65536.0, false, (0, 0), 0, false)],
            ..Default::default()
        };
        let bytes = section(&data(npcs), VER);
        let (read_back, pos) = read_at(&bytes, VER);
        assert_eq!(pos, bytes.len());
        let record = &read_back.unwrap().town[0];
        assert_eq!((record.x, record.y), (-1.5f32, 65536.0f32));
    }

    // --- DW-4.5: the citations --------------------------------------------

    /// The house rule's citation probe: the source carries the anchors the
    /// golden rule demands for every ported block.
    #[test]
    fn test_DW_4_5_the_citations_are_present() {
        let src = include_str!("npcs.rs");
        // The DW item's named anchors, in the house style.
        assert!(src.contains("WorldFile.SaveNPCs:1703"), "the writer's anchor");
        assert!(src.contains("WorldFile.LoadNPCs:2884"), "the reader's anchor");
        for anchor in [
            "// `LoadNPCs:2886-2893` — the shimmered bitmap.",
            "// `LoadNPCs:2894-2922`: a `true` starts a record",
            "// `SaveNPCs:1705-1713`: the truth indices of the shimmered bitmap.",
            "// `SaveNPCs:1725-1733`: bit 0 gates the variation index.",
            "// `SaveNPCs:1736`: the town list's terminator",
            "// `SaveNPCs:1737-1748`: the towers.",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
