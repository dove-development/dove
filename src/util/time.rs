use {
    crate::util::require,
    solana_program::clock::Clock,
};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Time {
    unix_timestamp: u64,
}

impl Time {
    pub const fn now(clock: &Clock) -> Self {
        require(clock.unix_timestamp >= 0, "Clock unix_timestamp is negative");
        Self {
            unix_timestamp: clock.unix_timestamp as u64,
        }
    }
    pub const fn secs_since(self, earlier: Self) -> u64 {
        self.unix_timestamp.saturating_sub(earlier.unix_timestamp)
    }
    pub const fn secs_elapsed(self, clock: &Clock) -> u64 {
        Self::now(clock).secs_since(self)
    }
    pub const fn from_unix_timestamp(unix_timestamp: u64) -> Self {
        Self { unix_timestamp }
    }
    #[cfg(feature = "wasm")]
    pub const fn to_unix_timestamp(&self) -> u64 {
        self.unix_timestamp
    }
}
