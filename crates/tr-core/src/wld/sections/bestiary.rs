//! Section 10 (pointer 9): bestiary — `WorldFile.SaveBestiary:3396` /
//! `WorldFile.LoadBestiary:3402`, whose bytes are
//! `BestiaryUnlocksTracker.Save/Load` (`BestiaryUnlocksTracker.cs:13-25`, the
//! Save body `:14-18`, the Load body `:20-25`):
//! three sub-lists, all keyed by bestiary **credit-id string** (never a net
//! id — `NPC.GetBestiaryCreditId`'s `ContentSamples.NpcBestiaryCreditIdsBy-
//! NpcNetIds` value, `ContentSamples.cs:862-873`).
//!
//! Wire shape, in this exact order (`BestiaryUnlocksTracker.cs:13-18`):
//! 1. **kills** — `NPCKillsTracker.Save:59-70`: `i32 count`, then per entry
//!    `(string credit id, i32 kill count)`;
//! 2. **sights** — `NPCWasNearPlayerTracker.Save:64-74`: `i32 count`, then one
//!    `string` per credit id;
//! 3. **chats** — `NPCWasChatWithTracker.Save:54-64`: the same shape.
//!
//! The section is gated `>= 210` by the loader (`LoadWorld_Version2:1832`
//! — the registry's `gate_210`); below it the C#'s
//! `LoadBestiaryForVersionsBefore210:3407` calls
//! `FillBasedOnVersionBefore210` (`BestiaryUnlocksTracker.cs:48-50`), whose
//! body is empty, so an old file simply leaves the tracker empty and this
//! codec is never asked to read anything. The creative section that follows
//! rides its own `>= 220` gate (`:1844`).
//!
//! **Save-side contract:** `write` reads [`SectionData`] (the frozen
//! `SAVE_DEFS` closure), so the caller materializes the bag with [`to_wld`]
//! before invoking `wld::to_bytes`/`wld::save` — a world whose bestiary was
//! loaded from a `.wld` must not save an empty section.
//!
//! The file is untrusted input: a negative sub-list count is refused rather
//! than read as the C# `for` loop's zero iterations, and a count larger than
//! the remaining bytes is answered by the short read. No sub-list
//! pre-allocates from its hostile length — the port holds no
//! `NPCID.Count`-sized array for these trackers (the C# side is an unbounded
//! `Dictionary`/`HashSet`), so every record is pushed and the `Reader`'s
//! EOF is the only bound.

use super::{SectionData, SectionPayload};
use crate::bestiary::WorldBestiary;
use crate::binio::{Reader, Writer};

/// The three credit-id trackers in write order: kills (`i32 count` + pairs),
/// sights (`i32 count` + strings), chats (`i32 count` + strings).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldBestiary {
    pub kills: Vec<(String, i32)>,
    pub sights: Vec<String>,
    pub chats: Vec<String>,
}

impl WldBestiary {
    pub fn is_empty(&self) -> bool {
        self.kills.is_empty() && self.sights.is_empty() && self.chats.is_empty()
    }
}

pub fn implemented() -> bool {
    true
}

/// `NPCKillsTracker.Load:72-81`: the count, then the `(string, i32)` pairs.
/// The pairs land in the wire list unchanged (including duplicates — the
/// C#'s dictionary folds them at `_killCountsByNpcId[key] = value`, which is
/// [`WorldBestiary`]'s load half, not the codec's).
fn read_kills(r: &mut Reader) -> Option<Vec<(String, i32)>> {
    let count = r.i32()?;
    if count < 0 {
        return None; // the C# `for` would read it as no records; the port refuses
    }
    let mut kills = Vec::new();
    for _ in 0..count {
        let id = r.cs_string_lossy()?;
        let count = r.i32()?;
        kills.push((id, count));
    }
    Some(kills)
}

/// `NPCWasNearPlayerTracker.Load:76-84` / `NPCWasChatWithTracker.Load:66-74`:
/// the count, then one credit id per entry (`HashSet.Add` deduplicates).
fn read_credit_ids(r: &mut Reader) -> Option<Vec<String>> {
    let count = r.i32()?;
    if count < 0 {
        return None; // as `read_kills`
    }
    let mut ids = Vec::new();
    for _ in 0..count {
        ids.push(r.cs_string_lossy()?);
    }
    Some(ids)
}

