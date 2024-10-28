use {
    crate::{
        accounts::{MintAccount, Signer, TokenAccount, TokenProgramAccount, Writable},
        keys::{DvdMintKey, UserKey},
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};

/// Executes the end of a flash mint operation
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` World account (PDA)
/// 2. `[writable]` Debt token mint account
/// 3. `[writable]` User's DVD account
/// 4. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct FlashMintEnd {
    _private: (),
}

#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl FlashMintEnd {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
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

unsafe impl Pod for FlashMintEnd {}

impl Command for FlashMintEnd {
    const ID: u32 = 0xc4169f48;
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
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[3]));
        let token_program_account = TokenProgramAccount::new(&accounts[4]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        world.flash_mint.end(
            user_account,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            world.config.get_flash_mint_config(),
            &mut world.dvd,
        );
    }
}
