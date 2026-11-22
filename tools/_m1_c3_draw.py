# W4-c3 merge-side E8: render/entities.rs + ui/bestiary.rs + anchors.txt.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep

def edit_file(p, fn):
    src = open(p, encoding="utf8", newline="").read()
    box = [src]
    rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
    fn(rep, box)
    open(p, "w", encoding="utf8", newline="").write(box[0])
    print(p, "done")

def entities(rep, box):
    # STYLE10_SHEETS: 694 only (34/289 ride d34/d38's sheet consts)
    rep("""/// (`TextureAssets.Npc[type]`), keyed by the vanilla net id in
/// `NpcAssets::npc_sheets` exactly like `SMALL_KIND_SHEETS`. The Cursed
/// Skull's protocol id is port-local 355, but its DRAW sheet is its own
/// NPC_34 (`texture_net_id` reads `net_id()` = the bestiary entry's 34).
pub(crate) const STYLE10_SHEETS: &[u32] = &[
    34,  // the Cursed Skull
    289, // the Giant Cursed Skull
    694, // the Water Bolt Mimic
];
""", """/// (`TextureAssets.Npc[type]`), keyed by the vanilla net id in
/// `NpcAssets::npc_sheets` exactly like `SMALL_KIND_SHEETS`. E8 merge
/// drop (329 s3): the 34/289 rows ride d34/d38's sheet consts; only the
/// WaterBoltMimic's NPC_694 is c3's own.
pub(crate) const STYLE10_SHEETS: &[u32] = &[
    694, // the Water Bolt Mimic
];
""")
    # draw arms: drop the c3 34/289 dups (d34/d38 own identical draws),
    # keep the 694 arm; the tint note stays (the tint seam keeps all three)
    rep("""            // W4-c3 (the style-10 family): each kind draws its own sheet
            // (`STYLE10_SHEETS`) at `SetFrameSize`'s integer-division row
            // height over `Main.npcFrameCount[type]` (34 three frames,
            // 289 six, 694 thirty; `Main.cs:67082-67153`) with the generic
            // centred anchor - the pumpkin arm's shape. The C#'s
            // `GetAlpha` overrides (34 white via `Main.cs:23120-23123`,
            // 289 (250,250,250,50) `NPC.cs:93609-93611`, the 694 book-state
            // white/lerp `:93417-93427`) ARE carried - the tint seam below
            // (W4-c3 gap 4, the d70 co-lane pattern).
            NpcKind::CursedSkull => {
                pumpkin_moon_draw(assets.npc_sheet(34), npc_rows(34), n, w, h)
            }
            NpcKind::GiantCursedSkull => {
                pumpkin_moon_draw(assets.npc_sheet(289), npc_rows(289), n, w, h)
            }
            NpcKind::WaterBoltMimic => {
                pumpkin_moon_draw(assets.npc_sheet(694), npc_rows(694), n, w, h)
            }
""", """            // W4-c3: the WaterBoltMimic draws its own sheet
            // (`STYLE10_SHEETS`) at `SetFrameSize`'s integer-division row
            // height over `Main.npcFrameCount[694]` = 30 (`Main.cs:67153`)
            // with the generic centred anchor - the pumpkin arm's shape.
            // The 34/289 draw arms dropped per E8 (d34/d38's arms above
            // own them); the C#'s `GetAlpha` overrides for ALL THREE are
            // carried by the tint seam below (W4-c3 gap 4, keep-tint-
            // rekeyed per 329 s3 - d34/d38 have no GetAlpha overrides).
            NpcKind::WaterBoltMimic => {
                pumpkin_moon_draw(assets.npc_sheet(694), npc_rows(694), n, w, h)
            }
""")

def ui_bestiary(rep, box):
    # portraits: drop the c3 34/289 dup arms (d34/d38 own), keep 694
    rep("""        // W4-c3 (the style-10 family): the three kinds' portraits - the
        // sheet's own row height over `Main.npcFrameCount[type]` (34 three
        // frames, 289 six, 694 thirty; `Main.cs:67082-67153`), the frame
        // width the sheet's own (the small-kinds arm's shape).
        (NpcKind::CursedSkull, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 3.0)
                .unwrap_or(28.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::GiantCursedSkull, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(44.0);
            (sheet_i, h, 44.0, h, white)
        }
        (NpcKind::WaterBoltMimic, _) => {
""", """        // W4-c3: the WaterBoltMimic's portrait - the sheet's own row
        // height over `Main.npcFrameCount[694]` = 30 (`Main.cs:67153`),
        // the frame width the sheet's own (the small-kinds arm's shape).
        // The 34/289 portrait arms dropped per E8 (d34/d38's arms own).
        (NpcKind::WaterBoltMimic, _) => {
""")

def anchors(rep, box):
    # repoint the style-10 wiring row (the marker survives on the 694 arm)
    rep("""crates/tr-app/src/npc.rs|unreachable!("aiStyle 10 rides the style axis (W4-c3)")|W4-c3 style-10 wiring - the three kinds WIRED: NpcKind::{CursedSkull (protocol 355, type 34), GiantCursedSkull (289), WaterBoltMimic (694)} + the aiStyle-axis 10 claim (npc.rs update_rng_dust) calling npc_ai_style10::style10_ai (cs_id/just_hit/netMode-0); the drains kb_resist_override + wants_sound Item8 + wants_sphere WaterSphere (Bottom placement); the 299 Shadowflames proj_data row + AI_001 arm (projectile_ai001_a, the localAI[0] Item8-at-6 cue) + the on-hit Cursed 23/240 (damage_player, StatusPlayer :12280); FindFrame 34/289 reuse npc_findframe_b + the 694 book strip inline; the bestiary entries/sheets/portraits + NPC_Killed_52/dd2_book_staff_cast_0 cues; the oldPosition/dead-band-X integration stamps; code-only - 3 wiring witnesses written not run, no compiler invoked
""", """crates/tr-app/src/npc.rs|unreachable!("aiStyle 10 rides the style axis (W4-c3)")|W4-c3 style-10 wiring - the family WIRED through the axis: WaterBoltMimic (694, c3's own kind) + the canonical CursedSkull (d34@267, type 34) / GiantCursedSkull (d38@289) routed through the aiStyle-axis 10 claim (npc.rs update_rng_dust) calling npc_ai_style10::style10_ai (cs_id/just_hit/netMode-0; the E8 merge dropped c3's 355/289 dup identities per research/329 s3, the axis superset keeps the machine for all three); the drains kb_resist_override + wants_sound Item8 + wants_sphere WaterSphere (Bottom placement); the 299 Shadowflames proj_data row + AI_001 arm (projectile_ai001_a, the localAI[0] Item8-at-6 cue) + the on-hit Cursed 23/240 (damage_player, StatusPlayer :12280); FindFrame 34/289 ride d34/d38's inline case arms + the 694 book strip inline; the bestiary entries/sheets/portraits + NPC_Killed_52/dd2_book_staff_cast_0 cues; the oldPosition/dead-band-X integration stamps; code-only - 3 wiring witnesses written not run, no compiler invoked
""")

edit_file("crates/tr-app/src/render/entities.rs", entities)
edit_file("crates/tr-app/src/ui/bestiary.rs", ui_bestiary)
edit_file("anchors.txt", anchors)
