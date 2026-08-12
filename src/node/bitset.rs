#[cfg(feature = "roaring")]
use croaring::{Bitmap, Portable};
use serde::{Deserialize, Serialize};

/// Dynamic bitset supporting >128 bits for multi-tenant filtering.
///
/// Backed by `croaring::Bitmap` (RoaringBitmap) — a compressed bitset that
/// is efficient for both sparse and dense sets. The sentinel `all_set()`
/// uses an internal `all_set` flag to signal "match everything" without
/// allocating a full u32::MAX bitmap.
///
/// Serialization (serde) uses croaring's Portable format wrapped in a
/// `(all_set: bool, bytes: Vec<u8>)` tuple.
#[cfg(feature = "roaring")]
#[derive(Clone, Debug)]
pub struct FilterBitset {
    inner: Bitmap,
    all_set: bool,
}

/// Dynamic bitset supporting >128 bits for multi-tenant filtering.
///
/// Pure-Rust `Vec<u64>` fallback used when croaring's C FFI is unavailable
/// (e.g. `wasm32-unknown-unknown`). The all-set sentinel is a single
/// `u64::MAX` word.
#[cfg(not(feature = "roaring"))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterBitset(Vec<u64>);

#[cfg(feature = "roaring")]
impl FilterBitset {
    /// Create an empty bitset.
    pub fn new() -> Self {
        Self {
            inner: Bitmap::new(),
            all_set: false,
        }
    }

    /// Pre-allocate capacity for `bits` entries (hint only).
    pub fn with_capacity(bits: usize) -> Self {
        let containers = bits.div_ceil(1 << 16) as u32;
        Self {
            inner: Bitmap::with_container_capacity(containers),
            all_set: false,
        }
    }

    /// Sentinel meaning "match everything" — used as a no-filter query mask.
    /// Uses an internal flag rather than allocating a full bitmap of all u32 values.
    pub fn all_set() -> Self {
        Self {
            inner: Bitmap::new(),
            all_set: true,
        }
    }

    /// Returns `true` if this is the all-set sentinel.
    pub fn is_all_set(&self) -> bool {
        self.all_set
    }

    /// Returns `true` if no bits are set (empty bitset, non-all-set).
    pub fn is_empty(&self) -> bool {
        !self.all_set && self.inner.is_empty()
    }

    /// Number of u64 words backing this bitset (deprecated — always 0 for RoaringBitmap).
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        0
    }

    /// Set bit at position `pos`.
    pub fn set_bit(&mut self, pos: usize) {
        self.inner.add(pos as u32);
    }

    /// Check if bit at position `pos` is set.
    pub fn has_bit(&self, pos: usize) -> bool {
        self.inner.contains(pos as u32)
    }

    /// Check if ALL bits set in `mask` are also set in `self`.
    ///
    /// The all-set sentinel (produced by `FilterBitset::all_set()`) causes
    /// this method to return `true` unconditionally, acting as a no-filter.
    pub fn matches_mask(&self, mask: &FilterBitset) -> bool {
        if mask.all_set {
            return true;
        }
        // If `self` is all_set, it matches every mask.
        if self.all_set {
            return true;
        }
        // All bits in mask must be present in self → mask is subset of self
        mask.inner.is_subset(&self.inner)
    }

    /// Convert to a `u128`, reading the first 128 bits of the bitmap.
    pub fn to_u128(&self) -> u128 {
        if self.all_set {
            return !0;
        }
        let mut result: u128 = 0;
        for i in 0..128 {
            if self.inner.contains(i as u32) {
                result |= 1u128 << i;
            }
        }
        result
    }

    /// Create from a `u128` (legacy format — max 128 bits).
    pub fn from_u128(v: u128) -> Self {
        let mut bm = Bitmap::new();
        for i in 0..128 {
            if (v & (1u128 << i)) != 0 {
                bm.add(i as u32);
            }
        }
        Self {
            inner: bm,
            all_set: false,
        }
    }

    /// Serialize to croaring Portable format bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.serialize::<Portable>()
    }

    /// Deserialize from croaring Portable format bytes. Returns `(Self, bytes_consumed)`.
    pub fn from_bytes(data: &[u8]) -> std::io::Result<(Self, usize)> {
        let bitmap = Bitmap::try_deserialize::<Portable>(data).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "FilterBitset: invalid croaring bitmap data",
            )
        })?;
        let consumed = bitmap.get_serialized_size_in_bytes::<Portable>();
        Ok((
            Self {
                inner: bitmap,
                all_set: false,
            },
            consumed,
        ))
    }
}

#[cfg(not(feature = "roaring"))]
impl FilterBitset {
    /// Create an empty bitset.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Pre-allocate capacity for `bits` entries.
    pub fn with_capacity(bits: usize) -> Self {
        let words = bits.div_ceil(64);
        Self(Vec::with_capacity(words))
    }

    /// Sentinel meaning "match everything" — used as a no-filter query mask.
    /// A single `u64::MAX` word signals unbounded matching in `matches_mask`.
    pub fn all_set() -> Self {
        Self(vec![u64::MAX])
    }

