use crate::{
    accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable}, finance::Decimal, store::Authority, token::Mint, util::Expect
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// A token controlled by the Authority.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Token {
    mint: Mint,
    supply: Decimal,
    decimals: u8
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Token {
    #[wasm_bindgen(getter)]
    pub fn supply(&self) -> f64 {
        self.supply.to_f64()
    }
    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
    #[wasm_bindgen(getter, js_name = "mint")]
    pub fn mint_wasm(&self) -> Vec<u8> {
        self.mint.get_key().to_bytes().to_vec()
    }
}

impl Token {
    pub fn from_account(
        account: MintAccount<Readonly>,
        authority: Authority,
        expected_freeze_authority: Expect<Readonly>,
        expected_supply: Expect<u64>,
    ) -> Self {
        let mut decimals = 0;
        let mut supply = 0;
        let mint = Mint::from_account(
            account,
            Expect::Some(authority.get_account()),
            expected_freeze_authority,
            expected_supply,
            &mut decimals,
            &mut supply,
        );
        Self {
            decimals,
            mint,
            supply: Decimal::from_token_amount(supply, decimals),
        }
    }

    pub const fn get_supply(&self) -> Decimal {
        self.supply
    }

    pub fn mint(
        &mut self,
        amount: Decimal,
        mint_account: MintAccount<Writable>,
        token_account: TokenAccount<Writable>,
        authority: Authority,
        token_program_account: TokenProgramAccount,
    ) {
        self.mint.mint(
            mint_account,
            token_account,
            authority.get_account(),
            token_program_account,
            amount.to_token_amount(self.decimals),
            &authority.get_seeds(),
        );
        self.supply += amount;
    }

    pub fn burn(
        &mut self,
        amount: Decimal,
        mint_account: MintAccount<Writable>,
        token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        user_account: Signer,
    ) {
        self.mint.burn(
            mint_account,
            token_account,
            token_program_account,
            user_account,
            amount.to_token_amount(self.decimals),
        );
        self.supply -= amount;
    }
}
