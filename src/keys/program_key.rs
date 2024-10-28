use {
    super::{CollateralMintKey, StableMintKey, UserKey},
    crate::{
        oracle::UserFeed,
        store::{Authority, Collateral, Savings, Stability, Vault, World},
        token::Safe,
        traits::Store,
    },
    solana_program::pubkey::Pubkey,
    std::ops::Deref,
};

pub struct ProgramKey(Pubkey);
impl ProgramKey {
    #[cfg(feature = "wasm")]
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }

    pub fn derive_authority(&self) -> Pubkey {
        Authority::derive_address(&self.0)
    }
    pub fn derive_collateral(&self, collateral_mint_key: &CollateralMintKey) -> Pubkey {
        Collateral::derive_address(&self.0, collateral_mint_key)
    }
    pub fn derive_savings(&self, user_key: &UserKey) -> Pubkey {
        Savings::derive_address(&self.0, user_key)
    }
    pub fn derive_safe(&self, mint: &Pubkey) -> Pubkey {
        Safe::derive_address(&self.0, &mint)
    }
    pub fn derive_stability(&self, stable_mint_key: &StableMintKey) -> Pubkey {
        Stability::derive_address(&self.0, stable_mint_key)
    }
    pub fn derive_world(&self) -> Pubkey {
        World::derive_address(&self.0, ())
    }
    pub fn derive_user_feed(&self, user_key: &UserKey, index: u8) -> Pubkey {
        UserFeed::derive_address(&self.0, (user_key, &[index]))
    }
    pub fn derive_vault(&self, user_key: &UserKey) -> Pubkey {
        Vault::derive_address(&self.0, user_key)
    }
}

impl Deref for ProgramKey {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
