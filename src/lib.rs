#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;

pub type HashSet<T> = std::collections::HashSet<T, RandomState>;

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

//------------------------------------------------------------------------------

#[derive(Clone)]
pub struct FixedState {
    per_hasher_seed_a: u64,
    per_hasher_seed_b: u64,
}

impl FixedState {
    #[inline(always)]
    pub fn new(seed_a: u64, seed_b: u64) -> Self {
        Self {
            per_hasher_seed_a: seed_a,
            per_hasher_seed_b: seed_b,
        }
    }
}

impl core::hash::BuildHasher for FixedState {
    type Hasher = MuseHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        MuseHasher::new(self.per_hasher_seed_a, self.per_hasher_seed_b)
    }
}

//------------------------------------------------------------------------------

#[derive(Clone)]
pub struct RandomState {
    per_hasher_seed_a: u64,
    per_hasher_seed_b: u64,
}

impl RandomState {
    pub fn new() -> Self {
        use std::cell::Cell;

        thread_local! {
            static THREAD_UNIQUE: Cell<(u64, u64)> = const { Cell::new((0, 1123)) };
        }

        let mut seed_a = CONSTANT[0];
        let mut seed_b = CONSTANT[1] ^ core::ptr::addr_of!(seed_a) as u64;

        THREAD_UNIQUE.with(|cell| {
            let (a, b) = cell.get();

            let (lo0, hi0) = wmul(a ^ seed_a, b ^ seed_b);
            let (lo1, hi1) = wmul(a ^ CONSTANT[2], seed_b ^ CONSTANT[3]);
            let (lo2, hi2) = wmul(seed_a ^ CONSTANT[4], b ^ CONSTANT[5]);

            let (a, b) = (lo0 ^ hi1 ^ lo2, hi0 ^ lo1 ^ hi2);

            cell.set((a, b));

            seed_a = seed_a.wrapping_sub(a);
            seed_b = seed_b.wrapping_sub(b);
        });

        Self {
            per_hasher_seed_a: seed_a,
            per_hasher_seed_b: seed_b,
        }
    }
}

impl core::hash::BuildHasher for RandomState {
    type Hasher = MuseHasher;
    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        MuseHasher::new(self.per_hasher_seed_a, self.per_hasher_seed_b)
    }
}

//------------------------------------------------------------------------------

#[derive(Clone)]
pub struct MuseHasher {
    seed_a: u64,
    seed_b: u64,
    acc_lo: u64,
    acc_hi: u64,
    sponge: u128,
    sponge_len: usize,
}

impl core::hash::Hasher for MuseHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.finish()
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        match bytes.len() {
            0..=32 => self.write_short(bytes),
            33..=256 => self.write_medim(bytes),
            257.. => self.write_loong(bytes),
        };
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.write_num(i)
    }
    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.write_num(i)
    }
    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.write_num(i)
    }
    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.write_num(i)
    }
    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.write_num(i)
    }
    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        #[cfg(target_pointer_width = "32")]
        self.write_num(i as u32);
        #[cfg(target_pointer_width = "64")]
        self.write_num(i as u64);
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }
    #[inline(always)]
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }
    #[inline(always)]
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }
    #[inline(always)]
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }
    #[inline(always)]
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }
    #[inline(always)]
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }
}

//------------------------------------------------------------------------------

/// `Gamma(2/3)` fractional part calculated by Y-Cruncher.
const CONSTANT: [u64; 7] = [
    0x5aa77928c3678cab,
    0x2f4feb702b26990a,
    0x54f7edbc621298be,
    0xb6e4e1eb259b0c87,
    0xa38abf7cde765fa6,
    0x283d1db180df5862,
    0xff0d89fac6d1825e,
];

/// Lower 64-bit, then upper 64-bit.
#[inline(always)]
const fn wmul(a: u64, b: u64) -> (u64, u64) {
    u128_to_u64s(a as u128 * b as u128)
}

/// Lower 64-bit, then upper 64-bit.
#[inline(always)]
const fn fmul(a: u64, b: u64) -> u64 {
    let (lo, hi) = wmul(a, b);
    lo ^ hi
}

/// Lower 64-bit, then upper 64-bit.
#[inline(always)]
const fn u128_to_u64s(x: u128) -> (u64, u64) {
    (x as u64, (x >> 64) as u64)
}

