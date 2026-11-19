# LEAD DIAGNOSIS — F13c2 gate10 atlas-fits failure (delete before commit)

gate10: 1 failure = `item_icon_uvs_are_normalized_and_atlas_fits`: "atlas 8416 px tall must stay within the strict-GPU bound" (8192). Main PASSES this test (verified: p1-e21i-gate2 shows it ok on main).

YOUR branch adds 9 lines to assets.rs (frost NPC sheets enumeration) + 73 to entities.rs (FROST_SHEETS). The new sheets push the atlas from ≤8192 to 8416 (+224px). This is the SAME class as E-4b gore SKIP (atlas packing efficiency), NOT a port defect.

OPTIONS (pick the correct one per house rules):
1. **Packing efficiency**: the atlas packer (tr-render/src/atlas.rs) may pack the new sheets inefficiently. E-4b's first-fit attempt failed (8305), but your +224 is smaller. Check if a better shelf arrangement fits — BUT do not spend >1 fix round on packing algorithms (E-4b precedent: SKIP).
2. **Sheet necessity**: are ALL 9 new asset lines required for defaults-b? The frost kinds need their NPC sheets for RENDER, but defaults-b is DEFAULTS+FRAMES only (stand-in routing documented). If the sheets are only needed when the AI arms wire (later wave), DEFER the sheet enumeration to the wiring wave — remove the 9 assets.rs lines + FROST_SHEETS from entities.rs IF the defaults tests don't reference them. Check: do your s1_f_13c tests assert sheet presence? If yes, keep sheets but move them to a SEPARATE atlas page or the static half (check how main's atlas splits world/static — the test name says item_icon; NPC sheets may ride a different atlas that doesn't have the 8192 bound).
3. **Investigate which atlas**: the failing test is item_icon_uvs — does it pack NPC sheets at all? If FROST_SHEETS ride the NPC atlas (not item), the failure may be a DIFFERENT mechanism (your entities.rs changes affecting the shared content_lookups). Read the test + content_lookups to see WHY frost sheets affect item atlas height.

DO NOT delete the atlas-bound test or raise the bound (rendering invariant). DO NOT spend >1 round. If unresolvable in one round → SKIP F13c2 with the atlas reason (same as E-4b), preserve branch, record for the packing-algorithm task. The frost DEFAULTS data (npc/data.rs rows) is the valuable part — if sheets are the only blocker, consider splitting: land defaults WITHOUT sheet enumeration (defer sheets to wiring wave), document.
