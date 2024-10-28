use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct SovereignKey(Pubkey);

#[cfg(feature = "wasm")]
impl SovereignKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for SovereignKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
