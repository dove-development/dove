use {solana_program::pubkey::Pubkey, std::ops::Deref};

pub struct UserKey(Pubkey);

#[cfg(feature = "wasm")]
impl UserKey {
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
    pub fn derive_associated_token_address(&self, address: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(&self.0, address)
    }
}

impl Deref for UserKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
