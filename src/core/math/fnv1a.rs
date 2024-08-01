#[must_use]
const fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash
}

#[must_use]
pub const fn hash_str(s: &str) -> u64 {
    fnv1a_hash(s.as_bytes())
}

#[must_use]
pub const fn hash_bytes(bytes: &[u8]) -> u64 {
    fnv1a_hash(bytes)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fnv1a(u64);

impl Fnv1a {
    #[must_use]
    pub const fn from_str(s: &str) -> Self {
        Self(hash_str(s))
    }

    #[must_use]
    pub const fn from_bytes(bytes: &[u8]) -> Self {
        Self(hash_bytes(bytes))
    }

    pub const fn id(self) -> u64 {
        self.0
    }
}