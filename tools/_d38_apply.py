# W4-d38 sheets phase: apply the whitelist edit + witness module to
# entities.rs byte-safely (utf-8, LF preserved), with slice control.
# usage: python tools/_d38_apply.py <slice 1..7> [--whitelist b2b4|b5b6|all]
import subprocess, sys, os, re
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENT = os.path.join(ROOT,'crates','tr-app','src','render','entities.rs')
base = subprocess.run(['git','show','8176c078:crates/tr-app/src/render/entities.rs'],
                      capture_output=True,cwd=ROOT).stdout.decode('utf-8')

WL_OLD = """                    // The small-kinds wiring wave: the BlazingWheel's spin
                    // (`NPC.cs:24626`/`:24645`, \u00b10.13/tick) and the Dungeon
                    // Spirit's aim (`NPC.cs:32753`, the atan2 - 1.57).
                    | NpcKind::BlazingWheel
                    | NpcKind::DungeonSpirit"""
WL_B2B4 = """                    // W4-d38 sheets phase (b2/b4) \u2014 the rotation whitelist:
                    // ONLY kinds whose ported AI/FindFrame writes `rotation`
                    // join (the C#'s generic draw rotates every NPC,
                    // `Main.cs:26632`, but the port gates per kind). The
                    // style-8 casters (281-286, `NPC.cs:20817-21376`), the
                    // style-3 shooters (287/290-296), the style-1 slime hops
                    // (302/304/333-336) and the style-7 critters
                    // (299/300/303/337) write NONE (the slime balloon-freeze
                    // zero is not a tilt) \u2014 they stay OUT. In here: the 289
                    // skull's aim (`NPC.cs:21750-21766`), the 297/298 birds'
                    // vx*0.1 (`:73634`), the 301 raven's vx*0.1 (`:73589`).
                    | NpcKind::GiantCursedSkull
                    | NpcKind::BirdBlue
                    | NpcKind::BirdRed
                    | NpcKind::Raven
                    // The small-kinds wiring wave: the BlazingWheel's spin
                    // (`NPC.cs:24626`/`:24645`, \u00b10.13/tick) and the Dungeon
                    // Spirit's aim (`NPC.cs:32753`, the atan2 - 1.57).
                    | NpcKind::BlazingWheel
                    | NpcKind::DungeonSpirit"""
WL_B5B6 = """                    // W4-d38 sheets phase (b5/b6): the 317/318 eyes' atan2
                    // (`NPC.cs:74019-74031`), the 316 ghost's vx*0.1
                    // (`:74858`), the 328 blade's atan2+1.57 (`:33296`/
                    // `:33305`/`:33340`).
                    | NpcKind::DemonEyeOwl
                    | NpcKind::DemonEyeSpaceship
                    | NpcKind::Ghost
                    | NpcKind::PumpkingBlade
"""

ROWS = [
 ("RaggedCaster","40.0","168.0",3),("RaggedCasterOpenCoat","40.0","168.0",3),
 ("Necromancer","40.0","168.0",3),("NecromancerArmored","44.0","168.0",3),
 ("DiabolistRed","42.0","168.0",3),
 ("DiabolistWhite","40.0","170.0",3),("BoneLee","44.0","896.0",16),
 ("GiantCursedSkull","52.0","384.0",6),("Paladin","66.0","816.0",12),
 ("SkeletonSniper","68.0","1260.0",21),
 ("TacticalSkeleton","60.0","1260.0",21),("SkeletonCommando","60.0","1200.0",20),
 ("AngryBonesBig","40.0","896.0",16),("AngryBonesBigMuscle","40.0","840.0",15),
 ("AngryBonesBigHelmet","40.0","840.0",15),
 ("BirdBlue","30.0","120.0",5),("BirdRed","30.0","120.0",5),
 ("Squirrel","50.0","192.0",6),("Mouse","20.0","96.0",6),("Raven","44.0","270.0",5),
 ("SlimeMasked","40.0","56.0",2),("BunnySlimed","48.0","280.0",7),
 ("HoppinJack","48.0","80.0",2),("Ghost","40.0","224.0",4),("DemonEyeOwl","48.0","64.0",2),
 ("DemonEyeSpaceship","40.0","64.0",2),("SkeletonTopHat","32.0","896.0",16),
 ("SkeletonAstonaut","42.0","840.0",15),("SkeletonAlien","36.0","896.0",16),
 ("PumpkingBlade","106.0","86.0",1),
 ("SlimeRibbonWhite","38.0","76.0",2),("SlimeRibbonYellow","38.0","76.0",2),
 ("SlimeRibbonGreen","38.0","76.0",2),("SlimeRibbonRed","38.0","76.0",2),
 ("BunnyXmas","48.0","280.0",7),
]

