//! Section 11 (pointer 10): creative powers —
//! `WorldFile.SaveCreativePowers:3412` / `WorldFile.LoadCreativePowers:3418`,
//! whose bytes are `CreativePowerManager.SaveToWorld:125-137` /
//! `LoadFromWorld:139-151` over the powers that implement
//! `IPersistentPerWorldContent` (`IPersistentPerWorldContent.cs:5-14`).
//!
//! Wire shape: **terminator-delimited, not counted** —
//! `{ bool true; ushort id; payload }` repeated until a `bool false`
//! terminates. The payload depends on the id, not on a length:
//! the four toggles (`FreezeTime` 0, `FreezeRainPower` 9,
//! `FreezeWindDirectionAndStrength` 10, `StopBiomeSpreadPower` 13) write
//! `bool Enabled` (`CreativePowers.cs:1458`/`:1524`/`:1491`/`:1557`), the two
//! sliders (`ModifyTimeRate` 8, `DifficultySliderPower` 12) write the raw
//! `f32 _sliderCurrentValueCache` (`:967`/`:1162`). The id is
//! `CreativePowerManager.Register`'s registration-order counter
//! (`CreativePowerManager.cs:35-55`) handed out by `Initialize`
//! (`CreativePowerManager.cs:86-104`) — **not** the port's own `PowerId`
//! order; the `PowerId ↔ u16` table lives in the state owner
//! (`tr-app/src/creative.rs::PERSISTENT_POWERS`).
//!
//! **Unknown id aborts the load loop** (`:146-149`): a `break`, not a skip.
//! The C# consumes `bool true` + `ushort` and stops, keeping everything it
//! read so far; the section then ends short of its pointer and the loader
//! rejects the file (`StatusID.BadSectionPointer`, the port's
//! `load_sections` end-offset check). This codec reproduces the break
//! faithfully — the partial records and the cursor position are observable.
//!
//! **Save-side contract:** `write` reads [`SectionData`] (the frozen
//! `SAVE_DEFS` closure), so the caller materializes the bag with
//! `tr_app::creative::persistent_powers` before invoking
//! `wld::to_bytes`/`wld::save`. The writer emits the caller's records in the
//! C#'s `_powersById` walk order (ascending registration id) and never invents
//! an id: powers the port has not ported (freeze rain/wind, the difficulty
//! slider) are simply absent from the caller's list.
//!
//! ## The slider values are remapped here, not stored verbatim
//! The two slider records' payload is `ASharedSliderPower._sliderCurrentValue
//! Cache` — a float in **[0,1]** (`ModifyTimeRate.Save:967`,
//! `DifficultySliderPower.Save:1162`) that the C# turns into its gameplay
//! quantity on load through `UpdateInfoFromSliderValueCache`
//! (`ModifyTimeRate.UpdateInfoFromSliderValueCache:877-880` =
//! `(int)Math.Round(Utils.Remap(cache, 0, 1, 1, 24))`, so cache 0.5 is **x12**
//! and cache 1 is **x24**; the difficulty slider's is at `:1018-1027`). The
//! port's state owner (`tr_app::creative::PowerState::Slider`) carries that
//! *derived* quantity — the time rate 1..24 — so the codec remaps in both
//! directions and this module is the only place that knows the two
//! representations differ. The difficulty record rides the file verbatim
//! because the port has no difficulty power to remap it onto
//! (`creative.rs::PERSISTENT_POWERS`' `port: None`).

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};
use crate::damage::round_bankers;

