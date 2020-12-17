use std::fs::File;
use std::io::Read;

pub struct Rand();

impl Rand {
    pub fn random_u8() -> Option<u8> {
        if let Ok(mut f) = File::open("/dev/urandom") {
            let mut buf = [0u8];
            f.read_exact(&mut buf).unwrap();
            Some(buf[0])
        } else {
            None
        }
    }
}