def module(n):
    rows = ROWS[:n*5]
    lines = "".join(f"        (NpcKind::{k}, {w}, {h}, {r}),"
        + (" // 170/3 = 56 r2 (the C# drops the 2 px tail)" if k=="DiabolistWhite" else "")
        + "\n" for k,w,h,r in rows)
    return f"""
/// W4-d38 sheets phase \u2014 the frame-strip witnesses for the wave's kinds:
/// every draw arm's sheet is packed (`STATIC_SHEET_NAMES` + the `D38_*` rows
/// feeding `npc_sheets`), every kind's row count matches the verbatim
/// `Main.npcFrameCount` table (`Main.cs:67082`), and the REAL retail dims
/// (measured by tools/_d38_dims.py through atlas_sim's LZX/XNB reader) divide
/// into whole `SetFrameSize` rows (`NPC.cs:90780-90800` integer division).
/// WRITTEN, NOT RUN (code-only mode).
#[cfg(test)]
mod d38_sheet_tests {{
    use super::*;

    /// (kind, real sheet w, real sheet h, rows) \u2014 the dims decoded from
    /// `assets/Content/Images/NPC_<id>.xnb` (tools/_d38_dims.json).
    const STRIPS: &[(NpcKind, f32, f32, u32)] = &[
{lines}    ];

    /// Every wave kind draws its OWN sheet, the row count is the verbatim
    /// `Main.npcFrameCount` value, and the real strip divides into whole rows
    /// (`NPC.SetFrameSize`'s integer division \u2014 the 286 tail is the ONE
    /// non-multiple, kept as the C# drops it).
    #[test]
    fn w4_d38_the_strips_divide_into_their_vanilla_rows() {{
        assert_eq!(STRIPS.len(), {len(rows)}, "the wave so far");
        for &(kind, w, h, rows) in STRIPS {{
            let net = texture_net_id(kind);
            assert_eq!(net, kind.cs_id(), "{{kind:?}} draws its own sheet");
            assert_eq!(npc_rows(net), rows, "{{kind:?}}: NPC_{{net}} rows");
            let t = BgTex {{ uv: [0.0, 0.0, 1.0, 1.0], w, h }};
            let fh = vanilla_row_h(&t, rows);
            assert_eq!(fh, (h as i32 / rows as i32) as f32, "{{kind:?}} row height");
            if net != 286 {{
                assert_eq!(h as i32 % rows as i32, 0, "{{kind:?}}: whole rows");
            }} else {{
                assert_eq!(h as i32 % rows as i32, 2, "NPC_286's 2 px tail");
            }}
            // The draw arm's own arithmetic: pumpkin_moon_draw keeps the
            // sheet width and centres the row on the box.
            let n = Npc::new(kind, 100.0, 100.0, 1, 0.0);
            let (bw, bh) = kind.size();
            let (_, dfh, size, pos) = pumpkin_moon_draw(Some(&t), rows, &n, bw, bh);
            assert_eq!(dfh, fh, "{{kind:?}}: draw row == SetFrameSize row");
            assert_eq!(size, [w, fh], "{{kind:?}}: the frame keeps the sheet width");
            assert_eq!(pos, [100.0 + bw * 0.5 - w * 0.5, 100.0 + bh * 0.5 - fh * 0.5]);
            // The frame the machine walks stays inside the strip.
            let uv = npc_frame_uv(&t, fh, rows - 1);
            assert!(uv[3] <= 1.0 + 1e-6, "{{kind:?}}: the last row stays in-bounds");
        }}
    }}

    /// The M4-NPC-3v audit, d38 edition: every sheet the wave's draw arms ask
    /// `npc_sheet` for is in the packing list, and the `D38_*` rows feeding
    /// `npc_sheets` are exactly the wave's 35 net ids.
    #[test]
    fn w4_d38_every_draw_sheet_is_packed() {{
        use crate::assets::STATIC_SHEET_NAMES;
        let all: Vec<u32> = D38_B1_SHEETS.iter().chain(D38_B2_SHEETS)
            .chain(D38_B3_SHEETS).chain(D38_B4_SHEETS).chain(D38_B5_SHEETS)
            .chain(D38_B6_SHEETS).chain(D38_B7_SHEETS).copied().collect();
        assert_eq!(all.len(), 35);
        for &id in &all {{
            let name = format!("NPC_{{id}}");
            assert!(STATIC_SHEET_NAMES.contains(&name.as_str()),
                "{{name}} has a D38 row but is never packed");
        }}
        for &(kind, _, _, _) in STRIPS {{
            let net = texture_net_id(kind);
            assert!(all.contains(&(net as u32)), "NPC_{{net}} has no D38 row");
        }}
    }}
}}
"""

slice_n = int(sys.argv[1])
wl = sys.argv[2] if len(sys.argv)>2 else 'none'
out = base
if wl in ('b2b4','all'):
    assert WL_OLD in out
    out = out.replace(WL_OLD, WL_B2B4)
if wl in ('b5b6','all'):
    anchor = "                    | NpcKind::Raven\n"
    assert anchor in out
    out = out.replace(anchor, anchor + WL_B5B6)
if slice_n > 0:
    out = out + module(slice_n)
open(ENT,'w',encoding='utf-8',newline='').write(out)
add = len(out.splitlines()) - len(base.splitlines())
print(f'slice={slice_n} wl={wl}: +{add} lines vs HEAD')