/// `BestiaryUnlocksTracker.Load:22-24` — the three sub-lists in the C#'s load
/// order, each with its own `Load`.
pub fn read(r: &mut Reader, _version: i32) -> Option<WldBestiary> {
    let kills = read_kills(r)?;
    let sights = read_credit_ids(r)?;
    let chats = read_credit_ids(r)?;
    Some(WldBestiary {
        kills,
        sights,
        chats,
    })
}

/// `BestiaryUnlocksTracker.Save:15-17` — the three sub-lists in the C#'s save
/// order, each with its own `Save`. The write emits the bag's lists verbatim,
/// so the order the caller's `to_wld` produced (the sorted tracker order) is
/// the file's.
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    // `NPCKillsTracker.Save:63-68`: `writer.Write(Count)` then per pair
    // `writer.Write(Key)` and `writer.Write(Value)`.
    w.i32(data.bestiary.kills.len() as i32);
    for (id, count) in &data.bestiary.kills {
        w.cs_string(id);
        w.i32(*count);
    }
    // `NPCWasNearPlayerTracker.Save:68-72`: the count, then the set's items.
    w.i32(data.bestiary.sights.len() as i32);
    for id in &data.bestiary.sights {
        w.cs_string(id);
    }
    // `NPCWasChatWithTracker.Save:58-62` — the same shape.
    w.i32(data.bestiary.chats.len() as i32);
    for id in &data.bestiary.chats {
        w.cs_string(id);
    }
}

/// The state-facing save half: `BestiaryUnlocksTracker.Save`'s input. The
/// kills ride `KillTracker::pairs` (clamped at load, see
/// `tr_core::bestiary::KillTracker`) and the two sets ride
/// [`WorldBestiary::sight_ids`]/[`WorldBestiary::chat_ids`], so the file's
/// order is the tracker's deterministic order (the C# collection order is
/// unspecified — see [`WorldBestiary`]).
pub fn to_wld(tracker: &WorldBestiary) -> WldBestiary {
    WldBestiary {
        kills: tracker
            .kills
            .pairs()
            .map(|(id, count)| (id.to_string(), count))
            .collect(),
        sights: tracker.sight_ids().map(str::to_string).collect(),
        chats: tracker.chat_ids().map(str::to_string).collect(),
    }
}

/// The state-facing load half: `BestiaryUnlocksTracker.Load`'s output over a
/// fresh tracker. Each half keeps its C# semantics — the kills' pairs
/// overwrite their key and clamp (`KillTracker::set_kill_count_directly`,
/// `NPCKillsTracker.cs:40-51`, so a hostile file's duplicate ids are
/// last-wins), the two sets add (`HashSet.Add`, so a duplicate id stays once).
pub fn from_wld(records: &WldBestiary) -> WorldBestiary {
    let mut tracker = WorldBestiary::new();
    tracker
        .kills
        .load_pairs(records.kills.iter().map(|(id, c)| (id.clone(), *c)));
    tracker.load_sights(records.sights.iter().cloned());
    tracker.load_chats(records.chats.iter().cloned());
    tracker
}

pub fn empty(data: &SectionData) -> bool {
    data.bestiary.is_empty()
}

