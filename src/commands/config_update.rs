#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{Signer, Writable},
        keys::SovereignKey,
        state::Config,
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Sets a new config for the world
///
/// Accounts expected:
///
/// 0. `[signer]` Sovereign account
/// 1. `[writable]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ConfigUpdate {
    new_config: Config,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ConfigUpdate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm(new_config: Config) -> Vec<u8> {
        Self { new_config }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        sovereignKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let sovereignKey = SovereignKey::new(b2pk(sovereignKey)?);
        let accounts = Self::get_accounts(programKey, sovereignKey)
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for ConfigUpdate {}

impl Command for ConfigUpdate {
    const ID: u32 = 0x32e65ae0;
    type Keys = SovereignKey;

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, sovereign_key: Self::Keys) -> Vec<AccountMeta> {
        vec![
            AccountMeta {
                pubkey: *sovereign_key,
                is_signer: true,
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
        let sovereign_account = Signer::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let sovereign_auth = world.sovereign.authorize(sovereign_account);
        world.config.update(sovereign_auth, self.new_config);
    }
}
