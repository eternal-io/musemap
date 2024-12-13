#![doc = include_str!("../README.md")]

pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;
pub trait HashMapExt {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
}
impl<K, V> HashMapExt for HashMap<K, V> {
    fn new() -> Self {
        HashMap::with_hasher(RandomState::new())
    }
    fn with_capacity(capacity: usize) -> Self {
        HashMap::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

pub type HashSet<T> = std::collections::HashSet<T, RandomState>;
pub trait HashSetExt {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
}
impl<T> HashSetExt for HashSet<T> {
    fn new() -> Self {
        HashSet::with_hasher(RandomState::new())
    }
    fn with_capacity(capacity: usize) -> Self {
        HashSet::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

pub struct MuseHasher(pub(crate) u64);
impl core::hash::Hasher for MuseHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0 = museair::bfast::hash(bytes, self.0);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RandomState(u64);
impl RandomState {
    pub fn new() -> Self {
        use std::cell::Cell;
        thread_local!(static THREAD_UNIQUE_SEED: Cell<u64> = const { Cell::new(1123) });

        let mut unique = THREAD_UNIQUE_SEED.get();
        unique ^= &unique as *const _ as u64;
        unique = museair::bfast::hash(b"laqubikh", unique);
        THREAD_UNIQUE_SEED.set(unique);

        Self(unique)
    }
}
impl core::hash::BuildHasher for RandomState {
    type Hasher = MuseHasher;
    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        MuseHasher(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedState(u64);
impl FixedState {
    #[inline(always)]
    pub fn with_seed(seed: u64) -> Self {
        Self(seed)
    }
}
impl core::hash::BuildHasher for FixedState {
    type Hasher = MuseHasher;
    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        MuseHasher(self.0)
    }
}
