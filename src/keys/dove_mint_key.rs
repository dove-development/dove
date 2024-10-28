use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct DoveMintKey(Pubkey);

#[cfg(feature = "wasm")]
impl DoveMintKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for DoveMintKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
