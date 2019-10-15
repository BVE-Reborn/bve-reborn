fn main() {
    loop {
        let closure = |data: &[u8]| {
            let string = String::from_utf8_lossy(data);
            bve::parse::mesh::instructions::create_instructions(&string, bve::parse::mesh::FileType::B3D);
            bve::parse::mesh::instructions::create_instructions(&string, bve::parse::mesh::FileType::CSV);
        };
        honggfuzz::fuzz!(|data: &[u8]| { closure(data) });
    }
}
