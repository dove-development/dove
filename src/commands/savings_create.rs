#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{Signer, SystemProgramAccount, Writable},
        keys::UserKey,
        store::Savings,
        traits::{Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey, rent::Rent, sysvar::Sysvar},
};

/// Creates a new savings account
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` Savings account (PDA, will be created)
/// 2. `[]` System program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct SavingsCreate {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SavingsCreate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let accounts = Self::get_accounts(program_key, user_key)
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for SavingsCreate {}

impl Command for SavingsCreate {
    const ID: u32 = 0xce206862;
    type Keys = UserKey;

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, user_key: Self::Keys) -> Vec<AccountMeta> {
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
                pubkey: solana_program::system_program::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let savings_account = Writable::new(&accounts[1]);
        let system_program_account = SystemProgramAccount::new(&accounts[2]);

        Savings::create(
            program_id,
            user_account,
            savings_account,
            system_program_account,
            user_account,
            &Rent::get().map_err(|_| "Failed to get rent").unwrap(),
            (),
        )
    }
}
