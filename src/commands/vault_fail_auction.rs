#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Readonly, TokenAccount, TokenProgramAccount, Writable},
        keys::{DvdMintKey, UserKey, VaultKey},
        store::{Authority, Vault, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Fails the auction for a liquidated vault
///
/// Accounts expected:
/// 0. `[writable]` Vault account (PDA) for which to fail the auction
/// 1. `[writable]` World account (PDA)
/// 2. `[writable]` Debt token mint account
/// 3. `[writable]` Debt token account (to receive auction failure reward)
/// 4. `[]` Authority account (PDA)
/// 5. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultFailAuction {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultFailAuction {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        vaultKey: &[u8],
        dvdMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let vault_key = VaultKey::new(b2pk(vaultKey)?);
        let dvd_mint_key = DvdMintKey::new(b2pk(dvdMintKey)?);
        let accounts = Self::get_accounts(program_key, (user_key, vault_key, dvd_mint_key))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultFailAuction {}

impl Command for VaultFailAuction {
    const ID: u32 = 0x9a634fdd;
    type Keys = (UserKey, VaultKey, DvdMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, vault_key, dvd_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *vault_key,
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
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let vault_account = Writable::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[3]));
        let authority_account = Readonly::new(&accounts[4]);
        let token_program_account = TokenProgramAccount::new(&accounts[5]);

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let vault = Vault::load_mut(program_id, vault_account, &mut vault_data[..], ());

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.fail_auction(
            &mut world.debt,
            &world.config.get_debt_config(),
            &world.config.get_vault_config(),
            &world.config.get_auction_config(),
            &mut world.dvd,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            authority,
            &clock,
        );
    }
}
