#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
use {
    crate::{
        accounts::{Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        store::{Authority, Collateral},
        token::Mint,
        util::require,
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Reserve {
    mint: Mint,
    balance: Decimal,
}

impl Reserve {
    pub const fn new(collateral: &Collateral) -> Self {
        let mint = collateral.get_mint();
        Reserve {
            mint: *mint,
            balance: Decimal::zero(),
        }
    }
    pub fn deposit(
        &mut self,
        amount: Decimal,

        program_id: &Pubkey,
        collateral: &mut Collateral,

        user_account: Signer,
        source_token_account: TokenAccount<Writable>,
        program_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        require(collateral.get_mint() == &self.mint, "mint mismatch");
        collateral.receive(
            amount,
            program_id,
            user_account,
            source_token_account,
            program_token_account,
            token_program_account,
        );
        self.balance += amount;

    }
    pub fn withdraw(
        &mut self,
        amount: Decimal,

        program_id: &Pubkey,
        collateral: &mut Collateral,

        safe_account: TokenAccount<Writable>,
        destination_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        authority: Authority,
    ) {
        require(collateral.get_mint() == &self.mint, "mint mismatch");
        require(amount <= self.balance, "amount too large");
        collateral.send(
            amount,
            program_id,
            safe_account,
            destination_token_account,
            token_program_account,
            authority,
        );
        self.balance -= amount;
    }
    pub fn get_value(
        &self,
        collateral: &Collateral,
        oracle_account: Readonly,
        clock: &Clock,
    ) -> Decimal {
        require(collateral.get_mint() == &self.mint, "mint mismatch");
        collateral.get_price(oracle_account, clock) * self.balance
    }
    pub const fn get_mint(&self) -> &Mint {
        &self.mint
    }
    pub const fn get_balance(&self) -> Decimal {
        self.balance
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Reserve {
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> f64 {
        self.balance.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "mintKey")]
    pub fn mint_key(&self) -> Vec<u8> {
        self.mint.get_key().to_bytes().to_vec()
    }
}