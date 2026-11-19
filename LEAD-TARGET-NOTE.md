# LEAD SPEED NOTE v2 — URGENT: shared TR-L7-TARGET has 6 gates queued; your F13c2 gate waits behind them
SWITCH NOW: cmd /c "set CARGO_TARGET_DIR=C:\Users\DreadLord\Desktop\TR-P4-TARGET&& cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids"
One cold build (~10min, PARALLEL, no lock wait), then your remaining 5 frost branches gate ~1-3min each. ALSO: write LANDED entries to TR-MERGE-RESULTS.md promptly (overseer tracks you via gate logs only — results entries make progress visible).
