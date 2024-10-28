use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct OracleKey(Pubkey);

#[cfg(feature = "wasm")]
impl OracleKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for OracleKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
