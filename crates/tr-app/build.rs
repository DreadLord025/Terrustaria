// Build script: draws the executable icon and the window icon from the game's own
// `Logo.xnb`, then attaches it to the Windows resources.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let content = manifest.join("../../assets/Content/Images/Logo.xnb");
    println!("cargo:rerun-if-changed={}", content.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    if let Ok(logo) = tr_content::load_texture(&content) {
        // Square canvas, logo scaled to fit at 256x256.
        let mut canvas = image::RgbaImage::new(256, 256);
        let scale = (256.0 / logo.width.max(1) as f32).min(256.0 / logo.height.max(1) as f32);
        let w = (logo.width as f32 * scale).round().max(1.0) as u32;
        let h = (logo.height as f32 * scale).round().max(1.0) as u32;
        let src = image::RgbaImage::from_raw(logo.width, logo.height, logo.pixels.clone())
            .unwrap_or_else(|| image::RgbaImage::new(1, 1));
        let resized = image::imageops::resize(&src, w, h, image::imageops::FilterType::Lanczos3);
        image::imageops::overlay(
            &mut canvas,
            &resized,
            ((256 - w) / 2) as i64,
            ((256 - h) / 2) as i64,
        );
        // Window icon (RGBA, decoded by the app).
        let _ = canvas.save(out.join("icon.png"));
        // ICO with a single 256x256 PNG-compressed entry.
        let mut cursor = std::io::Cursor::new(Vec::new());
        if image::DynamicImage::ImageRgba8(canvas)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .is_ok()
        {
            let png = cursor.into_inner();
            let mut ico = Vec::new();
            ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // ICONDIR: type icon, 1 image
            ico.extend_from_slice(&[0, 0, 0, 0]); // 256x256, no palette, reserved
            ico.extend_from_slice(&1u16.to_le_bytes()); // planes
            ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
            ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
            ico.extend_from_slice(&22u32.to_le_bytes()); // data offset
            ico.extend_from_slice(&png);
            let _ = std::fs::write(out.join("icon.ico"), ico);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ico = out.join("icon.ico");
        if ico.exists() {
            match find_rc() {
                Some(rc) => {
                    let rc_file = out.join("app.rc");
                    let ico_path = ico.display().to_string().replace('\\', "\\\\");
                    let _ = std::fs::write(&rc_file, format!("1 ICON \"{ico_path}\"\n"));
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
