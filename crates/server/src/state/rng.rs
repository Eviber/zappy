use core::mem::MaybeUninit;

/// A random number generator.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a new `Rng` instance with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Creates a new `Rng` instance by reading the `/dev/urandom` file.
    pub fn from_urandom() -> Option<Self> {
        let fd = ft::File::open(ft::charstar!("/dev/urandom")).ok()?;
        let mut seed: MaybeUninit<u64> = MaybeUninit::uninit();
        fd.read(&mut seed.as_bytes_mut()).ok()?;
        Some(Self::new(unsafe { seed.assume_init() }))
    }

    /// Generates a random 64-bit unsigned integer.
    pub fn next_u64(&mut self) -> u64 {
        pub const CONST0: u64 = 0x2d35_8dcc_aa6c_78a5;
        pub const CONST1: u64 = 0x8bb8_4b93_962e_acc9;

        let s = self.state.wrapping_add(CONST0);
        self.state = s;
        let t = s as u128 * (s ^ CONST1) as u128;
        (t as u64) ^ (t >> 64) as u64
    }
}