pub fn payload(value: WldBestiary) -> SectionPayload {
    SectionPayload::Bestiary(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bestiary::POSITIVE_KILL_COUNT_CAP;

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

    fn data(bestiary: WldBestiary) -> SectionData {
        SectionData {
            bestiary,
            ..Default::default()
        }
    }

    fn section(records: &WldBestiary) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, &data(records.clone()), VER);
        w.into_inner()
    }

    fn read_at(bytes: &[u8]) -> (Option<WldBestiary>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, VER), r.pos())
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_6_4_the_section_is_implemented() {
        assert!(implemented(), "phase 6 replaces the stub in place");
        assert!(empty(&data(WldBestiary::default())));
        assert!(!empty(&data(WldBestiary {
            sights: vec!["Slime".into()],
            ..Default::default()
        })));
    }

    // --- DW-6.1: the round-trip and the C#'s byte layout ------------------

    #[test]
    fn test_DW_6_1_the_three_sub_sections_round_trip_byte_equal() {
        let records = WldBestiary {
            kills: vec![("BlueSlime".into(), 3), ("Pinky".into(), 1)],
            sights: vec!["GreenSlime".into()],
            chats: vec!["BlueSlime".into(), "Guide".into()],
        };
        let bytes = section(&records);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back.as_ref(), Some(&records), "the struct survives");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
        // …and writing it out again gives the same bytes.
        assert_eq!(section(back.as_ref().unwrap()), bytes);
    }

    #[test]
    fn test_DW_6_1_a_hard_coded_c_sharp_layout_parses_identically() {
        // Hand-computed `BestiaryUnlocksTracker.Save` over two kills, one
        // sight and no chats — the `tile_entities.rs:726` precedent: the
        // bytes are derived from the C# by hand, not from the writer.
        let want = [
            &i32_bytes(2)[..], // kills: count
            &cs_string("BlueSlime")[..],
            &i32_bytes(3)[..],
            &cs_string("Pinky")[..],
            &i32_bytes(1)[..],
            &i32_bytes(1)[..], // sights: count
            &cs_string("GreenSlime")[..],
            &i32_bytes(0)[..], // chats: count
        ]
        .concat();
        let (back, pos) = read_at(&want);
        assert_eq!(
            back,
            Some(WldBestiary {
                kills: vec![("BlueSlime".into(), 3), ("Pinky".into(), 1)],
                sights: vec!["GreenSlime".into()],
                chats: Vec::new(),
            })
        );
        assert_eq!(pos, want.len(), "the cursor lands on the last byte");
        // The writer reproduces those exact bytes (the field order — kills,
        // then sights, then chats — is the one thing a stub could get wrong).
        assert_eq!(section(back.as_ref().unwrap()), want);
    }

    #[test]
    fn test_DW_6_1_an_empty_sub_list_is_a_zero_count() {
        // All three empty: the section is twelve zero bytes
        // (`wld/mod.rs`'s framing test asserts the same).
        let bytes = section(&WldBestiary::default());
        assert_eq!(bytes, vec![0u8; 12]);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(WldBestiary::default()));
        assert_eq!(pos, 12);
    }

    #[test]
    fn test_DW_6_1_an_empty_string_credit_id_rides_the_file() {
        // The C# `writer.Write(string)` writes an empty string as one 0 byte;
        // a credit id is a plain string, so nothing forces it non-empty.
        let records = WldBestiary {
            kills: vec![("".into(), 0)],
            sights: vec!["".into()],
            ..Default::default()
        };
        let bytes = section(&records);
        assert_eq!(&bytes[4..6], &[0, 0], "the kill id's length prefix");
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(records));
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_DW_6_1_a_truncated_sub_list_is_a_short_read() {
        // A count of 3 with one record: the second string's prefix is missing.
        let mut bytes = i32_bytes(3);
        bytes.extend(cs_string("BlueSlime"));
        bytes.extend(i32_bytes(1));
        assert!(read_at(&bytes).0.is_none());
        // The same truncation inside the sights list.
        let mut bytes = i32_bytes(0); // kills: none
        bytes.extend(i32_bytes(2)); // sights: two
        bytes.extend(cs_string("GreenSlime"));
        assert!(read_at(&bytes).0.is_none());
    }

    // --- DW-6.2: the gate, the empty world, the deterministic order -------

    #[test]
    fn test_DW_6_2_the_registry_gates_the_sections_at_210_and_220() {
        // `LoadWorld_Version2:1832`/`:1844` — the section-level gates live in
        // the frozen registry, and this phase's flip is what makes them bite.
        let bestiary = crate::wld::sections::DEFS
            .iter()
            .find(|d| d.name == "bestiary")
            .expect("registered");
        let creative = crate::wld::sections::DEFS
            .iter()
            .find(|d| d.name == "creative powers")
            .expect("registered");
        assert!(!(bestiary.gate)(209), "below 210 the section is not read");
        assert!((bestiary.gate)(210));
        assert!(!(creative.gate)(219));
        assert!((creative.gate)(220));
    }

    #[test]
    fn test_DW_6_2_a_pre_210_file_does_not_read_the_bestiary_section() {
        // A v209 file that carries a bestiary pointer in its table: the gate
        // keeps the section unread, so the walk proceeds to the footer at the
        // last real section's end (`positions[8]`, as the C# does).
        let mut bytes = Vec::new();
        bytes.extend(i32_bytes(209));
        bytes.extend([0u8; 20]); // metadata: recorded, never validated
        bytes.extend(10i16.to_le_bytes()); // ten pointers (0..9)
        bytes.extend([0u8; 40]); // placeholders, backfilled below
        bytes.extend(8u16.to_le_bytes()); // eight tile ids
        bytes.extend([0u8]); // …one importance byte
        let header_end = bytes.len();
        // The empty encodings of pointers 3..8.
        bytes.extend([0, 0, 0, 0]); // 3 chests: count + the pre-294 shared slot count
        bytes.extend([0, 0]); // 4 signs
        bytes.extend([0, 0]); // 5 npcs: town + tower terminators (>= 140)
        bytes.extend(i32_bytes(0)); // 6 tile entities
        bytes.extend(i32_bytes(0)); // 7 pressure plates
        bytes.extend(i32_bytes(0)); // 8 town rooms
        let footer_at = bytes.len();
        // The footer triple, right after town rooms — where a file without a
        // bestiary section puts it.
        bytes.push(1);
        bytes.extend(cs_string("Pre210"));
        bytes.extend(i32_bytes(0));
        let mut pointers = [header_end; 10];
        let mut at = header_end;
        for (i, len) in [4usize, 2, 2, 4, 4, 4].iter().enumerate() {
            at += len;
            pointers[3 + i] = at;
        }
        // Index 9 (the bestiary slot) points at the footer: the section is
        // absent, so the running position never moved past town rooms.
        pointers[9] = footer_at;
        for (i, p) in pointers.iter().enumerate() {
            let at = 4 + 20 + 2 + 4 * i;
            bytes[at..at + 4].copy_from_slice(&(*p as i32).to_le_bytes());
        }
        let table = crate::wld::SectionTable::open(&bytes).expect("opens");
        assert_eq!(table.version(), 209);
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.bestiary, WldBestiary::default());
        assert_eq!(loaded.creative_powers, Default::default());
        assert_eq!(
            loaded,
            crate::wld::sections::SectionData::default(),
            "a pre-210 bestiary section stays empty, exactly like the C#'s empty FillBasedOnVersionBefore210"
        );
    }

    #[test]
    fn test_DW_6_2_the_store_survives_a_whole_wld_round_trip() {
        // The registry flip end to end: the section bytes land back in the bag
        // through `load_sections`' end-offset check, and the same store writes
        // back out unchanged.
        let world = crate::world::World::new("S".into(), 8, 6);
        let records = WldBestiary {
            kills: vec![("BlueSlime".into(), 12), ("Pinky".into(), 1)],
            sights: vec!["BlueSlime".into()],
            chats: vec!["Guide".into()],
        };
        let file = crate::wld::to_bytes(&world, &data(records.clone()), 1).expect("writes");
        let table = crate::wld::SectionTable::open(&file).expect("opens");
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.bestiary, records);
        let file2 = crate::wld::to_bytes(&world, &data(loaded.bestiary), 1).expect("writes");
        assert_eq!(file, file2, "the round-trip is byte-stable");
    }

    #[test]
    fn test_DW_6_2_the_tracker_round_trips_through_the_state_half() {
        // The full state cycle: tracker → file bytes → tracker, with the
        // deterministic order the sorted sets give.
        let mut tracker = WorldBestiary::new();
        for id in ["Pinky", "BlueSlime", "GreenSlime"] {
            tracker.kills.set_kill_count_directly(id, 2);
            tracker.set_was_seen_directly(id);
            tracker.set_was_chat_with_directly(id);
        }
        let records = to_wld(&tracker);
        let bytes = section(&records);
        let (back, pos) = read_at(&bytes);
        assert_eq!(pos, bytes.len());
        let restored = from_wld(&back.expect("reads"));
        let kills: Vec<(String, i32)> =
            restored.kills.pairs().map(|(k, v)| (k.into(), v)).collect();
        assert_eq!(kills, records.kills);
        assert_eq!(
            restored.sight_ids().collect::<Vec<_>>(),
            ["BlueSlime", "GreenSlime", "Pinky"]
        );
        assert_eq!(
            restored.chat_ids().collect::<Vec<_>>(),
            ["BlueSlime", "GreenSlime", "Pinky"]
        );
        // Writing the restored tracker reproduces the same file bytes.
        assert_eq!(section(&to_wld(&restored)), bytes);
    }

    #[test]
    fn test_DW_6_2_duplicate_credit_ids_fold_like_the_c_sharp_collections() {
        // `NPCKillsTracker.Load:72-81` writes `dict[key] = value` — the last
        // duplicate wins; the two `HashSet.Add`s keep the first.
        let records = WldBestiary {
            kills: vec![("BlueSlime".into(), 5), ("BlueSlime".into(), 9)],
            sights: vec!["GreenSlime".into(), "GreenSlime".into()],
            chats: vec!["Pinky".into(), "Pinky".into()],
        };
        let tracker = from_wld(&records);
        assert_eq!(tracker.kills.get_kill_count("BlueSlime"), 9, "last-wins");
        assert_eq!(tracker.sight_ids().count(), 1, "a set keeps one");
        assert_eq!(tracker.chat_ids().count(), 1);
        // …so the folded tracker writes one record per id, where the hostile
        // file carried two (the C#'s `Save` would write one too).
        assert_eq!(to_wld(&tracker).kills, vec![("BlueSlime".to_string(), 9)]);
    }

    #[test]
    fn test_DW_6_2_unported_credit_ids_are_kept() {
        // A foreign file may carry ids for NPCs the port has never ported:
        // the trackers are string-keyed, so nothing is dropped on load and the
        // save writes them back (the port owns no filter that would).
        let records = WldBestiary {
            kills: vec![("SomeFutureNPC".into(), POSITIVE_KILL_COUNT_CAP)],
            sights: vec!["SomeFutureNPC".into()],
            chats: Vec::new(),
        };
        let tracker = from_wld(&records);
        assert_eq!(
            tracker.kills.get_kill_count("SomeFutureNPC"),
            POSITIVE_KILL_COUNT_CAP,
            "the cap clamp lets the file's own maximum through"
        );
        assert_eq!(to_wld(&tracker), records);
    }

    #[test]
    fn test_DW_6_2_a_hostile_kill_count_is_clamped_by_the_tracker() {
        // `NPCKillsTracker.Load` does *not* clamp (`dict[key] = value`
        // directly); the port's load half routes through
        // `SetKillCountDirectly`, which does (`:44-53`'s `Utils.Clamp`). The
        // deviation is one-sided by design: the wire keeps the raw value, the
        // in-memory tracker cannot.
        let records = WldBestiary {
            kills: vec![("BlueSlime".into(), -7)],
            ..Default::default()
        };
        assert_eq!(
            read_at(&section(&records)).0,
            Some(records.clone()),
            "the wire keeps it"
        );
        let tracker = from_wld(&records);
        assert_eq!(tracker.kills.get_kill_count("BlueSlime"), 0);
    }

    // --- the defensive arms -----------------------------------------------

    #[test]
    fn a_negative_sub_list_count_is_refused() {
        // The C# `for (int i = 0; i < num; i++)` reads a negative count as no
        // records; the port refuses instead of silently accepting it.
        let mut bytes = (-1i32).to_le_bytes().to_vec();
        bytes.extend([0u8; 8]);
        assert!(read_at(&bytes).0.is_none());
        // …and the same for the sights and chats lists.
        let mut bytes = i32_bytes(0);
        bytes.extend((-3i32).to_le_bytes());
        assert!(read_at(&bytes).0.is_none());
    }

    #[test]
    fn a_hostile_string_prefix_does_not_allocate() {
        // A 5-byte 7-bit length prefix asks for ~2^28 bytes; the read must
        // refuse (and must not pre-allocate) inside a 5-byte buffer.
        let mut bytes = i32_bytes(1);
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x7f]);
        assert!(read_at(&bytes).0.is_none());
    }

    /// The house rule's citation probe: the source carries the anchors the
    /// golden rule demands for every ported block.
    #[test]
    fn test_DW_6_4_the_citations_are_present() {
        let src = include_str!("bestiary.rs");
        // The DW item's named anchors, in the house style.
        assert!(src.contains("WorldFile.cs:3396"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:3402"), "the reader's anchor");
        for anchor in [
            "// `NPCKillsTracker.Load:72-81`",
            "// `NPCWasNearPlayerTracker.Load:76-84` / `NPCWasChatWithTracker.Load:66-74`",
            "// `BestiaryUnlocksTracker.Load:22-24`",
            "// `BestiaryUnlocksTracker.Save:15-17`",
            "// `NPCKillsTracker.Save:63-68`",
            "// `NPCWasNearPlayerTracker.Save:68-72`",
            "// `NPCWasChatWithTracker.Save:58-62`",
            "`BestiaryUnlocksTracker.cs:13-18`",
            "`NPCKillsTracker.Save:59-70`",
            "`NPCWasNearPlayerTracker.Save:64-74`",
            "`NPCWasChatWithTracker.Save:54-64`",
            "`LoadWorld_Version2:1832`",
            "`LoadBestiaryForVersionsBefore210:3407`",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
