fn main() {
    let mut args = std::env::args().skip(1);
    while let Some(p) = args.next() {
        let raw = std::fs::read(&p).unwrap();
        let f = tr_content::xnb::XnbFile::read(&raw).unwrap();
        let mut r = tr_content::xnb::ByteReader::new(&f.payload);
        let _ = r.read_i32();
        let w = r.read_i32().unwrap();
        let h = r.read_i32().unwrap();
        println!("{p} {w}x{h}");
    }
}