/// One per-world persistent power value: the id is the manager's
/// registration-order `ushort`, the payload a `bool` (a toggle) or an `f32`
/// (a slider).
#[derive(Clone, Debug, PartialEq)]
pub enum WldCreativeValue {
    Bool(bool),
    F32(f32),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldCreativePowers {
    pub records: Vec<(u16, WldCreativeValue)>,
}

impl WldCreativePowers {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

pub fn implemented() -> bool {
    true
}

/// The six `IPersistentPerWorldContent` powers of the fifteen registrations
/// `CreativePowerManager.cs:86-104` makes, with the payload each writes.
/// Everything else is not per-world content — the four
/// `Start*Immediately` buttons (1-4) and `FarPlacementRangePower` (11) are
/// `ASharedButtonPower`, `GodmodePower` (5) and `SpawnRateSliderPerPlayerPower`
/// (14) are per-player, and `ModifyWindDirectionAndStrength` (6) /
/// `ModifyRainPower` (7) are sliders that do not implement the interface — so
/// `SaveToWorld` never writes them and `LoadFromWorld` aborts on them.
const PERSISTENT_IDS: [(u16, PayloadKind); 6] = [
    (0, PayloadKind::Bool),  // `FreezeTime.Save:1458` — `writer.Write(Enabled)`
    (8, PayloadKind::F32),   // `ModifyTimeRate.Save:967` — `writer.Write(_sliderCurrentValueCache)`
    (9, PayloadKind::Bool),  // `FreezeRainPower.Save:1524`
    (10, PayloadKind::Bool), // `FreezeWindDirectionAndStrength.Save:1491`
    (12, PayloadKind::F32),  // `DifficultySliderPower.Save:1162` — the same [0,1] cache
    (13, PayloadKind::Bool), // `StopBiomeSpreadPower.Save:1557`
];

/// The two halves of `Utils.Remap:345-348` for the one slider whose ported
/// quantity differs from its wire value. Both directions clamp the lerp
/// factor to [0,1] (`Utils.GetLerpValue:315-343`'), so a hostile or aliased
/// cache outside [0,1] reads as the slider's end, exactly as the C#'s slider
/// would.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = ((value - from_min) / (from_max - from_min)).clamp(0.0, 1.0);
    to_min + (to_max - to_min) * t
}

/// `ModifyTimeRate` (id 8): the wire cache in [0,1] ↔ the ported
/// `TargetTimeRate` 1..24 (`UpdateInfoFromSliderValueCache:877-880`).
const TIME_RATE_MIN: f32 = 1.0;
const TIME_RATE_MAX: f32 = 24.0;

/// The wire cache → the ported value for the id's slider payload
/// (`ModifyTimeRate.Load:970-975`'s `UpdateInfoFromSliderValueCache` call).
/// `None` = the id's payload rides the file verbatim (a toggle, or the
/// unported difficulty slider).
fn slider_from_wire(id: u16, cache: f32) -> Option<f32> {
    (id == 8).then(|| {
        // `(int)Math.Round(Utils.Remap(cache, 0, 1, 1, 24))` — banker's
        // rounding, the C# `Math.Round` (`damage::round_bankers`).
        round_bankers(f64::from(remap(
            cache,
            0.0,
            1.0,
            TIME_RATE_MIN,
            TIME_RATE_MAX,
        ))) as f32
    })
}

/// The ported value → the wire cache (`ModifyTimeRate.Save:967`'s inverse:
/// the cache whose `UpdateInfoFromSliderValueCache` reads back as `rate`).
fn slider_to_wire(id: u16, value: f32) -> f32 {
    if id == 8 {
        return remap(value, TIME_RATE_MIN, TIME_RATE_MAX, 0.0, 1.0);
    }
    value
}

/// What an id's payload is on the wire: the toggle's `bool` or the slider's
/// `f32` (`ASharedTogglePower.Save` vs `ASharedSliderPower.Save`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PayloadKind {
    Bool,
    F32,
}

/// The payload kind a persistent id writes — `None` for every id that is not
/// per-world content, which is the `LoadFromWorld:146-149` abort case.
fn payload_kind(id: u16) -> Option<PayloadKind> {
    PERSISTENT_IDS
        .iter()
        .find(|(known, _)| *known == id)
        .map(|(_, kind)| *kind)
}

/// `LoadFromWorld:139-151`: `while (reader.ReadBoolean())` reads a record
/// marker, the id, and the id's payload; an id that is not per-world content
/// breaks the loop with everything read so far kept.
pub fn read(r: &mut Reader, _version: i32) -> Option<WldCreativePowers> {
    let mut records: Vec<(u16, WldCreativeValue)> = Vec::new();
    while r.boolean()? {
        let id = r.u16()?;
        let Some(kind) = payload_kind(id) else {
            break; // `:146-149`: unknown id — abort, keep what was read
        };
        let value = match kind {
            PayloadKind::Bool => WldCreativeValue::Bool(r.boolean()?),
            PayloadKind::F32 => {
                let cache = r.f32()?;
                WldCreativeValue::F32(slider_from_wire(id, cache).unwrap_or(cache))
            }
        };
        records.push((id, value));
    }
    Some(WldCreativePowers { records })
}

