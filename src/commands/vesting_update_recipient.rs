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
        keys::UserKey,
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Updates the recipient of a vesting account
///
/// Accounts expected:
///
/// 0. `[signer]` Current recipient account
/// 1. `[writable]` World account (PDA)
/// 2. `[]` New recipient account
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VestingUpdateRecipient {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VestingUpdateRecipient {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }
    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        currentRecipientKey: &[u8],
        newRecipientKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let current_recipient_key = UserKey::new(b2pk(currentRecipientKey)?);
        let new_recipient_key = UserKey::new(b2pk(newRecipientKey)?);
        let accounts = Self::get_accounts(program_key, (current_recipient_key, new_recipient_key))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VestingUpdateRecipient {}

impl Command for VestingUpdateRecipient {
    const ID: u32 = 0xdf117316;
    type Keys = (UserKey, UserKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (current_recipient_key, new_recipient_key) = keys;
        vec![
            AccountMeta {
                pubkey: *current_recipient_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *new_recipient_key,
                is_signer: true,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);
        let new_recipient_account = Readonly::new(&accounts[2]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        world
            .vesting
            .update_recipient(&user_account, new_recipient_account);
    }
}
