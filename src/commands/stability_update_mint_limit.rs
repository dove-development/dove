use {
    crate::{
        accounts::{Readonly, Signer, Writable},
        finance::Decimal,
        keys::{StableMintKey, SovereignKey},
        store::{Stability, World},
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

/// Updates the mint limit for a stability pool
///
/// Accounts expected:
///
/// 0. `[signer]` Sovereign account
/// 1. `[writable]` Stability account (PDA)
/// 2. `[]` World account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct StabilityUpdateMintLimit {
    new_mint_limit: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl StabilityUpdateMintLimit {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(newMintLimit: f64) -> Vec<u8> {
        Self {
            new_mint_limit: Decimal::from(newMintLimit),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        sovereignKey: &[u8],
        stableMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let sovereignKey = SovereignKey::new(b2pk(sovereignKey)?);
        let stableMintKey = StableMintKey::new(b2pk(stableMintKey)?);
        let accounts = Self::get_accounts(programKey, (sovereignKey, stableMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for StabilityUpdateMintLimit {}

impl Command for StabilityUpdateMintLimit {
    const ID: u32 = 0xd6c2c553;
    type Keys = (SovereignKey, StableMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (sovereign_key, stable_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *sovereign_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_stability(&stable_mint_key),
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
        let stability_account = Writable::new(&accounts[1]);
        let world_account = Readonly::new(&accounts[2]);

        let world_data = world_account.get_info().data.borrow();
        let world = World::load(program_id, world_account, &world_data, ());
        let sovereign_auth = world.sovereign.authorize(sovereign_account);

        let mut stability_data = stability_account.get_info().data.borrow_mut();
        let (stability, stability_auth) = Stability::load_auth(
            program_id,
            stability_account,
            &mut stability_data[..],
            sovereign_auth,
        );

        stability.update_mint_limit(stability_auth, self.new_mint_limit);
    }
}
