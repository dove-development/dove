use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct VaultKey(Pubkey);

#[cfg(feature = "wasm")]
impl VaultKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl Deref for VaultKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