macro_rules! u64 {
    ($n:literal) => {
        $n * 8
    };
}

macro_rules! min {
    ( $left:expr, $right:expr $(,)? ) => {
        match ($left, $right) {
            (left_val, right_val) => {
                if left_val < right_val {
                    left_val
                } else {
                    right_val
                }
            }
        }
    };
}

//------------------------------------------------------------------------------

#[inline(always)]
const fn read_u32(bytes: &[u8], offset: usize) -> u64 {
    u32::from_ne_bytes(*match bytes.split_at(offset).1.first_chunk() {
        Some(bytes) => bytes,
        None => unreachable!(),
    }) as u64
}
#[inline(always)]
const fn read_u32_r(bytes: &[u8], offset_r: usize) -> u64 {
    u32::from_ne_bytes(*match bytes.split_at(bytes.len() - offset_r - 4).1.first_chunk() {
        Some(bytes) => bytes,
        None => unreachable!(),
    }) as u64
}

#[inline(always)]
const fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_ne_bytes(*match bytes.split_at(offset).1.first_chunk() {
        Some(bytes) => bytes,
        None => unreachable!(),
    })
}
#[inline(always)]
const fn read_u64_r(bytes: &[u8], offset_r: usize) -> u64 {
    u64::from_ne_bytes(*match bytes.split_at(bytes.len() - offset_r - 8).1.first_chunk() {
        Some(bytes) => bytes,
        None => unreachable!(),
    })
}

#[inline(always)]
const fn read_short(bytes: &[u8]) -> (u64, u64) {
    debug_assert!(bytes.len() <= u64!(2));

    let len = bytes.len();
    if len >= 4 {
        let off = (len & 24) >> (len >> 3); // len >= 8 ? 4 : 0
        let head = read_u32(bytes, 0);
        let head_off = read_u32(bytes, off);
        let tail = read_u32_r(bytes, 0);
        let tail_off = read_u32_r(bytes, off);

        (head << 32 | tail, head_off << 32 | tail_off)
    } else if len > 0 {
        // MSB <-> LSB
        // [0] [0] [0] @ len == 1 (0b01)
        // [0] [1] [1] @ len == 2 (0b10)
        // [0] [1] [2] @ len == 3 (0b11)
        let fst = bytes[0] as u64;
        let snd = bytes[len >> 1] as u64;
        let thd = bytes[len - 1] as u64;

        (fst << 48 | snd << 24 | thd, 0)
    } else {
        (0, 0)
    }
}

//------------------------------------------------------------------------------

impl MuseHasher {
    #[inline(always)]
    pub const fn new(seed_a: u64, seed_b: u64) -> Self {
        Self {
            seed_a,
            seed_b,
            acc_lo: seed_a, // please note the seed order.
            acc_hi: seed_b,
            sponge: 0,
            sponge_len: 0,
        }
    }

    #[inline(always)]
    pub const fn finish(&self) -> u64 {
        let (mut i, mut j) = (self.acc_lo, self.acc_hi);
        let (mut u, mut v) = (0, 0);

        if self.sponge_len != 0 {
            (u, v) = u128_to_u64s(self.sponge);
            (u, v) = wmul(u ^ j, v ^ i);
            (u, v) = wmul(u ^ CONSTANT[0], v ^ CONSTANT[1]);
        }

        (i, j) = wmul(i ^ CONSTANT[2], j ^ CONSTANT[3]);
        (i, j) = wmul(i ^ CONSTANT[4], j ^ CONSTANT[5]);

        i ^ j ^ u ^ v
    }

    #[inline(always)]
    fn write_num<T: Into<u128>>(&mut self, x: T) {
        let bits = core::mem::size_of::<T>() * 8;

        if self.sponge_len + bits > 128 {
            let (lo, hi) = u128_to_u64s(self.sponge);
            let (lolo, lohi) = wmul(lo ^ self.acc_lo, hi ^ self.seed_b);
            let (hilo, hihi) = wmul(hi ^ self.acc_hi, lo ^ self.seed_a);

            self.acc_lo = lolo ^ hihi;
            self.acc_hi = hilo ^ lohi;

            self.sponge ^= u128::MAX;
            self.sponge_len = 0;
        }

        self.sponge ^= x.into() << self.sponge_len;
        self.sponge_len += bits;
    }