    /// Returns `true` if this is the all-set sentinel.
    pub fn is_all_set(&self) -> bool {
        self.0.len() == 1 && self.0[0] == u64::MAX
    }

    /// Returns `true` if no bits are set (empty bitset).
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    /// Number of u64 words backing this bitset.
    pub fn word_count(&self) -> usize {
        self.0.len()
    }

    /// Set bit at position `pos`.
    pub fn set_bit(&mut self, pos: usize) {
        // Match croaring's u32 index space (node_ids are 128-bit hashes).
        let pos = pos as u32 as usize;
        let word = pos / 64;
        let bit = pos % 64;
        if word >= self.0.len() {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1u64 << bit;
    }

    /// Check if bit at position `pos` is set.
    pub fn has_bit(&self, pos: usize) -> bool {
        let pos = pos as u32 as usize;
        let word = pos / 64;
        let bit = pos % 64;
        word < self.0.len() && (self.0[word] & (1u64 << bit)) != 0
    }

    /// Check if ALL bits set in `mask` are also set in `self`.
    ///
    /// The all-set sentinel (produced by `FilterBitset::all_set()`) causes
    /// this method to return `true` unconditionally, acting as a no-filter.
    pub fn matches_mask(&self, mask: &FilterBitset) -> bool {
        if mask.is_all_set() {
            return true;
        }
        let min_len = self.0.len().min(mask.0.len());
        for i in 0..min_len {
            if (self.0[i] & mask.0[i]) != mask.0[i] {
                return false;
            }
        }
        // Any bits set in mask words beyond self's length can't be matched
        if self.0.len() < mask.0.len() {
            for &w in mask.0.iter().skip(self.0.len()) {
                if w != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Convert to a `u128`, truncating if the bitset exceeds 128 bits.
    pub fn to_u128(&self) -> u128 {
        let lo = self.0.first().copied().unwrap_or(0) as u128;
        let hi = self.0.get(1).copied().unwrap_or(0) as u128;
        lo | (hi << 64)
    }

    /// Create from a `u128` (legacy format — max 128 bits).
    pub fn from_u128(v: u128) -> Self {
        let lo = v as u64;
        let hi = (v >> 64) as u64;
        if hi == 0 {
            Self(vec![lo])
        } else {
            Self(vec![lo, hi])
        }
    }

    /// Serialize to length-prefixed bytes: `[word_count: u32 LE][words × u64 LE]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.0.len() * 8);
        buf.extend_from_slice(&(self.0.len() as u32).to_le_bytes());
        for &w in &self.0 {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        buf
    }

    /// Maximum sane number of filter words (256K bits = 32KB).
    const MAX_WORDS: usize = 4096;

    /// Deserialize from length-prefixed bytes. Returns `(Self, bytes_consumed)`.
    pub fn from_bytes(data: &[u8]) -> std::io::Result<(Self, usize)> {
        use std::io::{Error, ErrorKind};
        if data.len() < 4 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "FilterBitset: truncated length",
            ));
        }
        let word_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if word_count > Self::MAX_WORDS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "FilterBitset: word_count exceeds maximum",
            ));
        }
        let needed = word_count
            .checked_mul(8)
            .and_then(|v| v.checked_add(4))
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidData, "FilterBitset: word_count overflow")
            })?;
        if data.len() < needed {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "FilterBitset: truncated words",
            ));
        }
        let mut words = Vec::with_capacity(word_count);
        for i in 0..word_count {
            let off = 4 + i * 8;
            let w = u64::from_le_bytes([
                data[off],
                data[off + 1],
                data[off + 2],
                data[off + 3],
                data[off + 4],
                data[off + 5],
                data[off + 6],
                data[off + 7],
            ]);
            words.push(w);
        }
        Ok((Self(words), needed))
    }
}

impl Default for FilterBitset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "roaring")]
impl PartialEq for FilterBitset {
    fn eq(&self, other: &Self) -> bool {
        if self.all_set != other.all_set {
            return false;
        }
        if self.all_set {
            return true; // both are all_set
        }
        self.inner == other.inner
    }
}

impl From<u128> for FilterBitset {
    fn from(v: u128) -> Self {
        Self::from_u128(v)
    }
}

impl From<FilterBitset> for u128 {
    fn from(bs: FilterBitset) -> Self {
        bs.to_u128()
    }
}

#[cfg(feature = "roaring")]
mod filter_bitset_serde {
    use super::*;

    impl Serialize for FilterBitset {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let bytes = self.inner.serialize::<Portable>();
            (self.all_set, bytes).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for FilterBitset {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let (all_set, bytes): (bool, Vec<u8>) = Deserialize::deserialize(deserializer)?;
            if all_set {
                return Ok(Self {
                    inner: Bitmap::new(),
                    all_set: true,
                });
            }
            let inner = if bytes.is_empty() {
                Bitmap::new()
            } else {
                Bitmap::try_deserialize::<Portable>(&bytes).ok_or_else(|| {
                    serde::de::Error::custom("FilterBitset: invalid croaring bitmap data")
                })?
            };
            Ok(Self {
                inner,
                all_set: false,
            })
        }
    }
}

