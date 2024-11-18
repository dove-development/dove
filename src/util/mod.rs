mod list;
mod time;

pub use {list::List, time::Time};

#[inline(always)]
pub const fn require(that: bool, orelse: &'static str) {
    if !that {
        panic!("{}", orelse);
    }
}

#[inline(always)]
pub const fn revert(reason: &'static str) -> ! {
    panic!("{}", reason);
}

pub enum Expect<T> {
    Any,
    None,
    Some(T),
}

#[cfg(feature = "wasm")]
mod account_wasm;

#[cfg(feature = "wasm")]
mod wasm {
    pub use super::account_wasm::AccountWasm;
    use solana_program::pubkey::Pubkey;

    pub fn b2pk(b: &[u8]) -> Result<Pubkey, String> {
        b.try_into()
            .map(Pubkey::new_from_array)
            .map_err(|e| format!("Invalid pubkey: {}", e))
    }
}

#[cfg(feature = "wasm")]
pub use wasm::*;

/// Number of seconds in a day
/// ```math
/// 60 * 60 * 24 = 86400
/// ```
pub const SECS_PER_DAY: u64 = 60 * 60 * 24;

#[cfg(feature = "wasm")]
/// Number of seconds in a year
/// ```math
/// 60 * 60 * 24 * 365 = 31_536_000
/// ```
pub const SECS_PER_YEAR: u64 = 60 * 60 * 24 * 365;
