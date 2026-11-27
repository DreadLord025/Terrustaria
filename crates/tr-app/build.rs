// Build script: attaches the repo-root `app.ico` to the Windows executable and
// extracts its 256x256 PNG entry as the window icon (`OUT_DIR/icon.png`, decoded
// by `app/loop.rs`). The old path drew both from the game's `Logo.xnb`, which
// coupled the launcher icon to retail content; the icon now lives in the repo.

use std::path::PathBuf;

/// A 1x1 fully transparent RGBA PNG, written when `app.ico` is missing or
/// carries no PNG entry (the window icon is optional, the `include_bytes!`
/// in `app/loop.rs` is not).
const FALLBACK_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let app_ico = manifest.join("../../app.ico");
    println!("cargo:rerun-if-changed={}", app_ico.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // `app/loop.rs` `include_bytes!`s `OUT_DIR/icon.png`, so the file must exist
    // even when the icon cannot be read: a missing output would fail the build,
    // not just drop the icon. Fall back to a 1x1 transparent PNG.
    match std::fs::read(&app_ico) {
        Ok(bytes) => match largest_png(&bytes) {
            Some(png) => {
                let _ = std::fs::write(out.join("icon.png"), png);
            }
            None => {
                println!("cargo:warning=app.ico has no PNG entry; using a blank window icon");
                let _ = std::fs::write(out.join("icon.png"), FALLBACK_PNG);
            }
        },
        Err(e) => {
            println!(
                "cargo:warning=cannot read {} ({e}); using a blank window icon",
                app_ico.display()
            );
            let _ = std::fs::write(out.join("icon.png"), FALLBACK_PNG);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if app_ico.is_file() {
            match find_rc() {
                Some(rc) => {
                    let rc_file = out.join("app.rc");
                    let ico_path = app_ico.display().to_string().replace('\\', "\\\\");
                    // The exe's version metadata: the port ships the original game's
                    // 4-part version (Cargo's SemVer carries it as `1.4.5+8`).
                    let rc_text = format!(
                        r#"1 ICON "{ico_path}"
1 VERSIONINFO
FILEVERSION 1,4,5,8
PRODUCTVERSION 1,4,5,8
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Terraria-Rust"
      VALUE "FileDescription", "Terraria 1.4.5.8 (Rust port)"
      VALUE "FileVersion", "1.4.5.8"
      VALUE "InternalName", "terraria-rust"
      VALUE "OriginalFilename", "terraria-rust.exe"
      VALUE "ProductName", "Terraria"
      VALUE "ProductVersion", "1.4.5.8"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
                    );
                    let _ = std::fs::write(&rc_file, rc_text);
                    let res = out.join("app.res");
                    let ok = std::process::Command::new(&rc)
                        .arg("/nologo")
                        .arg("/fo")
                        .arg(&res)
                        .arg(&rc_file)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if ok {
                        println!("cargo:rustc-link-arg-bins={}", res.display());
                    } else {
                        println!("cargo:warning=rc.exe failed; the exe icon is skipped");
                    }
                }
                None => println!("cargo:warning=rc.exe not found; the exe icon is skipped"),
            }
        }
    }
}

/// The largest PNG-compressed image inside an `.ico` container (the ICO's
/// `ICONDIR` + 16-byte entries; a PNG entry starts with the PNG signature).
fn largest_png(ico: &[u8]) -> Option<&[u8]> {
    if ico.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
    let mut best: Option<(u32, &[u8])> = None;
    for i in 0..count {
        let o = 6 + i * 16;
        if o + 16 > ico.len() {
            break;
        }
        let w = if ico[o] == 0 { 256 } else { u32::from(ico[o]) };
        let h = if ico[o + 1] == 0 { 256 } else { u32::from(ico[o + 1]) };
        let size = u32::from_le_bytes(ico[o + 8..o + 12].try_into().ok()?) as usize;
        let off = u32::from_le_bytes(ico[o + 12..o + 16].try_into().ok()?) as usize;
        if size < 8 || off.checked_add(size).is_none_or(|end| end > ico.len()) {
            continue;
        }
        let data = &ico[off..off + size];
        if data.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
            && best.is_none_or(|(area, _)| w * h > area)
        {
            best = Some((w * h, data));
        }
    }
    best.map(|(_, png)| png)
}

/// Finds the newest `rc.exe` of the installed Windows SDK (build tools are x64).
fn find_rc() -> Option<PathBuf> {
    let kits = std::path::Path::new("C:/Program Files (x86)/Windows Kits/10/bin");
    let mut versions: Vec<_> = std::fs::read_dir(kits)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    versions.sort();
    versions.reverse();
    versions
        .into_iter()
        .map(|v| v.join("x64/rc.exe"))
        .find(|p| p.is_file())
}