/// Global sentinel for "match everything" (no filter) in HNSW queries.
pub static ALL_BITSET: std::sync::LazyLock<FilterBitset> =
    std::sync::LazyLock::new(FilterBitset::all_set);

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bitset_with_capacity() {
        let bs = FilterBitset::with_capacity(128);
        assert!(bs.is_empty());
        let mut bs = FilterBitset::with_capacity(128);
        bs.set_bit(0);
        assert!(bs.has_bit(0));
        bs.set_bit(64);
        assert!(bs.has_bit(64));
    }

    #[test]
    fn test_filter_bitset_all_set() {
        let all = FilterBitset::all_set();
        assert!(all.is_all_set());
        assert!(!all.is_empty());
    }

    #[test]
    fn test_filter_bitset_is_empty() {
        assert!(FilterBitset::new().is_empty());
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        assert!(!bs.is_empty());
    }

    #[cfg(feature = "roaring")]
    #[test]
    fn test_filter_bitset_word_count_deprecated() {
        // word_count is deprecated for croaring — always returns 0
        assert_eq!(FilterBitset::new().word_count(), 0);
        let mut bs = FilterBitset::new();
        bs.set_bit(63);
        assert_eq!(bs.word_count(), 0);
        bs.set_bit(64);
        assert_eq!(bs.word_count(), 0);
    }

    #[cfg(not(feature = "roaring"))]
    #[test]
    fn test_filter_bitset_word_count() {
        assert_eq!(FilterBitset::new().word_count(), 0);
        let mut bs = FilterBitset::new();
        bs.set_bit(63);
        assert_eq!(bs.word_count(), 1);
        bs.set_bit(64);
        assert_eq!(bs.word_count(), 2);
    }

    #[test]
    fn test_filter_bitset_set_bit_growth() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        assert!(bs.has_bit(0));

        bs.set_bit(64);
        assert!(bs.has_bit(64));

        bs.set_bit(128);
        assert!(bs.has_bit(128));
    }

    #[test]
    fn test_filter_bitset_has_bit_out_of_range() {
        assert!(!FilterBitset::new().has_bit(0));
        assert!(!FilterBitset::new().has_bit(1000));
    }

    #[test]
    fn test_filter_bitset_matches_mask_all_set() {
        let mut bs = FilterBitset::new();
        bs.set_bit(5);
        assert!(bs.matches_mask(&FilterBitset::all_set()));
    }

    #[test]
    fn test_filter_bitset_matches_mask_self_longer() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        bs.set_bit(100);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_mask_longer_nonzero() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        mask.set_bit(64);
        assert!(!bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_mask_longer_with_zeros() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_empty() {
        let mut bs = FilterBitset::new();
        bs.set_bit(5);
        let mask = FilterBitset::new();
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_u128_roundtrip() {
        let val: u128 = 0xDEAD_BEEF_CAFE_F00D;
        let bs = FilterBitset::from(val);
        let back: u128 = bs.into();
        assert_eq!(back, val);
        let bs2 = FilterBitset::from_u128(val);
        assert_eq!(bs2.to_u128(), val);
    }

    #[test]
    fn test_filter_bitset_u128_high_bits() {
        let high = 0xCAFE_F00D_0000_0000_0000_0000_0000_0000u128;
        let bs = FilterBitset::from_u128(high);
        assert_eq!(bs.to_u128(), high);
    }

    #[test]
    fn test_filter_bitset_u128_zero() {
        let bs = FilterBitset::from_u128(0);
        assert!(bs.is_empty());
        assert_eq!(bs.to_u128(), 0);
    }

    #[test]
    fn test_filter_bitset_bytes_roundtrip() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        bs.set_bit(63);
        bs.set_bit(64);
        bs.set_bit(128);
        let bytes = bs.to_bytes();
        let (restored, consumed) = FilterBitset::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(restored, bs);
    }

    #[test]
    fn test_filter_bitset_bytes_truncated() {
        assert!(FilterBitset::from_bytes(&[0x00]).is_err());
        assert!(FilterBitset::from_bytes(&[]).is_err());
    }

    #[test]
    fn test_filter_bitset_bytes_exceeds_max_words() {
        let mut data = Vec::new();
        data.extend_from_slice(&5000u32.to_le_bytes());
        assert!(FilterBitset::from_bytes(&data).is_err());
    }

    #[test]
    fn test_filter_bitset_default() {
        let bs: FilterBitset = Default::default();
        assert!(bs.is_empty());
    }

    #[test]
    fn test_filter_bitset_clone_eq() {
        let mut a = FilterBitset::new();
        a.set_bit(1);
        a.set_bit(2);
        let b = a.clone();
        assert_eq!(a, b);
        let mut c = FilterBitset::new();
        c.set_bit(1);
        assert_ne!(a, c);
    }
}
