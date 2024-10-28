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
        store::{Authority, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Claims vesting rewards
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` World account (PDA)
/// 2. `[writable]` DOVE mint account
/// 3. `[writable]` DOVE token account (to receive rewards)
/// 4. `[]` SPL Token program
/// 5. `[]` Authority account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VestingClaim {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VestingClaim {
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
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let dove_mint_key = DoveMintKey::new(b2pk(doveMintKey)?);
        let accounts = Self::get_accounts(program_key, (user_key, dove_mint_key))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VestingClaim {}

impl Command for VestingClaim {
    const ID: u32 = 0x2fac2f3c;
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
        let world_account = Writable::new(&accounts[1]);
        let dove_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let dove_token_account = TokenAccount::new(Writable::new(&accounts[3]));
        let token_program_account = TokenProgramAccount::new(&accounts[4]);
        let authority_account = Readonly::new(&accounts[5]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        world.vesting.claim_emission(
            user_account,
            &mut world.dove,
            dove_mint_account,
            dove_token_account,
            token_program_account,
            authority,
            &clock,
        );
    }
}
