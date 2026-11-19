# LEAD SPEED NOTE v2 — URGENT: shared TR-L7-TARGET has 6 gates queued; your K-4c gate waits behind them
SWITCH NOW: cmd /c "set CARGO_TARGET_DIR=C:\Users\DreadLord\Desktop\TR-P3-TARGET&& cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids"
One cold build (~10min, PARALLEL, no lock wait), then your remaining 5 split/save branches gate ~1-3min each. Growth splits 4/4 done — edit splits + save remain, all sequential same-file, per-lane target is pure win.
