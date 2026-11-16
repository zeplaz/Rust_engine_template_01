//! Stable hashed string ids (MEM-07 · gea4v1 §6.4.3.1–2; LEGACY `memory_management`).
//!
//! FNV-1a 64 over the UTF-8 bytes: pure function of the string — no pointer identity, no
//! per-process seed (unlike `std` `RandomState`), so ids are identical across runs, reloads
//! and platforms. Catalogs key on [`StringId`] and keep the source tag only for collision checks.

use std::fmt;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Deterministic 64-bit id of a tag / cue / RON key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringId(pub u64);

impl StringId {
    /// Hash `s` (usable in `const` context for compile-time tag ids).
    #[must_use]
    pub const fn new(s: &str) -> Self {
        let bytes = s.as_bytes();
        let mut h = FNV_OFFSET;
        let mut i = 0;
        while i < bytes.len() {
            h ^= bytes[i] as u64;
            h = h.wrapping_mul(FNV_PRIME);
            i += 1;
        }
        Self(h)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<&str> for StringId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Debug for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringId({:#018x})", self.0)
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Published FNV-1a 64 test vectors — locks the algorithm so saved/hashed ids never drift.
    #[test]
    fn fnv1a64_reference_vectors() {
        assert_eq!(StringId::new("").raw(), 0xcbf2_9ce4_8422_2325);
        assert_eq!(StringId::new("a").raw(), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(StringId::new("foobar").raw(), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn same_string_same_id_distinct_allocations() {
        let owned = String::from("ui.click");
        let other = format!("ui.{}", "click");
        assert_ne!(owned.as_ptr(), other.as_ptr());
        assert_eq!(StringId::new(&owned), StringId::new(&other));
        assert_eq!(StringId::new("ui.click"), StringId::from(owned.as_str()));
        assert_ne!(StringId::new("ui.click"), StringId::new("ui.confirm"));
    }

    #[test]
    fn const_and_runtime_agree() {
        const ID: StringId = StringId::new("combat.shot");
        assert_eq!(ID, StringId::new(&String::from("combat.shot")));
    }
}
