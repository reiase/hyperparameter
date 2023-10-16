use std::ffi::{CStr, CString};
use xxhash_rust::const_xxh64;

// rust version of xxhash, original C++ verison is from: https://github.com/ekpyron/xxhashct/blob/master/xxh64.hpp
static PRIME1: u64 = 11400714785074694791u64;
static PRIME2: u64 = 14029467366897019727u64;
static PRIME3: u64 = 1609587929392839161u64;
static PRIME4: u64 = 9650029242287828579u64;
static PRIME5: u64 = 2870177450012600261u64;

fn rotl(x: u64, r: i32) -> u64 {
    (x << r) | (x >> (64 - r))
}

fn mix1(h: u64, prime: u64, rshift: i32) -> u64 {
    (h ^ (h >> rshift)) * prime
}

fn mix2(p: u64, v: u64) -> u64 {
    // v = 0, by default
    rotl(v + p * PRIME2, 31) * PRIME1
}

fn mix3(h: u64, v: u64) -> u64 {
    (h ^ mix2(v, 0)) * PRIME1 + PRIME4
}

fn endian32(v: &[u8]) -> u32 {
    // return uint32_t(uint8_t(v[0]))
    //     | (uint32_t(uint8_t(v[1])) << 8)
    //     | (uint32_t(uint8_t(v[2])) << 16)
    //     | (uint32_t(uint8_t(v[3])) << 24);
    (v[0] as u32) + ((v[1] as u32) << 8) + ((v[2] as u32) << 16) + ((v[3] as u32) << 24)
}

fn endian64(v: &[u8]) -> u64 {
    // return uint64_t(uint8_t(v[0]))
    //     | (uint64_t(uint8_t(v[1])) << 8)
    //     | (uint64_t(uint8_t(v[2])) << 16)
    //     | (uint64_t(uint8_t(v[3])) << 24)
    //     | (uint64_t(uint8_t(v[4])) << 32)
    //     | (uint64_t(uint8_t(v[5])) << 40)
    //     | (uint64_t(uint8_t(v[6])) << 48)
    //     | (uint64_t(uint8_t(v[7])) << 56);
    (v[0] as u64)
        + ((v[1] as u64) << 8)
        + ((v[2] as u64) << 16)
        + ((v[3] as u64) << 24)
        + ((v[4] as u64) << 32)
        + ((v[5] as u64) << 40)
        + ((v[6] as u64) << 48)
        + ((v[7] as u64) << 56)
}

fn fetch64(p: &[u8], v: u64) -> u64 {
    // v=0, by default
    mix2(endian64(p), v)
}

fn fetch32(p: &[u8]) -> u64 {
    (endian32(p) as u64) * PRIME1
}

fn fetch8(p: &[u8]) -> u64 {
    (p[0] as u64) * PRIME5
}

fn finalize(h: u64, p: &[u8], len: u64) -> u64 {
    // (len >= 8) ? (finalize(rotl(h ^ fetch64(p), 27) * PRIME1 + PRIME4, p + 8, len - 8)) : ((len >= 4) ? (finalize(rotl(h ^ fetch32(p), 23) * PRIME2 + PRIME3, p + 4, len - 4)) : ((len > 0) ? (finalize(rotl(h ^ fetch8(p), 11) * PRIME1, p + 1, len - 1)) : (mix1(mix1(mix1(h, PRIME2, 33), PRIME3, 29), 1, 32))))
    if len >= 8 {
        finalize(
            rotl(h ^ fetch64(p, 0), 27) * PRIME1 + PRIME4,
            &p[8..],
            len - 8,
        )
    } else if len >= 4 {
        finalize(rotl(h ^ fetch32(p), 23) * PRIME2 + PRIME3, &p[4..], len - 4)
    } else if len > 0 {
        finalize(rotl(h ^ fetch8(p), 11) * PRIME1, &p[1..], len - 1)
    } else {
        mix1(mix1(mix1(h, PRIME2, 33), PRIME3, 29), 1, 32)
    }
}

fn h32bytes(p: &[u8], len: u64, v1: u64, v2: u64, v3: u64, v4: u64) -> u64 {
    //  (len >= 32) ? h32bytes(p + 32, len - 32, fetch64(p, v1), fetch64(p + 8, v2), fetch64(p + 16, v3), fetch64(p + 24, v4)) : mix3(mix3(mix3(mix3(rotl(v1, 1) + rotl(v2, 7) + rotl(v3, 12) + rotl(v4, 18), v1), v2), v3), v4)
    if len >= 32 {
        h32bytes(
            &p[32..],
            len - 32,
            fetch64(p, v1),
            fetch64(&p[8..], v2),
            fetch64(&p[16..], v3),
            fetch64(&p[24..], v4),
        )
    } else {
        mix3(
            mix3(
                mix3(
                    mix3(rotl(v1, 1) + rotl(v2, 7) + rotl(v3, 12) + rotl(v4, 18), v1),
                    v2,
                ),
                v3,
            ),
            v4,
        )
    }
}

fn h32(p: &[u8], len: u64, seed: u64) -> u64 {
    h32bytes(
        p,
        len,
        seed + PRIME1 + PRIME2,
        seed + PRIME2,
        seed,
        seed - PRIME1,
    )
}

fn xxh(p: &[u8], len: u64, seed: u64) -> u64 {
    // finalize((len >= 32 ? h32bytes(p, len, seed) : seed + PRIME5) + len, p + (len & ~0x1F), len & 0x1F)
    if len >= 32 {
        finalize(
            h32(p, len, seed) + len,
            &p[((len & (!0x1F)) as usize)..],
            len & 0x1F,
        )
    } else {
        finalize(
            seed + PRIME5 + len,
            &p[((len & (!0x1F)) as usize)..],
            len & 0x1F,
        )
    }
}

pub const fn xxhash(u: &[u8]) -> u64 {
    const_xxh64::xxh64(u, 42)
}

pub trait XXHashable {
    fn xxh(&self) -> u64;
}

impl XXHashable for String {
    fn xxh(&self) -> u64 {
        xxh(self.as_bytes(), self.len() as u64, 42)
    }
}

impl XXHashable for &String {
    fn xxh(&self) -> u64 {
        xxh(self.as_bytes(), self.len() as u64, 42)
    }
}

impl XXHashable for &str {
    fn xxh(&self) -> u64 {
        xxh(self.as_bytes(), self.len() as u64, 42)
    }
}

impl XXHashable for CStr {
    fn xxh(&self) -> u64 {
        let bs = self.to_bytes();
        xxh(bs, bs.len() as u64, 42)
    }
}

impl XXHashable for CString {
    fn xxh(&self) -> u64 {
        let bs = self.to_bytes();
        xxh(bs, bs.len() as u64, 42)
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
