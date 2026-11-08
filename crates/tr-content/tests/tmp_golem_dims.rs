//! TEMP probe: print real sheet dimensions (delete after use).
use tr_content::load_texture;

#[test]
fn probe_golem_sheet_dims() {
    for n in [
        "NPC_245", "NPC_246", "NPC_247", "NPC_249", "NPC_4", "NPC_50", "NPC_222", "NPC_113",
        "NPC_35", "NPC_636",
    ] {
        let p = format!("C:/Users/DreadLord/Desktop/Terraria-Rust/assets/Content/Images/{n}.xnb");
        match load_texture(&p) {
            Ok(img) => println!(
                "{n}: {}x{} (rows of width: {:.3})",
                img.width,
                img.height,
                img.height as f32 / img.width as f32
            ),
            Err(e) => println!("{n}: ERR {e}"),
        }
    }
}
