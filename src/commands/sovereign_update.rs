#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{Readonly, Signer, Writable},
        keys::SovereignKey,
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Updates the sovereign of the world
///
/// Accounts expected:
///
/// 0. `[signer]` Current sovereign account
/// 1. `[]` New sovereign account
/// 2. `[writable]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct SovereignUpdate {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SovereignUpdate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        currentSovereignKey: &[u8],
        newSovereignKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let current_sovereign_key = SovereignKey::new(b2pk(currentSovereignKey)?);
        let new_sovereign_key = SovereignKey::new(b2pk(newSovereignKey)?);
        let accounts = Self::get_accounts(program_key, (current_sovereign_key, new_sovereign_key))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for SovereignUpdate {}

impl Command for SovereignUpdate {
    const ID: u32 = 0x9d1cef50;
    type Keys = (SovereignKey, SovereignKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (current_sovereign_key, new_sovereign_key) = keys;
        vec![
            AccountMeta {
                pubkey: *current_sovereign_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: *new_sovereign_key,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let current_sovereign_account = Signer::new(&accounts[0]);
        let new_sovereign_account = Readonly::new(&accounts[1]);
        let world_account = Writable::new(&accounts[2]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let sovereign_auth = world.sovereign.authorize(current_sovereign_account);
        world
            .sovereign
            .update(sovereign_auth, new_sovereign_account);
    }
}
