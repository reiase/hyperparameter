use std::ffi::{CStr, CString};
use xxhash_rust::const_xxh64;

pub const fn xxhash(u: &[u8]) -> u64 {
    const_xxh64::xxh64(u, 42)
}

pub trait XXHashable {
    fn xxh(&self) -> u64;
}

impl XXHashable for String {
    fn xxh(&self) -> u64 {
        xxhash(self.as_bytes())
    }
}

impl XXHashable for &String {
    fn xxh(&self) -> u64 {
        xxhash(self.as_bytes())
    }
}

impl XXHashable for &str {
    fn xxh(&self) -> u64 {
        xxhash(self.as_bytes())
    }
}

impl XXHashable for CStr {
    fn xxh(&self) -> u64 {
        xxhash(self.to_bytes())
    }
}

impl XXHashable for CString {
    fn xxh(&self) -> u64 {
        xxhash(self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::xxh::xxhash;
    use crate::xxh::XXHashable;
    #[test]
    fn test_xxhstr() {
        assert_eq!("12345".xxh(), 13461425039964245335u64);
        assert_eq!(
            "12345678901234567890123456789012345678901234567890".xxh(),
            5815762531248152886
        );
        assert_eq!(
            "0123456789abcdefghijklmnopqrstuvwxyz".xxh(),
            5308235351123835395
        );
    }

    #[test]
    fn test_xxhash() {
        assert_eq!(xxhash("12345".as_bytes()), 13461425039964245335u64);
        assert_eq!(
            xxhash("12345678901234567890123456789012345678901234567890".as_bytes()),
            5815762531248152886
        );
        assert_eq!(
            xxhash("0123456789abcdefghijklmnopqrstuvwxyz".as_bytes()),
            5308235351123835395
        );
    }
}
