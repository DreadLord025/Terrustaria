fn main() {
    for a in std::env::args().skip(1) {
        let p = std::path::PathBuf::from(&a);
        match tr_content::texture::load_texture(&p) {
            Ok(img) => println!("{} {}x{}", p.file_name().unwrap().to_string_lossy(), img.width, img.height),
            Err(e) => println!("{a} ERR {e}"),
        }
    }
}
