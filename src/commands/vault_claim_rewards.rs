#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        keys::{DoveMintKey, UserKey},
        store::{Authority, Vault, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Claims rewards from the vault account
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` Vault account (PDA)
/// 2. `[writable]` World account (PDA)
/// 3. `[writable]` DOVE mint account
/// 4. `[writable]` DOVE token account (to receive rewards)
/// 5. `[]` SPL Token program
/// 6. `[]` Authority account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultClaimRewards {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultClaimRewards {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        doveMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let doveMintKey = DoveMintKey::new(b2pk(doveMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, doveMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultClaimRewards {}

impl Command for VaultClaimRewards {
    const ID: u32 = 0x3134bf60;
    type Keys = (UserKey, DoveMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dove_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_vault(&user_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dove_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dove_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: spl_token::ID,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let vault_account = Writable::new(&accounts[1]);
        let world_account = Writable::new(&accounts[2]);
        let dove_mint_account = MintAccount::new(Writable::new(&accounts[3]));
        let dove_token_account = TokenAccount::new(Writable::new(&accounts[4]));
        let token_program_account = TokenProgramAccount::new(&accounts[5]);
        let authority_account = Readonly::new(&accounts[6]);

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let (vault, vault_auth) =
            Vault::load_auth(program_id, vault_account, &mut vault_data[..], user_account);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.claim_rewards(
            vault_auth,
            &mut world.dove,
            dove_mint_account,
            dove_token_account,
            authority,
            token_program_account,
            &mut world.debt,
            &world.config.get_debt_config(),
            &clock,
        );
    }
}
