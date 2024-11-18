#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{Readonly, Writable},
        keys::OracleKey,
        store::World,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Starts a world offering if either the deficit or surplus exceeds their limit.
///
/// Accounts expected:
///
/// 0. `[writable]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OfferingStart {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OfferingStart {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        oracleKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let oracleKey = OracleKey::new(b2pk(oracleKey)?);
        let accounts = Self::get_accounts(programKey, oracleKey)
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for OfferingStart {}

impl Command for OfferingStart {
    const ID: u32 = 0x67f937e4;
    type Keys = OracleKey;

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, oracle_key: Self::Keys) -> Vec<AccountMeta> {
        vec![
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *oracle_key,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let world_account = Writable::new(&accounts[0]);
        let oracle_account = Readonly::new(&accounts[1]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        world.offering.start(
            &clock,
            oracle_account,
            &mut world.debt,
            &mut world.savings,
            &mut world.dvd,
            &mut world.dvd_price,
            &world.config.get_dvd_interest_rate(),
            &mut world.stable_dvd,
            world.config.get_dove_oracle(),
            world.config.get_offering_config(),
            world.config.get_debt_config(),
            world.config.get_savings_config(),
        );
    }
}
