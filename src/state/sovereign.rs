use {
    crate::{accounts::{Readonly, Signer}, traits::Account, util::require},
    solana_program::pubkey::Pubkey,
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
pub struct SovereignAuth {
    _v: (),
}

impl SovereignAuth {
    const fn new() -> Self {
        Self { _v: () }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Sovereign {
    key: Pubkey,
}

impl Sovereign {
    pub fn new<T: Account>(account: T) -> Self {
        Self {
            key: *account.get_info().key,
        }
    }

    pub fn authorize(&self, sovereign_account: Signer) -> SovereignAuth {
        require(
            &self.key == sovereign_account.get_info().key,
            "Sovereign key does not match account key",
        );
        SovereignAuth::new()
    }

    pub fn update(&mut self, _: SovereignAuth, new_sovereign: Readonly) {
        self.key = *new_sovereign.get_info().key;
    }
}