    #[inline(always)]
    fn write_short(&mut self, bytes: &[u8]) {
        debug_assert!(bytes.len() <= u64!(4));

        let len = bytes.len();
        let len_64 = bytes.len() as u64;
        let (lo2, hi2) = wmul(CONSTANT[0] ^ self.seed_b, CONSTANT[1] ^ len_64);

        (self.acc_lo, self.acc_hi) = read_short(bytes.split_at(min!(u64!(2), len)).0);
        self.acc_lo ^= lo2 ^ len_64;
        self.acc_hi ^= hi2 ^ self.seed_a;

        if len > u64!(2) {
            let (u, v) = read_short(bytes.split_at(u64!(2)).1);
            let (lo0, hi0) = wmul(CONSTANT[2], CONSTANT[3] ^ u);
            let (lo1, hi1) = wmul(CONSTANT[4], CONSTANT[5] ^ v);
            self.acc_lo ^= lo0 ^ hi1;
            self.acc_hi ^= lo1 ^ hi0;
        }
    }

    fn write_medim(&mut self, bytes: &[u8]) {
        debug_assert!(bytes.len() >= u64!(2));

        let ltr = bytes.chunks_exact(u64!(2));
        let rtl = bytes.rchunks_exact(u64!(2));
        let times = bytes.len().div_ceil(u64!(4));
        let (mut i, mut j) = (self.acc_lo, self.acc_hi);

        for (left, right) in ltr.zip(rtl).take(times) {
            i = fmul(i ^ read_u64(left, 0), self.seed_b ^ read_u64_r(right, 0));
            j = fmul(j ^ read_u64(left, 8), self.seed_a ^ read_u64_r(right, 8));
        }

        let rot = bytes.len() as u32 & 63;
        self.acc_lo = self.acc_lo.rotate_left(rot);
        self.acc_hi = self.acc_hi.rotate_right(rot);

        self.acc_lo ^= i;
        self.acc_hi ^= j;
    }

    #[cold]
    #[inline(never)]
    fn write_loong(&mut self, bytes: &[u8]) {
        debug_assert!(bytes.len() >= u64!(12));

        let mut remainder = bytes;
        let mut ring_prev = CONSTANT[6];
        let mut state = [
            CONSTANT[0].wrapping_add(self.seed_a),
            CONSTANT[1].wrapping_sub(self.seed_b),
            CONSTANT[2] ^ self.seed_a,
            CONSTANT[3].wrapping_add(self.seed_b),
            CONSTANT[4].wrapping_sub(self.seed_a),
            CONSTANT[5] ^ self.seed_b,
        ];

        while let Some((chunk, rest)) = remainder.split_first_chunk::<{ u64!(12) }>() {
            remainder = rest;

            state[0] ^= read_u64(chunk, u64!(0));
            state[1] ^= read_u64(chunk, u64!(1));
            let (lo0, hi0) = wmul(state[0], state[1]);
            state[0] = ring_prev ^ hi0;

            state[1] ^= read_u64(chunk, u64!(2));
            state[2] ^= read_u64(chunk, u64!(3));
            let (lo1, hi1) = wmul(state[1], state[2]);
            state[1] = lo0 ^ hi1;

            state[2] ^= read_u64(chunk, u64!(4));
            state[3] ^= read_u64(chunk, u64!(5));
            let (lo2, hi2) = wmul(state[2], state[3]);
            state[2] = lo1 ^ hi2;

            state[3] ^= read_u64(chunk, u64!(6));
            state[4] ^= read_u64(chunk, u64!(7));
            let (lo3, hi3) = wmul(state[3], state[4]);
            state[3] = lo2 ^ hi3;

            state[4] ^= read_u64(chunk, u64!(8));
            state[5] ^= read_u64(chunk, u64!(9));
            let (lo4, hi4) = wmul(state[4], state[5]);
            state[4] = lo3 ^ hi4;

            state[5] ^= read_u64(chunk, u64!(10));
            state[0] ^= read_u64(chunk, u64!(11));
            let (lo5, hi5) = wmul(state[5], state[0]);
            state[5] = lo4 ^ hi5;

            ring_prev = lo5;
        }

        state[0] ^= ring_prev;

        self.acc_lo ^= state[0].wrapping_add(state[2]).wrapping_add(state[4]);
        self.acc_hi ^= state[1].wrapping_add(state[3]).wrapping_add(state[5]);

        self.write_medim(remainder);
    }
}
