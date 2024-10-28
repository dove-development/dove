use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct CollateralMintKey(Pubkey);

#[cfg(feature = "wasm")]
impl CollateralMintKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for CollateralMintKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
