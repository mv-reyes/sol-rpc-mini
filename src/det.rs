//! Deterministic fixtures for reproducible tests.
//!
//! `rand::rngs::StdRng` makes no stability promise across releases, so a sweep
//! seeded with it can silently change shape after a toolchain bump. The stream
//! below is xorshift64* with a fixed spec: same seed, same sequence, on every
//! platform and every version of this crate.

use crate::to_base58;

/// A deterministic `u64` stream (xorshift64*, period 2^64 - 1).
pub struct Rng(u64);

impl Rng {
    /// Seed the stream. Zero is remapped to a fixed nonzero state.
    pub fn seeded(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    /// Next word in the stream.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform value in `0..n` (rejection sampling, so no modulo bias).
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "below() needs a nonzero bound");
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let v = self.next_u64();
            if v < zone {
                return v % n;
            }
        }
    }

    /// Coin flip.
    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    /// Fill a 32-byte fixture (pubkey-sized).
    pub fn bytes32(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for chunk in out.chunks_mut(8) {
            chunk.copy_from_slice(&self.next_u64().to_le_bytes());
        }
        out
    }
}

/// Hash a label into an `Rng` seed (FNV-1a, 64-bit).
pub fn label_seed(label: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in label.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A deterministic 32-byte fixture derived from a label.
pub fn fixture_bytes(label: &str) -> [u8; 32] {
    Rng::seeded(label_seed(label)).bytes32()
}

/// A deterministic base58 address-like string, for fixtures that need stable
/// pubkeys without pulling in `solana-sdk`.
pub fn fixture_address(label: &str) -> String {
    to_base58(&fixture_bytes(label))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_is_stable() {
        // Pinned vector: if the stream ever changes, sweeps seeded with it
        // silently change shape, and this test is the tripwire.
        let mut r = Rng::seeded(42);
        let first: Vec<u64> = (0..4).map(|_| r.next_u64()).collect();
        assert_eq!(
            first,
            [
                6255019084209693600,
                14430073426741505498,
                14575455857230217846,
                17414512882241728735,
            ]
        );
    }

    #[test]
    fn seeded_zero_is_remapped() {
        assert_ne!(Rng::seeded(0).next_u64(), Rng::seeded(1).next_u64());
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::seeded(7);
        for _ in 0..1_000 {
            assert!(r.below(13) < 13);
        }
    }

    #[test]
    fn labels_are_deterministic() {
        assert_eq!(fixture_bytes("mint/index"), fixture_bytes("mint/index"));
        assert_ne!(fixture_bytes("mint/index"), fixture_bytes("mint/long"));
        let addr = fixture_address("mint/index");
        assert_eq!(addr.len(), 44);
        assert_eq!(addr, fixture_address("mint/index"));
    }
}
