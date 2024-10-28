use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
};

use crate::{
    accounts::{
        InstructionSysvarAccount, MintAccount, Signer, TokenAccount, TokenProgramAccount, Writable,
    },
    finance::Decimal,
    store::Authority,
    token::Token,
    traits::Account,
    util::{require, revert},
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Configuration for the flash mint system.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct FlashMintConfig {
    /// The fee for a flash mint. For example: 0.0005 (0.05%)
    fee: Decimal,
    /// The maximum amount of DVD that can be minted via a flash mint.
    limit: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl FlashMintConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(fee: f64, limit: f64) -> Self {
        Self {
            fee: Decimal::from(fee),
            limit: Decimal::from(limit),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn fee(&self) -> f64 {
        self.fee.to_f64()
    }

    #[wasm_bindgen(getter)]
    pub fn limit(&self) -> f64 {
        self.limit.to_f64()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct FlashMint {
    borrow_amount: Option<Decimal>,
}

impl FlashMint {
    pub const fn new() -> Self {
        Self {
            borrow_amount: None,
        }
    }
    pub fn begin(
        &mut self,
        borrow_amount: Decimal,
        flash_mint_end_instruction_data: &[u8],

        program_id: &Pubkey,
        authority: Authority,

        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        instruction_sysvar_account: InstructionSysvarAccount,
        flash_mint_config: &FlashMintConfig,

        dvd: &mut Token,
    ) {
        require(
            self.borrow_amount.is_none(),
            "Already have active flash mint",
        );
        require(!borrow_amount.is_zero(), "Borrow amount must be positive");
        self.borrow_amount = Some(borrow_amount);

        require(
            borrow_amount <= flash_mint_config.limit,
            "Flash mint amount exceeds the limit",
        );

        let instruction_sysvar_account_info = instruction_sysvar_account.get_info();

        let current_index = load_current_index_checked(instruction_sysvar_account_info)
            .map_err(|_| "Invalid instruction sysvar account")
            .unwrap() as usize;
        let current_ix =
            load_instruction_at_checked(current_index, instruction_sysvar_account_info)
                .map_err(|_| "can't load current instruction")
                .unwrap();
        require(
            &current_ix.program_id == program_id,
            "Flash mint must be a top-level instruction",
        );

        let mut found_repay = false;
        for i in current_index + 1.. {
            let ix = match load_instruction_at_checked(i, instruction_sysvar_account_info) {
                Ok(ix) => ix,
                Err(ProgramError::InvalidArgument) => break,
                Err(_) => revert("could not load instruction"),
            };
            if &ix.program_id != program_id {
                continue;
            }
            if ix.data == flash_mint_end_instruction_data {
                found_repay = true;
                break;
            }
        }
        require(found_repay, "can't find flash mint end instruction");

        dvd.mint(
            borrow_amount,
            dvd_mint_account,
            dvd_account,
            authority,
            token_program_account,
        );
    }

    pub fn end(
        &mut self,
        user_account: Signer,
        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        flash_mint_config: &FlashMintConfig,
        dvd: &mut Token,
    ) {
        let borrow_amount = match self.borrow_amount.take() {
            Some(v) => v,
            None => revert("active flash mint not found"),
        };
        let repay_amount = borrow_amount * (Decimal::one() + flash_mint_config.fee);
        dvd.burn(
            repay_amount,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            user_account,
        );
    }
}
