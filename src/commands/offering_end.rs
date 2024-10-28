#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::Writable,
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Ends the current world offering
///
/// Accounts expected:
///
/// 0. `[writable]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OfferingEnd {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OfferingEnd {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    pub fn get_accounts_wasm(program_key: &[u8]) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(program_key)?);
        let accounts = Self::get_accounts(program_key, ())
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for OfferingEnd {}

impl Command for OfferingEnd {
    const ID: u32 = 0x1cadf543;
    type Keys = ();

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, _: Self::Keys) -> Vec<AccountMeta> {
        vec![AccountMeta {
            pubkey: program_key.derive_world(),
            is_signer: false,
            is_writable: true,
        }]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let world_account = Writable::new(&accounts[0]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        world
            .offering
            .end(&clock, world.config.get_auction_config());
    }
}
