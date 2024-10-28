#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{DvdMintKey, UserKey},
        store::{Savings, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Deposits tokens into the savings account
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` Savings account (PDA)
/// 2. `[writable]` World account (PDA)
/// 3. `[writable]` Debt token mint account
/// 4. `[writable]` Debt token account (to transfer tokens from)
/// 5. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct SavingsDeposit {
    amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SavingsDeposit {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm(amount: f64) -> Vec<u8> {
        Self {
            amount: Decimal::from(amount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        doveMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let dvd_mint_key = DvdMintKey::new(b2pk(doveMintKey)?);
        let accounts = Self::get_accounts(program_key, (user_key, dvd_mint_key))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for SavingsDeposit {}

impl Command for SavingsDeposit {
    const ID: u32 = 0x14d7657c;
    type Keys = (UserKey, DvdMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dvd_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_savings(&user_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: spl_token::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let savings_account = Writable::new(&accounts[1]);
        let world_account = Writable::new(&accounts[2]);
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[3]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[4]));
        let token_program_account = TokenProgramAccount::new(&accounts[5]);

        let mut savings_data = savings_account.get_info().data.borrow_mut();
        let (savings, savings_auth) = Savings::load_auth(
            program_id,
            savings_account,
            &mut savings_data[..],
            user_account,
        );

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        savings.deposit(
            savings_auth,
            self.amount,
            &mut world.dvd,
            &mut world.savings,
            &world.config.get_savings_config(),
            user_account,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            &clock,
        );
    }
}