/// `SaveToWorld:125-137`: one `{ bool true; ushort id; payload }` per power in
/// `_powersById` order (ascending registration id), then the `bool false`
/// terminator. The records are sorted here so the writer emits the C#'s order
/// whatever order the caller's list happens to carry — the C# cannot emit them
/// out of order, and neither should the port.
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    let mut records = data.creative_powers.records.clone();
    records.sort_by(|a, b| a.0.cmp(&b.0)); // `_powersById`'s registration order
    for (id, value) in records {
        // `writer.Write(value: true)` (`:131`).
        w.boolean(true);
        // `writer.Write(item.Key)` (`:132`) — the registration-order `ushort`.
        w.u16(id);
        match value {
            WldCreativeValue::Bool(v) => w.boolean(v),
            // The slider's cache, not the ported value
            // (`ModifyTimeRate.Save:967`'s inverse — the module header).
            WldCreativeValue::F32(v) => w.f32(slider_to_wire(id, v)),
        }
    }
    // `writer.Write(value: false)` (`:137`) — the list terminator.
    w.boolean(false);
}

pub fn empty(data: &SectionData) -> bool {
    data.creative_powers.is_empty()
}

pub fn payload(value: WldCreativePowers) -> SectionPayload {
    SectionPayload::CreativePowers(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    fn u16_bytes(v: u16) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    /// One `{ bool true; ushort id; payload }` record, hand-built from the C#.
    fn record(id: u16, value: &WldCreativeValue) -> Vec<u8> {
        let mut v = vec![1u8];
        v.extend(u16_bytes(id));
        match value {
            WldCreativeValue::Bool(b) => v.push(u8::from(*b)),
            WldCreativeValue::F32(f) => v.extend(f.to_le_bytes()),
        }
        v
    }

    fn data(powers: WldCreativePowers) -> SectionData {
        SectionData {
            creative_powers: powers,
            ..Default::default()
        }
    }

    fn section(records: &[(u16, WldCreativeValue)]) -> Vec<u8> {
        let mut w = Writer::new();
        write(
            &mut w,
            &data(WldCreativePowers {
                records: records.to_vec(),
            }),
            VER,
        );
        w.into_inner()
    }

    fn read_at(bytes: &[u8]) -> (Option<WldCreativePowers>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, VER), r.pos())
    }

    fn bool_record(id: u16, v: bool) -> (u16, WldCreativeValue) {
        (id, WldCreativeValue::Bool(v))
    }

    fn f32_record(id: u16, v: f32) -> (u16, WldCreativeValue) {
        (id, WldCreativeValue::F32(v))
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_6_4_the_section_is_implemented() {
        assert!(implemented(), "phase 6 replaces the stub in place");
        assert!(empty(&data(WldCreativePowers::default())));
        assert!(!empty(&data(WldCreativePowers {
            records: vec![bool_record(0, true)],
        })));
    }

    // --- DW-6.3: the round-trip, the layout, the abort --------------------

    #[test]
    fn test_DW_6_3_the_records_round_trip_byte_equal() {
        // The ported persistent values: a toggle on, the time-rate slider at
        // its maximum, the biome-spread freeze off. The records hold the
        // ported values; the wire carries id 8's [0,1] cache.
        let records = vec![
            bool_record(0, true),
            f32_record(8, 24.0),
            bool_record(13, false),
        ];
        let bytes = section(&records);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(WldCreativePowers { records }));
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
        assert_eq!(section(&back.unwrap().records), bytes);
    }

    #[test]
    fn test_DW_7_6_a_hand_built_vanilla_slider_record_reads_as_the_rate() {
        // F3 (Phase-6 review N1): a vanilla file carries the [0,1]
        // `_sliderCurrentValueCache` (`ModifyTimeRate.Save:967`), which the C#
        // derives its rate from (`UpdateInfoFromSliderValueCache:877-880` =
        // round(Remap(cache, 0, 1, 1, 24))). A record of 0.5 is x12, not 0.5x.
        let mut bytes = record(8, &WldCreativeValue::F32(0.5));
        bytes.push(0); // the terminator
        let (back, pos) = read_at(&bytes);
        let records = back.expect("reads").records;
        assert_eq!(pos, bytes.len(), "the cursor lands on the terminator");
        assert_eq!(
            records,
            vec![(8, WldCreativeValue::F32(12.0))],
            "cache 0.5 reads back as the rate 12"
        );
        // The C#'s own mapping, pinned at the endpoints and the middle text:
        // 0 → x1, 0.5 → x12, 1 → x24 (`x1`/`x12`/`x24` are the slider's three
        // labels, `ProvideSlider`'s `bottom/middle/topText_OnClick`).
        for (cache, rate) in [(0.0, 1.0), (0.5, 12.0), (1.0, 24.0)] {
            let mut wire = record(8, &WldCreativeValue::F32(cache));
            wire.push(0);
            let (back, _) = read_at(&wire);
            assert_eq!(
                back.unwrap().records,
                vec![(8, WldCreativeValue::F32(rate))],
                "cache {cache} → x{rate}"
            );
        }
        // A hostile cache outside [0,1] clamps to the slider's end
        // (`Utils.Remap`'s clamped lerp, `Utils.cs:345-348`), never extrapolates.
        let mut huge = record(8, &WldCreativeValue::F32(24.0));
        huge.push(0);
        let (back, _) = read_at(&huge);
        assert_eq!(back.unwrap().records, vec![(8, WldCreativeValue::F32(24.0))]);
        let mut low = record(8, &WldCreativeValue::F32(-7.0));
        low.push(0);
        let (back, _) = read_at(&low);
        assert_eq!(back.unwrap().records, vec![(8, WldCreativeValue::F32(1.0))]);
    }

    #[test]
    fn test_DW_7_6_the_port_s_write_emits_the_vanilla_cache() {
        // The mirror image: the port's state holds the rate, so the writer
        // emits the cache the C# would have read that rate from.
        let bytes = section(&[f32_record(8, 12.0)]);
        // 12.0 → (12-1)/23 = 0.47826087f = 0x3EF50A3D.
        assert_eq!(&bytes[3..7], &remap(12.0, 1.0, 24.0, 0.0, 1.0).to_le_bytes());
        assert_eq!(&bytes[3..7], &0.47826087f32.to_le_bytes());
        let (back, _) = read_at(&bytes);
        assert_eq!(
            back.unwrap().records,
            vec![(8, WldCreativeValue::F32(12.0))],
            "the remap is an involution on the integral rates"
        );
        // The two ends of the slider, and the byte the C# writes at x24:
        // `SetValueKeyboardForced(1f)` → cache 1.0 = 0x3F800000.
        assert_eq!(&section(&[f32_record(8, 24.0)])[3..7], &1.0f32.to_le_bytes());
        assert_eq!(&section(&[f32_record(8, 1.0)])[3..7], &0.0f32.to_le_bytes());
    }

    #[test]
    fn test_DW_6_3_a_hard_coded_c_sharp_layout_parses_identically() {
        // Hand-computed `SaveToWorld:131-137` for the same three powers:
        // `true, ushort 0, true`, `true, ushort 8, f32 1.0` (the cache of
        // x24 — `ModifyTimeRate.Save:967` writes `_sliderCurrentValueCache`,
        // not the rate), `true, ushort 13, false`, `false`. 1.0f = 0x3F800000.
        let mut want = Vec::new();
        want.push(1);
        want.extend(u16_bytes(0));
        want.push(1); // FreezeTime enabled
        want.push(1);
        want.extend(u16_bytes(8));
        want.extend(1.0f32.to_le_bytes()); // the slider cache for x24
        want.push(1);
        want.extend(u16_bytes(13));
        want.push(0); // StopBiomeSpread disabled
        want.push(0); // the terminator
        assert_eq!(want.len(), 1 + 3 + 1 + 6 + 1 + 3 + 1);
        let (back, pos) = read_at(&want);
        assert_eq!(
            back,
            Some(WldCreativePowers {
                records: vec![
                    bool_record(0, true),
                    f32_record(8, 24.0),
                    bool_record(13, false)
                ],
            })
        );
        assert_eq!(pos, want.len(), "the cursor lands on the terminator");
        // The writer reproduces those exact bytes (record-marker-first, id,
        // payload, terminator — the shape the stub warned about).
        assert_eq!(
            section(&[
                bool_record(0, true),
                f32_record(8, 24.0),
                bool_record(13, false)
            ]),
            want
        );
    }

    #[test]
    fn test_DW_6_3_an_unknown_id_aborts_the_loop_and_keeps_the_earlier_values() {
        // `LoadFromWorld:146-149`: id 5 (godmode) is per-player, not per-world,
        // so the loop breaks. The port keeps what it read and leaves the
        // cursor right after the id — a faithful break, not a skip.
        let mut bytes = record(0, &WldCreativeValue::Bool(true));
        let abort_at = bytes.len();
        bytes.push(1); // the record marker
        let id_at = bytes.len();
        bytes.extend(u16_bytes(5));
        // …and the payload the C# never reads (it broke before knowing the kind).
        bytes.extend([1, 2, 3, 4]);
        let (back, pos) = read_at(&bytes);
        assert_eq!(
            back,
            Some(WldCreativePowers {
                records: vec![bool_record(0, true)],
            }),
            "the earlier values survive the abort"
        );
        assert_eq!(
            pos,
            id_at + 2,
            "the cursor stops right after the unknown id ({abort_at} bytes of record behind it)"
        );
    }

    #[test]
    fn test_DW_6_3_an_unknown_first_id_aborts_before_any_record() {
        // A hostile file whose very first id is unknown: no records, and the
        // cursor sits after the id.
        let mut bytes = vec![1u8];
        bytes.extend(u16_bytes(7)); // the rain slider — not per-world content
        bytes.extend([0, 0, 0, 0]);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(WldCreativePowers::default()));
        assert_eq!(pos, 3);
    }

    #[test]
    fn test_DW_6_3_a_truncated_record_is_a_short_read() {
        // A well-formed record is `{ marker, id, payload }`; the terminator
        // behind the last record is part of the stream, so a file that stops
        // after the last payload has no terminator to read (`LoadFromWorld`'s
        // `while (ReadBoolean())` would throw at EOF).
        let mut bytes = record(8, &WldCreativeValue::F32(1.5));
        bytes.push(0); // the terminator
        assert!(
            read_at(&bytes).0.is_some(),
            "marker, id, payload, terminator"
        );
        bytes.pop();
        assert!(
            read_at(&bytes).0.is_none(),
            "the terminator itself is missing"
        );
        // …and the same payload cut in half, and a marker with no id behind it.
        bytes.truncate(bytes.len() - 2);
        assert!(read_at(&bytes).0.is_none(), "the f32 payload is cut");
        assert!(read_at(&[1u8]).0.is_none());
    }

    #[test]
    fn test_DW_6_3_the_empty_section_is_the_terminator_only() {
        // A world with no persistent power changed writes a single `false`
        // (`wld/mod.rs`'s framing test asserts the same byte).
        let bytes = section(&[]);
        assert_eq!(bytes, vec![0]);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(WldCreativePowers::default()));
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_DW_6_3_the_write_emits_the_registration_order() {
        // `SaveToWorld` walks `_powersById` (ascending id); the writer sorts,
        // so a caller's list in any order still emits the C#'s.
        let bytes = section(&[
            bool_record(13, false),
            f32_record(8, 24.0),
            bool_record(0, true),
        ]);
        assert_eq!(
            bytes,
            section(&[
                bool_record(0, true),
                f32_record(8, 24.0),
                bool_record(13, false)
            ])
        );
        // …and the sorted form is what the record markers say.
        let (back, pos) = read_at(&bytes);
        assert_eq!(pos, bytes.len());
        let ids: Vec<u16> = back.unwrap().records.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, [0, 8, 13]);
    }

    #[test]
    fn test_DW_6_3_an_unported_vanilla_id_round_trips_through_the_codec() {
        // Freeze-rain (9), freeze-wind (10) and the difficulty slider (12) are
        // per-world content in Vanilla, so the codec decodes them — the port's
        // *state* has no power for them, which is `tr-app`'s decision, not the
        // format's (the writer would re-emit them if the caller carried them).
        let bytes = section(&[
            bool_record(9, true),
            bool_record(10, true),
            f32_record(12, 0.33),
        ]);
        let (back, pos) = read_at(&bytes);
        assert_eq!(
            back,
            Some(WldCreativePowers {
                records: vec![
                    bool_record(9, true),
                    bool_record(10, true),
                    f32_record(12, 0.33)
                ],
            })
        );
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_DW_7_6_the_difficulty_slider_cache_rides_the_file_verbatim() {
        // "difficulty likewise": `DifficultySliderPower.Save:1162` writes the
        // same [0,1] cache (derived into `StrengthMultiplierToGiveNPCs` at
        // `UpdateInfoFromSliderValueCache:1018-1027`), but the port carries no
        // difficulty power, so there is no ported quantity to remap onto and
        // the record must ride the file byte-exact — inventing a strength
        // would corrupt the vanilla bytes on the next save.
        let bytes = section(&[f32_record(12, 0.75)]);
        assert_eq!(&bytes[3..7], &0.75f32.to_le_bytes(), "the raw cache");
        let (back, pos) = read_at(&bytes);
        assert_eq!(
            back,
            Some(WldCreativePowers {
                records: vec![f32_record(12, 0.75)],
            }),
            "id 12 is not remapped: the cache is the value the file carries"
        );
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_DW_6_3_a_truncated_terminator_is_a_short_read() {
        // The dirty arm of DW-6.3's plan line: an empty stream is not a
        // terminator, it is EOF.
        assert!(read_at(&[]).0.is_none());
        assert!(read_at(&[1]).0.is_none());
    }

    #[test]
    fn test_DW_6_4_the_two_sections_participate_in_one_framed_file() {
        // Both of this phase's sections in one file: the pointers land where
        // `load_sections` expects them and the records come back from the bag.
        let world = crate::world::World::new("C".into(), 8, 6);
        let records = vec![
            bool_record(0, true),
            f32_record(8, 12.0),
            bool_record(13, true),
        ];
        let mut bag = SectionData::default();
        bag.creative_powers.records = records.clone();
        bag.bestiary = super::super::bestiary::WldBestiary {
            kills: vec![("BlueSlime".into(), 4)],
            ..Default::default()
        };
        let file = crate::wld::to_bytes(&world, &bag, 1).expect("writes");
        let table = crate::wld::SectionTable::open(&file).expect("opens");
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.creative_powers.records, records);
        assert_eq!(loaded.bestiary.kills, bag.bestiary.kills);
        // Every section's codec has landed, so no lane is skip-read any more.
        for def in crate::wld::sections::DEFS {
            assert!(
                (def.implemented)(),
                "pointer {} ({})",
                def.pointer,
                def.name
            );
        }
    }

    /// The house rule's citation probe: the source carries the anchors the
    /// golden rule demands for every ported block.
    #[test]
    fn test_DW_6_4_the_citations_are_present() {
        let src = include_str!("creative_powers.rs");
        // The DW item's named anchors, in the house style. The probe reads the
        // whole file, so the anchors must sit in the prose/comments above —
        // this function's own body is a deliberately tiny corner of `src`.
        assert!(src.contains("WorldFile.cs:3412"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:3418"), "the reader's anchor");
        assert!(
            src.contains("CreativePowerManager.cs:86-104"),
            "the registration order"
        );
        for anchor in [
            "//! Wire shape: **terminator-delimited, not counted** —",
            "`CreativePowerManager.SaveToWorld:125-137`",
            "`LoadFromWorld:139-151`",
            "// `LoadFromWorld:139-151`",
            "// `SaveToWorld:125-137`",
            "`IPersistentPerWorldContent.cs:5-14`",
            "`CreativePowerManager.Register`'s registration-order counter",
            // F3's anchors: the [0,1] cache and the two directions of its
            // derivation (`ModifyTimeRate` + `DifficultySliderPower`).
            "`ModifyTimeRate.Save:967`",
            "`DifficultySliderPower.Save:1162`",
            "`ModifyTimeRate.UpdateInfoFromSliderValueCache:877-880`",
            "`ModifyTimeRate.Load:970-975`",
            "`:1018-1027`",
            "`Utils.Remap:345-348`",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
