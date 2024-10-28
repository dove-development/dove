use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct StableMintKey(Pubkey);

#[cfg(feature = "wasm")]
impl StableMintKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for StableMintKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
