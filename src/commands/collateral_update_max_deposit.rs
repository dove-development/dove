use {
    crate::{
        accounts::{Readonly, Signer, Writable},
        finance::Decimal,
        keys::{CollateralMintKey, SovereignKey},
        store::{Collateral, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};
#[cfg(feature = "wasm")]
use {
    crate::{
        keys::ProgramKey,
        util::{b2pk, AccountWasm},
    },
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};

/// Updates the maximum deposit limit for a collateral account
///
/// Accounts expected:
///
/// 0. `[signer]` Sovereign account
/// 1. `[writable]` Collateral account (PDA)
/// 2. `[]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct CollateralUpdateMaxDeposit {
    new_max_deposit: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl CollateralUpdateMaxDeposit {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(newMaxDeposit: f64) -> Vec<u8> {
        Self {
            new_max_deposit: Decimal::from(newMaxDeposit),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        sovereignKey: &[u8],
        collateralMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let sovereignKey = SovereignKey::new(b2pk(sovereignKey)?);
        let collateralMintKey = CollateralMintKey::new(b2pk(collateralMintKey)?);
        let accounts = Self::get_accounts(programKey, (sovereignKey, collateralMintKey))
            .into_iter()
            .map(|x| AccountWasm::from(x))
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for CollateralUpdateMaxDeposit {}

impl Command for CollateralUpdateMaxDeposit {
    const ID: u32 = 0x9fecbb56;
    type Keys = (SovereignKey, CollateralMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (sovereign_key, collateral_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *sovereign_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_collateral(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let sovereign_account = Signer::new(&accounts[0]);
        let collateral_account = Writable::new(&accounts[1]);
        let world_account = Readonly::new(&accounts[2]);

        let world_data = world_account.get_info().data.borrow();
        let world = World::load(program_id, world_account, &world_data, ());
        let sovereign_auth = world.sovereign.authorize(sovereign_account);

        let mut collateral_data = collateral_account.get_info().data.borrow_mut();
        let (collateral, collateral_auth) = Collateral::load_auth(
            program_id,
            collateral_account,
            &mut collateral_data[..],
            sovereign_auth,
        );

        collateral.update_max_deposit(collateral_auth, self.new_max_deposit);
    }
}
