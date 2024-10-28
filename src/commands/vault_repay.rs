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
        store::{Vault, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Repays borrowed DVD to the vault
///
/// Accounts expected:
/// 0. `[signer]` User account
/// 1. `[writable]` Debt token account (must be owned by user)
/// 2. `[writable]` Mint account (for the DVD)
/// 3. `[writable]` World account (PDA)
/// 4. `[writable]` Vault account (PDA)
/// 5. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultRepay {
    requested_amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultRepay {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(requestedAmount: f64) -> Vec<u8> {
        Self {
            requested_amount: Decimal::from(requestedAmount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        dvdMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, dvdMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultRepay {}

impl Command for VaultRepay {
    const ID: u32 = 0x1af52fc3;
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
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_vault(&user_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let dvd_account = TokenAccount::new(Writable::new(&accounts[1]));
        let mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let world_account = Writable::new(&accounts[3]);
        let vault_account = Writable::new(&accounts[4]);
        let token_program_account = TokenProgramAccount::new(&accounts[5]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let (vault, vault_auth) =
            Vault::load_auth(program_id, vault_account, &mut vault_data[..], user_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.repay(
            vault_auth,
            self.requested_amount,
            &mut world.debt,
            &world.config.get_debt_config(),
            &mut world.dvd,
            user_account,
            mint_account,
            dvd_account,
            token_program_account,
            &clock,
        );
    }
}
