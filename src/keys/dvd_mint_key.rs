use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct DvdMintKey(Pubkey);

#[cfg(feature = "wasm")]
impl DvdMintKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for DvdMintKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
